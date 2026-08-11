# Test Fixtures

Direktori ini berisi fixture untuk testing.

## Structure

```
fixtures/
├── checkpoints/          # Tiny test checkpoints
│   ├── tiny/
│   │   └── model.cd
│   └── small/
│       └── model.cd
├── golden/               # Golden .cd files
│   ├── superblock.cd
│   ├── manifest.cd
│   └── tile.cd
└── models/               # Sample model files
    ├── tiny.safetensors
    └── small.gguf
```

## Generating Fixtures

```bash
# Generate tiny checkpoint
cargo run --bin cnws -- init fixtures/checkpoints/tiny --compression none

# Generate golden files
python3 scripts/generate_golden.py
```
