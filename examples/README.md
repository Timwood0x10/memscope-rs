# MemScope-rs Examples

This directory contains examples demonstrating different tracking strategies using the **new unified API** (`tracker!()` + `track!()` macros).

## 📁 Available Examples

### Core Examples

| Example | Description | Key Features |
|---------|-------------|--------------|
| [`basic_usage.rs`](basic_usage.rs) | Basic single-threaded tracking | Simple API usage, JSON export |
| [`complex_lifecycle_showcase.rs`](complex_lifecycle_showcase.rs) | Variable lifecycle analysis | Built-in types, smart pointers, complex patterns |
| [`large_binary.rs`](large_binary.rs) | Large dataset handling | Extreme performance benchmarking |

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

# Global tracker (single-thread + multi-thread + async)
cargo run --example global_tracker_showcase
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

## 📄 Export Files

Each example generates JSON files in `MemoryAnalysis/<example_name>/`. The global tracker exports **9 files**, while other examples export **4 files**.

### 9 Files (Global Tracker - `global_tracker_showcase`)

| File | Description | Key Fields |
|------|-------------|------------|
| `memory_analysis.json` | 所有内存分配详情 | `ptr`, `size`, `type_name`, `var_name`, `thread_id`, `allocated_at` |
| `lifetime.json` | 变量生命周期和所有权历史 | `var_name`, `allocated_at`, `deallocated_at`, `ownership_chain` |
| `thread_analysis.json` | 按线程分类的内存统计 | `thread_id`, `allocations`, `total_bytes`, `thread_name` |
| `ownership_graph.json` | 所有权图分析 | `nodes`, `edges`, `cycles`, `diagnostics`, `root_cause` |
| `memory_passports.json` | 内存护照元数据 | `passport_id`, `allocation_ptr`, `size_bytes`, `created_at`, `status` |
| `leak_detection.json` | 泄漏检测结果 | `total_leaks`, `passport_id`, `memory_address`, `size_bytes`, `lifecycle_summary` |
| `unsafe_ffi.json` | FFI边界追踪数据 | `passport_id`, `allocation_ptr`, `boundary_events`, `event_type`, `context` |
| `system_resources.json` | 系统资源监控 | `os_name`, `cpu_cores`, `total_physical`, `used_physical` |
| `async_analysis.json` | 异步任务分析 | `total_tasks`, `active_tasks`, `total_allocations` |

### 4 Files (Other Examples)

| File | Description | Key Fields |
|------|-------------|------------|
| `memory_analysis.json` | 所有分配详情 | `ptr`, `size`, `type_name`, `var_name`, `thread_id` |
| `lifetime.json` | 生命周期事件 | `var_name`, `allocated_at`, `deallocated_at` |
| `thread_analysis.json` | 线程统计 | `thread_id`, `allocations`, `total_bytes` |
| `ownership_graph.json` | 所有权图 | `nodes`, `edges`, `cycles`, `diagnostics` |

### Export Data Examples

**memory_passports.json:**
```json
{
  "memory_passports": [
    {
      "passport_id": "passport_131008200_...",
      "allocation_ptr": "0x131008200",
      "size_bytes": 256,
      "created_at": 1775115134,
      "status": "InForeignCustody"
    }
  ]
}
```

**leak_detection.json:**
```json
{
  "leak_detection": {
    "total_leaks": 5,
    "leak_details": [
      {
        "passport_id": "passport_131008200_...",
        "memory_address": "0x131008200",
        "size_bytes": 256,
        "lifecycle_summary": "Never deallocated"
      }
    ]
  }
}
```

**unsafe_ffi.json:**
```json
{
  "unsafe_reports": [
    {
      "passport_id": "passport_131008200_...",
      "allocation_ptr": "0x131008200",
      "size_bytes": 512,
      "boundary_events": [
        {"event_type": "AllocatedInRust", "context": "ffi_alloc_1"},
        {"event_type": "HandoverToFfi", "context": "foreign_function"}
      ]
    }
  ]
}
```

---

**Happy memory tracking! 🚀🦀**
