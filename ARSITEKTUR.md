CNWS-X adalah canonical weight infrastructure yang mengubah checkpoint LLM berukuran besar dari berbagai format seperti Safetensors, GGUF, dan PyTorch menjadi representasi canonical yang terstruktur, modular, dan independen dari format sumber. Fondasinya adalah Streaming-First Pipeline, yang memproses weight secara incremental melalui bounded buffer dan parallel pipeline sehingga penggunaan RAM tidak bergantung langsung pada ukuran model. Weight dipetakan ke Columnar Model Graph sebagai unit logis dan kemudian dipecah menjadi Tiles sebagai unit fisik yang dapat di-load, di-cache, dikompresi, dan direpresentasikan secara adaptif. Setiap Tile menggunakan BLAKE3-256 content addressing, memungkinkan immutable identity, deduplication, integrity verification, dan sharing antar-model maupun revision.

Di sisi runtime, CNWS-X menjadikan model.cd sebagai canonical source of truth, sehingga runtime tidak perlu memahami layout Safetensors, GGUF, atau format checkpoint lainnya. Runtime dapat melakukan selective loading, adaptive representation, cache-aware execution, serta hanya memuat expert yang dibutuhkan pada MoE. Di atas physical Tile layer terdapat incremental versioning berbasis revision DAG, sehingga fine-tuning atau specialization hanya menghasilkan Tile yang berubah dan tetap dapat berbagi Tile yang identik dengan model induk atau branch lain. Dengan kombinasi streaming, canonical representation, content-addressed storage, selective execution, dan incremental versioning, CNWS-X pada dasarnya menjadi storage, conversion, dan runtime infrastructure untuk membuat model besar lebih tractable, modular, dan efisien tanpa terikat pada format checkpoint tertentu.

--

Inti arsitektur CNWS X bisa diringkas menjadi 7 komponen arsitektural utama:

1. Streaming-First Pipeline
Weight diproses secara incremental, bukan seluruh model dimuat ke RAM. Ini menjadi fondasi bounded-memory conversion.


2. Columnar Model Graph
Checkpoint eksternal diterjemahkan menjadi struktur logis berbasis Column dengan semantic identity seperti layer.2.attn.q_proj, sehingga tidak bergantung pada layout shard sumber.


3. Tile-Based Physical Storage
Column dipecah menjadi Tile sebagai unit fisik yang bisa di-load, di-cache, dikompresi, di-quantize, dan dipindahkan secara independen.


4. BLAKE3-256 Content Addressing
Setiap Tile memiliki identity berdasarkan kontennya. Ini menjadi dasar deduplication, immutable storage, integrity, dan shared Tiles antar revision/model.


5. Canonical Manifest & Zero Format Coupling
model.cd menjadi source of truth. Runtime tidak perlu memahami Safetensors, GGUF, atau format checkpoint lainnya.


6. Selective & Adaptive Runtime
Runtime hanya mengambil Column/Tile yang dibutuhkan, dengan cache hierarchy dan adaptive representation berdasarkan hardware serta kebutuhan inference. Ini termasuk selective loading untuk MoE.


7. Incremental Versioning / Revision DAG
Revision baru hanya menyimpan perubahan. Tile yang tidak berubah direferensikan kembali, sehingga branching, fine-tuning, specialization, dan cross-model sharing tidak membutuhkan duplikasi full checkpoint.



Secara konseptual:

CNWS X
                       │
        ┌──────────────┴──────────────┐
        │                             │
   LOGICAL MODEL                 PHYSICAL MODEL
        │                             │
 Columnar Graph ────────────────► Tile Storage
        │                             │
        │                       BLAKE3-256
        │                             │
        └──────────────┬──────────────┘
                       │
              Canonical Manifest
                   model.cd
                       │
             ┌─────────┴─────────┐
             │                   │
       Runtime Resolver      Revision DAG
             │                   │
       Selective Loading    Incremental Delta
             │
       Cache / NVMe / GPU

Jadi kalau harus dipadatkan menjadi satu kalimat:

> CNWS X adalah streaming canonical-weight architecture yang memisahkan semantic model graph dari physical Tile storage, menggunakan BLAKE3-256 content addressing, lalu menyediakan selective runtime loading dan incremental revision melalui satu canonical manifest.


Spesifikasi Columnar Model Graph (CMG) untuk CNWS X sebaiknya menjadi kontrak antara model semantics dan physical Tile storage. Intinya, CMG tidak mengetahui offset byte atau layout shard sumber.

1. Struktur utama

struct ModelGraph {
    version: String,
    model_id: String,

    columns: Vec<Column>,
    dependencies: DependencyGraph,

    architecture: ArchitectureMetadata,
    metadata: GraphMetadata,
}

2. Column

Column adalah unit logis weight.

struct Column {
    id: ColumnId,

    semantic_type: ColumnType,

    shape: Vec<usize>,
    dtype: DataType,

    tiles: Vec<TileRef>,

    dependencies: Vec<ColumnId>,

    metadata: ColumnMetadata,
}

Contoh:

model.layer.2.self_attn.q_proj

bukan:

shard_017 + offset 483920184

Dengan demikian:

Column
   │
   ├── semantic identity
   ├── tensor shape
   ├── dtype
   ├── dependencies
   └── TileRefs
          │
          ├── Tile A
          ├── Tile B
          └── Tile C

3. ColumnType

Minimal:

enum ColumnType {
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

    CUSTOM(String),
}

Tetapi ColumnType sebaiknya semantic, bukan terlalu terikat pada satu architecture.

Misalnya Qwen, Llama, Mistral, dan architecture baru tetap dapat dipetakan ke:

ATTENTION_Q_PROJ
ATTENTION_K_PROJ
ATTENTION_V_PROJ

4. Identity

Column.id harus deterministik.

Contoh:

model.embedding.token_embedding

model.layer.0.self_attn.q_proj
model.layer.0.self_attn.k_proj
model.layer.0.self_attn.v_proj

model.layer.0.mlp.gate
model.layer.0.mlp.up
model.layer.0.mlp.down

