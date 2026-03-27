// Unified Backend System for Memory Tracking
// Provides intelligent routing between different tracking strategies
// Maintains compatibility with existing core systems

pub mod backend;
pub mod environment_detector;
pub mod tracking_dispatcher;

pub use backend::{
    AsyncRuntimeType, BackendConfig, MemoryAnalysisData, MemoryStatistics,
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

use crate::core::error::Result;

pub fn quick_start() -> Result<UnifiedBackend> {
    UnifiedBackend::initialize(BackendConfig::default())
}

pub fn test_unified_system() -> Result<()> {
    let mut backend = UnifiedBackend::initialize(BackendConfig::default())?;
    let mut session = backend.start_tracking()?;
    let ptr = 0x1000 as usize;
    backend.track_allocation(ptr, 1024)?;
    backend.track_deallocation(ptr)?;
    session.end_session()?;
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
    fn test_unified_system() {
        let result = test_unified_system();
        assert!(result.is_ok());
    }
}
