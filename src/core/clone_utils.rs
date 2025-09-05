//! Utilities for optimizing clone operations
//!
//! This module provides simple utility functions to help optimize clone
//! operations by using Arc sharing where appropriate.

use crate::core::clone_monitor;
use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::types::AllocationInfo;
use std::sync::Arc;

/// Create an Arc-shared version of AllocationInfo
pub fn share_allocation_info(info: AllocationInfo) -> Arc<OptimizedAllocationInfo> {
    let optimized = OptimizedAllocationInfo::from(info);
    let size = std::mem::size_of::<OptimizedAllocationInfo>();

    clone_monitor::record_optimized_clone("AllocationInfo", size);
    Arc::new(optimized)
}

/// Clone an Arc-shared AllocationInfo (cheap operation)
pub fn clone_shared_allocation(
    arc_info: &Arc<OptimizedAllocationInfo>,
) -> Arc<OptimizedAllocationInfo> {
    let size = std::mem::size_of::<OptimizedAllocationInfo>();
    clone_monitor::record_avoided_clone("AllocationInfo", size);
    Arc::clone(arc_info)
}

/// Convert Arc-shared back to regular AllocationInfo when needed
pub fn unshare_allocation_info(arc_info: Arc<OptimizedAllocationInfo>) -> AllocationInfo {
    // Try to avoid cloning if we're the only reference
    match Arc::try_unwrap(arc_info) {
        Ok(optimized) => optimized.into(),
        Err(arc_info) => (*arc_info).clone().into(),
    }
}

/// Create a shared vector of allocations
pub fn share_allocation_vector(
    infos: Vec<AllocationInfo>,
) -> Arc<Vec<Arc<OptimizedAllocationInfo>>> {
    let shared_infos: Vec<Arc<OptimizedAllocationInfo>> =
        infos.into_iter().map(share_allocation_info).collect();

    Arc::new(shared_infos)
}

/// Check if a type should use Arc sharing based on size and usage patterns
pub fn should_use_arc_sharing(type_name: &str, size: usize) -> bool {
    // Use Arc for large objects or frequently cloned types
    size > 1024
        || type_name.contains("AllocationInfo")
        || type_name.contains("Config")
        || type_name.contains("Result")
        || type_name.contains("Collection")
}

