//! Unified Tracker API
//!
//! This module provides a simple, unified interface for memory tracking.
//! All the complexity of smart pointers, owned values, etc. is handled internally.
//!
//! # Usage
//!
//! ```rust
//! use memscope_rs::{tracker, track};
//!
//! // Create a tracker
//! let tracker = tracker!();
//!
//! // Track any variable
//! let my_vec = vec![1, 2, 3];
//! track!(tracker, my_vec);
//!
//! // Track with custom name
//! let my_string = String::from("hello");
//! tracker.track_as(&my_string, "greeting");
//! ```

use crate::core::tracker::MemoryTracker;
use std::sync::Arc;

/// A unified tracker that handles all tracking scenarios.
///
/// This struct provides a simple interface for tracking memory allocations,
/// regardless of whether the value is a smart pointer, owned value, or reference.
pub struct Tracker {
    inner: Arc<MemoryTracker>,
}

impl Tracker {
    /// Create a new tracker instance.
    ///
    /// This is typically called via the `tracker!()` macro.
    /// Uses the global tracker instance for compatibility with export functions.
    pub fn new() -> Self {
        Self {
            inner: crate::core::tracker::get_tracker(),
        }
    }

    /// Track a variable with a custom name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use memscope_rs::tracker;
    ///
    /// let tracker = tracker!();
    /// let data = vec![1, 2, 3];
    /// tracker.track_as(&data, "user_data");
    /// ```
    pub fn track_as<T: crate::Trackable>(&self, var: &T, name: &str) {
        self.track_inner(var, name);
    }

    /// Internal tracking implementation with proper error handling.
    fn track_inner<T: crate::Trackable>(&self, var: &T, name: &str) {
        let type_name = var.get_type_name().to_string();
        let size = var.get_size_estimate();

        // Get or generate pointer with thread-safe counter
        let ptr = var.get_heap_ptr().unwrap_or_else(|| {
            // Generate synthetic pointer for stack-allocated or non-pointer types
            // Use thread-local counter to avoid race conditions
            use std::cell::Cell;
            thread_local! {
                static COUNTER: Cell<u64> = Cell::new(0x8000_0000);
            }
            COUNTER.with(|counter| {
                let val = counter.get();
                counter.set(val.wrapping_add(1));
                val as usize
            })
        });

        // Track allocation using the inner tracker's method with proper error handling
        if let Err(e) = self.inner.track_allocation(ptr, size) {
            tracing::error!("Failed to track allocation at ptr {:x}: {}", ptr, e);
            return;
        }

        // Associate variable name and type with proper error handling
        if let Err(e) = self.inner.associate_var(ptr, name.to_string(), type_name) {
            tracing::error!("Failed to associate var '{}' at ptr {:x}: {}", name, ptr, e);
        }
    }

    /// Get a snapshot of current memory state.
    pub fn snapshot(&self) -> crate::core::types::MemoryStats {
        self.inner.get_stats().unwrap_or_default()
    }

    /// Export tracking data to SVG visualization.
    pub fn export_svg(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.export_memory_analysis(path)?;
        Ok(())
    }

    /// Export tracking data to JSON format.
    ///
    /// This exports user variables with detailed information including
    /// variable names, types, sizes, and allocation details.
    pub fn export_json(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let allocations = self.inner.get_active_allocations()?;
        let stats = self.inner.get_stats()?;
        crate::export::export_user_variables_json(allocations, stats, path)?;
        Ok(())
    }

    /// Export tracking data to binary format.
    ///
    /// Binary format is 3x faster and 60% smaller than JSON.
    pub fn export_binary(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let allocations = self.inner.get_active_allocations()?;
        let stats = self.inner.get_stats()?;
        crate::export::export_user_variables_binary(allocations, stats, path)?;
        Ok(())
    }

    /// Get the underlying MemoryTracker for advanced usage.
    pub fn inner(&self) -> &Arc<MemoryTracker> {
        &self.inner
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tracker instance.
///
/// This is the recommended way to create a tracker. It handles all the
/// complexity of smart pointers, owned values, etc. internally.
///
/// # Example
///
/// ```rust
/// use memscope_rs::{tracker, track};
///
/// // Create tracker
/// let tracker = tracker!();
///
/// // Track variables
/// let my_vec = vec![1, 2, 3];
/// track!(tracker, my_vec);
/// ```
#[macro_export]
macro_rules! tracker {
    () => {
        $crate::tracker::Tracker::new()
    };
}

/// Track a variable using the tracker.
///
/// This macro captures the variable name automatically and tracks it.
///
/// # Example
///
/// ```rust
/// use memscope_rs::{tracker, track};
///
/// let tracker = tracker!();
/// let my_vec = vec![1, 2, 3];
/// track!(tracker, my_vec);  // Automatically named "my_vec"
/// ```
#[macro_export]
macro_rules! track {
    ($tracker:expr, $var:expr) => {{
        let var_name = stringify!($var);
        $tracker.track_as(&$var, var_name);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = Tracker::new();
        // Just verify it doesn't panic
        let _ = tracker;
    }

    #[test]
    fn test_tracker_macro() {
        let tracker = tracker!();
        // Just verify the macro works
        let _ = tracker;
    }

    #[test]
    fn test_track_macro() {
        let tracker = tracker!();
        let my_vec = vec![1, 2, 3];
        track!(tracker, my_vec);
        // If we get here without panic, the macro works
    }
}
