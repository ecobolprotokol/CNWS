//! Revision API - public interface for revision management

use super::super::substrate::revision::RevisionManager;
use crate::error::Result;
use crate::types::Blake3Hash;
use std::collections::HashMap;
use std::sync::Arc;

/// Revision API
pub struct RevisionApi {
    manager: Arc<RevisionManager>,
}

impl RevisionApi {
    /// Create a new revision API
    pub fn new(manager: Arc<RevisionManager>) -> Self {
        Self { manager }
    }

    /// Commit a new revision
    pub fn commit(
        &self,
        parent: Option<Blake3Hash>,
        changed_cells: Vec<Blake3Hash>,
        changed_tiles: Vec<Blake3Hash>,
        metadata: HashMap<String, String>,
    ) -> Result<Blake3Hash> {
        self.manager.commit(parent, changed_cells, changed_tiles, metadata)
    }

    /// Get current head revision
    pub fn head(&self) -> Option<Blake3Hash> {
        self.manager.head()
    }

    /// Get revision by ID
    pub fn get(&self, id: &Blake3Hash) -> Option<crate::substrate::revision::Revision> {
        self.manager.get(id)
    }

    /// Check if revision exists
    pub fn exists(&self, id: &Blake3Hash) -> bool {
        self.manager.exists(id)
    }

    /// Get ancestors of a revision
    pub fn ancestors(&self, id: Blake3Hash) -> Vec<Blake3Hash> {
        self.manager.ancestors(id).into_iter().collect()
    }

    /// Get common ancestor of two revisions
    pub fn common_ancestor(&self, id1: Blake3Hash, id2: Blake3Hash) -> Option<Blake3Hash> {
        self.manager.common_ancestor(id1, id2)
    }

    /// Load revisions from store
    pub fn load(&self) -> Result<()> {
        self.manager.load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_revision_api() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = std::sync::Arc::new(crate::substrate::revision::RevisionManager::new(engine));
        let api = RevisionApi::new(manager);

        let id = api.commit(None, vec![], vec![], HashMap::new()).unwrap();
        assert!(api.exists(&id));
        assert_eq!(api.head(), Some(id));
    }

    #[test]
    fn test_revision_merge() {
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use std::sync::Arc;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = Arc::new(crate::substrate::revision::RevisionManager::new(engine));

        let root = manager.commit(None, vec![], vec![], HashMap::new()).unwrap();

        let a = manager.commit(Some(root), vec![], vec![], HashMap::new()).unwrap();

        let b = manager.commit(Some(root), vec![], vec![], HashMap::new()).unwrap();

        let merged = manager.merge(a, b).unwrap();

        let rev = manager.get(&merged).unwrap();
        assert_eq!(rev.parents.len(), 2);
        assert!(rev.parents.contains(&a));
        assert!(rev.parents.contains(&b));
    }

    #[test]
    fn test_revision_rollback() {
        use crate::substrate::storage::{StorageEngine, StoreConfig};
        use std::sync::Arc;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let engine = Arc::new(engine);
        let manager = Arc::new(crate::substrate::revision::RevisionManager::new(engine));

        let rev1 = manager.commit(None, vec![], vec![], HashMap::new()).unwrap();
        let rev2 = manager.commit(Some(rev1), vec![], vec![], HashMap::new()).unwrap();

        assert_eq!(manager.head(), Some(rev2));

        let old_head = manager.rollback(rev1).unwrap();
        assert_eq!(old_head, rev2);
        assert_eq!(manager.head(), Some(rev1));

        assert!(manager.exists(&rev2));
    }
}
