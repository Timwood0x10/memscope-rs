# Capture Backends

> How MemScope captures memory allocation events — four strategies, one trait

---

## The CaptureBackend Trait

**File:** `src/capture/backends/mod.rs:133-157`

All capture backends implement this unified trait:

```rust
pub trait CaptureBackend: Send + Sync {
    fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> MemoryEvent;
    fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) -> MemoryEvent;
}
```

Each method receives raw allocation parameters and returns a `MemoryEvent` that gets pushed to the `EventStore`. The trait is `Send + Sync` so backends can be shared across threads.

---

## Backend 1: Core

**File:** `src/capture/backends/core_tracker.rs`

### Design

The Core backend is the **feature-rich, original tracker**. It stores every allocation in a concurrent `DashMap` and maintains atomic counters for real-time statistics.

### Core Data Structures

```rust
// core_tracker.rs:55-65
pub struct MemoryTracker {
    active_allocations: DashMap<usize, AllocationInfo>,  // ptr → full info
    total_allocations: AtomicU64,
    total_allocated: AtomicU64,
    total_deallocations: AtomicU64,
    total_deallocated: AtomicU64,
    peak_memory: AtomicU64,
    peak_allocations: AtomicUsize,
}
```

- **`DashMap<usize, AllocationInfo>`** — A sharded concurrent HashMap. Each shard has its own read-write lock, providing much better concurrent read performance than a single `Mutex<HashMap>`.
- **`AtomicU64` counters** — All statistics updated lock-free with `fetch_add`.
- **`AtomicUsize` for `peak_allocations`** — Tracks maximum simultaneous active allocations.

### Allocation Flow (Source-Level)

```rust
// core_tracker.rs:92-120
pub fn track_allocation(&self, ptr: usize, size: usize, ...) -> TrackingResult<()> {
    // 1. Build AllocationInfo with 25+ fields
    let alloc_info = AllocationInfo {
        ptr, size,
        var_name, type_name,
        thread_id: thread::current().id(),
        allocated_at_ns: timestamp_ns(),
        stack_trace, borrow_info, clone_info, ...
    };

    // 2. Insert into DashMap — lock-free per-shard
    self.active_allocations.insert(ptr, alloc_info);

    // 3. Update atomic counters — no locks
    self.total_allocations.fetch_add(1, Ordering::Relaxed);
    self.total_allocated.fetch_add(size as u64, Ordering::Relaxed);

    // 4. Update peak — TOCTOU race (known issue)
    let current = self.active_allocations.len();
    let peak = self.peak_allocations.load(Ordering::Relaxed);
    if current > peak {
        self.peak_allocations.store(current, Ordering::Relaxed);
    }
    Ok(())
}
```

### Deallocation Flow

```rust
// core_tracker.rs:126-136
pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<()> {
    if let Some((_, allocation)) = self.active_allocations.remove(&ptr) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated.fetch_add(allocation.size as u64, Ordering::Relaxed);
    }
    Ok(())  // Silently ignores unknown pointers
}
```

### Threading Model

Two modes (`core_tracker.rs:14-32`):

```rust
// Mode 1: Global singleton
static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

// Mode 2: Thread-local registry
static LOCAL_THREAD_REGISTRY: OnceLock<DashMap<ThreadId, Arc<MemoryTracker>>> = OnceLock::new();
```

### Performance

| Metric | Value |
|--------|-------|
| Per-allocation memory | ~80 bytes (AllocationInfo) + ~32 bytes (DashMap node) |
| Tracking overhead | 5-10% |
| Query latency | O(1) for stats, O(n) for iteration |
| Thread safety | DashMap sharded locks — good for read-heavy workloads |

### When to Use

Single-threaded or low-concurrency programs (< 8 threads) where you need **rich type metadata** per allocation.

---

## Backend 2: Lockfree

**File:** `src/capture/backends/lockfree_tracker.rs`

### Design

The Lockfree backend is designed for **high-concurrency production environments**. It uses thread-local storage to avoid cross-thread synchronization entirely during the hot path, with probabilistic sampling to bound memory usage.

### Core Data Structures

```rust
// lockfree_tracker.rs:28-45
pub struct ThreadLocalTracker {
    events: Arc<Mutex<Vec<Event>>>,                    // Shared event buffer
    active_allocations: Arc<Mutex<HashMap<usize, usize>>>,  // ptr → size
    stats: Arc<Mutex<MemoryStats>>,                    // Aggregated stats
    sample_rate: f64,                                  // 0.0-1.0
    total_seen: AtomicUsize,                           // All allocations seen
    total_tracked: AtomicUsize,                        // Actually tracked
}
```

