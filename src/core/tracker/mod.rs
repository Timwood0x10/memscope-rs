//! Memory tracking functionality.
//!
//! This module provides comprehensive memory allocation tracking, analysis,
//! and export capabilities. The main entry point is the `MemoryTracker` struct
//! which can be accessed via `get_global_tracker()`.

pub mod export_options;
pub mod memory_tracker;
pub mod allocation_tracking;
pub mod export_methods;
pub mod missing_methods;

// Re-export main types for convenience
pub use export_options::{ExportOptions, ExportMode};
pub use memory_tracker::{MemoryTracker, get_global_tracker};

// TODO: Add other modules as we split them
// pub mod analysis_methods;
// pub mod tracking_manager;
// pub mod reports;