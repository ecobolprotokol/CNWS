#!/usr/bin/env bash
# CNWS Build Script
set -euo pipefail

echo "Building CNWS..."

# Format code
echo "Formatting code..."
cargo fmt --all

# Run clippy
echo "Running clippy..."
cargo clippy --workspace -- -D warnings

# Build
echo "Building..."
cargo build --workspace

# Build release
echo "Building release..."
cargo build --workspace --release

echo "Build complete!"
