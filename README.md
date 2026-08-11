# CNWS

**Canonical Neural Weight System**

> Streaming canonical-intelligence architecture yang memisahkan semantic Cell graph dari physical Tile storage, menggunakan BLAKE3-256 content addressing, menyediakan dynamic adaptive execution dan incremental revision melalui satu canonical `.cd` store.

---

## Status

| Item | Nilai |
|---|---|
| **Versi Spesifikasi** | `1.0.0` (Final) |
| **Versi Implementasi** | `0.1.0-dev` |
| **Status** | 🟡 Specification Complete — Implementation In Progress |
| **Fase** | Phase 1: Core Substrate |
| **License** | Apache-2.0 |
| **Spesifikasi** | 17 dokumen final |

---

## Apa itu CNWS?

CNWS adalah **canonical intelligence infrastructure** yang mengubah checkpoint LLM besar dari berbagai format (Safetensors, GGUF, PyTorch, custom) menjadi representasi canonical yang terstruktur, modular, dan independen dari format sumber.

CNWS menggabungkan dua paradigma menjadi satu sistem terpadu:

- **CNWS Substrate** — lapisan infrastruktur: streaming conversion, tile-based content-addressed storage, revision DAG, integrity verification, recovery.
- **CNWS Lattice** — lapisan intelligence: content-addressed Cell, dynamic adaptive execution, persistent memory, structural learning, selective routing.

### Arsitektur

```text
┌─────────────────────────────────────────────────────────────┐
│                          CNWS                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              CNWS LATTICE (Intelligence)            │  │
│   │                                                     │  │
│   │   Cell · Query · Cell Graph · Memory                │  │
│   │   WorkingState · Update · Dynamic Execution         │  │
│   │   Adaptive Compute · Routing · Learning             │  │
│   └──────────────────────┬──────────────────────────────┘  │
│                          │ uses                             │
│                          ▼                                  │
│   ┌─────────────────────────────────────────────────────┐  │
│   │              CNWS SUBSTRATE (Infrastructure)        │  │
│   │                                                     │  │
│   │   Streaming Pipeline · Tile Storage                 │  │
│   │   BLAKE3-256 · .cd Store · Revision DAG             │  │
│   │   Selective Loading · GC · Recovery                 │  │
│   └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Konsep Fundamental

| Konsep | Definisi |
|---|---|
| **Cell** | Unit fundamental universal CNWS. Setiap knowledge, computation, memory, routing, dan composition direpresentasikan sebagai Cell content-addressed. |
| **Tile** | Unit physical storage immutable. Cell disimpan sebagai satu atau lebih Tile. Tile diidentifikasi oleh BLAKE3-256 hash dari canonical payload. |
| **Cell Graph** | Graph semantic dari Cell dan asosiasinya. Menjawab "knowledge/computation apa ini dan bagaimana hubungannya?" |
| **Memory** | First-class persistent intelligence state. Episodic, semantic, procedural. Bukan sekadar storage. |
| **Revision** | Immutable snapshot dari perubahan Cell/Tile. Revision DAG menyimpan delta, bukan full model. |
| **`.cd`** | Canonical store directory. Source of truth untuk seluruh state CNWS. Berisi manifest, segments, revisions, memory, dan provenance. |

> 📖 Detail lengkap: [Engineering Contract](docs/specs/01-engineering-contract.md) · [Cell & Schema Spec](docs/specs/05-cell-schema.md) · [.cd Format Spec](docs/specs/04-cd-format-serialization.md)

---

## Struktur Repository

```text
cnws/
├── README.md                        # This file — entry point
├── LICENSE                          # Apache-2.0
├── Cargo.toml                       # Rust workspace manifest
├── Cargo.lock
├── rust-toolchain.toml              # Rust version pinning
│
├── docs/
│   ├── specs/                       # 17 spesifikasi final (authoritative)
│   │   ├── 01-engineering-contract.md
│   │   ├── 02-product-requirements.md
│   │   ├── 03-detailed-architecture.md
│   │   ├── 04-cd-format-serialization.md
│   │   ├── 05-cell-schema.md
│   │   ├── 06-runtime-execution.md
│   │   ├── 07-conversion-import.md
│   │   ├── 08-revision-learning.md
│   │   ├── 09-memory-retrieval.md
│   │   ├── 10-security-threat-model.md
│   │   ├── 11-reliability-recovery.md
│   │   ├── 12-api-protocol.md
│   │   ├── 13-testing-conformance.md
│   │   ├── 14-performance-benchmark.md
│   │   ├── 15-observability.md
│   │   ├── 16-operations-deployment.md
│   │   └── 17-compatibility-migration.md
│   ├── guides/                      # User & developer guides
│   └── rfcs/                        # RFC untuk perubahan spesifikasi
│
├── src/
│   ├── lib.rs                       # Public API exports
│   ├── error.rs                     # CnwsError types
│   │
│   ├── substrate/                   # CNWS Substrate layer
│   │   ├── mod.rs
│   │   ├── storage/                 # Storage Engine
│   │   ├── conversion/              # Conversion Pipeline
│   │   ├── revision/                # Revision DAG
│   │   ├── integrity/               # BLAKE3 verification
│   │   ├── gc/                      # Garbage Collection
│   │   └── recovery/                # Recovery subsystem
│   │
│   ├── lattice/                     # CNWS Lattice layer
│   │   ├── mod.rs
│   │   ├── runtime/                 # Execution Engine
│   │   ├── memory/                  # Memory System
│   │   ├── routing/                 # Routing Engine
│   │   ├── learning/                # Learning Engine
│   │   └── cache/                   # Cache Manager
│   │
│   ├── api/                         # Public API layer
│   │   ├── mod.rs
│   │   ├── storage.rs
│   │   ├── conversion.rs
│   │   ├── runtime.rs
│   │   ├── revision.rs
│   │   ├── memory.rs
│   │   └── admin.rs
│   │
│   └── telemetry/                   # Observability
│       ├── mod.rs
│       ├── metrics.rs
│       ├── logging.rs
│       └── tracing.rs
│
├── tests/
│   ├── unit/                        # Unit tests
│   ├── integration/                 # Integration tests
│   ├── conformance/                 # Conformance test suite (CS-01..CS-10)
│   └── interoperability/            # Cross-implementation tests
│
├── fixtures/                        # Test fixtures & golden files
│   ├── cells/
│   ├── tiles/
│   ├── manifests/
│   ├── checkpoints/                 # Tiny checkpoints untuk testing
│   └── golden/                      # Golden .cd files
│
├── benches/                         # Criterion benchmarks
│
├── tools/
│   ├── cnws-cli/                    # CLI binary
│   ├── conformance-runner/          # Conformance test runner
│   └── migration/                   # Migration tools
│
├── examples/
│   ├── basic_usage.rs
│   ├── conversion.rs
│   ├── runtime_inference.rs
│   ├── revision_branching.rs
│   └── memory_operations.rs
│
├── scripts/
│   ├── setup.sh                     # Development environment setup
│   ├── build.sh
│   ├── test.sh
│   └── conformance.sh
│
└── .github/
    └── workflows/
        ├── ci.yml                   # Continuous integration
        ├── conformance.yml          # Conformance test suite
        └── benchmark.yml            # Performance benchmarks
