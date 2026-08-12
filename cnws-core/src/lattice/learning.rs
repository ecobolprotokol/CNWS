//! Learning engine - structural learning for Cell Graph optimization
//! Implements composition pattern detection and graph optimization

use crate::error::Result;
use crate::types::Blake3Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Learning update type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningUpdateType {
    /// New composition pattern discovered
    NewPattern,
    /// Existing pattern updated
    PatternUpdate,
    /// Cell merged
    CellMerge,
    /// Tile merged
    TileMerge,
    /// Routing optimization
    RoutingOptimization,
    /// Cache optimization
    CacheOptimization,
}

/// Learning update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningUpdate {
    /// Update type
    pub update_type: LearningUpdateType,
    /// Affected cell hashes
    pub cells: Vec<Blake3Hash>,
    /// Affected tile hashes
    pub tiles: Vec<Blake3Hash>,
    /// Pattern data (if applicable)
    pub pattern: Option<CompositionPattern>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Timestamp
    pub timestamp: u64,
}

impl LearningUpdate {
    /// Create a new learning update
    pub fn new(update_type: LearningUpdateType, cells: Vec<Blake3Hash>, tiles: Vec<Blake3Hash>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            update_type,
            cells,
            tiles,
            pattern: None,
            confidence: 1.0,
            timestamp: now,
        }
    }

    /// Set pattern
    pub fn with_pattern(mut self, pattern: CompositionPattern) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Composition pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionPattern {
    /// Pattern ID
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Cell sequence
    pub cells: Vec<Blake3Hash>,
    /// Frequency of occurrence
    pub frequency: u64,
    /// Average compute cost
    pub avg_compute: f64,
    /// Description
    pub description: String,
}

impl CompositionPattern {
    /// Create a new composition pattern
    pub fn new(id: impl Into<String>, name: impl Into<String>, cells: Vec<Blake3Hash>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            cells,
            frequency: 1,
            avg_compute: 0.0,
            description: String::new(),
        }
    }

    /// Increment frequency
    pub fn increment_frequency(&mut self) {
        self.frequency += 1;
    }
}

/// Tile reference
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TileRef {
    /// Tile hash
    pub hash: Blake3Hash,
    /// Reference count
    pub ref_count: u64,
}

impl TileRef {
    /// Create a new tile reference
    pub fn new(hash: Blake3Hash) -> Self {
        Self {
            hash,
            ref_count: 1,
        }
    }

    /// Increment reference count
    pub fn increment(&mut self) {
        self.ref_count += 1;
    }

    /// Decrement reference count
    pub fn decrement(&mut self) {
        self.ref_count = self.ref_count.saturating_sub(1);
    }
}

/// Learning engine
pub struct LearningEngine {
    patterns: Arc<RwLock<HashMap<String, CompositionPattern>>>,
    updates: Arc<RwLock<Vec<LearningUpdate>>>,
    tile_refs: Arc<RwLock<HashMap<Blake3Hash, TileRef>>>,
}

impl LearningEngine {
    /// Create a new learning engine
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(RwLock::new(HashMap::new())),
            updates: Arc::new(RwLock::new(Vec::new())),
            tile_refs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Apply a learning update
    pub fn apply_update(&self, update: LearningUpdate) -> Result<()> {
        match update.update_type {
            LearningUpdateType::NewPattern => {
                if let Some(pattern) = &update.pattern {
                    let mut patterns = self.patterns.write();
                    patterns.insert(pattern.id.clone(), pattern.clone());
                }
            }
            LearningUpdateType::PatternUpdate => {
                if let Some(pattern) = &update.pattern {
                    let mut patterns = self.patterns.write();
                    if let Some(existing) = patterns.get_mut(&pattern.id) {
                        existing.frequency += pattern.frequency;
                        existing.avg_compute = (existing.avg_compute + pattern.avg_compute) / 2.0;
                    }
                }
            }
            LearningUpdateType::CellMerge => {
                // Update tile references
                let _tile_refs = self.tile_refs.write();
                for &_cell_hash in &update.cells {
                    // In real implementation, would merge cell tiles
                }
            }
            LearningUpdateType::TileMerge => {
                // Merge tiles
                let mut tile_refs = self.tile_refs.write();
                for &tile_hash in &update.tiles {
                    if let Some(tile_ref) = tile_refs.get_mut(&tile_hash) {
                        tile_ref.increment();
                    } else {
                        tile_refs.insert(tile_hash, TileRef::new(tile_hash));
                    }
                }
            }
            LearningUpdateType::RoutingOptimization => {
                // Update routing based on learned patterns
            }
            LearningUpdateType::CacheOptimization => {
                // Update cache based on learned patterns
            }
        }

        // Record update
        self.updates.write().push(update);

        Ok(())
    }

    /// Discover composition patterns
    pub fn discover_patterns(&self, cell_sequences: &[Vec<Blake3Hash>]) -> Result<Vec<CompositionPattern>> {
        let mut patterns = Vec::new();
        let mut pattern_map: HashMap<String, (Vec<Blake3Hash>, u64)> = HashMap::new();

        for sequence in cell_sequences {
            let key = sequence.iter()
                .map(|h| format!("{:x}", h))
                .collect::<Vec<_>>()
                .join("->");

            pattern_map.entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((sequence.clone(), 1));
        }

        for (_key, (cells, frequency)) in pattern_map {
            if frequency >= 2 {
                let mut pattern = CompositionPattern::new(
                    format!("pattern_{}", patterns.len()),
                    format!("Pattern {}", patterns.len()),
                    cells,
                );
                pattern.frequency = frequency;
                patterns.push(pattern);
            }
        }

        Ok(patterns)
    }

    /// Get all patterns
    pub fn patterns(&self) -> Vec<CompositionPattern> {
        self.patterns.read().values().cloned().collect()
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, id: &str) -> Option<CompositionPattern> {
        self.patterns.read().get(id).cloned()
    }

    /// Get updates
    pub fn updates(&self) -> Vec<LearningUpdate> {
        self.updates.read().clone()
    }

    /// Get tile reference count
    pub fn tile_ref_count(&self, hash: &Blake3Hash) -> Option<u64> {
        self.tile_refs.read().get(hash).map(|r| r.ref_count)
    }

    /// Get all tile references
    pub fn tile_refs(&self) -> Vec<TileRef> {
        self.tile_refs.read().values().cloned().collect()
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Blake3Hash;

    #[test]
    fn test_learning_update_creation() {
        let hash = Blake3Hash::hash(b"test");
        let update = LearningUpdate::new(
            LearningUpdateType::NewPattern,
            vec![hash],
            vec![],
        );
        assert_eq!(update.update_type, LearningUpdateType::NewPattern);
        assert_eq!(update.confidence, 1.0);
    }

    #[test]
    fn test_composition_pattern() {
        let hash = Blake3Hash::hash(b"test");
        let mut pattern = CompositionPattern::new("p1", "Pattern 1", vec![hash]);
        pattern.increment_frequency();
        assert_eq!(pattern.frequency, 2);
    }

    #[test]
    fn test_tile_ref() {
        let hash = Blake3Hash::hash(b"test");
        let mut tile_ref = TileRef::new(hash);
        assert_eq!(tile_ref.ref_count, 1);
        tile_ref.increment();
        assert_eq!(tile_ref.ref_count, 2);
        tile_ref.decrement();
        assert_eq!(tile_ref.ref_count, 1);
    }
}
