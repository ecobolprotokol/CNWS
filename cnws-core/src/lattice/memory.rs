//! Memory system - persistent memory with episodic, semantic, and procedural types
//!
//! Spec Ref: 09-memory-retrieval.md
//!
//! Implements:
//! - Content-addressed memory with BLAKE3-256 identity
//! - Memory lifecycle (Created → Active → Consolidated → Archived → Forgotten)
//! - Working memory with bounded LRU eviction
//! - Memory associations
//! - Importance-based retention

use super::routing::RoutingEngine;
use crate::error::{CnwsError, Result};
use crate::types::{Blake3Hash, MemoryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Memory lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLifecycle {
    /// Just created
    Created,
    /// Active (recently accessed)
    Active,
    /// Consolidated (compiled from episodic to semantic)
    Consolidated,
    /// Archived (not recently accessed, but preserved)
    Archived,
    /// Marked for forgetting
    Forgotten,
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory ID (BLAKE3-256 of key + value)
    pub id: Blake3Hash,
    /// Memory type
    pub memory_type: MemoryType,
    /// Lifecycle state
    pub lifecycle: MemoryLifecycle,
    /// Key
    pub key: Vec<u8>,
    /// Value
    pub value: Vec<u8>,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Access count
    pub access_count: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Tags
    pub tags: Vec<String>,
    /// Association IDs (other memory entries this is related to)
    pub associations: Vec<Blake3Hash>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(
        memory_type: MemoryType,
        key: Vec<u8>,
        value: Vec<u8>,
        tags: Vec<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute ID from key + value
        let mut hasher = blake3::Hasher::new();
        hasher.update(&key);
        hasher.update(&value);
        let hash_bytes: [u8; 32] = hasher.finalize().into();
        let id = Blake3Hash(hash_bytes);

        Self {
            id,
            memory_type,
            lifecycle: MemoryLifecycle::Created,
            key,
            value,
            timestamp: now,
            access_count: 0,
            last_accessed: now,
            importance: 0.5,
            tags,
            associations: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Touch (update access time and importance)
    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Increase importance based on access frequency
        self.importance = (self.importance + 0.05).min(1.0);

        // Transition lifecycle
        if self.lifecycle == MemoryLifecycle::Created {
            self.lifecycle = MemoryLifecycle::Active;
        } else if self.lifecycle == MemoryLifecycle::Archived {
            self.lifecycle = MemoryLifecycle::Active;
        }
    }

    /// Apply decay based on time since last access
    pub fn apply_decay(&mut self, decay_rate: f32) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age_secs = now.saturating_sub(self.last_accessed);
        let age_hours = age_secs as f32 / 3600.0;

        // Exponential decay
        self.importance *= (-decay_rate * age_hours).exp();
        self.importance = self.importance.max(0.0);

        // Lifecycle transitions
        if self.importance < 0.1 && self.lifecycle == MemoryLifecycle::Active {
            self.lifecycle = MemoryLifecycle::Archived;
        }
        if self.importance < 0.01 && self.lifecycle == MemoryLifecycle::Archived {
            self.lifecycle = MemoryLifecycle::Forgotten;
        }
    }

    /// Add an association
    pub fn add_association(&mut self, other_id: Blake3Hash) {
        if !self.associations.contains(&other_id) {
            self.associations.push(other_id);
        }
    }

    /// Check if memory should be forgotten
    pub fn should_forget(&self) -> bool {
        self.lifecycle == MemoryLifecycle::Forgotten
    }
}

/// Memory index entry (fixed 104 bytes on-disk)
#[derive(Debug, Clone)]
pub struct MemoryIndexEntry {
    /// Memory ID
    pub id: Blake3Hash,
    /// Memory type
    pub memory_type: MemoryType,
    /// Lifecycle state
    pub lifecycle: u8,
    /// Timestamp
    pub timestamp: u64,
    /// Access count
    pub access_count: u64,
    /// Importance score (as u32, multiplied by 1000)
    pub importance_scaled: u32,
    /// Tag count
    pub tag_count: u8,
}

impl MemoryIndexEntry {
    /// Create from memory entry
    pub fn from_entry(entry: &MemoryEntry) -> Self {
        Self {
            id: entry.id,
            memory_type: entry.memory_type,
            lifecycle: entry.lifecycle as u8,
            timestamp: entry.timestamp,
            access_count: entry.access_count,
            importance_scaled: (entry.importance * 1000.0) as u32,
            tag_count: entry.tags.len() as u8,
        }
    }

    /// Serialize to bytes (104 bytes)
    pub fn to_bytes(&self) -> [u8; 104] {
        let mut buf = [0u8; 104];
        buf[0..32].copy_from_slice(&self.id.0);
        buf[32..33].copy_from_slice(&[self.memory_type as u8]);
        buf[33..34].copy_from_slice(&[self.lifecycle]);
        buf[34..42].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[42..50].copy_from_slice(&self.access_count.to_le_bytes());
        buf[50..54].copy_from_slice(&self.importance_scaled.to_le_bytes());
        buf[54..55].copy_from_slice(&[self.tag_count]);
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8; 104]) -> Result<Self> {
        let id = Blake3Hash(<[u8; 32]>::try_from(&buf[0..32]).map_err(|_| CnwsError::CorruptStore)?);
        let memory_type = MemoryType::try_from(buf[32]).map_err(|_| CnwsError::CorruptStore)?;
        let lifecycle = buf[33];
        let timestamp = u64::from_le_bytes(buf[34..42].try_into().unwrap());
        let access_count = u64::from_le_bytes(buf[42..50].try_into().unwrap());
        let importance_scaled = u32::from_le_bytes(buf[50..54].try_into().unwrap());
        let tag_count = buf[54];

        Ok(Self {
            id,
            memory_type,
            lifecycle,
            timestamp,
            access_count,
            importance_scaled,
            tag_count,
        })
    }
}