```

---

## Dokumen Spesifikasi

CNWS didefinisikan oleh **17 dokumen spesifikasi final**. Seluruh implementasi, code review, testing, dan maintenance MUST conformant terhadap dokumen-dokumen ini.

| # | Dokumen | Scope | Status |
|---|---|---|---|
| 01 | [Engineering Contract](docs/specs/01-engineering-contract.md) | Arsitektur final, invariant, keputusan non-negotiable | ✅ Final |
| 02 | [Product Requirements](docs/specs/02-product-requirements.md) | Use cases, capability matrix, acceptance criteria | ✅ Final |
| 03 | [Detailed Architecture](docs/specs/03-detailed-architecture.md) | Component architecture, module boundaries, data flow | ✅ Final |
| 04 | [.cd Format & Serialization](docs/specs/04-cd-format-serialization.md) | Wire format, binary layout, canonical serialization | ✅ Final |
| 05 | [Cell & Schema](docs/specs/05-cell-schema.md) | Cell taxonomy, schema, compatibility rules | ✅ Final |
| 06 | [Runtime & Execution](docs/specs/06-runtime-execution.md) | Dynamic execution, adaptive compute, halt conditions | ✅ Final |
| 07 | [Conversion & Import](docs/specs/07-conversion-import.md) | Safetensors/GGUF/PyTorch import, normalization, tiling | ✅ Final |
| 08 | [Revision & Learning](docs/specs/08-revision-learning.md) | Revision DAG, branching, merging, structural learning | ✅ Final |
| 09 | [Memory & Retrieval](docs/specs/09-memory-retrieval.md) | Episodic/semantic/procedural memory, retrieval, consolidation | ✅ Final |
| 10 | [Security & Threat Model](docs/specs/10-security-threat-model.md) | Trust boundaries, threat catalog, mitigations | ✅ Final |
| 11 | [Reliability & Recovery](docs/specs/11-reliability-recovery.md) | Crash consistency, WAL, recovery, quarantine | ✅ Final |
| 12 | [API & Protocol](docs/specs/12-api-protocol.md) | Public API, traits, error semantics, SDK boundary | ✅ Final |
| 13 | [Testing & Conformance](docs/specs/13-testing-conformance.md) | Conformance suite, test vectors, interoperability | ✅ Final |
| 14 | [Performance Benchmark](docs/specs/14-performance-benchmark.md) | Workloads, measurement, acceptance thresholds | ✅ Final |
| 15 | [Observability](docs/specs/15-observability.md) | Metrics, logging, tracing, diagnostics | ✅ Final |
| 16 | [Operations & Deployment](docs/specs/16-operations-deployment.md) | Installation, configuration, backup, monitoring | ✅ Final |
| 17 | [Compatibility & Migration](docs/specs/17-compatibility-migration.md) | Version migration, legacy migration, compatibility rules | ✅ Final |

> ⚠️ **Engineering Contract (dokumen 01) adalah authority tertinggi.** Jika terjadi konflik antara README dan Engineering Contract, Engineering Contract menang.

---

## Fase Pengembangan

```text
Phase 0: Specification          ████████████████████  100%  ✅ COMPLETE
Phase 1: Core Substrate         ████░░░░░░░░░░░░░░░░   20%  🟡 IN PROGRESS
Phase 2: Core Lattice           ░░░░░░░░░░░░░░░░░░░░    0%  ⚪ NOT STARTED
Phase 3: Versioning & Learning  ░░░░░░░░░░░░░░░░░░░░    0%  ⚪ NOT STARTED
Phase 4: Conformance & Testing  ░░░░░░░░░░░░░░░░░░░░    0%  ⚪ NOT STARTED
Phase 5: Certification & Release░░░░░░░░░░░░░░░░░░░░    0%  ⚪ NOT STARTED
```

### Phase 1: Core Substrate (Current)

| Task | Status | Spec Reference |
|---|---|---|
| BLAKE3-256 integration | 🟡 | .cd Format Spec §4 |
| .cd store layout | 🟡 | .cd Format Spec §10 |
| SUPERBLOCK read/write | 🟡 | .cd Format Spec §10.2 |
| Segment format | ⚪ | .cd Format Spec §10.4 |
| Tile storage | ⚪ | .cd Format Spec §10.5 |
| Manifest serialization | ⚪ | .cd Format Spec §10.3 |
| Conversion pipeline skeleton | ⚪ | Conversion Spec §2 |
| Safetensors importer | ⚪ | Conversion Spec §5 |
| Atomic commit / WAL | ⚪ | Reliability Spec §5-6 |
| Store initialization | ⚪ | Operations Spec §4 |

---

## Quick Start

### Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust | 1.75+ | `rust-toolchain.toml` pinned |
| Cargo | Latest | Bundled dengan Rust |
| BLAKE3 | Via crate | Otomatis via dependency |
| CUDA | 12.0+ | Optional, untuk GPU support |
| GCC/Clang | Latest | Untuk native dependencies |

```bash
# Install Rust (jika belum)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version   # >= 1.75.0
cargo --version
```

### Build

```bash
# Clone repository
git clone https://github.com/cnws/cnws.git
cd cnws

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Build dengan GPU support
cargo build --release --features gpu
```

### Test

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Conformance test suite
cargo test --test conformance

# Conformance suite spesifik
cargo test --test conformance -- cs01  # Content Addressing
cargo test --test conformance -- cs03  # .cd Format

# Semua tests dengan output
cargo test -- --nocapture
```

