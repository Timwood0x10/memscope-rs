use std::{
    backtrace::Backtrace,
    collections::HashMap,
    sync::Arc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use std::sync::Mutex;
use thiserror::Error;

/// Error type for memory tracking operations
#[derive(Error, Debug)]
pub enum MemoryError {
    /// Failed to lock a mutex
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Invalid operation on memory tracking
    #[error("Memory tracking error: {0}")]
    TrackingError(String),
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub timestamp_alloc: u128,
    pub timestamp_dealloc: Option<u128>,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub backtrace: String,
    pub thread_id: u64,
}

#[derive(Default)]
pub struct MemoryTracker {
    active_allocations: Mutex<HashMap<usize, AllocationInfo>>,
    allocation_log: Mutex<Vec<AllocationInfo>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_allocation(
        &self,
        ptr: usize,
        size: usize,
        type_name: Option<String>,
    ) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations: {}", e)))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MemoryError::TrackingError("System time before UNIX_EPOCH".to_string()))?
            .as_millis();

        active.insert(
            ptr,
            AllocationInfo {
                ptr,
                size,
                timestamp_alloc: timestamp,
                timestamp_dealloc: None,
                var_name: None,
                type_name,
                backtrace: format!("{:?}", Backtrace::capture()),
                thread_id: {
                    // Stable way to get a unique thread identifier
                    use std::hash::{Hash, Hasher};
                    use std::collections::hash_map::DefaultHasher;
                    
                    let mut hasher = DefaultHasher::new();
                    thread::current().id().hash(&mut hasher);
                    hasher.finish()
                },
            },
        );
        
        Ok(())
    }

    pub fn track_deallocation(&self, ptr: usize) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations: {}", e)))?;
            
        if let Some(mut info) = active.remove(&ptr) {
            info.timestamp_dealloc = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| MemoryError::TrackingError("System time before UNIX_EPOCH".to_string()))?
                    .as_millis(),
            );
            
            let mut log = self.allocation_log.lock()
                .map_err(|e| MemoryError::LockError(format!("Failed to lock allocation_log: {}", e)))?;
            log.push(info);
            
            Ok(())
        } else {
            Err(MemoryError::TrackingError(format!("No active allocation found for pointer: 0x{:x}", ptr)))
        }
    }

    pub fn associate_var(&self, ptr: usize, var_name: String, type_name: String) -> Result<(), MemoryError> {
        let mut active = self.active_allocations.lock()
            .map_err(|e| MemoryError::LockError(format!("Failed to lock active_allocations: {}", e)))?;
            
        if let Some(info) = active.get_mut(&ptr) {
            info.var_name = Some(var_name);
            info.type_name = Some(type_name);
            Ok(())
        } else {
            Err(MemoryError::TrackingError(
                format!("No active allocation found for pointer: 0x{:x}", ptr)
            ))
        }
    }

    pub fn get_stats(&self) -> MemoryStats {
        let active = match self.active_allocations.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        let total_allocations = active.len();
        let total_memory = active.values().map(|a| a.size).sum();
        
        MemoryStats {
            total_allocations,
            total_memory,
        }
    }

    pub fn get_active_allocations(&self) -> Vec<AllocationInfo> {
        let active = match self.active_allocations.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        active.values().cloned().collect()
    }

    pub fn get_allocation_log(&self) -> Vec<AllocationInfo> {
        let log = match self.allocation_log.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        log.clone()
    }

    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        crate::export::export_to_json(self, path)
    }

    pub fn export_to_svg<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        crate::export::export_to_svg(self, path)
    }

}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_allocations: usize,
    pub total_memory: usize,
}

pub fn thread_id() -> std::thread::ThreadId {
    thread::current().id()
}

lazy_static::lazy_static! {
    static ref GLOBAL_TRACKER: Arc<MemoryTracker> = Arc::new(MemoryTracker::new());
}

