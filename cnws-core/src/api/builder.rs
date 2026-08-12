//! CNWS System Builder - unified interface for wiring all components
//!
//! Provides a builder pattern to construct a fully-configured CNWS system
//! with all layers (Substrate + Lattice) properly wired together.

use crate::error::Result;
use crate::lattice::cache::CacheManager;
use crate::lattice::memory::MemorySystem;
use crate::lattice::prefetch::PrefetchEngine;
use crate::lattice::routing::{RoutingEngine, RoutingPolicy};
use crate::lattice::runtime::ExecutionEngine;
use crate::substrate::conversion::ConversionPipeline;
use crate::substrate::gc::GarbageCollector;
use crate::substrate::recovery::RecoveryManager;
use crate::substrate::revision::RevisionManager;
use crate::substrate::storage::{StorageEngine, StoreConfig};
use crate::types::{Compression, ComputeBudget};
use std::path::PathBuf;
use std::sync::Arc;

/// Complete CNWS system - all components wired together
pub struct CnwsSystem {
    pub store: Arc<StorageEngine>,
    pub revision_manager: Arc<RevisionManager>,
    pub recovery_manager: Arc<RecoveryManager>,
    pub gc: Arc<GarbageCollector>,
    pub cache: Arc<CacheManager>,
    pub memory: Arc<MemorySystem>,
    pub routing: Arc<RoutingEngine>,
    pub prefetch: Arc<PrefetchEngine>,
    pub execution_engine: Arc<ExecutionEngine>,
    pub conversion_pipeline: ConversionPipeline,
}

/// Builder for constructing a CNWS system
pub struct CnwsBuilder {
    store_path: PathBuf,
    compression: Compression,
    segment_size: u64,
    enable_wal: bool,
    cache_l0: usize,
    cache_l1: usize,
    cache_l2: usize,
    cache_l3: usize,
    routing_policy: RoutingPolicy,
    budget: ComputeBudget,
    prefetch_max_concurrent: usize,
    prefetch_max_buffer: u64,
}

impl CnwsBuilder {
    /// Create a new builder with default settings
    pub fn new(store_path: impl Into<PathBuf>) -> Self {
        Self {
            store_path: store_path.into(),
            compression: Compression::Zstd,
            segment_size: 1024 * 1024 * 1024, // 1 GB
            enable_wal: true,
            cache_l0: 256 * 1024 * 1024,    // 256 MB
            cache_l1: 2 * 1024 * 1024 * 1024,  // 2 GB
            cache_l2: 16 * 1024 * 1024 * 1024, // 16 GB
            cache_l3: 128 * 1024 * 1024 * 1024, // 128 GB
            routing_policy: RoutingPolicy::Auto,
            budget: ComputeBudget::default(),
            prefetch_max_concurrent: 16,
            prefetch_max_buffer: 512 * 1024 * 1024, // 512 MB
        }
    }

    /// Set compression algorithm
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Set segment size
    pub fn with_segment_size(mut self, size: u64) -> Self {
        self.segment_size = size;
        self
    }

    /// Enable/disable WAL
    pub fn with_wal(mut self, enable: bool) -> Self {
        self.enable_wal = enable;
        self
    }

    /// Set cache capacities
    pub fn with_cache_sizes(mut self, l0: usize, l1: usize, l2: usize, l3: usize) -> Self {
        self.cache_l0 = l0;
        self.cache_l1 = l1;
        self.cache_l2 = l2;
        self.cache_l3 = l3;
        self
    }

    /// Set routing policy
    pub fn with_routing_policy(mut self, policy: RoutingPolicy) -> Self {
        self.routing_policy = policy;
        self
    }

    /// Set compute budget
    pub fn with_budget(mut self, budget: ComputeBudget) -> Self {
        self.budget = budget;
        self
    }

    /// Set prefetch engine settings
    pub fn with_prefetch_settings(mut self, max_concurrent: usize, max_buffer: u64) -> Self {
        self.prefetch_max_concurrent = max_concurrent;
        self.prefetch_max_buffer = max_buffer;
        self
    }

