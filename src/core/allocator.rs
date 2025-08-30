//! Custom global allocator for tracking memory allocations.

use crate::core::enhanced_type_inference::TypeInferenceEngine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;

/// A custom allocator that tracks memory allocations and deallocations.
///
/// This allocator wraps the system allocator and records all allocation
/// and deallocation events through the global memory tracker.
pub struct TrackingAllocator;

// Global type inference engine for the allocator
static _TYPE_INFERENCE_ENGINE: std::sync::LazyLock<Mutex<TypeInferenceEngine>> =
    std::sync::LazyLock::new(|| Mutex::new(TypeInferenceEngine::new()));

impl TrackingAllocator {
    /// Create a new tracking allocator instance.
    pub const fn new() -> Self {
        Self
    }

    /// Simple type inference using static strings to avoid recursive allocations
    fn _infer_type_from_allocation_context(size: usize) -> &'static str {
        // CRITICAL FIX: Use static strings to prevent recursive allocations
        match size {
            // Common Rust type sizes
            1 => "u8",
            2 => "u16",
            4 => "u32",
            8 => "u64",
            16 => "u128",

            // String and Vec common sizes
            24 => "String",
            32 => "Vec<T>",
            48 => "HashMap<K,V>",

            // Smart pointer sizes
            size if size == std::mem::size_of::<std::sync::Arc<String>>() => "Arc<T>",
            size if size == std::mem::size_of::<std::rc::Rc<String>>() => "Rc<T>",
            size if size == std::mem::size_of::<Box<String>>() => "Box<T>",

            // Default for other sizes - use static strings
            _ => "unknown",
        }
    }

    // REMOVED: fallback_type_inference - no longer needed with static strings

    /// Get a simplified call stack for context
    fn _get_simplified_call_stack() -> Vec<String> {
        // For now, return a simple placeholder
        // In a real implementation, this could use backtrace crate
        vec!["global_allocator".to_string(), "system_alloc".to_string()]
    }

    /// Simple variable name inference using static strings to avoid recursive allocations
    fn _infer_variable_from_allocation_context(size: usize) -> &'static str {
        // CRITICAL FIX: Use static strings to prevent recursive allocations
        match size {
            // Small allocations - likely primitives
            1..=8 => "primitive_data",

            // Medium allocations - likely structs or small collections
            9..=64 => "struct_data",

            // Large allocations - likely collections or buffers
            65..=1024 => "collection_data",

            // Very large allocations - likely buffers or large data structures
            _ => "buffer_data",
        }
    }
}

// Thread-local flag to prevent recursive tracking
thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Allocate memory first
        let ptr = System.alloc(layout);

        // Track the allocation if it succeeded and tracking is not disabled
        if !ptr.is_null() {
            // Check if tracking is disabled for this thread to prevent recursion
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

            if should_track {
                // Temporarily disable tracking to prevent recursion during tracking operations
                TRACKING_DISABLED.with(|disabled| disabled.set(true));

                // CRITICAL FIX: Use simple tracking like master branch to avoid recursion
                if let Ok(tracker) =
                    std::panic::catch_unwind(crate::core::tracker::get_global_tracker)
                {
                    // Simple tracking without context to prevent recursive allocations
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }

                // Re-enable tracking
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Track the deallocation first
        let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

        if should_track {
            // Temporarily disable tracking to prevent recursion
            TRACKING_DISABLED.with(|disabled| disabled.set(true));

            // Track the deallocation - use try_lock approach to avoid deadlocks
            if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_global_tracker)
            {
                // Ignore errors to prevent deallocation failures from breaking the program
                let _ = tracker.track_deallocation(ptr as usize);
            }

            // Re-enable tracking
            TRACKING_DISABLED.with(|disabled| disabled.set(false));
        }

        // Deallocate the memory
        System.dealloc(ptr, layout);
    }
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout};

    #[test]
    fn test_allocation_tracking() {
        let allocator = TrackingAllocator::new();
        let layout = Layout::from_size_align(1024, 8).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());

            // Test deallocation
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_zero_sized_allocation() {
        let allocator = TrackingAllocator::new();
        let layout = Layout::from_size_align(0, 1).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            // Zero-sized allocations may return null or a valid pointer
            // Both are acceptable according to the GlobalAlloc trait
            allocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_large_allocation() {
        let allocator = TrackingAllocator::new();
        let layout = Layout::from_size_align(1024 * 1024, 8).unwrap(); // 1MB

        unsafe {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                // Only test deallocation if allocation succeeded
                allocator.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_multiple_allocations() {
        let allocator = TrackingAllocator::new();
        let mut ptrs = Vec::new();

        // Allocate multiple blocks
        for i in 1..=10 {
            let layout = Layout::from_size_align(i * 64, 8).unwrap();
            unsafe {
                let ptr = allocator.alloc(layout);
                if !ptr.is_null() {
                    ptrs.push((ptr, layout));
                }
            }
        }

        // Deallocate all blocks
        for (ptr, layout) in ptrs {
            unsafe {
                allocator.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_type_inference_from_size() {
        // Test the static type inference
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(1),
            "u8"
        );
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(4),
            "u32"
        );
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(8),
            "u64"
        );
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(24),
            "String"
        );
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(32),
            "Vec<T>"
        );
        assert_eq!(
            TrackingAllocator::_infer_type_from_allocation_context(999),
            "unknown"
        );
    }

    #[test]
    fn test_variable_inference_from_size() {
        // Test the static variable inference
        assert_eq!(
            TrackingAllocator::_infer_variable_from_allocation_context(4),
            "primitive_data"
        );
        assert_eq!(
            TrackingAllocator::_infer_variable_from_allocation_context(32),
            "struct_data"
        );
        assert_eq!(
            TrackingAllocator::_infer_variable_from_allocation_context(512),
            "collection_data"
        );
        assert_eq!(
            TrackingAllocator::_infer_variable_from_allocation_context(2048),
            "buffer_data"
        );
    }

    #[test]
    fn test_simplified_call_stack() {
        let stack = TrackingAllocator::_get_simplified_call_stack();
        assert!(!stack.is_empty());
        assert!(stack.contains(&"global_allocator".to_string()));
    }

    #[test]
    fn test_default_implementation() {
        let allocator = TrackingAllocator;
        // Test that default works
        let layout = Layout::from_size_align(64, 8).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                allocator.dealloc(ptr, layout);
            }
        }
    }
}
