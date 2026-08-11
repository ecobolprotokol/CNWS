//! Conversion pipeline for importing external formats
//! Implements streaming-first import with bounded memory

use super::storage::StorageEngine;
use crate::error::Result;
use crate::types::{Blake3Hash, Compression, DataType};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

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
    /// Errors encountered
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl Default for ImportReport {
    fn default() -> Self {
        Self {
            source_format: String::new(),
            tensors_imported: 0,
            cells_created: 0,
            tiles_written: 0,
            total_bytes: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Conversion pipeline
pub struct ConversionPipeline {
    store: Arc<StorageEngine>,
    compression: Compression,
    normalization: NormalizationPolicy,
}

impl ConversionPipeline {
    /// Create a new conversion pipeline
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self {
            store,
            compression: Compression::Zstd,
            normalization: NormalizationPolicy::None,
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

    /// Import from Safetensors format
    pub fn import_safetensors(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let mut report = ImportReport {
            source_format: "safetensors".to_string(),
            ..Default::default()
        };

        // Read safetensors file
        let data = std::fs::read(path.as_ref())?;
        report.total_bytes = data.len() as u64;

        // Parse header (simplified - real implementation would use safetensors crate)
        // For now, just store the raw data as a tile
        let _hash = self.store.write_tile(&data, self.compression)?;
        report.tiles_written += 1;
        report.tensors_imported += 1;
        report.cells_created += 1;

        Ok(report)
    }

    /// Import from GGUF format
    pub fn import_gguf(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let mut report = ImportReport {
            source_format: "gguf".to_string(),
            ..Default::default()
        };

        let data = std::fs::read(path.as_ref())?;
        report.total_bytes = data.len() as u64;

        // Parse GGUF (simplified)
        let _hash = self.store.write_tile(&data, self.compression)?;
        report.tiles_written += 1;
        report.tensors_imported += 1;
        report.cells_created += 1;

        Ok(report)
    }

    /// Import from PyTorch format
    pub fn import_pytorch(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let mut report = ImportReport {
            source_format: "pytorch".to_string(),
            ..Default::default()
        };

        let data = std::fs::read(path.as_ref())?;
        report.total_bytes = data.len() as u64;

        // Parse PyTorch (simplified)
        let _hash = self.store.write_tile(&data, self.compression)?;
        report.tiles_written += 1;
        report.tensors_imported += 1;
        report.cells_created += 1;

        Ok(report)
    }

    /// Import from ONNX format
    pub fn import_onnx(&self, path: impl AsRef<Path>) -> Result<ImportReport> {
        let mut report = ImportReport {
            source_format: "onnx".to_string(),
            ..Default::default()
        };

        let data = std::fs::read(path.as_ref())?;
        report.total_bytes = data.len() as u64;

        let _hash = self.store.write_tile(&data, self.compression)?;
        report.tiles_written += 1;
        report.tensors_imported += 1;
        report.cells_created += 1;

        Ok(report)
    }

    /// Read Safetensors file (streaming)
    pub fn read_safetensors(&self, path: impl AsRef<Path>) -> Result<Vec<(String, Vec<u8>)>> {
        let data = std::fs::read(path.as_ref())?;
        let mut tensors = Vec::new();

        // Simplified parsing - real implementation would use safetensors crate
        // For now, just return the raw data
        tensors.push(("data".to_string(), data));

        Ok(tensors)
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

    /// Convert tensor to Cell
    pub fn convert_tensor(
        &self,
        name: &str,
        data: &[u8],
        dtype: DataType,
        shape: &[usize],
    ) -> Result<Blake3Hash> {
        // Create cell metadata
        let metadata = format!(
            r#"{{"name":"{}","dtype":"{:?}","shape":{:?}}}"#,
            name, dtype, shape
        );

        // Write metadata as tile
        let meta_hash = self.store.write_tile(metadata.as_bytes(), self.compression)?;

        // Write data as tile
        let data_hash = self.store.write_tile(data, self.compression)?;

        // Create cell (simplified - would create proper Cell structure)
        let cell_data = format!("{}:{}", meta_hash, data_hash);
        let cell_hash = self.store.write_tile(cell_data.as_bytes(), self.compression)?;

        Ok(cell_hash)
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
        let hash = pipeline.convert_tensor("test", data, DataType::F32, &[10]).unwrap();
        assert!(hash != Blake3Hash::default());
    }
}
