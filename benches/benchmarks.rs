//! CNWS Performance Benchmarks
//! Uses Criterion for statistical analysis

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cnws_core::{
    types::Blake3Hash,
    substrate::storage::{StorageEngine, StoreConfig},
    lattice::{
        cache::CacheManager,
        memory::MemorySystem,
        routing::RoutingEngine,
        learning::LearningEngine,
    },
};
use std::sync::Arc;
use tempfile::tempdir;

// ============================================================================
// BLAKE3-256 Hashing Benchmarks
// ============================================================================

fn bench_blake3_hash(c: &mut Criterion) {
    let data_sizes = vec![64, 1024, 65536, 1048576]; // 64B, 1KB, 64KB, 1MB

    let mut group = c.benchmark_group("blake3_hash");
    for size in data_sizes {
        let data = vec![0u8; size];
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(Blake3Hash::hash(black_box(&data))))
        });
    }
    group.finish();
}

// ============================================================================
// Tile Storage Benchmarks
// ============================================================================

fn bench_tile_write(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let sizes = vec![1024, 65536, 1048576]; // 1KB, 64KB, 1MB

    let mut group = c.benchmark_group("tile_write");
    for size in sizes {
        let data = vec![0u8; size];
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                let _ = engine.write_tile(black_box(&data), cnws_core::types::Compression::None);
            })
        });
    }
    group.finish();
}

fn bench_tile_read(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let data = vec![0u8; 65536]; // 64KB
    let hash = engine.write_tile(&data, cnws_core::types::Compression::None).unwrap();

    c.bench_function("tile_read", |b| {
        b.iter(|| {
            let _ = engine.read_tile(black_box(&hash));
        })
    });
}

// ============================================================================
// Cache Benchmarks
// ============================================================================

fn bench_cache_insert(c: &mut Criterion) {
    let cache = CacheManager::new();
    let sizes = vec![64, 1024, 65536]; // 64B, 1KB, 64KB

    let mut group = c.benchmark_group("cache_insert");
    for size in sizes {
        let data = vec![0u8; size];
        let hash = Blake3Hash::hash(&data);
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| {
                cache.insert(black_box(hash), black_box(data.clone()), cnws_core::lattice::cache::CacheLevel::L1);
            })
        });
    }
    group.finish();
}

fn bench_cache_lookup(c: &mut Criterion) {
    let cache = CacheManager::new();
    let data = vec![0u8; 1024];
    let hash = Blake3Hash::hash(&data);
    cache.insert(hash, data, cnws_core::lattice::cache::CacheLevel::L1);

    c.bench_function("cache_lookup", |b| {
        b.iter(|| {
            let _ = cache.get(black_box(&hash), cnws_core::lattice::cache::CacheLevel::L1);
        })
    });
}

// ============================================================================
// Memory Benchmarks
// ============================================================================

fn bench_memory_write(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let memory = Arc::new(MemorySystem::new(engine, None));

    c.bench_function("memory_write", |b| {
        b.iter(|| {
            let _ = memory.write(
                cnws_core::types::MemoryType::Episodic,
                black_box(b"key".to_vec()),
                black_box(b"value".to_vec()),
                vec![],
            );
        })
    });
}

// ============================================================================
// Revision DAG Benchmarks
// ============================================================================

fn bench_revision_commit(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let manager = Arc::new(cnws_core::substrate::revision::RevisionManager::new(engine));

    c.bench_function("revision_commit", |b| {
        b.iter(|| {
            let _ = manager.commit(None, vec![], vec![], std::collections::HashMap::new());
        })
    });
}

// ============================================================================
// Learning Benchmarks
// ============================================================================

fn bench_learning_discover(c: &mut Criterion) {
    let engine = LearningEngine::new();
    let mut sequences = Vec::new();

    for i in 0..100 {
        let hash = Blake3Hash::hash(&i.to_le_bytes());
        sequences.push(vec![hash]);
    }

    c.bench_function("learning_discover", |b| {
        b.iter(|| {
            let _ = engine.discover_patterns(black_box(&sequences));
        })
    });
}

// ============================================================================
// Routing Benchmarks
// ============================================================================

fn bench_routing_select(c: &mut Criterion) {
    let engine = RoutingEngine::new(cnws_core::lattice::routing::RoutingPolicy::Auto);

    // Register some cells
    for i in 0..100 {
        let hash = Blake3Hash::hash(&i.to_le_bytes());
        engine.register_cell(cnws_core::lattice::routing::CellMetadata::new(
            hash, "tensor", 1024
        ));
    }

    let query = vec![0.1f32; 128];
    let candidates: Vec<Blake3Hash> = (0..100)
        .map(|i| Blake3Hash::hash(&i.to_le_bytes()))
        .collect();

    c.bench_function("routing_select", |b| {
        b.iter(|| {
            let _ = engine.select(black_box(&query), black_box(&candidates), 10);
        })
    });
}

// ============================================================================
// Compression Benchmarks
// ============================================================================

fn bench_compression(c: &mut Criterion) {
    let data = vec![0u8; 1048576]; // 1MB
    let algorithms = vec![
        ("none", cnws_core::types::Compression::None),
        ("zstd", cnws_core::types::Compression::Zstd),
        ("lz4", cnws_core::types::Compression::Lz4),
    ];

    let mut group = c.benchmark_group("compression");
    for (name, algo) in algorithms {
        group.bench_with_input(BenchmarkId::from_parameter(name), &name, |b, _| {
            b.iter(|| {
                let _ = cnws_core::substrate::storage::StorageEngine::compress(
                    black_box(&data),
                    black_box(algo),
                );
            })
        });
    }
    group.finish();
}

// ============================================================================
// Integrity Verification Benchmarks
// ============================================================================

fn bench_integrity_verify(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let verifier = cnws_core::substrate::integrity::IntegrityVerifier::new(engine);

    let data = vec![0u8; 65536]; // 64KB
    let hash = verifier.store.write_tile(&data, cnws_core::types::Compression::None).unwrap();

    c.bench_function("integrity_verify", |b| {
        b.iter(|| {
            let _ = verifier.verify_tile(black_box(&hash));
        })
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    bench_blake3_hash,
    bench_tile_write,
    bench_tile_read,
    bench_cache_insert,
    bench_cache_lookup,
    bench_memory_write,
    bench_revision_commit,
    bench_learning_discover,
    bench_routing_select,
    bench_compression,
    bench_integrity_verify
);

criterion_main!(benches);
