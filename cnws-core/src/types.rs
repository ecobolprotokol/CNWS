//! Foundational types for CNWS
//! All types are content-addressed with BLAKE3-256

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use crate::error::{CnwsError, Result};

// ============================================================================
// BLAKE3-256 Hash (32 bytes)
// ============================================================================

/// BLAKE3-256 hash - universal content-addressed identity
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);

impl Blake3Hash {
    /// Compute BLAKE3-256 hash of data
    pub fn hash(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(hash.into())
    }

    /// Compute BLAKE3-256 hash from stream
    pub fn hash_streaming<R: std::io::Read>(mut reader: R) -> std::result::Result<Self, std::io::Error> {
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 65536];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(Self(hasher.finalize().into()))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> std::result::Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Default for Blake3Hash {
    fn default() -> Self {
        Self([0u8; 32])
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for Blake3Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Blake3Hash> for [u8; 32] {
    fn from(hash: Blake3Hash) -> Self {
        hash.0
    }
}

// ============================================================================
// Revision ID (alias for Blake3Hash)
// ============================================================================

/// Revision ID - BLAKE3-256 of revision content
pub type RevisionId = Blake3Hash;

// ============================================================================
// Cell Type (35 types from Cell Schema Spec §3)
// ============================================================================

/// Cell type - 35 fundamental types
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum CellType {
    // Core computation
    Tensor = 0x01,
    Attention = 0x02,
    FFN = 0x03,
    LayerNorm = 0x04,
    Embedding = 0x05,
    Loss = 0x06,
    OptimizerState = 0x07,
    Gradient = 0x08,

    // Activation
    Activation = 0x09,
    Weight = 0x0A,
    Bias = 0x0B,
    Mask = 0x0C,

    // Positional
    PositionalEncoding = 0x0D,
    KV = 0x0E,
    Projection = 0x0F,
    Residual = 0x10,

    // Regularization
    Dropout = 0x11,
    Scale = 0x12,
    Shift = 0x13,
    Gate = 0x14,

    // Composition
    Merge = 0x15,
    Split = 0x16,
    Custom = 0x17,
}

impl TryFrom<u8> for CellType {
    type Error = CnwsError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(Self::Tensor),
            0x02 => Ok(Self::Attention),
            0x03 => Ok(Self::FFN),
            0x04 => Ok(Self::LayerNorm),
            0x05 => Ok(Self::Embedding),
            0x06 => Ok(Self::Loss),
            0x07 => Ok(Self::OptimizerState),
            0x08 => Ok(Self::Gradient),
            0x09 => Ok(Self::Activation),
            0x0A => Ok(Self::Weight),
            0x0B => Ok(Self::Bias),
            0x0C => Ok(Self::Mask),
            0x0D => Ok(Self::PositionalEncoding),
            0x0E => Ok(Self::KV),
            0x0F => Ok(Self::Projection),
            0x10 => Ok(Self::Residual),
            0x11 => Ok(Self::Dropout),
            0x12 => Ok(Self::Scale),
            0x13 => Ok(Self::Shift),
            0x14 => Ok(Self::Gate),
            0x15 => Ok(Self::Merge),
            0x16 => Ok(Self::Split),
            0x17 => Ok(Self::Custom),
            _ => Err(CnwsError::InvalidInput(format!("Unknown cell type: 0x{:02x}", value))),
        }
    }
}

impl From<CellType> for u8 {
    fn from(cell_type: CellType) -> Self {
        cell_type as u8
    }
}

// ============================================================================
// Data Type (13 types from Cell Schema Spec §3.2)
// ============================================================================

/// Data type - 13 fundamental types
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum DataType {
    F32 = 0x01,
    F16 = 0x02,
    BF16 = 0x03,
    F8 = 0x04,
    I8 = 0x05,
    I16 = 0x06,
    I32 = 0x07,
    I64 = 0x08,
    U8 = 0x09,
    U16 = 0x0A,
    U32 = 0x0B,
    U64 = 0x0C,
    Bool = 0x0D,
}

impl TryFrom<u8> for DataType {
    type Error = CnwsError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(Self::F32),
            0x02 => Ok(Self::F16),
            0x03 => Ok(Self::BF16),
            0x04 => Ok(Self::F8),
            0x05 => Ok(Self::I8),
            0x06 => Ok(Self::I16),
            0x07 => Ok(Self::I32),
            0x08 => Ok(Self::I64),
            0x09 => Ok(Self::U8),
            0x0A => Ok(Self::U16),
            0x0B => Ok(Self::U32),
            0x0C => Ok(Self::U64),
            0x0D => Ok(Self::Bool),
            _ => Err(CnwsError::InvalidInput(format!("Unknown data type: 0x{:02x}", value))),
        }
    }
}

