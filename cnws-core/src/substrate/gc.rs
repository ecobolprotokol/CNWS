//! Garbage collection for .cd store
//! Implements mark-and-sweep algorithm for unreachable tiles

use super::revision::RevisionManager;
use super::storage::StorageEngine;
use crate::error::{CnwsError, Result};
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;

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

        // Phase 2: Sweep - remove unreachable tiles
        let all_tiles = self.store.list_tiles();
        report.total_tiles_before = all_tiles.len() as u64;

        let mut unreachable = Vec::new();
        for tile_hash in all_tiles {
            if !reachable.contains(&tile_hash) {
                unreachable.push(tile_hash);
            }
        }

        report.unreachable_tiles = unreachable.len() as u64;

        if !dry_run {
            for tile_hash in unreachable {
                match self.store.delete_tile(&tile_hash) {
                    Ok(_) => {
                        report.freed_tiles += 1;
                        report.bytes_freed += TILE_SIZE as u64;
                    }
                    Err(e) => {
                        report.errors.push(format!("Failed to delete tile {:x}: {}", tile_hash, e));
                    }
                }
            }
        }

        report.total_tiles_after = report.total_tiles_before - report.freed_tiles;
        report.duration_ms = start.elapsed().as_millis() as u64;

        Ok(report)
    }

    /// Mark phase - find all reachable tiles
    fn mark_phase(&self) -> Result<HashSet<Blake3Hash>> {
        let mut reachable = HashSet::new();

        // Get all revisions
        let dag = self.revision_manager.dag();
        let dag = dag.read();

        for revision_id in dag.revision_ids() {
            if let Some(revision) = dag.get(revision_id) {
                // Add tiles from revision
                for &tile_hash in &revision.changed_tiles {
                    reachable.insert(tile_hash);
                }

                // Add tiles from changed cells
                for &cell_hash in &revision.changed_cells {
                    // In real implementation, would load cell and get its tiles
                    // For now, just mark the cell hash
                    reachable.insert(cell_hash);
                }
            }
        }

        Ok(reachable)
    }

    /// Sweep phase - remove unreachable tiles
    fn sweep_phase(&self, unreachable: &[Blake3Hash]) -> Result<()> {
        for tile_hash in unreachable {
            self.store.delete_tile(tile_hash)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_gc_report_default() {
        let report = GcReport::default();
        assert_eq!(report.total_tiles_before, 0);
        assert_eq!(report.freed_tiles, 0);
    }
}
