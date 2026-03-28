//! Snapshot Engine - Snapshot construction and aggregation
//!
//! This module provides the SnapshotEngine which is responsible for
//! building memory snapshots from event data.

use std::sync::Arc;

pub mod engine;
pub mod types;
pub mod memory;

pub use engine::SnapshotEngine;
pub use types::{ActiveAllocation, MemorySnapshot, MemoryStats, ThreadMemoryStats};

/// Shared reference to SnapshotEngine
pub type SharedSnapshotEngine = Arc<SnapshotEngine>;