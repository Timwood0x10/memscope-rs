# Snapshot Module

## Overview

The snapshot module provides memory snapshot construction and aggregation capabilities. It builds point-in-time views of memory usage from events stored in the EventStore, enabling analysis of current memory state and patterns.

## Components

### 1. SnapshotEngine

**File**: `src/snapshot/engine.rs`

**Purpose**: Builds memory snapshots from event data.

**Key Features**:
- Read-only: Does not consume events from EventStore
- Efficient: Optimized for fast snapshot construction
- Comprehensive: Captures all relevant memory state
- Thread-safe: Safe for concurrent access

**Core Implementation**:

```rust
pub struct SnapshotEngine {
    /// Reference to the event store
    event_store: SharedEventStore,
}

impl SnapshotEngine {
    /// Build a snapshot from the current event store state
    pub fn build_snapshot(&self) -> MemorySnapshot {
        let events = self.event_store.snapshot();
        self.build_snapshot_from_events(events)
    }

    /// Build a snapshot from a specific set of events
    pub fn build_snapshot_from_events(&self, events: Vec<MemoryEvent>) -> MemorySnapshot {
        let mut snapshot = MemorySnapshot::new();
        let mut ptr_to_allocation: HashMap<usize, ActiveAllocation> = HashMap::new();
        let mut thread_stats: HashMap<u64, ThreadMemoryStats> = HashMap::new();
        let mut peak_memory: usize = 0;
        let mut current_memory: usize = 0;

        for event in events {
            match event.event_type {
                MemoryEventType::Allocate | MemoryEventType::Reallocate => {
                    // Record allocation
                    let allocation = ActiveAllocation {
                        ptr: event.ptr,
                        size: event.size,
                        allocated_at: event.timestamp,
                        var_name: event.var_name,
                        type_name: event.type_name,
                        thread_id: event.thread_id,
                    };

                    ptr_to_allocation.insert(event.ptr, allocation);

                    // Update stats
                    snapshot.stats.total_allocations += 1;
                    snapshot.stats.total_allocated += event.size;
                    current_memory += event.size;

                    // Update thread stats
                    let thread_stat = thread_stats.entry(event.thread_id).or_insert_with(
                        || ThreadMemoryStats {
                            thread_id: event.thread_id,
                            allocation_count: 0,
                            total_allocated: 0,
                            current_memory: 0,
                            peak_memory: 0,
                        }
                    );
                    thread_stat.allocation_count += 1;
                    thread_stat.total_allocated += event.size;
                    thread_stat.current_memory += event.size;
                    if thread_stat.current_memory > thread_stat.peak_memory {
                        thread_stat.peak_memory = thread_stat.current_memory;
                    }
                }
                MemoryEventType::Deallocate => {
                    // Remove allocation
                    if let Some(allocation) = ptr_to_allocation.remove(&event.ptr) {
                        snapshot.stats.total_deallocations += 1;
                        snapshot.stats.total_deallocated += allocation.size;
                        current_memory -= allocation.size;

                        // Update thread stats
                        if let Some(thread_stat) = thread_stats.get_mut(&event.thread_id) {
                            thread_stat.current_memory -= allocation.size;
                        }
                    } else {
                        snapshot.stats.unmatched_deallocations += 1;
                    }
                }
                _ => {}
            }

            // Update peak memory
            if current_memory > peak_memory {
                peak_memory = current_memory;
            }
        }

        // Build final snapshot
        snapshot.active_allocations = ptr_to_allocation;
        snapshot.thread_stats = thread_stats;
        snapshot.stats.active_allocations = snapshot.active_allocations.len();
        snapshot.stats.current_memory = current_memory;
        snapshot.stats.peak_memory = peak_memory;

        snapshot
    }
}
```

### 2. MemorySnapshot

**File**: `src/snapshot/types.rs`

**Purpose**: Represents a point-in-time view of memory state.

**Snapshot Structure**:

```rust
pub struct MemorySnapshot {
    /// Active allocations
    pub active_allocations: HashMap<usize, ActiveAllocation>,
    /// Thread-specific memory statistics
    pub thread_stats: HashMap<u64, ThreadMemoryStats>,
    /// Overall statistics
    pub stats: SnapshotStats,
}

pub struct SnapshotStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub unmatched_deallocations: usize,
}
```

## Usage Examples

### Basic Usage

```rust
use memscope::snapshot::{SnapshotEngine, MemorySnapshot};
use memscope::event_store::EventStore;
use std::sync::Arc;

// Create snapshot engine
let event_store = Arc::new(EventStore::new());
let engine = SnapshotEngine::new(event_store);

// Build snapshot
let snapshot = engine.build_snapshot();

// Access snapshot data
println!("Active allocations: {}", snapshot.active_count());
println!("Current memory: {} bytes", snapshot.current_memory());
println!("Peak memory: {} bytes", snapshot.peak_memory());
```

### Snapshot with Events

```rust
// Add some events
event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
event_store.record(MemoryEvent::allocate(0x2000, 2048, 1));
event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

// Build snapshot
let snapshot = engine.build_snapshot();

// Check results
assert_eq!(snapshot.active_count(), 1);
assert_eq!(snapshot.current_memory(), 2048);
```

## Design Principles

### 1. Read-only Operations
Snapshots do not modify event data:
- **Benefits**: Safe, reproducible analysis
- **Trade-off**: Must copy data for snapshot

### 2. Efficient Construction
Optimized for fast snapshot building:
- **Benefits**: Quick analysis
- **Trade-off**: May use more memory during construction

### 3. Thread Safety
Safe for concurrent access:
- **Benefits**: Multi-threaded analysis
- **Trade-off**: Synchronization overhead

## Best Practices

1. **Snapshot Frequency**: Balance between accuracy and performance
2. **Memory Management**: Clear old snapshots to free memory
3. **Thread Safety**: Use Arc<SnapshotEngine> for shared access
4. **Error Handling**: Always handle snapshot construction errors

## Limitations

1. **Point-in-time**: Only shows state at construction time
2. **Memory Usage**: Snapshots consume memory
3. **Construction Time**: Large datasets may take time to build
4. **Event Order**: Depends on correct event ordering

## Future Improvements

1. **Incremental Snapshots**: Update existing snapshots incrementally
2. **Differential Snapshots**: Compare snapshots to find changes
3. **Compression**: Compress snapshot data
4. **Persistence**: Save snapshots to disk
5. **Time Travel**: Reconstruct past states from events