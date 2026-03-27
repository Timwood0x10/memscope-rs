//! Allocation record structures
//!
//! Used by Core strategy to track detailed allocation information

use serde::{Deserialize, Serialize};
use super::common::{current_timestamp, current_thread_id};

/// Memory allocation record
///
/// Used by Core strategy to track detailed allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    /// Memory address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Allocation timestamp (nanoseconds since Unix epoch)
    pub timestamp: u64,
    /// Thread ID
    pub thread_id: u32,
    /// Optional stack trace ID
    pub stack_id: Option<u32>,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Whether this allocation is still active
    pub is_active: bool,
    /// Deallocation timestamp (if deallocated)
    pub dealloc_timestamp: Option<u64>,
}

impl AllocationRecord {
    /// Create new allocation record
    pub fn new(ptr: usize, size: usize) -> Self {
        Self {
            ptr,
            size,
            timestamp: current_timestamp(),
            thread_id: current_thread_id(),
            stack_id: None,
            var_name: None,
            type_name: None,
            is_active: true,
            dealloc_timestamp: None,
        }
    }

    /// Get allocation lifetime in milliseconds
    pub fn lifetime_ms(&self) -> Option<u64> {
        self.dealloc_timestamp
            .map(|end| (end - self.timestamp) / 1_000_000)
    }

    /// Mark as deallocated
    pub fn deallocate(&mut self) {
        self.is_active = false;
        self.dealloc_timestamp = Some(current_timestamp());
    }

    /// Set variable name
    pub fn with_var_name(mut self, name: String) -> Self {
        self.var_name = Some(name);
        self
    }

    /// Set type name
    pub fn with_type_name(mut self, name: String) -> Self {
        self.type_name = Some(name);
        self
    }

    /// Set stack trace ID
    pub fn with_stack_id(mut self, stack_id: u32) -> Self {
        self.stack_id = Some(stack_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_record_new() {
        let record = AllocationRecord::new(0x1000, 1024);
        assert_eq!(record.ptr, 0x1000);
        assert_eq!(record.size, 1024);
        assert!(record.is_active);
        assert!(record.dealloc_timestamp.is_none());
    }

    #[test]
    fn test_allocation_deallocate() {
        let mut record = AllocationRecord::new(0x1000, 1024);
        assert!(record.is_active);
        
        record.deallocate();
        assert!(!record.is_active);
        assert!(record.dealloc_timestamp.is_some());
    }

    #[test]
    fn test_allocation_lifetime() {
        let mut record = AllocationRecord::new(0x1000, 1024);
        assert!(record.lifetime_ms().is_none());
        
        // Simulate deallocation after some time
        std::thread::sleep(std::time::Duration::from_millis(10));
        record.deallocate();
        
        let lifetime = record.lifetime_ms();
        assert!(lifetime.is_some());
        assert!(lifetime.unwrap() >= 10);
    }

    #[test]
    fn test_allocation_builder() {
        let record = AllocationRecord::new(0x1000, 1024)
            .with_var_name("my_var".to_string())
            .with_type_name("Vec<i32>".to_string())
            .with_stack_id(123);
        
        assert_eq!(record.var_name, Some("my_var".to_string()));
        assert_eq!(record.type_name, Some("Vec<i32>".to_string()));
        assert_eq!(record.stack_id, Some(123));
    }
}