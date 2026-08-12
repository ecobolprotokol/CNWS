//! Storage engine for .cd store format
//!
//! Spec Ref: 04-cd-format-serialization.md
//!
//! Implements tile-based immutable storage with content addressing,
//! multi-segment support, and proper deduplication.

use crate::types::{Blake3Hash, Compression, TileLocation, SUPERBLOCK_MAGIC, SUPERBLOCK_SIZE, SEGMENT_HEADER_SIZE};
use crate::error::{CnwsError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;

/// Segment type constants
pub const SEGMENT_TYPE_TILES: u32 = 0x01;
pub const SEGMENT_TYPE_INDEX: u32 = 0x02;
pub const SEGMENT_TYPE_MEMORY: u32 = 0x03;
pub const SEGMENT_TYPE_ROUTING: u32 = 0x04;

/// Superblock - fixed 4096-byte header at start of store
///
/// Spec Ref: 04-cd-format-serialization.md §4.2
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
    /// Number of cells
    pub cell_count: u64,
    /// Head revision ID (32 bytes, zero if none)
    pub head_revision: [u8; 32],
    /// Manifest hash (32 bytes)
    pub manifest_hash: [u8; 32],
    /// Total logical (uncompressed) bytes
    pub total_logical_bytes: u64,
    /// Total stored (compressed) bytes
    pub total_stored_bytes: u64,
    /// Store flags (bitfield)
    pub flags: u32,
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
            cell_count: 0,
            head_revision: [0u8; 32],
            manifest_hash: [0u8; 32],
            total_logical_bytes: 0,
            total_stored_bytes: 0,
            flags: 0,
        }
    }

    /// Update modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
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
        buf[48..56].copy_from_slice(&self.cell_count.to_le_bytes());
        buf[56..88].copy_from_slice(&self.head_revision);
        buf[88..120].copy_from_slice(&self.manifest_hash);
        buf[120..128].copy_from_slice(&self.total_logical_bytes.to_le_bytes());
        buf[128..136].copy_from_slice(&self.total_stored_bytes.to_le_bytes());
        buf[136..140].copy_from_slice(&self.flags.to_le_bytes());
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
        let cell_count = u64::from_le_bytes(buf[48..56].try_into().unwrap());
        let mut head_revision = [0u8; 32];
        head_revision.copy_from_slice(&buf[56..88]);
        let mut manifest_hash = [0u8; 32];
        manifest_hash.copy_from_slice(&buf[88..120]);
        let total_logical_bytes = u64::from_le_bytes(buf[120..128].try_into().unwrap());
        let total_stored_bytes = u64::from_le_bytes(buf[128..136].try_into().unwrap());
        let flags = u32::from_le_bytes(buf[136..140].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            created_at,
            modified_at,
            segment_count,
            tile_count,
            total_size,
            cell_count,
            head_revision,
            manifest_hash,
            total_logical_bytes,
            total_stored_bytes,
            flags,
        })
    }
}

/// Segment header - fixed 4096-byte header for each segment
///
/// Spec Ref: 04-cd-format-serialization.md §8.3
#[derive(Debug, Clone)]
pub struct SegmentHeader {
    /// Magic bytes: "CNWSSEG1"
    pub magic: [u8; 8],
    /// Segment format version major
    pub version_major: u32,
    /// Segment format version minor
    pub version_minor: u32,
    /// Segment index (0-based)
    pub index: u32,
    /// Segment type
    pub segment_type: u32,
    /// Creation timestamp (nanoseconds)
    pub created_at_ns: u64,
    /// Number of tiles in this segment
    pub tile_count: u32,
    /// Payload region offset (after header)
    pub payload_offset: u64,
    /// Payload region size in bytes
    pub payload_size: u64,
    /// Index offset within segment
    pub index_offset: u64,
    /// Index size in bytes
    pub index_size: u64,
    /// Compression flags
    pub compression_flags: u32,
    /// Segment checksum (BLAKE3-256 of payload)
    pub checksum: [u8; 32],
    /// Segment flags
    pub flags: u32,
}

