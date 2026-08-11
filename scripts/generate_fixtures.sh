#!/usr/bin/env bash
# CNWS Test Fixtures Generator
set -euo pipefail

echo "Generating CNWS test fixtures..."

# Create directories
mkdir -p fixtures/checkpoints/tiny
mkdir -p fixtures/checkpoints/small
mkdir -p fixtures/golden
mkdir -p fixtures/models

# Generate golden files
python3 scripts/generate_golden.py

# Create tiny checkpoint
echo "Creating tiny checkpoint..."
cargo run --bin cnws -- init fixtures/checkpoints/tiny --compression none
echo "test data" > fixtures/checkpoints/tiny/test.txt

# Create small checkpoint
echo "Creating small checkpoint..."
cargo run --bin cnws -- init fixtures/checkpoints/small --compression zstd

echo "Fixtures generated successfully!"
