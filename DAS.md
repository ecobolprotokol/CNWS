# CNWS
## Detailed Architecture Specification (DAS)

| Field | Value |
|---|---|
| Dokumen | CNWS Detailed Architecture Specification |
| Status | **FINAL, NORMATIF, MENGIKAT** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract |
| Hulu ke | Implementasi modul, code review, integration testing |
| Otoritas | Blueprint teknis tunggal untuk struktur internal CNWS |

---

# 0. Document Control

## 0.1 Posisi DAS dalam Rantai Spesifikasi

```text
Engineering Contract        DAS                        Implementation
─────────────────────       ─────────────────────      ─────────────
WHAT invariants hold   ──►  WHICH component does   ──► Code modules
Final architecture          WHAT, and HOW they         Unit tests
Data model                  interact                   Integration tests
"MUST" statements           Component boundaries       Benchmarks
                            Dependency rules
                            Data & control flow
                            Interfaces
                            Threading model
                            Failure boundaries
                            Lifecycle
```

`[DAS-DOC-1]` DAS menjelaskan **komponen mana melakukan apa dan berinteraksi bagaimana**.

`[DAS-DOC-2]` Jika Engineering Contract mengatakan "MUST X", DAS menetapkan "komponen Y bertanggung jawab atas X, melalui mekanisme Z, berinteraksi dengan komponen W".

`[DAS-DOC-3]` DAS MUST NOT bertentangan dengan Engineering Contract.

`[DAS-DOC-4]` Jika terjadi konflik, Engineering Contract menang.

## 0.2 Terminologi Lapisan Internal

`[DAS-TERM-1]` CNWS adalah **satu paradigma terpadu** (sesuai Engineering Contract).

`[DAS-TERM-2]` Secara internal, CNWS diorganisasikan menjadi dua **lapisan fungsional**:

| Lapisan | Nama | Tanggung Jawab |
|---|---|---|
| Lapisan Bawah | **CNWS Substrate** | Penyimpanan, konversi, versioning, integritas, recovery |
| Lapisan Atas | **CNWS Lattice** | Eksekusi, resolusi, memori, routing, learning, cache |

`[DAS-TERM-3]` Substrate dan Lattice adalah **lapisan internal satu produk**, bukan dua produk terpisah.

`[DAS-TERM-4]` Interface antara keduanya disebut **Substrate–Lattice Interface (SLI)**.

## 0.3 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

---

# 1. Executive Summary

## 1.1 Tujuan DAS

Dokumen ini adalah blueprint teknis detail CNWS. Ia menjawab pertanyaan-pertanyaan yang tidak dijawab Engineering Contract:

| Pertanyaan | Dijawab di |
|---|---|
| Komponen apa saja yang ada? | §2 Component Architecture |
| Di mana batas antar modul? | §3 Module Boundaries |
| Siapa boleh bergantung pada siapa? | §4 Dependency Rules |
| Bagaimana data mengalir? | §5 Data Flow |
| Bagaimana kontrol mengalir? | §6 Control Flow |
| Bagaimana Substrate dan Lattice berinteraksi? | §7 SLI |
| Bagaimana threading diatur? | §8 Threading Model |
| Bagaimana async I/O bekerja? | §9 Async I/O Model |
| Bagaimana cache diorganisasi? | §10 Cache Architecture |
| Bagaimana hierarki memori? | §11 Memory Hierarchy |
| Di mana batas kegagalan? | §12 Failure Boundaries |
| Bagaimana lifecycle tiap subsystem? | §13 Lifecycle |

## 1.2 Prinsip Desain DAS

`[DAS-PRIN-1]` **Layering ketat**: Lattice bergantung pada Substrate, tidak sebaliknya.

`[DAS-PRIN-2]` **Acyclic dependency**: tidak ada dependency cycle antar modul.

`[DAS-PRIN-3]` **Interface minimal**: setiap modul mengekspos interface sempit.

`[DAS-PRIN-4]` **Failure isolation**: kegagalan satu modul tidak merambat tak terkendali.

`[DAS-PRIN-5]` **Bounded resources**: setiap modul memiliki budget resource eksplisit.

---

# 2. Component Architecture

## 2.1 Peta Komponen Lengkap

