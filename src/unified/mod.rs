// Unified Backend System for Memory Tracking
// Provides intelligent routing between different tracking strategies
// Maintains compatibility with existing core systems

//! # Unified Backend System
//!
//! **DEPRECATED**: Use `capture::backends::unified_tracker` instead.
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
//! ```rust
//! use memscope_rs::unified::{UnifiedBackend, BackendConfig};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize unified backend
//!     let backend = UnifiedBackend::initialize(BackendConfig::default())?;
//!     Ok(())
//! }
//! ```

#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub mod backend;
#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub mod environment_detector;
#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub mod strategies;
#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub mod tracking_dispatcher;

// Re-export main types for convenience
pub use backend::{
    AsyncRuntimeType, BackendConfig, BackendError, MemoryAnalysisData, MemoryStatistics,
    RuntimeEnvironment, SessionMetadata, TrackingSession, TrackingStrategy, UnifiedBackend,
};

pub use environment_detector::{
    detect_environment, detect_environment_detailed, DetectionConfig, DetectionMetadata,
    DetectionMethod, EnvironmentAnalysis, EnvironmentDetector,
};

pub use tracking_dispatcher::{
    DispatcherConfig, DispatcherMetrics, MemoryTracker, TrackerConfig, TrackerError,
    TrackerStatistics, TrackerType, TrackingDispatcher, TrackingOperation,
};

#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub fn quick_start() -> Result<UnifiedBackend, BackendError> {
    UnifiedBackend::initialize(BackendConfig::default())
}

#[deprecated(
    since = "0.6.0",
    note = "Use capture::backends::unified_tracker instead. This module will be removed in a future version."
)]
pub fn test_unified_system() -> Result<(), BackendError> {
    let mut backend = quick_start()?;
    let session = backend.start_tracking()?;
    let _data = session.collect_data()?;
    let _final_data = session.end_session()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_backend_quick_start() {
        let result = quick_start();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unified_system_integration() {
        let result = test_unified_system();
        assert!(result.is_ok());
    }
}
