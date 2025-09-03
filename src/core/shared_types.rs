//! Arc-shared data structures for reducing clone overhead
//!
//! This module provides Arc-wrapped versions of commonly cloned data structures
//! to reduce memory usage and improve performance while maintaining API compatibility.

use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::types::AllocationInfo;
use std::sync::Arc;

/// Arc-shared allocation information for reduced clone overhead
#[derive(Debug, Clone)]
pub struct SharedAllocationInfo {
    /// The actual allocation info wrapped in Arc
    inner: Arc<OptimizedAllocationInfo>,
}

impl SharedAllocationInfo {
    /// Create a new shared allocation info
    pub fn new(info: OptimizedAllocationInfo) -> Self {
        Self {
            inner: Arc::new(info),
        }
    }

    /// Create from regular AllocationInfo
    pub fn from_allocation_info(info: AllocationInfo) -> Self {
        Self::new(OptimizedAllocationInfo::from(info))
    }

    /// Get a reference to the inner data
    pub fn inner(&self) -> &OptimizedAllocationInfo {
        &self.inner
    }

    /// Get the Arc for sharing
    pub fn arc(&self) -> Arc<OptimizedAllocationInfo> {
        self.inner.clone()
    }

    /// Convert back to regular AllocationInfo
    pub fn to_allocation_info(&self) -> AllocationInfo {
        (*self.inner).clone().into()
    }

    // Delegate common methods to inner
    pub fn ptr(&self) -> usize {
        self.inner.ptr
    }

    pub fn size(&self) -> usize {
        self.inner.size
    }

    pub fn var_name_str(&self) -> Option<&str> {
        self.inner.var_name_str()
    }

    pub fn type_name_str(&self) -> Option<&str> {
        self.inner.type_name_str()
    }

    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    pub fn lifetime_duration_ms(&self) -> Option<u64> {
        self.inner.lifetime_duration_ms()
    }
}

/// Arc-shared collection of allocations
#[derive(Debug, Clone)]
pub struct SharedAllocationCollection {
    /// The allocations wrapped in Arc
    allocations: Arc<Vec<SharedAllocationInfo>>,
}

impl SharedAllocationCollection {
    /// Create a new shared collection
    pub fn new(allocations: Vec<SharedAllocationInfo>) -> Self {
        Self {
            allocations: Arc::new(allocations),
        }
    }

    /// Create from regular AllocationInfo vector
    pub fn from_allocation_infos(infos: Vec<AllocationInfo>) -> Self {
        let shared_infos = infos
            .into_iter()
            .map(SharedAllocationInfo::from_allocation_info)
            .collect();
        Self::new(shared_infos)
    }

    /// Get the number of allocations
    pub fn len(&self) -> usize {
        self.allocations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.allocations.is_empty()
    }

    /// Get an iterator over the allocations
    pub fn iter(&self) -> std::slice::Iter<SharedAllocationInfo> {
        self.allocations.iter()
    }

    /// Get a specific allocation by index
    pub fn get(&self, index: usize) -> Option<&SharedAllocationInfo> {
        self.allocations.get(index)
    }

    /// Get the Arc for sharing the entire collection
    pub fn arc(&self) -> Arc<Vec<SharedAllocationInfo>> {
        self.allocations.clone()
    }

    /// Convert back to regular AllocationInfo vector
    pub fn to_allocation_infos(&self) -> Vec<AllocationInfo> {
        self.allocations
            .iter()
            .map(|shared| shared.to_allocation_info())
            .collect()
    }

    /// Filter allocations by predicate
    pub fn filter<F>(&self, predicate: F) -> SharedAllocationCollection
    where
        F: Fn(&SharedAllocationInfo) -> bool,
    {
        let filtered: Vec<SharedAllocationInfo> = self
            .allocations
            .iter()
            .filter(|info| predicate(info))
            .cloned()
            .collect();
        SharedAllocationCollection::new(filtered)
    }

    /// Get total memory usage
    pub fn total_memory(&self) -> usize {
        self.allocations.iter().map(|info| info.size()).sum()
    }

    /// Get active allocations
    pub fn active_allocations(&self) -> SharedAllocationCollection {
        self.filter(|info| info.is_active())
    }
}

/// Arc-shared configuration for reduced clone overhead
#[derive(Debug, Clone)]
pub struct SharedConfig<T> {
    /// The configuration wrapped in Arc
    inner: Arc<T>,
}

impl<T> SharedConfig<T> {
    /// Create a new shared config
    pub fn new(config: T) -> Self {
        Self {
            inner: Arc::new(config),
        }
    }

