//! Manifest Authority - JSON source of truth for .cd store
//!
//! Spec Ref: 04-cd-format-serialization.md §5
//!
//! The Manifest is the canonical, authoritative record of:
//! - All cells in the store (hashes, types, locations)
//! - All tiles in the store (hashes, locations, checksums)
//! - Store metadata (created, updated, version)
//! - Indices (ANN, routing, memory)
//! - Provenance, architecture, and dependency graph
//!
//! The Manifest is stored as MANIFEST.cd (JSON) and is the only mutable file.

use crate::types::Blake3Hash;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manifest - JSON source of truth for .cd store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version
    pub schema_version: String,

    /// Store metadata
    pub metadata: ManifestMetadata,

    /// Architecture description
    pub architecture: Architecture,

    /// All cells in the store (by hash)
    pub cells: HashMap<String, CellRecord>,

    /// All tiles in the store (by hash)
    pub tiles: HashMap<String, TileRecord>,

    /// Cell indices for fast lookup
    pub indices: ManifestIndices,

    /// Dependency graph between cells
    pub dependency_graph: DependencyGraph,

    /// Provenance information
    pub provenance: Option<ProvenanceInfo>,

    /// Representation registry
    pub representations: HashMap<String, RepresentationRecord>,

    /// Segment registry
    pub segments: Vec<SegmentRecord>,

    /// Runtime defaults
    pub runtime_defaults: RuntimeDefaults,
}

/// Manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    /// Store version (semver)
    pub version: String,

    /// Format version (compatibility)
    pub format_version: u32,

    /// Creation timestamp (Unix seconds)
    pub created_at: u64,

    /// Last update timestamp
    pub updated_at: u64,

    /// Total cells
    pub total_cells: u64,

    /// Total tiles
    pub total_tiles: u64,

    /// Total store size (bytes)
    pub total_size: u64,

    /// Store owner/creator
    pub owner: String,

    /// Optional description
    pub description: Option<String>,

    /// Store ID (unique identifier)
    pub store_id: String,
}

/// Architecture description
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Architecture {
    /// Model family (e.g., "llama", "mistral")
    pub model_family: Option<String>,
    /// Number of layers
    pub num_layers: Option<u32>,
    /// Hidden dimension
    pub hidden_dim: Option<u32>,
    /// Number of attention heads
    pub num_heads: Option<u32>,
    /// Number of experts (for MoE)
    pub num_experts: Option<u32>,
    /// Vocabulary size
    pub vocab_size: Option<u32>,
    /// Max sequence length
    pub max_seq_len: Option<u32>,
}

/// Record for a cell in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRecord {
    /// Cell hash (BLAKE3-256)
    pub hash: String,

    /// Cell type enum value
    pub cell_type: u32,

    /// Data type enum value
    pub data_type: u32,

    /// Shape as JSON array
    pub shape: Vec<u32>,

    /// Number of elements
    pub num_elements: u64,

    /// Compression type
    pub compression: u32,

    /// Compressed size
    pub compressed_size: u64,

    /// Uncompressed size
    pub uncompressed_size: u64,

    /// Child cell hashes
    pub children: Vec<String>,

    /// Dependency targets
    pub dependencies: Vec<DependencyRecord>,

    /// Tile where this cell's data lives
    pub tile_hash: Option<String>,

    /// Timestamp when added
    pub added_at: u64,

    /// Cell lifecycle state
    pub lifecycle: String,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Dependency record in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRecord {
    /// Target cell hash
    pub target: String,
    /// Dependency type
    pub dep_type: String,
    /// Strength (0.0-1.0)
    pub strength: f32,
}

/// Record for a tile in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRecord {
    /// Tile hash (BLAKE3-256)
    pub hash: String,

    /// Location in store
    pub location: TileLocationRecord,

    /// Cell hashes in this tile
    pub cell_hashes: Vec<String>,

    /// Tile size (bytes)
    pub size: u64,

    /// Deduplication count
    pub dedup_count: u32,

    /// Tile checksum
    pub checksum: String,

    /// Creation timestamp
    pub created_at: u64,

    /// Compression algorithm
    pub compression: String,
}