```text
┌─────────────────────────────────────────────────────────────────────┐
│                          CNWS LATTICE LAYER                         │
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ Execution   │  │ Query       │  │ Memory      │                │
│  │ Engine      │  │ Engine      │  │ System      │                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│         │                │                 │                        │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐                │
│  │ Cell        │  │ Routing     │  │ Learning    │                │
│  │ Resolver    │  │ Engine      │  │ Engine      │                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│         │                │                 │                        │
│  ┌──────┴───────────────────────────────────────┐                  │
│  │              Cache Manager                    │                  │
│  │   (GPU / CPU / NVMe hierarchy + eviction)     │                  │
│  └──────┬───────────────────────────────────────┘                  │
│         │                                                          │
│  ┌──────┴──────┐                                                    │
│  │ Prefetch    │                                                    │
│  │ Engine      │                                                    │
│  └──────┬──────┘                                                    │
└─────────┼──────────────────────────────────────────────────────────┘
          │                                                         
          │  ═══════════ SUBSTRATE–LATTICE INTERFACE (SLI) ═══════
          │                                                         
┌─────────┼──────────────────────────────────────────────────────────┐
│         ▼               CNWS SUBSTRATE LAYER                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │ Storage     │  │ Manifest    │  │ Revision    │                │
│  │ Engine      │  │ Authority   │  │ DAG         │                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│         │                │                 │                        │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐                │
│  │ Integrity   │  │ Garbage     │  │ Recovery    │                │
│  │ Subsystem   │  │ Collector   │  │ Subsystem   │                │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│         │                │                 │                        │
│  ┌──────┴───────────────────────────────────────┐                  │
│  │         Conversion Pipeline                    │                  │
│  │  (FormatReader → Normalizer → Planner →       │                  │
│  │   Hasher → Dedup → SegmentWriter → Commit)    │                  │
│  └───────────────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Daftar Komponen

### 2.2.1 CNWS Substrate

| ID | Komponen | Tanggung Jawab Utama |
|---|---|---|
| SUB-01 | Conversion Pipeline | Import checkpoint → canonical Cell/Tile |
| SUB-02 | Storage Engine | Segment & Tile read/write, registry |
| SUB-03 | Manifest Authority | Parse, serialize, validate manifest |
| SUB-04 | Revision DAG | Branch, merge, rollback, resolution |
| SUB-05 | Integrity Subsystem | BLAKE3 verification, quarantine |
| SUB-06 | Garbage Collector | Reachability, mark-sweep, reclaim |
| SUB-07 | Recovery Subsystem | Journal replay, crash recovery |

### 2.2.2 CNWS Lattice

| ID | Komponen | Tanggung Jawab Utama |
|---|---|---|
| LAT-01 | Cell Resolver | Resolve CellId → TileRef |
| LAT-02 | Query Engine | Content-addressed Cell selection |
| LAT-03 | Execution Engine | Dynamic composition, adaptive depth |
| LAT-04 | Memory System | Persistent memory hierarchy |
| LAT-05 | Routing Engine | Cell selection policy |
| LAT-06 | Learning Engine | Structural/incremental updates |
| LAT-07 | Cache Manager | GPU/CPU/NVMe cache hierarchy |
| LAT-08 | Prefetch Engine | Async load planning |

### 2.2.3 Cross-Cutting

| ID | Komponen | Tanggung Jawab Utama |
|---|---|---|
| X-01 | Observability | Metrics, logging, tracing |
| X-02 | Error Handling | Error propagation, classification |
| X-03 | Configuration | Runtime & store configuration |
| X-04 | Concurrency Primitives | Lock, channel, atomic |

---

# 3. Module Boundaries

## 3.1 Prinsip Boundary

`[BOUND-1]` Setiap modul MUST memiliki public interface eksplisit.

`[BOUND-2]` Internal state modul MUST NOT diakses langsung oleh modul lain.

`[BOUND-3]` Komunikasi antar modul MUST melalui interface atau message passing.

`[BOUND-4]` Setiap modul MUST memiliki ownership jelas atas data yang dimutasikan.

## 3.2 Boundary tiap Modul Substrate

### 3.2.1 Conversion Pipeline

```text
┌────────────────────────────────────────────────┐
│ Conversion Pipeline                            │
│                                                │
│ PUBLIC INTERFACE:                              │
│   convert(source, target, policy) → Report     │
│   validate_source(source) → Result             │
│                                                │
│ INTERNAL (tidak terekspos):                    │
│   - FormatReader instances                     │
│   - Normalizer state                           │
│   - TilePlanner buffers                        │
│   - Hasher streaming state                     │
│                                                │
│ OWNS:                                          │
│   - staging area selama conversion             │
│   - temporary segment files                    │
│                                                │
│ DEPENDS ON:                                    │
│   - Storage Engine (write Tiles)               │
│   - Manifest Authority (build manifest)        │
│   - Integrity Subsystem (hash verification)    │
│                                                │
│ DEPENDED BY:                                   │
│   - (none — entry point)                       │
└────────────────────────────────────────────────┘
```

### 3.2.2 Storage Engine

```text
┌────────────────────────────────────────────────┐
│ Storage Engine                                 │
│                                                │
│ PUBLIC INTERFACE:                              │
│   open_store(path) → Store                     │
│   write_tile(tile) → TileLocation              │
│   read_tile(tile_id) → TileReader              │
│   lookup_tile(tile_id) → TileLocation          │
│   list_tiles(filter) → Vec<TileId>             │
│                                                │
│ INTERNAL:                                      │
│   - SegmentManager                             │
│   - TileRegistry (HashMap)                     │
│   - AlignmentManager                           │
│   - IndexManager                               │
│                                                │
│ OWNS:                                          │
│   - segment files                              │
│   - segment index                              │
│   - tile registry                              │
│                                                │
│ DEPENDS ON:                                    │
│   - Integrity Subsystem (verify on read)       │
│                                                │
│ DEPENDED BY:                                   │
│   - Conversion Pipeline                        │
│   - Revision DAG                               │
│   - Garbage Collector                          │
│   - SLI (Tile read untuk Lattice)              │
└────────────────────────────────────────────────┘
```

### 3.2.3 Manifest Authority

```text
┌────────────────────────────────────────────────┐
│ Manifest Authority                             │
│                                                │
│ PUBLIC INTERFACE:                              │
│   load_manifest(path) → Manifest               │
│   serialize_manifest(manifest) → Bytes         │
│   validate_manifest(manifest) → Result         │
│   canonical_hash(manifest) → Blake3Hash        │
│                                                │
│ INTERNAL:                                      │
│   - SchemaManager                              │
│   - CanonicalSerializer                        │
│   - Validator                                  │
│                                                │
│ OWNS:                                          │
│   - manifest schema definition                 │
│   - canonical serialization rules              │
│                                                │
│ DEPENDS ON:                                    │
│   - Integrity Subsystem (hash)                 │
│                                                │
│ DEPENDED BY:                                   │
│   - Conversion Pipeline                        │
│   - Revision DAG                               │
│   - Recovery Subsystem                         │
│   - SLI (manifest load untuk Lattice)          │
└────────────────────────────────────────────────┘
```

### 3.2.4 Revision DAG

```text
┌────────────────────────────────────────────────┐
│ Revision DAG                                   │
│                                                │
│ PUBLIC INTERFACE:                              │
│   create_revision(delta) → RevisionID          │
│   branch(base, name) → RevisionID              │
│   merge(a, b) → RevisionID                     │
│   rollback(target) → Result                    │
│   resolve(rev) → EffectiveGraph                │
│   set_active(rev) → Result                     │
│                                                │
│ INTERNAL:                                      │
│   - DAG store                                  │
│   - EffectiveGraphBuilder                      │
│   - ResolutionCache                            │
│   - ThreeWayMerger                             │
│                                                │
│ OWNS:                                          │
│   - revision objects                           │
│   - revision DAG structure                     │
│   - resolution cache                           │
│                                                │
│ DEPENDS ON:                                    │
│   - Storage Engine (Tile refs)                 │
│   - Manifest Authority                         │
│                                                │
│ DEPENDED BY:                                   │
│   - Learning Engine (via SLI)                  │
│   - Garbage Collector                          │
└────────────────────────────────────────────────┘
```

### 3.2.5 Integrity Subsystem

```text
┌────────────────────────────────────────────────┐
│ Integrity Subsystem                            │
│                                                │
│ PUBLIC INTERFACE:                              │
│   hash_payload(stream) → Blake3Hash            │
│   verify_tile(tile) → Result                   │
│   verify_manifest(manifest) → Result           │
│   quarantine(tile_id) → Result                 │
│                                                │
│ INTERNAL:                                      │
│   - Blake3 hasher pool                         │
│   - CorruptionDetector                         │
│   - QuarantineManager                          │
│                                                │
│ OWNS:                                          │
│   - corrupt/ directory                         │
│   - quarantine metadata                        │
│                                                │
│ DEPENDS ON:                                    │
│   - (none — primitive)                         │
│                                                │
│ DEPENDED BY:                                   │
│   - Storage Engine                             │
│   - Conversion Pipeline                        │
│   - Manifest Authority                         │
│   - SLI (verify sebelum eksekusi)              │
└────────────────────────────────────────────────┘
```

### 3.2.6 Garbage Collector

```text
┌────────────────────────────────────────────────┐
│ Garbage Collector                              │
│                                                │
│ PUBLIC INTERFACE:                              │
│   gc() → GcReport                              │
│   dry_run() → GcReport                         │
│                                                │
│ INTERNAL:                                      │
│   - ReachabilityAnalyzer                       │
│   - MarkPhase                                  │
│   - SweepPhase                                 │
│   - ReclaimPhase                               │
│                                                │
│ OWNS:                                          │
│   - GC metadata                                │
│                                                │
│ DEPENDS ON:                                    │
│   - Revision DAG (roots)                       │
│   - Storage Engine (Tile list)                 │
│                                                │
│ DEPENDED BY:                                   │
│   - (none — maintenance operation)             │
└────────────────────────────────────────────────┘
```

### 3.2.7 Recovery Subsystem

```text
┌────────────────────────────────────────────────┐
│ Recovery Subsystem                             │
│                                                │
│ PUBLIC INTERFACE:                              │
│   recover() → RecoveryReport                   │
│   check_consistency() → ConsistencyReport      │
│                                                │
│ INTERNAL:                                      │
│   - JournalReader                              │
│   - CommitRecovery                             │
│   - StateReconstructor                         │
│                                                │
│ OWNS:                                          │
│   - journal/commit.wal                         │
│                                                │
│ DEPENDS ON:                                    │
│   - Manifest Authority                         │
│   - Storage Engine                             │
│                                                │
│ DEPENDED BY:                                   │
│   - (invoked at startup)                       │
└────────────────────────────────────────────────┘
```

## 3.3 Boundary tiap Modul Lattice

### 3.3.1 Cell Resolver

```text
┌────────────────────────────────────────────────┐
│ Cell Resolver                                  │
│                                                │
│ PUBLIC INTERFACE:                              │
│   resolve_cell(cell_id) → CellHandle           │
│   resolve_tiles(cell, policy) → Vec<TileRef>   │
│                                                │
│ INTERNAL:                                      │
│   - CellIndex (HashMap)                        │
│   - TileRefResolver                            │
│                                                │
│ OWNS:                                          │
│   - in-memory cell index                       │
│                                                │
│ DEPENDS ON:                                    │
│   - SLI (manifest data)                        │
│                                                │
│ DEPENDED BY:                                   │
│   - Execution Engine                           │
│   - Query Engine                               │
│   - Prefetch Engine                            │
└────────────────────────────────────────────────┘
```

### 3.3.2 Query Engine

```text
┌────────────────────────────────────────────────┐
│ Query Engine                                   │
│                                                │
│ PUBLIC INTERFACE:                              │
│   select(query) → Vec<CellRef>                 │
│   build_index(cells) → Result                  │
│                                                │
│ INTERNAL:                                      │
│   - ANNIndex (HNSW/IVF)                        │
│   - ThresholdFilter                            │
│   - ScoreCalculator                            │
│                                                │
│ OWNS:                                          │
│   - ANN index structure                        │
│                                                │
│ DEPENDS ON:                                    │
│   - Cell Resolver                              │
│   - Routing Engine (statistics)                │
│                                                │
│ DEPENDED BY:                                   │
│   - Execution Engine                           │
└────────────────────────────────────────────────┘
```

### 3.3.3 Execution Engine

```text
┌────────────────────────────────────────────────┐
│ Execution Engine                               │
│                                                │
│ PUBLIC INTERFACE:                              │
│   execute(input, budget) → Output              │
│   generate(prompt, config) → TokenStream       │
│                                                │
│ INTERNAL:                                      │
│   - ExecutionPlanner                           │
│   - CompositionEngine                          │
│   - AdaptiveDepthController                    │
│   - BudgetEnforcer                             │
│   - HaltDetector                               │
│                                                │
│ OWNS:                                          │
│   - WorkingState selama eksekusi               │
│                                                │
│ DEPENDS ON:                                    │
│   - Cell Resolver                              │
│   - Query Engine                               │
│   - Cache Manager                              │
│   - Memory System                              │
│   - BudgetEnforcer                             │
│                                                │
│ DEPENDED BY:                                   │
│   - (none — top-level entry)                   │
└────────────────────────────────────────────────┘
```

### 3.3.4 Memory System

```text
┌────────────────────────────────────────────────┐
│ Memory System                                  │
│                                                │
│ PUBLIC INTERFACE:                              │
│   store(key, value, type) → MemoryId           │
│   retrieve(query, k) → Vec<MemoryEntry>        │
│   associate(a, b, strength) → Result           │
│   consolidate(entries) → MemoryId              │
│                                                │
│ INTERNAL:                                      │
│   - WorkingMemory (bounded)                    │
│   - EpisodicMemory                             │
│   - SemanticMemory                             │
│   - ProceduralMemory                           │
│   - MemoryConsolidator                         │
│                                                │
│ OWNS:                                          │
│   - memory entries                             │
│   - memory index                               │
│                                                │
│ DEPENDS ON:                                    │
│   - SLI (persist memory cells)                 │
│                                                │
│ DEPENDED BY:                                   │
│   - Execution Engine                           │
│   - Learning Engine                            │
└────────────────────────────────────────────────┘
```

### 3.3.5 Cache Manager

```text
┌────────────────────────────────────────────────┐
│ Cache Manager                                  │
│                                                │
│ PUBLIC INTERFACE:                              │
│   get(tile_id, repr) → Option<TileHandle>      │
│   put(tile_id, repr, data) → Result            │
│   evict(policy) → Vec<TileId>                  │
│   prefetch(requests) → Result                  │
│                                                │
│ INTERNAL:                                      │
│   - GPUCache                                   │
│   - CPUCache                                   │
│   - NVMeCache                                  │
│   - EvictionPolicy                             │
│   - AdmissionPolicy                            │
│   - BudgetTracker                              │
│                                                │
│ OWNS:                                          │
│   - cache entries                              │
│   - cache metadata                             │
│                                                │
│ DEPENDS ON:                                    │
│   - SLI (load Tile dari Substrate)             │
│                                                │
│ DEPENDED BY:                                   │
│   - Execution Engine                           │
│   - Prefetch Engine                            │
└────────────────────────────────────────────────┘
```

---

# 4. Dependency Rules

## 4.1 Layering Rule

`[DEP-1]` Lattice MAY bergantung pada Substrate.

`[DEP-2]` Substrate MUST NOT bergantung pada Lattice.

`[DEP-3]` Komunikasi Substrate → Lattice hanya melalui event/callback, bukan direct call.

## 4.2 Dependency Graph

```text
                    Execution Engine
                   /    |    \      \
                  /     |     \      \
          Query  Cell   Cache  Memory  Budget
          Engine Resolver Manager System Enforcer
             \     |       |      |
              \    |       |      |
               ┌───┴───────┴──────┘
               │
        ═══════╪═══════════════════════  SLI
               │
       ┌───────┴────────┐
       │                │
  Storage Engine   Manifest Authority
       │                │
       └───────┬────────┘
               │
        Integrity Subsystem
               │
        (BLAKE3 primitive)
