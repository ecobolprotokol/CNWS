//! Integrity integration tests

use cnws_core::{
    substrate::integrity::{IntegrityVerifier, Quarantine},
    substrate::storage::{StorageEngine, StoreConfig},
    types::Blake3Hash,
};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_integrity_verification() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let verifier = IntegrityVerifier::new(engine);

    // Empty store should pass
    let results = verifier.verify_all().unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_quarantine() {
    let mut quarantine = Quarantine::new();
    let hash = Blake3Hash::hash(b"test");

    quarantine.add(hash, "test reason".to_string(), None);
    assert!(quarantine.contains(&hash));
    assert_eq!(quarantine.len(), 1);

    let entry = quarantine.remove(&hash);
    assert!(entry.is_some());
    assert!(!quarantine.contains(&hash));
    assert!(quarantine.is_empty());
}
