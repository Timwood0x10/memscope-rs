//! Core memory tracker structure and basic functionality.

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global memory tracker instance
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

/// Get the global memory tracker instance.
///
/// This function returns a reference to the singleton memory tracker
/// that is used throughout the application.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER
        .get_or_init(|| Arc::new(MemoryTracker::new()))
        .clone()
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
pub struct MemoryTracker {
    /// Active allocations (ptr -> allocation info)
    pub active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Complete allocation history (for analysis)
    pub allocation_history: Mutex<Vec<AllocationInfo>>,
    /// Memory usage statistics
    pub stats: Mutex<MemoryStats>,
    /// Fast mode flag for testing (reduces overhead)
    pub fast_mode: std::sync::atomic::AtomicBool,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode =
            std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test) || cfg!(feature = "test");
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_history: Mutex::new(Vec::new()),
            stats: Mutex::new(MemoryStats::default()),
            fast_mode: std::sync::atomic::AtomicBool::new(fast_mode),
        }
    }

    /// Check if tracker is in fast mode (for testing)
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Enable fast mode for testing
    pub fn enable_fast_mode(&self) {
        self.fast_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current memory statistics for analysis and export
    ///
    /// Returns a snapshot of the current memory usage statistics including
    /// allocation counts, memory totals, and performance metrics. This method
    /// provides safe access to internal statistics without exposing implementation details.
    ///
    /// # Returns
    /// A cloned copy of the current MemoryStats structure
    ///
    /// # Example
    /// ```rust
    /// let tracker = get_global_tracker();
    /// let stats = tracker.get_memory_stats();
    /// println!("Active memory: {} bytes", stats.active_memory);
    /// ```
    pub fn get_memory_stats(&self) -> MemoryStats {
        self.stats
            .lock()
            .unwrap_or_else(|poisoned| {
                // Handle poisoned mutex by taking the data anyway
                poisoned.into_inner()
            })
            .clone()
    }

    /// Get all active allocations for analysis and export
    ///
    /// Returns a vector containing information about all currently active
    /// memory allocations. This method provides safe access to allocation
    /// data without exposing internal HashMap structure.
    ///
    /// # Returns
    /// A vector of AllocationInfo structures for all active allocations
    ///
    /// # Performance Note
    /// This method creates a copy of all allocation data, which may be
    /// expensive for large numbers of allocations. Consider using pagination
    /// or filtering for production use with many allocations.
    ///
    /// # Example
    /// ```rust
    /// let tracker = get_global_tracker();
    /// let allocations = tracker.get_all_active_allocations();
    /// println!("Found {} active allocations", allocations.len());
    /// ```
    pub fn get_all_active_allocations(&self) -> Vec<AllocationInfo> {
        self.active_allocations
            .lock()
            .unwrap_or_else(|poisoned| {
                // Handle poisoned mutex by taking the data anyway
                poisoned.into_inner()
            })
            .values()
            .cloned()
            .collect()
    }

    /// Get complete allocation history for comprehensive analysis
    ///
    /// Returns all allocations that have been tracked, including both active
    /// and deallocated allocations. This provides the complete picture of
    /// memory usage patterns throughout the program's execution.
    ///
    /// # Returns
    /// A vector containing all tracked allocations (active and historical)
    ///
    /// # Memory Usage
    /// This method returns a copy of the entire allocation history, which
    /// can be memory-intensive for long-running programs with many allocations.
    ///
    /// # Example
    /// ```rust
    /// let tracker = get_global_tracker();
    /// let history = tracker.get_complete_allocation_history();
    /// let leaked = history.iter().filter(|a| a.is_leaked).count();
    /// println!("Found {} potential memory leaks", leaked);
    /// ```
    pub fn get_complete_allocation_history(&self) -> Vec<AllocationInfo> {
        self.allocation_history
            .lock()
            .unwrap_or_else(|poisoned| {
                // Handle poisoned mutex by taking the data anyway
                poisoned.into_inner()
            })
            .clone()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to convert std::io::Error to TrackingError::IoError
#[allow(dead_code)]
pub(crate) fn io_error_to_tracking_error(e: std::io::Error) -> crate::core::types::TrackingError {
    crate::core::types::TrackingError::IoError(e.to_string())
}