/// Tile location record in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLocationRecord {
    /// Segment index
    pub segment_idx: u32,

    /// Tile offset within segment
    pub tile_offset: u32,

    /// Byte offset within segment
    pub byte_offset: u64,
}

/// Indices in manifest
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestIndices {
    /// Cell type → cell hashes
    #[serde(default)]
    pub by_type: HashMap<u32, Vec<String>>,

    /// Data type → cell hashes
    #[serde(default)]
    pub by_data_type: HashMap<u32, Vec<String>>,

    /// Layer → cell hashes (for transformer layers)
    #[serde(default)]
    pub by_layer: HashMap<String, Vec<String>>,

    /// Name → cell hashes
    #[serde(default)]
    pub by_name: HashMap<String, Vec<String>>,
}

/// Dependency graph
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyGraph {
    /// Adjacency list: source hash → list of target hashes
    pub edges: HashMap<String, Vec<String>>,
}

/// Provenance information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvenanceInfo {
    /// Source model name
    pub source_model: Option<String>,
    /// Import format
    pub import_format: Option<String>,
    /// Import timestamp
    pub import_timestamp: Option<u64>,
    /// Source file hashes
    pub source_hashes: Vec<String>,
    /// Import tool version
    pub tool_version: Option<String>,
}

/// Representation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentationRecord {
    /// Representation hash
    pub hash: String,
    /// Data type
    pub data_type: u32,
    /// Shape
    pub shape: Vec<u64>,
    /// Compression
    pub compression: String,
    /// Size in bytes
    pub size: u64,
}

/// Segment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentRecord {
    /// Segment index
    pub index: u32,
    /// Segment type
    pub segment_type: u32,
    /// Tile count
    pub tile_count: u32,
    /// Size in bytes
    pub size: u64,
    /// Segment checksum
    pub checksum: String,
}

/// Runtime defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeDefaults {
    /// Default compute budget
    pub max_compute: Option<u64>,
    /// Default max depth
    pub max_depth: Option<u32>,
    /// Default selection top-k
    pub selection_k: Option<u32>,
    /// Default confidence threshold
    pub confidence_threshold: Option<f32>,
}

impl Manifest {
    /// Create a new empty manifest
    pub fn new(owner: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let store_id = format!("cnws-{}", uuid::Uuid::new_v4());

        Self {
            schema_version: "1.0.0".to_string(),
            metadata: ManifestMetadata {
                version: "1.0.0".to_string(),
                format_version: 1,
                created_at: now,
                updated_at: now,
                total_cells: 0,
                total_tiles: 0,
                total_size: 0,
                owner,
                description: None,
                store_id,
            },
            architecture: Architecture::default(),
            cells: HashMap::new(),
            tiles: HashMap::new(),
            indices: ManifestIndices::default(),
            dependency_graph: DependencyGraph::default(),
            provenance: None,
            representations: HashMap::new(),
            segments: Vec::new(),
            runtime_defaults: RuntimeDefaults::default(),
        }
    }

    /// Add a cell record to manifest
    pub fn add_cell_record(&mut self, record: CellRecord) -> Result<()> {
        // Update indices
        self.indices.by_type
            .entry(record.cell_type)
            .or_default()
            .push(record.hash.clone());
        self.indices.by_data_type
            .entry(record.data_type)
            .or_default()
            .push(record.hash.clone());

        // Update dependency graph
        for dep in &record.dependencies {
            self.dependency_graph.edges
                .entry(record.hash.clone())
                .or_default()
                .push(dep.target.clone());
        }

        self.cells.insert(record.hash.clone(), record);
        self.metadata.total_cells += 1;
        self.update_timestamp();
        Ok(())
    }

    /// Remove a cell record from manifest
    pub fn remove_cell_record(&mut self, hash: &str) -> Option<CellRecord> {
        if let Some(record) = self.cells.remove(hash) {
            self.metadata.total_cells = self.metadata.total_cells.saturating_sub(1);
            self.dependency_graph.edges.remove(hash);
            self.update_timestamp();
            Some(record)
        } else {
            None
        }
    }

