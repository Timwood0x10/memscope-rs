# Event Store Module

## Overview

The event store module provides centralized storage for all memory events in the memscope-rs system. It is the single source of truth for memory tracking data, using a lock-free design for high-concurrency recording and efficient snapshot generation for analysis.

## Components

### 1. MemoryEvent

**File**: `src/event_store/event.rs`

**Purpose**: Unified memory event type used across all tracking backends.

**Key Features**:
- Unified event type for all memory operations
- Comprehensive metadata (timestamp, thread info, call stack)
- Serializable for persistence and analysis
- Type-safe event classification

**Event Types**:

```rust
pub enum MemoryEventType {
    /// Memory allocation event
    Allocate,
    /// Memory deallocation event
    Deallocate,
    /// Memory reallocation event
    Reallocate,
    /// Memory move event
    Move,
    /// Memory borrow event
    Borrow,
    /// Memory return event
    Return,
}
```

**Event Structure**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Event timestamp (nanoseconds since epoch)
    pub timestamp: u64,
    /// Event type
    pub event_type: MemoryEventType,
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Thread identifier
    pub thread_id: u64,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Optional call stack hash
    pub call_stack_hash: Option<u64>,
    /// Optional thread name
    pub thread_name: Option<String>,
}
```

**Event Creation Methods**:

```rust
impl MemoryEvent {
    /// Create a new allocation event
    pub fn allocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Allocate,
            ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Create a new deallocation event
    pub fn deallocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Deallocate,
            ptr,
            size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Create a new reallocation event
    pub fn reallocate(ptr: usize, _old_size: usize, new_size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Reallocate,
            ptr,
            size: new_size,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Set variable name
    pub fn with_var_name(mut self, name: String) -> Self {
        self.var_name = Some(name);
        self
    }

    /// Set type name
    pub fn with_type_name(mut self, name: String) -> Self {
        self.type_name = Some(name);
        self
    }

    /// Set call stack hash
    pub fn with_call_stack_hash(mut self, hash: u64) -> Self {
        self.call_stack_hash = Some(hash);
        self
    }

    /// Set thread name
    pub fn with_thread_name(mut self, name: String) -> Self {
        self.thread_name = Some(name);
        self
    }

    /// Check if this is an allocation event
    pub fn is_allocation(&self) -> bool {
        matches!(
            self.event_type,
            MemoryEventType::Allocate | MemoryEventType::Reallocate
        )
    }

    /// Check if this is a deallocation event
    pub fn is_deallocation(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Deallocate)
    }

    /// Check if this is a move event
    pub fn is_move(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Move)
    }

    /// Check if this is a borrow event
    pub fn is_borrow(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Borrow)
    }

    /// Check if this is a return event
    pub fn is_return(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Return)
    }
}
```

### 2. EventStore

**File**: `src/event_store/store.rs`

**Purpose**: Centralized storage for memory events with lock-free recording and efficient snapshots.

**Key Features**:
- Lock-free recording: Uses SegQueue for O(1) append without blocking
- Thread-safe: All operations are safe for concurrent use
- Efficient snapshots: Uses RwLock for fast read access
- Atomic counting: Approximate event count without locking

**Core Implementation**:

```rust
#[derive(Debug)]
pub struct EventStore {
    /// Lock-free queue for high-concurrency recording
    queue: SegQueue<MemoryEvent>,
    /// Cached events for fast snapshot access
    cache: RwLock<Vec<MemoryEvent>>,
    /// Approximate count of events (may be slightly stale)
    count: AtomicUsize,
}

impl EventStore {
    /// Create a new EventStore
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            cache: RwLock::new(Vec::new()),
            count: AtomicUsize::new(0),
        }
    }

    /// Record a memory event
    ///
    /// This method is lock-free and can be called from any thread
    /// without blocking other recording operations.
    pub fn record(&self, event: MemoryEvent) {
        self.queue.push(event);
        // Use Release ordering to ensure the push is visible before the count increment
        self.count.fetch_add(1, Ordering::Release);
    }

    /// Flush pending events from queue to cache
    fn flush_to_cache(&self) {
        let mut cache = self.cache.write();
        while let Some(event) = self.queue.pop() {
            cache.push(event);
        }
    }

    /// Get all events as a snapshot
    ///
    /// Returns a snapshot of all events currently in the store.
    /// This method flushes any pending events from the lock-free queue
    /// to the cache before returning.
    pub fn snapshot(&self) -> Vec<MemoryEvent> {
        self.flush_to_cache();
        self.cache.read().clone()
    }

    /// Get the number of events in the store
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all events from the store
    pub fn clear(&self) {
        // Acquire write lock first to prevent concurrent modifications
        let mut cache = self.cache.write();

        // Clear the queue
        while self.queue.pop().is_some() {}

        // Clear the cache
        cache.clear();

        // Reset count last, while still holding the write lock
        self.count.store(0, Ordering::Release);
    }
}

