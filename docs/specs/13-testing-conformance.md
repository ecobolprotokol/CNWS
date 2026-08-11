# CNWS
## Testing & Conformance Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Testing & Conformance Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (CONFORMANCE SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | Seluruh spesifikasi CNWS |
| Hulu ke | Implementasi test suite, CI/CD, certification |
| Otoritas | Spesifikasi tunggal untuk conformance testing CNWS |
| Jaminan | Dua implementasi conformant MUST menghasilkan behavior/data kompatibel |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
All CNWS Specs              Testing & Conformance Spec       Implementation
──────────────────          ────────────────────────────     ─────────────
Engineering Contract    ──► Normative conformance suite  ──► Test suite
.cd Format Spec             Test vectors                     CI/CD pipeline
Cell & Schema Spec          Golden files                     Certification
Runtime Spec                Pass/fail criteria               Interop tests
Conversion Spec             Interoperability tests
Memory Spec
Revision Spec
Security Spec
Reliability Spec
```

`[TEST-DOC-1]` Dokumen ini mendefinisikan **conformance testing** untuk CNWS, bukan sekadar test plan.

`[TEST-DOC-2]` Tujuan utama: **jika implementasi A dan implementasi B sama-sama conformant, keduanya MUST menghasilkan behavior/data yang kompatibel pada contract-defined cases.**

`[TEST-DOC-3]` Jika terjadi konflik dengan spesifikasi lain untuk hal behavior yang diuji, spesifikasi tersebut menang. Untuk hal testing methodology, dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-TEST-01 | Conformance suite bersifat normatif dan wajib. |
| DF-TEST-02 | Test vectors menggunakan format JSON + binary fixtures. |
| DF-TEST-03 | Golden files disimpan dalam repository terpisah `cnws-conformance`. |
| DF-TEST-04 | Hash test vectors menggunakan payload deterministik. |
| DF-TEST-05 | Crash tests menggunakan fault injection. |
| DF-TEST-06 | Corruption tests menggunakan bit-flip injection. |
| DF-TEST-07 | Fuzz tests menggunakan coverage-guided fuzzing. |
| DF-TEST-08 | Interoperability tests menggunakan cross-implementation verification. |
| DF-TEST-09 | Pass criteria: 100% mandatory tests lulus. |
| DF-TEST-10 | Conformance certificate dikeluarkan oleh CNWS governing body. |
| DF-TEST-11 | Test suite version mengikuti spesifikasi version. |
| DF-TEST-12 | Regression tests MUST dijalankan pada setiap commit. |

---

# 1. Executive Summary

## 1.1 Conformance Philosophy

`[TEST-EXEC-1]` Conformance testing CNWS menjamin:

1. **Correctness**: implementasi berperilaku sesuai spesifikasi.
2. **Interoperability**: dua implementasi conformant menghasilkan data kompatibel.
3. **Determinism**: output deterministik untuk input yang sama.
4. **Safety**: failure modes ditangani dengan benar.
5. **Performance**: performance targets terpenuhi.

## 1.2 The Interoperability Guarantee

`[TEST-EXEC-2]` **Interoperability Guarantee**:

> Jika implementasi A dan implementasi B sama-sama conformant terhadap CNWS specification version X.Y, maka:
>
> 1. `.cd` store yang dihasilkan A MUST dapat dibaca oleh B.
> 2. `.cd` store yang dihasilkan B MUST dapat dibaca oleh A.
> 3. Cell/Tile yang dihasilkan A MUST memiliki identity yang sama jika dihasilkan B dari input yang sama.
> 4. Manifest yang dihasilkan A MUST byte-identical dengan yang dihasilkan B untuk logical content yang sama.
> 5. Revision yang dihasilkan A MUST dapat di-resolve oleh B.
> 6. Behavior runtime A MUST kompatibel dengan B untuk input yang sama.

## 1.3 Conformance Levels

| Level | Nama | Requirement |
|---|---|---|
| Level 0 | Not Conformant | Tidak memenuhi mandatory tests |
| Level 1 | Core Conformant | Lulus seluruh CS-01 s/d CS-05 |
| Level 2 | Full Conformant | Lulus seluruh CS-01 s/d CS-10 |
| Level 3 | Certified Conformant | Level 2 + interoperability tests + performance targets |

---

# 2. Conformance Model

## 2.1 Conformance Suite Structure

```text
CNWS Conformance Suite
│
├── CS-01: Content Addressing
│   ├── Hash vectors
│   ├── Identity stability
│   └── Compression independence
│
├── CS-02: Canonical Serialization
│   ├── JSON canonicalization
│   ├── Binary serialization
│   └── Determinism
│
├── CS-03: .cd Format
│   ├── Superblock
│   ├── Segment format
│   ├── Index format
│   └── Golden file comparison
│
├── CS-04: Conversion
│   ├── Safetensors import
│   ├── GGUF import
│   ├── PyTorch import
│   ├── Normalization
│   └── Tiling determinism
│
├── CS-05: Runtime
│   ├── Cell resolution
│   ├── Tile selection
│   ├── Cache behavior
│   └── Budget enforcement
│
├── CS-06: Revision
│   ├── Revision creation
│   ├── Branching
│   ├── Merging
│   ├── Rollback
│   └── GC
│
├── CS-07: Memory
│   ├── Memory write
│   ├── Retrieval
│   ├── Consolidation
│   └── Forgetting
│
├── CS-08: Integrity
│   ├── Corruption detection
│   ├── Quarantine
│   └── Manifest verification
│
├── CS-09: Recovery
│   ├── Crash recovery
│   ├── WAL replay
│   └── Repair
│
└── CS-10: Security
    ├── Malicious checkpoint
    ├── Path traversal
    ├── Resource exhaustion
    └── Restricted unpickling
```

## 2.2 Test Categories

| Category | Mandatory | Description |
|---|---|---|
| Functional tests | MUST | Verify correct behavior |
| Determinism tests | MUST | Verify deterministic output |
| Interoperability tests | MUST | Verify cross-implementation compatibility |
| Crash tests | MUST | Verify crash recovery |
| Corruption tests | MUST | Verify corruption detection |
| Security tests | MUST | Verify security controls |
| Performance tests | SHOULD | Verify performance targets |
| Fuzz tests | SHOULD | Verify robustness |

## 2.3 Test Identification

`[TEST-ID-1]` Setiap test MUST memiliki unique ID:

```text
Format: CNWS-<SUITE>-<NUMBER>

Contoh:
  CNWS-CS01-0001  Content addressing test 1
  CNWS-CS03-0042  .cd format test 42
  CNWS-INT-0001   Interoperability test 1
