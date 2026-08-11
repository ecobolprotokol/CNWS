//! Runtime API - public interface for Cell Graph execution

use super::super::lattice::runtime::{ExecutionEngine, WorkingState};
use super::super::types::{Blake3Hash, ComputeBudget, Query};
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

    /// Execute a query
    pub async fn execute(&self, query: &Query) -> Result<WorkingState> {
        self.engine.execute(query).await
    }

    /// Get execution state
    pub fn state(&self) -> WorkingState {
        self.engine.state()
    }

    /// Set compute budget
    pub fn with_budget(self, _budget: ComputeBudget) -> Self {
        // In real implementation, would update engine budget
        self
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
}
