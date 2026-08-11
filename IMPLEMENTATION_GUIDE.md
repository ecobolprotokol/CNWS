# CNWS Complete End-to-End Implementation Guide

**Authority:** Specifications in `docs/specs/` (Engineering Contract + 17 normative specifications)  
**Date:** 2026-08-11  
**Status:** Active Implementation Plan  
**Target:** Byte-for-byte compliant with CNWS Engineering Contract  

---

## I. Executive Overview

CNWS is a **canonical intelligence infrastructure** that unifies knowledge representation, computation, memory, routing, and state through **content-addressed Cells**.

### Three Core Capabilities

1. **Conversion**: Import LLM checkpoints (Safetensors, GGUF, PyTorch) → canonical `.cd` format
2. **Runtime**: Selective, adaptive, bounded-memory weight loading for inference
3. **Versioning**: Incremental revision DAG with branching, merging, rollback

### Binding Architecture

```
EXTERNAL CHECKPOINTS     CNWS SUBSTRATE          CNWS LATTICE            EXECUTION
(Safetensors, GGUF)  →  (Storage, Versioning) →  (Runtime, Memory)  →  (CPU/GPU/NVMe)
                        ↑ Immutable storage      ↑ Adaptive execution    
                        ↑ Content-addressed      ↑ Persistent memory    
```

### Engineering Contract Final Decisions (Binding)

| Decision | Value | Specification |
|----------|-------|---|
| **DF-01** | Product name | **CNWS** |
| **DF-02** | Universal unit | **Cell** (knowledge + computation) |
| **DF-03** | Storage unit | **Tile** (immutable, deduplicable) |
| **DF-04** | Content addressing | **BLAKE3-256** (32 bytes) |
| **DF-05** | Canonical store | **`.cd`** directory with `MANIFEST.cd` |
| **DF-06** | Store contents | Weight, memory, routing, composition, provenance |
| **DF-07** | Versioning | **Revision DAG** (immutable snapshots) |
| **DF-08** | Import method | **Streaming-first pipeline** (bounded memory) |
| **DF-09** | Runtime | **CNWS Execution Engine** (dynamic, adaptive) |
| **DF-10** | Memory | **First-class persistent state** (not cache) |
| **DF-11** | Compute | **Adaptive allocation** (by difficulty) |
| **DF-12** | Learning | **Structural + incremental** (not global updates) |
| **DF-13** | Format coupling | **Zero** (runtime format-independent) |

---

## II. How to Implement CNWS Correctly

### Step 1: Use Specifications as Single Source of Truth

**Before writing any code**, read the relevant specification:

| Phase | Must Read | Should Read |
|-------|-----------|-------------|
| Foundation | 01-engineering-contract.md, 05-cell-schema.md | 02-product-requirements.md |
| Substrate | 04-cd-format-serialization.md | 11-reliability-recovery.md |
| Conversion | 07-conversion-import.md | 13-testing-conformance.md |
| Lattice | 06-runtime-execution.md, 09-memory-retrieval.md | 14-performance-benchmark.md |
| API | 12-api-protocol.md | 15-observability.md |
| Operations | 16-operations-deployment.md | 17-compatibility-migration.md |

### Step 2: Implement by Phase (Not Random Order)

**Phase 1 (Foundation)** → **Phase 2 (Substrate)** → **Phase 3 (Lattice)** → **Phase 4+ (API, Testing, Ops)**

Each phase MUST compile and pass tests before proceeding.

### Step 3: Follow Implementation Patterns

**For each component:**

1. **Read the specification** - Understand the normative requirements
2. **Define types** - Create Rust structs that match the spec
3. **Implement core logic** - Implement the behavioral requirements
4. **Write tests** - Create conformance tests based on spec examples
5. **Document compliance** - Add comments linking code to spec sections

---

## III. Phase-by-Phase Implementation Roadmap

### PHASE 1: Foundation & Core Types

**Goal:** Establish the type system as defined in Cell & Schema Specification (05)

**Specification Reference:** 05-cell-schema.md §1-3

#### 1.1 Blake3Hash Content Addressing

**What it is:** Universal content identifier (32-byte BLAKE3-256 hash)

**From spec (04-cd-format-serialization.md §0.3):**
- DF-CD-06: BLAKE3-256 (32 bytes)
- DF-CD-07: Text representation `b3:` + 64 lowercase hex

**Implementation:**

```rust
// types.rs
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    pub fn hash(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(hash.into())
    }
    
    pub fn to_b3_string(&self) -> String {
        format!("b3:{}", hex::encode(self.0))
    }
    
    pub fn from_b3_string(s: &str) -> Result<Self> {
        if !s.starts_with("b3:") {
            return Err(CnwsError::InvalidInput("Invalid b3 format".into()));
        }
        let hex_part = &s[3..];
        let bytes = hex::decode(hex_part)?;
        // ... validate and construct
    }
}
```

**Tests needed:**
- [ ] Hash empty data
- [ ] Hash file content (streaming)
- [ ] Serialization round-trip
- [ ] B3 string encoding/decoding

#### 1.2 Cell Type System (35 Types)

**From spec (05-cell-schema.md §2.1):**

35 fundamental Cell types organized by category:

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CellType {
    // Core computation (0x01-0x08)
    Tensor = 0x01,
    Attention = 0x02,
    FFN = 0x03,
    LayerNorm = 0x04,
    Embedding = 0x05,
    Loss = 0x06,
    OptimizerState = 0x07,
    Gradient = 0x08,
    
    // ... remaining 27 types (see spec)
}