```

## 4.3 Aturan Dependency Detail

| From | To | Allowed | Mekanisme |
|---|---|---|---|
| Execution Engine | Cell Resolver | YES | Direct call |
| Execution Engine | Query Engine | YES | Direct call |
| Execution Engine | Cache Manager | YES | Direct call |
| Execution Engine | Memory System | YES | Direct call |
| Query Engine | Cell Resolver | YES | Direct call |
| Query Engine | Routing Engine | YES | Direct call |
| Cache Manager | SLI | YES | Async call |
| Cell Resolver | SLI | YES | Sync call |
| Memory System | SLI | YES | Async call |
| Learning Engine | SLI | YES | Async call |
| Conversion Pipeline | Storage Engine | YES | Direct call |
| Conversion Pipeline | Manifest Authority | YES | Direct call |
| Storage Engine | Integrity Subsystem | YES | Direct call |
| Revision DAG | Storage Engine | YES | Direct call |
| Revision DAG | Manifest Authority | YES | Direct call |
| Garbage Collector | Revision DAG | YES | Direct call |
| Garbage Collector | Storage Engine | YES | Direct call |
| Recovery Subsystem | Manifest Authority | YES | Direct call |
| Recovery Subsystem | Storage Engine | YES | Direct call |
| **Substrate → Lattice** | **any** | **NO** | **Forbidden** |

`[DEP-4]` Tidak boleh ada import atau reference dari modul Substrate ke modul Lattice.

`[DEP-5]` Dependency graph MUST acyclic.

## 4.4 Dependency Invariants

`[DEP-6]` Integrity Subsystem adalah leaf dependency (tidak bergantung pada modul lain).

`[DEP-7]` Execution Engine adalah root consumer (tidak depended-by modul lain).

`[DEP-8]` SLI adalah satu-satunya jalur komunikasi Lattice → Substrate.

---

# 5. Data Flow

## 5.1 Data Flow: Conversion

```text
External Checkpoint
        │
        │ raw bytes
        ▼