```

---

# 3. Test Infrastructure

## 3.1 Test Harness

`[TEST-INF-1]` Test harness MUST menyediakan:

```rust
trait ConformanceHarness {
    // Setup
    fn create_temp_store(&self) -> Result<StoreHandle, CnwsError>;
    fn load_golden_file(&self, name: &str) -> Result<Vec<u8>, CnwsError>;
    fn load_fixture(&self, name: &str) -> Result<Fixture, CnwsError>;
    
    // Execution
    fn run_test(&self, test_id: &str) -> TestResult;
    fn run_suite(&self, suite: SuiteId) -> SuiteResult;
    
    // Verification
    fn verify_hash(&self, data: &[u8], expected: &Blake3Hash) -> bool;
    fn verify_bytes(&self, actual: &[u8], expected: &[u8]) -> bool;
    fn verify_json(&self, actual: &Value, expected: &Value) -> bool;
    
    // Reporting
    fn report(&self) -> ConformanceReport;
}
```

## 3.2 Test Vector Format

`[TEST-INF-2]` Test vectors MUST menggunakan format JSON:

```json
{
  "test_id": "CNWS-CS01-0001",
  "suite": "CS-01",
  "name": "BLAKE3-256 empty payload",
  "description": "Verify BLAKE3-256 hash of empty payload",
  "input": {
    "payload": "",
    "payload_encoding": "hex",
    "payload_size": 0
  },
  "expected": {
    "hash": "af1349b9f5f9a1a6a0404dee36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    "hash_encoding": "hex",
    "hash_prefix": "b3:"
  },
  "metadata": {
    "spec_reference": "BLAKE3-256 Content Addressing",
    "mandatory": true
  }
}
```

## 3.3 Fixture Format

`[TEST-INF-3]` Fixtures MUST disimpan dalam format berikut:

```text
fixtures/
├── cells/
│   ├── cell_attention_q.json
│   ├── cell_embedding.json
│   ├── cell_moe_expert.json
│   └── ...
├── tiles/
│   ├── tile_128mib_zeros.bin
│   ├── tile_128mib_random.bin
│   ├── tile_64mib_pattern.bin
│   └── ...
├── manifests/
│   ├── manifest_minimal.json
│   ├── manifest_full.json
│   └── ...
└── checkpoints/
    ├── safetensors_tiny.safetensors
    ├── gguf_tiny.gguf
    ├── pytorch_tiny.pt
    └── ...
