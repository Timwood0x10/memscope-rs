//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.


#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod allocator;
pub mod export;
// pub mod procmacros; // This was removed in a previous subtask
pub mod tracker;
pub mod types;

use std::rc::Rc;
use std::sync::Arc;

// Re-export common types for easier use
pub use tracker::{get_global_tracker, MemoryTracker, MemoryError}; // Added MemoryError here
pub use types::AllocationInfo;


// Import the allocator
#[cfg(feature = "tracking-allocator")]
pub use allocator::TrackingAllocator;

/// Global allocator for tracking memory allocations
#[cfg(feature = "tracking-allocator")]
// Only enable the global allocator when the tracking-allocator feature is enabled
// and we're not in test mode
#[cfg(all(feature = "tracking-allocator", not(test)))]
#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Initialize the memory tracking system
///
/// This function sets up the tracing subscriber with default configuration.
/// It should be called early in your application's main function.
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .try_init();
}

/// Track a variable's memory allocation
///
/// This macro associates a variable name with its memory allocation
/// for better tracking and visualization.
#[macro_export]
macro_rules! track_var {
    ($var:ident) => {
        if let Err(e) = $crate::__internal_track_var(stringify!($var), &$var) {
             ::tracing::error!("Failed to track variable '{}': {}", stringify!($var), e);
        }
    };
}

/// Internal function used by the `track_var!` macro
#[doc(hidden)]
pub fn __internal_track_var<T: Trackable>(name: &str, value: &T) -> Result<(), MemoryError> {
    if let Some(ptr) = value.get_trackable_raw_ptr() {
        let tracker = get_global_tracker();
        let type_name = value.get_type_name(); // Use trait method for consistency
        tracker.associate_var(ptr, name.to_string(), type_name)
    } else {
        // If a type is not heap-allocated or empty (e.g. empty Vec/String), it might return None.
        // This is not necessarily an error from track_var's perspective.
        Ok(())
    }
}

/// Trait for types that can be tracked by the memory tracker
pub trait Trackable {
    /// Get a raw pointer to the tracked allocation
    fn get_trackable_raw_ptr(&self) -> Option<usize>;
    
    /// Get the type name as a string
    fn get_type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

// Implement Trackable for common types
impl<T> Trackable for Box<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        Some(Box::as_ref(self) as *const T as usize)
    }
}

impl<T> Trackable for Vec<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.is_empty() { // Important: empty Vecs might not have a valid pointer from as_ptr()
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}

impl Trackable for String {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.is_empty() { // Similar to Vec
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}

// Implement Trackable for Rc<T>
impl<T> Trackable for Rc<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        Some(Rc::as_ptr(self) as usize)
    }
}