### Allocation Flow (Source-Level)

```rust
// lockfree_tracker.rs:71-106
pub fn track_allocation(&mut self, ptr: usize, size: usize, ...) {
    // 1. Sampling check — skip most small allocations
    if size < 1024 && !self.should_sample() {
        self.total_seen.fetch_add(1, Ordering::Relaxed);
        return;  // Dropped — zero overhead
    }

    // 2. Non-blocking insert — drop event if lock unavailable
    if let Ok(mut events) = self.events.try_lock() {
        events.push(Event {
            timestamp: now(),
            event_type: EventType::Allocation,
            ptr, size,
            call_stack_hash: self.hash_call_stack(),
            thread_id: self.thread_id,
        });
        self.total_tracked.fetch_add(1, Ordering::Relaxed);
    }
    // If try_lock() fails → event silently dropped (graceful degradation)
}
```

### Sampling Strategy

```rust
// lockfree_tracker.rs:76-79
fn should_sample(&self) -> bool {
    rand::random::<f64>() < self.sample_rate
}
```

At `sample_rate = 0.1`, approximately 10% of small allocations are tracked. Large allocations (≥1KB) bypass sampling and are always tracked.

### Thread-Local Storage

```rust
// lockfree_tracker.rs:358-360
thread_local! {
    static THREAD_TRACKER: RefCell<Option<ThreadLocalTracker>> = const { RefCell::new(None) };
}
```

Each thread initializes its own tracker on first use. **Zero cross-thread synchronization** during tracking.

### Performance

| Metric | Value |
|--------|-------|
| Per-event memory | ~64 bytes (simplified Event struct) |
| Tracking overhead | 2-5% |
| Memory bounded | Yes — events can be flushed to disk |
| Thread safety | Thread-local — zero contention |

### When to Use

Multi-threaded programs with >8 threads, production environments where overhead must be minimal, long-running processes.

---

## Backend 3: Async

**File:** `src/capture/backends/async_tracker.rs`

### Design

The Async backend tracks memory **per async task**, not per OS thread. This is critical because async tasks migrate between OS threads, making thread-based attribution meaningless.

### Task Identification

```rust
// async_types.rs:470-484
pub fn generate_task_id(cx: &Context<'_>) -> AsyncResult<TaskId> {
    let waker_addr = cx.waker() as *const _ as u64;
    let epoch = TASK_EPOCH.fetch_add(1, Ordering::Relaxed);
    // 128-bit ID: (epoch << 64) | waker_addr
    let task_id = ((epoch as u128) << 64) | (waker_addr as u128);
    Ok(task_id)
}
```

The task ID is a **128-bit composite**: a monotonic epoch counter (upper 64 bits) + the waker pointer address (lower 64 bits). This guarantees uniqueness even when waker addresses are reused by the runtime.

### TrackedFuture Wrapper

```rust
// async_types.rs:375-407
impl<F> Future for TrackedFuture<F> where F: Future {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Generate task ID on first poll
        if self.task_id.is_none() {
            self.task_id = Some(generate_task_id(cx)?);
        }

        // Set task context in thread-local storage
        let task_info = ExtendedTaskInfo::new(self.task_id, None);
        set_current_task(task_info);

        let result = self.inner.as_mut().poll(cx);

        // Clear context when future completes
        if result.is_ready() {
            clear_current_task();
        }
        result
    }
}
```

Every `poll()` call sets the task context, so any allocation during the poll is attributed to the correct task.

### Allocation Flow (Source-Level)

```rust
// async_tracker.rs:227-267
pub fn track_allocation_with_location(
    &self, ptr: usize, size: usize, task_id: u64,
    var_name: Option<String>, type_name: Option<String>,
    source_location: Option<SourceLocation>,
) {
    // 1. Record allocation with task association
    let allocation = AsyncAllocation { ptr, size, timestamp, task_id, var_name, type_name, source_location };
    self.allocations.lock().unwrap().insert(ptr, allocation);

    // 2. Update per-task profile
    if let Some(profile) = self.profiles.lock().unwrap().get_mut(&task_id) {
        profile.record_allocation(size as u64);
    }

    // 3. Update global stats
    let mut stats = self.stats.lock().unwrap();
    stats.total_allocations += 1;
    stats.total_memory += size;
    stats.active_memory += size;
    if stats.active_memory > stats.peak_memory {
        stats.peak_memory = stats.active_memory;
    }
}
```

