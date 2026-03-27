//! Unified data model layer
//!
//! This module contains the unified data structures that are used
//! across all tracking strategies and renderers.

pub mod snapshot;
pub mod allocation;
pub mod event;
pub mod task;
pub mod stats;
pub mod common;

// Re-export commonly used types
pub use snapshot::TrackingSnapshot;
pub use allocation::AllocationRecord;
pub use event::{MemoryEvent, EventType};
pub use task::{TaskRecord, TaskStatus};
pub use stats::TrackingStats;
pub use common::{TrackingStrategy, ExportFormat, RenderOutput, RenderResult};