//! Prefetch engine - dependency-aware prefetching for Cell Graph execution
//!
//! Spec Ref: 06-runtime-execution.md §6.7 (Prefetch Engine)
//!
//! Implements:
//! - Dependency-aware prefetching (DATA, CONTROL, EXECUTION_ORDER)
//! - MoE expert prefetching (for Mixture of Experts models)
//! - Adaptive depth prefetching
//! - Backpressure management

use super::cache::CacheManager;
use crate::error::Result;
use crate::types::{Blake3Hash, Dependency, DependencyType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Prefetch priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PrefetchPriority {
    /// Critical - must prefetch (DATA dependencies)
    Critical = 0,
    /// High - should prefetch (CONTROL dependencies)
    High = 1,
    /// Medium - nice to have (EXECUTION_ORDER)
    Medium = 2,
    /// Low - prefetch if bandwidth available (PREFETCH_HINT)
    Low = 3,
}

/// Prefetch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefetchRequest {
    /// Cell hash to prefetch
    pub cell_hash: Blake3Hash,
    /// Priority
    pub priority: PrefetchPriority,
    /// Estimated size in bytes
    pub estimated_size: u64,
    /// Deadline (microseconds from now)
    pub deadline_us: Option<u64>,
}

/// Prefetch statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefetchStatistics {
    /// Total prefetch requests
    pub total_requests: u64,
    /// Successful prefetches
    pub successful: u64,
    /// Failed prefetches
    pub failed: u64,
    /// Bytes prefetched
    pub bytes_prefetched: u64,
    /// Evictions triggered by prefetch
    pub evictions: u64,
    /// MoE prefetches (expert predictions)
    pub moe_prefetches: u64,
}

/// Prefetch plan
#[derive(Debug, Clone)]
pub struct PrefetchPlan {
    /// Ordered list of cells to prefetch
    pub requests: Vec<PrefetchRequest>,
    /// Total estimated bytes
    pub total_bytes: u64,
    /// Estimated latency (microseconds)
    pub estimated_latency_us: u64,
}

/// Prefetch engine
///
/// Implements dependency-aware prefetching per spec §6.7:
/// - Analyzes cell dependency graph to determine prefetch order
/// - Prioritizes DATA dependencies over CONTROL, EXECUTION_ORDER, PREFETCH_HINT
/// - Supports MoE expert prefetching (predict next expert based on router output)
/// - Manages backpressure to avoid overwhelming the cache
pub struct PrefetchEngine {
    cache: Arc<CacheManager>,
    /// Pending prefetch requests
    pending: Arc<RwLock<VecDeque<PrefetchRequest>>>,
    /// Completed prefetches
    completed: Arc<RwLock<HashSet<Blake3Hash>>>,
    /// Statistics
    statistics: Arc<RwLock<PrefetchStatistics>>,
    /// Maximum concurrent prefetches
    max_concurrent: usize,
    /// Maximum prefetch buffer size (bytes)
    max_buffer_size: u64,
    /// Current buffer size
    #[allow(dead_code)]
    current_buffer_size: Arc<RwLock<u64>>,
}

