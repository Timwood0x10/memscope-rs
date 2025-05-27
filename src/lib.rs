//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

/// Provides a custom global allocator (`TrackingAllocator`) for monitoring memory operations.
///
/// When the `tracking-allocator` feature is enabled (which it is by default),
/// this module's `TrackingAllocator` is set as the `#[global_allocator]`.
/// It intercepts all standard heap allocations and deallocations, records them
/// via the `MemoryTracker`, and then forwards them to the system's default allocator.
pub mod allocator;

/// Handles the export of tracked memory data into various formats.
///
/// Currently supports exporting:
/// *   A snapshot of active allocations to JSON (`MemorySnapshot`).
/// *   A timeline visualization of allocation lifecycles (from the deallocation log) to SVG.
pub mod export;
// pub mod procmacros; // This was removed in a previous subtask
/// Core data structures and logic for tracking memory allocations.
pub mod tracker;
// pub mod types; // types.rs is deleted
/// Provides functionality to compare memory snapshots and identify differences.
pub mod diff_engine;

use std::rc::Rc;
use std::sync::Arc;

// Re-export common types for easier use
pub use tracker::{get_global_tracker, MemoryTracker, MemoryError, AllocationInfo, HotspotInfo, TypeMemoryUsage};
pub use export::MemorySnapshot;
pub use diff_engine::SnapshotDiff; // Re-export SnapshotDiff


// Import the allocator
#[cfg(feature = "tracking-allocator")]
pub use allocator::TrackingAllocator;

/// Global allocator for tracking memory allocations
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Initialize the memory tracking system.
///
/// This function sets up the tracing subscriber with default configuration.
/// It should be called early in your application's main function to ensure
/// that tracing and memory tracking are active from the start.
///
/// # Example
/// ```
/// use trace_tools::init;
///
/// fn main() {
///     init();
///     // Your application code here
/// }
/// ```
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

/// Associates a variable name and its type with its heap allocation via the [`MemoryTracker`].
///
/// This macro relies on the variable implementing the [`Trackable`] trait, which provides
/// the necessary pointer to the heap-allocated data. If the variable's pointer can be obtained,
/// this macro calls `tracker.associate_var` to record the variable's name and type.
///
/// If the process of associating the variable fails (e.g., if the pointer obtained from
/// `Trackable` is not currently tracked by the allocator, or if `Trackable` returns `None`),
/// an error is logged using `tracing::error!`.
///
/// ## Usage
///
/// ```
/// use trace_tools::{init, track_var, Trackable, get_global_tracker, MemoryError};
///
/// // In a real scenario with the custom global allocator, `track_allocation`
/// // would be called by the allocator itself. For this example, we simulate it.
/// fn simulate_allocation_and_track<T: Trackable>(var: &T, name: &str, size: usize) {
///     if let Some(ptr) = var.get_trackable_raw_ptr() {
///         let type_name = var.get_type_name();
///         get_global_tracker().track_allocation(ptr, size, Some(type_name.clone())).unwrap();
///         // Now call track_var!
///         // Note: track_var!(var_name_literal) expects a literal identifier.
///         // To use a string name, you'd call __internal_track_var directly (not recommended for users).
///     }
/// }
///
/// fn main() {
///     init(); // Initialize tracing and memory tracker
///
///     let my_vec = vec![1, 2, 3];
///     // Simulate the allocator tracking this Vec's allocation.
///     if let Some(ptr) = my_vec.get_trackable_raw_ptr() {
///         get_global_tracker().track_allocation(
///             ptr,
///             std::mem::size_of_val(my_vec.as_slice()),
///             Some(my_vec.get_type_name())
///         ).unwrap();
///     }
///     track_var!(my_vec); // Associates "my_vec" with its allocation data
///
///     // ... later in your code, the allocation data for my_vec can be exported.
/// #   if let Some(ptr) = my_vec.get_trackable_raw_ptr() { // Cleanup for example
/// #       let _ = get_global_tracker().track_deallocation(ptr);
/// #   }
/// }
/// ```
#[macro_export]
macro_rules! track_var {
    ($var:ident) => {
        // Rely on `Trackable` being in scope at the call site.
        // Users of the macro (including examples/tests) will need to:
        // `use trace_tools::Trackable;` or `use crate::Trackable;`
        if let Err(e) = $crate::__internal_track_var(stringify!($var), &$var) {
             tracing::error!("Failed to track variable '{}': {}", stringify!($var), e);
        }
    };
}

/// Internal helper function for `track_var!`. **Not intended for direct public use.**
///
/// This function performs the actual work of associating a variable's name and type
/// with its memory allocation data via the global [`MemoryTracker`].
///
/// # Arguments
/// * `name`: The string representation of the variable's name.
/// * `value`: A reference to the variable, which must implement [`Trackable`].
///
/// # Returns
/// * `Ok(())` if the association was successful or if the variable provided no pointer
///   (e.g., an empty `Vec` or `String`, which is not considered an error here).
/// * `Err(MemoryError)` if the association failed (e.g., pointer not tracked by allocator).
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

