//! Admin API - public interface for administrative operations

use super::super::substrate::gc::GarbageCollector;
use super::super::substrate::integrity::{IntegrityVerifier, QuarantineEntry};
use super::super::substrate::recovery::{RecoveryManager, RecoveryReport};
use super::super::substrate::storage::StorageEngine;
use crate::error::{CnwsError, Result};
use std::sync::Arc;

/// Admin API
pub struct AdminApi {
    store: Arc<StorageEngine>,
    recovery: Arc<RecoveryManager>,
    gc: Arc<GarbageCollector>,
}

impl AdminApi {
    /// Create a new admin API
    pub fn new(
        store: Arc<StorageEngine>,
        recovery: Arc<RecoveryManager>,
        gc: Arc<GarbageCollector>,
    ) -> Self {
        Self { store, recovery, gc }
    }

    /// Run garbage collection
    pub fn gc(&self, dry_run: bool) -> Result<crate::substrate::gc::GcReport> {
        self.gc.run(dry_run)
    }

    /// Recover store
    pub fn recover(&self) -> Result<RecoveryReport> {
        self.recovery.recover()
    }

    /// Verify store integrity
    pub fn verify(&self) -> Result<Vec<crate::substrate::integrity::VerificationResult>> {
        let verifier = IntegrityVerifier::new(Arc::clone(&self.store));
        verifier.verify_all()
    }

    /// Get quarantine entries
    pub fn quarantined_tiles(&self) -> Vec<QuarantineEntry> {
        let verifier = IntegrityVerifier::new(Arc::clone(&self.store));
        verifier.quarantined_tiles()
    }

    /// Release tile from quarantine
    pub fn release_from_quarantine(&self, tile_hash: &str) -> Result<Option<QuarantineEntry>> {
        let hash = parse_tile_hash(tile_hash)?;
        let verifier = IntegrityVerifier::new(Arc::clone(&self.store));
        Ok(verifier.release_from_quarantine(&hash))
    }

    /// Check recovery status
    pub fn recovery_status(&self) -> crate::substrate::recovery::RecoveryState {
        self.recovery.state()
    }

    /// Get store statistics
    pub fn store_stats(&self) -> crate::substrate::storage::StoreStats {
        self.store.stats()
    }
}

/// Parse tile hash from hex string
fn parse_tile_hash(hash: &str) -> Result<crate::types::Blake3Hash> {
    let bytes = hex::decode(hash)
        .map_err(|_| CnwsError::InvalidInput(format!("Invalid tile hash: {}", hash)))?;

    if bytes.len() != 32 {
        return Err(CnwsError::InvalidInput(format!(
            "Invalid tile hash length: expected 32, got {}",
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
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use crate::substrate::recovery::RecoveryManager;
    use crate::substrate::gc::GarbageCollector;
    use crate::substrate::revision::RevisionManager;
    use tempfile::tempdir;

    #[test]
    fn test_admin_api_creation() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);

        let revision_manager = Arc::new(RevisionManager::new(Arc::clone(&engine)));
        let recovery = Arc::new(RecoveryManager::new(Arc::clone(&engine), dir.path().join("wal.log")));
        let gc = Arc::new(GarbageCollector::new(Arc::clone(&engine), Arc::clone(&revision_manager)));

        let api = AdminApi::new(engine, recovery, gc);
        let stats = api.store_stats();
        assert_eq!(stats.total_tiles, 0);
    }
}
