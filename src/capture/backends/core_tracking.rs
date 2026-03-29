//! Core tracking functionality for memory allocations.
//!
//! This module contains simplified tracking implementation for memory allocations.

use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};

use super::core_tracker::MemoryTracker;

macro_rules! acquire_lock {
    ($lock:expr, $name:literal) => {
        $lock
            .lock()
            .map_err(|_| TrackingError::LockError($name.to_string()))?
    };
}

impl MemoryTracker {
    /// Simplified track allocation implementation.
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        let allocation = AllocationInfo::new(ptr, size);

        let mut active = acquire_lock!(self.active_allocations, "active_allocations");
        let mut bounded_stats = acquire_lock!(self.bounded_stats, "bounded_stats");

        active.insert(ptr, allocation.clone());
        bounded_stats.add_allocation(&allocation);

        Ok(())
    }

    /// Simplified track deallocation implementation.
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        let mut active = acquire_lock!(self.active_allocations, "active_allocations");

        if let Some(allocation) = active.remove(&ptr) {
            let mut bounded_stats = acquire_lock!(self.bounded_stats, "bounded_stats");
            bounded_stats.record_deallocation(ptr, allocation.size);
        }

        Ok(())
    }

    /// Simplified associate variable implementation.
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        let mut active = acquire_lock!(self.active_allocations, "active_allocations");

        if let Some(allocation) = active.get_mut(&ptr) {
            allocation.var_name = Some(var_name);
            allocation.type_name = Some(type_name);
        }

        Ok(())
    }

    /// Simplified fast track allocation implementation.
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.var_name = Some(var_name);

        let mut active = acquire_lock!(self.active_allocations, "active_allocations");
        let mut bounded_stats = acquire_lock!(self.bounded_stats, "bounded_stats");

        active.insert(ptr, allocation.clone());
        bounded_stats.add_allocation(&allocation);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_allocation() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_allocation(0x1000, 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_track_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        let result = tracker.track_deallocation(0x1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_associate_var() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 1024).unwrap();
        let result = tracker.associate_var(0x1000, "test_var".to_string(), "Vec<u8>".to_string());
        assert!(result.is_ok());
    }
}
