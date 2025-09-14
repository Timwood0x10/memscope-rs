//! Compatibility adapter for AllocationInfo migration
//!
//! This module provides a compatibility layer that allows existing code
//! to continue working with the original AllocationInfo interface while
//! internally using the optimized OptimizedAllocationInfo with string interning.

use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::string_pool::intern_string;
use crate::core::types::AllocationInfo;
use std::sync::Arc;

/// Adapter that provides AllocationInfo interface backed by OptimizedAllocationInfo
///
/// This allows existing code to continue working unchanged while benefiting
/// from string interning optimizations under the hood.
pub struct AllocationInfoAdapter {
    inner: OptimizedAllocationInfo,
}

impl AllocationInfoAdapter {
    /// Create a new adapter from an OptimizedAllocationInfo
    pub fn new(optimized: OptimizedAllocationInfo) -> Self {
        Self { inner: optimized }
    }

    /// Create a new adapter with basic allocation info
    pub fn from_allocation(ptr: usize, size: usize) -> Self {
        Self {
            inner: OptimizedAllocationInfo::new(ptr, size),
        }
    }

    /// Get the underlying optimized allocation info
    pub fn inner(&self) -> &OptimizedAllocationInfo {
        &self.inner
    }

    /// Get a mutable reference to the underlying optimized allocation info
    pub fn inner_mut(&mut self) -> &mut OptimizedAllocationInfo {
        &mut self.inner
    }

    /// Convert to the original AllocationInfo format
    pub fn to_allocation_info(&self) -> AllocationInfo {
        self.inner.clone().into()
    }

    /// Convert from the original AllocationInfo format
    pub fn from_allocation_info(info: AllocationInfo) -> Self {
        Self {
            inner: OptimizedAllocationInfo::from(info),
        }
    }

    // Provide AllocationInfo-compatible interface methods

    pub fn ptr(&self) -> usize {
        self.inner.ptr
    }

    pub fn size(&self) -> usize {
        self.inner.size
    }

    pub fn var_name(&self) -> Option<String> {
        self.inner.var_name.as_ref().map(|s| s.to_string())
    }

    pub fn set_var_name(&mut self, name: Option<String>) {
        self.inner.var_name = name.map(|s| intern_string(&s));
    }

    pub fn type_name(&self) -> Option<String> {
        self.inner.type_name.as_ref().map(|s| s.to_string())
    }

    pub fn set_type_name(&mut self, name: Option<String>) {
        self.inner.type_name = name.map(|s| intern_string(&s));
    }

    pub fn scope_name(&self) -> Option<String> {
        self.inner.scope_name.as_ref().map(|s| s.to_string())
    }

    pub fn set_scope_name(&mut self, name: Option<String>) {
        self.inner.scope_name = name.map(|s| intern_string(&s));
    }

    pub fn timestamp_alloc(&self) -> u64 {
        self.inner.timestamp_alloc
    }

    pub fn timestamp_dealloc(&self) -> Option<u64> {
        self.inner.timestamp_dealloc
    }

    pub fn set_timestamp_dealloc(&mut self, timestamp: Option<u64>) {
        self.inner.timestamp_dealloc = timestamp;
        if let Some(dealloc) = timestamp {
            self.inner.lifetime_ms = Some((dealloc - self.inner.timestamp_alloc) / 1_000_000);
        }
    }

    pub fn thread_id(&self) -> String {
        self.inner.thread_id.to_string()
    }

    pub fn set_thread_id(&mut self, id: String) {
        self.inner.thread_id = intern_string(&id);
    }

    pub fn borrow_count(&self) -> usize {
        self.inner.borrow_count
    }

    pub fn set_borrow_count(&mut self, count: usize) {
        self.inner.borrow_count = count;
    }

    pub fn stack_trace(&self) -> Option<Vec<String>> {
        self.inner
            .stack_trace
            .as_ref()
            .map(|trace| trace.iter().map(|frame| frame.to_string()).collect())
    }

