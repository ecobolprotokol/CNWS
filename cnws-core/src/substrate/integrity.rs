//! Integrity verification subsystem
//! Implements BLAKE3-256 verification, quarantine, and corruption detection

use super::storage::StorageEngine;
use crate::error::Result;
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Integrity verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Tile hash
    pub tile_hash: Blake3Hash,
    /// Whether verification passed
    pub passed: bool,
    /// Expected hash
    pub expected_hash: Blake3Hash,
    /// Actual hash
    pub actual_hash: Blake3Hash,
    /// Error message if failed
    pub error: Option<String>,
}

/// Quarantine entry for corrupt tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    /// Tile hash
    pub tile_hash: Blake3Hash,
    /// Reason for quarantine
    pub reason: String,
    /// Timestamp when quarantined
    pub quarantined_at: u64,
}

/// Quarantine manager
#[derive(Debug, Clone, Default)]
pub struct Quarantine {
    entries: HashMap<Blake3Hash, QuarantineEntry>,
}

impl Quarantine {
    /// Create a new quarantine
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tile to quarantine
    pub fn add(&mut self, tile_hash: Blake3Hash, reason: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(tile_hash, QuarantineEntry {
            tile_hash,
            reason,
            quarantined_at: now,
        });
    }

    /// Remove a tile from quarantine
    pub fn remove(&mut self, tile_hash: &Blake3Hash) -> Option<QuarantineEntry> {
        self.entries.remove(tile_hash)
    }

    /// Check if tile is quarantined
    pub fn contains(&self, tile_hash: &Blake3Hash) -> bool {
        self.entries.contains_key(tile_hash)
    }

    /// Get all quarantined tiles
    pub fn entries(&self) -> impl Iterator<Item = &QuarantineEntry> {
        self.entries.values()
    }

    /// Get number of quarantined tiles
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Integrity verifier
pub struct IntegrityVerifier {
    store: Arc<StorageEngine>,
    quarantine: Arc<RwLock<Quarantine>>,
}

impl IntegrityVerifier {
    /// Create a new integrity verifier
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self {
            store,
            quarantine: Arc::new(RwLock::new(Quarantine::new())),
        }
    }

    /// Verify a single tile
    pub fn verify_tile(&self, tile_hash: &Blake3Hash) -> Result<VerificationResult> {
        // Read tile data
        let data = self.store.read_tile(tile_hash)?;

        // Compute actual hash
        let actual_hash = Blake3Hash::hash(&data);

        let passed = actual_hash == *tile_hash;

        if !passed {
            self.quarantine.write().add(
                *tile_hash,
                format!("Hash mismatch: expected {:x}, got {:x}", tile_hash, actual_hash),
            );
        }

        Ok(VerificationResult {
            tile_hash: *tile_hash,
            passed,
            expected_hash: *tile_hash,
            actual_hash,
            error: if !passed {
                Some(format!("Hash mismatch: expected {:x}, got {:x}", tile_hash, actual_hash))
            } else {
                None
            },
        })
    }

    /// Verify all tiles in store
    pub fn verify_all(&self) -> Result<Vec<VerificationResult>> {
        let tiles = self.store.list_tiles();
        let mut results = Vec::new();

        for tile_hash in tiles {
            match self.verify_tile(&tile_hash) {
                Ok(result) => results.push(result),
                Err(e) => {
                    self.quarantine.write().add(
                        tile_hash,
                        format!("Verification error: {}", e),
                    );
                }
            }
        }

        Ok(results)
    }

    /// Get quarantine
    pub fn quarantine(&self) -> Arc<RwLock<Quarantine>> {
        Arc::clone(&self.quarantine)
    }

    /// Get quarantine entries
    pub fn quarantined_tiles(&self) -> Vec<QuarantineEntry> {
        self.quarantine.read().entries().cloned().collect()
    }

    /// Remove tile from quarantine
    pub fn release_from_quarantine(&self, tile_hash: &Blake3Hash) -> Option<QuarantineEntry> {
        self.quarantine.write().remove(tile_hash)
    }

    /// Clear quarantine
    pub fn clear_quarantine(&self) {
        self.quarantine.write().entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use crate::types::Compression;
    use tempfile::tempdir;

    #[test]
    fn test_quarantine() {
        let mut q = Quarantine::new();
        let hash = Blake3Hash::hash(b"test");
        q.add(hash, "test reason".to_string());
        assert!(q.contains(&hash));
        assert_eq!(q.len(), 1);
        q.remove(&hash);
        assert!(!q.contains(&hash));
    }

    #[test]
    fn test_verify_tile() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);

        let data = b"test data";
        let hash = engine.write_tile(data, Compression::None).unwrap();

        let verifier = IntegrityVerifier::new(engine);
        let result = verifier.verify_tile(&hash).unwrap();
        assert!(result.passed);
    }
}
