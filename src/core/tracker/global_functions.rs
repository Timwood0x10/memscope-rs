//! Global convenience functions for memory tracking.

use super::memory_tracker::get_tracker;
use crate::capture::backends::core_types::{AllocationInfo, MemoryStats, TrackingResult};

/// Get the global memory tracker
pub fn get_global_tracker() -> std::sync::Arc<super::MemoryTracker> {
    get_tracker()
}

/// Track allocation - convenience function
pub fn track_allocation(ptr: usize, size: usize) -> TrackingResult<()> {
    let tracker = get_tracker();
    tracker.track_allocation(ptr, size)
}

/// Track deallocation - convenience function
/// Returns true if the allocation was found and removed
pub fn track_deallocation(ptr: usize) -> TrackingResult<bool> {
    let tracker = get_tracker();
    tracker.track_deallocation(ptr)
}

/// Get active allocations - convenience function
///
/// Note: This function calls the underlying tracker's `get_active_allocations()`
/// method. For a unified data source that includes both HeapOwner and Container
/// allocations, use `Tracker::event_store().snapshot()` and
/// `DashboardRenderer::rebuild_allocations_from_events()`.
pub fn get_active_allocations() -> TrackingResult<Vec<AllocationInfo>> {
    let tracker = get_tracker();
    tracker.get_active_allocations()
}

/// Get memory stats - convenience function
pub fn get_stats() -> TrackingResult<MemoryStats> {
    let tracker = get_tracker();
    tracker.get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_global_tracker() {
        let t1 = get_global_tracker();
        let t2 = get_global_tracker();
        assert!(std::sync::Arc::ptr_eq(&t1, &t2));
    }

    #[test]
    fn test_track_allocation() {
        let result = track_allocation(0x1000, 64);
        assert!(result.is_ok());

        let active = get_active_allocations().unwrap();
        assert!(!active.is_empty());

        let _ = track_deallocation(0x1000);
    }
}
