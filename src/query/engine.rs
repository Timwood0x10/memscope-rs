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
        allocations.sort_by(|a, b| b.size.cmp(&a.size));

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

    #[test]
    fn test_query_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let query_engine = QueryEngine::new(snapshot_engine);

        let result = query_engine.summary();
        match result {
            QueryResult::Summary(summary) => {
                assert_eq!(summary.total_allocations, 0);
            }
            _ => panic!("Expected summary result"),
        }
    }

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
                assert_eq!(allocations.count, 2);
                assert_eq!(allocations.allocations[0].size, 2048);
                assert_eq!(allocations.allocations[1].size, 1024);
            }
            _ => panic!("Expected allocations result"),
        }
    }

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
                assert_eq!(allocations.count, 1);
                assert_eq!(allocations.total_bytes, 1024);
            }
            _ => panic!("Expected allocations result"),
        }
    }

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
                assert_eq!(threads.count, 2);
            }
            _ => panic!("Expected threads result"),
        }
    }

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
                assert_eq!(summary.total_allocations, 1);
                assert_eq!(summary.total_deallocations, 1);
                assert_eq!(summary.active_allocations, 0);
            }
            _ => panic!("Expected summary result"),
        }
    }
}
