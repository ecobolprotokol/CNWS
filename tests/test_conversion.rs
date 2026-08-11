//! Conversion integration tests

use cnws_core::{
    substrate::conversion::ConversionPipeline,
    substrate::storage::{StorageEngine, StoreConfig},
    types::Compression,
};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_conversion_pipeline_creation() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let _pipeline = ConversionPipeline::new(engine);
}

#[test]
fn test_convert_tensor() {
    let dir = tempdir().unwrap();
    let config = StoreConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    let engine = StorageEngine::create_store(config).unwrap();
    let engine = Arc::new(engine);
    let pipeline = ConversionPipeline::new(engine);

    let data = b"tensor data";
    let hash = pipeline.convert_tensor("test_tensor", data, cnws_core::types::DataType::F32, &[10, 10]).unwrap();

    // Verify tile was written
    assert!(engine.has_tile(&hash));
}
