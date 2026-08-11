//! CNWS Core Library
//! Canonical Neural Weight System - Core Implementation
//!
//! This library implements the CNWS specification as defined in docs/specs/
//! All invariants from the Engineering Contract (01) are enforced.

pub mod error;
pub mod types;

pub mod substrate;
pub mod lattice;
pub mod api;
pub mod telemetry;

// Tests
#[cfg(test)]
mod types_tests;

// Re-export commonly used types
pub use error::{CnwsError, Result};
pub use types::*;
