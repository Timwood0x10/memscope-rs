//! Memory analysis functionality
//!
//! This module provides advanced analysis capabilities:
//! - Enhanced memory analysis
//! - Circular reference detection
//! - Unsafe FFI tracking
//! - Unknown memory region analysis

pub mod circular_reference;
pub mod enhanced_memory_analysis;
pub mod unknown_memory_regions;
pub mod unsafe_ffi_tracker;

// Re-export key analysis functions
pub use circular_reference::*;
pub use enhanced_memory_analysis::EnhancedMemoryAnalyzer;
pub use unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, UnsafeFFITracker};
