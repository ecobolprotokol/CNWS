# CNWS
## Cell & Schema Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Cell & Schema Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (SEMANTIC SCHEMA SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS .cd Format & Serialization Specification |
| Hulu ke | Implementasi Cell Resolver, Converter, Runtime, Learning Engine |
| Otoritas | Spesifikasi semantic tunggal untuk seluruh Cell CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract     .cd Format Spec      Cell & Schema Spec      Implementation
─────────────────────    ────────────────     ──────────────────      ─────────────
Cell invariants      ──► Byte layouts     ──► Semantic structure  ──► Cell code
"MUST be immutable"      Wire format          Cell schema              Converters
"MUST be content-        Serialization        CellType taxonomy        Resolvers
 addressed"                                   Dependency semantics     Validators
                                              Metadata schema          Runtime
```

`[CELL-DOC-1]` Dokumen ini mendefinisikan **struktur semantic Cell** secara lengkap.

`[CELL-DOC-2]`.cd Format Specification mendefinisikan **bagaimana Cell di-serialize sebagai bytes**; dokumen ini mendefinisikan **apa arti semantic dari bytes tersebut**.

`[CELL-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[CELL-DOC-4]` Jika terjadi konflik dengan .cd Format Spec untuk hal serialization, .cd Format Spec menang.

`[CELL-DOC-5]` Untuk hal semantic (makna, struktur, aturan), dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-CELL-01 | Cell adalah unit fundamental universal CNWS. |
| DF-CELL-02 | Cell identity = BLAKE3-256 dari canonical payload. |
| DF-CELL-03 | CellType menggunakan `u8` discriminant dengan range per kategori. |
| DF-CELL-04 | Index vector default dimensions = 512. |
| DF-CELL-05 | Similarity metric default = cosine. |
| DF-CELL-06 | Dependency types: `DATA`, `CONTROL`, `EXECUTION_ORDER`, `PREFETCH_HINT`. |
| DF-CELL-07 | Metadata schema extensible melalui `attributes` map. |
| DF-CELL-08 | Custom Cell menggunakan discriminant `0xFF` dengan registered type string. |
| DF-CELL-09 | Cell version mengikuti semver. |
| DF-CELL-10 | Cell compatibility berdasarkan type + schema + version. |

---

# 1. Executive Summary

## 1.1 Cell sebagai Abstraction Universal

Cell adalah unit fundamental universal CNWS. Seluruh persistent state — weight, memory, routing, composition, computation — direpresentasikan sebagai Cell.

```text
┌─────────────────────────────────────────────────────────────┐
│                        CELL                                  │
│                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │ Weight Cell │  │ Memory Cell │  │ Routing Cell│        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │ Composition │  │ Computation │  │ Control Cell│        │
│   │ Cell        │  │ Cell        │  │             │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│   ┌─────────────┐  ┌─────────────┐                         │
│   │ Meta Cell   │  │ Custom Cell │                         │
│   └─────────────┘  └─────────────┘                         │
│                                                             │
│   Semua berbagi:                                            │
│   - BLAKE3-256 identity                                     │
│   - Tile-based storage                                      │
│   - Dependency semantics                                    │
│   - Metadata schema                                         │
│   - Index vector                                            │
│   - Versioning                                              │
│   - Immutability                                            │
└─────────────────────────────────────────────────────────────┘
```

`[CELL-EXEC-1]` Seluruh state CNWS MUST dapat direpresentasikan sebagai Cell.

`[CELL-EXEC-2]` Tidak ada state persistent di luar Cell.

`[CELL-EXEC-3]` Cell menjawab: "knowledge/computation apa ini?"

`[CELL-EXEC-4]` Tile menjawab: "data fisiknya disimpan dan diambil bagaimana?"

## 1.2 Prinsip Desain Cell

| Prinsip | Makna |
|---|---|
| Content-addressed | Identity dari konten, bukan lokasi |
| Immutable | Tidak berubah setelah dibuat |
| Self-describing | Schema lengkap dalam Cell |
| Independently loadable | Dapat di-load tanpa Cell lain |
| Independently versionable | Dapat di-version secara independen |
| Composable | Dapat dikomposisikan dengan Cell lain |
| Extensible | Dapat diperluas tanpa breaking |

---

# 2. Cell Schema (Inti)

## 2.1 Struktur Cell Lengkap

`[CELL-SCH-1]` Setiap Cell MUST memiliki struktur berikut:

```rust
struct Cell {
    // Identity
    id: Blake3Hash,              // 32 bytes, BLAKE3-256
    cell_type: CellType,         // u8 discriminant
    version: CellVersion,        // semver
    
    // Interface
    input_schema: Schema,        // expected input
    output_schema: Schema,       // produced output
    
    // Storage
    tiles: Vec<TileRef>,         // physical storage references
    
    // Selection
    index_vector: IndexVector,   // content embedding for retrieval
    
    // Relationships
    dependencies: Vec<Dependency>, // Cell dependencies
    
    // Metadata
    metadata: CellMetadata,      // semantic metadata
    
    // Representations
    representations: Vec<RepresentationRef>, // alternative forms
}
```

## 2.2 Field Requirements

`[CELL-SCH-2]` Field requirements:

| Field | Required | Notes |
|---|---|---|
| `id` | MUST | BLAKE3-256, 32 bytes |
| `cell_type` | MUST | u8 discriminant |
| `version` | MUST | semver |
| `input_schema` | MUST | may be empty for source Cells |
| `output_schema` | MUST | may be empty for sink Cells |
| `tiles` | MUST | may be empty for virtual Cells |
| `index_vector` | SHOULD | required for selectable Cells |
| `dependencies` | MUST | may be empty |
| `metadata` | MUST | may be minimal |
| `representations` | SHOULD | may be empty |

## 2.3 Cell Identity

`[CELL-ID-1]` Cell identity dihitung dari canonical Cell payload:

```text
cell_id = BLAKE3-256(canonical_cell_payload)
```

`[CELL-ID-2]` Canonical Cell payload mencakup:

1. CellType discriminant
2. Version
3. Input schema
4. Output schema
5. Tile payloads (concatenated in order)
6. Index vector
7. Dependency list (sorted)
8. Metadata (canonical form)

`[CELL-ID-3]` Cell identity MUST independen dari:
- Storage location
- Compression
- Representation variant
- Segment placement

`[CELL-ID-4]` Cell identity MUST deterministik untuk konten yang sama.

## 2.4 Cell Version

```rust
struct CellVersion {
    major: u32,
    minor: u32,
    patch: u32,
}
```

`[CELL-VER-1]` Cell version mengikuti semver.

`[CELL-VER-2]` Breaking changes MUST menaikkan `major`.

`[CELL-VER-3]` Backward-compatible changes MUST menaikkan `minor`.

`[CELL-VER-4]` Patch changes MUST menaikkan `patch`.

`[CELL-VER-5]` Cell version berbeda dari format version (.cd).

---

# 3. CellType Taxonomy

## 3.1 Kategori CellType

`[CELL-TYPE-1]` CellType menggunakan `u8` discriminant dengan range per kategori:

| Range | Kategori | Deskripsi |
|---|---|---|
| `0x01–0x1F` | Weight Cells | Tensor weight model |
| `0x20–0x2F` | Memory Cells | Persistent memory |
| `0x30–0x3F` | Routing Cells | Routing & selection |
| `0x40–0x4F` | Composition Cells | Composition patterns |
| `0x50–0x5F` | Computation Cells | Executable modules |
| `0x60–0x6F` | Control Cells | Control flow |
| `0x70–0x7F` | Meta Cells | Metadata & provenance |
| `0x80–0xFE` | Reserved | Untuk ekspansi masa depan |
| `0xFF` | Custom | Extensible custom types |

## 3.2 Weight Cells (0x01–0x1F)

`[CELL-TYPE-2]` Weight Cells merepresentasikan tensor weight model.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x01` | `EMBEDDING` | Token/position embedding |
| `0x02` | `ATTENTION_Q_PROJ` | Attention query projection |
| `0x03` | `ATTENTION_K_PROJ` | Attention key projection |
| `0x04` | `ATTENTION_V_PROJ` | Attention value projection |
| `0x05` | `ATTENTION_OUT` | Attention output projection |
| `0x06` | `MLP_GATE` | MLP gate projection |
| `0x07` | `MLP_UP` | MLP up projection |
| `0x08` | `MLP_DOWN` | MLP down projection |
| `0x09` | `EXPERT_GATE` | MoE expert gate |
| `0x0A` | `EXPERT_ROUTE` | MoE expert router |
| `0x0B` | `EXPERT_WEIGHT` | MoE expert weight |
| `0x0C` | `LAYERNORM_WEIGHT` | LayerNorm weight |
| `0x0D` | `LAYERNORM_BIAS` | LayerNorm bias |
| `0x0E` | `LM_HEAD` | Language model head |
| `0x0F` | `VISION_ENCODER` | Vision encoder weight |
| `0x10` | `CONV_WEIGHT` | Convolutional weight |
| `0x11` | `NORM_SCALE` | Generic normalization scale |
| `0x12` | `NORM_BIAS` | Generic normalization bias |
| `0x13` | `POSITIONAL` | Positional encoding |
| `0x14` | `RESIDUAL_GATE` | Residual gating |
| `0x15–0x1F` | Reserved | — |

## 3.3 Memory Cells (0x20–0x2F)

`[CELL-TYPE-3]` Memory Cells merepresentasikan persistent memory.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x20` | `MEMORY_EPISODIC` | Episodic memory (experiences) |
| `0x21` | `MEMORY_SEMANTIC` | Semantic memory (facts) |
| `0x22` | `MEMORY_PROCEDURAL` | Procedural memory (patterns) |
| `0x23` | `MEMORY_WORKING` | Working memory (bounded) |
| `0x24` | `MEMORY_CONSOLIDATED` | Consolidated memory |
| `0x25` | `MEMORY_ASSOCIATION` | Memory associations |
| `0x26–0x2F` | Reserved | — |

## 3.4 Routing Cells (0x30–0x3F)

`[CELL-TYPE-4]` Routing Cells merepresentasikan routing & selection.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x30` | `ROUTING_POLICY` | Routing policy parameters |
| `0x31` | `ROUTING_STATISTICS` | Routing statistics |
| `0x32` | `ROUTING_INDEX` | ANN index for Cell selection |
| `0x33` | `ROUTING_ASSOCIATION` | Cell association graph |
| `0x34` | `ROUTING_THRESHOLD` | Selection thresholds |
| `0x35–0x3F` | Reserved | — |

## 3.5 Composition Cells (0x40–0x4F)

`[CELL-TYPE-5]` Composition Cells merepresentasikan composition patterns.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x40` | `COMPOSITION_PATTERN` | Cached composition pattern |
| `0x41` | `COMPOSITION_TEMPLATE` | Reusable composition template |
| `0x42` | `COMPOSITION_MACRO` | Compiled macro-Cell |
| `0x43` | `COMPOSITION_SEQUENCE` | Sequential composition |
| `0x44` | `COMPOSITION_PARALLEL` | Parallel composition |
| `0x45` | `COMPOSITION_CONDITIONAL` | Conditional composition |
| `0x46` | `COMPOSITION_ITERATIVE` | Iterative composition |
| `0x47–0x4F` | Reserved | — |

## 3.6 Computation Cells (0x50–0x5F)

`[CELL-TYPE-6]` Computation Cells merepresentasikan executable modules.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x50` | `TRANSFORM_MODULE` | Generic transformation |
| `0x51` | `ENCODE_MODULE` | Input encoder |
| `0x52` | `DECODE_MODULE` | Output decoder |
| `0x53` | `NORMALIZE_MODULE` | Normalization module |
| `0x54` | `ACTIVATION_MODULE` | Activation function |
| `0x55` | `POOLING_MODULE` | Pooling operation |
| `0x56` | `ATTENTION_MODULE` | Attention computation |
| `0x57` | `CONVOLUTION_MODULE` | Convolution computation |
| `0x58` | `RECURRENT_MODULE` | Recurrent computation |
| `0x59–0x5F` | Reserved | — |

## 3.7 Control Cells (0x60–0x6F)

`[CELL-TYPE-7]` Control Cells merepresentasikan control flow.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x60` | `HALT_CONDITION` | Halt condition |
| `0x61` | `BUDGET_POLICY` | Compute budget policy |
| `0x62` | `BRANCH_CONDITION` | Branching condition |
| `0x63` | `LOOP_CONTROL` | Loop control |
| `0x64` | `ERROR_HANDLER` | Error handling |
| `0x65–0x6F` | Reserved | — |

## 3.8 Meta Cells (0x70–0x7F)

`[CELL-TYPE-8]` Meta Cells merepresentasikan metadata & provenance.

| Discriminant | Name | Deskripsi |
|---|---|---|
| `0x70` | `PROVENANCE` | Provenance information |
| `0x71` | `CONFIGURATION` | Configuration parameters |
| `0x72` | `STATISTICS` | Usage statistics |
| `0x73` | `ANNOTATION` | Human annotations |
| `0x74` | `VALIDATION` | Validation metadata |
| `0x75–0x7F` | Reserved | — |

## 3.9 Custom Cells (0xFF)

`[CELL-TYPE-9]` Custom Cells menggunakan discriminant `0xFF` dengan registered type string.

```rust
struct CustomCellType {
    discriminant: u8,           // MUST be 0xFF
    type_string: String,        // registered type identifier
    vendor: String,             // vendor/organization
    version: u32,               // custom type version
}
```

`[CELL-TYPE-10]` Custom type string MUST registered dalam registry.

`[CELL-TYPE-11]` Custom type string MUST menggunakan format `vendor.type_name`.

Contoh: `com.example.custom_attention_v2`

---

# 4. Input/Output Schema

## 4.1 Schema Structure

`[CELL-IO-1]` Setiap Cell MUST mendefinisikan input dan output schema.

```rust
struct Schema {
    kind: SchemaKind,
    tensor: Option<TensorSchema>,
    structured: Option<StructuredSchema>,
    scalar: Option<ScalarSchema>,
    graph: Option<GraphSchema>,
}

enum SchemaKind {
    Tensor,       // tensor data
    Structured,   // structured data (key-value)
    Scalar,       // single value
    Graph,        // graph structure
    Empty,        // no input/output
}
```

## 4.2 TensorSchema

`[CELL-IO-2]` TensorSchema untuk tensor data:

```rust
struct TensorSchema {
    shape: Vec<u64>,           // tensor dimensions
    dtype: DataType,           // element type
    layout: TensorLayout,      // memory layout
    dynamic_dims: Vec<u32>,    // indices of dynamic dimensions
}

enum TensorLayout {
    RowMajor,      // C-order
    ColumnMajor,   // Fortran-order
    Blocked,       // blocked layout
    Sparse,        // sparse representation
}
```

`[CELL-IO-3]` `shape` menggunakan `u64` per dimensi.

`[CELL-IO-4]` Dynamic dimensions ditandai dengan `dynamic_dims`.

`[CELL-IO-5]` Default layout adalah `RowMajor`.

Contoh:

```json
{
  "kind": "Tensor",
  "tensor": {
    "shape": [4096, 4096],
    "dtype": "bf16",
    "layout": "RowMajor",
    "dynamic_dims": []
  }
}
```

## 4.3 StructuredSchema

`[CELL-IO-4]` StructuredSchema untuk structured data:

```rust
struct StructuredSchema {
    fields: Vec<FieldSchema>,
}

struct FieldSchema {
    name: String,
    dtype: DataType,
    shape: Option<Vec<u64>>,
    required: bool,
}
```

## 4.4 ScalarSchema

`[CELL-IO-5]` ScalarSchema untuk single value:

```rust
struct ScalarSchema {
    dtype: DataType,
}
```

## 4.5 GraphSchema

`[CELL-IO-6]` GraphSchema untuk graph structure:

```rust
struct GraphSchema {
    node_schema: Box<Schema>,
    edge_schema: Box<Schema>,
    directed: bool,
}
```

## 4.6 Empty Schema

`[CELL-IO-7]` Empty Schema untuk Cell tanpa input/output:

```rust
struct EmptySchema {}
```

`[CELL-IO-8]` Source Cells (mis. EMBEDDING) MAY memiliki Empty input schema.

`[CELL-IO-9]` Sink Cells (mis. LM_HEAD) MAY memiliki Empty output schema.

## 4.7 Schema Compatibility

`[CELL-IO-10]` Dua schema kompatibel jika:

1. `kind` sama.
2. Untuk Tensor: dtype kompatibel, shape kompatibel (dengan dynamic dims).
3. Untuk Structured: field names dan types kompatibel.
4. Untuk Scalar: dtype kompatibel.

`[CELL-IO-11]` Dtype compatibility:

| From | To | Compatible |
|---|---|---|
| f32 | f16, bf16, f8 | YES (narrowing) |
| f16, bf16 | f32 | YES (widening) |
| i8 | i16, i32, i64 | YES (widening) |
| i32 | i8 | NO (lossy) |
| f32 | i32 | NO (different domain) |

---

# 5. Dependency Semantics

## 5.1 Dependency Structure

`[CELL-DEP-1]` Setiap dependency MUST memiliki type dan target.

```rust
struct Dependency {
    target: CellId,            // target Cell
    dep_type: DependencyType,  // type of dependency
    metadata: DependencyMetadata,
}

enum DependencyType {
    DATA,              // data flows from target to this Cell
    CONTROL,           // control flow dependency
    EXECUTION_ORDER,   // execution ordering constraint
    PREFETCH_HINT,     // prefetch hint (not hard dependency)
    SEMANTIC,          // semantic relationship (no execution impact)
}
```

## 5.2 Dependency Types

`[CELL-DEP-2]` Semantik setiap dependency type:

| Type | Makna | Execution Impact |
|---|---|---|
| `DATA` | Output target adalah input Cell ini | MUST execute target first |
| `CONTROL` | Cell ini dikontrol oleh target | MUST evaluate target first |
| `EXECUTION_ORDER` | Cell ini harus execute setelah target | MUST order after target |
| `PREFETCH_HINT` | Target sebaiknya di-prefetch bersama | SHOULD prefetch, not required |
| `SEMANTIC` | Hubungan semantic tanpa execution impact | No execution impact |

## 5.3 Dependency Rules

`[CELL-DEP-3]` Dependency graph MUST acyclic.

`[CELL-DEP-4]` `DATA` dan `CONTROL` dependencies MUST dipenuhi sebelum eksekusi.

`[CELL-DEP-5]` `EXECUTION_ORDER` MUST dipatuhi oleh scheduler.

`[CELL-DEP-6]` `PREFETCH_HINT` SHOULD digunakan untuk prefetch planning.

`[CELL-DEP-7]` `SEMANTIC` MUST NOT mempengaruhi eksekusi.

## 5.4 Dependency Examples

```text
Weight Cell dependencies:
  model.layer.0.self_attn.q_proj
    ├── DATA → model.embedding.token_embedding
    └── EXECUTION_ORDER → model.layer.0.self_attn.k_proj

MoE dependencies:
  model.layer.0.moe.expert.7
    ├── CONTROL → model.layer.0.moe.router
    └── PREFETCH_HINT → model.layer.0.moe.expert.8

Composition dependencies:
  composition.attn_mlp_fused
    ├── DATA → model.layer.0.self_attn.out
    └── DATA → model.layer.0.mlp.down
```

## 5.5 Dependency Metadata

```rust
struct DependencyMetadata {
    strength: f32,             // 0.0-1.0, for weighted dependencies
    conditional: bool,         // true if dependency is conditional
    condition: Option<String>, // condition expression
    annotations: HashMap<String, String>,
}
```

`[CELL-DEP-8]` `strength` default 1.0 (hard dependency).

`[CELL-DEP-9]` Conditional dependencies MUST memiliki `condition` expression.

---

# 6. Metadata Schema

## 6.1 CellMetadata Structure

`[CELL-META-1]` Setiap Cell MUST memiliki metadata.

```rust
struct CellMetadata {
    // Common metadata
    created_at_ns: u64,
    modified_at_ns: u64,
    author: Option<String>,
    description: Option<String>,
    
    // Type-specific metadata
    type_metadata: TypeMetadata,
    
    // Provenance
    provenance: ProvenanceMetadata,
    
    // Quantization
    quantization_policy: Option<QuantizationPolicy>,
    
    // Extensible attributes
    attributes: HashMap<String, Value>,
}
```

## 6.2 Common Metadata

`[CELL-META-2]` Common metadata fields:

| Field | Type | Required |
|---|---|---|
| `created_at_ns` | u64 | MUST |
| `modified_at_ns` | u64 | MUST |
| `author` | String | SHOULD |
| `description` | String | MAY |

## 6.3 TypeMetadata

`[CELL-META-3]` TypeMetadata berbeda per CellType:

```rust
enum TypeMetadata {
    Weight(WeightMetadata),
    Memory(MemoryMetadata),
    Routing(RoutingMetadata),
    Composition(CompositionMetadata),
    Computation(ComputationMetadata),
    Control(ControlMetadata),
    Meta(MetaMetadata),
    Custom(CustomMetadata),
}
```

## 6.4 WeightMetadata

```rust
struct WeightMetadata {
    layer_index: Option<u32>,
    attention_head: Option<u32>,
    expert_index: Option<u32>,
    architecture: Option<String>,
    init_method: Option<String>,
    trainable: bool,
}
```

## 6.5 MemoryMetadata

```rust
struct MemoryMetadata {
    memory_type: MemoryType,
    consolidation_level: u8,
    access_count: u64,
    last_access_ns: u64,
    associations: Vec<CellId>,
}
```

## 6.6 RoutingMetadata

```rust
struct RoutingMetadata {
    usage_count: u64,
    success_rate: f32,
    avg_latency_us: u64,
    policy_version: u64,
}
```

## 6.7 CompositionMetadata

```rust
struct CompositionMetadata {
    composition_type: CompositionType,
    cell_count: u64,
    execution_count: u64,
    avg_execution_us: u64,
}
```

## 6.8 ProvenanceMetadata

```rust
struct ProvenanceMetadata {
    source_format: Option<String>,
    source_uri: Option<String>,
    source_tensor_name: Option<String>,
    importer_version: Option<String>,
    conversion_policy_hash: Option<Blake3Hash>,
    created_revision: Option<RevisionId>,
    lineage: Vec<CellId>,
}
```

## 6.9 QuantizationPolicy

```rust
struct QuantizationPolicy {
    scheme: QuantizationScheme,
    calibration: Option<String>,
    group_size: Option<u32>,
    symmetric: bool,
    per_channel: bool,
}

enum QuantizationScheme {
    None,
    FP8_E4M3,
    FP8_E5M2,
    INT8,
    INT4,
    INT4_ASYM,
    INT2,
    NF4,
    Custom(String),
}
```

## 6.10 Extensible Attributes

`[CELL-META-4]` `attributes` adalah HashMap untuk ekstensibilitas.

`[CELL-META-5]` Attribute keys MUST lowercase snake_case.

`[CELL-META-6]` Attribute values MUST salah satu dari:

```rust
enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

`[CELL-META-7]` Unknown attributes MUST dipertahankan saat round-trip.

`[CELL-META-8]` Attributes MUST NOT mengubah Cell identity.

---

# 7. Index Vector Schema

## 7.1 IndexVector Structure

`[CELL-IDX-1]` Index vector digunakan untuk content-addressed selection.

```rust
struct IndexVector {
    dimensions: u32,
    values: Vec<f32>,
    metric: SimilarityMetric,
    normalized: bool,
}

enum SimilarityMetric {
    Cosine,
    DotProduct,
    Euclidean,
    Manhattan,
}
```

## 7.2 Index Vector Requirements

`[CELL-IDX-2]` Default dimensions = 512.

`[CELL-IDX-3]` Default metric = Cosine.

`[CELL-IDX-4]` Index vector SHOULD normalized untuk Cosine metric.

`[CELL-IDX-5]` Index vector MUST dapat di-update melalui learning.

## 7.3 Index Vector Generation

`[CELL-IDX-6]` Index vector dihasilkan dari:

1. Cell content (untuk weight Cells: tensor statistics)
2. Cell metadata (layer, type, domain)
3. Cell usage patterns (routing statistics)
4. Learned embeddings (dari training)

## 7.4 Index Vector Update

`[CELL-IDX-7]` Index vector update menghasilkan Cell version baru.

`[CELL-IDX-8]` Index vector update MUST incremental, bukan rebuild penuh.

## 7.5 Index Vector Serialization

`[CELL-IDX-9]` Index vector di-serialize sebagai:

```text
dimensions: u32
metric: u8
normalized: u8
padding: u16
values: [f32; dimensions]
```

---

# 8. Weight Cell Schema

## 8.1 Weight Cell Definition

`[CELL-W-1]` Weight Cell merepresentasikan tensor weight model.

`[CELL-W-2]` Weight Cell MUST memiliki:
- Non-empty `tiles`
- Tensor `input_schema` dan `output_schema`
- `WeightMetadata`

## 8.2 Weight Cell Structure

```rust
struct WeightCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x01-0x1F
    version: CellVersion,
    
    // Weight-specific
    tensor_shape: Vec<u64>,
    dtype: DataType,
    tiles: Vec<TileRef>,
    
    // Semantic
    semantic_id: String,         // e.g., "model.layer.0.self_attn.q_proj"
    layer_index: Option<u32>,
    attention_head: Option<u32>,
    expert_index: Option<u32>,
    
    // Representations
    representations: Vec<RepresentationRef>,
    
    // Metadata
    metadata: CellMetadata,
}
```

## 8.3 Weight Cell Examples

### 8.3.1 Attention Q Projection

```json
{
  "id": "b3:7f3a8e...",
  "cell_type": "ATTENTION_Q_PROJ",
  "version": "1.0.0",
  "semantic_id": "model.layer.0.self_attn.q_proj",
  "tensor_shape": [4096, 4096],
  "dtype": "bf16",
  "tiles": [
    {
      "tile_id": "b3:1c33...",
      "shape": [4096, 4096],
      "offset": [0, 0],
      "size": [4096, 4096],
      "segment_id": 1
    }
  ],
  "metadata": {
    "layer_index": 0,
    "attention_head": null,
    "expert_index": null,
    "architecture": "llama",
    "trainable": true
  }
}
```

### 8.3.2 MoE Expert Weight

```json
{
  "id": "b3:9d4f2a...",
  "cell_type": "EXPERT_WEIGHT",
  "version": "1.0.0",
  "semantic_id": "model.layer.10.moe.expert.7",
  "tensor_shape": [14336, 4096],
  "dtype": "bf16",
  "tiles": [
    {
      "tile_id": "b3:3e55...",
      "shape": [7168, 4096],
      "offset": [0, 0],
      "size": [7168, 4096],
      "segment_id": 2
    },
    {
      "tile_id": "b3:4f66...",
      "shape": [7168, 4096],
      "offset": [7168, 0],
      "size": [7168, 4096],
      "segment_id": 2
    }
  ],
  "metadata": {
    "layer_index": 10,
    "expert_index": 7,
    "quantization_policy": {
      "scheme": "INT4",
      "group_size": 128,
      "symmetric": false
    }
  }
}
```

## 8.4 Weight Cell Invariants

| ID | Invariant |
|---|---|
| CELL-W-INV-1 | Weight Cell MUST memiliki minimal satu Tile |
| CELL-W-INV-2 | Tile shape MUST konsisten dengan tensor_shape |
| CELL-W-INV-3 | Tile offsets MUST non-overlapping |
| CELL-W-INV-4 | Tile concatenation MUST menghasilkan tensor lengkap |
| CELL-W-INV-5 | dtype MUST konsisten di semua Tiles |
| CELL-W-INV-6 | semantic_id MUST unik dalam model |

---

# 9. Memory Cell Schema

## 9.1 Memory Cell Definition

`[CELL-M-1]` Memory Cell merepresentasikan persistent memory.

`[CELL-M-2]` Memory Cell MUST memiliki:
- `MemoryMetadata`
- Key/value structure
- Access statistics

## 9.2 Memory Cell Structure

```rust
struct MemoryCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x20-0x2F
    version: CellVersion,
    
    // Memory-specific
    memory_type: MemoryType,
    key_vector: Vec<f32>,
    value_payload: Vec<u8>,
    consolidation_level: u8,
    
    // Associations
    associations: Vec<CellId>,
    
    // Statistics
    access_count: u64,
    last_access_ns: u64,
    
    // Metadata
    metadata: CellMetadata,
}

enum MemoryType {
    Episodic,       // 0x20
    Semantic,       // 0x21
    Procedural,     // 0x22
    Working,        // 0x23
    Consolidated,   // 0x24
    Association,    // 0x25
}
```

## 9.3 Memory Cell Examples

### 9.3.1 Episodic Memory

```json
{
  "id": "b3:mem_ep_001...",
  "cell_type": "MEMORY_EPISODIC",
  "version": "1.0.0",
  "memory_type": "Episodic",
  "key_vector": [0.12, -0.34, 0.56, "..."],
  "value_payload": "b3:value_hash...",
  "consolidation_level": 0,
  "associations": [
    "b3:mem_sem_042...",
    "b3:mem_proc_017..."
  ],
  "metadata": {
    "access_count": 42,
    "last_access_ns": 1786500000000000000,
    "created_at_ns": 1786400000000000000
  }
}
```

### 9.3.2 Semantic Memory

```json
{
  "id": "b3:mem_sem_042...",
  "cell_type": "MEMORY_SEMANTIC",
  "version": "1.0.0",
  "memory_type": "Semantic",
  "key_vector": [0.78, 0.91, -0.23, "..."],
  "value_payload": "b3:value_hash...",
  "consolidation_level": 1,
  "associations": [
    "b3:mem_sem_043...",
    "b3:mem_sem_044..."
  ],
  "metadata": {
    "access_count": 1042,
    "last_access_ns": 1786500000000000000
  }
}
```

## 9.4 Memory Cell Invariants

| ID | Invariant |
|---|---|
| CELL-M-INV-1 | Memory Cell MUST memiliki key_vector |
| CELL-M-INV-2 | Memory Cell MUST memiliki value_payload |
| CELL-M-INV-3 | Memory identity = BLAKE3-256(key + value) |
| CELL-M-INV-4 | Memory Cell MUST immutable setelah ditulis |
| CELL-M-INV-5 | Associations MUST merefer Cell yang valid |
| CELL-M-INV-6 | Working memory MUST bounded |

---

# 10. Routing Cell Schema

## 10.1 Routing Cell Definition

`[CELL-R-1]` Routing Cell merepresentasikan routing & selection.

`[CELL-R-2]` Routing Cell MUST memiliki:
- Policy parameters atau statistics
- Version tracking

## 10.2 Routing Cell Structure

```rust
struct RoutingCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x30-0x3F
    version: CellVersion,
    
    // Routing-specific
    routing_type: RoutingType,
    policy_parameters: Vec<f32>,
    statistics: RoutingStatistics,
    
    // Index (for ROUTING_INDEX)
    index_structure: Option<IndexStructure>,
    
    // Metadata
    metadata: CellMetadata,
}

enum RoutingType {
    Policy,          // 0x30
    Statistics,      // 0x31
    Index,           // 0x32
    Association,     // 0x33
    Threshold,       // 0x34
}

struct RoutingStatistics {
    usage_count: u64,
    success_rate: f32,
    avg_latency_us: u64,
    last_updated_ns: u64,
}
```

## 10.3 Routing Cell Examples

### 10.3.1 Routing Policy

```json
{
  "id": "b3:route_pol_v42...",
  "cell_type": "ROUTING_POLICY",
  "version": "42.0.0",
  "routing_type": "Policy",
  "policy_parameters": [0.5, 0.3, 0.2, "..."],
  "metadata": {
    "policy_version": 42,
    "created_at_ns": 1786500000000000000
  }
}
```

### 10.3.2 Routing Index

```json
{
  "id": "b3:route_idx_hnsw...",
  "cell_type": "ROUTING_INDEX",
  "version": "1.0.0",
  "routing_type": "Index",
  "index_structure": {
    "type": "HNSW",
    "dimensions": 512,
    "m": 32,
    "ef_construction": 200,
    "cell_count": 16384
  }
}
```

## 10.4 Routing Cell Invariants

| ID | Invariant |
|---|---|
| CELL-R-INV-1 | Routing policy change MUST membuat revision baru |
| CELL-R-INV-2 | Routing statistics MAY di-update tanpa revision |
| CELL-R-INV-3 | Routing index MUST konsisten dengan Cell set |
| CELL-R-INV-4 | Routing Cell MUST versioned |

---

# 11. Composition Cell Schema

## 11.1 Composition Cell Definition

`[CELL-C-1]` Composition Cell merepresentasikan composition patterns.

`[CELL-C-2]` Composition Cell MUST memiliki:
- Pattern definition
- Cell sequence
- Execution mode

## 11.2 Composition Cell Structure

```rust
struct CompositionCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x40-0x4F
    version: CellVersion,
    
    // Composition-specific
    composition_type: CompositionType,
    cell_sequence: Vec<CellId>,
    execution_mode: ExecutionMode,
    
    // Statistics
    execution_count: u64,
    avg_execution_us: u64,
    
    // Metadata
    metadata: CellMetadata,
}

enum CompositionType {
    Pattern,         // 0x40
    Template,        // 0x41
    Macro,           // 0x42
    Sequence,        // 0x43
    Parallel,        // 0x44
    Conditional,     // 0x45
    Iterative,       // 0x46
}

enum ExecutionMode {
    Sequential,
    Parallel,
    Conditional,
    Iterative,
}
```

## 11.3 Composition Cell Examples

### 11.3.1 Composition Pattern

```json
{
  "id": "b3:comp_attn_mlp...",
  "cell_type": "COMPOSITION_PATTERN",
  "version": "1.0.0",
  "composition_type": "Pattern",
  "cell_sequence": [
    "b3:attn_q_proj...",
    "b3:attn_k_proj...",
    "b3:attn_v_proj...",
    "b3:attn_out...",
    "b3:mlp_gate...",
    "b3:mlp_up...",
    "b3:mlp_down..."
  ],
  "execution_mode": "Sequential",
  "metadata": {
    "execution_count": 1048576,
    "avg_execution_us": 1200
  }
}
```

### 11.3.2 Composition Macro

```json
{
  "id": "b3:macro_fused_attn...",
  "cell_type": "COMPOSITION_MACRO",
  "version": "1.0.0",
  "composition_type": "Macro",
  "cell_sequence": [
    "b3:fused_attention_kernel..."
  ],
  "execution_mode": "Parallel",
  "metadata": {
    "execution_count": 524288,
    "avg_execution_us": 800
  }
}
```

## 11.4 Composition Cell Invariants

| ID | Invariant |
|---|---|
| CELL-C-INV-1 | Composition identity = BLAKE3-256(cell_sequence + type) |
| CELL-C-INV-2 | Cell sequence MUST merefer Cell yang valid |
| CELL-C-INV-3 | Composition MUST acyclic |
| CELL-C-INV-4 | Macro Cells MUST verifiable terhadap sub-Cells |

---

# 12. Computation Cell Schema

## 12.1 Computation Cell Definition

`[CELL-CMP-1]` Computation Cell merepresentasikan executable modules.

`[CELL-CMP-2]` Computation Cell MUST memiliki:
- Input/output schema
- Executable specification

## 12.2 Computation Cell Structure

```rust
struct ComputationCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x50-0x5F
    version: CellVersion,
    
    // Computation-specific
    computation_type: ComputationType,
    input_schema: Schema,
    output_schema: Schema,
    
    // Parameters (for learned computations)
    parameters: Option<Vec<f32>>,
    
    // Tiles (for parameter storage)
    tiles: Vec<TileRef>,
    
    // Metadata
    metadata: CellMetadata,
}

enum ComputationType {
    Transform,       // 0x50
    Encode,          // 0x51
    Decode,          // 0x52
    Normalize,       // 0x53
    Activation,      // 0x54
    Pooling,         // 0x55
    Attention,       // 0x56
    Convolution,     // 0x57
    Recurrent,       // 0x58
}
```

## 12.3 Computation Cell Examples

### 12.3.1 Transform Module

```json
{
  "id": "b3:transform_linear...",
  "cell_type": "TRANSFORM_MODULE",
  "version": "1.0.0",
  "computation_type": "Transform",
  "input_schema": {
    "kind": "Tensor",
    "tensor": {
      "shape": [4096],
      "dtype": "bf16",
      "layout": "RowMajor"
    }
  },
  "output_schema": {
    "kind": "Tensor",
    "tensor": {
      "shape": [4096],
      "dtype": "bf16",
      "layout": "RowMajor"
    }
  },
  "parameters": "b3:params_hash...",
  "tiles": [
    {
      "tile_id": "b3:param_tile...",
      "shape": [4096, 4096],
      "offset": [0, 0],
      "size": [4096, 4096],
      "segment_id": 3
    }
  ]
}
```

### 12.3.2 Encode Module

```json
{
  "id": "b3:encode_tokenizer...",
  "cell_type": "ENCODE_MODULE",
  "version": "1.0.0",
  "computation_type": "Encode",
  "input_schema": {
    "kind": "Structured",
    "structured": {
      "fields": [
        {"name": "text", "dtype": "string", "required": true}
      ]
    }
  },
  "output_schema": {
    "kind": "Tensor",
    "tensor": {
      "shape": ["seq_len"],
      "dtype": "i64",
      "layout": "RowMajor",
      "dynamic_dims": [0]
    }
  }
}
```

## 12.4 Computation Cell Invariants

| ID | Invariant |
|---|---|
| CELL-CMP-INV-1 | Computation Cell MUST memiliki input dan output schema |
| CELL-CMP-INV-2 | Parameter Tiles MUST konsisten dengan schema |
| CELL-CMP-INV-3 | Computation Cell MUST deterministic untuk input sama |
| CELL-CMP-INV-4 | Computation Cell MAY memiliki parameters atau tidak |

---

# 13. Control Cell Schema

## 13.1 Control Cell Definition

`[CELL-CTL-1]` Control Cell merepresentasikan control flow.

## 13.2 Control Cell Structure

```rust
struct ControlCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x60-0x6F
    version: CellVersion,
    
    // Control-specific
    control_type: ControlType,
    condition: Option<ConditionExpression>,
    parameters: Vec<f32>,
    
    // Metadata
    metadata: CellMetadata,
}

enum ControlType {
    HaltCondition,    // 0x60
    BudgetPolicy,     // 0x61
    BranchCondition,  // 0x62
    LoopControl,      // 0x63
    ErrorHandler,     // 0x64
}
```

## 13.3 Control Cell Examples

### 13.3.1 Halt Condition

```json
{
  "id": "b3:halt_confidence...",
  "cell_type": "HALT_CONDITION",
  "version": "1.0.0",
  "control_type": "HaltCondition",
  "condition": {
    "type": "threshold",
    "metric": "confidence",
    "operator": ">=",
    "value": 0.90
  }
}
```

### 13.3.2 Budget Policy

```json
{
  "id": "b3:budget_adaptive...",
  "cell_type": "BUDGET_POLICY",
  "version": "1.0.0",
  "control_type": "BudgetPolicy",
  "parameters": [15.0, 8.0, 1000000000.0]
}
```

## 13.4 Control Cell Invariants

| ID | Invariant |
|---|---|
| CELL-CTL-INV-1 | Control Cell MUST memiliki condition atau parameters |
| CELL-CTL-INV-2 | Control Cell MUST dievaluasi setiap iterasi |
| CELL-CTL-INV-3 | Control Cell MUST NOT mengubah state model |

---

# 14. Meta Cell Schema

## 14.1 Meta Cell Definition

`[CELL-MT-1]` Meta Cell merepresentasikan metadata & provenance.

## 14.2 Meta Cell Structure

```rust
struct MetaCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // 0x70-0x7F
    version: CellVersion,
    
    // Meta-specific
    meta_type: MetaType,
    content: Vec<u8>,
    
    // Metadata
    metadata: CellMetadata,
}

enum MetaType {
    Provenance,      // 0x70
    Configuration,   // 0x71
    Statistics,      // 0x72
    Annotation,      // 0x73
    Validation,      // 0x74
}
```

## 14.3 Meta Cell Invariants

| ID | Invariant |
|---|---|
| CELL-MT-INV-1 | Meta Cell MUST NOT mempengaruhi eksekusi |
| CELL-MT-INV-2 | Meta Cell MAY di-update tanpa revision (untuk statistics) |
| CELL-MT-INV-3 | Meta Cell MUST dipertahankan saat round-trip |

---

# 15. Custom Cell Extension Mechanism

## 15.1 Custom Cell Definition

`[CELL-CUST-1]` Custom Cell memungkinkan ekstensi tanpa mengubah spesifikasi inti.

`[CELL-CUST-2]` Custom Cell menggunakan discriminant `0xFF`.

## 15.2 Custom Cell Structure

```rust
struct CustomCell {
    // Common Cell fields
    id: Blake3Hash,
    cell_type: CellType,         // MUST be 0xFF
    version: CellVersion,
    
    // Custom-specific
    custom_type: CustomCellType,
    payload: Vec<u8>,
    
    // Standard schemas (optional)
    input_schema: Option<Schema>,
    output_schema: Option<Schema>,
    
    // Metadata
    metadata: CellMetadata,
}

struct CustomCellType {
    type_string: String,         // "vendor.type_name"
    vendor: String,
    type_version: u32,
    schema_hash: Blake3Hash,     // hash of custom schema definition
}
```

## 15.3 Custom Cell Registration

`[CELL-CUST-3]` Custom type MUST registered dalam registry.

`[CELL-CUST-4]` Registration MUST mencakup:

1. `type_string`: unique identifier
2. `vendor`: organization identifier
3. `schema`: custom schema definition
4. `validation`: validation rules

## 15.4 Custom Cell Type String Format

`[CELL-CUST-5]` Type string MUST mengikuti format:

```text
<vendor>.<type_name>[_v<version>]
```

Contoh:
- `com.example.custom_attention`
- `org.research.sparse_moe_v2`
- `net.vendor.quantized_conv`

`[CELL-CUST-6]` Type string MUST lowercase.

`[CELL-CUST-7]` Type string MUST NOT menggunakan reserved prefixes: `cnws.`, `system.`.

## 15.5 Custom Cell Validation

`[CELL-CUST-8]` Custom Cell MUST divalidasi terhadap registered schema.

`[CELL-CUST-9]` Validasi MUST mencakup:

1. Type string terdaftar
2. Payload sesuai schema
3. Dependencies valid
4. Metadata lengkap

## 15.6 Custom Cell Compatibility

`[CELL-CUST-10]` Custom Cell dengan type tidak dikenal MUST:

1. Disimpan sebagai opaque data
2. Tidak dieksekusi
3. Dilaporkan sebagai unknown type
4. Dipertahankan saat round-trip

`[CELL-CUST-11]` Custom Cell MUST NOT mempengaruhi Cell lain yang dikenal.

---

# 16. Cell Compatibility Rules

## 16.1 Compatibility Dimensions

`[CELL-COMPAT-1]` Cell compatibility ditentukan oleh empat dimensi:

1. **Type compatibility**: CellType cocok
2. **Schema compatibility**: input/output schema kompatibel
3. **Version compatibility**: version kompatibel
4. **Dependency compatibility**: dependencies kompatibel

## 16.2 Type Compatibility

`[CELL-COMPAT-2]` Dua Cell type-compatible jika:

1. Discriminant sama, ATAU
2. Keduanya Custom dengan type_string sama, ATAU
3. Salah satu adalah supertype dari yang lain

`[CELL-COMPAT-3]` Weight Cell types (0x01-0x1F) MUST NOT kompatibel dengan non-Weight types.

## 16.3 Schema Compatibility

`[CELL-COMPAT-4]` Schema compatibility mengikuti aturan di §4.7.

`[CELL-COMPAT-5]` Input schema compatible jika consumer dapat menerima output producer.

`[CELL-COMPAT-6]` Output schema compatible jika producer menghasilkan apa yang consumer harapkan.

## 16.4 Version Compatibility

`[CELL-COMPAT-7]` Version compatibility mengikuti semver:

| Producer Version | Consumer Version | Compatible |
|---|---|---|
| 1.0.0 | 1.0.0 | YES |
| 1.0.0 | 1.1.0 | YES (consumer newer) |
| 1.1.0 | 1.0.0 | MAY (backward compat) |
| 1.0.0 | 2.0.0 | NO (major bump) |
| 2.0.0 | 1.0.0 | NO (major bump) |

`[CELL-COMPAT-8]` Major version mismatch MUST dianggap incompatible.

## 16.5 Dependency Compatibility

`[CELL-COMPAT-9]` Dependency compatible jika:

1. Semua `DATA` dependencies tersedia
2. Semua `CONTROL` dependencies dapat dievaluasi
3. Semua `EXECUTION_ORDER` constraints dapat dipenuhi

## 16.6 Representation Compatibility

`[CELL-COMPAT-10]` Representation compatible jika:

1. Dtype dapat dikonversi tanpa loss yang tidak dapat diterima
2. Shape konsisten
3. Layout kompatibel

## 16.7 Compatibility Check Algorithm

```pseudo
function check_compatibility(cell_a, cell_b):
    // Type check
    if not type_compatible(cell_a.cell_type, cell_b.cell_type):
        return INCOMPATIBLE_TYPE
    
    // Schema check
    if not schema_compatible(cell_a.output_schema, cell_b.input_schema):
        return INCOMPATIBLE_SCHEMA
    
    // Version check
    if not version_compatible(cell_a.version, cell_b.version):
        return INCOMPATIBLE_VERSION
    
    // Dependency check
    if not dependency_compatible(cell_a.dependencies, cell_b.dependencies):
        return INCOMPATIBLE_DEPENDENCY
    
    return COMPATIBLE
```

---

# 17. Cell Lifecycle

## 17.1 Cell State Machine

```text
┌──────────┐   create   ┌──────────┐   commit   ┌──────────┐
│ DRAFT    │──────────►│ STAGED   │──────────►│ ACTIVE   │
└──────────┘           └──────────┘           └────┬─────┘
                                                   │
                                    ┌──────────────┼──────────────┐
                                    │              │              │
                                    ▼              ▼              ▼
                              ┌──────────┐  ┌──────────┐  ┌──────────┐
                              │ REFINED  │  │ SUPERSEDED│ │ DEPRECATED│
                              └──────────┘  └──────────┘  └──────────┘
                                    │              │              │
                                    └──────────────┴──────────────┘
                                                   │
                                                   ▼
                                          ┌──────────┐
                                          │ ARCHIVED │
                                          └──────────┘
```

## 17.2 Cell Lifecycle Transitions

`[CELL-LIFE-1]` Cell lifecycle transitions:

| From | To | Trigger |
|---|---|---|
| DRAFT | STAGED | Cell created, pending commit |
| STAGED | ACTIVE | Commit successful |
| ACTIVE | REFINED | Cell refined (new version) |
| ACTIVE | SUPERSEDED | New Cell replaces this |
| ACTIVE | DEPRECATED | Cell marked deprecated |
| REFINED | ACTIVE | Refined version committed |
| SUPERSEDED | ARCHIVED | No longer referenced |
| DEPRECATED | ARCHIVED | No longer referenced |

`[CELL-LIFE-2]` ACTIVE Cell MUST immutable.

`[CELL-LIFE-3]` REFINED menghasilkan Cell version baru, bukan modifikasi.

`[CELL-LIFE-4]` ARCHIVED Cell MAY di-GC jika tidak reachable.

## 17.3 Cell Creation

```pseudo
function create_cell(spec):
    // 1. Validate spec
    validate(spec)
    
    // 2. Generate tiles
    tiles = plan_tiles(spec)
    
    // 3. Compute identity
    cell_id = compute_cell_id(spec, tiles)
    
    // 4. Create Cell object
    cell = Cell {
        id: cell_id,
        cell_type: spec.cell_type,
        version: spec.version,
        input_schema: spec.input_schema,
        output_schema: spec.output_schema,
        tiles: tiles,
        index_vector: spec.index_vector,
        dependencies: spec.dependencies,
        metadata: spec.metadata,
        representations: spec.representations,
    }
    
    // 5. Stage
    stage(cell)
    
    return cell
```

## 17.4 Cell Refinement

```pseudo
function refine_cell(old_cell, updates):
    // 1. Apply updates
    new_spec = apply_updates(old_cell, updates)
    
    // 2. Create new Cell
    new_cell = create_cell(new_spec)
    
    // 3. Link versions
    new_cell.metadata.lineage.append(old_cell.id)
    
    // 4. Mark old as refined
    old_cell.state = REFINED
    
    // 5. Commit
    commit(new_cell)
    
    return new_cell
```

## 17.5 Cell Deprecation

`[CELL-LIFE-5]` Deprecation MUST NOT menghapus Cell.

`[CELL-LIFE-6]` Deprecated Cell tetap accessible untuk rollback.

`[CELL-LIFE-7]` Deprecated Cell SHOULD tidak digunakan untuk eksekusi baru.

---

# 18. Cell Serialization

## 18.1 Cell dalam MANIFEST.cd

Cell dideskripsikan dalam MANIFEST.cd sebagai JSON object.

`[CELL-SER-1]` Cell entry dalam MANIFEST.cd MUST memuat:

| Field | Required | Type |
|---|---|---|
| `id` | MUST | string (b3:...) |
| `cell_type` | MUST | string (CellType name) |
| `version` | MUST | string (semver) |
| `input_schema` | MUST | object |
| `output_schema` | MUST | object |
| `tiles` | MUST | array of TileRef |
| `index_vector` | SHOULD | object |
| `dependencies` | MUST | array of Dependency |
| `metadata` | MUST | object |
| `representations` | SHOULD | array |

## 18.2 Cell dalam Binary Index

Cell index binary menggunakan format yang didefinisikan dalam .cd Format Specification.

`[CELL-SER-2]` CellIndexEntry MUST sesuai dengan .cd Format Spec §6.3.

## 18.3 Cell Payload Serialization

`[CELL-SER-3]` Cell payload (untuk hashing) MUST di-serialize sebagai:

```text
cell_payload = 
    cell_type (1 byte) ||
    version_major (4 bytes LE) ||
    version_minor (4 bytes LE) ||
    version_patch (4 bytes LE) ||
    input_schema_canonical ||
    output_schema_canonical ||
    tile_payloads (concatenated) ||
    index_vector_bytes ||
    dependencies_canonical ||
    metadata_canonical
```

`[CELL-SER-4]` Canonical serialization MUST deterministik.

---

# 19. Final Cell Schema Contract

## 19.1 Ringkasan Keputusan Schema

| ID | Keputusan |
|---|---|
| CELL-F01 | Cell adalah unit fundamental universal CNWS. |
| CELL-F02 | Cell identity = BLAKE3-256 dari canonical payload. |
| CELL-F03 | CellType menggunakan u8 discriminant dengan range per kategori. |
| CELL-F04 | Weight Cells: 0x01-0x1F. |
| CELL-F05 | Memory Cells: 0x20-0x2F. |
| CELL-F06 | Routing Cells: 0x30-0x3F. |
| CELL-F07 | Composition Cells: 0x40-0x4F. |
| CELL-F08 | Computation Cells: 0x50-0x5F. |
| CELL-F09 | Control Cells: 0x60-0x6F. |
| CELL-F10 | Meta Cells: 0x70-0x7F. |
| CELL-F11 | Custom Cells: 0xFF dengan registered type string. |
| CELL-F12 | Index vector default 512 dimensions, Cosine metric. |
| CELL-F13 | Dependency types: DATA, CONTROL, EXECUTION_ORDER, PREFETCH_HINT, SEMANTIC. |
| CELL-F14 | Metadata extensible melalui attributes map. |
| CELL-F15 | Cell version mengikuti semver. |
| CELL-F16 | Cell compatibility: type + schema + version + dependency. |
| CELL-F17 | Cell lifecycle: DRAFT → STAGED → ACTIVE → REFINED/SUPERSEDED/DEPRECATED → ARCHIVED. |
| CELL-F18 | Custom Cell type string format: vendor.type_name. |
| CELL-F19 | Cell payload serialization deterministik. |
| CELL-F20 | Cell MUST immutable setelah ACTIVE. |

## 19.2 Cell Schema Invariants

| ID | Invariant |
|---|---|
| CELL-INV-1 | Setiap Cell MUST memiliki BLAKE3-256 identity unik. |
| CELL-INV-2 | Cell MUST immutable setelah ACTIVE. |
| CELL-INV-3 | Cell identity MUST independen dari storage location. |
| CELL-INV-4 | Cell identity MUST independen dari compression. |
| CELL-INV-5 | Cell identity MUST independen dari representation. |
| CELL-INV-6 | Cell MUST memiliki input dan output schema. |
| CELL-INV-7 | Cell MUST memiliki metadata. |
| CELL-INV-8 | Dependency graph MUST acyclic. |
| CELL-INV-9 | CellType discriminant MUST valid. |
| CELL-INV-10 | Custom Cell MUST registered. |
| CELL-INV-11 | Cell version MUST semver. |
| CELL-INV-12 | Cell compatibility MUST diverifikasi sebelum composition. |
| CELL-INV-13 | Cell lifecycle MUST dipatuhi. |
| CELL-INV-14 | Cell refinement MUST menghasilkan version baru. |
| CELL-INV-15 | Cell deprecation MUST NOT menghapus Cell. |

## 19.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi semantic Cell final dan mengikat** untuk CNWS. Ia mendefinisikan struktur lengkap Cell sebagai abstraction universal CNWS, dari schema inti hingga CellType taxonomy, dari dependency semantics hingga compatibility rules.

Seluruh implementasi Cell Resolver, Converter, Runtime, dan Learning Engine CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan schema yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN CELL & SCHEMA SPECIFICATION**
