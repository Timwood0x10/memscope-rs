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