```

## 3.4 Golden File Format

`[TEST-INF-4]` Golden files MUST disimpan dengan metadata:

```json
{
  "golden_file": "golden/model_minimal.cd",
  "version": "1.0.0",
  "created_at": "2026-08-11T00:00:00Z",
  "hash": "b3:...",
  "size_bytes": 4096,
  "description": "Minimal .cd store with one Cell",
  "components": {
    "SUPERBLOCK": {
      "offset": 0,
      "size": 4096,
      "hash": "b3:..."
    }
  }
}
```

---

# 4. Canonical Test Vectors

## 4.1 Hash Verification Vectors

`[TEST-HASH-1]` Hash test vectors MUST mencakup:

### 4.1.1 Empty Payload

```json
{
  "test_id": "CNWS-CS01-0001",
  "name": "Empty payload",
  "input": {
    "payload_hex": "",
    "payload_size": 0
  },
  "expected": {
    "blake3_256": "af1349b9f5f9a1a6a0404dee36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
  }
}
```

### 4.1.2 Single Byte

```json
{
  "test_id": "CNWS-CS01-0002",
  "name": "Single zero byte",
  "input": {
    "payload_hex": "00",
    "payload_size": 1
  },
  "expected": {
    "blake3_256": "2d3adedff11b61f14c885e585f3b789a46a78c15e37695e21e2e7f89e8f6c1c2"
  }
}
```

### 4.1.3 All Zeros (1 KiB)

```json
{
  "test_id": "CNWS-CS01-0003",
  "name": "1 KiB of zeros",
  "input": {
    "payload_pattern": "zeros",
    "payload_size": 1024
  },
  "expected": {
    "blake3_256": "<computed>"
  }
}
```

### 4.1.4 All 0xFF (1 KiB)

```json
{
  "test_id": "CNWS-CS01-0004",
  "name": "1 KiB of 0xFF",
  "input": {
    "payload_pattern": "0xff",
    "payload_size": 1024
  },
  "expected": {
    "blake3_256": "<computed>"
  }
}
```

### 4.1.5 Incremental Pattern

```json
{
  "test_id": "CNWS-CS01-0005",
  "name": "Incremental byte pattern",
  "input": {
    "payload_pattern": "incremental",
    "payload_size": 4096,
    "description": "Bytes 0x00, 0x01, 0x02, ..., 0xFF, 0x00, ..."
  },
  "expected": {
    "blake3_256": "<computed>"
  }
}
```

### 4.1.6 Streaming Hash Equivalence

```json
{
  "test_id": "CNWS-CS01-0006",
  "name": "Streaming hash equals batch hash",
  "input": {
    "payload_size": 1048576,
    "chunk_sizes": [1024, 4096, 65536, 1048576]
  },
  "expected": {
    "all_chunks_same_hash": true
  }
}
```

`[TEST-HASH-2]` Streaming hash dengan chunk size berbeda MUST menghasilkan hash yang sama.

## 4.2 Canonical Cell Fixtures

### 4.2.1 Minimal Cell

```json
{
  "fixture_id": "cell_minimal",
  "description": "Minimal valid Cell",
  "cell": {
    "id": "<computed from canonical payload>",
    "cell_type": "LAYERNORM_WEIGHT",
    "version": "1.0.0",
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
    "tiles": [
      {
        "tile_id": "<computed>",
        "shape": [4096],
        "offset": [0],
        "size": [4096],
        "segment_id": 1
      }
    ],
    "dependencies": [],
    "metadata": {
      "created_at_ns": 0,
      "modified_at_ns": 0,
      "layer_index": 0,
      "trainable": true
    }
  },
  "canonical_json": "<sorted keys, no whitespace>",
  "canonical_json_hash": "<computed>"
}
```

### 4.2.2 Attention Q Projection Cell

```json
{
  "fixture_id": "cell_attention_q",
  "description": "Attention Q projection Cell",
  "cell": {
    "id": "<computed>",
    "cell_type": "ATTENTION_Q_PROJ",
    "version": "1.0.0",
    "input_schema": {
      "kind": "Tensor",
      "tensor": {
        "shape": [4096, 4096],
        "dtype": "bf16",
        "layout": "RowMajor"
      }
    },
    "output_schema": {
      "kind": "Tensor",
      "tensor": {
        "shape": [4096, 4096],
        "dtype": "bf16",
        "layout": "RowMajor"
      }
    },
    "tiles": [
      {
        "tile_id": "<computed>",
        "shape": [4096, 4096],
        "offset": [0, 0],
        "size": [4096, 4096],
        "segment_id": 1
      }
    ],
    "dependencies": [
      {
        "target": "<embedding_cell_id>",
        "dep_type": "DATA"
      }
    ],
    "metadata": {
      "created_at_ns": 0,
      "modified_at_ns": 0,
      "layer_index": 0,
      "attention_head": null,
      "expert_index": null,
      "architecture": "test-model",
      "trainable": true
    }
  }
}
```

### 4.2.3 MoE Expert Cell

```json
{
  "fixture_id": "cell_moe_expert",
  "description": "MoE expert Cell",
  "cell": {
    "id": "<computed>",
    "cell_type": "EXPERT_WEIGHT",
    "version": "1.0.0",
    "input_schema": {
      "kind": "Tensor",
      "tensor": {
        "shape": [14336, 4096],
        "dtype": "bf16",
        "layout": "RowMajor"
      }
    },
    "output_schema": {
      "kind": "Tensor",
      "tensor": {
        "shape": [14336, 4096],
        "dtype": "bf16",
        "layout": "RowMajor"
      }
    },
    "tiles": [
      {
        "tile_id": "<computed>",
        "shape": [7168, 4096],
        "offset": [0, 0],
        "size": [7168, 4096],
        "segment_id": 1
      },
      {
        "tile_id": "<computed>",
        "shape": [7168, 4096],
        "offset": [7168, 0],
        "size": [7168, 4096],
        "segment_id": 1
      }
    ],
    "dependencies": [
      {
        "target": "<router_cell_id>",
        "dep_type": "CONTROL"
      }
    ],
    "metadata": {
      "created_at_ns": 0,
      "modified_at_ns": 0,
      "layer_index": 10,
      "expert_index": 7,
      "trainable": true
    }
  }
}
```

## 4.3 Canonical Tile Fixtures

### 4.3.1 Small Tile (All Zeros)

```json
{
  "fixture_id": "tile_small_zeros",
  "description": "Small tile with all zeros",
  "tile": {
    "payload_pattern": "zeros",
    "payload_size": 8192,
    "dtype": "bf16",
    "shape": [4096],
    "compression": "None"
  },
  "expected": {
    "tile_id": "<computed BLAKE3-256>",
    "canonical_size": 8192,
    "stored_size": 8192
  }
}
```

### 4.3.2 Tile with Compression

```json
{
  "fixture_id": "tile_compressed",
  "description": "Tile with zstd compression",
  "tile": {
    "payload_pattern": "incremental",
    "payload_size": 134217728,
    "dtype": "bf16",
    "shape": [4096, 4096],
    "compression": "Zstd3"
  },
  "expected": {
    "tile_id": "<computed from UNCOMPRESSED payload>",
    "canonical_size": 134217728,
    "stored_size": "<compressed size>",
    "identity_independent_of_compression": true
  }
}
```

### 4.3.3 Tile Identity Stability

```json
{
  "fixture_id": "tile_identity_stability",
  "description": "Same payload, different compression, same identity",
  "variants": [
    {"compression": "None", "expected_same_id": true},
    {"compression": "Zstd1", "expected_same_id": true},
    {"compression": "Zstd3", "expected_same_id": true},
    {"compression": "Zstd9", "expected_same_id": true},
    {"compression": "Lz4", "expected_same_id": true}
  ]
}
```

## 4.4 .cd Golden Files

### 4.4.1 Minimal .cd Store

```json
{
  "golden_file_id": "golden_minimal_cd",
  "description": "Minimal valid .cd store",
  "contents": {
    "SUPERBLOCK": {
      "size": 4096,
      "magic": "CNWSSB01",
      "version_major": 1,
      "version_minor": 0,
      "version_patch": 0,
      "cell_count": 1,
      "tile_count": 1,
      "segment_count": 1
    },
    "MANIFEST.cd": {
      "format_version": "1.0.0",
      "model_id": "test-model",
      "cells": ["<cell_minimal>"],
      "segments": [1]
    },
    "segments/segment-000001.cd": {
      "magic": "CNWSSEG1",
      "tile_count": 1,
      "tiles": ["<tile_small_zeros>"]
    }
  },
  "expected_store_hash": "<computed>"
}
```

### 4.4.2 Multi-Cell .cd Store

```json
{
  "golden_file_id": "golden_multi_cell_cd",
  "description": "Multi-cell .cd store",
  "contents": {
    "SUPERBLOCK": {
      "cell_count": 3,
      "tile_count": 5,
      "segment_count": 1
    },
    "MANIFEST.cd": {
      "cells": [
        "<cell_embedding>",
        "<cell_attention_q>",
        "<cell_layernorm>"
      ]
    }
  }
}
```

---

# 5. Normative Conformance Suite

## 5.1 CS-01: Content Addressing

`[CS01-1]` Content addressing tests MUST lulus untuk conformance.

| Test ID | Name | Description | Mandatory |
|---|---|---|---|
| CNWS-CS01-0001 | Empty payload hash | BLAKE3-256 of empty payload | MUST |
| CNWS-CS01-0002 | Single byte hash | BLAKE3-256 of single byte | MUST |
| CNWS-CS01-0003 | 1 KiB zeros hash | BLAKE3-256 of 1 KiB zeros | MUST |
| CNWS-CS01-0004 | 1 KiB 0xFF hash | BLAKE3-256 of 1 KiB 0xFF | MUST |
| CNWS-CS01-0005 | Incremental pattern | BLAKE3-256 of incremental bytes | MUST |
| CNWS-CS01-0006 | Streaming equivalence | Streaming hash = batch hash | MUST |
| CNWS-CS01-0007 | Identity stability | Same content = same ID | MUST |
| CNWS-CS01-0008 | Compression independence | ID independent of compression | MUST |
| CNWS-CS01-0009 | Location independence | ID independent of location | MUST |
| CNWS-CS01-0010 | Collision detection | Same ID + different payload = error | MUST |

### CS01-0007: Identity Stability

```pseudo
test_identity_stability():
    payload = create_payload(pattern="incremental", size=1024)
    
    // Hash multiple times
    hash1 = blake3_256(payload)
    hash2 = blake3_256(payload)
    hash3 = blake3_256(payload)
    
    assert hash1 == hash2 == hash3
    
    // Hash from different sources
    hash_from_file = blake3_256(read_file(payload_file))
    hash_from_memory = blake3_256(payload)
    
    assert hash_from_file == hash_from_memory
