//! Core types and error handling for the memscope-rs library.

use serde::{Deserialize, Serialize};

/// Error type for memory tracking operations
#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    /// Failed to acquire a lock
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Invalid pointer for association
    #[error("Invalid pointer association: {ptr:?}")]
    InvalidPointer {
        /// The invalid pointer address
        ptr: usize,
    },

    /// Allocation tracking is disabled
    #[error("Allocation tracking disabled")]
    TrackingDisabled,

    /// Memory corruption detected
    #[error("Memory corruption detected")]
    MemoryCorruption,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error during export
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for tracking operations
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Information about a memory allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Timestamp when the allocation occurred (milliseconds since UNIX_EPOCH)
    pub timestamp_alloc: u128,
    /// Timestamp when the deallocation occurred (if applicable)
    pub timestamp_dealloc: Option<u128>,
    /// Optional name of the variable associated with this allocation
    pub var_name: Option<String>,
    /// Optional type name of the variable associated with this allocation
    pub type_name: Option<String>,
    /// Thread ID where the allocation occurred
    pub thread_id: String,
    /// Backtrace information (if available)
    #[cfg(feature = "backtrace")]
    pub backtrace: Option<Vec<String>>,
}

impl AllocationInfo {
    /// Create a new allocation info
    pub fn new(ptr: usize, size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let thread_id = format!("{:?}", std::thread::current().id());

        Self {
            ptr,
            size,
            timestamp_alloc: timestamp,
            timestamp_dealloc: None,
            var_name: None,
            type_name: None,
            thread_id,
            #[cfg(feature = "backtrace")]
            backtrace: None,
        }
    }

    /// Mark this allocation as deallocated
    pub fn mark_deallocated(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        self.timestamp_dealloc = Some(timestamp);
    }

    /// Check if this allocation is still active
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }

    /// Get the lifetime of this allocation in milliseconds
    pub fn lifetime_ms(&self) -> Option<u128> {
        self.timestamp_dealloc
            .map(|dealloc| dealloc - self.timestamp_alloc)
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of allocations tracked
    pub total_allocations: usize,
    /// Total number of deallocations tracked
    pub total_deallocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current number of active allocations
    pub active_allocations: usize,
    /// Current bytes in active allocations
    pub active_memory: usize,
    /// Peak number of active allocations
    pub peak_allocations: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Lifecycle statistics
    pub lifecycle_stats: LifecycleStats,
}

/// Memory usage by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMemoryUsage {
    /// The name of the data type
    pub type_name: String,
    /// Total size in bytes for this type
    pub total_size: usize,
    /// Number of allocations for this type
    pub allocation_count: usize,
}

/// Allocation hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotInfo {
    /// Location identifier (could be function name, file:line, etc.)
    pub location: String,
    /// Number of allocations from this location
    pub count: usize,
    /// Total size of allocations from this location
    pub total_size: usize,
    /// Average allocation size
    pub average_size: f64,
}

/// Lifecycle statistics for memory allocations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleStats {
    /// Number of completed allocations (with deallocation timestamps)
    pub completed_allocations: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Median lifetime in milliseconds
    pub median_lifetime_ms: f64,
    /// Lifecycle percentiles
    pub lifetime_percentiles: LifecyclePercentiles,
    /// Shortest lifetime in milliseconds
    pub min_lifetime_ms: u128,
    /// Longest lifetime in milliseconds
    pub max_lifetime_ms: u128,
    /// Number of instant allocations (< 1ms)
    pub instant_allocations: usize,
    /// Number of short-term allocations (1ms - 100ms)
    pub short_term_allocations: usize,
    /// Number of medium-term allocations (100ms - 1s)
    pub medium_term_allocations: usize,
    /// Number of long-term allocations (> 1s)
    pub long_term_allocations: usize,
    /// Number of suspected memory leaks (active > 10s)
    pub suspected_leaks: usize,
}

/// Lifecycle percentile statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecyclePercentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Lifecycle statistics by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecycleStats {
    /// Type name
    pub type_name: String,
    /// Average lifetime for this type
    pub average_lifetime_ms: f64,
    /// Number of allocations for this type
    pub allocation_count: usize,
    /// Lifecycle category
    pub category: LifecycleCategory,
}

/// Categories for lifecycle duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleCategory {
    /// Very short-lived (< 1ms)
    Instant,
    /// Short-lived (1ms - 100ms)
    ShortTerm,
    /// Medium-lived (100ms - 1s)
    MediumTerm,
    /// Long-lived (> 1s)
    LongTerm,
}
