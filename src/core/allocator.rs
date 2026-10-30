//! Custom global allocator for tracking memory allocations.

use std::alloc::{GlobalAlloc, Layout, System};

/// A custom allocator that tracks memory allocations and deallocations.
///
/// This allocator wraps the system allocator and records all allocation
/// and deallocation events through the global memory tracker.
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Create a new tracking allocator instance.
    pub const fn new() -> Self {
        Self
    }

    // Removed: `_infer_type_from_allocation_context` and friends mapped a request
    // size to "String"/"Vec<T>" style names. A byte count cannot identify a type,
    // so those helpers were dead weight that invited false conclusions. The
    // allocator now records only what it actually observes: address and size.
}

// Thread-local flag to prevent recursive tracking
thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Scope guard that suppresses tracking while the allocator works.
///
/// The previous code toggled the thread-local flag manually, so any early return (or a
/// panic inside the tracker) left tracking disabled for the rest of the thread's life.
/// Restoring on `Drop` makes the suppression impossible to leak.
struct TrackingGuard;

impl TrackingGuard {
    /// Take the suppression slot, or `None` when tracking is already suppressed
    /// (recursive allocation from inside the tracker itself).
    fn acquire() -> Option<Self> {
        TRACKING_DISABLED.with(|disabled| {
            if disabled.get() {
                None
            } else {
                disabled.set(true);
                Some(Self)
            }
        })
    }
}

impl Drop for TrackingGuard {
    fn drop(&mut self) {
        TRACKING_DISABLED.with(|disabled| disabled.set(false));
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Allocate memory first
        let ptr = System.alloc(layout);

        // Track the allocation if it succeeded and tracking is not disabled
        if !ptr.is_null() {
            if let Some(_suppression) = TrackingGuard::acquire() {
                // `catch_unwind` keeps a tracker failure from breaking user allocation.
                if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                    let _ = tracker.track_allocation(ptr as usize, layout.size());
                }
            }
        }

