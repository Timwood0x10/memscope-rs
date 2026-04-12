//! Query Presets - Predefined common queries
//!
//! This module provides preset queries for common use cases,
//! making it easy to get standard information without writing
//! custom queries.

use super::types::Query;

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

    /// Objective: Verify that top_allocations_by_size creates correct query with limit.
    /// Invariants: Query variant must be TopAllocationsBySize with exact limit value.
    #[test]
    fn test_top_allocations_by_size() {
        let query = QueryPresets::top_allocations_by_size(10);
        assert!(
            matches!(query, Query::TopAllocationsBySize { limit: 10 }),
            "Query should be TopAllocationsBySize with limit 10"
        );
    }

    /// Objective: Verify that top_allocations_by_count creates correct query.
    /// Invariants: Query variant must be TopAllocationsByCount with exact limit value.
    #[test]
    fn test_top_allocations_by_count() {
        let query = QueryPresets::top_allocations_by_count(25);
        assert!(
            matches!(query, Query::TopAllocationsByCount { limit: 25 }),
            "Query should be TopAllocationsByCount with limit 25"
        );
    }

    /// Objective: Verify that memory_leaks creates correct query with age threshold.
    /// Invariants: Query variant must be MemoryLeaks with exact min_age_ms value.
    #[test]
    fn test_memory_leaks() {
        let query = QueryPresets::memory_leaks(1000);
        assert!(
            matches!(query, Query::MemoryLeaks { min_age_ms: 1000 }),
            "Query should be MemoryLeaks with min_age_ms 1000"
        );
    }

    /// Objective: Verify that large_allocations creates correct query with size threshold.
    /// Invariants: Query variant must be LargeAllocations with exact min_size value.
    #[test]
    fn test_large_allocations() {
        let query = QueryPresets::large_allocations(1024);
        assert!(
            matches!(query, Query::LargeAllocations { min_size: 1024 }),
            "Query should be LargeAllocations with min_size 1024"
        );
    }

    /// Objective: Verify that thread_memory_stats creates correct query with thread ID.
    /// Invariants: Query variant must be ThreadMemoryStats with exact thread_id value.
    #[test]
    fn test_thread_memory_stats() {
        let query = QueryPresets::thread_memory_stats(42);
        assert!(
            matches!(query, Query::ThreadMemoryStats { thread_id: 42 }),
            "Query should be ThreadMemoryStats with thread_id 42"
        );
    }

    /// Objective: Verify that scope_memory_stats creates correct query with scope name.
    /// Invariants: Query variant must be ScopeMemoryStats with exact scope_name value.
    #[test]
    fn test_scope_memory_stats() {
        let query = QueryPresets::scope_memory_stats("main".to_string());
        match query {
            Query::ScopeMemoryStats { scope_name } => {
                assert_eq!(scope_name, "main", "Scope name should be 'main'");
            }
            _ => panic!("Expected ScopeMemoryStats query"),
        }
    }

    /// Objective: Verify that type_memory_stats creates correct query with type name.
    /// Invariants: Query variant must be TypeMemoryStats with exact type_name value.
    #[test]
    fn test_type_memory_stats() {
        let query = QueryPresets::type_memory_stats("Vec<u8>".to_string());
        match query {
            Query::TypeMemoryStats { type_name } => {
                assert_eq!(type_name, "Vec<u8>", "Type name should be 'Vec<u8>'");
            }
            _ => panic!("Expected TypeMemoryStats query"),
        }
    }

    /// Objective: Verify that system_summary creates correct summary query.
    /// Invariants: Query variant must be Summary.
    #[test]
    fn test_system_summary() {
        let query = QueryPresets::system_summary();
        assert!(
            matches!(query, Query::Summary),
            "Query should be Summary variant"
        );
    }

    /// Objective: Verify edge case with zero limit for allocations query.
    /// Invariants: Query should still be created with zero limit.
    #[test]
    fn test_top_allocations_zero_limit() {
        let query = QueryPresets::top_allocations_by_size(0);
        assert!(
            matches!(query, Query::TopAllocationsBySize { limit: 0 }),
            "Query should accept zero limit"
        );
    }

    /// Objective: Verify edge case with large thread ID.
    /// Invariants: Query should handle large u64 values correctly.
    #[test]
    fn test_thread_memory_stats_large_id() {
        let large_id = u64::MAX;
        let query = QueryPresets::thread_memory_stats(large_id);
        assert!(
            matches!(query, Query::ThreadMemoryStats { thread_id } if thread_id == large_id),
            "Query should handle max u64 thread ID"
        );
    }
}