┌─────────────────┐
│ FormatReader    │  parse header, tensor metadata
└────────┬────────┘
         │ ExternalTensor stream
         ▼
┌─────────────────┐
│ Normalizer      │  map to semantic CellId, canonical dtype
└────────┬────────┘
         │ CanonicalCellTensor
         ▼
┌─────────────────┐
│ TilePlanner     │  split into Tiles
└────────┬────────┘
         │ Vec<TilePayload>
         ▼
┌─────────────────┐
│ Hasher          │  BLAKE3-256 streaming
└────────┬────────┘
         │ (TilePayload, TileId)
         ▼
┌─────────────────┐
│ Deduplicator    │  check TileRegistry
└────────┬────────┘
         │ new Tiles only
         ▼
┌─────────────────┐
│ SegmentWriter   │  write aligned Tiles
└────────┬────────┘
         │ TileLocation
         ▼
┌─────────────────┐
│ ManifestBuilder │  build canonical manifest
└────────┬────────┘
         │ Manifest
         ▼
┌─────────────────┐
│ Committer       │  atomic commit
└────────┬────────┘
         │
         ▼
   model.cd ready
```

`[FLOW-1]` Data conversion mengalir satu arah: source → canonical store.

`[FLOW-2]` Tidak ada data yang mengalir balik dari Storage ke Conversion selama conversion.

## 5.2 Data Flow: Inference

```text
Input
  │
  │ text/tokens
  ▼
┌─────────────────┐
│ Execution       │  Encode(input) → WorkingState
│ Engine          │
└────────┬────────┘
         │ Query
         ▼
┌─────────────────┐
│ Query Engine    │  Select(query) → Vec<CellRef>
└────────┬────────┘
         │ CellRefs
         ▼
┌─────────────────┐
│ Cell Resolver   │  resolve CellId → TileRefs
└────────┬────────┘
         │ TileRefs
         ▼
┌─────────────────┐
│ Cache Manager   │  lookup or load
└────────┬────────┘
         │ (miss)
         ▼
┌─────────────────┐  SLI
│ Storage Engine  │  read_tile(tile_id)
└────────┬────────┘
         │ TileData
         ▼
┌─────────────────┐
│ Integrity       │  verify BLAKE3
└────────┬────────┘
         │ verified TileData
         ▼
┌─────────────────┐
│ Cache Manager   │  insert into cache
└────────┬────────┘
         │ TileHandle
         ▼
┌─────────────────┐
│ Execution       │  Execute(cells, state)
│ Engine          │
└────────┬────────┘
         │ new WorkingState
         ▼
   (iterate or halt)
         │
         ▼
      Output
```

`[FLOW-3]` Data inference mengalir: input → query → resolve → load → execute → output.

`[FLOW-4]` Tile data hanya masuk ke Lattice melalui Cache Manager.

## 5.3 Data Flow: Learning

```text
Feedback / New Experience
        │
        ▼
┌─────────────────┐
│ Learning Engine │  assess, derive updates
└────────┬────────┘
         │ Vec<Update>
         │
         ├──────────────┬──────────────┐
         ▼              ▼              ▼
   CellCreate     CellRefine     RoutingUpdate
         │              │              │
         ▼              ▼              ▼
┌─────────────────────────────────────────┐  SLI
│ Substrate: apply updates                │
│   - write new Tiles                     │
│   - create new Revision                 │
│   - update routing cells                │
└────────┬────────────────────────────────┘
         │
         ▼
   New Revision committed
```

`[FLOW-5]` Learning updates mengalir dari Lattice ke Substrate melalui SLI.

`[FLOW-6]` Setiap learning update menghasilkan Revision baru.

---

# 6. Control Flow

## 6.1 Control Flow: Startup

```text
CNWS.start()
    │
    ├─► Recovery Subsystem
    │       recover()
    │       check_consistency()
    │
    ├─► Manifest Authority
    │       load_manifest()
    │       validate_manifest()
    │
    ├─► Revision DAG
    │       resolve(active_revision)
    │       build_effective_graph()
    │
    ├─► Storage Engine
    │       open_store()
    │       load_tile_registry()
    │
    ├─► Cell Resolver
    │       build_index(effective_graph)
    │
    ├─► Cache Manager
    │       initialize_hierarchy()
    │
    └─► Execution Engine
            ready()
```

`[CTRL-1]` Startup MUST sekuensial: Recovery → Manifest → Revision → Storage → Resolver → Cache → Execution.

`[CTRL-2]` Kegagalan pada langkah startup MUST menghentikan proses.

## 6.2 Control Flow: Inference Loop

```text
execute(input, budget):
    │
    ├─► state = Encode(input)
    │
    ├─► LOOP:
    │     │
    │     ├─► BudgetEnforcer.check(state.compute_used)
    │     │     └─► if exceeded → HALT
    │     │
    │     ├─► query = DeriveQuery(state)
    │     │
    │     ├─► selected = QueryEngine.select(query)
    │     │     └─► if empty → CreateCell or BREAK
    │     │
    │     ├─► tiles = CellResolver.resolve_tiles(selected)
    │     │
    │     ├─► CacheManager.ensure_loaded(tiles)
    │     │     └─► async prefetch parallel
    │     │
    │     ├─► outputs = Execute(selected, state)
    │     │
    │     ├─► state = Compose(state, outputs)
    │     │
    │     ├─► RoutingEngine.update(selected, state)
    │     │
    │     └─► HaltDetector.check(state)
    │           └─► if halt → BREAK
    │
    └─► return Decode(state)
```

`[CTRL-3]` Inference loop MUST memeriksa budget setiap iterasi.

`[CTRL-4]` HaltDetector MUST dievaluasi setiap iterasi.

## 6.3 Control Flow: Commit

```text
commit(manifest):
    │
    ├─► ManifestAuthority.serialize(manifest)
    │
    ├─► IntegritySubsystem.hash(serialized)
    │
    ├─► write staging/manifest-<hash>.cd
    │
    ├─► fsync(staging)
    │
    ├─► append journal/commit.wal
    │
    ├─► fsync(journal)
    │
    ├─► atomic rename staging → MANIFEST.cd
    │
    ├─► fsync(directory)
    │
    ├─► update SUPERBLOCK
    │
    ├─► fsync(SUPERBLOCK)
    │
    ├─► append commit-complete to journal
    │
    └─► fsync(journal)
