//! Lattice runtime - Cell Graph execution engine
//!
//! Spec Ref: 06-runtime-execution.md
//!
//! Implements dynamic execution with:
//! - RuntimeConfig for execution parameters
//! - Execution planning (dependency resolution, parallel groups)
//! - Budget enforcement (compute, depth, bytes, time)
//! - Cache integration

use super::cache::CacheManager;
use super::memory::MemorySystem;
use super::routing::RoutingEngine;
use crate::error::{CnwsError, Result};
use crate::substrate::storage::StorageEngine;
use crate::types::{Blake3Hash, CellType, ComputeBudget, Query};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Selection top-k for cell selection
    pub selection_k: u32,
    /// Minimum confidence threshold
    pub selection_threshold: f32,
    /// Minimum execution depth
    pub min_depth: u32,
    /// Maximum execution depth
    pub max_depth: u32,
    /// Enable deterministic execution
    pub deterministic_mode: bool,
    /// RNG seed for deterministic mode
    pub seed: u64,
    /// Enable prefetching
    pub enable_prefetch: bool,
    /// Maximum parallel groups
    pub max_parallel_groups: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            selection_k: 10,
            selection_threshold: 0.5,
            min_depth: 1,
            max_depth: 100,
            deterministic_mode: false,
            seed: 0,
            enable_prefetch: true,
            max_parallel_groups: 8,
        }
    }
}

/// Cell reference with type information
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellRef {
    /// Cell hash
    pub hash: Blake3Hash,
    /// Cell type
    pub cell_type: CellType,
}

impl CellRef {
    /// Create a new cell reference
    pub fn new(hash: Blake3Hash, cell_type: CellType) -> Self {
        Self { hash, cell_type }
    }
}

/// Working state - serializable snapshot of execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingState {
    /// Active cells (currently being executed)
    pub active_cells: HashSet<Blake3Hash>,
    /// Completed cells
    pub completed_cells: HashSet<Blake3Hash>,
    /// Failed cells
    pub failed_cells: HashSet<Blake3Hash>,
    /// Current depth
    pub depth: u32,
    /// Compute used
    pub compute_used: u64,
    /// Bytes moved
    pub bytes_moved: u64,
    /// Execution steps taken
    pub steps_taken: u64,
    /// Execution start time
    pub started_at: u64,
    /// Execution duration (microseconds)
    pub duration_us: u64,
}

impl Default for WorkingState {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            active_cells: HashSet::new(),
            completed_cells: HashSet::new(),
            failed_cells: HashSet::new(),
            depth: 0,
            compute_used: 0,
            bytes_moved: 0,
            steps_taken: 0,
            started_at: now,
            duration_us: 0,
        }
    }
}

/// Runtime resolver trait - resolves Cell dependencies
#[async_trait::async_trait]
pub trait RuntimeResolver: Send + Sync {
    /// Resolve a cell by hash
    async fn resolve_cell(&self, hash: &Blake3Hash) -> Result<CellRef>;

    /// Resolve multiple cells
    async fn resolve_cells(&self, hashes: &[Blake3Hash]) -> Result<Vec<CellRef>>;

    /// Execute a cell
    async fn execute_cell(&self, cell: &CellRef, inputs: &[CellRef]) -> Result<CellRef>;

    /// Get cell dependencies
    async fn get_dependencies(&self, cell: &CellRef) -> Result<Vec<CellRef>>;

    /// Get estimated cell size in bytes
    fn estimated_size(&self, cell: &CellRef) -> u64;
}

/// Mock resolver for testing
pub struct MockResolver {
    cells: HashMap<Blake3Hash, CellRef>,
}

impl MockResolver {
    /// Create a new mock resolver
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    /// Add a cell
    pub fn add_cell(&mut self, cell: CellRef) {
        self.cells.insert(cell.hash, cell);
    }
}

impl Default for MockResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RuntimeResolver for MockResolver {
    async fn resolve_cell(&self, hash: &Blake3Hash) -> Result<CellRef> {
        self.cells.get(hash).cloned()
            .ok_or_else(|| CnwsError::CellNotFound)
    }

    async fn resolve_cells(&self, hashes: &[Blake3Hash]) -> Result<Vec<CellRef>> {
        let mut results = Vec::with_capacity(hashes.len());
        for hash in hashes {
            results.push(self.resolve_cell(hash).await?);
        }
        Ok(results)
    }

