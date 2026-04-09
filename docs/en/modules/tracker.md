# Tracker Module

## Overview

The tracker module provides a unified, simple API for memory tracking in Rust applications. It offers a high-level interface that abstracts away the complexity of memory tracking while providing powerful analysis capabilities.

## Features

- **Simple API**: Easy-to-use `tracker!()` and `track!()` macros
- **Auto-capture**: Automatic variable name and type capture
- **System Monitoring**: Background CPU, memory, disk, and network monitoring with zero overhead
- **Per-thread Tracking**: Independent tracking per thread
- **Sampling**: Configurable sampling rates for performance optimization
- **Hotspot Analysis**: Automatic detection of allocation hotspots
- **HTML Dashboard**: Interactive visualization of memory data
- **Multiple Export Formats**: JSON, HTML, and binary export support

## Architecture

### System Monitoring

System monitoring runs in a background thread that collects metrics every 100ms. The `track!()` macro only reads atomic values (nanosecond overhead), ensuring no blocking on data collection.

**Key Points**:
- Non-blocking: Metrics collection doesn't block tracking operations
- Low overhead: Only atomic reads in the hot path
- Thread-safe: All metrics are thread-safe

### Tracking Flow

```
User Code
   ↓
track!() macro
   ↓
Tracker::track_as()
   ↓
MemoryTracker::track_allocation()
   ↓
EventStore::record()
   ↓
Analysis & Export
```

## Core Components

### 1. Tracker

**Purpose**: Main tracking interface that combines memory tracking, event storage, and system monitoring.

**Source Code**:

```rust
pub struct Tracker {
    inner: Arc<MemoryTracker>,
    event_store: Arc<EventStore>,
    config: Arc<Mutex<TrackerConfig>>,
    start_time: Instant,
    system_snapshots: Arc<Mutex<Vec<SystemSnapshot>>>,
}
```

**Key Methods**:

```rust
impl Tracker {
    pub fn new() -> Self
    pub fn global() -> Self
    pub fn with_system_monitoring(self) -> Self
    pub fn with_sampling(self, config: SamplingConfig) -> Self
    pub fn with_auto_export(self, path: &str) -> Self
    pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32)
    pub fn analyze(&self) -> AnalysisReport
    pub fn stats(&self) -> MemoryStats
    pub fn events(&self) -> Vec<MemoryEvent>
    pub fn current_system_snapshot(&self) -> SystemSnapshot
}
```

**Creation**:

```rust
// Basic tracker
let tracker = Tracker::new();

// With system monitoring
let tracker = Tracker::new().with_system_monitoring();

// With sampling
let tracker = Tracker::new()
    .with_sampling(SamplingConfig::high_performance());

// With auto-export
let tracker = Tracker::new()
    .with_auto_export("./output/memscope.json");
```

### 2. SamplingConfig

**Purpose**: Configure sampling behavior to balance between tracking completeness and performance.

**Source Code**:

```rust
pub struct SamplingConfig {
    pub sample_rate: f64,           // 0.0 to 1.0
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}
```

**Presets**:

```rust
impl SamplingConfig {
    pub fn demo() -> Self {
        Self {
            sample_rate: 0.1,         // 10% sampling
            capture_call_stack: false,
            max_stack_depth: 5,
        }
    }

    pub fn full() -> Self {
        Self {
            sample_rate: 1.0,         // 100% sampling
            capture_call_stack: true,
            max_stack_depth: 20,
        }
    }

    pub fn high_performance() -> Self {
        Self {
            sample_rate: 0.01,        // 1% sampling
            capture_call_stack: false,
            max_stack_depth: 0,
        }
    }
}
```

**Sampling Algorithm**:

```rust
pub fn track_as<T: Trackable>(&self, var: &T, name: &str, file: &str, line: u32) {
    if let Ok(cfg) = self.config.lock() {
        if cfg.sampling.sample_rate < 1.0 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();

            // Use timestamp for randomness
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            timestamp.hash(&mut hasher);
            std::thread::current().id().hash(&mut hasher);
            name.hash(&mut hasher);
            file.hash(&mut hasher);
            line.hash(&mut hasher);

            let hash = hasher.finish();
            let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;

            if (hash % 1000) > threshold {
                return; // Skip this tracking
            }
        }
    }

    self.track_inner(var, name, file, line);
}
```

**Design Philosophy**:

1. **Deterministic sampling**: Hash-based sampling provides consistent behavior
2. **Low overhead**: Hash computation is fast and doesn't require external randomness
3. **Thread-safe**: Each thread gets independent sampling decisions
4. **Configurable**: Easy to adjust sampling rate for different scenarios

### 3. AnalysisReport

**Purpose**: Comprehensive analysis of memory tracking data.

**Source Code**:

```rust
pub struct AnalysisReport {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: u64,
    pub current_memory_bytes: u64,
    pub allocation_rate_per_sec: f64,
    pub deallocation_rate_per_sec: f64,
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
}
```

**Allocation Hotspot**:

```rust
pub struct AllocationHotspot {
    pub var_name: String,
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub location: Option<String>,
}
```

### 4. SystemSnapshot

**Purpose**: Captures system metrics at a point in time.

**Source Code**:

```rust
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub thread_count: usize,
    pub disk_read_bps: u64,
    pub disk_write_bps: u64,
    pub network_rx_bps: u64,
    pub network_tx_bps: u64,
    pub gpu_usage_percent: f64,
    pub gpu_memory_used: u64,
    pub gpu_memory_total: u64,
}
```

## Macros

### tracker!()

Creates a new tracker instance.

