//! Global tracking module for passive memory capture.
//!
//! This module provides a unified entry point for automatic memory tracking
//! across all execution modes (single-thread, multi-thread, async, unsafe/ffi).
//!
//! # Design Principles
//!
//! - **Merged**: All tracker data stored in a unified GlobalTracker
//! - **Simplified**: Minimal public API surface
//! - **Backward Compatible**: Legacy APIs preserved as deprecated wrappers
//!
//! # Core API (Recommended)
//!
//! ```ignore
//! use memscope_rs::capture::backends::global_tracking::{init_global_tracking, global_tracker};
//!
//! init_global_tracking().unwrap();
//! let tracker = global_tracker().unwrap();
//!
//! tracker.track!(my_variable);
//! tracker.export_json("output").unwrap();
//! tracker.export_html("output").unwrap();
//! ```

use crate::analysis::memory_passport_tracker::{MemoryPassportTracker, PassportTrackerConfig};
use crate::capture::backends::async_tracker::AsyncTracker;
use crate::core::{MemScopeError, MemScopeResult};
use crate::tracker::{AnalysisReport, Tracker};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

static GLOBAL_TRACKER: std::sync::RwLock<Option<Arc<GlobalTracker>>> = std::sync::RwLock::new(None);

#[derive(Debug, Clone)]
pub struct TrackerConfig {
    pub max_allocations: usize,
    pub enable_statistics: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            max_allocations: 1_000_000,
            enable_statistics: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GlobalTrackerConfig {
    pub tracker: TrackerConfig,
    pub passport: PassportTrackerConfig,
}

pub struct GlobalTracker {
    tracker: Tracker,
    passport_tracker: Arc<MemoryPassportTracker>,
    async_tracker: Arc<AsyncTracker>,
    start_time: Instant,
}

impl std::fmt::Debug for GlobalTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalTracker")
            .field("start_time", &self.start_time)
            .finish()
    }
}

impl GlobalTracker {
    pub fn new() -> Self {
        Self::with_config(GlobalTrackerConfig::default())
    }

