//! Memory tracking module with modular structure.
//!
//! This module provides comprehensive memory allocation tracking, analysis,
//! and export functionality. It has been restructured from a single large
//! file into focused, maintainable modules.

pub mod global_functions;
pub mod memory_tracker;

// Re-export public types for backward compatibility
pub use global_functions::*;
pub use memory_tracker::{configure_tracking_strategy, get_tracker, MemoryTracker};
