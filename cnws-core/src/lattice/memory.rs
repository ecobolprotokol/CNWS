//! Memory system - persistent memory with episodic, semantic, and procedural types
//! Implements content-addressed memory with BLAKE3-256 identity

use super::routing::RoutingEngine;
use crate::error::{CnwsError, Result};
use crate::types::{Blake3Hash, MemoryType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory ID (BLAKE3-256 of key + value)
    pub id: Blake3Hash,
    /// Memory type
    pub memory_type: MemoryType,
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
    /// Tags
    pub tags: Vec<String>,
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
        let id = Blake3Hash(*hasher.finalize().into());

        Self {
            id,
            memory_type,
            key,
            value,
            timestamp: now,
            access_count: 0,
            last_accessed: now,
            tags,
            metadata: HashMap::new(),
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

/// Memory index entry (fixed 104 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIndexEntry {
    /// Memory ID
    pub id: Blake3Hash,
    /// Memory type
    pub memory_type: MemoryType,
    /// Timestamp
    pub timestamp: u64,
    /// Access count
    pub access_count: u64,
    /// Last accessed
    pub last_accessed: u64,
    /// Tag count
    pub tag_count: u8,
    /// Reserved
    pub reserved: [u8; 55],
}

impl MemoryIndexEntry {
    /// Create from memory entry
    pub fn from_entry(entry: &MemoryEntry) -> Self {
        Self {
            id: entry.id,
            memory_type: entry.memory_type,
            timestamp: entry.timestamp,
            access_count: entry.access_count,
            last_accessed: entry.last_accessed,
            tag_count: entry.tags.len() as u8,
            reserved: [0u8; 55],
        }
    }

    /// Serialize to bytes (104 bytes)
    pub fn to_bytes(&self) -> [u8; 104] {
        let mut buf = [0u8; 104];
        buf[0..32].copy_from_slice(&self.id.0);
        buf[32..33].copy_from_slice(&[self.memory_type as u8]);
        buf[33..41].copy_from_slice(&self.timestamp.to_le_bytes());
        buf[41..49].copy_from_slice(&self.access_count.to_le_bytes());
        buf[49..57].copy_from_slice(&self.last_accessed.to_le_bytes());
        buf[57..58].copy_from_slice(&[self.tag_count]);
        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(buf: &[u8; 104]) -> Result<Self> {
        let id = Blake3Hash(<[u8; 32]>::try_from(&buf[0..32]).map_err(|_| CnwsError::CorruptStore)?);
        let memory_type = MemoryType::try_from(buf[32]).map_err(|_| CnwsError::CorruptStore)?;
        let timestamp = u64::from_le_bytes(buf[33..41].try_into().unwrap());
        let access_count = u64::from_le_bytes(buf[41..49].try_into().unwrap());
        let last_accessed = u64::from_le_bytes(buf[49..57].try_into().unwrap());
        let tag_count = buf[57];

        Ok(Self {
            id,
            memory_type,
            timestamp,
            access_count,
            last_accessed,
            tag_count,
            reserved: [0u8; 55],
        })
    }
}

/// Memory system
pub struct MemorySystem {
    store: Arc<crate::substrate::storage::StorageEngine>,
    entries: Arc<RwLock<HashMap<Blake3Hash, MemoryEntry>>>,
    routing: Option<Arc<RoutingEngine>>,
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
        let entry = MemoryEntry::new(memory_type, key, value, tags);
        let id = entry.id;

        // Store value as tile
        self.store.write_tile(&entry.value, crate::types::Compression::Zstd)?;

        // Update index
        self.entries.write().insert(id, entry);

        Ok(id)
    }

    /// Read a memory entry
    pub fn read(&self, id: &Blake3Hash) -> Result<MemoryEntry> {
        self.entries.read()
            .get(id)
            .cloned()
            .ok_or_else(|| CnwsError::MemoryNotFound(*id))
    }

    /// Search memory by query
    pub fn search(&self, query: &str, memory_type: Option<MemoryType>) -> Vec<MemoryEntry> {
        let entries = self.entries.read();
        let query_lower = query.to_lowercase();

        entries.values()
            .filter(|entry| {
                // Filter by type
                if let Some(mt) = memory_type {
                    if entry.memory_type != mt {
                        return false;
                    }
                }

                // Search in key, value, and tags
                let key_match = String::from_utf8_lossy(&entry.key).to_lowercase().contains(&query_lower);
                let value_match = String::from_utf8_lossy(&entry.value).to_lowercase().contains(&query_lower);
                let tag_match = entry.tags.iter().any(|t| t.to_lowercase().contains(&query_lower));

                key_match || value_match || tag_match
            })
            .cloned()
            .collect()
    }

    /// Delete a memory entry
    pub fn delete(&self, id: &Blake3Hash) -> Result<()> {
        self.entries.write()
            .remove(id)
            .ok_or_else(|| CnwsError::MemoryNotFound(*id))?;
        Ok(())
    }

    /// Get all entries of a type
    pub fn get_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        self.entries.read()
            .values()
            .filter(|e| e.memory_type == memory_type)
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
            Err(CnwsError::MemoryNotFound(*id))
        }
    }

    /// Get routing engine
    pub fn routing(&self) -> Option<Arc<RoutingEngine>> {
        self.routing.as_ref().map(Arc::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

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
    }

    #[test]
    fn test_memory_index_entry_serialization() {
        let entry = MemoryIndexEntry {
            id: Blake3Hash::hash(b"test"),
            memory_type: MemoryType::Semantic,
            timestamp: 1234567890,
            access_count: 5,
            last_accessed: 1234567890,
            tag_count: 2,
            reserved: [0u8; 55],
        };

        let bytes = entry.to_bytes();
        assert_eq!(bytes.len(), 104);

        let entry2 = MemoryIndexEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry.id, entry2.id);
        assert_eq!(entry.memory_type, entry2.memory_type);
    }
}