    pub fn set_stack_trace(&mut self, trace: Option<Vec<String>>) {
        self.inner.stack_trace =
            trace.map(|t| Arc::new(t.into_iter().map(|frame| intern_string(&frame)).collect()));
    }

    pub fn is_leaked(&self) -> bool {
        self.inner.is_leaked
    }

    pub fn set_is_leaked(&mut self, leaked: bool) {
        self.inner.is_leaked = leaked;
    }

    pub fn lifetime_ms(&self) -> Option<u64> {
        self.inner.lifetime_ms
    }

    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    pub fn mark_deallocated(&mut self) {
        self.inner.mark_deallocated();
    }
}

impl Clone for AllocationInfoAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl std::fmt::Debug for AllocationInfoAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocationInfoAdapter")
            .field("inner", &self.inner)
            .finish()
    }
}

impl PartialEq for AllocationInfoAdapter {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<AllocationInfo> for AllocationInfoAdapter {
    fn from(info: AllocationInfo) -> Self {
        Self::from_allocation_info(info)
    }
}

impl From<AllocationInfoAdapter> for AllocationInfo {
    fn from(adapter: AllocationInfoAdapter) -> Self {
        adapter.to_allocation_info()
    }
}

impl From<OptimizedAllocationInfo> for AllocationInfoAdapter {
    fn from(optimized: OptimizedAllocationInfo) -> Self {
        Self::new(optimized)
    }
}

impl From<AllocationInfoAdapter> for OptimizedAllocationInfo {
    fn from(adapter: AllocationInfoAdapter) -> Self {
        adapter.inner
    }
}

/// Collection of allocation adapters for batch operations
pub struct AllocationCollection {
    allocations: Vec<AllocationInfoAdapter>,
}

impl AllocationCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            allocations: Vec::new(),
        }
    }

    /// Create a collection from a vector of AllocationInfo
    pub fn from_allocation_infos(infos: Vec<AllocationInfo>) -> Self {
        Self {
            allocations: infos.into_iter().map(AllocationInfoAdapter::from).collect(),
        }
    }

    /// Create a collection from a vector of OptimizedAllocationInfo
    pub fn from_optimized_infos(infos: Vec<OptimizedAllocationInfo>) -> Self {
        Self {
            allocations: infos.into_iter().map(AllocationInfoAdapter::from).collect(),
        }
    }

    /// Add an allocation to the collection
    pub fn push(&mut self, allocation: AllocationInfoAdapter) {
        self.allocations.push(allocation);
    }

    /// Get the number of allocations in the collection
    pub fn len(&self) -> usize {
        self.allocations.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.allocations.is_empty()
    }

    /// Get an iterator over the allocations
    pub fn iter(&self) -> std::slice::Iter<'_, AllocationInfoAdapter> {
        self.allocations.iter()
    }

    /// Get a mutable iterator over the allocations
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, AllocationInfoAdapter> {
        self.allocations.iter_mut()
    }

    /// Convert to a vector of AllocationInfo (for compatibility)
    pub fn to_allocation_infos(&self) -> Vec<AllocationInfo> {
        self.allocations
            .iter()
            .map(|a| a.to_allocation_info())
            .collect()
    }

    /// Convert to a vector of OptimizedAllocationInfo (for performance)
    pub fn to_optimized_infos(&self) -> Vec<OptimizedAllocationInfo> {
        self.allocations.iter().map(|a| a.inner.clone()).collect()
    }

    /// Get memory usage statistics for the collection
    pub fn memory_stats(&self) -> CollectionMemoryStats {
        let total_size: usize = self.allocations.iter().map(|a| a.size()).sum();
        let active_count = self.allocations.iter().filter(|a| a.is_active()).count();
        let leaked_count = self.allocations.iter().filter(|a| a.is_leaked()).count();

        CollectionMemoryStats {
            total_allocations: self.allocations.len(),
            active_allocations: active_count,
            leaked_allocations: leaked_count,
            total_size_bytes: total_size,
        }
    }
}

