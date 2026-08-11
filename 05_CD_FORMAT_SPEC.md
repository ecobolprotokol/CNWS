# CNWS
## `.cd` Format & Serialization Specification

| Field | Value |
|---|---|
| Dokumen | CNWS `.cd` Format & Serialization Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (WIRE/STORAGE-LEVEL SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract |
| Hulu ke | Implementasi Storage Engine, Converter, Loader, Verifier |
| Otoritas | Spesifikasi wire/storage tunggal untuk seluruh artefak `.cd` |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract        .cd Format Spec              Implementation
─────────────────────       ─────────────────────        ─────────────
Invariants & principles ──► Byte layouts, offsets,   ──► Writer/Reader code
"MUST be immutable"         magic bytes, alignment,      Serializer/Deserializer
"MUST use BLAKE3-256"       endianness, serialization    Checksum verifier
"MUST be canonical"         rules, version compat        Migration tooling
```

`[CD-DOC-1]` Dokumen ini adalah **wire/storage-level specification**. Ia mendefinisikan setiap byte yang ditulis dan dibaca.

`[CD-DOC-2]` Engineering Contract menetapkan invariant; dokumen ini menetapkan **bagaimana invariant itu direpresentasikan sebagai bytes**.

`[CD-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[CD-DOC-4]` Dua implementasi independen yang mematuhi spesifikasi ini MUST menghasilkan artefak `.cd` yang byte-compatible.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-CD-01 | Seluruh integer binary menggunakan **little-endian**. |
| DF-CD-02 | SUPERBLOCK berukuran tetap **4096 bytes**. |
| DF-CD-03 | Segment header berukuran tetap **4096 bytes**. |
| DF-CD-04 | Tile payload alignment minimum **4 KiB**, preferred **64 KiB**. |
| DF-CD-05 | Magic bytes: `CNWSSB01` (superblock), `CNWSSEG1` (segment), `CNWSIDX1` (index), `CNWSMEM1` (memory), `CNWSRTG1` (routing), `CNWSCMP1` (composition), `CNWSPRV1` (provenance). |
| DF-CD-06 | Hash identity menggunakan **BLAKE3-256** (32 bytes). |
| DF-CD-07 | Representasi hash dalam teks: `b3:` + 64 lowercase hex. |
| DF-CD-08 | MANIFEST.cd menggunakan **JSON UTF-8** dengan canonical serialization. |
| DF-CD-09 | Version format: tiga `u32` (major, minor, patch). |
| DF-CD-10 | Segment target size: **32 GiB**. |
| DF-CD-11 | Tile size default: **128 MiB** (range 32–256 MiB). |
| DF-CD-12 | Canonical payload = uncompressed, densely packed, row-major, little-endian. |

---

# 1. Store Layout

## 1.1 Directory Structure

`[CD-LAYOUT-1]` `.cd` MUST berupa directory dengan struktur berikut.

```text
model.cd/
├── SUPERBLOCK                    # 4096 bytes, fixed
├── LOCK                          # advisory lock file
├── MANIFEST.cd                   # canonical JSON manifest
├── MANIFEST.cd.prev              # previous committed manifest
│
├── journal/
│   └── commit.wal                # write-ahead commit journal
│
├── staging/
│   └── manifest-<hash>.cd        # staged manifests
│
├── index/
│   ├── cells.idx                 # Cell index (binary)
│   ├── tiles.idx                 # Tile index (binary)
│   ├── memory.idx                # Memory entry index (binary)
│   └── routing.idx               # Routing statistics index (binary)
│
├── segments/
│   ├── segment-000001.cd         # Tile payload storage
│   ├── segment-000002.cd
│   └── ...
│
├── lattice/
│   ├── graph.cd                  # Cell Graph structure
│   ├── compositions.cd           # Composition patterns
│   └── routing_policy.cd         # Routing policy
│
├── memory/
│   ├── episodic/                 # Episodic memory segments
│   │   └── segment-000001.mcd
│   ├── semantic/
│   │   └── segment-000001.mcd
│   ├── procedural/
│   │   └── segment-000001.mcd
│   └── index.cd                  # Memory index
│
├── revisions/
│   ├── rev-000000.json           # Revision objects
│   ├── rev-000001.json
│   └── ...
│
├── corrupt/
│   └── <tile-id>.quarantine      # Quarantined corrupt Tiles
│
└── meta/
    ├── provenance/
    │   └── <tile-id>.prov.json   # Provenance records
    └── routing_stats/
        └── stats.json            # Routing statistics
```

`[CD-LAYOUT-2]` Seluruh file dalam `.cd` MUST menggunakan little-endian untuk struktur binary.

`[CD-LAYOUT-3]` File JSON MUST menggunakan UTF-8 tanpa BOM.

`[CD-LAYOUT-4]` Directory `segments/` dan `memory/` MUST hanya berisi file immutable.

## 1.2 Naming Conventions

| Artefak | Pola Nama | Contoh |
|---|---|---|
| Segment Tile | `segment-<6 digit>.cd` | `segment-000001.cd` |
| Segment Memory | `segment-<6 digit>.mcd` | `segment-000001.mcd` |
| Revision | `rev-<6 digit>.json` | `rev-000001.json` |
| Staged manifest | `manifest-<hash>.cd` | `manifest-b3_7f3a...cd` |
| Quarantine | `<tile-id>.quarantine` | `b3_7f3a....quarantine` |
| Provenance | `<tile-id>.prov.json` | `b3_7f3a....prov.json` |

`[CD-NAME-1]` Tile ID dalam filename MUST menggunakan `_` sebagai pengganti `:` (karena `:` tidak valid di beberapa filesystem).

`[CD-NAME-2]` Hash dalam filename MUST lowercase hex.

---

# 2. Primitive Types

## 2.1 Scalar Types

`[CD-PRIM-1]` Seluruh tipe integer menggunakan little-endian.

| Type | Size | Range | Encoding |
|---|---|---|---|
| `u8` | 1 byte | 0–255 | LE |
| `u16` | 2 bytes | 0–65535 | LE |
| `u32` | 4 bytes | 0–4294967295 | LE |
| `u64` | 8 bytes | 0–18446744073709551615 | LE |
| `u128` | 16 bytes | — | LE |
| `f32` | 4 bytes | IEEE 754 | LE |
| `f64` | 8 bytes | IEEE 754 | LE |
| `Blake3Hash` | 32 bytes | — | raw bytes |
| `CellId` | 32 bytes | — | BLAKE3-256 |
| `TileId` | 32 bytes | — | BLAKE3-256 |
| `SegmentId` | 8 bytes | 1–2^64-1 | LE u64 |
| `RevisionId` | 32 bytes | — | BLAKE3-256 |

## 2.2 Compound Types

```rust
// Length-prefixed byte array
struct Bytes {
    len: u64,
    data: [u8; len],
}

// Length-prefixed UTF-8 string
struct String {
    len: u64,          // byte length of UTF-8
    data: [u8; len],   // UTF-8 bytes
}

// Variable-length array
struct Array<T> {
    count: u64,
    items: [T; count],
}

// Optional value
struct Option<T> {
    present: u8,       // 0 = absent, 1 = present
    value: T,          // valid only if present == 1
}

// Key-value pair
struct KeyValue<K, V> {
    key: K,
    value: V,
}
```

`[CD-PRIM-2]` `Option<T>` MUST menggunakan 1 byte tag.

`[CD-PRIM-3]` Padding bytes MUST zero-filled.

## 2.3 Enum Encoding

`[CD-PRIM-4]` Enum di-encode sebagai `u8` discriminant + payload.

```rust
// Contoh: CellType
enum CellType {
    // Weight cells: 0x01–0x1F
    EMBEDDING          = 0x01,
    ATTENTION_Q_PROJ   = 0x02,
    ATTENTION_K_PROJ   = 0x03,
    ATTENTION_V_PROJ   = 0x04,
    ATTENTION_OUT      = 0x05,
    MLP_GATE           = 0x06,
    MLP_UP             = 0x07,
    MLP_DOWN           = 0x08,
    EXPERT_GATE        = 0x09,
    EXPERT_ROUTE       = 0x0A,
    EXPERT_WEIGHT      = 0x0B,
    LAYERNORM_WEIGHT   = 0x0C,
    LAYERNORM_BIAS     = 0x0D,
    LM_HEAD            = 0x0E,
    VISION_ENCODER     = 0x0F,

    // Memory cells: 0x20–0x2F
    MEMORY_EPISODIC    = 0x20,
    MEMORY_SEMANTIC    = 0x21,
    MEMORY_PROCEDURAL  = 0x22,

    // Routing & composition: 0x30–0x3F
    ROUTING_POLICY     = 0x30,
    COMPOSITION_PATTERN= 0x31,

    // Computation: 0x40–0x4F
    TRANSFORM_MODULE   = 0x40,
    ENCODE_MODULE      = 0x41,
    DECODE_MODULE      = 0x42,

    // Custom: 0xFF
    CUSTOM             = 0xFF,
}
```

`[CD-PRIM-5]` Discriminant value yang tidak dikenal MUST menghasilkan error, bukan silent ignore.

## 2.4 DataType Encoding

```rust
enum DataType {
    F32    = 0x01,   // IEEE 754 single
    F16    = 0x02,   // IEEE 754 half
    BF16   = 0x03,   // bfloat16
    F8E4M3 = 0x04,   // FP8 e4m3
    F8E5M2 = 0x05,   // FP8 e5m2
    I8     = 0x06,   // int8
    U8     = 0x07,   // uint8
    I16    = 0x08,   // int16
    I32    = 0x09,   // int32
    I64    = 0x0A,   // int64
    BOOL   = 0x0B,   // boolean
    I4     = 0x0C,   // int4 (packed)
    I2     = 0x0D,   // int2 (packed)
}
```

`[CD-PRIM-6]` Size per element untuk tiap DataType:

| DataType | Bytes/element | Notes |
|---|---|---|
| F32 | 4 | — |
| F16 | 2 | — |
| BF16 | 2 | — |
| F8E4M3 | 1 | — |
| F8E5M2 | 1 | — |
| I8 | 1 | — |
| U8 | 1 | — |
| I16 | 2 | — |
| I32 | 4 | — |
| I64 | 8 | — |
| BOOL | 1 | 0=false, 1=true |
| I4 | 0.5 | 2 elements/byte, packed LE |
| I2 | 0.25 | 4 elements/byte, packed LE |

`[CD-PRIM-7]` Untuk sub-byte types (I4, I2), elements MUST di-pack dari LSB ke MSB dalam setiap byte.

---

# 3. Endianness & Alignment

## 3.1 Endianness

`[CD-END-1]` Seluruh integer, floating-point, dan multi-byte value dalam artefak binary `.cd` MUST little-endian.

`[CD-END-2]` Tidak ada big-endian di mana pun dalam `.cd`.

`[CD-END-3]` Hash digest (BLAKE3-256) disimpan sebagai 32 raw bytes dalam urutan natural (byte 0 pertama).

## 3.2 Alignment Rules

`[CD-ALIGN-1]` Alignment minimum dan preferred per struktur:

| Struktur | Alignment Minimum | Alignment Preferred |
|---|---|---|
| SUPERBLOCK | 4 KiB | 4 KiB |
| Segment header | 4 KiB | 4 KiB |
| Tile metadata | 8 bytes | 64 bytes |
| Tile payload | 4 KiB | 64 KiB |
| Index entries | 8 bytes | 64 bytes |
| Memory entries | 8 bytes | 4 KiB |
| JSON files | N/A (text) | N/A |

`[CD-ALIGN-2]` Padding untuk alignment MUST zero-filled.

`[CD-ALIGN-3]` Offset Tile payload dalam segment MUST aligned ke minimum 4 KiB.

`[CD-ALIGN-4]` Implementasi SHOULD menggunakan 64 KiB alignment untuk Tile payload jika memungkinkan.

## 3.3 Offset Convention

`[CD-OFF-1]` Seluruh offset dalam spesifikasi ini adalah **absolute offset dari awal file**, kecuali dinyatakan lain.

`[CD-OFF-2]` Offset diukur dalam bytes.

`[CD-OFF-3]` Offset 0 adalah byte pertama file.

---

# 4. SUPERBLOCK

## 4.1 Purpose

SUPERBLOCK adalah struktur pertama yang dibaca saat membuka `.cd` store. Ia berisi metadata global dan pointer ke MANIFEST.cd.

`[CD-SB-1]` SUPERBLOCK MUST berada di offset 0 dari file `SUPERBLOCK`.

`[CD-SB-2]` SUPERBLOCK MUST berukuran tepat 4096 bytes.

`[CD-SB-3]` SUPERBLOCK MUST menggunakan alignment 4 KiB.

## 4.2 Layout

```text
Offset  Size  Field                  Type         Notes
──────  ────  ─────────────────────  ───────────  ──────────────────
0x0000  8     magic                  [u8; 8]      "CNWSSB01"
0x0008  4     version_major          u32          Format version major
0x000C  4     version_minor          u32          Format version minor
0x0010  4     version_patch          u32          Format version patch
0x0014  4     flags                  u32          Bit flags
0x0018  32    store_id               Blake3Hash   Unique store identity
0x0038  32    model_id_hash          Blake3Hash   BLAKE3-256 of model_id string
0x0058  8     created_at_ns          u64          Unix nanoseconds UTC
0x0060  8     last_modified_ns       u64          Unix nanoseconds UTC
0x0068  32    manifest_hash          Blake3Hash   BLAKE3-256 of MANIFEST.cd
0x0088  8     manifest_size          u64          Size of MANIFEST.cd in bytes
0x0090  32    head_revision          Blake3Hash   Active RevisionId
0x00B0  8     segment_count          u64          Number of segments
0x00B8  8     tile_count             u64          Total Tiles
0x00C0  8     cell_count             u64          Total Cells
0x00C8  8     memory_entry_count     u64          Total memory entries
0x00D0  8     revision_count         u64          Total revisions
0x00D8  8     total_logical_bytes    u64          Total logical size
0x00E0  8     total_stored_bytes     u64          Total stored size (after compression)
0x00E8  8     routing_version        u64          Routing policy version
0x00F0  8     composition_cache_count u64         Cached compositions
0x00F8  8     reserved_flags         u64          Reserved for future flags
0x0100  3840  reserved               [u8; 3840]   MUST be zero
──────  ────  ─────────────────────  ───────────  ──────────────────
Total: 4096 bytes
```

## 4.3 Field Specifications

### 4.3.1 Magic

`[CD-SB-4]` Magic MUST exactly `0x43 0x4E 0x57 0x53 0x53 0x42 0x30 0x31` ("CNWSSB01" ASCII).

`[CD-SB-5]` Jika magic tidak cocok, reader MUST menolak file sebagai bukan SUPERBLOCK valid.

### 4.3.2 Version

`[CD-SB-6]` `version_major` untuk spesifikasi ini adalah `1`.

`[CD-SB-7]` Reader MUST menolak `version_major` lebih tinggi dari yang didukung.

`[CD-SB-8]` Reader MUST menerima `version_minor` ≤ versi yang didukung.

### 4.3.3 Flags

```rust
// Bit flags (u32)
const FLAG_COMPRESSION_ENABLED: u32 = 1 << 0;   // 0x00000001
const FLAG_MEMORY_ENABLED:      u32 = 1 << 1;   // 0x00000002
const FLAG_ROUTING_ENABLED:     u32 = 1 << 2;   // 0x00000004
const FLAG_LATTICE_ENABLED:     u32 = 1 << 3;   // 0x00000008
const FLAG_SEALED:              u32 = 1 << 31;  // 0x80000000 (read-only)
```

`[CD-SB-9]` Bit yang tidak dikenal MUST diabaikan oleh reader.

`[CD-SB-10]` Writer MUST hanya menggunakan bit yang terdefinisi.

### 4.3.4 Reserved

`[CD-SB-11]` Field `reserved` MUST zero-filled.

`[CD-SB-12]` Reader SHOULD memverifikasi reserved zero dan SHOULD warn jika non-zero.

## 4.4 Superblock Update

`[CD-SB-13]` SUPERBLOCK di-update saat commit manifest baru.

`[CD-SB-14]` Update SUPERBLOCK MUST atomic (write to temp + rename).

`[CD-SB-15]` SUPERBLOCK MUST NOT di-update selama operasi read-only.

---

# 5. MANIFEST.cd

## 5.1 Purpose

MANIFEST.cd adalah root canonical manifest. Ia mendeskripsikan seluruh Cell, Tile, dependency, representation, memory, routing, dan revision.

`[CD-MAN-1]` MANIFEST.cd MUST berupa JSON UTF-8.

`[CD-MAN-2]` MANIFEST.cd adalah source of truth untuk semantic model.

## 5.2 Canonical Serialization Rules

`[CD-MAN-3]` JSON MUST di-canonicalize sebagai berikut:

1. Object keys MUST sorted by Unicode code point.
2. Tidak ada duplicate keys.
3. Numbers MUST finite (no NaN, no Infinity).
4. Strings MUST UTF-8 NFC.
5. Tidak ada trailing commas.
6. Tidak ada comments.
7. Whitespace antar tokens MUST single space atau none (implementation-defined, but consistent).

`[CD-MAN-4]` Hash MANIFEST.cd MUST dihitung atas canonical serialization bytes.

`[CD-MAN-5]` Dua implementasi MUST menghasilkan byte-identical canonical output untuk logical content yang sama.

## 5.3 Manifest Schema

```json
{
  "format_version": "1.0.0",
  "model_id": "example-org/model-70b",
  "content_addressing": {
    "algorithm": "BLAKE3",
    "digest_bits": 256,
    "encoding": "hex",
    "domain_prefix": "b3"
  },
  "architecture": {
    "architecture_type": "dense-transformer",
    "num_layers": 80,
    "hidden_dim": 8192,
    "num_heads": 64,
    "vocab_size": 128256,
    "num_experts": null,
    "experts_per_token": null,
    "special_components": []
  },
  "cells": [
    {
      "id": "b3:7f3a8e...",
      "cell_type": "ATTENTION_Q_PROJ",
      "shape": [4096, 4096],
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
      "dependencies": [],
      "metadata": {
        "layer_index": 0,
        "attention_head": null,
        "expert_index": null,
        "quantization_policy": null
      }
    }
  ],
  "dependency_graph": {
    "edges": {
      "model.layer.0.self_attn.q_proj": [
        "model.embedding.token_embedding"
      ]
    }
  },
  "representations": {
    "b3:1c33...": [
      {
        "representation_id": "bf16",
        "storage_id": "b3:1c33...",
        "dtype": "bf16",
        "size_bytes": 134217728,
        "quantization": null
      },
      {
        "representation_id": "fp8_e4m3",
        "storage_id": "b3:aa99...",
        "dtype": "fp8_e4m3",
        "size_bytes": 67108864,
        "quantization": {
          "scheme": "fp8_e4m3",
          "calibration": "none"
        }
      }
    ]
  },
  "segments": [
    {
      "segment_id": 1,
      "path": "segments/segment-000001.cd",
      "tile_count": 256,
      "size_bytes": 34359738368,
      "index_hash": "b3:90ab..."
    }
  ],
  "memory": {
    "episodic_entries": 1048576,
    "semantic_entries": 4194304,
    "procedural_entries": 262144,
    "working_memory_bound_bytes": 268435456,
    "total_memory_bytes": 1099511627776
  },
  "routing": {
    "index_dimensions": 512,
    "index_structure": "HNSW",
    "routing_policy_version": 42
  },
  "revision": {
    "id": "b3:rev00...",
    "parents": [],
    "revision_number": 0,
    "created_at": "2026-08-11T09:00:00Z",
    "message": "base import"
  },
  "provenance": {
    "source_format": "safetensors",
    "source_uri": "file:///checkpoints/model-70b",
    "importer_version": "1.0.0",
    "policy_hash": "b3:pp00..."
  },
  "runtime_defaults": {
    "prefetch_policy": "DependencyAware",
    "prefetch_depth": 2,
    "cache_eviction": "LRU_BY_PRIORITY",
    "gpu_reserve_bytes": 2147483648,
    "cpu_reserve_bytes": 2147483648
  }
}
```

## 5.4 Manifest Field Requirements

`[CD-MAN-6]` MANIFEST.cd MUST memuat field-field berikut:

| Field | Required | Type |
|---|---|---|
| `format_version` | MUST | string (semver) |
| `model_id` | MUST | string |
| `content_addressing` | MUST | object |
| `architecture` | MUST | object |
| `cells` | MUST | array |
| `dependency_graph` | MUST | object |
| `representations` | MUST | object |
| `segments` | MUST | array |
| `memory` | MUST | object |
| `routing` | MUST | object |
| `revision` | MUST | object |
| `provenance` | MUST | object |
| `runtime_defaults` | SHOULD | object |

`[CD-MAN-7]` Unknown fields SHOULD dipertahankan saat round-trip.

`[CD-MAN-8]` `format_version` MUST sesuai dengan SUPERBLOCK version.

---

# 6. Cell Serialization

## 6.1 Cell in Manifest

Cell dideskripsikan dalam MANIFEST.cd sebagai JSON object.

`[CD-CELL-1]` Setiap Cell entry MUST memuat:

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

## 6.2 Cell Metadata

```json
{
  "layer_index": 0,
  "attention_head": null,
  "expert_index": null,
  "quantization_policy": null,
  "architecture": "llama",
  "attributes": {}
}
```

`[CD-CELL-2]` `metadata` MUST JSON object.

`[CD-CELL-3]` `metadata` MUST NOT mengubah Tile identity.

## 6.3 Cell Index (Binary)

Cell index adalah binary file untuk lookup cepat.

`[CD-CELL-4]` File `index/cells.idx` menggunakan magic `CNWSIDX1`.

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x0000  8     magic                  [u8; 8]     "CNWSIDX1"
0x0008  4     version_major          u32
0x000C  4     version_minor          u32
0x0010  8     entry_count            u64
0x0018  8     index_type             u8          0x01 = CellIndex
0x0019  7     padding                [u8; 7]     zero
0x0020  ...   entries                CellIndexEntry[]
```

### CellIndexEntry

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    32    cell_id                Blake3Hash
0x20    2     cell_type              u16
0x22    2     dtype                  u16
0x24    4     ndim                   u32
0x28    8     shape_offset           u64         offset into shape table
0x30    8     tile_count             u64
0x38    8     tile_ref_offset        u64         offset into tile ref table
0x40    8     metadata_offset        u64         offset into metadata section
0x48    8     flags                  u64
```

`[CD-CELL-5]` CellIndexEntry MUST sorted by `cell_id`.

`[CD-CELL-6]` Lookup menggunakan binary search pada `cell_id`.

---

# 7. Tile Serialization

## 7.1 Tile Metadata

Tile metadata disimpan dalam segment index, bukan inline dengan payload.

```rust
struct TileMeta {
    tile_id: Blake3Hash,        // 32 bytes
    cell_id: Blake3Hash,        // 32 bytes (parent Cell)
    ndim: u32,                  // number of dimensions
    dtype: u16,                 // DataType discriminant
    compression: u16,           // Compression discriminant
    element_offset: u64,        // element offset within Cell
    element_count: u64,         // number of elements
    payload_size: u64,          // canonical uncompressed size in bytes
    stored_size: u64,           // actual stored size (after compression)
    representation_count: u16,  // number of representations
    flags: u16,                 // tile flags
}
```

## 7.2 Tile in Segment

Tile payload disimpan dalam segment. Lihat §8 untuk segment format.

`[CD-TILE-1]` Tile payload MUST disimpan aligned.

`[CD-TILE-2]` Tile payload MUST NOT mengandung header inline.

`[CD-TILE-3]` Tile metadata berada di segment index, bukan di payload.

## 7.3 Tile Index (Binary)

`[CD-TILE-4]` File `index/tiles.idx` menggunakan magic `CNWSIDX1` dengan `index_type = 0x02`.

### TileIndexEntry

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    32    tile_id                Blake3Hash
0x20    32    cell_id                Blake3Hash
0x40    8     segment_id             u64
0x48    8     offset_in_segment      u64
0x50    8     stored_size            u64
0x58    8     payload_size           u64
0x60    2     compression            u16
0x62    2     dtype                  u16
0x64    4     ndim                   u32
0x68    8     shape_offset           u64
0x70    8     element_offset         u64
0x78    8     element_count          u64
0x80    8     flags                  u64
```

`[CD-TILE-5]` TileIndexEntry MUST sorted by `tile_id`.

`[CD-TILE-6]` Total size TileIndexEntry adalah 136 bytes.

## 7.4 Canonical Payload Encoding

`[CD-TILE-7]` Canonical Tile payload MUST:

1. Uncompressed.
2. Densely packed (no padding between elements).
3. Row-major (C-order).
4. Little-endian.
5. Sesuai DataType yang dideklarasikan.

`[CD-TILE-8]` Untuk multi-dimensional tensor, layout adalah row-major:

```text
shape = [d0, d1, d2, ..., dn]
index = i0 * (d1*d2*...*dn) + i1 * (d2*...*dn) + ... + in
offset_bytes = index * bytes_per_element
```

`[CD-TILE-9]` Hash identity dihitung atas canonical payload bytes:

```text
tile_id = BLAKE3-256(canonical_payload_bytes)
```

## 7.5 Compression Encoding

`[CD-TILE-10]` Compression discriminant:

| Value | Codec | Notes |
|---|---|---|
| 0x0000 | None (raw) | Tidak dikompresi |
| 0x0001 | Zstd level 1 | — |
| 0x0002 | Zstd level 3 | Default |
| 0x0003 | Zstd level 5 | — |
| 0x0004 | Zstd level 9 | — |
| 0x0005 | Zstd level 19 | Maximum |
| 0x0010 | LZ4 | — |
| 0x0020 | Snappy | — |

`[CD-TILE-11]` Compression MUST NOT mengubah Tile identity.

`[CD-TILE-12]` Hash dihitung sebelum compression.

`[CD-TILE-13]` `stored_size` adalah ukuran setelah compression.

`[CD-TILE-14]` `payload_size` adalah ukuran sebelum compression (canonical).

## 7.6 Multiple Representations

`[CD-TILE-15]` Setiap representation memiliki Tile ID sendiri.

`[CD-TILE-16]` Representation Tile disimpan seperti Tile biasa.

`[CD-TILE-17]` Manifest menghubungkan canonical Tile dengan representations:

```json
{
  "b3:canonical_tile_id": [
    {
      "representation_id": "bf16",
      "storage_id": "b3:canonical_tile_id",
      "dtype": "bf16",
      "size_bytes": 134217728
    },
    {
      "representation_id": "fp8_e4m3",
      "storage_id": "b3:fp8_tile_id",
      "dtype": "fp8_e4m3",
      "size_bytes": 67108864
    }
  ]
}
```

---

# 8. Segment Format

## 8.1 Purpose

Segment adalah container fisik untuk Tile payloads. Satu segment berisi banyak Tiles.

`[CD-SEG-1]` Segment target size SHOULD 32 GiB.

`[CD-SEG-2]` Segment MUST immutable setelah committed.

`[CD-SEG-3]` Segment filename MUST mengikuti pola `segment-<6 digit>.cd`.

## 8.2 Segment Layout

```text
┌──────────────────────────────────────────────────────────┐
│ Segment Header (4096 bytes)                              │
│   offset 0x0000                                          │
├──────────────────────────────────────────────────────────┤
│ Tile Payload Region                                      │
│   offset 0x1000                                          │
│   ┌────────────────────────────────────────────────────┐ │
│   │ Tile A payload (aligned)                           │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ padding (zero)                                     │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ Tile B payload (aligned)                           │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ padding (zero)                                     │ │
│   ├────────────────────────────────────────────────────┤ │
│   │ ...                                                │ │
│   └────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────┤
│ Segment Index                                            │
│   offset = index_offset (dari header)                    │
│   ┌────────────────────────────────────────────────────┐ │
│   │ SegmentIndexHeader                                 │ │
│   │ SegmentIndexEntry[0]                               │ │
│   │ SegmentIndexEntry[1]                               │ │
│   │ ...                                                │ │
│   │ ShapeTable                                         │ │
│   └────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────┤
│ Segment Trailer (64 bytes)                               │
└──────────────────────────────────────────────────────────┘
```

## 8.3 Segment Header

`[CD-SEG-4]` Segment header MUST berukuran tepat 4096 bytes.

```text
Offset  Size  Field                  Type         Notes
──────  ────  ─────────────────────  ───────────  ──────────────────
0x0000  8     magic                  [u8; 8]      "CNWSSEG1"
0x0008  4     version_major          u32
0x000C  4     version_minor          u32
0x0010  8     segment_id             u64          Sequential, 1-based
0x0018  8     created_at_ns          u64          Unix nanoseconds UTC
0x0020  8     tile_count             u64          Number of Tiles
0x0028  8     payload_region_offset  u64          Absolute offset, MUST >= 0x1000
0x0030  8     payload_region_size    u64          Total payload region bytes
0x0038  8     index_offset           u64          Absolute offset of segment index
0x0040  8     index_size             u64          Size of segment index
0x0048  32    index_hash             Blake3Hash   BLAKE3-256 of segment index
0x0068  8     compression_flags      u64          Bitmask of codecs used
0x0070  8     flags                  u64          Segment flags
0x0078  8     reserved               u64          MUST be zero
0x0080  3968  padding                [u8; 3968]   MUST be zero
──────  ────  ─────────────────────  ───────────  ──────────────────
Total: 4096 bytes
```

`[CD-SEG-5]` Magic MUST exactly `0x43 0x4E 0x57 0x53 0x53 0x45 0x47 0x31` ("CNWSSEG1").

`[CD-SEG-6]` `payload_region_offset` MUST >= 0x1000 (4 KiB).

`[CD-SEG-7]` `index_offset` MUST > `payload_region_offset + payload_region_size`.

`[CD-SEG-8]` `index_hash` MUST BLAKE3-256 dari serialized segment index.

## 8.4 Segment Index

### 8.4.1 Segment Index Header

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSSIDX"
0x08    4     version_major          u32
0x0C    4     version_minor          u32
0x10    8     entry_count            u64
0x18    8     shape_table_offset     u64         relative to index start
0x20    8     shape_table_size       u64
0x28    40    reserved               [u8; 40]
```

### 8.4.2 SegmentIndexEntry

```text
Offset  Size  Field                  Type         Notes
──────  ────  ─────────────────────  ───────────  ──────────────────
0x00    32    tile_id                Blake3Hash
0x20    32    cell_id                Blake3Hash
0x40    8     offset_in_segment      u64          Absolute offset
0x48    8     stored_size            u64          After compression
0x50    8     payload_size           u64          Before compression
0x58    2     compression            u16          Codec discriminant
0x5A    2     dtype                  u16          DataType discriminant
0x5C    4     ndim                   u32
0x60    8     shape_offset           u64          Offset into shape table
0x68    8     element_offset         u64
0x70    8     element_count          u64
0x78    8     flags                  u64
──────  ────  ─────────────────────  ───────────  ──────────────────
Total: 128 bytes per entry
```

`[CD-SEG-9]` SegmentIndexEntry MUST sorted by `tile_id`.

`[CD-SEG-10]` SegmentIndexEntry size MUST 128 bytes.

### 8.4.3 Shape Table

Shape table menyimpan shape arrays untuk semua Tiles dalam segment.

```text
ShapeTable:
  For each unique shape:
    ndim: u32
    dims: [u64; ndim]
```

`[CD-SEG-11]` Shape table MUST menggunakan deduplication untuk shape yang sama.

`[CD-SEG-12]` `shape_offset` dalam SegmentIndexEntry menunjuk ke entri shape table.

## 8.5 Segment Trailer

`[CD-SEG-13]` Segment trailer MUST berukuran 64 bytes.

```text
Offset  Size  Field                  Type         Notes
──────  ────  ─────────────────────  ───────────  ──────────────────
0x00    8     magic                  [u8; 8]      "CNWSSEGT"
0x08    8     index_entry_count      u64
0x10    32    index_hash             Blake3Hash   MUST match header
0x30    16    reserved               [u8; 16]     MUST be zero
──────  ────  ─────────────────────  ───────────  ──────────────────
Total: 64 bytes
```

`[CD-SEG-14]` Trailer `index_hash` MUST sama dengan header `index_hash`.

`[CD-SEG-15]` Trailer magic MUST exactly "CNWSSEGT".

## 8.6 Tile Payload dalam Segment

`[CD-SEG-16]` Setiap Tile payload ditempatkan sebagai berikut:

```text
[padding to alignment]
[compressed or raw payload bytes]
[padding to next alignment]
```

`[CD-SEG-17]` Alignment untuk Tile payload MUST minimum 4 KiB.

`[CD-SEG-18]` Padding MUST zero-filled.

`[CD-SEG-19]` `offset_in_segment` menunjuk ke awal payload (setelah padding).

`[CD-SEG-20]` `stored_size` adalah ukuran payload saja, tidak termasuk padding.

## 8.7 Segment Reading

`[CD-SEG-21]` Untuk membaca Tile:

```text
1. Lookup TileId di Tile Registry / Segment Index
2. Dapatkan segment_id, offset_in_segment, stored_size
3. Open segment file
4. Seek ke offset_in_segment
5. Read stored_size bytes
6. Jika compression != None: decompress
7. Verify BLAKE3-256
8. Return canonical payload
```

`[CD-SEG-22]` Reader MUST NOT melakukan scan segment.

`[CD-SEG-23]` Reader MUST menggunakan index untuk lookup O(1).

---

# 9. Index Format

## 9.1 General Index Structure

`[CD-IDX-1]` Seluruh index files menggunakan header umum:

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSIDX1"
0x08    4     version_major          u32
0x0C    4     version_minor          u32
0x10    8     entry_count            u64
0x18    1     index_type             u8
0x19    7     padding                [u8; 7]
0x20    ...   entries                (type-specific)
```

`[CD-IDX-2]` `index_type` values:

| Value | Index Type |
|---|---|
| 0x01 | Cell Index |
| 0x02 | Tile Index |
| 0x03 | Memory Index |
| 0x04 | Routing Index |

`[CD-IDX-3]` Entries MUST sorted ascending by primary key.

`[CD-IDX-4]` Lookup MUST menggunakan binary search.

## 9.2 Index Regeneration

`[CD-IDX-5]` Index files bersifat regenerable.

`[CD-IDX-6]` Index BUKAN source of truth; MANIFEST.cd adalah source of truth.

`[CD-IDX-7]` Jika index corrupt atau hilang, sistem MUST dapat meregenerasi dari MANIFEST.cd dan segments.

---

# 10. Memory Format

## 10.1 Memory Store Layout

Memory entries disimpan dalam segment khusus dengan ekstensi `.mcd`.

`[CD-MEM-1]` Memory segment menggunakan magic `CNWSMEM1`.

`[CD-MEM-2]` Memory segment filename: `segment-<6 digit>.mcd`.

## 10.2 Memory Segment Header

```text
Offset  Size  Field                  Type         Notes
──────  ────  ─────────────────────  ───────────  ──────────────────
0x0000  8     magic                  [u8; 8]      "CNWSMEM1"
0x0008  4     version_major          u32
0x000C  4     version_minor          u32
0x0010  8     segment_id             u64
0x0018  8     created_at_ns          u64
0x0020  8     entry_count            u64
0x0028  8     memory_type            u8           0x20=episodic, 0x21=semantic, 0x22=procedural; other MemoryType values are Cell-level only
0x0029  7     padding                [u8; 7]
0x0030  8     payload_region_offset  u64
0x0038  8     payload_region_size    u64
0x0040  8     index_offset           u64
0x0048  8     index_size             u64
0x0050  32    index_hash             Blake3Hash
0x0070  8     flags                  u64
0x0078  3976  padding                [u8; 3976]
──────  ────  ─────────────────────  ───────────  ──────────────────
Total: 4096 bytes
```

## 10.3 Memory Entry

```rust
struct MemoryEntry {
    memory_id: Blake3Hash,       // 32 bytes, BLAKE3-256 of key+value
    memory_type: u8,             // episodic/semantic/procedural
    consolidation_level: u8,     // 0=raw, 1=consolidated, 2=compiled
    key_dim: u64,                // dimension of key vector
    value_size: u64,             // size of value payload in bytes
    association_count: u64,      // number of associations
    created_at_ns: u64,          // Unix nanoseconds
    // Followed by:
    // key_vector: [f32; key_dim]
    // value_payload: [u8; value_size]
    // associations: [Blake3Hash; association_count]
}
```

`[CD-MEM-3]` Memory entry identity:

```text
memory_id = BLAKE3-256(key_vector_bytes || value_payload_bytes)
```

`[CD-MEM-4]` Memory entries MUST immutable setelah ditulis.

`[CD-MEM-4a]` Access statistics MUST disimpan dan dimutasi pada `MemoryIndex`, bukan pada immutable Memory entry.

`[CD-MEM-5]` Memory entries MUST independently loadable.

## 10.4 Memory Index

`[CD-MEM-6]` Memory index menggunakan magic `CNWSIDX1` dengan `index_type = 0x03`.

### MemoryIndexEntry

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    32    memory_id              Blake3Hash
0x20    1     memory_type            u8
0x21    1     consolidation_level    u8
0x22    6     padding                [u8; 6]
0x28    8     segment_id             u64
0x30    8     offset_in_segment      u64
0x38    8     stored_size            u64
0x40    8     key_dim                u64
0x48    8     value_size             u64
0x50    8     access_count           u64
0x58    8     last_access_ns         u64
0x60    8     flags                  u64
```

`[CD-MEM-7]` MemoryIndexEntry size MUST 104 bytes.

`[CD-MEM-8]` MemoryIndexEntry MUST sorted by `memory_id`.

---

# 11. Routing Format

## 11.1 Routing Policy

Routing policy disimpan dalam `lattice/routing_policy.cd`.

`[CD-ROUTE-1]` Routing policy menggunakan magic `CNWSRTG1`.

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSRTG1"
0x08    4     version_major          u32
0x0C    4     version_minor          u32
0x10    8     policy_version         u64
0x18    8     created_at_ns          u64
0x20    4     index_dimensions       u32
0x24    4     index_structure        u32          0x01=HNSW, 0x02=IVF
0x28    8     cell_count             u64
0x30    8     edge_count             u64
0x38    8     parameters_size        u64
0x40    8     statistics_size        u64
0x48    32    parameters_hash        Blake3Hash
0x68    32    statistics_hash        Blake3Hash
0x88    8     flags                  u64
0x90    3952  padding                [u8; 3952]
```

## 11.2 Routing Statistics

Routing statistics disimpan dalam `meta/routing_stats/stats.json` sebagai JSON.

```json
{
  "policy_version": 42,
  "updated_at": "2026-08-11T12:00:00Z",
  "cell_statistics": {
    "b3:7f3a...": {
      "usage_count": 1048576,
      "success_rate": 0.94,
      "avg_latency_us": 12,
      "last_used": "2026-08-11T11:59:00Z"
    }
  },
  "edge_statistics": {
    "b3:7f3a...->b3:8b4c...": {
      "traversal_count": 524288,
      "success_rate": 0.91
    }
  }
}
```

`[CD-ROUTE-2]` Routing statistics MAY di-update tanpa membuat revision baru.

`[CD-ROUTE-3]` Routing policy change MUST membuat revision baru.

## 11.3 Routing Index

`[CD-ROUTE-4]` Routing index menggunakan magic `CNWSIDX1` dengan `index_type = 0x04`.

### RoutingIndexEntry

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    32    cell_id                Blake3Hash
0x20    8     usage_count            u64
0x28    4     success_rate_x1000     u32          success rate * 1000
0x2C    4     avg_latency_us         u32
0x30    8     last_used_ns           u64
0x38    8     flags                  u64
```

`[CD-ROUTE-5]` RoutingIndexEntry size MUST 64 bytes.

---

# 12. Composition Format

## 12.1 Composition Patterns

Composition patterns disimpan dalam `lattice/compositions.cd`.

`[CD-COMP-1]` Composition patterns menggunakan magic `CNWSCMP1`.

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSCMP1"
0x08    4     version_major          u32
0x0C    4     version_minor          u32
0x10    8     pattern_count          u64
0x18    8     created_at_ns          u64
0x20    8     patterns_offset        u64
0x28    8     patterns_size          u64
0x30    32    patterns_hash          Blake3Hash
0x50    8     flags                  u64
0x58    3992  padding                [u8; 3992]
```

## 12.2 Composition Pattern Entry

```rust
struct CompositionPattern {
    pattern_id: Blake3Hash,      // BLAKE3-256 of composition spec
    cell_count: u64,             // number of Cells in pattern
    execution_count: u64,        // how many times used
    avg_execution_us: u64,       // average execution time
    cell_ids: Vec<Blake3Hash>,   // ordered Cell IDs
    composition_type: u8,        // sequential/parallel/conditional/iterative
    flags: u8,
}
```

`[CD-COMP-2]` Composition pattern identity:

```text
pattern_id = BLAKE3-256(serialize(cell_ids) || composition_type)
```

`[CD-COMP-3]` Composition patterns MAY di-cache tanpa membuat revision.

---

# 13. Provenance Format

## 13.1 Provenance Records

Provenance disimpan sebagai JSON files dalam `meta/provenance/`.

`[CD-PROV-1]` Provenance file MUST JSON UTF-8.

`[CD-PROV-2]` Provenance filename: `<tile-id>.prov.json`.

## 13.2 Provenance Schema

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
  },
  "verification": {
    "blake3_verified": true,
    "verified_at": "2026-08-11T09:01:00Z",
    "verification_count": 1
  }
}
```

`[CD-PROV-3]` Provenance SHOULD mencakup source format, URI, dan tensor name.

`[CD-PROV-4]` Provenance MUST mencakup conversion metadata.

`[CD-PROV-5]` Provenance MUST mencakup lineage information.

`[CD-PROV-6]` Provenance MUST NOT mengubah Tile identity.

---

# 14. Revision Format

## 14.1 Revision Files

Revision disimpan sebagai JSON files dalam `revisions/`.

`[CD-REV-1]` Revision file MUST JSON UTF-8.

`[CD-REV-2]` Revision filename: `rev-<6 digit>.json`.

## 14.2 Revision Schema

```json
{
  "id": "b3:rev01...",
  "model_id": "example-org/model-70b",
  "revision_number": 1,
  "parents": ["b3:rev00..."],
  "root_manifest": "b3:manifest_hash...",
  "changed_cells": [
    "b3:cell1...",
    "b3:cell2..."
  ],
  "changed_tiles": {
    "added": [
      "b3:new_tile1...",
      "b3:new_tile2..."
    ],
    "removed": [],
    "replaced": [
      {
        "old": "b3:old_tile...",
        "new": "b3:new_tile1..."
      }
    ]
  },
  "changed_memory": [],
  "changed_routing": [],
  "metadata": {
    "created_at": "2026-08-11T10:00:00Z",
    "author": "researcher@example.com",
    "message": "fine-tune on coding dataset",
    "specialization": "coding"
  }
}
```

`[CD-REV-3]` Revision file MUST immutable setelah ditulis.

`[CD-REV-4]` `removed` dalam `changed_tiles` MUST NOT berarti physical deletion.

`[CD-REV-5]` Physical deletion hanya boleh dilakukan oleh GC.

---

# 15. Canonicalization

## 15.1 Canonical Payload

`[CD-CANON-1]` Canonical Tile payload MUST:

1. Uncompressed.
2. Densely packed.
3. Row-major (C-order).
4. Little-endian.
5. Tanpa padding antar elemen.

`[CD-CANON-2]` Canonical payload adalah input untuk BLAKE3-256.

## 15.2 Canonical JSON

`[CD-CANON-3]` Canonical JSON MUST mengikuti aturan di §5.2.

`[CD-CANON-4]` Canonical JSON digunakan untuk hashing MANIFEST.cd dan revision files.

## 15.3 Canonical Cell Serialization

Untuk hashing Cell identity:

`[CD-CANON-5]` Cell identity dihitung dari canonical Cell payload:

```text
cell_payload = canonical_cell_payload_as_defined_by_CELL-SER-3
cell_id = BLAKE3-256(cell_payload)
```

`[CD-CANON-6]` Canonical Cell serialization MUST deterministik.

`[CD-CANON-7]` Source yang sama dengan policy yang sama MUST menghasilkan canonical Cell payload dan Cell ID yang sama.

## 15.4 Canonical Manifest Hash

`[CD-CANON-8]` MANIFEST.cd hash dihitung atas canonical serialization:

```text
manifest_hash = BLAKE3-256(canonical_json_bytes(MANIFEST.cd))
```

`[CD-CANON-9]` `manifest_hash` disimpan di SUPERBLOCK.

---

# 16. Checksum / Hash Placement

## 16.1 Hash Placement Summary

| Struktur | Hash | Lokasi Hash |
|---|---|---|
| Tile payload | BLAKE3-256 | Tile ID (content-addressed) |
| Segment index | BLAKE3-256 | Segment header + trailer |
| MANIFEST.cd | BLAKE3-256 | SUPERBLOCK |
| Cell payload | BLAKE3-256 | Cell ID (content-addressed) |
| Memory entry | BLAKE3-256 | Memory ID (content-addressed) |
| Composition pattern | BLAKE3-256 | Pattern ID |
| Routing policy | BLAKE3-256 | Routing header |
| Revision file | BLAKE3-256 | Revision ID |

## 16.2 Hash Verification Points

`[CD-HASH-1]` Tile MUST diverifikasi saat:
1. Load dari segment.
2. Masuk cache.
3. Sebelum eksekusi.

`[CD-HASH-2]` Segment index MUST diverifikasi saat:
1. Segment open.
2. Index load.

`[CD-HASH-3]` MANIFEST.cd MUST diverifikasi saat:
1. Store open.
2. Revision switch.

`[CD-HASH-4]` Hash mismatch MUST menghasilkan error `CNWS-E-CORRUPT`.

## 16.3 No SHA-256

`[CD-HASH-5]` SHA-256 MUST NOT digunakan sebagai identity primitive.

`[CD-HASH-6]` BLAKE3-256 adalah satu-satunya hash primitive.

---

# 17. Version Compatibility

## 17.1 Version Scheme

`[CD-VER-1]` Version menggunakan tiga komponen: `major.minor.patch`.

`[CD-VER-2]` Setiap struktur binary menyimpan `version_major` dan `version_minor`.

`[CD-VER-3]` `version_patch` disimpan hanya di MANIFEST.cd.

## 17.2 Compatibility Rules

`[CD-VER-4]` Reader MUST menolak `version_major` lebih tinggi dari yang didukung.

`[CD-VER-5]` Reader MUST menerima `version_minor` ≤ versi yang didukung.

`[CD-VER-6]` Writer MUST NOT menulis feature minor lebih tinggi tanpa menaikkan `version_minor`.

`[CD-VER-7]` Breaking changes MUST menaikkan `version_major`.

`[CD-VER-8]` Backward-compatible changes MUST menaikkan `version_minor`.

`[CD-VER-9]` Patch changes MUST menaikkan `version_patch`.

## 17.3 Migration

`[CD-VER-10]` Migration antar major version MUST eksplisit dan teruji.

`[CD-VER-11]` Migration tool MUST tersedia untuk setiap major version transition.

`[CD-VER-12]` Migration MUST mempertahankan content-addressed identities.

## 17.4 Reserved Fields

`[CD-VER-13]` Reserved fields MUST zero-filled oleh writer.

`[CD-VER-14]` Reader MUST mengabaikan reserved fields yang tidak dikenal.

`[CD-VER-15]` Unknown fields dalam JSON SHOULD dipertahankan saat round-trip.

---

# 18. Recovery Metadata

## 18.1 Write-Ahead Journal

`[CD-REC-1]` Journal disimpan di `journal/commit.wal`.

`[CD-REC-2]` Journal MUST di-append sebelum commit.

`[CD-REC-3]` Journal MUST di-fsync setelah append.

## 18.2 Journal Record Format

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSWAL1"
0x08    8     record_id              u64
0x10    8     timestamp_ns           u64
0x18    1     record_type            u8
0x19    7     padding                [u8; 7]
0x20    32    manifest_hash          Blake3Hash
0x40    8     manifest_size          u64
0x48    8     staging_offset         u64
0x50    32    prev_manifest_hash     Blake3Hash
0x70    8     flags                  u64
0x78    ...   payload                (variable)
```

