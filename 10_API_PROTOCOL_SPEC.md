# CNWS
## API & Protocol Specification

| Field | Value |
|---|---|
| Dokumen | CNWS API & Protocol Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (API SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS DAS; seluruh spesifikasi subsystem |
| Hulu ke | Implementasi API layer, SDK, CLI, RPC server, Integration tests |
| Otoritas | Spesifikasi tunggal untuk seluruh public API CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    Subsystem Specs       API & Protocol Spec         Implementation
─────────────────────   ──────────────────    ────────────────────────    ─────────────
High-level interfaces ──► RuntimeResolver  ──► Complete trait defs     ──► API Layer
"MUST provide API"        StorageEngine         Type definitions           SDK
                          ConversionPipeline    Error semantics            CLI
                          RevisionManager       Async patterns             RPC Server
                          MemorySystem          Versioning                 Bindings
                                                SDK boundary
```

`[API-DOC-1]` Dokumen ini mendefinisikan **API yang benar-benar dipakai** oleh konsumen CNWS.

`[API-DOC-2]` Engineering Contract menetapkan interface level tinggi; dokumen ini mendefinisikan signature lengkap, type, error, async semantics, dan versioning.

`[API-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[API-DOC-4]` Jika terjadi konflik dengan subsystem spec untuk hal perilaku, subsystem spec menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-API-01 | API versioning menggunakan semver. |
| DF-API-02 | Error type menggunakan enum hierarchy dengan error code. |
| DF-API-03 | Async runtime: tokio (default), configurable. |
| DF-API-04 | SDK boundary: Rust crate `cnws` sebagai primary SDK. |
| DF-API-05 | RPC: gRPC (optional), untuk remote access. |
| DF-API-06 | Backward compatibility: minor version additions only. |
| DF-API-07 | Breaking changes: major version bump. |
| DF-API-08 | Deprecation: minimum 2 minor versions notice. |
| DF-API-09 | API surface: Storage, Conversion, Runtime, Revision, Memory, Admin. |
| DF-API-10 | Handle-based resource management. |
| DF-API-11 | Builder pattern untuk configuration. |
| DF-API-12 | Result<T, CnwsError> untuk semua fallible operations. |

---

# 1. Executive Summary

## 1.1 API Philosophy

`[API-EXEC-1]` CNWS API mengikuti prinsip:

1. **Explicit over implicit**: semua operasi eksplisit, tidak ada magic.
2. **Type-safe**: Rust type system menangkap error saat compile time.
3. **Async-first**: operasi I/O async by default.
4. **Handle-based**: resource dikelola melalui handle dengan lifetime jelas.
5. **Versioned**: API memiliki version yang jelas dan stabil.
6. **Minimal surface**: API surface sekecil mungkin, tetapi lengkap.

## 1.2 API Surface Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS API SURFACE                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Storage API │  │ Conversion  │  │ Runtime API │        │
│  │             │  │ API         │  │             │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Revision API│  │ Memory API  │  │ Admin API   │        │
│  │             │  │             │  │             │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Core Types & Errors                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Async Runtime & Lifecycle               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 1.3 Consumer Types

| Consumer | API Used |
|---|---|
| Application developer | Runtime API, Memory API |
| ML Engineer | Conversion API, Revision API |
| Platform engineer | Storage API, Admin API |
| SDK developer | All APIs |
| CLI tool | All APIs via SDK |

---

# 2. API Architecture Overview

## 2.1 Layered API Design

```text
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                         │
│              (User code, CLI, Services)                      │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    SDK LAYER (cnws crate)                    │
│                                                             │
│   High-level APIs, builders, helpers                        │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    CORE API LAYER                            │
│                                                             │
│   Traits, types, error handling                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    SUBSYSTEM LAYER                           │
│                                                             │
│   Storage Engine, Conversion, Runtime, Revision, Memory     │
└─────────────────────────────────────────────────────────────┘
```

## 2.2 API Categories

| Category | Purpose | Sync/Async |
|---|---|---|
| Storage API | Store operations, Tile I/O | Async |
| Conversion API | Checkpoint import | Async |
| Runtime API | Cell/Tile resolution, execution | Async |
| Revision API | Versioning, branching, merging | Mixed |
| Memory API | Memory read/write/retrieve | Async |
| Admin API | Maintenance, GC, health | Mixed |

---

# 3. Core Types

## 3.1 Identity Types

```rust
/// BLAKE3-256 hash (32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self;
    pub fn to_hex(&self) -> String;           // "b3:..." format
    pub fn from_hex(s: &str) -> Result<Self, CnwsError>;
}

/// Cell identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId(pub Blake3Hash);

/// Tile identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId(pub Blake3Hash);

/// Revision identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RevisionId(pub Blake3Hash);

/// Segment identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SegmentId(pub u64);

/// Memory entry identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryId(pub Blake3Hash);

/// Model identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelId(pub String);
```

## 3.2 Handle Types

```rust
/// Handle to an open CNWS store
pub struct StoreHandle {
    inner: Arc<StoreInner>,
}

/// Handle to a loaded Cell
pub struct CellHandle {
    cell_id: CellId,
    store: StoreHandle,
}

/// Handle to a loaded Tile
pub struct TileHandle {
    tile_id: TileId,
    data: Arc<TileData>,
    representation: RepresentationId,
}

/// Handle to a memory entry
pub struct MemoryHandle {
    memory_id: MemoryId,
    store: StoreHandle,
}

/// Handle to a revision
pub struct RevisionHandle {
    revision_id: RevisionId,
    store: StoreHandle,
}
```

## 3.3 Data Types

```rust
/// Data type for tensor elements
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
    F32 = 0x01,
    F16 = 0x02,
    BF16 = 0x03,
    F8E4M3 = 0x04,
    F8E5M2 = 0x05,
    I8 = 0x06,
    U8 = 0x07,
    I16 = 0x08,
    I32 = 0x09,
    I64 = 0x0A,
    BOOL = 0x0B,
    I4 = 0x0C,
    I2 = 0x0D,
}

impl DataType {
    pub fn bytes_per_element(&self) -> f64;
    pub fn is_quantized(&self) -> bool;
}

/// Cell type taxonomy
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CellType {
    // Weight cells (0x01-0x1F)
    Embedding = 0x01,
    AttentionQProj = 0x02,
    AttentionKProj = 0x03,
    AttentionVProj = 0x04,
    AttentionOut = 0x05,
    MlpGate = 0x06,
    MlpUp = 0x07,
    MlpDown = 0x08,
    ExpertGate = 0x09,
    ExpertRoute = 0x0A,
    ExpertWeight = 0x0B,
    LayerNormWeight = 0x0C,
    LayerNormBias = 0x0D,
    LmHead = 0x0E,
    VisionEncoder = 0x0F,
    
    // Memory cells (0x20-0x2F)
    MemoryEpisodic = 0x20,
    MemorySemantic = 0x21,
    MemoryProcedural = 0x22,
    
    // Routing cells (0x30-0x3F)
    RoutingPolicy = 0x30,
    RoutingStatistics = 0x31,
    
    // Composition cells (0x40-0x4F)
    CompositionPattern = 0x40,
    
    // Computation cells (0x50-0x5F)
    TransformModule = 0x50,
    EncodeModule = 0x51,
    DecodeModule = 0x52,
    
    // Custom
    Custom(String) = 0xFF,
}

