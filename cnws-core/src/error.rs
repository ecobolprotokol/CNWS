//! Error types for CNWS
//! All error codes follow the CNWS-E-* pattern from Engineering Contract §21

use thiserror::Error;
use std::path::PathBuf;

/// CNWS Error types
#[derive(Error, Debug)]
pub enum CnwsError {
    // Storage errors
    #[error("Store not found: {0}")]
    StoreNotFound(PathBuf),

    #[error("Corrupt store: {0}")]
    CorruptStore,

    #[error("Tile not found: {0}")]
    TileNotFound([u8; 32]),

    #[error("Cell not found: {0}")]
    CellNotFound([u8; 32]),

    #[error("Memory not found: {0}")]
    MemoryNotFound([u8; 32]),

    #[error("Revision not found: {0}")]
    RevisionNotFound([u8; 32]),

    #[error("Store full")]
    StoreFull,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    // Compression errors
    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Unsupported compression: {0:?}")]
    UnsupportedCompression(crate::types::Compression),

    // Integrity errors
    #[error("Integrity verification failed: {0}")]
    IntegrityFailed(String),

    #[error("Quarantine full")]
    QuarantineFull,

    // Revision errors
    #[error("Invalid revision: {0}")]
    InvalidRevision(String),

    #[error("Revision cycle detected")]
    RevisionCycle,

    #[error("Revision not found: {0}")]
    RevisionNotFound2(String),

    // Recovery errors
    #[error("WAL error: {0}")]
    WalError(String),

    #[error("Invalid WAL record: {0}")]
    InvalidWalRecord(u32),

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    // Conversion errors
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid model file: {0}")]
    InvalidModelFile(String),

    // Runtime errors
    #[error("Cell execution failed: {0}")]
    CellExecutionFailed(String),

    #[error("Budget exceeded")]
    BudgetExceeded,

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Dependency cycle detected")]
    DependencyCycle,

    // Memory errors
    #[error("Memory full")]
    MemoryFull,

    #[error("Invalid memory type: {0}")]
    InvalidMemoryType(String),

    // Routing errors
    #[error("Routing failed: {0}")]
    RoutingFailed(String),

    #[error("No route found")]
    NoRouteFound,

    // Cache errors
    #[error("Cache miss: {0}")]
    CacheMiss([u8; 32]),

    #[error("Cache full")]
    CacheFull,

    // API errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Timeout")]
    Timeout,

    // Generic errors
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl CnwsError {
    /// Get error code
    pub fn code(&self) -> &'static str {
        match self {
            Self::StoreNotFound(_) => "CNWS-E-STORE",
            Self::CorruptStore => "CNWS-E-CORRUPT",
            Self::TileNotFound(_) => "CNWS-E-TILE",
            Self::CellNotFound(_) => "CNWS-E-CELL",
            Self::MemoryNotFound(_) => "CNWS-E-MEMORY",
            Self::RevisionNotFound(_) => "CNWS-E-REVISION",
            Self::StoreFull => "CNWS-E-STORE-FULL",
            Self::Io(_) => "CNWS-E-IO",
            Self::Serialization(_) => "CNWS-E-SERIAL",
            Self::Deserialization(_) => "CNWS-E-DESERIAL",
            Self::Compression(_) => "CNWS-E-COMPRESS",
            Self::UnsupportedCompression(_) => "CNWS-E-COMPRESS",
            Self::IntegrityFailed(_) => "CNWS-E-INTEGRITY",
            Self::QuarantineFull => "CNWS-E-QUARANTINE",
            Self::InvalidRevision(_) => "CNWS-E-REVISION",
            Self::RevisionCycle => "CNWS-E-REVISION-CYCLE",
            Self::RevisionNotFound2(_) => "CNWS-E-REVISION",
            Self::WalError(_) => "CNWS-E-WAL",
            Self::InvalidWalRecord(_) => "CNWS-E-WAL",
            Self::RecoveryFailed(_) => "CNWS-E-RECOVERY",
            Self::UnsupportedFormat(_) => "CNWS-E-FORMAT",
            Self::ConversionError(_) => "CNWS-E-CONVERSION",
            Self::InvalidModelFile(_) => "CNWS-E-MODEL",
            Self::CellExecutionFailed(_) => "CNWS-E-EXECUTION",
            Self::BudgetExceeded => "CNWS-E-BUDGET",
            Self::InvalidQuery(_) => "CNWS-E-QUERY",
            Self::DependencyCycle => "CNWS-E-CYCLE",
            Self::MemoryFull => "CNWS-E-MEMORY-FULL",
            Self::InvalidMemoryType(_) => "CNWS-E-MEMORY",
            Self::RoutingFailed(_) => "CNWS-E-ROUTING",
            Self::NoRouteFound => "CNWS-E-ROUTE",
            Self::CacheMiss(_) => "CNWS-E-CACHE",
            Self::CacheFull => "CNWS-E-CACHE",
            Self::InvalidInput(_) => "CNWS-E-INPUT",
            Self::NotImplemented(_) => "CNWS-E-NOTIMPL",
            Self::PermissionDenied => "CNWS-E-PERM",
            Self::Timeout => "CNWS-E-TIMEOUT",
            Self::Internal(_) => "CNWS-E-INTERNAL",
            Self::Unknown(_) => "CNWS-E-UNKNOWN",
        }
    }

    /// Check if error is fatal
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::CorruptStore | Self::RevisionCycle | Self::DependencyCycle | Self::Internal(_)
        )
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Timeout | Self::CacheMiss(_) | Self::RoutingFailed(_)
        )
    }

    /// Check if error is transient
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Timeout | Self::StoreFull | Self::CacheFull | Self::MemoryFull
        )
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, CnwsError>;