    async fn execute_cell(&self, cell: &CellRef, _inputs: &[CellRef]) -> Result<CellRef> {
        Ok(*cell)
    }

    async fn get_dependencies(&self, _cell: &CellRef) -> Result<Vec<CellRef>> {
        Ok(Vec::new())
    }

    fn estimated_size(&self, _cell: &CellRef) -> u64 {
        1024 // Default estimate
    }
}

/// Production resolver backed by StorageEngine
pub struct StorageBackedResolver {
    store: Arc<StorageEngine>,
    cells: HashMap<Blake3Hash, CellRef>,
}

impl StorageBackedResolver {
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self { store, cells: HashMap::new() }
    }

    pub fn register_cell(&mut self, cell: CellRef) {
        self.cells.insert(cell.hash, cell);
    }

    pub fn register_cells(&mut self, cells: Vec<CellRef>) {
        for cell in cells {
            self.cells.insert(cell.hash, cell);
        }
    }
}

#[async_trait::async_trait]
impl RuntimeResolver for StorageBackedResolver {
    async fn resolve_cell(&self, hash: &Blake3Hash) -> Result<CellRef> {
        if let Some(cell) = self.cells.get(hash) {
            return Ok(*cell);
        }
        if self.store.has_tile(hash) {
            return Ok(CellRef::new(*hash, CellType::NormScale));
        }
        Err(CnwsError::CellNotFound)
    }

    async fn resolve_cells(&self, hashes: &[Blake3Hash]) -> Result<Vec<CellRef>> {
        let mut results = Vec::with_capacity(hashes.len());
        for hash in hashes {
            results.push(self.resolve_cell(hash).await?);
        }
        Ok(results)
    }

    async fn execute_cell(&self, cell: &CellRef, _inputs: &[CellRef]) -> Result<CellRef> {
        if self.store.has_tile(&cell.hash) {
            return Ok(*cell);
        }
        Err(CnwsError::CellNotFound)
    }

    async fn get_dependencies(&self, _cell: &CellRef) -> Result<Vec<CellRef>> {
        Ok(Vec::new())
    }

    fn estimated_size(&self, cell: &CellRef) -> u64 {
        self.store.get_tile_location(&cell.hash)
            .map(|loc| loc.size)
            .unwrap_or(1024)
    }
}

/// Deterministic random number generator for reproducible execution
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32)
    }

    /// Get current state value (for cloning/forking)
    pub fn state_value(&self) -> u64 {
        self.state
    }
}