impl TryFrom<u8> for CellType {
    type Error = CnwsError;
    fn try_from(value: u8) -> Result<Self> { /* ... */ }
}
```

**Tests needed:**
- [ ] All 35 types have correct discriminants
- [ ] Round-trip u8 ↔ CellType conversion
- [ ] Serialization preserves discriminant

#### 1.3 Data Type System (13 Types)

**From spec (05-cell-schema.md §3.2):**

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum DataType {
    F32 = 0x01,
    F16 = 0x02,
    I32 = 0x03,
    // ... 10 more types (see spec §3.2)
}
```

#### 1.4 Core Data Structures

**Cell Structure** (from spec 05-cell-schema.md §4):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Content-addressed identity
    pub hash: Blake3Hash,
    
    /// Cell semantic type
    pub cell_type: CellType,
    
    /// Immutable payload
    pub payload: Vec<u8>,
    
    /// Data type of payload (if tensor)
    pub data_type: Option<DataType>,
    
    /// Shape (if tensor)
    pub shape: Option<Vec<u64>>,
    
    /// Index vector (512-dim default, for similarity search)
    pub index_vector: Option<Vec<f32>>,
    
    /// Dependencies (backward edges to other Cells)
    pub dependencies: Vec<CellDependency>,
    
    /// Metadata (extensible attributes)
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Version (semver)
    pub version: Version,
}
```

**Tile Structure** (from spec 04-cd-format-serialization.md §2):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    /// Content hash (BLAKE3-256)
    pub hash: Blake3Hash,
    
    /// Payload size (bytes)
    pub size: u64,
    
    /// Offset in segment
    pub offset: u64,
    
    /// Segment ID
    pub segment_id: u32,
    
    /// Compression format
    pub compression: Compression,
}
```

**TileRef & CellRef:**

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TileRef {
    pub hash: Blake3Hash,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellRef {
    pub hash: Blake3Hash,
    pub dependency_type: DependencyType,
}
```

**CellDependency Types** (from spec):

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum DependencyType {
    Data,                    // Data flow
    Control,                 // Control flow  
    ExecutionOrder,          // Execution ordering
    PrefetchHint,            // Prefetch suggestion
}

#[derive(Debug, Clone)]
pub struct CellDependency {
    pub cell_hash: Blake3Hash,
    pub dependency_type: DependencyType,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### 1.5 Compression Enum

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Compression {
    None = 0x00,
    Zstd = 0x01,
    LZ4 = 0x02,
    Brotli = 0x03,
    Deflate = 0x04,
}
```

#### 1.6 Index Vector Type

```rust
pub struct IndexVector {
    /// Default 512 dimensions (from spec)
    pub embeddings: Vec<f32>,
    
    /// Similarity metric
    pub metric: SimilarityMetric,
}

pub enum SimilarityMetric {
    Cosine,        // Default
    Euclidean,
    Dot,
}

impl IndexVector {
    /// Compute cosine similarity (default metric)
    pub fn similarity(&self, other: &IndexVector) -> f32 { /* ... */ }
}
```

#### 1.7 Version & Constants

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

// Magic bytes (from spec DF-CD-05)
pub const SUPERBLOCK_MAGIC: &[u8; 8] = b"CNWSSB01";
pub const SEGMENT_MAGIC: &[u8; 8] = b"CNWSSEG1";
pub const INDEX_MAGIC: &[u8; 8] = b"CNWSIDX1";
pub const MEMORY_MAGIC: &[u8; 8] = b"CNWSMEM1";
pub const ROUTING_MAGIC: &[u8; 8] = b"CNWSRTG1";
pub const COMPOSITION_MAGIC: &[u8; 8] = b"CNWSCMP1";
pub const PROVENANCE_MAGIC: &[u8; 8] = b"CNWSPRV1";

// Sizes (from spec DF-CD)
pub const SUPERBLOCK_SIZE: usize = 4096;           // DF-CD-02
pub const SEGMENT_HEADER_SIZE: usize = 4096;       // DF-CD-03
pub const TILE_SIZE_MIN: usize = 32 * 1024 * 1024; // 32 MiB min
pub const TILE_SIZE_DEFAULT: usize = 128 * 1024 * 1024; // 128 MiB default
pub const TILE_SIZE_MAX: usize = 256 * 1024 * 1024; // 256 MiB max
pub const SEGMENT_TARGET_SIZE: u64 = 32 * 1024 * 1024 * 1024; // 32 GiB
pub const ALIGNMENT_MIN: usize = 4 * 1024;         // 4 KiB
pub const ALIGNMENT_PREFERRED: usize = 64 * 1024;  // 64 KiB
```

---

### PHASE 2: CNWS Substrate Layer

**Goal:** Implement immutable storage, versioning, integrity, recovery

**Key Specification:** 04-cd-format-serialization.md, 11-reliability-recovery.md

#### 2.1 `.cd` Store Layout

**Directory structure** (spec 04-cd-format-serialization.md §1.1):

```
model.cd/
├── SUPERBLOCK                 # 4096 bytes, root metadata
├── LOCK                       # advisory file lock
├── MANIFEST.cd                # canonical JSON manifest (source of truth)
├── MANIFEST.cd.prev           # previous committed manifest
├── journal/
│   └── commit.wal             # write-ahead log
├── staging/
│   └── manifest-<hash>.cd     # staged manifests during commit
├── index/
│   ├── cells.idx              # Cell registry (binary)
│   ├── tiles.idx              # Tile registry (binary)
│   ├── memory.idx             # Memory entry index (binary)
│   └── routing.idx            # Routing statistics (binary)
├── segments/
│   ├── segment-000001.cd      # Tile payload (immutable)
│   ├── segment-000002.cd
│   └── ...
├── lattice/
│   ├── graph.cd               # Cell Graph structure
│   ├── compositions.cd        # Composition patterns
│   └── routing_policy.cd      # Routing policy
├── memory/
│   ├── episodic/
│   │   └── segment-000001.mcd # Episodic memory
│   ├── semantic/
│   │   └── segment-000001.mcd # Semantic memory
│   └── procedural/
│       └── segment-000001.mcd # Procedural memory
└── provenance/
    └── provenance.cd          # Model provenance tracking
```

#### 2.2 Superblock (4096 bytes)

**From spec 04-cd-format-serialization.md §2.1:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Superblock {
    /// Magic: "CNWSSB01"
    pub magic: [u8; 8],
    
    /// Version (major, minor, patch)
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    
    /// Timestamp of creation
    pub created_at: u64,
    
    /// Timestamp of last update
    pub updated_at: u64,
    
    /// Hash of MANIFEST.cd (current state)
    pub manifest_hash: Blake3Hash,
    
    /// Hash of previous manifest (for recovery)
    pub prev_manifest_hash: Option<Blake3Hash>,
    
    /// Total Tiles in store
    pub tile_count: u64,
    
    /// Total bytes stored
    pub total_size: u64,
    
    /// Segment count
    pub segment_count: u32,
    
    /// Metadata (extensible)
    pub metadata: [u8; 4052], // Rest of 4096 bytes for metadata
}

impl Superblock {
    /// Serialize to fixed 4096 bytes (little-endian)
    pub fn to_bytes(&self) -> [u8; SUPERBLOCK_SIZE];
    
    /// Deserialize from 4096 bytes
    pub fn from_bytes(buf: &[u8; SUPERBLOCK_SIZE]) -> Result<Self>;
    
    /// Validate magic bytes
    pub fn validate_magic(&self) -> Result<()>;
}
```

#### 2.3 Segment Header (4096 bytes)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentHeader {
    /// Magic: "CNWSSEG1"
    pub magic: [u8; 8],
    
    /// Segment ID (sequential)
    pub segment_id: u32,
    
    /// Segment type (data, index, memory, etc.)
    pub segment_type: u32,
    
    /// Offset in file where Tile data begins
    pub data_start: u64,
    
    /// Current size of Tile data
    pub current_size: u64,
    
    /// Maximum segment size (default 32 GiB)
    pub max_size: u64,
    
    /// Number of Tiles in this segment
    pub tile_count: u32,
    
    /// Compression format for Tiles in this segment
    pub compression: u8,
    
    /// Index table (offset, size) for Tiles
    pub tile_index: Vec<(u64, u32)>, // Serialized in remainder
}