// Implement Trackable for Arc<T>
impl<T> Trackable for Arc<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        Some(Arc::as_ptr(self) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Brings init, track_var!, Trackable, __internal_track_var, etc. into scope
    use crate::tracker::get_global_tracker; // For accessing the GLOBAL_TRACKER
    use crate::types::AllocationInfo;
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    // Setup function that runs once before any tests
    fn setup() {
        INIT.call_once(|| {
            // Initialize the global tracker without enabling the global allocator
            let _ = get_global_tracker();
        });
    }

    // Helper function to find an allocation by pointer in active allocations
    fn find_active_allocation_info(ptr: usize) -> Option<crate::tracker::AllocationInfo> {
        let tracker = get_global_tracker();
        tracker.get_active_allocations().into_iter().find(|a| a.ptr == ptr)
    }
    
    // Helper to clean up a tracked variable by deallocating it from the GLOBAL_TRACKER
    // This is important because GLOBAL_TRACKER is shared state across these tests.
    fn cleanup_tracked_var(ptr: usize) {
        if ptr != 0 { // Ensure ptr is not null before trying to deallocate
            let tracker = get_global_tracker();
            // This attempts to move it to the allocation_log.
            // Errors are ignored as the main purpose is cleanup of active_allocations.
            let _ = tracker.track_deallocation(ptr);
        }
    }

    // Resets parts of the global tracker for a cleaner test slate.
    // WARNING: This is a simplified reset. A full reset of lazy_static is hard.
    // This clears active allocations and the log, which is good enough for these tests.
    fn partial_reset_global_tracker() {
        let tracker = get_global_tracker();
        // Drop the locks immediately after getting the values to avoid holding them
        // while potentially allocating memory during clear()
        {
            let mut active = tracker.active_allocations.lock().unwrap();
            active.clear();
        }
        {
            let mut log = tracker.allocation_log.lock().unwrap();
            log.clear();
        }
    }


    #[test]
    fn test_track_var_macro_vec() {
        setup(); // Initialize test environment
        partial_reset_global_tracker();
        
        // Create a vector and get its pointer before tracking
        let my_vec = vec![1, 2, 3];
        let vec_ptr = my_vec.as_ptr() as usize;

        // Simulate allocation being tracked by the global allocator
        // This step is essential for associate_var (called by track_var!) to succeed.
        get_global_tracker().track_allocation(vec_ptr, std::mem::size_of_val(my_vec.as_slice()), Some(my_vec.get_type_name())).unwrap();

        track_var!(my_vec);

        let info = find_active_allocation_info(vec_ptr)
            .expect("Vec allocation info not found after track_var!");

        assert_eq!(info.var_name.as_deref(), Some("my_vec"), "var_name mismatch for Vec");
        assert_eq!(info.type_name.as_deref(), Some(std::any::type_name::<Vec<i32>>().as_ref()), "type_name mismatch for Vec");

        cleanup_tracked_var(vec_ptr);
    }

    #[test]
    fn test_track_var_macro_string_box_rc_arc() {
        setup();
        partial_reset_global_tracker();
        
        // Test String
        let mut my_string = String::with_capacity(20);
        my_string.push_str("test string");
        let string_ptr = my_string.as_ptr() as usize;
        // For String, capacity is a reasonable proxy for allocated size if non-empty.
        // If empty, get_trackable_raw_ptr returns None, so track_allocation wouldn't be called by allocator.
        if string_ptr != 0 { // only simulate track_allocation if pointer is valid
             get_global_tracker().track_allocation(string_ptr, my_string.capacity(), Some(my_string.get_type_name())).unwrap();
        }
        track_var!(my_string); // Should call __internal_track_var
        if string_ptr != 0 { // Only assert if we expected it to be tracked
            let string_info = find_active_allocation_info(string_ptr).expect("String info not found");
            assert_eq!(string_info.var_name.as_deref(), Some("my_string"));
            assert_eq!(string_info.type_name.as_deref(), Some(std::any::type_name::<String>().as_ref()));
            cleanup_tracked_var(string_ptr);
        }

        // Test Box
        partial_reset_global_tracker(); // Clean before next variable
        let my_box = Box::new("boxed string");
        
        // Get the pointer that the Trackable implementation would use
        let box_ptr = my_box.get_trackable_raw_ptr().expect("Box should have a trackable pointer");
        
        // Track the allocation manually
        get_global_tracker().track_allocation(
            box_ptr, 
            std::mem::size_of_val(&*my_box), 
            Some(my_box.get_type_name())
        ).unwrap();
        
        // Use track_var which will use get_trackable_raw_ptr
        track_var!(my_box);
        
        // The pointer used by track_var for Box<&str> will be the address of the &str fat pointer on the heap.
        // This is different from box_inner_ptr. So we need to use the pointer from the Trackable trait.
        let trackable_box_ptr = my_box.get_trackable_raw_ptr().expect("Box should have a trackable ptr");
        // We need to ensure this trackable_box_ptr was what the allocator tracked, or adjust the test.
        // For simplicity, let's assume the allocator would track `trackable_box_ptr`.
        // So, we should have called track_allocation with `trackable_box_ptr`.
        // Since we can't easily re-do track_allocation, we'll adjust the test logic slightly.
        // The critical part is that __internal_track_var uses my_box.get_trackable_raw_ptr().
        // We need to ensure that this pointer is what we query.
        
        // Re-simulating allocation for the actual pointer track_var will use:
        partial_reset_global_tracker();
        get_global_tracker().track_allocation(trackable_box_ptr, std::mem::size_of::<&str>(), Some(my_box.get_type_name())).unwrap();
        track_var!(my_box);

        let box_info = find_active_allocation_info(trackable_box_ptr).expect("Box info not found");
        assert_eq!(box_info.var_name.as_deref(), Some("my_box"));
        assert_eq!(box_info.type_name.as_deref(), Some(std::any::type_name::<Box<&str>>().as_ref()));
        cleanup_tracked_var(trackable_box_ptr);


        // Test Rc
        partial_reset_global_tracker();
        let my_rc = Rc::new(true);
        let rc_ptr = my_rc.get_trackable_raw_ptr().expect("Rc should have a trackable ptr");
        get_global_tracker().track_allocation(rc_ptr, std::mem::size_of_val(&*my_rc), Some(my_rc.get_type_name())).unwrap();
        track_var!(my_rc);
        let rc_info = find_active_allocation_info(rc_ptr).expect("Rc info not found");
        assert_eq!(rc_info.var_name.as_deref(), Some("my_rc"));
        assert_eq!(rc_info.type_name.as_deref(), Some(std::any::type_name::<Rc<bool>>().as_ref()));
        cleanup_tracked_var(rc_ptr);
        
        // Test Arc
        partial_reset_global_tracker();
        let my_arc = Arc::new(123.45f64);
        let arc_ptr = my_arc.get_trackable_raw_ptr().expect("Arc should have a trackable ptr");
        get_global_tracker().track_allocation(arc_ptr, std::mem::size_of_val(&*my_arc), Some(my_arc.get_type_name())).unwrap();
        track_var!(my_arc);
        let arc_info = find_active_allocation_info(arc_ptr).expect("Arc info not found");
        assert_eq!(arc_info.var_name.as_deref(), Some("my_arc"));
        assert_eq!(arc_info.type_name.as_deref(), Some(std::any::type_name::<Arc<f64>>().as_ref()));
        cleanup_tracked_var(arc_ptr);
    }
    
    // test_track_var_logging_on_error:
    // As discussed, direct assertion of `tracing::error!` is omitted due to complexity.
    // The `if let Err(e)` structure in `track_var!` covers the error path.
}
