// In src/allocator.rs
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};

static TRACKING_ENABLED: AtomicBool = AtomicBool::new(true);

/// A custom allocator that tracks memory allocations and deallocations
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Create a new instance of the tracking allocator
    pub const fn new() -> Self {
        Self
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() && TRACKING_ENABLED.load(Ordering::Relaxed) {
            // Disable tracking temporarily to prevent recursive allocations
            TRACKING_ENABLED.store(false, Ordering::Relaxed);

            if let Err(e) = crate::tracker::get_global_tracker().track_allocation(
                ptr as usize,
                layout.size(),
                None, // type_name is None at the allocator level
            ) {
                eprintln!("Failed to track allocation: {:?}", e);
            }

            // Re-enable tracking
            TRACKING_ENABLED.store(true, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING_ENABLED.load(Ordering::Relaxed) {
            // Disable tracking temporarily to prevent recursive deallocations
            TRACKING_ENABLED.store(false, Ordering::Relaxed);

            if let Err(e) = crate::tracker::get_global_tracker().track_deallocation(ptr as usize) {
                eprintln!("Failed to track deallocation: {:?}", e);
            }

            // Re-enable tracking
            TRACKING_ENABLED.store(true, Ordering::Relaxed);
        }

        // Always perform the actual deallocation
        System.dealloc(ptr, layout);
    }
}