### Record Types

| Value | Type | Description |
|---|---|---|
| 0x01 | BEGIN_COMMIT | Mulai commit |
| 0x02 | STAGE_MANIFEST | Manifest ditulis ke staging |
| 0x03 | RENAME_MANIFEST | Manifest di-rename ke MANIFEST.cd |
| 0x04 | UPDATE_SUPERBLOCK | SUPERBLOCK di-update |
| 0x05 | COMMIT_COMPLETE | Commit selesai |
| 0x06 | ROLLBACK | Rollback dilakukan |

## 18.3 Recovery State Machine

```text
Startup
   │
   ▼
Read journal
   │
   ├── no incomplete commit → normal
   │
   └── incomplete commit
          │
          ├── MANIFEST.cd hash matches commit record
          │       → complete superblock update
          │
          └── MANIFEST.cd hash does not match
                  → restore from MANIFEST.cd.prev
                    or replay staged manifest
```

`[CD-REC-4]` Recovery MUST idempotent.

`[CD-REC-5]` Recovery MUST NOT menghasilkan partial manifest.

`[CD-REC-6]` Recovery MUST memverifikasi manifest hash.

## 18.4 Staging Area

`[CD-REC-7]` Staging manifest disimpan di `staging/manifest-<hash>.cd`.

`[CD-REC-8]` Staging manifest MUST di-fsync sebelum journal append.