```

### CS01-0008: Compression Independence

```pseudo
test_compression_independence():
    payload = create_payload(pattern="incremental", size=1048576)
    
    // Hash uncompressed
    canonical_hash = blake3_256(payload)
    
    // Compress with different codecs
    compressed_zstd3 = compress(payload, Zstd3)
    compressed_zstd9 = compress(payload, Zstd9)
    compressed_lz4 = compress(payload, Lz4)
    
    // Decompress and hash
    assert blake3_256(decompress(compressed_zstd3)) == canonical_hash
    assert blake3_256(decompress(compressed_zstd9)) == canonical_hash
    assert blake3_256(decompress(compressed_lz4)) == canonical_hash
```

## 5.2 CS-02: Canonical Serialization

`[CS02-1]` Canonical serialization tests MUST lulus.

| Test ID | Name | Description | Mandatory |
|---|---|---|---|
| CNWS-CS02-0001 | JSON key sorting | Keys sorted by code point | MUST |
| CNWS-CS02-0002 | JSON no duplicates | No duplicate keys | MUST |
| CNWS-CS02-0003 | JSON finite numbers | No NaN/Infinity | MUST |
| CNWS-CS02-0004 | JSON UTF-8 NFC | Strings NFC normalized | MUST |
| CNWS-CS02-0005 | Binary LE | Integers little-endian | MUST |
| CNWS-CS02-0006 | Binary alignment | Alignment rules followed | MUST |
| CNWS-CS02-0007 | Serialization determinism | Same input = same bytes | MUST |
| CNWS-CS02-0008 | Round-trip | Serialize → deserialize = original | MUST |

### CS02-0001: JSON Key Sorting

```pseudo
test_json_key_sorting():
    // Input with unsorted keys
    input = '{"zebra": 1, "apple": 2, "mango": 3}'
    
    // Canonicalize
    canonical = canonicalize_json(input)
    
    // Expected: keys sorted
    expected = '{"apple":2,"mango":3,"zebra":1}'
    
    assert canonical == expected
```

### CS02-0007: Serialization Determinism

```pseudo
test_serialization_determinism():
    manifest = create_test_manifest()
    
    // Serialize multiple times
    bytes1 = serialize_canonical(manifest)
    bytes2 = serialize_canonical(manifest)
    bytes3 = serialize_canonical(manifest)
    
    assert bytes1 == bytes2 == bytes3
    
    // Hash must be identical
    assert blake3_256(bytes1) == blake3_256(bytes2) == blake3_256(bytes3)
```

## 5.3 CS-03: .cd Format

`[CS03-1]` .cd format tests MUST lulus.

| Test ID | Name | Description | Mandatory |
|---|---|---|---|
| CNWS-CS03-0001 | Superblock magic | Magic = "CNWSSB01" | MUST |
| CNWS-CS03-0002 | Superblock size | Size = 4096 bytes | MUST |
| CNWS-CS03-0003 | Segment magic | Magic = "CNWSSEG1" | MUST |
| CNWS-CS03-0004 | Segment header size | Header = 4096 bytes | MUST |
| CNWS-CS03-0005 | Segment trailer | Trailer = 64 bytes | MUST |
| CNWS-CS03-0006 | Tile alignment | Tile payload aligned | MUST |
| CNWS-CS03-0007 | Index sorted | Index entries sorted | MUST |
| CNWS-CS03-0008 | Index hash | Index hash verified | MUST |
| CNWS-CS03-0009 | Golden file match | Byte-identical to golden | MUST |
| CNWS-CS03-0010 | Manifest hash | Manifest hash matches SUPERBLOCK | MUST |

### CS03-0009: Golden File Match

```pseudo
test_golden_file_match():
    // Generate .cd store from fixture
    store = create_store_from_fixture("cell_minimal")
    
    // Load golden file
    golden = load_golden_file("golden_minimal_cd")
    
    // Compare byte-by-byte
    for component in ["SUPERBLOCK", "MANIFEST.cd", "segments/segment-000001.cd"]:
        actual_bytes = read_component(store, component)
        golden_bytes = read_component(golden, component)
        
        assert actual_bytes == golden_bytes, 
            f"Mismatch in {component}"
```

## 5.4 CS-04: Conversion

`[CS04-1]` Conversion tests MUST lulus.

| Test ID | Name | Description | Mandatory |
|---|---|---|---|
| CNWS-CS04-0001 | Safetensors import | Import tiny Safetensors | MUST |
| CNWS-CS04-0002 | GGUF import | Import tiny GGUF | MUST |
| CNWS-CS04-0003 | PyTorch import | Import tiny PyTorch | MUST |
| CNWS-CS04-0004 | Conversion determinism | Same input = same output | MUST |
| CNWS-CS04-0005 | Normalization | Tensor names mapped correctly | MUST |
| CNWS-CS04-0006 | Tiling determinism | Same tiling for same input | MUST |
| CNWS-CS04-0007 | Dtype handling | Canonical dtype correct | MUST |
| CNWS-CS04-0008 | Bounded memory | Peak RAM bounded | MUST |
| CNWS-CS04-0009 | Atomic conversion | No partial .cd | MUST |
| CNWS-CS04-0010 | Provenance recorded | Provenance complete | MUST |

### CS04-0004: Conversion Determinism

```pseudo
test_conversion_determinism():
    source = "fixtures/checkpoints/safetensors_tiny.safetensors"
    
    // Convert twice
    convert(source, "output1.cd")
    convert(source, "output2.cd")
    
    // Compare stores
    assert stores_identical("output1.cd", "output2.cd")
    
    // Compare all Tile IDs
    tiles1 = list_tiles("output1.cd")
    tiles2 = list_tiles("output2.cd")
    assert tiles1 == tiles2
