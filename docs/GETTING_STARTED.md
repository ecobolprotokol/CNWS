# Getting Started with CNWS

## Prerequisites

- Rust 1.75.0 atau lebih baru
- Cargo
- Git (opsional, untuk cloning repository)

## Installation

### 1. Clone Repository

```bash
git clone https://github.com/example/cnws.git
cd cnws
```

### 2. Install Rust Toolchain

```bash
rustup toolchain install 1.75.0
rustup default 1.75.0
rustup component add rustfmt clippy
```

### 3. Build Workspace

```bash
cargo build --workspace
```

### 4. Run Tests

```bash
cargo test --workspace
```

## Quick Start

### Initialize a Store

```bash
cargo run --bin cnws -- init ./my-store --compression zstd
```

### Import a Model

```bash
cargo run --bin cnws -- import ./model.safetensors --format safetensors --store ./my-store
```

### Run Diagnostics

```bash
cargo run --bin cnws -- diag integrity --store ./my-store
cargo run --bin cnws -- diag store-status --store ./my-store
```

### Commit a Revision

```bash
cargo run --bin cnws -- revision commit --cells <hash1> <hash2> --store ./my-store
```

### Write to Memory

```bash
cargo run --bin cnws -- memory write --memory-type episodic --key "context" --value "data" --store ./my-store
```

### Query

```bash
cargo run --bin cnws -- query <cell_hash> --store ./my-store
```

## Development

### Setup Development Environment

```bash
bash scripts/setup.sh
```

### Run All Checks

```bash
just check-all
# atau
cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace && cargo run --bin cnws-conformance
```

### Run Benchmarks

```bash
cargo bench --workspace
```

## Next Steps

- Baca [Architecture Overview](ARCHITECTURE.md)
- Pelajari [API Reference](API.md)
- Ikuti [Contributing Guide](../CONTRIBUTING.md)
