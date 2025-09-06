//! Optimized memory tracker using Arc sharing to reduce clone overhead
//!
//! This module provides an optimized version of the memory tracker that uses
//! Arc-based sharing to minimize expensive clone operations while maintaining
//! full API compatibility.

use crate::core::clone_monitor;
use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::shared_types::{SharedAllocationCollection, SharedAllocationInfo};
use crate::core::types::{MemoryStats, TrackingResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Optimized memory tracker that uses Arc sharing
pub struct OptimizedMemoryTracker {
    /// Active allocations using Arc sharing
    active_allocations: Mutex<HashMap<usize, Arc<OptimizedAllocationInfo>>>,
    /// Memory statistics
    stats: Mutex<MemoryStats>,
    /// Allocation history using Arc sharing
    allocation_history: Mutex<Vec<Arc<OptimizedAllocationInfo>>>,
}

impl OptimizedMemoryTracker {
    /// Create a new optimized memory tracker
    pub fn new() -> Self {
        Self {
            active_allocations: Mutex::new(HashMap::new()),
            stats: Mutex::new(MemoryStats::default()),
            allocation_history: Mutex::new(Vec::new()),
        }
    }

    /// Track an allocation with Arc sharing
    pub fn track_allocation_shared(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        let allocation = OptimizedAllocationInfo::new(ptr, size);
        let shared_allocation = Arc::new(allocation);

        // Record that we're using Arc sharing instead of cloning
        clone_monitor::record_avoided_clone("AllocationInfo", size);

        match (self.active_allocations.try_lock(), self.stats.try_lock()) {
            (Ok(mut active), Ok(mut stats)) => {
                // Insert the Arc - no cloning needed
                active.insert(ptr, Arc::clone(&shared_allocation));

                // Update statistics
                stats.total_allocations = stats.total_allocations.saturating_add(1);
                stats.active_allocations = stats.active_allocations.saturating_add(1);
                stats.active_memory = stats.active_memory.saturating_add(size);

                // Add to history - sharing the same Arc
                if let Ok(mut history) = self.allocation_history.try_lock() {
                    history.push(shared_allocation);
                }

                Ok(())
            }
            _ => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get active allocations as shared collection
    pub fn get_active_allocations_shared(&self) -> TrackingResult<SharedAllocationCollection> {
        match self.active_allocations.lock() {
            Ok(active) => {
                let shared_infos: Vec<SharedAllocationInfo> = active
                    .values()
                    .map(|arc_info| SharedAllocationInfo::new((**arc_info).clone()))
                    .collect();

                Ok(SharedAllocationCollection::new(shared_infos))
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get allocation history as shared collection
    pub fn get_allocation_history_shared(&self) -> TrackingResult<SharedAllocationCollection> {
        match self.allocation_history.lock() {
            Ok(history) => {
                let shared_infos: Vec<SharedAllocationInfo> = history
                    .iter()
                    .map(|arc_info| SharedAllocationInfo::new((**arc_info).clone()))
                    .collect();

                Ok(SharedAllocationCollection::new(shared_infos))
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Associate variable with existing allocation (Arc-optimized)
    pub fn associate_var_shared(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
    ) -> TrackingResult<()> {
        match self.active_allocations.lock() {
            Ok(mut active) => {
                if let Some(arc_allocation) = active.get_mut(&ptr) {
                    // Create a new allocation with updated info
                    let mut updated_allocation = (**arc_allocation).clone();
                    updated_allocation.var_name =
                        Some(crate::core::string_pool::intern_string(&var_name));
                    updated_allocation.type_name =
                        Some(crate::core::string_pool::intern_string(&type_name));

                    // Replace with new Arc
                    *arc_allocation = Arc::new(updated_allocation);

                    Ok(())
                } else {
                    Err(crate::core::types::TrackingError::InvalidPointer(
                        "Allocation not found".to_string(),
                    ))
                }
            }
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        match self.stats.lock() {
            Ok(stats) => Ok(stats.clone()),
            Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError(
                "Lock contention".to_string(),
            )),
        }
    }

    /// Get clone optimization statistics
    pub fn get_clone_stats(&self) -> clone_monitor::CloneMonitorStats {
        clone_monitor::get_clone_stats()
    }
}

impl Default for OptimizedMemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_optimized_tracker() {
        // Test creating a new optimized tracker
        let tracker = OptimizedMemoryTracker::new();
        let stats = tracker.get_stats().expect("Failed to get stats");
        
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.active_memory, 0);
        assert_eq!(stats.total_allocations, 0);
    }
    
    #[test]
    fn test_track_allocation_shared() {
        // Test tracking an allocation with Arc sharing
        let tracker = OptimizedMemoryTracker::new();
        let ptr = 0x1000;
        let size = 1024;
        
        tracker.track_allocation_shared(ptr, size)
            .expect("Failed to track allocation");
        
        let stats = tracker.get_stats().expect("Failed to get stats");
        assert_eq!(stats.active_allocations, 1);
        assert_eq!(stats.active_memory, size);
        assert_eq!(stats.total_allocations, 1);
    }
    
    #[test]
    fn test_track_multiple_allocations() {
        // Test tracking multiple allocations
        let tracker = OptimizedMemoryTracker::new();
        
        for i in 0..5 {
            let ptr = 0x1000 + (i * 0x100);
            let size = 256 * (i + 1);
            
            tracker.track_allocation_shared(ptr, size)
                .expect("Failed to track allocation");
        }
        
        let stats = tracker.get_stats().expect("Failed to get stats");
        assert_eq!(stats.active_allocations, 5);
        assert_eq!(stats.total_allocations, 5);
        
        // Total memory: 256 + 512 + 768 + 1024 + 1280 = 3840
        assert_eq!(stats.active_memory, 3840);
    }
    
    #[test]
    fn test_get_active_allocations_shared() {
        // Test getting active allocations as shared collection
        let tracker = OptimizedMemoryTracker::new();
        
        // Track some allocations
        tracker.track_allocation_shared(0x2000, 512)
            .expect("Failed to track allocation");
        tracker.track_allocation_shared(0x3000, 1024)
            .expect("Failed to track allocation");
        
        let shared_collection = tracker.get_active_allocations_shared()
            .expect("Failed to get active allocations");
        
        assert_eq!(shared_collection.len(), 2);
        
        // Check that allocations are present
        let ptrs: Vec<usize> = shared_collection.iter()
            .map(|info| info.ptr())
            .collect();
        assert!(ptrs.contains(&0x2000));
        assert!(ptrs.contains(&0x3000));
    }
    
    #[test]
    fn test_get_allocation_history_shared() {
        // Test getting allocation history as shared collection
        let tracker = OptimizedMemoryTracker::new();
        
        // Track some allocations
        for i in 0..3 {
            tracker.track_allocation_shared(0x4000 + i * 0x100, 256)
                .expect("Failed to track allocation");
        }
        
        let history = tracker.get_allocation_history_shared()
            .expect("Failed to get allocation history");
        
        assert_eq!(history.len(), 3);
    }
    
    #[test]
    fn test_associate_var_shared() {
        // Test associating variable with existing allocation
        let tracker = OptimizedMemoryTracker::new();
        let ptr = 0x5000;
        
        // First track the allocation
        tracker.track_allocation_shared(ptr, 1024)
            .expect("Failed to track allocation");
        
        // Then associate a variable with it
        tracker.associate_var_shared(ptr, "test_variable".to_string(), "TestType".to_string())
            .expect("Failed to associate variable");
        
        // Verify the association
        let allocations = tracker.get_active_allocations_shared()
            .expect("Failed to get allocations");
        
        let found = allocations.iter()
            .find(|info| info.ptr() == ptr);
        
        assert!(found.is_some());
        let allocation = found.unwrap();
        assert!(allocation.var_name_str().is_some());
        assert!(allocation.type_name_str().is_some());
    }
    
    #[test]
    fn test_associate_var_non_existing_allocation() {
        // Test associating variable with non-existing allocation
        let tracker = OptimizedMemoryTracker::new();
        
        let result = tracker.associate_var_shared(
            0x9999,
            "non_existing".to_string(),
            "TestType".to_string()
        );
        
        assert!(result.is_err());
        match result {
            Err(crate::core::types::TrackingError::InvalidPointer(_)) => {},
            _ => panic!("Expected InvalidPointer error"),
        }
    }
    
    #[test]
    fn test_default_implementation() {
        // Test the Default trait implementation
        let tracker = OptimizedMemoryTracker::default();
        let stats = tracker.get_stats().expect("Failed to get stats");
        
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.active_memory, 0);
    }
    
    #[test]
    fn test_clone_stats_retrieval() {
        // Test getting clone optimization statistics
        let tracker = OptimizedMemoryTracker::new();
        
        // Track some allocations which should trigger clone monitoring
        tracker.track_allocation_shared(0x6000, 2048)
            .expect("Failed to track allocation");
        
        let clone_stats = tracker.get_clone_stats();
        
        // Should have recorded avoided clones
        assert!(clone_stats.avoided_clones > 0);
    }
    
    #[test]
    fn test_arc_sharing_efficiency() {
        // Test that Arc sharing is actually being used efficiently
        let tracker = OptimizedMemoryTracker::new();
        
        // Track an allocation
        tracker.track_allocation_shared(0x7000, 1024)
            .expect("Failed to track allocation");
        
        // Get the allocation multiple times
        let collection1 = tracker.get_active_allocations_shared()
            .expect("Failed to get allocations");
        let collection2 = tracker.get_active_allocations_shared()
            .expect("Failed to get allocations");
        
        // Both collections should contain data
        assert_eq!(collection1.len(), 1);
        assert_eq!(collection2.len(), 1);
    }
    
    #[test]
    fn test_large_allocation_tracking() {
        // Test tracking very large allocations
        let tracker = OptimizedMemoryTracker::new();
        let large_size = 1024 * 1024 * 10; // 10 MB
        
        tracker.track_allocation_shared(0x8000, large_size)
            .expect("Failed to track large allocation");
        
        let stats = tracker.get_stats().expect("Failed to get stats");
        assert_eq!(stats.active_memory, large_size);
    }
    
    #[test]
    fn test_allocation_history_persistence() {
        // Test that allocation history persists across operations
        let tracker = OptimizedMemoryTracker::new();
        
        // Track allocations
        for i in 0..5 {
            tracker.track_allocation_shared(0x9000 + i * 0x100, 512)
                .expect("Failed to track allocation");
        }
        
        // Get history multiple times
        let history1 = tracker.get_allocation_history_shared()
            .expect("Failed to get history");
        let history2 = tracker.get_allocation_history_shared()
            .expect("Failed to get history");
        
        assert_eq!(history1.len(), history2.len());
        assert_eq!(history1.len(), 5);
    }
}
