//! Manifest Authority - JSON source of truth for .cd store
//! 
//! Spec Ref: 04-cd-format-serialization.md §2.1 (Manifest Authority)
//! 
//! The Manifest is the canonical, authoritative record of:
//! - All cells in the store (hashes, types, locations)
//! - All tiles in the store (hashes, locations, checksums)
//! - Store metadata (created, updated, version)
//! - Indices (ANN, routing, memory)
//! 
//! The Manifest is stored as MANIFEST.cd (JSON) and is the only mutable file.
//! All other files (.tiles, indices) are immutable.

use crate::types::Blake3Hash;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manifest - JSON source of truth for .cd store
/// 
/// From Spec: "The Manifest.cd file is a JSON document that serves as the
/// authoritative record of all state in the store. It must be kept in sync
/// with the physical tiles through WAL-based atomic updates."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Store metadata
    pub metadata: ManifestMetadata,
    
    /// All cells in the store (by hash)
    pub cells: HashMap<String, CellRecord>,
    
    /// All tiles in the store (by hash)
    pub tiles: HashMap<String, TileRecord>,
    
    /// Cell indices for fast lookup
    pub indices: ManifestIndices,
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
    
    /// Tile where this cell lives
    pub tile_hash: Option<String>,
    
    /// Timestamp when added
    pub added_at: u64,
    
    /// Custom metadata
    pub metadata: HashMap<String, String>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Manifest {
    /// Create a new empty manifest
    pub fn new(owner: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
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
            },
            cells: HashMap::new(),
            tiles: HashMap::new(),
            indices: ManifestIndices {
                by_type: HashMap::new(),
                by_data_type: HashMap::new(),
                by_layer: HashMap::new(),
            },
        }
    }
    
    /// Add a cell record to manifest
    pub fn add_cell_record(&mut self, record: CellRecord) -> Result<()> {
        self.cells.insert(record.hash.clone(), record);
        self.metadata.total_cells += 1;
        self.update_timestamp();
        Ok(())
    }
    
    /// Add a tile record to manifest
    pub fn add_tile_record(&mut self, record: TileRecord) -> Result<()> {
        self.metadata.total_size += record.size;
        self.tiles.insert(record.hash.clone(), record);
        self.metadata.total_tiles += 1;
        self.update_timestamp();
        Ok(())
    }
    
    /// Save manifest to JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::CnwsError::Serialization(format!("Failed to serialize manifest: {}", e)))?;
        
        fs::write(path, json)?;
        
        Ok(())
    }
    
    /// Load manifest from JSON file
    pub fn load(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        
        serde_json::from_str(&json)
            .map_err(|e| crate::error::CnwsError::Deserialization(format!("Failed to parse manifest: {}", e)))
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
    }
    
    #[test]
    fn test_manifest_add_cell_record() {
        let mut manifest = Manifest::new("owner".to_string());
        let record = CellRecord {
            hash: "abc123".to_string(),
            cell_type: 1,
            data_type: 1,
            shape: vec![10, 20],
            num_elements: 200,
            compression: 0,
            compressed_size: 1000,
            uncompressed_size: 1600,
            children: vec![],
            tile_hash: None,
            added_at: 0,
            metadata: HashMap::new(),
        };
        
        manifest.add_cell_record(record).unwrap();
        
        assert_eq!(manifest.metadata.total_cells, 1);
        assert!(manifest.get_cell(&Blake3Hash::default()).is_none());
    }
    
    #[test]
    fn test_manifest_add_tile_record() {
        let mut manifest = Manifest::new("owner".to_string());
        let record = TileRecord {
            hash: "tile123".to_string(),
            location: TileLocationRecord {
                segment_idx: 0,
                tile_offset: 0,
                byte_offset: 0,
            },
            cell_hashes: vec![],
            size: 4 * 1024 * 1024,
            dedup_count: 1,
            checksum: "checksum".to_string(),
            created_at: 0,
        };
        
        manifest.add_tile_record(record).unwrap();
        
        assert_eq!(manifest.metadata.total_tiles, 1);
        assert_eq!(manifest.metadata.total_size, 4 * 1024 * 1024);
    }
    
    #[test]
    fn test_manifest_serde() {
        let manifest = Manifest::new("owner".to_string());
        let json = serde_json::to_string(&manifest).unwrap();
        let recovered: Manifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(manifest.metadata.owner, recovered.metadata.owner);
    }
}