impl SegmentHeader {
    /// Create a new segment header
    pub fn new(index: u32, segment_type: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Self {
            magic: *b"CNWSSEG1",
            version_major: 1,
            version_minor: 0,
            index,
            segment_type,
            created_at_ns: now,
            tile_count: 0,
            payload_offset: SEGMENT_HEADER_SIZE as u64,
            payload_size: 0,
            index_offset: 0,
            index_size: 0,
            compression_flags: 0,
            checksum: [0u8; 32],
            flags: 0,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; SEGMENT_HEADER_SIZE] {
        let mut buf = [0u8; SEGMENT_HEADER_SIZE];
        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version_major.to_le_bytes());
        buf[12..16].copy_from_slice(&self.version_minor.to_le_bytes());
        buf[16..20].copy_from_slice(&self.index.to_le_bytes());
        buf[20..24].copy_from_slice(&self.segment_type.to_le_bytes());
        buf[24..32].copy_from_slice(&self.created_at_ns.to_le_bytes());
        buf[32..36].copy_from_slice(&self.tile_count.to_le_bytes());
        buf[36..44].copy_from_slice(&self.payload_offset.to_le_bytes());
        buf[44..52].copy_from_slice(&self.payload_size.to_le_bytes());
        buf[52..60].copy_from_slice(&self.index_offset.to_le_bytes());
        buf[60..68].copy_from_slice(&self.index_size.to_le_bytes());
        buf[68..72].copy_from_slice(&self.compression_flags.to_le_bytes());
        buf[72..104].copy_from_slice(&self.checksum);
        buf[104..108].copy_from_slice(&self.flags.to_le_bytes());
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8; SEGMENT_HEADER_SIZE]) -> Result<Self> {
        let magic = <[u8; 8]>::try_from(&buf[0..8]).map_err(|_| CnwsError::CorruptStore)?;
        if magic != *b"CNWSSEG1" {
            return Err(CnwsError::CorruptStore);
        }

        Ok(Self {
            magic,
            version_major: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            version_minor: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            index: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            segment_type: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            created_at_ns: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            tile_count: u32::from_le_bytes(buf[32..36].try_into().unwrap()),
            payload_offset: u64::from_le_bytes(buf[36..44].try_into().unwrap()),
            payload_size: u64::from_le_bytes(buf[44..52].try_into().unwrap()),
            index_offset: u64::from_le_bytes(buf[52..60].try_into().unwrap()),
            index_size: u64::from_le_bytes(buf[60..68].try_into().unwrap()),
            compression_flags: u32::from_le_bytes(buf[68..72].try_into().unwrap()),
            checksum: {
                let mut c = [0u8; 32];
                c.copy_from_slice(&buf[72..104]);
                c
            },
            flags: u32::from_le_bytes(buf[104..108].try_into().unwrap()),
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

    /// Get all entries
    pub fn entries(&self) -> &HashMap<Blake3Hash, TileLocation> {
        &self.entries
    }

    /// Get number of tiles
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get total size of all tiles
    pub fn total_size(&self) -> u64 {
        self.entries.values().map(|loc| loc.size).sum()
    }
}

/// Segment metadata - tracks state of a segment file
#[derive(Debug, Clone)]
pub struct SegmentState {
    /// Segment index
    pub index: u32,
    /// Current write offset (bytes used in payload region)
    pub write_offset: u64,
    /// Number of tiles written
    pub tile_count: u32,
    /// Segment size limit
    pub size_limit: u64,
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
    /// Number of deduplications
    pub dedup_count: u64,
}

/// Storage engine - manages .cd store
///
/// Implements multi-segment tile storage with content addressing.
pub struct StorageEngine {
    pub config: StoreConfig,
    superblock: Arc<RwLock<Superblock>>,
    registry: Arc<RwLock<TileRegistry>>,
    segments: Arc<RwLock<Vec<SegmentState>>>,
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

        // Scan segments
        let segments = Self::scan_segments(store_path, &config)?;

        Ok(Self {
            config,
            superblock: Arc::new(RwLock::new(superblock)),
            registry: Arc::new(RwLock::new(registry)),
            segments: Arc::new(RwLock::new(segments)),
            stats: Arc::new(RwLock::new(StoreStats::default())),
        })
    }

    /// Create a new store
    pub fn create_store(config: StoreConfig) -> Result<Self> {
        let store_path = &config.path;
        fs::create_dir_all(store_path)?;
        fs::create_dir_all(store_path.join("segments"))?;

        // Create superblock
        let superblock = Superblock::new(1);
        let mut file = File::create(store_path.join("SUPERBLOCK"))?;
        file.write_all(&superblock.to_bytes())?;

        // Create initial segment header
        let header = SegmentHeader::new(0, SEGMENT_TYPE_TILES);
        let seg_file = store_path.join("segments").join("segment_00000000.cd");
        let mut seg = File::create(&seg_file)?;
        seg.write_all(&header.to_bytes())?;

        // Create empty index
        let registry = TileRegistry::new();
        Self::save_registry(store_path, &registry)?;

        let segments = vec![SegmentState {
            index: 0,
            write_offset: 0,
            tile_count: 0,
            size_limit: config.segment_size,
        }];

        Ok(Self {
            config,
            superblock: Arc::new(RwLock::new(superblock)),
            registry: Arc::new(RwLock::new(registry)),
            segments: Arc::new(RwLock::new(segments)),
            stats: Arc::new(RwLock::new(StoreStats::default())),
        })
    }

    /// Scan existing segments to rebuild state
    fn scan_segments(store_path: &Path, config: &StoreConfig) -> Result<Vec<SegmentState>> {
        let segments_dir = store_path.join("segments");
        if !segments_dir.exists() {
            return Ok(Vec::new());
        }

        let mut segments = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(&segments_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "cd").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if let Ok(mut file) = File::open(&path) {
                let mut header_buf = [0u8; SEGMENT_HEADER_SIZE];
                if file.read_exact(&mut header_buf).is_ok() {
                    if let Ok(header) = SegmentHeader::from_bytes(&header_buf) {
                        let file_size = file.seek(SeekFrom::End(0)).unwrap_or(0);
                        let used = file_size.saturating_sub(SEGMENT_HEADER_SIZE as u64);
                        segments.push(SegmentState {
                            index: header.index,
                            write_offset: used,
                            tile_count: header.tile_count,
                            size_limit: config.segment_size,
                        });
                    }
                }
            }
        }

        Ok(segments)
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

        Ok(TileRegistry { entries })
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
        // Compute hash for content addressing
        let hash = Blake3Hash::hash(data);

        // Deduplication: check if already exists
        {
            let registry = self.registry.read();
            if registry.get(&hash).is_some() {
                let mut stats = self.stats.write();
                stats.dedup_count += 1;
                return Ok(hash);
            }
        }

        // Compress data
        let compressed = self.compress(data, compression)?;

        // Write to segment (multi-segment support)
        let location = self.write_to_segment(&compressed, compression)?;

        // Update registry
        {
            let mut registry = self.registry.write();
            registry.insert(hash, location);
        }

        // Update stats and superblock
        {
            let mut stats = self.stats.write();
            stats.total_tiles += 1;
            stats.write_count += 1;
            stats.total_size += compressed.len() as u64;
            stats.compressed_size += compressed.len() as u64;
        }

        {
            let mut sb = self.superblock.write();
            sb.tile_count += 1;
            sb.total_stored_bytes += compressed.len() as u64;
            sb.total_logical_bytes += data.len() as u64;
            sb.touch();
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

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.read_count += 1;
        }

        // Decompress using the compression recorded in the location
        self.decompress(&compressed, location.compression)
    }

    /// Write data to a segment with multi-segment support
    fn write_to_segment(&self, data: &[u8], compression: Compression) -> Result<TileLocation> {
        let store_path = &self.config.path;
        let segment_path = store_path.join("segments");
        fs::create_dir_all(&segment_path)?;

        let mut segments = self.segments.write();

        // Find a segment with enough space, or create a new one
        let mut target_idx = None;
        for seg in segments.iter_mut() {
            let available = seg.size_limit.saturating_sub(seg.write_offset + SEGMENT_HEADER_SIZE as u64);
            if data.len() as u64 <= available {
                target_idx = Some(seg.index);
                break;
            }
        }

        // Create new segment if needed
        let segment_idx = match target_idx {
            Some(idx) => idx,
            None => {
                let new_idx = segments.len() as u32;
                let header = SegmentHeader::new(new_idx, SEGMENT_TYPE_TILES);
                let seg_file = segment_path.join(format!("segment_{:08}.cd", new_idx));
                let mut seg = File::create(&seg_file)?;
                seg.write_all(&header.to_bytes())?;

                segments.push(SegmentState {
                    index: new_idx,
                    write_offset: 0,
                    tile_count: 0,
                    size_limit: self.config.segment_size,
                });

                // Update superblock
                {
                    let mut sb = self.superblock.write();
                    sb.segment_count = new_idx + 1;
                }

                new_idx
            }
        };

        let seg = &mut segments[segment_idx as usize];
        let byte_offset = seg.write_offset;

        // Write tile data
        let seg_file = segment_path.join(format!("segment_{:08}.cd", segment_idx));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&seg_file)?;

        file.write_all(data)?;
        file.sync_all()?;

        // Update segment state
        seg.write_offset += data.len() as u64;
        seg.tile_count += 1;

        Ok(TileLocation {
            segment_idx,
            tile_offset: seg.tile_count - 1,
            byte_offset: byte_offset + SEGMENT_HEADER_SIZE as u64,
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
        let mut stats = self.stats.read().clone();
        stats.total_segments = self.segments.read().len() as u32;
        stats.total_size = self.registry.read().total_size();
        stats
    }

    /// Get tile location
    pub fn get_tile_location(&self, hash: &Blake3Hash) -> Option<TileLocation> {
        self.registry.read().get(hash).cloned()
    }

    /// Get the superblock
    pub fn superblock(&self) -> Superblock {
        self.superblock.read().clone()
    }

    /// Check if tile exists
    pub fn has_tile(&self, hash: &Blake3Hash) -> bool {
        self.registry.read().get(hash).is_some()
    }

    /// Delete a tile (removes from registry; segment data becomes reclaimable by GC)
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

    /// Get number of segments
    pub fn segment_count(&self) -> u32 {
        self.segments.read().len() as u32
    }

    /// Get registry reference (for GC and other subsystems)
    pub fn registry(&self) -> &Arc<RwLock<TileRegistry>> {
        &self.registry
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
        assert_eq!(sb.cell_count, sb2.cell_count);
        assert_eq!(sb.total_logical_bytes, sb2.total_logical_bytes);
    }

    #[test]
    fn test_superblock_touch() {
        let mut sb = Superblock::new(1);
        let before = sb.modified_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        sb.touch();
        assert!(sb.modified_at >= before);
    }

    #[test]
    fn test_segment_header_serialization() {
        let header = SegmentHeader::new(0, SEGMENT_TYPE_TILES);
        let bytes = header.to_bytes();
        let header2 = SegmentHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.magic, header2.magic);
        assert_eq!(header.index, header2.index);
        assert_eq!(header.segment_type, header2.segment_type);
        assert_eq!(header.version_major, header2.version_major);
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
        assert_eq!(registry.total_size(), 4);
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

        let stats = engine.stats();
        assert_eq!(stats.total_tiles, 1);
    }

    #[test]
    fn test_storage_engine_deduplication() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let data = b"dedup test";

        let hash1 = engine.write_tile(data, Compression::None).unwrap();
        let hash2 = engine.write_tile(data, Compression::None).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(engine.stats().total_tiles, 1);
        assert_eq!(engine.stats().dedup_count, 1);
    }

    #[test]
    fn test_multi_segment_write() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            segment_size: 100, // Very small to force multiple segments
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();

        // Write tiles that exceed segment size
        for i in 0..5u8 {
            let data = vec![i; 40];
            engine.write_tile(&data, Compression::None).unwrap();
        }

        // Should have created multiple segments
        assert!(engine.segment_count() > 1);
    }

    #[test]
    fn test_storage_engine_compression() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let data = vec![0u8; 10000]; // Highly compressible

        let hash = engine.write_tile(&data, Compression::Zstd).unwrap();
        let read_data = engine.read_tile(&hash).unwrap();
        assert_eq!(data, read_data);
    }

    #[test]
    fn test_tile_not_found() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let hash = Blake3Hash::hash(b"nonexistent");
        assert!(engine.read_tile(&hash).is_err());
    }
}
