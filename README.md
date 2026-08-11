# CNWS - Canonical Neural Weight System

[![CI](https://github.com/example/cnws/workflows/CI/badge.svg)](https://github.com/example/cnws/actions)
[![Conformance](https://github.com/example/cnws/workflows/Conformance/badge.svg)](https://github.com/example/cnws/actions)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

**CNWS** adalah sistem penyimpanan weight neural network yang kanonis, immutable, dan content-addressed. Sistem ini mengimplementasikan spesifikasi lengkap yang didefinisikan dalam [`docs/specs/`](docs/specs/).

## Fitur Utama

- **Content Addressing**: Semua identitas menggunakan BLAKE3-256 hash
- **Immutable Storage**: Data tidak bisa diubah setelah ditulis
- **Revision DAG**: Versioning dengan delta di level Cell/Tile
- **Streaming Import**: Import model besar dengan bounded memory
- **Zero Format Coupling**: Runtime independen dari format checkpoint eksternal
- **Multi-level Cache**: L0 (GPU VRAM) → L1 (CPU RAM) → L2 (NVMe) → L3 (Network)
- **Memory Hierarchy**: Episodic, Semantic, Procedural, Working, Long-term
- **Full Observability**: Metrics (Prometheus), Logging (JSON), Tracing (OpenTelemetry)

## Struktur Repository

```
cnws/
├── Cargo.toml                 # Workspace root
├── README.md                  # Dokumentasi ini
├── LICENSE                    # Apache 2.0
├── rust-toolchain.toml        # Rust 1.75.0
├── .gitignore
├── .claude/instructions.md    # Development instructions
├── .github/
│   ├── workflows/
│   │   ├── ci.yml            # CI pipeline
│   │   ├── conformance.yml   # Conformance tests
│   │   └── benchmark.yml     # Performance benchmarks
│   └── dependabot.yml
├── docs/
│   └── specs/                 # 17 spesifikasi teknis
│       ├── 01-engineering-contract.md
│       ├── 02-product-requirements.md
│       ├── 03-detailed-architecture.md
│       ├── 04-cd-format-spec.md
│       ├── 05-cell-schema-spec.md
│       ├── 06-conversion-import-spec.md
│       ├── 07-runtime-execution-spec.md
│       ├── 08-memory-retrieval-spec.md
│       ├── 09-revision-learning-spec.md
│       ├── 10-api-protocol-spec.md
│       ├── 11-security-threat-model.md
│       ├── 12-reliability-recovery-spec.md
│       ├── 13-testing-conformance-spec.md
│       ├── 14-performance-benchmark-spec.md
│       ├── 15-observability-spec.md
│       ├── 16-operations-deployment-spec.md
│       └── 17-compatibility-migration-spec.md
├── cnws-core/                 # Core library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs           # CnwsError enum
│       ├── types.rs           # Foundational types
│       ├── substrate/         # Storage layer
│       │   ├── mod.rs
│       │   ├── storage.rs     # Tile storage engine
│       │   ├── integrity.rs   # BLAKE3-256 verification
│       │   ├── revision.rs    # Revision DAG
│       │   ├── gc.rs          # Garbage collection
│       │   ├── recovery.rs    # WAL-based recovery
│       │   └── conversion.rs  # Format import pipeline
│       ├── lattice/           # Computation layer
│       │   ├── mod.rs
│       │   ├── runtime.rs     # Cell Graph execution
│       │   ├── memory.rs      # Memory system
│       │   ├── routing.rs     # Cell routing
│       │   ├── learning.rs    # Structural learning
│       │   └── cache.rs       # Multi-level cache
│       ├── api/               # Public APIs
│       │   ├── mod.rs
│       │   ├── storage.rs
│       │   ├── conversion.rs
│       │   ├── runtime.rs
│       │   ├── revision.rs
│       │   ├── memory.rs
│       │   └── admin.rs
│       ├── telemetry/         # Observability
│       │   ├── mod.rs
│       │   ├── metrics.rs     # Prometheus metrics
│       │   ├── logging.rs     # Structured logging
│       │   └── tracing.rs     # Distributed tracing
│       └── bin/
│           └── main.rs        # CLI binary
├── cnws-cli/                  # CLI crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── cnws-conformance/          # Conformance tests
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Quick Start

### Prerequisites

- Rust 1.75.0 or later
- Cargo

### Build

```bash
# Build workspace
cargo build --workspace

# Build with release optimizations
cargo build --workspace --release
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run conformance tests
cargo run --bin cnws-conformance

# Run benchmarks
cargo bench --workspace
```

### CLI Usage

```bash
# Initialize a new store
cargo run --bin cnws -- init ./my-store --compression zstd

# Import a model
cargo run --bin cnws -- import ./model.safetensors --format safetensors --store ./my-store

# Run diagnostics
cargo run --bin cnws -- diag integrity --store ./my-store
cargo run --bin cnws -- diag store-status --store ./my-store

# Commit a revision
cargo run --bin cnws -- revision commit --cells <hash1> <hash2> --store ./my-store

# Write to memory
cargo run --bin cnws -- memory write --memory-type episodic --key "context" --value "data" --store ./my-store

# Query
cargo run --bin cnws -- query <cell_hash> --store ./my-store

# Export metrics
cargo run --bin cnws -- metrics --format prometheus
```

## Arsitektur

CNWS dibangun di atas 4 lapisan utama:

### 1. Substrate Layer
- **Storage Engine**: Tile-based immutable storage dengan BLAKE3-256 content addressing
- **Integrity**: Verifikasi BLAKE3-256, quarantine untuk tile korup
- **Revision DAG**: Versioning immutable dengan delta di level Cell/Tile
- **GC**: Mark-and-sweep garbage collection
- **Recovery**: WAL-based crash recovery
- **Conversion**: Streaming import untuk Safetensors, GGUF, PyTorch, ONNX

### 2. Lattice Layer
- **Runtime**: Cell Graph execution engine dengan dependency resolution
- **Memory**: Hierarchical memory (Episodic, Semantic, Procedural, Working, Long-term)
- **Routing**: Cell selection dengan cosine similarity
- **Learning**: Structural learning dan composition pattern detection
- **Cache**: Multi-level cache (L0-L3) dengan LRU eviction

### 3. API Layer
- **StorageApi**: Operasi tile dasar
- **ConversionApi**: Import model eksternal
- **RuntimeApi**: Eksekusi Cell Graph
- **RevisionApi**: Manajemen revisi
- **MemoryApi**: Operasi memory
- **AdminApi**: GC, recovery, verifikasi

### 4. Telemetry Layer
- **Metrics**: Prometheus metrics (gauge, counter, histogram)
- **Logging**: Structured JSON logging
- **Tracing**: Distributed tracing dengan OpenTelemetry

## Spesifikasi

Semua spesifikasi teknis tersedia di [`docs/specs/`](docs/specs/):

| # | Spesifikasi | Deskripsi |
|---|-------------|-----------|
| 01 | [Engineering Contract](docs/specs/01-engineering-contract.md) | Arsitektur dan invariant sistem |
| 02 | [Product Requirements](docs/specs/02-product-requirements.md) | Capability matrix dan use cases |
| 03 | [Detailed Architecture](docs/specs/03-detailed-architecture.md) | Module boundaries dan interfaces |
| 04 | [.cd Format Spec](docs/specs/04-cd-format-spec.md) | Wire format dan storage layout |
| 05 | [Cell Schema Spec](docs/specs/05-cell-schema-spec.md) | Cell taxonomy dan types |
| 06 | [Conversion Import Spec](docs/specs/06-conversion-import-spec.md) | Import pipeline dan streaming |
| 07 | [Runtime Execution Spec](docs/specs/07-runtime-execution-spec.md) | Cell Graph execution |
| 08 | [Memory Retrieval Spec](docs/specs/08-memory-retrieval-spec.md) | Memory system design |
| 09 | [Revision Learning Spec](docs/specs/09-revision-learning-spec.md) | Versioning dan learning |
| 10 | [API Protocol Spec](docs/specs/10-api-protocol-spec.md) | Public API definitions |
| 11 | [Security Threat Model](docs/specs/11-security-threat-model.md) | Security analysis |
| 12 | [Reliability Recovery Spec](docs/specs/12-reliability-recovery-spec.md) | Recovery dan WAL |
| 13 | [Testing Conformance Spec](docs/specs/13-testing-conformance-spec.md) | Test strategy |
| 14 | [Performance Benchmark Spec](docs/specs/14-performance-benchmark-spec.md) | Benchmark targets |
| 15 | [Observability Spec](docs/specs/15-observability-spec.md) | Metrics, logging, tracing |
| 16 | [Operations Deployment Spec](docs/specs/16-operations-deployment-spec.md) | Deployment guide |
| 17 | [Compatibility Migration Spec](docs/specs/17-compatibility-migration-spec.md) | Migration strategy |

## Kontribusi

Lihat [CONTRIBUTING.md](CONTRIBUTING.md) untuk panduan kontribusi.

## License

Apache 2.0 - Lihat [LICENSE](LICENSE) untuk detail.
