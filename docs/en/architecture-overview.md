# Architecture Overview

> MemScope v0.1.10 — 9-Engine Pipeline Architecture

***

## System Architecture

MemScope captures, analyzes, and visualizes memory allocations through a **9-engine pipeline**:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Code                         │
│                    (your Rust program)                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
              ┌────────────▼────────────┐
              │   #[global_allocator]   │
              │   TrackingAllocator     │
              └────────────┬────────────┘
                           │ alloc / dealloc
              ┌────────────▼────────────┐
              │     CaptureEngine       │  ← Pluggable backend
              │  ┌────┬────┬────┬────┐  │
              │  │Core│Lock│Async│Unified│ │
              │  └────┴────┴────┴────┘  │
              └────────────┬────────────┘
                           │ MemoryEvent
              ┌────────────▼────────────┐
              │       EventStore        │  ← SegQueue (lock-free)
              └────────────┬────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ MetadataEngine│  │SnapshotEngine │  │ TimelineEngine│
│  scopes,      │  │  point-in-    │  │  time-based   │
│  stack traces │  │  time view    │  │  replay       │
└───────┬───────┘  └───────┬───────┘  └───────┬───────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
              ┌────────────────────┐
              │    QueryEngine     │  ← summary, top-N, filters
              └────────┬───────────┘
                       │
              ┌────────▼───────────┐
              │  AnalysisEngine    │  ← detectors, analyzers
              │  ┌──────────────┐  │
              │  │ LeakDetector │  │
              │  │ UafDetector  │  │
              │  │ Overflow     │  │
              │  │ Safety       │  │
              │  │ Lifecycle    │  │
              │  └──────────────┘  │
              └────────┬───────────┘
                       │
              ┌────────▼───────────┐
              │    RenderEngine    │  ← JSON, HTML dashboard, SVG
              └────────────────────┘
```

***

## The 9 Engines

### 1. CaptureEngine — Data Ingestion

**Module:** `src/capture/engine.rs`

The entry point for all memory events. Routes allocation/deallocation calls to a pluggable backend and forwards events to the EventStore.

**Four backends:**

| Backend      | File                  | Strategy                                                           | Best For                                |
| ------------ | --------------------- | ------------------------------------------------------------------ | --------------------------------------- |
| **Core**     | `core_tracker.rs`     | `DashMap<usize, AllocationInfo>` + `AtomicU64` counters            | Single-threaded or low-concurrency apps |
| **Lockfree** | `lockfree_tracker.rs` | `Arc<Mutex<Vec<Event>>>` with `try_lock()` + thread-local tracking | High-concurrency multi-threaded apps    |
| **Async**    | `async_tracker.rs`    | `Mutex<HashMap<task_id, AsyncAllocation>>`                         | Async runtimes (tokio, async-std)       |
| **Unified**  | `unified_tracker.rs`  | Auto-detects: CPU count → Core/Lockfree; async runtime → Async     | "Just work" — recommended default       |

**Unified auto-detection logic:**

```
thread_count <= 1        → Core backend
thread_count > 1         → Lockfree backend
tokio/async-std detected → Async backend (planned)
```

### 2. EventStore — Centralized Event Storage

**Module:** `src/event_store/store.rs`

Stores all `MemoryEvent` records in a lock-free `SegQueue<MemoryEvent>` (from crossbeam). Every allocation, deallocation, reallocation, and move event flows through here.

**Key operations:**

- `record(event)` — Push event to queue (lock-free, O(1))
- `snapshot()` — Drain-and-restore for point-in-time view (O(n))
- `len()` — Current event count (concurrent read)

### 3. MetadataEngine — Contextual Information

**Module:** `src/metadata/`

Enriches raw allocation events with contextual metadata:

- **Scope tracking** — Variable scopes and lifetime boundaries
- **Thread metadata** — Thread names, IDs, grouping
- **Stack traces** — Call stack capture and normalization
- **Smart pointer analysis** — Rc/Arc reference count tracking, clone/borrow detection

### 4. SnapshotEngine — Point-in-Time Views

**Module:** `src/snapshot/engine.rs`

Builds `MemorySnapshot` from the EventStore:

- Maps of active allocations by pointer address
- Per-thread statistics
- Aggregate memory stats (total, active, peak)

### 5. QueryEngine — Data Access

**Module:** `src/query/engine.rs`

Unified query interface over snapshots:

- `summary()` — Overall memory statistics
- `top_allocations(n)` — Top N allocations by size
- `by_thread(thread_id)` — Filter by thread
- `by_type(type_name)` — Filter by type

### 6. TimelineEngine — Time-Based Analysis

**Module:** `src/timeline/engine.rs`

Time-series memory analysis:

- `get_events_in_range(start, end)` — Events in time window
- `get_memory_usage_over_time(start, end, interval)` — Memory trend
- `get_peak_memory_in_range(start, end)` — Peak memory
- `TimelineReplay` — Replay events chronologically

### 7. AnalysisEngine — Detectors & Analyzers

**Module:** `src/analysis_engine/engine.rs` + `src/analysis/`

Pluggable analysis via the `Detector` trait:

| Detector              | Purpose                                         |
| --------------------- | ----------------------------------------------- |
| **LeakDetector**      | Finds allocations with no matching deallocation |
| **UafDetector**       | Detects use-after-free patterns                 |
| **OverflowDetector**  | Identifies buffer overflow risks                |
| **SafetyDetector**    | General unsafe code safety violations           |
| **LifecycleDetector** | RAII/Drop pattern analysis                      |

Additional analysis modules:

- **Async analysis** — Task memory profiling, efficiency scoring
- **Borrow analysis** — Mutable/immutable borrow pattern detection
- **Generic analysis** — Generic type instantiation statistics
- **Closure analysis** — Closure capture and lifetime analysis
- **Memory Passport** — FFI boundary ownership tracking
- **Unsafe type inference** — Heuristic type detection for raw allocations

### 8. RenderEngine — Output Generation

**Module:** `src/render_engine/`

Multiple export formats:

- **JSON** — Machine-readable analysis results
- **HTML Dashboard** — Interactive web-based visualization with charts

## 9. Tracker API — Simplified Interface

**Module:** `src/tracker.rs`

A higher-level, user-friendly API built on top of the engines:

```rust
let tracker = tracker!();
track!(tracker, my_vec);
let report = tracker.analyze();
```

Features:

- **System monitoring** — CPU, memory, disk I/O, network, GPU metrics
- **Sampling** — Configurable sample rate to reduce overhead
- **Auto-export** — Export on drop
- **Hotspot analysis** — Identify allocation hotspots by call site

***

## Data Flow: End-to-End

### Scenario: Tracking a `Vec` in a multi-threaded program

```
1. User code:
   let data = vec![1, 2, 3];
   track_var!(data);