/// A trait for types whose heap allocations can be tracked by `track_var!`.
///
/// Implementing this trait allows `track_var!` to obtain a raw pointer to the
/// heap-allocated data associated with an instance of the type. This pointer is
/// then used by the [`MemoryTracker`] to associate a variable name and type information
/// with the corresponding allocation record.
///
/// ## How to Implement
///
/// 1.  **`get_trackable_raw_ptr(&self) -> Option<usize>`**:
///     This method should return `Some(pointer_as_usize)` if the instance has
///     heap-allocated data that `track_var!` should be associated with. The pointer
///     should be the starting address of the heap allocation.
///     If the instance does not have a direct heap allocation (e.g., it's empty,
///     like an empty `Vec` or `String`, or it's a type that is entirely stack-allocated
///     but might be part of a larger heap-allocated structure elsewhere), it can
///     return `None`. Returning `None` signifies that `track_var!` should not attempt
///     to associate this specific instance with an allocation.
///
/// 2.  **`get_type_name(&self) -> String` (Optional override)**:
///     This method has a default implementation that returns `std::any::type_name::<Self>()`.
///     For most types, the default implementation is sufficient. You might override it
///     if you need a more specific or customized type name for tracking purposes.
///
/// ## Example Implementation for a Custom Type
///
/// ```
/// use trace_tools::{Trackable, MemoryError}; // Assuming MemoryError is also pub use
///
/// struct MyStruct {
///     data: Box<[i32]>, // Heap-allocated data
///     name: String,     // String is also heap-allocated
/// }
///
/// impl Trackable for MyStruct {
///     fn get_trackable_raw_ptr(&self) -> Option<usize> {
///         // Let's say we want to track the allocation of `data`.
///         // If `data` could be empty and not allocated, add logic for None.
///         Some(self.data.as_ptr() as usize)
///     }
///
///     // get_type_name() can use the default implementation.
/// }
///
/// // If MyStruct itself was heap-allocated (e.g. Box<MyStruct>),
/// // the Trackable impl for Box would handle getting the pointer to MyStruct.
/// ```
///
/// **Note on Collections:** For complex collections like `HashMap` or `HashSet`,
/// `track_var!` applied to the collection itself will associate the name with the
/// main structure of the collection if it's heap-allocated (e.g., `Box<HashMap<...>>`).
/// The internal buffers or individual elements within these collections are typically
/// managed by the collection's own allocator logic. While the `TrackingAllocator` (if enabled)
/// will see these internal allocations, `track_var!` on the collection doesn't individually
/// name those internal buffers.
pub trait Trackable {
    /// Returns a raw pointer to the primary heap-allocated data segment of this instance.
    ///
    /// This pointer is used by `track_var!` to link a variable name to its allocation record.
    /// Returns `None` if the object is not heap-allocated in a way that `track_var!`
    /// can directly associate (e.g., an empty `Vec` or `String`).
    fn get_trackable_raw_ptr(&self) -> Option<usize>;
    
    /// Returns the type name of this instance as a string.
    ///
    /// The default implementation uses `std::any::type_name::<Self>()`.
    /// This can be overridden if a custom type name string is desired for tracking.
    fn get_type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

// Implement Trackable for common types
impl<T> Trackable for Box<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        // Returns the pointer to the data T on the heap.
        Some(Box::as_ref(self) as *const T as usize)
    }
}

impl<T> Trackable for Vec<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.is_empty() { 
            // An empty Vec might not have allocated, or its pointer might be a sentinel.
            // Returning None prevents trying to track a non-existent/invalid allocation.
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
        // Rc::as_ptr returns a pointer to the contained data T.
        // This data (along with the reference counts) is on the heap.
        Some(Rc::as_ptr(self) as usize)
    }
}