impl SegmentHeader {
    pub fn to_bytes(&self) -> [u8; SEGMENT_HEADER_SIZE];
    pub fn from_bytes(buf: &[u8; SEGMENT_HEADER_SIZE]) -> Result<Self>;
}
```

#### 2.4 Tile Registry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRegistry {
    /// Hash -> TileLocation mapping
    tiles: HashMap<Blake3Hash, TileLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLocation {
    /// Which segment stores this Tile
    pub segment_id: u32,
    
    /// Offset within segment
    pub offset: u64,
    
    /// Uncompressed size
    pub uncompressed_size: u64,
    
    /// Compressed size (if compressed)
    pub compressed_size: u64,
    
    /// Compression format
    pub compression: Compression,
    
    /// Timestamp added
    pub added_at: u64,
    
    /// Reference count (for GC)
    pub ref_count: u32,
}

impl TileRegistry {
    pub fn get(&self, hash: &Blake3Hash) -> Option<&TileLocation>;
    pub fn put(&mut self, hash: Blake3Hash, location: TileLocation);
    pub fn delete(&mut self, hash: &Blake3Hash) -> Option<TileLocation>;
    pub fn load_from_file(path: &Path) -> Result<Self>;
    pub fn save_to_file(&self, path: &Path) -> Result<()>;
}
```

#### 2.5 Manifest Authority

