//! Routing engine - selects optimal Cells for queries
//! Implements cosine similarity and routing policies

use crate::error::Result;
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Routing policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingPolicy {
    /// Always use local
    Local,
    /// Always use remote
    Remote,
    /// Auto-select based on availability
    Auto,
    /// Load-balanced
    LoadBalanced,
    /// Lowest latency
    LowestLatency,
}

/// Routing statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingStatistics {
    /// Total queries routed
    pub total_queries: u64,
    /// Local queries
    pub local_queries: u64,
    /// Remote queries
    pub remote_queries: u64,
    /// Average latency (microseconds)
    pub avg_latency_us: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Cell metadata for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    /// Cell hash
    pub hash: Blake3Hash,
    /// Cell type
    pub cell_type: String,
    /// Size in bytes
    pub size: u64,
    /// Location (local/remote)
    pub is_local: bool,
    /// Latency (microseconds, 0 for local)
    pub latency_us: u64,
    /// Access count
    pub access_count: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
}

impl CellMetadata {
    /// Create new cell metadata
    pub fn new(hash: Blake3Hash, cell_type: impl Into<String>, size: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            hash,
            cell_type: cell_type.into(),
            size,
            is_local: true,
            latency_us: 0,
            access_count: 0,
            last_accessed: now,
        }
    }

    /// Touch (update access time)
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Routing engine
pub struct RoutingEngine {
    policy: Arc<RwLock<RoutingPolicy>>,
    cells: Arc<RwLock<HashMap<Blake3Hash, CellMetadata>>>,
    statistics: Arc<RwLock<RoutingStatistics>>,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(policy: RoutingPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            cells: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(RoutingStatistics::default())),
        }
    }

    /// Register a cell
    pub fn register_cell(&self, metadata: CellMetadata) {
        self.cells.write().insert(metadata.hash, metadata);
    }

    /// Select cells for a query using cosine similarity
    pub fn select(&self, query_vector: &[f32], candidates: &[Blake3Hash], top_k: usize) -> Result<Vec<(Blake3Hash, f32)>> {
        let cells = self.cells.read();
        let mut scores: Vec<(Blake3Hash, f32)> = Vec::new();

        for &hash in candidates {
            if let Some(metadata) = cells.get(&hash) {
                // Compute cosine similarity (simplified - in real impl would use actual vectors)
                let score = self.compute_similarity(query_vector, metadata);
                scores.push((hash, score));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k
        scores.truncate(top_k);

        // Update statistics
        {
            let mut stats = self.statistics.write();
            stats.total_queries += 1;
            stats.local_queries += scores.len() as u64;
        }

        Ok(scores)
    }

    /// Compute similarity score (simplified cosine similarity)
    fn compute_similarity(&self, _query: &[f32], metadata: &CellMetadata) -> f32 {
        // Simplified - in real implementation would use actual embedding vectors
        // For now, return a score based on access count and recency
        let recency_score = if metadata.last_accessed > 0 {
            let age = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - metadata.last_accessed;
            1.0 / (1.0 + (age as f32 / 3600.0))
        } else {
            0.0
        };

        let popularity_score = (metadata.access_count as f32).ln();
        let size_score = 1.0 / (1.0 + (metadata.size as f32 / 1_000_000.0));

        (recency_score + popularity_score + size_score) / 3.0
    }

    /// Route a query
    pub fn route(&self, _query: &str) -> Result<Vec<Blake3Hash>> {
        let policy = *self.policy.read();
        let cells = self.cells.read();

        let mut results = Vec::new();

        match policy {
            RoutingPolicy::Local => {
                // Return all local cells
                for (hash, metadata) in cells.iter() {
                    if metadata.is_local {
                        results.push(*hash);
                    }
                }
            }
            RoutingPolicy::Remote => {
                // Return all remote cells
                for (hash, metadata) in cells.iter() {
                    if !metadata.is_local {
                        results.push(*hash);
                    }
                }
            }
            RoutingPolicy::Auto => {
                // Return local cells first, then remote
                for (hash, metadata) in cells.iter() {
                    if metadata.is_local {
                        results.push(*hash);
                    }
                }
                for (hash, metadata) in cells.iter() {
                    if !metadata.is_local {
                        results.push(*hash);
                    }
                }
            }
            RoutingPolicy::LoadBalanced => {
                // Return cells with lowest access count
                let mut cell_vec: Vec<_> = cells.values().collect();
                cell_vec.sort_by(|a, b| a.access_count.cmp(&b.access_count));
                for metadata in cell_vec.iter().take(10) {
                    results.push(metadata.hash);
                }
            }
            RoutingPolicy::LowestLatency => {
                // Return cells with lowest latency
                let mut cell_vec: Vec<_> = cells.values().collect();
                cell_vec.sort_by(|a, b| a.latency_us.cmp(&b.latency_us));
                for metadata in cell_vec.iter().take(10) {
                    results.push(metadata.hash);
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.statistics.write();
            stats.total_queries += 1;
            stats.remote_queries += results.len() as u64;
        }

        Ok(results)
    }

    /// Set routing policy
    pub fn set_policy(&self, policy: RoutingPolicy) {
        *self.policy.write() = policy;
    }

    /// Get routing policy
    pub fn policy(&self) -> RoutingPolicy {
        *self.policy.read()
    }

    /// Get statistics
    pub fn statistics(&self) -> RoutingStatistics {
        self.statistics.read().clone()
    }

    /// Get cell metadata
    pub fn get_cell_metadata(&self, hash: &Blake3Hash) -> Option<CellMetadata> {
        self.cells.read().get(hash).cloned()
    }

    /// Update cell metadata
    pub fn update_cell_metadata(&self, hash: &Blake3Hash, metadata: CellMetadata) {
        self.cells.write().insert(*hash, metadata);
    }

    /// Remove cell
    pub fn remove_cell(&self, hash: &Blake3Hash) {
        self.cells.write().remove(hash);
    }

    /// List all cells
    pub fn list_cells(&self) -> Vec<Blake3Hash> {
        self.cells.read().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Blake3Hash;

    #[test]
    fn test_routing_policy() {
        let engine = RoutingEngine::new(RoutingPolicy::Local);
        assert_eq!(engine.policy(), RoutingPolicy::Local);

        engine.set_policy(RoutingPolicy::Auto);
        assert_eq!(engine.policy(), RoutingPolicy::Auto);
    }

    #[test]
    fn test_cell_metadata() {
        let hash = Blake3Hash::hash(b"test");
        let metadata = CellMetadata::new(hash, "tensor", 1024);
        assert_eq!(metadata.size, 1024);
        assert!(metadata.is_local);
    }
}