impl PrefetchEngine {
    /// Create a new prefetch engine
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self {
            cache,
            pending: Arc::new(RwLock::new(VecDeque::new())),
            completed: Arc::new(RwLock::new(HashSet::new())),
            statistics: Arc::new(RwLock::new(PrefetchStatistics::default())),
            max_concurrent: 16,
            max_buffer_size: 512 * 1024 * 1024, // 512 MB
            current_buffer_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a prefetch engine with custom settings
    pub fn with_settings(
        cache: Arc<CacheManager>,
        max_concurrent: usize,
        max_buffer_size: u64,
    ) -> Self {
        Self {
            cache,
            pending: Arc::new(RwLock::new(VecDeque::new())),
            completed: Arc::new(RwLock::new(HashSet::new())),
            statistics: Arc::new(RwLock::new(PrefetchStatistics::default())),
            max_concurrent,
            max_buffer_size,
            current_buffer_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Analyze dependencies and create a prefetch plan
    ///
    /// Spec Ref: §6.7.1 - Dependency-Aware Prefetching
    pub fn plan_prefetch(
        &self,
        target_cells: &[Blake3Hash],
        dependency_graph: &HashMap<Blake3Hash, Vec<Dependency>>,
        estimated_sizes: &HashMap<Blake3Hash, u64>,
    ) -> Result<PrefetchPlan> {
        let mut requests = Vec::new();
        let mut visited = HashSet::new();
        let mut total_bytes = 0u64;

        // BFS from target cells, respecting dependency types
        let mut queue = VecDeque::new();

        // First, add all target cells with Critical priority
        for &cell_hash in target_cells {
            if !visited.contains(&cell_hash) && !self.cache.get_any(&cell_hash).is_some() {
                queue.push_back((cell_hash, PrefetchPriority::Critical, 0u32));
                visited.insert(cell_hash);
            }
        }

        // BFS with depth tracking
        while let Some((cell_hash, priority, depth)) = queue.pop_front() {
            let size = estimated_sizes.get(&cell_hash).copied().unwrap_or(0);

            // Check backpressure
            if total_bytes + size > self.max_buffer_size {
                break;
            }

            requests.push(PrefetchRequest {
                cell_hash,
                priority,
                estimated_size: size,
                deadline_us: None,
            });
            total_bytes += size;

            // Enqueue dependencies with appropriate priority
            if let Some(deps) = dependency_graph.get(&cell_hash) {
                for dep in deps {
                    if !visited.contains(&dep.target) {
                        let dep_priority = match dep.dep_type {
                            DependencyType::Data => PrefetchPriority::Critical,
                            DependencyType::Control => PrefetchPriority::High,
                            DependencyType::ExecutionOrder => PrefetchPriority::Medium,
                            DependencyType::PrefetchHint => PrefetchPriority::Low,
                            DependencyType::Semantic => continue, // Skip semantic deps
                        };

                        // Only prefetch hard dependencies + prefetch hints
                        if dep_priority as u8 <= PrefetchPriority::Low as u8 {
                            visited.insert(dep.target);
                            queue.push_back((dep.target, dep_priority, depth + 1));
                        }
                    }
                }
            }
        }

        // Sort by priority (Critical first)
        requests.sort_by_key(|r| r.priority);

        // Estimate latency (simplified: 1ms per request + 0.1ms per MB)
        let estimated_latency_us = (requests.len() as u64) * 1000
            + (total_bytes / (1024 * 1024)) * 100;

        Ok(PrefetchPlan {
            requests,
            total_bytes,
            estimated_latency_us,
        })
    }

    /// Execute a prefetch plan
    pub fn execute_plan(&self, plan: &PrefetchPlan) -> Result<()> {
        let mut pending = self.pending.write();

        for request in &plan.requests {
            // Check if already completed or cached
            if self.completed.read().contains(&request.cell_hash) {
                continue;
            }
            if self.cache.get_any(&request.cell_hash).is_some() {
                self.completed.write().insert(request.cell_hash);
                continue;
            }

            // Add to pending queue
            pending.push_back(request.clone());
        }

        Ok(())
    }

    /// Process pending prefetches (called by runtime)
    pub fn process_pending(&self, fetch_fn: &dyn Fn(&Blake3Hash) -> Result<Vec<u8>>) -> Result<u64> {
        let mut processed = 0u64;
        let mut pending = self.pending.write();
        let mut completed = self.completed.write();
        let mut stats = self.statistics.write();

        let batch_size = self.max_concurrent.min(pending.len());
        let mut to_remove = Vec::new();

        for i in 0..batch_size {
            if let Some(request) = pending.get(i) {
                stats.total_requests += 1;

                // Try to fetch the data
                match fetch_fn(&request.cell_hash) {
                    Ok(data) => {
                        // Insert into appropriate cache level
                        self.cache.insert(request.cell_hash, data.clone(), super::cache::CacheLevel::L1);

                        completed.insert(request.cell_hash);
                        stats.successful += 1;
                        stats.bytes_prefetched += data.len() as u64;
                        to_remove.push(i);
                        processed += 1;
                    }
                    Err(_) => {
                        stats.failed += 1;
                        to_remove.push(i);
                    }
                }
            }
        }

        // Remove processed items (in reverse order to maintain indices)
        for &i in to_remove.iter().rev() {
            pending.remove(i);
        }

        Ok(processed)
    }

    /// Prefetch MoE experts based on router output
    ///
    /// Spec Ref: §6.7.2 - MoE Expert Prefetching
    pub fn prefetch_moe_experts(
        &self,
        _router_hash: Blake3Hash,
        expert_hashes: &[Blake3Hash],
        top_k: usize,
        estimated_sizes: &HashMap<Blake3Hash, u64>,
    ) -> Result<()> {
        let mut stats = self.statistics.write();
        stats.moe_prefetches += 1;
        drop(stats);

        // Select top-k experts (simplified - in real impl, would use router output)
        let selected: Vec<_> = expert_hashes.iter().take(top_k).cloned().collect();

        let mut pending = self.pending.write();
        for expert_hash in selected {
            if !self.completed.read().contains(&expert_hash)
                && self.cache.get_any(&expert_hash).is_none()
            {
                let size = estimated_sizes.get(&expert_hash).copied().unwrap_or(0);
                pending.push_back(PrefetchRequest {
                    cell_hash: expert_hash,
                    priority: PrefetchPriority::Medium,
                    estimated_size: size,
                    deadline_us: None,
                });
            }
        }

        Ok(())
    }

    /// Adaptive depth prefetching - prefetch deeper based on available bandwidth
    pub fn adaptive_depth_prefetch(
        &self,
        _current_depth: u32,
        _max_depth: u32,
        available_bandwidth_bytes: u64,
        dependency_graph: &HashMap<Blake3Hash, Vec<Dependency>>,
        estimated_sizes: &HashMap<Blake3Hash, u64>,
        entry_cells: &[Blake3Hash],
    ) -> Result<PrefetchPlan> {
        let mut requests = Vec::new();
        let mut visited = HashSet::new();
        let mut total_bytes = 0u64;

        let mut queue = VecDeque::new();

        for &cell_hash in entry_cells {
            if !visited.contains(&cell_hash) && self.cache.get_any(&cell_hash).is_none() {
                queue.push_back((cell_hash, PrefetchPriority::Critical, 0u32));
                visited.insert(cell_hash);
            }
        }

        while let Some((cell_hash, priority, depth)) = queue.pop_front() {
            let size = estimated_sizes.get(&cell_hash).copied().unwrap_or(0);

            if total_bytes + size > available_bandwidth_bytes {
                break;
            }

            requests.push(PrefetchRequest {
                cell_hash,
                priority,
                estimated_size: size,
                deadline_us: None,
            });
            total_bytes += size;

            if let Some(deps) = dependency_graph.get(&cell_hash) {
                for dep in deps {
                    if !visited.contains(&dep.target) {
                        let dep_priority = match dep.dep_type {
                            DependencyType::Data => PrefetchPriority::Critical,
                            DependencyType::Control => PrefetchPriority::High,
                            DependencyType::ExecutionOrder => PrefetchPriority::Medium,
                            DependencyType::PrefetchHint => PrefetchPriority::Low,
                            DependencyType::Semantic => continue,
                        };

                        if dep_priority as u8 <= PrefetchPriority::Low as u8 {
                            visited.insert(dep.target);
                            queue.push_back((dep.target, dep_priority, depth + 1));
                        }
                    }
                }
            }
        }

        requests.sort_by_key(|r| r.priority);

        let estimated_latency_us = (requests.len() as u64) * 1000
            + (total_bytes / (1024 * 1024)) * 100;

        Ok(PrefetchPlan {
            requests,
            total_bytes,
            estimated_latency_us,
        })
    }

    /// Check if a cell is already prefetched
    pub fn is_prefetched(&self, hash: &Blake3Hash) -> bool {
        self.completed.read().contains(hash) || self.cache.get_any(hash).is_some()
    }

    /// Get pending prefetch count
    pub fn pending_count(&self) -> usize {
        self.pending.read().len()
    }

    /// Get statistics
    pub fn statistics(&self) -> PrefetchStatistics {
        self.statistics.read().clone()
    }

    /// Clear pending prefetches
    pub fn clear(&self) {
        self.pending.write().clear();
        self.completed.write().clear();
    }
}

impl Default for PrefetchEngine {
    fn default() -> Self {
        Self::new(Arc::new(CacheManager::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::cache::CacheManager;
    use crate::types::{Blake3Hash, Dependency};

    #[test]
    fn test_prefetch_plan() {
        let cache = Arc::new(CacheManager::new());
        let engine = PrefetchEngine::new(cache);

        let cell1 = Blake3Hash::hash(b"cell1");
        let cell2 = Blake3Hash::hash(b"cell2");
        let cell3 = Blake3Hash::hash(b"cell3");

        let mut dep_graph = HashMap::new();
        dep_graph.insert(cell1, vec![
            Dependency::data(cell2),
            Dependency::prefetch_hint(cell3),
        ]);

        let mut sizes = HashMap::new();
        sizes.insert(cell1, 1024);
        sizes.insert(cell2, 2048);
        sizes.insert(cell3, 512);

        let plan = engine.plan_prefetch(&[cell1], &dep_graph, &sizes).unwrap();
        assert!(plan.requests.len() >= 1);
        assert!(plan.total_bytes > 0);
    }

    #[test]
    fn test_prefetch_moe_experts() {
        let cache = Arc::new(CacheManager::new());
        let engine = PrefetchEngine::new(cache);

        let router = Blake3Hash::hash(b"router");
        let experts: Vec<Blake3Hash> = (0u8..8).map(|i| Blake3Hash::hash(&i.to_le_bytes())).collect();

        engine.prefetch_moe_experts(router, &experts, 2, &HashMap::new()).unwrap();
        assert_eq!(engine.pending_count(), 2);
    }

    #[test]
    fn test_prefetch_statistics() {
        let cache = Arc::new(CacheManager::new());
        let engine = PrefetchEngine::new(cache);
        let stats = engine.statistics();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful, 0);
    }

    #[test]
    fn test_adaptive_depth_prefetch() {
        use crate::lattice::cache::CacheManager;
        use crate::types::Dependency;
        use std::collections::HashMap;

        let cache = Arc::new(CacheManager::new());
        let engine = PrefetchEngine::new(cache);

        let cell1 = Blake3Hash::hash(b"cell1");
        let cell2 = Blake3Hash::hash(b"cell2");
        let cell3 = Blake3Hash::hash(b"cell3");

        let mut dep_graph = HashMap::new();
        dep_graph.insert(cell1, vec![Dependency::data(cell2)]);
        dep_graph.insert(cell2, vec![Dependency::data(cell3)]);

        let mut sizes = HashMap::new();
        sizes.insert(cell1, 1024);
        sizes.insert(cell2, 2048);
        sizes.insert(cell3, 4096);

        let plan = engine.adaptive_depth_prefetch(
            0, 10, 5000,
            &dep_graph, &sizes, &[cell1]
        ).unwrap();

        assert!(plan.requests.len() >= 1);
        assert!(plan.total_bytes <= 5000);
    }
}