```

## 5.5 CS-05: Runtime

`[CS05-1]` Runtime tests MUST lulus.

| Test ID | Name | Description | Mandatory |
|---|---|---|---|
| CNWS-CS05-0001 | Cell resolution O(1) | Resolution is O(1) | MUST |
| CNWS-CS05-0002 | Tile selection | Correct Tiles selected | MUST |
| CNWS-CS05-0003 | Cache hit | Cached Tile returned | MUST |
| CNWS-CS05-0004 | Cache miss | Tile loaded on miss | MUST |
| CNWS-CS05-0005 | Budget enforcement | Budget hard-enforced | MUST |
| CNWS-CS05-0006 | Integrity verification | BLAKE3 verified before use | MUST |
| CNWS-CS05-0007 | Representation selection | Correct representation selected | MUST |
| CNWS-CS05-0008 | Prefetch | Prefetch works correctly | MUST |
| CNWS-CS05-0009 | Eviction | Eviction respects priority | MUST |
| CNWS-CS05-0010 | Zero format coupling | No format knowledge in runtime | MUST |

## 5.6 CS-06: Revision

| Test ID | Name | Mandatory |
|---|---|---|
| CNWS-CS06-0001 | Revision creation | MUST |
| CNWS-CS06-0002 | Revision immutability | MUST |
| CNWS-CS06-0003 | Tile-level delta | MUST |
| CNWS-CS06-0004 | Branching no copy | MUST |
| CNWS-CS06-0005 | Three-way merge | MUST |
| CNWS-CS06-0006 | Merge conflict detection | MUST |
| CNWS-CS06-0007 | Rollback | MUST |
| CNWS-CS06-0008 | GC reachability | MUST |
| CNWS-CS06-0009 | GC safety | MUST |
| CNWS-CS06-0010 | Resolution cache | MUST |

## 5.7 CS-07: Memory

| Test ID | Name | Mandatory |
|---|---|---|
| CNWS-CS07-0001 | Memory write | MUST |
| CNWS-CS07-0002 | Memory retrieval O(log N) | MUST |
| CNWS-CS07-0003 | Working memory bounded | MUST |
| CNWS-CS07-0004 | Consolidation | MUST |
| CNWS-CS07-0005 | Forgetting policy | MUST |
| CNWS-CS07-0006 | Association traversal | MUST |
| CNWS-CS07-0007 | Context not linear | MUST |
| CNWS-CS07-0008 | Memory deduplication | MUST |

## 5.8 CS-08: Integrity

| Test ID | Name | Mandatory |
|---|---|---|
| CNWS-CS08-0001 | Tile corruption detection | MUST |
| CNWS-CS08-0002 | Segment corruption detection | MUST |
| CNWS-CS08-0003 | Manifest tampering detection | MUST |
| CNWS-CS08-0004 | Quarantine | MUST |
| CNWS-CS08-0005 | Recovery from replica | MUST |
| CNWS-CS08-0006 | Integrity report | MUST |

## 5.9 CS-09: Recovery

| Test ID | Name | Mandatory |
|---|---|---|
| CNWS-CS09-0001 | WAL recovery | MUST |
| CNWS-CS09-0002 | Crash during commit | MUST |
| CNWS-CS09-0003 | Crash during conversion | MUST |
| CNWS-CS09-0004 | Manifest recovery | MUST |
| CNWS-CS09-0005 | Segment recovery | MUST |
| CNWS-CS09-0006 | Recovery idempotency | MUST |
| CNWS-CS09-0007 | No committed data loss | MUST |

## 5.10 CS-10: Security

| Test ID | Name | Mandatory |
|---|---|---|
| CNWS-CS10-0001 | Malicious checkpoint rejection | MUST |
| CNWS-CS10-0002 | PyTorch restricted unpickling | MUST |
| CNWS-CS10-0003 | Path traversal prevention | MUST |
| CNWS-CS10-0004 | Resource exhaustion prevention | MUST |
| CNWS-CS10-0005 | Decompression bomb prevention | MUST |
| CNWS-CS10-0006 | Manifest tampering rejection | MUST |
| CNWS-CS10-0007 | Version downgrade rejection | MUST |
| CNWS-CS10-0008 | Security logging | MUST |

---

# 6. Crash Tests

## 6.1 Crash Test Methodology

`[TEST-CRASH-1]` Crash tests MUST menggunakan fault injection.

```pseudo
function crash_test(test_name, crash_point, recovery_check):
    // Start operation
    operation = start_operation(test_name)
    
    // Inject crash at specific point
    inject_crash(crash_point)
    
    // Restart
    restart()
    
    // Run recovery
    result = recover()
    
    // Verify recovery
    recovery_check(result)
    
    // Verify store consistency
    assert store_is_consistent()
    
    // Verify no committed data loss
    assert no_committed_data_loss()
```

## 6.2 Crash Test Cases

| Test ID | Crash Point | Expected Recovery |
|---|---|---|
| CNWS-CRASH-0001 | Before staging write | No change |
| CNWS-CRASH-0002 | After staging write | Cleanup staging |
| CNWS-CRASH-0003 | After journal append | Replay or rollback |
| CNWS-CRASH-0004 | After manifest rename | Complete SUPERBLOCK |
| CNWS-CRASH-0005 | After SUPERBLOCK update | Mark complete |
| CNWS-CRASH-0006 | During Tile write | Cleanup partial Tile |
| CNWS-CRASH-0007 | During conversion | Cleanup or resume |
| CNWS-CRASH-0008 | During GC | Restart GC |
| CNWS-CRASH-0009 | During merge | Rollback merge |
| CNWS-CRASH-0010 | During memory write | Cleanup partial entry |

## 6.3 Crash Test Invariants

| ID | Invariant |
|---|---|
| TEST-CRASH-INV-1 | Store MUST konsisten setelah crash |
| TEST-CRASH-INV-2 | Committed data MUST NOT hilang |
| TEST-CRASH-INV-3 | Recovery MUST idempotent |
| TEST-CRASH-INV-4 | Recovery MUST selesai dalam timeout |
| TEST-CRASH-INV-5 | Crash MUST NOT menghasilkan partial commit |

---

# 7. Corruption Tests

## 7.1 Corruption Test Methodology

`[TEST-CORR-1]` Corruption tests MUST menggunakan bit-flip injection.

```pseudo
function corruption_test(file_path, corruption_type, offset, expected_detection):
    // Read original file
    original = read_file(file_path)
    original_hash = blake3_256(original)
    
    // Inject corruption
    corrupted = inject_corruption(original, corruption_type, offset)
    write_file(file_path, corrupted)
    
    // Attempt to load
    result = load_and_verify(file_path)
    
    // Verify detection
    assert result.status == expected_detection
    
    // Verify quarantine
    if expected_detection == DETECTED:
        assert is_quarantined(file_path)
