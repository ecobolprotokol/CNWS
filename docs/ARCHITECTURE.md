# CNWS Architecture Overview

## System Architecture

CNWS dibangun dengan arsitektur berlapis (layered architecture) yang memisahkan concerns dengan jelas:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│                  (CLI, APIs, Libraries)                     │
├─────────────────────────────────────────────────────────────┤
│                    Public API Layer                         │
│         (StorageApi, ConversionApi, RuntimeApi, etc)        │
├─────────────────────────────────────────────────────────────┤
│                    Lattice Layer                            │
│    (Cell Graph, Memory, Routing, Learning, Cache)           │
├─────────────────────────────────────────────────────────────┤
│                    Substrate Layer                          │
│  (Storage, Integrity, Revision, GC, Recovery, Conversion)   │
├─────────────────────────────────────────────────────────────┤
│                    Foundation Layer                         │
│         (Types, Error Handling, Hashing)                    │
└─────────────────────────────────────────────────────────────┘
```

## Layer Descriptions

### Foundation Layer
- **Types**: Blake3Hash, CellType, DataType, Compression, MemoryType, etc.
- **Error Handling**: CnwsError enum dengan error codes
- **Hashing**: BLAKE3-256 content addressing

### Substrate Layer
- **Storage Engine**: Tile-based immutable storage
- **Integrity**: BLAKE3-256 verification, quarantine
- **Revision DAG**: Immutable versioning
- **GC**: Mark-and-sweep garbage collection
- **Recovery**: WAL-based crash recovery
- **Conversion**: Streaming import pipeline

### Lattice Layer
- **Runtime**: Cell Graph execution engine
- **Memory**: Hierarchical memory system
- **Routing**: Cell selection dan routing
- **Learning**: Structural learning
- **Cache**: Multi-level cache hierarchy

### Public API Layer
- **StorageApi**: Operasi tile dasar
- **ConversionApi**: Import model eksternal
- **RuntimeApi**: Eksekusi Cell Graph
- **RevisionApi**: Manajemen revisi
- **MemoryApi**: Operasi memory
- **AdminApi**: GC, recovery, verifikasi

## Data Flow

```
Input Model → Conversion Pipeline → Tiles → Store
                                    ↓
                            Revision DAG
                                    ↓
                            Cell Graph → Runtime
                                    ↓
                            Memory System
                                    ↓
                            Cache Hierarchy
```

## Key Invariants

1. **Content Addressing**: Semua identitas adalah BLAKE3-256 hash
2. **Immutability**: Data tidak bisa diubah setelah ditulis
3. **Determinism**: Hash yang sama untuk data yang sama
4. **Streaming**: Support bounded-memory processing
5. **Zero Coupling**: Runtime independen dari format eksternal
