// Unified Backend System for Memory Tracking
// Provides intelligent routing between different tracking strategies
// Maintains compatibility with existing core systems

//! # Deprecated
//!
//! This module is deprecated. Please use the new unified tracking system
//! located in `src/new/tracker/mod.rs` with `UnifiedTracker`.
//!
//! The new unified tracking system provides:
//! - Better performance through configurable strategies
//! - Cleaner API with reduced complexity
//! - Unified type system across all tracking modes
//!
//! Migration Guide:
//! - Replace `crate::unified::UnifiedBackend` with `crate::new::tracker::UnifiedTracker`
//! - Replace `crate::unified::tracking_dispatcher::TrackingDispatcher` with `crate::new::tracker::UnifiedTracker`
//! - Use `TrackingConfig` for more flexible configuration
//! - All functionality is preserved for backward compatibility
//!
//! This module provides a unified backend system for intelligent memory tracking
//! across different runtime environments without conflicting with existing core modules.
//!
//! ## Key Components
//!
//! - [`UnifiedBackend`](crate::unified::backend::UnifiedBackend) - Main orchestrator
//! - [`EnvironmentDetector`](crate::unified::environment_detector::EnvironmentDetector) - Runtime detection
//! - [`TrackingDispatcher`](crate::unified::tracking_dispatcher::TrackingDispatcher) - Strategy routing
//!
//! ## Quick Start
//!
/// ```rust
/// use memscope_rs::unified::{UnifiedBackend, BackendConfig};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize unified backend
///     let backend = UnifiedBackend::initialize(BackendConfig::default())?;
///     Ok(())
/// }
/// ```
pub mod backend;
pub mod environment_detector;
pub mod strategies;
pub mod tracking_dispatcher;

// Re-export main types for convenience
pub use backend::{
    AsyncRuntimeType, BackendConfig, MemoryAnalysisData, MemoryStatistics, RuntimeEnvironment,
    SessionMetadata, TrackingSession, TrackingStrategy, UnifiedBackend,
};

pub use environment_detector::{
    detect_environment, detect_environment_detailed, DetectionConfig, DetectionMetadata,
    DetectionMethod, EnvironmentAnalysis, EnvironmentDetector,
};

pub use tracking_dispatcher::{
    DispatcherConfig, DispatcherMetrics, MemoryTracker, TrackerConfig, TrackerError,
    TrackerStatistics, TrackerType, TrackingDispatcher, TrackingOperation,
};

use crate::core::error::Result;

/// Quick initialization function for unified backend
/// Provides simple setup with default configuration
pub fn quick_start() -> Result<UnifiedBackend> {
    UnifiedBackend::initialize(BackendConfig::default())
}

/// Test function for unified system
/// Validates that all components work together correctly
#[cfg(test)]
pub fn test_unified_system() -> Result<()> {
    let backend = UnifiedBackend::initialize(BackendConfig::default())?;

    // Test basic operations
    let ptr = 0x1000 as usize;
    backend.track_allocation(ptr, 1024)?;
    backend.track_deallocation(ptr)?;

    let snapshot = backend.snapshot();
    assert_eq!(
        snapshot.allocations.len(),
        0,
        "Snapshot should be empty after dealloc"
    );

    Ok(())
}
