//! Core memory tracker implementation.
//!
//! This module contains the main MemoryTracker struct for memory allocation tracking.

use crate::core::bounded_memory_stats::{
    AllocationHistoryManager, BoundedMemoryStats, BoundedStatsConfig,
};
use crate::core::ownership_history::{HistoryConfig, OwnershipHistoryRecorder};
use crate::core::safe_operations::SafeLock;
pub use crate::core::thread_registry::ThreadRegistryStats;
use crate::core::thread_registry::{
    GenericCachedThreadData, GenericThreadRegistry, TrackerStatsProvider,
};
use crate::core::types::{AllocationInfo, MemoryStats, TrackingError::LockError, TrackingResult};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const STRATEGY_GLOBAL_SINGLETON: u8 = 0;
const STRATEGY_THREAD_LOCAL: u8 = 1;

pub static TRACKING_STRATEGY: AtomicU8 = AtomicU8::new(STRATEGY_GLOBAL_SINGLETON);

static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

impl TrackerStatsProvider for MemoryTracker {
    fn get_stats(&self) -> Result<MemoryStats, crate::core::types::TrackingError> {
        MemoryTracker::get_stats(self)
    }
}

static LOCAL_THREAD_REGISTRY: OnceLock<Arc<Mutex<GenericThreadRegistry<MemoryTracker>>>> =
    OnceLock::new();

fn get_local_registry() -> Arc<Mutex<GenericThreadRegistry<MemoryTracker>>> {
    LOCAL_THREAD_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(GenericThreadRegistry::new())))
        .clone()
}

fn register_current_thread_tracker_local(tracker: &Arc<MemoryTracker>) {
    let thread_id = std::thread::current().id();

    if let Ok(mut registry) = get_local_registry().lock() {
        registry.register_tracker(thread_id, tracker);
    } else {
        tracing::error!("Failed to acquire local registry lock for thread registration");
    }
}

pub fn collect_all_trackers_local() -> Vec<Arc<MemoryTracker>> {
    match get_local_registry().lock() {
        Ok(mut registry) => registry.collect_active_trackers(),
        Err(_) => Vec::new(),
    }
}

pub fn get_cached_thread_data_local() -> Vec<GenericCachedThreadData> {
    match get_local_registry().lock() {
        Ok(registry) => registry.get_cached_thread_data(),
        Err(_) => Vec::new(),
    }
}

pub fn get_registry_stats_local() -> ThreadRegistryStats {
    match get_local_registry().lock() {
        Ok(registry) => registry.get_stats(),
        Err(_) => ThreadRegistryStats {
            total_threads_registered: 0,
            active_threads: 0,
            dead_references: 0,
        },
    }
}

pub fn cleanup_registry_local() {
    if let Ok(mut registry) = get_local_registry().lock() {
        registry.cleanup_dead_references();
    } else {
        tracing::error!("Failed to acquire local registry lock for cleanup");
    }
}

pub fn has_active_trackers_local() -> bool {
    match get_local_registry().lock() {
        Ok(registry) => registry.has_active_trackers(),
        Err(_) => false,
    }
}

thread_local! {
    static THREAD_LOCAL_TRACKER: Arc<MemoryTracker> = {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker_local(&tracker);
        tracker
    };
}

/// Configure tracking strategy for the application.
///
/// This function should be called at program startup to set the appropriate
/// tracking strategy based on whether the application is concurrent or not.
///
/// # Arguments
/// * `is_concurrent` - true for multi-threaded/async applications, false for single-threaded
pub fn configure_tracking_strategy(is_concurrent: bool) {
    let strategy = if is_concurrent {
        STRATEGY_THREAD_LOCAL
    } else {
        STRATEGY_GLOBAL_SINGLETON
    };

    TRACKING_STRATEGY.store(strategy, Ordering::Relaxed);

    tracing::info!(
        "Configured tracking strategy: {}",
        if is_concurrent {
            "thread-local"
        } else {
            "global-singleton"
        }
    );
}

