// In src/allocator.rs
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::types::AllocationEvent;

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
            
            // Send allocation event without holding any locks
            let event = AllocationEvent::Alloc {
                ptr: ptr as usize,
                size: layout.size(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
                thread_id: format!("{:?}", std::thread::current().id()),
            };
            
            // Process the event directly without going through the global tracker
            if let Err(e) = crate::types::EVENT_PROCESSOR.send_event(event) {
                eprintln!("Failed to send allocation event: {:?}", e);
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
            
            // Send deallocation event without holding any locks
            let event = AllocationEvent::Dealloc {
                ptr: ptr as usize,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
                thread_id: format!("{:?}", std::thread::current().id()),
            };
            
            // Process the event directly without going through the global tracker
            if let Err(e) = crate::types::EVENT_PROCESSOR.send_event(event) {
                eprintln!("Failed to send deallocation event: {:?}", e);
            }
            
            // Re-enable tracking
            TRACKING_ENABLED.store(true, Ordering::Relaxed);
        }
        
        // Always perform the actual deallocation
        System.dealloc(ptr, layout);
    }
}