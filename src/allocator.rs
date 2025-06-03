// In src/allocator.rs
use std::alloc::{GlobalAlloc, Layout, System};
// No longer using AtomicBool or Ordering here for pure pass-through

// static TRACKING_ENABLED: AtomicBool = AtomicBool::new(true); // Removed

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
        System.alloc(layout) // Pure pass-through
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout) // Pure pass-through
    }
}