    /// Add a tile record to manifest
    pub fn add_tile_record(&mut self, record: TileRecord) -> Result<()> {
        self.metadata.total_size += record.size;
        self.tiles.insert(record.hash.clone(), record);
        self.metadata.total_tiles += 1;
        self.update_timestamp();
        Ok(())
    }

    /// Remove a tile record from manifest
    pub fn remove_tile_record(&mut self, hash: &str) -> Option<TileRecord> {
        if let Some(record) = self.tiles.remove(hash) {
            self.metadata.total_size = self.metadata.total_size.saturating_sub(record.size);
            self.metadata.total_tiles = self.metadata.total_tiles.saturating_sub(1);
            self.update_timestamp();
            Some(record)
        } else {
            None
        }
    }

    /// Compute canonical JSON and its BLAKE3-256 hash
    pub fn compute_hash(&self) -> Blake3Hash {
        let canonical = self.to_canonical_json();
        Blake3Hash::hash(canonical.as_bytes())
    }

    /// Convert to canonical JSON (sorted keys, deterministic)
    pub fn to_canonical_json(&self) -> String {
        let value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        canonicalize_json(&value)
    }

    /// Verify the manifest's hash matches its current content
    pub fn verify_hash(&self) -> bool {
        let canonical = self.to_canonical_json();
        let computed = Blake3Hash::hash(canonical.as_bytes());
        computed == self.compute_hash()
    }

    /// Save manifest to JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::CnwsError::Serialization(
                format!("Failed to serialize manifest: {}", e)
            ))?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load manifest from JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| crate::error::CnwsError::Deserialization(
                format!("Failed to parse manifest: {}", e)
            ))
    }

    /// Update timestamp to current time
    fn update_timestamp(&mut self) {
        self.metadata.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get a cell record by hash
    pub fn get_cell(&self, hash: &Blake3Hash) -> Option<&CellRecord> {
        self.cells.get(&hash.to_hex())
    }

    /// Get a tile record by hash
    pub fn get_tile(&self, hash: &Blake3Hash) -> Option<&TileRecord> {
        self.tiles.get(&hash.to_hex())
    }

    /// Find all cells of a given type
    pub fn cells_of_type(&self, cell_type: u32) -> Vec<&CellRecord> {
        self.cells.values()
            .filter(|c| c.cell_type == cell_type)
            .collect()
    }

    /// Find all cells of a given data type
    pub fn cells_of_data_type(&self, data_type: u32) -> Vec<&CellRecord> {
        self.cells.values()
            .filter(|c| c.data_type == data_type)
            .collect()
    }

    /// Get dependencies of a cell
    pub fn get_dependencies(&self, cell_hash: &str) -> Vec<&DependencyRecord> {
        self.cells.get(cell_hash)
            .map(|c| c.dependencies.iter().collect())
            .unwrap_or_default()
    }

    /// Get dependents of a cell (cells that depend on it)
    pub fn get_dependents(&self, cell_hash: &str) -> Vec<&str> {
        self.dependency_graph.edges.iter()
            .filter(|(_, targets)| targets.iter().any(|t| t == cell_hash))
            .map(|(source, _)| source.as_str())
            .collect()
    }

    /// Rebuild indices from cell records
    pub fn rebuild_indices(&mut self) {
        self.indices = ManifestIndices::default();
        self.dependency_graph = DependencyGraph::default();

        for record in self.cells.values() {
            self.indices.by_type
                .entry(record.cell_type)
                .or_default()
                .push(record.hash.clone());
            self.indices.by_data_type
                .entry(record.data_type)
                .or_default()
                .push(record.hash.clone());

            for dep in &record.dependencies {
                self.dependency_graph.edges
                    .entry(record.hash.clone())
                    .or_default()
                    .push(dep.target.clone());
            }
        }
    }
}

