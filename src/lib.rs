//! Memory tracking and visualization tools for Rust applications.
//!
//! This crate provides tools for tracking memory allocations and visualizing
//! memory usage in Rust applications. It includes a custom global allocator
//! that tracks all heap allocations and deallocations, and provides utilities
//! for exporting memory usage data in various formats.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#[allow(warnings)]
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
pub mod tracker;
pub mod types;

// Re-export common types for easier use
pub use tracker::MemoryError;
pub use tracker::{get_global_tracker, MemoryTracker};
pub use types::AllocationInfo;

use std::cell::Cell;

thread_local! {
    static IN_TRACING:Cell<bool> = Cell::new(false);
}

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

/// Initialize the memory tracking system.
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
    #[cfg(feature = "enable-subscriber")]
    {
        // Ensure this part is inside the cfg block.
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
    // If the feature is not enabled, init() does nothing.
}

/// Associates a variable name and its type with its heap allocation via the [`MemoryTracker`].
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
        let _ = $crate::__internal_track_var(stringify!($var), &$var);
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
// local storage for tracking recursion depth
thread_local! {
    static RECURSION_DEPTH: std::cell::Cell<u32> = std::cell::Cell::new(0);
    static MAX_RECURSION_DEPTH: std::cell::Cell<u32> = std::cell::Cell::new(10);
}

#[doc(hidden)]
pub fn __internal_track_var<T: Trackable>(name: &str, value: &T) -> Result<(), MemoryError> {
    // check recursion depth
    let (recursion_depth, recursion_error) = RECURSION_DEPTH.with(|d| {
        let current = d.get();
        let max = MAX_RECURSION_DEPTH.with(|m| m.get());
        if current >= max {
            (
                current,
                Some(MemoryError::TrackingError(format!(
                    "Maximum recursion depth ({}) reached in track_var for: {}",
                    max, name
                ))),
            )
        } else {
            let new_depth = current + 1;
            d.set(new_depth);
            (new_depth, None)
        }
    });

    // if recursion depth exceeds max, return error
    if let Some(err) = recursion_error {
        tracing::error!("{}", err);
        return Err(err);
    }

    // ensure recursion depth is reduced when function returns
    let _guard = scopeguard::guard((), |_| {
        RECURSION_DEPTH.with(|d| d.set(recursion_depth - 1));
    });

    tracing::debug!(
        "[Depth: {}] Starting track_var for: {}, type: {}",
        recursion_depth,
        name,
        std::any::type_name::<T>()
    );

    let mut tracing = false;
    IN_TRACING.with(|flag| {
        tracing = flag.get();
    });
    if tracing {
        tracing::debug!(
            "[Depth: {}] Already in tracing context, skipping track_var for: {}",
            recursion_depth,
            name
        );
        return Ok(());
    }

    // set tracing flag to prevent reentrancy
    IN_TRACING.with(|flag| {
        flag.set(true);
    });

    // ensure tracing flag is reset when function returns
    let _tracing_guard = scopeguard::guard((), |_| {
        IN_TRACING.with(|flag| flag.set(false));
    });

    let result = if let Some(ptr) = value.get_trackable_raw_ptr() {
        tracing::debug!(
            "[Depth: {}] Got pointer for {}: 0x{:x}",
            recursion_depth,
            name,
            ptr
        );

        let tracker = get_global_tracker();
        let type_name = value.get_type_name();
        tracing::debug!(
            "[Depth: {}] Got type name for {}: {}",
            recursion_depth,
            name,
            type_name
        );

        // use with_allocations_disabled to prevent allocations during tracking
        let result = {
            let _span = tracing::debug_span!("with_allocations_disabled").entered();
            tracker.with_allocations_disabled(|| {
                let _span = tracing::debug_span!("associate_var").entered();
                let res = tracker.associate_var(ptr, name.to_string(), type_name);
                tracing::debug!(
                    "[Depth: {}] associate_var result for {}: {:?}",
                    recursion_depth,
                    name,
                    res
                );
                res
            })
        };

        tracing::debug!(
            "[Depth: {}] Completed with_allocations_disabled for {}",
            recursion_depth,
            name
        );
        result
    } else {
        // If a type is not heap-allocated or empty (e.g. empty Vec/String), it might return None.
        tracing::debug!(
            "[Depth: {}] No pointer to track for {} (type: {})",
            recursion_depth,
            name,
            std::any::type_name::<T>()
        );
        Ok(())
    };

    tracing::debug!(
        "[Depth: {}] Completed track_var for: {}",
        recursion_depth,
        name
    );
    result
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
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}

impl Trackable for String {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.is_empty() {
            // Similar to Vec
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }
}

use std::rc::Rc;
use std::sync::Arc;

impl<T> Trackable for Rc<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        // Returns the pointer to the data T on the heap.
        Some(Rc::as_ptr(self) as usize)
    }
}

impl<T> Trackable for Arc<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        // Returns the pointer to the data T on the heap.
        Some(Arc::as_ptr(self) as usize)
    }
}
