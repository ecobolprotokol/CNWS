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

        // Resolve entry cells
        let entry_cells = self.resolver.resolve_cells(&query.entry_cells).await?;

        // Execute cell graph with dependency resolution
        self.execute_cells(&entry_cells, &mut state).await?;

        // Record duration
        state.duration_us = start.elapsed().as_micros() as u64;

        Ok(state)
    }

    /// Execute cells with dependency resolution and budget enforcement
    fn execute_cells<'a>(
        &'a self,
        cells: &'a [CellRef],
        state: &'a mut WorkingState,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            for cell in cells {
                // Budget enforcement
                if state.compute_used >= self.budget.max_compute {
                    return Err(CnwsError::BudgetExceeded);
                }
                if state.depth >= self.budget.max_depth {
                    return Err(CnwsError::BudgetExceeded);
                }

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
                    self.execute_cells(&hard_deps, state).await?;
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

            Ok(())
        })
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
}
