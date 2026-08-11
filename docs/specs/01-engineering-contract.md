# CNWS
## Engineering Contract — Final Technical & Architecture Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Engineering Contract — Final Technical & Architecture Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (BINDING ENGINEERING CONTRACT)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Otoritas | Single Source of Truth untuk seluruh produk CNWS |
| Perubahan | Hanya melalui amandemen eksplisit terhadap dokumen ini |

---

# 0. Document Control

## 0.1 Sifat Dokumen

Dokumen ini adalah **Engineering Contract final dan mengikat** untuk CNWS. Ia bukan roadmap, bukan phased plan, bukan proposal, bukan brainstorm, dan bukan dokumen konseptual. Seluruh ketentuan mengikat secara teknis dan wajib dipatuhi oleh seluruh implementasi, code review, testing, validation, interoperability, dan maintenance CNWS.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, dan **OPTIONAL** diinterpretasikan sesuai RFC 2119.

- **Normatif**: seluruh teks ber-keyword RFC 2119, tabel berlabel **NORMATIVE**, invariant ber-ID (`[XXX-n]`, `FAC-*`), dan requirement ber-ID.
- **Informatif**: teks berlabel **"Catatan informatif"** dan contoh berlabel **"Contoh informatif"** tidak menambah requirement baru.

**Urutan prioritas bila terjadi konflik** (tertinggi → terendah):
1. Final Normative Architecture Contract (`FAC-*`, §24)
2. Subsystem Invariants (`[XXX-n]`)
3. Architecture Specification
4. Explanatory / contoh

## 0.3 Keputusan Engineering Final (DF)

Seluruh keputusan ditetapkan final melalui `DF-*`. Tidak ada open question.

| ID | Keputusan Final |
|---|---|
| DF-01 | Nama paradigma dan produk tunggal adalah **CNWS**. |
| DF-02 | Unit fundamental universal CNWS adalah **Cell**. |
| DF-03 | Unit physical storage adalah **Tile**. Cell disimpan sebagai satu atau lebih Tile. |
| DF-04 | Content addressing tunggal adalah **BLAKE3-256** untuk seluruh Cell, Tile, Memory, Revision. |
| DF-05 | Canonical store adalah **`.cd`** dengan `MANIFEST.cd` sebagai root source of truth. |
| DF-06 | `.cd` menyimpan Cell, Memory, Routing, Composition, dan Provenance — bukan hanya weight. |
| DF-07 | Versioning menggunakan **Revision DAG** yang mencakup seluruh perubahan state. |
| DF-08 | Conversion menggunakan **Streaming-First Pipeline** dengan bounded memory. |
| DF-09 | Runtime adalah **CNWS Execution Engine** dengan dynamic & adaptive execution. |
| DF-10 | Memory adalah **first-class persistent state**, bukan cache sementara. |
| DF-11 | Compute dialokasikan secara **adaptif** berdasarkan difficulty. |
| DF-12 | Learning bersifat **structural & incremental**, bukan global parameter update. |
| DF-13 | Zero Format Coupling berlaku: runtime tidak memahami format checkpoint eksternal. |

---

# 1. Executive Definition

**CNWS** adalah canonical intelligence infrastructure yang merepresentasikan knowledge, computation, memory, routing, dan state sebagai **Cell content-addressed** dalam satu canonical `.cd` store, menyediakan streaming conversion, dynamic adaptive execution, persistent memory, structural learning, dan incremental versioning melalui satu paradigma terpadu.

**Definisi satu kalimat:**

> CNWS adalah streaming canonical-intelligence architecture yang merepresentasikan seluruh state sistem sebagai Cell content-addressed dengan BLAKE3-256, memisahkan semantic Cell graph dari physical Tile storage, menyediakan dynamic adaptive execution dan persistent memory, serta incremental revision melalui satu canonical manifest.

## 1.1 Karakteristik Paradigma

CNWS didefinisikan oleh sepuluh karakteristik yang saling menguatkan:

| # | Karakteristik | Makna |
|---|---|---|
| 1 | Content-Addressed Intelligence | Seluruh knowledge/computation diidentifikasi oleh kontennya (BLAKE3-256) |
| 2 | Cell sebagai Unit Universal | Satu primitive untuk weight, memory, routing, composition |
| 3 | Dynamic Computation | Kedalaman dan lebar komputasi ditentukan oleh input, bukan arsitektur tetap |
| 4 | Persistent First-Class Memory | Memory adalah bagian dari model, bukan cache sementara |
| 5 | Adaptive Compute | Compute dialokasikan berdasarkan difficulty |
| 6 | Structural Learning | Learning menambah/memodifikasi Cell, bukan update global |
| 7 | Incremental Versioning | Revision DAG menyimpan delta, bukan full copy |
| 8 | Zero Format Coupling | Runtime independen dari format checkpoint eksternal |
| 9 | Streaming-First Conversion | Conversion bounded-memory |
| 10 | Tile-Based Physical Storage | Unit storage immutable, deduplicable, cacheable |

`[EXEC-1]` Kesepuluh karakteristik MUST hadir secara bersama-sama dalam implementasi CNWS.

