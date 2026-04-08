//! Core memory tracker implementation (self-contained, no old system dependencies)
//!
//! This module provides high-performance memory tracking using DashMap
//! and atomic operations, completely independent of the old system.

use super::core_types::{
    AllocationInfo, MemoryStats, ThreadRegistryStats, TrackingError, TrackingResult,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;

const STRATEGY_GLOBAL_SINGLETON: u64 = 0;
const STRATEGY_THREAD_LOCAL: u64 = 1;

pub static TRACKING_STRATEGY: AtomicU64 = AtomicU64::new(STRATEGY_GLOBAL_SINGLETON);

static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

thread_local! {
    static THREAD_LOCAL_TRACKER: Arc<MemoryTracker> = {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker_local(&tracker);
        tracker
    };
}

/// Thread-local registry for tracking thread-local trackers
static LOCAL_THREAD_REGISTRY: OnceLock<
    Arc<dashmap::DashMap<thread::ThreadId, Arc<MemoryTracker>>>,
> = OnceLock::new();

fn get_local_registry() -> Arc<dashmap::DashMap<thread::ThreadId, Arc<MemoryTracker>>> {
    LOCAL_THREAD_REGISTRY
        .get_or_init(|| Arc::new(dashmap::DashMap::new()))
        .clone()
}

fn register_current_thread_tracker_local(tracker: &Arc<MemoryTracker>) {
    let thread_id = thread::current().id();
    get_local_registry().insert(thread_id, tracker.clone());
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
///
/// # Performance Characteristics
/// - **DashMap**: Lock-free concurrent access for allocations
/// - **Atomic counters**: Lock-free statistics updates
/// - **Thread-local**: Zero-contention for per-thread tracking
pub struct MemoryTracker {
    /// Active allocations (DashMap for lock-free concurrent access)
    active_allocations: DashMap<usize, AllocationInfo>,

    /// Atomic statistics (lock-free updates)
    total_allocations: AtomicU64,
    total_allocated: AtomicU64,
    total_deallocations: AtomicU64,
    total_deallocated: AtomicU64,
    peak_allocations: AtomicUsize,
    peak_memory: AtomicU64,

    /// Fast mode flag for testing (reduces overhead)
    fast_mode: AtomicU64,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode = std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test);

        Self {
            active_allocations: DashMap::new(),
            total_allocations: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            peak_allocations: AtomicUsize::new(0),
            peak_memory: AtomicU64::new(0),
            fast_mode: AtomicU64::new(fast_mode as u64),
        }
    }

    /// Track a memory allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        let allocation = AllocationInfo::new(ptr, size);

        // Insert into DashMap (lock-free)
        self.active_allocations.insert(ptr, allocation);

        // Update atomic statistics (lock-free)
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated
            .fetch_add(size as u64, Ordering::Relaxed);

        // Update peak allocations using CAS loop to avoid TOCTOU race
        loop {
            let current_count = self.active_allocations.len();
            let current_peak = self.peak_allocations.load(Ordering::Relaxed);
            if current_count <= current_peak {
                break;
            }
            if self
                .peak_allocations
                .compare_exchange_weak(
                    current_peak,
                    current_count,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        // Update peak memory using CAS loop
        loop {
            let current_memory = self
                .total_allocated
                .load(Ordering::Relaxed)
                .saturating_sub(self.total_deallocated.load(Ordering::Relaxed));
            let current_peak_memory = self.peak_memory.load(Ordering::Relaxed);
            if current_memory <= current_peak_memory {
                break;
            }
            if self
                .peak_memory
                .compare_exchange_weak(
                    current_peak_memory,
                    current_memory,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        Ok(())
    }

    /// Track a memory deallocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    ///
    /// # Returns
    /// * `Ok(true)` if the allocation was found and removed
    /// * `Ok(false)` if the pointer was not tracked (possible double-free or untracked allocation)
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<bool> {
        // Remove from DashMap (lock-free)
        if let Some((_, allocation)) = self.active_allocations.remove(&ptr) {
            // Update atomic statistics (lock-free)
            self.total_deallocations.fetch_add(1, Ordering::Relaxed);
            self.total_deallocated
                .fetch_add(allocation.size as u64, Ordering::Relaxed);
            Ok(true)
        } else {
            // Pointer not found - could be double-free or untracked allocation
            // Log warning in debug mode
            #[cfg(debug_assertions)]
            eprintln!(
                "[memscope] Warning: deallocation called for untracked pointer {:x}. \
                 This may indicate a double-free or memory not tracked by memscope.",
                ptr
            );
            Ok(false)
        }
    }

    /// Get the size of an active allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    ///
    /// # Returns
    /// * `Some(size)` if the allocation exists
    /// * `None` if the pointer is not tracked
    pub fn get_allocation_size(&self, ptr: usize) -> Option<usize> {
        self.active_allocations.get(&ptr).map(|a| a.size)
    }

    /// Associate a variable name and type with an allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `var_name` - Variable name
    /// * `type_name` - Type name
    /// * `source_file` - Source file (optional)
    /// * `source_line` - Source line (optional)
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
        source_file: Option<&str>,
        source_line: Option<u32>,
    ) -> TrackingResult<()> {
        if let Some(mut allocation) = self.active_allocations.get_mut(&ptr) {
            allocation.var_name = Some(var_name);
            allocation.type_name = Some(type_name);
            if let (Some(file), Some(line)) = (source_file, source_line) {
                allocation.set_source_location(file, line);
            }
        }

        Ok(())
    }

    /// Fast track allocation with variable name.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    /// * `var_name` - Variable name
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);

        // Insert into DashMap (lock-free)
        self.active_allocations.insert(ptr, allocation);

        // Update atomic statistics (lock-free)
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated
            .fetch_add(size as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Get current memory statistics.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        let active_count = self.active_allocations.len();
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);
        let total_deallocated = self.total_deallocated.load(Ordering::Relaxed);
        let active_memory = total_allocated.saturating_sub(total_deallocated);

        Ok(MemoryStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_allocated,
            active_allocations: active_count,
            active_memory,
            peak_allocations: self.peak_allocations.load(Ordering::Relaxed),
            peak_memory: self.peak_memory.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            total_deallocated,
            leaked_allocations: 0,
            leaked_memory: 0,
        })
    }

    /// Detect memory leaks at program shutdown.
    ///
    /// This should be called when the program is shutting down to detect
    /// allocations that were never freed. Returns the count and total size
    /// of allocations that are still active.
    pub fn detect_leaks(&self) -> (usize, u64) {
        let active_count = self.active_allocations.len();
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);
        let total_deallocated = self.total_deallocated.load(Ordering::Relaxed);
        let active_memory = total_allocated.saturating_sub(total_deallocated);
        (active_count, active_memory)
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        Ok(self
            .active_allocations
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Get memory grouped by type.
    pub fn get_memory_by_type(&self) -> TrackingResult<std::collections::HashMap<String, usize>> {
        let mut type_sizes: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for entry in self.active_allocations.iter() {
            let alloc = entry.value();
            let type_name = alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            *type_sizes.entry(type_name).or_insert(0) += alloc.size;
        }

        Ok(type_sizes)
    }

    /// Enable or disable fast mode.
    pub fn set_fast_mode(&self, enabled: bool) {
        self.fast_mode.store(enabled as u64, Ordering::Relaxed);
    }

    /// Check if fast mode is enabled.
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(Ordering::Relaxed) != 0
    }

    /// Enable fast mode for testing.
    pub fn enable_fast_mode(&self) {
        self.fast_mode.store(1, Ordering::Relaxed);
    }

    /// Ensure memory analysis path exists and return the full path.
    pub fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::path::PathBuf {
        let path = path.as_ref();
        let memory_analysis_dir = std::path::Path::new("MemoryAnalysis");

        if let Err(e) = std::fs::create_dir_all(memory_analysis_dir) {
            tracing::warn!("Failed to create MemoryAnalysis directory: {}", e);
        }

        memory_analysis_dir.join(path)
    }

    /// Ensure path uses .memscope extension and is in MemoryAnalysis directory.
    pub fn ensure_memscope_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::path::PathBuf {
        let mut output_path = self.ensure_memory_analysis_path(path);

        if output_path.extension().is_none()
            || output_path.extension() != Some(std::ffi::OsStr::new("memscope"))
        {
            output_path.set_extension("memscope");
        }

        output_path
    }

    /// Export memory tracking data to .memscope file format (JSON content).
    ///
    /// This method exports memory tracking data with a .memscope file extension.
    /// The content is serialized as JSON for human readability and interoperability.
    pub fn export_to_memscope<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.export_to_json(path)
    }

    /// Export memory tracking data to JSON format.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(&path);

        let final_path = if output_path.is_dir() {
            output_path.join("memory_analysis.json")
        } else {
            output_path
        };

        let allocations = self.get_active_allocations()?;

        let json = serde_json::to_string_pretty(&allocations)
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

        std::fs::write(&final_path, json).map_err(|e| TrackingError::ExportError(e.to_string()))?;

        Ok(())
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        if std::env::var("MEMSCOPE_VERBOSE").is_ok() {
            tracing::info!(
                "💡 Tip: Use tracker.export_to_json() before drop to save analysis results"
            );
        }

        let active_count = self.active_allocations.len();
        if active_count > 0 {
            tracing::warn!(
                "Dropping MemoryTracker with {} active allocations (potential memory leaks)",
                active_count
            );
        }
    }
}

