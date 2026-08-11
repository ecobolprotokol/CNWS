#!/usr/bin/env bash
# CNWS Development Environment Setup
set -euo pipefail

echo "Setting up CNWS development environment..."

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.75.0
    source "$HOME/.cargo/env"
fi

# Verify Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust version: $RUST_VERSION"

# Install components
echo "Installing Rust components..."
rustup component add rustfmt clippy

# Build workspace
echo "Building workspace..."
cargo build --workspace

# Run tests
echo "Running tests..."
cargo test --workspace

echo "Setup complete!"
