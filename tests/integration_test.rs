//! Integration tests for CNWS
//! Tests end-to-end workflows

use cnws_core::{
    api::builder::CnwsBuilder,
    error::Result,
    types::{
        Blake3Hash, CellType, Compression, DataType, Dependency, MemoryType, Schema,
        DEFAULT_INDEX_DIMENSIONS,
    },
};
use tempfile::tempdir;

#[test]
fn test_end_to_end_store_create_write_read() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd"))
        .with_compression(Compression::Zstd)
        .build()?;

    // Write tiles
    let data1 = b"Hello, CNWS!";
    let hash1 = system.store.write_tile(data1, Compression::Zstd)?;

    let data2 = b"World of neural weights";
    let hash2 = system.store.write_tile(data2, Compression::None)?;

    // Read tiles
    let read1 = system.store.read_tile(&hash1)?;
    let read2 = system.store.read_tile(&hash2)?;

    assert_eq!(data1, read1.as_slice());
    assert_eq!(data2, read2.as_slice());

    // Verify stats
    let stats = system.store_stats();
    assert_eq!(stats.total_tiles, 2);
    assert!(stats.total_size > 0);

    Ok(())
}

#[test]
fn test_end_to_end_deduplication() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd"))
        .with_compression(Compression::None)
        .build()?;

    let data = b"deduplication test data";
    let hash1 = system.store.write_tile(data, Compression::None)?;
    let hash2 = system.store.write_tile(data, Compression::None)?;
    let hash3 = system.store.write_tile(data, Compression::None)?;

    // All should be the same hash
    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);

    // Only one tile should exist
    let stats = system.store_stats();
    assert_eq!(stats.total_tiles, 1);
    assert_eq!(stats.dedup_count, 2);

    Ok(())
}

#[test]
fn test_end_to_end_multi_segment() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd"))
        .with_segment_size(100) // Very small to force multiple segments
        .build()?;

    // Write tiles that exceed segment size
    for i in 0..10u8 {
        let data = vec![i; 40];
        system.store.write_tile(&data, Compression::None)?;
    }

    assert!(system.store.segment_count() > 1);

    // All tiles should still be readable
    let tiles = system.store.list_tiles();
    assert_eq!(tiles.len(), 10);

    Ok(())
}

#[test]
fn test_end_to_end_integrity_verification() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Write some tiles
    for i in 0..5u8 {
        let data = vec![i; 100];
        system.store.write_tile(&data, Compression::Zstd)?;
    }

    // Verify integrity
    let results = system.verify_integrity()?;
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.passed));

    Ok(())
}

#[test]
fn test_end_to_end_revision_management() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Write a tile
    let data = b"revision test";
    let tile_hash = system.store.write_tile(data, Compression::None)?;

    // Commit a revision
    let rev1 = system.revision_manager.commit(
        None,
        vec![],
        vec![tile_hash],
        std::collections::HashMap::new(),
    )?;

    // Verify revision exists
    assert!(system.revision_manager.exists(&rev1));
    assert_eq!(system.revision_manager.head(), Some(rev1));

    // Commit another revision
    let rev2 = system.revision_manager.commit(
        Some(rev1),
        vec![],
        vec![],
        std::collections::HashMap::new(),
    )?;

    // Verify ancestry
    assert!(system.revision_manager.dag().read().is_ancestor(rev1, rev2));

    Ok(())
}

#[test]
fn test_end_to_end_memory_system() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Write memories
    let id1 = system.memory.write(
        MemoryType::Episodic,
        b"key1".to_vec(),
        b"value1".to_vec(),
        vec!["tag1".to_string()],
    )?;

    let id2 = system.memory.write(
        MemoryType::Semantic,
        b"key2".to_vec(),
        b"value2".to_vec(),
        vec!["tag2".to_string()],
    )?;

    // Read memory
    let entry1 = system.memory.read(&id1)?;
    assert_eq!(entry1.key, b"key1");
    assert_eq!(entry1.value, b"value1");
    assert_eq!(entry1.lifecycle, cnws_core::lattice::memory::MemoryLifecycle::Active);

    // Search
    let results = system.memory.search("key1", None);
    assert_eq!(results.len(), 1);

    // Count
    assert_eq!(system.memory.count(), 2);

    // Add association
    system.memory.add_association(&id1, &id2)?;
    let assocs = system.memory.get_associations(&id1)?;
    assert_eq!(assocs.len(), 1);

    Ok(())
}

