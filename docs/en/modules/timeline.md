# Timeline Engine Module

## Overview

The timeline engine provides time-based memory analysis and event replay capabilities. It indexes events by timestamp and allows querying events within time ranges.

## Components

### 1. TimelineEngine

**File**: `src/timeline/engine.rs`

**Purpose**: Time-based indexing and querying of memory events.

```rust
pub struct TimelineEngine {
    event_store: SharedEventStore,
    cached_events: RwLock<Vec<MemoryEvent>>,
    cache_version: AtomicU64,
}

impl TimelineEngine {
    pub fn new(event_store: SharedEventStore) -> Self

    pub fn get_events_in_range(&self, start: u64, end: u64) -> Vec<MemoryEvent>

    pub fn get_events_for_pointer(&self, ptr: usize) -> Vec<MemoryEvent>

    pub fn get_events_for_thread(&self, thread_id: u64) -> Vec<MemoryEvent>
}
```

### 2. TimelineIndex

**File**: `src/timeline/index.rs`

**Purpose**: Provides time-based indexing of events for fast range queries.

### 3. TimelineQuery

**File**: `src/timeline/query.rs`

**Purpose**: Structured time-based queries.

### 4. TimelineReplay

**File**: `src/timeline/replay.rs`

**Purpose**: Replays events in chronological order for analysis.

## Key Features

1. **Time-ordered indexing**: Events sorted by timestamp
2. **Range queries**: Fast retrieval of events in time ranges
3. **Caching**: In-memory cache with version tracking
4. **Replay**: Chronological event replay

## Usage Example

```rust
use memscope_rs::timeline::TimelineEngine;

let timeline = TimelineEngine::new(event_store.clone());

// Get events in time range
let events = timeline.get_events_in_range(start_time, end_time);

// Get all events for a specific pointer
let ptr_events = timeline.get_events_for_pointer(0x1000);

// Get all events for a thread
let thread_events = timeline.get_events_for_thread(thread_id);
```

## Design Decisions

1. **Lazy caching**: Cache built on first query
2. **Version tracking**: Detects when cache is stale
3. **Binary search**: Fast range queries using partition_point

## Limitations

1. **Single-threaded indexing**: Index building is not concurrent
2. **In-memory only**: No disk persistence
3. **Full snapshot required**: Needs entire event history