fn canonicalize_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: Vec<_> = map.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            let inner: Vec<String> = sorted.iter()
                .map(|(k, v)| format!("\"{}\":{}", k, canonicalize_json(v)))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        serde_json::Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(canonicalize_json).collect();
            format!("[{}]", inner.join(","))
        }
        serde_json::Value::String(s) => serde_json::to_string(s).unwrap(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new("test@example.com".to_string());
        assert_eq!(manifest.metadata.owner, "test@example.com");
        assert_eq!(manifest.metadata.total_cells, 0);
        assert_eq!(manifest.metadata.total_tiles, 0);
        assert!(!manifest.metadata.store_id.is_empty());
    }

    #[test]
    fn test_manifest_add_cell_record() {
        let mut manifest = Manifest::new("owner".to_string());
        let record = CellRecord {
            hash: "abc123".to_string(),
            cell_type: 0x01,
            data_type: 0x01,
            shape: vec![10, 20],
            num_elements: 200,
            compression: 0,
            compressed_size: 1000,
            uncompressed_size: 1600,
            children: vec![],
            dependencies: vec![DependencyRecord {
                target: "def456".to_string(),
                dep_type: "DATA".to_string(),
                strength: 1.0,
            }],
            tile_hash: None,
            added_at: 0,
            lifecycle: "Live".to_string(),
            metadata: HashMap::new(),
        };

        manifest.add_cell_record(record).unwrap();

        assert_eq!(manifest.metadata.total_cells, 1);
        assert!(manifest.dependency_graph.edges.contains_key("abc123"));
    }

    #[test]
    fn test_manifest_remove_records() {
        let mut manifest = Manifest::new("owner".to_string());
        let cell = CellRecord {
            hash: "cell1".to_string(),
            cell_type: 1, data_type: 1, shape: vec![], num_elements: 0,
            compression: 0, compressed_size: 0, uncompressed_size: 0,
            children: vec![], dependencies: vec![], tile_hash: None,
            added_at: 0, lifecycle: "Live".to_string(), metadata: HashMap::new(),
        };
        manifest.add_cell_record(cell).unwrap();

        let tile = TileRecord {
            hash: "tile1".to_string(),
            location: TileLocationRecord { segment_idx: 0, tile_offset: 0, byte_offset: 0 },
            cell_hashes: vec![], size: 1024, dedup_count: 1,
            checksum: "abc".to_string(), created_at: 0,
            compression: "None".to_string(),
        };
        manifest.add_tile_record(tile).unwrap();

        assert_eq!(manifest.metadata.total_cells, 1);
        assert_eq!(manifest.metadata.total_tiles, 1);

        manifest.remove_cell_record("cell1");
        manifest.remove_tile_record("tile1");

        assert_eq!(manifest.metadata.total_cells, 0);
        assert_eq!(manifest.metadata.total_tiles, 0);
    }

    #[test]
    fn test_manifest_hash() {
        let manifest = Manifest::new("owner".to_string());
        let hash = manifest.compute_hash();
        assert!(hash != Blake3Hash::default());
    }

    #[test]
    fn test_manifest_rebuild_indices() {
        let mut manifest = Manifest::new("owner".to_string());
        let cell = CellRecord {
            hash: "cell1".to_string(),
            cell_type: 0x01, data_type: 0x01, shape: vec![], num_elements: 0,
            compression: 0, compressed_size: 0, uncompressed_size: 0,
            children: vec![], dependencies: vec![], tile_hash: None,
            added_at: 0, lifecycle: "Live".to_string(), metadata: HashMap::new(),
        };
        manifest.add_cell_record(cell).unwrap();
        manifest.cells.clear();
        manifest.indices.by_type.clear();

        manifest.rebuild_indices();
        assert!(manifest.indices.by_type.is_empty());
    }

    #[test]
    fn test_manifest_serde() {
        let manifest = Manifest::new("owner".to_string());
        let json = serde_json::to_string(&manifest).unwrap();
        let recovered: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest.metadata.owner, recovered.metadata.owner);
        assert_eq!(manifest.schema_version, recovered.schema_version);
    }
}