model.layer.0.moe.expert.0
model.layer.0.moe.expert.1

Identity ini tidak boleh bergantung pada:

filename shard

offset byte

segment number

urutan file

compression

physical storage location


Itulah yang memungkinkan physical storage berubah tanpa mengubah semantic graph.

5. TileRef

CMG hanya menyimpan referensi terhadap Tile:

struct TileRef {
    tile_id: Blake3Hash,

    shape: Vec<usize>,
    offset: Vec<usize>,
    size: Vec<usize>,

    segment_id: SegmentId,
}

Contoh:

Column:
model.layer.0.self_attn.q_proj
shape = [4096, 4096]

        ┌───────────────┐
        │ Tile 0        │
        ├───────────────┤
        │ Tile 1        │
        ├───────────────┤
        │ Tile 2        │
        ├───────────────┤
        │ Tile 3        │
        └───────────────┘

Jadi Column = logical tensor, sedangkan Tile = physical storage unit.

6. Dependency Graph

Setiap Column dapat mempunyai dependency:

struct DependencyGraph {
    edges: HashMap<ColumnId, Vec<ColumnId>>,
}

Contoh:

embedding
    ↓
layer.0.attention
    ↓
layer.0.mlp
    ↓
layer.1.attention
    ↓
layer.1.mlp

Untuk MoE:

layer.10.moe.router
        │
        ├──► expert.2
        ├──► expert.7
        └──► expert.11

Graph ini digunakan runtime untuk execution ordering dan prefetch planning, bukan sekadar dokumentasi.

7. Semantic Metadata

struct ColumnMetadata {
    architecture: Option<String>,
    layer_index: Option<u32>,
    attention_head: Option<u32>,
    expert_index: Option<u32>,

    quantization_policy: Option<QuantizationPolicy>,

    attributes: HashMap<String, Value>,
}

Contoh:

{
  "id": "model.layer.10.moe.expert.7",
  "semantic_type": "EXPERT_WEIGHT",
  "shape": [14336, 4096],
  "dtype": "bf16",
  "metadata": {
    "layer_index": 10,
    "expert_index": 7,
    "quantization_policy": "int4"
  }
}

8. Architecture Metadata

CMG juga menyimpan informasi global:

struct ArchitectureMetadata {
    architecture_type: String,

    num_layers: usize,
    hidden_dim: usize,
    num_heads: usize,

    vocab_size: usize,

    num_experts: Option<usize>,
    experts_per_token: Option<usize>,

    special_components: Vec<String>,
}

Ini memungkinkan runtime memahami struktur model tanpa membaca checkpoint sumber.


---

9. Prinsip paling penting

CMG harus mempunyai strict separation:

SOURCE FORMAT
          Safetensors / GGUF / PT
                    │
                    ▼
            Format Reader
                    │
                    ▼
             NORMALIZATION
                    │
                    ▼
          ┌───────────────────┐
          │ Columnar Model    │
          │ Graph             │
          └───────────────────┘
             │             │
        semantic        dependencies
             │             │
             ▼             ▼
           Tiles ───────► Runtime
             │
             ▼
       Physical .cd storage

Artinya:

CMG tidak boleh menjadi abstraction dari file.

CMG adalah abstraction dari model itu sendiri.

Itu yang membuat CNWS X berbeda: checkpoint seperti Safetensors pada dasarnya menjawab "di mana tensor berada?", sedangkan CMG menjawab "tensor ini secara semantik apa, bagaimana hubungannya dengan model, dan Tile mana yang merepresentasikannya?"

10. Kontrak CMG yang saya anggap final

Column
 ├── Stable Semantic ID
 ├── Semantic Type
 ├── Shape
 ├── DType
 ├── Tile References
 ├── Dependencies
 └── Semantic Metadata

Tile
 ├── BLAKE3-256 Identity
 ├── Physical Shape
 ├── Physical Location
 └── Representation Metadata

Dengan pemisahan ini, Columnar Model Graph menjadi logical layer, sementara Tile/Segment menjadi physical layer. Itu adalah salah satu boundary terpenting dalam arsitektur CNWS X.


Untuk CNWS X, Tile-Based Physical Storage sebaiknya menjadi lapisan yang sangat ketat karena Tile adalah unit dasar untuk storage, dedup, cache, compression, versioning, dan runtime loading.

1. Definisi Tile

Tile adalah unit fisik immutable yang merepresentasikan sebagian data weight dari sebuah Column.

Column
└── Tile 0
└── Tile 1
└── Tile 2
└── ...

Contoh:

model.layer.2.self_attn.q_proj
        │
        ├── Tile #0   128 MiB
        ├── Tile #1   128 MiB
        ├── Tile #2   128 MiB
        └── Tile #3    64 MiB

Column = logical unit
Tile = physical unit

Tile tidak boleh bergantung pada posisi byte dari format sumber.


---

2. Identity Tile

Setiap Tile memiliki BLAKE3-256 sebagai canonical identity.

type TileId = [u8; 32];

Secara konseptual:

Tile ID = BLAKE3-256(canonical tile payload)

Sifatnya:

immutable

deterministic

content-addressed

global across model/revision

dapat digunakan sebagai cache key

dapat digunakan untuk deduplication


Jika dua Tile memiliki payload identik:

Model A ──┐
          ├──► BLAKE3-256(X) ──► satu physical Tile
Model B ──┘

Tidak perlu menyimpan dua copy.


---

3. Tile Metadata

Minimal Tile metadata:

struct TileMeta {
    id: TileId,

    column_id: ColumnId,

    shape: Vec<u64>,
    dtype: DataType,

    element_offset: u64,
    element_count: u64,

    payload_size: u64,
    stored_size: u64,

    compression: Compression,

    representations: Vec<RepresentationRef>,
}

Contoh:

{
  "id": "b3:7f31...",
  "column_id": "model.layer.2.self_attn.q_proj",
  "shape": [4096, 4096],
  "dtype": "bf16",
  "element_offset": 0,
  "element_count": 16777216,
  "payload_size": 33554432,
  "stored_size": 28741321,
  "compression": "zstd"
}