`[EXEC-2]` Tidak ada implementasi sah CNWS yang menghilangkan salah satu karakteristik.

---

# 2. Normative Principles

## 2.1 Prinsip Arsitektural

| ID | Prinsip | Status |
|---|---|---|
| AP-1 | Streaming-First Pipeline | Normative |
| AP-2 | Cell Graph sebagai semantic layer | Normative |
| AP-3 | Tile-Based Physical Storage | Normative |
| AP-4 | BLAKE3-256 Content Addressing | Normative |
| AP-5 | Canonical `.cd` sebagai source of truth | Normative |
| AP-6 | Zero Format Coupling | Normative |
| AP-7 | Incremental Revision DAG | Normative |
| AP-8 | Content-Addressed Computation | Normative |
| AP-9 | Persistent First-Class Memory | Normative |
| AP-10 | Adaptive-Depth Composition | Normative |
| AP-11 | Structural Learning | Normative |
| AP-12 | Adaptive Compute | Normative |

`[PRIN-1]` Implementasi MUST NOT melanggar salah satu dari AP-1 sampai AP-12.

## 2.2 Prinsip Efisiensi

`[AXIOM-1]` Biaya intelligence diukur dalam **bytes moved per useful computation**, bukan FLOPs.

`[AXIOM-2]` Knowledge yang tidak relevan MUST NOT dipindahkan ke active memory.

`[AXIOM-3]` Computation yang tidak berkontribusi pada output MUST NOT dieksekusi.

`[AXIOM-4]` Total stored intelligence dapat tumbuh tanpa linear growth pada compute per token.

---

# 3. Scope & Non-Scope

## 3.1 Scope

| Scope | Status |
|---|---|
| Import checkpoint eksternal ke canonical `.cd` | In scope |
| Streaming conversion bounded-memory | In scope |
| Cell-based canonical storage | In scope |
| Content addressing & deduplication | In scope |
| Dynamic adaptive execution | In scope |
| Persistent memory first-class | In scope |
| Selective & adaptive loading | In scope |
| Revision DAG (Cell, Memory, Routing, Composition) | In scope |
| Structural/incremental learning | In scope |
| Integrity verification | In scope |
| Observability & conformance testing | In scope |

## 3.2 Non-Scope

| Non-Scope | Status |
|---|---|
| Training engine / optimizer eksternal | Out of scope |
| Inference kernel implementation | Out of scope (CNWS menyediakan Cell, bukan kernel) |
| Distributed cluster scheduler | Out of scope untuk inti |
| Tokenizer training | Out of scope |
| Encryption-at-rest mandatory | Out of scope (MAY optional) |

---

# 4. Unified Terminology

| Term | Definisi Normatif |
|---|---|
| **CNWS** | Paradigma dan produk canonical intelligence infrastructure. |
| **Cell** | Unit fundamental universal: knowledge/computation content-addressed. |
| **Tile** | Unit physical storage immutable untuk Cell. |
| **Cell Graph** | Graph semantic dari Cell dan asosiasinya. |
| **Query** | Primitive seleksi content-addressed. |
| **MemoryEntry** | Unit persistent learned information, disimpan sebagai Cell. |
| **WorkingState** | State komputasi aktif yang bounded. |
| **Update** | Unit atomic learning. |
| **Revision** | Snapshot immutable mapping Cell/Tile. |
| **`.cd`** | Canonical store directory. |
| **MANIFEST.cd** | Root canonical manifest. |
| **Execution Engine** | Runtime CNWS yang melakukan dynamic adaptive execution. |
| **Conversion Pipeline** | Streaming-first import dari format eksternal. |
| **Storage Engine** | Subsystem penyimpanan, commit, recovery, GC. |

---

# 5. System Boundaries & Trust Model

## 5.1 Arsitektur Boundary

```text
        EXTERNAL WORLD                    CNWS WORLD
 ┌─────────────────────┐    ┌──────────────────────────────┐
 │ Safetensors         │    │   CNWS                       │
 │ GGUF                │───►│   Conversion Pipeline        │
 │ PyTorch             │    │   Storage Engine             │
 │ Custom              │    │   Execution Engine           │
 └─────────────────────┘    │   Memory System              │
                            │   Revision DAG               │
                            └──────────────┬───────────────┘
                                           │
                                           ▼
                                EXECUTION WORLD
                             CPU / GPU / NVMe / Remote
```

## 5.2 Trust Model

| Entity | Trust Level | Perlakuan |
|---|---|---|
| External checkpoint | Untrusted | Validasi, no code execution |
| PyTorch pickle | Untrusted | Restricted/safe unpickler |
| Cell setelah import | Trusted after verification | BLAKE3 verification |
| MANIFEST.cd | Trusted after integrity check | Hash divergence = fatal |
| Remote Cell source | Untrusted until verified | BLAKE3 verification |

`[SEC-1]` Conversion layer MUST NOT mengeksekusi kode dari checkpoint.

`[SEC-2]` PyTorch importer MUST menggunakan restricted unpickler.

`[SEC-3]` Path traversal dari metadata checkpoint MUST dicegah.

