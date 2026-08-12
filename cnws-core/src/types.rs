//! Foundational types for CNWS
//! All types are content-addressed with BLAKE3-256
//!
//! Spec Ref: 05-cell-schema.md (Cell & Schema Specification)

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

impl fmt::LowerHex for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

// ============================================================================
// Revision ID (alias for Blake3Hash)
// ============================================================================

/// Revision ID - BLAKE3-256 of revision content
pub type RevisionId = Blake3Hash;

// ============================================================================
// CellType (57 types from Cell Schema Spec §3)
// ============================================================================

/// Cell type - 57 fundamental types organized by category
///
/// Spec Ref: 05-cell-schema.md §3 (CellType Taxonomy)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CellType {
    // ── Weight Cells (0x01–0x1F) ──────────────────────────────────────────

    /// Token/position embedding
    Embedding = 0x01,
    /// Attention query projection
    AttentionQProj = 0x02,
    /// Attention key projection
    AttentionKProj = 0x03,
    /// Attention value projection
    AttentionVProj = 0x04,
    /// Attention output projection
    AttentionOut = 0x05,
    /// MLP gate projection
    MlpGate = 0x06,
    /// MLP up projection
    MlpUp = 0x07,
    /// MLP down projection
    MlpDown = 0x08,
    /// MoE expert gate
    ExpertGate = 0x09,
    /// MoE expert router
    ExpertRoute = 0x0A,
    /// MoE expert weight
    ExpertWeight = 0x0B,
    /// LayerNorm weight
    LayerNormWeight = 0x0C,
    /// LayerNorm bias
    LayerNormBias = 0x0D,
    /// Language model head
    LmHead = 0x0E,
    /// Vision encoder weight
    VisionEncoder = 0x0F,
    /// Convolutional weight
    ConvWeight = 0x10,
    /// Generic normalization scale
    NormScale = 0x11,
    /// Generic normalization bias
    NormBias = 0x12,
    /// Positional encoding
    Positional = 0x13,
    /// Residual gating
    ResidualGate = 0x14,

    // ── Memory Cells (0x20–0x2F) ──────────────────────────────────────────

    /// Episodic memory (experiences)
    MemoryEpisodic = 0x20,
    /// Semantic memory (facts)
    MemorySemantic = 0x21,
    /// Procedural memory (patterns)
    MemoryProcedural = 0x22,
    /// Working memory (bounded)
    MemoryWorking = 0x23,
    /// Consolidated memory
    MemoryConsolidated = 0x24,
    /// Memory associations
    MemoryAssociation = 0x25,

    // ── Routing Cells (0x30–0x3F) ─────────────────────────────────────────

    /// Routing policy parameters
    RoutingPolicy = 0x30,
    /// Routing statistics
    RoutingStatistics = 0x31,
    /// ANN index for Cell selection
    RoutingIndex = 0x32,
    /// Cell association graph
    RoutingAssociation = 0x33,
    /// Selection thresholds
    RoutingThreshold = 0x34,

    // ── Composition Cells (0x40–0x4F) ─────────────────────────────────────

    /// Cached composition pattern
    CompositionPattern = 0x40,
    /// Reusable composition template
    CompositionTemplate = 0x41,
    /// Compiled macro-Cell
    CompositionMacro = 0x42,
    /// Sequential composition
    CompositionSequence = 0x43,
    /// Parallel composition
    CompositionParallel = 0x44,
    /// Conditional composition
    CompositionConditional = 0x45,
    /// Iterative composition
    CompositionIterative = 0x46,

    // ── Computation Cells (0x50–0x5F) ─────────────────────────────────────

    /// Generic transformation
    TransformModule = 0x50,
    /// Input encoder
    EncodeModule = 0x51,
    /// Output decoder
    DecodeModule = 0x52,
    /// Normalization module
    NormalizeModule = 0x53,
    /// Activation function
    ActivationModule = 0x54,
    /// Pooling operation
    PoolingModule = 0x55,
    /// Attention computation
    AttentionModule = 0x56,
    /// Convolution computation
    ConvolutionModule = 0x57,
    /// Recurrent computation
    RecurrentModule = 0x58,

    // ── Control Cells (0x60–0x6F) ─────────────────────────────────────────

    /// Halt condition
    HaltCondition = 0x60,
    /// Compute budget policy
    BudgetPolicy = 0x61,
    /// Branching condition
    BranchCondition = 0x62,
    /// Loop control
    LoopControl = 0x63,
    /// Error handling
    ErrorHandler = 0x64,

    // ── Meta Cells (0x70–0x7F) ────────────────────────────────────────────

    /// Provenance information
    Provenance = 0x70,
    /// Configuration parameters
    Configuration = 0x71,
    /// Usage statistics
    Statistics = 0x72,
    /// Human annotations
    Annotation = 0x73,
    /// Validation metadata
    Validation = 0x74,

    // ── Custom (0xFF) ─────────────────────────────────────────────────────

    /// Extensible custom type (uses registered type string)
    Custom = 0xFF,
}

