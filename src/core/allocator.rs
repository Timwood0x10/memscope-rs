//! Custom global allocator for tracking memory allocations.

use std::alloc::{GlobalAlloc, Layout, System};

/// A custom allocator that tracks memory allocations and deallocations.
///
/// This allocator wraps the system allocator and records all allocation
/// and deallocation events through the global memory tracker.
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Create a new tracking allocator instance.
    pub const fn new() -> Self {
        Self
    }
}

// Thread-local flag to prevent recursive tracking
thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Allocate memory first
        let ptr = System.alloc(layout);

        // Track the allocation if it succeeded and tracking is not disabled
        if !ptr.is_null() {
            // Check if tracking is disabled for this thread to prevent recursion
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

            if should_track {
                // Temporarily disable tracking to prevent recursion during tracking operations
                TRACKING_DISABLED.with(|disabled| disabled.set(true));

                // Track the allocation - use try_lock approach to avoid deadlocks
                if let Ok(tracker) =
                    std::panic::catch_unwind(crate::core::tracker::get_global_tracker)
                {
                    // Ignore errors to prevent allocation failures from breaking the program
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }

                // Re-enable tracking
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Track the deallocation first
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

        if should_track {
            // Temporarily disable tracking to prevent recursion
            TRACKING_DISABLED.with(|disabled| disabled.set(true));

            // Track the deallocation - use try_lock approach to avoid deadlocks
            if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_global_tracker)
            {
                // Ignore errors to prevent deallocation failures from breaking the program
                let _ = tracker.track_deallocation(ptr as usize);
            }

            // Re-enable tracking
            TRACKING_DISABLED.with(|disabled| disabled.set(false));
        }

        // Deallocate the memory
        System.dealloc(ptr, layout);
    }
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}
