//! Storage API - public interface for store operations

use super::super::substrate::storage::{StorageEngine, StoreConfig, StoreStats};
use super::super::types::{Blake3Hash, Compression};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Store manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest hash
    pub hash: Blake3Hash,
    /// Store format version
    pub version: u32,
    /// Store creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Number of cells
    pub cell_count: u64,
    /// Number of tiles
    pub tile_count: u64,
    /// Total size in bytes
    pub total_size: u64,
    /// Compression algorithm
    pub compression: Compression,
    /// Root cell hash (if any)
    pub root_cell: Option<Blake3Hash>,
    /// Head revision ID
    pub head_revision: Option<Blake3Hash>,
    /// Number of segments
    pub segment_count: u32,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Manifest {
    /// Create a new manifest
    pub fn new(version: u32, compression: Compression) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            hash: Blake3Hash::default(),
            version,
            created_at: now,
            modified_at: now,
            cell_count: 0,
            tile_count: 0,
            total_size: 0,
            compression,
            root_cell: None,
            head_revision: None,
            segment_count: 0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Compute manifest hash
    pub fn compute_hash(&mut self) {
        let data = bincode::serialize(self).unwrap();
        self.hash = Blake3Hash::hash(&data);
    }
}

/// Storage API
pub struct StorageApi {
    store: Arc<StorageEngine>,
}

impl StorageApi {
    /// Create a new storage API
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self { store }
    }

    /// Open an existing store
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let config = StoreConfig {
            path: path.into(),
            ..Default::default()
        };
        let store = StorageEngine::open(config)?;
        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Create a new store
    pub fn create(path: impl Into<PathBuf>, compression: Compression) -> Result<Self> {
        let config = StoreConfig {
            path: path.into(),
            compression,
            ..Default::default()
        };
        let store = StorageEngine::create_store(config)?;
        Ok(Self {
            store: Arc::new(store),
        })
    }

    /// Write a tile using the store's configured compression
    pub fn write_tile(&self, data: &[u8]) -> Result<Blake3Hash> {
        let compression = self.store.config.compression;
        self.store.write_tile(data, compression)
    }

    /// Write a tile with explicit compression
    pub fn write_tile_with(&self, data: &[u8], compression: Compression) -> Result<Blake3Hash> {
        self.store.write_tile(data, compression)
    }

    /// Read a tile
    pub fn read_tile(&self, hash: &Blake3Hash) -> Result<Vec<u8>> {
        self.store.read_tile(hash)
    }

    /// Check if tile exists
    pub fn has_tile(&self, hash: &Blake3Hash) -> bool {
        self.store.has_tile(hash)
    }

    /// Delete a tile
    pub fn delete_tile(&self, hash: &Blake3Hash) -> Result<()> {
        self.store.delete_tile(hash)
    }

    /// List all tiles
    pub fn list_tiles(&self) -> Vec<Blake3Hash> {
        self.store.list_tiles()
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    /// Get tile location
    pub fn get_tile_location(&self, hash: &Blake3Hash) -> Option<super::super::types::TileLocation> {
        self.store.get_tile_location(hash)
    }

    /// Get segment count
    pub fn segment_count(&self) -> u32 {
        self.store.segment_count()
    }

    /// Get superblock info
    pub fn superblock(&self) -> crate::substrate::storage::Superblock {
        self.store.superblock()
    }

    /// Get manifest
    pub fn manifest(&self) -> Result<Manifest> {
        let stats = self.store.stats();
        let sb = self.store.superblock();
        let mut manifest = Manifest::new(sb.version, self.store.config.compression);
        manifest.tile_count = stats.total_tiles;
        manifest.total_size = stats.total_size;
        manifest.segment_count = stats.total_segments;
        manifest.cell_count = sb.cell_count;
        manifest.created_at = sb.created_at;
        manifest.modified_at = sb.modified_at;
        manifest.compute_hash();
        Ok(manifest)
    }

    /// Verify store integrity
    pub fn verify(&self) -> Result<Vec<crate::substrate::integrity::VerificationResult>> {
        use crate::substrate::integrity::IntegrityVerifier;
        let verifier = IntegrityVerifier::new(Arc::clone(&self.store));
        verifier.verify_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_api_create() {
        let dir = tempdir().unwrap();
        let api = StorageApi::create(dir.path().join("test.cd"), Compression::Zstd).unwrap();
        let data = b"test data";
        let hash = api.write_tile(data).unwrap();
        let read_data = api.read_tile(&hash).unwrap();
        assert_eq!(data, read_data.as_slice());
    }

    #[test]
    fn test_storage_api_write_tile_with() {
        let dir = tempdir().unwrap();
        let api = StorageApi::create(dir.path().join("test.cd"), Compression::None).unwrap();
        let data = b"test data with explicit compression";
        let hash = api.write_tile_with(data, Compression::Zstd).unwrap();
        let read_data = api.read_tile(&hash).unwrap();
        assert_eq!(data, read_data.as_slice());
    }

    #[test]
    fn test_storage_api_tile_location() {
        let dir = tempdir().unwrap();
        let api = StorageApi::create(dir.path().join("test.cd"), Compression::None).unwrap();
        let data = b"location test";
        let hash = api.write_tile(data).unwrap();

        let location = api.get_tile_location(&hash);
        assert!(location.is_some());
        assert_eq!(location.unwrap().size, data.len() as u64);
    }

    #[test]
    fn test_manifest() {
        let mut manifest = Manifest::new(1, Compression::Zstd);
        manifest.tile_count = 10;
        manifest.segment_count = 2;
        manifest.compute_hash();
        assert!(manifest.hash != Blake3Hash::default());
    }
}
