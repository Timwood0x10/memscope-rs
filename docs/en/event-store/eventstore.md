# EventStore

> The central event bus — lock-free event storage and snapshot mechanism

---

## Overview

**File:** `src/event_store/store.rs`

The EventStore is the **central event bus** where all capture backends push `MemoryEvent` records. It uses a lock-free `SegQueue` from crossbeam for high-throughput event ingestion and provides a snapshot mechanism for point-in-time analysis.

---

## Core Data Structures

### EventStore

```rust
// store.rs:23-28
pub struct EventStore {
    events: SegQueue<MemoryEvent>,   // Lock-free MPMC queue
    snapshot_lock: Mutex<()>,        // Protects drain-and-restore during snapshot
}
```

- **`SegQueue<MemoryEvent>`** — A lock-free multi-producer, multi-consumer queue from the `crossbeam` crate. Multiple threads can push events concurrently without any locks.
- **`snapshot_lock`** — A mutex that serializes concurrent snapshot operations. Only one snapshot can run at a time.

### MemoryEvent

```rust
// event_store/event.rs
pub struct MemoryEvent {
    pub timestamp: u64,
    pub event_type: MemoryEventType,
    pub ptr: usize,
    pub size: usize,
    pub thread_id: u64,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub call_stack_hash: Option<u64>,
    pub thread_name: Option<String>,
}

pub enum MemoryEventType {
    Allocate,
    Deallocate,
    Reallocate { old_size: usize },
    Move,
}
```

Each event captures a single memory operation with its timestamp, type, pointer, size, and optional metadata (variable name, type name, call stack hash, thread name).

---

## Event Recording

```rust
// store.rs:46-48
pub fn record(&self, event: MemoryEvent) {
    self.events.push(event);  // Lock-free, O(1)
}
```

**How it works:** `SegQueue::push()` is a lock-free operation. Multiple threads can call `record()` simultaneously without blocking each other. The event is appended to the queue's internal segment list.

**Performance:** O(1) amortized, no locks, no atomics on the hot path (beyond the queue's internal CAS).

---

## Snapshot Mechanism

```rust
// store.rs:62-74
pub fn snapshot(&self) -> Vec<MemoryEvent> {
    let _guard = self.snapshot_lock.lock().unwrap();
    let mut events = Vec::new();

    // Drain all events from the queue
    while let Some(event) = self.events.pop() {
        events.push(event);
    }

    // Restore them so future snapshots still see them
    for event in &events {
        self.events.push(event.clone());
    }

    events
}
```

**How it works:**

1. Acquire `snapshot_lock` — ensures only one snapshot at a time
2. Drain all events from the `SegQueue` into a `Vec`
3. Push all events back into the queue so they're available for future snapshots
4. Return the `Vec` to the caller

**Why drain-and-restore?** `SegQueue` doesn't support iteration without consuming. To get a point-in-time view, we must drain the queue and restore the events.

**Known issue:** During the drain-and-restore window, concurrent `record()` calls may push events that get interleaved with restored events in an unpredictable order. The `snapshot_lock` only protects against concurrent snapshots, not concurrent `record()` calls.

---

## Length Query

```rust
// store.rs:80-82
pub fn len(&self) -> usize {
    self.events.len()
}
```

Returns the current number of events in the queue. This is a concurrent read — it may be stale by the time the caller uses it.

---

## Event Creation Helpers

```rust
// event_store/event.rs
impl MemoryEvent {
    pub fn allocate(ptr: usize, size: usize, thread_id: u64) -> Self { ... }
    pub fn deallocate(ptr: usize, size: usize, thread_id: u64) -> Self { ... }
    pub fn reallocate(ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> Self { ... }
    pub fn move_event(from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> Self { ... }

    pub fn with_var_name(mut self, name: String) -> Self { ... }
    pub fn with_type_name(mut self, name: String) -> Self { ... }
    pub fn with_call_stack_hash(mut self, hash: u64) -> Self { ... }
    pub fn with_thread_name(mut self, name: String) -> Self { ... }

    pub fn is_allocation(&self) -> bool { ... }
    pub fn is_deallocation(&self) -> bool { ... }
}
```

Builder-pattern helpers for constructing events with optional metadata.

---

## Timestamp Generation

```rust
// event_store/event.rs:112-118
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()  // BUG: panics if clock is before 1970
        .as_nanos() as u64
}
```

**Known issue:** Uses `.unwrap()` instead of `.unwrap_or_default()`. If the system clock is set before the Unix epoch, this will panic.

---

## Performance Characteristics

| Operation | Complexity | Locks | Notes |
|-----------|------------|-------|-------|
| `record(event)` | O(1) | None (lock-free) | Crossbeam SegQueue push |
| `snapshot()` | O(n) | `snapshot_lock` | Drain-and-restore all events |
| `len()` | O(1) | None | May be stale |

**Memory cost per event:** ~80-100 bytes for `MemoryEvent` + ~32 bytes for SegQueue node overhead.

---

## Usage Pattern

```rust
// 1. Capture backends push events
let event = MemoryEvent::allocate(ptr, size, thread_id);
event_store.record(event);

// 2. Analysis engines take snapshots
let events = event_store.snapshot();

// 3. Process events
for event in &events {
    if event.is_allocation() {
        // Process allocation
    }
}
```

---

## Integration with Other Modules

| Module | How it uses EventStore |
|--------|----------------------|
| **CaptureEngine** | Calls `record()` for every allocation/deallocation |
| **SnapshotEngine** | Calls `snapshot()` to build point-in-time views |
| **TimelineEngine** | Calls `snapshot()` to get events for time-series analysis |
| **QueryEngine** | Uses snapshots to answer queries |
