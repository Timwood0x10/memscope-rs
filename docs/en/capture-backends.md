# Capture Backends Deep Dive

> Understanding the four data collection strategies in MemScope

---

## Overview

MemScope's `CaptureEngine` supports four pluggable backends, each optimized for different workload patterns. The backend determines **how allocation events are captured, stored, and aggregated**.

```
                    ┌───────────────┐
                    │ CaptureEngine │
                    └───────┬───────┘
                            │ routes to
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
    ┌──────────┐    ┌──────────────┐  ┌───────────┐
    │   Core   │    │  Lockfree    │  │   Async   │
    │ Backend  │    │  Backend     │  │  Backend  │
    └──────────┘    └──────────────┘  └───────────┘
                          │
                   ┌──────┴──────┐
                   │  Unified    │  ← Auto-selects best backend
                   │  Backend    │
                   └─────────────┘
```

---

## 1. Core Backend

**File:** `src/capture/backends/core_tracker.rs`

### How It Works

```
alloc(ptr, size)
  → MemoryTracker::track_allocation()
  → DashMap::insert(ptr, AllocationInfo)    ← sharded concurrent HashMap
  → AtomicU64::fetch_add() for counters     ← lock-free stats
```

### Data Structures

- **`DashMap<usize, AllocationInfo>`** — Stores all active allocations. DashMap uses per-shard read-write locks internally, providing much better concurrent read performance than a single `Mutex<HashMap>`.
- **`AtomicU64` counters** — `total_allocations`, `total_allocated`, `total_deallocations`, `total_deallocated`, `peak_memory` — all updated lock-free.
- **`AtomicUsize` for `peak_allocations`** — Tracks the maximum number of simultaneous active allocations.

### Threading Model

Two modes controlled by `STRATEGY_THREAD_LOCAL` flag:
- **Global singleton** — One `MemoryTracker` shared across all threads via `Arc`
- **Thread-local** — Each thread gets its own `MemoryTracker`, registered in a `DashMap<ThreadId, Arc<MemoryTracker>>`

### Pros
- Full `AllocationInfo` with 25+ fields (type name, variable name, stack trace, borrow info, etc.)
- Real-time queries via `get_stats()`, `get_active_allocations()`
- Rich type information for analysis

### Cons
- DashMap shard locks introduce contention under very high write rates
- Memory grows unbounded — all active allocations stay in memory
- Peak allocation tracking has a TOCTOU race condition

### When to Use
- Single-threaded programs
- Low-to-moderate concurrency (< 8 threads)
- When you need rich type metadata per allocation

---

## 2. Lockfree Backend

**File:** `src/capture/backends/lockfree_tracker.rs`

### How It Works

```
alloc(ptr, size)
  → THREAD_TRACKER.with(|tl| ...)              ← thread-local tracker
  → ThreadLocalTracker::track_allocation()
  → Sampling check (rand::random() < rate)     ← probabilistic sampling
  → Arc<Mutex<Vec<Event>>>::try_lock()         ← non-blocking
  → If lock acquired: vec.push(Event)          ← event stored
  → If lock failed: event dropped              ← graceful degradation
```

### Data Structures

- **`thread_local! { RefCell<Option<ThreadLocalTracker>> }`** — Each thread has its own tracker, zero cross-thread synchronization during the hot path.
- **`Arc<Mutex<Vec<Event>>>`** — Shared event buffer, accessed via `try_lock()` to avoid blocking.
- **`Arc<Mutex<HashMap<usize, usize>>>`** — Active allocation size tracking.
- **`AtomicUsize` for `total_seen` / `total_tracked`** — Lock-free sampling statistics.

### Sampling Strategy

Simple rate-based sampling:
```rust
if rand::random::<f64>() < self.sample_rate {
    // Track this allocation
}
```

This means at `sample_rate = 0.1`, approximately 10% of allocations are tracked. Large allocations are always tracked (size-based override).

### Pros
- Near-zero contention — thread-local during tracking
- Bounded memory — events can be flushed to disk
- Graceful degradation under contention (`try_lock` drops events rather than blocking)

### Cons
- Simpler `Event` struct (fewer fields than `AllocationInfo`)
- Sampling means you miss some allocations
- `try_lock()` drops events under high contention
- No real-time query support — must finalize first

### When to Use
- High-concurrency multi-threaded programs (> 8 threads)
- Production environments where overhead must be minimal
- Long-running processes where memory must be bounded

---

## 3. Async Backend

**File:** `src/capture/backends/async_tracker.rs`

### How It Works

```
async task spawns
  → TrackedFuture::poll() intercepts
  → AsyncTracker::track_allocation()
  → Mutex<HashMap<usize, AsyncAllocation>>  ← keyed by ptr
  → Updates TaskMemoryProfile (per-task stats)
  → Updates AsyncStats (global stats)
```

### Data Structures

- **`Mutex<HashMap<usize, AsyncAllocation>>`** — Active allocations, keyed by pointer.
- **`Mutex<AsyncStats>`** — Global statistics: total allocations, active memory, task count.
- **`Mutex<HashMap<u64, TaskMemoryProfile>>`** — Per-task memory profiles, keyed by task ID.

### Task Tracking

Each async task gets its own `TaskMemoryProfile`:
- `allocation_count` — Total allocations in this task
- `active_memory` — Currently allocated bytes
- `peak_memory` — Peak memory usage
- `efficiency_score` — Resource utilization efficiency

### Pros
- Task-level memory isolation — see which async task uses how much memory
- Tracks task lifecycle (spawn → run → complete)
- Resource ranking across tasks

### Cons
- Blocking `Mutex` throughout — contention under high task concurrency
- No sampling — all allocations tracked
- Higher overhead (~10-20%) due to task context management

### When to Use
- Async-first applications (tokio, async-std)
- When you need per-task memory profiling
- Debugging async memory leaks

---

## 4. Unified Backend

**File:** `src/capture/backends/unified_tracker.rs`

### How It Works

The Unified backend is a **strategy selector** — it doesn't collect data itself, but delegates to the best available backend based on runtime environment detection.

### Auto-Detection Logic

```rust
fn detect_best_backend() -> (Box<dyn CaptureBackend>, CaptureBackendType) {
    let thread_count = std::thread::available_parallelism()
        .map(|p| p.get()).unwrap_or(1);

    if thread_count <= 1 {
        (Box::new(CoreBackend), CaptureBackendType::Core)
    } else {
        (Box::new(LockfreeBackend), CaptureBackendType::Lockfree)
    }
    // Note: AsyncBackend detection is planned but not yet implemented
}
```

### Pros
- Zero configuration — "just works"
- Automatic optimization for the current environment
- Easy to extend with new backends

### Cons
- Async detection not yet implemented (relies on env vars)
- May not choose optimally for mixed workloads

### When to Use
- Default choice for most users
- When you don't want to think about backend selection
- Libraries that need to work across different host environments

---

## Comparison Matrix

| Aspect | Core | Lockfree | Async | Unified |
|--------|------|----------|-------|---------|
| **Synchronization** | DashMap (sharded locks) | Thread-local + try_lock | Mutex (blocking) | Delegates |
| **Per-allocation overhead** | ~80 bytes | ~64 bytes | ~80 bytes | Varies |
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

### Via Environment

The Unified backend respects:
- `TOKIO_WORKER_THREADS` — Detects tokio runtime
- `ASYNC_STD_THREAD_COUNT` — Detects async-std runtime
