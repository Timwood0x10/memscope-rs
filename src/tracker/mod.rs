//! Unified tracking system
//!
//! This module provides the unified tracking backend system that consolidates
//! the 4 existing tracking systems into a single configurable architecture.

pub mod backend;

// Re-export key types for convenience
pub use backend::{
    configure_tracking_strategy, get_global_tracker as backend_get_global_tracker, snapshot,
    track_allocation, track_deallocation, track_ffi_alloc, track_ffi_free, track_task_end,
    track_task_spawn, AllocationContext, AsyncBackend, HybridBackend, OverheadLimit,
    SamplingConfig, SingleThreadBackend, ThreadLocalBackend, TrackingAllocator, TrackingBackend,
    TrackingConfig, TrackingStrategy, UnifiedStorage, UnifiedTracker,
};

// Global unified tracker instance
use std::sync::{Arc, OnceLock};

static GLOBAL_TRACKER: OnceLock<Arc<UnifiedTracker>> = OnceLock::new();

/// Initialize the global unified tracker with default configuration
pub fn initialize() {
    GLOBAL_TRACKER.get_or_init(|| Arc::new(UnifiedTracker::default()));
}

/// Get the global unified tracker instance
pub fn get_global_tracker() -> Option<Arc<UnifiedTracker>> {
    GLOBAL_TRACKER.get().cloned()
}

/// Initialize the global unified tracker with custom configuration
pub fn initialize_with_config(config: TrackingConfig) {
    GLOBAL_TRACKER.get_or_init(|| Arc::new(UnifiedTracker::new(config)));
}
