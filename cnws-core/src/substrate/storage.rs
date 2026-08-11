//! Storage engine for .cd store format
//! Implements tile-based immutable storage with content addressing

use crate::types::{Blake3Hash, Compression, TileLocation, SUPERBLOCK_MAGIC, SUPERBLOCK_SIZE, SEGMENT_HEADER_SIZE};
use crate::error::{CnwsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;

/// Superblock - fixed 4096-byte header at start of store
/// Stored in binary format (NOT serde) for compatibility
#[derive(Debug, Clone)]
pub struct Superblock {
    /// Magic bytes: "CNWSSB01"
    pub magic: [u8; 8],
    /// Store format version
    pub version: u32,
    /// Store creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Number of segments
    pub segment_count: u32,
    /// Number of tiles
    pub tile_count: u64,
    /// Total store size in bytes
    pub total_size: u64,
}

impl Superblock {
    /// Create a new superblock
    pub fn new(version: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            magic: *SUPERBLOCK_MAGIC,
            version,
            created_at: now,
            modified_at: now,
            segment_count: 0,
            tile_count: 0,
            total_size: SUPERBLOCK_SIZE as u64,
        }
    }

    /// Serialize to bytes (little-endian, 4096 bytes)
    pub fn to_bytes(&self) -> [u8; SUPERBLOCK_SIZE] {
        let mut buf = [0u8; SUPERBLOCK_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12..20].copy_from_slice(&self.created_at.to_le_bytes());
        buf[20..28].copy_from_slice(&self.modified_at.to_le_bytes());
        buf[28..32].copy_from_slice(&self.segment_count.to_le_bytes());
        buf[32..40].copy_from_slice(&self.tile_count.to_le_bytes());
        buf[40..48].copy_from_slice(&self.total_size.to_le_bytes());
        buf
    }

    /// Deserialize from bytes (little-endian)
    pub fn from_bytes(buf: &[u8; SUPERBLOCK_SIZE]) -> Result<Self> {
        let magic = <[u8; 8]>::try_from(&buf[0..8]).map_err(|_| CnwsError::CorruptStore)?;
        if magic != *SUPERBLOCK_MAGIC {
            return Err(CnwsError::CorruptStore);
        }

        let version = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let created_at = u64::from_le_bytes(buf[12..20].try_into().unwrap());
        let modified_at = u64::from_le_bytes(buf[20..28].try_into().unwrap());
        let segment_count = u32::from_le_bytes(buf[28..32].try_into().unwrap());
        let tile_count = u64::from_le_bytes(buf[32..40].try_into().unwrap());
        let total_size = u64::from_le_bytes(buf[40..48].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            created_at,
            modified_at,
            segment_count,
            tile_count,
            total_size,
        })
    }
}

/// Segment header - fixed 4096-byte header for each segment
/// Stored in binary format (NOT serde) for compatibility
#[derive(Debug, Clone)]
pub struct SegmentHeader {
    /// Magic bytes: "CNWSSEG1"
    pub magic: [u8; 8],
    /// Segment index (0-based)
    pub index: u32,
    /// Segment type
    pub segment_type: u32,
    /// Number of tiles in this segment
    pub tile_count: u32,
    /// Segment start offset in store
    pub start_offset: u64,
    /// Segment size in bytes
    pub size: u64,
    /// Segment checksum (BLAKE3-256)
    pub checksum: [u8; 32],
}

impl SegmentHeader {
    /// Create a new segment header
    pub fn new(index: u32, segment_type: u32, start_offset: u64) -> Self {
        Self {
            magic: *b"CNWSSEG1",
            index,
            segment_type,
            tile_count: 0,
            start_offset,
            size: 0,
            checksum: [0u8; 32],
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; SEGMENT_HEADER_SIZE] {
        let mut buf = [0u8; SEGMENT_HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.index.to_le_bytes());
        buf[12..16].copy_from_slice(&self.segment_type.to_le_bytes());
        buf[16..20].copy_from_slice(&self.tile_count.to_le_bytes());
        buf[20..28].copy_from_slice(&self.start_offset.to_le_bytes());
        buf[28..36].copy_from_slice(&self.size.to_le_bytes());
        buf[36..68].copy_from_slice(&self.checksum);
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8; SEGMENT_HEADER_SIZE]) -> Result<Self> {
        let magic = <[u8; 8]>::try_from(&buf[0..8]).map_err(|_| CnwsError::CorruptStore)?;
        if magic != *b"CNWSSEG1" {
            return Err(CnwsError::CorruptStore);
        }

        let index = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let segment_type = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let tile_count = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let start_offset = u64::from_le_bytes(buf[20..28].try_into().unwrap());
        let size = u64::from_le_bytes(buf[28..36].try_into().unwrap());
        let checksum = <[u8; 32]>::try_from(&buf[36..68]).map_err(|_| CnwsError::CorruptStore)?;

        Ok(Self {
            magic,
            index,
            segment_type,
            tile_count,
            start_offset,
            size,
            checksum,
        })
    }
}