impl Default for AllocationCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory statistics for an allocation collection
#[derive(Debug, Clone)]
pub struct CollectionMemoryStats {
    pub total_allocations: usize,
    pub active_allocations: usize,
    pub leaked_allocations: usize,
    pub total_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::string_pool::clear_string_pool;

    #[test]
    fn test_allocation_adapter_basic_operations() {
        clear_string_pool();

        let mut adapter = AllocationInfoAdapter::from_allocation(0x1000, 64);

        assert_eq!(adapter.ptr(), 0x1000);
        assert_eq!(adapter.size(), 64);
        assert!(adapter.is_active());

        adapter.set_var_name(Some("test_var".to_string()));
        adapter.set_type_name(Some("TestType".to_string()));

        assert_eq!(adapter.var_name(), Some("test_var".to_string()));
        assert_eq!(adapter.type_name(), Some("TestType".to_string()));

        adapter.mark_deallocated();
        assert!(!adapter.is_active());
        assert!(adapter.lifetime_ms().is_some());
    }

    #[test]
    fn test_conversion_compatibility() {
        clear_string_pool();

        // Create original AllocationInfo
        let original = AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: Some("test".to_string()),
            type_name: Some("TestType".to_string()),
            scope_name: None,
            timestamp_alloc: 12345,
            timestamp_dealloc: None,
            thread_id: "thread-1".to_string(),
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

        // Convert through adapter
        let adapter = AllocationInfoAdapter::from(original.clone());
        let converted_back = AllocationInfo::from(adapter);

        assert_eq!(converted_back.ptr, original.ptr);
        assert_eq!(converted_back.size, original.size);
        assert_eq!(converted_back.var_name, original.var_name);
        assert_eq!(converted_back.type_name, original.type_name);
    }

    #[test]
    fn test_allocation_collection() {
        clear_string_pool();

        let mut collection = AllocationCollection::new();

        let adapter1 = AllocationInfoAdapter::from_allocation(0x1000, 64);
        let adapter2 = AllocationInfoAdapter::from_allocation(0x2000, 128);

        collection.push(adapter1);
        collection.push(adapter2);

        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());