impl CellType {
    /// Get the category of this cell type
    pub fn category(&self) -> CellTypeCategory {
        match *self as u8 {
            0x01..=0x1F => CellTypeCategory::Weight,
            0x20..=0x2F => CellTypeCategory::Memory,
            0x30..=0x3F => CellTypeCategory::Routing,
            0x40..=0x4F => CellTypeCategory::Composition,
            0x50..=0x5F => CellTypeCategory::Computation,
            0x60..=0x6F => CellTypeCategory::Control,
            0x70..=0x7F => CellTypeCategory::Meta,
            0xFF => CellTypeCategory::Custom,
            _ => CellTypeCategory::Reserved,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Embedding => "EMBEDDING",
            Self::AttentionQProj => "ATTENTION_Q_PROJ",
            Self::AttentionKProj => "ATTENTION_K_PROJ",
            Self::AttentionVProj => "ATTENTION_V_PROJ",
            Self::AttentionOut => "ATTENTION_OUT",
            Self::MlpGate => "MLP_GATE",
            Self::MlpUp => "MLP_UP",
            Self::MlpDown => "MLP_DOWN",
            Self::ExpertGate => "EXPERT_GATE",
            Self::ExpertRoute => "EXPERT_ROUTE",
            Self::ExpertWeight => "EXPERT_WEIGHT",
            Self::LayerNormWeight => "LAYERNORM_WEIGHT",
            Self::LayerNormBias => "LAYERNORM_BIAS",
            Self::LmHead => "LM_HEAD",
            Self::VisionEncoder => "VISION_ENCODER",
            Self::ConvWeight => "CONV_WEIGHT",
            Self::NormScale => "NORM_SCALE",
            Self::NormBias => "NORM_BIAS",
            Self::Positional => "POSITIONAL",
            Self::ResidualGate => "RESIDUAL_GATE",
            Self::MemoryEpisodic => "MEMORY_EPISODIC",
            Self::MemorySemantic => "MEMORY_SEMANTIC",
            Self::MemoryProcedural => "MEMORY_PROCEDURAL",
            Self::MemoryWorking => "MEMORY_WORKING",
            Self::MemoryConsolidated => "MEMORY_CONSOLIDATED",
            Self::MemoryAssociation => "MEMORY_ASSOCIATION",
            Self::RoutingPolicy => "ROUTING_POLICY",
            Self::RoutingStatistics => "ROUTING_STATISTICS",
            Self::RoutingIndex => "ROUTING_INDEX",
            Self::RoutingAssociation => "ROUTING_ASSOCIATION",
            Self::RoutingThreshold => "ROUTING_THRESHOLD",
            Self::CompositionPattern => "COMPOSITION_PATTERN",
            Self::CompositionTemplate => "COMPOSITION_TEMPLATE",
            Self::CompositionMacro => "COMPOSITION_MACRO",
            Self::CompositionSequence => "COMPOSITION_SEQUENCE",
            Self::CompositionParallel => "COMPOSITION_PARALLEL",
            Self::CompositionConditional => "COMPOSITION_CONDITIONAL",
            Self::CompositionIterative => "COMPOSITION_ITERATIVE",
            Self::TransformModule => "TRANSFORM_MODULE",
            Self::EncodeModule => "ENCODE_MODULE",
            Self::DecodeModule => "DECODE_MODULE",
            Self::NormalizeModule => "NORMALIZE_MODULE",
            Self::ActivationModule => "ACTIVATION_MODULE",
            Self::PoolingModule => "POOLING_MODULE",
            Self::AttentionModule => "ATTENTION_MODULE",
            Self::ConvolutionModule => "CONVOLUTION_MODULE",
            Self::RecurrentModule => "RECURRENT_MODULE",
            Self::HaltCondition => "HALT_CONDITION",
            Self::BudgetPolicy => "BUDGET_POLICY",
            Self::BranchCondition => "BRANCH_CONDITION",
            Self::LoopControl => "LOOP_CONTROL",
            Self::ErrorHandler => "ERROR_HANDLER",
            Self::Provenance => "PROVENANCE",
            Self::Configuration => "CONFIGURATION",
            Self::Statistics => "STATISTICS",
            Self::Annotation => "ANNOTATION",
            Self::Validation => "VALIDATION",
            Self::Custom => "CUSTOM",
        }
    }

