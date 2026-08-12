//! Runtime API - public interface for Cell Graph execution

use super::super::lattice::runtime::{ExecutionEngine, WorkingState};
use super::super::types::{Blake3Hash, Query};
use crate::error::Result;
use std::sync::Arc;

/// Runtime API
pub struct RuntimeApi {
    engine: Arc<ExecutionEngine>,
}

impl RuntimeApi {
    /// Create a new runtime API
    pub fn new(engine: Arc<ExecutionEngine>) -> Self {
        Self { engine }
    }

    /// Get runtime config
    pub fn config(&self) -> &super::super::lattice::runtime::RuntimeConfig {
        self.engine.config()
    }

    /// Execute a query
    pub async fn execute(&self, query: &Query) -> Result<WorkingState> {
        self.engine.execute(query).await
    }

    /// Get execution state
    pub fn state(&self) -> WorkingState {
        self.engine.state()
    }
}

/// Query builder
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: Query {
                entry_cells: Vec::new(),
                parameters: std::collections::HashMap::new(),
                max_depth: 100,
                max_compute: 1_000_000,
            },
        }
    }

    /// Set entry cells
    pub fn with_entry_cells(mut self, cells: Vec<Blake3Hash>) -> Self {
        self.query.entry_cells = cells;
        self
    }

    /// Add entry cell
    pub fn add_entry_cell(mut self, cell: Blake3Hash) -> Self {
        self.query.entry_cells.push(cell);
        self
    }

    /// Set parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.parameters.insert(key.into(), value.into());
        self
    }

    /// Set max depth
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.query.max_depth = depth;
        self
    }

    /// Set max compute
    pub fn with_max_compute(mut self, compute: u64) -> Self {
        self.query.max_compute = compute;
        self
    }

    /// Build the query
    pub fn build(self) -> Query {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Blake3Hash;

    #[test]
    fn test_query_builder() {
        let hash = Blake3Hash::hash(b"test");
        let query = QueryBuilder::new()
            .add_entry_cell(hash)
            .with_max_depth(50)
            .with_max_compute(500_000)
            .build();

        assert_eq!(query.entry_cells.len(), 1);
        assert_eq!(query.max_depth, 50);
        assert_eq!(query.max_compute, 500_000);
    }

    #[tokio::test]
    async fn test_budget_enforcement() {
        use crate::lattice::cache::CacheManager;
        use crate::lattice::memory::MemorySystem;
        use crate::lattice::routing::{RoutingEngine, RoutingPolicy};
        use crate::lattice::runtime::{ExecutionEngine, MockResolver};
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use crate::types::{Blake3Hash, ComputeBudget, Query};
        use tempfile::tempdir;
        use std::sync::Arc;

        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = Arc::new(StorageEngine::create_store(config).unwrap());
        let resolver = Arc::new(MockResolver::new());
        let cache = Arc::new(CacheManager::new());
        let memory = Arc::new(MemorySystem::new(Arc::clone(&store), None));
        let routing = Arc::new(RoutingEngine::new(RoutingPolicy::Local));

        let budget = ComputeBudget {
            max_compute: 0,
            max_depth: 0,
            max_bytes: 0,
            max_time_secs: 0,
        };

        let engine = ExecutionEngine::new(
            store, resolver, cache, memory, routing, budget
        );

        let query = Query {
            entry_cells: vec![Blake3Hash::hash(b"test")],
            parameters: std::collections::HashMap::new(),
            max_depth: 100,
            max_compute: 1_000_000,
        };

        let result = engine.execute(&query).await;
        assert!(result.is_err());
    }
}
