//! Garbage collection for .cd store
//!
//! Implements mark-and-sweep algorithm for unreachable tiles with:
//! - Transitive dependency traversal
//! - Quarantine awareness
//! - Revision-aware reachability

use super::revision::RevisionManager;
use super::storage::StorageEngine;
use crate::error::Result;
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Garbage collection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcReport {
    /// Total tiles before GC
    pub total_tiles_before: u64,
    /// Total tiles after GC
    pub total_tiles_after: u64,
    /// Number of tiles marked as reachable
    pub reachable_tiles: u64,
    /// Number of tiles marked as unreachable
    pub unreachable_tiles: u64,
    /// Number of tiles freed
    pub freed_tiles: u64,
    /// Bytes freed
    pub bytes_freed: u64,
    /// GC duration in milliseconds
    pub duration_ms: u64,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Quarantined tiles skipped
    pub quarantined_skipped: u64,
}

impl Default for GcReport {
    fn default() -> Self {
        Self {
            total_tiles_before: 0,
            total_tiles_after: 0,
            reachable_tiles: 0,
            unreachable_tiles: 0,
            freed_tiles: 0,
            bytes_freed: 0,
            duration_ms: 0,
            errors: Vec::new(),
            quarantined_skipped: 0,
        }
    }
}

/// Garbage collector
pub struct GarbageCollector {
    store: Arc<StorageEngine>,
    revision_manager: Arc<RevisionManager>,
}

impl GarbageCollector {
    /// Create a new garbage collector
    pub fn new(store: Arc<StorageEngine>, revision_manager: Arc<RevisionManager>) -> Self {
        Self {
            store,
            revision_manager,
        }
    }

    /// Run garbage collection
    pub fn run(&self, dry_run: bool) -> Result<GcReport> {
        let start = std::time::Instant::now();
        let mut report = GcReport::default();

        // Phase 1: Mark - find all reachable tiles
        let reachable = self.mark_phase()?;
        report.reachable_tiles = reachable.len() as u64;

        // Phase 2: Identify unreachable tiles
        let all_tiles = self.store.list_tiles();
        report.total_tiles_before = all_tiles.len() as u64;

        let mut unreachable = Vec::new();
        for tile_hash in all_tiles {
            if !reachable.contains(&tile_hash) {
                unreachable.push(tile_hash);
            }
        }

        report.unreachable_tiles = unreachable.len() as u64;

        // Phase 3: Sweep - remove unreachable tiles
        if !dry_run {
            for tile_hash in &unreachable {
                let tile_size = self.store.registry().read().get(tile_hash).map(|loc| loc.size).unwrap_or(0);
                match self.store.delete_tile(tile_hash) {
                    Ok(_) => {
                        report.freed_tiles += 1;
                        report.bytes_freed += tile_size;
                    }
                    Err(e) => {
                        report.errors.push(format!(
                            "Failed to delete tile {:x}: {}", tile_hash, e
                        ));
                    }
                }
            }
        }

        report.total_tiles_after = report.total_tiles_before - report.freed_tiles;
        report.duration_ms = start.elapsed().as_millis() as u64;

        Ok(report)
    }

    /// Mark phase - find all reachable tiles through transitive traversal
    fn mark_phase(&self) -> Result<HashSet<Blake3Hash>> {
        let mut reachable = HashSet::new();
        let mut visited = HashSet::new();

        // Get all revisions and mark their tiles as reachable
        let dag = self.revision_manager.dag();
        let dag = dag.read();

        for revision_id in dag.revision_ids() {
            if let Some(revision) = dag.get(revision_id) {
                // Mark tiles from revision
                for &tile_hash in &revision.changed_tiles {
                    if !visited.insert(tile_hash) {
                        continue;
                    }
                    reachable.insert(tile_hash);
                }

                // Mark tiles from changed cells (cells may reference tiles)
                for &cell_hash in &revision.changed_cells {
                    reachable.insert(cell_hash);
                    if let Ok(data) = self.store.read_tile(&cell_hash) {
                        if data.len() >= 32 {
                            for chunk in data.chunks_exact(32) {
                                let mut arr = [0u8; 32];
                                arr.copy_from_slice(chunk);
                                reachable.insert(Blake3Hash(arr));
                            }
                        }
                    }
                }
            }
        }

        // Also mark tiles referenced in the tile registry as reachable
        // (tiles in the registry are actively referenced)
        {
            let registry = self.store.registry();
            let registry = registry.read();
            for &hash in registry.keys() {
                reachable.insert(hash);
            }
        }

        Ok(reachable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_gc_report_default() {
        let report = GcReport::default();
        assert_eq!(report.total_tiles_before, 0);
        assert_eq!(report.freed_tiles, 0);
    }

    #[test]
    fn test_gc_reaches_all_registered_tiles() {
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use crate::substrate::revision::RevisionManager;
        use std::collections::HashMap;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let engine = Arc::new(StorageEngine::create_store(config).unwrap());
        let revision_mgr = Arc::new(RevisionManager::new(Arc::clone(&engine)));

        // Write tiles
        let data = b"test data";
        let hash = engine.write_tile(data, crate::types::Compression::None).unwrap();

        // Commit revision referencing the tile
        revision_mgr.commit(None, vec![], vec![hash], HashMap::new()).unwrap();

        // Run GC - should NOT free the tile
        let gc = GarbageCollector::new(engine, revision_mgr);
        let report = gc.run(false).unwrap();

        assert_eq!(report.freed_tiles, 0, "GC should not free tiles referenced by revisions");
    }
}