**Syntax**:

```rust
let tracker = tracker!();
```

**Equivalent to**:

```rust
let tracker = Tracker::new();
```

### track!()

Tracks a variable with automatic name capture.

**Syntax**:

```rust
track!(tracker, variable_name);
```

**Equivalent to**:

```rust
tracker.track_as(&variable_name, "variable_name", file!(), line!());
```

**Example**:

```rust
let tracker = tracker!();

let my_vec = vec![1, 2, 3, 4, 5];
track!(tracker, my_vec);

let my_string = String::from("Hello");
track!(tracker, my_string);

let my_map: HashMap<i32, String> = HashMap::new();
track!(tracker, my_map);
```

## Usage Examples

### Basic Usage

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!();

    // Track various types
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let string_data = String::from("Hello, world!");
    track!(tracker, string_data);

    // Analyze the tracked allocations
    let report = tracker.analyze();
    println!("Total allocations: {}", report.total_allocations);
    println!("Active allocations: {}", report.active_allocations);
    println!("Peak memory: {} bytes", report.peak_memory_bytes);
}
```

### With System Monitoring

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!().with_system_monitoring();

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // Get current system snapshot
    let snapshot = tracker.current_system_snapshot();
    println!("CPU usage: {:.2}%", snapshot.cpu_usage_percent);
    println!("Memory usage: {:.2}%", snapshot.memory_usage_percent);

    // Get analysis report with system data
    let report = tracker.analyze();
    println!("System snapshots: {}", report.system_snapshots.len());
}
```

### With Sampling

```rust
use memscope_rs::{tracker, track, SamplingConfig};

fn main() {
    // High-performance mode with 1% sampling
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    // In a loop with many allocations
    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    let report = tracker.analyze();
    println!("Tracked allocations: {}", report.total_allocations);
}
```

### With Auto-Export

```rust
use memscope_rs::{tracker, track};

fn main() {
    let tracker = tracker!()
        .with_auto_export("./output/memory_report.json");

    let data = vec![1, 2, 3];
    track!(tracker, data);

    // Export happens automatically when tracker goes out of scope
}
```

### Multi-threaded Tracking

```rust
use memscope_rs::{tracker, track};
use std::thread;

fn main() {
    let tracker = tracker!();

    let handles: Vec<_> = (0..4).map(|id| {
        let tracker = tracker.clone();
        thread::spawn(move || {
            for i in 0..100 {
                let data = vec![i; 16];
                track!(tracker, data);
            }
        })
    }).collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let report = tracker.analyze();
    println!("Total allocations: {}", report.total_allocations);
}
```

## Design Philosophy

### 1. Simplicity First
The API is designed to be as simple as possible:
- Two macros: `tracker!()` and `track!()`
- Automatic variable name capture
- Minimal boilerplate

### 2. Zero Overhead
Performance is a primary concern:
- Atomic operations only (no locks in hot path)
- Optional sampling to reduce overhead
- Non-blocking system monitoring

### 3. Type Safety
Leverages Rust's type system:
- Generic `Trackable` trait
- Compile-time type checking
- No runtime type information needed

### 4. Thread Safety
All operations are thread-safe:
- `Arc` for shared state
- `Mutex` for write-heavy data
- Atomic operations for counters

## Performance Considerations

### Sampling Overhead

| Sampling Rate | Overhead | Use Case |
|---------------|----------|----------|
| 1.0 (100%) | ~5% | Development, debugging |
| 0.1 (10%) | ~1% | Testing, profiling |
| 0.01 (1%) | ~0.1% | Production monitoring |

### System Monitoring Overhead

- **Background thread**: Runs every 100ms
- **Hot path**: Only atomic reads (nanosecond overhead)
- **Memory**: ~1KB per snapshot

### Memory Overhead

- **Per allocation**: ~100 bytes
- **Event storage**: Grows with tracked allocations
- **System snapshots**: Configurable, ~1KB each

## Integration

The tracker module integrates with other modules:

```
tracker.rs
  ↓
core/         (MemoryTracker)
  ↓
event_store/  (EventStore)
  ↓
capture/      (SystemMonitor)
  ↓
render/       (Export functions)
```

## Best Practices

1. **Create tracker once**: Initialize tracker at program startup
2. **Use sampling in production**: Reduce overhead with sampling
3. **Enable system monitoring**: Get comprehensive metrics
4. **Auto-export**: Export data automatically on drop
5. **Clone for threads**: Use `tracker.clone()` for multi-threading

## Limitations

1. **Stack variables**: Only heap allocations are tracked
2. **Static variables**: Static allocations are not tracked
3. **External memory**: Memory allocated by external libraries may not be tracked
4. **Type inference**: Limited to types that implement `Trackable` trait

## Testing

```rust
#[test]
fn test_tracker_creation() {
    let tracker = Tracker::new();
    let _ = tracker;
}

#[test]
fn test_track_macro() {
    let tracker = tracker!();
    let my_vec = vec![1, 2, 3];
    track!(tracker, my_vec);
}

#[test]
fn test_analyze() {
    let tracker = tracker!();
    let data = vec![1, 2, 3];
    track!(tracker, data);
    let report = tracker.analyze();
    assert!(report.total_allocations > 0);
}
```

## Future Improvements

1. **Better type inference**: Integration with compiler for accurate type information
2. **Call stack capture**: Improved call stack tracking with less overhead
3. **Real-time monitoring**: Web-based real-time dashboard
4. **Advanced analysis**: More sophisticated analysis algorithms
5. **Plugin system**: Allow custom analyzers and exporters