---

# 6. Architectural Overview

## 6.1 Satu Paradigma Terpadu

```text
                          CNWS
│
├──────────────────────────────────────────────┐
│                                              │
LOGICAL LAYER                    PHYSICAL LAYER
│                                              │
Cell Graph                       Tile Storage
│                                              │
Cell                             BLAKE3-256
│                                              │
Query / Routing                  Segment
│                                              │
Memory                           .cd Store
│                                              │
Dynamic Execution                Revision DAG
│                                              │
Adaptive Compute                 GC / Recovery
│                                              │
└──────────────────┬───────────────────────────┘
                   │
        CANONICAL MANIFEST (.cd/MANIFEST.cd)
                   │
        ┌──────────┴──────────┐
        │                     │
  Execution Engine      Storage Engine
  (resolve/execute)     (store/load/version)
        │
   Cache / NVMe / GPU
```

## 6.2 Subsystem Ownership

| Subsystem | Responsibility |
|---|---|
| Conversion Pipeline | Import, normalize, tile, hash, write |
| Storage Engine | Segment, Tile registry, commit, recovery, GC |
| Manifest Authority | Schema, versioning, integrity |
| Revision DAG | Branch, merge, rollback, resolution |
| Integrity Subsystem | BLAKE3 verification |
| Cell Resolver | Cell/Tile resolution |
| Query Engine | Content-addressed selection |
| Execution Engine | Dynamic composition, adaptive compute |
| Memory System | Persistent memory hierarchy |
| Routing Engine | Cell selection policy |
| Learning Engine | Structural/incremental updates |
| Cache Manager | GPU/CPU/NVMe hierarchy |
| Prefetch Engine | Async load planning |

---

# 7. Fundamental Primitives

CNWS didefinisikan oleh enam primitive fundamental. Seluruh perilaku sistem muncul dari interaksi mereka.

| Primitive | Nama | Peran |
|---|---|---|
| Knowledge/Computation | **Cell** | Unit komputasi content-addressed immutable |
| Selection | **Query** | Mekanisme retrieval content-addressed |
| Composition | **Cell Graph** | Graph dinamis penghubung Cell |
| Persistence | **MemoryEntry** | Informasi learned persistent |
| State | **WorkingState** | Konteks komputasi aktif bounded |
| Learning | **Update** | Mekanisme modifikasi state sistem |

## 7.1 Cell

```rust
struct Cell {
    id: [u8; 32],             // BLAKE3-256 content address
    cell_type: CellType,
    input_schema: Schema,
    output_schema: Schema,
    tiles: Vec<TileRef>,      // physical storage
    index_vector: Vec<f32>,   // content embedding for retrieval
    dependencies: Vec<CellId>,
    metadata: CellMetadata,
}
```

`[CELL-1]` Setiap Cell MUST memiliki BLAKE3-256 identity unik.

`[CELL-2]` Cell MUST immutable setelah dibuat.

`[CELL-3]` Cell identity MUST independen dari lokasi storage, compression, dan representation.

`[CELL-4]` Cell MUST dapat di-load secara independen.

`[CELL-5]` Cell size MUST dalam range [64 KiB, 512 MiB]. Default target: 4 MiB.

### 7.1.1 CellType

```rust
enum CellType {
    // Weight cells
    EMBEDDING,
    ATTENTION_Q_PROJ,
    ATTENTION_K_PROJ,
    ATTENTION_V_PROJ,
    ATTENTION_OUT,
    MLP_GATE,
    MLP_UP,
    MLP_DOWN,
    EXPERT_GATE,
    EXPERT_ROUTE,
    EXPERT_WEIGHT,
    LAYERNORM_WEIGHT,
    LAYERNORM_BIAS,
    LM_HEAD,
    VISION_ENCODER,

    // Memory cells
    MEMORY_EPISODIC,
    MEMORY_SEMANTIC,
    MEMORY_PROCEDURAL,

    // Routing & composition cells
    ROUTING_POLICY,
    COMPOSITION_PATTERN,

    // Computation cells
    TRANSFORM_MODULE,
    ENCODE_MODULE,
    DECODE_MODULE,

    CUSTOM(String),
}
```

`[CELL-6]` CellType MUST semantic dan tidak terikat pada satu arsitektur.

## 7.2 Query

```rust
struct Query {
    vector: Vec<f32>,
    max_cells: usize,
    threshold: f32,
    budget: ComputeBudget,
}
```

`[QUERY-1]` Seleksi Cell MUST content-based, bukan position-based.

`[QUERY-2]` Seleksi MUST O(log N) atau O(1) terhadap total Cell N.

`[QUERY-3]` Seleksi MUST mendukung multi-scale retrieval.

## 7.3 WorkingState

```rust
struct WorkingState {
    active_vector: Vec<f32>,
    context_entries: Vec<CellRef>,
    current_cells: Vec<CellRef>,
    compute_used: u64,
    bytes_moved: u64,
    steps_taken: u32,
    halt_signal: bool,
    confidence: f32,
}
```

`[STATE-1]` WorkingState MUST bounded.