/// Configure tracking strategy for the application.
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

/// Get the appropriate memory tracker based on the current strategy.
///
/// # Returns
/// * In single-threaded mode: returns the global singleton tracker
/// * In concurrent mode: returns the current thread's local tracker
pub fn get_tracker() -> Arc<MemoryTracker> {
    match TRACKING_STRATEGY.load(Ordering::Relaxed) {
        STRATEGY_GLOBAL_SINGLETON => GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone(),
        STRATEGY_THREAD_LOCAL => THREAD_LOCAL_TRACKER.with(|tracker| tracker.clone()),
        _ => {
            tracing::warn!("Unknown tracking strategy, falling back to global singleton");
            GLOBAL_TRACKER
                .get_or_init(|| Arc::new(MemoryTracker::new()))
                .clone()
        }
    }
}

/// Collect all thread-local trackers.
pub fn collect_all_trackers_local() -> Vec<Arc<MemoryTracker>> {
    get_local_registry()
        .iter()
        .map(|entry| entry.value().clone())
        .collect()
}

/// Get registry statistics.
pub fn get_registry_stats_local() -> ThreadRegistryStats {
    let registry = get_local_registry();
    let total_threads = registry.len();

    ThreadRegistryStats {
        total_threads_registered: total_threads,
        active_threads: total_threads,
        dead_references: 0,
    }
}

