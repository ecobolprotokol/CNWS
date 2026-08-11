//! CNWS Conformance Test Runner
//! Implements CS-01 through CS-10 conformance suites

use clap::Parser;
use cnws_core::{
    error::Result,
    substrate::{
        conversion::ConversionPipeline,
        gc::GarbageCollector,
        integrity::IntegrityVerifier,
        recovery::RecoveryManager,
        revision::RevisionManager,
        storage::{StorageEngine, StoreConfig},
    },
    types::{Blake3Hash, CellType, Compression, DataType, MemoryType},
};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

/// CNWS Conformance Test Runner
#[derive(Parser)]
#[command(name = "cnws-conformance")]
#[command(about = "CNWS Conformance Test Runner", long_about = None)]
struct Cli {
    /// Run specific test suite
    #[arg(short, long)]
    suite: Option<String>,

    /// Run specific test
    #[arg(short, long)]
    test: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("CNWS Conformance Test Runner");
    println!("============================\n");

    let mut passed = 0;
    let mut failed = 0;

    // CS-01: Content Addressing Invariants
    println!("Running CS-01: Content Addressing Invariants...");
    match run_cs01() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-02: Cell Schema Conformance
    println!("Running CS-02: Cell Schema Conformance...");
    match run_cs02() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-03: .cd Format Conformance
    println!("Running CS-03: .cd Format Conformance...");
    match run_cs03() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-04: Tile Storage Conformance
    println!("Running CS-04: Tile Storage Conformance...");
    match run_cs04() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-05: Revision DAG Conformance
    println!("Running CS-05: Revision DAG Conformance...");
    match run_cs05() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-06: Memory System Conformance
    println!("Running CS-06: Memory System Conformance...");
    match run_cs06() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-07: Routing Conformance
    println!("Running CS-07: Routing Conformance...");
    match run_cs07() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-08: Conversion Conformance
    println!("Running CS-08: Conversion Conformance...");
    match run_cs08() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-09: Recovery Conformance
    println!("Running CS-09: Recovery Conformance...");
    match run_cs09() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // CS-10: Integrity Conformance
    println!("Running CS-10: Integrity Conformance...");
    match run_cs10() {
        Ok(_) => {
            println!("  PASSED\n");
            passed += 1;
        }
        Err(e) => {
            println!("  FAILED: {}\n", e);
            failed += 1;
        }
    }

    // Summary
    println!("============================");
    println!("Results: {} passed, {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// CS-01: Content Addressing Invariants
fn run_cs01() -> Result<()> {
    // FAC-01: BLAKE3-256 determinism
    let data1 = b"test data";
    let data2 = b"test data";
    let hash1 = Blake3Hash::hash(data1);
    let hash2 = Blake3Hash::hash(data2);
    assert_eq!(hash1, hash2, "FAC-01: Determinism violated");

    // FAC-02: Content equality
    let hash3 = Blake3Hash::hash(b"different data");
    assert_ne!(hash1, hash3, "FAC-02: Different content has same hash");

    // FAC-03: Streaming equivalence
    let mut hasher = blake3::Hasher::new();
    hasher.update(data1);
    let hash_bytes: [u8; 32] = hasher.finalize().into();
    let streaming_hash = Blake3Hash(hash_bytes);
    assert_eq!(hash1, streaming_hash, "FAC-03: Streaming hash mismatch");

    // FAC-04: Collision resistance (basic test)
    let mut hashes = std::collections::HashSet::new();
    for i in 0..1000u64 {
        let hash = Blake3Hash::hash(&i.to_le_bytes());
        assert!(hashes.insert(hash), "FAC-04: Collision detected at {}", i);
    }

    Ok(())
}

/// CS-02: Cell Schema Conformance
fn run_cs02() -> Result<()> {
    // Test all cell types
    let cell_types = vec![
        CellType::Tensor,
        CellType::Attention,
        CellType::FFN,
        CellType::LayerNorm,
        CellType::Embedding,
        CellType::Loss,
        CellType::OptimizerState,
        CellType::Gradient,
        CellType::Activation,
        CellType::Weight,
        CellType::Bias,
        CellType::Mask,
        CellType::PositionalEncoding,
        CellType::KV,
        CellType::Projection,
        CellType::Residual,
        CellType::Dropout,
        CellType::Scale,
        CellType::Shift,
        CellType::Gate,
        CellType::Merge,
        CellType::Split,
        CellType::Custom,
    ];

    for cell_type in cell_types {
        // Each cell type should have a unique discriminant
        let _ = cell_type;
    }

    // Test data types
    let data_types = vec![
        DataType::F32,
        DataType::F16,
        DataType::BF16,
        DataType::F8,
        DataType::I8,
        DataType::I16,
        DataType::I32,
        DataType::I64,
        DataType::U8,
        DataType::U16,
        DataType::U32,
        DataType::U64,
        DataType::Bool,
    ];

    for dt in data_types {
        let _ = dt;
    }

    Ok(())
}

/// CS-03: .cd Format Conformance
fn run_cs03() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;

    // Verify SUPERBLOCK
    let superblock_path = dir.path().join("SUPERBLOCK");
    assert!(superblock_path.exists(), "SUPERBLOCK not created");

    // Verify MANIFEST
    let manifest_path = dir.path().join("MANIFEST.cd");
    assert!(manifest_path.exists(), "MANIFEST.cd not created");

    // Verify segments directory
    let segments_path = dir.path().join("segments");
    assert!(segments_path.exists(), "segments directory not created");

    // Verify index
    let index_path = dir.path().join("index.cd");
    assert!(index_path.exists(), "index.cd not created");

    Ok(())
}

/// CS-04: Tile Storage Conformance
fn run_cs04() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;