`[STATE-2]` WorkingState MUST serializable untuk pause/resume.

`[STATE-3]` Context MUST NOT tumbuh linear terhadap sequence length.

## 7.4 Update

```rust
enum Update {
    CellCreate(Cell),
    CellRefine(CellId, CellId),   // (old, new)
    RoutingStrengthen(CellId, CellId, f32),
    MemoryWrite(MemoryEntry),
    MemoryConsolidate(Vec<CellId>),
    CompositionCache(CompositionPattern),
}
```

`[UPDATE-1]` Update MUST atomic dan content-addressed.

`[UPDATE-2]` Update MUST NOT memodifikasi Cell yang tidak terkait.

`[UPDATE-3]` Update MUST versioned melalui Revision DAG.

---

# 8. Canonical Data Model

## 8.1 Cell Graph

```rust
struct CellGraph {
    version: String,
    model_id: String,
    cells: Vec<Cell>,
    dependencies: DependencyGraph,
    architecture: ArchitectureMetadata,
    memory_index: MemoryIndex,
    routing: RoutingMetadata,
    metadata: GraphMetadata,
}
```

`[CMG-1]` CellGraph adalah abstraction dari intelligence, bukan dari file checkpoint.

`[CMG-2]` Cell MUST memiliki semantic identity stabil.

`[CMG-3]` Cell MUST NOT bergantung pada filename shard, byte offset, atau physical location.

`[CMG-4]` CellGraph MUST menyimpan dependency graph.

`[CMG-5]` CellGraph MUST menyimpan architecture metadata.

## 8.2 CellId Grammar

```ebnf
cell_id   = segment, { ".", segment } ;
segment   = alpha_segment | index_segment ;
alpha_segment = ( "_" | lowercase ), { lowercase | digit | "_" } ;
index_segment = digit, { digit } ;
```

`[CELLID-1]` CellId MUST lowercase.

`[CELLID-2]` CellId MUST NOT mengandung spasi, uppercase, atau `..`.

`[CELLID-3]` CellId maksimum 512 karakter.

Contoh normative:

```text
model.embedding.token_embedding
model.layer.0.self_attn.q_proj
model.layer.0.moe.expert.7
memory.episodic.ctx_0001
routing.policy.v42
composition.attn_mlp_fused
```

## 8.3 TileRef

```rust
struct TileRef {
    tile_id: Blake3Hash,
    shape: Vec<u64>,
    offset: Vec<u64>,
    size: Vec<u64>,
    segment_id: SegmentId,
}
```

`[TREF-1]` `tile_id` MUST BLAKE3-256 canonical payload.

`[TREF-2]` `offset` dan `size` dalam elemen, bukan byte.

`[TREF-3]` TileRef MUST NOT menyimpan byte offset dari format sumber.

## 8.4 Dependency Graph

```rust
struct DependencyGraph {
    edges: HashMap<CellId, Vec<CellId>>,
}
```

`[DEP-1]` Dependency graph MUST digunakan untuk execution ordering dan prefetch planning.

`[DEP-2]` Dependency graph MUST NOT mengandung cycle.

## 8.5 Tile Invariants

| ID | Invariant |
|---|---|
| TILE-1 | Tile immutable |
| TILE-2 | Tile content-addressed |
| TILE-3 | Tile identity = BLAKE3-256 |
| TILE-4 | Tile independently readable |
| TILE-5 | Tile independently cacheable |
| TILE-6 | Tile independently deduplicable |
| TILE-7 | Tile independently versionable |
| TILE-8 | Tile independent from source format |

---

# 9. BLAKE3-256 Content Addressing

## 9.1 Spesifikasi Inti

| Property | Value |
|---|---|
| Algorithm | BLAKE3 |
| Digest | 256-bit / 32 bytes |
| Identity source | Canonical uncompressed payload |
| Encoding | lowercase hexadecimal |
| Prefix | `b3:` |
| Mutability | immutable |
| Compression | independent of identity |
| Storage location | independent of identity |
| SHA-256 | not used |

`[HASH-1]` Seluruh entitas content-addressed (Cell, Tile, Memory, Revision) MUST menggunakan BLAKE3-256.

`[HASH-2]` Identity dihitung dari canonical uncompressed payload.

`[HASH-3]` Compression MUST NOT mengubah identity.

`[HASH-4]` Hash MUST streaming, tidak membutuhkan buffer duplikat penuh.

`[HASH-5]` same ID + different payload MUST fatal integrity error.

## 9.2 Streaming Hash

```rust
let mut hasher = blake3::Hasher::new();
while let Some(chunk) = reader.read_chunk()? {
    hasher.update(&chunk);
    process_chunk(chunk)?;
}
let digest = hasher.finalize();
```

`[HASH-6]` Hashing MUST streaming untuk bounded memory.

---

# 10. `.cd` Format Specification

## 10.1 Store Layout

`[STORE-1]` `.cd` MUST berupa canonical store directory.