---

4. Tile Size

Tile size tidak boleh fixed secara global.

Default dapat:

32–256 MiB

dengan default praktis:

128 MiB

Tetapi allocator menentukan ukuran berdasarkan:

available RAM
NVMe throughput
GPU VRAM
compression ratio
access pattern
tensor shape
runtime workload

Contoh:

4 GB RAM   → 32–64 MiB
16 GB RAM  → 64–128 MiB
64 GB RAM  → 128–256 MiB
128 GB RAM → 128–256 MiB

Namun RAM tidak boleh menjadi satu-satunya parameter.


---

5. Alignment

Physical Tile payload sebaiknya aligned:

Metadata alignment : 4 / 8 bytes
Tile payload       : 4 KiB minimum
Preferred           : 64 KiB / 2 MiB jika cocok dengan storage

Tujuannya untuk:

efficient NVMe I/O

mmap

direct I/O

DMA

GPU transfer

predictable offsets


Jadi:

segment.cd

[header]
[metadata]
[padding]
[Tile A] ← aligned
[padding]
[Tile B] ← aligned
[padding]
[Tile C] ← aligned


---

6. Immutable Storage

Setelah Tile ditulis:

Tile ID = BLAKE3(payload)

Tile tidak boleh dimodifikasi.

Jika weight berubah:

Old Tile
   │
   └── immutable

New weight
   │
   ▼
New Tile
   │
   └── new BLAKE3 ID

Ini sangat penting untuk revision system.

Revision tidak mengubah Tile lama.


---

7. Deduplication

Deduplication dilakukan pada level Tile.

Incoming Tile
      │
      ▼
BLAKE3-256
      │
      ▼
Global Tile Registry
      │
 ┌────┴────┐
 │         │
exists    new
 │         │
reuse     write

Contoh:

Revision 0
 ├─ Tile A
 ├─ Tile B
 ├─ Tile C
 └─ Tile D

Revision 1
 ├─ Tile A → reuse
 ├─ Tile B → reuse
 ├─ Tile X → new
 └─ Tile D → reuse

Hanya Tile X yang perlu ditulis.


---

8. Segment Storage

Tile tidak harus satu file per Tile.

Itu akan menghasilkan terlalu banyak filesystem objects.

Lebih baik:

model.cd
   │
   ├── segment-000001.cd
   ├── segment-000002.cd
   ├── segment-000003.cd
   └── ...

Setiap segment berisi banyak Tile.

Misalnya:

Segment target = 32 GiB

Segment 000001
├── Tile 0001
├── Tile 0002
├── Tile 0003
├── ...
└── Tile 0256

Dengan Tile 128 MiB:

32 GiB / 128 MiB ≈ 256 Tiles


---

9. Segment Index

Segment harus memiliki index sehingga runtime tidak melakukan scan.

struct TileLocation {
    segment_id: SegmentId,
    offset: u64,
    stored_size: u64,
}

Lookup:

TileId
  ↓
Tile Registry / Manifest
  ↓
Segment ID
  ↓
Offset
  ↓
Read exactly N bytes

Tidak:

open segment
→ scan
→ search Tile
→ read

Tetapi:

open
→ seek(offset)
→ read(size)


---

10. Compression

Compression adalah properti physical representation, bukan identity Tile.

Tile
 ├── identity: BLAKE3-256
 │
 └── representation
       ├── raw
       ├── zstd
       └── ...

Contoh:

{
  "compression": {
    "codec": "zstd",
    "level": 3
  }
}

Yang penting:

hash identity harus didefinisikan dengan jelas terhadap payload canonical, bukan terhadap compressed bytes.

Dengan demikian:

same weight
   │
   ├── zstd level 3
   └── zstd level 5

tetap memiliki identity yang sama.


---

11. Multiple Representations

Satu logical Tile dapat memiliki beberapa representation:

Tile
 │
 ├── canonical BF16
 ├── FP8
 ├── INT8
 └── INT4

Contoh:

struct Representation {
    format: RepresentationFormat,
    dtype: DataType,

    storage_id: TileId,

    size_bytes: u64,

    quantization: Option<QuantizationMetadata>,
}

Runtime kemudian memilih:

GPU H100
    → FP8

CPU
    → INT8 / BF16

High accuracy
    → canonical

Ini memungkinkan physical storage yang sama mendukung berbagai execution modes.


---

12. Tile Lifecycle

Lifecycle ideal:

Source Tensor
     │
     ▼
Tile Planner
     │
     ▼
Streaming Read
     │
     ▼
Normalize
     │
     ▼
BLAKE3-256
     │
     ├── duplicate ──► reuse
     │
     └── new
          │
          ▼
      Encode
          │
          ▼
      Compress
          │
          ▼
      Segment
          │
          ▼
      Immutable Tile
          │
          ▼
      Runtime Cache


---

13. Runtime Loading

Tile menjadi unit minimum I/O runtime.

Column requested
       │
       ▼
Tile references
       │
       ▼
Cache lookup
       │
 ┌─────┼─────┐
 ▼     ▼     ▼
GPU   RAM   NVMe
 │
 └── miss → lower layer

Runtime tidak perlu membaca seluruh segment jika hanya membutuhkan satu Tile.

Contoh:

Column = 1 GB
Tile = 128 MB

Runtime membutuhkan Column
        ↓
load 8 Tiles

Tetapi jika hanya membutuhkan subset:
        ↓
load Tile 2
load Tile 3
load Tile 7


---

14. Tile dan Versioning

Ini bagian yang sangat penting.

Revision tidak menyimpan:

Revision 1 = full model

Tetapi:

Revision 1
│
├── Column A
│    ├── Tile A1 → Revision 0
│    ├── Tile A2 → Revision 0
│    └── Tile A3 → NEW
│
├── Column B
│    └── unchanged → Revision 0
│
└── Column C
     └── unchanged → Revision 0

Dengan demikian Tile menjadi building block dari revision DAG.


