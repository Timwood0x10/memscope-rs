# New Features & Highlights

> MemScope v0.1.10 — What's new and what makes it stand out

---

## 1. Nine-Engine Pipeline Architecture

MemScope has been re-architected from a monolithic tracker into a **modular 9-engine pipeline**:

| Engine | Purpose |
|--------|---------|
| **CaptureEngine** | Pluggable backends (Core, Lockfree, Async, Unified) |
| **EventStore** | Lock-free `SegQueue` event storage |
| **MetadataEngine** | Scopes, stack traces, smart pointer analysis |
| **SnapshotEngine** | Point-in-time memory snapshots |
| **QueryEngine** | Unified query interface (summary, top-N, filters) |
| **TimelineEngine** | Time-series memory analysis and replay |
| **AnalysisEngine** | Pluggable detectors (leak, UAF, overflow, safety, lifecycle) |
| **RenderEngine** | JSON, HTML dashboard, SVG, binary export |
| **Tracker API** | High-level simplified interface with system monitoring |

**Why it matters:** Each engine is independently testable, replaceable, and composable. You can use just the capture engine for lightweight tracking, or the full pipeline for deep analysis.

---

## 2. Unified Auto-Detection Backend

The **Unified backend** automatically selects the best capture strategy based on your runtime environment:

- Single-threaded → **Core backend** (DashMap + atomic counters)
- Multi-threaded → **Lockfree backend** (thread-local + sampling)
- Async runtime detected → **Async backend** (task-level profiling)

```rust
// Zero configuration — just works
let memscope = MemScope::new();
```

---

## 3. MemScope Facade — Single Entry Point

The new `MemScope` struct provides a **unified API** that ties all 9 engines together:

```rust
use memscope_rs::facade::MemScope;

let memscope = MemScope::new();

// ... your code runs with tracking ...

// Query
let summary = memscope.summary()?;
let top = memscope.top_allocations(10)?;

// Run detectors
memscope.run_leak_detector()?;
memscope.run_uaf_detector()?;

// Export
memscope.export_html("report.html")?;
memscope.export_json("report.json")?;
```

**One struct, full control.** No more juggling between `get_global_tracker()`, `get_capture_tracker()`, and manual engine coordination.

---

## 4. Pluggable Detector System

Five built-in detectors, plus the ability to register your own:

| Detector | What It Finds |
|----------|--------------|
| **LeakDetector** | Allocations with no matching deallocation |
| **UafDetector** | Use-after-free patterns (access after free) |
| **OverflowDetector** | Buffer overflow risks (writes beyond allocation bounds) |
| **SafetyDetector** | Unsafe code safety violations |
| **LifecycleDetector** | RAII/Drop pattern analysis |

```rust
// Register a custom detector
memscope.register_detector(MyDetector::new())?;

// Run all registered detectors
memscope.run_detectors()?;
```

---

## 5. System Monitoring

The Tracker API includes **real-time system monitoring**:

- **CPU usage** — Per-core and average utilization
- **Memory** — RSS, virtual memory, swap usage
- **Disk I/O** — Read/write bytes and operations
- **Network** — Bytes sent/received
- **GPU** — Memory and utilization (where available)

```rust
let tracker = tracker!().with_system_monitoring();
let report = tracker.analyze();
println!("CPU: {:.1}%, Memory: {}MB",
    report.system_snapshot.cpu_usage,
    report.system_snapshot.memory_rss / 1024 / 1024);
```

---

## 6. Unsafe Type Inference Engine

A **heuristic-based type inference** system for FFI/unsafe allocations where type information is unavailable:

- **Size heuristic** — Detects common type sizes (8 = pointer, 24 = Vec/String)
- **Layout detection** — Identifies `(ptr, len, cap)` triplet for Vec/String
- **Content analysis** — Shannon entropy for binary data, zero-fill detection
- **Pointer counting** — Distinguishes buffers (0 pointers) from C structs (2+ pointers)
- **Power-of-two signal** — Rust Vec capacity growth pattern detection

```rust
use memscope_rs::analysis::unsafe_inference::{
    UnsafeInferenceEngine, TypeKind,
};

let memory = /* raw bytes from FFI allocation */;
let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, size);
println!("Likely type: {} ({}% confidence)", guess.kind, guess.confidence);
```

---

## 7. Memory Passport System

Tracks **memory ownership across FFI boundaries**:

- Records when memory crosses from Rust → C or C → Rust
- Tracks ownership transfers, borrows, and returns
- Detects leaks at FFI boundaries (memory allocated in Rust, never freed in C, or vice versa)

---

## 8. Timeline Replay

**Replay memory events chronologically** to understand how memory evolved over time:

```rust
use memscope_rs::timeline::TimelineEngine;

let timeline = TimelineEngine::new(&events);
let replay = timeline.replay_until(timestamp);
println!("At this point, {} allocations were active", replay.len());
```

---

## 9. HTML Dashboard

Rich, **interactive HTML reports** with:

- Memory usage charts over time
- Top allocations by size and count
- Per-thread breakdown
- Detector results (leaks, UAF, etc.)
- Type distribution pie charts
- Allocation timeline visualization

```rust
memscope.export_html("dashboard.html")?;
// Open dashboard.html in your browser
```

---

## 10. Trackable Trait + Derive Macro

Implement `Trackable` for custom types to get **automatic type-aware tracking**:

```rust
// Built-in implementations for Vec, String, HashMap, Box, Rc, Arc, etc.

// Custom types with derive macro (feature = "derive")
#[derive(Trackable)]
struct MyStruct {
    data: Vec<u8>,
    name: String,
}
```

The trait provides:
- `get_heap_ptr()` — Heap allocation address
- `get_type_name()` — Static type name
- `get_size_estimate()` — Estimated memory footprint
- `get_ref_count()` — Reference count (for Rc/Arc)
- `get_data_ptr()` / `get_data_size()` — Inner data location

---

## Performance Benchmarks

| Scenario | Overhead | Memory Cost |
|----------|----------|-------------|
| Single-threaded (Core) | 5-10% | ~80 bytes/alloc |
| Multi-threaded (Lockfree) | 2-5% | ~64 bytes/event |
| Async (Async) | 10-20% | ~80 bytes/task |
| Auto-select (Unified) | Adaptive | Adaptive |

---

## Migration from v0.1.x

### Old API (deprecated)
```rust
memscope_rs::init();
let tracker = get_global_tracker();
tracker.export_to_json("output.json")?;
```

### New API (recommended)
```rust
let memscope = MemScope::new();
// ... code ...
memscope.export_json("output.json")?;
```

Or for quick scripts:
```rust
let t = tracker!();
track!(t, my_data);
let report = t.analyze();
```
