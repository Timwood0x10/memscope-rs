//! Core memory tracking functionality - Re-exports from capture module

// Re-export everything from capture::backends::core_tracker
pub use crate::capture::backends::core_tracker::{
    cleanup_registry_local, collect_all_trackers_local, configure_tracking_strategy,
    get_registry_stats_local, get_tracker, has_active_trackers_local, MemoryTracker,
};