```

`[CTRL-5]` Commit MUST mengikuti urutan staging → journal → rename → superblock → complete.

`[CTRL-6]` Setiap langkah kritis MUST fsync.

---

# 7. Substrate–Lattice Interface (SLI)

## 7.1 Definisi SLI

`[SLI-1]` SLI adalah satu-satunya interface antara Lattice dan Substrate.

`[SLI-2]` Lattice MUST mengakses Substrate hanya melalui SLI.

`[SLI-3]` Substrate MUST NOT memanggil Lattice secara langsung.

## 7.2 SLI Components

```text
┌─────────────────────────────────────────────────────────────┐
│                 SUBSTRATE–LATTICE INTERFACE                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐    ┌──────────────────┐              │
│  │  Tile Access API │    │ Manifest Query   │              │
│  │  (read, verify)  │    │ API              │              │
│  └──────────────────┘    └──────────────────┘              │
│                                                             │
│  ┌──────────────────┐    ┌──────────────────┐              │
│  │  Revision API    │    │ Learning         │              │
│  │  (resolve,       │    │ Commit API       │              │
│  │   branch)        │    │                  │              │
│  └──────────────────┘    └──────────────────┘              │
│                                                             │
│  ┌──────────────────┐    ┌──────────────────┐              │
│  │  Event Bus       │    │ Configuration    │              │
│  │  (Substrate →    │    │ API              │              │
│  │   Lattice)       │    │                  │              │
│  └──────────────────┘    └──────────────────┘              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 7.3 SLI API Specification

### 7.3.1 Tile Access API

```rust
trait SliTileAccess {
    // Sync: lookup metadata
    fn lookup_tile(&self, tile_id: &TileId) -> Result<TileLocation>;
    
    // Async: read Tile data
    async fn read_tile(&self, tile_id: &TileId) -> Result<TileData>;
    
    // Async: read Tile with verification
    async fn read_tile_verified(&self, tile_id: &TileId) -> Result<TileData>;
    
    // Async: batch read
    async fn read_tiles(&self, tile_ids: &[TileId]) -> Result<Vec<TileData>>;
    
    // Sync: check existence
    fn tile_exists(&self, tile_id: &TileId) -> bool;
}
```

### 7.3.2 Manifest Query API

```rust
trait SliManifestQuery {
    // Load manifest
    fn load_manifest(&self) -> Result<Manifest>;
    
    // Query Cell metadata
    fn get_cell(&self, cell_id: &CellId) -> Result<CellMeta>;
    
    // Query Cell dependencies
    fn get_dependencies(&self, cell_id: &CellId) -> Result<Vec<CellId>>;
    
    // Query architecture metadata
    fn get_architecture(&self) -> Result<ArchitectureMetadata>;
    
    // List Cells by type
    fn list_cells_by_type(&self, cell_type: CellType) -> Result<Vec<CellId>>;
}
```

### 7.3.3 Revision API

```rust
trait SliRevision {
    // Get active revision
    fn active_revision(&self) -> Result<RevisionID>;
    
    // Resolve effective graph
    fn resolve_effective_graph(&self, rev: RevisionID) -> Result<EffectiveGraph>;
    
    // List revisions
    fn list_revisions(&self) -> Result<Vec<RevisionID>>;
    
    // Get revision metadata
    fn get_revision(&self, rev: RevisionID) -> Result<Revision>;
}
```

### 7.3.4 Learning Commit API

```rust
trait SliLearningCommit {
    // Commit new Cells (from learning)
    async fn commit_cells(&self, cells: Vec<Cell>) -> Result<RevisionID>;
    
    // Commit Cell refinement
    async fn commit_refinement(&self, old: CellId, new: Cell) -> Result<RevisionID>;
    
    // Commit routing update
    async fn commit_routing(&self, updates: Vec<RoutingUpdate>) -> Result<RevisionID>;
    
    // Commit memory entries
    async fn commit_memory(&self, entries: Vec<MemoryEntry>) -> Result<RevisionID>;
}
```

### 7.3.5 Event Bus (Substrate → Lattice)

```rust
enum SubstrateEvent {
    TileCorrupted { tile_id: TileId },
    RecoveryCompleted { report: RecoveryReport },
    GcCompleted { report: GcReport },
    RevisionCommitted { rev: RevisionID },
    StoreConsistencyWarning { details: String },
}

trait SliEventBus {
    fn subscribe(&self, handler: Box<dyn Fn(SubstrateEvent)>);
    fn publish(&self, event: SubstrateEvent);
}
```

`[SLI-4]` Event Bus adalah satu-satunya mekanisme Substrate → Lattice.

`[SLI-5]` Event MUST bersifat notification, bukan request.

## 7.4 SLI Invariants

| ID | Invariant |
|---|---|
| SLI-1 | SLI adalah satu-satunya jalur Lattice → Substrate |
| SLI-2 | Lattice tidak boleh bypass SLI |
| SLI-3 | Substrate tidak boleh memanggil Lattice langsung |
| SLI-4 | Event Bus adalah satu-satunya mekanisme Substrate → Lattice |
| SLI-5 | Event bersifat notification, bukan request |
| SLI-6 | SLI Tile Access MUST async untuk read |
| SLI-7 | SLI MUST thread-safe |
| SLI-8 | SLI MUST mendukung concurrent access |

---

# 8. Threading / Concurrency Model

## 8.1 Thread Pool Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS THREAD POOLS                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Main Thread      │  Control flow, API entry              │
│  └──────────────────┘                                       │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Execution Pool   │  Cell execution, composition          │
│  │ (N threads)      │  N = CPU cores                        │
│  └──────────────────┘                                       │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ I/O Pool         │  Tile loading, segment reads          │
│  │ (M threads)      │  M = configurable                     │
│  └──────────────────┘                                       │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Prefetch Pool    │  Async prefetch                       │
│  │ (K threads)      │  K = configurable                     │
│  └──────────────────┘                                       │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Conversion Pool  │  Streaming conversion                 │
│  │ (P threads)      │  P = configurable                     │
│  └──────────────────┘                                       │
│                                                             │
│  ┌──────────────────┐                                       │
│  │ Maintenance Pool │  GC, recovery, compaction             │
│  │ (1-2 threads)    │                                       │
│  └──────────────────┘                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 8.2 Threading Rules

`[THREAD-1]` Main thread MUST NOT melakukan blocking I/O.

`[THREAD-2]` Execution pool MUST terpisah dari I/O pool.

`[THREAD-3]` Conversion pool MUST terpisah dari runtime pools.

`[THREAD-4]` Maintenance operations (GC, recovery) MUST pada pool terpisah.

`[THREAD-5]` Cross-pool communication MUST melalui channel, bukan shared mutable state.

## 8.3 Concurrency Primitives

| Primitive | Penggunaan |
|---|---|
| `RwLock` | Manifest, Tile Registry (read-heavy) |
| `Mutex` | Cache metadata, budget tracker |
| `AtomicU64` | Counter, budget usage |
| `mpsc channel` | Cross-pool messaging |
| `async channel` | Async I/O completion |

## 8.4 Lock Hierarchy

`[LOCK-1]` Lock acquisition MUST mengikuti hierarchy untuk mencegah deadlock.

```text
Lock Hierarchy (acquire in this order):
  1. Global config lock
  2. Store lock
  3. Manifest lock
  4. Revision lock
  5. Cache lock
  6. Tile lock
```

`[LOCK-2]` Tidak boleh acquire lock level lebih rendah lalu level lebih tinggi.

`[LOCK-3]` Read lock pada RwLock MAY concurrent.