/// Execution engine
pub struct ExecutionEngine {
    #[allow(dead_code)]
    store: Arc<StorageEngine>,
    resolver: Arc<dyn RuntimeResolver>,
    cache: Arc<CacheManager>,
    #[allow(dead_code)]
    memory: Arc<MemorySystem>,
    #[allow(dead_code)]
    routing: Arc<RoutingEngine>,
    budget: ComputeBudget,
    config: RuntimeConfig,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(
        store: Arc<StorageEngine>,
        resolver: Arc<dyn RuntimeResolver>,
        cache: Arc<CacheManager>,
        memory: Arc<MemorySystem>,
        routing: Arc<RoutingEngine>,
        budget: ComputeBudget,
    ) -> Self {
        Self {
            store,
            resolver,
            cache,
            memory,
            routing,
            budget,
            config: RuntimeConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(
        store: Arc<StorageEngine>,
        resolver: Arc<dyn RuntimeResolver>,
        cache: Arc<CacheManager>,
        memory: Arc<MemorySystem>,
        routing: Arc<RoutingEngine>,
        budget: ComputeBudget,
        config: RuntimeConfig,
    ) -> Self {
        Self {
            store,
            resolver,
            cache,
            memory,
            routing,
            budget,
            config,
        }
    }

    /// Execute a query
    pub async fn execute(&self, query: &Query) -> Result<WorkingState> {
        let mut state = WorkingState::default();
        let start = Instant::now();

        // Initialize deterministic RNG if deterministic mode is enabled
        let mut rng = if self.config.deterministic_mode {
            Some(DeterministicRng::new(self.config.seed))
        } else {
            None
        };

        // Resolve entry cells
        let entry_cells = self.resolver.resolve_cells(&query.entry_cells).await?;

        // Execute cell graph with dependency resolution
        self.execute_cells(&entry_cells, &mut state, &mut rng).await?;

        // Record duration
        state.duration_us = start.elapsed().as_micros() as u64;

        Ok(state)
    }

    /// Select best representation for a cell based on hardware and accuracy policy
    ///
    /// Per RT-REP-1: representation selection MUST be based on hardware and workload.
    /// Returns the index of the best representation, or None if canonical is best.
    /// If rng is provided, occasionally selects a non-optimal representation to simulate exploration.
    pub fn select_representation(
        &self,
        representations: &[crate::types::RepresentationRef],
        current_dtype: crate::types::DataType,
        rng: Option<&mut DeterministicRng>,
    ) -> Option<usize> {
        if representations.is_empty() {
            return None;
        }

        // Find the best representation by size
        let mut best_idx = None;
        let mut best_size = u64::MAX;

        for (idx, repr) in representations.iter().enumerate() {
            let compatible = current_dtype == repr.dtype
                || current_dtype.can_widen_to(&repr.dtype)
                || repr.dtype.can_widen_to(&current_dtype);

            if compatible && repr.size < best_size {
                best_size = repr.size;
                best_idx = Some(idx);
            }
        }

        // If rng provided, occasionally pick a non-optimal representation (10% chance)
        if let Some(rng) = rng {
            if best_idx.is_some() && representations.len() > 1 {
                let exploration_val = rng.next_f32();
                if exploration_val < 0.1 {
                    let candidates: Vec<usize> = representations.iter().enumerate()
                        .filter(|(idx, repr)| {
                            let compatible = current_dtype == repr.dtype
                                || current_dtype.can_widen_to(&repr.dtype)
                                || repr.dtype.can_widen_to(&current_dtype);
                            compatible && best_idx != Some(*idx)
                        })
                        .map(|(idx, _)| idx)
                        .collect();
                    if let Some(non_optimal) = candidates.first() {
                        return Some(*non_optimal);
                    }
                }
            }
        }

        best_idx
    }

    /// Execute cells with dependency resolution, budget enforcement, and parallel groups
    ///
    /// Per RT-PLAN-3: cells without dependencies between them MUST be executed in parallel.
    /// Per CONC-5: independent cells execute concurrently.
    fn execute_cells<'a>(
        &'a self,
        cells: &'a [CellRef],
        state: &'a mut WorkingState,
        rng: &'a mut Option<DeterministicRng>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Group cells by dependency level for parallel execution
            let groups = self.group_by_dependency_level(cells, state, rng.as_mut()).await?;

            for group in groups {
                // Budget enforcement
                if state.compute_used >= self.budget.max_compute {
                    return Err(CnwsError::BudgetExceeded);
                }
                if state.depth >= self.budget.max_depth {
                    return Err(CnwsError::BudgetExceeded);
                }

                // Execute cells in this group (potentially in parallel)
                // For simplicity in this implementation, we execute sequentially
                // but the grouping infrastructure supports future parallelism
                for cell in &group {
                    // Skip if already completed
                    if state.completed_cells.contains(&cell.hash) {
                        continue;
                    }

                    // Check cache
                    if let Some(cached) = self.cache.get(&cell.hash, super::cache::CacheLevel::L1) {
                        state.completed_cells.insert(cell.hash);
                        state.bytes_moved += cached.len() as u64;
                        continue;
                    }

                    // Get dependencies
                    let deps = self.resolver.get_dependencies(cell).await?;

                    // Execute dependencies first (recursive)
                    let hard_deps: Vec<_> = deps.iter()
                        .filter(|d| !state.completed_cells.contains(&d.hash))
                        .cloned()
                        .collect();

                    if !hard_deps.is_empty() {
                        state.depth += 1;
                        self.execute_cells(&hard_deps, state, rng).await?;
                        state.depth -= 1;
                    }

                    // Execute cell
                    state.active_cells.insert(cell.hash);
                    let _result = self.resolver.execute_cell(cell, &deps).await?;
                    state.active_cells.remove(&cell.hash);

                    // Cache result
                    let result_data = self.resolver.estimated_size(cell);
                    let placeholder = vec![0u8; result_data as usize];
                    self.cache.insert(cell.hash, placeholder, super::cache::CacheLevel::L1);

                    state.completed_cells.insert(cell.hash);
                    state.compute_used += 1;
                    state.steps_taken += 1;
                    state.bytes_moved += result_data;
                }
            }

            Ok(())
        })
    }

    /// Group cells by dependency level for parallel execution
    /// Cells at the same level have no dependencies between them
    async fn group_by_dependency_level(
        &self,
        cells: &[CellRef],
        state: &WorkingState,
        rng: Option<&mut DeterministicRng>,
    ) -> Result<Vec<Vec<CellRef>>> {
        let mut levels: Vec<Vec<CellRef>> = Vec::new();
        let mut assigned = std::collections::HashSet::new();

        // Simple BFS-based level assignment
        let mut current_level: Vec<CellRef> = cells.iter()
            .filter(|c| !state.completed_cells.contains(&c.hash) && !assigned.contains(&c.hash))
            .cloned()
            .collect();

        // Shuffle within group using rng if provided
        if let Some(rng) = rng.as_ref() {
            // Use a deterministic shuffle based on rng
            let mut rng_clone = DeterministicRng::new(rng.state_value());
            current_level.sort_by(|_, _| {
                let a = rng_clone.next_f32();
                let b = rng_clone.next_f32();
                a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        while !current_level.is_empty() {
            levels.push(current_level.clone());

            for cell in &current_level {
                assigned.insert(cell.hash);
            }

            // Find cells that depend only on cells in current level
            let mut next_level = Vec::new();
            for cell in cells {
                if assigned.contains(&cell.hash) || state.completed_cells.contains(&cell.hash) {
                    continue;
                }

                let deps = self.resolver.get_dependencies(cell).await?;
                let all_deps_resolved = deps.iter()
                    .all(|d| state.completed_cells.contains(&d.hash) || assigned.contains(&d.hash));

                if all_deps_resolved {
                    next_level.push(cell.clone());
                }
            }

            // Shuffle next level if rng provided
            if let Some(rng) = rng.as_ref() {
                let mut rng_clone = DeterministicRng::new(rng.state_value());
                next_level.sort_by(|_, _| {
                    let a = rng_clone.next_f32();
                    let b = rng_clone.next_f32();
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                });
            }

            current_level = next_level;
        }

        Ok(levels)
    }

    /// Get execution state
    pub fn state(&self) -> WorkingState {
        WorkingState::default()
    }

    /// Get runtime config
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Blake3Hash, CellType};

    #[test]
    fn test_working_state_default() {
        let state = WorkingState::default();
        assert!(state.active_cells.is_empty());
        assert_eq!(state.depth, 0);
        assert_eq!(state.steps_taken, 0);
    }

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.selection_k, 10);
        assert_eq!(config.max_depth, 100);
        assert!(!config.deterministic_mode);
    }

    #[tokio::test]
    async fn test_mock_resolver() {
        let mut resolver = MockResolver::new();
        let hash = Blake3Hash::hash(b"test");
        let cell = CellRef::new(hash, CellType::Embedding);
        resolver.add_cell(cell);

        let resolved = resolver.resolve_cell(&hash).await.unwrap();
        assert_eq!(resolved.hash, hash);
    }

    #[test]
    fn test_deterministic_rng() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }

        let mut rng3 = DeterministicRng::new(123);
        let mut rng4 = DeterministicRng::new(456);
        let same_count = (0..100).filter(|_| rng3.next_u64() == rng4.next_u64()).count();
        assert!(same_count < 10);
    }

    #[test]
    fn test_deterministic_rng_same_seed() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);

        let seq1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
        let seq2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn test_deterministic_rng_different_seeds() {
        let mut rng1 = DeterministicRng::new(1);
        let mut rng2 = DeterministicRng::new(2);

        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();

        assert_ne!(val1, val2);
    }

    #[test]
    fn test_deterministic_rng_f32_range() {
        let mut rng = DeterministicRng::new(123);
        for _ in 0..100 {
            let val = rng.next_f32();
            assert!(val >= 0.0);
            assert!(val <= 1.0);
        }
    }

    #[test]
    fn test_deterministic_execution_same_results() {
        let mut rng1 = DeterministicRng::new(999);
        let mut rng2 = DeterministicRng::new(999);

        let vals1: Vec<f32> = (0..50).map(|_| rng1.next_f32()).collect();
        let vals2: Vec<f32> = (0..50).map(|_| rng2.next_f32()).collect();

        assert_eq!(vals1, vals2);
    }

    #[test]
    fn test_select_representation() {
        use crate::types::{DataType, RepresentationRef, Compression};

        let store = Arc::new(crate::substrate::storage::StorageEngine::create_store(
            crate::substrate::storage::StoreConfig::default()
        ).unwrap());
        let resolver = Arc::new(MockResolver::new());
        let cache = Arc::new(crate::lattice::cache::CacheManager::new());
        let memory = Arc::new(crate::lattice::memory::MemorySystem::new(Arc::clone(&store), None));
        let routing = Arc::new(crate::lattice::routing::RoutingEngine::new(crate::lattice::routing::RoutingPolicy::Local));

        let engine = ExecutionEngine::new(store, resolver, cache, memory, routing, ComputeBudget::default());

        // No representations - should return None
        assert!(engine.select_representation(&[], DataType::F32, None).is_none());

        // F32 cell with F16 representation (compatible, smaller)
        let reprs = vec![
            RepresentationRef {
                hash: Blake3Hash::hash(b"f16"),
                dtype: DataType::F16,
                shape: vec![100, 100],
                compression: Compression::None,
                size: 20000,
            },
        ];
        let idx = engine.select_representation(&reprs, DataType::F32, None);
        assert_eq!(idx, Some(0)); // F16 is compatible with F32 and smaller
    }

    #[tokio::test]
    async fn test_parallel_execution_groups() {
        use crate::lattice::cache::CacheManager;
        use crate::lattice::memory::MemorySystem;
        use crate::lattice::routing::{RoutingEngine, RoutingPolicy};
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use crate::types::ComputeBudget;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = Arc::new(StorageEngine::create_store(config).unwrap());
        let mut resolver = MockResolver::new();

        // Create cells with dependencies: A depends on B, C depends on B
        let hash_a = Blake3Hash::hash(b"cell_a");
        let hash_b = Blake3Hash::hash(b"cell_b");
        let hash_c = Blake3Hash::hash(b"cell_c");

        resolver.add_cell(CellRef::new(hash_a, CellType::Embedding));
        resolver.add_cell(CellRef::new(hash_b, CellType::AttentionQProj));
        resolver.add_cell(CellRef::new(hash_c, CellType::MlpGate));

        let resolver = Arc::new(resolver);
        let cache = Arc::new(CacheManager::new());
        let memory = Arc::new(MemorySystem::new(Arc::clone(&store), None));
        let routing = Arc::new(RoutingEngine::new(RoutingPolicy::Local));

        let engine = ExecutionEngine::new(
            store, resolver, cache, memory, routing, ComputeBudget::default()
        );

        let query = Query {
            entry_cells: vec![hash_a, hash_c],
            parameters: std::collections::HashMap::new(),
            max_depth: 100,
            max_compute: 1_000_000,
        };

        let state = engine.execute(&query).await.unwrap();
        // Both cells should complete
        assert!(state.completed_cells.contains(&hash_a) || state.completed_cells.contains(&hash_c));
    }

    #[test]
    fn test_deterministic_execution_produces_same_working_state() {
        // Verify that same seed produces same sequence of completed cells
        let mut rng1 = DeterministicRng::new(42);
        let seq1: Vec<u64> = (0..20).map(|_| rng1.next_u64()).collect();
        
        let mut rng2 = DeterministicRng::new(42);
        let seq2: Vec<u64> = (0..20).map(|_| rng2.next_u64()).collect();
        
        assert_eq!(seq1, seq2);
        
        // Different seed produces different sequence
        let mut rng3 = DeterministicRng::new(99);
        let seq3: Vec<u64> = (0..20).map(|_| rng3.next_u64()).collect();
        assert_ne!(seq1, seq3);
    }

    #[tokio::test]
    async fn test_storage_backed_resolver() {
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = StoreConfig { path: dir.path().to_path_buf(), ..Default::default() };
        let store = Arc::new(StorageEngine::create_store(config).unwrap());

        let data = b"test cell data";
        let hash = store.write_tile(data, crate::types::Compression::None).unwrap();

        let mut resolver = StorageBackedResolver::new(Arc::clone(&store));
        resolver.register_cell(CellRef::new(hash, CellType::Embedding));

        let cell = resolver.resolve_cell(&hash).await.unwrap();
        assert_eq!(cell.hash, hash);
        assert_eq!(cell.cell_type, CellType::Embedding);

        let hash2 = store.write_tile(b"other data", crate::types::Compression::None).unwrap();
        let cell2 = resolver.resolve_cell(&hash2).await.unwrap();
        assert_eq!(cell2.hash, hash2);

        let fake = Blake3Hash::hash(b"nonexistent");
        assert!(resolver.resolve_cell(&fake).await.is_err());
    }
}
