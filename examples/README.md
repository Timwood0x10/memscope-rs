# MemScope-rs Examples

This directory contains examples demonstrating different tracking strategies using the **new unified API** (`tracker!()` + `track!()` macros).

## 📁 Available Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`basic_usage.rs`](basic_usage.rs) | Basic single-threaded tracking | Simple API usage, JSON export |
| [`complex_lifecycle_showcase.rs`](complex_lifecycle_showcase.rs) | Variable lifecycle analysis | Built-in types, smart pointers, complex patterns |
| [`complex_multithread_showcase.rs`](complex_multithread_showcase.rs) | Multi-threaded tracking | 8 threads, 500 allocations/thread |
| [`comprehensive_async_showcase.rs`](comprehensive_async_showcase.rs) | Async task tracking | Tokio async runtime |
| [`unsafe_ffi_demo.rs`](unsafe_ffi_demo.rs) | FFI boundary tracking | Memory Passport, leak detection |
| [`large_binary.rs`](large_binary.rs) | Large dataset handling | Extreme performance benchmarking |

## 🚀 Quick Start

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

# Performance testing
cargo run --example large_binary
```

## 📖 New Unified API

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

## 📄 Export Files

Each example generates JSON files in `MemoryAnalysis/<example_name>/`:

| File | Description |
|------|-------------|
| `memory_analysis.json` | All allocations with details |
| `lifetime.json` | Ownership history and lifecycle events |
| `thread_analysis.json` | Per-thread memory statistics |
| `variable_relationships.json` | Variable dependency graph |

---

**Happy memory tracking! 🚀🦀**