**Source of truth for current `.cd` state** (spec 04-cd-format-serialization.md §2.2)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest schema version
    pub version: Version,
    
    /// Current revision ID
    pub current_revision: Blake3Hash,
    
    /// Cell registry (hash -> metadata)
    pub cells: HashMap<Blake3Hash, CellMetadata>,
    
    /// Tile registry
    pub tiles: HashMap<Blake3Hash, TileLocation>,
    
    /// Memory entries registry
    pub memory_entries: HashMap<Blake3Hash, MemoryEntryMetadata>,
    
    /// Routing policy reference
    pub routing_policy_hash: Option<Blake3Hash>,
    
    /// Composition patterns reference
    pub compositions_hash: Option<Blake3Hash>,
    
    /// Provenance information
    pub provenance: ProvenanceInfo,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Timestamp
    pub created_at: u64,
    pub updated_at: u64,
    
    /// Canonical representation hash (for integrity verification)
    pub manifest_hash: Option<Blake3Hash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub cell_type: CellType,
    pub hash: Blake3Hash,
    pub tile_refs: Vec<TileRef>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Manifest {
    /// Canonicalize to JSON (for hashing)
    pub fn to_canonical_json(&self) -> Result<String>;
    
    /// Compute BLAKE3-256 of canonical JSON
    pub fn compute_hash(&mut self) -> Blake3Hash;
    
    /// Load from MANIFEST.cd
    pub fn load(path: &Path) -> Result<Self>;
    
    /// Save to MANIFEST.cd (atomic)
    pub fn save(&self, path: &Path) -> Result<()>;
}
```

#### 2.6 Storage Engine

Main interface for read/write operations:

```rust
#[derive(Clone)]
pub struct StorageEngine {
    store_path: PathBuf,
    manifest: Arc<RwLock<Manifest>>,
    superblock: Arc<RwLock<Superblock>>,
    tile_registry: Arc<RwLock<TileRegistry>>,
    lock: Arc<parking_lot::RwLock<()>>,
}

impl StorageEngine {
    /// Open or create a `.cd` store
    pub async fn open(path: impl AsRef<Path>) -> Result<Self>;
    
    /// Get a Cell by hash
    pub async fn get_cell(&self, hash: Blake3Hash) -> Result<Cell>;
    
    /// Store a new Cell (with Tile deduplication)
    pub async fn put_cell(&self, cell: Cell) -> Result<()>;
    
    /// Get Tile data by hash
    pub async fn get_tile(&self, hash: Blake3Hash) -> Result<Vec<u8>>;
    
    /// Store Tile data (deduplicate if exists)
    pub async fn put_tile(&self, tile: &Tile, data: &[u8]) -> Result<Blake3Hash>;
    
    /// Get current manifest
    pub fn get_manifest(&self) -> Result<Manifest>;
    
    /// Commit changes to manifest (atomic)
    pub async fn commit(&self, changes: ManifestDelta) -> Result<()>;
    
    /// Get statistics
    pub fn get_stats(&self) -> Result<StoreStats>;
    
    /// Verify integrity of all Tiles
    pub async fn verify_integrity(&self) -> Result<VerificationReport>;
}
```

#### 2.7 Integrity Verification System

**From spec 11-reliability-recovery.md:**

```rust
pub struct IntegrityVerifier;

impl IntegrityVerifier {
    /// Verify hash of Tile data
    pub async fn verify_tile(&self, hash: Blake3Hash, data: &[u8]) -> Result<()>;
    
    /// Verify manifest hash
    pub async fn verify_manifest(&self, manifest: &Manifest) -> Result<()>;
    
    /// Verify Cell dependencies
    pub async fn verify_cell_dependencies(&self, cell: &Cell) -> Result<()>;
    
    /// Full store integrity check
    pub async fn verify_store(&self, store: &StorageEngine) -> Result<VerificationReport>;
}

pub struct VerificationReport {
    pub total_tiles_checked: u64,
    pub tiles_valid: u64,
    pub tiles_corrupted: u64,
    pub corrupted_hashes: Vec<Blake3Hash>,
    pub errors: Vec<String>,
}
```

#### 2.8 Revision DAG

**From spec 08-revision-learning.md:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Revision ID (hash of content)
    pub id: Blake3Hash,
    
    /// Parent revisions (may have multiple for merges)
    pub parents: Vec<Blake3Hash>,
    
    /// Cells added in this revision
    pub cells_added: HashMap<Blake3Hash, Cell>,
    
    /// Cells modified in this revision
    pub cells_modified: HashMap<Blake3Hash, Cell>,
    
    /// Cells removed in this revision
    pub cells_removed: Vec<Blake3Hash>,
    
    /// Metadata (author, message, timestamp, tags)
    pub metadata: RevisionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionMetadata {
    pub author: String,
    pub message: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

pub struct RevisionDAG {
    revisions: HashMap<Blake3Hash, Revision>,
}

impl RevisionDAG {
    /// Get revision by ID
    pub fn get(&self, id: Blake3Hash) -> Option<&Revision>;
    
    /// Create new revision
    pub fn create(&mut self, parents: Vec<Blake3Hash>, delta: RevisionDelta) -> Result<Blake3Hash>;
    
    /// Get all parents of a revision
    pub fn get_parents(&self, id: Blake3Hash) -> Vec<Blake3Hash>;
    
    /// Check if rev_a is ancestor of rev_b
    pub fn is_ancestor(&self, rev_a: Blake3Hash, rev_b: Blake3Hash) -> bool;
    
    /// Find common ancestor of two revisions
    pub fn common_ancestor(&self, rev_a: Blake3Hash, rev_b: Blake3Hash) -> Option<Blake3Hash>;
}
```

#### 2.9 Conversion Pipeline

**From spec 07-conversion-import.md - streaming-first import**