`[LOCK-4]` Write lock pada RwLock MUST exclusive.

## 8.5 Single-Writer / Multi-Reader

`[CONC-1]` Store MUST mendukung multiple concurrent readers.

`[CONC-2]` Commit MUST single-writer.

`[CONC-3]` Write operations MUST memegang advisory lock `LOCK`.

`[CONC-4]` Cell reads MUST lock-free (immutable Tiles).

---

# 9. Async I/O Model

## 9.1 Async Runtime

`[ASYNC-1]` CNWS MUST menggunakan async runtime (mis. tokio) untuk I/O.

`[ASYNC-2]` Async I/O MUST non-blocking terhadap execution thread.

## 9.2 I/O Queues

```text
┌─────────────────────────────────────────────────────────────┐
│                    ASYNC I/O PIPELINE                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Request ──► ┌──────────────┐                               │
│              │ I/O Queue    │  disk reads                   │
│              └──────┬───────┘                               │
│                     │                                       │
│                     ▼                                       │
│              ┌──────────────┐                               │
│              │ Decompression│  zstd decode                  │
│              │ Queue        │                               │
│              └──────┬───────┘                               │
│                     │                                       │
│                     ▼                                       │
│              ┌──────────────┐                               │
│              │ Verification │  BLAKE3 check                 │
│              │ Queue        │                               │
│              └──────┬───────┘                               │
│                     │                                       │
│                     ▼                                       │
│              ┌──────────────┐                               │
│              │ H2D Transfer │  CPU → GPU                    │
│              │ Queue        │                               │
│              └──────┬───────┘                               │
│                     │                                       │
│                     ▼                                       │
│              ┌──────────────┐                               │
│              │ Completion   │  notify requester             │
│              │ Queue        │                               │
│              └──────────────┘                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

`[ASYNC-3]` Setiap tahap I/O MUST memiliki queue terpisah.

`[ASYNC-4]` Queue MUST bounded untuk mencegah memory blowup.

`[ASYNC-5]` Backpressure MUST diterapkan jika queue penuh.

## 9.3 Async API Pattern

```rust
// Tile loading adalah async
async fn load_tile(tile_id: TileId) -> Result<TileHandle> {
    // 1. Check cache (sync, fast)
    if let Some(handle) = cache.get(tile_id) {
        return Ok(handle);
    }
    
    // 2. Submit I/O request (async)
    let data = sli.read_tile_verified(&tile_id).await?;
    
    // 3. Insert into cache (sync)
    cache.put(tile_id, data)?;
    
    // 4. Return handle
    Ok(cache.get(tile_id).unwrap())
}
```

## 9.4 Overlap Strategy

`[ASYNC-6]` I/O, decompression, verification, dan transfer MUST dapat overlap dengan compute.

```text
Time ──────────────────────────────────────────►

Compute:   [Layer N]──────────────[Layer N+1]──────
I/O:       ────[prefetch N+1]────────[prefetch N+2]
Decomp:    ────────[decomp N+1]──────────[decomp N+2]
Transfer:  ────────────[H2D N+1]────────────[H2D N+2]
```

---

# 10. Cache Architecture

## 10.1 Cache Hierarchy

```text
┌─────────────────────────────────────────────────────────────┐
│                    CACHE HIERARCHY                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  L0: GPU VRAM                                               │
│      - Active execution Tiles                               │
│      - Capacity: configurable (default 80% VRAM)            │
│      - Eviction: LRU + priority                             │
│      - Latency: < 1 μs                                      │
│                                                             │
│  L1: CPU RAM                                                │
│      - Hot Tile cache                                       │
│      - Capacity: configurable (default 50% RAM)             │
│      - Eviction: LRU + frequency                            │
│      - Latency: < 10 μs                                     │
│                                                             │
│  L2: NVMe                                                   │
│      - Local .cd store                                      │
│      - Capacity: disk size                                  │
│      - Eviction: none (persistent)                          │
│      - Latency: < 1 ms                                      │
│                                                             │
│  L3: Network / Object Storage                               │
│      - Remote Tile pool                                     │
│      - Capacity: unbounded                                  │
│      - Eviction: N/A                                        │
│      - Latency: < 100 ms                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 10.2 Cache Lookup Flow

```text
Tile request
    │
    ▼
┌──────────┐    hit     ┌──────────┐
│ L0 GPU   │───────────►│ Return   │
└────┬─────┘            └──────────┘
     │ miss
     ▼
┌──────────┐    hit     ┌──────────┐
│ L1 CPU   │───────────►│ Promote  │──► Return
└────┬─────┘            │ to L0    │
     │ miss             └──────────┘
     ▼
┌──────────┐    hit     ┌──────────┐
│ L2 NVMe  │───────────►│ Load to  │──► Promote ──► Return
└────┬─────┘            │ L1       │
     │ miss             └──────────┘
     ▼
┌──────────┐    hit     ┌──────────┐
│ L3 Remote│───────────►│ Fetch to │──► Promote ──► Return
└────┬─────┘            │ L2       │
     │ miss             └──────────┘
     ▼
  ERROR: Tile not found
```

## 10.3 Cache Entry Structure

```rust
struct CacheEntry {
    tile_id: TileId,
    representation: RepresentationId,
    size_bytes: u64,
    level: CacheLevel,       // L0, L1, L2
    last_access: Instant,
    access_count: u64,
    priority: Priority,      // Pinned, High, Normal, Low
    state: TileState,        // Loading, Ready, Evicting
}
```

## 10.4 Eviction Policy

`[CACHE-1]` Eviction MUST berbasis byte capacity, bukan jumlah Tile.

`[CACHE-2]` Priority classes: `Pinned > High > Normal > Low`.

`[CACHE-3]` Dalam priority class sama, eviction menggunakan LRU.

`[CACHE-4]` Tile `Pinned` MUST NOT dievict selama pin aktif.

`[CACHE-5]` Eviction MUST async, tidak blocking execution.

## 10.5 Admission Policy

`[CACHE-6]` Tidak semua Tile yang dibaca MUST masuk cache.

`[CACHE-7]` Admission mempertimbangkan: frequency, recency, size, reuse distance.

```rust
struct AdmissionScore {
    reuse_probability: f32,
    load_cost: f32,
    size_cost: f32,
    execution_priority: f32,
}

fn should_admit(score: AdmissionScore) -> bool {
    score.reuse_probability * score.execution_priority > 
    score.load_cost * score.size_cost
}
```

## 10.6 Cache Coherency

`[CACHE-8]` Tile immutable → tidak ada invalidation karena modifikasi.

`[CACHE-9]` Invalidation hanya terjadi pada: eviction, memory pressure, atau shutdown.

`[CACHE-10]` Cache key MUST `(TileId, RepresentationId)`.

---

# 11. Memory Hierarchy

## 11.1 System Memory Model

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS MEMORY MODEL                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ GPU VRAM                                             │   │
│  │   - Active Tiles (execution)                         │   │
│  │   - Working state tensors                            │   │
│  │   - Budget: hard limit                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ CPU RAM                                              │   │
│  │   - Hot Tile cache                                   │   │
│  │   - Working memory (LATTICE)                         │   │
│  │   - Manifest index                                   │   │
│  │   - Cell index                                       │   │
│  │   - Budget: hard limit                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ NVMe / SSD                                           │   │
│  │   - .cd store (canonical)                            │   │
│  │   - Segments                                         │   │
│  │   - Memory cells (persistent)                        │   │
│  │   - Budget: disk capacity                            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Network / Object Storage                             │   │
│  │   - Remote Tile pool                                 │   │
│  │   - Shared model registry                            │   │
│  │   - Budget: unbounded                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 11.2 Memory Budget