---

15. Prinsip Utama Tile Storage

Saya akan menetapkan 8 invariant:

1. Tile immutable
2. Tile content-addressed
3. Tile identity = BLAKE3-256
4. Tile independently readable
5. Tile independently cacheable
6. Tile independently deduplicable
7. Tile independently versionable
8. Tile independent from source checkpoint format

Kalau delapan invariant ini dipertahankan, Tile-Based Physical Storage benar-benar menjadi physical foundation CNWS X, sementara Columnar Model Graph menjadi logical foundation-nya.

Secara sederhana:

CNWS X
                   │
          ┌────────┴────────┐
          │                 │
       COLUMN             TILE
      logical            physical
          │                 │
 semantic identity      BLAKE3-256
 architecture           immutable
 dependency             dedup
          │             compression
          └──────┬──────────┘
                 │
             model.cd
                 │
              Runtime

Column menjawab: "weight apa ini?"
Tile menjawab: "data fisiknya disimpan dan diambil bagaimana?"


Untuk CNWS X, saya akan menetapkan BLAKE3-256 Content Addressing sebagai spesifikasi inti seperti ini.

1. Canonical Hash

Setiap objek content-addressed menggunakan:

Algorithm: BLAKE3
Output:    256-bit
Digest:    32 bytes
Encoding:  lowercase hexadecimal

Contoh:

b3:7f3a8e...64 hexadecimal characters...

b3: dipakai sebagai domain identifier agar jelas bahwa ID tersebut adalah BLAKE3-256.


---

2. Apa yang di-hash

Tile identity = hash dari canonical uncompressed Tile payload.

source tensor
    ↓
normalization
    ↓
canonical Tile bytes
    ↓
BLAKE3-256
    ↓
Tile ID

Jadi:

tile_id = BLAKE3_256(canonical_payload)

Bukan:

compressed payload
segment offset
filename
revision number
column ID

Dengan begitu Tile yang sama tetap mempunyai ID yang sama walaupun:

berada di segment berbeda

berada di model berbeda

berada di revision berbeda

dikompresi dengan codec berbeda

dipindahkan ke storage berbeda


Ini yang membuat deduplication benar-benar independen dari physical storage.


---

3. Immutable Identity

Setelah:

BLAKE3-256(payload) = Tile ID

maka Tile dianggap immutable.

Jika payload berubah satu byte:

Tile A
BLAKE3 → X

Tile A'
BLAKE3 → Y

Maka A' adalah Tile baru, bukan modifikasi Tile lama.

Ini menjadi dasar revision DAG.


---

4. Deduplication

Registry global:

struct TileRegistry {
    tiles: HashMap<Blake3Hash, TileLocation>,
}

Proses:

BLAKE3-256
                     │
                     ▼
                Tile ID
                     │
              ┌──────┴──────┐
              │             │
          ID exists      ID absent
              │             │
             reuse        store
              │             │
              └──────┬──────┘
                     ▼
                 Tile store

Contoh:

Base Model
 ├── Tile A
 ├── Tile B
 └── Tile C

Coding Revision
 ├── Tile A  ← shared
 ├── Tile B  ← shared
 └── Tile D  ← new

Reasoning Revision
 ├── Tile A  ← shared
 ├── Tile B  ← shared
 └── Tile E  ← new

Secara physical:

A
B
C
D
E

bukan:

A B C
A B D
A B E


---

5. Integrity Verification

BLAKE3 yang sama juga digunakan untuk memverifikasi payload.

actual = blake3(payload);

if actual != tile.id {
    return Err(TileCorruption);
}

Tidak diperlukan SHA-256.

Untuk CNWS X, BLAKE3-256 menjadi primitive tunggal untuk:

Content Identity
       │
       ├── Deduplication
       ├── Integrity
       ├── Tile Identity
       ├── Revision References
       ├── Cross-model Sharing
       └── Object Verification


---

6. Streaming Hashing

Hash tidak boleh membutuhkan seluruh Tile sebagai buffer tambahan.

Implementasinya:

let mut hasher = blake3::Hasher::new();

while let Some(chunk) = reader.read_chunk()? {
    hasher.update(&chunk);
    process_chunk(chunk)?;
}

let digest = hasher.finalize();

Dengan demikian:

Memory ≈ Tile working buffer

bukan:

Memory ≈ Tile + duplicate Tile buffer

Ini penting untuk prinsip Streaming-First CNWS X.


---

7. Compression Independence

Hash dihitung sebelum compression:

Canonical Payload
       │
       ├── BLAKE3-256 ──► Tile ID
       │
       └── Compression ──► Stored Payload

Misalnya:

Canonical:
100 MB

BLAKE3:
abc123...

zstd:
32 MB

Tile tetap:

b3:abc123...

walaupun compression level berubah dari zstd:3 menjadi zstd:9.

Ini penting supaya perubahan storage codec tidak menghasilkan Tile identity baru.


---

8. Representation Independence

Untuk Tile yang memiliki beberapa representation:

Tile
 ├── canonical fp32
 ├── fp8
 ├── int8
 └── int4

Saya menyarankan identity setiap representation juga memiliki BLAKE3-256 sendiri, tetapi parent Tile tetap memiliki canonical identity.

Tile ID
  │
  ├── canonical_id
  ├── fp8_id
  ├── int8_id
  └── int4_id

Sehingga representation dapat dideduplicate secara independen.


---

9. Revision Identity

Revision tidak perlu membuat hash dari seluruh model.

Revision cukup mereferensikan Tile IDs:

{
  "revision": 7,
  "parent": 6,
  "changed_tiles": [
    "b3:...",
    "b3:...",
    "b3:..."
  ]
}

Dengan demikian revision merupakan DAG of immutable content references, bukan kumpulan checkpoint penuh.


---

10. Collision Policy

Karena CNWS menggunakan BLAKE3-256:

Digest = 256 bit

collision dianggap tidak praktis untuk operational content addressing.

Tetapi implementasi tetap harus memperlakukan hash mismatch sebagai identity conflict, bukan diam-diam overwrite.

