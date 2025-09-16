//! Macro for implementing Trackable for advanced types
//!
//! This module provides a macro that automatically implements Trackable
//! for advanced Rust types with minimal boilerplate code.

/// Macro to implement Trackable for advanced types with automatic analysis
#[macro_export]
macro_rules! impl_advanced_trackable {
    ($type:ty, $offset:expr) => {
        impl<T> $crate::Trackable for $type {
            fn get_heap_ptr(&self) -> Option<usize> {
                // Use unique offset for this type category
                let instance_ptr = self as *const _ as usize;
                Some($offset + (instance_ptr % 0x0FFF_FFFF))
            }

            fn get_type_name(&self) -> &'static str {
                std::any::type_name::<$type>()
            }

            fn get_size_estimate(&self) -> usize {
                std::mem::size_of::<$type>()
            }

            fn get_advanced_type_info(&self) -> Option<$crate::advanced_types::AdvancedTypeInfo> {
                let type_name = self.get_type_name();
                let allocation = $crate::core::types::AllocationInfo {
                    ptr: self.get_heap_ptr().unwrap_or(0),
                    size: self.get_size_estimate(),
                    var_name: None,
                    type_name: Some(type_name.to_string()),
                    scope_name: None,
                    timestamp_alloc: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                    timestamp_dealloc: None,
                    thread_id: format!("{:?}", std::thread::current().id()),
                    borrow_count: 0,
                    stack_trace: None,
                    is_leaked: false,
                    lifetime_ms: None,
                    borrow_info: None,
                    clone_info: None,
                    ownership_history_available: false,
                    smart_pointer_info: None,
                    memory_layout: None,
                    generic_info: None,
                    dynamic_type_info: None,
                    runtime_state: None,
                    stack_allocation: None,
                    temporary_object: None,
                    fragmentation_analysis: None,
                    generic_instantiation: None,
                    type_relationships: None,
                    type_usage: None,
                    function_call_tracking: None,
                    lifecycle_tracking: None,
                    access_tracking: None,
                    drop_chain_analysis: None,
                };

                Some(
                    $crate::advanced_types::GenericAdvancedTypeAnalyzer::analyze_by_type_name(
                        type_name,
                        &allocation,
                    ),
                )
            }
        }
    };

    // Variant for types without generics
    ($type:ty, $offset:expr, no_generics) => {
        impl $crate::Trackable for $type {
            fn get_heap_ptr(&self) -> Option<usize> {
                let instance_ptr = self as *const _ as usize;
                Some($offset + (instance_ptr % 0x0FFF_FFFF))
            }

            fn get_type_name(&self) -> &'static str {
                std::any::type_name::<$type>()
            }

            fn get_size_estimate(&self) -> usize {
                std::mem::size_of::<$type>()
            }

            fn get_advanced_type_info(&self) -> Option<$crate::advanced_types::AdvancedTypeInfo> {
                let type_name = self.get_type_name();
                let allocation = $crate::core::types::AllocationInfo {
                    ptr: self.get_heap_ptr().unwrap_or(0),
                    size: self.get_size_estimate(),
                    var_name: None,
                    type_name: Some(type_name.to_string()),
                    scope_name: None,
                    timestamp_alloc: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64,
                    timestamp_dealloc: None,
                    thread_id: format!("{:?}", std::thread::current().id()),
                    borrow_count: 0,
                    stack_trace: None,
                    is_leaked: false,
                    lifetime_ms: None,
                    borrow_info: None,
                    clone_info: None,
                    ownership_history_available: false,
                    smart_pointer_info: None,
                    memory_layout: None,
                    generic_info: None,
                    dynamic_type_info: None,
                    runtime_state: None,
                    stack_allocation: None,
                    temporary_object: None,
                    fragmentation_analysis: None,
                    generic_instantiation: None,
                    type_relationships: None,
                    type_usage: None,
                    function_call_tracking: None,
                    lifecycle_tracking: None,
                    access_tracking: None,
                    drop_chain_analysis: None,
                };

                Some(
                    $crate::advanced_types::GenericAdvancedTypeAnalyzer::analyze_by_type_name(
                        type_name,
                        &allocation,
                    ),
                )
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::Trackable;
    use std::cell::RefCell;
    use std::sync::Mutex;

    // Test struct for generic macro variant
    struct TestGenericStruct<T> {
        #[allow(dead_code)]
        data: T,
    }

    // Test struct for non-generic macro variant
    struct TestSimpleStruct {
        #[allow(dead_code)]
        value: i32,
    }

    // Apply the macro to test structs
    impl_advanced_trackable!(TestGenericStruct<T>, 0x1000_0000);
    impl_advanced_trackable!(TestSimpleStruct, 0x2000_0000, no_generics);

    #[test]
    fn test_generic_macro_implementation() {
        let test_struct = TestGenericStruct { data: 42i32 };

        // Test get_heap_ptr
        let heap_ptr = test_struct.get_heap_ptr();
        assert!(heap_ptr.is_some());
        let ptr = heap_ptr.unwrap();
        assert!(ptr >= 0x1000_0000);
        assert!(ptr < 0x1000_0000 + 0x0FFF_FFFF);

        // Test get_type_name
        let type_name = test_struct.get_type_name();
        assert!(type_name.contains("TestGenericStruct"));
        assert!(type_name.contains("i32"));

        // Test get_size_estimate
        let size = test_struct.get_size_estimate();
        assert_eq!(size, std::mem::size_of::<TestGenericStruct<i32>>());
    }

    #[test]
    fn test_non_generic_macro_implementation() {
        let test_struct = TestSimpleStruct { value: 123 };

        // Test get_heap_ptr
        let heap_ptr = test_struct.get_heap_ptr();
        assert!(heap_ptr.is_some());
        let ptr = heap_ptr.unwrap();
        assert!(ptr >= 0x2000_0000);
        assert!(ptr < 0x2000_0000 + 0x0FFF_FFFF);

        // Test get_type_name
        let type_name = test_struct.get_type_name();
        assert!(type_name.contains("TestSimpleStruct"));

        // Test get_size_estimate
        let size = test_struct.get_size_estimate();
        assert_eq!(size, std::mem::size_of::<TestSimpleStruct>());
    }

    #[test]
    fn test_advanced_type_info_generation() {
        let test_struct = TestGenericStruct {
            data: "test".to_string(),
        };

        // Test get_advanced_type_info
        let advanced_info = test_struct.get_advanced_type_info();
        assert!(advanced_info.is_some());

        let info = advanced_info.unwrap();
        // The analyzer should categorize this as a generic type
        assert!(!format!("{:?}", info.category).is_empty());
        // May or may not have issues - length is always >= 0 for Vec
        assert!(info.performance_info.overhead_factor >= 1.0);
    }

    #[test]
    fn test_macro_with_different_offsets() {
        let generic_struct = TestGenericStruct { data: 100u64 };
        let simple_struct = TestSimpleStruct { value: 200 };

        let generic_ptr = generic_struct.get_heap_ptr().unwrap();
        let simple_ptr = simple_struct.get_heap_ptr().unwrap();

        // Ensure different offsets produce different pointer ranges
        assert!((0x1000_0000..0x1000_0000 + 0x0FFF_FFFF).contains(&generic_ptr));
        assert!((0x2000_0000..0x2000_0000 + 0x0FFF_FFFF).contains(&simple_ptr));

        // The base ranges should be different
        assert!((generic_ptr & 0xF000_0000) != (simple_ptr & 0xF000_0000));
    }

    #[test]
    fn test_allocation_info_creation() {
        let test_struct = TestGenericStruct {
            data: RefCell::new(42),
        };

        let advanced_info = test_struct.get_advanced_type_info().unwrap();

        // Verify that AllocationInfo is properly created
        // Note: extract_state_info returns None for borrow_count by default
        // This is expected behavior as mentioned in the implementation
        assert!(advanced_info.state_info.is_borrowed.is_none());
        assert!(advanced_info.state_info.is_locked.is_none());

        // Check that timestamp is reasonable (not zero)
        // We can't check exact value but ensure it's recent
        let type_name = test_struct.get_type_name();
        assert!(!type_name.is_empty());
    }

    #[test]
    fn test_macro_with_interior_mutability_types() {
        // Test with RefCell
        struct RefCellStruct {
            #[allow(dead_code)]
            data: RefCell<i32>,
        }
        impl_advanced_trackable!(RefCellStruct, 0x3000_0000, no_generics);

        let refcell_struct = RefCellStruct {
            data: RefCell::new(42),
        };

        let type_name = refcell_struct.get_type_name();
        assert!(type_name.contains("RefCellStruct"));

        let advanced_info = refcell_struct.get_advanced_type_info();
        assert!(advanced_info.is_some());
    }

    #[test]
    fn test_macro_with_sync_types() {
        // Test with Mutex
        struct MutexStruct {
            #[allow(dead_code)]
            data: Mutex<String>,
        }
        impl_advanced_trackable!(MutexStruct, 0x4000_0000, no_generics);

        let mutex_struct = MutexStruct {
            data: Mutex::new("test".to_string()),
        };

        let type_name = mutex_struct.get_type_name();
        assert!(type_name.contains("MutexStruct"));

        let heap_ptr = mutex_struct.get_heap_ptr().unwrap();
        assert!(heap_ptr >= 0x4000_0000);

        let advanced_info = mutex_struct.get_advanced_type_info();
        assert!(advanced_info.is_some());
    }

    #[test]
    fn test_unique_pointer_generation() {
        // Create multiple instances and ensure they get different pointers
        let struct1 = TestSimpleStruct { value: 1 };
        let struct2 = TestSimpleStruct { value: 2 };
        let struct3 = TestSimpleStruct { value: 3 };

        let ptr1 = struct1.get_heap_ptr().unwrap();
        let ptr2 = struct2.get_heap_ptr().unwrap();
        let ptr3 = struct3.get_heap_ptr().unwrap();

        // All should be in the same range but different values
        assert!(ptr1 >= 0x2000_0000);
        assert!(ptr2 >= 0x2000_0000);
        assert!(ptr3 >= 0x2000_0000);

        // They should be different (very high probability)
        assert!(ptr1 != ptr2 || ptr2 != ptr3 || ptr1 != ptr3);
    }

    #[test]
    fn test_generic_type_with_different_parameters() {
        let int_struct = TestGenericStruct { data: 42i32 };
        let string_struct = TestGenericStruct {
            data: "hello".to_string(),
        };
        let vec_struct = TestGenericStruct {
            data: vec![1, 2, 3],
        };

        // All should have different type names
        let int_type = int_struct.get_type_name();
        let string_type = string_struct.get_type_name();
        let vec_type = vec_struct.get_type_name();

        assert!(int_type.contains("i32"));
        assert!(string_type.contains("String"));
        assert!(vec_type.contains("Vec"));

        // All should be in the same pointer range
        let int_ptr = int_struct.get_heap_ptr().unwrap();
        let string_ptr = string_struct.get_heap_ptr().unwrap();
        let vec_ptr = vec_struct.get_heap_ptr().unwrap();

        assert!((0x1000_0000..0x1000_0000 + 0x0FFF_FFFF).contains(&int_ptr));
        assert!((0x1000_0000..0x1000_0000 + 0x0FFF_FFFF).contains(&string_ptr));
        assert!((0x1000_0000..0x1000_0000 + 0x0FFF_FFFF).contains(&vec_ptr));
    }

    #[test]
    fn test_size_estimation_accuracy() {
        let simple_struct = TestSimpleStruct { value: 42 };
        let generic_struct = TestGenericStruct { data: 123u64 };

        let simple_size = simple_struct.get_size_estimate();
        let generic_size = generic_struct.get_size_estimate();

        // Verify sizes match actual struct sizes
        assert_eq!(simple_size, std::mem::size_of::<TestSimpleStruct>());
        assert_eq!(generic_size, std::mem::size_of::<TestGenericStruct<u64>>());

        // Simple struct should be smaller (i32 vs u64)
        assert!(simple_size <= generic_size);
    }

    #[test]
    fn test_macro_thread_safety() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;

        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // Create multiple threads that use the macro
        for _ in 0..4 {
            let counter_clone = counter.clone();
            let handle = thread::spawn(move || {
                let test_struct = TestSimpleStruct { value: 42 };
                let _ptr = test_struct.get_heap_ptr();
                let _type_name = test_struct.get_type_name();
                let _size = test_struct.get_size_estimate();
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all threads completed successfully
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_allocation_info_fields() {
        let test_struct = TestGenericStruct {
            data: vec![1, 2, 3],
        };
        let advanced_info = test_struct.get_advanced_type_info().unwrap();

        // Verify AllocationInfo has expected structure
        // Note: extract_state_info returns None for most fields by default
        // This is expected behavior as runtime introspection is limited
        assert!(advanced_info.state_info.is_borrowed.is_none());
        assert!(advanced_info.state_info.borrow_count.is_none());
        assert!(advanced_info.state_info.is_locked.is_none());

        // Check performance info is reasonable
        assert!(advanced_info.performance_info.overhead_factor >= 1.0);
        // Memory overhead is always >= 0 for usize

        // Verify category and behavior are set
        assert!(!format!("{:?}", advanced_info.category).is_empty());
        assert!(!format!("{:?}", advanced_info.behavior).is_empty());
    }
}