### Contoh Dasar

```bash
# Inisialisasi store
cargo run --bin cnws -- init /tmp/test.cd \
    --model-id "test-model"

# Import checkpoint (Safetensors)
cargo run --bin cnws -- import \
    fixtures/checkpoints/tiny.safetensors \
    --target /tmp/test.cd

# Verifikasi integrity
cargo run --bin cnws -- diag integrity /tmp/test.cd

# Lihat store info
cargo run --bin cnws -- diag store-status /tmp/test.cd

# Buat revision
cargo run --bin cnws -- revision commit /tmp/test.cd \
    --message "initial import"

# Health check
cargo run --bin cnws -- diag health /tmp/test.cd
```

### Programmatic Usage

```rust
use cnws::prelude::*;

#[tokio::main]
async fn main() -> Result<(), CnwsError> {
    // 1. Open store
    let config = StoreConfigBuilder::new("/tmp/test.cd")
        .create_if_missing(true)
        .gpu_budget(8 * GB)
        .build()?;
    
    let store = StorageEngine::open(&config)?;
    
    // 2. Resolve a Cell
    let cell = store.resolve_cell_by_name("model.layer.0.self_attn.q_proj")?;
    
    // 3. Load Tiles
    let tiles = store.resolve_tiles(&cell, AccessPolicy::FullCell).await?;
    
    // 4. Use Tiles...
    for tile in &tiles {
        println!("Tile: {} ({} bytes)", tile.tile_id, tile.size_bytes);
    }
    
    // 5. Release
    for tile in tiles {
        store.release(tile);
    }
    
    // 6. Close
    store.close()?;
    
    Ok(())
}
```

