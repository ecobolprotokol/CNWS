//! Lattice runtime - Cell Graph execution engine
//! Implements dynamic execution with dependency resolution

use super::cache::CacheManager;
use super::memory::MemorySystem;
use super::routing::RoutingEngine;
use crate::error::{CnwsError, Result};
use crate::substrate::storage::StorageEngine;
use crate::types::{Blake3Hash, CellType, ComputeBudget, Query, RevisionId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Cell reference
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
    /// Active cells
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
}

impl Default for WorkingState {
    fn default() -> Self {
        Self {
            active_cells: HashSet::new(),
            completed_cells: HashSet::new(),
            failed_cells: HashSet::new(),
            depth: 0,
            compute_used: 0,
            bytes_moved: 0,
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
            .ok_or_else(|| CnwsError::CellNotFound(*hash))
    }

    async fn resolve_cells(&self, hashes: &[Blake3Hash]) -> Result<Vec<CellRef>> {
        hashes.iter()
            .map(|h| self.resolve_cell(h))
            .collect::<Result<Vec<_>>>()
    }

    async fn execute_cell(&self, cell: &CellRef, _inputs: &[CellRef]) -> Result<CellRef> {
        Ok(*cell)
    }

    async fn get_dependencies(&self, cell: &CellRef) -> Result<Vec<CellRef>> {
        Ok(Vec::new())
    }
}

/// Execution engine
pub struct ExecutionEngine {
    store: Arc<StorageEngine>,
    resolver: Arc<dyn RuntimeResolver>,
    cache: Arc<CacheManager>,
    memory: Arc<MemorySystem>,
    routing: Arc<RoutingEngine>,
    budget: ComputeBudget,
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
        }
    }

    /// Execute a query
    pub async fn execute(&self, query: &Query) -> Result<WorkingState> {
        let mut state = WorkingState::default();

        // Resolve entry cells
        let entry_cells = self.resolver.resolve_cells(&query.entry_cells).await?;

        // Execute cell graph
        self.execute_cells(&entry_cells, &mut state).await?;

        Ok(state)
    }

    /// Execute cells recursively
    async fn execute_cells(&self, cells: &[CellRef], state: &mut WorkingState) -> Result<()> {
        for cell in cells {
            // Check budget
            if state.compute_used >= self.budget.max_compute {
                return Err(CnwsError::BudgetExceeded);
            }

            if state.depth >= self.budget.max_depth {
                return Err(CnwsError::BudgetExceeded);
            }

            // Check cache
            if let Some(cached) = self.cache.get(&cell.hash) {
                state.completed_cells.insert(cell.hash);
                state.bytes_moved += cached.len() as u64;
                continue;
            }

            // Get dependencies
            let deps = self.resolver.get_dependencies(cell).await?;

            // Execute dependencies first
            if !deps.is_empty() {
                state.depth += 1;
                self.execute_cells(&deps, state).await?;
                state.depth -= 1;
            }

            // Execute cell
            let result = self.resolver.execute_cell(cell, &deps).await?;

            // Cache result
            let result_data = vec![]; // In real impl, would get actual data
            self.cache.insert(cell.hash, result_data);

            state.completed_cells.insert(cell.hash);
            state.compute_used += 1;
        }

        Ok(())
    }

    /// Get execution state
    pub fn state(&self) -> WorkingState {
        WorkingState::default()
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
    }

    #[tokio::test]
    async fn test_mock_resolver() {
        let mut resolver = MockResolver::new();
        let hash = Blake3Hash::hash(b"test");
        let cell = CellRef::new(hash, CellType::Tensor);
        resolver.add_cell(cell);

        let resolved = resolver.resolve_cell(&hash).await.unwrap();
        assert_eq!(resolved.hash, hash);
    }
}
