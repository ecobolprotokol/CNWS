//! Store integration tests

use cnws_core::{
    api::storage::StorageApi,
    substrate::storage::{StorageEngine, StoreConfig},
    types::{Blake3Hash, Compression},
};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_store_create_and_open() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.cd");

    // Create store
    let api = StorageApi::create(&path, Compression::Zstd).unwrap();

    // Open store
    let api2 = StorageApi::open(&path).unwrap();

    // Verify stats
    let stats = api2.stats();
    assert_eq!(stats.total_tiles, 0);
}

#[test]
fn test_store_write_read_tile() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();

    // Write tile
    let data = b"Hello, CNWS!";
    let hash = engine.write_tile(data, Compression::None).unwrap();

    // Read tile
    let read_data = engine.read_tile(&hash).unwrap();
    assert_eq!(data, read_data.as_slice());

    // Verify exists
    assert!(engine.has_tile(&hash));
}

#[test]
fn test_store_delete_tile() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();

    let data = b"test data";
    let hash = engine.write_tile(data, Compression::None).unwrap();
    assert!(engine.has_tile(&hash));

    engine.delete_tile(&hash).unwrap();
    assert!(!engine.has_tile(&hash));
}

#[test]
fn test_store_list_tiles() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();

    let data1 = b"data1";
    let data2 = b"data2";
    let hash1 = engine.write_tile(data1, Compression::None).unwrap();
    let hash2 = engine.write_tile(data2, Compression::None).unwrap();

    let tiles = engine.list_tiles();
    assert_eq!(tiles.len(), 2);
    assert!(tiles.contains(&hash1));
    assert!(tiles.contains(&hash2));
}

#[test]
fn test_store_compression() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();

    // Compressible data
    let data = vec![0u8; 10000];
    let hash = engine.write_tile(&data, Compression::Zstd).unwrap();
    let read_data = engine.read_tile(&hash).unwrap();
    assert_eq!(data, read_data);
}
