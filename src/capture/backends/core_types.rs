//! Core types for memory tracking (self-contained, no old system dependencies)
//!
//! This module defines all types needed by core tracker without
//! depending on crate::core::* modules.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Variable name (if known)
    pub var_name: Option<String>,
    /// Type name (if known)
    pub type_name: Option<String>,
    /// Allocation timestamp (nanoseconds since epoch)
    pub allocated_at_ns: u64,
    /// Thread ID that performed allocation
    pub thread_id: u64,
}

impl AllocationInfo {
    /// Create new allocation info
    #[inline]
    pub fn new(ptr: usize, size: usize) -> Self {
        Self {
            ptr,
            size,
            var_name: None,
            type_name: None,
            allocated_at_ns: Self::now_ns(),
            thread_id: Self::current_thread_id(),
        }
    }

    /// Get current timestamp in nanoseconds
    #[inline]
    pub fn now_ns() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Get current thread ID (hashed)
    #[inline]
    pub fn current_thread_id() -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        hasher.finish()
    }
}

/// Memory statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of allocations
    pub total_allocations: u64,
    /// Total bytes allocated
    pub total_allocated: u64,
    /// Currently active allocations
    pub active_allocations: usize,
    /// Currently active memory (bytes)
    pub active_memory: u64,
    /// Peak allocations count
    pub peak_allocations: usize,
    /// Peak memory usage (bytes)
    pub peak_memory: u64,
    /// Total deallocations
    pub total_deallocations: u64,
    /// Total bytes deallocated
    pub total_deallocated: u64,
    /// Leaked allocations (allocated but not freed)
    pub leaked_allocations: usize,
    /// Leaked memory (bytes)
    pub leaked_memory: u64,
}

/// Type memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMemoryUsage {
    /// Type name
    pub type_name: String,
    /// Total memory usage (bytes)
    pub total_size: usize,
    /// Allocation count
    pub allocation_count: usize,
}

/// Tracking error types
#[derive(Debug, Clone)]
pub enum TrackingError {
    /// Lock acquisition failed
    LockError(String),
    /// Invalid pointer
    InvalidPointer(String),
    /// Export error
    ExportError(String),
    /// Serialization error
    SerializationError(String),
}

impl std::fmt::Display for TrackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockError(msg) => write!(f, "Lock error: {msg}"),
            Self::InvalidPointer(msg) => write!(f, "Invalid pointer: {msg}"),
            Self::ExportError(msg) => write!(f, "Export error: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for TrackingError {}

/// Tracking result type
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Thread registry statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadRegistryStats {
    /// Total threads registered
    pub total_threads_registered: usize,
    /// Active threads
    pub active_threads: usize,
    /// Dead references cleaned up
    pub dead_references: usize,
}
