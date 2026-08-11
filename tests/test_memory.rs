//! Memory integration tests

use cnws_core::{
    api::memory::MemoryApi,
    lattice::memory::MemorySystem,
    substrate::storage::{StorageEngine, StoreConfig},
    types::MemoryType,
};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_memory_write_read() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let system = Arc::new(MemorySystem::new(engine, None));
    let api = MemoryApi::new(system);

    let id = api.write(
        MemoryType::Episodic,
        b"key".to_vec(),
        b"value".to_vec(),
        vec!["tag1".to_string()],
    ).unwrap();

    let entry = api.read(&id).unwrap();
    assert_eq!(entry.key, b"key");
    assert_eq!(entry.value, b"value");
    assert_eq!(entry.memory_type, MemoryType::Episodic);
}

#[test]
fn test_memory_search() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let system = Arc::new(MemorySystem::new(engine, None));
    let api = MemoryApi::new(system);

    api.write(MemoryType::Semantic, b"key1".to_vec(), b"value1".to_vec(), vec![]).unwrap();
    api.write(MemoryType::Semantic, b"key2".to_vec(), b"value2".to_vec(), vec![]).unwrap();

    let results = api.search("key1", None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, b"key1");
}

#[test]
fn test_memory_delete() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let system = Arc::new(MemorySystem::new(engine, None));
    let api = MemoryApi::new(system);

    let id = api.write(MemoryType::Working, b"key".to_vec(), b"value".to_vec(), vec![]).unwrap();
    assert!(api.read(&id).is_ok());

    api.delete(&id).unwrap();
    assert!(api.read(&id).is_err());
}