```rust
pub struct ConversionPipeline;

impl ConversionPipeline {
    /// Import checkpoint from path
    /// Supports: Safetensors, GGUF, PyTorch, custom
    pub async fn import(
        path: &Path,
        format: CheckpointFormat,
        output_store: &StorageEngine,
    ) -> Result<ImportReport>;
}

pub enum CheckpointFormat {
    Safetensors,
    GGUF,
    PyTorch,
    Custom(String),
}

pub struct ImportReport {
    pub total_cells: u64,
    pub total_bytes: u64,
    pub conversion_time_ms: u64,
    pub peak_ram_bytes: u64,
    pub deduped_tiles: u64,
    pub new_tiles: u64,
}

// Pipeline stages
struct FormatReader {
    /* Detect and parse checkpoint format */
}

struct Normalizer {
    /* Convert format tensors to semantic Cells */
}

struct Planner {
    /* Create streaming plan (chunking, tiling) */
}

struct Hasher {
    /* Compute BLAKE3-256 per Tile */
}

struct Deduplicator {
    /* Check against existing Tiles */
}

struct SegmentWriter {
    /* Write to .cd segments */
}

struct CommitManager {
    /* Atomic manifest update */
}
```

#### 2.10 Recovery System (WAL)

**From spec 11-reliability-recovery.md:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecordType {
    Write(Blake3Hash),      // Tile write
    Commit(Blake3Hash),     // Manifest commit
    Checkpoint,             // WAL checkpoint
    Rollback,               // Explicit rollback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    pub record_type: WalRecordType,
    pub transaction_id: u64,
    pub sequence: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

pub struct RecoveryManager;

impl RecoveryManager {
    /// Recover from crash on startup
    pub async fn recover(store: &StorageEngine) -> Result<RecoveryReport>;
    
    /// Replay WAL
    pub async fn replay_wal(store: &StorageEngine) -> Result<()>;
    
    /// Create checkpoint
    pub async fn checkpoint(store: &StorageEngine) -> Result<()>;
}

pub struct RecoveryReport {
    pub recovered_transactions: u64,
    pub replayed_records: u64,
    pub duration_ms: u64,
}
```

#### 2.11 Garbage Collector

**From spec 11-reliability-recovery.md:**

```rust
pub struct GarbageCollector;

impl GarbageCollector {
    /// Run garbage collection
    pub async fn collect(store: &StorageEngine, policy: GcPolicy) -> Result<GcReport>;
}

pub enum GcPolicy {
    KeepRevisions(usize),      // Keep N most recent revisions
    BySpaceUtilization(f32),   // Trigger at X% usage
    ByAge(u64),                // Delete data older than N seconds
}

pub struct GcReport {
    pub tiles_deleted: u64,
    pub bytes_freed: u64,
    pub duration_ms: u64,
}
```

---

### PHASE 3: CNWS Lattice Layer

**Goal:** Implement dynamic adaptive execution, memory, routing, learning

**Key Specification:** 06-runtime-execution.md, 09-memory-retrieval.md, 08-revision-learning.md

#### 3.1 Execution Engine

**From spec 06-runtime-execution.md:**

```rust
pub struct ExecutionEngine {
    storage: Arc<StorageEngine>,
    cache: Arc<CacheManager>,
    memory: Arc<MemorySystem>,
    routing: Arc<RoutingEngine>,
}

impl ExecutionEngine {
    /// Execute query with adaptive depth and budget
    pub async fn execute(
        &self,
        query: &Query,
        budget: ComputeBudget,
    ) -> Result<WorkingState>;
    
    /// Core algorithm: Query → Cell Selection → Execution Planning
    /// 1. Derive query from input
    /// 2. Select k most similar Cells (ANN search)
    /// 3. Plan execution order
    /// 4. Load Cells with budget constraints
    /// 5. Estimate difficulty
    /// 6. Allocate adaptive compute
    /// 7. Execute until halt condition
}
```

**Key Algorithms:**

```rust
// 1. Query Derivation (from spec §4)
fn derive_query(input: &[f32], working_state: &WorkingState) -> Query;

// 2. Cell Selection (from spec §5)
fn select_cells(
    query: &Query,
    candidates: &[Blake3Hash],
    top_k: usize,           // Default 16
    threshold: f32,         // Default 0.3
    metric: SimilarityMetric, // Default cosine
) -> Result<Vec<(Blake3Hash, f32)>>;

// 3. Execution Planning (from spec §8)
fn plan_execution(
    cells: &[Blake3Hash],
    dependencies: &RevisionDAG,
) -> Result<ExecutionPlan>;

// 4. Adaptive Depth (from spec §9)
fn estimate_depth(
    query: &Query,
    difficulty: f32,
    max_depth: u32,  // Default 25
    min_depth: u32,  // Default 3
) -> Result<u32>;

// 5. Difficulty Estimation (from spec §10)
fn estimate_difficulty(query: &Query, cells: &[Cell]) -> f32;

// 6. Halt Condition (from spec §11)
fn should_halt(
    state: &WorkingState,
    budget: &ComputeBudget,
    halt_criteria: &HaltCriteria,
) -> bool;
```

#### 3.2 Memory System (First-Class Persistent)

**From spec 09-memory-retrieval.md:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Episodic,   // Specific instances ("I saw model X...")
    Semantic,   // Generalizations ("Models have weights...")
    Procedural, // Learned patterns ("To infer, execute...")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Blake3Hash,
    pub memory_type: MemoryType,
    pub content: Cell,
    pub index_vector: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub access_count: u64,
    pub last_accessed: u64,
    pub version: u32,
}

pub struct MemorySystem {
    storage: Arc<StorageEngine>,
    cache: Arc<RwLock<HashMap<Blake3Hash, MemoryEntry>>>,
}

impl MemorySystem {
    /// Store memory entry persistently
    pub async fn store(&self, entry: MemoryEntry) -> Result<Blake3Hash>;
    
    /// Retrieve by similarity search
    pub async fn retrieve(
        &self,
        query: &[f32],
        memory_type: MemoryType,
        top_k: usize,
    ) -> Result<Vec<(Blake3Hash, f32)>>;
    
    /// Update entry
    pub async fn update(&self, id: Blake3Hash, entry: MemoryEntry) -> Result<()>;
    
    /// Evict entries by policy
    pub async fn evict(&self, policy: MemoryEvictionPolicy) -> Result<u64>;
}

pub enum MemoryEvictionPolicy {
    LRU(usize),         // Least recently used, keep N entries
    ByAge(u64),         // Delete entries older than N seconds
    ByBudget(u64),      // Keep total size under N bytes
}
```

#### 3.3 Routing Engine

**From spec 06-runtime-execution.md §6:**

```rust
pub struct RoutingEngine {
    policy: Arc<RwLock<RoutingPolicy>>,
    statistics: Arc<RwLock<RoutingStatistics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// Rules for Cell selection
    pub rules: Vec<RoutingRule>,
    
    /// Cost model
    pub cost_model: CostModel,
    
    /// Constraints
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct RoutingRule {
    pub condition: String,      // e.g., "cell_type == ATTENTION"
    pub action: RoutingAction,
}

pub enum RoutingAction {
    Select(f32),         // Select with priority weight
    Reject(String),      // Reject with reason
    Transform(String),   // Transform Cell before execution
}

pub struct RoutingStatistics {
    pub total_queries: u64,
    pub total_cells_routed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_latency_ms: f64,
}

impl RoutingEngine {
    /// Select Cells based on policy
    pub fn select(
        &self,
        query_vector: &[f32],
        candidates: &[Blake3Hash],
        top_k: usize,
    ) -> Result<Vec<(Blake3Hash, f32)>>;
    
    /// Route by semantic query string
    pub fn route(&self, query: &str) -> Result<Vec<Blake3Hash>>;
    
    /// Update routing policy
    pub fn update_policy(&self, policy: RoutingPolicy) -> Result<()>;
    
    /// Get statistics
    pub fn get_statistics(&self) -> RoutingStatistics;
}
```

#### 3.4 Learning Engine

**From spec 08-revision-learning.md:**

```rust
pub struct LearningEngine {
    storage: Arc<StorageEngine>,
    revision_dag: Arc<RevisionDAG>,
}

#[derive(Debug, Clone)]
pub enum LearningUpdateType {
    AddCell(Cell),
    ModifyCell(Blake3Hash, Cell),
    RemoveCell(Blake3Hash),
    UpdateRouting(RoutingPolicy),
    UpdateComposition(CompositionPattern),
}

#[derive(Debug, Clone)]
pub struct LearningUpdate {
    pub updates: Vec<LearningUpdateType>,
    pub metadata: HashMap<String, String>,
}

impl LearningEngine {
    /// Apply learning update (creates new revision)
    pub async fn apply_update(&self, update: LearningUpdate) -> Result<Blake3Hash>;
    
    /// Discover composition patterns
    pub async fn discover_patterns(&self) -> Result<Vec<CompositionPattern>>;
    
    /// Branch for experimentation
    pub async fn branch(&self, name: &str) -> Result<Blake3Hash>;
    
    /// Merge two branches
    pub async fn merge(
        &self,
        branch1: Blake3Hash,
        branch2: Blake3Hash,
    ) -> Result<Blake3Hash>;
    
    /// Rollback to previous revision
    pub async fn rollback(&self, revision_id: Blake3Hash) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPattern {
    pub cells: Vec<Blake3Hash>,
    pub execution_order: Vec<usize>,
    pub frequency: u64,
    pub performance: f32,
}
```

#### 3.5 Cache Manager (Hierarchy)

**From spec 06-runtime-execution.md §10:**

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum CacheLevel {
    GPU,    // Fastest, smallest (e.g., VRAM)
    CPU,    // Medium speed/size (e.g., RAM)
    NVMe,   // Large, slow (e.g., fast NVMe)
}

pub struct CacheManager {
    levels: HashMap<CacheLevel, Arc<LruCache>>,
    policy: Arc<RwLock<EvictionPolicy>>,
}

pub struct LruCache {
    entries: Arc<RwLock<LinkedHashMap<Blake3Hash, CacheEntry>>>,
    capacity_bytes: u64,
    current_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub hash: Blake3Hash,
    pub data: Vec<u8>,
    pub size: u64,
    pub priority: f32,
    pub last_accessed: u64,
    pub access_count: u64,
}

pub enum EvictionPolicy {
    LRU,                    // Least recently used
    LFU,                    // Least frequently used
    PriorityWeighted,       // By priority + recency
}

impl CacheManager {
    /// Get entry from cache (any level)
    pub fn get(&self, hash: &Blake3Hash) -> Option<Vec<u8>>;
    
    /// Put entry in appropriate cache level
    pub fn put(&self, entry: CacheEntry, level: CacheLevel) -> Result<()>;
    
    /// Promote entry to faster level
    pub fn promote(&self, hash: &Blake3Hash) -> Result<()>;
    
    /// Evict by policy
    pub fn evict(&self, target_bytes: u64) -> Result<()>;
    
    /// Get statistics per level
    pub fn get_statistics(&self) -> HashMap<CacheLevel, CacheStatistics>;
}

pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_size_bytes: u64,
    pub utilization_percent: f32,
}
```

#### 3.6 Prefetch Engine

**From spec 06-runtime-execution.md §14:**

```rust
pub struct PrefetchEngine {
    scheduler: Arc<RwLock<Vec<PrefetchEntry>>>,
    bandwidth_budget: u64,
}