```text
model.cd/
├── SUPERBLOCK
├── LOCK
├── MANIFEST.cd
├── MANIFEST.cd.prev
├── journal/
│   └── commit.wal
├── staging/
│   └── manifest-<hash>.cd
├── index/
│   ├── cells.idx
│   ├── tiles.idx
│   ├── memory.idx
│   └── routing.idx
├── segments/
│   ├── segment-000001.cd
│   └── ...
├── lattice/
│   ├── graph.cd
│   ├── compositions.cd
│   └── routing_policy.cd
├── memory/
│   ├── episodic/
│   ├── semantic/
│   ├── procedural/
│   └── index.cd
├── revisions/
│   └── rev-<id>.json
├── corrupt/
│   └── <tile-id>.quarantine
└── meta/
    ├── provenance/
    └── routing_stats/
```

`[STORE-2]`.cd MUST menyimpan Cell, Memory, Routing, Composition, dan Provenance.

`[STORE-3]`.cd MUST NOT terbatas pada parameter weight saja.

`[STORE-4]`.cd adalah canonical persistent state dari seluruh sistem CNWS.

## 10.2 Serialization Rules

`[SER-1]` Seluruh binary integer MUST little-endian.

`[SER-2]` Seluruh JSON MUST UTF-8 dengan canonical serialization (sorted keys, no duplicate keys, finite numbers, NFC).

`[SER-3]` Timestamp MUST Unix nanoseconds UTC.

`[SER-4]` Hash direpresentasikan sebagai `b3:` + 64 lowercase hex.

`[SER-5]` Padding MUST zero-filled.

## 10.3 Versioning & Compatibility

`[VER-1]` `format_version` MUST semver.

`[VER-2]` Reader MUST menolak major lebih tinggi.

`[VER-3]` Reader MUST menerima minor ≤ versi didukung.

---

# 11. Conversion Pipeline

## 11.1 Streaming-First

`[CONV-1]` Conversion MUST memproses weight secara incremental.

`[CONV-2]` Conversion MUST bounded-memory.

`[CONV-3]` Peak RAM MUST NOT sebanding dengan total model size.

`[CONV-4]` Hashing MUST streaming.

## 11.2 Zero Format Coupling

```text
Safetensors ──┐
GGUF ─────────┤
PyTorch ──────┤
Custom ───────┘
      │
      ▼
FormatReader (conversion layer only)
      │
      ▼
Normalizer
      │
      ▼
Cell Planner
      │
      ▼
Streaming BLAKE3
      │
      ▼
Dedup / Write
      │
      ▼
Segment / Manifest Commit
```

`[CONV-5]` FormatReader MUST hidup hanya di conversion layer.

`[CONV-6]` Runtime MUST NOT memiliki dependency terhadap FormatReader.

## 11.3 Normalization

`[NORM-1]` Normalizer MUST memetakan tensor eksternal ke semantic CellId.

`[NORM-2]` Normalizer MUST deterministic.

`[NORM-3]` Normalizer MUST NOT bergantung pada urutan file shard sebagai semantic identity.

---

# 12. Storage Engine

## 12.1 Tanggung Jawab

Storage Engine bertanggung jawab atas:
1. Segment allocation
2. Tile writing
3. Tile registry
4. Atomic commit
5. Recovery
6. Garbage collection
7. Corruption quarantine

## 12.2 Tile Registry & Deduplication

```rust
struct TileRegistry {
    tiles: HashMap<Blake3Hash, TileLocation>,
}
```

`[DEDUP-1]` Jika Tile ID sudah ada dan payload sama, sistem MUST reuse.

`[DEDUP-2]` Jika Tile ID sama tetapi payload berbeda, sistem MUST fatal error.

`[DEDUP-3]` Deduplication MUST bekerja antar revision.

`[DEDUP-4]` Deduplication SHOULD bekerja antar model melalui global pool.

## 12.3 Atomic Commit

`[COMMIT-1]` Commit MUST menggunakan staging + journal + atomic rename + fsync.

`[COMMIT-2]` Commit MUST fsync file dan directory pada langkah kritis.

`[COMMIT-3]` Jika crash sebelum commit-complete, recovery MUST menyelesaikan atau membatalkan secara konsisten.

## 12.4 Garbage Collection

`[GC-1]` GC MUST berbasis reachability dari revision roots.

`[GC-2]` GC MUST NOT menghapus Cell/Tile reachable dari revision root mana pun.

`[GC-3]` GC MUST two-phase: mark then sweep.

## 12.5 Corruption Handling

`[COR-1]` Tile MUST diverifikasi BLAKE3-256 sebelum digunakan.

`[COR-2]` Jika verifikasi gagal, Tile MUST dipindahkan ke `corrupt/`.

`[COR-3]` Sistem MUST mengembalikan error `CNWS-E-CORRUPT`.

---

# 13. Execution Engine

## 13.1 Tujuan

Execution Engine memutuskan:
1. Cell apa yang dimuat.
2. Representation apa yang digunakan.
3. Kapan dimuat.
4. Kapan dibuang.
5. Di level cache mana ditempatkan.
6. Berapa banyak compute dialokasikan.

## 13.2 Dynamic Execution Loop

