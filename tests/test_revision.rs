//! Revision integration tests

use cnws_core::{
    substrate::revision::RevisionManager,
    substrate::storage::{StorageEngine, StoreConfig},
};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_revision_commit() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let manager = Arc::new(RevisionManager::new(engine));

    // Commit root revision
    let rev1 = manager.commit(None, vec![], vec![], HashMap::new()).unwrap();
    assert!(manager.exists(&rev1));
    assert_eq!(manager.head(), Some(rev1));

    // Commit child revision
    let rev2 = manager.commit(Some(rev1), vec![], vec![], HashMap::new()).unwrap();
    assert!(manager.exists(&rev2));
    assert_eq!(manager.head(), Some(rev2));

    // Verify ancestry
    let dag = manager.dag();
    let dag = dag.read();
    assert!(dag.is_ancestor(rev1, rev2));
}

#[test]
fn test_revision_common_ancestor() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let manager = Arc::new(RevisionManager::new(engine));

    // Create a simple tree: rev1 -> rev2, rev1 -> rev3
    let rev1 = manager.commit(None, vec![], vec![], HashMap::new()).unwrap();
    let rev2 = manager.commit(Some(rev1), vec![], vec![], HashMap::new()).unwrap();
    let rev3 = manager.commit(Some(rev1), vec![], vec![], HashMap::new()).unwrap();

    // Common ancestor of rev2 and rev3 should be rev1
    let common = manager.common_ancestor(rev2, rev3);
    assert_eq!(common, Some(rev1));
}