#[derive(Debug, Clone)]
pub struct PrefetchEntry {
    pub cell_hash: Blake3Hash,
    pub predicted_latency: u32,
    pub priority: f32,
    pub deadline: u64,
}

impl PrefetchEngine {
    /// Schedule prefetch of upcoming Cells
    pub fn schedule_prefetch(&self, cells: &[Blake3Hash], deadlines: &[u64]) -> Result<()>;
    
    /// Execute pending prefetches
    pub async fn execute_pending(&self, cache: &CacheManager) -> Result<()>;
    
    /// Update bandwidth budget
    pub fn set_bandwidth_budget(&self, bytes_per_sec: u64);
    
    /// Get prefetch statistics
    pub fn get_statistics(&self) -> PrefetchStatistics;
}

pub struct PrefetchStatistics {
    pub scheduled: u64,
    pub executed: u64,
    pub hit_rate: f32,
    pub avg_latency_saved_ms: f64,
}
```

---

### PHASE 4: Public API & CLI

**Goal:** Stable interfaces for users

**Key Specification:** 12-api-protocol.md

#### 4.1 Runtime API

```rust
/// Primary user-facing interface for CNWS runtime
pub struct RuntimeApi {
    engine: Arc<ExecutionEngine>,
}

impl RuntimeApi {
    /// Open a `.cd` model
    pub async fn open(path: &Path) -> Result<Self>;
    