---

## Development Sequence

Development sequence ringkas untuk engineer baru:

```text
1. Baca Engineering Contract (docs/specs/01)
   → Pahami 7 prinsip arsitektural dan invariant
   → Pahami Cell, Tile, .cd, BLAKE3-256

2. Baca spesifikasi yang relevan dengan task
   → Storage? → .cd Format Spec (04)
   → Runtime? → Runtime & Execution Spec (06)
   → Conversion? → Conversion & Import Spec (07)
   → dst.

3. Implementasi sesuai spesifikasi
   → Setiap fungsi harus traceable ke requirement spesifikasi
   → Gunakan ID requirement dalam comment: [CELL-1], [HASH-2], dll.

4. Tulis tests
   → Unit tests untuk setiap fungsi
   → Conformance tests untuk setiap spesifikasi yang relevan

5. Jalankan conformance suite
   → cargo test --test conformance
   → Semua mandatory tests harus lulus

6. Submit PR
   → Code review terhadap spesifikasi
   → CI harus hijau
```

> 📖 Detail lengkap: [Testing & Conformance Spec](docs/specs/13-testing-conformance.md)

---

## Governance

### Authority

```text
┌─────────────────────────────────────────────────────────────┐
│                  ENGINEERING CONTRACT                       │
│                  (docs/specs/01)                            │
│                                                             │
│         Authority tertinggi untuk seluruh CNWS              │
│         Invariant final, non-negotiable                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│              16 DOKUMEN SPESIFIKASI LAINNYA                 │
│                                                             │
│         Mendefinisikan detail per subsystem                 │
│         MUST conformant terhadap Engineering Contract       │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                      README.md                              │
│                                                             │
│         Entry point dan orientasi                           │
│         BUKAN dokumen spesifikasi tambahan                  │
│         Jika konflik → Engineering Contract menang          │
└─────────────────────────────────────────────────────────────┘
```

`README.md` berfungsi sebagai **entry point**. Ia tidak mendefinisikan behavior normatif. Seluruh detail teknis dirujuk ke dokumen spesifikasi masing-masing.

### Development Rules

| Rule | Requirement |
|---|---|
| Spesifikasi pertama | Setiap perubahan arsitektural harus melalui amandemen spesifikasi |
| Traceability | Setiap fungsi harus traceable ke requirement spesifikasi |
| Conformance | Semua mandatory conformance tests harus lulus |
| Review | Code review terhadap spesifikasi, bukan hanya style |
| Testing | Unit tests + conformance tests wajib |
| Documentation | Public API harus terdokumentasi |

### Conformance Requirements

Implementasi CNWS dinyatakan **conformant** jika:

1. ✅ Seluruh mandatory conformance tests lulus (CS-01 s/d CS-10)
2. ✅ Tidak ada invariant `FAC-*` yang dilanggar
3. ✅ Integrity verification berfungsi
4. ✅ Crash recovery tidak menghasilkan inconsistent store
5. ✅ Zero Format Coupling terjaga
6. ✅ Performance targets terpenuhi

> 📖 Detail: [Testing & Conformance Spec](docs/specs/13-testing-conformance.md)

### Security Overview

CNWS mengadopsi **zero-trust posture** terhadap seluruh input eksternal:

- Checkpoint eksternal diperlakukan sebagai **untrusted**
- PyTorch importer menggunakan **restricted unpickler**
- BLAKE3-256 verification **sebelum eksekusi**
- Path traversal **ditolak di semua layer**
- Resource limits **di-enforce**
- Security events **logged dan reported**

> 📖 Detail: [Security & Threat Model](docs/specs/10-security-threat-model.md)

### Compatibility & Versioning

