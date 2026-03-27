//! Allocation record structures
//!
//! Used by Core strategy to track detailed allocation information

use super::common::{current_thread_id, current_timestamp};
use serde::{Deserialize, Serialize};

/// Enhanced borrowing information for allocations
///
/// This structure tracks borrowing patterns for individual allocations,
/// providing insights into how data is accessed and shared across the program.
///
/// # Fields
///
/// - `immutable_borrows`: Total count of immutable borrow operations (e.g., `&T`) during the allocation's lifetime.
///   This helps identify frequently read data that might benefit from caching or immutability optimizations.
///
/// - `mutable_borrows`: Total count of mutable borrow operations (e.g., `&mut T`) during the allocation's lifetime.
///   High mutable borrow counts may indicate contention or suggest refactoring opportunities.
///
/// - `max_concurrent_borrows`: Peak number of simultaneous borrows observed at any point in time.
///   Calculated by tracking active borrows during each borrow/check operation.
///   This metric helps identify hotspots with high contention.
///
/// - `last_borrow_timestamp`: Timestamp (in microseconds since Unix epoch) of the most recent borrow event.
///   Useful for tracking "cold" allocations that haven't been accessed recently.
///
/// # Example
///
/// ```rust
/// use memscope_rs::data::BorrowInfo;
///
/// let borrow_info = BorrowInfo {
///     immutable_borrows: 150,  // 150 immutable references taken
///     mutable_borrows: 5,      // 5 mutable references taken
///     max_concurrent_borrows: 8,  // At most 8 borrows active simultaneously
///     last_borrow_timestamp: Some(1234567890),  // Last borrow at this timestamp
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Total number of immutable borrows during lifetime
    pub immutable_borrows: usize,
    /// Total number of mutable borrows during lifetime
    pub mutable_borrows: usize,
    /// Peak number of simultaneous borrows observed
    pub max_concurrent_borrows: usize,
    /// Timestamp of the last borrow event
    pub last_borrow_timestamp: Option<u64>,
}

impl BorrowInfo {
    /// Calculate total borrow count (immutable + mutable)
    pub fn total_borrows(&self) -> usize {
        self.immutable_borrows + self.mutable_borrows
    }

    /// Calculate borrow contention ratio (mutable / total)
    pub fn contention_ratio(&self) -> f64 {
        let total = self.total_borrows();
        if total == 0 {
            0.0
        } else {
            self.mutable_borrows as f64 / total as f64
        }
    }
}

/// Enhanced cloning information for allocations
///
/// This structure tracks cloning behavior for reference-counted types (Arc, Rc) and cloneable values,
/// providing insights into data sharing patterns and potential performance optimizations.
///
/// # Fields
///
/// - `clone_count`: Total number of times this allocation was cloned.
///   For `Arc<T>` or `Rc<T>`, this tracks `Arc::clone()` / `Rc::clone()` operations.
///   High clone counts indicate data is heavily shared.
///
/// - `is_clone`: Whether this allocation itself is a result of a clone operation.
///   When `true`, this allocation shares underlying data with another allocation.
///   Useful for tracing the "origin" of shared data.
///
/// - `original_ptr`: Pointer to the original allocation if `is_clone` is `true`.
///   Maintained during clone operations by recording the source pointer.
///   Allows reconstruction of the clone chain/dependency graph.
///
/// # Example
///
/// ```rust
/// use memscope_rs::data::CloneInfo;
///
/// // Original allocation
/// let original_info = CloneInfo {
///     clone_count: 0,
///     is_clone: false,
///     original_ptr: None,
/// };
///
/// // Cloned allocation
/// let cloned_info = CloneInfo {
///     clone_count: 1,
///     is_clone: true,
///     original_ptr: Some(0x1000),  // Points to original
/// };
/// ```
///
/// # Original Pointer Maintenance
///
/// When a clone occurs:
/// 1. The clone's `is_clone` is set to `true`
/// 2. The source's pointer is stored in the clone's `original_ptr`
/// 3. The source's `clone_count` is incremented
/// This enables tracking the complete cloning history and identifying clone cycles.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Number of times this object was cloned
    pub clone_count: usize,
    /// Whether this allocation itself is a result of a clone
    pub is_clone: bool,
    /// If is_clone is true, points to the original object's pointer
    pub original_ptr: Option<usize>,
}

impl CloneInfo {
    /// Create a new CloneInfo for an original allocation
    pub fn new_original() -> Self {
        CloneInfo {
            clone_count: 0,
            is_clone: false,
            original_ptr: None,
        }
    }

    /// Create a CloneInfo for a cloned allocation
    pub fn new_clone(original_ptr: usize) -> Self {
        CloneInfo {
            clone_count: 1,
            is_clone: true,
            original_ptr: Some(original_ptr),
        }
    }
}

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
    /// Enhanced borrowing information
    pub borrow_info: Option<BorrowInfo>,
    /// Enhanced cloning information
    pub clone_info: Option<CloneInfo>,
    /// Whether ownership history is available
    pub ownership_history_available: bool,
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
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
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

    /// Set borrow information
    pub fn with_borrow_info(mut self, borrow_info: BorrowInfo) -> Self {
        self.borrow_info = Some(borrow_info);
        self
    }

    /// Set clone information
    pub fn with_clone_info(mut self, clone_info: CloneInfo) -> Self {
        self.clone_info = Some(clone_info);
        self
    }

    /// Enable ownership history tracking
    pub fn with_ownership_history(mut self) -> Self {
        self.ownership_history_available = true;
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