### Task Memory Profile

```rust
// task_profile.rs
pub struct TaskMemoryProfile {
    pub task_id: u64,
    pub task_name: String,
    pub task_type: TaskType,  // CpuIntensive, MemoryIntensive, IoIntensive, ...
    pub total_allocations: u64,
    pub total_bytes: u64,
    pub peak_memory: u64,
    pub current_memory: u64,
    pub allocation_rate: f64,
    pub created_at_ms: u64,
    pub completed_at_ms: Option<u64>,
}
```

### Task Efficiency Analysis

```rust
// async_tracker.rs:353-436
pub fn analyze_task(&self, task_id: u64, task_type: TaskType) -> Option<TaskReport> {
    // Computes:
    // - cpu_efficiency: allocations per ms (or memory reuse ratio for MemoryIntensive)
    // - memory_efficiency: allocation density
    // - io_efficiency: throughput in MB/s
    // - efficiency_score: average of the three
    // - bottleneck: "Execution Time" | "Memory" | "Allocations" | "None"
    // - recommendations: actionable suggestions
}
```

### Performance

| Metric | Value |
|--------|-------|
| Per-allocation memory | ~80 bytes (AsyncAllocation) + HashMap node overhead |
| Tracking overhead | 10-20% |
| Per-task overhead | ~200 bytes (TaskMemoryProfile) |
| Thread safety | Mutex (blocking) |

### When to Use

Async-first applications (tokio, async-std), per-task memory profiling, debugging async memory leaks.

---

## Backend 4: Unified

**File:** `src/capture/backends/mod.rs:348-424`

### Design

The Unified backend is a **strategy selector** — it delegates to the best available backend based on runtime environment detection.

```rust
// mod.rs:362-372
fn detect_best_backend() -> (Box<dyn CaptureBackend>, CaptureBackendType) {
    let thread_count = std::thread::available_parallelism()
        .map(|p| p.get()).unwrap_or(1);

    if thread_count <= 1 {
        (Box::new(CoreBackend), CaptureBackendType::Core)
    } else {
        (Box::new(LockfreeBackend), CaptureBackendType::Lockfree)
    }
    // Note: AsyncBackend detection is not yet implemented
}
```

### Performance

Zero overhead — it's a thin delegation layer. The actual overhead depends on the selected backend.

### When to Use

Default choice. "Just works" without configuration.

---

## Comparison Matrix

| Aspect | Core | Lockfree | Async | Unified |
|--------|------|----------|-------|---------|
| **Synchronization** | DashMap (sharded locks) | Thread-local + try_lock | Mutex (blocking) | Delegates |
| **Per-allocation memory** | ~80 + 32 bytes | ~64 bytes | ~80 bytes | Varies |
| **Tracking overhead** | 5-10% | 2-5% | 10-20% | Auto |
| **Rich type info** | ✅ 25+ fields | ❌ Minimal | ✅ Per-task | Varies |
| **Real-time queries** | ✅ | ❌ | ✅ | Varies |
| **Sampling** | ❌ All tracked | ✅ Rate-based | ❌ All tracked | Varies |
| **Crash resilience** | ❌ | ✅ (if flushed) | ❌ | Varies |
| **Async task support** | ❌ | ❌ | ✅ | Partial |

---

## Choosing the Right Backend

```
Is your program async?
├── Yes → Async Backend
└── No
    ├── Single-threaded or < 8 threads?
    │   └── Yes → Core Backend
    └── Multi-threaded with > 8 threads?
        └── Yes → Lockfree Backend

Not sure? → Unified Backend (auto-selects)
```

---

## Switching Backends

### Via MemScope Facade

```rust
use memscope_rs::facade::MemScope;
use memscope_rs::capture::backends::CaptureBackendType;

// Explicit backend
let memscope = MemScope::new()
    .with_backend(CaptureBackendType::Lockfree)?;

// Auto-select (default)
let memscope = MemScope::new();  // Uses Unified
```

### Via CaptureBackendType Enum

```rust
let backend = CaptureBackendType::Core.create_backend();
let event = backend.capture_alloc(0x1000, 1024, 1);
```