```rust
struct MemoryBudget {
    // GPU
    gpu_total: u64,
    gpu_reserved: u64,        // for runtime, not Tiles
    gpu_tile_budget: u64,     // gpu_total - gpu_reserved
    
    // CPU
    cpu_total: u64,
    cpu_reserved: u64,
    cpu_tile_budget: u64,
    cpu_working_memory: u64,  // LATTICE working state
    
    // NVMe
    nvme_total: u64,
    nvme_store_budget: u64,
}
```

`[MEM-1]` Setiap level memori MUST memiliki hard budget.

`[MEM-2]` Budget MUST di-enforce oleh admission control.

`[MEM-3]` Pelanggaran budget MUST memicu eviction atau rejection.

## 11.3 Memory Ownership

| Data | Owner | Location |
|---|---|---|
| Manifest | Manifest Authority | CPU RAM |
| Cell Index | Cell Resolver | CPU RAM |
| Tile Registry | Storage Engine | CPU RAM |
| Cache Entries | Cache Manager | GPU/CPU/NVMe |
| Working State | Execution Engine | GPU/CPU |
| Memory Entries | Memory System | CPU/NVMe |
| Segments | Storage Engine | NVMe |
| Revision DAG | Revision DAG | CPU/NVMe |

`[MEM-4]` Setiap data MUST memiliki owner tunggal.

`[MEM-5]` Mutasi hanya boleh dilakukan oleh owner.

---

# 12. Failure Boundaries

## 12.1 Failure Domain Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    FAILURE DOMAINS                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Domain 1: Conversion                                       │
│    Failure: import error, corrupt source                    │
│    Impact: conversion fails, no partial store               │
│    Recovery: retry conversion                               │
│                                                             │
│  Domain 2: Storage I/O                                      │
│    Failure: disk error, segment corrupt                     │
│    Impact: Tile unavailable                                 │
│    Recovery: quarantine, reload from replica                │
│                                                             │
│  Domain 3: Integrity                                        │
│    Failure: BLAKE3 mismatch                                 │
│    Impact: Tile rejected                                    │
│    Recovery: quarantine, alert                              │
│                                                             │
│  Domain 4: Cache                                            │
│    Failure: cache corruption, eviction error                │
│    Impact: cache miss, reload                               │
│    Recovery: rebuild cache                                  │
│                                                             │
│  Domain 5: Execution                                        │
│    Failure: budget exceeded, halt condition                 │
│    Impact: partial result                                   │
│    Recovery: return partial, log                            │
│                                                             │
│  Domain 6: Revision                                         │
│    Failure: merge conflict, commit failure                  │
│    Impact: revision not committed                           │
│    Recovery: rollback, manual resolution                    │
│                                                             │
│  Domain 7: Network (optional)                               │
│    Failure: remote unavailable                              │
│    Impact: remote Tiles unavailable                         │
│    Recovery: fallback to local                              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 12.2 Failure Boundary Rules

`[FAIL-1]` Kegagalan satu domain MUST NOT merambat ke domain lain tanpa kontrol.

`[FAIL-2]` Setiap domain MUST memiliki mekanisme recovery sendiri.

`[FAIL-3]` Kegagalan integrity MUST selalu fatal untuk Tile yang bersangkutan.

`[FAIL-4]` Kegagalan I/O MAY di-retry dengan backoff.

`[FAIL-5]` Kegagalan execution MUST menghasilkan partial result atau error eksplisit.

## 12.3 Error Propagation

```text
Error terjadi di Storage Engine
    │
    ├─► Tile corrupt → Integrity error
    │       │
    │       ├─► Quarantine Tile
    │       ├─► Notify via Event Bus
    │       └─► Return error to caller
    │
    ├─► I/O error → Transient error
    │       │
    │       ├─► Retry (up to N times)
    │       └─► If still fails → propagate
    │
    └─► Manifest error → Fatal store error
            │
            ├─► Stop store
            └─► Require manual intervention
```

`[FAIL-6]` Error MUST diklasifikasikan: transient, permanent, fatal.

`[FAIL-7]` Transient error MAY di-retry.

`[FAIL-8]` Permanent error MUST dilaporkan ke caller.

`[FAIL-9]` Fatal error MUST menghentikan operasi dan memerlukan intervensi.

## 12.4 Crash Recovery

```text
Crash
  │
  ▼
Restart
  │
  ▼
Recovery Subsystem
  │
  ├─► Read journal
  │
  ├─► Detect incomplete commits
  │
  ├─► Replay or rollback
  │
  ├─► Verify store consistency
  │
  └─► Resume normal operation
```

`[FAIL-10]` Recovery MUST idempotent.

`[FAIL-11]` Recovery MUST NOT kehilangan committed data.

---

# 13. Lifecycle tiap Subsystem

## 13.1 Lifecycle State Machine Umum

```text
┌──────────┐   init    ┌──────────┐   start   ┌──────────┐
│ CREATED  │──────────►│ INITIALIZED │────────►│ RUNNING  │
└──────────┘           └──────────┘           └────┬─────┘
                                                   │
                                    ┌──────────────┼──────────────┐
                                    │              │              │
                                    ▼              ▼              ▼
                              ┌──────────┐  ┌──────────┐  ┌──────────┐
                              │ PAUSED   │  │ DEGRADED │  │ STOPPING │
                              └────┬─────┘  └────┬─────┘  └────┬─────┘
                                   │             │              │
                                   └─────────────┴──────────────┘
                                                 │
                                                 ▼
                                          ┌──────────┐
                                          │ STOPPED  │
                                          └──────────┘
```

## 13.2 Lifecycle: Conversion Pipeline

```text
States:
  IDLE → CONVERTING → FINALIZING → COMMITTED → IDLE
              │
              └──► FAILED → IDLE

Transitions:
  IDLE + convert() → CONVERTING
  CONVERTING + all_tiles_done → FINALIZING
  FINALIZING + manifest_built → COMMITTED
  COMMITTED → IDLE
  CONVERTING + error → FAILED
  FAILED + reset → IDLE
```

`[LIFE-CONV-1]` Conversion MUST atomic: berhasil penuh atau gagal penuh.

`[LIFE-CONV-2]` Partial conversion MUST NOT meninggalkan store inconsistent.

## 13.3 Lifecycle: Storage Engine

```text
States:
  CLOSED → OPENING → OPEN → CLOSING → CLOSED
                       │
                       └──► ERROR → CLOSED

Transitions:
  CLOSED + open() → OPENING
  OPENING + loaded → OPEN
  OPEN + close() → CLOSING
  CLOSING + flushed → CLOSED
  OPEN + fatal_error → ERROR
  ERROR + recovery → CLOSED
```

`[LIFE-STOR-1]` Store MUST di-open sebelum operasi.

`[LIFE-STOR-2]` Store MUST di-flush sebelum close.

## 13.4 Lifecycle: Cache Manager