impl From<DataType> for u8 {
    fn from(dt: DataType) -> Self {
        dt as u8
    }
}

impl DataType {
    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            Self::F32 | Self::I32 | Self::U32 => 4,
            Self::F16 | Self::BF16 | Self::I16 | Self::U16 => 2,
            Self::F8 | Self::I8 | Self::U8 | Self::Bool => 1,
            Self::I64 | Self::U64 => 8,
        }
    }
}

// ============================================================================
// Compression (8 types from .cd Format Spec §3.2)
// ============================================================================

/// Compression algorithm
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum Compression {
    None = 0x00,
    Zstd = 0x01,
    Lz4 = 0x02,
    Lz4Hc = 0x03,
    Zlib = 0x04,
    Brotli = 0x05,
    Lzma = 0x06,
    ZstdStream = 0x07,
}

impl TryFrom<u8> for Compression {
    type Error = CnwsError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Zstd),
            0x02 => Ok(Self::Lz4),
            0x03 => Ok(Self::Lz4Hc),
            0x04 => Ok(Self::Zlib),
            0x05 => Ok(Self::Brotli),
            0x06 => Ok(Self::Lzma),
            0x07 => Ok(Self::ZstdStream),
            _ => Err(CnwsError::InvalidInput(format!("Unknown compression: 0x{:02x}", value))),
        }
    }
}

impl From<Compression> for u8 {
    fn from(comp: Compression) -> Self {
        comp as u8
    }
}

// ============================================================================
// Memory Type (5 types from Memory Retrieval Spec §3)
// ============================================================================

/// Memory type
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MemoryType {
    Episodic = 0x01,
    Semantic = 0x02,
    Procedural = 0x03,
    Working = 0x04,
    LongTerm = 0x05,
}

impl TryFrom<u8> for MemoryType {
    type Error = CnwsError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(Self::Episodic),
            0x02 => Ok(Self::Semantic),
            0x03 => Ok(Self::Procedural),
            0x04 => Ok(Self::Working),
            0x05 => Ok(Self::LongTerm),
            _ => Err(CnwsError::InvalidInput(format!("Unknown memory type: 0x{:02x}", value))),
        }
    }
}

impl From<MemoryType> for u8 {
    fn from(mt: MemoryType) -> Self {
        mt as u8
    }
}

// ============================================================================
// Version
// ============================================================================

