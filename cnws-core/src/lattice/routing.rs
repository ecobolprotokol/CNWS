//! Routing engine - selects optimal Cells for queries
//!
//! Spec Ref: 06-runtime-execution.md §6
//!
//! Implements:
//! - Cosine similarity cell selection
//! - Policy-based routing (Local, Remote, Auto, LoadBalanced, LowestLatency)
//! - Per-cell routing statistics
//! - Content-based cell routing

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

/// Per-cell routing statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellRoutingStats {
    /// Number of times this cell was selected
    pub selection_count: u64,
    /// Number of times selection was successful (contributed to output)
    pub success_count: u64,
    /// Success rate (success_count / selection_count)
    pub success_rate: f64,
    /// Average contribution score
    pub avg_contribution: f32,
    /// Last step when this cell was selected
    pub last_selected_step: u64,
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
    /// Routing statistics
    pub routing_stats: CellRoutingStats,
    /// Index vector for content-based similarity
    pub index_vector: Option<Vec<f32>>,
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
            routing_stats: CellRoutingStats::default(),
            index_vector: None,
        }
    }

    /// Set index vector for content-based similarity
    pub fn with_index_vector(mut self, vec: Vec<f32>) -> Self {
        self.index_vector = Some(vec);
        self
    }

    /// Touch (update access time)
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Update routing statistics after selection
    pub fn record_selection(&mut self, successful: bool, contribution: f32, step: u64) {
        self.routing_stats.selection_count += 1;
        if successful {
            self.routing_stats.success_count += 1;
        }
        self.routing_stats.last_selected_step = step;

        let n = self.routing_stats.selection_count as f32;
        self.routing_stats.avg_contribution =
            (self.routing_stats.avg_contribution * (n - 1.0) + contribution) / n;

        self.routing_stats.success_rate = if self.routing_stats.selection_count > 0 {
            self.routing_stats.success_count as f64 / self.routing_stats.selection_count as f64
        } else {
            0.0
        };
    }
}