`[CD-REC-9]` Staging manifest di-rename ke MANIFEST.cd saat commit.

## 18.5 MANIFEST.cd.prev

`[CD-REC-10]` MANIFEST.cd.prev menyimpan manifest sebelumnya.

`[CD-REC-11]` MANIFEST.cd.prev digunakan untuk rollback saat recovery.

`[CD-REC-12]` MANIFEST.cd.prev MUST di-update setelah commit sukses.

---

# 19. Error Handling

## 19.1 Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-CORRUPT` | BLAKE3 mismatch / payload corrupt |
| `CNWS-E-MAGIC` | Magic bytes tidak cocok |
| `CNWS-E-VERSION` | Version incompatible |
| `CNWS-E-ALIGNMENT` | Alignment violation |
| `CNWS-E-TRUNCATED` | File truncated / incomplete |
| `CNWS-E-MANIFEST` | Manifest invalid |
| `CNWS-E-INDEX` | Index corrupt / inconsistent |
| `CNWS-E-SEGMENT` | Segment corrupt / inconsistent |
| `CNWS-E-RECOVERY` | Recovery failed |
| `CNWS-E-IO` | I/O error |

## 19.2 Error Severity

`[CD-ERR-1]` Error diklasifikasikan:

| Severity | Examples | Action |
|---|---|---|
| Fatal | MAGIC, VERSION, CORRUPT | Stop, require intervention |
| Recoverable | RECOVERY, INDEX | Attempt recovery |
| Transient | IO | Retry with backoff |