/// Working memory - bounded LRU cache for recent memories
pub struct WorkingMemory {
    entries: Vec<Blake3Hash>,
    max_size: usize,
}

impl WorkingMemory {
    /// Create a new working memory
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
        }
    }

    /// Add a memory to working memory
    pub fn push(&mut self, id: Blake3Hash) -> Option<Blake3Hash> {
        // Remove if already present
        self.entries.retain(|&e| e != id);

        self.entries.push(id);

        // Evict oldest if over capacity
        if self.entries.len() > self.max_size {
            Some(self.entries.remove(0))
        } else {
            None
        }
    }

    /// Check if a memory is in working memory
    pub fn contains(&self, id: &Blake3Hash) -> bool {
        self.entries.contains(id)
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Memory system
pub struct MemorySystem {
    store: Arc<crate::substrate::storage::StorageEngine>,
    entries: Arc<RwLock<HashMap<Blake3Hash, MemoryEntry>>>,
    routing: Option<Arc<RoutingEngine>>,
    working_memory: Arc<RwLock<WorkingMemory>>,
    decay_rate: f32,
}

impl MemorySystem {
    /// Create a new memory system
    pub fn new(
        store: Arc<crate::substrate::storage::StorageEngine>,
        routing: Option<Arc<RoutingEngine>>,
    ) -> Self {
        Self {
            store,
            entries: Arc::new(RwLock::new(HashMap::new())),
            routing,
            working_memory: Arc::new(RwLock::new(WorkingMemory::new(1000))),
            decay_rate: 0.01,
        }
    }

    /// Create with custom working memory size and decay rate
    pub fn with_settings(
        store: Arc<crate::substrate::storage::StorageEngine>,
        routing: Option<Arc<RoutingEngine>>,
        working_memory_size: usize,
        decay_rate: f32,
    ) -> Self {
        Self {
            store,
            entries: Arc::new(RwLock::new(HashMap::new())),
            routing,
            working_memory: Arc::new(RwLock::new(WorkingMemory::new(working_memory_size))),
            decay_rate,
        }
    }

    /// Write a memory entry
    pub fn write(
        &self,
        memory_type: MemoryType,
        key: Vec<u8>,
        value: Vec<u8>,
        tags: Vec<String>,
    ) -> Result<Blake3Hash> {
        let mut entry = MemoryEntry::new(memory_type, key, value, tags);
        entry.lifecycle = MemoryLifecycle::Active;
        let id = entry.id;

        // Store value as tile
        self.store.write_tile(&entry.value, crate::types::Compression::Zstd)?;

        // Update index
        self.entries.write().insert(id, entry);

        // Add to working memory
        self.working_memory.write().push(id);

        Ok(id)
    }

    /// Read a memory entry
    pub fn read(&self, id: &Blake3Hash) -> Result<MemoryEntry> {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(id) {
            entry.touch();
            Ok(entry.clone())
        } else {
            Err(CnwsError::MemoryNotFound)
        }
    }

    /// Search memory by query
    pub fn search(&self, query: &str, memory_type: Option<MemoryType>) -> Vec<MemoryEntry> {
        let entries = self.entries.read();
        let query_lower = query.to_lowercase();

        let mut results: Vec<MemoryEntry> = entries.values()
            .filter(|entry| {
                // Filter by type
                if let Some(mt) = memory_type {
                    if entry.memory_type != mt {
                        return false;
                    }
                }

                // Skip forgotten entries
                if entry.lifecycle == MemoryLifecycle::Forgotten {
                    return false;
                }

                // Search in key, value, and tags
                let key_match = String::from_utf8_lossy(&entry.key).to_lowercase().contains(&query_lower);
                let value_match = String::from_utf8_lossy(&entry.value).to_lowercase().contains(&query_lower);
                let tag_match = entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));

                key_match || value_match || tag_match
            })
            .cloned()
            .collect();

        // Sort by importance (most important first)
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Delete a memory entry
    pub fn delete(&self, id: &Blake3Hash) -> Result<()> {
        self.entries.write()
            .remove(id)
            .ok_or_else(|| CnwsError::MemoryNotFound)?;
        Ok(())
    }

    /// Get all entries of a type
    pub fn get_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        self.entries.read()
            .values()
            .filter(|e| e.memory_type == memory_type && e.lifecycle != MemoryLifecycle::Forgotten)
            .cloned()
            .collect()
    }

    /// Get entry count
    pub fn count(&self) -> usize {
        self.entries.read().len()
    }

    /// Touch (update access time)
    pub fn touch(&self, id: &Blake3Hash) -> Result<()> {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(id) {
            entry.touch();
            Ok(())
        } else {
            Err(CnwsError::MemoryNotFound)
        }
    }

    /// Add association between two memories
    pub fn add_association(&self, id1: &Blake3Hash, id2: &Blake3Hash) -> Result<()> {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get_mut(id1) {
            entry.add_association(*id2);
        }
        if let Some(entry) = entries.get_mut(id2) {
            entry.add_association(*id1);
        }
        Ok(())
    }

    /// Get associations of a memory
    pub fn get_associations(&self, id: &Blake3Hash) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read();
        if let Some(entry) = entries.get(id) {
            let assoc_ids = entry.associations.clone();
            let mut results = Vec::new();
            for assoc_id in &assoc_ids {
                if let Some(assoc) = entries.get(assoc_id) {
                    results.push(assoc.clone());
                }
            }
            Ok(results)
        } else {
            Err(CnwsError::MemoryNotFound)
        }
    }

    /// Apply decay to all memories (call periodically)
    pub fn apply_decay(&self) {
        let mut entries = self.entries.write();
        for entry in entries.values_mut() {
            entry.apply_decay(self.decay_rate);
        }
    }

    /// Forget all forgotten memories
    pub fn gc_forgotten(&self) -> u64 {
        let mut entries = self.entries.write();
        let before = entries.len();
        entries.retain(|_, e| !e.should_forget());
        (before - entries.len()) as u64
    }

    /// Get routing engine
    pub fn routing(&self) -> Option<Arc<RoutingEngine>> {
        self.routing.as_ref().map(Arc::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry::new(
            MemoryType::Episodic,
            b"key".to_vec(),
            b"value".to_vec(),
            vec!["tag1".to_string()],
        );
        assert_eq!(entry.memory_type, MemoryType::Episodic);
        assert_eq!(entry.access_count, 0);
        assert_eq!(entry.lifecycle, MemoryLifecycle::Created);
        assert_eq!(entry.importance, 0.5);
    }

    #[test]
    fn test_memory_entry_lifecycle() {
        let mut entry = MemoryEntry::new(
            MemoryType::Episodic,
            b"key".to_vec(),
            b"value".to_vec(),
            vec![],
        );
        assert_eq!(entry.lifecycle, MemoryLifecycle::Created);

        entry.touch();
        assert_eq!(entry.lifecycle, MemoryLifecycle::Active);
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_memory_entry_decay() {
        let mut entry = MemoryEntry::new(
            MemoryType::Episodic,
            b"key".to_vec(),
            b"value".to_vec(),
            vec![],
        );
        entry.lifecycle = MemoryLifecycle::Active;
        entry.last_accessed = 0; // Set to epoch start to simulate old memory

        // Apply significant decay
        for _ in 0..100 {
            entry.apply_decay(0.5);
        }
        assert!(entry.importance < 0.1, "importance was {}", entry.importance);
    }

    #[test]
    fn test_memory_entry_associations() {
        let mut entry = MemoryEntry::new(
            MemoryType::Episodic,
            b"key".to_vec(),
            b"value".to_vec(),
            vec![],
        );
        let other = Blake3Hash::hash(b"other");
        entry.add_association(other);
        assert!(entry.associations.contains(&other));

        // Adding same association again should not duplicate
        entry.add_association(other);
        assert_eq!(entry.associations.len(), 1);
    }

    #[test]
    fn test_working_memory() {
        let mut wm = WorkingMemory::new(3);
        let h1 = Blake3Hash::hash(b"1");
        let h2 = Blake3Hash::hash(b"2");
        let h3 = Blake3Hash::hash(b"3");
        let h4 = Blake3Hash::hash(b"4");

        assert!(wm.push(h1).is_none());
        assert!(wm.push(h2).is_none());
        assert!(wm.push(h3).is_none());

        // Adding 4th should evict oldest
        let evicted = wm.push(h4);
        assert_eq!(evicted, Some(h1));
        assert_eq!(wm.len(), 3);
    }

    #[test]
    fn test_memory_index_entry_serialization() {
        let entry = MemoryIndexEntry {
            id: Blake3Hash::hash(b"test"),
            memory_type: MemoryType::Semantic,
            lifecycle: MemoryLifecycle::Active as u8,
            timestamp: 1234567890,
            access_count: 5,
            importance_scaled: 750,
            tag_count: 2,
        };

        let bytes = entry.to_bytes();
        assert_eq!(bytes.len(), 104);

        let entry2 = MemoryIndexEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry.id, entry2.id);
        assert_eq!(entry.memory_type, entry2.memory_type);
        assert_eq!(entry.importance_scaled, entry2.importance_scaled);
    }
}
