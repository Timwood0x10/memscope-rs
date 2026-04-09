# MemScope-rs Examples

This directory contains examples demonstrating different tracking strategies using the **new unified API** (`tracker!()` + `track!()` macros).

## Available Examples

### Core Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`basic_usage.rs`](basic_usage.rs) | Basic single-threaded tracking | Simple API usage, JSON export |
| [`complex_lifecycle_showcase.rs`](complex_lifecycle_showcase.rs) | Variable lifecycle analysis | Built-in types, smart pointers, complex patterns |
| [`merkle_tree.rs`](merkle_tree.rs) | Complex data structure tracking | HashMap, Vec, tree structures, ownership relationships |

### Multi-Threaded & Async Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`complex_multithread_showcase.rs`](complex_multithread_showcase.rs) | Multi-threaded tracking | 8 threads, 500 allocations/thread |
| [`comprehensive_async_showcase.rs`](comprehensive_async_showcase.rs) | Async task tracking | Tokio async runtime |

### FFI Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`unsafe_ffi_demo.rs`](unsafe_ffi_demo.rs) | FFI boundary tracking | Memory Passport, leak detection |

### Global Tracker Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`global_tracker_showcase.rs`](global_tracker_showcase.rs) | Global tracker unified API | Single-thread, multi-thread, async modes |

## Quick Start

### Run Examples

```bash
# Basic single-threaded
cargo run --example basic_usage

# Multi-threaded
cargo run --example complex_multithread_showcase

# Async tracking
cargo run --example comprehensive_async_showcase

# FFI tracking
cargo run --example unsafe_ffi_demo

# Variable relationships (ownership graph)
cargo run --example merkle_tree

# Global tracker (single-thread + multi-thread + async)
cargo run --example global_tracker_showcase

# Lifecycle analysis
cargo run --example complex_lifecycle_showcase
```

## New Unified API

```rust
use memscope_rs::{track, tracker};
use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;

// 1. Initialize tracker
let tracker = tracker!();

// 2. Track variables
let data = vec![1, 2, 3];
track!(tracker, data);

// 3. Get analysis report
let report = tracker.analyze();
println!("Total Allocations: {}", report.total_allocations);

// 4. Export to JSON
let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
let snapshot = MemorySnapshot::from_allocation_infos(allocations);
export_snapshot_to_json(&snapshot, "output".as_ref(), &ExportJsonOptions::default())?;
```

### Global Tracker API (Lazy Init, Multi-Mode)

```rust
use memscope_rs::capture::backends::global_tracking::*;

// 1. Initialize (optional - lazy by default)
init_global_tracking().ok();

// 2. Get global tracker
let tracker = global_tracker().unwrap();
let passport_tracker = global_passport_tracker().unwrap();

// 3. Track allocations
tracker.track_allocation(ptr, size).ok();

// 4. Export using global singleton (convenience)
export_to_json("output").ok();

// Or export with custom tracker (generic)
export_all_json("output", &tracker, &passport_tracker).ok();

// 5. Individual exports
export_memory_passports_json("output", &passport_tracker).ok();
export_leak_detection_json("output", &passport_tracker).ok();
export_unsafe_ffi_json("output", &passport_tracker).ok();
```

## Export Files

Each example generates JSON files in `MemoryAnalysis/<example_name>/`.

### Global Tracker - 9 Files (`global_tracker_showcase`)

| File | Description |
|------|-------------|
| `memory_analysis.json` | All allocation details |
| `lifetime.json` | Variable lifecycle and ownership history |
| `thread_analysis.json` | Memory statistics by thread |
| `ownership_graph.json` | Ownership graph with real pointer relationships |
| `memory_passports.json` | Memory passport metadata |
| `leak_detection.json` | Leak detection results |
| `unsafe_ffi.json` | FFI boundary tracking data |
| `system_resources.json` | System resource monitoring |
| `async_analysis.json` | Async task analysis |

### Other Examples - 4 Files

| File | Description |
|------|-------------|
| `memory_analysis.json` | All allocation details |
| `lifetime.json` | Lifecycle events |
| `thread_analysis.json` | Thread statistics |
| `ownership_graph.json` | Ownership graph |

## Variable Relationships

The ownership graph in `ownership_graph.json` and the dashboard HTML contain real **ownership relationships** detected by analyzing heap memory:

- **Owner** - A contains pointer to B (e.g., Vec metadata -> buffer)
- **Slice** - A points into B's interior (e.g., slice metadata -> buffer)
- **Clone** - A and B are copies with similar content
- **Shared** - Multiple Arc/Rc pointing to same data

Examples with rich variable relationships:

| Example | Relationships | Allocations |
|---------|---------------|-------------|
| merkle_tree | ~500 | ~1164 |
| variable_relationships_showcase | ~265 | ~52 |
| multithread_new_api | ~133 | ~82 |
| complex_lifecycle_showcase | ~20 | ~18 |

---
