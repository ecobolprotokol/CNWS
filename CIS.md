# CNWS
## Conversion & Import Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Conversion & Import Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (CONVERSION LAYER SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS .cd Format Spec; CNWS Cell & Schema Spec |
| Hulu ke | Implementasi Conversion Pipeline, Importers, Validators |
| Otoritas | Spesifikasi tunggal untuk seluruh external checkpoint → `.cd` |
| Prinsip Dijaga | **Zero Format Coupling** tetap terealisasi |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    .cd Format Spec     Conversion & Import Spec    Implementation
─────────────────────   ────────────────    ────────────────────────    ─────────────
Zero Format Coupling ──► Wire format    ──► Importer behavior       ──► Importer code
Streaming-First          Serialization       Normalization rules         Normalizers
BLAKE3-256               Segment layout      Tiling algorithm            Tile planners
Canonical payload                            Dtype handling              Validators
Cell/Tile model                              Validation                  Converters
```

`[CONV-DOC-1]` Dokumen ini mendefinisikan **bagaimana external checkpoint dikonversi menjadi canonical `.cd`**.

`[CONV-DOC-2]` Dokumen ini adalah **satu-satunya tempat** format eksternal dipahami. Di luar lapisan ini, tidak ada komponen CNWS yang boleh memahami Safetensors, GGUF, PyTorch, atau format eksternal lainnya.

`[CONV-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[CONV-DOC-4]` Jika terjadi konflik dengan .cd Format Spec untuk hal serialization output, .cd Format Spec menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-CONV-01 | Conversion menggunakan bounded buffer default 2 GiB. |
| DF-CONV-02 | Canonical dtype: non-quantized source → source dtype; quantized source → fp32 dequantized. |
| DF-CONV-03 | Tiling default 128 MiB, range 32–256 MiB. |
| DF-CONV-04 | Tiling untuk tensor 2D memecah pada row boundary. |
| DF-CONV-05 | PyTorch importer MUST menggunakan restricted unpickler. |
| DF-CONV-06 | Import MUST deterministik untuk input dan policy yang sama. |
| DF-CONV-07 | Malformed input MUST ditolak dengan error eksplisit, bukan partial import. |
| DF-CONV-08 | Provenance MUST dicatat untuk setiap Tile. |
| DF-CONV-09 | Pre-import validation MUST dilakukan sebelum payload processing. |
| DF-CONV-10 | Format detection menggunakan magic bytes + heuristik. |
| DF-CONV-11 | Custom importer menggunakan trait `FormatImporter`. |
| DF-CONV-12 | Conversion bersifat atomic: berhasil penuh atau gagal penuh. |

---

# 1. Executive Summary

## 1.1 Tujuan Conversion Layer

Conversion Layer adalah satu-satunya komponen CNWS yang memahami format checkpoint eksternal. Ia bertanggung jawab mengubah checkpoint dari berbagai format menjadi canonical `.cd` store.

`[CONV-EXEC-1]` Conversion Layer MUST menjaga **Zero Format Coupling**: setelah conversion, runtime tidak boleh memiliki dependency terhadap format sumber.

`[CONV-EXEC-2]` Conversion Layer MUST bersifat **streaming-first**: peak memory tidak bergantung pada ukuran model.

`[CONV-EXEC-3]` Conversion Layer MUST **deterministik**: input dan policy yang sama menghasilkan `.cd` yang identik.

`[CONV-EXEC-4]` Conversion Layer MUST **atomic**: tidak ada partial `.cd` store.

## 1.2 Zero Format Coupling Realization

```text
┌─────────────────────────────────────────────────────────────┐
│              CONVERSION LAYER (memahami format)             │
│                                                             │
│   SafetensorsImporter                                       │
│   GGUFImporter                                              │
│   PyTorchImporter                                           │
│   CustomImporter                                            │
│         │                                                   │
│         ▼                                                   │
│   Normalizer (map ke semantic CellId)                       │
│         │                                                   │
│         ▼                                                   │
│   Canonical Representation                                  │
└─────────┬───────────────────────────────────────────────────┘
          │
          │  (hanya canonical .cd yang keluar)
          ▼
┌─────────────────────────────────────────────────────────────┐
│              CNWS RUNTIME (TIDAK memahami format)           │
│                                                             │
│   Hanya melihat: Cell, Tile, BLAKE3-256, Manifest          │
│   Tidak melihat: Safetensors, GGUF, PyTorch, shard, offset │
└─────────────────────────────────────────────────────────────┘
```

`[CONV-EXEC-5]` Output Conversion Layer MUST hanya canonical `.cd`.

`[CONV-EXEC-6]` Informasi format sumber MUST hanya ada di provenance, bukan di runtime path.

## 1.3 Format yang Didukung

| Format | Status | Importer |
|---|---|---|
| Safetensors | MUST | SafetensorsImporter |
| GGUF | MUST | GGUFImporter |
| PyTorch (.pt, .bin) | MUST | PyTorchImporter |
| Custom | SHOULD | CustomImporter via trait |

---

# 2. Conversion Pipeline Architecture

## 2.1 Pipeline Overview

```text
External Checkpoint
        │
        ▼
┌─────────────────┐
│ Format Detection│  [CONV-STAGE-1]
└────────┬────────┘
         │ SourceFormat
         ▼
┌─────────────────┐
│ Validation      │  [CONV-STAGE-2]
└────────┬────────┘
         │ ValidationReport
         ▼
┌─────────────────┐
│ Format Reader   │  [CONV-STAGE-3]
└────────┬────────┘
         │ ExternalTensor stream
         ▼
┌─────────────────┐
│ Normalizer      │  [CONV-STAGE-4]
└────────┬────────┘
         │ CanonicalTensor
         ▼
┌─────────────────┐
│ Cell Planner    │  [CONV-STAGE-5]
└────────┬────────┘
         │ Vec<CellSpec>
         ▼
┌─────────────────┐
│ Tile Planner    │  [CONV-STAGE-6]
└────────┬────────┘
         │ Vec<TilePayload>
         ▼
┌─────────────────┐
│ Hasher          │  [CONV-STAGE-7]
└────────┬────────┘
         │ (TilePayload, TileId)
         ▼
┌─────────────────┐
│ Deduplicator    │  [CONV-STAGE-8]
└────────┬────────┘
         │ new Tiles only
         ▼
┌─────────────────┐
│ Encoder         │  [CONV-STAGE-9]
└────────┬────────┘
         │ encoded Tiles
         ▼
┌─────────────────┐
│ Segment Writer  │  [CONV-STAGE-10]
└────────┬────────┘
         │ TileLocation
         ▼
┌─────────────────┐
│ Manifest Builder│  [CONV-STAGE-11]
└────────┬────────┘
         │ Manifest
         ▼
┌─────────────────┐
│ Committer       │  [CONV-STAGE-12]
└────────┬────────┘
         │
         ▼
   model.cd ready
```

## 2.2 Stage Requirements

| Stage | Nama | Tanggung Jawab |
|---|---|---|
| 1 | Format Detection | Identifikasi format sumber |
| 2 | Validation | Validasi integritas sumber |
| 3 | Format Reader | Parse struktur format, stream tensor |
| 4 | Normalizer | Map ke semantic CellId, canonicalize dtype |
| 5 | Cell Planner | Group tensor menjadi Cells |
| 6 | Tile Planner | Split Cells menjadi Tiles |
| 7 | Hasher | BLAKE3-256 streaming |
| 8 | Deduplicator | Cek Tile Registry |
| 9 | Encoder | Optional compression |
| 10 | Segment Writer | Tulis aligned Tiles |
| 11 | Manifest Builder | Bangun canonical manifest |
| 12 | Committer | Atomic commit |

`[CONV-PIPE-1]` Seluruh stage MUST streaming.

`[CONV-PIPE-2]` Tidak ada stage yang boleh memerlukan seluruh model dalam memori.

`[CONV-PIPE-3]` Setiap stage MUST memiliki bounded buffer.

## 2.3 Pipeline Invariants

| ID | Invariant |
|---|---|
| CONV-PIPE-INV-1 | Pipeline MUST streaming-first |
| CONV-PIPE-INV-2 | Peak memory MUST bounded |
| CONV-PIPE-INV-3 | Setiap stage MUST bounded buffer |
| CONV-PIPE-INV-4 | Pipeline MUST atomic |
| CONV-PIPE-INV-5 | Pipeline MUST deterministik |
| CONV-PIPE-INV-6 | Output MUST hanya canonical .cd |

---

# 3. Format Detection

## 3.1 Detection Algorithm

`[CONV-DET-1]` Format detection MUST menggunakan magic bytes + heuristik.

```pseudo
function detect_format(source: Path) -> SourceFormat:
    // Read first 16 bytes
    header = read_bytes(source, 0, 16)
    
    // Check magic bytes
    if header starts with SAFETENSORS_MAGIC:
        return SourceFormat::Safetensors
    
    if header starts with GGUF_MAGIC:
        return SourceFormat::GGUF
    
    if header starts with PYTORCH_MAGIC:
        return SourceFormat::PyTorch
    
    // Heuristic fallback
    if source.extension == ".safetensors":
        return SourceFormat::Safetensors
    
    if source.extension == ".gguf":
        return SourceFormat::GGUF
    
    if source.extension in [".pt", ".pth", ".bin"]:
        return SourceFormat::PyTorch
    
    // Unknown
    return SourceFormat::Unknown
```

## 3.2 Magic Bytes

`[CONV-DET-2]` Magic bytes untuk setiap format:

| Format | Magic Bytes | Offset |
|---|---|---|
| Safetensors | Tidak ada magic tetap; header length + JSON | 0 |
| GGUF | `0x47 0x47 0x55 0x46` ("GGUF") | 0 |
| PyTorch | `0x80` (pickle protocol) atau ZIP magic `0x50 0x4B` | 0 |

`[CONV-DET-3]` Untuk Safetensors, deteksi menggunakan:
1. Read 8 bytes pertama sebagai `u64` LE (header length).
2. Read header JSON.
3. Validasi JSON structure.

## 3.3 Multi-File Detection

`[CONV-DET-4]` Untuk model yang tersebar di beberapa file (sharded):

```pseudo
function detect_sharded(directory: Path) -> ShardSet:
    files = list_files(directory)
    
    // Group by format
    safetensors_files = filter(files, "*.safetensors")
    gguf_files = filter(files, "*.gguf")
    pytorch_files = filter(files, "*.pt") + filter(files, "*.bin")
    
    // Detect sharding
    if safetensors_files.len() > 1:
        return ShardSet::Safetensors(safetensors_files)
    
    // ... similar for other formats
```

`[CONV-DET-5]` Sharded import MUST memproses semua shards sebagai satu logical model.

## 3.4 Detection Invariants

| ID | Invariant |
|---|---|
| CONV-DET-INV-1 | Detection MUST sebelum processing |
| CONV-DET-INV-2 | Unknown format MUST ditolak |
| CONV-DET-INV-3 | Sharded import MUST atomic |

---

# 4. Importer Interface

## 4.1 FormatImporter Trait

`[CONV-IMP-1]` Seluruh importer MUST mengimplementasikan trait `FormatImporter`.

```rust
trait FormatImporter {
    // Identity
    fn source_format(&self) -> SourceFormat;
    fn importer_version(&self) -> String;
    
    // Validation
    fn validate(
        &self,
        source: &SourceDescriptor,
    ) -> Result<ValidationReport>;
    
    // Metadata
    fn read_metadata(
        &mut self,
        source: &SourceDescriptor,
    ) -> Result<SourceMetadata>;
    
    // Streaming tensor access
    fn tensor_stream(
        &mut self,
        source: &SourceDescriptor,
    ) -> Result<TensorStream>;
    
    // Capabilities
    fn supports_streaming(&self) -> bool;
    fn supports_parallel(&self) -> bool;
}
```

## 4.2 SourceDescriptor

```rust
struct SourceDescriptor {
    path: PathBuf,
    format: SourceFormat,
    shard_index: Option<u32>,
    total_shards: Option<u32>,
    options: ImportOptions,
}

struct ImportOptions {
    // Dtype handling
    force_dtype: Option<DataType>,
    dequantize: bool,
    
    // Filtering
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    
    // Streaming
    buffer_size: u64,
    
    // Validation
    strict_validation: bool,
}
```

## 4.3 TensorStream

`[CONV-IMP-2]` TensorStream MUST menghasilkan tensor secara streaming.

```rust
trait TensorStream {
    // Get next tensor metadata (without loading data)
    fn next_metadata(&mut self) -> Result<Option<ExternalTensorMeta>>;
    
    // Read tensor data in chunks
    fn read_chunk(
        &mut self,
        tensor_id: TensorIndex,
        buffer: &mut [u8],
    ) -> Result<usize>;
    
    // Total tensor count
    fn tensor_count(&self) -> usize;
}

struct ExternalTensorMeta {
    name: String,
    shape: Vec<u64>,
    dtype: ExternalDtype,
    offset: u64,          // offset in source file
    size_bytes: u64,
    quantization: Option<QuantizationInfo>,
}
```

`[CONV-IMP-3]` TensorStream MUST NOT memuat seluruh tensor ke memori sekaligus.

`[CONV-IMP-4]` TensorStream MUST mendukung chunked reading.

## 4.4 Importer Registration

`[CONV-IMP-5]` Importer MUST registered dalam registry.

```rust
struct ImporterRegistry {
    importers: HashMap<SourceFormat, Box<dyn FormatImporter>>,
}

impl ImporterRegistry {
    fn register(&mut self, format: SourceFormat, importer: Box<dyn FormatImporter>);
    fn get(&self, format: SourceFormat) -> Result<&dyn FormatImporter>;
}
```

---

# 5. Safetensors Importer

## 5.1 Safetensors Format Overview

`[CONV-ST-1]` Safetensors importer MUST mendukung Safetensors format.

Safetensors structure:

```text
┌──────────────────────────────────────────┐
│ Header length (8 bytes, u64 LE)          │
├──────────────────────────────────────────┤
│ JSON header (variable length)            │
│   {                                      │
│     "tensor_name": {                     │
│       "dtype": "F16",                    │
│       "shape": [4096, 4096],            │
│       "data_offsets": [start, end]      │
│     },                                   │
│     ...                                  │
│   }                                      │
├──────────────────────────────────────────┤
│ Tensor data (concatenated)               │
└──────────────────────────────────────────┘
```

## 5.2 Safetensors Importer Implementation

```rust
struct SafetensorsImporter {
    header: SafetensorsHeader,
    tensor_metadata: Vec<SafetensorsTensorMeta>,
}

struct SafetensorsHeader {
    tensors: HashMap<String, SafetensorsTensorInfo>,
    metadata: HashMap<String, Value>,
}

struct SafetensorsTensorInfo {
    dtype: String,           // "F32", "F16", "BF16", etc.
    shape: Vec<u64>,
    data_offsets: [u64; 2],  // [start, end]
}
```

## 5.3 Safetensors Validation

`[CONV-ST-2]` Safetensors validation MUST memeriksa:

| Check | Requirement |
|---|---|
| Header length valid | MUST < 100 MiB |
| JSON valid | MUST parse sebagai JSON |
| Tensor names unique | MUST tidak ada duplikat |
| Data offsets valid | MUST dalam range file |
| Data offsets non-overlapping | MUST tidak overlap |
| Dtype recognized | MUST dtype dikenal |
| Shape valid | MUST dimensi > 0 |

## 5.4 Safetensors Dtype Mapping

`[CONV-ST-3]` Safetensors dtype mapping ke CNWS DataType:

| Safetensors | CNWS DataType |
|---|---|
| `F32` | F32 |
| `F16` | F16 |
| `BF16` | BF16 |
| `F64` | F64 |
| `I64` | I64 |
| `I32` | I32 |
| `I16` | I16 |
| `I8` | I8 |
| `U8` | U8 |
| `BOOL` | BOOL |

`[CONV-ST-4]` Dtype yang tidak dikenal MUST menghasilkan error.

## 5.5 Safetensors Streaming Read

`[CONV-ST-5]` Safetensors streaming read:

```pseudo
function read_tensor_chunk(file, tensor_info, buffer):
    start_offset = header_size + tensor_info.data_offsets[0]
    end_offset = header_size + tensor_info.data_offsets[1]
    
    // Seek to tensor data
    file.seek(start_offset + bytes_read)
    
    // Read chunk
    bytes_to_read = min(buffer.len(), end_offset - current_position)
    file.read(buffer[:bytes_to_read])
    
    return bytes_to_read
```

`[CONV-ST-6]` Safetensors importer MUST mendukung zero-copy read jika memungkinkan.

## 5.6 Safetensors Sharding

`[CONV-ST-7]` Untuk sharded Safetensors:

```text
model-00001-of-00008.safetensors
model-00002-of-00008.safetensors
...
model-00008-of-00008.safetensors
model.safetensors.index.json
```

`[CONV-ST-8]` Index file MUST dibaca untuk menentukan tensor-to-shard mapping.

`[CONV-ST-9]` Sharded import MUST memproses shards dalam urutan deterministik.

---

# 6. GGUF Importer

## 6.1 GGUF Format Overview

`[CONV-GGUF-1]` GGUF importer MUST mendukung GGUF format.

GGUF structure:

```text
┌──────────────────────────────────────────┐
│ Magic (4 bytes): "GGUF"                  │
├──────────────────────────────────────────┤
│ Version (4 bytes, u32)                   │
├──────────────────────────────────────────┤
│ Tensor count (8 bytes, u64)              │
├──────────────────────────────────────────┤
│ Metadata kv count (8 bytes, u64)         │
├──────────────────────────────────────────┤
│ Metadata key-value pairs                 │
├──────────────────────────────────────────┤
│ Tensor info entries                      │
├──────────────────────────────────────────┤
│ Tensor data (aligned)                    │
└──────────────────────────────────────────┘
```

## 6.2 GGUF Validation

`[CONV-GGUF-2]` GGUF validation MUST memeriksa:

| Check | Requirement |
|---|---|
| Magic valid | MUST "GGUF" |
| Version supported | MUST versi yang didukung |
| Tensor count valid | MUST konsisten dengan entries |
| Metadata valid | MUST parseable |
| Tensor info valid | MUST shape dan dtype valid |
| Data alignment valid | MUST alignment sesuai |

## 6.3 GGUF Quantization Handling

`[CONV-GGUF-3]` GGUF sering menggunakan quantized types. Canonical dtype policy:

| GGUF Type | Quantized? | Canonical Dtype |
|---|---|---|
| F32 | No | F32 |
| F16 | No | F16 |
| Q4_0 | Yes | F32 (dequantized) |
| Q4_1 | Yes | F32 (dequantized) |
| Q5_0 | Yes | F32 (dequantized) |
| Q5_1 | Yes | F32 (dequantized) |
| Q8_0 | Yes | F32 (dequantized) |
| Q2_K | Yes | F32 (dequantized) |
| Q3_K | Yes | F32 (dequantized) |
| Q4_K | Yes | F32 (dequantized) |
| Q5_K | Yes | F32 (dequantized) |
| Q6_K | Yes | F32 (dequantized) |
| IQ1_S | Yes | F32 (dequantized) |
| IQ2_XXS | Yes | F32 (dequantized) |
| IQ4_XS | Yes | F32 (dequantized) |

`[CONV-GGUF-4]` Quantized GGUF tensors MUST di-dequantize ke F32 untuk canonical payload.

`[CONV-GGUF-5]` Dequantization MUST deterministik.

## 6.4 GGUF Dequantization

`[CONV-GGUF-6]` Dequantization algorithm untuk setiap GGUF quantization type MUST diimplementasikan sesuai spesifikasi GGUF.

```pseudo
function dequantize_gguf(tensor: GgufTensor) -> Vec<f32>:
    match tensor.quantization_type:
        case Q4_0:
            return dequantize_q4_0(tensor.data)
        case Q4_K:
            return dequantize_q4_k(tensor.data)
        // ... etc
```

`[CONV-GGUF-7]` Dequantization MUST menghasilkan nilai F32 yang deterministik.

## 6.5 GGUF Tensor Name Mapping

`[CONV-GGUF-8]` GGUF menggunakan nama tensor seperti:

```text
token_embd.weight
blk.0.attn_q.weight
blk.0.attn_k.weight
blk.0.attn_v.weight
blk.0.attn_output.weight
blk.0.ffn_gate.weight
blk.0.ffn_up.weight
blk.0.ffn_down.weight
output.weight
```

`[CONV-GGUF-9]` GGUF tensor names MUST dipetakan ke semantic CellId melalui normalization rules (§10).

## 6.6 GGUF Metadata

`[CONV-GGUF-10]` GGUF metadata MUST di-extract untuk provenance:

| Metadata Key | Deskripsi |
|---|---|
| `general.architecture` | Architecture type |
| `general.name` | Model name |
| `<arch>.context_length` | Context length |
| `<arch>.embedding_length` | Embedding dimension |
| `<arch>.block_count` | Number of layers |
| `<arch>.attention.head_count` | Number of attention heads |
| `<arch>.expert_count` | Number of experts (MoE) |

---

# 7. PyTorch Importer

## 7.1 PyTorch Format Overview

`[CONV-PT-1]` PyTorch importer MUST mendukung PyTorch checkpoint formats:
- `.pt` / `.pth` (pickle-based)
- `.bin` (pickle-based, sering digunakan HuggingFace)
- PyTorch ZIP format

`[CONV-PT-2]` PyTorch importer MUST menggunakan **restricted unpickling**.

## 7.2 Restricted Unpickling

`[CONV-PT-3]` PyTorch checkpoint menggunakan Python pickle, yang dapat mengeksekusi kode arbitrer. Ini adalah risiko keamanan serius.

`[CONV-PT-4]` PyTorch importer MUST menggunakan restricted unpickler yang hanya mengizinkan:

| Allowed | Description |
|---|---|
| Tensor data | Raw tensor bytes |
| Tensor metadata | Shape, dtype, strides |
| OrderedDict | State dict structure |
| Primitive types | int, float, str, bool, None |
| torch.dtype | Dtype enums |
| torch.Size | Shape tuples |

`[CONV-PT-5]` PyTorch importer MUST menolak:

| Rejected | Reason |
|---|---|
| Arbitrary Python objects | Code execution risk |
| Lambda functions | Code execution risk |
| Class instances | Code execution risk |
| Module references | Code execution risk |
| File handles | Security risk |
| Network references | Security risk |

## 7.3 Restricted Unpickler Implementation

`[CONV-PT-6]` Restricted unpickler MUST:

1. Menggunakan allowlist class yang ketat.
2. Menolak semua class yang tidak ada di allowlist.
3. Tidak mengeksekusi `__reduce__` methods.
4. Tidak mengizinkan `subprocess`, `os`, `sys`, atau module berbahaya.
5. Membatasi ukuran object yang di-unpickle.

```rust
struct RestrictedUnpickler {
    allowed_classes: HashSet<String>,
    max_object_size: u64,
    max_depth: u32,
}

impl RestrictedUnpickler {
    fn new() -> Self {
        RestrictedUnpickler {
            allowed_classes: allowed_pytorch_classes(),
            max_object_size: 100 * GB,
            max_depth: 100,
        }
    }
    
    fn load(&self, data: &[u8]) -> Result<PyTorchStateDict> {
        // Parse pickle stream with restrictions
        // Reject any disallowed class
        // Return only state dict with tensors
    }
}

fn allowed_pytorch_classes() -> HashSet<String> {
    let mut allowed = HashSet::new();
    allowed.insert("collections.OrderedDict");
    allowed.insert("torch._utils._rebuild_tensor_v2");
    allowed.insert("torch.storage._load_from_bytes");
    allowed.insert("torch.BFloat16Storage");
    allowed.insert("torch.FloatStorage");
    allowed.insert("torch.HalfStorage");
    allowed.insert("torch.LongStorage");
    allowed.insert("torch.IntStorage");
    allowed.insert("torch.ShortStorage");
    allowed.insert("torch.CharStorage");
    allowed.insert("torch.ByteStorage");
    allowed.insert("torch.BoolStorage");
    // ... other safe storage types
    allowed
}
```

## 7.4 PyTorch Safe Loading

`[CONV-PT-7]` Untuk PyTorch versi yang mendukung, importer SHOULD menggunakan `weights_only=True` equivalent.

`[CONV-PT-8]` Jika checkpoint mengandung object non-tensor yang tidak aman:

```pseudo
function handle_unsafe_object(obj):
    if obj.type in ALLOWED_TYPES:
        return process(obj)
    else:
        log_warning("Skipping unsafe object: {}", obj.type)
        return skip(obj)
        // NOT execute, NOT instantiate
```

`[CONV-PT-9]` Unsafe objects MUST di-skip, bukan dieksekusi.

## 7.5 PyTorch State Dict Extraction

`[CONV-PT-10]` PyTorch state dict extraction:

```pseudo
function extract_state_dict(checkpoint):
    // Load with restricted unpickler
    state_dict = restricted_unpickle(checkpoint)
    
    // Handle nested structures
    if "state_dict" in state_dict:
        state_dict = state_dict["state_dict"]
    
    if "model" in state_dict:
        state_dict = state_dict["model"]
    
    // Extract tensors only
    tensors = {}
    for key, value in state_dict:
        if is_tensor(value):
            tensors[key] = value
    
    return tensors
```

## 7.6 PyTorch Tensor Name Mapping

`[CONV-PT-11]` PyTorch menggunakan nama tensor seperti:

```text
model.embed_tokens.weight
model.layers.0.self_attn.q_proj.weight
model.layers.0.self_attn.k_proj.weight
model.layers.0.self_attn.v_proj.weight
model.layers.0.self_attn.o_proj.weight
model.layers.0.mlp.gate_proj.weight
model.layers.0.mlp.up_proj.weight
model.layers.0.mlp.down_proj.weight
model.norm.weight
lm_head.weight
```

`[CONV-PT-12]` PyTorch tensor names MUST dipetakan ke semantic CellId melalui normalization rules (§10).

## 7.7 PyTorch Sharding

`[CONV-PT-13]` Untuk sharded PyTorch (mis. HuggingFace):

```text
pytorch_model-00001-of-00008.bin
pytorch_model-00002-of-00008.bin
...
pytorch_model-00008-of-00008.bin
pytorch_model.bin.index.json
```

`[CONV-PT-14]` Index file MUST dibaca untuk menentukan tensor-to-shard mapping.

---

# 8. Custom Importer Interface

## 8.1 Custom Importer Requirements

`[CONV-CUST-1]` Custom importer MUST mengimplementasikan trait `FormatImporter`.

`[CONV-CUST-2]` Custom importer MUST registered dalam ImporterRegistry.

`[CONV-CUST-3]` Custom importer MUST mematuhi seluruh invariant conversion pipeline.

## 8.2 Custom Importer Registration

```rust
// Example: Register custom importer
let mut registry = ImporterRegistry::new();
registry.register(
    SourceFormat::Custom("my_format".to_string()),
    Box::new(MyFormatImporter::new()),
);
```

## 8.3 Custom Importer Validation

`[CONV-CUST-4]` Custom importer MUST mengimplementasikan validasi yang setara dengan built-in importers.

`[CONV-CUST-5]` Custom importer MUST menghasilkan ExternalTensorMeta yang konsisten.

## 8.4 Custom Importer Restrictions

`[CONV-CUST-6]` Custom importer MUST NOT:
- Mengeksekusi kode dari checkpoint.
- Mengakses jaringan tanpa izin eksplisit.
- Memodifikasi source file.
- Menghasilkan non-deterministic output.

---

# 9. Tensor Normalization

## 9.1 Purpose

Normalizer memetakan tensor eksternal ke semantic CellId yang independen dari format sumber.

`[CONV-NORM-1]` Normalizer MUST menghasilkan CellId yang deterministik dan independen dari format.

`[CONV-NORM-2]` Normalizer adalah kunci Zero Format Coupling.

## 9.2 Normalization Pipeline

```text
External Tensor
      │
      ▼
┌─────────────────┐
│ Name Mapping    │  Map source name → canonical pattern
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Dtype Normalize │  Canonicalize dtype
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Layout Normalize│  Ensure row-major
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Cell Type Infer │  Determine CellType
└────────┬────────┘
         │
         ▼
Canonical Cell Tensor
```

## 9.3 Name Mapping Rules

`[CONV-NORM-3]` Name mapping menggunakan pattern-based rules.

### 9.3.1 Common Patterns

| Source Pattern | Canonical CellId |
|---|---|
| `model.embed_tokens.weight` | `model.embedding.token_embedding` |
| `token_embd.weight` (GGUF) | `model.embedding.token_embedding` |
| `model.layers.{N}.self_attn.q_proj.weight` | `model.layer.{N}.self_attn.q_proj` |
| `blk.{N}.attn_q.weight` (GGUF) | `model.layer.{N}.self_attn.q_proj` |
| `model.layers.{N}.self_attn.k_proj.weight` | `model.layer.{N}.self_attn.k_proj` |
| `blk.{N}.attn_k.weight` (GGUF) | `model.layer.{N}.self_attn.k_proj` |
| `model.layers.{N}.self_attn.v_proj.weight` | `model.layer.{N}.self_attn.v_proj` |
| `blk.{N}.attn_v.weight` (GGUF) | `model.layer.{N}.self_attn.v_proj` |
| `model.layers.{N}.self_attn.o_proj.weight` | `model.layer.{N}.self_attn.out` |
| `blk.{N}.attn_output.weight` (GGUF) | `model.layer.{N}.self_attn.out` |
| `model.layers.{N}.mlp.gate_proj.weight` | `model.layer.{N}.mlp.gate` |
| `blk.{N}.ffn_gate.weight` (GGUF) | `model.layer.{N}.mlp.gate` |
| `model.layers.{N}.mlp.up_proj.weight` | `model.layer.{N}.mlp.up` |
| `blk.{N}.ffn_up.weight` (GGUF) | `model.layer.{N}.mlp.up` |
| `model.layers.{N}.mlp.down_proj.weight` | `model.layer.{N}.mlp.down` |
| `blk.{N}.ffn_down.weight` (GGUF) | `model.layer.{N}.mlp.down` |
| `model.layers.{N}.input_layernorm.weight` | `model.layer.{N}.input_norm` |
| `blk.{N}.attn_norm.weight` (GGUF) | `model.layer.{N}.input_norm` |
| `model.layers.{N}.post_attention_layernorm.weight` | `model.layer.{N}.post_attn_norm` |
| `blk.{N}.ffn_norm.weight` (GGUF) | `model.layer.{N}.post_attn_norm` |
| `model.norm.weight` | `model.final_norm` |
| `output_norm.weight` (GGUF) | `model.final_norm` |
| `lm_head.weight` | `model.lm_head` |
| `output.weight` (GGUF) | `model.lm_head` |

### 9.3.2 MoE Patterns

| Source Pattern | Canonical CellId |
|---|---|
| `model.layers.{N}.block_sparse_moe.gate.weight` | `model.layer.{N}.moe.router` |
| `blk.{N}.ffn_gate_inp.weight` (GGUF) | `model.layer.{N}.moe.router` |
| `model.layers.{N}.block_sparse_moe.experts.{E}.w1.weight` | `model.layer.{N}.moe.expert.{E}.gate` |
| `blk.{N}.ffn_gate.{E}.weight` (GGUF) | `model.layer.{N}.moe.expert.{E}.gate` |
| `model.layers.{N}.block_sparse_moe.experts.{E}.w2.weight` | `model.layer.{N}.moe.expert.{E}.down` |
| `blk.{N}.ffn_down.{E}.weight` (GGUF) | `model.layer.{N}.moe.expert.{E}.down` |
| `model.layers.{N}.block_sparse_moe.experts.{E}.w3.weight` | `model.layer.{N}.moe.expert.{E}.up` |
| `blk.{N}.ffn_up.{E}.weight` (GGUF) | `model.layer.{N}.moe.expert.{E}.up` |

## 9.4 Name Mapping Implementation

`[CONV-NORM-4]` Name mapping MUST menggunakan rule-based system dengan fallback.

```rust
struct NameMappingRule {
    pattern: Regex,
    replacement: String,
    cell_type: CellType,
}

struct NameMapper {
    rules: Vec<NameMappingRule>,
    fallback: Option<Box<dyn Fn(&str) -> Option<String>>>,
}

impl NameMapper {
    fn map(&self, source_name: &str) -> Result<CellId> {
        // Try rules in order
        for rule in &self.rules {
            if let Some(mapped) = rule.pattern.replace(source_name, &rule.replacement) {
                return Ok(CellId::from(mapped));
            }
        }
        
        // Try fallback
        if let Some(fallback) = &self.fallback {
            if let Some(mapped) = fallback(source_name) {
                return Ok(CellId::from(mapped));
            }
        }
        
        // Unknown
        Err(Error::UnknownTensorName(source_name.to_string()))
    }
}
```

`[CONV-NORM-5]` Unknown tensor names MUST menghasilkan error, bukan silent skip.

`[CONV-NORM-6]` Name mapping rules MUST configurable per architecture.

## 9.5 Architecture-Specific Mappings

`[CONV-NORM-7]` Setiap architecture MAY memiliki mapping rules spesifik.

```rust
struct ArchitectureMapping {
    architecture_type: String,
    rules: Vec<NameMappingRule>,
}

// Example architectures
let llama_mapping = ArchitectureMapping {
    architecture_type: "llama".to_string(),
    rules: vec![
        // Llama-specific rules
    ],
};

let qwen_mapping = ArchitectureMapping {
    architecture_type: "qwen".to_string(),
    rules: vec![
        // Qwen-specific rules
    ],
};
```

## 9.6 Normalization Invariants

| ID | Invariant |
|---|---|
| CONV-NORM-INV-1 | CellId MUST deterministik |
| CONV-NORM-INV-2 | CellId MUST independen dari format sumber |
| CONV-NORM-INV-3 | Unknown tensor names MUST error |
| CONV-NORM-INV-4 | Name mapping MUST configurable |
| CONV-NORM-INV-5 | Normalization MUST NOT mengubah data |

---

# 10. Tensor → Cell Mapping

## 10.1 Cell Type Inference

`[CONV-MAP-1]` CellType MUST di-infer dari tensor name dan metadata.

```pseudo
function infer_cell_type(tensor_name: String, metadata: TensorMeta) -> CellType:
    // Use name mapping rules
    cell_id = name_mapper.map(tensor_name)
    
    // Determine CellType from canonical pattern
    match cell_id:
        case "model.embedding.*":
            return CellType::EMBEDDING
        case "*.self_attn.q_proj":
            return CellType::ATTENTION_Q_PROJ
        case "*.self_attn.k_proj":
            return CellType::ATTENTION_K_PROJ
        case "*.self_attn.v_proj":
            return CellType::ATTENTION_V_PROJ
        case "*.self_attn.out":
            return CellType::ATTENTION_OUT
        case "*.mlp.gate":
            return CellType::MLP_GATE
        case "*.mlp.up":
            return CellType::MLP_UP
        case "*.mlp.down":
            return CellType::MLP_DOWN
        case "*.moe.router":
            return CellType::EXPERT_ROUTE
        case "*.moe.expert.*.gate":
            return CellType::EXPERT_GATE
        case "*.moe.expert.*.up":
            return CellType::MLP_UP  // or EXPERT_WEIGHT
        case "*.moe.expert.*.down":
            return CellType::MLP_DOWN  // or EXPERT_WEIGHT
        case "*.layernorm.*" | "*.norm.*":
            return CellType::LAYERNORM_WEIGHT
        case "model.lm_head":
            return CellType::LM_HEAD
        case _:
            return CellType::CUSTOM(infer_custom_type(tensor_name))
```

## 10.2 Cell Metadata Extraction

`[CONV-MAP-2]` Cell metadata MUST di-extract dari tensor name:

```pseudo
function extract_metadata(cell_id: String, tensor_meta: TensorMeta) -> CellMetadata:
    metadata = CellMetadata::new()
    
    // Extract layer index
    if let Some(layer) = extract_layer_index(cell_id):
        metadata.layer_index = Some(layer)
    
    // Extract attention head
    if let Some(head) = extract_attention_head(cell_id):
        metadata.attention_head = Some(head)
    
    // Extract expert index
    if let Some(expert) = extract_expert_index(cell_id):
        metadata.expert_index = Some(expert)
    
    // Architecture
    metadata.architecture = detect_architecture(tensor_meta)
    
    return metadata
```

## 10.3 Cell Grouping

`[CONV-MAP-3]` Tensors yang termasuk dalam satu logical Cell MUST digroup.

Contoh: QKV projection yang digabung dalam satu tensor:

```text
Source: model.layers.0.self_attn.qkv_proj.weight  (shape: [3*4096, 4096])
                    │
                    ▼ Split
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   q_proj       k_proj      v_proj
   [4096,4096]  [4096,4096] [4096,4096]
```

`[CONV-MAP-4]` Fused tensors MUST di-split menjadi Cells terpisah jika semantic berbeda.

## 10.4 Mapping Invariants

| ID | Invariant |
|---|---|
| CONV-MAP-INV-1 | CellType MUST deterministic |
| CONV-MAP-INV-2 | Cell metadata MUST lengkap |
| CONV-MAP-INV-3 | Fused tensors MUST di-split jika semantic berbeda |
| CONV-MAP-INV-4 | Mapping MUST konsisten antar format |

---

# 11. Tiling Algorithm

## 11.1 Purpose

Tile Planner memecah Cell menjadi Tiles untuk storage.

`[CONV-TILE-1]` Tiling MUST deterministik.

`[CONV-TILE-2]` Tiling MUST menghormati alignment.

## 11.2 Tiling Parameters

```rust
struct TilingConfig {
    target_tile_size: u64,      // default 128 MiB
    min_tile_size: u64,         // default 32 MiB
    max_tile_size: u64,         // default 256 MiB
    alignment: u64,             // default 4 KiB
    preferred_alignment: u64,   // default 64 KiB
}
```

`[CONV-TILE-3]` Default tiling parameters:

| Parameter | Default | Range |
|---|---|---|
| `target_tile_size` | 128 MiB | 32–256 MiB |
| `min_tile_size` | 32 MiB | — |
| `max_tile_size` | 256 MiB | — |
| `alignment` | 4 KiB | — |
| `preferred_alignment` | 64 KiB | — |

## 11.3 Tiling Algorithm

`[CONV-TILE-4]` Tiling algorithm untuk tensor N-dimensi:

```pseudo
function plan_tiles(tensor: CanonicalTensor, config: TilingConfig) -> Vec<TileSpec>:
    shape = tensor.shape
    dtype_size = bytes_per_element(tensor.dtype)
    total_elements = product(shape)
    total_bytes = total_elements * dtype_size
    
    // If tensor smaller than min_tile_size, single tile
    if total_bytes <= config.min_tile_size:
        return [single_tile(tensor)]
    
    // Determine split dimension
    // For 2D: split along dimension 0 (rows)
    // For 1D: single tile if small, else split
    // For ND: split along outermost dimension
    
    split_dim = determine_split_dimension(shape, dtype_size, config)
    
    // Calculate elements per tile
    elements_per_tile = config.target_tile_size / dtype_size
    
    // Calculate rows per tile (for 2D)
    if shape.len() == 2:
        row_size = shape[1]  // elements per row
        rows_per_tile = elements_per_tile / row_size
        
        // Ensure at least 1 row per tile
        rows_per_tile = max(1, rows_per_tile)
        
        // Generate tile specs
        tiles = []
        row_offset = 0
        while row_offset < shape[0]:
            tile_rows = min(rows_per_tile, shape[0] - row_offset)
            tiles.append(TileSpec {
                offset: [row_offset, 0],
                size: [tile_rows, shape[1]],
            })
            row_offset += tile_rows
        
        return tiles
    
    // For other dimensions, similar logic
    return split_along_dimension(tensor, split_dim, config)
```

## 11.4 Split Dimension Selection

`[CONV-TILE-5]` Split dimension selection:

| Tensor Dim | Split Strategy |
|---|---|
| 1D | Single tile jika kecil, else split along dim 0 |
| 2D | Split along dim 0 (rows) |
| 3D | Split along dim 0 |
| 4D+ | Split along dim 0 |

`[CONV-TILE-6]` Untuk 2D tensors, split MUST pada row boundary.

`[CONV-TILE-7]` Tile MUST NOT memecah di tengah row.

## 11.5 Tile Size Adaptation

`[CONV-TILE-8]` Tile size MAY diadaptasi berdasarkan:

| Factor | Adaptation |
|---|---|
| Available RAM | Smaller tiles jika RAM terbatas |
| Tensor shape | Adjust untuk shape yang tidak biasa |
| Compression ratio | Larger tiles jika compression tinggi |
| Access pattern | Smaller tiles untuk random access |

## 11.6 Tile Spec

```rust
struct TileSpec {
    offset: Vec<u64>,       // element offset dalam tensor
    size: Vec<u64>,         // element count per dimensi
    payload_size: u64,      // bytes
}
```

## 11.7 Tiling Examples

### 11.7.1 2D Tensor

```text
Tensor: [4096, 4096], dtype: bf16 (2 bytes)
Total: 4096 * 4096 * 2 = 32 MiB

Tile size target: 128 MiB
→ Single tile (32 MiB < 128 MiB)

Result: 1 tile
  Tile 0: offset [0, 0], size [4096, 4096]
```

### 11.7.2 Large 2D Tensor

```text
Tensor: [32768, 8192], dtype: bf16 (2 bytes)
Total: 32768 * 8192 * 2 = 512 MiB

Tile size target: 128 MiB
Row size: 8192 * 2 = 16 KiB
Rows per tile: 128 MiB / 16 KiB = 8192 rows

Result: 4 tiles
  Tile 0: offset [0, 0],     size [8192, 8192]
  Tile 1: offset [8192, 0],  size [8192, 8192]
  Tile 2: offset [16384, 0], size [8192, 8192]
  Tile 3: offset [24576, 0], size [8192, 8192]
```

### 11.7.3 1D Tensor

```text
Tensor: [4096], dtype: bf16 (2 bytes)
Total: 4096 * 2 = 8 KiB

→ Single tile (8 KiB << 32 MiB min)

Result: 1 tile
  Tile 0: offset [0], size [4096]
```

## 11.8 Tiling Invariants

| ID | Invariant |
|---|---|
| CONV-TILE-INV-1 | Tiling MUST deterministik |
| CONV-TILE-INV-2 | Tile boundaries MUST pada row boundary (untuk 2D) |
| CONV-TILE-INV-3 | Tile size MUST dalam range [min, max] |
| CONV-TILE-INV-4 | Tile concatenation MUST menghasilkan tensor lengkap |
| CONV-TILE-INV-5 | Tiles MUST non-overlapping |
| CONV-TILE-INV-6 | Tiles MUST cover seluruh tensor |

---

# 12. Streaming Constraints

## 12.1 Bounded Memory

`[CONV-STREAM-1]` Conversion MUST bounded-memory.

`[CONV-STREAM-2]` Peak RAM MUST NOT sebanding dengan ukuran model.

`[CONV-STREAM-3]` Default working set budget: 2 GiB.

## 12.2 Buffer Management

```rust
struct StreamingBuffer {
    capacity: u64,
    used: u64,
    buffer: Vec<u8>,
}

impl StreamingBuffer {
    fn new(capacity: u64) -> Self;
    fn read_chunk(&mut self, reader: &mut dyn Read) -> Result<usize>;
    fn is_full(&self) -> bool;
    fn flush(&mut self) -> Result<()>;
}
```

`[CONV-STREAM-4]` Buffer capacity MUST configurable.

`[CONV-STREAM-5]` Default buffer capacity: 64 MiB per tensor chunk.

## 12.3 Backpressure

`[CONV-STREAM-6]` Pipeline MUST menerapkan backpressure jika downstream lambat.

```pseudo
function process_with_backpressure(upstream, downstream):
    while upstream.has_next():
        item = upstream.next()
        
        // Wait if downstream is full
        while downstream.is_full():
            downstream.process_one()
        
        downstream.enqueue(item)
```

## 12.4 Parallel Streaming

`[CONV-STREAM-7]` Conversion MAY parallel untuk independent tensors.

`[CONV-STREAM-8]` Parallel conversion MUST tetap bounded-memory.

`[CONV-STREAM-9]` Parallel conversion MUST deterministik.

## 12.5 Streaming Invariants

| ID | Invariant |
|---|---|
| CONV-STREAM-INV-1 | Peak memory MUST bounded |
| CONV-STREAM-INV-2 | Buffer MUST configurable |
| CONV-STREAM-INV-3 | Backpressure MUST diterapkan |
| CONV-STREAM-INV-4 | Parallel conversion MUST bounded-memory |
| CONV-STREAM-INV-5 | Streaming MUST NOT mengubah output |

---

# 13. Dtype Handling

## 13.1 Canonical Dtype Policy

`[CONV-DTYPE-1]` Canonical dtype policy:

| Source Condition | Canonical Dtype |
|---|---|
| Non-quantized source | Source dtype |
| Quantized source | F32 (dequantized) |

`[CONV-DTYPE-2]` Kebijakan ini MUST deterministik.

## 13.2 Dtype Mapping Table

`[CONV-DTYPE-3]` Dtype mapping untuk setiap format:

### Safetensors

| Safetensors Dtype | Canonical Dtype |
|---|---|
| F32 | F32 |
| F16 | F16 |
| BF16 | BF16 |
| F64 | F64 |
| I64 | I64 |
| I32 | I32 |
| I16 | I16 |
| I8 | I8 |
| U8 | U8 |
| BOOL | BOOL |

### GGUF

| GGUF Dtype | Quantized | Canonical Dtype |
|---|---|---|
| F32 | No | F32 |
| F16 | No | F16 |
| Q* | Yes | F32 (dequantized) |
| IQ* | Yes | F32 (dequantized) |

### PyTorch

| PyTorch Dtype | Canonical Dtype |
|---|---|
| float32 | F32 |
| float16 | F16 |
| bfloat16 | BF16 |
| float64 | F64 |
| int64 | I64 |
| int32 | I32 |
| int16 | I16 |
| int8 | I8 |
| uint8 | U8 |
| bool | BOOL |

## 13.3 Dtype Conversion

`[CONV-DTYPE-4]` Dtype conversion rules:

| From | To | Allowed | Notes |
|---|---|---|---|
| F32 | F16, BF16 | YES | Narrowing (lossy) |
| F16, BF16 | F32 | YES | Widening (lossless) |
| F64 | F32 | YES | Narrowing |
| I8 | I16, I32, I64 | YES | Widening |
| I32 | I8 | NO | Lossy |
| F32 | I32 | NO | Different domain |

`[CONV-DTYPE-5]` Dtype conversion MUST hanya dilakukan jika diperlukan untuk canonical policy.

`[CONV-DTYPE-6]` Dtype conversion MUST deterministik.

## 13.4 Quantization Detection

`[CONV-DTYPE-7]` Quantization detection:

```pseudo
function is_quantized(tensor_meta: ExternalTensorMeta) -> bool:
    // GGUF quantized types
    if tensor_meta.dtype in GGUF_QUANTIZED_TYPES:
        return true
    
    // Check metadata for quantization info
    if tensor_meta.quantization is not None:
        return true
    
    return false
```

## 13.5 Dtype Invariants

| ID | Invariant |
|---|---|
| CONV-DTYPE-INV-1 | Canonical dtype MUST deterministik |
| CONV-DTYPE-INV-2 | Quantized source MUST dequantize ke F32 |
| CONV-DTYPE-INV-3 | Dtype conversion MUST deterministic |
| CONV-DTYPE-INV-4 | Unknown dtype MUST error |

---

# 14. Shape Handling

## 14.1 Shape Validation

`[CONV-SHAPE-1]` Shape validation MUST memeriksa:

| Check | Requirement |
|---|---|
| Dimensions > 0 | MUST setiap dimensi > 0 |
| Total elements | MUST total elements > 0 |
| Size consistency | MUST shape konsisten dengan data size |
| Reasonable bounds | MUST dimensi dalam bounds yang wajar |

## 14.2 Shape Normalization

`[CONV-SHAPE-2]` Shape normalization:

```pseudo
function normalize_shape(shape: Vec<u64>, dtype: DataType) -> Vec<u64>:
    // Remove leading/trailing 1s if appropriate
    // (implementation-defined, but must be deterministic)
    
    // Ensure shape is non-empty
    if shape.is_empty():
        return [1]  // scalar
    
    return shape
```

## 14.3 Dynamic Shapes

`[CONV-SHAPE-3]` Untuk tensors dengan dynamic dimensions:

`[CONV-SHAPE-4]` Dynamic dimensions MUST ditandai dalam Cell schema.

## 14.4 Edge Cases

| Edge Case | Handling |
|---|---|
| Scalar tensor (shape []) | Treat sebagai shape [1] |
| Empty tensor (0 elements) | MUST error |
| Very large single dimension | Split jika > max_tile_size |
| Very many dimensions | Supported, tapi split pada dim 0 |

## 14.5 Shape Invariants

| ID | Invariant |
|---|---|
| CONV-SHAPE-INV-1 | Shape MUST valid |
| CONV-SHAPE-INV-2 | Shape MUST konsisten dengan data |
| CONV-SHAPE-INV-3 | Empty tensor MUST error |
| CONV-SHAPE-INV-4 | Shape normalization MUST deterministik |

---

# 15. Malformed Input Handling

## 15.1 Malformed Input Categories

`[CONV-MAL-1]` Malformed input MUST ditolak dengan error eksplisit.

| Category | Example | Handling |
|---|---|---|
| Corrupt header | Invalid magic, truncated header | Error: `CNWS-E-IMPORT-CORRUPT` |
| Invalid metadata | Shape mismatch, unknown dtype | Error: `CNWS-E-IMPORT-INVALID` |
| Truncated data | Data size < declared size | Error: `CNWS-E-IMPORT-TRUNCATED` |
| Checksum mismatch | Hash verification failed | Error: `CNWS-E-IMPORT-CHECKSUM` |
| Unsafe content | PyTorch arbitrary object | Error: `CNWS-E-IMPORT-UNSAFE` |
| Unknown tensor name | Cannot map to CellId | Error: `CNWS-E-IMPORT-UNKNOWN` |

## 15.2 Validation Before Processing

`[CONV-MAL-2]` Validation MUST dilakukan sebelum payload processing.

```pseudo
function validate_before_processing(source):
    // 1. Format detection
    format = detect_format(source)
    if format == Unknown:
        return Error::UnknownFormat
    
    // 2. Header validation
    header = read_header(source)
    if not validate_header(header):
        return Error::InvalidHeader
    
    // 3. Metadata validation
    metadata = read_metadata(source)
    if not validate_metadata(metadata):
        return Error::InvalidMetadata
    
    // 4. Size validation
    file_size = get_file_size(source)
    declared_size = compute_declared_size(metadata)
    if file_size < declared_size:
        return Error::TruncatedFile
    
    return Ok
```

## 15.3 Partial Import Prevention

`[CONV-MAL-3]` Conversion MUST atomic: berhasil penuh atau gagal penuh.

`[CONV-MAL-4]` Jika error terjadi di tengah conversion:

1. Stop processing.
2. Cleanup staging area.
3. Do NOT commit partial manifest.
4. Report error dengan detail.

## 15.4 Recovery from Malformed Input

`[CONV-MAL-5]` Malformed input MUST NOT menghasilkan partial `.cd` store.

`[CONV-MAL-6]` Error report MUST mencakup:

- Error code
- Error message
- Source file path
- Tensor name (jika applicable)
- Byte offset (jika applicable)
- Suggested action

## 15.5 Malformed Input Invariants

| ID | Invariant |
|---|---|
| CONV-MAL-INV-1 | Malformed input MUST ditolak |
| CONV-MAL-INV-2 | Validation MUST sebelum processing |
| CONV-MAL-INV-3 | Conversion MUST atomic |
| CONV-MAL-INV-4 | Partial .cd MUST NOT dihasilkan |
| CONV-MAL-INV-5 | Error report MUST lengkap |

---

# 16. Provenance

## 16.1 Provenance Requirements

`[CONV-PROV-1]` Setiap Tile MUST memiliki provenance record.

`[CONV-PROV-2]` Provenance MUST mencakup:

| Field | Required | Description |
|---|---|---|
| `tile_id` | MUST | Tile BLAKE3-256 ID |
| `cell_id` | MUST | Parent Cell ID |
| `source_format` | MUST | Format sumber |
| `source_uri` | MUST | URI atau path sumber |
| `source_tensor_name` | MUST | Nama tensor di sumber |
| `shard_index` | SHOULD | Index shard (jika sharded) |
| `importer_version` | MUST | Versi importer |
| `normalizer_version` | MUST | Versi normalizer |
| `policy_hash` | MUST | Hash conversion policy |
| `converted_at` | MUST | Timestamp konversi |
| `conversion_duration_ms` | SHOULD | Durasi konversi |

## 16.2 Provenance Record

```json
{
  "tile_id": "b3:7f3a8e...",
  "cell_id": "b3:1c33...",
  "source": {
    "format": "safetensors",
    "uri": "file:///checkpoints/model-70b/shard-001.safetensors",
    "tensor_name": "model.layers.0.self_attn.q_proj.weight",
    "shard_index": 0,
    "byte_offset": null
  },
  "conversion": {
    "importer_version": "1.0.0",
    "normalizer_version": "1.0.0",
    "policy_hash": "b3:pp00...",
    "converted_at": "2026-08-11T09:00:00Z",
    "conversion_duration_ms": 45000
  },
  "lineage": {
    "created_revision": "b3:rev00...",
    "parent_tile": null,
    "derived_from": []
  }
}
```

## 16.3 Provenance Storage

`[CONV-PROV-3]` Provenance disimpan di `meta/provenance/<tile-id>.prov.json`.

`[CONV-PROV-4]` Provenance MUST NOT mengubah Tile identity.

`[CONV-PROV-5]` Provenance MAY di-update untuk menambah informasi, tetapi MUST NOT mengubah informasi existing.

## 16.4 Provenance Invariants

| ID | Invariant |
|---|---|
| CONV-PROV-INV-1 | Setiap Tile MUST memiliki provenance |
| CONV-PROV-INV-2 | Provenance MUST mencakup source info |
| CONV-PROV-INV-3 | Provenance MUST mencakup conversion info |
| CONV-PROV-INV-4 | Provenance MUST NOT mengubah Tile identity |
| CONV-PROV-INV-5 | Provenance MUST immutable setelah initial write |

---

# 17. Validation

## 17.1 Pre-Import Validation

`[CONV-VAL-1]` Pre-import validation MUST dilakukan sebelum conversion.

| Check | Requirement |
|---|---|
| Source exists | MUST file/directory ada |
| Source readable | MUST dapat dibaca |
| Format detected | MUST format terdeteksi |
| Header valid | MUST header valid |
| Metadata valid | MUST metadata valid |
| Size valid | MUST size konsisten |

## 17.2 During-Import Validation

`[CONV-VAL-2]` During-import validation:

| Check | Requirement |
|---|---|
| Tensor name mappable | MUST dapat dipetakan ke CellId |
| Dtype valid | MUST dtype dikenal |
| Shape valid | MUST shape valid |
| Data size consistent | MUST size konsisten |
| Hash computable | MUST hash dapat dihitung |

## 17.3 Post-Import Validation

`[CONV-VAL-3]` Post-import validation MUST dilakukan sebelum commit.

| Check | Requirement |
|---|---|
| All Cells present | MUST semua Cell terkonversi |
| All Tiles verified | MUST semua Tile hash verified |
| Manifest valid | MUST manifest valid |
| Dependency graph valid | MUST acyclic |
| No orphan Tiles | MUST tidak ada Tile tanpa Cell |
| Provenance complete | MUST semua Tile punya provenance |

## 17.4 Validation Report

```rust
struct ValidationReport {
    status: ValidationStatus,
    checks_performed: u64,
    checks_passed: u64,
    checks_failed: u64,
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
}

enum ValidationStatus {
    Passed,
    PassedWithWarnings,
    Failed,
}
```

## 17.5 Validation Invariants

| ID | Invariant |
|---|---|
| CONV-VAL-INV-1 | Pre-import validation MUST sebelum conversion |
| CONV-VAL-INV-2 | Post-import validation MUST sebelum commit |
| CONV-VAL-INV-3 | Validation failure MUST membatalkan conversion |
| CONV-VAL-INV-4 | Validation report MUST lengkap |

---

# 18. Import Determinism

## 18.1 Determinism Requirements

`[CONV-DET-1]` Import MUST deterministik.

`[CONV-DET-2]` Source yang sama + policy yang sama MUST menghasilkan `.cd` yang identik.

`[CONV-DET-3]` Determinism mencakup:

| Aspect | Requirement |
|---|---|
| CellId generation | MUST deterministik |
| Tile splitting | MUST deterministik |
| Hash computation | MUST deterministik |
| Manifest serialization | MUST canonical |
| Segment ordering | MUST deterministik |
| Tile ordering | MUST deterministik |

## 18.2 Sources of Non-Determinism

| Source | Mitigation |
|---|---|
| Timestamp | Gunakan fixed timestamp untuk deterministic mode |
| Random number | Tidak ada RNG dalam conversion |
| Parallel execution order | Deterministic scheduling |
| Hash map iteration | Sorted iteration |
| File system order | Explicit ordering |

## 18.3 Deterministic Mode

`[CONV-DET-4]` Deterministic mode MUST default aktif.

`[CONV-DET-5]` Dalam deterministic mode:

1. Timestamps menggunakan fixed value atau logical clock.
2. Parallel operations menggunakan deterministic ordering.
3. Hash map iteration menggunakan sorted keys.
4. File processing menggunakan explicit ordering.

## 18.4 Reproducibility

`[CONV-DET-6]` Reproducibility dapat diverifikasi dengan:

```text
1. Import source A → .cd X
2. Import source A → .cd Y (dengan policy sama)
3. Compare X dan Y
4. MUST identical (byte-level)
```

## 18.5 Determinism Invariants

| ID | Invariant |
|---|---|
| CONV-DET-INV-1 | Import MUST deterministik |
| CONV-DET-INV-2 | Deterministic mode MUST default |
| CONV-DET-INV-3 | Reproducibility MUST verifiable |
| CONV-DET-INV-4 | Non-determinism MUST eksplisit ditandai |

---

# 19. Error Handling

## 19.1 Conversion Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-IMPORT-FORMAT` | Format detection failed |
| `CNWS-E-IMPORT-CORRUPT` | Source file corrupt |
| `CNWS-E-IMPORT-INVALID` | Invalid metadata |
| `CNWS-E-IMPORT-TRUNCATED` | Truncated file |
| `CNWS-E-IMPORT-CHECKSUM` | Checksum mismatch |
| `CNWS-E-IMPORT-UNSAFE` | Unsafe content detected |
| `CNWS-E-IMPORT-UNKNOWN` | Unknown tensor name |
| `CNWS-E-IMPORT-DTYPE` | Unknown or unsupported dtype |
| `CNWS-E-IMPORT-SHAPE` | Invalid shape |
| `CNWS-E-IMPORT-VALIDATION` | Validation failed |
| `CNWS-E-IMPORT-IO` | I/O error |
| `CNWS-E-IMPORT-MEMORY` | Memory budget exceeded |

## 19.2 Error Severity

| Severity | Examples | Action |
|---|---|---|
| Fatal | CORRUPT, UNSAFE, TRUNCATED | Abort conversion, cleanup |
| Recoverable | IO, MEMORY | Retry with backoff |
| Warning | Unknown optional tensor | Log and continue |

---

# 20. Final Conversion Contract

## 20.1 Ringkasan Keputusan Conversion

| ID | Keputusan |
|---|---|
| CONV-F01 | Conversion menggunakan bounded buffer default 2 GiB. |
| CONV-F02 | Canonical dtype: non-quantized → source dtype; quantized → F32 dequantized. |
| CONV-F03 | Tiling default 128 MiB, range 32-256 MiB. |
| CONV-F04 | Tiling 2D memecah pada row boundary. |
| CONV-F05 | PyTorch importer MUST menggunakan restricted unpickler. |
| CONV-F06 | Import MUST deterministik. |
| CONV-F07 | Malformed input MUST ditolak dengan error eksplisit. |
| CONV-F08 | Provenance MUST dicatat untuk setiap Tile. |
| CONV-F09 | Pre-import validation MUST sebelum processing. |
| CONV-F10 | Format detection menggunakan magic bytes + heuristik. |
| CONV-F11 | Custom importer menggunakan trait FormatImporter. |
| CONV-F12 | Conversion bersifat atomic. |
| CONV-F13 | Safetensors importer MUST didukung. |
| CONV-F14 | GGUF importer MUST didukung. |
| CONV-F15 | PyTorch importer MUST didukung. |
| CONV-F16 | GGUF quantized MUST dequantize ke F32. |
| CONV-F17 | Name mapping MUST configurable per architecture. |
| CONV-F18 | Unknown tensor names MUST error. |
| CONV-F19 | Fused tensors MUST di-split jika semantic berbeda. |
| CONV-F20 | Zero Format Coupling MUST terjaga. |

## 20.2 Conversion Invariants

| ID | Invariant |
|---|---|
| CONV-INV-1 | Conversion MUST streaming-first. |
| CONV-INV-2 | Peak memory MUST bounded. |
| CONV-INV-3 | Conversion MUST atomic. |
| CONV-INV-4 | Conversion MUST deterministik. |
| CONV-INV-5 | Output MUST hanya canonical .cd. |
| CONV-INV-6 | Format reader MUST hanya di conversion layer. |
| CONV-INV-7 | Runtime MUST NOT memahami format sumber. |
| CONV-INV-8 | CellId MUST deterministik dan format-independent. |
| CONV-INV-9 | Tile identity MUST BLAKE3-256. |
| CONV-INV-10 | Malformed input MUST ditolak. |
| CONV-INV-11 | Provenance MUST lengkap. |
| CONV-INV-12 | Validation MUST sebelum commit. |
| CONV-INV-13 | Quantized source MUST dequantize. |
| CONV-INV-14 | Unsafe content MUST ditolak. |
| CONV-INV-15 | Zero Format Coupling MUST terjaga. |

## 20.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi conversion final dan mengikat** untuk seluruh external checkpoint → `.cd` CNWS. Ia mendefinisikan bagaimana setiap format sumber diimpor, dinormalisasi, di-tile, di-hash, dan ditulis ke canonical store, dengan menjaga Zero Format Coupling tetap terealisasi.

Seluruh implementasi Conversion Pipeline, Importers, dan Validators CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan conversion yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN CONVERSION & IMPORT SPECIFICATION**