Aturannya:

same ID + same payload
    → deduplicate

same ID + different payload
    → fatal integrity error


---

11. Manifest Specification

model.cd sebaiknya secara eksplisit menyatakan:

{
  "content_addressing": {
    "algorithm": "BLAKE3",
    "digest_bits": 256,
    "encoding": "hex",
    "domain_prefix": "b3"
  }
}

Dan Tile:

{
  "id": "b3:7f3a8e...",
  "size_bytes": 134217728,
  "dtype": "bf16",
  "shape": [4096, 4096]
}


---

Inti spesifikasinya

CNWS X BLAKE3-256 Content Addressing
────────────────────────────────────

Hash algorithm       BLAKE3
Digest               256-bit / 32 bytes
Identity source      Canonical uncompressed Tile payload
Encoding             lowercase hexadecimal
Prefix               b3:
Mutability           immutable
Compression          independent of identity
Storage location     independent of identity
Revision             references Tile ID
Deduplication        exact content match
Integrity             BLAKE3-256 verification
Streaming             incremental hashing
SHA-256               tidak digunakan

Dengan desain ini, BLAKE3 bukan sekadar checksum di CNWS X. Ia menjadi identity layer yang menghubungkan Tile storage, deduplication, revisioning, dan cross-model sharing.

Canonical Manifest & Zero Format Coupling

Ini adalah salah satu inti paling penting CNWS X.

Konsepnya sederhana:

> Runtime tidak tahu dan tidak peduli model asalnya Safetensors, GGUF, PyTorch, atau format lainnya. Runtime hanya memahami canonical representation CNWS X yang dideskripsikan oleh model.cd.



Arsitekturnya:

Safetensors ──┐
GGUF ─────────┤
PyTorch ──────┤
Custom ───────┘
       │
       ▼
   CNWS Import
       │
       ▼
┌──────────────────────┐
│   Canonical Graph    │
│                      │
│ Columns              │
│ Tiles                │
│ Dependencies         │
│ Representations      │
│ Revisions            │
└──────────┬───────────┘
           │
           ▼
      model.cd
           │
           ▼
┌──────────────────────┐
│     CNWS Runtime     │
│                      │
│ Manifest Loader      │
│ Column Resolver      │
│ Tile Selector        │
│ Cache Manager        │
│ Execution Planner    │
└──────────────────────┘

model.cd menjadi Source of Truth

model.cd bukan sekadar metadata file. Ia adalah root manifest yang mendeskripsikan canonical model:

model.cd
│
├── model identity
├── architecture
├── columns
├── tile references
├── dependency graph
├── representations
├── revision information
├── segment mapping
├── runtime configuration
└── provenance

Misalnya runtime meminta:

resolve_column("model.layer.42.self_attn.q_proj")

Runtime tidak perlu tahu:

apakah bobot tersebut sebelumnya:
- berada di shard 17 Safetensors
- berada di GGUF block tertentu
- berasal dari PyTorch checkpoint
- tersebar di beberapa file

Ia hanya melihat:

Column
   ↓
TileRef
   ↓
Segment
   ↓
Physical bytes

Pemisahan tanggung jawab

Ini menghasilkan boundary yang sangat bersih:

INPUT WORLD
────────────────────────
Safetensors
GGUF
PyTorch
HF
Custom formats

        │
        │ Import / Normalize
        ▼

CNWS WORLD
────────────────────────
Column
Tile
BLAKE3-256
Segment
Revision
model.cd

        │
        │ Runtime API
        ▼

EXECUTION WORLD
────────────────────────
CPU
GPU
NVMe
Cache
Inference

Dengan demikian format reader hanya hidup di conversion layer.

Runtime tidak memiliki dependency terhadap:

SafetensorsReader
GGUFReader
PyTorchReader
HF loader
shard index
shard offset
format-specific metadata

Kenapa ini sangat penting?

Karena kalau format sumber berubah:

Safetensors vX
GGUF vY
format baru Z

yang berubah hanya:

FormatReader
      ↓
Normalizer
      ↓
Canonical representation

Sedangkan:

CNWS Runtime
Column Resolver
Tile Cache
Revision System
Execution Planner

tetap sama.

Itulah makna sebenarnya dari Zero Format Coupling.

Bukan berarti CNWS tidak punya format internal. Justru CNWS punya format sendiri yang sangat terdefinisi.

Yang dimaksud adalah:

> Runtime coupling terhadap external checkpoint format = 0.



Dan ini yang membuat model.cd menjadi canonical contract antara conversion infrastructure dan runtime.


Untuk CNWS X, Selective & Adaptive Runtime sebaiknya menjadi subsistem inti yang berada di antara model.cd dan execution engine. Intinya bukan sekadar cache, tetapi memutuskan weight apa yang perlu dimuat, representasi apa yang dipakai, kapan dimuat, dan kapan dibuang.

1. Tujuan arsitektur

Runtime harus mampu:

tidak memuat seluruh model ke RAM/VRAM

mengambil hanya Column/Tile yang dibutuhkan execution step

memilih representation berdasarkan hardware dan workload

melakukan asynchronous prefetch

memanfaatkan CPU RAM, NVMe, dan GPU VRAM sebagai hierarchy

menghindari duplicate loading untuk Tile yang sama

mendukung MoE top-K selective loading

mempertahankan bounded memory

memungkinkan eviction tanpa mengubah model state


model.cd
                    │
                    ▼
             Manifest Loader
                    │
                    ▼
             Column Resolver
                    │
                    ▼
            Execution Planner
             /            \
            /              \
     Tile Selector    Representation
          │               Selector
          └───────┬────────┘
                  ▼
            Cache Manager
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
     GPU VRAM    CPU RAM    NVMe
       │          │          │
       └──────────┴──────────┘
                  │
                  ▼
            Execution Engine


---

2. Komponen inti

ColumnResolver

Bertanggung jawab menemukan Column berdasarkan semantic ID.

struct ColumnResolver {
    columns: HashMap<ColumnId, ColumnMeta>,
}

