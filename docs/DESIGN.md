# CNWS Design Decisions

## Design Principles

### 1. Content Addressing
Semua identitas dalam CNWS menggunakan BLAKE3-256 hash. Ini memastikan:
- **Determinism**: Hash yang sama untuk data yang sama
- **Collision Resistance**: Kemungkinan collision sangat kecil
- **Tamper Detection**: Perubahan data mengubah hash

### 2. Immutability
Data yang ditulis ke store tidak bisa diubah. Ini memastikan:
- **Consistency**: Data selalu konsisten
- **Auditability**: Semua perubahan tercatat dalam revision DAG
- **Safety**: Tidak ada accidental modification

### 3. Streaming-First
Import model besar menggunakan streaming untuk:
- **Bounded Memory**: Memory usage terbatas
- **Scalability**: Support model berukuran besar
- **Progress Tracking**: Bisa track progress import

### 4. Zero Format Coupling
Runtime independen dari format checkpoint eksternal:
- **Flexibility**: Bisa support format baru tanpa ubah runtime
- **Portability**: Runtime bisa dipindah antar platform
- **Maintainability**: Perubahan format tidak affect runtime

### 5. Little-Endian
Semua binary integers menggunakan little-endian:
- **Consistency**: Konsisten dengan x86/ARM architectures
- **Portability**: Bisa dibaca di platform berbeda
- **Simplicity**: Lebih mudah diimplementasikan

## Architecture Decisions

### ADR-001: Rust sebagai Bahasa Utama
**Keputusan**: Menggunakan Rust untuk implementasi CNWS.

**Alasan**:
- Memory safety tanpa garbage collector
- Performance setara C++
- Strong type system
- Excellent ecosystem untuk systems programming

### ADR-002: BLAKE3 untuk Hashing
**Keputusan**: Menggunakan BLAKE3-256 untuk content addressing.

**Alasan**:
- Lebih cepat dari SHA-256
- Cryptographic security
- Streaming support
- Small code footprint

### ADR-003: Tile-based Storage
**Keputusan**: Menggunakan tile-based storage dengan ukuran 4MB.

**Alasan**:
- Efficient untuk model neural network
- Support streaming read/write
- Easy garbage collection
- Good cache locality

### ADR-004: Revision DAG
**Keputusan**: Menggunakan DAG untuk versioning.

**Alasan**:
- Support branching
- Efficient merge
- Immutable history
- Easy traversal

### ADR-005: Multi-level Cache
**Keputusan**: Menggunakan hierarchy cache L0-L3.

**Alasan**:
- Optimal untuk berbagai workload
- GPU VRAM untuk hot data
- Network untuk distributed access
- Automatic promotion/demotion

## Trade-offs

### Memory vs Performance
- **Trade-off**: Lebih banyak memory untuk better performance
- **Decision**: Use multi-level cache dengan automatic eviction

### Complexity vs Flexibility
- **Trade-off**: Lebih kompleks untuk lebih flexible
- **Decision**: Use layered architecture dengan clear interfaces

### Safety vs Performance
- **Trade-off**: Lebih aman tapi lebih lambat
- **Decision**: Use Rust untuk memory safety tanpa GC overhead
