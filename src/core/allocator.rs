//! Custom global allocator for tracking memory allocations.

use crate::core::enhanced_type_inference::{AllocationContext, TypeInferenceEngine};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;

/// A custom allocator that tracks memory allocations and deallocations.
///
/// This allocator wraps the system allocator and records all allocation
/// and deallocation events through the global memory tracker.
pub struct TrackingAllocator;

// Global type inference engine for the allocator
static TYPE_INFERENCE_ENGINE: std::sync::LazyLock<Mutex<TypeInferenceEngine>> =
    std::sync::LazyLock::new(|| Mutex::new(TypeInferenceEngine::new()));

impl TrackingAllocator {
    /// Create a new tracking allocator instance.
    pub const fn new() -> Self {
        Self
    }

    /// Enhanced type inference using the new type inference engine
    fn infer_type_from_allocation_context(size: usize) -> String {
        // Try to use the enhanced type inference engine
        if let Ok(mut engine) = TYPE_INFERENCE_ENGINE.try_lock() {
            let context = AllocationContext {
                size,
                call_stack: Self::get_simplified_call_stack(),
                compile_time_type: None,
                variable_name: None,
                allocation_site: Some("global_allocator".to_string()),
                thread_context: Some(format!("{:?}", std::thread::current().id())),
            };

            let inferred = engine.infer_type(&context);
            inferred.type_name
        } else {
            // Fallback to simple size-based inference if engine is busy
            Self::fallback_type_inference(size)
        }
    }

    /// Fallback type inference when the engine is unavailable
    fn fallback_type_inference(size: usize) -> String {
        match size {
            // Common Rust type sizes
            1 => "u8".to_string(),
            2 => "u16".to_string(),
            4 => "u32".to_string(),
            8 => "u64".to_string(),
            16 => "u128".to_string(),

            // String and Vec common sizes
            24 => "alloc::string::String".to_string(),
            32 => "alloc::vec::Vec<T>".to_string(),
            48 => "std::collections::HashMap<K,V>".to_string(),

            // Smart pointer sizes
            size if size == std::mem::size_of::<std::sync::Arc<String>>() => {
                "alloc::sync::Arc<T>".to_string()
            }
            size if size == std::mem::size_of::<std::rc::Rc<String>>() => {
                "alloc::rc::Rc<T>".to_string()
            }
            size if size == std::mem::size_of::<Box<String>>() => {
                "alloc::boxed::Box<T>".to_string()
            }

            // Buffer sizes - likely Vec or String data
            size if size > 1024 => "alloc::vec::Vec<u8>".to_string(),
            size if size > 256 => "alloc::collections::BTreeMap<K,V>".to_string(),
            size if size > 64 => "std::collections::HashMap<K,V>".to_string(),

            // Default for other sizes
            _ => format!("system_type_{}bytes", size),
        }
    }

    /// Get a simplified call stack for context
    fn get_simplified_call_stack() -> Vec<String> {
        // For now, return a simple placeholder
        // In a real implementation, this could use backtrace crate
        vec!["global_allocator".to_string(), "system_alloc".to_string()]
    }

    /// Infer likely variable name from allocation size and context
    /// This provides meaningful variable names for system allocations
    fn infer_variable_from_allocation_context(size: usize) -> String {
        match size {
            // Small allocations - likely primitives
            1..=8 => "primitive_data".to_string(),

            // Medium allocations - likely structs or small collections
            9..=64 => "struct_data".to_string(),

            // Large allocations - likely collections or buffers
            65..=1024 => "collection_data".to_string(),

            // Very large allocations - likely buffers or large data structures
            _ => "buffer_data".to_string(),
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

                // Track the allocation with enhanced context - use try_lock approach to avoid deadlocks
                if let Ok(tracker) =
                    std::panic::catch_unwind(crate::core::tracker::get_global_tracker)
                {
                    // Enhanced tracking: try to infer type and context from allocation size and stack
                    let inferred_type = Self::infer_type_from_allocation_context(layout.size());
                    let inferred_var = Self::infer_variable_from_allocation_context(layout.size());

                    // Ignore errors to prevent allocation failures from breaking the program
                    let _ = tracker.track_allocation_with_context(
                        ptr as usize,
                        layout.size(),
                        inferred_var,
                        inferred_type,
                    );
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