    /// Execute query with budget
    pub async fn execute(
        &self,
        query: &Query,
        budget: ComputeBudget,
    ) -> Result<WorkingState>;
    
    /// Prefetch Cells into cache
    pub async fn prefetch(&self, cells: &[Blake3Hash]) -> Result<()>;
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<CacheLevel, CacheStatistics>;
}

/// Builder for Query construction
pub struct QueryBuilder {
    embedding: Vec<f32>,
    metadata: HashMap<String, serde_json::Value>,
}

impl QueryBuilder {
    pub fn new(embedding: Vec<f32>) -> Self;
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self;
    pub fn build(self) -> Query;
}
```

#### 4.2 Storage API

```rust
pub struct StorageApi {
    engine: Arc<StorageEngine>,
}

impl StorageApi {
    /// Open `.cd` store
    pub async fn open(path: &Path) -> Result<Self>;
    
    /// Import checkpoint (conversion)
    pub async fn import_checkpoint(
        &self,
        path: &Path,
        format: CheckpointFormat,
    ) -> Result<ImportReport>;
    
    /// Get Cell
    pub async fn get_cell(&self, hash: Blake3Hash) -> Result<Cell>;
    
    /// Get Tile data
    pub async fn get_tile(&self, hash: Blake3Hash) -> Result<Vec<u8>>;
    
    /// Get manifest
    pub fn get_manifest(&self) -> Result<Manifest>;
    
    /// Get statistics
    pub fn get_stats(&self) -> Result<StoreStatistics>;
}
```

#### 4.3 Revision API

```rust
pub struct RevisionApi {
    dag: Arc<RevisionDAG>,
    storage: Arc<StorageEngine>,
}

impl RevisionApi {
    pub fn get_current(&self) -> Result<Blake3Hash>;
    pub fn get_revision(&self, id: Blake3Hash) -> Result<Revision>;
    pub async fn branch(&self, name: &str) -> Result<Blake3Hash>;
    pub async fn merge(&self, branch: Blake3Hash) -> Result<Blake3Hash>;
    pub async fn rollback(&self, revision_id: Blake3Hash) -> Result<()>;
    pub fn get_history(&self, limit: usize) -> Result<Vec<Revision>>;
}
```

#### 4.4 CLI Tool

```
$ cnws import model.safetensors output.cd
$ cnws info output.cd
$ cnws query output.cd --embedding "..."
$ cnws revision output.cd --list
$ cnws gc output.cd
$ cnws verify output.cd
```

---

### PHASE 5: Testing & Conformance

**Goal:** Verify specification compliance

**Key Specification:** 13-testing-conformance.md

#### 5.1 Unit Tests

- [ ] Blake3Hash serialization
- [ ] All 35 CellTypes round-trip
- [ ] Tile registry operations
- [ ] Cache eviction policies
- [ ] Similarity search algorithms

#### 5.2 Integration Tests

- [ ] Import Safetensors checkpoint → `.cd`
- [ ] Execute query with budget constraints
- [ ] Memory storage and retrieval
- [ ] Revision branching and merging
- [ ] Integrity verification
- [ ] Recovery from crash

#### 5.3 Conformance Tests

From spec 13-testing-conformance.md:

- [ ] Binary format compliance (magic bytes, sizes, alignment)
- [ ] BLAKE3-256 verification
- [ ] Manifest JSON canonicality
- [ ] Streaming memory bounds (peak RAM independent of model size)
- [ ] Dependency DAG validity
- [ ] Specification requirement coverage matrix

#### 5.4 Performance Benchmarks

From spec 14-performance-benchmark.md:

- [ ] Conversion throughput (MB/s per core)
- [ ] Query latency (ms, p50/p99)
- [ ] Memory overhead (bytes per Cell)
- [ ] Cache hit rate (%, by level)
- [ ] Prefetch accuracy

---

### PHASE 6: Observability & Operations

**Goal:** Production-ready system

**Key Specification:** 15-observability.md, 16-operations-deployment.md

#### 6.1 Logging

Implement structured JSON logging with trace context:

```rust
pub struct Logger;

