# Query Module

## Overview

The query module provides a unified interface for querying memory snapshot data. It supports various query types and filtering options, enabling flexible analysis of memory usage patterns.

## Components

### 1. QueryEngine

**File**: `src/query/engine.rs`

**Purpose**: Unified query interface for memory data.

**Key Features**:
- Unified: Single interface for all query types
- Efficient: Optimized for fast query execution
- Flexible: Supports filtering and sorting
- Thread-safe: Safe for concurrent access

**Core Implementation**:

```rust
pub struct QueryEngine {
    /// Reference to the snapshot engine
    snapshot_engine: SharedSnapshotEngine,
}

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
    pub fn allocations_by_thread(&self, thread_id: u64) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
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
        let total_bytes = threads.iter().map(|t| t.total_allocated).sum();

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
    pub fn allocations_by_variable(&self, var_name: &str) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
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
    pub fn allocations_larger_than(&self, min_size: usize) -> QueryResult {
        let snapshot = self.get_snapshot();
        let allocations: Vec<_> = snapshot.active_allocations.values()
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
```

### 2. Query Types

**File**: `src/query/types.rs`

**Purpose**: Type definitions for query results.

**Query Results**:

```rust
pub enum QueryResult {
    /// Allocation query results
    Allocations(AllocationQueryResult),
    /// Thread query results
    Threads(ThreadQueryResult),
    /// Summary query results
    Summary(SummaryQueryResult),
}

pub struct AllocationQueryResult {
    pub count: usize,
    pub total_bytes: usize,
    pub allocations: Vec<ActiveAllocation>,
}

pub struct ThreadQueryResult {
    pub count: usize,
    pub total_bytes: usize,
    pub threads: Vec<ThreadMemoryStats>,
}

pub struct SummaryQueryResult {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub thread_count: usize,
}
```

## Usage Examples

### Basic Usage

```rust
use memscope::query::QueryEngine;
use memscope::snapshot::SnapshotEngine;
use std::sync::Arc;

// Create query engine
let event_store = Arc::new(EventStore::new());
let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
let query_engine = QueryEngine::new(snapshot_engine);

// Query summary
let result = query_engine.summary();
match result {
    QueryResult::Summary(summary) => {
        println!("Total allocations: {}", summary.total_allocations);
        println!("Current memory: {} bytes", summary.current_memory);
    }
    _ => {}
}
```

### Query Top Allocations

```rust
// Get top 10 allocations by size
let result = query_engine.top_allocations(10);
match result {
    QueryResult::Allocations(allocations) => {
        println!("Top allocations:");
        for alloc in &allocations.allocations {
            println!("  0x{:x}: {} bytes", alloc.ptr, alloc.size);
        }
    }
    _ => {}
}
```

### Query by Thread

```rust
// Get allocations from thread 1
let result = query_engine.allocations_by_thread(1);
match result {
    QueryResult::Allocations(allocations) => {
        println!("Thread 1 allocations: {}", allocations.count);
    }
    _ => {}
}
```

### Query Large Allocations

```rust
// Find allocations larger than 1MB
let result = query_engine.allocations_larger_than(1024 * 1024);
match result {
    QueryResult::Allocations(allocations) => {
        println!("Large allocations: {}", allocations.count);
    }
    _ => {}
}
```

## Design Principles

### 1. Unified Interface
Single interface for all query types:
- **Benefits**: Consistent API, easier to use
- **Trade-off**: Less flexibility for specific queries

### 2. Type Safety
Strong typing for query results:
- **Benefits**: Compile-time safety, better documentation
- **Trade-off**: More verbose code

### 3. Efficiency
Optimized for fast query execution:
- **Benefits**: Quick analysis
- **Trade-off**: May use more memory for caching

## Best Practices

1. **Query Optimization**: Use specific queries when possible
2. **Result Handling**: Always handle all query result types
3. **Thread Safety**: Use Arc<QueryEngine> for shared access
4. **Error Handling**: Always handle query errors

## Limitations

1. **Snapshot Based**: Queries operate on snapshots, not live data
2. **Memory Usage**: Query results may be large
3. **Performance**: Complex queries may be slow
4. **Filtering**: Limited filtering capabilities

## Future Improvements

1. **Advanced Filtering**: More powerful filtering options
2. **Aggregation Queries**: Group by, sum, avg, etc.
3. **Time-based Queries**: Query by time ranges
4. **Custom Queries**: Allow custom query logic
5. **Query Caching**: Cache query results