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

// Re-export key types for easier access
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};

// Re-export the new unified error system
pub use error::{MemScopeError, Result as MemScopeResult, ErrorRecovery, DefaultErrorRecovery, RecoveryAction, MemoryOperation, SystemErrorType, ErrorSeverity};
pub use error_adapter::{ErrorAdapter, DefaultErrorAdapter, from_tracking_error, to_tracking_error, adapt_result, to_tracking_result};