impl Logger {
    pub fn info(msg: &str, fields: &HashMap<String, serde_json::Value>);
    pub fn error(msg: &str, error: &CnwsError);
    pub fn trace(span: &TraceSpan, msg: &str);
}
```

#### 6.2 Metrics

Prometheus-compatible metrics:

- `cnws_cells_total` - Total Cell count
- `cnws_storage_bytes` - Total storage size
- `cnws_query_latency_ms` - Query execution latency
- `cnws_cache_hit_rate` - Cache hit percentage
- `cnws_import_throughput_mbs` - Import speed

#### 6.3 Distributed Tracing

OpenTelemetry integration:

- Trace import operations
- Trace query execution
- Trace cache operations
- Export to Jaeger

---

## IV. Compliance Checklist

### Must-Have (Binding)

- [ ] Engineering Contract invariants enforced
- [ ] All 35 CellTypes correctly implemented
- [ ] BLAKE3-256 content addressing
- [ ] `.cd` format byte-compatible with spec
- [ ] Streaming import (bounded memory)
- [ ] Immutable Tiles with deduplication
- [ ] Revision DAG with branching
- [ ] Adaptive execution engine
- [ ] Persistent first-class memory
- [ ] Integrity verification
- [ ] Recovery from crash

### Should-Have (High Priority)

- [ ] Comprehensive test coverage (unit + integration)
- [ ] Performance benchmarks pass targets
- [ ] Observability (logging, metrics, tracing)
- [ ] CLI tool fully featured
- [ ] Documentation with examples
- [ ] Garbage collection working

### Nice-to-Have (Lower Priority)

- [ ] Distributed deployment support
- [ ] GPU kernel optimization
- [ ] Advanced caching strategies
- [ ] Model specialization tools

---

## V. Getting Started

### For New Contributors

1. **Read the specifications:**
   ```bash
   cd docs/specs/
   # Start with 01-engineering-contract.md
   # Then 05-cell-schema.md
   # Then the phase-relevant spec
   ```

2. **Pick a phase (start with Phase 1):**
   - Begin with foundational types
   - Implement one component at a time
   - Write tests before code (TDD)
   - Reference spec sections in code comments

3. **Follow the structure:**
   ```
   cnws-core/
   ├── types.rs           # Phase 1: Core types
   ├── substrate/
   │   ├── storage.rs     # Phase 2: Superblock, Tiles, Manifest
   │   ├── integrity.rs   # Phase 2: Verification
   │   ├── revision.rs    # Phase 2: Revision DAG
   │   └── conversion.rs  # Phase 2: Import pipeline
   ├── lattice/
   │   ├── runtime.rs     # Phase 3: Execution engine
   │   ├── memory.rs      # Phase 3: Persistent memory
   │   ├── routing.rs     # Phase 3: Cell selection
   │   └── cache.rs       # Phase 3: Cache hierarchy
   └── api/
       └── *api.rs        # Phase 4: Public interfaces
   ```

4. **Test thoroughly:**
   - Unit tests for each type
   - Integration tests for workflows
   - Conformance tests against spec
   - Performance benchmarks

5. **Document compliance:**
   - Add comments linking code to spec sections
   - Create compliance matrix
   - Document design decisions

### Development Workflow

```bash
# 1. Check out branch
git checkout -b feature/phase1-types

# 2. Implement component
# Follow spec precisely
# Use types.rs as example

# 3. Write tests
# Cover all cases from spec

# 4. Verify compilation
cargo build --lib
cargo test

# 5. Run conformance
cargo test --test conformance_tests

# 6. Submit PR
# Reference spec sections in PR description
```

---

## VI. Key Resources

- **Engineering Contract:** `docs/specs/01-engineering-contract.md` (THE authority)
- **Architecture:** `docs/specs/03-detailed-architecture.md`
- **Format Spec:** `docs/specs/04-cd-format-serialization.md`
- **Cell Schema:** `docs/specs/05-cell-schema.md`
- **Runtime Spec:** `docs/specs/06-runtime-execution.md`
- **Memory Spec:** `docs/specs/09-memory-retrieval.md`
- **Testing Spec:** `docs/specs/13-testing-conformance.md`

---

## VII. Summary

This guide provides a **phase-by-phase roadmap** for building CNWS according to its Engineering Contract and specifications.

**Critical Principle:** Specifications are binding. All implementation decisions must be justified by reference to spec sections.

**Success Metric:** Fully conformant CNWS implementation that:
1. Compiles without errors
2. Passes all conformance tests
3. Achieves performance targets
4. Handles production workloads reliably

Start with Phase 1 (core types), move systematically through phases, write tests at each stage, and always reference the specifications.

Good luck building CNWS! 🚀
