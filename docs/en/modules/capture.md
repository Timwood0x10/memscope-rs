# Capture Module

## Overview

The capture module provides the event capture backend for the memscope-rs library. It is responsible for intercepting memory events from the application and forwarding them to the EventStore. The capture module implements a backend abstraction pattern, allowing different capture strategies to be plugged in without changing the core tracking logic.

## Components

### 1. CaptureEngine

**File**: `src/capture/engine.rs`

**Purpose**: Main event capture engine that coordinates capture backends and forwards events to the EventStore.

**Key Features**:
- Backend abstraction: Supports multiple capture backends
- Non-blocking operations: All capture operations are non-blocking
- Zero-storage: Events are forwarded to EventStore, not stored locally
- Thread-safe: Safe for concurrent use from multiple threads

**Core Implementation**:

```rust
pub struct CaptureEngine {
    /// The capture backend being used
    backend: Box<dyn CaptureBackend>,
    /// Reference to the event store for recording events
    event_store: SharedEventStore,
}

impl CaptureEngine {
    /// Create a new CaptureEngine with the specified backend
    pub fn new(backend_type: CaptureBackendType, event_store: SharedEventStore) -> Self {
        let backend = backend_type.create_backend();
        Self {
            backend,
            event_store,
        }
    }

    /// Capture an allocation event
    pub fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_alloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a deallocation event
    pub fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_dealloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a reallocation event
    pub fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) {
        let event = self.backend.capture_realloc(ptr, old_size, new_size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a move event
    pub fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_move(from_ptr, to_ptr, size, thread_id);
        self.event_store.record(event);
    }
}
```

**Design Philosophy**:

1. **Backend Abstraction**: Separates capture logic from event storage
2. **Non-blocking**: All operations are designed to be fast and non-blocking
3. **Zero-storage**: Does not store events locally, forwards immediately
4. **Pluggable**: Easy to add new capture backends

### 2. Capture Backends

**Files**: `src/capture/backends/`

**Purpose**: Different capture strategies for various use cases.

**Backend Types**:

```rust
pub enum CaptureBackendType {
    /// Core backend using the tracking allocator
    Core,
    /// Lock-free backend for high-performance scenarios
    Lockfree,
    /// Async backend for async applications
    Async,
    /// Unified backend combining all strategies
    Unified,
}
```

**Backend Interface**:

```rust
pub trait CaptureBackend: Send + Sync {
    /// Capture an allocation event
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// Capture a deallocation event
    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;

    /// Capture a reallocation event
    fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> MemoryEvent;

    /// Capture a move event
    fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
}
```

**Backend Implementations**:

1. **CoreBackend**: Uses the tracking allocator for basic capture
2. **LockfreeBackend**: Lock-free implementation for high-concurrency scenarios
3. **AsyncBackend**: Specialized for async applications
4. **UnifiedBackend**: Combines all strategies

### 3. System Monitor

**File**: `src/capture/system_monitor.rs`

**Purpose**: Monitors system resources and provides context for memory events.

**Key Features**:
- CPU usage monitoring
- Memory usage tracking
- Thread activity monitoring
- Resource utilization analysis

**Monitoring Data**:

```rust
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub thread_count: usize,
    pub active_allocations: usize,
    pub peak_memory: usize,
}
```

### 4. Platform Abstraction

**Files**: `src/capture/platform/`

**Purpose**: Platform-specific implementations for different operating systems.

**Supported Platforms**:
- Linux
- macOS
- Windows

**Platform Detection**:

```rust
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;
```

### 5. Type System

**File**: `src/capture/types/`

**Purpose**: Type definitions and data structures for the capture module.

**Key Types**:

```rust
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub scope_name: Option<String>,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    pub thread_id: ThreadId,
    pub thread_id_u64: u64,
    pub borrow_count: u32,
    pub stack_trace: Option<Vec<String>>,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,
    // ... more fields
}

pub enum MemoryOperation {
    Allocation,
    Deallocation,
    Reallocation,
    Move,
    Borrow,
    Return,
}

pub type TrackingResult<T> = Result<T, TrackingError>;
```

## Design Principles

