//! Cache manager - multi-level cache hierarchy
//! Implements L0/L1/L2/L3 cache with LRU eviction

use crate::error::{CnwsError, Result};
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Cache level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheLevel {
    /// L0: GPU VRAM (fastest, smallest)
    L0,
    /// L1: CPU RAM (fast, medium)
    L1,
    /// L2: NVMe SSD (slower, larger)
    L2,
    /// L3: Network (slowest, largest)
    L3,
}

impl std::fmt::Display for CacheLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::L0 => write!(f, "L0 (GPU VRAM)"),
            Self::L1 => write!(f, "L1 (CPU RAM)"),
            Self::L2 => write!(f, "L2 (NVMe)"),
            Self::L3 => write!(f, "L3 (Network)"),
        }
    }
}

/// Cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Tile hash
    pub hash: Blake3Hash,
    /// Data
    pub data: Vec<u8>,
    /// Size in bytes
    pub size: usize,
    /// Access count
    pub access_count: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
    /// Cache level
    pub level: CacheLevel,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(hash: Blake3Hash, data: Vec<u8>, level: CacheLevel) -> Self {
        let size = data.len();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            hash,
            data,
            size,
            access_count: 1,
            last_accessed: now,
            level,
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

/// LRU cache with byte capacity
pub struct LruCache {
    entries: HashMap<Blake3Hash, CacheEntry>,
    capacity: usize,
    current_size: usize,
}

impl LruCache {
    /// Create a new LRU cache
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity,
            current_size: 0,
        }
    }

    /// Insert an entry
    pub fn insert(&mut self, entry: CacheEntry) {
        let size = entry.size;

        // Evict if necessary
        while self.current_size + size > self.capacity && !self.entries.is_empty() {
            self.evict_lru();
        }

        self.entries.insert(entry.hash, entry);
        self.current_size += size;
    }

    /// Get an entry
    pub fn get(&mut self, hash: &Blake3Hash) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get_mut(hash) {
            entry.touch();
            Some(entry)
        } else {
            None
        }
    }

    /// Remove an entry
    pub fn remove(&mut self, hash: &Blake3Hash) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.remove(hash) {
            self.current_size -= entry.size;
            Some(entry)
        } else {
            None
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((&hash, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
        {
            if let Some(entry) = self.entries.remove(&hash) {
                self.current_size -= entry.size;
            }
        }
    }

    /// Get current size
    pub fn current_size(&self) -> usize {
        self.current_size
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
    }
}

/// Cache manager
pub struct CacheManager {
    l0: Arc<RwLock<LruCache>>,
    l1: Arc<RwLock<LruCache>>,
    l2: Arc<RwLock<LruCache>>,
    l3: Arc<RwLock<LruCache>>,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl CacheManager {
    /// Create a new cache manager with default capacities
    pub fn new() -> Self {
        Self::with_capacities(
            256 * 1024 * 1024,  // L0: 256MB
            2 * 1024 * 1024 * 1024,  // L1: 2GB
            16 * 1024 * 1024 * 1024, // L2: 16GB
            128 * 1024 * 1024 * 1024, // L3: 128GB
        )
    }

    /// Create a new cache manager with custom capacities
    pub fn with_capacities(l0: usize, l1: usize, l2: usize, l3: usize) -> Self {
        Self {
            l0: Arc::new(RwLock::new(LruCache::new(l0))),
            l1: Arc::new(RwLock::new(LruCache::new(l1))),
            l2: Arc::new(RwLock::new(LruCache::new(l2))),
            l3: Arc::new(RwLock::new(LruCache::new(l3))),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get cache for level
    fn cache_for_level(&self, level: CacheLevel) -> &Arc<RwLock<LruCache>> {
        match level {
            CacheLevel::L0 => &self.l0,
            CacheLevel::L1 => &self.l1,
            CacheLevel::L2 => &self.l2,
            CacheLevel::L3 => &self.l3,
        }
    }

    /// Insert data into cache
    pub fn insert(&self, hash: Blake3Hash, data: Vec<u8>, level: CacheLevel) {
        let entry = CacheEntry::new(hash, data, level);
        self.cache_for_level(level).write().insert(entry);
    }

    /// Get data from cache
    pub fn get(&self, hash: &Blake3Hash, level: CacheLevel) -> Option<Vec<u8>> {
        let mut cache = self.cache_for_level(level).write();
        if let Some(entry) = cache.get(hash) {
            *self.hits.write() += 1;
            Some(entry.data.clone())
        } else {
            *self.misses.write() += 1;
            None
        }
    }

    /// Get from any level (L0 first, then L1, etc.)
    pub fn get_any(&self, hash: &Blake3Hash) -> Option<(Vec<u8>, CacheLevel)> {
        for level in [CacheLevel::L0, CacheLevel::L1, CacheLevel::L2, CacheLevel::L3] {
            if let Some(data) = self.get(hash, level) {
                return Some((data, level));
            }
        }
        None
    }

    /// Remove from all levels
    pub fn remove(&self, hash: &Blake3Hash) {
        for level in [CacheLevel::L0, CacheLevel::L1, CacheLevel::L2, CacheLevel::L3] {
            self.cache_for_level(level).write().remove(hash);
        }
    }

    /// Get cache statistics
    pub fn statistics(&self) -> CacheStatistics {
        let l0 = self.l0.read();
        let l1 = self.l1.read();
        let l2 = self.l2.read();
        let l3 = self.l3.read();
        let hits = *self.hits.read();
        let misses = *self.misses.read();

        CacheStatistics {
            l0_size: l0.current_size(),
            l0_capacity: l0.capacity(),
            l0_entries: l0.len(),
            l1_size: l1.current_size(),
            l1_capacity: l1.capacity(),
            l1_entries: l1.len(),
            l2_size: l2.current_size(),
            l2_capacity: l2.capacity(),
            l2_entries: l2.len(),
            l3_size: l3.current_size(),
            l3_capacity: l3.capacity(),
            l3_entries: l3.len(),
            hits,
            misses,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.l0.write().clear();
        self.l1.write().clear();
        self.l2.write().clear();
        self.l3.write().clear();
    }

    /// Promote entry to higher level
    pub fn promote(&self, hash: Blake3Hash, from: CacheLevel, to: CacheLevel) -> Result<()> {
        if let Some(data) = self.get(&hash, from) {
            self.insert(hash, data, to);
            Ok(())
        } else {
            Err(CnwsError::CacheMiss(hash))
        }
    }

    /// Demote entry to lower level
    pub fn demote(&self, hash: Blake3Hash, from: CacheLevel, to: CacheLevel) -> Result<()> {
        if let Some(data) = self.get(&hash, from) {
            self.insert(hash, data, to);
            Ok(())
        } else {
            Err(CnwsError::CacheMiss(hash))
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub l0_size: usize,
    pub l0_capacity: usize,
    pub l0_entries: usize,
    pub l1_size: usize,
    pub l1_capacity: usize,
    pub l1_entries: usize,
    pub l2_size: usize,
    pub l2_capacity: usize,
    pub l2_entries: usize,
    pub l3_size: usize,
    pub l3_capacity: usize,
    pub l3_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Blake3Hash;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(100);
        let hash = Blake3Hash::hash(b"test");
        let entry = CacheEntry::new(hash, vec![1, 2, 3], CacheLevel::L1);
        cache.insert(entry);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&hash).is_some());
    }

    #[test]
    fn test_cache_manager() {
        let manager = CacheManager::new();
        let hash = Blake3Hash::hash(b"test");
        manager.insert(hash, vec![1, 2, 3], CacheLevel::L1);
        let data = manager.get(&hash, CacheLevel::L1);
        assert!(data.is_some());
    }

    #[test]
    fn test_cache_statistics() {
        let manager = CacheManager::new();
        let stats = manager.statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }
}
