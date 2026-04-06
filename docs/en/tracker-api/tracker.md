# Tracker API

> High-level simplified interface with system monitoring and hotspot analysis

---

## Overview

**File:** `src/tracker.rs`

The Tracker API is a **higher-level, user-friendly interface** built on top of the engine pipeline. It provides a simple macro-based API for quick memory tracking with built-in system monitoring, sampling, and hotspot analysis.

---

## Quick Start

```rust
use memscope_rs::{tracker, track};

// Create a tracker with the macro
let t = tracker!();

// Track variables
let data = vec![1, 2, 3, 4, 5];
track!(t, data);

// Analyze
let report = t.analyze();
println!("Hotspots: {:?}", report.hotspots);
```

---

## The `tracker!` Macro

```rust
// tracker.rs
#[macro_export]
macro_rules! tracker {
    () => {
        $crate::Tracker::new()
    };
}
```

Creates a new `Tracker` instance with default configuration.

---

## The `track!` Macro

```rust
// tracker.rs
#[macro_export]
macro_rules! track {
    ($tracker:expr, $var:ident) => {
        $tracker.track_variable(
            &$var,
            stringify!($var),
            file!(),
            line!(),
        );
    };
}
```

Automatically captures the variable name, file, and line number. Calls `track_variable()` on the tracker with the variable reference and its stringified name.

---

## Tracker Struct

```rust
// tracker.rs
pub struct Tracker {
    config: Arc<Mutex<TrackerConfig>>,
    allocations: Arc<Mutex<HashMap<usize, TrackedAllocation>>>,
    system_monitor: Option<Arc<SystemMonitor>>,
    hotspots: Arc<Mutex<Vec<AllocationHotspot>>>,
}
```

### Configuration

```rust
pub struct TrackerConfig {
    pub sampling_rate: f64,           // 0.0-1.0, what fraction to track
    pub auto_export_on_drop: bool,    // Export when tracker is dropped
    pub export_path: Option<String>,  // Where to export on drop
    pub enable_system_monitoring: bool, // Collect system metrics
    pub max_hotspots: usize,          // Maximum hotspot entries to keep
}
```

### Builder Pattern

```rust
let tracker = Tracker::new()
    .with_sampling(0.5)              // Track 50% of allocations
    .with_system_monitoring()        // Enable CPU/memory/disk monitoring
    .with_auto_export("output.json"); // Auto-export on drop
```

---

## Variable Tracking

```rust
pub fn track_variable<T: Trackable>(
    &self,
    value: &T,
    name: &str,
    file: &str,
    line: u32,
) {
    let ptr = value.get_heap_ptr();
    let size = value.get_size_estimate();
    let type_name = value.get_type_name();

    let allocation = TrackedAllocation {
        ptr,
        size,
        var_name: name.to_string(),
        type_name: type_name.to_string(),
        file: file.to_string(),
        line,
        timestamp: now(),
    };

    self.allocations.lock().unwrap().insert(ptr, allocation);
}
```

Uses the `Trackable` trait to extract heap pointer, size estimate, and type name from any tracked value.

---

## The Trackable Trait

**File:** `src/lib.rs`

```rust
pub trait Trackable {
    fn get_heap_ptr(&self) -> Option<usize>;
    fn get_type_name(&self) -> &'static str;
    fn get_size_estimate(&self) -> usize;
    fn get_ref_count(&self) -> Option<usize> { None }
    fn get_data_ptr(&self) -> Option<usize>;
    fn get_data_size(&self) -> Option<usize>;
}
```

### Built-in Implementations

Implemented for: `Vec<T>`, `String`, `HashMap<K,V>`, `BTreeMap<K,V>`, `VecDeque<T>`, `Box<T>`, `Rc<T>`, `Arc<T>`, `RefCell<T>`, `RwLock<T>`.

### Derive Macro

```rust
// Feature: "derive"
#[derive(Trackable)]
struct MyStruct {
    data: Vec<u8>,
    name: String,
}
```

---

## System Monitoring

**File:** `src/tracker.rs` + `src/capture/system_monitor.rs`

The Tracker can collect real-time system metrics alongside memory data:

```rust
pub struct SystemSnapshot {
    pub cpu_usage: f64,           // 0.0-100.0%
    pub memory_rss: usize,        // Resident set size (bytes)
    pub memory_virtual: usize,    // Virtual memory size (bytes)
    pub disk_read_bytes: u64,     // Total bytes read
    pub disk_write_bytes: u64,    // Total bytes written
    pub network_rx_bytes: u64,    // Bytes received
    pub network_tx_bytes: u64,    // Bytes sent
    pub gpu_memory: Option<u64>,  // GPU memory (if available)
}
```

### How It Works

```rust
// tracker.rs
pub fn with_system_monitoring(mut self) -> Self {
    self.system_monitor = Some(SystemMonitor::global());
    self
}
```

`SystemMonitor::global()` returns a `'static` singleton that runs a background thread collecting system metrics at regular intervals.

**Implementation:** Uses platform-specific APIs:
- **macOS:** `host_statistics64` for CPU/memory, `sysctl` for disk/network
- **Linux:** `/proc/stat`, `/proc/meminfo`, `/proc/diskstats`
- **Windows:** `GetSystemTimes`, `GlobalMemoryStatusEx`

---

## Analysis

```rust
pub fn analyze(&self) -> AnalysisReport {
    let allocations = self.allocations.lock().unwrap();
    let alloc_vec: Vec<_> = allocations.values().cloned().collect();

    // Build hotspot analysis
    let hotspots = self.build_hotspots(&alloc_vec);

    // Collect system snapshot
    let system_snapshots = if let Some(ref monitor) = self.system_monitor {
        vec![monitor.get_current_snapshot()]
    } else {
        vec![]
    };

    // Calculate peak memory (workaround for broken stats.peak_memory)
    let current_memory: usize = alloc_vec.iter().map(|a| a.size).sum();
    let peak_memory = current_memory;  // Known limitation

    AnalysisReport {
        hotspots,
        system_snapshots,
        total_allocations: alloc_vec.len(),
        total_memory: current_memory,
        peak_memory,
    }
}
```

### Analysis Report

```rust
pub struct AnalysisReport {
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
    pub total_allocations: usize,
    pub total_memory: usize,
    pub peak_memory: usize,
}

pub struct AllocationHotspot {
    pub file: String,
    pub line: u32,
    pub function: String,
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub average_size: f64,
}
```

---

## Auto-Export on Drop

```rust
impl Drop for Tracker {
    fn drop(&mut self) {
        if let Ok(cfg) = self.config.lock() {
            if cfg.auto_export_on_drop {
                if let Some(ref path) = cfg.export_path {
                    // Export to JSON
                    export_snapshot_to_json(&snapshot, Path::new(path), &options);
                }
            }
        }
    }
}
```

When the tracker is dropped, it automatically exports results if configured. **Note:** This performs blocking I/O in `Drop`, which can be slow and may cause issues during panic unwinding.

---

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `track!()` | O(1) | HashMap insert |
| `analyze()` | O(n) | Iterates all allocations |
| System monitoring | Background thread | ~1ms per sample interval |
| Auto-export | O(n) | Full snapshot serialization |

---

## When to Use

- **Quick scripts** — When you don't need the full 9-engine pipeline
- **Simple programs** — Single-threaded, low-complexity applications
- **System monitoring** — When you need CPU/memory/disk metrics alongside memory tracking
- **Hotspot analysis** — When you want to identify which source lines allocate the most
