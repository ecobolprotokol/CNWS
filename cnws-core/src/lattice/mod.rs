//! Lattice layer - Cell Graph, memory, routing, learning, cache
//! Implements the logical computation layer of CNWS

pub mod runtime;
pub mod memory;
pub mod routing;
pub mod learning;
pub mod cache;

pub use runtime::{ExecutionEngine, MockResolver, Query, QueryBuilder, RuntimeResolver, WorkingState, CellRef};
pub use memory::{MemoryEntry, MemoryIndexEntry, MemorySystem};
pub use routing::{CellMetadata, RoutingEngine, RoutingPolicy, RoutingStatistics};
pub use learning::{CompositionPattern, LearningEngine, LearningUpdate, LearningUpdateType, TileRef};
pub use cache::{CacheEntry, CacheLevel, CacheManager, CacheStatistics, LruCache};
