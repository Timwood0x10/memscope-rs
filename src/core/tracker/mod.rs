//! Memory tracking module with modular structure.
//!
//! This module provides comprehensive memory allocation tracking, analysis,
//! and export functionality. It has been restructured from a single large
//! file into focused, maintainable modules.
//!
//! # Deprecated
//!
//! This module is deprecated. Please use the new unified tracking system
//! located in `src/tracker/` and `src/manager/` with `TrackingManager`.
//!
//! Migration Guide:
//! - Replace `get_tracker()` with `manager::get_global_tracker()`
//! - Replace `MemoryTracker` with `manager::TrackingManager`
//! - Use `TrackingManager` for all tracking operations
//! - All functionality is preserved for backward compatibility

pub mod allocation_tracking;
pub mod config;
pub mod export_html;
pub mod export_json;
pub mod global_functions;
pub mod memory_analysis;
pub mod memory_tracker;
pub mod tracking_manager;

// Re-export public types for backward compatibility
#[deprecated(since = "0.4.0", note = "Use manager::TrackingManager instead")]
pub use config::{ExportMode, ExportOptions};
pub use export_json::build_unified_dashboard_structure;
pub use global_functions::*;
#[deprecated(since = "0.4.0", note = "Use manager::get_global_tracker() instead")]
pub use memory_tracker::{configure_tracking_strategy, get_tracker, MemoryTracker};
pub use tracking_manager::{ComprehensiveTrackingReport, TrackingManager};