    /// Get a reference to the inner config
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get the Arc for sharing
    pub fn arc(&self) -> Arc<T> {
        self.inner.clone()
    }
}

impl<T: Clone> SharedConfig<T> {
    /// Get a cloned copy of the inner config
    pub fn to_owned(&self) -> T {
        (*self.inner).clone()
    }
}

impl<T> std::ops::Deref for SharedConfig<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    fn create_test_allocation_info(ptr: usize, size: usize, var_name: &str) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: Some(var_name.to_string()),
            type_name: Some("i32".to_string()),
            timestamp_alloc: 1000,
            borrow_count: 0,
            scope_name: None,
            timestamp_dealloc: None,
            thread_id: "test_thread".to_string(),
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
    fn test_shared_allocation_info_creation() {
        let allocation_info = create_test_allocation_info(0x1000, 1024, "test_var");
        let shared_info = SharedAllocationInfo::from_allocation_info(allocation_info);
        
        assert_eq!(shared_info.ptr(), 0x1000);
        assert_eq!(shared_info.size(), 1024);
        assert_eq!(shared_info.var_name_str(), Some("test_var"));
        assert_eq!(shared_info.type_name_str(), Some("i32"));
        assert!(shared_info.is_active());
    }

    #[test]
    fn test_shared_allocation_info_conversion() {
        let original_info = create_test_allocation_info(0x2000, 2048, "test_vector");
        let shared_info = SharedAllocationInfo::from_allocation_info(original_info.clone());
        let converted_back = shared_info.to_allocation_info();
        
        assert_eq!(converted_back.ptr, original_info.ptr);
        assert_eq!(converted_back.size, original_info.size);
        assert_eq!(converted_back.var_name, original_info.var_name);
        assert_eq!(converted_back.type_name, original_info.type_name);
    }

    #[test]
    fn test_shared_allocation_info_arc_sharing() {
        let allocation_info = create_test_allocation_info(0x3000, 512, "shared_var");
        let shared_info = SharedAllocationInfo::from_allocation_info(allocation_info);
        
        let arc_copy = shared_info.arc();
        let inner_ref = shared_info.inner();
        
        // Both should point to the same data
        assert_eq!(inner_ref.ptr, arc_copy.ptr);
        assert_eq!(inner_ref.size, arc_copy.size);
        assert!(std::ptr::eq(inner_ref, arc_copy.as_ref()));
    }

    #[test]
    fn test_shared_allocation_info_lifetime_duration() {
        let mut allocation_info = create_test_allocation_info(0x4000, 256, "timed_var");
        // Set both timestamp_dealloc and lifetime_ms to simulate a deallocated allocation
        allocation_info.timestamp_dealloc = Some(allocation_info.timestamp_alloc + 500_000_000); // 500ms later in nanoseconds
        allocation_info.lifetime_ms = Some(500);
        
        let shared_info = SharedAllocationInfo::from_allocation_info(allocation_info);
        assert_eq!(shared_info.lifetime_duration_ms(), Some(500));
    }

    #[test]
    fn test_shared_allocation_collection_creation() {
        let infos = vec![
            create_test_allocation_info(0x1000, 512, "var1"),
            create_test_allocation_info(0x2000, 1024, "var2"),
            create_test_allocation_info(0x3000, 256, "var3"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        
        assert_eq!(collection.len(), 3);
        assert!(!collection.is_empty());
        assert_eq!(collection.total_memory(), 1792); // 512 + 1024 + 256
    }

    #[test]
    fn test_shared_allocation_collection_empty() {
        let collection = SharedAllocationCollection::new(vec![]);
        
        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
        assert_eq!(collection.total_memory(), 0);
    }

    #[test]
    fn test_shared_allocation_collection_get() {
        let infos = vec![
            create_test_allocation_info(0x1000, 512, "var1"),
            create_test_allocation_info(0x2000, 1024, "var2"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        
        let first = collection.get(0);
        assert!(first.is_some());
        assert_eq!(first.unwrap().ptr(), 0x1000);
        
        let second = collection.get(1);
        assert!(second.is_some());
        assert_eq!(second.unwrap().ptr(), 0x2000);
        
        let third = collection.get(2);
        assert!(third.is_none());
    }

    #[test]
    fn test_shared_allocation_collection_iteration() {
        let infos = vec![
            create_test_allocation_info(0x1000, 512, "var1"),
            create_test_allocation_info(0x2000, 1024, "var2"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        
        let mut count = 0;
        let mut total_size = 0;
        for info in collection.iter() {
            count += 1;
            total_size += info.size();
        }
        
        assert_eq!(count, 2);
        assert_eq!(total_size, 1536);
    }

    #[test]
    fn test_shared_allocation_collection_filter() {
        let infos = vec![
            create_test_allocation_info(0x1000, 512, "small_var"),
            create_test_allocation_info(0x2000, 2048, "large_var"),
            create_test_allocation_info(0x3000, 256, "tiny_var"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        let large_allocations = collection.filter(|info| info.size() > 1000);
        
        assert_eq!(large_allocations.len(), 1);
        assert_eq!(large_allocations.get(0).unwrap().size(), 2048);
        assert_eq!(large_allocations.get(0).unwrap().var_name_str(), Some("large_var"));
    }

    #[test]
    fn test_shared_allocation_collection_active_allocations() {
        let mut active_info = create_test_allocation_info(0x1000, 512, "active_var");
        let mut inactive_info = create_test_allocation_info(0x2000, 1024, "inactive_var");
        inactive_info.timestamp_dealloc = Some(2000); // Mark as deallocated
        
        let infos = vec![active_info, inactive_info];
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        let active = collection.active_allocations();
        
        assert_eq!(active.len(), 1);
        assert_eq!(active.get(0).unwrap().var_name_str(), Some("active_var"));
    }

    #[test]
    fn test_shared_allocation_collection_conversion() {
        let original_infos = vec![
            create_test_allocation_info(0x1000, 512, "var1"),
            create_test_allocation_info(0x2000, 1024, "var2"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(original_infos.clone());
        let converted_back = collection.to_allocation_infos();
        
        assert_eq!(converted_back.len(), original_infos.len());
        for (original, converted) in original_infos.iter().zip(converted_back.iter()) {
            assert_eq!(original.ptr, converted.ptr);
            assert_eq!(original.size, converted.size);
            assert_eq!(original.var_name, converted.var_name);
        }
    }

    #[test]
    fn test_shared_allocation_collection_arc_sharing() {
        let infos = vec![
            create_test_allocation_info(0x1000, 512, "var1"),
        ];
        
        let collection = SharedAllocationCollection::from_allocation_infos(infos);
        let arc_copy = collection.arc();
        
        // Both should point to the same data
        assert_eq!(collection.len(), arc_copy.len());
        assert!(std::ptr::eq(collection.allocations.as_ref(), arc_copy.as_ref()));
    }

    #[test]
    fn test_shared_config_creation() {
        #[derive(Debug, Clone, PartialEq)]
        struct TestConfig {
            value: i32,
            name: String,
        }
        
        let config = TestConfig {
            value: 42,
            name: "test".to_string(),
        };
        
        let shared_config = SharedConfig::new(config.clone());
        
        assert_eq!(shared_config.inner().value, 42);
        assert_eq!(shared_config.inner().name, "test");
        assert_eq!(shared_config.to_owned(), config);
    }

    #[test]
    fn test_shared_config_deref() {
        #[derive(Debug, Clone)]
        struct TestConfig {
            value: i32,
        }
        
        let config = TestConfig { value: 100 };
        let shared_config = SharedConfig::new(config);
        
        // Test deref functionality
        assert_eq!(shared_config.value, 100);
    }

    #[test]
    fn test_shared_config_arc_sharing() {
        #[derive(Debug, Clone)]
        struct TestConfig {
            data: Vec<i32>,
        }
        
        let config = TestConfig {
            data: vec![1, 2, 3, 4, 5],
        };
        
        let shared_config = SharedConfig::new(config);
        let arc_copy = shared_config.arc();
        
        // Both should point to the same data
        assert_eq!(shared_config.inner().data, arc_copy.data);
        assert!(std::ptr::eq(shared_config.inner(), arc_copy.as_ref()));
    }

    #[test]
    fn test_shared_config_clone() {
        #[derive(Debug, Clone, PartialEq)]
        struct TestConfig {
            value: String,
        }
        
        let config = TestConfig {
            value: "original".to_string(),
        };
        
        let shared_config = SharedConfig::new(config);
        let cloned_config = shared_config.clone();
        
        // Both should point to the same Arc
        assert!(std::ptr::eq(shared_config.inner(), cloned_config.inner()));
        assert_eq!(shared_config.inner().value, cloned_config.inner().value);
    }

    #[test]
    fn test_shared_config_non_clone_type() {
        struct NonCloneConfig {
            value: i32,
        }
        
        let config = NonCloneConfig { value: 42 };
        let shared_config = SharedConfig::new(config);
        
        assert_eq!(shared_config.inner().value, 42);
        assert_eq!(shared_config.value, 42); // Test deref
        
        // Note: to_owned() is not available for non-Clone types
        // This is expected behavior
    }
}