    /// Build the CNWS system (creates store if it doesn't exist)
    pub fn build(self) -> Result<CnwsSystem> {
        // ── Substrate Layer ─────────────────────────────────────────────────

        // Storage
        let config = StoreConfig {
            path: self.store_path.clone(),
            segment_size: self.segment_size,
            compression: self.compression,
            enable_wal: self.enable_wal,
            wal_path: Some(self.store_path.join("wal.log")),
        };

        let store = if self.store_path.exists() {
            Arc::new(StorageEngine::open(config)?)
        } else {
            Arc::new(StorageEngine::create_store(config)?)
        };

        // Revision manager
        let revision_manager = Arc::new(RevisionManager::new(Arc::clone(&store)));

        // Recovery manager
        let wal_path = self.store_path.join("wal.log");
        let recovery_manager = Arc::new(RecoveryManager::new(
            Arc::clone(&store),
            wal_path,
        ));

        // Garbage collector
        let gc = Arc::new(GarbageCollector::new(
            Arc::clone(&store),
            Arc::clone(&revision_manager),
        ));

        // ── Lattice Layer ───────────────────────────────────────────────────

        // Cache
        let cache = Arc::new(CacheManager::with_capacities(
            self.cache_l0,
            self.cache_l1,
            self.cache_l2,
            self.cache_l3,
        ));

        // Memory
        let memory = Arc::new(MemorySystem::new(Arc::clone(&store), None));

        // Routing
        let routing = Arc::new(RoutingEngine::new(self.routing_policy));

        // Prefetch
        let prefetch = Arc::new(PrefetchEngine::with_settings(
            Arc::clone(&cache),
            self.prefetch_max_concurrent,
            self.prefetch_max_buffer,
        ));

        // Runtime (uses a default MockResolver - user should replace with real resolver)
        let resolver = Arc::new(crate::lattice::runtime::MockResolver::new());
        let execution_engine = Arc::new(ExecutionEngine::new(
            Arc::clone(&store),
            resolver,
            Arc::clone(&cache),
            Arc::clone(&memory),
            Arc::clone(&routing),
            self.budget,
        ));

        // Conversion pipeline
        let conversion_pipeline = ConversionPipeline::new(Arc::clone(&store))
            .with_compression(self.compression);

        Ok(CnwsSystem {
            store,
            revision_manager,
            recovery_manager,
            gc,
            cache,
            memory,
            routing,
            prefetch,
            execution_engine,
            conversion_pipeline,
        })
    }
}

impl CnwsSystem {
    /// Create a builder for this system's store path
    pub fn builder(store_path: impl Into<PathBuf>) -> CnwsBuilder {
        CnwsBuilder::new(store_path)
    }

    /// Get store statistics
    pub fn store_stats(&self) -> crate::substrate::storage::StoreStats {
        self.store.stats()
    }

    /// Run integrity verification
    pub fn verify_integrity(&self) -> Result<Vec<crate::substrate::integrity::VerificationResult>> {
        use crate::substrate::integrity::IntegrityVerifier;
        let verifier = IntegrityVerifier::new(Arc::clone(&self.store));
        verifier.verify_all()
    }

    /// Run garbage collection
    pub fn run_gc(&self, dry_run: bool) -> Result<crate::substrate::gc::GcReport> {
        self.gc.run(dry_run)
    }

    /// Run recovery check
    pub fn check_recovery(&self) -> Result<crate::substrate::recovery::RecoveryAction> {
        self.recovery_manager.check()
    }

    /// Perform recovery
    pub fn recover(&self) -> Result<crate::substrate::recovery::RecoveryReport> {
        self.recovery_manager.recover()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Blake3Hash;
    use tempfile::tempdir;

    #[test]
    fn test_builder_default() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .build()
            .unwrap();

        let stats = system.store_stats();
        assert_eq!(stats.total_tiles, 0);
    }

    #[test]
    fn test_builder_with_compression() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .with_compression(Compression::Lz4)
            .with_wal(false)
            .build()
            .unwrap();

        let stats = system.store_stats();
        assert_eq!(stats.total_tiles, 0);
    }

    #[test]
    fn test_builder_with_cache() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .with_cache_sizes(
                64 * 1024 * 1024,   // 64 MB
                512 * 1024 * 1024,  // 512 MB
                4 * 1024 * 1024 * 1024,  // 4 GB
                32 * 1024 * 1024 * 1024, // 32 GB
            )
            .build()
            .unwrap();

        let stats = system.cache.statistics();
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_system_verify_integrity() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .build()
            .unwrap();

        let results = system.verify_integrity().unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_system_gc() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .build()
            .unwrap();

        let report = system.run_gc(true).unwrap();
        assert_eq!(report.freed_tiles, 0);
    }

    #[test]
    fn test_system_conversion_pipeline() {
        let dir = tempdir().unwrap();
        let system = CnwsBuilder::new(dir.path().join("test.cd"))
            .build()
            .unwrap();

        let data = b"test tensor";
        let hash = system.conversion_pipeline
            .convert_tensor("model.weight", data, crate::types::DataType::F32, &[10])
            .unwrap();
        assert!(hash != Blake3Hash::default());
    }
}
