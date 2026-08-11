# CNWS Development Instructions

## Project Overview

CNWS (Canonical Neural Weight System) adalah streaming canonical-intelligence architecture yang memisahkan semantic Cell graph dari physical Tile storage, menggunakan BLAKE3-256 content addressing.

## Architecture

```
cnws/
├── Cargo.toml                 # Workspace manifest
├── LICENSE                    # Apache-2.0
├── rust-toolchain.toml        # Rust 1.75
├── README.md                  # Entry point
├── docs/
│   ├── specs/                 # 17 spesifikasi final (authoritative)
│   ├── guides/                # User & developer guides
│   └── rfcs/                  # RFC untuk perubahan spesifikasi
├── cnws-core/                 # Core library
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── types.rs
│       ├── substrate/         # CNWS Substrate layer
│       │   ├── storage/       # Storage Engine
│       │   ├── conversion/    # Conversion Pipeline
│       │   ├── revision/      # Revision DAG
│       │   ├── integrity/     # BLAKE3 verification
│       │   ├── gc/            # Garbage Collection
│       │   └── recovery/      # Recovery subsystem
│       ├── lattice/           # CNWS Lattice layer
│       │   ├── runtime/       # Execution Engine
│       │   ├── memory/        # Memory System
│       │   ├── routing/       # Routing Engine
│       │   ├── learning/      # Learning Engine
│       │   └── cache/         # Cache Manager
│       ├── api/               # Public API layer
│       └── telemetry/         # Observability
├── cnws-cli/                  # CLI binary
├── cnws-conformance/          # Conformance test suite
├── tests/                     # Integration tests
├── fixtures/                  # Test fixtures
├── benches/                   # Criterion benchmarks
├── tools/                     # Additional tools
├── examples/                  # Usage examples
└── scripts/                   # Build/test scripts
```

## Development Rules

### 1. Specification First
- Seluruh perubahan arsitektural harus melalui amandemen spesifikasi
- Setiap fungsi harus traceable ke requirement spesifikasi
- Gunakan ID requirement dalam comment: [CELL-1], [HASH-2], dst.

### 2. Conformance
- Semua mandatory conformance tests harus lulus
- Implementasi MUST conformant terhadap Engineering Contract (01)
- Engineering Contract adalah authority tertinggi

### 3. Code Quality
- Gunakan `cargo fmt` untuk formatting
- Gunakan `cargo clippy` untuk linting
- Semua public API harus terdokumentasi
- Unit tests + conformance tests wajib

### 4. Error Handling
- Gunakan `CnwsError` untuk semua error
- Error codes mengikuti spesifikasi: CNWS-E-*
- Fatal error MUST menghentikan operasi
- Recoverable error MUST mencoba recovery

### 5. Content Addressing
- Seluruh entitas content-addressed menggunakan BLAKE3-256
- Hash dihitung dari canonical uncompressed payload
- Compression MUST NOT mengubah identity

### 6. Binary Format
- Seluruh integer little-endian
- JSON UTF-8 dengan canonical serialization
- Alignment minimum 4 KiB, preferred 64 KiB
- Magic bytes harus exact match

## Key Invariants (FAC-*)

- FAC-1: CNWS adalah satu paradigma terpadu
- FAC-2: Unit fundamental universal adalah Cell
- FAC-3: Tile adalah unit physical storage
- FAC-4: Cell:Tile = satu-ke-banyak
- FAC-5: Cell identity = BLAKE3-256 canonical payload
- FAC-6: Cell dan Tile immutable
- FAC-7: Content addressing tunggal BLAKE3-256
- FAC-8: `.cd` adalah canonical store dan source of truth
- FAC-9: `.cd` menyimpan Cell, Memory, Routing, Composition, Provenance
- FAC-10: MANIFEST.cd adalah root manifest
- FAC-11: Zero Format Coupling
- FAC-12: Streaming-First conversion bounded-memory
- FAC-13: Cell selection content-based
- FAC-14: Computation dynamically composed per input
- FAC-15: Tidak ada fixed-depth layer stack
- FAC-16: Context ditangani memory content-addressed
- FAC-17: Context MUST NOT tumbuh linear terhadap sequence length
- FAC-18: Memory first-class dan persistent
- FAC-19: Working memory bounded
- FAC-20: Learning tidak membutuhkan global parameter update
- FAC-21: Learning cost O(affected_cells)
- FAC-22: Learning MUST NOT catastrophic forgetting
- FAC-23: Specialization tanpa full-model copy
- FAC-24: Versioning melalui satu Revision DAG
- FAC-25: Revision delta pada level Cell/Tile
- FAC-26: Inference tidak membutuhkan full-model loading
- FAC-27: Inference hanya mengaktifkan Cell relevan
- FAC-28: Compute adaptif terhadap difficulty
- FAC-29: Budget hard-enforced
- FAC-30: Active parameter ratio < 10%
- FAC-31: Total knowledge dapat tumbuh tanpa menaikkan compute per token
- FAC-32: GC berbasis reachability dari revision roots
- FAC-33: Integrity verification sebelum eksekusi
- FAC-34: Deterministik untuk input dan state sama
- FAC-35: Engineering Contract adalah single source of truth

## Building

```bash
# Build all
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run conformance tests
cargo test --test conformance

# Run CLI
cargo run --bin cnws -- init /tmp/test.cd --model-id "test-model"
```

## Contributing

1. Fork repository dan buat feature branch
2. Baca spesifikasi yang relevan sebelum implementasi
3. Implementasi sesuai spesifikasi
4. Tulis tests: unit tests + conformance tests
5. Jalankan `cargo test` dan `cargo test --test conformance`
6. Submit PR dengan deskripsi yang merujuk ke spesifikasi
