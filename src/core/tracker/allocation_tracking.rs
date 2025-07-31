//! Memory allocation tracking methods.

use crate::core::tracker::memory_tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, TrackingResult};

impl MemoryTracker {
    /// Fast track allocation for testing (minimal overhead)
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        if !self.is_fast_mode() {
            return self.create_synthetic_allocation(ptr, size, var_name, "unknown".to_string(), 0);
        }

        // In fast mode, create minimal allocation info but still track it
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);
        allocation.type_name = Some("fast_tracked".to_string());

        // Try to update both active allocations and stats
        if let (Ok(mut active), Ok(mut stats)) =
            (self.active_allocations.try_lock(), self.stats.try_lock())
        {
            active.insert(ptr, allocation);
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }
        Ok(())
    }

    /// Create synthetic allocation for testing purposes
    pub fn create_synthetic_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        line_number: u32,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);
        allocation.type_name = Some(type_name);
        // allocation.line_number = Some(line_number); // TODO: Add line_number field to AllocationInfo

        // Update active allocations
        if let Ok(mut active) = self.active_allocations.try_lock() {
            active.insert(ptr, allocation.clone());
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.try_lock() {
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }

        // Add to history
        if let Ok(mut history) = self.allocation_history.try_lock() {
            history.push(allocation);
        }

        Ok(())
    }

    /// Track a memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        let allocation = AllocationInfo::new(ptr, size);

        // Update active allocations
        if let Ok(mut active) = self.active_allocations.lock() {
            active.insert(ptr, allocation.clone());
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            if stats.active_memory > stats.peak_memory {
                stats.peak_memory = stats.active_memory;
            }
        }

        // Add to history
        if let Ok(mut history) = self.allocation_history.lock() {
            history.push(allocation);
        }

        Ok(())
    }

    /// Track a memory deallocation
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        let mut deallocated_size = 0;

        // Remove from active allocations
        if let Ok(mut active) = self.active_allocations.lock() {
            if let Some(allocation) = active.remove(&ptr) {
                deallocated_size = allocation.size;
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.active_allocations = stats.active_allocations.saturating_sub(1);
            stats.active_memory = stats.active_memory.saturating_sub(deallocated_size);
            stats.total_deallocations = stats.total_deallocations.saturating_add(1);
        }

        Ok(())
    }

    /// Associate a variable name and type with an allocation
    pub fn associate_variable(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        // Update active allocation
        if let Ok(mut active) = self.active_allocations.lock() {
            if let Some(allocation) = active.get_mut(&ptr) {
                allocation.var_name = Some(var_name.clone());
                allocation.type_name = Some(type_name.clone());
            }
        }

        // Update in history as well
        if let Ok(mut history) = self.allocation_history.lock() {
            for allocation in history.iter_mut() {
                if allocation.ptr == ptr {
                    allocation.var_name = Some(var_name.clone());
                    allocation.type_name = Some(type_name.clone());
                }
            }
        }

        Ok(())
    }
}