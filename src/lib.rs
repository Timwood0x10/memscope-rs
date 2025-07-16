//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

#![warn(missing_docs)]
#![allow(dead_code, unused_variables, unused_imports)]

pub mod allocator;
pub mod export_enhanced;
pub mod tracker;
pub mod types;
pub mod unsafe_ffi_tracker;
pub mod unsafe_ffi_visualization;
pub mod utils;
pub mod visualization;
/// Offline HTML report generation from JSON data
pub mod report_generator;

// Re-export key functions
pub use report_generator::*;

// Re-export main types for easier use
pub use allocator::TrackingAllocator;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::{AllocationInfo, TrackingError, TrackingResult};
pub use unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, UnsafeFFITracker};
pub use utils::{format_bytes, get_simple_type, simplify_type_name};
pub use visualization::{export_lifecycle_timeline, export_memory_analysis};

// Set up the global allocator when the tracking-allocator feature is enabled
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
/// Global tracking allocator instance used when the tracking-allocator feature is enabled.
pub static GLOBAL: TrackingAllocator = TrackingAllocator::new();

/// Trait for types that can be tracked by the memory tracker.
pub trait Trackable {
    /// Get the pointer to the heap allocation for this value.
    fn get_heap_ptr(&self) -> Option<usize>;

    /// Get the type name for this value.
    fn get_type_name(&self) -> &'static str;
}

// Implement Trackable for common heap-allocated types
impl<T> Trackable for Vec<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self.as_ptr() as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Vec<T>>()
    }
}

impl Trackable for String {
    fn get_heap_ptr(&self) -> Option<usize> {
        if self.capacity() > 0 {
            Some(self.as_ptr() as usize)
        } else {
            None
        }
    }

    fn get_type_name(&self) -> &'static str {
        "String"
    }
}

impl<T> Trackable for Box<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        Some(self.as_ref() as *const T as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<Box<T>>()
    }
}

impl<T> Trackable for std::rc::Rc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // For Rc, the allocation tracking is complex because Rc uses a control block
        // We'll track the Rc itself rather than the inner data to avoid pointer issues
        Some(std::rc::Rc::as_ptr(self) as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::rc::Rc<T>>()
    }
}

impl<T> Trackable for std::sync::Arc<T> {
    fn get_heap_ptr(&self) -> Option<usize> {
        // For Arc, the allocation tracking is complex because Arc uses a control block
        // We'll track the Arc itself rather than the inner data to avoid pointer issues
        Some(std::sync::Arc::as_ptr(self) as usize)
    }

    fn get_type_name(&self) -> &'static str {
        std::any::type_name::<std::sync::Arc<T>>()
    }
}

/// Macro to track a variable's memory allocation.
///
/// This macro associates a variable name with its heap allocation,
/// allowing the memory tracker to provide meaningful names in reports.
///
/// # Example
/// ```rust
/// use memscope_rs::track_var;
///
/// let my_vec = vec![1, 2, 3, 4, 5];
/// track_var!(my_vec);
/// ```
#[macro_export]
macro_rules! track_var {
    ($var:ident) => {
        $crate::_track_var_impl(&$var, stringify!($var))
    };
}

/// Internal implementation function for the track_var! macro.
/// This function should not be called directly.
#[doc(hidden)]
pub fn _track_var_impl<T: Trackable>(var: &T, var_name: &str) -> TrackingResult<()> {
    if let Some(ptr) = var.get_heap_ptr() {
        let tracker = get_global_tracker();
        let type_name = var.get_type_name().to_string();

        // Debug: Print tracking attempt
        // tracing::debug!(
        //     "Tracking variable '{}' of type '{}' at ptr {:x}",
        //     var_name,
        //     type_name,
        //     ptr
        // );

        tracker.associate_var(ptr, var_name.to_string(), type_name)
    } else {
        // Variable doesn't have a heap allocation (e.g., empty Vec)
        // tracing::debug!("Variable '{}' has no heap allocation to track", var_name);
        Ok(())
    }
}

/// Initialize the memory tracking system.
///
/// This function sets up the tracing subscriber and prepares the global tracker.
/// Call this early in your application, typically in main().
///
/// # Example
/// ```rust
/// memscope_rs::init();
/// // Your application code here
/// ```
pub fn init() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "memscope_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("memscope-rs initialized");
}
