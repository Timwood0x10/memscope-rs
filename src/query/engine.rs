//! Query Engine - Unified query interface
//!
//! This module provides the QueryEngine which is responsible for
//! querying snapshot data.

use crate::query::types::{
    AllocationQueryResult, QueryResult, SummaryQueryResult, ThreadQueryResult,
};
use crate::snapshot::{MemorySnapshot, SharedSnapshotEngine};
use std::sync::Arc;

/// Query Engine - Unified query interface for memory data
///
/// The QueryEngine provides a unified interface for querying memory
/// snapshot data, supporting various query types and filtering options.
///
/// Key properties:
/// - Unified: Single interface for all query types
/// - Efficient: Optimized for fast query execution
/// - Flexible: Supports filtering and sorting
pub struct QueryEngine {
    /// Reference to the snapshot engine
    snapshot_engine: SharedSnapshotEngine,
}

/// Shared reference to QueryEngine
pub type SharedQueryEngine = Arc<QueryEngine>;

impl QueryEngine {
    /// Create a new QueryEngine
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self {
        Self { snapshot_engine }
    }

    /// Get the current snapshot
    fn get_snapshot(&self) -> MemorySnapshot {
        self.snapshot_engine.build_snapshot()
    }

    /// Query for top allocations by size
    ///
    /// # Arguments
    /// * `limit` - Maximum number of allocations to return
    pub fn top_allocations(&self, limit: usize) -> QueryResult {
        let snapshot = self.get_snapshot();
        let mut allocations: Vec<_> = snapshot.active_allocations.values().cloned().collect();

        // Sort by size descending
        allocations.sort_by_key(|b| std::cmp::Reverse(b.size));

        // Limit results
        allocations.truncate(limit);

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// Query for allocations from a specific thread
    ///
    /// # Arguments
    /// * `thread_id` - The thread ID to filter by
    pub fn allocations_by_thread(&self, thread_id: u64) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot
            .active_allocations
            .values()
            .filter(|a| a.thread_id == thread_id)
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// Query for thread statistics
    pub fn thread_stats(&self) -> QueryResult {
        let snapshot = self.get_snapshot();
        let threads: Vec<_> = snapshot.thread_stats.values().cloned().collect();
        let total_bytes = threads.iter().map(|t| t.current_memory).sum();

        QueryResult::Threads(ThreadQueryResult {
            count: threads.len(),
            total_bytes,
            threads,
        })
    }

    /// Query for a summary of memory usage
    pub fn summary(&self) -> QueryResult {
        let snapshot = self.get_snapshot();

        QueryResult::Summary(SummaryQueryResult {
            total_allocations: snapshot.stats.total_allocations,
            total_deallocations: snapshot.stats.total_deallocations,
            active_allocations: snapshot.stats.active_allocations,
            total_allocated: snapshot.stats.total_allocated,
            total_deallocated: snapshot.stats.total_deallocated,
            current_memory: snapshot.stats.current_memory,
            peak_memory: snapshot.stats.peak_memory,
            thread_count: snapshot.thread_stats.len(),
        })
    }

    /// Query for allocations with a specific variable name
    ///
    /// # Arguments
    /// * `var_name` - The variable name to filter by
    pub fn allocations_by_variable(&self, var_name: &str) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot
            .active_allocations
            .values()
            .filter(|a| a.var_name.as_ref().map(|n| n == var_name).unwrap_or(false))
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }

    /// Query for allocations larger than a certain size
    ///
    /// # Arguments
    /// * `min_size` - Minimum allocation size in bytes
    pub fn allocations_larger_than(&self, min_size: usize) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot
            .active_allocations
            .values()
            .filter(|a| a.size > min_size)
            .cloned()
            .collect();

        let total_bytes = allocations.iter().map(|a| a.size).sum();