    pub fn with_config(config: GlobalTrackerConfig) -> Self {
        let tracker = Tracker::new();
        let passport_tracker = Arc::new(MemoryPassportTracker::new(config.passport));
        let async_tracker = Arc::new(AsyncTracker::new());
        async_tracker.set_initialized();

        Self {
            tracker,
            passport_tracker,
            async_tracker,
            start_time: Instant::now(),
        }
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

    pub fn track<T: crate::Trackable>(&self, var: &T) {
        self.track_as(var, "unknown", "", 0, "");
    }

    pub fn track_as<T: crate::Trackable>(
        &self,
        var: &T,
        name: &str,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        self.tracker.track_as(var, name, file, line, module_path);

        if let Some(task_id) = AsyncTracker::get_current_task() {
            let kind = var.track_kind();
            if let crate::core::types::TrackKind::HeapOwner { ptr, size } = kind {
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
    }

    pub fn create_passport(
        &self,
        ptr: usize,
        size: usize,
        context: String,
    ) -> Result<String, crate::capture::types::TrackingError> {
        self.passport_tracker
            .create_passport_simple(ptr, size, context)
    }

    pub fn record_handover(&self, ptr: usize, context: String, function: String) {
        let _ = self
            .passport_tracker
            .record_handover_to_ffi(ptr, context, function);
    }

    pub fn record_free(&self, ptr: usize, context: String, function: String) {
        let _ = self
            .passport_tracker
            .record_freed_by_foreign(ptr, context, function);
    }

    pub fn detect_leaks(&self) -> crate::analysis::memory_passport_tracker::LeakDetectionResult {
        self.passport_tracker.detect_leaks_at_shutdown()
    }

    pub fn analyze(&self) -> AnalysisReport {
        self.tracker.analyze()
    }

    pub fn get_stats(&self) -> GlobalTrackerStats {
        let report = self.tracker.analyze();
        let passport_stats = self.passport_tracker.get_stats();
        let async_stats = self.async_tracker.get_stats();

        GlobalTrackerStats {
            total_allocations: report.total_allocations,
            active_allocations: report.active_allocations,
            peak_memory_bytes: report.peak_memory_bytes,
            current_memory_bytes: report.current_memory_bytes,
            passport_count: passport_stats.total_passports_created,
            active_passports: passport_stats.active_passports,
            leaks_detected: passport_stats.leaks_detected,
            async_task_count: async_stats.total_tasks,
            active_async_tasks: async_stats.active_tasks,
            uptime: self.elapsed(),
        }
    }

    pub fn export_json<P: AsRef<Path>>(&self, path: P) -> MemScopeResult<()> {
        use crate::render_engine::export::export_all_json;

        let path = path.as_ref();
        export_all_json(
            path,
            &self.tracker,
            &self.passport_tracker,
            &self.async_tracker,
        )
        .map_err(|e| MemScopeError::error("global_tracking", "export_json", e.to_string()))?;
        Ok(())
    }

    pub fn export_html<P: AsRef<Path>>(&self, path: P) -> MemScopeResult<()> {
        use crate::render_engine::export::export_dashboard_html_with_async;

        let path = path.as_ref();
        export_dashboard_html_with_async(
            path,
            &self.tracker,
            &self.passport_tracker,
            &self.async_tracker,
        )
        .map_err(|e| MemScopeError::error("global_tracking", "export_html", e.to_string()))?;
        Ok(())
    }

    pub fn export_html_with_template<P: AsRef<Path>>(
        &self,
        path: P,
        template: crate::render_engine::export::DashboardTemplate,
    ) -> MemScopeResult<()> {
        use crate::render_engine::export::export_dashboard_html_with_template;

        let path = path.as_ref();
        export_dashboard_html_with_template(
            path,
            &self.tracker,
            &self.passport_tracker,
            template,
            Some(&self.async_tracker),
        )
        .map_err(|e| {
            MemScopeError::error(
                "global_tracking",
                "export_html_with_template",
                e.to_string(),
            )
        })?;
        Ok(())
    }
}

impl Default for GlobalTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GlobalTrackerStats {
    pub total_allocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: u64,
    pub current_memory_bytes: u64,
    pub passport_count: usize,
    pub active_passports: usize,
    pub leaks_detected: usize,
    pub async_task_count: usize,
    pub active_async_tasks: usize,
    pub uptime: std::time::Duration,
}

pub fn init_global_tracking() -> MemScopeResult<()> {
    let mut guard = GLOBAL_TRACKER.write().map_err(|_| {
        MemScopeError::error(
            "global_tracking",
            "init_global_tracking",
            "Failed to acquire global tracker lock",
        )
    })?;

    if guard.is_some() {
        return Err(MemScopeError::error(
            "global_tracking",
            "init_global_tracking",
            "Global tracking already initialized",
        ));
    }

    *guard = Some(Arc::new(GlobalTracker::new()));
    info!("Global tracking initialized");
    Ok(())
}

pub fn init_global_tracking_with_config(config: GlobalTrackerConfig) -> MemScopeResult<()> {
    let mut guard = GLOBAL_TRACKER.write().map_err(|_| {
        MemScopeError::error(
            "global_tracking",
            "init_global_tracking_with_config",
            "Failed to acquire global tracker lock",
        )
    })?;

    if guard.is_some() {
        return Err(MemScopeError::error(
            "global_tracking",
            "init_global_tracking_with_config",
            "Global tracking already initialized",
        ));
    }

    *guard = Some(Arc::new(GlobalTracker::with_config(config)));
    info!("Global tracking initialized with config");
    Ok(())
}

pub fn reset_global_tracking() {
    if let Ok(mut guard) = GLOBAL_TRACKER.write() {
        *guard = None;
    }
}

pub fn is_initialized() -> bool {
    GLOBAL_TRACKER
        .read()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

pub fn global_tracker() -> MemScopeResult<Arc<GlobalTracker>> {
    GLOBAL_TRACKER
        .read()
        .map(|guard| {
            guard.as_ref().cloned().ok_or_else(|| {
                MemScopeError::error(
                    "global_tracking",
                    "global_tracker",
                    "Global tracking not initialized",
                )
            })
        })
        .map_err(|_| {
            MemScopeError::error(
                "global_tracking",
                "global_tracker",
                "Failed to acquire global tracker lock",
            )
        })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn reset_global_state() {
        reset_global_tracking();
    }

    #[test]
    fn test_unified_tracker() {
        let tracker = GlobalTracker::new();
        assert!(tracker.tracker().analyze().total_allocations == 0);

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0);
    }

    #[test]
    fn test_tracker_config_default() {
        let config = TrackerConfig::default();
        assert_eq!(config.max_allocations, 1_000_000);
        assert!(config.enable_statistics);
    }

    #[test]
    fn test_tracker_config_clone() {
        let config = TrackerConfig {
            max_allocations: 500,
            enable_statistics: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_allocations, 500);
        assert!(!cloned.enable_statistics);
    }

    #[test]
    fn test_tracker_config_debug() {
        let config = TrackerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TrackerConfig"));
        assert!(debug_str.contains("max_allocations"));
        assert!(debug_str.contains("enable_statistics"));
    }

    #[test]
    fn test_global_tracker_config_default() {
        let config = GlobalTrackerConfig::default();
        assert_eq!(config.tracker.max_allocations, 1_000_000);
        assert!(config.tracker.enable_statistics);
    }

    #[test]
    fn test_global_tracker_config_clone() {
        let config = GlobalTrackerConfig {
            tracker: TrackerConfig {
                max_allocations: 100,
                enable_statistics: false,
            },
            passport: PassportTrackerConfig::default(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.tracker.max_allocations, 100);
        assert!(!cloned.tracker.enable_statistics);
    }

    #[test]
    fn test_global_tracker_config_debug() {
        let config = GlobalTrackerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GlobalTrackerConfig"));
    }

    #[test]
    fn test_global_tracker_new() {
        let tracker = GlobalTracker::new();
        assert!(tracker.elapsed() < Duration::from_secs(1));
        assert!(tracker.tracker().analyze().total_allocations == 0);
    }

    #[test]
    fn test_global_tracker_with_config() {
        let config = GlobalTrackerConfig {
            tracker: TrackerConfig {
                max_allocations: 100,
                enable_statistics: true,
            },
            passport: PassportTrackerConfig::default(),
        };
        let tracker = GlobalTracker::with_config(config);
        assert!(tracker.tracker().analyze().total_allocations == 0);
    }

    #[test]
    fn test_global_tracker_default() {
        let tracker = GlobalTracker::default();
        assert!(tracker.tracker().analyze().total_allocations == 0);
    }

    #[test]
    fn test_global_tracker_debug() {
        let tracker = GlobalTracker::new();
        let debug_str = format!("{:?}", tracker);
        assert!(debug_str.contains("GlobalTracker"));
        assert!(debug_str.contains("start_time"));
    }

    #[test]
    fn test_global_tracker_elapsed() {
        let tracker = GlobalTracker::new();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = tracker.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_global_tracker_tracker_accessor() {
        let tracker = GlobalTracker::new();
        let inner_tracker = tracker.tracker();
        assert!(inner_tracker.analyze().total_allocations == 0);
    }

    #[test]
    fn test_global_tracker_passport_tracker_accessor() {
        let tracker = GlobalTracker::new();
        let passport = tracker.passport_tracker();
        let stats = passport.get_stats();
        assert_eq!(stats.total_passports_created, 0);
    }

    #[test]
    fn test_global_tracker_async_tracker_accessor() {
        let tracker = GlobalTracker::new();
        let async_tracker = tracker.async_tracker();
        let stats = async_tracker.get_stats();
        assert_eq!(stats.total_tasks, 0);
    }

    #[test]
    fn test_global_tracker_analyze() {
        let tracker = GlobalTracker::new();
        let report = tracker.analyze();
        assert_eq!(report.total_allocations, 0);
        assert_eq!(report.active_allocations, 0);
    }

    #[test]
    fn test_global_tracker_stats() {
        let tracker = GlobalTracker::new();
        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.peak_memory_bytes, 0);
        assert_eq!(stats.current_memory_bytes, 0);
        assert_eq!(stats.passport_count, 0);
        assert_eq!(stats.active_passports, 0);
        assert_eq!(stats.leaks_detected, 0);
        assert_eq!(stats.async_task_count, 0);
        assert_eq!(stats.active_async_tasks, 0);
        assert!(stats.uptime < Duration::from_secs(1));
    }

    #[test]
    fn test_global_tracker_stats_debug() {
        let tracker = GlobalTracker::new();
        let stats = tracker.get_stats();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("GlobalTrackerStats"));
        assert!(debug_str.contains("total_allocations"));
    }

    #[test]
    fn test_global_tracker_stats_clone() {
        let tracker = GlobalTracker::new();
        let stats = tracker.get_stats();
        let cloned = stats.clone();
        assert_eq!(cloned.total_allocations, stats.total_allocations);
        assert_eq!(cloned.active_allocations, stats.active_allocations);
    }

    #[test]
    fn test_init_global_tracking_double_init() {
        reset_global_state();
        let result1 = init_global_tracking();
        assert!(result1.is_ok(), "First init should succeed");
        let result2 = init_global_tracking();
        assert!(result2.is_err(), "Second init should fail");
        reset_global_state();
    }

    #[test]
    fn test_init_global_tracking_with_config_double_init() {
        reset_global_state();
        let config = GlobalTrackerConfig::default();
        let result1 = init_global_tracking_with_config(config);
        assert!(result1.is_ok(), "First init should succeed");
        let config2 = GlobalTrackerConfig::default();
        let result2 = init_global_tracking_with_config(config2);
        assert!(result2.is_err(), "Second init should fail");
        reset_global_state();
    }

    #[test]
    fn test_global_tracker_accessor_not_initialized() {
        reset_global_state();
        let result = global_tracker();
        assert!(result.is_err(), "Should fail when not initialized");
        reset_global_state();
    }

    #[test]
    fn test_reset_global_tracking() {
        reset_global_state();
        init_global_tracking().unwrap();
        assert!(is_initialized());
        reset_global_tracking();
        assert!(!is_initialized());
    }

    #[test]
    fn test_global_tracker_create_passport() {
        let tracker = GlobalTracker::new();
        let result = tracker.create_passport(0x1000, 64, "test_context".to_string());
        assert!(result.is_ok());
        let passport_id = result.unwrap();
        assert!(!passport_id.is_empty());
    }

    #[test]
    fn test_global_tracker_detect_leaks() {
        let tracker = GlobalTracker::new();
        let result = tracker.detect_leaks();
        assert_eq!(result.total_leaks, 0);
    }

    #[test]
    fn test_global_tracker_record_handover() {
        let tracker = GlobalTracker::new();
        tracker
            .create_passport(0x2000, 128, "test".to_string())
            .unwrap();
        tracker.record_handover(0x2000, "context".to_string(), "function".to_string());
    }

    #[test]
    fn test_global_tracker_record_free() {
        let tracker = GlobalTracker::new();
        tracker
            .create_passport(0x3000, 256, "test".to_string())
            .unwrap();
        tracker.record_free(0x3000, "context".to_string(), "function".to_string());
    }

    #[test]
    fn test_global_tracker_track_simple() {
        let tracker = GlobalTracker::new();
        let value = String::from("test");
        tracker.track(&value);
    }

    #[test]
    fn test_global_tracker_track_as() {
        let tracker = GlobalTracker::new();
        let value = String::from("test_value");
        tracker.track_as(&value, "test_var", "test.rs", 10, "test_module");
    }
}