impl ColumnResolver {
    fn resolve(&self, id: &ColumnId) -> Result<&ColumnMeta>;
}

Target:

Column lookup: O(1)

Tidak boleh ada scanning seluruh manifest saat inference.


---

TileSelector

Menentukan Tile mana yang benar-benar diperlukan.

struct TileSelector {
    access_policy: AccessPolicy,
}

enum AccessPolicy {
    FullColumn,
    Range,
    TopK,
    Predicate,
    Custom,
}

Contoh:

Attention Q projection
        ↓
Column
        ↓
Tile 0
Tile 1
Tile 2
Tile 3
        ↓
hanya Tile yang dibutuhkan


---

3. Representation Selector

Satu Tile dapat memiliki beberapa representation:

Tile
 ├── fp32
 ├── bf16
 ├── fp16
 ├── fp8
 ├── int8
 └── int4

Runtime memilih representation:

struct RepresentationSelector {
    fn select(
        column: &ColumnMeta,
        hardware: &HardwareProfile,
        workload: &WorkloadProfile,
    ) -> RepresentationId;
}

Keputusan dapat mempertimbangkan:

hardware
├── GPU VRAM
├── GPU compute capability
├── CPU SIMD
├── available RAM
└── NVMe bandwidth

workload
├── latency target
├── throughput target
├── batch size
├── sequence length
└── accuracy policy

Contoh:

GPU VRAM cukup
→ FP8

GPU VRAM terbatas
→ INT8

CPU inference
→ BF16 / INT8

Accuracy-critical layer
→ FP32/BF16

Canonical representation tetap immutable. Runtime hanya memilih representation yang sudah tersedia.


---

4. Cache Hierarchy

Saya akan menetapkan tiga level utama:

L0 — GPU VRAM
     active execution tiles

L1 — CPU RAM
     hot Tile cache

L2 — NVMe
     local persistent Tile store / staging

L3 — Network/Object Storage
     optional remote source

Alur normal:

GPU
 │ miss
 ▼
CPU RAM
 │ miss
 ▼
NVMe
 │ miss
 ▼
Remote Storage

Yang penting, L2 bukan sekadar "cache" jika NVMe merupakan canonical storage lokal. Runtime harus bisa membaca langsung dari .cd tanpa harus menyalin seluruh segment ke RAM.


---

5. Tile Cache

Cache harus menggunakan TileID = BLAKE3-256.

struct TileCache {
    entries: HashMap<Blake3Hash, CacheEntry>,
    capacity_bytes: u64,
    used_bytes: u64,
}

Metadata:

struct CacheEntry {
    tile_id: Blake3Hash,
    representation: RepresentationId,
    size_bytes: u64,
    last_access: Instant,
    access_count: u64,
    priority: Priority,
    residency: Residency,
}

Residency:

GPU
CPU
NVMe

Cache eviction berbasis byte capacity, bukan jumlah Tile.


---

6. Asynchronous Loading

Tile loading tidak boleh blocking execution jika bisa dihindari.

Current Layer
     │
     ├──── compute ────────────────┐
     │                             │
     │                    async prefetch
     │                             │
     ▼                             ▼
Next Layer                    NVMe → RAM → GPU
     │
     ▼
execute

API:

async fn load_tile(
    tile: &TileRef,
    representation: RepresentationId,
) -> Result<TileHandle>;

Runtime harus memiliki:

IO queue
Decode queue
Decompression queue
GPU transfer queue
Prefetch queue

Sehingga:

I/O
  +
decompression
  +
H2D transfer
  +
compute

dapat overlap.


---

7. Prefetch Engine

Prefetch bukan load everything.

Ia harus menggunakan execution graph.

Contoh:

Layer 10 executing
       │
       ├── current tiles → GPU
       │
       └── predictor
             │
             ├── Layer 11 QKV
             ├── Layer 11 MLP
             └── Layer 11 router
                    ↓
                 prefetch

Policy dasar:

enum PrefetchPolicy {
    NextLayer,
    DependencyAware,
    MoETopK,
    Sequential,
    Adaptive,
}

Runtime dapat menaikkan/menurunkan prefetch depth berdasarkan:

cache hit rate
I/O latency
GPU utilization
queue depth
available memory
prediction confidence


---

8. MoE Selective Loading

Ini salah satu bagian terpenting.

Misalnya:

64 experts
Top-K = 2

Runtime tidak melakukan:

load expert 0..63

tetapi:

router
  ↓
top-k
  ↓
expert 7
expert 42
  ↓
resolve Columns
  ↓
resolve Tiles
  ↓
load only required Tiles

Namun ada satu detail penting: expert selection terjadi setelah router computation.

Jadi pipeline:

Input
 ↓
Router
 ↓
Top-K expert IDs
 ↓
Deduplicate expert IDs
 ↓
Tile selection
 ↓
Async load
 ↓
Expert execution

Untuk batch:

Batch
 ├── token 1 → expert 7
 ├── token 2 → expert 7
 ├── token 3 → expert 42
 └── token 4 → expert 7

cukup:

expert 7
expert 42

bukan empat loading operation.


---

9. Memory Budget Manager

Selective runtime harus punya hard memory budget.

struct MemoryBudget {
    gpu_bytes: u64,
    cpu_bytes: u64,
    nvme_bytes: u64,

    reserved_gpu: u64,
    reserved_cpu: u64,
}

Contoh:

GPU VRAM
total      = 24 GB
model      = 18 GB
runtime    = 2 GB
reserve    = 2 GB
cache      = 2 GB

Runtime tidak boleh mengatakan:

> "cache penuh, tapi kita coba load satu Tile lagi."



Harus ada admission control:

requested Tile
      │
      ▼
budget available?
   /       \
 yes        no
 │           │
load      eviction
             │
             ▼
          load

Dengan demikian bounded-memory benar-benar menjadi runtime invariant, bukan sekadar target.


---

10. Tile Admission Policy

Tidak semua Tile yang dibaca harus masuk GPU/CPU cache.