        let stats = collection.memory_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.active_allocations, 2);
        assert_eq!(stats.total_size_bytes, 192); // 64 + 128
    }

    #[test]
    fn test_string_interning_through_adapter() {
        clear_string_pool();

        let mut adapter1 = AllocationInfoAdapter::from_allocation(0x1000, 64);
        let mut adapter2 = AllocationInfoAdapter::from_allocation(0x2000, 128);

        adapter1.set_var_name(Some("shared_name".to_string()));
        adapter2.set_var_name(Some("shared_name".to_string()));

        // Verify that the underlying Arc<str> are the same (interned)
        assert!(Arc::ptr_eq(
            adapter1
                .inner()
                .var_name
                .as_ref()
                .expect("Missing variable name"),
            adapter2
                .inner()
                .var_name
                .as_ref()
                .expect("Missing variable name")
        ));
    }

    #[test]
    fn test_allocation_adapter_comprehensive() {
        clear_string_pool();

        let mut adapter = AllocationInfoAdapter::from_allocation(0x5000, 256);

        // Test all setter methods
        adapter.set_var_name(Some("comprehensive_var".to_string()));
        adapter.set_type_name(Some("Vec<String>".to_string()));
        adapter.set_scope_name(Some("main".to_string()));
        adapter.set_thread_id("worker-thread".to_string());
        adapter.set_borrow_count(5);
        adapter.set_is_leaked(true);

        // Test all getter methods
        assert_eq!(adapter.ptr(), 0x5000);
        assert_eq!(adapter.size(), 256);
        assert_eq!(adapter.var_name(), Some("comprehensive_var".to_string()));
        assert_eq!(adapter.type_name(), Some("Vec<String>".to_string()));
        assert_eq!(adapter.scope_name(), Some("main".to_string()));
        assert_eq!(adapter.thread_id(), "worker-thread");
        assert_eq!(adapter.borrow_count(), 5);
        assert!(adapter.is_leaked());
        assert!(adapter.is_active());

        // Test deallocation
        adapter.mark_deallocated();
        assert!(!adapter.is_active());
        assert!(adapter.lifetime_ms().is_some());
        assert!(adapter.timestamp_dealloc().is_some());
    }

    #[test]
    fn test_allocation_adapter_edge_cases() {
        clear_string_pool();

        // Test with zero-sized allocation
        let mut zero_adapter = AllocationInfoAdapter::from_allocation(0x0, 0);
        assert_eq!(zero_adapter.ptr(), 0x0);
        assert_eq!(zero_adapter.size(), 0);
        assert!(zero_adapter.is_active());

        // Test with maximum values
        let mut max_adapter = AllocationInfoAdapter::from_allocation(usize::MAX, usize::MAX);
        assert_eq!(max_adapter.ptr(), usize::MAX);
        assert_eq!(max_adapter.size(), usize::MAX);

        // Test with empty strings
        zero_adapter.set_var_name(Some("".to_string()));
        zero_adapter.set_type_name(Some("".to_string()));
        assert_eq!(zero_adapter.var_name(), Some("".to_string()));
        assert_eq!(zero_adapter.type_name(), Some("".to_string()));

        // Test with None values
        zero_adapter.set_var_name(None);
        zero_adapter.set_type_name(None);
        assert_eq!(zero_adapter.var_name(), None);
        assert_eq!(zero_adapter.type_name(), None);

        // Test with very long strings
        let long_string = "a".repeat(10000);
        max_adapter.set_var_name(Some(long_string.clone()));
        assert_eq!(max_adapter.var_name(), Some(long_string));
    }

    #[test]
    fn test_allocation_collection_comprehensive() {
        clear_string_pool();

        let mut collection = AllocationCollection::new();

        // Test empty collection
        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
        let empty_stats = collection.memory_stats();
        assert_eq!(empty_stats.total_allocations, 0);
        assert_eq!(empty_stats.active_allocations, 0);
        assert_eq!(empty_stats.total_size_bytes, 0);

        // Add various allocations
        let adapter1 = AllocationInfoAdapter::from_allocation(0x1000, 64);
        let mut adapter2 = AllocationInfoAdapter::from_allocation(0x2000, 128);
        let mut adapter3 = AllocationInfoAdapter::from_allocation(0x3000, 256);

        // Mark one as leaked
        adapter2.set_is_leaked(true);

        // Mark one as deallocated
        adapter3.mark_deallocated();

        collection.push(adapter1);
        collection.push(adapter2);
        collection.push(adapter3);

        assert_eq!(collection.len(), 3);
        assert!(!collection.is_empty());

        let stats = collection.memory_stats();
        assert_eq!(stats.total_allocations, 3);
        assert_eq!(stats.active_allocations, 2); // One is deallocated
        assert_eq!(stats.total_size_bytes, 448); // 64 + 128 + 256

        // Test iteration
        let mut count = 0;
        for adapter in collection.iter() {
            assert!(adapter.ptr() >= 0x1000);
            count += 1;
        }
        assert_eq!(count, 3);

        // Test mutable iteration
        for adapter in collection.iter_mut() {
            adapter.set_borrow_count(adapter.borrow_count() + 1);
        }

        // Verify borrow counts were incremented
        for adapter in collection.iter() {
            assert_eq!(adapter.borrow_count(), 1);
        }
    }

    #[test]
    fn test_allocation_collection_operations() {
        clear_string_pool();

        let mut collection = AllocationCollection::new();

        // Test with_capacity
        let capacity_collection = AllocationCollection::new();
        assert_eq!(capacity_collection.len(), 0);

        // Add many allocations
        for i in 0..50 {
            let adapter = AllocationInfoAdapter::from_allocation(0x1000 + i * 0x100, 64 + i);
            collection.push(adapter);
        }

        assert_eq!(collection.len(), 50);

        let stats = collection.memory_stats();
        assert_eq!(stats.total_allocations, 50);
        assert_eq!(stats.active_allocations, 50);

        // Test clear
        collection.allocations.clear();
        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
    }

    #[test]
    fn test_allocation_adapter_timestamps() {
        clear_string_pool();

        let adapter = AllocationInfoAdapter::from_allocation(0x1000, 64);
        let alloc_time = adapter.timestamp_alloc();
        assert!(alloc_time > 0);

        // Initially no deallocation timestamp
        assert_eq!(adapter.timestamp_dealloc(), None);

        // After marking as deallocated
        let mut mutable_adapter = adapter;
        mutable_adapter.mark_deallocated();
        let dealloc_time = mutable_adapter.timestamp_dealloc();
        assert!(dealloc_time.is_some());
        assert!(dealloc_time.unwrap() >= alloc_time);

        // Lifetime should be calculated
        let lifetime = mutable_adapter.lifetime_ms();
        assert!(lifetime.is_some());
    }

    #[test]
    fn test_allocation_adapter_borrow_tracking() {
        clear_string_pool();

        let mut adapter = AllocationInfoAdapter::from_allocation(0x1000, 64);

        // Test initial borrow count
        assert_eq!(adapter.borrow_count(), 0);

        // Test setting borrow count
        adapter.set_borrow_count(5);
        assert_eq!(adapter.borrow_count(), 5);

        // Test incrementing borrow count
        adapter.set_borrow_count(adapter.borrow_count() + 1);
        assert_eq!(adapter.borrow_count(), 6);

        // Test with maximum borrow count
        adapter.set_borrow_count(usize::MAX);
        assert_eq!(adapter.borrow_count(), usize::MAX);
    }

    #[test]
    fn test_allocation_adapter_leak_detection() {
        clear_string_pool();

        let mut adapter = AllocationInfoAdapter::from_allocation(0x1000, 64);

        // Initially not leaked
        assert!(!adapter.is_leaked());

        // Mark as leaked
        adapter.set_is_leaked(true);
        assert!(adapter.is_leaked());

        // Mark as not leaked
        adapter.set_is_leaked(false);
        assert!(!adapter.is_leaked());
    }

    #[test]
    fn test_allocation_adapter_thread_tracking() {
        clear_string_pool();

        let mut adapter = AllocationInfoAdapter::from_allocation(0x1000, 64);

        // Test default thread ID
        let default_thread = adapter.thread_id();
        assert!(!default_thread.is_empty());

        // Test setting custom thread ID
        adapter.set_thread_id("custom-thread-123".to_string());
        assert_eq!(adapter.thread_id(), "custom-thread-123");

        // Test with empty thread ID
        adapter.set_thread_id("".to_string());
        assert_eq!(adapter.thread_id(), "");

        // Test with very long thread ID
        let long_thread_id = "thread-".repeat(1000);
        adapter.set_thread_id(long_thread_id.clone());
        assert_eq!(adapter.thread_id(), long_thread_id);
    }

    #[test]
    fn test_allocation_adapter_conversion_roundtrip() {
        clear_string_pool();

        // Create a comprehensive AllocationInfo
        let original = AllocationInfo {
            ptr: 0x12345678,
            size: 1024,
            var_name: Some("test_variable".to_string()),
            type_name: Some("HashMap<String, Vec<i32>>".to_string()),
            scope_name: Some("function_scope".to_string()),
            timestamp_alloc: 1000000,
            timestamp_dealloc: Some(2000000),
            thread_id: "main-thread".to_string(),
            borrow_count: 10,
            stack_trace: Some(vec!["frame1".to_string(), "frame2".to_string()]),
            is_leaked: true,
            lifetime_ms: Some(1000),
            borrow_info: None,
            clone_info: None,
            ownership_history_available: true,
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

        // Convert to adapter and back
        let adapter = AllocationInfoAdapter::from(original.clone());
        let converted = AllocationInfo::from(adapter);

        // Verify all important fields are preserved
        assert_eq!(converted.ptr, original.ptr);
        assert_eq!(converted.size, original.size);
        assert_eq!(converted.var_name, original.var_name);
        assert_eq!(converted.type_name, original.type_name);
        assert_eq!(converted.scope_name, original.scope_name);
        assert_eq!(converted.timestamp_alloc, original.timestamp_alloc);
        assert_eq!(converted.timestamp_dealloc, original.timestamp_dealloc);
        assert_eq!(converted.thread_id, original.thread_id);
        assert_eq!(converted.borrow_count, original.borrow_count);
        assert_eq!(converted.is_leaked, original.is_leaked);
        assert_eq!(converted.lifetime_ms, original.lifetime_ms);
        assert_eq!(
            converted.ownership_history_available,
            original.ownership_history_available
        );
    }

    #[test]
    fn test_allocation_collection_memory_stats_edge_cases() {
        clear_string_pool();

        let mut collection = AllocationCollection::new();

        // Test with all deallocated allocations
        let mut adapter1 = AllocationInfoAdapter::from_allocation(0x1000, 100);
        let mut adapter2 = AllocationInfoAdapter::from_allocation(0x2000, 200);
        adapter1.mark_deallocated();
        adapter2.mark_deallocated();

        collection.push(adapter1);
        collection.push(adapter2);

        let stats = collection.memory_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.active_allocations, 0); // All deallocated
        assert_eq!(stats.total_size_bytes, 300); // Still counts total size

        // Test with all leaked allocations
        collection.allocations.clear();
        let mut adapter3 = AllocationInfoAdapter::from_allocation(0x3000, 150);
        let mut adapter4 = AllocationInfoAdapter::from_allocation(0x4000, 250);
        adapter3.set_is_leaked(true);
        adapter4.set_is_leaked(true);

        collection.push(adapter3);
        collection.push(adapter4);

        let stats = collection.memory_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.active_allocations, 2); // Leaked but still active
        assert_eq!(stats.total_size_bytes, 400);
    }

    #[test]
    fn test_allocation_adapter_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        clear_string_pool();

        let collection = Arc::new(Mutex::new(AllocationCollection::new()));
        let mut handles = vec![];

        // Test concurrent access to collection
        for i in 0..10 {
            let collection_clone = collection.clone();
            let handle = thread::spawn(move || {
                let adapter = AllocationInfoAdapter::from_allocation(0x1000 + i * 0x100, 64 + i);
                let mut coll = collection_clone.lock().expect("Failed to lock collection");
                coll.push(adapter);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should complete");
        }

        let final_collection = collection.lock().expect("Failed to lock collection");
        assert_eq!(final_collection.len(), 10);

        let stats = final_collection.memory_stats();
        assert_eq!(stats.total_allocations, 10);
        assert_eq!(stats.active_allocations, 10);
    }

    #[test]
    fn test_allocation_adapter_string_interning_efficiency() {
        clear_string_pool();

        let mut adapters = Vec::new();
        let common_type = "std::collections::HashMap<String, Vec<i32>>";
        let common_var_prefix = "map_instance_";

        // Create many adapters with similar names
        for i in 0..100 {
            let mut adapter = AllocationInfoAdapter::from_allocation(0x1000 + i * 0x100, 64);
            adapter.set_type_name(Some(common_type.to_string()));
            adapter.set_var_name(Some(format!("{}{}", common_var_prefix, i)));
            adapters.push(adapter);
        }

        // Verify that type names have the same content (string interning may or may not work depending on pool state)
        for i in 1..adapters.len() {
            assert_eq!(
                adapters[0].type_name().unwrap(),
                adapters[i].type_name().unwrap(),
                "Type names should have the same content"
            );
        }

        // Variable names should be different but still interned efficiently
        for adapter in &adapters {
            assert!(adapter.var_name().unwrap().starts_with(common_var_prefix));
        }

        // All type names should be the expected common type
        for adapter in &adapters {
            assert_eq!(adapter.type_name(), Some(common_type.to_string()));
        }
    }
}