`[CD-ERR-2]` Fatal error MUST menghentikan operasi.

`[CD-ERR-3]` Recoverable error MUST mencoba recovery sebelum gagal.

---

# 20. Final `.cd` Format Contract

## 20.1 Ringkasan Keputusan Format

| ID | Keputusan |
|---|---|
| CD-F01 | Seluruh binary little-endian. |
| CD-F02 | SUPERBLOCK 4096 bytes, magic "CNWSSB01". |
| CD-F03 | Segment header 4096 bytes, magic "CNWSSEG1". |
| CD-F04 | Segment trailer 64 bytes, magic "CNWSSEGT". |
| CD-F05 | Segment index magic "CNWSSIDX". |
| CD-F06 | Tile payload alignment minimum 4 KiB, preferred 64 KiB. |
| CD-F07 | Tile metadata di segment index, bukan inline. |
| CD-F08 | SegmentIndexEntry 128 bytes. |
| CD-F09 | TileIndexEntry 136 bytes. |
| CD-F10 | CellIndexEntry sorted by cell_id. |
| CD-F11 | Memory segment magic "CNWSMEM1". |
| CD-F12 | Routing policy magic "CNWSRTG1". |
| CD-F13 | Composition magic "CNWSCMP1". |
| CD-F14 | Index magic "CNWSIDX1". |
| CD-F15 | Journal magic "CNWSWAL1". |
| CD-F16 | MANIFEST.cd canonical JSON UTF-8. |
| CD-F17 | Hash BLAKE3-256 untuk semua content addressing. |
| CD-F18 | Canonical payload: uncompressed, dense, row-major, LE. |
| CD-F19 | Version: major.minor.patch. |
| CD-F20 | Segment target 32 GiB. |

