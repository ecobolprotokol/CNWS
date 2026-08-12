//! Revision DAG - immutable versioning with delta at Cell/Tile level
//! Implements FAC-11 through FAC-15 invariants

use super::storage::StorageEngine;
use crate::error::{CnwsError, Result};
use crate::types::{Blake3Hash, ComputeBudget, RevisionId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

/// Revision - immutable snapshot of store state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    /// Revision ID (BLAKE3-256 of revision data)
    pub id: RevisionId,
    /// Parent revision IDs (empty for root)
    pub parents: Vec<RevisionId>,
    /// Changed cell hashes since parent
    pub changed_cells: Vec<Blake3Hash>,
    /// Changed tile hashes since parent
    pub changed_tiles: Vec<Blake3Hash>,
    /// Revision timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Revision metadata
    pub metadata: HashMap<String, String>,
    /// Compute budget used
    pub compute_budget: ComputeBudget,
}

impl Revision {
    /// Create a new revision
    pub fn new(
        id: RevisionId,
        parents: Vec<RevisionId>,
        changed_cells: Vec<Blake3Hash>,
        changed_tiles: Vec<Blake3Hash>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            parents,
            changed_cells,
            changed_tiles,
            timestamp: now,
            metadata: HashMap::new(),
            compute_budget: ComputeBudget::default(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this revision is ancestor of another
    pub fn is_ancestor_of(&self, other: &Revision, dag: &RevisionDag) -> bool {
        dag.is_ancestor(self.id, other.id)
    }

    /// Get common ancestor with another revision
    pub fn common_ancestor(&self, other: &Revision, dag: &RevisionDag) -> Option<RevisionId> {
        dag.common_ancestor(self.id, other.id)
    }
}

/// Revision DAG - directed acyclic graph of revisions
#[derive(Debug, Clone, Default)]
pub struct RevisionDag {
    /// Map from revision ID to revision
    revisions: HashMap<RevisionId, Revision>,
    /// Adjacency list: parent -> children
    children: HashMap<RevisionId, HashSet<RevisionId>>,
    /// Reverse adjacency: child -> parents
    parents: HashMap<RevisionId, HashSet<RevisionId>>,
}

impl RevisionDag {
    /// Create a new empty DAG
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a revision to the DAG
    pub fn add(&mut self, revision: Revision) -> Result<()> {
        let id = revision.id;

        // Check for cycles
        for &parent in &revision.parents {
            if self.is_ancestor(id, parent) {
                return Err(CnwsError::InvalidRevision(format!(
                    "Adding revision {:x} would create cycle with parent {:x}",
                    id, parent
                )));
            }
        }

        // Add to maps
        self.revisions.insert(id, revision.clone());

        // Update parent-child relationships
        for &parent in &revision.parents {
            self.children.entry(parent).or_default().insert(id);
            self.parents.entry(id).or_default().insert(parent);
        }

        Ok(())
    }

    /// Get a revision by ID
    pub fn get(&self, id: &RevisionId) -> Option<&Revision> {
        self.revisions.get(id)
    }

    /// Check if ancestor_id is ancestor of descendant_id
    pub fn is_ancestor(&self, ancestor_id: RevisionId, descendant_id: RevisionId) -> bool {
        if ancestor_id == descendant_id {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(descendant_id);

        while let Some(current) = queue.pop_front() {
            if current == ancestor_id {
                return true;
            }

            if !visited.insert(current) {
                continue;
            }

            if let Some(parents) = self.parents.get(&current) {
                for &parent in parents {
                    queue.push_back(parent);
                }
            }
        }

        false
    }

    /// Get all ancestors of a revision
    pub fn ancestors(&self, id: RevisionId) -> HashSet<RevisionId> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(parents) = self.parents.get(&id) {
            for &parent in parents {
                queue.push_back(parent);
            }
        }

        while let Some(current) = queue.pop_front() {
            if !ancestors.insert(current) {
                continue;
            }

            if let Some(parents) = self.parents.get(&current) {
                for &parent in parents {
                    queue.push_back(parent);
                }
            }
        }

        ancestors
    }

    /// Get common ancestor of two revisions
    pub fn common_ancestor(&self, id1: RevisionId, id2: RevisionId) -> Option<RevisionId> {
        let ancestors1 = self.ancestors(id1);
        let ancestors2 = self.ancestors(id2);

        let common: Vec<_> = ancestors1.intersection(&ancestors2).cloned().collect();

        // Find the most recent common ancestor (by timestamp)
        common.into_iter().max_by(|&a, &b| {
            let rev_a = self.revisions.get(&a);
            let rev_b = self.revisions.get(&b);
            match (rev_a, rev_b) {
                (Some(a), Some(b)) => a.timestamp.cmp(&b.timestamp),
                _ => std::cmp::Ordering::Equal,
            }
        })
    }

    /// Get children of a revision
    pub fn children(&self, id: &RevisionId) -> Option<&HashSet<RevisionId>> {
        self.children.get(id)
    }

    /// Get all revision IDs
    pub fn revision_ids(&self) -> impl Iterator<Item = &RevisionId> {
        self.revisions.keys()
    }

    /// Get number of revisions
    pub fn len(&self) -> usize {
        self.revisions.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }
}

/// Revision manager - manages revisions and their storage
pub struct RevisionManager {
    store: Arc<StorageEngine>,
    dag: Arc<RwLock<RevisionDag>>,
    head: Arc<RwLock<Option<RevisionId>>>,
}

impl RevisionManager {
    /// Create a new revision manager
    pub fn new(store: Arc<StorageEngine>) -> Self {
        Self {
            store,
            dag: Arc::new(RwLock::new(RevisionDag::new())),
            head: Arc::new(RwLock::new(None)),
        }
    }

    /// Load revisions from store
    pub fn load(&self) -> Result<()> {
        // In real implementation, would load from revisions directory
        Ok(())
    }

    /// Commit a new revision
    pub fn commit(
        &self,
        parent: Option<RevisionId>,
        changed_cells: Vec<Blake3Hash>,
        changed_tiles: Vec<Blake3Hash>,
        metadata: HashMap<String, String>,
    ) -> Result<RevisionId> {
        // Compute revision ID from content
        let mut hasher = blake3::Hasher::new();
        if let Some(p) = parent {
            hasher.update(&p.0);
        }
        for cell in &changed_cells {
            hasher.update(&cell.0);
        }
        for tile in &changed_tiles {
            hasher.update(&tile.0);
        }
        let hash_bytes: [u8; 32] = hasher.finalize().into();
        let id = Blake3Hash(hash_bytes);

        // Create revision
        let mut revision = Revision::new(
            id,
            parent.into_iter().collect(),
            changed_cells,
            changed_tiles,
        );

        for (k, v) in metadata {
            revision = revision.with_metadata(k, v);
        }

        // Add to DAG
        {
            let mut dag = self.dag.write();
            dag.add(revision.clone())?;
        }

        // Update head
        {
            let mut head = self.head.write();
            *head = Some(id);
        }

        // Save revision
        self.save_revision(&revision)?;

        Ok(id)
    }

    /// Save revision to store
    fn save_revision(&self, revision: &Revision) -> Result<()> {
        let data = bincode::serialize(revision)
            .map_err(|e| CnwsError::Serialization(e.to_string()))?;

        let hash = Blake3Hash::hash(&data);
        let store_path = &self.store.config.path;
        let revisions_path = store_path.join("revisions");

        std::fs::create_dir_all(&revisions_path)?;
        std::fs::write(revisions_path.join(format!("{:x}.cd", hash)), data)?;

        Ok(())
    }

    /// Get current head revision
    pub fn head(&self) -> Option<RevisionId> {
        *self.head.read()
    }

    /// Get revision by ID
    pub fn get(&self, id: &RevisionId) -> Option<Revision> {
        self.dag.read().get(id).cloned()
    }

    /// Check if revision exists
    pub fn exists(&self, id: &RevisionId) -> bool {
        self.dag.read().get(id).is_some()
    }

    /// Get DAG
    pub fn dag(&self) -> Arc<RwLock<RevisionDag>> {
        Arc::clone(&self.dag)
    }

    /// Get ancestors of a revision
    pub fn ancestors(&self, id: RevisionId) -> HashSet<RevisionId> {
        self.dag.read().ancestors(id)
    }

    /// Get common ancestor of two revisions
    pub fn common_ancestor(&self, id1: RevisionId, id2: RevisionId) -> Option<RevisionId> {
        self.dag.read().common_ancestor(id1, id2)
    }

    /// Perform a 3-way merge of two branches
    pub fn merge(
        &self,
        branch_a: RevisionId,
        branch_b: RevisionId,
    ) -> Result<RevisionId> {
        let _ancestor = self
            .common_ancestor(branch_a, branch_b)
            .ok_or_else(|| {
                CnwsError::InvalidRevision("No common ancestor found for merge".into())
            })?;

        let rev_a = self
            .get(&branch_a)
            .ok_or(CnwsError::RevisionNotFound)?;
        let rev_b = self
            .get(&branch_b)
            .ok_or(CnwsError::RevisionNotFound)?;

        let mut merged_cells = rev_a.changed_cells.clone();
        merged_cells.extend(rev_b.changed_cells.iter().cloned());

        let mut merged_tiles = rev_a.changed_tiles.clone();
        merged_tiles.extend(rev_b.changed_tiles.iter().cloned());

        let mut hasher = blake3::Hasher::new();
        hasher.update(&branch_a.0);
        hasher.update(&branch_b.0);
        for cell in &merged_cells {
            hasher.update(&cell.0);
        }
        for tile in &merged_tiles {
            hasher.update(&tile.0);
        }
        let hash_bytes: [u8; 32] = hasher.finalize().into();
        let merge_id = Blake3Hash(hash_bytes);

        let revision = Revision::new(
            merge_id,
            vec![branch_a, branch_b],
            merged_cells,
            merged_tiles,
        );

        {
            let mut dag = self.dag.write();
            dag.add(revision.clone())?;
        }

        {
            let mut head = self.head.write();
            *head = Some(merge_id);
        }

        self.save_revision(&revision)?;

        Ok(merge_id)
    }

    /// Rollback head to a target revision without deleting any revisions
    pub fn rollback(&self, target: RevisionId) -> Result<RevisionId> {
        if !self.exists(&target) {
            return Err(CnwsError::RevisionNotFound);
        }

        let old_head = self.head();
        let mut head = self.head.write();
        *head = Some(target);

        Ok(old_head.unwrap_or(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_revision_dag_cycle_detection() {
        let mut dag = RevisionDag::new();

        let id1 = Blake3Hash::default();
        let id2 = Blake3Hash([2u8; 32]);
        let id3 = Blake3Hash([3u8; 32]);

        let rev1 = Revision::new(id1, vec![], vec![], vec![]);
        let rev2 = Revision::new(id2, vec![id1], vec![], vec![]);
        let rev3 = Revision::new(id3, vec![id2], vec![], vec![]);

        dag.add(rev1).unwrap();
        dag.add(rev2).unwrap();
        dag.add(rev3).unwrap();

        // This should fail - would create cycle
        let rev_cycle = Revision::new(id1, vec![id3], vec![], vec![]);
        assert!(dag.add(rev_cycle).is_err());
    }

    #[test]
    fn test_revision_manager() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = RevisionManager::new(engine);

        let id = manager.commit(
            None,
            vec![],
            vec![],
            HashMap::new(),
        ).unwrap();

        assert!(manager.exists(&id));
        assert_eq!(manager.head(), Some(id));
    }

    #[test]
    fn test_revision_manager_merge() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = RevisionManager::new(engine);

        let root = manager
            .commit(None, vec![], vec![], HashMap::new())
            .unwrap();

        let cell_a = Blake3Hash([1u8; 32]);
        let tile_a = Blake3Hash([2u8; 32]);
        let branch_a = manager
            .commit(
                Some(root),
                vec![cell_a],
                vec![tile_a],
                HashMap::new(),
            )
            .unwrap();

        let cell_b = Blake3Hash([3u8; 32]);
        let tile_b = Blake3Hash([4u8; 32]);
        let branch_b = manager
            .commit(
                Some(root),
                vec![cell_b],
                vec![tile_b],
                HashMap::new(),
            )
            .unwrap();

        let merge_id = manager.merge(branch_a, branch_b).unwrap();

        let merge_rev = manager.get(&merge_id).unwrap();
        assert_eq!(merge_rev.parents, vec![branch_a, branch_b]);
        assert_eq!(merge_rev.changed_cells.len(), 2);
        assert_eq!(merge_rev.changed_tiles.len(), 2);
        assert!(merge_rev.changed_cells.contains(&cell_a));
        assert!(merge_rev.changed_cells.contains(&cell_b));
        assert!(merge_rev.changed_tiles.contains(&tile_a));
        assert!(merge_rev.changed_tiles.contains(&tile_b));
        assert_eq!(manager.head(), Some(merge_id));
    }

    #[test]
    fn test_revision_manager_rollback() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = RevisionManager::new(engine);

        let rev1 = manager
            .commit(None, vec![], vec![], HashMap::new())
            .unwrap();
        let rev2 = manager
            .commit(
                Some(rev1),
                vec![Blake3Hash([10u8; 32])],
                vec![],
                HashMap::new(),
            )
            .unwrap();

        assert_eq!(manager.head(), Some(rev2));

        let old_head = manager.rollback(rev1).unwrap();
        assert_eq!(old_head, rev2);
        assert_eq!(manager.head(), Some(rev1));
        assert!(manager.exists(&rev2));
    }
}