/// Representation identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepresentationId(pub String);

impl RepresentationId {
    pub const CANONICAL_BF16: Self = Self("bf16".to_string());
    pub const FP16: Self = Self("fp16".to_string());
    pub const FP8_E4M3: Self = Self("fp8_e4m3".to_string());
    pub const INT8: Self = Self("int8".to_string());
    pub const INT4: Self = Self("int4".to_string());
}

/// Compression codec
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Compression {
    None = 0x0000,
    Zstd1 = 0x0001,
    Zstd3 = 0x0002,
    Zstd5 = 0x0003,
    Zstd9 = 0x0004,
    Zstd19 = 0x0005,
    Lz4 = 0x0010,
    Snappy = 0x0020,
}
```

## 3.4 Configuration Types

```rust
/// Store configuration
#[derive(Clone, Debug)]
pub struct StoreConfig {
    pub path: PathBuf,
    pub create_if_missing: bool,
    pub read_only: bool,
    pub cache_config: CacheConfig,
    pub budget_config: BudgetConfig,
}

/// Cache configuration
#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub gpu_budget_bytes: u64,
    pub cpu_budget_bytes: u64,
    pub eviction_policy: EvictionPolicy,
    pub prefetch_policy: PrefetchPolicy,
    pub prefetch_depth: u32,
}

/// Budget configuration
#[derive(Clone, Debug)]
pub struct BudgetConfig {
    pub max_flops: u64,
    pub max_bytes_moved: u64,
    pub max_wall_time_us: u64,
    pub working_memory_bytes: u64,
}

/// Eviction policy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvictionPolicy {
    Fifo,
    Lru,
    Lfu,
    LruByPriority,
}

/// Prefetch policy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefetchPolicy {
    NextLayer,
    DependencyAware,
    MoeTopK,
    Sequential,
    Adaptive,
}
```

## 3.5 Builder Pattern

`[API-TYPE-1]` Configuration MUST menggunakan builder pattern.

```rust
/// Builder for StoreConfig
pub struct StoreConfigBuilder {
    config: StoreConfig,
}

impl StoreConfigBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self;
    pub fn create_if_missing(mut self, create: bool) -> Self;
    pub fn read_only(mut self, read_only: bool) -> Self;
    pub fn gpu_budget(mut self, bytes: u64) -> Self;
    pub fn cpu_budget(mut self, bytes: u64) -> Self;
    pub fn eviction_policy(mut self, policy: EvictionPolicy) -> Self;
    pub fn prefetch_policy(mut self, policy: PrefetchPolicy) -> Self;
    pub fn build(self) -> Result<StoreConfig, CnwsError>;
}

// Usage:
let config = StoreConfigBuilder::new("model.cd")
    .create_if_missing(true)
    .gpu_budget(16 * GB)
    .cpu_budget(64 * GB)
    .eviction_policy(EvictionPolicy::LruByPriority)
    .build()?;