#[test]
fn test_end_to_end_routing() -> Result<()> {
    use cnws_core::lattice::routing::{CellMetadata, RoutingPolicy};

    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd"))
        .with_routing_policy(RoutingPolicy::Auto)
        .build()?;

    // Register cells
    let hash1 = Blake3Hash::hash(b"cell1");
    let hash2 = Blake3Hash::hash(b"cell2");

    system.routing.register_cell(CellMetadata::new(hash1, "tensor", 1024));
    system.routing.register_cell(CellMetadata::new(hash2, "tensor", 2048));

    // Route
    let results = system.routing.route("test")?;
    assert_eq!(results.len(), 2);

    // Check stats
    let stats = system.routing.statistics();
    assert!(stats.total_queries > 0);

    Ok(())
}

#[test]
fn test_end_to_end_cache() -> Result<()> {
    use cnws_core::lattice::cache::CacheLevel;

    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Insert into cache
    let hash = Blake3Hash::hash(b"cache_test");
    system.cache.insert(hash, vec![1, 2, 3, 4], CacheLevel::L1);

    // Get from cache
    let data = system.cache.get(&hash, CacheLevel::L1);
    assert!(data.is_some());
    assert_eq!(data.unwrap(), vec![1, 2, 3, 4]);

    // Get from any level
    let data = system.cache.get_any(&hash);
    assert!(data.is_some());

    // Stats
    let stats = system.cache.statistics();
    assert!(stats.hits > 0);

    Ok(())
}

#[test]
fn test_end_to_end_learning_engine() -> Result<()> {
    use cnws_core::lattice::learning::{LearningEngine, LearningUpdate, LearningUpdateType};

    let engine = LearningEngine::new();
    let hash1 = Blake3Hash::hash(b"cell_a");
    let hash2 = Blake3Hash::hash(b"cell_b");

    // Apply update
    let update = LearningUpdate::new(
        LearningUpdateType::NewPattern,
        vec![hash1],
        vec![],
    );
    engine.apply_update(update).unwrap();

    // Discover patterns (need 2+ occurrences of same sequence)
    let sequences = vec![
        vec![hash1, hash2],
        vec![hash1, hash2],
    ];
    let patterns = engine.discover_patterns(&sequences).unwrap();

    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].frequency, 2);

    Ok(())
}

#[test]
fn test_end_to_end_prefetch_engine() -> Result<()> {
    use cnws_core::lattice::prefetch::PrefetchEngine;
    use cnws_core::types::Dependency;
    use std::collections::HashMap;

    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    let prefetch = PrefetchEngine::new(system.cache.clone());

    let cell1 = Blake3Hash::hash(b"cell1");
    let cell2 = Blake3Hash::hash(b"cell2");

    let mut dep_graph = HashMap::new();
    dep_graph.insert(cell1, vec![Dependency::data(cell2)]);

    let mut sizes = HashMap::new();
    sizes.insert(cell1, 1024);
    sizes.insert(cell2, 2048);

    let plan = prefetch.plan_prefetch(&[cell1], &dep_graph, &sizes)?;
    assert!(plan.requests.len() >= 1);
    assert!(plan.total_bytes > 0);

    Ok(())
}

#[test]
fn test_end_to_end_gc() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Write some tiles
    for i in 0..5u8 {
        let data = vec![i; 100];
        system.store.write_tile(&data, Compression::None)?;
    }

    // Run GC (dry run)
    let report = system.run_gc(true)?;
    assert_eq!(report.freed_tiles, 0); // All tiles in registry, nothing to free

    Ok(())
}

#[test]
fn test_end_to_end_conversion() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Convert a tensor
    let data = vec![0f32; 100];
    let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
    let hash = system.conversion_pipeline.convert_tensor(
        "model.layer.0.weight",
        &bytes,
        DataType::F32,
        &[10, 10],
    )?;

    assert!(hash != Blake3Hash::default());

    // The tile should be readable
    let read_data = system.store.read_tile(&hash)?;
    assert_eq!(read_data.len(), bytes.len());

    Ok(())
}

#[test]
fn test_end_to_end_builder_configurations() -> Result<()> {
    let dir = tempdir()?;

    // Test various configurations
    let _system = CnwsBuilder::new(dir.path().join("test1.cd"))
        .with_compression(Compression::Lz4)
        .with_wal(false)
        .with_cache_sizes(
            64 * 1024 * 1024,
            512 * 1024 * 1024,
            4 * 1024 * 1024 * 1024,
            32 * 1024 * 1024 * 1024,
        )
        .with_segment_size(256 * 1024 * 1024)
        .with_prefetch_settings(32, 1024 * 1024 * 1024)
        .build()?;

    Ok(())
}

#[test]
fn test_end_to_end_recovery() -> Result<()> {
    let dir = tempdir()?;
    let system = CnwsBuilder::new(dir.path().join("test.cd")).build()?;

    // Check recovery (should be clean)
    let action = system.check_recovery()?;
    assert_eq!(action, cnws_core::substrate::recovery::RecoveryAction::None);

    Ok(())
}