| Aspek | Kebijakan |
|---|---|
| Format version | Semver (major.minor.patch) |
| Backward compatibility | Dalam major version yang sama |
| Forward incompatibility | Major lebih tinggi ditolak |
| Deprecation | Minimum 2 minor versions notice |
| Migration | Atomic, rollback-able, verified |
| Legacy support | CNWS-X dan LATTICE migration paths tersedia |

> 📖 Detail: [Compatibility & Migration Spec](docs/specs/17-compatibility-migration.md)

### Contribution Rules

1. **Fork** repository dan buat feature branch.
2. **Baca** spesifikasi yang relevan sebelum implementasi.
3. **Implementasi** sesuai spesifikasi. Gunakan ID requirement dalam comment.
4. **Tulis tests**: unit tests + conformance tests.
5. **Jalankan** `cargo test` dan `cargo test --test conformance`.
6. **Submit PR** dengan deskripsi yang merujuk ke spesifikasi.
7. **Code review** akan memeriksa konformitas terhadap spesifikasi.
8. **CI** harus hijau sebelum merge.

Perubahan terhadap spesifikasi harus melalui proses **RFC** di `docs/rfcs/`.

### License

CNWS dilisensikan di bawah **Apache License 2.0**. Lihat [LICENSE](LICENSE) untuk detail.

```text
Copyright 2026 CNWS Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

---

## Tiga Pertanyaan Utama

### "Apa CNWS?"

CNWS adalah canonical intelligence infrastructure yang merepresentasikan seluruh state model (knowledge, computation, memory, routing, composition) sebagai **Cell content-addressed** dalam satu canonical `.cd` store. CNWS menyediakan streaming conversion dari berbagai format checkpoint, dynamic adaptive execution, persistent memory, structural learning, dan incremental versioning melalui Revision DAG.

→ [Engineering Contract](docs/specs/01-engineering-contract.md) · [Product Requirements](docs/specs/02-product-requirements.md)

### "Bagaimana repository ini digunakan dan dikembangkan?"

Repository ini digunakan melalui CLI (`cnws`) atau Rust API (`cnws` crate). Development mengikuti spesifikasi: baca spesifikasi → implementasi → tulis tests → jalankan conformance → submit PR. Seluruh perubahan arsitektural harus melalui amandemen spesifikasi.

→ [Quick Start](#quick-start) · [Development Sequence](#development-sequence) · [Contribution Rules](#contribution-rules)

### "Di mana specification resmi untuk setiap bagian sistem?"

Seluruh spesifikasi resmi berada di `docs/specs/`. Ada 17 dokumen final yang mendefinisikan setiap aspek CNWS. Engineering Contract (dokumen 01) adalah authority tertinggi.

→ [Dokumen Spesifikasi](#dokumen-spesifikasi)

---

## Link Cepat

| Tujuan | Dokumen |
|---|---|
| Pahami arsitektur CNWS | [01 - Engineering Contract](docs/specs/01-engineering-contract.md) |
| Pahami kebutuhan produk | [02 - Product Requirements](docs/specs/02-product-requirements.md) |
| Implementasi storage | [04 - .cd Format](docs/specs/04-cd-format-serialization.md) |
| Implementasi Cell | [05 - Cell & Schema](docs/specs/05-cell-schema.md) |
| Implementasi runtime | [06 - Runtime & Execution](docs/specs/06-runtime-execution.md) |
| Implementasi converter | [07 - Conversion & Import](docs/specs/07-conversion-import.md) |
| Implementasi versioning | [08 - Revision & Learning](docs/specs/08-revision-learning.md) |
| Implementasi memory | [09 - Memory & Retrieval](docs/specs/09-memory-retrieval.md) |
| Security review | [10 - Security & Threat Model](docs/specs/10-security-threat-model.md) |
| Implementasi recovery | [11 - Reliability & Recovery](docs/specs/11-reliability-recovery.md) |
| Implementasi API | [12 - API & Protocol](docs/specs/12-api-protocol.md) |
| Tulis tests | [13 - Testing & Conformance](docs/specs/13-testing-conformance.md) |
| Ukur performa | [14 - Performance Benchmark](docs/specs/14-performance-benchmark.md) |
| Implementasi observability | [15 - Observability](docs/specs/15-observability.md) |
| Deploy ke produksi | [16 - Operations & Deployment](docs/specs/16-operations-deployment.md) |
| Migrasi antar versi | [17 - Compatibility & Migration](docs/specs/17-compatibility-migration.md) |

---

*CNWS — Canonical Neural Weight System. Specification version 1.0.0. Implementation version 0.1.0-dev.*