    /// Check if this is a weight cell type
    pub fn is_weight(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Weight)
    }

    /// Check if this is a memory cell type
    pub fn is_memory(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Memory)
    }

    /// Check if this is a routing cell type
    pub fn is_routing(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Routing)
    }

    /// Check if this is a composition cell type
    pub fn is_composition(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Composition)
    }

    /// Check if this is a computation cell type
    pub fn is_computation(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Computation)
    }

    /// Check if this is a control cell type
    pub fn is_control(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Control)
    }

    /// Check if this is a meta cell type
    pub fn is_meta(&self) -> bool {
        matches!(self.category(), CellTypeCategory::Meta)
    }
}

/// Cell type categories
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellTypeCategory {
    Weight,
    Memory,
    Routing,
    Composition,
    Computation,
    Control,
    Meta,
    Custom,
    Reserved,
}

impl TryFrom<u8> for CellType {
    type Error = CnwsError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            // Weight cells
            0x01 => Ok(Self::Embedding),
            0x02 => Ok(Self::AttentionQProj),
            0x03 => Ok(Self::AttentionKProj),
            0x04 => Ok(Self::AttentionVProj),
            0x05 => Ok(Self::AttentionOut),
            0x06 => Ok(Self::MlpGate),
            0x07 => Ok(Self::MlpUp),
            0x08 => Ok(Self::MlpDown),
            0x09 => Ok(Self::ExpertGate),
            0x0A => Ok(Self::ExpertRoute),
            0x0B => Ok(Self::ExpertWeight),
            0x0C => Ok(Self::LayerNormWeight),
            0x0D => Ok(Self::LayerNormBias),
            0x0E => Ok(Self::LmHead),
            0x0F => Ok(Self::VisionEncoder),
            0x10 => Ok(Self::ConvWeight),
            0x11 => Ok(Self::NormScale),
            0x12 => Ok(Self::NormBias),
            0x13 => Ok(Self::Positional),
            0x14 => Ok(Self::ResidualGate),
            // Memory cells
            0x20 => Ok(Self::MemoryEpisodic),
            0x21 => Ok(Self::MemorySemantic),
            0x22 => Ok(Self::MemoryProcedural),
            0x23 => Ok(Self::MemoryWorking),
            0x24 => Ok(Self::MemoryConsolidated),
            0x25 => Ok(Self::MemoryAssociation),
            // Routing cells
            0x30 => Ok(Self::RoutingPolicy),
            0x31 => Ok(Self::RoutingStatistics),
            0x32 => Ok(Self::RoutingIndex),
            0x33 => Ok(Self::RoutingAssociation),
            0x34 => Ok(Self::RoutingThreshold),
            // Composition cells
            0x40 => Ok(Self::CompositionPattern),
            0x41 => Ok(Self::CompositionTemplate),
            0x42 => Ok(Self::CompositionMacro),
            0x43 => Ok(Self::CompositionSequence),
            0x44 => Ok(Self::CompositionParallel),
            0x45 => Ok(Self::CompositionConditional),
            0x46 => Ok(Self::CompositionIterative),
            // Computation cells
            0x50 => Ok(Self::TransformModule),
            0x51 => Ok(Self::EncodeModule),
            0x52 => Ok(Self::DecodeModule),
            0x53 => Ok(Self::NormalizeModule),
            0x54 => Ok(Self::ActivationModule),
            0x55 => Ok(Self::PoolingModule),
            0x56 => Ok(Self::AttentionModule),
            0x57 => Ok(Self::ConvolutionModule),
            0x58 => Ok(Self::RecurrentModule),
            // Control cells
            0x60 => Ok(Self::HaltCondition),
            0x61 => Ok(Self::BudgetPolicy),
            0x62 => Ok(Self::BranchCondition),
            0x63 => Ok(Self::LoopControl),
            0x64 => Ok(Self::ErrorHandler),
            // Meta cells
            0x70 => Ok(Self::Provenance),
            0x71 => Ok(Self::Configuration),
            0x72 => Ok(Self::Statistics),
            0x73 => Ok(Self::Annotation),
            0x74 => Ok(Self::Validation),
            // Custom
            0xFF => Ok(Self::Custom),
            // Reserved ranges: 0x15–0x1F, 0x26–0x2F, 0x35–0x3F, 0x47–0x4F,
            // 0x59–0x5F, 0x65–0x6F, 0x75–0x7F, 0x80–0xFE
            _ => Err(CnwsError::InvalidInput(format!(
                "Unknown or reserved cell type: 0x{:02x}",
                value
            ))),
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
#[repr(u8)]
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

