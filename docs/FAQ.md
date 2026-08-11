# Frequently Asked Questions

## General

### Apa itu CNWS?
CNWS (Canonical Neural Weight System) adalah sistem penyimpanan weight neural network yang kanonis, immutable, dan content-addressed.

### Mengapa CNWS dibutuhkan?
CNWS menyelesaikan masalah:
- Format checkpoint yang beragam dan tidak standar
- Versioning yang tidak terstruktur
- Import model yang memakan memory
- Integrity verification yang sulit

### Apakah CNWS open source?
Ya, CNWS dilisensikan under Apache 2.0.

## Technical

### Bagaimana content addressing bekerja?
Setiap data dihash menggunakan BLAKE3-256. Hash ini menjadi identitas unik untuk data tersebut. Jika data berubah, hash juga berubah.

### Apakah CNWS mendukung GPU?
Ya, CNWS mendukung GPU melalui feature flag `gpu`. Aktifkan dengan `--features gpu`.

### Bagaimana cara import model besar?
CNWS menggunakan streaming import yang bounded-memory. Model besar bisa diimport tanpa memuat seluruhnya ke memory.

### Apakah CNWS immutable?
Ya, CNWS adalah immutable store. Data yang ditulis tidak bisa diubah. Perubahan membuat entri baru dalam revision DAG.

### Bagaimana cara recovery jika ada crash?
CNWS menggunakan WAL (Write-Ahead Log) untuk recovery. Jika ada crash, sistem bisa recovery ke state yang konsisten.

## Performance

### Berapa ukuran tile default?
Default tile size adalah 4MB. Ini optimal untuk model neural network.

### Bagaimana performa CNWS?
Lihat [Performance Benchmark Spec](specs/14-performance-benchmark-spec.md) untuk target performa.

### Apakah ada cache?
Ya, CNWS memiliki multi-level cache:
- L0: GPU VRAM (256MB)
- L1: CPU RAM (2GB)
- L2: NVMe SSD (16GB)
- L3: Network (128GB)

## Development

### Bagaimana cara berkontribusi?
Lihat [CONTRIBUTING.md](../CONTRIBUTING.md) untuk panduan kontribusi.

### Di mana spesifikasi teknis?
Semua spesifikasi ada di [`docs/specs/`](specs/).

### Bagaimana cara menjalankan tests?
```bash
cargo test --workspace
cargo run --bin cnws-conformance
```
