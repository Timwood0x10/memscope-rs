//! HeapPtr - Trait for automatic HeapOwner identification
//!
//! This module defines the `HeapPtr` trait which is implemented by types
//! that can provide stable heap pointers. This trait is used to identify
//! HeapOwner types in the three-layer object model.
//!
//! Note: This trait is defined for future extensibility but is not currently
//! used for automatic Trackable implementation due to specialization being
//! unavailable in stable Rust. Instead, Trackable is implemented manually
//! for each HeapOwner type.

/// Types that can provide stable heap pointer
///
/// This trait is implemented by types that have a stable, accessible heap pointer
/// and known size. Implementing this trait indicates that the type is a HeapOwner
/// in the three-layer object model.
///
/// # Safety
///
/// Implementations must ensure that:
/// - The returned pointer is valid and points to heap memory
/// - The size accurately reflects the allocated heap size
/// - The pointer remains stable for the lifetime of the object
pub trait HeapPtr {
    /// Returns heap pointer and size
    ///
    /// # Returns
    ///
    /// A tuple of (pointer, size) where:
    /// - `pointer` is a raw pointer to the heap memory
    /// - `size` is the total allocated size in bytes
    fn heap_ptr(&self) -> (*const u8, usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_ptr_trait_exists() {
        // This test verifies that the HeapPtr trait is defined
        // Actual implementations are in src/lib.rs
        // We use a simple struct to test the trait definition
        struct TestStruct {
            ptr: *const u8,
            size: usize,
        }

        impl HeapPtr for TestStruct {
            fn heap_ptr(&self) -> (*const u8, usize) {
                (self.ptr, self.size)
            }
        }

        let test = TestStruct {
            ptr: 0x1000 as *const u8,
            size: 1024,
        };

        let (ptr, size) = test.heap_ptr();
        assert_eq!(ptr, 0x1000 as *const u8);
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_heap_ptr_send_sync() {
        // Verify that concrete types implementing HeapPtr are Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        // Test with a concrete type that implements HeapPtr
        assert_send_sync::<Vec<u8>>();
    }
}
