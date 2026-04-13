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
use tracing::warn;

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
            warn!(
                "deallocation called for untracked pointer {:x}. \
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
        let base_dir = path
            .parent()
            .unwrap_or(std::path::Path::new("MemoryAnalysis"));

        if let Err(e) = std::fs::create_dir_all(base_dir) {
            tracing::warn!("Failed to create directory {:?}: {}", base_dir, e);
        }

        if base_dir == std::path::Path::new("") {
            std::path::Path::new("MemoryAnalysis").join(path.file_name().unwrap_or_default())
        } else {
            path.to_path_buf()
        }
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

        // Clear active_allocations to release memory
        self.active_allocations.clear();
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
    use std::sync::Arc;
    use std::thread;

    /// Objective: Verify MemoryTracker creation with default values
    /// Invariants: New tracker should have zero allocations and fast_mode enabled in test
    #[test]
    fn test_memory_tracker_creation() {
        let tracker = MemoryTracker::new();
        assert!(
            tracker.is_fast_mode(),
            "Fast mode should be enabled in test mode"
        );

        let stats = tracker.get_stats().expect("Should get stats");
        assert_eq!(
            stats.total_allocations, 0,
            "Initial total allocations should be 0"
        );
        assert_eq!(
            stats.active_allocations, 0,
            "Initial active allocations should be 0"
        );
        assert_eq!(
            stats.peak_allocations, 0,
            "Initial peak allocations should be 0"
        );
    }

    /// Objective: Verify Default trait implementation
    /// Invariants: Default should create same as new()
    #[test]
    fn test_memory_tracker_default() {
        let tracker = MemoryTracker::default();
        let stats = tracker.get_stats().expect("Should get stats");
        assert_eq!(
            stats.total_allocations, 0,
            "Default tracker should have 0 allocations"
        );
    }

    /// Objective: Verify track_allocation updates statistics correctly
    /// Invariants: Should increment total_allocations and active_allocations
    #[test]
    fn test_track_allocation() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_allocation(0x1000, 1024);
        assert!(result.is_ok(), "track_allocation should succeed");

        let stats = tracker.get_stats().expect("Should get stats");
        assert_eq!(stats.total_allocations, 1, "Total allocations should be 1");
        assert_eq!(
            stats.active_allocations, 1,
            "Active allocations should be 1"
        );
        assert_eq!(
            stats.total_allocated, 1024,
            "Total allocated should be 1024"
        );
    }

    /// Objective: Verify track_deallocation removes allocation correctly
    /// Invariants: Should decrement active_allocations and increment total_deallocations
    #[test]
    fn test_track_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        let result = tracker.track_deallocation(0x1000);
        assert!(result.is_ok(), "track_deallocation should succeed");
        assert!(
            result.unwrap(),
            "track_deallocation should return true for tracked pointer"
        );

        let stats = tracker.get_stats().expect("Should get stats");
        assert_eq!(
            stats.total_deallocations, 1,
            "Total deallocations should be 1"
        );
        assert_eq!(
            stats.active_allocations, 0,
            "Active allocations should be 0"
        );
        assert_eq!(
            stats.total_deallocated, 1024,
            "Total deallocated should be 1024"
        );
    }

    /// Objective: Verify deallocation of untracked pointer returns false
    /// Invariants: Should return Ok(false) for untracked pointer
    #[test]
    fn test_deallocation_untracked_pointer() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_deallocation(0xdead);
        assert!(result.is_ok(), "Should not error on untracked pointer");
        assert!(
            !result.unwrap(),
            "Should return false for untracked pointer"
        );
    }

    /// Objective: Verify get_allocation_size returns correct size
    /// Invariants: Should return Some(size) for tracked pointer
    #[test]
    fn test_get_allocation_size() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 2048).unwrap();

        let size = tracker.get_allocation_size(0x1000);
        assert_eq!(size, Some(2048), "Should return correct allocation size");
    }

    /// Objective: Verify get_allocation_size returns None for untracked pointer
    /// Invariants: Should return None for untracked pointer
    #[test]
    fn test_get_allocation_size_untracked() {
        let tracker = MemoryTracker::new();

        let size = tracker.get_allocation_size(0xdead);
        assert!(size.is_none(), "Should return None for untracked pointer");
    }

    /// Objective: Verify associate_var updates allocation metadata
    /// Invariants: Should set var_name and type_name correctly
    #[test]
    fn test_associate_var() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker
            .associate_var(
                0x1000,
                "test_var".to_string(),
                "String".to_string(),
                Some("test.rs"),
                Some(42),
            )
            .unwrap();

        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(
            allocations[0].var_name,
            Some("test_var".to_string()),
            "Variable name should be set"
        );
        assert_eq!(
            allocations[0].type_name,
            Some("String".to_string()),
            "Type name should be set"
        );
    }

    /// Objective: Verify associate_var for non-existent pointer
    /// Invariants: Should succeed silently without error
    #[test]
    fn test_associate_var_nonexistent() {
        let tracker = MemoryTracker::new();
        let result = tracker.associate_var(
            0xdead,
            "test_var".to_string(),
            "String".to_string(),
            None,
            None,
        );
        assert!(
            result.is_ok(),
            "Should succeed silently for non-existent pointer"
        );
    }

    /// Objective: Verify fast_track_allocation with variable name
    /// Invariants: Should track allocation with var_name in one call
    #[test]
    fn test_fast_track_allocation() {
        let tracker = MemoryTracker::new();
        tracker
            .fast_track_allocation(0x1000, 1024, "test_var".to_string())
            .unwrap();

        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(
            allocations[0].var_name,
            Some("test_var".to_string()),
            "Variable name should be set"
        );
        assert_eq!(allocations[0].size, 1024, "Size should be correct");
    }

    /// Objective: Verify peak allocations tracking
    /// Invariants: Peak should track maximum concurrent allocations
    #[test]
    fn test_peak_tracking() {
        let tracker = MemoryTracker::new();

        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();
        tracker.track_allocation(0x3000, 4096).unwrap();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.peak_allocations, 3, "Peak allocations should be 3");
        assert_eq!(stats.peak_memory, 7168, "Peak memory should be 7168");

        tracker.track_deallocation(0x1000).unwrap();
        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.peak_allocations, 3,
            "Peak should remain 3 after deallocation"
        );
        assert_eq!(
            stats.active_allocations, 2,
            "Active should be 2 after deallocation"
        );
    }

    /// Objective: Verify fast mode toggle
    /// Invariants: Fast mode should be toggleable
    #[test]
    fn test_fast_mode() {
        let tracker = MemoryTracker::new();
        tracker.set_fast_mode(true);
        assert!(tracker.is_fast_mode(), "Fast mode should be enabled");

        tracker.set_fast_mode(false);
        assert!(!tracker.is_fast_mode(), "Fast mode should be disabled");

        tracker.enable_fast_mode();
        assert!(tracker.is_fast_mode(), "Fast mode should be enabled again");
    }

    /// Objective: Verify detect_leaks returns correct counts
    /// Invariants: Should return count and size of active allocations
    #[test]
    fn test_detect_leaks() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();

        let (count, size) = tracker.detect_leaks();
        assert_eq!(count, 2, "Should detect 2 leaks");
        assert_eq!(size, 3072, "Total leak size should be 3072");

        tracker.track_deallocation(0x1000).unwrap();
        let (count, _) = tracker.detect_leaks();
        assert_eq!(count, 1, "Should detect 1 leak after deallocation");
    }

    /// Objective: Verify get_memory_by_type groups allocations
    /// Invariants: Should group allocations by type name
    #[test]
    fn test_get_memory_by_type() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();

        tracker
            .associate_var(0x1000, "v1".to_string(), "String".to_string(), None, None)
            .unwrap();
        tracker
            .associate_var(0x2000, "v2".to_string(), "String".to_string(), None, None)
            .unwrap();

        let by_type = tracker.get_memory_by_type().unwrap();
        assert_eq!(
            by_type.get("String"),
            Some(&3072),
            "String type should have 3072 bytes"
        );
    }

    /// Objective: Verify get_memory_by_type with unknown types
    /// Invariants: Unknown types should be grouped as "unknown"
    #[test]
    fn test_get_memory_by_type_unknown() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();

        let by_type = tracker.get_memory_by_type().unwrap();
        assert_eq!(
            by_type.get("unknown"),
            Some(&1024),
            "Unknown type should have 1024 bytes"
        );
    }

    /// Objective: Verify export_to_json creates valid file
    /// Invariants: Should create file with valid JSON content
    #[test]
    fn test_export_to_json() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_export.json");
        let result = tracker.export_to_json(&file_path);
        assert!(result.is_ok(), "Export should succeed");
        assert!(file_path.exists(), "Export file should exist");

        let content = std::fs::read_to_string(&file_path).expect("Should read file");
        assert!(!content.is_empty(), "JSON content should not be empty");
        assert!(content.contains("size"), "JSON should contain size field");
    }

    /// Objective: Verify export_to_memscope creates file
    /// Invariants: Should create file with valid content
    #[test]
    fn test_export_to_memscope() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_export.memscope");
        let result = tracker.export_to_memscope(&file_path);
        assert!(result.is_ok(), "Export should succeed: {:?}", result);

        assert!(
            file_path.exists(),
            "Export file should exist at {:?}",
            file_path
        );

        let content = std::fs::read_to_string(&file_path).expect("Should read file");
        assert!(!content.is_empty(), "Export content should not be empty");
    }

    /// Objective: Verify ensure_memory_analysis_path creates directory
    /// Invariants: Should create parent directory if needed
    #[test]
    fn test_ensure_memory_analysis_path() {
        let tracker = MemoryTracker::new();
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let nested_path = temp_dir.path().join("nested").join("dir").join("file.json");

        let result = tracker.ensure_memory_analysis_path(&nested_path);
        assert!(
            result.parent().unwrap().exists(),
            "Parent directory should be created"
        );
    }

    /// Objective: Verify global tracker singleton behavior
    /// Invariants: Should return same instance when using global strategy
    #[test]
    fn test_global_tracker_singleton() {
        let tracker1 = GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone();
        let tracker2 = GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone();

        assert!(
            Arc::ptr_eq(&tracker1, &tracker2),
            "Should return same instance from GLOBAL_TRACKER"
        );
    }

    /// Objective: Verify thread-local tracker behavior
    /// Invariants: Should register thread-local tracker
    #[test]
    fn test_thread_local_tracker() {
        configure_tracking_strategy(true);

        let tracker1 = get_tracker();
        let trackers = collect_all_trackers_local();

        assert!(!trackers.is_empty(), "Should have at least one tracker");
        assert!(
            trackers.iter().any(|t| Arc::ptr_eq(t, &tracker1)),
            "Current thread's tracker should be in registry"
        );
    }

    /// Objective: Verify registry statistics
    /// Invariants: Should return correct thread count
    #[test]
    fn test_registry_stats() {
        configure_tracking_strategy(true);
        get_tracker();

        let stats = get_registry_stats_local();
        assert!(
            stats.total_threads_registered >= 1,
            "Should have at least one thread registered"
        );
        assert_eq!(stats.dead_references, 0, "Should have no dead references");
    }

    /// Objective: Verify has_active_trackers_local
    /// Invariants: Should return true when trackers exist
    #[test]
    fn test_has_active_trackers() {
        configure_tracking_strategy(true);
        get_tracker();

        assert!(has_active_trackers_local(), "Should have active trackers");
    }

    /// Objective: Verify concurrent allocation tracking
    /// Invariants: Should handle concurrent allocations correctly
    #[test]
    fn test_concurrent_allocations() {
        let tracker = Arc::new(MemoryTracker::new());
        let mut handles = vec![];

        for i in 0..10 {
            let tracker_clone = tracker.clone();
            let handle = thread::spawn(move || {
                let ptr = 0x1000 + i * 0x100;
                tracker_clone.track_allocation(ptr, 1024).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.total_allocations, 10,
            "Should have 10 allocations from 10 threads"
        );
    }

    /// Objective: Verify concurrent allocation and deallocation
    /// Invariants: Should maintain consistency under concurrent operations
    #[test]
    fn test_concurrent_alloc_dealloc() {
        let tracker = Arc::new(MemoryTracker::new());
        let mut handles = vec![];

        for i in 0..5 {
            let tracker_clone = tracker.clone();
            let handle = thread::spawn(move || {
                let ptr = 0x1000 + i * 0x100;
                tracker_clone.track_allocation(ptr, 1024).unwrap();
                tracker_clone.track_deallocation(ptr).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.active_allocations, 0,
            "All allocations should be deallocated"
        );
        assert_eq!(
            stats.total_allocations, 5,
            "Should have 5 total allocations"
        );
        assert_eq!(
            stats.total_deallocations, 5,
            "Should have 5 total deallocations"
        );
    }

    /// Objective: Verify zero-size allocation handling
    /// Invariants: Should handle zero-size allocation without error
    #[test]
    fn test_zero_size_allocation() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_allocation(0x1000, 0);
        assert!(result.is_ok(), "Zero-size allocation should succeed");

        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.total_allocations, 1,
            "Should count zero-size allocation"
        );
    }

    /// Objective: Verify large allocation handling
    /// Invariants: Should handle large allocations correctly
    #[test]
    fn test_large_allocation() {
        let tracker = MemoryTracker::new();
        let large_size = 1024 * 1024 * 1024;
        let result = tracker.track_allocation(0x1000, large_size);
        assert!(result.is_ok(), "Large allocation should succeed");

        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.total_allocated as usize, large_size,
            "Should track large allocation size"
        );
    }

    /// Objective: Verify multiple allocations at same address
    /// Invariants: Later allocation should overwrite earlier one
    #[test]
    fn test_duplicate_address_allocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x1000, 2048).unwrap();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(
            stats.active_allocations, 1,
            "Should have 1 active allocation"
        );
        assert_eq!(stats.total_allocations, 2, "Should count both allocations");

        let size = tracker.get_allocation_size(0x1000);
        assert_eq!(size, Some(2048), "Should have later allocation size");
    }

    /// Objective: Verify Drop implementation logs warnings
    /// Invariants: Should not panic when dropping with active allocations
    #[test]
    fn test_drop_with_active_allocations() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();

        drop(tracker);
    }

    /// Objective: Verify get_active_allocations returns all allocations
    /// Invariants: Should return all active allocations
    #[test]
    fn test_get_active_allocations() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        tracker.track_allocation(0x2000, 2048).unwrap();
        tracker.track_deallocation(0x1000).unwrap();

        let allocations = tracker.get_active_allocations().unwrap();
        assert_eq!(allocations.len(), 1, "Should have 1 active allocation");
        assert_eq!(
            allocations[0].ptr, 0x2000,
            "Remaining allocation should be at 0x2000"
        );
    }

    /// Objective: Verify configure_tracking_strategy logging
    /// Invariants: Should accept both strategies
    #[test]
    fn test_configure_tracking_strategy() {
        configure_tracking_strategy(false);
        assert_eq!(
            TRACKING_STRATEGY.load(Ordering::Relaxed),
            STRATEGY_GLOBAL_SINGLETON,
            "Should set global singleton strategy"
        );

        configure_tracking_strategy(true);
        assert_eq!(
            TRACKING_STRATEGY.load(Ordering::Relaxed),
            STRATEGY_THREAD_LOCAL,
            "Should set thread-local strategy"
        );
    }
}
