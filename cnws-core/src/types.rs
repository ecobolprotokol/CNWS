//! Foundational types for CNWS
//! All types are content-addressed with BLAKE3-256

use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub fn hash_streaming<R: std::io::Read>(mut reader: R) -> Result<Self, std::io::Error> {
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
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
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
    pub parameters: std::collections::HashMap<String, String>,
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
