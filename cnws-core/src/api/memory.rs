//! Memory API - public interface for memory operations

use super::super::lattice::memory::{MemoryEntry, MemorySystem};
use crate::error::{CnwsError, Result};
use crate::types::MemoryType;
use std::sync::Arc;

/// Memory API
pub struct MemoryApi {
    system: Arc<MemorySystem>,
}

impl MemoryApi {
    /// Create a new memory API
    pub fn new(system: Arc<MemorySystem>) -> Self {
        Self { system }
    }

    /// Write a memory entry
    pub fn write(
        &self,
        memory_type: MemoryType,
        key: Vec<u8>,
        value: Vec<u8>,
        tags: Vec<String>,
    ) -> Result<String> {
        let id = self.system.write(memory_type, key, value, tags)?;
        Ok(format!("{:x}", id))
    }

    /// Read a memory entry by ID
    pub fn read(&self, id: &str) -> Result<MemoryEntry> {
        let hash = parse_memory_id(id)?;
        self.system.read(&hash)
    }

    /// Search memory
    pub fn search(&self, query: &str, memory_type: Option<MemoryType>) -> Vec<MemoryEntry> {
        self.system.search(query, memory_type)
    }

    /// Delete a memory entry
    pub fn delete(&self, id: &str) -> Result<()> {
        let hash = parse_memory_id(id)?;
        self.system.delete(&hash)
    }

    /// Get all entries of a type
    pub fn get_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        self.system.get_by_type(memory_type)
    }

    /// Get entry count
    pub fn count(&self) -> usize {
        self.system.count()
    }

    /// Touch a memory entry
    pub fn touch(&self, id: &str) -> Result<()> {
        let hash = parse_memory_id(id)?;
        self.system.touch(&hash)
    }
}

/// Parse memory ID from hex string
fn parse_memory_id(id: &str) -> Result<crate::types::Blake3Hash> {
    let bytes = hex::decode(id)
        .map_err(|_| CnwsError::InvalidInput(format!("Invalid memory ID: {}", id)))?;

    if bytes.len() != 32 {
        return Err(CnwsError::InvalidInput(format!(
            "Invalid memory ID length: expected 32, got {}",
            bytes.len()
        )));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(crate::types::Blake3Hash(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::memory::MemorySystem;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_memory_api() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let system = Arc::new(MemorySystem::new(Arc::new(engine), None));
        let api = MemoryApi::new(system);

        let id = api.write(
            MemoryType::Episodic,
            b"key".to_vec(),
            b"value".to_vec(),
            vec!["tag".to_string()],
        ).unwrap();

        let entry = api.read(&id).unwrap();
        assert_eq!(entry.key, b"key");
        assert_eq!(entry.value, b"value");
    }
}
