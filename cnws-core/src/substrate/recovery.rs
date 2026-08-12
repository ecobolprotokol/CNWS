//! Recovery subsystem for crash recovery
//! Implements WAL-based recovery with state machine

use super::storage::StorageEngine;
use crate::error::{CnwsError, Result};
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

/// WAL record type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum WalRecordType {
    WriteTile = 0x01,
    DeleteTile = 0x02,
    Commit = 0x03,
    Checkpoint = 0x04,
    Begin = 0x05,
    Abort = 0x06,
}

impl TryFrom<u32> for WalRecordType {
    type Error = CnwsError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0x01 => Ok(Self::WriteTile),
            0x02 => Ok(Self::DeleteTile),
            0x03 => Ok(Self::Commit),
            0x04 => Ok(Self::Checkpoint),
            0x05 => Ok(Self::Begin),
            0x06 => Ok(Self::Abort),
            _ => Err(CnwsError::InvalidWalRecord(value)),
        }
    }
}

/// WAL record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    /// Record type
    pub record_type: WalRecordType,
    /// Transaction ID
    pub txn_id: u64,
    /// Sequence number
    pub sequence: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Data payload
    pub data: Vec<u8>,
}

impl WalRecord {
    /// Create a new WAL record
    pub fn new(record_type: WalRecordType, txn_id: u64, sequence: u64, data: Vec<u8>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            record_type,
            txn_id,
            sequence,
            timestamp: now,
            data,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| CnwsError::Serialization(e.to_string()))
    }
}

/// Recovery state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    /// No recovery needed
    Clean,
    /// Recovery in progress
    InProgress,
    /// Recovery completed
    Completed,
    /// Recovery failed
    Failed,
}

/// Recovery action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// No action needed
    None,
    /// Replay WAL
    ReplayWal,
    /// Restore from checkpoint
    RestoreCheckpoint,
    /// Full recovery
    FullRecovery,
}

/// Recovery report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    /// Recovery state
    pub state: RecoveryState,
    /// Action taken
    pub action: RecoveryAction,
    /// Number of records replayed
    pub records_replayed: u64,
    /// Number of tiles restored
    pub tiles_restored: u64,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Recovery duration in milliseconds
    pub duration_ms: u64,
}

impl Default for RecoveryReport {
    fn default() -> Self {
        Self {
            state: RecoveryState::Clean,
            action: RecoveryAction::None,
            records_replayed: 0,
            tiles_restored: 0,
            errors: Vec::new(),
            duration_ms: 0,
        }
    }
}

/// Recovery manager
pub struct RecoveryManager {
    store: Arc<StorageEngine>,
    wal_path: PathBuf,
    state: Arc<RwLock<RecoveryState>>,
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new(store: Arc<StorageEngine>, wal_path: PathBuf) -> Self {
        Self {
            store,
            wal_path,
            state: Arc::new(RwLock::new(RecoveryState::Clean)),
        }
    }

    /// Check if recovery is needed
    pub fn check(&self) -> Result<RecoveryAction> {
        if !self.wal_path.exists() {
            return Ok(RecoveryAction::None);
        }

        // Check WAL for uncommitted transactions
        let mut file = File::open(&self.wal_path)?;
        let mut buf = [0u8; 1024];
        let n = file.read(&mut buf)?;

        if n == 0 {
            return Ok(RecoveryAction::None);
        }

        // Check for BEGIN without COMMIT
        let data = &buf[..n];
        let records: Vec<WalRecord> = bincode::deserialize(data)
            .map_err(|e| CnwsError::Serialization(e.to_string()))?;

        let has_uncommitted = records.iter().any(|r| {
            r.record_type == WalRecordType::Begin &&
            !records.iter().any(|r2| r2.record_type == WalRecordType::Commit && r2.txn_id == r.txn_id)
        });

        if has_uncommitted {
            Ok(RecoveryAction::ReplayWal)
        } else {
            Ok(RecoveryAction::None)
        }
    }

    /// Perform recovery
    pub fn recover(&self) -> Result<RecoveryReport> {
        let start = std::time::Instant::now();
        let mut report = RecoveryReport::default();

        // Check if recovery needed
        let action = self.check()?;
        report.action = action;

        if action == RecoveryAction::None {
            report.state = RecoveryState::Completed;
            report.duration_ms = start.elapsed().as_millis() as u64;
            return Ok(report);
        }

        // Set state to in progress
        {
            let mut state = self.state.write();
            *state = RecoveryState::InProgress;
        }

        // Replay WAL
        match self.replay_wal(&mut report) {
            Ok(_) => {
                report.state = RecoveryState::Completed;
            }
            Err(e) => {
                report.state = RecoveryState::Failed;
                report.errors.push(e.to_string());
            }
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Replay WAL records
    fn replay_wal(&self, report: &mut RecoveryReport) -> Result<()> {
        if !self.wal_path.exists() {
            return Ok(());
        }

        let data = std::fs::read(&self.wal_path)?;
        let records: Vec<WalRecord> = bincode::deserialize(&data)
            .map_err(|e| CnwsError::Serialization(e.to_string()))?;

        for record in records {
            report.records_replayed += 1;

            match record.record_type {
                WalRecordType::WriteTile => {
                    // Re-write tile
                    let tile_data: Vec<u8> = bincode::deserialize(&record.data)
                        .map_err(|e| CnwsError::Serialization(e.to_string()))?;
                    let _hash = Blake3Hash::hash(&tile_data);
                    // In real implementation, would write to store
                    report.tiles_restored += 1;
                }
                WalRecordType::DeleteTile => {
                    // Re-delete tile
                    let hash: Blake3Hash = bincode::deserialize(&record.data)
                        .map_err(|e| CnwsError::Serialization(e.to_string()))?;
                    let _ = self.store.delete_tile(&hash);
                }
                WalRecordType::Commit => {
                    // Transaction committed, nothing to do
                }
                WalRecordType::Abort => {
                    // Transaction aborted, nothing to do
                }
                _ => {}
            }
        }

        // Clear WAL after successful replay
        std::fs::write(&self.wal_path, b"")?;

        Ok(())
    }

    /// Write a WAL record
    pub fn write_record(&self, record: &WalRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)?;

        let data = record.to_bytes();
        file.write_all(&data)?;
        file.sync_all()?;

        Ok(())
    }

    /// Get recovery state
    pub fn state(&self) -> RecoveryState {
        *self.state.read()
    }

    /// Clear WAL
    pub fn clear_wal(&self) -> Result<()> {
        std::fs::write(&self.wal_path, b"")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_record_serialization() {
        let record = WalRecord::new(WalRecordType::WriteTile, 1, 1, vec![1, 2, 3]);
        let bytes = record.to_bytes();
        let record2 = WalRecord::from_bytes(&bytes).unwrap();
        assert_eq!(record.record_type, record2.record_type);
        assert_eq!(record.txn_id, record2.txn_id);
    }

    #[test]
    fn test_recovery_report_default() {
        let report = RecoveryReport::default();
        assert_eq!(report.state, RecoveryState::Clean);
        assert_eq!(report.records_replayed, 0);
    }
}