    /// Check if this type is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F16 | Self::BF16 | Self::F8)
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::U8 | Self::U16 | Self::U32 | Self::U64)
    }

    /// Check widening compatibility: can `self` widen to `target`?
    pub fn can_widen_to(&self, target: &DataType) -> bool {
        matches!(
            (self, target),
            (Self::F16, Self::F32)
                | (Self::BF16, Self::F32)
                | (Self::F8, Self::F32)
                | (Self::F8, Self::F16)
                | (Self::I8, Self::I16)
                | (Self::I8, Self::I32)
                | (Self::I8, Self::I64)
                | (Self::I16, Self::I32)
                | (Self::I16, Self::I64)
                | (Self::I32, Self::I64)
                | (Self::U8, Self::U16)
                | (Self::U8, Self::U32)
                | (Self::U8, Self::U64)
                | (Self::U16, Self::U32)
                | (Self::U16, Self::U64)
                | (Self::U32, Self::U64)
        )
    }
}

// ============================================================================
// Compression (8 types from .cd Format Spec §3.2)
// ============================================================================

/// Compression algorithm
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
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
#[repr(u8)]
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
// Schema Types (from Cell Schema Spec §4)
// ============================================================================

/// Schema kind
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SchemaKind {
    Tensor,
    Structured,
    Scalar,
    Graph,
    Empty,
}

/// Tensor layout
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TensorLayout {
    RowMajor,
    ColumnMajor,
    Blocked,
    Sparse,
}

impl Default for TensorLayout {
    fn default() -> Self {
        Self::RowMajor
    }
}

/// Tensor schema
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TensorSchema {
    pub shape: Vec<u64>,
    pub dtype: DataType,
    pub layout: TensorLayout,
    pub dynamic_dims: Vec<u32>,
}

impl TensorSchema {
    pub fn new(shape: Vec<u64>, dtype: DataType) -> Self {
        Self {
            shape,
            dtype,
            layout: TensorLayout::default(),
            dynamic_dims: Vec::new(),
        }
    }
}

/// Field schema for structured data
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FieldSchema {
    pub name: String,
    pub dtype: DataType,
    pub shape: Option<Vec<u64>>,
    pub required: bool,
}

/// Structured schema
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct StructuredSchema {
    pub fields: Vec<FieldSchema>,
}

/// Scalar schema
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ScalarSchema {
    pub dtype: DataType,
}

/// Graph schema
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GraphSchema {
    pub node_schema: Box<Schema>,
    pub edge_schema: Box<Schema>,
    pub directed: bool,
}

/// Schema for cell input/output
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Schema {
    pub kind: SchemaKind,
    pub tensor: Option<TensorSchema>,
    pub structured: Option<StructuredSchema>,
    pub scalar: Option<ScalarSchema>,
    pub graph: Option<GraphSchema>,
}

impl Schema {
    /// Create an empty schema
    pub fn empty() -> Self {
        Self {
            kind: SchemaKind::Empty,
            tensor: None,
            structured: None,
            scalar: None,
            graph: None,
        }
    }

    /// Create a tensor schema
    pub fn tensor(shape: Vec<u64>, dtype: DataType) -> Self {
        Self {
            kind: SchemaKind::Tensor,
            tensor: Some(TensorSchema::new(shape, dtype)),
            structured: None,
            scalar: None,
            graph: None,
        }
    }

