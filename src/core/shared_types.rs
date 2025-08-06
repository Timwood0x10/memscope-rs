//! Arc-shared data structures for reducing clone overhead
//! 
//! This module provides Arc-wrapped versions of commonly cloned data structures
//! to reduce memory usage and improve performance while maintaining API compatibility.

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::core::types::AllocationInfo;
use crate::core::optimized_types::OptimizedAllocationInfo;

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
        let shared_infos = infos.into_iter()
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
        self.allocations.iter()
            .map(|shared| shared.to_allocation_info())
            .collect()
    }
    
    /// Filter allocations by predicate
    pub fn filter<F>(&self, predicate: F) -> SharedAllocationCollection
    where
        F: Fn(&SharedAllocationInfo) -> bool,
    {
        let filtered: Vec<SharedAllocationInfo> = self.allocations.iter()
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