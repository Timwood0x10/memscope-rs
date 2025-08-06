//! Utilities for optimizing clone operations
//! 
//! This module provides simple utility functions to help optimize clone
//! operations by using Arc sharing where appropriate.

use std::sync::Arc;
use crate::core::types::AllocationInfo;
use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::clone_monitor;

/// Create an Arc-shared version of AllocationInfo
pub fn share_allocation_info(info: AllocationInfo) -> Arc<OptimizedAllocationInfo> {
    let optimized = OptimizedAllocationInfo::from(info);
    let size = std::mem::size_of::<OptimizedAllocationInfo>();
    
    clone_monitor::record_optimized_clone("AllocationInfo", size);
    Arc::new(optimized)
}

/// Clone an Arc-shared AllocationInfo (cheap operation)
pub fn clone_shared_allocation(arc_info: &Arc<OptimizedAllocationInfo>) -> Arc<OptimizedAllocationInfo> {
    let size = std::mem::size_of::<OptimizedAllocationInfo>();
    clone_monitor::record_avoided_clone("AllocationInfo", size);
    arc_info.clone()
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
pub fn share_allocation_vector(infos: Vec<AllocationInfo>) -> Arc<Vec<Arc<OptimizedAllocationInfo>>> {
    let shared_infos: Vec<Arc<OptimizedAllocationInfo>> = infos
        .into_iter()
        .map(share_allocation_info)
        .collect();
    
    Arc::new(shared_infos)
}

/// Check if a type should use Arc sharing based on size and usage patterns
pub fn should_use_arc_sharing(type_name: &str, size: usize) -> bool {
    // Use Arc for large objects or frequently cloned types
    size > 1024 || 
    type_name.contains("AllocationInfo") ||
    type_name.contains("Config") ||
    type_name.contains("Result") ||
    type_name.contains("Collection")
}

/// Optimize a clone operation by choosing between regular clone and Arc sharing
pub fn optimized_clone<T: Clone>(value: &T) -> T 
where
    T: 'static,
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

    #[test]
    fn test_share_allocation_info() {
        let info = AllocationInfo::new(0x1000, 64);
        let shared = share_allocation_info(info);
        
        assert_eq!(shared.ptr, 0x1000);
        assert_eq!(shared.size, 64);
    }

    #[test]
    fn test_clone_shared_allocation() {
        let info = AllocationInfo::new(0x1000, 64);
        let shared1 = share_allocation_info(info);
        let shared2 = clone_shared_allocation(&shared1);
        
        // Both should point to the same data
        assert!(Arc::ptr_eq(&shared1, &shared2));
    }

    #[test]
    fn test_should_use_arc_sharing() {
        assert!(should_use_arc_sharing("AllocationInfo", 100));
        assert!(should_use_arc_sharing("SomeConfig", 100));
        assert!(should_use_arc_sharing("SmallType", 2000)); // Large size
        assert!(!should_use_arc_sharing("i32", 4)); // Small primitive
    }
}