```pseudo
function execute(input, budget):
    state = Encode(input)
    steps = 0
    while not Halt(state) and steps < budget.max_depth:
        query = DeriveQuery(state)
        selected = Select(query, budget)
        if selected is empty:
            if budget.allow_growth:
                new_cell = CreateCell(state)
                selected = [new_cell]
            else:
                break
        outputs = Execute(selected, state)
        state = Compose(state, outputs)
        UpdateRouting(selected, state)
        steps += 1
    return Decode(state)
```

`[EXEC-1]` Kedalaman komputasi MUST adaptif terhadap input.

`[EXEC-2]` Tidak boleh ada fixed-depth layer stack.

`[EXEC-3]` Budget MUST hard-enforced.

## 13.3 Selective Loading Invariants

| ID | Invariant |
|---|---|
| RT-1 | Never require full-model residency |
| RT-2 | Load at Tile granularity |
| RT-3 | Resolve by semantic Cell ID |
| RT-4 | Select representation at runtime |
| RT-5 | Prefer asynchronous I/O |
| RT-6 | Prefetch based on execution dependency |
| RT-7 | MoE loads only selected experts |
| RT-8 | Enforce hard memory budgets |
| RT-9 | Cache by BLAKE3-256 Tile identity |
| RT-10 | Verify Tile integrity before execution |
| RT-11 | Keep storage format invisible to execution engine |

## 13.4 Cache Hierarchy

| Level | Media | Konten |
|---|---|---|
| L0 | GPU VRAM | Active execution Cells |
| L1 | CPU RAM | Hot Cell cache |
| L2 | NVMe | Local persistent `.cd` / staging |
| L3 | Network | Optional remote source |

`[CACHE-1]` Cache key MUST `(TileId, RepresentationId)`.

`[CACHE-2]` Eviction MUST berbasis byte capacity.

## 13.5 Adaptive Compute

`[ADAPT-1]` Alokasi compute MUST proporsional terhadap estimasi difficulty.

`[ADAPT-2]` Difficulty estimation MUST lightweight.

`[ADAPT-3]` Sistem MUST mendukung komputasi rekursif untuk masalah sulit.

---

# 14. Memory System

## 14.1 Prinsip

`[MEM-1]` Memory MUST first-class dan persistent.

`[MEM-2]` Working memory MUST bounded.

`[MEM-3]` Context MUST ditangani melalui memory content-addressed, bukan KV-cache.

`[MEM-4]` Context MUST NOT tumbuh linear terhadap sequence length.

## 14.2 Memory Hierarchy

```text
┌─────────────────────────────────────────────────┐
│ L0: Working Memory (bounded, active)            │
├─────────────────────────────────────────────────┤
│ L1: Hot Memory (recently accessed)              │
├─────────────────────────────────────────────────┤
│ L2: Warm Memory (persistent, local)             │
├─────────────────────────────────────────────────┤
│ L3: Cold Memory (persistent, remote)            │
└─────────────────────────────────────────────────┘
```

`[MEM-5]` Memory hierarchy MUST transparent terhadap computation layer.

`[MEM-6]` Memory entries MUST independently loadable.

## 14.3 Memory Types

```rust
enum MemoryType {
    Episodic,      // Specific experiences and sequences
    Semantic,      // Factual knowledge and associations
    Procedural,    // Learned composition patterns
    Working,       // Temporary active context (bounded)
    Consolidated,  // Compiled from frequently-accessed entries
}
```

---

# 15. Revision DAG

## 15.1 Revision Object

```rust
struct Revision {
    id: RevisionID,
    model_id: ModelID,
    revision_number: u64,
    parents: Vec<RevisionID>,
    root_manifest: ManifestID,
    changed_cells: Vec<CellID>,
    changed_tiles: Vec<TileID>,
    changed_memory: Vec<CellID>,
    changed_routing: Vec<CellID>,
    metadata: RevisionMetadata,
    created_at: Timestamp,
    author: Option<String>,
    message: Option<String>,
}
```

`[REV-1]` Revision MUST immutable.

`[REV-2]` Revision MUST mendukung multiple parents (merge).

`[REV-3]` Revision delta MUST pada level Cell/Tile, bukan full copy.

`[REV-4]` Cell yang tidak berubah MUST direferensikan dari ancestor.

## 15.2 Branching & Merge

`[REV-5]` Branching MUST NOT menyalin seluruh Cell fisik.

`[REV-6]` Merge MUST menggunakan three-way merge pada level Cell/Tile.

`[REV-7]` Merge conflict MUST dilaporkan eksplisit.

`[REV-8]` Rollback MUST NOT menghapus revision baru.

## 15.3 Learning sebagai Revision

`[REV-9]` Setiap learning update MUST menghasilkan revision baru.

`[REV-10]` Learning cost MUST O(affected_cells), bukan O(total_cells).

`[REV-11]` Learning MUST NOT catastrophic forgetting.

---

# 16. Learning Architecture

## 16.1 Prinsip

`[LEARN-1]` Learning MUST NOT memerlukan global parameter updates.

`[LEARN-2]` Learning cost MUST proportional terhadap apa yang berubah.

`[LEARN-3]` Learning MUST incremental.