```

## 7.2 Corruption Test Cases

| Test ID | Target | Corruption Type | Expected |
|---|---|---|---|
| CNWS-CORR-0001 | Tile payload | Single bit flip | Detected, quarantined |
| CNWS-CORR-0002 | Tile payload | Multi-byte corruption | Detected, quarantined |
| CNWS-CORR-0003 | Tile payload | Truncation | Detected, quarantined |
| CNWS-CORR-0004 | Segment header | Magic corruption | Detected, rejected |
| CNWS-CORR-0005 | Segment index | Hash mismatch | Detected, rejected |
| CNWS-CORR-0006 | MANIFEST.cd | Byte modification | Detected, rejected |
| CNWS-CORR-0007 | SUPERBLOCK | Hash mismatch | Detected, rejected |
| CNWS-CORR-0008 | Memory entry | Payload corruption | Detected, quarantined |
| CNWS-CORR-0009 | WAL record | Record corruption | Detected, skipped |
| CNWS-CORR-0010 | Index file | Entry corruption | Detected, rebuilt |

## 7.3 Corruption Test Invariants

| ID | Invariant |
|---|---|
| TEST-CORR-INV-1 | Corruption MUST terdeteksi |
| TEST-CORR-INV-2 | Corrupted data MUST dikarantina |
| TEST-CORR-INV-3 | Corruption MUST NOT silent |
| TEST-CORR-INV-4 | Recovery MUST dicoba |
| TEST-CORR-INV-5 | Unrecoverable corruption MUST dilaporkan |

---

# 8. Revision Tests

## 8.1 Revision Test Cases

| Test ID | Name | Description |
|---|---|---|
| CNWS-REV-0001 | Create revision | Create and verify |
| CNWS-REV-0002 | Immutability | Committed revision unchanged |
| CNWS-REV-0003 | Tile delta | Only changed Tiles stored |
| CNWS-REV-0004 | Branch | Branch without copy |
| CNWS-REV-0005 | Merge | Three-way merge |
| CNWS-REV-0006 | Conflict | Conflict detection |
| CNWS-REV-0007 | Rollback | Rollback without delete |
| CNWS-REV-0008 | GC reachability | GC respects reachability |
| CNWS-REV-0009 | Resolution | Effective graph correct |
| CNWS-REV-0010 | Multi-parent | DAG with multiple parents |

## 8.2 Revision Test Example

```pseudo
test_tile_delta():
    // Create base revision
    base = create_revision(base_cells)
    
    // Modify one Cell
    modified_cell = refine_cell(base.cells[0])
    
    // Create new revision
    rev1 = create_revision([modified_cell])
    
    // Verify delta
    assert rev1.changed_tiles.len() == 1  // Only changed Tile
    assert rev1.unchanged_refs.len() == base.tiles.len() - 1
    
    // Verify shared Tiles
    for tile_id in base.tiles:
        if tile_id != modified_cell.tile_id:
            assert rev1.references(tile_id)  // Shared, not copied
    
    // Verify storage efficiency
    additional_storage = rev1.storage_size - base.storage_size
    assert additional_storage == modified_cell.tile_size  // Only new Tile
```

---

# 9. Runtime Tests

## 9.1 Runtime Test Cases

| Test ID | Name | Description |
|---|---|---|
| CNWS-RT-0001 | Cell resolution | Resolve by name |
| CNWS-RT-0002 | Tile selection | Select correct Tiles |
| CNWS-RT-0003 | Cache hierarchy | GPU → CPU → NVMe |
| CNWS-RT-0004 | Budget enforcement | Hard budget limit |
| CNWS-RT-0005 | Adaptive depth | Depth varies by input |
| CNWS-RT-0006 | Adaptive compute | Compute varies by difficulty |
| CNWS-RT-0007 | MoE selective | Only selected experts |
| CNWS-RT-0008 | Prefetch | Prefetch by dependency |
| CNWS-RT-0009 | Eviction | Priority-based eviction |
| CNWS-RT-0010 | Determinism | Same input = same output |

## 9.2 Adaptive Compute Test

```pseudo
test_adaptive_compute():
    // Easy input
    easy_input = create_easy_input()
    easy_result = execute(easy_input, budget)
    
    // Hard input
    hard_input = create_hard_input()
    hard_result = execute(hard_input, budget)
    
    // Verify adaptive behavior
    assert easy_result.steps_taken < hard_result.steps_taken
    assert easy_result.compute_used.flops < hard_result.compute_used.flops
    
    // Verify budget respected
    assert easy_result.compute_used.flops <= budget.max_flops
    assert hard_result.compute_used.flops <= budget.max_flops
```

## 9.3 MoE Selective Loading Test

```pseudo
test_moe_selective_loading():
    // Setup: 64 experts, top-K = 2
    model = create_moe_model(num_experts=64, top_k=2)
    
    // Execute
    input = create_input()
    result = execute(input, budget)
    
    // Verify only 2 experts loaded
    loaded_experts = result.loaded_experts
    assert loaded_experts.len() == 2
    
    // Verify all 64 experts NOT loaded
    assert not all_experts_loaded()
    
    // Verify deduplication in batch
    batch = create_batch_with_duplicate_experts()
    batch_result = execute(batch, budget)
    assert batch_result.unique_experts_loaded <= 2
```

---

# 10. Memory Tests

## 10.1 Memory Test Cases

| Test ID | Name | Description |
|---|---|---|
| CNWS-MEM-0001 | Episodic write | Write episodic memory |
| CNWS-MEM-0002 | Semantic write | Write semantic memory |
| CNWS-MEM-0003 | Retrieval | Retrieve by content |
| CNWS-MEM-0004 | Retrieval O(log N) | Complexity verified |
| CNWS-MEM-0005 | Working memory bound | Bound enforced |
| CNWS-MEM-0006 | Consolidation | Consolidate frequent entries |
| CNWS-MEM-0007 | Forgetting | Forget low importance |
| CNWS-MEM-0008 | Association | Associate and traverse |
| CNWS-MEM-0009 | Context O(1) | Context not linear |
| CNWS-MEM-0010 | Deduplication | Same content = same entry |

## 10.2 Retrieval Complexity Test

```pseudo
test_retrieval_complexity():
    // Create memories at different scales
    for n in [1000, 10000, 100000, 1000000]:
        create_n_memories(n)
        
        // Measure retrieval time
        query = create_query()
        start = now()
        results = retrieve(query, k=16)
        elapsed = now() - start
        
        // Verify O(log N) complexity
        // Time should not grow linearly with N
        assert elapsed < max_expected_time(n)
        
        // Log for analysis
        log(f"N={n}, time={elapsed}")
```

---

# 11. Importer Fuzz Tests

## 11.1 Fuzz Test Methodology

`[TEST-FUZZ-1]` Fuzz tests MUST menggunakan coverage-guided fuzzing.

`[TEST-FUZZ-2]` Fuzz tests SHOULD dijalankan minimum 24 jam.

## 11.2 Fuzz Targets

| Fuzz Target | Input | Expected |
|---|---|---|
| Safetensors parser | Random bytes | No crash, no hang |
| GGUF parser | Random bytes | No crash, no hang |
| PyTorch unpickler | Random pickle | No code execution |
| JSON manifest parser | Random JSON | No crash |
| Cell schema validator | Random Cell | No crash |
| Tile payload parser | Random payload | No crash |
| Name mapper | Random tensor names | No crash, no injection |

## 11.3 Fuzz Test Configuration

```rust
struct FuzzConfig {
    // Duration
    min_duration_hours: u64,      // default 24
    
    // Coverage
    coverage_target: f64,          // default 0.8
    
