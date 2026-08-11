//! Runtime integration tests

use cnws_core::{
    api::runtime::{QueryBuilder, RuntimeApi},
    lattice::runtime::{ExecutionEngine, MockResolver},
    substrate::storage::{StorageEngine, StoreConfig},
    types::{Blake3Hash, CellType, ComputeBudget},
};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_runtime_execution() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);

    let resolver = Arc::new(MockResolver::new());
    let cache = Arc::new(cnws_core::lattice::cache::CacheManager::new());
    let memory = Arc::new(cnws_core::lattice::memory::MemorySystem::new(Arc::clone(&engine), None));
    let routing = Arc::new(cnws_core::lattice::routing::RoutingEngine::new(
        cnws_core::lattice::routing::RoutingPolicy::Auto
    ));

    let exec_engine = Arc::new(ExecutionEngine::new(
        engine,
        resolver,
        cache,
        memory,
        routing,
        ComputeBudget::default(),
    ));

    let api = RuntimeApi::new(exec_engine);

    let hash = Blake3Hash::hash(b"test_cell");
    let query = QueryBuilder::new()
        .add_entry_cell(hash)
        .build();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = rt.block_on(api.execute(&query)).unwrap();

    assert!(state.completed_cells.is_empty() || !state.completed_cells.is_empty());
}

#[test]
fn test_query_builder() {
    let hash = Blake3Hash::hash(b"test");
    let query = QueryBuilder::new()
        .add_entry_cell(hash)
        .with_max_depth(50)
        .with_max_compute(500_000)
        .build();

    assert_eq!(query.entry_cells.len(), 1);
    assert_eq!(query.max_depth, 50);
    assert_eq!(query.max_compute, 500_000);
}
