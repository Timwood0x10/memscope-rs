# Compile-time Semantic Enhancement

## Overview

Phase 0 is the core feature enhancement stage of memscope-rs, providing accurate memory layout information, lifecycle tracking, and intelligent reports through compile-time macro injection and runtime analysis.

## Completed Features

### P0.1 Compile-time Macro Injection

**Goal**: Add semantic anchors (filename, line number, module path) to the `track!` macro

**Implementation**:
- Use `file!()`, `line!()`, `module_path!()` macros
- Record source code location information in `MemoryEvent`
- Support library path filtering to distinguish user code from library code

**Effects**:
- Precise source code location
- Support filtering Rust standard library paths
- Module-level memory analysis

### P0.2 Accurate Type Layout Calculation

**Goal**: Use `std::mem::Layout::from_val` to get accurate memory layout information

**Implementation**:
- Call `std::mem::size_of_val` and `std::mem::align_of_val` in `track!` macro
- Record precise allocation size and alignment information
- Support recursive layout calculation for complex types

**Effects**:
- Accurate memory size calculation
- Correct alignment information
- Support generic types and custom types

### P0.3 Timestamp-based Lifecycle Analysis

**Goal**: Implement timestamp-based memory lifecycle tracking

**Implementation**:
- Use `std::time::Instant` to record allocation timestamp
- Automatically record deallocation timestamp on Tracker drop
- Calculate lifecycle (millisecond precision)

**Effects**:
- Automatic lifecycle tracking without manual operation
- Support temporary object detection (short lifecycle)
- Support long-lived object analysis
- Lifecycle distribution statistics

### P0.4 Sampling Mode

**Goal**: Implement time-based or event-based sampling mechanism

**Implementation**:
- `SamplingConfig` to configure sampling rate
- Support time interval sampling
- Support event count sampling
- Zero-overhead sampling (read-only atomic values)

**Effects**:
- Reduce performance overhead
- Support production deployment
- Configurable sampling strategies

### P0.5 Top N Reports

**Goal**: Implement Top N allocation reports based on memory usage

**Implementation**:
- `TopNAnalyzer` analyzer
- Top allocation sites (sorted by memory usage)
- Top leaked allocations (sorted by leaked memory)
- Top temporary churn (sorted by allocation frequency)

**Effects**:
- Quickly locate memory hotspots
- Optimization suggestions
- Visualized reports

### P0.6 HTML Report Integration

**Goal**: Leverage existing render_engine/dashboard capabilities to integrate Top N report display and module_path information

**Implementation**:
- Extend `DashboardContext` to include Top N reports
- Display module path information in HTML dashboard
- Integrate lifecycle analysis results

**Effects**:
- Interactive visualization
- Module-level analysis
- Lifecycle heatmaps

## Usage Examples

### Basic Usage

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    // Automatically capture type information, timestamp, module path
    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    // Analysis results
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    println!("Total allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);

    Ok(())
}
```

### With Sampling

```rust
use memscope_rs::{tracker, tracker::SamplingConfig, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    let tracker = tracker!()
        .with_sampling(SamplingConfig::high_performance());

    // Only 1% of allocations will be recorded
    for i in 0..10000 {
        let data = vec![i; 100];
        track!(tracker, data);
    }

    Ok(())
}
```

### Export HTML Report

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    // Export HTML dashboard
    tracker.render_unified_dashboard("output/dashboard.html")?;

    Ok(())
}
```

### Lifecycle Analysis

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    {
        let temporary_data = vec![1, 2, 3];
        track!(tracker, temporary_data);
        // temporary_data goes out of scope here, lifecycle auto-recorded
    }

    let long_lived_data = vec![1; 1000];
    track!(tracker, long_lived_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();

    // View lifecycle statistics
    for alloc in &report.allocations {
        println!("{}: lifetime_ms = {}", alloc.type_name, alloc.lifetime_ms);
    }

    Ok(())
}
```

## Data Flow

```
track! macro
  ↓
MemoryEvent (includes file, line, module_path, timestamp)
  ↓
event_store (unified data source)
  ↓
rebuild_allocations_from_events (single processing flow)
  ↓
AllocationInfo (includes type layout, lifecycle, module_path)
  ↓
Analyzers (TopN, lifecycle, circular reference detection)
  ↓
DashboardContext
  ↓
HTML Dashboard
```

## Performance Features

- **Zero-overhead sampling**: Read-only atomic values, no blocking
- **Compile-time calculation**: Type layout calculated at compile time
- **Automatic lifecycle tracking**: No manual operation required
- **Unified data source**: All data from event_store