    /// Create a scalar schema
    pub fn scalar(dtype: DataType) -> Self {
        Self {
            kind: SchemaKind::Scalar,
            tensor: None,
            structured: None,
            scalar: Some(ScalarSchema { dtype }),
            graph: None,
        }
    }

    /// Check if this schema is compatible with another
    pub fn is_compatible_with(&self, other: &Schema) -> bool {
        if self.kind != other.kind {
            return false;
        }
        match (&self.kind, &other.kind) {
            (SchemaKind::Tensor, SchemaKind::Tensor) => {
                let a = self.tensor.as_ref().unwrap();
                let b = other.tensor.as_ref().unwrap();
                a.dtype == b.dtype || a.dtype.can_widen_to(&b.dtype)
            }
            (SchemaKind::Scalar, SchemaKind::Scalar) => {
                let a = self.scalar.as_ref().unwrap();
                let b = other.scalar.as_ref().unwrap();
                a.dtype == b.dtype || a.dtype.can_widen_to(&b.dtype)
            }
            (SchemaKind::Empty, SchemaKind::Empty) => true,
            _ => true,
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// Dependency Types (from Cell Schema Spec §5)
// ============================================================================

/// Dependency type
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Data flows from target to this Cell
    Data,
    /// Control flow dependency
    Control,
    /// Cell must execute after target
    ExecutionOrder,
    /// Target should be prefetched (not hard dependency)
    PrefetchHint,
    /// Semantic relationship (no execution impact)
    Semantic,
}

/// Dependency metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyMetadata {
    /// Weight of dependency (0.0–1.0)
    pub strength: f32,
    /// Whether dependency is conditional
    pub conditional: bool,
    /// Condition expression (if conditional)
    pub condition: Option<String>,
    /// Custom annotations
    pub annotations: HashMap<String, String>,
}

impl Default for DependencyMetadata {
    fn default() -> Self {
        Self {
            strength: 1.0,
            conditional: false,
            condition: None,
            annotations: HashMap::new(),
        }
    }
}

/// Cell dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    /// Target cell ID
    pub target: Blake3Hash,
    /// Dependency type
    pub dep_type: DependencyType,
    /// Dependency metadata
    pub metadata: DependencyMetadata,
}

impl Dependency {
    /// Create a new DATA dependency
    pub fn data(target: Blake3Hash) -> Self {
        Self {
            target,
            dep_type: DependencyType::Data,
            metadata: DependencyMetadata::default(),
        }
    }

    /// Create a new CONTROL dependency
    pub fn control(target: Blake3Hash) -> Self {
        Self {
            target,
            dep_type: DependencyType::Control,
            metadata: DependencyMetadata::default(),
        }
    }

    /// Create a new EXECUTION_ORDER dependency
    pub fn execution_order(target: Blake3Hash) -> Self {
        Self {
            target,
            dep_type: DependencyType::ExecutionOrder,
            metadata: DependencyMetadata::default(),
        }
    }

    /// Create a new PREFETCH_HINT dependency
    pub fn prefetch_hint(target: Blake3Hash) -> Self {
        Self {
            target,
            dep_type: DependencyType::PrefetchHint,
            metadata: DependencyMetadata::default(),
        }
    }

    /// Create a new SEMANTIC dependency
    pub fn semantic(target: Blake3Hash) -> Self {
        Self {
            target,
            dep_type: DependencyType::Semantic,
            metadata: DependencyMetadata::default(),
        }
    }

    /// Check if this is a hard dependency (must be satisfied before execution)
    pub fn is_hard(&self) -> bool {
        matches!(
            self.dep_type,
            DependencyType::Data | DependencyType::Control | DependencyType::ExecutionOrder
        )
    }
}

// ============================================================================
// Cell Lifecycle (from Cell Schema Spec §7)
// ============================================================================

/// Cell lifecycle state
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellLifecycle {
    /// Cell is being constructed
    Constructing,
    /// Cell is complete and immutable
    Live,
    /// Cell is superseded by a newer version
    Deprecated,
    /// Cell is marked for removal
    Tombstone,
}

// ============================================================================
// Cell Metadata (from Cell Schema Spec §6)
// ============================================================================

/// Cell metadata
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CellMetadata {
    /// Creation timestamp (nanoseconds since epoch)
    pub created_at_ns: u64,
    /// Last modification timestamp
    pub modified_at_ns: u64,
    /// Author
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Custom tags
    pub tags: Vec<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
    /// Cell lifecycle state
    pub lifecycle: CellLifecycle,
}