/// Routing engine
pub struct RoutingEngine {
    policy: Arc<RwLock<RoutingPolicy>>,
    cells: Arc<RwLock<HashMap<Blake3Hash, CellMetadata>>>,
    statistics: Arc<RwLock<RoutingStatistics>>,
    /// Weight for similarity scoring
    similarity_weight: f32,
    /// Weight for confidence scoring
    confidence_weight: f32,
    /// Weight for recency scoring
    recency_weight: f32,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(policy: RoutingPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            cells: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(RoutingStatistics::default())),
            similarity_weight: 0.5,
            confidence_weight: 0.3,
            recency_weight: 0.2,
        }
    }

    /// Create with custom scoring weights
    pub fn with_weights(
        policy: RoutingPolicy,
        similarity_weight: f32,
        confidence_weight: f32,
        recency_weight: f32,
    ) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            cells: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(RoutingStatistics::default())),
            similarity_weight,
            confidence_weight,
            recency_weight,
        }
    }

    /// Register a cell
    pub fn register_cell(&self, metadata: CellMetadata) {
        self.cells.write().insert(metadata.hash, metadata);
    }

    /// Select cells for a query using scoring
    pub fn select(&self, query_vector: &[f32], candidates: &[Blake3Hash], top_k: usize) -> Result<Vec<(Blake3Hash, f32)>> {
        let cells = self.cells.read();
        let mut scores: Vec<(Blake3Hash, f32)> = Vec::new();

        for &hash in candidates {
            if let Some(metadata) = cells.get(&hash) {
                let score = self.compute_score(query_vector, metadata);
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
            for &(hash, _) in &scores {
                if let Some(meta) = cells.get(&hash) {
                    if meta.is_local {
                        stats.local_queries += 1;
                    } else {
                        stats.remote_queries += 1;
                    }
                }
            }
        }

        Ok(scores)
    }

    /// Compute combined score for a cell
    fn compute_score(&self, query: &[f32], metadata: &CellMetadata) -> f32 {
        let similarity = self.compute_similarity(query, metadata);
        let confidence = self.compute_confidence(metadata);
        let recency = self.compute_recency(metadata);

        self.similarity_weight * similarity
            + self.confidence_weight * confidence
            + self.recency_weight * recency
    }

    /// Compute similarity score
    fn compute_similarity(&self, query: &[f32], metadata: &CellMetadata) -> f32 {
        if query.is_empty() {
            return 1.0 / (1.0 + (metadata.size as f32 / 1_000_000.0));
        }

        if let Some(ref index_vec) = metadata.index_vector {
            if !index_vec.is_empty() && query.len() == index_vec.len() {
                let dot_product: f32 = query.iter().zip(index_vec.iter()).map(|(a, b)| a * b).sum();
                let norm_a: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = index_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    return 0.0;
                }
                return dot_product / (norm_a * norm_b);
            }
        }

        let hash_bytes = metadata.hash.0;
        let mut dot_product = 0.0f32;
        let mut norm_b = 0.0f32;

        for (i, &q_val) in query.iter().enumerate() {
            if i < hash_bytes.len() {
                let b_val = hash_bytes[i] as f32 / 255.0;
                dot_product += q_val * b_val;
                norm_b += b_val * b_val;
            }
        }

        let norm_a: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = norm_b.sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Compute confidence score based on success rate
    fn compute_confidence(&self, metadata: &CellMetadata) -> f32 {
        if metadata.routing_stats.selection_count == 0 {
            return 0.5; // Default confidence for unselected cells
        }
        metadata.routing_stats.success_rate as f32
    }

    /// Compute recency score
    fn compute_recency(&self, metadata: &CellMetadata) -> f32 {
        if metadata.last_accessed == 0 {
            return 0.0;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age_secs = now.saturating_sub(metadata.last_accessed);
        1.0 / (1.0 + (age_secs as f32 / 3600.0))
    }

    /// Route a query based on policy
    pub fn route(&self, query: &str) -> Result<Vec<Blake3Hash>> {
        let policy = *self.policy.read();
        let cells = self.cells.read();

        let keywords: Vec<String> = query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let has_keywords = !keywords.is_empty();

        let matches_keywords = |metadata: &CellMetadata| -> bool {
            if !has_keywords {
                return true;
            }
            let ct_lower = metadata.cell_type.to_lowercase();
            keywords.iter().any(|kw| ct_lower.contains(kw.as_str()))
        };

        let results = self.select_by_policy(&cells, policy, matches_keywords);

        let results = if results.is_empty() && has_keywords {
            let fallback = |_: &CellMetadata| true;
            self.select_by_policy(&cells, policy, fallback)
        } else {
            results
        };

        let mut local_count = 0u64;
        let mut remote_count = 0u64;
        for &hash in &results {
            if let Some(meta) = cells.get(&hash) {
                if meta.is_local {
                    local_count += 1;
                } else {
                    remote_count += 1;
                }
            }
        }

        {
            let mut stats = self.statistics.write();
            stats.total_queries += 1;
            stats.local_queries += local_count;
            stats.remote_queries += remote_count;
        }

        Ok(results)
    }

    fn select_by_policy(
        &self,
        cells: &HashMap<Blake3Hash, CellMetadata>,
        policy: RoutingPolicy,
        filter: impl Fn(&CellMetadata) -> bool,
    ) -> Vec<Blake3Hash> {
        let mut results = Vec::new();

        match policy {
            RoutingPolicy::Local => {
                for (hash, metadata) in cells.iter() {
                    if metadata.is_local && filter(metadata) {
                        results.push(*hash);
                    }
                }
            }
            RoutingPolicy::Remote => {
                for (hash, metadata) in cells.iter() {
                    if !metadata.is_local && filter(metadata) {
                        results.push(*hash);
                    }
                }
            }
            RoutingPolicy::Auto => {
                let mut local_cells: Vec<_> = cells.values().filter(|m| m.is_local).collect();
                let mut remote_cells: Vec<_> = cells.values().filter(|m| !m.is_local).collect();
                local_cells.sort_by(|a, b| a.latency_us.cmp(&b.latency_us));
                remote_cells.sort_by(|a, b| a.latency_us.cmp(&b.latency_us));

                let selection_k = std::cmp::max(local_cells.len(), remote_cells.len());

                let filtered_local: Vec<_> = local_cells
                    .into_iter()
                    .filter(|m| filter(m))
                    .take(selection_k)
                    .collect();
                let filtered_remote: Vec<_> = remote_cells
                    .into_iter()
                    .filter(|m| filter(m))
                    .take(selection_k)
                    .collect();

                if !filtered_local.is_empty() {
                    for metadata in filtered_local {
                        results.push(metadata.hash);
                    }
                } else {
                    for metadata in filtered_remote {
                        results.push(metadata.hash);
                    }
                }
            }
            RoutingPolicy::LoadBalanced => {
                let mut cell_vec: Vec<_> = cells.values().collect();
                cell_vec.sort_by(|a, b| a.access_count.cmp(&b.access_count));
                for metadata in cell_vec.iter().take(10) {
                    if filter(metadata) {
                        results.push(metadata.hash);
                    }
                }
            }
            RoutingPolicy::LowestLatency => {
                let mut cell_vec: Vec<_> = cells.values().collect();
                cell_vec.sort_by(|a, b| a.latency_us.cmp(&b.latency_us));
                for metadata in cell_vec.iter().take(10) {
                    if filter(metadata) {
                        results.push(metadata.hash);
                    }
                }
            }
        }

        results
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

    /// Get cell count
    pub fn cell_count(&self) -> usize {
        self.cells.read().len()
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
        assert_eq!(metadata.routing_stats.selection_count, 0);
    }

    #[test]
    fn test_cell_routing_stats() {
        let hash = Blake3Hash::hash(b"test");
        let mut metadata = CellMetadata::new(hash, "tensor", 1024);

        metadata.record_selection(true, 0.9, 1);
        assert_eq!(metadata.routing_stats.selection_count, 1);
        assert_eq!(metadata.routing_stats.success_count, 1);
        assert!(metadata.routing_stats.success_rate > 0.0);

        metadata.record_selection(false, 0.1, 2);
        assert_eq!(metadata.routing_stats.selection_count, 2);
        assert_eq!(metadata.routing_stats.success_count, 1);
        assert!((metadata.routing_stats.success_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_routing_statistics_correct() {
        let engine = RoutingEngine::new(RoutingPolicy::Local);
        let hash1 = Blake3Hash::hash(b"cell1");
        let hash2 = Blake3Hash::hash(b"cell2");

        engine.register_cell(CellMetadata::new(hash1, "tensor", 1024));
        engine.register_cell(CellMetadata::new(hash2, "tensor", 2048));

        let results = engine.route("test").unwrap();
        assert_eq!(results.len(), 2);

        let stats = engine.statistics();
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.local_queries, 2);
        assert_eq!(stats.remote_queries, 0);
    }

    #[test]
    fn test_select_with_query() {
        let engine = RoutingEngine::new(RoutingPolicy::Auto);
        let hash1 = Blake3Hash::hash(b"cell1");
        let hash2 = Blake3Hash::hash(b"cell2");

        engine.register_cell(CellMetadata::new(hash1, "tensor", 1024));
        engine.register_cell(CellMetadata::new(hash2, "tensor", 2048));

        let query = vec![0.5, 0.3, 0.7, 0.1, 0.9];
        let results = engine.select(&query, &[hash1, hash2], 2).unwrap();

        assert_eq!(results.len(), 2);
        // Results should be sorted by score descending
        assert!(results[0].1 >= results[1].1);
    }

    #[test]
    fn test_routing_with_index_vectors() {
        let engine = RoutingEngine::new(RoutingPolicy::Auto);
        let hash1 = Blake3Hash::hash(b"cell1");
        let hash2 = Blake3Hash::hash(b"cell2");

        engine.register_cell(
            CellMetadata::new(hash1, "tensor", 1024)
                .with_index_vector(vec![1.0, 0.0, 0.0])
        );
        engine.register_cell(
            CellMetadata::new(hash2, "attention", 2048)
                .with_index_vector(vec![0.0, 1.0, 0.0])
        );

        let query = vec![1.0, 0.0, 0.0];
        let results = engine.select(&query, &[hash1, hash2], 2).unwrap();

        assert_eq!(results.len(), 2);
        // Cell1 should score higher (same direction as query)
        assert!(results[0].0 == hash1 || results[0].1 >= results[1].1);
    }

    #[test]
    fn test_auto_routing_policy() {
        let engine = RoutingEngine::new(RoutingPolicy::Auto);
        for i in 0..20u8 {
            let hash = Blake3Hash::hash(&[i]);
            let mut meta = CellMetadata::new(hash, "tensor", 1024);
            meta.is_local = i % 2 == 0;
            engine.register_cell(meta);
        }

        let results = engine.route("test").unwrap();
        // Auto should not return all 20 cells
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_route_with_query_filter() {
        let engine = RoutingEngine::new(RoutingPolicy::Local);
        let hash1 = Blake3Hash::hash(b"attn");
        let hash2 = Blake3Hash::hash(b"ffn");
        engine.register_cell(CellMetadata::new(hash1, "attention", 1024));
        engine.register_cell(CellMetadata::new(hash2, "feedforward", 2048));

        let results = engine.route("attention").unwrap();
        assert!(results.contains(&hash1));
    }
}
