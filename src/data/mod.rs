//! Unified data model layer
//!
//! This module contains the unified data structures that are used
//! across all tracking strategies and renderers.

pub mod allocation;
pub mod common;
pub mod event;
pub mod snapshot;
pub mod stats;
pub mod task;

// Re-export commonly used types
pub use allocation::{AllocationRecord, BorrowInfo, CloneInfo};
pub use common::{ExportFormat, RenderOutput, RenderResult, TrackingStrategy};
pub use event::{EventType, MemoryEvent};
pub use snapshot::TrackingSnapshot;
pub use stats::TrackingStats;
pub use task::{TaskRecord, TaskStatus};