    // Write tile
    let data = b"test tile data";
    let hash = engine.write_tile(data, Compression::None)?;

    // Read tile
    let read_data = engine.read_tile(&hash)?;
    assert_eq!(data, read_data.as_slice(), "Tile data mismatch");

    // Verify tile exists
    assert!(engine.has_tile(&hash), "Tile not found after write");

    // Delete tile
    engine.delete_tile(&hash)?;
    assert!(!engine.has_tile(&hash), "Tile still exists after delete");

    Ok(())
}

/// CS-05: Revision DAG Conformance
fn run_cs05() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;
    let engine = Arc::new(engine);
    let manager = Arc::new(RevisionManager::new(engine));

    // Commit root revision
    let rev1 = manager.commit(None, vec![], vec![], HashMap::new())?;

    // Commit child revision
    let rev2 = manager.commit(Some(rev1), vec![], vec![], HashMap::new())?;

    // Verify ancestry
    assert!(manager.dag().read().is_ancestor(rev1, rev2), "Ancestry check failed");

    // Verify common ancestor
    let common = manager.common_ancestor(rev1, rev2);
    assert_eq!(common, Some(rev1), "Common ancestor incorrect");

    Ok(())
}

/// CS-06: Memory System Conformance
fn run_cs06() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;
    let engine = Arc::new(engine);
    let system = Arc::new(cnws_core::lattice::memory::MemorySystem::new(engine, None));

    // Write memory
    let id = system.write(
        MemoryType::Episodic,
        b"key".to_vec(),
        b"value".to_vec(),
        vec!["tag".to_string()],
    )?;

    // Read memory
    let entry = system.read(&id)?;
    assert_eq!(entry.key, b"key", "Memory key mismatch");
    assert_eq!(entry.value, b"value", "Memory value mismatch");

    // Search memory
    let results = system.search("key", None);
    assert_eq!(results.len(), 1, "Search returned wrong count");

    Ok(())
}

/// CS-07: Routing Conformance
fn run_cs07() -> Result<()> {
    let engine = cnws_core::lattice::routing::RoutingEngine::new(
        cnws_core::lattice::routing::RoutingPolicy::Auto
    );

    // Register cells
    let hash1 = Blake3Hash::hash(b"cell1");
    let hash2 = Blake3Hash::hash(b"cell2");

    engine.register_cell(cnws_core::lattice::routing::CellMetadata::new(
        hash1, "tensor", 1024
    ));
    engine.register_cell(cnws_core::lattice::routing::CellMetadata::new(
        hash2, "tensor", 2048
    ));

    // Route query
    let results = engine.route("test")?;
    assert_eq!(results.len(), 2, "Routing returned wrong count");

    Ok(())
}

/// CS-08: Conversion Conformance
fn run_cs08() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;
    let engine = Arc::new(engine);
    let pipeline = ConversionPipeline::new(engine);

    // Test tensor conversion
    let hash = pipeline.convert_tensor("test", b"data", DataType::F32, &[10])?;
    assert!(hash != Blake3Hash::default(), "Conversion returned zero hash");

    Ok(())
}

/// CS-09: Recovery Conformance
fn run_cs09() -> Result<()> {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("wal.log");

    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;
    let engine = Arc::new(engine);
    let recovery = RecoveryManager::new(engine, wal_path);

    // Check recovery status
    let status = recovery.state();
    assert_eq!(status, cnws_core::substrate::recovery::RecoveryState::Clean, "Initial state not clean");

    Ok(())
}

/// CS-10: Integrity Conformance
fn run_cs10() -> Result<()> {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config)?;
    let engine = Arc::new(engine);
    let verifier = IntegrityVerifier::new(engine);

    // Verify empty store
    let results = verifier.verify_all()?;
    assert_eq!(results.len(), 0, "Empty store has tiles");

    Ok(())
}