impl Default for CellMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            created_at_ns: now,
            modified_at_ns: now,
            author: None,
            description: None,
            tags: Vec::new(),
            attributes: HashMap::new(),
            lifecycle: CellLifecycle::Live,
        }
    }
}

// ============================================================================
// Representation Reference
// ============================================================================

/// Reference to an alternative representation of a cell
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RepresentationRef {
    /// Representation hash
    pub hash: Blake3Hash,
    /// Data type of this representation
    pub dtype: DataType,
    /// Shape of this representation
    pub shape: Vec<u64>,
    /// Compression used
    pub compression: Compression,
    /// Size in bytes
    pub size: u64,
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

/// Default index vector dimensions
pub const DEFAULT_INDEX_DIMENSIONS: u32 = 512;

/// Default tile size for conversion (128 MiB)
pub const DEFAULT_CONVERSION_TILE_SIZE: usize = 128 * 1024 * 1024;

// ============================================================================
// Cell - Universal Unit (from Cell Schema Spec §2)
// ============================================================================

/// Cell - the universal unit in CNWS
///
/// Every weight, memory location, routing policy, and composition is a Cell.
///
/// Spec Ref: 05-cell-schema.md §2.1 (Cell Structure)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cell {
    /// Content-addressed identity (BLAKE3-256)
    pub id: Blake3Hash,

    /// Cell type
    pub cell_type: CellType,

    /// Data type
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

    /// Cell version (semver)
    pub version: Version,

    /// Input schema
    pub input_schema: Schema,

    /// Output schema
    pub output_schema: Schema,

    /// Tile references (physical storage)
    pub tiles: Vec<Blake3Hash>,

    /// Child cell references (for composition)
    pub children: Vec<Blake3Hash>,

    /// Dependencies
    pub dependencies: Vec<Dependency>,

    /// Index vector for content-based retrieval
    pub index_vector: IndexVector,

    /// Alternative representations
    pub representations: Vec<RepresentationRef>,

    /// Cell metadata
    pub cell_metadata: CellMetadata,

    /// Custom metadata (key-value pairs)
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
            version: Version::current(),
            input_schema: Schema::empty(),
            output_schema: Schema::empty(),
            tiles: Vec::new(),
            children: Vec::new(),
            dependencies: Vec::new(),
            index_vector: IndexVector::new(DEFAULT_INDEX_DIMENSIONS),
            representations: Vec::new(),
            cell_metadata: CellMetadata::default(),
            metadata: HashMap::new(),
        }
    }

    /// Set compression
    pub fn with_compression(mut self, compression: Compression, compressed_size: u64) -> Self {
        self.compression = compression;
        self.compressed_size = compressed_size;
        self
    }

    /// Set input schema
    pub fn with_input_schema(mut self, schema: Schema) -> Self {
        self.input_schema = schema;
        self
    }

    /// Set output schema
    pub fn with_output_schema(mut self, schema: Schema) -> Self {
        self.output_schema = schema;
        self
    }

    /// Add child cell reference
    pub fn add_child(&mut self, child_id: Blake3Hash) {
        self.children.push(child_id);
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    /// Add a tile reference
    pub fn add_tile(&mut self, tile_hash: Blake3Hash) {
        self.tiles.push(tile_hash);
    }

    /// Add a representation
    pub fn add_representation(&mut self, repr: RepresentationRef) {
        self.representations.push(repr);
    }

    /// Get hard dependencies (DATA, CONTROL, EXECUTION_ORDER)
    pub fn hard_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.is_hard()).collect()
    }

    /// Get prefetch hints
    pub fn prefetch_hints(&self) -> Vec<&Dependency> {
        self.dependencies.iter()
            .filter(|d| matches!(d.dep_type, DependencyType::PrefetchHint))
            .collect()
    }

    /// Get semantic dependencies
    pub fn semantic_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter()
            .filter(|d| matches!(d.dep_type, DependencyType::Semantic))
            .collect()
    }

    /// Compute the hash of this cell (content-addressed identity)
    pub fn compute_id(&mut self) -> Result<Blake3Hash> {
        let serialized = serde_json::to_vec(self)
            .map_err(|e| CnwsError::InvalidInput(format!("Failed to serialize cell: {}", e)))?;
        self.id = Blake3Hash::hash(&serialized);
        Ok(self.id)
    }

    /// Check if cell has a specific lifecycle state
    pub fn is_live(&self) -> bool {
        self.cell_metadata.lifecycle == CellLifecycle::Live
    }

    /// Check if cell is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.cell_metadata.lifecycle == CellLifecycle::Deprecated
    }

    /// Check if cell is a tombstone
    pub fn is_tombstone(&self) -> bool {
        self.cell_metadata.lifecycle == CellLifecycle::Tombstone
    }

    /// Get storage ratio (compressed / uncompressed)
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 0.0;
        }
        self.compressed_size as f64 / self.uncompressed_size as f64
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

    /// Size of the stored data in bytes
    pub size: u64,

    /// Compression used for this tile
    pub compression: Compression,
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
    pub value: Vec<u8>,
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

    /// Check if vector is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Compute cosine similarity with another vector (simplified)
    pub fn cosine_similarity(&self, other: &IndexVector) -> f32 {
        if self.values.is_empty() || other.values.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0f32;
        let self_map: HashMap<u32, &[u8]> = self.values.iter()
            .map(|e| (e.index, e.value.as_slice()))
            .collect();

        for entry in &other.values {
            if let Some(self_val) = self_map.get(&entry.index) {
                // Simplified: compare byte-level similarity
                let min_len = self_val.len().min(entry.value.len());
                for i in 0..min_len {
                    dot_product += (self_val[i] as f32) * (entry.value[i] as f32);
                }
            }
        }

        let norm_product = self.norm * other.norm;
        if norm_product == 0.0 {
            0.0
        } else {
            dot_product / norm_product
        }
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
            attributes: HashMap::new(),
            provenance: None,
        }
    }
}