/// Get a reference to the global memory tracker
///
/// This function returns a thread-safe reference counted pointer to the
/// global memory tracker instance.
pub fn get_global_tracker() -> Arc<MemoryTracker> {
    GLOBAL_TRACKER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_new_tracker() {
        let tracker = MemoryTracker::new();
        assert!(tracker.get_active_allocations().is_empty(), "New tracker should have no active allocations");
        assert!(tracker.get_allocation_log().is_empty(), "New tracker should have an empty allocation log");
        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0, "New tracker should have 0 total allocations");
        assert_eq!(stats.total_memory, 0, "New tracker should have 0 total memory");
    }

    #[test]
    fn test_track_allocation_and_stats() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x1000, 100, Some("TestType1".to_string())).unwrap();
        tracker.track_allocation(0x2000, 200, Some("TestType2".to_string())).unwrap();

        let active_allocs = tracker.get_active_allocations();
        assert_eq!(active_allocs.len(), 2, "Should have two active allocations");

        let alloc1 = active_allocs.iter().find(|a| a.ptr == 0x1000).expect("Allocation 0x1000 not found");
        assert_eq!(alloc1.size, 100);
        assert_eq!(alloc1.type_name.as_deref(), Some("TestType1"));

        let alloc2 = active_allocs.iter().find(|a| a.ptr == 0x2000).expect("Allocation 0x2000 not found");
        assert_eq!(alloc2.size, 200);
        assert_eq!(alloc2.type_name.as_deref(), Some("TestType2"));

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 2, "Stats should show 2 total allocations");
        assert_eq!(stats.total_memory, 300, "Stats should show 300 total memory");
    }

    #[test]
    fn test_track_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x3000, 50, Some("TestDealloc".to_string())).unwrap();
        
        // Ensure timestamps are different
        sleep(Duration::from_millis(1)); 
        tracker.track_deallocation(0x3000).unwrap();

        assert!(tracker.get_active_allocations().iter().find(|a| a.ptr == 0x3000).is_none(), "Allocation 0x3000 should not be active");
        
        let log = tracker.get_allocation_log();
        assert_eq!(log.len(), 1, "Allocation log should have one entry");
        let logged_alloc = &log[0];
        assert_eq!(logged_alloc.ptr, 0x3000);
        assert!(logged_alloc.timestamp_dealloc.is_some(), "Deallocation timestamp should be set");
        assert_ne!(logged_alloc.timestamp_alloc, logged_alloc.timestamp_dealloc.unwrap(), "Alloc and dealloc timestamps should differ");


        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 0, "Stats should show 0 active allocations after deallocation");
    }

    #[test]
    fn test_associate_var() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x4000, 70, Some("BaseType".to_string())).unwrap();
        tracker.associate_var(0x4000, "my_var".to_string(), "SpecificType".to_string()).unwrap();

        let active_allocs = tracker.get_active_allocations();
        let alloc_info = active_allocs.iter().find(|a| a.ptr == 0x4000).expect("Allocation 0x4000 not found");
        
        assert_eq!(alloc_info.var_name.as_deref(), Some("my_var"), "Variable name should be 'my_var'");
        assert_eq!(alloc_info.type_name.as_deref(), Some("SpecificType"), "Type name should be 'SpecificType'");

        let result = tracker.associate_var(0xBAD_PTR, "bad_var".to_string(), "BadType".to_string());
        assert!(result.is_err(), "Associating var to a bad pointer should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }
    
    #[test]
    fn test_deallocation_of_unknown_ptr() {
        let tracker = MemoryTracker::new();
        let result = tracker.track_deallocation(0xDEADBEEF);
        assert!(result.is_err(), "Deallocating an unknown pointer should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }

    #[test]
    fn test_double_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation(0x5000, 10, Some("DoubleDealloc".to_string())).unwrap();
        tracker.track_deallocation(0x5000).unwrap(); // First deallocation should be Ok

        let result = tracker.track_deallocation(0x5000); // Second deallocation
        assert!(result.is_err(), "Double deallocation should return an error");
        matches!(result, Err(MemoryError::TrackingError(_)));
    }
}