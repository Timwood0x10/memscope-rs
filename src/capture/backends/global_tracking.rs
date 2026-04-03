//! Global tracking module for passive memory capture.
//!
//! This module provides a unified entry point for automatic memory tracking
//! across all execution modes (single-thread, multi-thread, async, unsafe/ffi).
//!
//! # Design Philosophy
//!
//! - **Lazy Initialization**: Global tracker is initialized on first access
//! - **Passive Capture**: Automatically captures all user program allocations
//! - **Zero Configuration**: Works out of the box with sensible defaults
//! - **Mode Agnostic**: Same API for single-thread, multi-thread, async, and unsafe/ffi

use crate::analysis::memory_passport_tracker::{
    initialize_global_passport_tracker, MemoryPassportTracker, PassportTrackerConfig,
};
use crate::tracker::Tracker;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, warn};

static GLOBAL_TRACKING: std::sync::OnceLock<GlobalTrackingState> = std::sync::OnceLock::new();

#[derive(Error, Debug)]
pub enum GlobalTrackingError {
    #[error("Global tracking already initialized")]
    AlreadyInitialized,

    #[error("Global tracking not initialized")]
    NotInitialized,

    #[error("Export failed: {0}")]
    ExportFailed(String),
}

#[derive(Debug, Clone)]
pub struct GlobalTrackingConfig {
    pub tracker: TrackerConfig,
    pub passport: PassportTrackerConfig,
}

#[derive(Debug, Clone)]
pub struct TrackerConfig {
    pub max_allocations: usize,
    pub enable_statistics: bool,
}

impl Default for GlobalTrackingConfig {
    fn default() -> Self {
        Self {
            tracker: TrackerConfig {
                max_allocations: 1000000,
                enable_statistics: true,
            },
            passport: PassportTrackerConfig {
                detailed_logging: true,
                max_events_per_passport: 100,
                enable_leak_detection: true,
                enable_validation: false,
                max_passports: 10000,
                track_rust_internal_stack: false,
                user_code_prefixes: vec!["examples/".to_string()],
            },
        }
    }
}

pub struct GlobalTrackingState {
    tracker: Tracker,
    passport_tracker: Arc<MemoryPassportTracker>,
    start_time: Instant,
}

impl std::fmt::Debug for GlobalTrackingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalTrackingState")
            .field("start_time", &self.start_time)
            .finish()
    }
}

impl GlobalTrackingState {
    pub fn new_with_config(config: GlobalTrackingConfig) -> Self {
        let tracker = Tracker::new();
        let passport_tracker = initialize_global_passport_tracker(config.passport);
        Self {
            tracker,
            passport_tracker,
            start_time: Instant::now(),
        }
    }

    pub fn new() -> Self {
        Self::new_with_config(GlobalTrackingConfig::default())
    }

    pub fn tracker(&self) -> &Tracker {
        &self.tracker
    }

    pub fn passport_tracker(&self) -> &Arc<MemoryPassportTracker> {
        &self.passport_tracker
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl Default for GlobalTrackingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize global tracking with default configuration.
pub fn init_global_tracking() -> Result<(), GlobalTrackingError> {
    let state = GlobalTrackingState::new();
    let result = GLOBAL_TRACKING.set(state);

    if result.is_err() {
        warn!("Global tracking already initialized, skipping");
        return Err(GlobalTrackingError::AlreadyInitialized);
    }

    info!("Global tracking initialized");
    Ok(())
}

/// Check if global tracking is initialized.
pub fn is_initialized() -> bool {
    GLOBAL_TRACKING.get().is_some()
}

/// Get the global tracker instance.
pub fn global_tracker() -> Result<Tracker, GlobalTrackingError> {
    let state = GLOBAL_TRACKING.get_or_init(|| GlobalTrackingState::new());
    Ok(state.tracker().clone())
}

/// Get the global passport tracker instance.
pub fn global_passport_tracker() -> Result<Arc<MemoryPassportTracker>, GlobalTrackingError> {
    let state = GLOBAL_TRACKING.get_or_init(|| GlobalTrackingState::new());
    Ok(state.passport_tracker().clone())
}

/// Get global tracking state (for advanced usage).
pub fn get_global_state() -> Result<&'static GlobalTrackingState, GlobalTrackingError> {
    GLOBAL_TRACKING
        .get()
        .ok_or(GlobalTrackingError::NotInitialized)
}

pub use crate::render_engine::export::{
    export_all_json, export_leak_detection_json, export_memory_passports_json,
    export_unsafe_ffi_json,
};

/// Export captured memory data to JSON files using global tracker.
///
/// This is a convenience wrapper that uses the global singleton tracker.
///
/// # Exported Files
///
/// This function creates the following JSON files in the specified directory:
///
/// - `memory_analysis.json` - Main memory allocation analysis
/// - `lifetime.json` - Ownership and lifetime tracking
/// - `thread_analysis.json` - Thread-local memory statistics
/// - `variable_relationships.json` - Variable relationship graph
/// - `memory_passports.json` - Memory passport tracking
/// - `leak_detection.json` - Memory leak detection results
/// - `unsafe_ffi.json` - Unsafe/FFI tracking data
/// - `system_resources.json` - System resource monitoring (CPU, memory, pressure indicators)
pub fn export_to_json<P: AsRef<Path>>(path: P) -> Result<(), GlobalTrackingError> {
    let tracker = global_tracker()?;
    let passport_tracker = global_passport_tracker()?;
    export_all_json(path, &tracker, &passport_tracker)
        .map_err(|e| GlobalTrackingError::ExportFailed(e.to_string()))
}

/// Get tracking statistics.
pub fn get_stats() -> Result<GlobalTrackingStats, GlobalTrackingError> {
    let state = get_global_state()?;
    let report = state.tracker().analyze();

    Ok(GlobalTrackingStats {
        total_allocations: report.total_allocations,
        active_allocations: report.active_allocations,
        peak_memory_bytes: report.peak_memory_bytes as usize,
        current_memory_bytes: report.current_memory_bytes as usize,
        passport_count: state.passport_tracker().get_stats().total_passports_created,
        uptime: state.elapsed(),
    })
}

#[derive(Debug, Clone)]
pub struct GlobalTrackingStats {
    pub total_allocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: usize,
    pub current_memory_bytes: usize,
    pub passport_count: usize,
    pub uptime: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_init() {
        // OnceLock cannot be reset, so we just test lazy initialization
        let tracker = global_tracker().unwrap();
        assert!(is_initialized());

        // Subsequent calls return the same instance
        let tracker2 = global_tracker().unwrap();
        assert_eq!(
            tracker.analyze().total_allocations,
            tracker2.analyze().total_allocations
        );
    }

    #[test]
    fn test_stats() {
        let _tracker = global_tracker().unwrap();
        let stats = get_stats().unwrap();
        assert!(stats.uptime.as_secs() >= 0);
    }
}