`[LEARN-4]` Learning MUST preserve existing knowledge.

## 16.2 Learning Mechanisms

| Mechanism | What Changes | Cost | Scope |
|---|---|---|---|
| Cell Refinement | Parameters of existing Cell | O(cell_size) | Local |
| Cell Creation | New Cell added | O(cell_size) | Local |
| Routing Update | Association strengths | O(affected_edges) | Local |
| Consolidation | Multiple entries → single unit | O(entries) | Local |

## 16.3 Specialization Without Duplication

```text
Base Model (shared Cells)
    │
    ├── Domain A Specialization
    │   └── Adds: domain_A Cells, domain_A routing
    │   └── Shares: all base Cells
    │
    ├── Domain B Specialization
    │   └── Adds: domain_B Cells, domain_B routing
    │   └── Shares: all base Cells
```

`[SPEC-1]` Specialization MUST menambah domain-specific Cells tanpa copying base Cells.

`[SPEC-2]` Specialization MUST menggunakan Revision DAG untuk versioning.

`[SPEC-3]` Multiple specializations MUST share common Cells.

---

# 17. APIs

## 17.1 Public vs Internal

| Interface | Visibility |
|---|---|
| RuntimeResolver | Public |
| StorageEngine | Public |
| ConversionPipeline | Public |
| RevisionManager | Public |
| Segment allocator | Internal |
| Cache eviction internals | Internal |
| Journal internals | Internal |

## 17.2 RuntimeResolver API

```rust
trait RuntimeResolver {
    fn resolve_cell(&self, cell_id: &CellId) -> Result<CellHandle>;
    fn resolve_tiles(&self, cell: &CellHandle, policy: AccessPolicy)
        -> Result<Vec<TileHandle>>;
    fn select_representation(&self, tile: &TileRef) -> Result<RepresentationId>;
    async fn prefetch(&self, requests: &[PrefetchRequest]) -> Result<()>;
    fn release(&self, tile: TileHandle);
}
```

`[API-1]` Execution engine MUST hanya menggunakan RuntimeResolver untuk mendapatkan Cell.

`[API-2]` Execution engine MUST NOT mengakses `.cd` binary layout langsung.

## 17.3 StorageEngine API

```rust
trait StorageEngine {
    fn open_store(path: &Path) -> Result<Store>;
    fn import_tiles(&mut self, tiles: impl Iterator<Item = CanonicalTile>)
        -> Result<ImportReport>;
    fn lookup_tile(&self, tile_id: &TileId) -> Result<TileLocation>;
    fn read_tile(&self, tile_id: &TileId) -> Result<TileReader>;
    fn commit(&mut self, manifest: Manifest) -> Result<CommitReceipt>;
    fn recover(&mut self) -> Result<RecoveryReport>;
    fn gc(&mut self) -> Result<GcReport>;
}
```

---

# 18. Reliability & Security

`[INT-1]` Tile integrity MUST diverifikasi dengan BLAKE3-256.

`[INT-2]` Manifest integrity MUST diverifikasi dengan BLAKE3-256.

`[SEC-4]` Importer MUST melakukan bounds checking.

`[SEC-5]` Sistem MUST menolak path traversal.

`[SEC-6]` Remote fetch MUST menggunakan integrity verification.

`[PROV-1]` Setiap Cell SHOULD memiliki provenance record.

`[PROV-2]` Setiap Revision MUST memiliki provenance minimal timestamp dan parent.

---

# 19. Performance

| ID | Requirement |
|---|---|
| PERF-1 | Conversion MUST bounded-memory |
| PERF-2 | Cell resolve MUST O(1) setelah manifest loaded |
| PERF-3 | Tile lookup MUST O(1) |
| PERF-4 | Segment read MUST menggunakan index, bukan scan |
| PERF-5 | Runtime MUST mendukung overlap I/O, decompression, transfer, compute |
| PERF-6 | MoE loading MUST proportional terhadap selected experts |
| PERF-7 | Active parameter ratio MUST < 10% dari total |
| PERF-8 | Context memory MUST O(1) terhadap sequence length |
| PERF-9 | Bytes moved per token MUST tracked sebagai metrik utama |

---

# 20. Observability

`[OBS-1]` Sistem MUST menghasilkan logs.

`[OBS-2]` Sistem SHOULD menghasilkan metrics.

`[OBS-3]` Sistem SHOULD menghasilkan tracing spans.

Metrik minimum:

| Metric | Type |
|---|---|
| `cnws_cells_active` | Gauge |
| `cnws_cells_total` | Gauge |
| `cnws_bytes_moved_per_token` | Histogram |
| `cnws_composition_depth` | Histogram |
| `cnws_cache_hit_rate` | Gauge |
| `cnws_active_param_ratio` | Gauge |
| `cnws_context_memory_bytes` | Gauge |
| `cnws_revision_count` | Gauge |
| `cnws_corrupt_tiles_total` | Counter |

---

# 21. Error Model