## 20.2 Format Invariants

| ID | Invariant |
|---|---|
| CD-INV-1 | Seluruh binary little-endian. |
| CD-INV-2 | Magic bytes harus exact match. |
| CD-INV-3 | Alignment harus dipatuhi. |
| CD-INV-4 | Padding harus zero-filled. |
| CD-INV-5 | Hash harus BLAKE3-256. |
| CD-INV-6 | Canonical payload harus uncompressed, dense, row-major, LE. |
| CD-INV-7 | Tile identity harus independen dari compression. |
| CD-INV-8 | Tile identity harus independen dari lokasi storage. |
| CD-INV-9 | Segment harus immutable setelah committed. |
| CD-INV-10 | MANIFEST.cd harus canonical JSON. |
| CD-INV-11 | Version compatibility harus dipatuhi. |
| CD-INV-12 | Recovery harus idempotent. |
| CD-INV-13 | Index harus regenerable. |
| CD-INV-14 | MANIFEST.cd adalah source of truth, bukan index. |
| CD-INV-15 | Dua implementasi harus byte-compatible. |

## 20.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi wire/storage final dan mengikat** untuk seluruh artefak `.cd` CNWS. Ia mendefinisikan setiap byte yang ditulis dan dibaca, dari SUPERBLOCK hingga segment trailer, dari canonical payload hingga recovery journal.

Seluruh implementasi Storage Engine, Converter, Loader, dan Verifier CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan format yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN .cd FORMAT & SERIALIZATION SPECIFICATION**