Misalnya Tile:

access_count = 1

dan Tile:

access_count = 10,000

tidak boleh diperlakukan sama.

Policy dapat menggunakan:

frequency
recency
size
load latency
reuse distance
priority

Contoh:

struct AdmissionScore {
    reuse_probability: f32,
    load_cost: f32,
    size_cost: f32,
    execution_priority: f32,
}


---

11. Representation Switching

Runtime tidak perlu menyimpan semua representation di VRAM.

Contoh:

NVMe
 ├── fp32
 ├── bf16
 ├── fp8
 └── int8

CPU RAM
 └── bf16

GPU
 └── fp8

Jika hardware berubah:

H100
→ FP8

CPU
→ BF16

GPU VRAM kecil
→ INT8

Jadi canonical storage tetap sama.


---

12. Integrity

Dengan keputusan terbaru:

Tile identity = BLAKE3-256
Tile integrity = BLAKE3-256
Segment integrity = BLAKE3-256
Manifest integrity = BLAKE3-256

Saat Tile masuk runtime:

read
 ↓
BLAKE3 streaming verification
 ↓
valid?
 ├── yes → cache
 └── no  → reject / recovery

Tidak perlu SHA-256.


---

13. Runtime State Machine

Secara keseluruhan Tile dapat memiliki state:

┌───────────┐
          │ NOT_LOADED│
          └─────┬─────┘
                │
             request
                ▼
          ┌───────────┐
          │ PREFETCH  │
          └─────┬─────┘
                │
              loaded
                ▼
          ┌───────────┐
          │ CPU_CACHE │
          └─────┬─────┘
                │
             H2D copy
                ▼
          ┌───────────┐
          │ GPU_CACHE │
          └─────┬─────┘
                │
             execute
                ▼
          ┌───────────┐
          │   ACTIVE  │
          └─────┬─────┘
                │
             eviction
                ▼
          ┌───────────┐
          │ EVICTED   │
          └───────────┘


---

14. Core API

Saya akan menjaga API runtime tetap kecil:

trait RuntimeResolver {

    fn resolve_column(
        &self,
        column_id: &ColumnId,
    ) -> Result<ColumnHandle>;

    fn resolve_tiles(
        &self,
        column: &ColumnHandle,
        policy: AccessPolicy,
    ) -> Result<Vec<TileHandle>>;

    fn select_representation(
        &self,
        tile: &TileRef,
    ) -> Result<RepresentationId>;

    async fn prefetch(
        &self,
        requests: &[PrefetchRequest],
    ) -> Result<()>;

    fn release(
        &self,
        tile: TileHandle,
    );
}

Execution engine tidak perlu tahu:

.cd binary layout

offset disk

shard

Safetensors

GGUF

BLAKE3 implementation detail

cache eviction

NVMe I/O


Execution engine hanya meminta:

"beri saya weight untuk Column X dalam representation Y"


---

Spesifikasi inti Selective & Adaptive Runtime

Kalau dipadatkan menjadi invariant:

1. Never require full-model residency.
2. Load at Tile granularity.
3. Resolve by semantic Column ID.
4. Select representation at runtime.
5. Prefer asynchronous I/O.
6. Prefetch based on execution dependency.
7. MoE loads only selected experts.
8. Enforce hard memory budgets.
9. Cache by BLAKE3-256 Tile identity.
10. Verify Tile integrity before execution.
11. Evict according to reuse/priority, not simply FIFO.
12. Keep storage format invisible to execution engine.

Ini yang membuat runtime CNWS X bukan sekadar "loader model", tetapi weight orchestration layer: ia menentukan apa yang berada di mana, dalam bentuk apa, dan kapan harus tersedia, sementara execution engine tetap hanya berurusan dengan tensor yang dibutuhkan.

Untuk CNWS X, saya akan menjadikan Incremental Versioning / Revision DAG sebagai subsystem inti, bukan sekadar fitur checkpoint. Spesifikasinya sebaiknya seperti ini.

1. Model dasar

Setiap model memiliki Revision immutable.

Model
 ├── Revision 0  ← base
 │
 ├── Revision 1  ← fine-tune A
 │      │
 │      ├── Revision 2
 │      │
 │      └── Revision 3
 │
 └── Revision 4  ← branch dari Revision 0

Revision bukan salinan model.

Revision adalah:

> manifest + perubahan mapping Tile terhadap parent revision.



Tile yang tidak berubah tetap menggunakan Tile dari ancestor.


---

2. Revision Object

struct Revision {
    id: RevisionID,
    model_id: ModelID,

    revision_number: u64,

    parents: Vec<RevisionID>,

    root_manifest: ManifestID,

    changed_columns: Vec<ColumnID>,

    changed_tiles: Vec<TileID>,

    metadata: RevisionMetadata,

    created_at: Timestamp,

    author: Option<String>,

    message: Option<String>,
}

parents berbentuk Vec karena secara arsitektur CNWS X bisa mendukung:

Revision A
           │
           ├──────┐
           │      │
           ▼      ▼
       Revision B Revision C
           │      │
           └──┬───┘
              ▼
          Revision D

Jadi DAG tidak dibatasi hanya linear history.


---

3. Immutable Revision

Setelah:

Revision 7 → committed

Revision 7 tidak boleh dimodifikasi.

Perubahan selalu menghasilkan:

Revision 7
    │
    ▼
Revision 8

Bukan:

Revision 7
    │
    └── overwrite

Ini penting untuk reproducibility dan rollback.


---

4. Tile-level Delta

Misalnya base:

Revision 0

layer.0.attn.q_proj
 ├── Tile A
 ├── Tile B
 ├── Tile C
 └── Tile D

Fine-tuning hanya mengubah Tile B:

Revision 1

layer.0.attn.q_proj
 ├── Tile A  ──► Revision 0
 ├── Tile B' ──► NEW
 ├── Tile C  ──► Revision 0
 └── Tile D  ──► Revision 0

Jadi delta sebenarnya berada pada level Tile, bukan harus satu tensor penuh.