        QueryResult::Allocations(AllocationQueryResult {
            count: allocations.len(),
            total_bytes,
            allocations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventStore;
    use crate::snapshot::SnapshotEngine;

    /// Objective: Verify that QueryEngine creates correctly with empty data.
    /// Invariants: Summary should show zero allocations for empty event store.
    #[test]
    fn test_query_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.summary();
        match result {
            QueryResult::Summary(summary) => {
                assert_eq!(
                    summary.total_allocations, 0,
                    "Empty event store should have zero allocations"
                );
            }
            _ => panic!("Expected summary result"),
        }
    }

    /// Objective: Verify that top_allocations returns largest allocations sorted by size.
    /// Invariants: Results must be sorted descending by size and limited to requested count.
    #[test]
    fn test_top_allocations() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x2000, 2048, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x3000, 512, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.top_allocations(2);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(allocations.count, 2, "Should return exactly 2 allocations");
                assert_eq!(
                    allocations.allocations[0].size, 2048,
                    "First should be largest"
                );
                assert_eq!(
                    allocations.allocations[1].size, 1024,
                    "Second should be second largest"
                );
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that allocations_by_thread filters correctly by thread ID.
    /// Invariants: Only allocations from specified thread should be returned.
    #[test]
    fn test_allocations_by_thread() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x2000, 2048, 2));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_by_thread(1);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 1,
                    "Should have one allocation for thread 1"
                );
                assert_eq!(allocations.total_bytes, 1024, "Total bytes should be 1024");
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that thread_stats returns correct thread statistics.
    /// Invariants: Thread count should match number of unique threads with allocations.
    #[test]
    fn test_thread_stats() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x2000, 2048, 2));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.thread_stats();
        match result {
            QueryResult::Threads(threads) => {
                assert_eq!(threads.count, 2, "Should have 2 threads with allocations");
            }
            _ => panic!("Expected threads result"),
        }
    }

    /// Objective: Verify that summary correctly tracks allocations and deallocations.
    /// Invariants: Summary must accurately reflect allocation/deallocation counts.
    #[test]
    fn test_summary() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(crate::event_store::MemoryEvent::deallocate(0x1000, 1024, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.summary();
        match result {
            QueryResult::Summary(summary) => {
                assert_eq!(summary.total_allocations, 1, "Should have one allocation");
                assert_eq!(
                    summary.total_deallocations, 1,
                    "Should have one deallocation"
                );
                assert_eq!(
                    summary.active_allocations, 0,
                    "Should have no active allocations"
                );
            }
            _ => panic!("Expected summary result"),
        }
    }

    /// Objective: Verify that allocations_by_variable filters correctly by variable name.
    /// Invariants: Only allocations with matching var_name should be returned.
    #[test]
    fn test_allocations_by_variable() {
        let event_store = Arc::new(EventStore::new());
        let mut event1 = crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1);
        event1.var_name = Some("test_var".to_string());
        event_store.record(event1);

        let mut event2 = crate::event_store::MemoryEvent::allocate(0x2000, 2048, 1);
        event2.var_name = Some("other_var".to_string());
        event_store.record(event2);

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_by_variable("test_var");
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 1,
                    "Should have one allocation with test_var"
                );
                assert_eq!(allocations.total_bytes, 1024, "Total bytes should be 1024");
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that allocations_larger_than filters by size correctly.
    /// Invariants: Only allocations larger than min_size should be returned.
    #[test]
    fn test_allocations_larger_than() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 100, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x2000, 500, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x3000, 1000, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_larger_than(200);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 2,
                    "Should have 2 allocations larger than 200"
                );
                assert_eq!(allocations.total_bytes, 1500, "Total bytes should be 1500");
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that top_allocations with zero limit returns empty result.
    /// Invariants: Zero limit should return empty allocations list.
    #[test]
    fn test_top_allocations_zero_limit() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.top_allocations(0);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 0,
                    "Zero limit should return empty result"
                );
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that allocations_by_thread returns empty for unknown thread.
    /// Invariants: Unknown thread ID should return empty allocations.
    #[test]
    fn test_allocations_by_unknown_thread() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_by_thread(999);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 0,
                    "Unknown thread should have no allocations"
                );
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that allocations_by_variable returns empty for unknown variable.
    /// Invariants: Unknown variable name should return empty allocations.
    #[test]
    fn test_allocations_by_unknown_variable() {
        let event_store = Arc::new(EventStore::new());
        let mut event = crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1);
        event.var_name = Some("known_var".to_string());
        event_store.record(event);

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_by_variable("unknown_var");
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 0,
                    "Unknown variable should have no allocations"
                );
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that allocations_larger_than returns empty when no matches.
    /// Invariants: Very large min_size should return empty allocations.
    #[test]
    fn test_allocations_larger_than_no_matches() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 100, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.allocations_larger_than(10000);
        match result {
            QueryResult::Allocations(allocations) => {
                assert_eq!(
                    allocations.count, 0,
                    "Very large min_size should return empty"
                );
            }
            _ => panic!("Expected allocations result"),
        }
    }

    /// Objective: Verify that summary tracks peak memory correctly.
    /// Invariants: Peak memory should reflect maximum memory usage.
    #[test]
    fn test_summary_peak_memory() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));
        event_store.record(crate::event_store::MemoryEvent::allocate(0x2000, 2048, 1));
        event_store.record(crate::event_store::MemoryEvent::deallocate(0x1000, 1024, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.summary();
        match result {
            QueryResult::Summary(summary) => {
                assert!(
                    summary.peak_memory >= 2048,
                    "Peak memory should be at least 2048"
                );
            }
            _ => panic!("Expected summary result"),
        }
    }
}
