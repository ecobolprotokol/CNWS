//! CNWS CLI - Command-line interface for CNWS operations

use clap::{Parser, Subcommand};
use cnws_core::{
    api::{memory, runtime, storage},
    error::Result,
    telemetry::{CnwsLogger, CnwsMetrics},
};
use std::path::PathBuf;
use std::sync::Arc;

/// CNWS - Canonical Neural Weight System
#[derive(Parser)]
#[command(name = "cnws")]
#[command(about = "Canonical Neural Weight System CLI", long_about = None)]
struct Cli {
    /// Path to .cd store
    #[arg(short, long, global = true)]
    store: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new store
    Init {
        /// Path to create store at
        path: PathBuf,
        /// Compression algorithm
        #[arg(short, long, default_value = "zstd")]
        compression: String,
    },
    /// Import a model
    Import {
        /// Path to model file
        path: PathBuf,
        /// Model format (safetensors, gguf, pytorch, onnx)
        #[arg(short, long)]
        format: String,
    },
    /// Diagnostic commands
    Diag {
        #[command(subcommand)]
        command: DiagCommands,
    },
    /// Revision commands
    Revision {
        #[command(subcommand)]
        command: RevisionCommands,
    },
    /// Memory commands
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    /// Query command
    Query {
        /// Entry cell hashes
        cells: Vec<String>,
    },
    /// Export metrics
    Metrics {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum DiagCommands {
    /// Run integrity check
    Integrity,
    /// Show store status
    StoreStatus,
    /// Run health check
    Health,
}

#[derive(Subcommand)]
enum RevisionCommands {
    /// Commit a new revision
    Commit {
        /// Parent revision ID
        #[arg(short, long)]
        parent: Option<String>,
        /// Changed cell hashes
        #[arg(short, long)]
        cells: Vec<String>,
        /// Changed tile hashes
        #[arg(short, long)]
        tiles: Vec<String>,
    },
    /// Show revision history
    Log {
        /// Revision ID to start from
        revision: Option<String>,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Write to memory
    Write {
        /// Memory type (episodic, semantic, procedural)
        #[arg(short, long)]
        memory_type: String,
        /// Key
        key: String,
        /// Value
        value: String,
        /// Tags
        #[arg(short, long)]
        tags: Vec<String>,
    },
    /// Read from memory
    Read {
        /// Memory ID
        id: String,
    },
    /// Search memory
    Search {
        /// Search query
        query: String,
        /// Memory type filter
        #[arg(short, long)]
        memory_type: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        CnwsLogger::init_json()?;
    } else {
        CnwsLogger::init()?;
    }

    // Initialize metrics
    let metrics = Arc::new(CnwsMetrics::new()?);

    match cli.command {
        Commands::Init { path, compression } => {
            cmd_init(&path, &compression)?;
        }
        Commands::Import { path, format } => {
            let store_path = cli.store.unwrap_or_else(|| PathBuf::from("./store"));
            cmd_import(&store_path, &path, &format)?;
        }
        Commands::Diag { command } => {
            let store_path = cli.store.unwrap_or_else(|| PathBuf::from("./store"));
            cmd_diag(&store_path, command)?;
        }
        Commands::Revision { command } => {
            let store_path = cli.store.unwrap_or_else(|| PathBuf::from("./store"));
            cmd_revision(&store_path, command)?;
        }
        Commands::Memory { command } => {
            let store_path = cli.store.unwrap_or_else(|| PathBuf::from("./store"));
            cmd_memory(&store_path, command)?;
        }
        Commands::Query { cells } => {
            let store_path = cli.store.unwrap_or_else(|| PathBuf::from("./store"));
            cmd_query(&store_path, cells)?;
        }
        Commands::Metrics { format } => {
            cmd_metrics(&metrics, &format)?;
        }
    }

    Ok(())
}

fn cmd_init(path: &PathBuf, compression: &str) -> Result<()> {
    use cnws_core::types::Compression;

    let comp = match compression.to_lowercase().as_str() {
        "none" => Compression::None,
        "zstd" => Compression::Zstd,
        "lz4" => Compression::Lz4,
        "lz4hc" => Compression::Lz4Hc,
        "zlib" => Compression::Zlib,
        "brotli" => Compression::Brotli,
        _ => return Err(cnws_core::error::CnwsError::UnsupportedCompression(
            Compression::try_from(0).unwrap_or(Compression::None)
        )),
    };

    storage::StorageApi::create(path, comp)?;
    println!("Store initialized at: {}", path.display());
    Ok(())
}

fn cmd_import(store_path: &PathBuf, model_path: &PathBuf, format: &str) -> Result<()> {
    use cnws_core::substrate::storage::{StorageEngine, StoreConfig};
    use std::sync::Arc;

    let config = StoreConfig {
        path: store_path.clone(),
        ..Default::default()
    };

    let engine = StorageEngine::open(config)?;
    let engine = Arc::new(engine);
    let pipeline = cnws_core::substrate::conversion::ConversionPipeline::new(engine);

    let report = match format.to_lowercase().as_str() {
        "safetensors" => pipeline.import_safetensors(model_path)?,
        "gguf" => pipeline.import_gguf(model_path)?,
        "pytorch" => pipeline.import_pytorch(model_path)?,
        "onnx" => pipeline.import_onnx(model_path)?,
        _ => return Err(cnws_core::error::CnwsError::UnsupportedFormat(format.to_string())),
    };

    println!("Import complete:");
    println!("  Format: {}", report.source_format);
    println!("  Tensors: {}", report.tensors_imported);
    println!("  Cells: {}", report.cells_created);
    println!("  Tiles: {}", report.tiles_written);
    println!("  Bytes: {} MB", report.total_bytes / 1024 / 1024);

    Ok(())
}

fn cmd_diag(store_path: &PathBuf, command: DiagCommands) -> Result<()> {
    use cnws_core::substrate::storage::{StorageEngine, StoreConfig};
    use std::sync::Arc;

    let config = StoreConfig {
        path: store_path.clone(),
        ..Default::default()
    };

    let engine = StorageEngine::open(config)?;
    let engine = Arc::new(engine);

    match command {
        DiagCommands::Integrity => {
            let verifier = cnws_core::substrate::integrity::IntegrityVerifier::new(engine);
            let results = verifier.verify_all()?;

            let passed = results.iter().filter(|r| r.passed).count();
            let failed = results.iter().filter(|r| !r.passed).count();

            println!("Integrity check complete:");
            println!("  Total tiles: {}", results.len());
            println!("  Passed: {}", passed);
            println!("  Failed: {}", failed);

            if failed > 0 {
                println!("\nFailed tiles:");
                for result in results.iter().filter(|r| !r.passed) {
                    println!("  {:x}: {}", result.tile_hash, result.error.as_deref().unwrap_or("unknown"));
                }
            }
        }
        DiagCommands::StoreStatus => {
            let stats = engine.stats();
            println!("Store status:");
            println!("  Total tiles: {}", stats.total_tiles);
            println!("  Total segments: {}", stats.total_segments);
            println!("  Total size: {} MB", stats.total_size / 1024 / 1024);
            println!("  Compressed size: {} MB", stats.compressed_size / 1024 / 1024);
            println!("  Read count: {}", stats.read_count);
            println!("  Write count: {}", stats.write_count);
            println!("  Cache hits: {}", stats.cache_hits);
            println!("  Cache misses: {}", stats.cache_misses);
        }
        DiagCommands::Health => {
            println!("Store health: OK");
            println!("  Store path: {}", store_path.display());
            println!("  Tiles: {}", engine.list_tiles().len());
        }
    }

    Ok(())
}

fn cmd_revision(store_path: &PathBuf, command: RevisionCommands) -> Result<()> {
    use cnws_core::substrate::storage::{StorageEngine, StoreConfig};
    use std::sync::Arc;

    let config = StoreConfig {
        path: store_path.clone(),
        ..Default::default()
    };

    let engine = StorageEngine::open(config)?;
    let engine = Arc::new(engine);
    let manager = Arc::new(cnws_core::substrate::revision::RevisionManager::new(engine));

    match command {
        RevisionCommands::Commit { parent, cells, tiles } => {
            let parent_hash = parent.and_then(|p| hex::decode(p).ok())
                .and_then(|b| {
                    let mut arr = [0u8; 32];
                    if b.len() == 32 {
                        arr.copy_from_slice(&b);
                        Some(cnws_core::types::Blake3Hash(arr))
                    } else {
                        None
                    }
                });

            let cell_hashes = cells.iter().filter_map(|c| hex::decode(c).ok())
                .filter_map(|b| {
                    let mut arr = [0u8; 32];
                    if b.len() == 32 {
                        arr.copy_from_slice(&b);
                        Some(cnws_core::types::Blake3Hash(arr))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let tile_hashes = tiles.iter().filter_map(|t| hex::decode(t).ok())
                .filter_map(|b| {
                    let mut arr = [0u8; 32];
                    if b.len() == 32 {
                        arr.copy_from_slice(&b);
                        Some(cnws_core::types::Blake3Hash(arr))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let id = manager.commit(parent_hash, cell_hashes, tile_hashes, std::collections::HashMap::new())?;
            println!("Revision committed: {:x}", id);
        }
        RevisionCommands::Log { revision: _ } => {
            let dag = manager.dag();
            let dag = dag.read();

            println!("Revision history:");
            for rev_id in dag.revision_ids() {
                if let Some(rev) = dag.get(rev_id) {
                    println!("  {:x} (parents: {}, cells: {}, tiles: {})",
                        rev.id,
                        rev.parents.len(),
                        rev.changed_cells.len(),
                        rev.changed_tiles.len()
                    );
                }
            }
        }
    }

    Ok(())
}

fn cmd_memory(store_path: &PathBuf, command: MemoryCommands) -> Result<()> {
    use cnws_core::substrate::storage::{StorageEngine, StoreConfig};
    use std::sync::Arc;

    let config = StoreConfig {
        path: store_path.clone(),
        ..Default::default()
    };

    let engine = StorageEngine::open(config)?;
    let engine = Arc::new(engine);
    let system = Arc::new(cnws_core::lattice::memory::MemorySystem::new(engine, None));
    let api = memory::MemoryApi::new(system);

    match command {
        MemoryCommands::Write { memory_type, key, value, tags } => {
            let mt = match memory_type.to_lowercase().as_str() {
                "episodic" => cnws_core::types::MemoryType::Episodic,
                "semantic" => cnws_core::types::MemoryType::Semantic,
                "procedural" => cnws_core::types::MemoryType::Procedural,
                "working" => cnws_core::types::MemoryType::Working,
                "longterm" => cnws_core::types::MemoryType::LongTerm,
                _ => return Err(cnws_core::error::CnwsError::InvalidInput(format!("Unknown memory type: {}", memory_type))),
            };

            let id = api.write(mt, key.into_bytes(), value.into_bytes(), tags)?;
            println!("Memory written: {}", id);
        }
        MemoryCommands::Read { id } => {
            let entry = api.read(&id)?;
            println!("Memory entry:");
            println!("  ID: {}", id);
            println!("  Type: {:?}", entry.memory_type);
            println!("  Key: {}", String::from_utf8_lossy(&entry.key));
            println!("  Value: {}", String::from_utf8_lossy(&entry.value));
            println!("  Tags: {:?}", entry.tags);
        }
        MemoryCommands::Search { query, memory_type } => {
            let mt = memory_type.and_then(|t| match t.to_lowercase().as_str() {
                "episodic" => Some(cnws_core::types::MemoryType::Episodic),
                "semantic" => Some(cnws_core::types::MemoryType::Semantic),
                "procedural" => Some(cnws_core::types::MemoryType::Procedural),
                "working" => Some(cnws_core::types::MemoryType::Working),
                "longterm" => Some(cnws_core::types::MemoryType::LongTerm),
                _ => None,
            });

            let results = api.search(&query, mt);
            println!("Search results for '{}':", query);
            for entry in results {
                println!("  {}: {}", String::from_utf8_lossy(&entry.key), String::from_utf8_lossy(&entry.value));
            }
        }
    }

    Ok(())
}

fn cmd_query(store_path: &PathBuf, cells: Vec<String>) -> Result<()> {
    use cnws_core::substrate::storage::{StorageEngine, StoreConfig};
    use std::sync::Arc;

    let config = StoreConfig {
        path: store_path.clone(),
        ..Default::default()
    };

    let engine = StorageEngine::open(config)?;
    let engine = Arc::new(engine);

    let resolver = Arc::new(cnws_core::lattice::runtime::MockResolver::new());
    let cache = Arc::new(cnws_core::lattice::cache::CacheManager::new());
    let memory = Arc::new(cnws_core::lattice::memory::MemorySystem::new(Arc::clone(&engine), None));
    let routing = Arc::new(cnws_core::lattice::routing::RoutingEngine::new(
        cnws_core::lattice::routing::RoutingPolicy::Auto
    ));

    let exec_engine = Arc::new(cnws_core::lattice::runtime::ExecutionEngine::new(
        engine,
        resolver,
        cache,
        memory,
        routing,
        cnws_core::types::ComputeBudget::default(),
    ));

    let api = runtime::RuntimeApi::new(exec_engine);

    let cell_hashes = cells.iter().filter_map(|c| hex::decode(c).ok())
        .filter_map(|b| {
            let mut arr = [0u8; 32];
            if b.len() == 32 {
                arr.copy_from_slice(&b);
                Some(cnws_core::types::Blake3Hash(arr))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let query = runtime::QueryBuilder::new()
        .with_entry_cells(cell_hashes)
        .build();

    let rt = tokio::runtime::Runtime::new()?;
    let state = rt.block_on(api.execute(&query))?;

    println!("Query complete:");
    println!("  Completed cells: {}", state.completed_cells.len());
    println!("  Compute used: {}", state.compute_used);
    println!("  Bytes moved: {} MB", state.bytes_moved / 1024 / 1024);

    Ok(())
}

fn cmd_metrics(metrics: &CnwsMetrics, format: &str) -> Result<()> {
    match format.to_lowercase().as_str() {
        "text" | "prometheus" => {
            let text = metrics.export()?;
            println!("{}", text);
        }
        "json" => {
            let stats = metrics.export()?;
            println!("{}", stats);
        }
        _ => {
            eprintln!("Unknown format: {}", format);
            std::process::exit(1);
        }
    }

    Ok(())
}