```

---

# 4. Error Semantics

## 4.1 Error Type Hierarchy

`[API-ERR-1]` Semua error MUST menggunakan `CnwsError` enum.

```rust
/// Top-level CNWS error type
#[derive(Debug, thiserror::Error)]
pub enum CnwsError {
    // Store errors
    #[error("Store error: {0}")]
    Store(#[from] StoreError),
    
    // Conversion errors
    #[error("Conversion error: {0}")]
    Conversion(#[from] ConversionError),
    
    // Runtime errors
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    
    // Revision errors
    #[error("Revision error: {0}")]
    Revision(#[from] RevisionError),
    
    // Memory errors
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),
    
    // Integrity errors
    #[error("Integrity error: {0}")]
    Integrity(#[from] IntegrityError),
    
    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    // Version errors
    #[error("Version error: {0}")]
    Version(#[from] VersionError),
}

impl CnwsError {
    pub fn error_code(&self) -> ErrorCode;
    pub fn is_recoverable(&self) -> bool;
    pub fn is_retryable(&self) -> bool;
}
```

## 4.2 Error Subtypes

```rust
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Store not found: {path}")]
    NotFound { path: PathBuf },
    
    #[error("Store already exists: {path}")]
    AlreadyExists { path: PathBuf },
    
    #[error("Store is locked by another process")]
    Locked,
    
    #[error("Store is corrupted")]
    Corrupted,
    
    #[error("Store is sealed (read-only)")]
    Sealed,
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Unknown format: {format}")]
    UnknownFormat { format: String },
    
    #[error("Invalid checkpoint: {reason}")]
    InvalidCheckpoint { reason: String },
    
    #[error("Malformed header: {reason}")]
    MalformedHeader { reason: String },
    
    #[error("Unsafe content detected: {reason}")]
    UnsafeContent { reason: String },
    
    #[error("Tensor name cannot be mapped: {name}")]
    UnknownTensorName { name: String },
    
    #[error("Unsupported dtype: {dtype}")]
    UnsupportedDtype { dtype: String },
    
    #[error("Conversion interrupted")]
    Interrupted,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Cell not found: {cell_id}")]
    CellNotFound { cell_id: CellId },
    
    #[error("Tile not found: {tile_id}")]
    TileNotFound { tile_id: TileId },
    
    #[error("Budget exceeded: {resource}")]
    BudgetExceeded { resource: String },
    
    #[error("No suitable representation for hardware")]
    NoSuitableRepresentation,
    
    #[error("Execution timeout after {timeout_us} us")]
    Timeout { timeout_us: u64 },
    
    #[error("Halt condition: {reason}")]
    HaltCondition { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum RevisionError {
    #[error("Revision not found: {revision_id}")]
    NotFound { revision_id: RevisionId },
    
    #[error("Merge conflict: {conflicts}")]
    MergeConflict { conflicts: Vec<CellId> },
    
    #[error("Invalid rollback target: {revision_id}")]
    InvalidRollbackTarget { revision_id: RevisionId },
    
    #[error("Branch already exists: {name}")]
    BranchAlreadyExists { name: String },
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Memory not found: {memory_id}")]
    NotFound { memory_id: MemoryId },
    
    #[error("Memory budget exceeded")]
    BudgetExceeded,
    
    #[error("Working memory full")]
    WorkingMemoryFull,
    
    #[error("Consolidation failed: {reason}")]
    ConsolidationFailed { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum IntegrityError {
    #[error("BLAKE3 mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: Blake3Hash, actual: Blake3Hash },
    
    #[error("Manifest tampered")]
    ManifestTampered,
    
    #[error("Segment corrupted: {segment_id}")]
    SegmentCorrupted { segment_id: SegmentId },
    
    #[error("Tile corrupted: {tile_id}")]
    TileCorrupted { tile_id: TileId },
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {reason}")]
    Invalid { reason: String },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Unsupported version: {version}")]
    Unsupported { version: String },
    
    #[error("Version downgrade not allowed: {from} -> {to}")]
    DowngradeNotAllowed { from: String, to: String },
}
```

## 4.3 Error Codes

`[API-ERR-2]` Setiap error MUST memiliki stable error code.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    // Store errors (1xxx)
    StoreNotFound = 1001,
    StoreAlreadyExists = 1002,
    StoreLocked = 1003,
    StoreCorrupted = 1004,
    StoreSealed = 1005,
    
    // Conversion errors (2xxx)
    ConversionUnknownFormat = 2001,
    ConversionInvalidCheckpoint = 2002,
    ConversionMalformedHeader = 2003,
    ConversionUnsafeContent = 2004,
    ConversionUnknownTensorName = 2005,
    ConversionUnsupportedDtype = 2006,
    ConversionInterrupted = 2007,
    
    // Runtime errors (3xxx)
    RuntimeCellNotFound = 3001,
    RuntimeTileNotFound = 3002,
    RuntimeBudgetExceeded = 3003,
    RuntimeNoSuitableRepresentation = 3004,
    RuntimeTimeout = 3005,
    RuntimeHaltCondition = 3006,
    
    // Revision errors (4xxx)
    RevisionNotFound = 4001,
    RevisionMergeConflict = 4002,
    RevisionInvalidRollbackTarget = 4003,
    RevisionBranchAlreadyExists = 4004,
    
    // Memory errors (5xxx)
    MemoryNotFound = 5001,
    MemoryBudgetExceeded = 5002,
    MemoryWorkingMemoryFull = 5003,
    MemoryConsolidationFailed = 5004,
    
    // Integrity errors (6xxx)
    IntegrityHashMismatch = 6001,
    IntegrityManifestTampered = 6002,
    IntegritySegmentCorrupted = 6003,
    IntegrityTileCorrupted = 6004,
    
    // Config errors (7xxx)
    ConfigInvalid = 7001,
    ConfigMissingField = 7002,
    
    // Version errors (8xxx)
    VersionUnsupported = 8001,
    VersionDowngradeNotAllowed = 8002,
}
```

## 4.4 Error Handling Pattern

`[API-ERR-3]` Semua fallible operations MUST mengembalikan `Result<T, CnwsError>`.

```rust
// Example usage
fn load_cell(store: &StoreHandle, cell_id: CellId) -> Result<CellHandle, CnwsError> {
    let cell = store.resolve_cell(cell_id)?;  // Propagates CnwsError
    Ok(cell)
}

// Pattern matching on errors
match store.load_tile(tile_id).await {
    Ok(tile) => { /* use tile */ }
    Err(CnwsError::Integrity(IntegrityError::TileCorrupted { tile_id })) => {
        // Handle corruption
        store.quarantine_tile(tile_id)?;
    }
    Err(CnwsError::Runtime(RuntimeError::BudgetExceeded { resource })) => {
        // Handle budget exceeded
        log::warn!("Budget exceeded for {}", resource);
    }
    Err(e) => {
        // Handle other errors
        return Err(e);
    }
}
```

## 4.5 Error Invariants

| ID | Invariant |
|---|---|
| API-ERR-INV-1 | Semua error MUST menggunakan CnwsError |
| API-ERR-INV-2 | Setiap error MUST memiliki stable error code |
| API-ERR-INV-3 | Error MUST implement std::error::Error |
| API-ERR-INV-4 | Error MUST implement Display |
| API-ERR-INV-5 | Error MUST dapat diklasifikasikan recoverable/retryable |

---

# 5. Storage API

## 5.1 Store Operations

```rust
/// Primary storage API trait
pub trait StorageEngine: Send + Sync {
    // Store lifecycle
    fn open(config: &StoreConfig) -> Result<StoreHandle, CnwsError>;
    fn close(store: &StoreHandle) -> Result<(), CnwsError>;
    
    // Store info
    fn store_info(store: &StoreHandle) -> Result<StoreInfo, CnwsError>;
    fn model_id(store: &StoreHandle) -> Result<ModelId, CnwsError>;
    
    // Tile operations
    async fn lookup_tile(
        store: &StoreHandle,
        tile_id: TileId,
    ) -> Result<TileLocation, CnwsError>;
    
    async fn read_tile(
        store: &StoreHandle,
        tile_id: TileId,
    ) -> Result<TileData, CnwsError>;
    
    async fn read_tile_verified(
        store: &StoreHandle,
        tile_id: TileId,
    ) -> Result<TileData, CnwsError>;
    
    async fn read_tiles(
        store: &StoreHandle,
        tile_ids: &[TileId],
    ) -> Result<Vec<TileData>, CnwsError>;
    
    fn tile_exists(store: &StoreHandle, tile_id: TileId) -> Result<bool, CnwsError>;
    
    // Segment operations
    fn list_segments(store: &StoreHandle) -> Result<Vec<SegmentInfo>, CnwsError>;
    fn segment_info(store: &StoreHandle, segment_id: SegmentId) -> Result<SegmentInfo, CnwsError>;
    
    // Maintenance
    async fn gc(store: &StoreHandle) -> Result<GcReport, CnwsError>;
    async fn gc_dry_run(store: &StoreHandle) -> Result<GcReport, CnwsError>;
    async fn verify_integrity(store: &StoreHandle) -> Result<IntegrityReport, CnwsError>;
    async fn recover(store: &StoreHandle) -> Result<RecoveryReport, CnwsError>;
}

/// Store information
#[derive(Clone, Debug)]
pub struct StoreInfo {
    pub model_id: ModelId,
    pub format_version: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub cell_count: u64,
    pub tile_count: u64,
    pub segment_count: u64,
    pub total_logical_bytes: u64,
    pub total_stored_bytes: u64,
    pub head_revision: RevisionId,
}

/// Tile location
#[derive(Clone, Debug)]
pub struct TileLocation {
    pub tile_id: TileId,
    pub segment_id: SegmentId,
    pub offset: u64,
    pub stored_size: u64,
    pub payload_size: u64,
    pub compression: Compression,
}

/// Tile data
#[derive(Clone)]
pub struct TileData {
    pub tile_id: TileId,
    pub data: Arc<Vec<u8>>,
    pub dtype: DataType,
    pub shape: Vec<u64>,
}

/// Segment information
#[derive(Clone, Debug)]
pub struct SegmentInfo {
    pub segment_id: SegmentId,
    pub tile_count: u64,
    pub size_bytes: u64,
    pub created_at: u64,
}

/// GC report
#[derive(Clone, Debug)]
pub struct GcReport {
    pub reachable_tiles: u64,
    pub unreachable_tiles: u64,
    pub reclaimed_tiles: u64,
    pub bytes_reclaimed: u64,
    pub duration_ms: u64,
}

/// Integrity report
#[derive(Clone, Debug)]
pub struct IntegrityReport {
    pub tiles_checked: u64,
    pub tiles_valid: u64,
    pub tiles_corrupted: u64,
    pub corrupted_tile_ids: Vec<TileId>,
    pub manifest_valid: bool,
    pub segments_valid: u64,
    pub segments_corrupted: u64,
}

/// Recovery report
#[derive(Clone, Debug)]
pub struct RecoveryReport {
    pub wal_records_processed: u64,
    pub manifest_recovered: bool,
    pub segments_recovered: u64,
    pub tiles_recovered: u64,
    pub tiles_unrecoverable: u64,
    pub status: RecoveryStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryStatus {
    Complete,
    Partial,
    Failed,
    NotNeeded,
}
```

## 5.2 Storage API Invariants

| ID | Invariant |
|---|---|
| API-STOR-INV-1 | Store open MUST idempotent untuk path yang sama |
| API-STOR-INV-2 | Tile read MUST async |
| API-STOR-INV-3 | Tile read_verified MUST melakukan BLAKE3 verification |
| API-STOR-INV-4 | GC MUST NOT menghapus Tile reachable |
| API-STOR-INV-5 | Store close MUST flush semua pending writes |

---

# 6. Conversion API

## 6.1 Conversion Operations

```rust
/// Conversion API trait
pub trait ConversionPipeline: Send + Sync {
    /// Convert a checkpoint to CNWS format
    async fn convert(
        &self,
        source: &SourceDescriptor,
        target: &Path,
        options: ConversionOptions,
    ) -> Result<ConversionReport, CnwsError>;
    
    /// Validate a checkpoint without converting
    async fn validate(
        &self,
        source: &SourceDescriptor,
    ) -> Result<ValidationReport, CnwsError>;
    
    /// Detect format of a checkpoint
    fn detect_format(
        &self,
        source: &Path,
    ) -> Result<SourceFormat, CnwsError>;
    
    /// List tensors in a checkpoint without loading data
    async fn list_tensors(
        &self,
        source: &SourceDescriptor,
    ) -> Result<Vec<TensorInfo>, CnwsError>;
}

/// Source descriptor
#[derive(Clone, Debug)]
pub struct SourceDescriptor {
    pub path: PathBuf,
    pub format: Option<SourceFormat>,
    pub options: ImportOptions,
}

/// Source format
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFormat {
    Safetensors,
    Gguf,
    PyTorch,
    Custom(String),
}

/// Import options
#[derive(Clone, Debug, Default)]
pub struct ImportOptions {
    pub force_dtype: Option<DataType>,
    pub dequantize: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub buffer_size: Option<u64>,
    pub strict_validation: bool,
}

/// Conversion options
#[derive(Clone, Debug)]
pub struct ConversionOptions {
    pub import_options: ImportOptions,
    pub tile_size_target: u64,
    pub compression: Compression,
    pub create_manifest: bool,
    pub record_provenance: bool,
}

/// Conversion report
#[derive(Clone, Debug)]
pub struct ConversionReport {
    pub source_format: SourceFormat,
    pub tensors_processed: u64,
    pub cells_created: u64,
    pub tiles_created: u64,
    pub tiles_deduplicated: u64,
    pub total_bytes_processed: u64,
    pub total_bytes_stored: u64,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}

/// Validation report
#[derive(Clone, Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub format: SourceFormat,
    pub tensor_count: u64,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Tensor information
#[derive(Clone, Debug)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<u64>,
    pub dtype: String,
    pub size_bytes: u64,
    pub quantized: bool,
}
```

## 6.2 Conversion API Invariants

| ID | Invariant |
|---|---|
| API-CONV-INV-1 | Convert MUST async |
| API-CONV-INV-2 | Convert MUST bounded-memory |
| API-CONV-INV-3 | Convert MUST atomic |
| API-CONV-INV-4 | Convert MUST deterministik |
| API-CONV-INV-5 | Validate MUST tidak memodifikasi source |

---

# 7. Runtime API

## 7.1 Runtime Resolver

```rust
/// Runtime resolver trait
#[async_trait]
pub trait RuntimeResolver: Send + Sync {
    /// Resolve a Cell by semantic ID
    fn resolve_cell(
        &self,
        cell_id: &CellId,
    ) -> Result<CellHandle, CnwsError>;
    
    /// Resolve a Cell by semantic name
    fn resolve_cell_by_name(
        &self,
        name: &str,
    ) -> Result<CellHandle, CnwsError>;
    
    /// Resolve Tiles for a Cell with access policy
    async fn resolve_tiles(
        &self,
        cell: &CellHandle,
        policy: AccessPolicy,
    ) -> Result<Vec<TileHandle>, CnwsError>;
    
    /// Select representation for a Tile
    fn select_representation(
        &self,
        tile: &TileHandle,
        hardware: &HardwareProfile,
        workload: &WorkloadProfile,
    ) -> Result<RepresentationId, CnwsError>;
    
    /// Prefetch Tiles asynchronously
    async fn prefetch(
        &self,
        requests: &[PrefetchRequest],
    ) -> Result<(), CnwsError>;
    
    /// Release a Tile handle
    fn release(&self, tile: TileHandle);
    
    /// Get runtime statistics
    fn stats(&self) -> RuntimeStats;
}

/// Access policy for Tile selection
#[derive(Clone, Debug)]
pub enum AccessPolicy {
    FullCell,
    Range { start: Vec<u64>, end: Vec<u64> },
    TopK { k: usize },
    Predicate { filter: Box<dyn Fn(&TileRef) -> bool + Send + Sync> },
    Custom { selector: Box<dyn TileSelector + Send + Sync> },
}

/// Prefetch request
#[derive(Clone, Debug)]
pub struct PrefetchRequest {
    pub cell_id: CellId,
    pub representation: Option<RepresentationId>,
    pub priority: Priority,
}

/// Priority levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Pinned = 3,
}

/// Hardware profile
#[derive(Clone, Debug)]
pub struct HardwareProfile {
    pub gpu_available: bool,
    pub gpu_vram_bytes: u64,
    pub gpu_fp8_supported: bool,
    pub cpu_ram_bytes: u64,
    pub cpu_simd_level: SimdLevel,
    pub nvme_bandwidth_mbps: u64,
}

/// Workload profile
#[derive(Clone, Debug)]
pub struct WorkloadProfile {
    pub latency_target_us: Option<u64>,
    pub throughput_target_tps: Option<u64>,
    pub batch_size: u32,
    pub sequence_length: u32,
    pub accuracy_policy: AccuracyPolicy,
}

/// Accuracy policy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccuracyPolicy {
    Strict,
    Balanced,
    Fast,
}

/// Runtime statistics
#[derive(Clone, Debug)]
pub struct RuntimeStats {
    pub cells_resolved: u64,
    pub tiles_loaded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub bytes_moved: u64,
    pub active_cells: u64,
    pub active_parameter_ratio: f64,
}
```

## 7.2 Execution API

```rust
/// Execution engine trait
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Execute inference with budget
    async fn execute(
        &self,
        input: &Input,
        budget: ComputeBudget,
    ) -> Result<Output, CnwsError>;
    
    /// Generate tokens autoregressively
    async fn generate(
        &self,
        prompt: &str,
        config: GenerationConfig,
    ) -> Result<TokenStream, CnwsError>;
    
    /// Get execution plan for input
    fn plan(
        &self,
        input: &Input,
        budget: ComputeBudget,
    ) -> Result<ExecutionPlan, CnwsError>;
}

/// Compute budget
#[derive(Clone, Debug)]
pub struct ComputeBudget {
    pub max_flops: u64,
    pub max_bytes_moved: u64,
    pub max_steps: u32,
    pub max_wall_time_us: u64,
}

/// Generation configuration
#[derive(Clone, Debug)]
pub struct GenerationConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_k: u32,
    pub top_p: f32,
    pub budget: ComputeBudget,
    pub stream: bool,
}

/// Token stream (async iterator)
pub struct TokenStream {
    inner: Box<dyn Stream<Item = Result<Token, CnwsError>> + Send + Unpin>,
}

impl TokenStream {
    pub async fn next_token(&mut self) -> Option<Result<Token, CnwsError>>;
    pub fn cancel(&mut self);
}
```

## 7.3 Runtime API Invariants

| ID | Invariant |
|---|---|
| API-RT-INV-1 | resolve_cell MUST O(1) |
| API-RT-INV-2 | resolve_tiles MUST async |
| API-RT-INV-3 | prefetch MUST async dan non-blocking |
| API-RT-INV-4 | release MUST idempotent |
| API-RT-INV-5 | execute MUST menghormati budget |

---

# 8. Revision API

## 8.1 Revision Operations

```rust
/// Revision manager trait
pub trait RevisionManager: Send + Sync {
    // Revision queries
    fn head(&self) -> Result<RevisionHandle, CnwsError>;
    fn get_revision(&self, id: RevisionId) -> Result<RevisionHandle, CnwsError>;
    fn list_revisions(&self) -> Result<Vec<RevisionInfo>, CnwsError>;
    fn active_revision(&self) -> Result<RevisionHandle, CnwsError>;
    
    // Revision creation
    fn commit(
        &self,
        delta: RevisionDelta,
        message: &str,
    ) -> Result<RevisionHandle, CnwsError>;
    
    // Branching
    fn branch(
        &self,
        base: RevisionId,
        name: &str,
    ) -> Result<RevisionHandle, CnwsError>;
    
    // Merging
    fn merge(
        &self,
        a: RevisionId,
        b: RevisionId,
    ) -> Result<RevisionHandle, CnwsError>;
    
    // Rollback
    fn set_active_revision(
        &self,
        revision: RevisionId,
    ) -> Result<(), CnwsError>;
    
    // Resolution
    fn resolve_effective_graph(
        &self,
        revision: RevisionId,
    ) -> Result<EffectiveGraph, CnwsError>;
}

/// Revision information
#[derive(Clone, Debug)]
pub struct RevisionInfo {
    pub id: RevisionId,
    pub revision_number: u64,
    pub parents: Vec<RevisionId>,
    pub created_at: u64,
    pub author: Option<String>,
    pub message: Option<String>,
    pub changed_cells: u64,
    pub changed_tiles: u64,
}

/// Revision delta
#[derive(Clone, Debug)]
pub struct RevisionDelta {
    pub cells_added: Vec<CellId>,
    pub cells_refined: Vec<(CellId, CellId)>,
    pub cells_removed: Vec<CellId>,
    pub memory_added: Vec<MemoryId>,
    pub routing_updated: bool,
    pub compositions_added: Vec<CellId>,
}

/// Effective graph
#[derive(Clone, Debug)]
pub struct EffectiveGraph {
    pub revision: RevisionId,
    pub cell_count: u64,
    pub tile_count: u64,
    pub resolved_at: u64,
}
```

## 8.2 Revision API Invariants

| ID | Invariant |
|---|---|
| API-REV-INV-1 | commit MUST atomic |
| API-REV-INV-2 | branch MUST NOT menyalin Cells |
| API-REV-INV-3 | merge MUST menggunakan three-way merge |
| API-REV-INV-4 | set_active_revision MUST O(1) |
| API-REV-INV-5 | resolve_effective_graph MUST di-cache |

---

# 9. Memory API

## 9.1 Memory Operations

```rust
/// Memory system trait
#[async_trait]
pub trait MemorySystem: Send + Sync {
    // Write operations
    async fn store(
        &self,
        key: Vec<f32>,
        value: Vec<u8>,
        mem_type: MemoryType,
    ) -> Result<MemoryHandle, CnwsError>;
    
    // Read operations
    async fn retrieve(
        &self,
        query: Vec<f32>,
        k: usize,
        config: RetrievalConfig,
    ) -> Result<Vec<MemoryEntry>, CnwsError>;
    
    async fn get(
        &self,
        memory_id: MemoryId,
    ) -> Result<MemoryEntry, CnwsError>;
    
    // Association operations
    async fn associate(
        &self,
        a: MemoryId,
        b: MemoryId,
        strength: f32,
        assoc_type: AssociationType,
    ) -> Result<(), CnwsError>;
    
    async fn traverse_associations(
        &self,
        start: MemoryId,
        depth: usize,
        min_strength: f32,
    ) -> Result<Vec<MemoryEntry>, CnwsError>;
    
    // Consolidation
    async fn consolidate(
        &self,
        memory_ids: &[MemoryId],
    ) -> Result<MemoryHandle, CnwsError>;
    
    // Working memory
    fn working_memory_load(
        &self,
        memory_id: MemoryId,
    ) -> Result<WorkingMemoryEntry, CnwsError>;
    
    fn working_memory_store(
        &self,
        key: Vec<f32>,
        value: Vec<u8>,
    ) -> Result<MemoryId, CnwsError>;
    
    // Statistics
    fn memory_stats(&self) -> MemoryStats;
}

/// Memory type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
}

/// Retrieval configuration
#[derive(Clone, Debug)]
pub struct RetrievalConfig {
    pub threshold: f32,
    pub mem_type: Option<MemoryType>,
    pub include_associations: bool,
    pub association_depth: usize,
    pub time_range: Option<TimeRange>,
    pub domain_filter: Option<String>,
}

/// Memory entry
#[derive(Clone, Debug)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub key_vector: Vec<f32>,
    pub value: Vec<u8>,
    pub created_at: u64,
    pub access_count: u64,
    pub importance_score: f32,
    pub consolidation_level: u8,
}

/// Memory statistics
#[derive(Clone, Debug)]
pub struct MemoryStats {
    pub episodic_count: u64,
    pub semantic_count: u64,
    pub procedural_count: u64,
    pub consolidated_count: u64,
    pub working_memory_used: u64,
    pub working_memory_capacity: u64,
    pub total_bytes: u64,
}
```

## 9.2 Memory API Invariants

| ID | Invariant |
|---|---|
| API-MEM-INV-1 | store MUST async |
| API-MEM-INV-2 | retrieve MUST O(log N) |
| API-MEM-INV-3 | working_memory MUST bounded |
| API-MEM-INV-4 | consolidate MUST menghasilkan memory baru |
| API-MEM-INV-5 | Memory operations MUST update access statistics |

---

# 10. Admin API

## 10.1 Admin Operations

```rust
/// Admin API trait
pub trait AdminApi: Send + Sync {
    // Health
    fn health_check(&self) -> Result<HealthStatus, CnwsError>;
    fn store_status(&self) -> Result<StoreStatus, CnwsError>;
    
    // Maintenance
    async fn rebuild_indexes(&self) -> Result<(), CnwsError>;
    async fn compact_segments(&self) -> Result<CompactionReport, CnwsError>;
    async fn verify_all(&self) -> Result<VerificationReport, CnwsError>;
    
    // Configuration
    fn get_config(&self) -> Result<RuntimeConfig, CnwsError>;
    fn update_config(&self, config: RuntimeConfig) -> Result<(), CnwsError>;
    
    // Diagnostics
    fn get_metrics(&self) -> Result<Metrics, CnwsError>;
    fn get_logs(&self, filter: LogFilter) -> Result<Vec<LogEntry>, CnwsError>;
    
    // Security
    fn get_security_events(&self) -> Result<Vec<SecurityEvent>, CnwsError>;
    fn get_quarantine_list(&self) -> Result<Vec<QuarantineRecord>, CnwsError>;
}

/// Health status
#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub store_accessible: bool,
    pub manifest_valid: bool,
    pub segments_valid: u64,
    pub segments_total: u64,
    pub degraded_mode: bool,
    pub issues: Vec<String>,
}

/// Store status
#[derive(Clone, Debug)]
pub struct StoreStatus {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub cell_count: u64,
    pub tile_count: u64,
    pub segment_count: u64,
    pub revision_count: u64,
    pub active_revision: RevisionId,
    pub gc_needed: bool,
    pub last_gc: Option<u64>,
}
```

---

# 11. Async Semantics

## 11.1 Async Runtime

`[API-ASYNC-1]` CNWS menggunakan **tokio** sebagai async runtime default.

`[API-ASYNC-2]` Async runtime MUST configurable.

```rust
/// Async runtime configuration
#[derive(Clone, Debug)]
pub struct AsyncConfig {
    pub runtime: AsyncRuntime,
    pub worker_threads: usize,
    pub io_threads: usize,
    pub blocking_threads: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncRuntime {
    Tokio,
    AsyncStd,
    Custom,
}
```

## 11.2 Async Patterns

`[API-ASYNC-3]` Async operations MUST menggunakan pattern berikut:

```rust
// Pattern 1: Simple async operation
async fn read_tile(&self, tile_id: TileId) -> Result<TileData, CnwsError>;

// Pattern 2: Async with cancellation
async fn read_tile_with_cancel(
    &self,
    tile_id: TileId,
    cancel: CancellationToken,
) -> Result<TileData, CnwsError>;

// Pattern 3: Async stream
fn token_stream(&self) -> impl Stream<Item = Result<Token, CnwsError>>;

// Pattern 4: Async with progress
async fn convert_with_progress(
    &self,
    source: &SourceDescriptor,
    target: &Path,
    progress: mpsc::Sender<ProgressUpdate>,
) -> Result<ConversionReport, CnwsError>;
```

## 11.3 Cancellation

`[API-ASYNC-4]` Long-running operations MUST mendukung cancellation.

```rust
/// Cancellation token
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<tokio::sync::Notify>,
}

impl CancellationToken {
    pub fn new() -> Self;
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
    pub async fn cancelled(&self);
}
```

## 11.4 Timeout

`[API-ASYNC-5]` Async operations MUST memiliki timeout.

```rust
/// Timeout configuration
#[derive(Clone, Copy, Debug)]
pub struct TimeoutConfig {
    pub default_timeout_ms: u64,
    pub io_timeout_ms: u64,
    pub network_timeout_ms: u64,
}

// Usage with timeout
let result = tokio::time::timeout(
    Duration::from_millis(config.io_timeout_ms),
    store.read_tile(tile_id),
).await??;
```

## 11.5 Async Invariants

| ID | Invariant |
|---|---|
| API-ASYNC-INV-1 | I/O operations MUST async |
| API-ASYNC-INV-2 | Long-running operations MUST mendukung cancellation |
| API-ASYNC-INV-3 | Async operations MUST memiliki timeout |
| API-ASYNC-INV-4 | Async operations MUST NOT blocking |
| API-ASYNC-INV-5 | Progress reporting MUST tersedia untuk long operations |

---

# 12. Request/Response Schema

## 12.1 RPC Protocol (Optional)

`[API-RPC-1]` CNWS MAY menyediakan RPC interface untuk remote access.

`[API-RPC-2]` RPC protocol default: **gRPC**.

## 12.2 gRPC Service Definition

```protobuf
syntax = "proto3";

package cnws.v1;

// Storage service
service StorageService {
    rpc OpenStore(OpenStoreRequest) returns (OpenStoreResponse);
    rpc CloseStore(CloseStoreRequest) returns (CloseStoreResponse);
    rpc LookupTile(LookupTileRequest) returns (LookupTileResponse);
    rpc ReadTile(ReadTileRequest) returns (stream ReadTileResponse);
    rpc Gc(GcRequest) returns (GcResponse);
    rpc VerifyIntegrity(VerifyIntegrityRequest) returns (VerifyIntegrityResponse);
}

// Runtime service
service RuntimeService {
    rpc ResolveCell(ResolveCellRequest) returns (ResolveCellResponse);
    rpc ResolveTiles(ResolveTilesRequest) returns (stream ResolveTilesResponse);
    rpc Prefetch(PrefetchRequest) returns (PrefetchResponse);
    rpc Execute(ExecuteRequest) returns (stream ExecuteResponse);
    rpc Generate(GenerateRequest) returns (stream GenerateResponse);
}

// Revision service
service RevisionService {
    rpc GetRevision(GetRevisionRequest) returns (GetRevisionResponse);
    rpc ListRevisions(ListRevisionsRequest) returns (ListRevisionsResponse);
    rpc Commit(CommitRequest) returns (CommitResponse);
    rpc Branch(BranchRequest) returns (BranchResponse);
    rpc Merge(MergeRequest) returns (MergeResponse);
    rpc Rollback(RollbackRequest) returns (RollbackResponse);
}

// Memory service
service MemoryService {
    rpc Store(StoreMemoryRequest) returns (StoreMemoryResponse);
    rpc Retrieve(RetrieveMemoryRequest) returns (RetrieveMemoryResponse);
    rpc Consolidate(ConsolidateRequest) returns (ConsolidateResponse);
}

// Admin service
service AdminService {
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
    rpc GetStatus(GetStatusRequest) returns (GetStatusResponse);
    rpc RebuildIndexes(RebuildIndexesRequest) returns (RebuildIndexesResponse);
}
```

## 12.3 Message Schemas

```protobuf
// Common types
message Blake3Hash {
    bytes value = 1;  // 32 bytes
}

message CellId {
    Blake3Hash hash = 1;
}

message TileId {
    Blake3Hash hash = 1;
}

message RevisionId {
    Blake3Hash hash = 1;
}

// Storage messages
message OpenStoreRequest {
    string path = 1;
    bool create_if_missing = 2;
    bool read_only = 3;
    CacheConfig cache_config = 4;
}

message OpenStoreResponse {
    string store_handle = 1;
    StoreInfo info = 2;
}

message ReadTileRequest {
    string store_handle = 1;
    TileId tile_id = 2;
    bool verify = 3;
}

message ReadTileResponse {
    bytes data = 1;
    TileMetadata metadata = 2;
}

// Runtime messages
message ResolveCellRequest {
    string store_handle = 1;
    string cell_name = 2;
}

message ResolveCellResponse {
    CellId cell_id = 1;
    CellMetadata metadata = 2;
}

message ExecuteRequest {
    string store_handle = 1;
    bytes input = 2;
    ComputeBudget budget = 3;
}

message ExecuteResponse {
    oneof result {
        bytes output = 1;
        ProgressUpdate progress = 2;
        Error error = 3;
    }
}

// Error message
message Error {
    int32 code = 1;
    string message = 2;
    bool recoverable = 3;
    bool retryable = 4;
    map<string, string> details = 5;
}
```

## 12.4 RPC Invariants

| ID | Invariant |
|---|---|
| API-RPC-INV-1 | RPC MUST menggunakan protobuf |
| API-RPC-INV-2 | RPC MUST memiliki timeout |
| API-RPC-INV-3 | RPC errors MUST menggunakan Error message |
| API-RPC-INV-4 | Streaming MUST didukung untuk large responses |
| API-RPC-INV-5 | RPC MUST versioned (cnws.v1, cnws.v2, ...) |

---

# 13. API Lifecycle

## 13.1 Store Lifecycle

```text
┌──────────┐   open    ┌──────────┐   use     ┌──────────┐
│ CLOSED   │─────────►│  OPEN    │─────────►│  ACTIVE  │
└──────────┘          └──────────┘          └────┬─────┘
                                                  │
                                                  │ close
                                                  ▼
                                            ┌──────────┐
                                            │ CLOSING  │
                                            └────┬─────┘
                                                 │
                                                 ▼
                                            ┌──────────┐
                                            │ CLOSED   │
                                            └──────────┘
```

## 13.2 Handle Lifecycle

`[API-LIFE-1]` Handle MUST mengikuti RAII pattern.

```rust
// Handle automatically releases resources when dropped
{
    let store = StorageEngine::open(&config)?;
    let cell = store.resolve_cell(&cell_id)?;
    let tile = store.read_tile(tile_id).await?;
    
    // Use tile...
    
} // tile, cell, store dropped here, resources released
```

## 13.3 Lifecycle Invariants

| ID | Invariant |
|---|---|
| API-LIFE-INV-1 | Handle MUST RAII |
| API-LIFE-INV-2 | Store close MUST flush pending writes |
| API-LIFE-INV-3 | Handle use after close MUST error |
| API-LIFE-INV-4 | Double close MUST idempotent |

---

# 14. Compatibility & Versioning

## 14.1 API Versioning

`[API-VER-1]` API version mengikuti semver.

`[API-VER-2]` Version format: `major.minor.patch`

| Change Type | Version Bump | Example |
|---|---|---|
| Breaking change | major | 1.0.0 → 2.0.0 |
| New feature (backward compatible) | minor | 1.0.0 → 1.1.0 |
| Bug fix | patch | 1.0.0 → 1.0.1 |

## 14.2 Backward Compatibility

`[API-VER-3]` Minor version additions MUST backward compatible.

`[API-VER-4]` Breaking changes MUST major version bump.

`[API-VER-5]` Breaking changes mencakup:
- Menghapus method dari trait
- Mengubah signature method
- Mengubah semantics method
- Mengubah type dari field public

## 14.3 Deprecation Policy

`[API-VER-6]` Deprecation MUST memberikan minimum 2 minor versions notice.

```rust
#[deprecated(since = "1.2.0", note = "Use resolve_cell_by_name instead")]
fn resolve_cell_by_id(&self, id: &str) -> Result<CellHandle, CnwsError>;
```

## 14.4 Version Negotiation

`[API-VER-7]` RPC MUST mendukung version negotiation.

```rust
// Client specifies API version
let client = CnwsClient::connect("localhost:50051")
    .api_version("1.0")
    .build()?;

// Server checks compatibility
if !server.supports_version(client.api_version()) {
    return Err(CnwsError::Version(VersionError::Unsupported {
        version: client.api_version().to_string(),
    }));
}
```

## 14.5 Compatibility Invariants

| ID | Invariant |
|---|---|
| API-VER-INV-1 | API version MUST semver |
| API-VER-INV-2 | Minor additions MUST backward compatible |
| API-VER-INV-3 | Breaking changes MUST major bump |
| API-VER-INV-4 | Deprecation MUST 2 minor versions notice |
| API-VER-INV-5 | RPC MUST version negotiation |

---

# 15. SDK Boundary

## 15.1 SDK Definition

`[API-SDK-1]` Primary SDK adalah Rust crate `cnws`.

`[API-SDK-2]` SDK menyediakan:
- High-level API wrappers
- Builder patterns
- Helper functions
- Type conversions
- Error handling utilities

## 15.2 SDK Structure

```text
cnws/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── types.rs            # Core types
│   ├── error.rs            # Error types
│   ├── storage.rs          # Storage API
│   ├── conversion.rs       # Conversion API
│   ├── runtime.rs          # Runtime API
│   ├── revision.rs         # Revision API
│   ├── memory.rs           # Memory API
│   ├── admin.rs            # Admin API
│   ├── builders.rs         # Builder patterns
│   └── prelude.rs          # Common imports
└── tests/
    ├── integration/
    └── conformance/
```

## 15.3 SDK Public API

```rust
// cnws/src/lib.rs

pub mod types;
pub mod error;
pub mod storage;
pub mod conversion;
pub mod runtime;
pub mod revision;
pub mod memory;
pub mod admin;
pub mod builders;
pub mod prelude;

// Re-exports for convenience
pub use types::*;
pub use error::CnwsError;
pub use storage::StorageEngine;
pub use conversion::ConversionPipeline;
pub use runtime::RuntimeResolver;
pub use revision::RevisionManager;
pub use memory::MemorySystem;
pub use admin::AdminApi;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported format version
pub const FORMAT_VERSION: &str = "1.0.0";
```

## 15.4 SDK Usage Example

```rust
use cnws::prelude::*;

#[tokio::main]
async fn main() -> Result<(), CnwsError> {
    // Open store
    let config = StoreConfigBuilder::new("model.cd")
        .gpu_budget(16 * GB)
        .build()?;
    
    let store = StorageEngine::open(&config)?;
    
    // Resolve a Cell
    let cell = store.resolve_cell_by_name("model.layer.0.self_attn.q_proj")?;
    
    // Load Tiles
    let tiles = store.resolve_tiles(&cell, AccessPolicy::FullCell).await?;
    
    // Use Tiles...
    
    // Release
    for tile in tiles {
        store.release(tile);
    }
    
    // Close
    store.close()?;
    
    Ok(())
}
```

## 15.5 SDK Boundary Rules

`[API-SDK-3]` SDK MUST NOT mengekspos internal implementation details.

`[API-SDK-4]` SDK MUST stabil sebelum 1.0.0.

`[API-SDK-5]` SDK MUST memiliki documentation lengkap.

`[API-SDK-6]` SDK MUST memiliki conformance tests.

## 15.6 Language Bindings

`[API-SDK-7]` Language bindings MAY disediakan untuk:
- Python (via PyO3)
- C/C++ (via FFI)
- JavaScript/TypeScript (via WASM atau NAPI)

`[API-SDK-8]` Bindings MUST menggunakan core Rust SDK.

---

# 16. API Testing

## 16.1 Testing Requirements

`[API-TEST-1]` API MUST memiliki:
- Unit tests
- Integration tests
- Conformance tests
- Async tests
- Error handling tests

## 16.2 Conformance Tests

```rust
#[test]
fn test_store_open_close() {
    let config = StoreConfigBuilder::new("test.cd")
        .create_if_missing(true)
        .build()
        .unwrap();
    
    let store = StorageEngine::open(&config).unwrap();
    assert!(store.store_info().is_ok());
    store.close().unwrap();
}

#[tokio::test]
async fn test_tile_read_verified() {
    let store = open_test_store().await;
    let tile_id = create_test_tile(&store).await;
    
    let result = store.read_tile_verified(tile_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_corrupted_tile_detection() {
    let store = open_test_store().await;
    let tile_id = create_corrupted_tile(&store).await;
    
    let result = store.read_tile_verified(tile_id).await;
    assert!(matches!(
        result,
        Err(CnwsError::Integrity(IntegrityError::TileCorrupted { .. }))
    ));
}
```

---

# 17. Final API Contract

## 17.1 Ringkasan Keputusan API

| ID | Keputusan |
|---|---|
| API-F01 | API versioning menggunakan semver. |
| API-F02 | Error type menggunakan CnwsError enum hierarchy. |
| API-F03 | Setiap error memiliki stable error code. |
| API-F04 | Async runtime default: tokio. |
| API-F05 | SDK primary: Rust crate `cnws`. |
| API-F06 | RPC optional: gRPC. |
| API-F07 | Backward compatibility: minor additions only. |
| API-F08 | Breaking changes: major version bump. |
| API-F09 | Deprecation: minimum 2 minor versions notice. |
| API-F10 | Handle-based resource management. |
| API-F11 | Builder pattern untuk configuration. |
| API-F12 | Result<T, CnwsError> untuk semua fallible operations. |
| API-F13 | I/O operations async. |
| API-F14 | Long-running operations mendukung cancellation. |
| API-F15 | Async operations memiliki timeout. |
| API-F16 | RPC menggunakan protobuf. |
| API-F17 | RPC versioned (cnws.v1, ...). |
| API-F18 | Handle mengikuti RAII pattern. |
| API-F19 | SDK tidak mengekspos internal details. |
| API-F20 | API memiliki conformance tests. |

## 17.2 API Invariants

| ID | Invariant |
|---|---|
| API-INV-1 | Semua fallible operations MUST mengembalikan Result<T, CnwsError>. |
| API-INV-2 | Setiap error MUST memiliki stable error code. |
| API-INV-3 | I/O operations MUST async. |
| API-INV-4 | Long-running operations MUST mendukung cancellation. |
| API-INV-5 | Async operations MUST memiliki timeout. |
| API-INV-6 | Handle MUST RAII. |
| API-INV-7 | API version MUST semver. |
| API-INV-8 | Minor additions MUST backward compatible. |
| API-INV-9 | Breaking changes MUST major bump. |
| API-INV-10 | Deprecation MUST 2 minor versions notice. |
| API-INV-11 | SDK MUST NOT mengekspos internal details. |
| API-INV-12 | API MUST memiliki conformance tests. |
| API-INV-13 | Store open MUST idempotent. |
| API-INV-14 | Tile read MUST async. |
| API-INV-15 | GC MUST NOT menghapus Tile reachable. |
| API-INV-16 | resolve_cell MUST O(1). |
| API-INV-17 | Memory retrieve MUST O(log N). |
| API-INV-18 | Revision commit MUST atomic. |
| API-INV-19 | Branch MUST NOT menyalin Cells. |
| API-INV-20 | RPC MUST versioned. |

## 17.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi API & Protocol final dan mengikat** untuk CNWS. Ia mendefinisikan API yang benar-benar dipakai, dari trait definitions hingga error semantics, dari async patterns hingga versioning, dari SDK boundary hingga RPC protocol.

Seluruh implementasi API layer, SDK, CLI, RPC server, dan language bindings CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan API yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN API & PROTOCOL SPECIFICATION**
