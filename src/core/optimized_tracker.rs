//! Optimized memory tracker using Arc sharing to reduce clone overhead
//!
//! This module provides an optimized version of the memory tracker that uses
//! Arc-based sharing to minimize expensive clone operations while maintaining
//! full API compatibility.

use crate::core::clone_monitor;
use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::shared_types::{SharedAllocationCollection, SharedAllocationInfo};
use crate::core::types::{MemoryStats, TrackingResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Optimized memory tracker that uses Arc sharing
pub struct OptimizedMemoryTracker {
    /// Active allocations using Arc sharing
    active_allocations: Mutex<HashMap<usize, Arc<OptimizedAllocationInfo>>>,
    /// Memory statistics
    stats: Mutex<MemoryStats>,
    /// Allocation history using Arc sharing
    allocation_history: Mutex<Vec<Arc<OptimizedAllocationInfo>>>,
}

impl OptimizedMemoryTracker {
    /// Create a new optimized memory tracker
    pub fn new() -> Self {
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            stats: Mutex::new(MemoryStats::default()),
            allocation_history: Mutex::new(Vec::new()),
        }
    }

    /// Track an allocation with Arc sharing
    pub fn track_allocation_shared(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        let allocation = OptimizedAllocationInfo::new(ptr, size);
        let shared_allocation = Arc::new(allocation);

        // Record that we're using Arc sharing instead of cloning
        clone_monitor::record_avoided_clone("AllocationInfo", size);

        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Insert the Arc - no cloning needed
                active.insert(ptr, Arc::clone(&shared_allocation));

                // Update statistics
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Add to history - sharing the same Arc
                if let Ok(mut history) = self.allocation_history.try_lock() {
                    history.push(shared_allocation);
                }

                Ok(())
            }
            _ => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get active allocations as shared collection
    pub fn get_active_allocations_shared(&self) -> TrackingResult<SharedAllocationCollection> {
        match self.active_allocations.lock() {
            Ok(active) => {
                let shared_infos: Vec<SharedAllocationInfo> = active
                    .values()
                    .map(|arc_info| SharedAllocationInfo::new((**arc_info).clone()))
                    .collect();

                Ok(SharedAllocationCollection::new(shared_infos))
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get allocation history as shared collection
    pub fn get_allocation_history_shared(&self) -> TrackingResult<SharedAllocationCollection> {
        match self.allocation_history.lock() {
            Ok(history) => {
                let shared_infos: Vec<SharedAllocationInfo> = history
                    .iter()
                    .map(|arc_info| SharedAllocationInfo::new((**arc_info).clone()))
                    .collect();

                Ok(SharedAllocationCollection::new(shared_infos))
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Associate variable with existing allocation (Arc-optimized)
    pub fn associate_var_shared(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        match self.active_allocations.lock() {
            Ok(mut active) => {
                if let Some(arc_allocation) = active.get_mut(&ptr) {
                    // Create a new allocation with updated info
                    let mut updated_allocation = (**arc_allocation).clone();
                    updated_allocation.var_name =
                        Some(crate::core::string_pool::intern_string(&var_name));
                    updated_allocation.type_name =
                        Some(crate::core::string_pool::intern_string(&type_name));

                    // Replace with new Arc
                    *arc_allocation = Arc::new(updated_allocation);

                    Ok(())
                } else {
                    Err(crate::core::types::TrackingError::InvalidPointer(
                        "Allocation not found".to_string(),
                    ))
                }
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        match self.stats.lock() {
            Ok(stats) => Ok(stats.clone()),
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get clone optimization statistics
    pub fn get_clone_stats(&self) -> clone_monitor::CloneMonitorStats {
        clone_monitor::get_clone_stats()
    }
}

impl Default for OptimizedMemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
