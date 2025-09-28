// Unified Backend System for Memory Tracking
// Provides intelligent routing between different tracking strategies
// Maintains compatibility with existing core systems

//! # Unified Backend System
//! 
//! This module provides a unified backend system for intelligent memory tracking
//! across different runtime environments without conflicting with existing core modules.
//! 
//! ## Key Components
//! 
//! - [`UnifiedBackend`](backend::UnifiedBackend) - Main orchestrator
//! - [`EnvironmentDetector`](environment_detector::EnvironmentDetector) - Runtime detection
//! - [`TrackingDispatcher`](tracking_dispatcher::TrackingDispatcher) - Strategy routing
//! 
//! ## Quick Start
//! 
//! ```rust
//! use memscope_rs::unified::{UnifiedBackend, BackendConfig};
//! 
//! // Initialize unified backend
//! let backend = UnifiedBackend::initialize(BackendConfig::default())?;
//! ```

pub mod backend;
pub mod environment_detector;
pub mod tracking_dispatcher;

// Re-export main types for convenience
pub use backend::{
    UnifiedBackend,
    BackendConfig,
    TrackingSession,
    MemoryAnalysisData,
    MemoryStatistics,
    SessionMetadata,
    RuntimeEnvironment,
    AsyncRuntimeType,
    TrackingStrategy,
    BackendError,
};

pub use environment_detector::{
    EnvironmentDetector,
    DetectionConfig,
    EnvironmentAnalysis,
    DetectionMetadata,
    DetectionMethod,
    detect_environment,
    detect_environment_detailed,
};

pub use tracking_dispatcher::{
    TrackingDispatcher,
    DispatcherConfig,
    DispatcherMetrics,
    MemoryTracker,
    TrackerConfig,
    TrackerStatistics,
    TrackerType,
    TrackerError,
    TrackingOperation,
};

/// Quick initialization function for unified backend
/// Provides simple setup with default configuration
pub fn quick_start() -> Result<UnifiedBackend, BackendError> {
    UnifiedBackend::initialize(BackendConfig::default())
}

/// Test the unified backend system
/// Ensures all components work together correctly
pub fn test_unified_system() -> Result<(), BackendError> {
    // Initialize backend
    let mut backend = quick_start()?;
    
    // Start tracking session
    let session = backend.start_tracking()?;
    
    // Collect data
    let _data = session.collect_data()?;
    
    // End session
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