### 1. Backend Abstraction
The capture module uses a backend abstraction pattern:
- **Benefits**: Easy to add new capture strategies
- **Trade-off**: Slight overhead from trait dispatch

### 2. Non-blocking Operations
All capture operations are designed to be non-blocking:
- **Benefits**: No performance impact on application threads
- **Trade-off**: Event processing may be delayed

### 3. Zero-storage
The capture engine does not store events locally:
- **Benefits**: Low memory footprint
- **Trade-off**: Depends on EventStore capacity

### 4. Thread Safety
All operations are thread-safe:
- **Benefits**: Safe for concurrent use
- **Trade-off**: Synchronization overhead

## Usage Examples

### Basic Usage

```rust
use memscope::capture::{CaptureEngine, CaptureBackendType};
use memscope::event_store::EventStore;
use std::sync::Arc;

// Create event store and capture engine
let event_store = Arc::new(EventStore::new());
let capture = Arc::new(CaptureEngine::new(
    CaptureBackendType::Core,
    event_store.clone(),
));

// Capture allocation event
capture.capture_alloc(0x1000, 1024, 1);

// Capture deallocation event
capture.capture_dealloc(0x1000, 1024, 1);
```

### Using Different Backends

```rust
// Lock-free backend for high performance
let capture = CaptureEngine::new(
    CaptureBackendType::Lockfree,
    event_store.clone(),
);

// Async backend for async applications
let capture = CaptureEngine::new(
    CaptureBackendType::Async,
    event_store.clone(),
);

// Unified backend for all scenarios
let capture = CaptureEngine::new(
    CaptureBackendType::Unified,
    event_store.clone(),
);
```

### Custom Backend

```rust
struct MyCustomBackend;

impl CaptureBackend for MyCustomBackend {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::allocate(ptr, size, thread_id)
    }

    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent {
        MemoryEvent::deallocate(ptr, size, thread_id)
    }

    // ... implement other methods
}

// Use custom backend
let capture = CaptureEngine::new(
    CaptureBackendType::Custom(Box::new(MyCustomBackend)),
    event_store.clone(),
);
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
Render Engine (visualizes results)
```

## Performance Considerations

### Backend Selection
Choose the appropriate backend for your use case:
- **Core**: Good for general use cases
- **Lockfree**: Best for high-concurrency scenarios
- **Async**: Best for async applications
- **Unified**: Combines all strategies

### Event Forwarding
Events are forwarded immediately to EventStore:
- **Benefits**: Low latency
- **Trade-off**: Potential contention on EventStore

### Memory Overhead
Capture engine has minimal memory overhead:
- **Benefits**: Low memory footprint
- **Trade-off**: Depends on EventStore capacity

## Testing

The capture module includes comprehensive tests:

```rust
#[test]
fn test_capture_engine_creation() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store);
    assert!(engine.event_store().is_empty());
}

#[test]
fn test_capture_alloc() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
    engine.capture_alloc(0x1000, 1024, 1);
    assert_eq!(event_store.len(), 1);
}

#[test]
fn test_capture_multiple_events() {
    let event_store = Arc::new(EventStore::new());
    let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
    engine.capture_alloc(0x1000, 1024, 1);
    engine.capture_dealloc(0x1000, 1024, 1);
    engine.capture_alloc(0x2000, 2048, 1);
    assert_eq!(event_store.len(), 3);
}
```

## Best Practices

1. **Backend Selection**: Choose the appropriate backend for your use case
2. **Error Handling**: Always check for errors when capturing events
3. **Performance**: Use lock-free backend for high-concurrency scenarios
4. **Testing**: Test capture logic thoroughly before production use

## Limitations

1. **Stack Allocations**: Only heap allocations are tracked
2. **External Memory**: Memory allocated by external libraries may not be tracked
3. **Performance Impact**: Some overhead from event capture
4. **Platform Support**: Some features may be platform-specific

## Future Improvements

1. **Better Stack Traces**: Capture more detailed stack traces
2. **Variable Names**: Capture actual variable names
3. **Type Information**: Provide more accurate type information
4. **Performance**: Further optimize capture overhead
5. **Custom Backends**: Easier custom backend registration