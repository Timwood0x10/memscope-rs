//! Memory allocation tracking functionality.

use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TypeMemoryUsage};
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
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    /// Complete allocation history (for analysis)
    allocation_history: Mutex<Vec<AllocationInfo>>,
    /// Memory usage statistics
    stats: Mutex<MemoryStats>,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            allocation_history: Mutex::new(Vec::new()),
            stats: Mutex::new(MemoryStats::default()),
        }
    }

    /// Track a new memory allocation.
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        // Create allocation info first (no locks needed)
        let allocation = AllocationInfo::new(ptr, size);

        // Use try_lock to avoid blocking during high allocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Add to active allocations
                active.insert(ptr, allocation.clone());

                // Update statistics with overflow protection
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.total_allocated = stats.total_allocated.saturating_add(size);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Update peaks
                if stats.active_allocations > stats.peak_allocations {
                    stats.peak_allocations = stats.active_allocations;
                }
                if stats.active_memory > stats.peak_memory {
                    stats.peak_memory = stats.active_memory;
                }

                // Release locks before adding to history
                drop(stats);
                drop(active);

                // Add to history with separate try_lock (optional, skip if busy)
                if let Ok(mut history) = self.allocation_history.try_lock() {
                    history.push(allocation);
                }

                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                // This is acceptable as we prioritize program stability over complete tracking
                Ok(())
            }
        }
    }

    /// Track a memory deallocation.
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        // Use try_lock to avoid blocking during high deallocation activity
        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                if let Some(allocation) = active.remove(&ptr) {
                    // Update statistics with overflow protection
                    stats.total_deallocations = stats.total_deallocations.saturating_add(1);
                    stats.total_deallocated =
                        stats.total_deallocated.saturating_add(allocation.size);
                    stats.active_allocations = stats.active_allocations.saturating_sub(1);
                    stats.active_memory = stats.active_memory.saturating_sub(allocation.size);
                }
                Ok(())
            }
            _ => {
                // If we can't get locks immediately, skip tracking to avoid deadlock
                Ok(())
            }
        }
    }

    /// Associate a variable name and type with an allocation.
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Use try_lock to avoid blocking if the allocator is currently tracking
        match self.active_allocations.try_lock() {
            Ok(mut active) => {
                if let Some(allocation) = active.get_mut(&ptr) {
                    allocation.var_name = Some(var_name);
                    allocation.type_name = Some(type_name);
                    Ok(())
                } else {
                    // The pointer might not be tracked by our allocator (e.g., for Rc/Arc data)
                    // or it might be a stack allocation. We'll just skip it gracefully.
                    // This is common for reference-counted types where the data might be
                    // allocated differently than regular heap allocations.
                    Ok(())
                }
            }
            Err(_) => {
                // If we can't get the lock immediately, it's likely the allocator is busy
                // We'll just skip the association to avoid deadlock
                Ok(())
            }
        }
    }

    /// Get current memory usage statistics.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        match self.stats.lock() {
            Ok(stats) => Ok(stats.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let stats = poisoned.into_inner();
                Ok(stats.clone())
            }
        }
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.active_allocations.lock() {
            Ok(active) => Ok(active.values().cloned().collect()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let active = poisoned.into_inner();
                Ok(active.values().cloned().collect())
            }
        }
    }

    /// Get the complete allocation history.
    pub fn get_allocation_history(&self) -> TrackingResult<Vec<AllocationInfo>> {
        match self.allocation_history.lock() {
            Ok(history) => Ok(history.clone()),
            Err(poisoned) => {
                // Handle poisoned lock by recovering the data
                let history = poisoned.into_inner();
                Ok(history.clone())
            }
        }
    }

    /// Get memory usage grouped by type.
    pub fn get_memory_by_type(&self) -> TrackingResult<Vec<TypeMemoryUsage>> {
        // Clone the active allocations to avoid holding the lock for too long
        let active_clone = {
            match self.active_allocations.lock() {
                Ok(active) => active.values().cloned().collect::<Vec<_>>(),
                Err(poisoned) => {
                    // Handle poisoned lock by recovering the data
                    let active = poisoned.into_inner();
                    active.values().cloned().collect::<Vec<_>>()
                }
            }
        };

        let mut type_usage: HashMap<String, (usize, usize)> = HashMap::new();

        for allocation in active_clone {
            let type_name = allocation
                .type_name
                .unwrap_or_else(|| "Unknown".to_string());

            let (total_size, count) = type_usage.entry(type_name).or_insert((0, 0));
            *total_size = total_size.saturating_add(allocation.size);
            *count = count.saturating_add(1);
        }

        let mut result: Vec<TypeMemoryUsage> = type_usage
            .into_iter()
            .map(
                |(type_name, (total_size, allocation_count))| TypeMemoryUsage {
                    type_name,
                    total_size,
                    allocation_count,
                },
            )
            .collect();

        // Sort by total size descending
        result.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        Ok(result)
    }

    /// Export memory data to JSON format.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        crate::export::export_to_json(self, path)
    }

    /// Export memory visualization to SVG format.
    pub fn export_to_svg<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        crate::export::export_to_svg(self, path)
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
