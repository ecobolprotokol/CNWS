# CNWS Justfile
# Just command runner (https://github.com/casey/just)

# Build the workspace
build:
    cargo build --workspace

# Build release
build-release:
    cargo build --workspace --release

# Run tests
test:
    cargo test --workspace

# Run clippy
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Run conformance tests
conformance:
    cargo run --bin cnws-conformance

# Run benchmarks
bench:
    cargo bench --workspace

# Clean build artifacts
clean:
    cargo clean

# Setup development environment
setup:
    bash scripts/setup.sh

# Run all checks
check-all: fmt lint test conformance
    @echo "All checks passed!"

# Generate fixtures
fixtures:
    bash scripts/generate_fixtures.sh

# Default task
default:
    @just --list