/// Tile registry - maps tile hashes to locations
#[derive(Debug, Clone, Default)]
pub struct TileRegistry {
    /// Map from tile hash to location
    entries: HashMap<Blake3Hash, TileLocation>,
}

impl TileRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a tile location
    pub fn insert(&mut self, hash: Blake3Hash, location: TileLocation) {
        self.entries.insert(hash, location);
    }

    /// Get tile location by hash
    pub fn get(&self, hash: &Blake3Hash) -> Option<&TileLocation> {
        self.entries.get(hash)
    }

    /// Remove a tile location
    pub fn remove(&mut self, hash: &Blake3Hash) -> Option<TileLocation> {
        self.entries.remove(hash)
    }

    /// Get all tile hashes
    pub fn keys(&self) -> impl Iterator<Item = &Blake3Hash> {
        self.entries.keys()
    }

    /// Get number of tiles
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Store configuration
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Store path
    pub path: PathBuf,
    /// Segment size (default 1GB)
    pub segment_size: u64,
    /// Compression algorithm
    pub compression: Compression,
    /// Enable WAL
    pub enable_wal: bool,
    /// WAL path
    pub wal_path: Option<PathBuf>,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./store"),
            segment_size: 1024 * 1024 * 1024, // 1GB
            compression: Compression::Zstd,
            enable_wal: true,
            wal_path: None,
        }
    }
}

/// Store statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreStats {
    /// Total tiles
    pub total_tiles: u64,
    /// Total segments
    pub total_segments: u32,
    /// Total size in bytes
    pub total_size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Number of reads
    pub read_count: u64,
    /// Number of writes
    pub write_count: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
}

/// Storage engine - manages .cd store
pub struct StorageEngine {
    pub config: StoreConfig,
    superblock: Arc<RwLock<Superblock>>,
    registry: Arc<RwLock<TileRegistry>>,
    stats: Arc<RwLock<StoreStats>>,
}

impl StorageEngine {
    /// Open an existing store
    pub fn open(config: StoreConfig) -> Result<Self> {
        let store_path = &config.path;
        if !store_path.exists() {
            return Err(CnwsError::StoreNotFound(store_path.clone()));
        }

        // Read superblock
        let mut file = File::open(store_path.join("SUPERBLOCK"))?;
        let mut buf = [0u8; SUPERBLOCK_SIZE];
        file.read_exact(&mut buf)?;
        let superblock = Superblock::from_bytes(&buf)?;

        // Load tile registry from index
        let registry = Self::load_registry(store_path)?;

        Ok(Self {
            config,
            superblock: Arc::new(RwLock::new(superblock)),
            registry: Arc::new(RwLock::new(registry)),
            stats: Arc::new(RwLock::new(StoreStats::default())),
        })
    }

    /// Create a new store
    pub fn create_store(config: StoreConfig) -> Result<Self> {
        let store_path = &config.path;
        fs::create_dir_all(store_path)?;

        // Create superblock
        let superblock = Superblock::new(1);
        let mut file = File::create(store_path.join("SUPERBLOCK"))?;
        file.write_all(&superblock.to_bytes())?;

        // Create empty index
        let registry = TileRegistry::new();
        Self::save_registry(store_path, &registry)?;

        Ok(Self {
            config,
            superblock: Arc::new(RwLock::new(superblock)),
            registry: Arc::new(RwLock::new(registry)),
            stats: Arc::new(RwLock::new(StoreStats::default())),
        })
    }

    /// Load tile registry from index file
    fn load_registry(store_path: &Path) -> Result<TileRegistry> {
        let index_path = store_path.join("index.cd");
        if !index_path.exists() {
            return Ok(TileRegistry::new());
        }

        let data = fs::read(index_path)?;
        let entries: HashMap<Blake3Hash, TileLocation> =
            bincode::deserialize(&data).map_err(|e| CnwsError::Serialization(e.to_string()))?;

        Ok(TileRegistry {
            entries,
        })
    }

    /// Save tile registry to index file
    fn save_registry(store_path: &Path, registry: &TileRegistry) -> Result<()> {
        let index_path = store_path.join("index.cd");
        let data = bincode::serialize(&registry.entries)
            .map_err(|e| CnwsError::Serialization(e.to_string()))?;
        fs::write(index_path, data)?;
        Ok(())
    }

    /// Write a tile to store
    pub fn write_tile(&self, data: &[u8], compression: Compression) -> Result<Blake3Hash> {
        // Compute hash
        let hash = Blake3Hash::hash(data);

        // Check if already exists
        {
            let registry = self.registry.read();
            if registry.get(&hash).is_some() {
                return Ok(hash);
            }
        }

        // Compress data
        let compressed = self.compress(data, compression)?;

        // Write to segment
        let location = self.write_to_segment(&compressed, compression)?;

        // Update registry
        {
            let mut registry = self.registry.write();
            registry.insert(hash, location);
        }

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_tiles += 1;
            stats.write_count += 1;
            stats.compressed_size += compressed.len() as u64;
        }

        // Save registry
        StorageEngine::save_registry(&self.config.path, &self.registry.read())?;

