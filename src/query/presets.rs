//! Query Presets - Predefined common queries
//!
//! This module provides preset queries for common use cases,
//! making it easy to get standard information without writing
//! custom queries.

use super::types::{Query, QueryResult};

/// Preset query factory
pub struct QueryPresets;

impl QueryPresets {
    /// Create a query to find top allocations by size
    pub fn top_allocations_by_size(limit: usize) -> Query {
        Query::TopAllocationsBySize { limit }
    }

    /// Create a query to find top allocations by count
    pub fn top_allocations_by_count(limit: usize) -> Query {
        Query::TopAllocationsByCount { limit }
    }

    /// Create a query to find memory leaks
    pub fn memory_leaks(min_age_ms: u64) -> Query {
        Query::MemoryLeaks { min_age_ms }
    }

    /// Create a query to find large allocations
    pub fn large_allocations(min_size: usize) -> Query {
        Query::LargeAllocations { min_size }
    }

    /// Create a query to get thread memory statistics
    pub fn thread_memory_stats(thread_id: u64) -> Query {
        Query::ThreadMemoryStats { thread_id }
    }

    /// Create a query to get scope memory statistics
    pub fn scope_memory_stats(scope_name: String) -> Query {
        Query::ScopeMemoryStats { scope_name }
    }

    /// Create a query to get type memory statistics
    pub fn type_memory_stats(type_name: String) -> Query {
        Query::TypeMemoryStats { type_name }
    }

    /// Create a query to get system-wide summary
    pub fn system_summary() -> Query {
        Query::Summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_allocations_by_size() {
        let query = QueryPresets::top_allocations_by_size(10);
        assert!(matches!(query, Query::TopAllocationsBySize { limit: 10 }));
    }

    #[test]
    fn test_memory_leaks() {
        let query = QueryPresets::memory_leaks(1000);
        assert!(matches!(query, Query::MemoryLeaks { min_age_ms: 1000 }));
    }

    #[test]
    fn test_large_allocations() {
        let query = QueryPresets::large_allocations(1024);
        assert!(matches!(query, Query::LargeAllocations { min_size: 1024 }));
    }
}