        ptr
    }

    /// Reallocate through the system allocator and record the transition once.
    ///
    /// The default `GlobalAlloc::realloc` implementation is alloc + copy + dealloc,
    /// which turns one logical operation into two unrelated events and loses the
    /// fact that the block kept its identity. Recording a single `Reallocate` event
    /// with both addresses keeps the ledger consistent for moved blocks.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Some(_suppression) = TrackingGuard::acquire() else {
            return System.realloc(ptr, layout, new_size);
        };

        let new_ptr = System.realloc(ptr, layout, new_size);
        if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
            if !new_ptr.is_null() {
                // The allocator path has no event store attached yet, so only the
                // in-memory ledger is updated here (the `Tracker` facade records the
                // matching `Reallocate` event when it owns one).
                let _ = tracker.track_reallocation(
                    ptr as usize,
                    new_ptr as usize,
                    layout.size(),
                    new_size,
                );
            }
            // A failed reallocation keeps the original block alive: nothing to record.
        }

        new_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Track the deallocation first, then hand the block back to the system.
        if let Some(_suppression) = TrackingGuard::acquire() {
            if let Ok(tracker) = std::panic::catch_unwind(crate::core::tracker::get_tracker) {
                // Ignore errors so a tracking failure cannot break the deallocation.
                let _ = tracker.track_deallocation(ptr as usize);
            }
        }

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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Once;

    // Helper to reset thread-local state between tests
    fn reset_thread_local_state() {
        TRACKING_DISABLED.with(|disabled| disabled.set(false));
    }

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

    /// Objective: Verify that zero-sized requests never reach the allocator.
    /// Invariants: `GlobalAlloc` requires `layout.size() > 0` (the standard
    /// library aborts otherwise), so ZST containers must use the dangling
    /// sentinel pointer and produce no tracked allocation at all.
    #[test]
    fn test_zero_sized_allocation() {
        let layout = Layout::from_size_align(0, 1).expect("zero-size layout must be constructible");
        assert_eq!(
            layout.size(),
            0,
            "the zero-size layout must report a zero length, otherwise the test is meaningless"
        );

        // ZST vectors share the non-null dangling sentinel: `with_capacity` on a
        // zero-sized element never asks the global allocator for memory.
        let with_capacity: Vec<()> = Vec::with_capacity(1024);
        let empty: Vec<()> = Vec::new();
        assert_eq!(
            with_capacity.as_ptr(),
            empty.as_ptr(),
            "a ZST vector must reuse the dangling sentinel instead of allocating"
        );
        assert!(
            with_capacity.capacity() >= 1024,
            "a ZST vector must satisfy its requested capacity without allocating, got {}",
            with_capacity.capacity()
        );
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
    fn test_default_implementation() {
        let allocator = TrackingAllocator::new();
        assert_eq!(
            std::mem::size_of_val(&allocator),
            std::mem::size_of::<TrackingAllocator>()
        );
    }

    #[test]
    fn test_thread_local_tracking() {
        reset_thread_local_state();

        // Test that tracking is enabled by default
        TRACKING_DISABLED.with(|disabled| {
            assert!(!disabled.get());
        });

        // Test disabling tracking
        TRACKING_DISABLED.with(|disabled| {
            disabled.set(true);
            assert!(disabled.get());
            disabled.set(false);
        });
    }

    #[test]
    fn test_allocation_edge_cases() {
        let allocator = TrackingAllocator::new();

        // Test with maximum alignment
        let max_align = std::mem::size_of::<usize>() * 2;
        let layout = Layout::from_size_align(16, max_align).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                // Test that the pointer is properly aligned
                assert_eq!((ptr as usize) % max_align, 0);
                allocator.dealloc(ptr, layout);
            }
        }

        // Test with minimal size but non-zero
        let layout = Layout::from_size_align(1, 1).unwrap();
        unsafe {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                allocator.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_recursive_allocation_handling() {
        // This test verifies that recursive allocations don't cause infinite loops
        let allocator = TrackingAllocator::new();
        let layout = Layout::from_size_align(64, 8).unwrap();

        // Set up a flag to detect if we're in a recursive call
        static RECURSION_DETECTED: AtomicBool = AtomicBool::new(false);
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            // Install a panic hook to detect if we hit a stack overflow
            let original_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                    if s.contains("stack overflow") {
                        RECURSION_DETECTED.store(true, Ordering::SeqCst);
                    }
                }
                original_hook(panic_info);
            }));
        });

        // This allocation will trigger tracking, but the thread-local flag should prevent recursion
        unsafe {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                allocator.dealloc(ptr, layout);
            }
        }

        // Verify we didn't hit a stack overflow
        assert!(
            !RECURSION_DETECTED.load(Ordering::SeqCst),
            "Recursive allocation detected - thread-local tracking failed"
        );
    }

    /// Objective: Verify the suppression guard restores tracking on drop, including
    /// when a nested acquisition is refused.
    /// Invariants: exactly one guard owns the slot; the nested one does not restore
    /// early; dropping the owner leaves tracking enabled.
    #[test]
    fn suppression_guard_restores_state() {
        reset_thread_local_state();

        {
            let outer = TrackingGuard::acquire();
            assert!(outer.is_some(), "the first guard must take the slot");
            assert!(
                TRACKING_DISABLED.with(|disabled| disabled.get()),
                "tracking must be suppressed while the guard is alive"
            );

            {
                let inner = TrackingGuard::acquire();
                assert!(
                    inner.is_none(),
                    "a nested acquisition must be refused instead of double-owning the slot"
                );
            }

            assert!(
                TRACKING_DISABLED.with(|disabled| disabled.get()),
                "the refused nested guard must not restore tracking early"
            );
        }

        assert!(
            !TRACKING_DISABLED.with(|disabled| disabled.get()),
            "dropping the owning guard must restore tracking"
        );
    }
}