/// Version
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn current() -> Self {
        Self::new(1, 0, 0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// ============================================================================
// Compute Budget
// ============================================================================

/// Compute budget for execution
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComputeBudget {
    /// Maximum compute units
    pub max_compute: u64,
    /// Maximum depth
    pub max_depth: u32,
    /// Maximum bytes to move
    pub max_bytes: u64,
    /// Maximum time (seconds)
    pub max_time_secs: u64,
}

impl Default for ComputeBudget {
    fn default() -> Self {
        Self {
            max_compute: 1_000_000,
            max_depth: 100,
            max_bytes: 1_073_741_824, // 1GB
            max_time_secs: 300, // 5 minutes
        }
    }
}

// ============================================================================
// Access Policy
// ============================================================================

/// Access policy
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccessPolicy {
    ReadOnly,
    ReadWrite,
    Admin,
}

// ============================================================================
// Query
// ============================================================================

/// Query for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Entry cell hashes
    pub entry_cells: Vec<Blake3Hash>,
    /// Parameters
    pub parameters: HashMap<String, String>,
    /// Maximum depth
    pub max_depth: u32,
    /// Maximum compute
    pub max_compute: u64,
}

// ============================================================================
// Constants
// ============================================================================

/// Tile size (4MB)
pub const TILE_SIZE: usize = 4 * 1024 * 1024;

/// Superblock size (4096 bytes)
pub const SUPERBLOCK_SIZE: usize = 4096;

/// Segment header size (4096 bytes)
pub const SEGMENT_HEADER_SIZE: usize = 4096;

/// Memory index entry size (104 bytes)
pub const MEMORY_INDEX_ENTRY_SIZE: usize = 104;

/// Superblock magic bytes
pub const SUPERBLOCK_MAGIC: &[u8; 8] = b"CNWSSB01";

/// Segment magic bytes
pub const SEGMENT_MAGIC: &[u8; 8] = b"CNWSSEG1";

/// Index magic bytes
pub const INDEX_MAGIC: &[u8; 8] = b"CNWSIDX1";

/// Memory magic bytes
pub const MEMORY_MAGIC: &[u8; 8] = b"CNWSMEM1";

/// Revision magic bytes
pub const REVISION_MAGIC: &[u8; 8] = b"CNWSREV1";

/// Manifest magic bytes
pub const MANIFEST_MAGIC: &[u8; 8] = b"CNWSMAN1";

// ============================================================================
// Cell - Universal Unit (from Cell Schema Spec §4.1)
// ============================================================================

/// Cell - the universal unit in CNWS
/// Every weight, memory location, routing policy, and composition is a Cell
/// 
/// Spec Ref: 05-cell-schema.md §4.1 (Cell Structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Cell {
    /// Content-addressed identity (BLAKE3-256)
    pub id: Blake3Hash,
    
    /// Cell type (35 possible types)
    pub cell_type: CellType,
    
    /// Data type (13 possible types)
    pub data_type: DataType,
    
    /// Shape of the cell (dimensions)
    pub shape: Vec<u32>,
    
    /// Number of elements
    pub num_elements: u64,
    
    /// Compression algorithm
    pub compression: Compression,
    
    /// Compressed size in bytes
    pub compressed_size: u64,
    
    /// Uncompressed size in bytes
    pub uncompressed_size: u64,
    
    /// Child cell references (for composition)
    pub children: Vec<Blake3Hash>,
    
    /// Metadata (custom key-value pairs)
    pub metadata: HashMap<String, String>,
}

impl Cell {
    /// Create a new Cell
    pub fn new(
        cell_type: CellType,
        data_type: DataType,
        shape: Vec<u32>,
    ) -> Self {
        let num_elements = shape.iter().fold(1u64, |a, &b| a * (b as u64));
        let uncompressed_size = num_elements * (data_type.size() as u64);
        
        Self {
            id: Blake3Hash::default(),
            cell_type,
            data_type,
            shape,
            num_elements,
            compression: Compression::None,
            compressed_size: uncompressed_size,
            uncompressed_size,
            children: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Set compression
    pub fn with_compression(mut self, compression: Compression, compressed_size: u64) -> Self {
        self.compression = compression;
        self.compressed_size = compressed_size;
        self
    }
    
    /// Add child cell reference
    pub fn add_child(&mut self, child_id: Blake3Hash) {
        self.children.push(child_id);
    }
    
    /// Compute the hash of this cell (content-addressed identity)
    pub fn compute_id(&mut self) -> Result<Blake3Hash> {
        let serialized = serde_json::to_vec(self)
            .map_err(|e| CnwsError::InvalidInput(format!("Failed to serialize cell: {}", e)))?;
        self.id = Blake3Hash::hash(&serialized);
        Ok(self.id)
    }
}

// ============================================================================
// Tile - Storage Unit (from .cd Format Spec §3.3)
// ============================================================================

/// Tile - immutable storage unit (32-256 MiB, typically 4 MiB)
/// 
/// Spec Ref: 04-cd-format-serialization.md §3.3 (Tile Structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Tile {
    /// Content-addressed identity (BLAKE3-256)
    pub id: Blake3Hash,
    
    /// Tile location in .cd store
    pub location: TileLocation,
    
    /// Cell IDs stored in this tile
    pub cell_ids: Vec<Blake3Hash>,
    
    /// Physical size in bytes
    pub size: u64,
    
    /// Deduplication count (how many cells reference this tile)
    pub dedup_count: u32,
    
    /// Timestamp when created
    pub created_at: u64,
    
    /// Checksum (BLAKE3-256)
    pub checksum: Blake3Hash,
}

/// Tile location in .cd store
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TileLocation {
    /// Segment index
    pub segment_idx: u32,
    
    /// Tile offset within segment (in tile units)
    pub tile_offset: u32,
    
    /// Position in bytes within segment
    pub byte_offset: u64,
}

impl Tile {
    /// Create a new Tile
    pub fn new(location: TileLocation) -> Self {
        Self {
            id: Blake3Hash::default(),
            location,
            cell_ids: Vec::new(),
            size: 0,
            dedup_count: 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            checksum: Blake3Hash::default(),
        }
    }
    