    // Crash detection
    detect_leaks: bool,            // true
    detect_timeout: bool,          // true
    detect_oom: bool,              // true
    
    // Sanitizers
    address_sanitizer: bool,       // true
    memory_sanitizer: bool,        // true
    undefined_behavior_sanitizer: bool, // true
}
```

## 11.4 Fuzz Test Invariants

| ID | Invariant |
|---|---|
| TEST-FUZZ-INV-1 | Fuzz MUST NOT crash |
| TEST-FUZZ-INV-2 | Fuzz MUST NOT hang |
| TEST-FUZZ-INV-3 | Fuzz MUST NOT leak memory |
| TEST-FUZZ-INV-4 | Fuzz MUST NOT execute code |
| TEST-FUZZ-INV-5 | Fuzz MUST menolak input invalid dengan error |

---

# 12. Interoperability Tests

## 12.1 Interoperability Test Philosophy

`[TEST-INT-1]` Interoperability tests memverifikasi bahwa dua implementasi conformant menghasilkan data kompatibel.

## 12.2 Interoperability Test Cases

| Test ID | Name | Description |
|---|---|---|
| CNWS-INT-0001 | Cross-read .cd | A writes, B reads |
| CNWS-INT-0002 | Cross-write .cd | B writes, A reads |
| CNWS-INT-0003 | Tile identity match | Same input = same Tile ID |
| CNWS-INT-0004 | Manifest byte-identical | Same content = same bytes |
| CNWS-INT-0005 | Revision cross-resolve | A creates, B resolves |
| CNWS-INT-0006 | Memory cross-access | A writes, B retrieves |
| CNWS-INT-0007 | Conversion determinism | Same checkpoint = same .cd |
| CNWS-INT-0008 | Golden file match | Both match golden |
| CNWS-INT-0009 | Error compatibility | Same error codes |
| CNWS-INT-0010 | Version compatibility | Cross-version read |

## 12.3 Cross-Implementation Test Protocol

```pseudo
test_cross_read():
    // Implementation A creates store
    store_a = ImplementationA.create_store("test.cd")
    store_a.import(fixture_checkpoint)
    store_a.commit()
    
    // Implementation B reads store
    store_b = ImplementationB.open_store("test.cd")
    
    // Verify B can read all Cells
    for cell_id in store_a.list_cells():
        cell_b = store_b.resolve_cell(cell_id)
        assert cell_b is not None
        
        tiles_b = store_b.resolve_tiles(cell_b)
        for tile in tiles_b:
            data_b = store_b.read_tile(tile.tile_id)
            assert verify_tile(tile.tile_id, data_b)
    
    // Verify B can execute
    result_b = store_b.execute(test_input)
    assert result_b is valid
```

## 12.4 Byte-Identical Manifest Test

```pseudo
test_manifest_byte_identical():
    // Both implementations create manifest from same logical content
    logical_content = load_fixture("manifest_full")
    
    manifest_a = ImplementationA.serialize_manifest(logical_content)
    manifest_b = ImplementationB.serialize_manifest(logical_content)
    
    // Must be byte-identical
    assert manifest_a == manifest_b
    
    // Hash must be identical
    assert blake3_256(manifest_a) == blake3_256(manifest_b)
```

## 12.5 Interoperability Test Invariants

| ID | Invariant |
|---|---|
| TEST-INT-INV-1 | .cd dari A MUST dapat dibaca B |
| TEST-INT-INV-2 | .cd dari B MUST dapat dibaca A |
| TEST-INT-INV-3 | Tile identity MUST sama untuk input sama |
| TEST-INT-INV-4 | Manifest MUST byte-identical untuk content sama |
| TEST-INT-INV-5 | Revision MUST cross-resolvable |
| TEST-INT-INV-6 | Error codes MUST kompatibel |

---

# 13. Pass/Fail Criteria

## 13.1 Test Result Classification

| Result | Meaning |
|---|---|
| PASS | Test passed |
| FAIL | Test failed |
| SKIP | Test skipped (not applicable) |
| ERROR | Test infrastructure error |

## 13.2 Suite Pass Criteria

`[TEST-PASS-1]` Conformance suite pass criteria:

| Suite | Pass Criteria |
|---|---|
| CS-01: Content Addressing | 100% mandatory tests PASS |
| CS-02: Canonical Serialization | 100% mandatory tests PASS |
| CS-03: .cd Format | 100% mandatory tests PASS |
| CS-04: Conversion | 100% mandatory tests PASS |
| CS-05: Runtime | 100% mandatory tests PASS |
| CS-06: Revision | 100% mandatory tests PASS |
| CS-07: Memory | 100% mandatory tests PASS |
| CS-08: Integrity | 100% mandatory tests PASS |
| CS-09: Recovery | 100% mandatory tests PASS |
| CS-10: Security | 100% mandatory tests PASS |

## 13.3 Conformance Level Criteria

`[TEST-PASS-2]` Conformance level criteria:

| Level | Criteria |
|---|---|
| Level 1: Core Conformant | CS-01 s/d CS-05: 100% PASS |
| Level 2: Full Conformant | CS-01 s/d CS-10: 100% PASS |
| Level 3: Certified | Level 2 + Interop tests PASS + Performance targets met |

## 13.4 Failure Handling

`[TEST-PASS-3]` Jika mandatory test gagal:

1. Implementasi dinyatakan **NOT CONFORMANT**.
2. Failure MUST dilaporkan dengan detail.
3. Implementasi MUST diperbaiki dan di-retest.
4. Tidak ada waiver untuk mandatory test failure.

## 13.5 Performance Pass Criteria

`[TEST-PASS-4]` Performance targets (untuk Level 3):

| Metric | Target |
|---|---|
| Cell resolution latency | ≤ 1 μs |
| Tile lookup latency | ≤ 10 μs |
| Conversion throughput | ≥ 500 MB/s |
| Peak conversion RAM | ≤ 4 GiB |
| Cache hit rate (steady state) | ≥ 90% |
| Retrieval complexity | O(log N) verified |

---

# 14. Conformance Certification

## 14.1 Certification Process

```text
┌─────────────────────────────────────────────────────────────┐
│                  CERTIFICATION PROCESS                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Implementation submits for testing                      │
│     │                                                       │
│     ▼                                                       │
│  2. Run full conformance suite                              │
│     │                                                       │
│     ├── FAIL → Not conformant, report issued               │
│     │                                                       │
│     └── PASS → Continue                                    │
│         │                                                   │
│         ▼                                                   │
│  3. Run interoperability tests                              │
│     │                                                       │
│     ├── FAIL → Level 2 conformant                          │
│     │                                                       │
│     └── PASS → Continue                                    │
│         │                                                   │
│         ▼                                                   │
│  4. Run performance benchmarks                              │
│     │                                                       │
│     ├── FAIL → Level 2 conformant                          │
│     │                                                       │
│     └── PASS → Level 3 Certified                           │
│         │                                                   │
│         ▼                                                   │
│  5. Issue certificate                                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 14.2 Certificate Format

