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