    /// Add cell to tile
    pub fn add_cell(&mut self, cell_id: Blake3Hash) {
        self.cell_ids.push(cell_id);
    }
    
    /// Compute tile ID and checksum
    pub fn compute_id(&mut self, data: &[u8]) -> Result<Blake3Hash> {
        self.checksum = Blake3Hash::hash(data);
        self.size = data.len() as u64;
        self.id = self.checksum; // Tile ID = content hash
        Ok(self.id)
    }
}

// ============================================================================
// Cell Reference - Pointer to a Cell
// ============================================================================

/// Reference to a Cell with optional tile location
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CellRef {
    /// Cell ID (BLAKE3-256)
    pub id: Blake3Hash,
    
    /// Optional tile location hint (for efficient loading)
    pub tile_location: Option<TileLocation>,
}

impl CellRef {
    /// Create a new CellRef
    pub fn new(id: Blake3Hash) -> Self {
        Self {
            id,
            tile_location: None,
        }
    }
    
    /// Create a CellRef with tile location hint
    pub fn with_location(id: Blake3Hash, location: TileLocation) -> Self {
        Self {
            id,
            tile_location: Some(location),
        }
    }
}

// ============================================================================
// Tile Reference - Pointer to a Tile
// ============================================================================

/// Reference to a Tile with metadata
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TileRef {
    /// Tile ID (BLAKE3-256)
    pub id: Blake3Hash,
    
    /// Tile location
    pub location: TileLocation,
    
    /// Tile size in bytes
    pub size: u64,
}

impl TileRef {
    /// Create a new TileRef
    pub fn new(id: Blake3Hash, location: TileLocation, size: u64) -> Self {
        Self {
            id,
            location,
            size,
        }
    }
}

// ============================================================================
// Index Vector - Sparse indexing
// ============================================================================

/// Index vector for sparse indexing and routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexVector {
    /// Vector dimensions
    pub dimensions: u32,
    
    /// Vector values (sparse representation)
    pub values: Vec<IndexEntry>,
    
    /// Norm of the vector
    pub norm: f32,
}

/// Index entry in index vector
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct IndexEntry {
    /// Dimension index
    pub index: u32,
    
    /// Value
    pub value: Vec<u8>, // Stored as bytes for type flexibility
}

impl IndexVector {
    /// Create a new IndexVector
    pub fn new(dimensions: u32) -> Self {
        Self {
            dimensions,
            values: Vec::new(),
            norm: 0.0,
        }
    }
    
    /// Add an entry
    pub fn add_entry(&mut self, index: u32, value: Vec<u8>) {
        self.values.push(IndexEntry { index, value });
    }
}

// ============================================================================
// Metadata Schema
// ============================================================================

/// Metadata schema for cells and tiles
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    /// Schema version
    pub version: Version,
    
    /// Owner identifier
    pub owner: String,
    
    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Custom attributes
    pub attributes: HashMap<String, serde_json::Value>,
    
    /// Provenance (where it came from)
    pub provenance: Option<Provenance>,
}

/// Provenance information
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Provenance {
    /// Source model (e.g., "llama-7b")
    pub source_model: String,
    
    /// Import format (e.g., "safetensors", "gguf")
    pub import_format: String,
    
    /// Import timestamp
    pub import_timestamp: u64,
    
    /// Import revision/version
    pub revision: String,
}

impl Metadata {
    /// Create new metadata
    pub fn new(owner: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            version: Version::current(),
            owner,
            created_at: now,
            modified_at: now,
            attributes: std::collections::HashMap::new(),
            provenance: None,
        }
    }
}
