# CNWS Development Instructions

## Project Overview
CNWS (Canonical Neural Weight System) is a Rust workspace implementing a content-addressed, immutable neural weight storage system.

## Architecture
- **Substrate Layer**: Immutable storage, revisioning, integrity, recovery
- **Lattice Layer**: Cell Graph, memory, routing, learning, cache
- **API Layer**: Public interfaces for all operations
- **Telemetry**: Metrics, logging, tracing

## Key Invariants (from Engineering Contract)
1. All identities are BLAKE3-256 hashes
2. All data is immutable
3. Content addressing is universal
4. Little-endian binary format
5. Canonical JSON for metadata
6. Streaming-first for large data
7. Zero format coupling

## Development Commands
```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Run conformance tests
cargo run --bin cnws-conformance

# Run CLI
cargo run --bin cnws -- --help
```

## Module Structure
```
cnws-core/
  src/
    lib.rs           - Library entry point
    error.rs         - CnwsError enum
    types.rs         - Foundational types
    substrate/       - Storage layer
    lattice/         - Computation layer
    api/             - Public APIs
    telemetry/       - Observability
    bin/
      main.rs        - CLI binary
```

## Coding Standards
- Use `thiserror` for error handling
- Use `serde` for serialization
- Use `parking_lot` for synchronization
- Use `blake3` for hashing
- All public APIs must be documented
- All invariants must be tested

## Testing
- Unit tests in each module
- Integration tests in `tests/`
- Conformance tests in `cnws-conformance`
- Benchmarks in `benches/`
