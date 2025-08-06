//! Optimizations for the memory tracker to reduce clone overhead
//! 
//! This module contains optimized versions of key tracker functions that use
//! Arc sharing instead of expensive clone operations.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::core::types::{AllocationInfo, TrackingResult, MemoryStats};
use crate::core::optimized_types::OptimizedAllocationInfo;
use crate::core::clone_monitor;

/// Optimized version of track_allocation that uses Arc sharing
pub fn track_allocation_optimized(
    active_allocations: &Mutex<HashMap<usize, Arc<OptimizedAllocationInfo>>>,
    stats: &Mutex<MemoryStats>,
    ptr: usize,
    size: usize,
) -> TrackingResult<()> {
    let allocation = OptimizedAllocationInfo::new(ptr, size);
    let shared_allocation = Arc::new(allocation);
    
    // Record that we avoided a clone by using Arc
    clone_monitor::record_avoided_clone("AllocationInfo", size);
    
    match (active_allocations.try_lock(), stats.try_lock()) {
        (Ok(mut active), Ok(mut stats)) => {
            // Insert the Arc - no cloning needed!
            active.insert(ptr, shared_allocation);
            
            // Update statistics
            stats.total_allocations = stats.total_allocations.saturating_add(1);
            stats.active_allocations = stats.active_allocations.saturating_add(1);
            stats.active_memory = stats.active_memory.saturating_add(size);
            
            Ok(())
        }
        _ => Err(crate::core::types::TrackingError::ThreadSafetyError("Lock contention".to_string())),
    }
}

/// Optimized version of get_active_allocations that returns shared data
pub fn get_active_allocations_optimized(
    active_allocations: &Mutex<HashMap<usize, Arc<OptimizedAllocationInfo>>>,
) -> TrackingResult<Vec<Arc<OptimizedAllocationInfo>>> {
    match active_allocations.lock() {
        Ok(active) => {
            let shared_allocations: Vec<Arc<OptimizedAllocationInfo>> = active
                .values()
                .map(|arc_info| {
                    // This is a cheap Arc clone, not a data clone
                    clone_monitor::record_avoided_clone("AllocationInfo", 
                        std::mem::size_of::<OptimizedAllocationInfo>());
                    arc_info.clone()
                })
                .collect();
            
            Ok(shared_allocations)
        }
        Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError("Lock contention".to_string())),
    }
}

/// Optimized version of associate_var that minimizes cloning
pub fn associate_var_optimized(
    active_allocations: &Mutex<HashMap<usize, Arc<OptimizedAllocationInfo>>>,
    ptr: usize,
    var_name: String,
    type_name: String,
) -> TrackingResult<()> {
    match active_allocations.lock() {
        Ok(mut active) => {
            if let Some(arc_allocation) = active.get_mut(&ptr) {
                // Only clone the data when we need to modify it
                let mut updated_allocation = (**arc_allocation).clone();
                updated_allocation.var_name = Some(crate::core::string_pool::intern_string(&var_name));
                updated_allocation.type_name = Some(crate::core::string_pool::intern_string(&type_name));
                
                // Replace with new Arc
                *arc_allocation = Arc::new(updated_allocation);
                
                clone_monitor::record_optimized_clone("AllocationInfo", 
                    std::mem::size_of::<OptimizedAllocationInfo>());
                
                Ok(())
            } else {
                Err(crate::core::types::TrackingError::InvalidPointer("Allocation not found".to_string()))
            }
        }
        Err(_) => Err(crate::core::types::TrackingError::ThreadSafetyError("Lock contention".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::collections::HashMap;

    #[test]
    fn test_track_allocation_optimized() {
        let active_allocations = Mutex::new(HashMap::new());
        let stats = Mutex::new(MemoryStats::default());
        
        let result = track_allocation_optimized(&active_allocations, &stats, 0x1000, 64);
        assert!(result.is_ok());
        
        // Verify allocation was stored
        let active = active_allocations.lock().unwrap();
        assert!(active.contains_key(&0x1000));
        assert_eq!(active.get(&0x1000).unwrap().size, 64);
    }

    #[test]
    fn test_get_active_allocations_optimized() {
        let mut active_map = HashMap::new();
        let allocation = Arc::new(OptimizedAllocationInfo::new(0x1000, 64));
        active_map.insert(0x1000, allocation);
        
        let active_allocations = Mutex::new(active_map);
        
        let result = get_active_allocations_optimized(&active_allocations);
        assert!(result.is_ok());
        
        let allocations = result.unwrap();
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].ptr, 0x1000);
    }
}