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
use crate::capture::backends::async_tracker::AsyncTracker;
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
    async_tracker: Arc<AsyncTracker>,
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
        let async_tracker = Arc::new(AsyncTracker::new());
        async_tracker.set_initialized();
        Self {
            tracker,
            passport_tracker,
            async_tracker,
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

    pub fn async_tracker(&self) -> &Arc<AsyncTracker> {
        &self.async_tracker
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

#[derive(Clone)]
pub struct GlobalTracker {
    inner: Tracker,
    async_tracker: Arc<AsyncTracker>,
}

impl GlobalTracker {
    pub fn track_as<T: crate::Trackable>(&self, var: &T, name: &str, file: &str, line: u32) {
        self.inner.track_as(var, name, file, line);

        if let Some(task_id) = AsyncTracker::get_current_task() {
            let ptr = var.get_heap_ptr().unwrap_or(0);
            let size = var.get_size_estimate();
            let type_name = var.get_type_name().to_string();
            self.async_tracker.track_allocation_with_location(
                ptr,
                size,
                task_id,
                Some(name.to_string()),
                Some(type_name),
                None,
            );
        }
    }

    pub fn analyze(&self) -> crate::tracker::AnalysisReport {
        self.inner.analyze()
    }

    pub fn inner(&self) -> &Tracker {
        &self.inner
    }

    pub fn async_tracker(&self) -> &Arc<AsyncTracker> {
        &self.async_tracker
    }
}

impl std::ops::Deref for GlobalTracker {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target {
        &self.inner
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

/// Get the global tracker instance (unified tracker with async support).
///
/// # Errors
/// Returns `GlobalTrackingError::NotInitialized` if `init_global_tracking()` was not called.
pub fn global_tracker() -> Result<GlobalTracker, GlobalTrackingError> {
    let state = GLOBAL_TRACKING
        .get()
        .ok_or(GlobalTrackingError::NotInitialized)?;
    Ok(GlobalTracker {
        inner: state.tracker().clone(),
        async_tracker: state.async_tracker().clone(),
    })
}

/// Get the global passport tracker instance.
///
/// # Errors
/// Returns `GlobalTrackingError::NotInitialized` if `init_global_tracking()` was not called.
pub fn global_passport_tracker() -> Result<Arc<MemoryPassportTracker>, GlobalTrackingError> {
    let state = GLOBAL_TRACKING
        .get()
        .ok_or(GlobalTrackingError::NotInitialized)?;
    Ok(state.passport_tracker().clone())
}

/// Get the global async tracker instance.
///
/// # Errors
/// Returns `GlobalTrackingError::NotInitialized` if `init_global_tracking()` was not called.
pub fn global_async_tracker() -> Result<Arc<AsyncTracker>, GlobalTrackingError> {
    let state = GLOBAL_TRACKING
        .get()
        .ok_or(GlobalTrackingError::NotInitialized)?;
    Ok(state.async_tracker().clone())
}

/// Get global tracking state (for advanced usage).
pub fn get_global_state() -> Result<&'static GlobalTrackingState, GlobalTrackingError> {
    GLOBAL_TRACKING
        .get()
        .ok_or(GlobalTrackingError::NotInitialized)
}

pub use crate::render_engine::export::{
    export_all_json, export_async_analysis_json, export_leak_detection_json,
    export_memory_passports_json, export_unsafe_ffi_json,
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
/// - `async_analysis.json` - Async task memory analysis
/// - `system_resources.json` - System resource monitoring (CPU, memory, pressure indicators)
pub fn export_to_json<P: AsRef<Path>>(path: P) -> Result<(), GlobalTrackingError> {
    let tracker = global_tracker()?;
    let passport_tracker = global_passport_tracker()?;
    let async_tracker = global_async_tracker()?;
    export_all_json(path, &tracker, &passport_tracker, &async_tracker)
        .map_err(|e| GlobalTrackingError::ExportFailed(e.to_string()))
}

/// Get tracking statistics.
pub fn get_stats() -> Result<GlobalTrackingStats, GlobalTrackingError> {
    let state = get_global_state()?;
    let report = state.tracker().analyze();
    let async_stats = state.async_tracker().get_stats();

    Ok(GlobalTrackingStats {
        total_allocations: report.total_allocations,
        active_allocations: report.active_allocations,
        peak_memory_bytes: report.peak_memory_bytes as usize,
        current_memory_bytes: report.current_memory_bytes as usize,
        passport_count: state.passport_tracker().get_stats().total_passports_created,
        async_task_count: async_stats.total_tasks,
        active_async_tasks: async_stats.active_tasks,
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
    pub async_task_count: usize,
    pub active_async_tasks: usize,
    pub uptime: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_init() {
        let tracker = global_tracker().unwrap();
        assert!(is_initialized());

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
        assert!(stats.uptime.as_secs() > 0 || stats.uptime.subsec_nanos() > 0);
    }

    #[test]
    fn test_async_tracker() {
        let async_tracker = global_async_tracker().unwrap();
        let stats = async_tracker.get_stats();
        assert_eq!(stats.total_tasks, 0);
    }
}
