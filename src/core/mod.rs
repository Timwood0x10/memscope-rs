//! Core memory tracking functionality
//!
//! This module contains the fundamental components for memory tracking:
//! - Memory tracker implementation
//! - Custom allocator
//! - Type definitions
//! - Scope tracking

pub mod allocator;
pub mod call_stack_normalizer;
pub mod error;
pub mod safe_operations;
pub mod scope_tracker;
pub mod tracker;
pub mod types;
pub mod unwrap_safe;

// Re-export key types for easier access
pub use allocator::TrackingAllocator;
pub use tracker::{get_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};

pub use crate::capture::backends::{ExportMode, ExportOptions};

pub use error::{
    DefaultErrorRecovery, ErrorRecovery, ErrorSeverity, MemScopeError, MemoryOperation,
    RecoveryAction, Result as MemScopeResult, SystemErrorType,
};

// Re-export safe unwrap utilities
pub use unwrap_safe::{get_unwrap_stats, update_unwrap_stats, UnwrapSafe, UnwrapStats};

// Re-export call stack normalizer functionality
pub use call_stack_normalizer::{
    get_global_call_stack_normalizer, initialize_global_call_stack_normalizer, CallStackId,
    CallStackNormalizer, CallStackRef, NormalizedCallStack, NormalizerConfig, NormalizerStats,
};

pub use safe_operations::SafeLock;

#[cfg(test)]
pub use unwrap_safe::get_unwrap_stats_mut;
