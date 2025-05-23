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
pub mod procmacros;
pub mod tracker;
pub mod types;

// Re-export common types for easier use
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::AllocationInfo;
pub use tracker::MemoryError;

// Import the allocator
#[cfg(feature = "tracking-allocator")]
pub use allocator::TrackingAllocator;

/// Global allocator for tracking memory allocations
#[cfg(feature = "tracking-allocator")]
#[global_allocator]
pub static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Initialize the memory tracking system
///
/// This function sets up the tracing subscriber with default configuration.
/// It should be called early in your application's main function.
///
/// # Example
/// ```no_run
/// use trace_tools::init;
///
/// fn main() {
///     // Initialize the memory tracking system
///     init();
///     
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

/// Track a variable's memory allocation
///
/// This macro associates a variable name with its memory allocation
/// for better tracking and visualization.
///
/// # Example
/// ```no_run
/// use trace_tools::track_var;
///
/// let my_vec = vec![1, 2, 3];
/// track_var!(my_vec);
/// ```
#[macro_export]
macro_rules! track_var {
    ($var:ident) => {
        #[allow(unused_imports)]
        use $crate::Trackable;
        let _ = $crate::__internal_track_var(stringify!($var), &$var);
    };
}

/// Internal function used by the `track_var!` macro
#[doc(hidden)]
pub fn __internal_track_var<T: Trackable>(name: &str, value: &T) -> std::io::Result<()> {
    if let Some(ptr) = value.get_trackable_raw_ptr() {
        let tracker = get_global_tracker();
        let type_name = std::any::type_name::<T>().to_string();
        tracker.associate_var(ptr, name.to_string(), type_name)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    } else {
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
        if self.is_empty() {
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}

impl Trackable for String {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}