// Implement Trackable for Arc<T>
impl<T> Trackable for Arc<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        // Arc::as_ptr returns a pointer to the contained data T.
        Some(Arc::as_ptr(self) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*; 
    use crate::tracker::get_global_tracker;
    // AllocationInfo is re-exported at crate root (super::AllocationInfo or just AllocationInfo)

    // Helper to find an allocation by pointer in active allocations
    fn find_active_allocation_info(ptr: usize) -> Option<AllocationInfo> {
        get_global_tracker().get_active_allocations().into_iter().find(|a| a.ptr == ptr)
    }
    
    // Helper to simulate clearing relevant parts of GLOBAL_TRACKER for test isolation.
    // This is a simplified approach. Full isolation with lazy_static is hard.
    fn clear_tracker_state() {
        let tracker = get_global_tracker();
        // These accesses would be to private fields. We cannot do this.
        // tracker.active_allocations.lock().unwrap().clear();
        // tracker.allocation_log.lock().unwrap().clear();
        // Instead, tests will deallocate items they add to try and clean up.
        // Perfect isolation for these macro tests on GLOBAL_TRACKER is not achieved here.
    }

    #[test]
    fn test_track_var_macro_vec() {
        init(); 
        clear_tracker_state(); // Attempt to reset before test
        
        let my_vec = vec![1, 2, 3];
        // get_trackable_raw_ptr() will return Some for non-empty Vec
        let vec_ptr = my_vec.get_trackable_raw_ptr().expect("Vec should have a pointer");

        // Simulate allocation being tracked by the global allocator
        get_global_tracker().track_allocation(vec_ptr, std::mem::size_of_val(my_vec.as_slice()), Some(my_vec.get_type_name())).unwrap();
        track_var!(my_vec);

        let info = find_active_allocation_info(vec_ptr)
            .expect("Vec allocation info not found after track_var!");

        assert_eq!(info.var_name.as_deref(), Some("my_vec"));
        assert_eq!(info.type_name.as_deref(), Some(std::any::type_name::<Vec<i32>>()));
        
        // Cleanup from tracker
        let _ = get_global_tracker().track_deallocation(vec_ptr);
    }

    #[test]
    fn test_track_var_macro_string() {
        init();
        clear_tracker_state();
        let my_string = String::from("hello test");
        if let Some(string_ptr) = my_string.get_trackable_raw_ptr() {
            get_global_tracker().track_allocation(string_ptr, my_string.capacity(), Some(my_string.get_type_name())).unwrap();
            track_var!(my_string);
            let info = find_active_allocation_info(string_ptr).expect("String info not found");
            assert_eq!(info.var_name.as_deref(), Some("my_string"));
            assert_eq!(info.type_name.as_deref(), Some(std::any::type_name::<String>()));
            let _ = get_global_tracker().track_deallocation(string_ptr);
        } else {
            // For empty string, track_var should do nothing problematic
            track_var!(my_string);
            // No pointer, so nothing to find or assert in tracker for empty string via track_var
        }
    }
    
    #[test]
    fn test_track_var_macro_empty_string_and_vec() {
        init();
        clear_tracker_state();

        let empty_string = String::new();
        assert!(empty_string.get_trackable_raw_ptr().is_none(), "Empty string should yield no pointer");
        // track_var! on empty string should not panic and __internal_track_var should receive None.
        track_var!(empty_string); 
        // No allocation to check in tracker, just ensure it runs.

        let empty_vec: Vec<u8> = Vec::new();
        assert!(empty_vec.get_trackable_raw_ptr().is_none(), "Empty vec should yield no pointer");
        track_var!(empty_vec);
        // No allocation to check.
    }


    #[test]
    fn test_track_var_macro_box() {
        init();
        clear_tracker_state();
        // Content must be 'static for std::any::type_name::<Box<&'static str>> to be consistent
        let my_box = Box::new("test content static"); 
        let box_ptr = my_box.get_trackable_raw_ptr().expect("Box should have a pointer");
        
        // For Box<T>, the size tracked by allocator would be size_of<T>.
        // For T = &'static str, size is size_of::<&str>()
        get_global_tracker().track_allocation(box_ptr, std::mem::size_of::<&'static str>(), Some(my_box.get_type_name())).unwrap();
        track_var!(my_box);
        
        let info = find_active_allocation_info(box_ptr).expect("Box info not found");
        assert_eq!(info.var_name.as_deref(), Some("my_box"));
        assert_eq!(info.type_name.as_deref(), Some(std::any::type_name::<Box<&'static str>>()));
        let _ = get_global_tracker().track_deallocation(box_ptr);
    }

    #[test]
    fn test_track_var_macro_rc_arc() {
        init();
        clear_tracker_state();
        
        // Test Rc
        let my_rc = Rc::new(true);
        let rc_ptr = my_rc.get_trackable_raw_ptr().expect("Rc should have a pointer");
        get_global_tracker().track_allocation(rc_ptr, std::mem::size_of_val(&*my_rc), Some(my_rc.get_type_name())).unwrap();
        track_var!(my_rc);
        let rc_info = find_active_allocation_info(rc_ptr).expect("Rc info not found");
        assert_eq!(rc_info.var_name.as_deref(), Some("my_rc"));
        assert_eq!(rc_info.type_name.as_deref(), Some(std::any::type_name::<Rc<bool>>()));
        let _ = get_global_tracker().track_deallocation(rc_ptr);
        
        clear_tracker_state(); // Clear for Arc test
        // Test Arc
        let my_arc = Arc::new(123.45f64);
        let arc_ptr = my_arc.get_trackable_raw_ptr().expect("Arc should have a pointer");
        get_global_tracker().track_allocation(arc_ptr, std::mem::size_of_val(&*my_arc), Some(my_arc.get_type_name())).unwrap();
        track_var!(my_arc);
        let arc_info = find_active_allocation_info(arc_ptr).expect("Arc info not found");
        assert_eq!(arc_info.var_name.as_deref(), Some("my_arc"));
        assert_eq!(arc_info.type_name.as_deref(), Some(std::any::type_name::<Arc<f64>>()));
        let _ = get_global_tracker().track_deallocation(arc_ptr);
    }
}