| Code | Meaning |
|---|---|
| `CNWS-E-CORRUPT` | BLAKE3 mismatch / payload corrupt |
| `CNWS-E-MANIFEST` | Manifest invalid |
| `CNWS-E-SCHEMA` | Schema version incompatible |
| `CNWS-E-STORE` | Store I/O error |
| `CNWS-E-LOCK` | Lock conflict |
| `CNWS-E-REVISION` | Revision conflict/invalid |
| `CNWS-E-MERGE` | Merge conflict |
| `CNWS-E-BUDGET` | Memory/compute budget exceeded |
| `CNWS-E-NOTFOUND` | Cell/Tile/Revision not found |
| `CNWS-E-IMPORT` | Importer failure |
| `CNWS-E-UNSAFE` | Unsafe checkpoint content |
| `CNWS-E-RECOVERY` | Recovery failure |

---

# 22. Testing & Conformance

| Kategori | Mandatory |
|---|---|
| Unit tests | MUST |
| Conformance tests | MUST |
| Interoperability tests | MUST |
| Corruption/failure tests | MUST |
| Crash/recovery tests | MUST |
| Deduplication tests | MUST |
| Revision DAG tests | MUST |
| Runtime cache tests | MUST |
| MoE selective loading tests | MUST |
| Adaptive compute tests | MUST |
| Memory persistence tests | MUST |
| Performance benchmarks | SHOULD |
| Fuzzing importer | SHOULD |

`[TEST-1]` Implementasi MUST lulus conformance suite sebelum dianggap conformant.

`[TEST-2]` Conformance tests MUST otomatis dan repeatable.

`[TEST-3]` Kegagalan conformance test MUST memblokir deployment.

---

# 23. Concurrency

`[CONC-1]` Runtime MUST mendukung concurrent inference requests.

`[CONC-2]` Cell loading MUST thread-safe.

`[CONC-3]` Memory writes MUST atomic.

`[CONC-4]` Revision commits MUST menggunakan single-writer semantics.

`[CONC-5]` Cell execution MUST mendukung parallel execution of independent Cells.

`[CONC-6]` Cell reads MUST NOT require locking (immutable).

---

# 24. Final Normative Architecture Contract

Bagian ini adalah ringkasan final tertinggi. Jika ada bagian lain yang tampak bertentangan, bagian ini menang.

| ID | Final Invariant |
|---|---|
| FAC-1 | CNWS adalah satu paradigma terpadu. |
| FAC-2 | Unit fundamental universal adalah Cell. |
| FAC-3 | Tile adalah unit physical storage. |
| FAC-4 | Cell:Tile = satu-ke-banyak. |
| FAC-5 | Cell identity = BLAKE3-256 canonical payload. |
| FAC-6 | Cell dan Tile immutable. |
| FAC-7 | Content addressing tunggal BLAKE3-256 untuk semua entitas. |
| FAC-8 | `.cd` adalah canonical store dan source of truth. |
| FAC-9 | `.cd` menyimpan Cell, Memory, Routing, Composition, Provenance — bukan hanya weight. |
| FAC-10 | MANIFEST.cd adalah root manifest. |
| FAC-11 | Zero Format Coupling: runtime tidak memahami format checkpoint eksternal. |
| FAC-12 | Streaming-First conversion bounded-memory. |
| FAC-13 | Cell selection content-based, bukan position-based. |
| FAC-14 | Computation dynamically composed per input. |
| FAC-15 | Tidak ada fixed-depth layer stack. |
| FAC-16 | Context ditangani memory content-addressed, bukan KV-cache. |
| FAC-17 | Context MUST NOT tumbuh linear terhadap sequence length. |
| FAC-18 | Memory first-class dan persistent. |
| FAC-19 | Working memory bounded. |
| FAC-20 | Learning tidak membutuhkan global parameter update. |
| FAC-21 | Learning cost O(affected_cells). |
| FAC-22 | Learning MUST NOT catastrophic forgetting. |
| FAC-23 | Specialization tanpa full-model copy. |
| FAC-24 | Versioning melalui satu Revision DAG. |
| FAC-25 | Revision delta pada level Cell/Tile. |
| FAC-26 | Inference tidak membutuhkan full-model loading. |
| FAC-27 | Inference hanya mengaktifkan Cell relevan. |
| FAC-28 | Compute adaptif terhadap difficulty. |
| FAC-29 | Budget hard-enforced. |
| FAC-30 | Active parameter ratio < 10%. |
| FAC-31 | Total knowledge dapat tumbuh tanpa menaikkan compute per token. |
| FAC-32 | GC berbasis reachability dari revision roots. |
| FAC-33 | Integrity verification sebelum eksekusi. |
| FAC-34 | Deterministik untuk input dan state sama. |
| FAC-35 | Dokumen ini single source of truth CNWS. |

## 24.1 Pernyataan Penutup

Dokumen ini adalah **Engineering Contract Final dan Mengikat** untuk CNWS sebagai satu paradigma terpadu.

Seluruh implementasi, code review, testing, validation, dan maintenance CNWS MUST conformant terhadap kontrak ini.

Tidak ada keputusan arsitektural yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final melalui `DF-*` dan invariant di atas.

**AKHIR DOKUMEN**