/// Shared reference to EventStore
pub type SharedEventStore = Arc<EventStore>;
```

**Design Philosophy**:

1. **Lock-free Recording**: Uses SegQueue for O(1) append operations
2. **Efficient Snapshots**: Flushed to cache for fast read access
3. **Atomic Counting**: Approximate count without locking overhead
4. **Thread Safety**: All operations are safe for concurrent use

## Design Principles

### 1. Lock-free Recording
The EventStore uses a lock-free queue for recording:
- **Benefits**: No blocking, high concurrency support
- **Trade-off**: Events in queue may not be immediately visible in snapshots

### 2. Dual Storage
Uses both queue and cache:
- **Queue**: Lock-free, optimized for writes
- **Cache**: RwLock-protected, optimized for reads
- **Benefits**: Optimized for both write and read operations

### 3. Atomic Counting
Uses atomic operations for counting:
- **Benefits**: Fast, no locking overhead
- **Trade-off**: Count may be slightly stale

### 4. Snapshot Isolation
Snapshots are isolated from ongoing recording:
- **Benefits**: Consistent view of events
- **Trade-off**: Snapshot creation may take time for large datasets

## Usage Examples

### Basic Usage

```rust
use memscope::event_store::{EventStore, MemoryEvent};
use std::sync::Arc;

// Create event store
let event_store = Arc::new(EventStore::new());

// Record events
event_store.record(MemoryEvent::allocate(0x1000, 1024, 1));
event_store.record(MemoryEvent::deallocate(0x1000, 1024, 1));

// Get snapshot
let events = event_store.snapshot();
println!("Total events: {}", events.len());

// Clear store
event_store.clear();
```

### Concurrent Recording

```rust
use std::thread;
use std::sync::Arc;

let event_store = Arc::new(EventStore::new());
let mut handles = vec![];

// Multiple threads recording events
for i in 0..10 {
    let store = Arc::clone(&event_store);
    let handle = thread::spawn(move || {
        for j in 0..100 {
            let event = MemoryEvent::allocate(i * 1000 + j, 1024, i as u64);
            store.record(event);
        }
    });
    handles.push(handle);
}

// Wait for all threads
for handle in handles {
    handle.join().unwrap();
}

// Get all events
let events = event_store.snapshot();
println!("Total events: {}", events.len());
```

### Event with Metadata

```rust
let event = MemoryEvent::allocate(0x1000, 1024, 1)
    .with_var_name("my_variable".to_string())
    .with_type_name("Vec<u8>".to_string())
    .with_thread_name("main".to_string())
    .with_call_stack_hash(0x12345678);

event_store.record(event);
```

## Integration with Other Modules

```
Capture Engine
    ↓
Event Store (records events)
    ↓
Snapshot Engine (builds snapshots from events)
    ↓
Query Engine (queries snapshot data)
    ↓
Analysis Engine (analyzes memory patterns)
    ↓
Timeline Engine (time-based analysis)
```

## Performance Considerations

### Lock-free Recording
Recording is lock-free and highly concurrent:
- **Benefits**: High throughput, no blocking
- **Trade-off**: Events may not be immediately visible

### Snapshot Creation
Snapshot creation flushes queue to cache:
- **Benefits**: Consistent view of all events
- **Trade-off**: May be slow for large datasets

### Memory Usage
Stores all events until cleared:
- **Benefits**: Complete event history
- **Trade-off**: Memory grows with event count

### Atomic Counting
Count is approximate and may be stale:
- **Benefits**: Fast, no locking
- **Trade-off**: May not reflect exact count

## Testing

The event store module includes comprehensive tests:

```rust
#[test]
fn test_event_store_creation() {
    let store = EventStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_record_event() {
    let store = EventStore::new();
    let event = MemoryEvent::allocate(0x1000, 1024, 1);
    store.record(event);
    assert_eq!(store.len(), 1);
}

#[test]
fn test_snapshot() {
    let store = EventStore::new();
    let event1 = MemoryEvent::allocate(0x1000, 1024, 1);
    let event2 = MemoryEvent::deallocate(0x1000, 1024, 1);
    store.record(event1.clone());
    store.record(event2.clone());

    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 2);
    // Verify events are still in store after snapshot
    assert_eq!(store.len(), 2);
}

#[test]
fn test_clear() {
    let store = EventStore::new();
    let event = MemoryEvent::allocate(0x1000, 1024, 1);
    store.record(event);
    assert_eq!(store.len(), 1);

    store.clear();
    assert!(store.is_empty());
}

#[test]
fn test_concurrent_access() {
    use std::thread;
    let store = Arc::new(EventStore::new());
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let event = MemoryEvent::allocate(i * 1000 + j, 1024, i as u64);
                store_clone.record(event);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(store.len(), 1000);
    let snapshot = store.snapshot();
    assert_eq!(snapshot.len(), 1000);
}
```

## Best Practices

1. **Event Recording**: Use lock-free recording for high concurrency
2. **Snapshot Management**: Clear store periodically to free memory
3. **Error Handling**: Always handle potential serialization errors
4. **Thread Safety**: Use Arc<EventStore> for shared access

## Limitations

1. **Memory Growth**: Store grows until cleared
2. **Approximate Count**: Count may be slightly stale
3. **Snapshot Latency**: Events in queue may not be in snapshot immediately
4. **Race Condition**: Clear and record may have race conditions

## Future Improvements

1. **Event Sampling**: Add event sampling to reduce memory usage
2. **Time-based Filtering**: Filter events by time range
3. **Compression**: Compress stored events
4. **Persistence**: Add disk persistence for long-term storage
5. **Event Filtering**: Filter events by type or metadata