```json
{
  "certificate_id": "CNWS-CERT-2026-0001",
  "implementation": "CNWS Reference Implementation",
  "version": "1.0.0",
  "spec_version": "1.0.0",
  "conformance_level": 3,
  "suites_passed": [
    "CS-01", "CS-02", "CS-03", "CS-04", "CS-05",
    "CS-06", "CS-07", "CS-08", "CS-09", "CS-10"
  ],
  "interop_tests_passed": true,
  "performance_targets_met": true,
  "certified_at": "2026-08-11T00:00:00Z",
  "certified_by": "CNWS Governing Body",
  "valid_until": "2027-08-11T00:00:00Z"
}
```

## 14.3 Certification Invariants

| ID | Invariant |
|---|---|
| TEST-CERT-INV-1 | Certificate MUST dikeluarkan oleh governing body |
| TEST-CERT-INV-2 | Certificate MUST memiliki expiry |
| TEST-CERT-INV-3 | Certificate MUST dapat dicabut jika violation |
| TEST-CERT-INV-4 | Re-certification REQUIRED untuk major version |

---

# 15. Test Maintenance

## 15.1 Test Suite Versioning

`[TEST-MAINT-1]` Test suite version mengikuti spesifikasi version.

`[TEST-MAINT-2]` Test suite MUST backward compatible untuk minor version.

## 15.2 Test Vector Updates

`[TEST-MAINT-3]` Test vectors MAY ditambahkan untuk minor version.

`[TEST-MAINT-4]` Test vectors MUST NOT dihapus tanpa major version bump.

`[TEST-MAINT-5]` Test vectors yang sudah ada MUST NOT diubah tanpa major version bump.

## 15.3 Regression Testing

`[TEST-MAINT-6]` Regression tests MUST dijalankan pada setiap commit.

`[TEST-MAINT-7]` Regression test failure MUST memblokir merge.

---

# 16. Final Testing Contract

## 16.1 Ringkasan Keputusan Testing

| ID | Keputusan |
|---|---|
| TEST-F01 | Conformance suite bersifat normatif dan wajib. |
| TEST-F02 | Test vectors menggunakan format JSON + binary fixtures. |
| TEST-F03 | Golden files disimpan dalam repository `cnws-conformance`. |
| TEST-F04 | Hash test vectors menggunakan payload deterministik. |
| TEST-F05 | Crash tests menggunakan fault injection. |
| TEST-F06 | Corruption tests menggunakan bit-flip injection. |
| TEST-F07 | Fuzz tests menggunakan coverage-guided fuzzing. |
| TEST-F08 | Interoperability tests menggunakan cross-implementation verification. |
| TEST-F09 | Pass criteria: 100% mandatory tests lulus. |
| TEST-F10 | Conformance certificate dikeluarkan oleh governing body. |
| TEST-F11 | Test suite version mengikuti spesifikasi version. |
| TEST-F12 | Regression tests dijalankan pada setiap commit. |
| TEST-F13 | 10 conformance suites (CS-01 s/d CS-10). |
| TEST-F14 | 3 conformance levels (Core, Full, Certified). |
| TEST-F15 | Interoperability guarantee untuk implementasi conformant. |
| TEST-F16 | Test IDs menggunakan format CNWS-<SUITE>-<NUMBER>. |
| TEST-F17 | Crash recovery MUST idempotent. |
| TEST-F18 | Corruption MUST terdeteksi. |
| TEST-F19 | Fuzz MUST NOT crash. |
| TEST-F20 | Cross-implementation data MUST kompatibel. |

## 16.2 Testing Invariants

| ID | Invariant |
|---|---|
| TEST-INV-1 | Conformance suite MUST wajib untuk semua implementasi. |
| TEST-INV-2 | Mandatory tests MUST 100% lulus untuk conformance. |
| TEST-INV-3 | Test vectors MUST deterministik. |
| TEST-INV-4 | Golden files MUST byte-identical. |
| TEST-INV-5 | Crash tests MUST idempotent recovery. |
| TEST-INV-6 | Corruption tests MUST deteksi. |
| TEST-INV-7 | Fuzz tests MUST NOT crash. |
| TEST-INV-8 | Interoperability tests MUST cross-implementation. |
| TEST-INV-9 | Performance targets MUST terukur. |
| TEST-INV-10 | Certification MUST oleh governing body. |
| TEST-INV-11 | Regression tests MUST pada setiap commit. |
| TEST-INV-12 | Test suite MUST versioned. |
| TEST-INV-13 | Test vectors MUST NOT dihapus tanpa major bump. |
| TEST-INV-14 | Failure MUST dilaporkan dengan detail. |
| TEST-INV-15 | No waiver untuk mandatory test failure. |

## 16.3 The Interoperability Guarantee (Restated)

`[TEST-FINAL-1]` **Final Interoperability Guarantee**:

> Jika implementasi A dan implementasi B sama-sama conformant terhadap CNWS specification version X.Y, maka:
>
> 1. `.cd` store yang dihasilkan A MUST dapat dibaca oleh B.
> 2. `.cd` store yang dihasilkan B MUST dapat dibaca oleh A.
> 3. Cell/Tile yang dihasilkan A MUST memiliki identity yang sama jika dihasilkan B dari input yang sama.
> 4. Manifest yang dihasilkan A MUST byte-identical dengan yang dihasilkan B untuk logical content yang sama.
> 5. Revision yang dihasilkan A MUST dapat di-resolve oleh B.
> 6. Behavior runtime A MUST kompatibel dengan B untuk input yang sama.
> 7. Error codes A MUST kompatibel dengan B.
> 8. Memory entries A MUST dapat diakses oleh B.
> 9. Golden files MUST cocok untuk kedua implementasi.
> 10. Seluruh mandatory conformance tests MUST lulus untuk kedua implementasi.

## 16.4 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Testing & Conformance final dan mengikat** untuk CNWS. Ia mendefinisikan conformance suite, test vectors, golden files, pass/fail criteria, dan interoperability guarantee yang memastikan bahwa dua implementasi conformant menghasilkan behavior/data yang kompatibel.

Conformance testing bukan sekadar test plan — ia adalah **contract** yang menjamin interoperability, determinism, dan correctness seluruh implementasi CNWS.

Seluruh implementasi CNWS MUST lulus conformance suite sebelum dinyatakan conformant.

Tidak ada keputusan testing yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN TESTING & CONFORMANCE SPECIFICATION**
