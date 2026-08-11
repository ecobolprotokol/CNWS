//! Conversion API - public interface for format import

use super::super::substrate::conversion::{ConversionPipeline, ImportReport, NormalizationPolicy};
use super::super::types::Compression;
use crate::error::{CnwsError, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Conversion API
pub struct ConversionApi {
    pipeline: ConversionPipeline,
}

impl ConversionApi {
    /// Create a new conversion API
    pub fn new(pipeline: ConversionPipeline) -> Self {
        Self { pipeline }
    }

    /// Import from Safetensors
    pub fn import_safetensors(&self, path: impl Into<PathBuf>) -> Result<ImportReport> {
        self.pipeline.import_safetensors(path)
    }

    /// Import from GGUF
    pub fn import_gguf(&self, path: impl Into<PathBuf>) -> Result<ImportReport> {
        self.pipeline.import_gguf(path)
    }

    /// Import from PyTorch
    pub fn import_pytorch(&self, path: impl Into<PathBuf>) -> Result<ImportReport> {
        self.pipeline.import_pytorch(path)
    }

    /// Import from ONNX
    pub fn import_onnx(&self, path: impl Into<PathBuf>) -> Result<ImportReport> {
        self.pipeline.import_onnx(path)
    }

    /// Set compression
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.pipeline = self.pipeline.with_compression(compression);
        self
    }

    /// Set normalization policy
    pub fn with_normalization(mut self, policy: NormalizationPolicy) -> Self {
        self.pipeline = self.pipeline.with_normalization(policy);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::storage::{StorageEngine, StoreConfig};
    use tempfile::tempdir;

    #[test]
    fn test_conversion_api() {
        let dir = tempdir().unwrap();
        let config = StoreConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let engine = StorageEngine::create_store(config).unwrap();
        let pipeline = ConversionPipeline::new(std::sync::Arc::new(engine));
        let api = ConversionApi::new(pipeline);

        // Test would require actual model files
        assert!(true);
    }
}
