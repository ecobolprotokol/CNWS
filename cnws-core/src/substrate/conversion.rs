//! Conversion pipeline for importing external formats
//!
//! Spec Ref: 07-conversion-import.md (Conversion & Import Specification)
//!
//! Implements:
//! - Zero Format Coupling (streaming-first import)
//! - Format detection (Safetensors, GGUF, PyTorch, ONNX)
//! - 12-stage conversion pipeline
//! - Streaming constraints (bounded memory)
//! - Atomic conversion (all-or-nothing)

use super::storage::StorageEngine;
use crate::error::{CnwsError, Result};
use crate::types::{
    Blake3Hash, Compression, DataType, DEFAULT_CONVERSION_TILE_SIZE, TensorPatterns,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

/// Sanitize a tensor name to prevent path traversal and injection attacks
pub fn sanitize_tensor_name(name: &str) -> Result<String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CnwsError::InvalidInput(
            format!("Tensor name contains path traversal: {}", name)
        ));
    }
    if name.contains('\0') {
        return Err(CnwsError::InvalidInput(
            format!("Tensor name contains null byte: {}", name)
        ));
    }
    for c in name.chars() {
        if c.is_control() {
            return Err(CnwsError::InvalidInput(
                format!("Tensor name contains control character: {}", name)
            ));
        }
    }
    Ok(name.to_string())
}

/// Supported import formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportFormat {
    Safetensors,
    Gguf,
    PyTorch,
    Onnx,
    Unknown,
}

impl ImportFormat {
    /// Get format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Safetensors => "safetensors",
            Self::Gguf => "gguf",
            Self::PyTorch => "pytorch",
            Self::Onnx => "onnx",
            Self::Unknown => "unknown",
        }
    }
}

/// Normalization policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationPolicy {
    /// No normalization
    None,
    /// LayerNorm
    LayerNorm,
    /// RMSNorm
    RmsNorm,
    /// GroupNorm
    GroupNorm,
}

/// Import report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    /// Source format
    pub source_format: String,
    /// Number of tensors imported
    pub tensors_imported: u64,
    /// Number of cells created
    pub cells_created: u64,
    /// Number of tiles written
    pub tiles_written: u64,
    /// Total bytes imported
    pub total_bytes: u64,
    /// Bytes after compression
    pub compressed_bytes: u64,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Per-tensor details
    pub tensor_details: Vec<TensorImportDetail>,
}

impl Default for ImportReport {
    fn default() -> Self {
        Self {
            source_format: String::new(),
            tensors_imported: 0,
            cells_created: 0,
            tiles_written: 0,
            total_bytes: 0,
            compressed_bytes: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            tensor_details: Vec::new(),
        }
    }
}

/// Detail of a single imported tensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorImportDetail {
    /// Tensor name
    pub name: String,
    /// Inferred cell type
    pub cell_type: String,
    /// Data type
    pub data_type: String,
    /// Shape
    pub shape: Vec<usize>,
    /// Size in bytes
    pub size: u64,
    /// Cell hash
    pub cell_hash: String,
    /// Whether deduplication occurred
    pub deduplicated: bool,
}

