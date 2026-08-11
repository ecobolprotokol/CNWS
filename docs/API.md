# CNWS API Reference

## Overview

CNWS menyediakan beberapa API publik untuk berinteraksi dengan sistem:

- `StorageApi` - Operasi storage dasar
- `ConversionApi` - Import model eksternal
- `RuntimeApi` - Eksekusi Cell Graph
- `RevisionApi` - Manajemen revisi
- `MemoryApi` - Operasi memory
- `AdminApi` - Operasi administratif

## StorageApi

Operasi dasar untuk .cd store.

```rust
use cnws_core::api::storage::StorageApi;

// Create store
let api = StorageApi::create("./store", Compression::Zstd)?;

// Open store
let api = StorageApi::open("./store")?;

// Write tile
let hash = api.write_tile(&data)?;

// Read tile
let data = api.read_tile(&hash)?;

// Check existence
assert!(api.has_tile(&hash));

// Delete tile
api.delete_tile(&hash)?;

// List tiles
let tiles = api.list_tiles();

// Get stats
let stats = api.stats();

// Verify integrity
let results = api.verify()?;
```

## ConversionApi

Import model dari format eksternal.

```rust
use cnws_core::api::conversion::ConversionApi;
use cnws_core::substrate::storage::StorageEngine;
use std::sync::Arc;

let engine = Arc::new(StorageEngine::open(config)?);
let pipeline = ConversionPipeline::new(engine);
let api = ConversionApi::new(pipeline);

// Import Safetensors
let report = api.import_safetensors("./model.safetensors")?;

// Import GGUF
let report = api.import_gguf("./model.gguf")?;

// Import PyTorch
let report = api.import_pytorch("./model.pt")?;
```

## RuntimeApi

Eksekusi Cell Graph.

```rust
use cnws_core::api::runtime::{RuntimeApi, QueryBuilder};

let api = RuntimeApi::new(exec_engine);

// Build query
let query = QueryBuilder::new()
    .add_entry_cell(cell_hash)
    .with_max_depth(100)
    .with_max_compute(1_000_000)
    .build();

// Execute
let state = api.execute(&query).await?;
```

## RevisionApi

Manajemen revisi.

```rust
use cnws_core::api::revision::RevisionApi;

let api = RevisionApi::new(manager);

// Commit revision
let id = api.commit(
    Some(parent_id),
    changed_cells,
    changed_tiles,
    metadata
)?;

// Get revision
let revision = api.get(&id);

// Get head
let head = api.head();

// Get ancestors
let ancestors = api.ancestors(id);
```

## MemoryApi

Operasi memory.

```rust
use cnws_core::api::memory::MemoryApi;

let api = MemoryApi::new(system);

// Write memory
let id = api.write(
    MemoryType::Episodic,
    b"key".to_vec(),
    b"value".to_vec(),
    vec!["tag".to_string()]
)?;

// Read memory
let entry = api.read(&id)?;

// Search memory
let results = api.search("query", Some(MemoryType::Semantic));

// Delete memory
api.delete(&id)?;
```

## AdminApi

Operasi administratif.

```rust
use cnws_core::api::admin::AdminApi;

let api = AdminApi::new(store, recovery, gc);

// Run GC
let report = api.gc(false)?;  // false = not dry run

// Recover store
let report = api.recover()?;

// Verify integrity
let results = api.verify()?;

// Get quarantine
let quarantined = api.quarantined_tiles();
```
