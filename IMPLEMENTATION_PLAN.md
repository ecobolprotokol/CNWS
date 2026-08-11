# CNWS Complete End-to-End Implementation Plan

**Source of Truth:** `docs/specs/` (Engineering Contract + all 17 specifications)

**Status:** Current codebase has skeleton implementations with compilation errors. Full implementation required.

**Date:** 2026-08-11

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Phase 1: Foundation & Core Infrastructure](#phase-1-foundation--core-infrastructure)
4. [Phase 2: CNWS Substrate Layer](#phase-2-cnws-substrate-layer)
5. [Phase 3: CNWS Lattice Layer](#phase-3-cnws-lattice-layer)
6. [Phase 4: Public API & CLI](#phase-4-public-api--cli)
7. [Phase 5: Testing & Conformance](#phase-5-testing--conformance)
8. [Phase 6: Observability & Operations](#phase-6-observability--operations)
9. [Implementation Checklist](#implementation-checklist)

---

## Executive Summary

CNWS is a **canonical intelligence infrastructure** that unifies knowledge representation, computation, memory, routing, and state through a content-addressed Cell-based paradigm.

### Key Objectives

1. **Build the CNWS Substrate** — immutable storage, versioning, integrity, recovery
2. **Build the CNWS Lattice** — adaptive execution, memory, routing, learning
3. **Implement Conversion Pipeline** — import from Safetensors, GGUF, PyTorch
4. **Deliver conformant, observable, and maintainable system**

### Binding Engineering Decisions (from Engineering Contract)

- **DF-01..DF-13:** Name, Cell, Tile, BLAKE3-256, `.cd`, streaming conversion, adaptive execution, persistent memory, structural learning, incremental versioning, zero format coupling
- **FAC-1..FAC-34:** Final Architecture Contract with 10 characteristics and 35 invariants

---

## Architecture Overview

```
                         ┌─────────────────────────────────────────┐
                         │        PUBLIC API LAYER                 │
                         │   (StorageApi, RuntimeApi, etc.)        │
                         └──────────────┬──────────────────────────┘
                                        │
     ┌──────────────────────────────────┴──────────────────────────────────┐
     │                                                                       │
     ▼                                                                       ▼
┌────────────────────────────────────┐  ┌────────────────────────────────────┐
│  CNWS LATTICE LAYER                │  │  CNWS SUBSTRATE LAYER              │
│  (Execution & Logic)                │  │  (Storage & Versioning)            │
│                                    │  │                                    │
│ • Execution Engine                 │  │ • Storage Engine                   │
│ • Query/Cell Resolution            │  │ • Manifest Authority               │
│ • Memory System (persistent)       │  │ • Revision DAG                     │
│ • Routing Engine                   │  │ • Integrity Verifier               │
│ • Learning Engine                  │  │ • Recovery Manager                 │
│ • Cache Manager                    │  │ • Garbage Collector                │
│ • Prefetch Engine                  │  │ • Conversion Pipeline              │
└────────────────────────────────────┘  └────────────────────────────────────┘
     │                                            │
     │                                            │
     └──────────────────┬───────────────────────┘
                        │
            SUBSTRATE-LATTICE INTERFACE (SLI)
                        │
     ┌──────────────────┴───────────────────────┐
     │                                           │
     ▼                                           ▼
┌─────────────────────┐              ┌──────────────────────┐
│  .cd Store          │              │  Execution Runtime   │
│  (Directory-based)  │              │  (GPU/CPU/NVMe)      │
└─────────────────────┘              └──────────────────────┘
```

---

## Phase 1: Foundation & Core Infrastructure

**Goals:**
- Fix compilation errors
- Establish type system and error handling
- Implement foundational types (Cell, Tile, Blake3Hash, etc.)
- Set up basic I/O and serialization infrastructure

### P1.1 Fix Build System & Dependencies

- [ ] Update `rust-toolchain.toml` to 1.97.1 ✓
- [ ] Fix `cnws-core/Cargo.toml` (opentelemetry-jaeger optional, remove bench) ✓
- [ ] Ensure all dependencies are compatible
- [ ] Add missing imports to resolve compilation errors
- [ ] Verify workspace structure (root Cargo.toml references all crates)

### P1.2 Implement Type System (types.rs)

From spec: Cell & Schema Specification (05-cell-schema.md)

**Blake3Hash (Content Addressing)**
- [x] Basic BLAKE3-256 type
- [ ] Serialization/deserialization
- [ ] Display/From/Into implementations
- [ ] Hashing utilities (file, stream, data)

**Cell Type System** (35 types defined in spec)
- [ ] Implement all 35 CellType variants with correct discriminants
- [ ] CellType ↔ u32 conversion
- [ ] CellTypeMetadata (version, schema, attributes)
- [ ] CellDependency types (DATA, CONTROL, EXECUTION_ORDER, PREFETCH_HINT)

**Core Types**
- [ ] `Cell` struct (id, type, payload, metadata, index_vector, dependencies, version)
- [ ] `Tile` struct (hash, size, offset, compressed, segment_id)
- [ ] `TileRef` (content address + offset)
- [ ] `CellRef` (cell_id + dependency type)
- [ ] `IndexVector` (512-dim default, similarity metric)
- [ ] `Metadata` schema (extensible attributes map)

### P1.3 Implement Error Handling (error.rs)

Current issues in compilation:
- Missing `From<SetGlobalDefaultError>` impl
- Missing error variants for specific subsystems
- Need structured error codes

**Tasks:**
- [ ] Add all required error variants
- [ ] Implement `From` for common error types (IO, serde, tracing, etc.)
- [ ] Add error codes per spec (IO, Validation, Integrity, etc.)
- [ ] Document error handling strategy

### P1.4 Implement Serialization Framework

From spec: .cd Format & Serialization (04-cd-format-serialization.md)

- [ ] BLAKE3 hash serialization/deserialization
- [ ] Cell serialization format
- [ ] Tile serialization format  
- [ ] Memory entry serialization
- [ ] Endianness handling (little-endian per spec)
- [ ] Magic bytes (CNWSSB01, CNWSSEG1, etc.)
- [ ] Version format (semver: u32, u32, u32)

### P1.5 Implement Basic I/O Infrastructure

- [ ] BufferedReader/Writer for efficient I/O
- [ ] Streaming hash computation
- [ ] Alignment utilities (4KiB, 64KiB preferred)
- [ ] File locking mechanism
- [ ] WAL (Write-Ahead Log) record format

---

## Phase 2: CNWS Substrate Layer

**Goals:**
- Implement immutable storage engine
- Build manifest system (source of truth)
- Implement integrity verification
- Build revision DAG and versioning
- Implement conversion pipeline
- Add recovery subsystem
- Add garbage collection

### P2.1 Storage Engine (substrate/storage.rs)

From spec: .cd Format (04-cd-format-serialization.md)

**Superblock** (4096 bytes, fixed)
- [ ] Superblock structure (magic, version, checksum, metadata)
- [ ] Superblock read/write
- [ ] Format: little-endian, aligned

**Store Directory Layout**
- [ ] Create `.cd` directory structure
- [ ] Segment management (32 GiB target size)
- [ ] Index files (cells.idx, tiles.idx, memory.idx, routing.idx)
- [ ] Lattice storage (graph.cd, compositions.cd, routing_policy.cd)
- [ ] Memory storage (episodic, semantic, procedural segments)
- [ ] Journal/WAL directory

**Tile Registry**
- [ ] Registry structure (hash → TileLocation)
- [ ] Tile deduplication logic
- [ ] Tile lookup by hash
- [ ] Persistent registry (to tiles.idx)

**Segment Management**
- [ ] Segment header (4096 bytes)
- [ ] Segment writer (append-only)
- [ ] Segment reader (random access)
- [ ] Segment rotation when size limit reached

### P2.2 Manifest Authority (substrate/storage.rs)

From spec: Detailed Architecture (03-detailed-architecture.md)

**Manifest Structure** (`MANIFEST.cd`)
- [ ] JSON canonical format
- [ ] Root hash computation
- [ ] Cell registry (hash → metadata)
- [ ] Tile registry (hash → location)
- [ ] Memory entries mapping
- [ ] Routing policy reference
- [ ] Composition patterns reference
- [ ] Provenance information

**Manifest Lifecycle**
- [ ] Manifest creation
- [ ] Manifest update (staging phase)
- [ ] Manifest commit (atomic write)
- [ ] Manifest rollback
- [ ] Version history (MANIFEST.cd.prev)

### P2.3 Integrity Subsystem (substrate/integrity.rs)

From spec: Security Threat Model (10-security-threat-model.md)

**Verification**
- [ ] BLAKE3-256 verification for all Cell/Tile
- [ ] Structural integrity checks
- [ ] Manifest consistency checks
- [ ] Dependency graph validation
- [ ] Version compatibility checking

**Quarantine System**
- [ ] Quarantine entry structure (hash, reason, timestamp)
- [ ] Quarantine manager
- [ ] Suspicious content marking
- [ ] Recovery procedures

**Error Conditions**
- [ ] Hash mismatch detection
- [ ] Missing dependency detection
- [ ] Corrupted data handling
- [ ] Version incompatibility

### P2.4 Revision System (substrate/revision.rs)

From spec: Revision & Learning (08-revision-learning.md)

**Revision Structure**
- [ ] Revision ID (Blake3Hash of content)
- [ ] Metadata (timestamp, author, message, tags)
- [ ] Cell delta (added, modified, removed)
- [ ] Memory delta
- [ ] Routing delta
- [ ] Composition delta

**Revision DAG**
- [ ] DAG structure (parent links)
- [ ] Branch support
- [ ] Merge detection
- [ ] Common ancestor computation
- [ ] Topological ordering
- [ ] Ancestry queries

**Operations**
- [ ] Create revision
- [ ] Query revision by ID
- [ ] Get parent revisions
- [ ] Find common ancestor
- [ ] Branch operations
- [ ] Merge strategies (fast-forward, 3-way)

### P2.5 Conversion Pipeline (substrate/conversion.rs)

From spec: Conversion & Import (07-conversion-import.md)

**Pipeline Stages**
1. [ ] Format Reader (detect format, parse header)
   - Safetensors reader
   - GGUF reader
   - PyTorch reader
   - Custom format support

2. [ ] Normalizer (format-agnostic representation)
   - Tensor → Cell mapping
   - Metadata extraction
   - Index vector computation

3. [ ] Planner (streaming plan)
   - Chunk boundaries
   - Tile allocation
   - Deduplication candidates

4. [ ] Hasher (BLAKE3-256 computation)
   - Per-chunk hashing
   - Streaming hash

5. [ ] Deduplicator (check existing Tiles)
   - Tile registry lookup
   - Reuse decision

6. [ ] SegmentWriter (write new Tiles)
   - Batch writes
   - Alignment

7. [ ] CommitManager (atomic manifest update)
   - Staging phase
   - Validation
   - Atomic commit

**Memory Bounds**
- [ ] Peak RAM independent of model size
- [ ] Streaming chunk size configuration (default 128 MiB)
- [ ] Progress reporting

### P2.6 Recovery Subsystem (substrate/recovery.rs)

From spec: Reliability & Recovery (11-reliability-recovery.md)

**WAL (Write-Ahead Log)**
- [ ] WAL record types (Write, Commit, Checkpoint, Rollback)
- [ ] WAL serialization/deserialization
- [ ] WAL replay on startup
- [ ] Truncation strategy

**Recovery Manager**
- [ ] Detect incomplete operations
- [ ] Replay WAL
- [ ] Recover from crashes
- [ ] State consistency validation

**Checkpoint**
- [ ] Periodic checkpoint creation
- [ ] Checkpoint content
- [ ] Truncate WAL after checkpoint

### P2.7 Garbage Collector (substrate/gc.rs)

From spec: GC policy in Detailed Architecture

**Operations**
- [ ] Mark phase (trace referenced Tiles)
- [ ] Sweep phase (delete unreferenced Tiles)
- [ ] Segment defragmentation
- [ ] Statistics collection

**Policies**
- [ ] Retention policy (keep N revisions)
- [ ] Space policy (trigger at X% utilization)
- [ ] Time policy (clean up old data)

---

## Phase 3: CNWS Lattice Layer

**Goals:**
- Implement dynamic adaptive execution engine
- Build Cell resolution and routing
- Implement persistent memory system
- Add learning and composition
- Implement cache hierarchy and prefetch

### P3.1 Execution Engine (lattice/runtime.rs)

From spec: Runtime & Execution (06-runtime-execution.md)

**Query Derivation** (§4)
- [ ] Query struct (cell_id, embedding, context)
- [ ] Embedding projection from WorkingState
- [ ] Query validation

**Cell Selection** (§5)
- [ ] ANN search (approximate nearest neighbor)
- [ ] Similarity metric (cosine default)
- [ ] Selection threshold (0.3 default)
- [ ] Top-k selection (k=16 default)

**Execution Planning** (§8)
- [ ] Dependency graph analysis
- [ ] Execution order determination
- [ ] Prefetch list generation
- [ ] Resource requirement estimation

**Adaptive Depth** (§9)
- [ ] Halt predictor (learned depth selector)
- [ ] Max depth (25 default)
- [ ] Min depth (3 default)
- [ ] Difficulty estimation

**Compute Budget** (§12)
- [ ] Budget allocation per query
- [ ] Hard budget enforcement (not advisory)
- [ ] Budget tracking per Cell
- [ ] Rejection when budget exceeded

**Memory Budget** (§13)
- [ ] Working state size limits
- [ ] Active Cell capacity limits
- [ ] Eviction when limit exceeded

**Halt Conditions** (§11)
- [ ] Confidence threshold
- [ ] Depth limit
- [ ] Compute budget exhausted
- [ ] Memory budget exhausted

### P3.2 Cell Resolver (lattice/runtime.rs)

- [ ] CellRef resolution (hash → actual Cell data)
- [ ] Dependency resolution
- [ ] Index vector retrieval
- [ ] Lazy loading
- [ ] Caching integration

### P3.3 Routing Engine (lattice/routing.rs)

From spec: Runtime & Execution (06-runtime-execution.md)

**Routing Policy**
- [ ] Policy structure (rules, weights, constraints)
- [ ] Policy serialization
- [ ] Policy updates

**Metadata-based Routing**
- [ ] Cell metadata tags
- [ ] Cost estimation
- [ ] Accuracy impact estimation
- [ ] Hardware compatibility

**Statistics**
- [ ] Query latency tracking
- [ ] Cache hit rate
- [ ] Memory utilization
- [ ] Budget utilization

### P3.4 Memory System (lattice/memory.rs)

From spec: Memory Retrieval (09-memory-retrieval.md)

**Memory Entry Types**
- [ ] Episodic memory (specific instances)
- [ ] Semantic memory (generalizations)
- [ ] Procedural memory (learned patterns)

**Memory Entry Structure**
- [ ] Content (Cell payload)
- [ ] Index vector (for similarity search)
- [ ] Metadata (timestamp, relevance, access count)
- [ ] Versioning

**Memory Operations**
- [ ] Store entry
- [ ] Retrieve by similarity
- [ ] Update entry
- [ ] Evict by policy
- [ ] Consolidate entries

**Memory Persistence**
- [ ] Save to `.cd` storage
- [ ] Load on startup
- [ ] Checkpointing

### P3.5 Learning Engine (lattice/learning.rs)

From spec: Revision & Learning (08-revision-learning.md)

**Structural Learning**
- [ ] Learn new Cell patterns
- [ ] Modify existing Cells
- [ ] Create new Tiles
- [ ] Update index vectors

**Learning Updates**
- [ ] Incremental updates (not full copy)
- [ ] Tile-level deltas
- [ ] Composition pattern discovery
- [ ] Routing policy updates

**Integration with Revision**
- [ ] Each learning update = new Revision
- [ ] Branching for experiments
- [ ] Rollback support

### P3.6 Cache Manager (lattice/cache.rs)

From spec: Cache Architecture in Runtime spec

**Cache Hierarchy**
- [ ] GPU cache (fast, limited)
- [ ] CPU cache (moderate, larger)
- [ ] NVMe cache (slow, large)

**Cache Entry**
- [ ] Hash (Blake3 of Cell/Tile)
- [ ] Data
- [ ] Size
- [ ] Priority (based on cost + frequency)
- [ ] Timestamp

**Eviction Policies**
- [ ] LRU by priority
- [ ] Byte capacity limits
- [ ] Priority-aware eviction
- [ ] Segment-level eviction

**Cache Operations**
- [ ] Put entry
- [ ] Get entry
- [ ] Check existence
- [ ] Evict entry
- [ ] Clear cache

### P3.7 Prefetch Engine (lattice/runtime.rs or prefetch.rs)

From spec: Runtime (§14)

**Prefetch Strategy**
- [ ] Dependency-aware prefetch
- [ ] Predicted next Cells
- [ ] Bandwidth-aware prefetch
- [ ] Prefetch into appropriate cache level

**Prefetch Scheduling**
- [ ] Priority queue
- [ ] Deadline-aware
- [ ] Network/disk I/O batching

---

## Phase 4: Public API & CLI

**Goals:**
- Implement stable public API
- Build CLI tools
- Document API contracts

### P4.1 Storage API (api/storage.rs)

- [ ] `StorageApi::new(path)` - open/create store
- [ ] `StorageApi::import_checkpoint(path, format)` - conversion
- [ ] `StorageApi::get_cell(id)` - retrieve Cell
- [ ] `StorageApi::put_cell(cell)` - store Cell
- [ ] `StorageApi::get_manifest()` - current manifest
- [ ] `StorageApi::get_stats()` - statistics

### P4.2 Runtime API (api/runtime.rs)

- [ ] `RuntimeApi::new(storage)` - initialize
- [ ] `RuntimeApi::query(embedding)` - find relevant Cells
- [ ] `RuntimeApi::execute(query, budget)` - adaptive execution
- [ ] `RuntimeApi::prefetch(cells)` - warm cache
- [ ] `RuntimeApi::get_cache_stats()` - cache statistics

### P4.3 Revision API (api/revision.rs)

- [ ] `RevisionApi::get_current()` - current revision
- [ ] `RevisionApi::get_revision(id)` - query by ID
- [ ] `RevisionApi::branch(name)` - create branch
- [ ] `RevisionApi::merge(branch)` - merge branches
- [ ] `RevisionApi::rollback(revision_id)` - revert
- [ ] `RevisionApi::get_history()` - revision history

### P4.4 Memory API (api/memory.rs)

- [ ] `MemoryApi::store(entry)` - save memory
- [ ] `MemoryApi::retrieve(query)` - similarity search
- [ ] `MemoryApi::update(id, entry)` - modify
- [ ] `MemoryApi::evict(id)` - remove

### P4.5 CLI Tool (cnws-cli/)

- [ ] `cnws import <checkpoint> <format> <output.cd>` - convert
- [ ] `cnws info <model.cd>` - show statistics
- [ ] `cnws query <model.cd> <embedding>` - query Cells
- [ ] `cnws revision <model.cd>` - show revision history
- [ ] `cnws gc <model.cd>` - run garbage collection
- [ ] `cnws verify <model.cd>` - verify integrity

### P4.6 Admin API (api/admin.rs)

- [ ] Store inspection
- [ ] Statistics computation
- [ ] Manual operations (GC, defrag, etc.)

---

## Phase 5: Testing & Conformance

**Goals:**
- Build comprehensive test suite
- Verify spec conformance
- Establish performance baselines

From spec: Testing & Conformance (13-testing-conformance.md)

### P5.1 Unit Tests

- [ ] Type system tests
- [ ] Serialization round-trip tests
- [ ] Hash verification tests
- [ ] Revision DAG tests
- [ ] Cache eviction policy tests
- [ ] Error handling tests

### P5.2 Integration Tests (tests/)

From existing test files:
- [ ] `test_store.rs` - Storage engine integration
- [ ] `test_conversion.rs` - Conversion pipeline
- [ ] `test_runtime.rs` - Runtime execution
- [ ] `test_memory.rs` - Memory system
- [ ] `test_revision.rs` - Revision DAG
- [ ] `test_integrity.rs` - Integrity verification
- [ ] `integration_test.rs` - End-to-end flows

### P5.3 Conformance Tests

- [ ] Binary format compliance
- [ ] BLAKE3-256 verification
- [ ] Manifest canonicality
- [ ] Streaming memory bounds
- [ ] Dependency DAG validity
- [ ] Specification coverage matrix

### P5.4 Benchmark Tests

From spec: Performance Benchmark (14-performance-benchmark.md)

- [ ] Conversion throughput (MB/s)
- [ ] Query latency (ms)
- [ ] Memory overhead
- [ ] Cache hit rate
- [ ] Prefetch accuracy

---

## Phase 6: Observability & Operations

**Goals:**
- Implement comprehensive logging
- Add distributed tracing
- Build metrics system
- Add operational tools

From spec: Observability (15-observability.md)

### P6.1 Logging System (telemetry/logging.rs)

- [ ] Fix compilation errors (SetGlobalDefaultError)
- [ ] Log levels (TRACE, DEBUG, INFO, WARN, ERROR)
- [ ] Structured logging (JSON format)
- [ ] Log rotation/retention
- [ ] Performance impact tracking

### P6.2 Metrics (telemetry/metrics.rs)

- [ ] Counter metrics (operations, errors)
- [ ] Gauge metrics (sizes, utilization)
- [ ] Histogram metrics (latencies, throughput)
- [ ] Prometheus export
- [ ] Custom metric registration

### P6.3 Distributed Tracing (telemetry/tracing.rs)

- [ ] Trace context propagation
- [ ] Span creation/closure
- [ ] OpenTelemetry integration
- [ ] Jaeger export (optional)

### P6.4 Operational Tools

- [ ] Health checks
- [ ] Diagnostics endpoints
- [ ] Performance profiling
- [ ] Resource monitoring

---

## Implementation Checklist

### Compilation Fix (Priority 1)
- [ ] Fix error handling for SetGlobalDefaultError
- [ ] Fix cache.rs borrow checker issue (line 63)
- [ ] Fix closure type annotations (logging.rs:150)
- [ ] Clean up unused variables (warnings)
- [ ] Verify build succeeds

### Foundation Types (Priority 1)
- [ ] Complete Blake3Hash implementation
- [ ] Implement all 35 CellType variants
- [ ] Complete Cell, Tile, CellRef, TileRef structs
- [ ] Implement IndexVector type
- [ ] Implement Metadata schema

### Substrate Engine (Priority 2)
- [ ] Superblock read/write
- [ ] Segment management
- [ ] Tile registry
- [ ] Manifest system
- [ ] Integrity verification

### Conversion Pipeline (Priority 2)
- [ ] Safetensors reader
- [ ] GGUF reader
- [ ] PyTorch reader
- [ ] Streaming normalizer
- [ ] Tile deduplication

### Lattice Execution (Priority 2)
- [ ] Query derivation
- [ ] Cell selection (ANN)
- [ ] Execution planning
- [ ] Adaptive depth
- [ ] Budget enforcement

### APIs & CLI (Priority 3)
- [ ] Public API layer
- [ ] CLI tools
- [ ] Integration examples

### Testing (Priority 3)
- [ ] Unit tests (all modules)
- [ ] Integration tests
- [ ] Conformance tests
- [ ] Performance benchmarks

### Observability (Priority 3)
- [ ] Logging
- [ ] Metrics
- [ ] Tracing
- [ ] Operations dashboard

---

## Success Criteria

1. **Compilation:** All code compiles without errors
2. **Tests:** All integration tests pass
3. **Conformance:** Specification compliance verified
4. **Performance:** Benchmarks meet targets
5. **Usability:** CLI and API work as documented
6. **Reliability:** Recovery from crashes verified
7. **Observability:** All operations traceable

---

## Next Steps

1. Fix compilation errors (P1.1-P1.3)
2. Implement type system completely (P1.2-P1.4)
3. Implement Substrate layer (P2)
4. Implement Lattice layer (P3)
5. Build public APIs (P4)
6. Comprehensive testing (P5)
7. Observability infrastructure (P6)

---

## Document Status

- **Created:** 2026-08-11
- **Status:** Active Implementation Plan
- **Authority:** Derived from `docs/specs/` (Engineering Contract + 17 specifications)
- **Reviews:** To be conducted per PR process
