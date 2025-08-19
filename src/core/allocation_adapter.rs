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
    pub fn iter(&self) -> std::slice::Iter<AllocationInfoAdapter> {
        self.allocations.iter()
    }

    /// Get a mutable iterator over the allocations
    pub fn iter_mut(&mut self) -> std::slice::IterMut<AllocationInfoAdapter> {
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
}