```text
States:
  UNINITIALIZED → INITIALIZING → ACTIVE → DRAINING → SHUTDOWN
                                      │
                                      └──► EVICTING → ACTIVE

Transitions:
  UNINITIALIZED + init() → INITIALIZING
  INITIALIZING + ready → ACTIVE
  ACTIVE + evict() → EVICTING → ACTIVE
  ACTIVE + shutdown() → DRAINING
  DRAINING + flushed → SHUTDOWN
```

`[LIFE-CACHE-1]` Cache MUST di-initialize dengan budget eksplisit.

`[LIFE-CACHE-2]` Shutdown MUST flush atau evict semua entry.

## 13.5 Lifecycle: Execution Engine

```text
States per execution:
  PENDING → ENCODING → ROUTING → EXECUTING → COMPOSING → HALTED → DECODING → DONE
                                     │
                                     └──► BUDGET_EXCEEDED → DONE (partial)

Transitions:
  PENDING + input → ENCODING
  ENCODING + state_ready → ROUTING
  ROUTING + cells_selected → EXECUTING
  EXECUTING + outputs → COMPOSING
  COMPOSING + not_halt → ROUTING (loop)
  COMPOSING + halt → HALTED
  HALTED → DECODING
  DECODING → DONE
```

`[LIFE-EXEC-1]` Setiap eksekusi MUST memiliki budget.

`[LIFE-EXEC-2]` Budget exceeded MUST menghasilkan partial result atau error.

## 13.6 Lifecycle: Revision

```text
States:
  DRAFT → COMMITTED → ACTIVE → SUPERSEDED
              │
              └──► CONFLICT → RESOLVED → COMMITTED

Transitions:
  DRAFT + commit() → COMMITTED
  COMMITTED + set_active() → ACTIVE
  ACTIVE + new_revision → SUPERSEDED
  DRAFT + merge_conflict → CONFLICT
  CONFLICT + resolve() → RESOLVED
  RESOLVED + commit() → COMMITTED
```

`[LIFE-REV-1]` Revision committed MUST immutable.

`[LIFE-REV-2]` Active revision MUST hanya satu.

## 13.7 Lifecycle: Tile

```text
States:
  NOT_LOADED → PREFETCHING → LOADING → CPU_CACHED → GPU_CACHED → ACTIVE → EVICTING → EVICTED
                                                                       │
                                                                       └──► CORRUPT → QUARANTINED

Transitions:
  NOT_LOADED + prefetch → PREFETCHING
  NOT_LOADED + request → LOADING
  PREFETCHING + loaded → CPU_CACHED
  LOADING + loaded → CPU_CACHED
  CPU_CACHED + h2d → GPU_CACHED
  GPU_CACHED + execute → ACTIVE
  ACTIVE + evict → EVICTING → EVICTED
  any + integrity_fail → CORRUPT → QUARANTINED
```

`[LIFE-TILE-1]` Tile state MUST dapat diobservasi.

`[LIFE-TILE-2]` Tile corrupt MUST dikarantina.

---

# 14. Cross-Cutting Concerns

## 14.1 Observability

`[OBS-1]` Setiap komponen MUST menghasilkan log.

`[OBS-2]` Setiap komponen SHOULD menghasilkan metrics.

`[OBS-3]` Setiap komponen SHOULD menghasilkan trace spans.

### 14.1.1 Metrics per Komponen

| Komponen | Metrics |
|---|---|
| Conversion | bytes_processed, tiles_written, tiles_dedup, duration |
| Storage | segments_total, tiles_total, read_latency, write_latency |
| Cache | hit_rate, miss_rate, evictions, admissions, usage_bytes |
| Execution | composition_depth, cells_active, budget_used, halt_reason |
| Memory | entries_total, retrieves, stores, consolidations |
| Revision | revisions_total, commits, branches, merges, conflicts |
| Integrity | verifications, failures, quarantines |
| GC | tiles_reclaimed, bytes_reclaimed, duration |

## 14.2 Error Handling

`[ERR-1]` Setiap komponen MUST menggunakan error code namespace `CNWS-E-*`.

`[ERR-2]` Error MUST mengandung konteks komponen.

`[ERR-3]` Error MUST dapat di-propagate melalui SLI.

## 14.3 Configuration

`[CONF-1]` Konfigurasi MUST terpusat.

`[CONF-2]` Setiap komponen MUST membaca konfigurasi dari sumber terpusat.

`[CONF-3]` Konfigurasi runtime MAY diubah tanpa restart (untuk parameter tertentu).

---

# 15. Final DAS Contract

## 15.1 Ringkasan Keputusan Arsitektural

| ID | Keputusan |
|---|---|
| DAS-F01 | CNWS diorganisasikan menjadi dua lapisan internal: Substrate dan Lattice. |
| DAS-F02 | Substrate dan Lattice berkomunikasi hanya melalui SLI. |
| DAS-F03 | Lattice boleh bergantung pada Substrate; sebaliknya dilarang. |
| DAS-F04 | Dependency graph antar modul MUST acyclic. |
| DAS-F05 | Setiap modul memiliki public interface eksplisit. |
| DAS-F06 | Threading menggunakan pool terpisah untuk execution, I/O, prefetch, conversion, maintenance. |
| DAS-F07 | Async I/O menggunakan queue per tahap. |
| DAS-F08 | Cache hierarchy: L0 GPU, L1 CPU, L2 NVMe, L3 Remote. |
| DAS-F09 | Eviction berbasis byte capacity dan priority. |
| DAS-F10 | Setiap level memori memiliki hard budget. |
| DAS-F11 | Failure diisolasi per domain. |
| DAS-F12 | Setiap subsystem memiliki lifecycle state machine. |
| DAS-F13 | Observability terintegrasi di semua komponen. |
| DAS-F14 | Error menggunakan namespace `CNWS-E-*`. |
| DAS-F15 | Konfigurasi terpusat. |

## 15.2 Invariant DAS

| ID | Invariant |
|---|---|
| DAS-INV-1 | Tidak ada dependency cycle antar modul. |
| DAS-INV-2 | Substrate tidak pernah memanggil Lattice langsung. |
| DAS-INV-3 | Tile data masuk ke Lattice hanya melalui Cache Manager. |
| DAS-INV-4 | Learning updates masuk ke Substrate hanya melalui SLI. |
| DAS-INV-5 | Main thread tidak melakukan blocking I/O. |
| DAS-INV-6 | Lock hierarchy dipatuhi untuk mencegah deadlock. |
| DAS-INV-7 | Async I/O queue bounded. |
| DAS-INV-8 | Cache eviction tidak melanggar pinned Tiles. |
| DAS-INV-9 | Memory budget hard-enforced. |
| DAS-INV-10 | Failure satu domain tidak merambat tak terkendali. |
| DAS-INV-11 | Recovery idempotent. |
| DAS-INV-12 | Lifecycle state machine dipatuhi. |

## 15.3 Pernyataan Penutup

Dokumen DAS ini adalah **blueprint teknis final dan mengikat** untuk struktur internal CNWS. Ia menjelaskan komponen mana melakukan apa, bagaimana mereka berinteraksi, dan bagaimana sistem berperilaku dalam kondisi normal dan gagal.

Seluruh implementasi modul CNWS MUST conformant terhadap DAS ini.

Tidak ada keputusan arsitektural internal yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN DAS**
