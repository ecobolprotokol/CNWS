//! Integration tests for CNWS
//! Tests end-to-end workflows

use cnws_core::{
    api::{self, storage, conversion, memory, revision, runtime},
    error::Result,
    substrate::storage::{StorageEngine, StoreConfig},
    types::{Blake3Hash, CellType, Compression, MemoryType},
};
use std::sync::Arc;
use tempfile::tempdir;

mod test_store;
mod test_conversion;
mod test_revision;
mod test_memory;
mod test_runtime;
mod test_integrity;
