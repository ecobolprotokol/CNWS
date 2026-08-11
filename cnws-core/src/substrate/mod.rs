//! Substrate layer - immutable storage, revisioning, integrity, recovery
//! Implements the physical storage layer of CNWS

pub mod storage;
pub mod integrity;
pub mod revision;
pub mod gc;
pub mod recovery;
pub mod conversion;
pub mod manifest;

pub use crate::types::TileLocation;
pub use storage::{StorageEngine, StoreConfig, StoreStats, Superblock, SegmentHeader, TileRegistry};
pub use integrity::{IntegrityVerifier, Quarantine, QuarantineEntry, VerificationResult};
pub use revision::{Revision, RevisionDag, RevisionManager};
pub use gc::{GarbageCollector, GcReport};
pub use recovery::{RecoveryManager, RecoveryReport, RecoveryState, RecoveryAction, WalRecord, WalRecordType};
pub use conversion::{ConversionPipeline, ImportReport, NormalizationPolicy};
pub use manifest::{Manifest, ManifestMetadata, CellRecord, TileRecord};