        Ok(hash)
    }

    /// Read a tile from store
    pub fn read_tile(&self, hash: &Blake3Hash) -> Result<Vec<u8>> {
        // Check registry
        let location = {
            let registry = self.registry.read();
            registry.get(hash).cloned()
                .ok_or_else(|| CnwsError::TileNotFound)?
        };

        // Read from segment
        let compressed = self.read_from_segment(&location)?;

        // Decompress using the compression recorded in the location
        self.decompress(&compressed, location.compression)
    }

    /// Write data to a segment
    fn write_to_segment(&self, data: &[u8], compression: Compression) -> Result<TileLocation> {
        let store_path = &self.config.path;
        let segment_size = self.config.segment_size;

        // Find or create current segment
        let segment_path = store_path.join("segments");
        fs::create_dir_all(&segment_path)?;

        // For simplicity, append to a single segment file
        let segment_file = segment_path.join("segment_00000001.cd");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&segment_file)?;

        let offset = file.seek(SeekFrom::End(0))?;

        if offset + data.len() as u64 > segment_size {
            // Need new segment (simplified - in real impl would create new file)
            return Err(CnwsError::StoreFull);
        }

        file.write_all(data)?;
        file.sync_all()?;

        Ok(TileLocation {
            segment_idx: 1,
            tile_offset: 0,
            byte_offset: offset,
            size: data.len() as u64,
            compression,
        })
    }

    /// Read data from a segment
    fn read_from_segment(&self, location: &TileLocation) -> Result<Vec<u8>> {
        let store_path = &self.config.path;
        let segment_file = store_path
            .join("segments")
            .join(format!("segment_{:08}.cd", location.segment_idx));

        let mut file = File::open(segment_file)?;
        file.seek(SeekFrom::Start(location.byte_offset))?;

        let mut buf = vec![0u8; location.size as usize];
        file.read_exact(&mut buf)?;

        Ok(buf)
    }

    /// Compress data
    fn compress(&self, data: &[u8], compression: Compression) -> Result<Vec<u8>> {
        match compression {
            Compression::None => Ok(data.to_vec()),
            Compression::Zstd => {
                zstd::encode_all(data, 3)
                    .map_err(|e| CnwsError::Compression(e.to_string()))
            }
            Compression::Lz4 => {
                lz4::block::compress(data, None, false)
                    .map_err(|e| CnwsError::Compression(e.to_string()))
            }
            _ => Err(CnwsError::UnsupportedCompression(compression)),
        }
    }

    /// Decompress data
    fn decompress(&self, data: &[u8], compression: Compression) -> Result<Vec<u8>> {
        match compression {
            Compression::None => Ok(data.to_vec()),
            Compression::Zstd => {
                zstd::decode_all(data)
                    .map_err(|e| CnwsError::Compression(e.to_string()))
            }
            Compression::Lz4 => {
                lz4::block::decompress(data, None)
                    .map_err(|e| CnwsError::Compression(e.to_string()))
            }
            _ => Err(CnwsError::UnsupportedCompression(compression)),
        }
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        self.stats.read().clone()
    }

    /// Get tile location
    pub fn get_tile_location(&self, hash: &Blake3Hash) -> Option<TileLocation> {
        self.registry.read().get(hash).cloned()
    }

    /// Check if tile exists
    pub fn has_tile(&self, hash: &Blake3Hash) -> bool {
        self.registry.read().get(hash).is_some()
    }

    /// Delete a tile
    pub fn delete_tile(&self, hash: &Blake3Hash) -> Result<()> {
        let mut registry = self.registry.write();
        if registry.remove(hash).is_some() {
            StorageEngine::save_registry(&self.config.path, &registry)?;
            Ok(())
        } else {
            Err(CnwsError::TileNotFound)
        }
    }

    /// List all tile hashes
    pub fn list_tiles(&self) -> Vec<Blake3Hash> {
        self.registry.read().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_superblock_serialization() {
        let sb = Superblock::new(1);
        let bytes = sb.to_bytes();
        let sb2 = Superblock::from_bytes(&bytes).unwrap();
        assert_eq!(sb.magic, sb2.magic);
        assert_eq!(sb.version, sb2.version);
    }

    #[test]
    fn test_segment_header_serialization() {
        let header = SegmentHeader::new(0, 1, 4096);
        let bytes = header.to_bytes();
        let header2 = SegmentHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.magic, header2.magic);
        assert_eq!(header.index, header2.index);
    }

    #[test]
    fn test_tile_registry() {
        let mut registry = TileRegistry::new();
        let hash = Blake3Hash::hash(b"test");
        let location = TileLocation {
            segment_idx: 1,
            tile_offset: 0,
            byte_offset: 0,
            size: 4,
            compression: Compression::None,
        };
        registry.insert(hash, location);
        assert!(registry.get(&hash).is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_storage_engine_create_and_write() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let data = b"Hello, CNWS!";
        let hash = engine.write_tile(data, Compression::None).unwrap();

        let read_data = engine.read_tile(&hash).unwrap();
        assert_eq!(data, read_data.as_slice());
    }
}
