use std::alloc::{GlobalAlloc, Layout, System};
use crate::get_global_tracker;

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
        if !ptr.is_null() {
            let tracker = get_global_tracker();
            if let Err(e) = tracker.track_allocation(ptr as usize, layout.size(), None) {
                eprintln!("Failed to track allocation: {:?}", e);
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let tracker = get_global_tracker();
        if let Err(e) = tracker.track_deallocation(ptr as usize) {
            eprintln!("Failed to track deallocation: {:?}", e);
        }
        System.dealloc(ptr, layout);
    }
}
