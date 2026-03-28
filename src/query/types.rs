//! Query types - Data structures for query results
//!
//! This module defines the data structures used for query results.

use crate::snapshot::types::{ActiveAllocation, ThreadMemoryStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResult {
    /// Allocation query result
    Allocations(AllocationQueryResult),
    /// Thread query result
    Threads(ThreadQueryResult),
    /// Summary query result
    Summary(SummaryQueryResult),
    /// Empty result
    Empty,
}

/// Result of an allocation query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationQueryResult {
    /// Number of allocations in the result
    pub count: usize,
    /// Total bytes
    pub total_bytes: usize,
    /// The allocations
    pub allocations: Vec<ActiveAllocation>,
}

/// Result of a thread query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadQueryResult {
    /// Number of threads in the result
    pub count: usize,
    /// Total bytes across all threads
    pub total_bytes: usize,
    /// Thread statistics
    pub threads: Vec<ThreadMemoryStats>,
}

/// Result of a summary query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryQueryResult {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current memory usage
    pub current_memory: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Number of threads
    pub thread_count: usize,
}