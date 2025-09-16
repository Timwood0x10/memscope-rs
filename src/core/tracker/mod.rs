//! Memory tracking module with modular structure.
//!
//! This module provides comprehensive memory allocation tracking, analysis,
//! and export functionality. It has been restructured from a single large
//! file into focused, maintainable modules.

pub mod allocation_tracking;
pub mod config;
pub mod export_html;
pub mod export_json;
pub mod global_functions;
pub mod memory_analysis;
pub mod memory_tracker;
pub mod tracking_manager;

// Re-export public types for backward compatibility
pub use config::{ExportMode, ExportOptions};
pub use export_json::build_unified_dashboard_structure;
pub use global_functions::*;
pub use memory_tracker::{get_global_tracker, MemoryTracker};
pub use tracking_manager::{ComprehensiveTrackingReport, TrackingManager};