// ============================================================================
// Tensor Name Patterns (for conversion spec §7)
// ============================================================================

/// Well-known tensor name patterns for LLM models
pub struct TensorPatterns;

impl TensorPatterns {
    /// Detect CellType from tensor name
    pub fn infer_cell_type(name: &str) -> CellType {
        let lower = name.to_lowercase();
        if lower.contains("embed") {
            CellType::Embedding
        } else if lower.contains("q_proj") || lower.contains("query") {
            CellType::AttentionQProj
        } else if lower.contains("k_proj") || lower.contains("key") {
            CellType::AttentionKProj
        } else if lower.contains("v_proj") || lower.contains("value") {
            CellType::AttentionVProj
        } else if lower.contains("o_proj") || lower.contains("output") && lower.contains("attn") {
            CellType::AttentionOut
        } else if lower.contains("gate_proj") || lower.contains("gate") && lower.contains("mlp") {
            CellType::MlpGate
        } else if lower.contains("up_proj") || lower.contains("up") && lower.contains("mlp") {
            CellType::MlpUp
        } else if lower.contains("down_proj") || lower.contains("down") && lower.contains("mlp") {
            CellType::MlpDown
        } else if lower.contains("lm_head") || lower.contains("head") {
            CellType::LmHead
        } else if lower.contains("layernorm") || lower.contains("ln") {
            if lower.contains("weight") || lower.contains("scale") || lower.contains("gamma") {
                CellType::LayerNormWeight
            } else {
                CellType::LayerNormBias
            }
        } else if lower.contains("norm") {
            if lower.contains("bias") {
                CellType::NormBias
            } else {
                CellType::NormScale
            }
        } else if lower.contains("position") || lower.contains("pos") {
            CellType::Positional
        } else if lower.contains("conv") || lower.contains("wq") {
            CellType::ConvWeight
        } else if lower.contains("expert") && lower.contains("gate") {
            CellType::ExpertGate
        } else if lower.contains("expert") && lower.contains("route") {
            CellType::ExpertRoute
        } else if lower.contains("expert") {
            CellType::ExpertWeight
        } else if lower.contains("residual") || lower.contains("res") {
            CellType::ResidualGate
        } else {
            // Default to a generic weight type
            CellType::NormScale
        }
    }
}

// ============================================================================
// Convenience type aliases
// ============================================================================

/// Cell ID (alias for Blake3Hash)
pub type CellId = Blake3Hash;

/// Tile ID (alias for Blake3Hash)
pub type TileId = Blake3Hash;

/// Segment index type
pub type SegmentIndex = u32;