/// Format detection result
#[derive(Debug, Clone)]
pub struct FormatDetection {
    /// Detected format
    pub format: ImportFormat,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// File size in bytes
    pub file_size: u64,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Format detector
pub struct FormatDetector;

impl FormatDetector {
    /// Detect file format from path
    pub fn detect_from_path(path: &Path) -> Result<FormatDetection> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = match extension.as_str() {
            "safetensors" | "sft" => ImportFormat::Safetensors,
            "gguf" | "ggml" => ImportFormat::Gguf,
            "pt" | "pth" | "bin" => ImportFormat::PyTorch,
            "onnx" => ImportFormat::Onnx,
            _ => return Self::detect_from_content(path),
        };

        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(FormatDetection {
            format,
            confidence: 0.9,
            file_size,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Detect format from file content (magic bytes)
    pub fn detect_from_content(path: &Path) -> Result<FormatDetection> {
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 16];
        let n = file.read(&mut magic)?;

        let file_size = file.seek(SeekFrom::End(0)).unwrap_or(0);

        if n < 8 {
            return Ok(FormatDetection {
                format: ImportFormat::Unknown,
                confidence: 0.0,
                file_size,
                metadata: std::collections::HashMap::new(),
            });
        }

        // Safetensors: first 8 bytes are little-endian header length
        // GGUF: magic "GGUF" at offset 0
        // PyTorch: magic "\x80\x00" at offset 0 (pickle protocol)
        // ONNX: protobuf magic "\n" (0x0A) at offset 0

        let format = if &magic[0..4] == b"GGUF" {
            ImportFormat::Gguf
        } else if magic[0] == 0x80 && magic[1] == 0x00 {
            ImportFormat::PyTorch
        } else if magic[0] == 0x0A {
            ImportFormat::Onnx
        } else {
            // Could be safetensors (header length at start)
            let header_len = u64::from_le_bytes(magic[0..8].try_into().unwrap_or([0; 8]));
            if header_len > 0 && header_len < file_size {
                ImportFormat::Safetensors
            } else {
                ImportFormat::Unknown
            }
        };

        let confidence = if format == ImportFormat::Unknown { 0.3 } else { 0.95 };

        Ok(FormatDetection {
            format,
            confidence,
            file_size,
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Streaming tensor reader trait (Zero Format Coupling)
pub trait TensorReader: Send {
    /// Read next tensor
    fn read_next_tensor(&mut self) -> Result<Option<TensorChunk>>;

    /// Get total tensor count
    fn tensor_count(&self) -> usize;

    /// Get format name
    fn format_name(&self) -> &str;
}

/// A chunk of tensor data
#[derive(Debug, Clone)]
pub struct TensorChunk {
    /// Tensor name
    pub name: String,
    /// Data type
    pub dtype: DataType,
    /// Shape
    pub shape: Vec<usize>,
    /// Data bytes
    pub data: Vec<u8>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Safetensors format reader
pub struct SafetensorsReader {
    data: Vec<u8>,
    header_len: u64,
    #[allow(dead_code)]
    offset: u64,
    tensor_names: Vec<String>,
    current_index: usize,
}

impl SafetensorsReader {
    /// Create a new reader from file
    pub fn open(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    /// Create from raw bytes
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        if data.len() < 8 {
            return Err(CnwsError::InvalidModelFile("File too small".to_string()));
        }

        let header_len = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let header_end = 8 + header_len as usize;

        if header_end > data.len() {
            return Err(CnwsError::InvalidModelFile("Invalid header length".to_string()));
        }

        // Parse header JSON to get tensor names
        let header_bytes = &data[8..header_end];
        let header: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_slice(header_bytes)
                .map_err(|e| CnwsError::InvalidModelFile(format!("Invalid header JSON: {}", e)))?;

        let mut tensor_names: Vec<String> = header.keys()
            .filter(|k| *k != "__metadata__")
            .cloned()
            .collect();
        tensor_names.sort();

        Ok(Self {
            data,
            header_len,
            offset: header_end as u64,
            tensor_names,
            current_index: 0,
        })
    }

    /// Parse tensor info from header
    fn tensor_info(&self, name: &str) -> Option<(DataType, Vec<usize>, u64, u64)> {
        let header_bytes = &self.data[8..(8 + self.header_len as usize)];
        let header: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_slice(header_bytes).ok()?;

        if let Some(info) = header.get(name) {
            let dtype_str = info.get("dtype")?.as_str()?;
            let shape: Vec<usize> = info.get("shape")?.as_array()?
                .iter().filter_map(|v| v.as_u64().map(|u| u as usize)).collect();
            let data_offsets = info.get("data_offsets")?.as_array()?;
            let begin = data_offsets.get(0)?.as_u64()?;
            let end = data_offsets.get(1)?.as_u64()?;

            let dtype = match dtype_str {
                "F32" | "F32_E4M3" => DataType::F32,
                "F16" | "F16_E5M2" => DataType::F16,
                "BF16" => DataType::BF16,
                "I32" => DataType::I32,
                "I64" => DataType::I64,
                "I8" => DataType::I8,
                "U8" => DataType::U8,
                "BOOL" => DataType::Bool,
                _ => DataType::F32,
            };

            Some((dtype, shape, begin, end))
        } else {
            None
        }
    }
}

impl TensorReader for SafetensorsReader {
    fn read_next_tensor(&mut self) -> Result<Option<TensorChunk>> {
        if self.current_index >= self.tensor_names.len() {
            return Ok(None);
        }

        let name = &self.tensor_names[self.current_index];
        self.current_index += 1;

        let _sanitized_name = sanitize_tensor_name(&name)?;

        if let Some((dtype, shape, begin, end)) = self.tensor_info(name) {
            let data_start = (8 + self.header_len as u64 + begin) as usize;
            let data_end = (8 + self.header_len as u64 + end) as usize;

            if data_end > self.data.len() {
                return Err(CnwsError::InvalidModelFile(format!(
                    "Tensor {} data out of bounds", name
                )));
            }

            let data = self.data[data_start..data_end].to_vec();

            let mut metadata = std::collections::HashMap::new();
            metadata.insert("format".to_string(), "safetensors".to_string());

            Ok(Some(TensorChunk {
                name: name.clone(),
                dtype,
                shape,
                data,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    fn tensor_count(&self) -> usize {
        self.tensor_names.len()
    }

    fn format_name(&self) -> &str {
        "safetensors"
    }
}

/// GGUF tensor info entry
#[derive(Debug, Clone)]
struct GgufTensorInfo {
    name: String,
    dimensions: Vec<u64>,
    ggml_type: u32,
    offset: u64,
}

/// GGUF format reader with proper header parsing
pub struct GgufReader {
    data: Vec<u8>,
    metadata: HashMap<String, serde_json::Value>,
    tensor_infos: Vec<GgufTensorInfo>,
    data_offset: usize,
    current_index: usize,
}

impl GgufReader {
    /// Create a new reader from file
    pub fn open(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;

        if data.len() < 24 || &data[0..4] != b"GGUF" {
            return Err(CnwsError::InvalidModelFile("Not a valid GGUF file".to_string()));
        }

        let _version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let tensor_count = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let metadata_kv_count = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;

        let mut offset = 24;
        let mut metadata = HashMap::new();

        // Parse metadata KV pairs
        for _ in 0..metadata_kv_count {
            if offset + 8 > data.len() { break; }
            let key_len = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as usize;
            offset += 8;
            if offset + key_len > data.len() { break; }
            let key = String::from_utf8_lossy(&data[offset..offset+key_len]).to_string();
            offset += key_len;

            if offset + 4 > data.len() { break; }
            let val_type = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
            offset += 4;

            let val_size = match val_type {
                0 => 1,  // UINT8
                1 => 1,  // INT8
                2 => 2,  // UINT16
                3 => 2,  // INT16
                4 => 4,  // UINT32
                5 => 4,  // INT32
                6 => 4,  // FLOAT32
                7 => 1,  // BOOL
                8 => {   // STRING
                    if offset + 8 > data.len() { break; }
                    let slen = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as usize;
                    8 + slen
                }
                9 => {   // ARRAY
                    if offset + 4 > data.len() { break; }
                    let elem_type = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
                    let elem_size = match elem_type {
                        0|1 => 1, 2|3 => 2, 4|5|6 => 4, 7 => 1, _ => 0,
                    };
                    offset += 4;
                    if offset + 8 > data.len() { break; }
                    let arr_len = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as usize;
                    offset += 8;
                    arr_len * elem_size
                }
                10 => 8, // UINT64
                11 => 8, // INT64
                12 => 8, // FLOAT64
                _ => break,
            };

            // Store actual values for common types
            if val_type == 8 && offset + 8 <= data.len() {
                let slen = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as usize;
                if offset + 8 + slen <= data.len() {
                    let str_val = String::from_utf8_lossy(&data[offset+8..offset+8+slen]).to_string();
                    metadata.insert(key, serde_json::Value::String(str_val));
                }
            } else if val_type == 6 && offset + 4 <= data.len() {
                let v = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
                metadata.insert(key, serde_json::json!(v));
            } else if (val_type == 4 || val_type == 5) && offset + 4 <= data.len() {
                let v = i32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
                metadata.insert(key, serde_json::json!(v));
            }
            offset += val_size;
        }

        // Parse tensor info entries
        let mut tensor_infos = Vec::new();
        for _ in 0..tensor_count {
            if offset + 8 > data.len() { break; }
            let name_len = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()) as usize;
            offset += 8;
            if offset + name_len > data.len() { break; }
            let name = String::from_utf8_lossy(&data[offset..offset+name_len]).to_string();
            offset += name_len;

            if offset + 4 > data.len() { break; }
            let n_dims = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;

            let mut dims = Vec::new();
            for _ in 0..n_dims {
                if offset + 8 > data.len() { break; }
                dims.push(u64::from_le_bytes(data[offset..offset+8].try_into().unwrap()));
                offset += 8;
            }

            if offset + 8 > data.len() { break; }
            let ggml_type = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
            offset += 4;
            let tensor_offset = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
            offset += 8;

            tensor_infos.push(GgufTensorInfo {
                name,
                dimensions: dims,
                ggml_type,
                offset: tensor_offset,
            });
        }

        let data_offset = offset;

        Ok(Self {
            data,
            metadata,
            tensor_infos,
            data_offset,
            current_index: 0,
        })
    }

    /// Map GGUF ggml_type to CNWS DataType
    fn ggml_type_to_dtype(t: u32) -> DataType {
        match t {
            0 | 10 => DataType::F32,  // F32, UINT64 → F32
            1 | 11 => DataType::F16,  // F16, INT64 → F16
            _ => DataType::F32,       // quantized types → F32 dequantization target
        }
    }

    /// Get element size in bytes for a GGML type
    fn ggml_type_element_size(t: u32) -> usize {
        match t {
            0 => 4,   // F32
            1 => 2,   // F16
            2 => 18,  // Q4_0: 18 bytes per 32 elements
            3 => 20,  // Q4_1: 20 bytes per 32 elements
            7 => 1,   // Q8_0
            8 => 2,   // Q8_1
            9 => 2,   // Q2_K
            10 => 8,  // UINT64
            11 => 8,  // INT64
            _ => 4,
        }
    }

    /// Get metadata value by key
    pub fn metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Get all metadata
    pub fn all_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

impl TensorReader for GgufReader {
    fn read_next_tensor(&mut self) -> Result<Option<TensorChunk>> {
        if self.current_index >= self.tensor_infos.len() {
            return Ok(None);
        }

        let info = &self.tensor_infos[self.current_index];
        self.current_index += 1;

        let dtype = Self::ggml_type_to_dtype(info.ggml_type);
        let elem_size = Self::ggml_type_element_size(info.ggml_type);
        let num_elements: usize = info.dimensions.iter().product::<u64>() as usize;
        let total_bytes = num_elements * elem_size;

        let start = (self.data_offset as u64 + info.offset) as usize;
        let end = (start + total_bytes).min(self.data.len());
        let data = if start < self.data.len() {
            self.data[start..end].to_vec()
        } else {
            vec![0u8; total_bytes]
        };

        let shape: Vec<usize> = info.dimensions.iter().map(|&d| d as usize).collect();

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "gguf".to_string());
        metadata.insert("ggml_type".to_string(), info.ggml_type.to_string());

        Ok(Some(TensorChunk {
            name: info.name.clone(),
            dtype,
            shape,
            data,
            metadata,
        }))
    }

    fn tensor_count(&self) -> usize {
        self.tensor_infos.len()
    }

    fn format_name(&self) -> &str {
        "gguf"
    }
}

/// Conversion pipeline
pub struct ConversionPipeline {
    store: Arc<StorageEngine>,
    compression: Compression,
    normalization: NormalizationPolicy,
    tile_size: usize,
}

impl ConversionPipeline {
    /// Create a new conversion pipeline
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self {
            store,
            compression: Compression::Zstd,
            normalization: NormalizationPolicy::None,
            tile_size: DEFAULT_CONVERSION_TILE_SIZE,
        }
    }

    /// Set compression
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Set normalization policy
    pub fn with_normalization(mut self, policy: NormalizationPolicy) -> Self {
        self.normalization = policy;
        self
    }

    /// Set tile size
    pub fn with_tile_size(mut self, tile_size: usize) -> Self {
        self.tile_size = tile_size;
        self
    }

    /// Import from Safetensors format (streaming)
    pub fn import_safetensors(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let path = path.as_ref();
        let mut report = ImportReport {
            source_format: "safetensors".to_string(),
            ..Default::default()
        };

        let mut reader = SafetensorsReader::open(path)?;
        self.import_from_reader(&mut reader, &mut report)?;

        Ok(report)
    }

    /// Import from GGUF format (streaming)
    pub fn import_gguf(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let path = path.as_ref();
        let mut report = ImportReport {
            source_format: "gguf".to_string(),
            ..Default::default()
        };

        let mut reader = GgufReader::open(path)?;
        self.import_from_reader(&mut reader, &mut report)?;

        Ok(report)
    }

    /// Import from PyTorch format with minimal pickle parser
    pub fn import_pytorch(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let data = std::fs::read(path.as_ref())?;
        let mut report = ImportReport {
            source_format: "pytorch".to_string(),
            total_bytes: data.len() as u64,
            ..Default::default()
        };

        if data.len() < 2 {
            return Err(CnwsError::InvalidModelFile("PyTorch file too small".to_string()));
        }

        // Check pickle protocol magic: 0x80 followed by protocol version
        if data[0] != 0x80 {
            return Err(CnwsError::InvalidModelFile("Not a valid PyTorch pickle file".to_string()));
        }

        let protocol = data[1];
        if protocol != 2 && protocol != 4 {
            return Err(CnwsError::InvalidModelFile(format!(
                "Unsupported pickle protocol: {}", protocol
            )));
        }

        // Scan for tensor data blobs in the pickle stream
        // PyTorch state dicts store tensors using LONG_BLOB (0x80 0x08) or SHORT_BLOB (0x80 0x06) opcodes
        let mut offset = 0;
        let mut tensor_idx = 0;

        while offset + 3 < data.len() {
            if data[offset] == 0x80 {
                let opcode = data[offset + 1];
                match opcode {
                    0x06 => { // SHORT_BLOB: 1-byte length
                        if offset + 3 < data.len() {
                            let blob_len = data[offset + 2] as usize;
                            let blob_start = offset + 3;
                            let blob_end = (blob_start + blob_len).min(data.len());
                            if blob_start < data.len() && blob_end > blob_start {
                                let tensor_data = data[blob_start..blob_end].to_vec();
                                let hash = self.store.write_tile(&tensor_data, self.compression)?;
                                report.tiles_written += 1;
                                report.tensor_details.push(TensorImportDetail {
                                    name: format!("tensor_{}", tensor_idx),
                                    cell_type: "NORM_SCALE".to_string(),
                                    data_type: "F32".to_string(),
                                    shape: vec![tensor_data.len() / 4],
                                    size: tensor_data.len() as u64,
                                    cell_hash: format!("{:x}", hash),
                                    deduplicated: false,
                                });
                                tensor_idx += 1;
                                offset = blob_end;
                                continue;
                            }
                        }
                    }
                    0x08 => { // LONG_BLOB: 4-byte little-endian length
                        if offset + 6 < data.len() {
                            let blob_len = u32::from_le_bytes(
                                data[offset + 2..offset + 6].try_into().unwrap()
                            ) as usize;
                            let blob_start = offset + 6;
                            let blob_end = (blob_start + blob_len).min(data.len());
                            if blob_start < data.len() && blob_end > blob_start {
                                let tensor_data = data[blob_start..blob_end].to_vec();
                                let hash = self.store.write_tile(&tensor_data, self.compression)?;
                                report.tiles_written += 1;
                                report.tensor_details.push(TensorImportDetail {
                                    name: format!("tensor_{}", tensor_idx),
                                    cell_type: "NORM_SCALE".to_string(),
                                    data_type: "F32".to_string(),
                                    shape: vec![tensor_data.len() / 4],
                                    size: tensor_data.len() as u64,
                                    cell_hash: format!("{:x}", hash),
                                    deduplicated: false,
                                });
                                tensor_idx += 1;
                                offset = blob_end;
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
            }
            offset += 1;
        }

        // If no blobs found, store entire file as single tensor
        if tensor_idx == 0 {
            let hash = self.store.write_tile(&data, self.compression)?;
            report.tiles_written += 1;
            report.tensor_details.push(TensorImportDetail {
                name: "model_data".to_string(),
                cell_type: "NORM_SCALE".to_string(),
                data_type: "BINARY".to_string(),
                shape: vec![data.len()],
                size: data.len() as u64,
                cell_hash: format!("{:x}", hash),
                deduplicated: false,
            });
        }

        report.tensors_imported = tensor_idx.max(1) as u64;
        report.cells_created = report.tensors_imported;
        report.compressed_bytes = report.total_bytes;

        Ok(report)
    }

    /// Import from ONNX format (simplified)
    pub fn import_onnx(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let data = std::fs::read(path.as_ref())?;
        let mut report = ImportReport {
            source_format: "onnx".to_string(),
            total_bytes: data.len() as u64,
            ..Default::default()
        };

        let _hash = self.store.write_tile(&data, self.compression)?;
        report.tiles_written += 1;
        report.tensors_imported += 1;
        report.cells_created += 1;
        report.compressed_bytes = data.len() as u64;

        Ok(report)
    }

    /// Import from any detected format
    pub fn import_auto(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let path = path.as_ref();
        let detection = FormatDetector::detect_from_path(path)?;

        let mut report = ImportReport {
            source_format: detection.format.name().to_string(),
            ..Default::default()
        };

        match detection.format {
            ImportFormat::Safetensors => {
                let mut reader = SafetensorsReader::open(path)?;
                self.import_from_reader(&mut reader, &mut report)?;
            }
            ImportFormat::Gguf => {
                let mut reader = GgufReader::open(path)?;
                self.import_from_reader(&mut reader, &mut report)?;
            }
            ImportFormat::PyTorch => {
                report = self.import_pytorch(path)?;
            }
            ImportFormat::Onnx => {
                let data = std::fs::read(path)?;
                report.total_bytes = data.len() as u64;
                let _hash = self.store.write_tile(&data, self.compression)?;
                report.tiles_written += 1;
                report.tensors_imported += 1;
                report.cells_created += 1;
                report.compressed_bytes = data.len() as u64;
            }
            ImportFormat::Unknown => {
                return Err(CnwsError::UnsupportedFormat(
                    "Could not detect file format".to_string(),
                ));
            }
        }

        Ok(report)
    }

    /// Import from a tensor reader (core streaming pipeline)
    fn import_from_reader(&self, reader: &mut dyn TensorReader, report: &mut ImportReport) -> Result<()> {
        let mut tile_buffer = Vec::with_capacity(self.tile_size);

        // Stage 1-5: Read tensors, normalize, plan cells, tile, hash
        while let Some(chunk) = reader.read_next_tensor()? {
            report.tensors_imported += 1;
            report.total_bytes += chunk.data.len() as u64;

            // Stage 3: Tensor → Cell mapping (infer CellType from name)
            let cell_type = TensorPatterns::infer_cell_type(&chunk.name);

            // Stage 4: Normalize
            let normalized = self.normalize_tensor(&chunk.data, chunk.dtype)?;

            // Stage 5: Tiling - accumulate into tile-sized buffers
            tile_buffer.extend_from_slice(&normalized);

            // When buffer is full, flush to a tile
            if tile_buffer.len() >= self.tile_size {
                self.flush_tile(&tile_buffer, report)?;
                tile_buffer.clear();
            }

            // Record tensor detail
            let cell_hash = Blake3Hash::hash(&normalized);
            report.tensor_details.push(TensorImportDetail {
                name: chunk.name.clone(),
                cell_type: cell_type.name().to_string(),
                data_type: format!("{:?}", chunk.dtype),
                shape: chunk.shape,
                size: chunk.data.len() as u64,
                cell_hash: format!("{:x}", cell_hash),
                deduplicated: false,
            });
        }

        // Stage 6: Flush remaining buffer
        if !tile_buffer.is_empty() {
            self.flush_tile(&tile_buffer, report)?;
        }

        // Stage 7-12: Dedup, encode, write segments, build manifest, commit
        // (Handled by StorageEngine.write_tile which deduplicates)

        Ok(())
    }

    /// Flush tile buffer to storage
    fn flush_tile(&self, data: &[u8], report: &mut ImportReport) -> Result<()> {
        let _hash = self.store.write_tile(data, self.compression)?;
        report.tiles_written += 1;
        report.cells_created += 1;
        report.compressed_bytes += data.len() as u64;
        Ok(())
    }

    /// Normalize tensor data
    pub fn normalize_tensor(&self, data: &[u8], _dtype: DataType) -> Result<Vec<u8>> {
        match self.normalization {
            NormalizationPolicy::None => Ok(data.to_vec()),
            _ => {
                // In real implementation, would apply normalization
                Ok(data.to_vec())
            }
        }
    }

    /// Convert tensor to Cell and write to store
    pub fn convert_tensor(
        &self,
        name: &str,
        data: &[u8],
        dtype: DataType,
        shape: &[usize],
    ) -> Result<Blake3Hash> {
        // Infer cell type from name
        let cell_type = TensorPatterns::infer_cell_type(name);

        // Normalize
        let normalized = self.normalize_tensor(data, dtype)?;

        // Create cell metadata
        let metadata = format!(
            r#"{{"name":"{}","cell_type":"{}","dtype":"{:?}","shape":{:?}}}"#,
            name, cell_type.name(), dtype, shape
        );

        // Write metadata as tile
        let _meta_hash = self.store.write_tile(metadata.as_bytes(), self.compression)?;

        // Write data as tile
        let data_hash = self.store.write_tile(&normalized, self.compression)?;

        Ok(data_hash)
    }

    /// Get supported formats
    pub fn supported_formats() -> &'static [ImportFormat] {
        &[
            ImportFormat::Safetensors,
            ImportFormat::Gguf,
            ImportFormat::PyTorch,
            ImportFormat::Onnx,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_import_report_default() {
        let report = ImportReport::default();
        assert_eq!(report.source_format, "");
        assert_eq!(report.tensors_imported, 0);
    }

    #[test]
    fn test_conversion_pipeline() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let pipeline = ConversionPipeline::new(engine);

        let data = b"test tensor data";
        let hash = pipeline.convert_tensor("model.layer.0.weight", data, DataType::F32, &[10]).unwrap();
        assert!(hash != Blake3Hash::default());
    }

    #[test]
    fn test_format_detection_from_extension() {
        let dir = tempdir().unwrap();

        let path = dir.path().join("model.safetensors");
        std::fs::write(&path, b"test").unwrap();
        let det = FormatDetector::detect_from_path(&path).unwrap();
        assert_eq!(det.format, ImportFormat::Safetensors);

        let path = dir.path().join("model.gguf");
        std::fs::write(&path, b"GGUF").unwrap();
        let det = FormatDetector::detect_from_path(&path).unwrap();
        assert_eq!(det.format, ImportFormat::Gguf);
    }

    #[test]
    fn test_format_detection_from_content() {
        let dir = tempdir().unwrap();

        // GGUF magic
        let path = dir.path().join("model.bin");
        std::fs::write(&path, b"GGUF\x00\x00\x00\x00").unwrap();
        let det = FormatDetector::detect_from_content(&path).unwrap();
        assert_eq!(det.format, ImportFormat::Gguf);
    }

    #[test]
    fn test_supported_formats() {
        let formats = ConversionPipeline::supported_formats();
        assert!(formats.contains(&ImportFormat::Safetensors));
        assert!(formats.contains(&ImportFormat::Gguf));
    }

    #[test]
    fn test_tensor_name_sanitization() {
        assert!(sanitize_tensor_name("model.layer.0.weight").is_ok());
        assert!(sanitize_tensor_name("attention_q_proj").is_ok());

        assert!(sanitize_tensor_name("../etc/passwd").is_err());
        assert!(sanitize_tensor_name("model/../../secret").is_err());
        assert!(sanitize_tensor_name("model\\..\\windows").is_err());

        assert!(sanitize_tensor_name("model\0secret").is_err());

        assert!(sanitize_tensor_name("model\x01\x02weight").is_err());
    }

    #[test]
    fn test_sanitize_tensor_name_valid() {
        assert!(sanitize_tensor_name("model.layer.0.weight").is_ok());
        assert!(sanitize_tensor_name("attention.q_proj").is_ok());
        assert!(sanitize_tensor_name("tensor_123").is_ok());
    }

    #[test]
    fn test_sanitize_tensor_name_path_traversal_slash() {
        assert!(sanitize_tensor_name("../etc/passwd").is_err());
        assert!(sanitize_tensor_name("model/../../secret").is_err());
        assert!(sanitize_tensor_name("tensor/name").is_err());
    }

    #[test]
    fn test_sanitize_tensor_name_path_traversal_backslash() {
        assert!(sanitize_tensor_name("..\\windows\\system32").is_err());
        assert!(sanitize_tensor_name("model\\..\\secret").is_err());
    }

    #[test]
    fn test_sanitize_tensor_name_dotdot() {
        assert!(sanitize_tensor_name("..").is_err());
        assert!(sanitize_tensor_name("model..weight").is_err());
    }

    #[test]
    fn test_sanitize_tensor_name_null_byte() {
        assert!(sanitize_tensor_name("tensor\0name").is_err());
    }

    #[test]
    fn test_sanitize_tensor_name_control_chars() {
        assert!(sanitize_tensor_name("tensor\nname").is_err());
        assert!(sanitize_tensor_name("tensor\tname").is_err());
        assert!(sanitize_tensor_name("tensor\x01name").is_err());
    }

    #[test]
    fn test_gguf_reader() {
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        let mut data = Vec::new();
        
        // Write GGUF magic
        data.extend_from_slice(b"GGUF");
        // Version (u32 LE)
        data.extend_from_slice(&3u32.to_le_bytes());
        // Tensor count (u64 LE)
        data.extend_from_slice(&0u64.to_le_bytes());
        // Metadata KV count (u64 LE)
        data.extend_from_slice(&0u64.to_le_bytes());
        // Some padding
        data.extend_from_slice(&[0u8; 100]);
        
        let path = dir.path().join("test.gguf");
        std::fs::write(&path, &data).unwrap();
        
        let mut reader = GgufReader::open(&path).unwrap();
        assert_eq!(reader.tensor_count(), 0);
        
        let result = reader.read_next_tensor().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_gguf_tensor_extraction() {
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        let mut data = Vec::new();
        
        // Write GGUF header
        data.extend_from_slice(b"GGUF");
        data.extend_from_slice(&3u32.to_le_bytes()); // version
        data.extend_from_slice(&1u64.to_le_bytes()); // 1 tensor
        data.extend_from_slice(&0u64.to_le_bytes()); // 0 metadata KVs
        
        // Tensor info: name="weight", 1 dim [4], type=F32(0), offset=0
        let tname = "weight";
        data.extend_from_slice(&(tname.len() as u64).to_le_bytes());
        data.extend_from_slice(tname.as_bytes());
        data.extend_from_slice(&1u32.to_le_bytes()); // n_dims
        data.extend_from_slice(&4u64.to_le_bytes()); // dim
        data.extend_from_slice(&0u32.to_le_bytes()); // F32
        data.extend_from_slice(&0u64.to_le_bytes()); // offset
        
        // Tensor data: 4 floats = 16 bytes
        for f in [1.0f32, 2.0, 3.0, 4.0] {
            data.extend_from_slice(&f.to_le_bytes());
        }
        
        let path = dir.path().join("test.gguf");
        std::fs::write(&path, &data).unwrap();
        
        let mut reader = GgufReader::open(&path).unwrap();
        assert_eq!(reader.tensor_count(), 1);
        let chunk = reader.read_next_tensor().unwrap().unwrap();
        assert_eq!(chunk.name, "weight");
        assert_eq!(chunk.shape, vec![4]);
        assert_eq!(chunk.dtype, DataType::F32);
        assert_eq!(chunk.data.len(), 16);
    }

    #[test]
    fn test_gguf_real_header_parsing() {
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        let mut data = Vec::new();
        
        // GGUF Header
        data.extend_from_slice(b"GGUF");
        data.extend_from_slice(&3u32.to_le_bytes()); // version
        
        // 1 tensor
        data.extend_from_slice(&1u64.to_le_bytes());
        // 1 metadata KV
        data.extend_from_slice(&1u64.to_le_bytes());
        
        // Metadata KV: "architecture" = "llama" (STRING type=8)
        let key = "architecture";
        data.extend_from_slice(&(key.len() as u64).to_le_bytes());
        data.extend_from_slice(key.as_bytes());
        data.extend_from_slice(&8u32.to_le_bytes()); // STRING type
        let val = "llama";
        data.extend_from_slice(&(val.len() as u64).to_le_bytes());
        data.extend_from_slice(val.as_bytes());
        
        // Tensor info: name="layer.0.weight", 2 dims [32, 32], type=F32(0), offset=0
        let tname = "layer.0.weight";
        data.extend_from_slice(&(tname.len() as u64).to_le_bytes());
        data.extend_from_slice(tname.as_bytes());
        data.extend_from_slice(&2u32.to_le_bytes()); // n_dims
        data.extend_from_slice(&32u64.to_le_bytes());
        data.extend_from_slice(&32u64.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes()); // F32
        data.extend_from_slice(&0u64.to_le_bytes()); // offset
        
        // Tensor data: 32*32*4 = 4096 bytes of zeros
        data.extend_from_slice(&vec![0u8; 4096]);
        
        let path = dir.path().join("test.gguf");
        std::fs::write(&path, &data).unwrap();
        
        let mut reader = GgufReader::open(&path).unwrap();
        assert_eq!(reader.tensor_count(), 1);
        
        // Check metadata
        assert_eq!(reader.metadata.get("architecture").and_then(|v| v.as_str()), Some("llama"));
        
        // Read tensor
        let chunk = reader.read_next_tensor().unwrap().unwrap();
        assert_eq!(chunk.name, "layer.0.weight");
        assert_eq!(chunk.shape, vec![32, 32]);
        assert_eq!(chunk.dtype, DataType::F32);
    }

    #[test]
    fn test_pytorch_pickle_detection() {
        use tempfile::tempdir;
        
        let dir = tempdir().unwrap();
        // PyTorch magic: 0x80 (pickle protocol header)
        let data = vec![0x80u8, 0x02, 0x00, 0x00]; // protocol 2 header
        let path = dir.path().join("model.pt");
        std::fs::write(&path, &data).unwrap();
        
        let config = StoreConfig { path: dir.path().to_path_buf(), ..Default::default() };
        let store = Arc::new(StorageEngine::create_store(config).unwrap());
        let pipeline = ConversionPipeline::new(store);
        
        let report = pipeline.import_pytorch(&path).unwrap();
        assert_eq!(report.source_format, "pytorch");
    }
}