2. TrackingAllocator::alloc() intercepts the heap allocation
   → ptr: 0x7f..., size: 24, thread_id: 3

3. CaptureEngine routes to LockfreeBackend (multi-threaded detected)
   → ThreadLocalTracker.push(Event { ptr, size, timestamp, ... })

4. Event forwarded to EventStore
   → SegQueue.push(MemoryEvent::Allocate { ... })

5. MetadataEngine enriches the event
   → var_name: "data", type_name: "Vec<i32>", scope: "main"

6. User calls: memscope.export_html("report.html")

7. SnapshotEngine builds MemorySnapshot from EventStore
   → active_allocations: { 0x7f...: ActiveAllocation { ... } }

8. QueryEngine runs summary() and top_allocations(10)

9. AnalysisEngine runs all registered detectors
   → LeakDetector: 0 leaks found
   → UafDetector: 0 issues found

10. RenderEngine generates HTML dashboard
    → Charts, tables, allocation timeline, detector results
```

***

## Two APIs: Which to Use?

### MemScope Facade (Recommended)

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();
// ... your code ...
memscope.export_html("report.html")?;
memscope.run_leak_detector()?;
```

**Best for:** Most users. Unified access to all 9 engines.

### Tracker Macros (Quick & Simple)

```rust
use memscope_rs::{tracker, track};

let t = tracker!();
track!(t, my_data);
let report = t.analyze();
```

**Best for:** Quick scripts, simple programs, when you don't need the full engine pipeline.

***

## Feature Flags

| Feature              | Default | Description                               |
| -------------------- | ------- | ----------------------------------------- |
| `tracking-allocator` | ✅       | Enable `#[global_allocator]` interception |
| `backtrace`          | ❌       | Call stack capture                        |
| `derive`             | ❌       | `#[derive(Trackable)]` macro              |
| `test`               | ❌       | Testing utilities                         |

***

## Performance Characteristics

| Backend  | Tracking Overhead | Memory Overhead  | Thread Safety            |
| -------- | ----------------- | ---------------- | ------------------------ |
| Core     | \~5-10%           | \~80 bytes/alloc | DashMap (sharded locks)  |
| Lockfree | \~2-5%            | \~64 bytes/event | Thread-local + try\_lock |
| Async    | \~10-20%          | \~80 bytes/task  | Mutex (per-task)         |
| Unified  | Auto-selects      | Auto-selects     | Auto-selects             |

