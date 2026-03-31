//! Lifecycle analysis modules
//!
//! This module contains types and functions for analyzing object lifetimes,
//! ownership patterns, and lifecycle events.

pub mod lifecycle_summary;
pub mod ownership_history;

// Re-export common types
pub use lifecycle_summary::{
    AllocationLifecycleSummary, ExportMetadata, LifecycleEvent, LifecycleEventSummary,
    LifecycleExportData, LifecyclePattern, LifecycleSummaryGenerator, SummaryConfig, VariableGroup,
};
pub use ownership_history::{
    BorrowInfo, CloneInfo, OwnershipEvent, OwnershipEventType, OwnershipHistoryRecorder,
    OwnershipStatistics, OwnershipSummary, RefCountInfo,
};