/// Check if there are active trackers.
pub fn has_active_trackers_local() -> bool {
    !get_local_registry().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();
        // In test mode, fast_mode is enabled by default
        assert!(tracker.is_fast_mode());
    }

    #[test]
    fn test_track_allocation() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_allocation(0x1000, 1024);
        assert!(result.is_ok());

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.active_allocations, 1);
    }

    #[test]
    fn test_track_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_deallocation(0x1000).unwrap();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.active_allocations, 0);
    }

    #[test]
    fn test_associate_var() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker
            .associate_var(
                0x1000,
                "test_var".to_string(),
                "String".to_string(),
                None,
                None,
            )
            .unwrap();

        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(allocations[0].var_name, Some("test_var".to_string()));
        assert_eq!(allocations[0].type_name, Some("String".to_string()));
    }

    #[test]
    fn test_fast_track_allocation() {
        let tracker = MemoryTracker::new();
        tracker
            .fast_track_allocation(0x1000, 1024, "test_var".to_string())
            .unwrap();

        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(allocations[0].var_name, Some("test_var".to_string()));
    }

    #[test]
    fn test_peak_tracking() {
        let tracker = MemoryTracker::new();

        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();
        tracker.track_allocation(0x3000, 4096).unwrap();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.peak_allocations, 3);
        assert_eq!(stats.peak_memory, 7168);
    }

    #[test]
    fn test_fast_mode() {
        let tracker = MemoryTracker::new();
        tracker.set_fast_mode(true);
        assert!(tracker.is_fast_mode());

        tracker.set_fast_mode(false);
        assert!(!tracker.is_fast_mode());

        tracker.enable_fast_mode();
        assert!(tracker.is_fast_mode());
    }

    #[test]
    fn test_export_to_json() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();

        let result = tracker.export_to_json("test_export");
        assert!(result.is_ok());

        std::fs::remove_file("MemoryAnalysis/test_export.json").ok();
    }

    #[test]
    fn test_global_tracker_singleton() {
        configure_tracking_strategy(false);

        let tracker1 = get_tracker();
        let tracker2 = get_tracker();

        assert!(Arc::ptr_eq(&tracker1, &tracker2));
    }

    #[test]
    fn test_thread_local_tracker() {
        configure_tracking_strategy(true);

        let tracker1 = get_tracker();
        let trackers = collect_all_trackers_local();

        assert!(!trackers.is_empty());
        assert!(trackers.iter().any(|t| Arc::ptr_eq(t, &tracker1)));
    }
}