Ini membuat granularitas versioning CNWS X sangat tinggi.


---

5. Effective Model State

Revision tidak menyimpan seluruh state.

Runtime harus melakukan:

resolve(revision)
      │
      ▼
walk revision → parents
      │
      ▼
collect Tile mappings
      │
      ▼
latest Tile wins
      │
      ▼
Effective Model Graph

Contoh:

Revision 0
A → Tile-001
B → Tile-002
C → Tile-003

Revision 1
B → Tile-004

Effective Revision 1:

A → Tile-001
B → Tile-004
C → Tile-003


---

6. Content Addressing

Karena CNWS X menggunakan BLAKE3-256, Tile identity:

TileID = BLAKE3-256(canonical_tile_content)

Contoh:

Tile-001
BLAKE3:
9f82...a71c

Jika Revision 20 menghasilkan Tile yang identik dengan Tile dari Revision 0:

Revision 0 ──► Tile X
Revision 20 ─► Tile X

Tidak ada physical copy kedua.


---

7. Revision Manifest

Contoh:

{
  "format_version": "1.0.0",
  "model_id": "model-x",

  "revision": {
    "id": "rev-000001",
    "parents": ["rev-000000"],
    "created_at": "2026-08-11T10:00:00Z",
    "message": "coding specialization"
  },

  "changes": {
    "columns": [
      "model.layer.20.mlp.down",
      "model.layer.21.mlp.down"
    ],

    "tiles": {
      "added": [
        "b3:91af...",
        "b3:72de..."
      ],
      "removed": [],
      "replaced": [
        {
          "old": "b3:abc...",
          "new": "b3:91af..."
        }
      ]
    }
  }
}

Namun ada satu prinsip penting:

removed tidak berarti Tile dihapus dari storage.

Tile lama tetap ada selama masih direferensikan revision lain.


---

8. Branching

CNWS X bisa membuat specialization tanpa copy model.

Base
                     │
              Revision 0
             /           \
            /             \
           ▼               ▼
     Coding v1         Reasoning v1
        │                   │
        ▼                   ▼
   Coding v2          Reasoning v2

Keduanya tetap share Tile:

Base Tile
   ▲
   │
   ├──────── Coding
   │
   └──────── Reasoning

Ini yang membuat cross-variant deduplication sangat kuat.


---

9. Merge

DAG memungkinkan merge:

Base
      /    \
     A      B
      \    /
       Merge

Merge harus menggunakan three-way merge:

Base
 ├── Branch A
 └── Branch B

Kemudian:

if A == Base:
    use B

if B == Base:
    use A

if A == B:
    use A

if A != Base && B != Base && A != B:
    CONFLICT

Conflict berada pada level:

Column
    ↓
Tile

bukan sekadar file.


---

10. Garbage Collection

Karena Tile immutable dan shared, CNWS membutuhkan GC.

Misalnya:

Revision 0 ──► Tile A
Revision 1 ──► Tile A
Revision 2 ──► Tile B

Jika Revision 0 dan 1 dihapus:

Tile A
  │
  └── no references

Tile A baru boleh direclaim.

Modelnya:

Revision roots
      │
      ▼
Reachability traversal
      │
      ▼
Referenced Tiles
      │
      ▼
Unreferenced Tiles
      │
      ▼
GC

Jadi reference counting saja tidak cukup; lebih aman memakai reachability dari revision roots sebagai authoritative GC mechanism.


---

11. Rollback

Rollback tidak menghapus revision baru.

Misalnya:

Rev 0
  ↓
Rev 1
  ↓
Rev 2  ← broken
  ↓
Rev 3

Runtime cukup memilih:

active_revision = Rev 1

Semua data tetap immutable.


---

12. Delta Size

CNWS X harus membedakan:

logical model size

dengan:

physical revision delta size

Misalnya:

Base:
1.56 TB

Revision 1:
changed = 78 GB

Revision 2:
changed = 125 GB

Maka:

logical size:
1.56 TB

physical incremental storage:
78 + 125 GB

Tetapi jika Revision 2 menggunakan Tile yang sudah ada dari Revision 1 atau model lain:

additional physical storage = hanya Tile baru


---

13. Revision Resolution Cache

Resolving DAG setiap inference tidak boleh dilakukan.

Runtime sebaiknya menghasilkan:

Revision
   ↓
Resolved Manifest
   ↓
Column Index
   ↓
Tile Mapping

dan menyimpannya sebagai cache.

Startup:

model.cd
   ↓
resolve revision
   ↓
build effective graph
   ↓
O(1) runtime lookup

Jadi DAG adalah version-control structure, bukan hot-path execution structure.


---

14. API inti

Saya akan menjaga API-nya sederhana:

let base = Model::open("model.cd")?;

let rev1 = base
    .revision(0)
    .branch("coding")
    .update_column("layer.20.mlp.down", weights)?
    .commit("coding specialization")?;

Branch:

let reasoning = base
    .revision(0)
    .branch("reasoning")
    .commit("reasoning specialization")?;

Merge:

let merged = model
    .merge(coding, reasoning)?
    .commit("combined specialization")?;

Rollback:

model.set_active_revision(1)?;

GC:

model.gc()?;


---

15. Arsitektur akhirnya

REVISION DAG
                         │
             ┌───────────┴───────────┐
             │                       │
         Revision                 Revision
             │                       │
             ▼                       ▼
        Delta Mapping           Delta Mapping
             │                       │
             └───────────┬───────────┘
                         ▼
                  TILE RESOLUTION
                         │
                         ▼
                 EFFECTIVE GRAPH
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
         Columns                    Tiles
                                     │
                                BLAKE3-256
                                     │
                                     ▼
                              Shared Storage

Inti desainnya: Revision DAG menyimpan perubahan, bukan model. Model efektif selalu merupakan hasil resolusi Revision + ancestors → Column/Tile mapping. Karena Tile immutable dan content-addressed dengan BLAKE3-256, satu Tile dapat dipakai oleh banyak revision, branch, bahkan model berbeda tanpa physical duplication.