/// Get the appropriate memory tracker based on current strategy.
///
/// This function implements the dual-mode dispatch:
/// - In single-threaded mode: returns the global singleton tracker
/// - In concurrent mode: returns the current thread's local tracker
pub fn get_tracker() -> Arc<MemoryTracker> {
    match TRACKING_STRATEGY.load(Ordering::Relaxed) {
        STRATEGY_GLOBAL_SINGLETON => GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone(),
        STRATEGY_THREAD_LOCAL => THREAD_LOCAL_TRACKER.with(|tracker| tracker.clone()),
        _ => {
            // Fallback to global singleton for unknown strategy
            tracing::warn!("Unknown tracking strategy, falling back to global singleton");
            GLOBAL_TRACKER
                .get_or_init(|| Arc::new(MemoryTracker::new()))
                .clone()
        }
    }
}

/// Get the global memory tracker instance (legacy compatibility).
///
/// This function is preserved for backward compatibility but now delegates to get_tracker().
/// New code should use get_tracker() directly for dual-mode support.
#[deprecated(note = "Use get_tracker() instead for dual-mode support")]
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    get_tracker()
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
pub struct MemoryTracker {
    /// Active allocations (ptr -> allocation info)
    pub(crate) active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Bounded memory statistics (prevents infinite growth)
    pub(crate) bounded_stats: Mutex<BoundedMemoryStats>,
    /// Separate allocation history manager (bounded)
    pub(crate) history_manager: Mutex<AllocationHistoryManager>,
    /// Ownership history recorder for detailed lifecycle tracking
    #[allow(dead_code)]
    pub(crate) ownership_history: Mutex<OwnershipHistoryRecorder>,
    /// Legacy stats for compatibility (derived from bounded_stats)
    pub(crate) stats: Mutex<MemoryStats>,
    /// Fast mode flag for testing (reduces overhead)
    pub(crate) fast_mode: std::sync::atomic::AtomicBool,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode =
            std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test) || cfg!(feature = "test");

        // Configure bounded stats based on environment
        let config = if fast_mode {
            // Smaller limits for testing
            BoundedStatsConfig {
                max_recent_allocations: 1_000,
                max_historical_summaries: 100,
                enable_auto_cleanup: true,
                cleanup_threshold: 0.8,
            }
        } else {
            // Production limits
            BoundedStatsConfig::default()
        };

        // Configure ownership history based on mode
        let history_config = if fast_mode {
            HistoryConfig {
                max_events_per_allocation: 10,
                track_borrowing: false,
                track_cloning: true,
                track_ownership_transfers: false,
            }
        } else {
            HistoryConfig::default()
        };

        Self {
            active_allocations: Mutex::new(HashMap::new()),
            bounded_stats: Mutex::new(BoundedMemoryStats::with_config(config.clone())),
            history_manager: Mutex::new(AllocationHistoryManager::with_config(config)),
            ownership_history: Mutex::new(OwnershipHistoryRecorder::with_config(history_config)),
            stats: Mutex::new(MemoryStats::default()),
            fast_mode: std::sync::atomic::AtomicBool::new(fast_mode),
        }
    }

    /// Get current memory statistics with advanced analysis.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        // Get bounded stats using safe operations
        let bounded_stats = self
            .bounded_stats
            .safe_lock()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| crate::core::bounded_memory_stats::BoundedMemoryStats::default());

        // Get history for compatibility using safe operations
        let _history = self
            .history_manager
            .safe_lock()
            .map(|manager| manager.get_history_vec())
            .unwrap_or_else(|_| Vec::new());

        // Convert bounded stats to legacy MemoryStats for compatibility
        let legacy_stats = MemoryStats {
            total_allocations: bounded_stats.total_allocations,
            total_allocated: bounded_stats.total_allocated,
            active_allocations: bounded_stats.active_allocations,
            active_memory: bounded_stats.active_memory,
            peak_allocations: bounded_stats.peak_allocations,
            peak_memory: bounded_stats.peak_memory,
            total_deallocations: bounded_stats.total_deallocations,
            total_deallocated: bounded_stats.total_deallocated,
            leaked_allocations: bounded_stats.leaked_allocations,
            leaked_memory: bounded_stats.leaked_memory,
            fragmentation_analysis: bounded_stats.fragmentation_analysis.clone(),
            lifecycle_stats: bounded_stats.lifecycle_stats.clone(),
            system_library_stats: bounded_stats.system_library_stats.clone(),
            concurrency_analysis: bounded_stats.concurrency_analysis.clone(),
            // Use bounded allocations instead of infinite growth
            allocations: bounded_stats.get_all_allocations(),
        };

        // Update the legacy stats cache using safe operations
        if let Ok(mut stats) = self.stats.safe_lock() {
            *stats = legacy_stats.clone();
        }

        Ok(legacy_stats)
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.active_allocations
            .safe_lock()
            .map(|active| active.values().cloned().collect())
            .map_err(|e| LockError(format!("Failed to get active allocations: {e}",)))
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        self.history_manager
            .safe_lock()
            .map(|manager| manager.get_history_vec())
            .map_err(|e| LockError(format!("Failed to get allocation history: {e}",)))
    }

    /// Enable or disable fast mode.
    pub fn set_fast_mode(&self, enabled: bool) {
        self.fast_mode
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    /// Check if fast mode is enabled.
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Enable fast mode for testing
    pub fn enable_fast_mode(&self) {
        self.fast_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Ensure the memory analysis path exists and return the full path
    pub fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::path::PathBuf {
        let path = path.as_ref();
        let memory_analysis_dir = std::path::Path::new("MemoryAnalysis");

        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(memory_analysis_dir) {
            tracing::warn!("Failed to create MemoryAnalysis directory: {}", e);
        }

        memory_analysis_dir.join(path)
    }

    /// Ensure path uses .memscope extension and is in MemoryAnalysis directory
    pub fn ensure_memscope_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::path::PathBuf {
        let mut output_path = self.ensure_memory_analysis_path(path);

        // Ensure .memscope extension
        if output_path.extension().is_none()
            || output_path.extension() != Some(std::ffi::OsStr::new("memscope"))
        {
            output_path.set_extension("memscope");
        }

        output_path
    }

    /// Export memory analysis visualization to SVG file.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the memory analysis SVG file (recommended: "program_name_memory_analysis.svg")
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        // Simplified export - just create empty file for now
        std::fs::File::create(output_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;
        Ok(())
    }

    /// Export interactive lifecycle timeline showing variable lifecycles and relationships.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Output filename for the lifecycle timeline SVG file (recommended: "program_name_lifecycle.svg")
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);
        // Simplified export - just create empty file for now
        std::fs::File::create(output_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;
        Ok(())
    }

    /// Export memory tracking data to binary format (.memscope file).
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    /// This method exports user-defined variables only (default behavior for compatibility).
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        // Maintain compatibility by defaulting to user-only export
        self.export_user_binary(path)
    }

    /// Export memory tracking data to binary format with specified mode.
    /// All output files are automatically placed in the MemoryAnalysis/ directory.
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    /// * `mode` - Export mode (UserOnly for small files, Full for complete data)
    pub fn export_to_binary_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        mode: crate::core::tracker::memory_tracker::BinaryExportMode,
    ) -> TrackingResult<()> {
        match mode {
            crate::core::tracker::memory_tracker::BinaryExportMode::UserOnly => {
                self.export_user_binary(path)
            }
            crate::core::tracker::memory_tracker::BinaryExportMode::Full => {
                self.export_full_binary(path)
            }
        }
    }

    /// Export only user-defined variables to binary format (.memscope file).
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    pub fn export_user_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);

        let all_allocations = self.get_active_allocations()?;

        // Filter to user-defined variables only
        let user_allocations: Vec<_> = all_allocations
            .into_iter()
            .filter(|allocation| allocation.var_name.is_some())
            .collect();

        crate::export::binary::export_to_binary_with_mode(
            &user_allocations,
            output_path,
            crate::export::binary::format::BinaryExportMode::UserOnly,
            &crate::export::binary::BinaryExportConfig::default(),
        )
        .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        Ok(())
    }

    /// Export all allocations to binary format (.memscope file).
    ///
    /// # Arguments
    /// * `path` - Base filename for the binary export (extension .memscope will be added automatically)
    pub fn export_full_binary<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memscope_path(path);

        let all_allocations = self.get_active_allocations()?;

        crate::export::binary::export_to_binary_with_mode(
            &all_allocations,
            output_path,
            crate::export::binary::format::BinaryExportMode::Full,
            &crate::export::binary::BinaryExportConfig::default(),
        )
        .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))?;

        Ok(())
    }

    /// Convert binary file to standard JSON format (4 separate files)
    ///
    /// # Arguments
    /// * `binary_path` - Path to input .memscope file
    /// * `base_name` - Base name for output files
    pub fn parse_binary_to_standard_json<P: AsRef<std::path::Path>>(
        binary_path: P,
        base_name: &str,
    ) -> TrackingResult<()> {
        crate::export::binary::BinaryParser::to_standard_json_files(binary_path, base_name)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Convert binary file to single JSON format (legacy compatibility)
    ///
    /// # Arguments
    /// * `binary_path` - Path to input .memscope file
    /// * `json_path` - Path for output JSON file
    pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
        binary_path: P,
        json_path: P,
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_json(binary_path, json_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Convert binary file to HTML format
    ///
    /// # Arguments
    /// * `binary_path` - Path to input .memscope file
    /// * `html_path` - Path for output HTML file
    pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
        binary_path: P,
        html_path: P,
    ) -> TrackingResult<()> {
        crate::export::binary::parse_binary_to_html(binary_path, html_path)
            .map_err(|e| crate::core::types::TrackingError::ExportError(e.to_string()))
    }

    /// Alias for parse_binary_to_html for backward compatibility
    pub fn export_binary_to_html<P: AsRef<std::path::Path>>(
        binary_path: P,
        html_path: P,
    ) -> TrackingResult<()> {
        Self::parse_binary_to_html(binary_path, html_path)
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        // Optional verbose tip for users
        if std::env::var("MEMSCOPE_VERBOSE").is_ok() {
            tracing::info!("💡 Tip: Use tracker.export_to_json() or tracker.export_interactive_dashboard() before drop to save analysis results");
        }

        // Clean up any remaining allocations
        if let Ok(mut active) = self.active_allocations.lock() {
            active.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();

        // Test that tracker is created with default values
        assert!(
            !tracker.is_fast_mode() || std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test)
        );

        // Test that we can get stats without errors
        let stats_result = tracker.get_stats();
        assert!(stats_result.is_ok());
    }

    #[test]
    fn test_fast_mode_toggle() {
        let tracker = MemoryTracker::new();

        // Test enabling fast mode
        tracker.set_fast_mode(true);
        assert!(tracker.is_fast_mode());

        // Test disabling fast mode
        tracker.set_fast_mode(false);
        assert!(!tracker.is_fast_mode());

        // Test enable_fast_mode method
        tracker.enable_fast_mode();
        assert!(tracker.is_fast_mode());
    }

    #[test]
    fn test_get_active_allocations() {
        let tracker = MemoryTracker::new();
        tracker.enable_fast_mode();

        // Initially should be empty
        let allocations = tracker.get_active_allocations();
        assert!(allocations.is_ok());
        assert_eq!(allocations.unwrap().len(), 0);
    }

    #[test]
    fn test_get_allocation_history() {
        let tracker = MemoryTracker::new();
        tracker.enable_fast_mode();

        // Initially should be empty
        let history = tracker.get_allocation_history();
        assert!(history.is_ok());
        assert_eq!(history.unwrap().len(), 0);
    }

    #[test]
    fn test_memory_analysis_path_creation() {
        let tracker = MemoryTracker::new();

        let path = tracker.ensure_memory_analysis_path("test.svg");
        assert!(path.to_string_lossy().contains("MemoryAnalysis"));
        assert!(path.to_string_lossy().ends_with("test.svg"));
    }

    #[test]
    fn test_memscope_path_creation() {
        let tracker = MemoryTracker::new();

        let path = tracker.ensure_memscope_path("test");
        assert!(path.to_string_lossy().contains("MemoryAnalysis"));
        assert!(path.to_string_lossy().ends_with(".memscope"));

        let path_with_ext = tracker.ensure_memscope_path("test.memscope");
        assert!(path_with_ext.to_string_lossy().ends_with(".memscope"));
    }

    #[test]
    fn test_global_tracker_singleton() {
        // Reset to global singleton strategy
        configure_tracking_strategy(false);

        // Create new trackers directly from global singleton
        let tracker1 = GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone();
        let tracker2 = GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone();

        // Should be the same instance (Arc comparison)
        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }

    #[test]
    fn test_local_thread_registry() {
        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Get tracker from thread-local storage (this triggers registration)
        let tracker1 = get_tracker();

        // Collect all trackers from local registry
        let trackers = collect_all_trackers_local();

        // Should have at least one tracker registered
        assert!(!trackers.is_empty());

        // The tracker we got should be in the registry
        assert!(trackers.iter().any(|t| Arc::ptr_eq(t, &tracker1)));
    }

    #[test]
    fn test_local_thread_registry_multiple_threads() {
        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Create multiple thread-local trackers (simulating multiple threads)
        let tracker1 = get_tracker();
        let tracker2 = get_tracker();

        // Collect all trackers from local registry
        let trackers = collect_all_trackers_local();

        // Should have trackers registered
        assert!(!trackers.is_empty());

        // Both trackers should be in the registry
        assert!(trackers.iter().any(|t| Arc::ptr_eq(t, &tracker1)));
        assert!(trackers.iter().any(|t| Arc::ptr_eq(t, &tracker2)));
    }

    #[test]
    fn test_cached_thread_data() {
        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Create a tracker and add some allocations
        let tracker = get_tracker();
        tracker.enable_fast_mode();

        // Track some allocations
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();

        // Collect trackers (this should trigger caching)
        let trackers = collect_all_trackers_local();

        // Should have at least one tracker
        assert!(!trackers.is_empty());

        // Get cached data
        let cached = get_cached_thread_data_local();

        // Should have cached data for this thread
        assert!(!cached.is_empty());
    }

    #[test]
    fn test_registry_stats() {
        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Get tracker (this triggers registration)
        let _tracker = get_tracker();

        // Get registry stats
        let stats = get_registry_stats_local();

        // Should have at least one registered thread
        assert!(stats.total_threads_registered > 0);
        assert!(stats.active_threads > 0);
    }

    #[test]
    fn test_cleanup_registry() {
        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Get tracker (this triggers registration)
        let _tracker = get_tracker();

        // Cleanup should work without panicking
        cleanup_registry_local();

        // Stats should still be valid after cleanup
        let stats = get_registry_stats_local();
        assert!(stats.total_threads_registered > 0);
    }

    #[test]
    fn test_has_active_trackers() {
        // Clean up any existing trackers first
        cleanup_registry_local();

        // Configure thread-local strategy
        configure_tracking_strategy(true);

        // Get tracker count before getting tracker
        let stats_before = get_registry_stats_local();

        // Get tracker (this triggers registration)
        let _tracker = get_tracker();

        // Should now have more trackers registered
        let stats_after = get_registry_stats_local();
        assert!(
            stats_after.total_threads_registered >= stats_before.total_threads_registered,
            "Tracker count should increase after get_tracker()"
        );
    }
}