/// Optimize a clone operation by choosing between regular clone and Arc sharing
pub fn optimized_clone<T>(value: &T) -> T
where
    T: Clone + 'static,
{
    let type_name = std::any::type_name::<T>();
    let size = std::mem::size_of::<T>();

    if should_use_arc_sharing(type_name, size) {
        // For types that benefit from Arc sharing, we would need to restructure
        // the calling code. For now, just record the clone and suggest optimization.
        clone_monitor::record_clone(type_name, size, 0);
        value.clone()
    } else {
        // Regular clone for small/simple types
        let start = std::time::Instant::now();
        let result = value.clone();
        let duration = start.elapsed().as_nanos() as u64;
        clone_monitor::record_clone(type_name, size, duration);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    
    /// Helper function to create a test AllocationInfo
    fn create_test_allocation_info(ptr: usize, size: usize) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: Some(format!("test_var_{}", ptr)),
            type_name: Some("TestType".to_string()),
            scope_name: None,
            timestamp_alloc: 1000000,
            timestamp_dealloc: None,
            thread_id: "test_thread".to_string(),
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
        }
    }
    
    #[test]
    fn test_share_allocation_info() {
        // Test converting AllocationInfo to Arc-shared OptimizedAllocationInfo
        let info = create_test_allocation_info(0x1000, 1024);
        let shared = share_allocation_info(info);
        
        assert_eq!(shared.ptr, 0x1000);
        assert_eq!(shared.size, 1024);
        assert_eq!(shared.var_name.as_ref().map(|s| s.as_ref()), Some("test_var_4096"));
    }
    
    #[test]
    fn test_clone_shared_allocation() {
        // Test that cloning an Arc-shared allocation is cheap
        let info = create_test_allocation_info(0x2000, 2048);
        let shared = share_allocation_info(info);
        
        // Clone the shared allocation
        let cloned = clone_shared_allocation(&shared);
        
        // Both should point to the same data
        assert_eq!(Arc::strong_count(&shared), 2);
        assert_eq!(shared.ptr, cloned.ptr);
        assert_eq!(shared.size, cloned.size);
    }
    
    #[test]
    fn test_unshare_allocation_info_single_reference() {
        // Test converting Arc-shared back to regular AllocationInfo when single reference
        let info = create_test_allocation_info(0x3000, 512);
        let original_ptr = info.ptr;
        let original_size = info.size;
        
        let shared = share_allocation_info(info);
        assert_eq!(Arc::strong_count(&shared), 1);
        
        let unshared = unshare_allocation_info(shared);
        assert_eq!(unshared.ptr, original_ptr);
        assert_eq!(unshared.size, original_size);
    }
    
    #[test]
    fn test_unshare_allocation_info_multiple_references() {
        // Test converting Arc-shared back when multiple references exist
        let info = create_test_allocation_info(0x4000, 256);
        let shared = share_allocation_info(info);
        let _cloned = Arc::clone(&shared);
        
        assert_eq!(Arc::strong_count(&shared), 2);
        
        // This should clone the data since there are multiple references
        let unshared = unshare_allocation_info(shared);
        assert_eq!(unshared.ptr, 0x4000);
        assert_eq!(unshared.size, 256);
    }
    
    #[test]
    fn test_share_allocation_vector() {
        // Test creating a shared vector of allocations
        let infos = vec![
            create_test_allocation_info(0x5000, 128),
            create_test_allocation_info(0x6000, 256),
            create_test_allocation_info(0x7000, 512),
        ];
        
        let shared_vec = share_allocation_vector(infos);
        
        assert_eq!(shared_vec.len(), 3);
        assert_eq!(shared_vec[0].ptr, 0x5000);
        assert_eq!(shared_vec[1].ptr, 0x6000);
        assert_eq!(shared_vec[2].ptr, 0x7000);
    }
    
    #[test]
    fn test_should_use_arc_sharing_by_size() {
        // Test Arc sharing decision based on size
        assert!(should_use_arc_sharing("SomeType", 2048)); // Large size
        assert!(!should_use_arc_sharing("SomeType", 512)); // Small size
        assert!(!should_use_arc_sharing("TinyType", 64));  // Very small size
    }
    
    #[test]
    fn test_should_use_arc_sharing_by_type_name() {
        // Test Arc sharing decision based on type name
        assert!(should_use_arc_sharing("AllocationInfo", 100));
        assert!(should_use_arc_sharing("SomeConfig", 100));
        assert!(should_use_arc_sharing("QueryResult", 100));
        assert!(should_use_arc_sharing("DataCollection", 100));
        
        // Should not use Arc for small simple types even if size is moderate
        assert!(!should_use_arc_sharing("SimpleStruct", 100));
    }
    
    #[test]
    fn test_optimized_clone_small_type() {
        // Test optimized clone for small types
        let small_value = 42u32;
        let cloned = optimized_clone(&small_value);
        assert_eq!(cloned, 42);
    }
    
    #[test]
    fn test_optimized_clone_string() {
        // Test optimized clone for String type
        let test_string = "Test string for cloning".to_string();
        let cloned = optimized_clone(&test_string);
        assert_eq!(cloned, test_string);
    }
    
    #[test]
    fn test_optimized_clone_vector() {
        // Test optimized clone for Vector type
        let test_vec = vec![1, 2, 3, 4, 5];
        let cloned = optimized_clone(&test_vec);
        assert_eq!(cloned, test_vec);
    }
    
    #[test]
    fn test_arc_reference_counting() {
        // Test that Arc reference counting works correctly
        let info = create_test_allocation_info(0x8000, 1024);
        let shared1 = share_allocation_info(info);
        
        assert_eq!(Arc::strong_count(&shared1), 1);
        
        let shared2 = clone_shared_allocation(&shared1);
        assert_eq!(Arc::strong_count(&shared1), 2);
        assert_eq!(Arc::strong_count(&shared2), 2);
        
        let shared3 = clone_shared_allocation(&shared2);
        assert_eq!(Arc::strong_count(&shared1), 3);
        assert_eq!(Arc::strong_count(&shared2), 3);
        assert_eq!(Arc::strong_count(&shared3), 3);
        
        drop(shared3);
        assert_eq!(Arc::strong_count(&shared1), 2);
        
        drop(shared2);
        assert_eq!(Arc::strong_count(&shared1), 1);
    }
}
