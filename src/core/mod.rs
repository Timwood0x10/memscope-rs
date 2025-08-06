//! Core memory tracking functionality
//!
//! This module contains the fundamental components for memory tracking:
//! - Memory tracker implementation
//! - Custom allocator
//! - Type definitions
//! - Scope tracking

pub mod allocator;
pub mod error;
pub mod error_adapter;
pub mod scope_tracker;
pub mod tracker;
/// Type definitions
pub mod types;
pub mod unwrap_safe;

// Re-export key types for easier access
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};

// Re-export the new unified error system
pub use error::{MemScopeError, Result as MemScopeResult, ErrorRecovery, DefaultErrorRecovery, RecoveryAction, MemoryOperation, SystemErrorType, ErrorSeverity};
pub use error_adapter::{ErrorAdapter, DefaultErrorAdapter, from_tracking_error, to_tracking_error, adapt_result, to_tracking_result};

// Re-export safe unwrap utilities
pub use unwrap_safe::{UnwrapSafe, UnwrapStats, get_unwrap_stats, update_unwrap_stats};

#[cfg(test)]
pub use unwrap_safe::get_unwrap_stats_mut;
