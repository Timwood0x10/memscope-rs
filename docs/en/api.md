# API Documentation

## Overview

memscope-rs provides a simple and unified API for memory tracking, analysis, and visualization.

## Core Macros

### `track!` Macro

Track memory allocation for a variable.

```rust
track!(tracker, variable);
```

**Parameters**:
- `tracker`: Tracker instance
- `variable`: Variable to track

**Features**:
- Auto-capture type information
- Record allocation timestamp
- Record source location (file, line, module path)
- Auto-detect smart pointer types
- Auto-track lifecycle

**Example**:
```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    Ok(())
}
```

### `tracker!` Macro

Create a new Tracker instance.

```rust
let tracker = tracker!();
```

**Returns**: New Tracker instance

## Tracker API

### Initialization

#### `init_global_tracking()`

Initialize the global tracker.

```rust
fn init_global_tracking() -> MemScopeResult<()>
```

**Returns**: `MemScopeResult<()>`

**Example**:
```rust
use memscope_rs::init_global_tracking;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    Ok(())
}
```

#### `global_tracker()`

Get the global tracker instance.

```rust
fn global_tracker() -> MemScopeResult<Tracker>
```

**Returns**: Global Tracker instance

**Example**:
```rust
use memscope_rs::{global_tracker, init_global_tracking};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;
    Ok(())
}
```

### Configuration

#### `with_sampling()`

Set sampling configuration.

```rust
fn with_sampling(self, config: SamplingConfig) -> Self
```

**Parameters**:
- `config`: Sampling configuration

**Example**:
```rust
use memscope_rs::{tracker, tracker::SamplingConfig};

let tracker = tracker!().with_sampling(SamplingConfig::demo());
```

#### `with_system_monitoring()`

Enable system monitoring.

```rust
fn with_system_monitoring(self) -> Self
```

**Example**:
```rust
use memscope_rs::tracker;

let tracker = tracker!().with_system_monitoring();
```

#### `with_auto_export()`

Set auto-export path.

```rust
fn with_auto_export(self, path: &str) -> Self
```

**Parameters**:
- `path`: Export path

**Example**:
```rust
use memscope_rs::tracker;

let tracker = tracker!().with_auto_export("output");
```

### Tracking Methods

#### `track_as()`

Track memory allocation for a variable.

```rust
fn track_as<T: Trackable>(
    &self,
    var: &T,
    name: &str,
    file: &str,
    line: u32,
    module_path: &str,
) -> usize
```

**Parameters**:
- `var`: Variable to track
- `name`: Variable name
- `file`: File name
- `line`: Line number
- `module_path`: Module path

**Returns**: Pointer address

**Note**: Usually called through `track!` macro, not used directly.

### Analysis Methods

#### `analyze()`

Perform complete memory analysis.

```rust
fn analyze(&self) -> AnalysisReport
```

**Returns**: Analysis report

**Example**:
```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    
    Ok(())
}
```

### Export Methods

#### `export_all_json()`

Export all analysis results as JSON.

```rust
fn export_all_json(&self, path: impl AsRef<Path>) -> MemScopeResult<()>
```

**Parameters**:
- `path`: Export path

**Example**:
```rust
use memscope_rs::tracker;

let tracker = tracker!();
tracker.export_all_json("output")?;
```

#### `render_unified_dashboard()`

Render unified HTML dashboard.

```rust
fn render_unified_dashboard(&self, path: impl AsRef<Path>) -> MemScopeResult<()>
```

**Parameters**:
- `path`: Output path

**Example**:
```rust
use memscope_rs::tracker;

let tracker = tracker!();
tracker.render_unified_dashboard("output/dashboard.html")?;
```

### Statistics Methods

#### `get_stats()`

Get current statistics.

```rust
fn get_stats(&self) -> TrackerStats
```

**Returns**: Tracker statistics

**Example**:
```rust
use memscope_rs::tracker;

let tracker = tracker!();
let stats = tracker.get_stats();
println!("Total allocations: {}", stats.total_allocations);
```

## SamplingConfig

Sampling configuration.

```rust
pub struct SamplingConfig {
    pub sample_rate: f64,
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}
```

### Preset Configurations

#### `default()`

Default configuration (100% sample rate).

```rust
SamplingConfig::default()
```

#### `demo()`

Demo configuration (10% sample rate).

```rust
SamplingConfig::demo()
```

#### `full()`

Full configuration (100% sample rate, capture call stack).

```rust
SamplingConfig::full()
```

#### `high_performance()`

High performance configuration (1% sample rate).

```rust
SamplingConfig::high_performance()
```

## Analyzer API

### `analyzer()`

Create analyzer.

```rust
fn analyzer(tracker: &Tracker) -> MemScopeResult<Analyzer>
```

**Parameters**:
- `tracker`: Tracker instance

**Returns**: Analyzer instance

**Example**:
```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let mut az = analyzer(&tracker)?;
    Ok(())
}
```

### Analysis Methods

#### `analyze()`

Perform complete analysis.

```rust
fn analyze(&mut self) -> AnalysisReport
```

**Returns**: Analysis report

#### `top_allocation_sites()`

Get top N allocation sites.

```rust
fn top_allocation_sites(&mut self, n: usize) -> Vec<TopAllocationSite>
```

**Parameters**:
- `n`: Number to return

**Returns**: List of top allocation sites

#### `top_leaked_allocations()`

Get top N leaked allocations.

```rust
fn top_leaked_allocations(&mut self, n: usize) -> Vec<TopLeakedAllocation>
```

**Parameters**:
- `n`: Number to return

**Returns**: List of top leaked allocations

#### `top_temporary_churn()`

Get top N temporary churn.

```rust
fn top_temporary_churn(&mut self, n: usize) -> Vec<TopTemporaryChurn>
```

**Parameters**:
- `n`: Number to return

**Returns**: List of top temporary churn

## Data Types

### AnalysisReport

Analysis report.

```rust
pub struct AnalysisReport {
    pub stats: AnalysisStats,
    pub hotspots: Vec<AllocationHotspot>,
    pub leaks: Vec<LeakInfo>,
}
```

### AnalysisStats

Analysis statistics.

```rust
pub struct AnalysisStats {
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub thread_count: usize,
}
```

### CircularReferenceReport

Circular reference report.

```rust
pub struct CircularReferenceReport {
    pub count: usize,
    pub total_leaked_memory: usize,
    pub pointers_in_cycles: usize,
    pub total_smart_pointers: usize,
    pub has_cycles: bool,
}
```

## Usage Examples

### Basic Usage

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(tracker, data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Allocations: {}", report.stats.allocation_count);
    println!("Total bytes: {}", report.stats.total_bytes);
    
    Ok(())
}
```

### Smart Pointer Tracking

```rust
use memscope_rs::{analyzer, global_tracker, init_global_tracking, track};
use std::rc::Rc;
use std::sync::Arc;

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(tracker, arc_data);

    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    
    println!("Smart pointers: {}", 
        report.circular_references.total_smart_pointers);
    println!("Circular refs: {}", 
        report.circular_references.count);
    
    Ok(())
}
```

### Export Reports

```rust
use memscope_rs::{global_tracker, init_global_tracking, track};

fn main() -> memscope_rs::MemScopeResult<()> {
    init_global_tracking()?;
    let tracker = global_tracker()?;

    let data = vec![1, 2, 3];
    track!(tracker, data);

    tracker.export_all_json("output")?;
    tracker.render_unified_dashboard("output/dashboard.html")?;
    
    Ok(())
}
```

## Performance Features

- **Zero-overhead sampling**: Read-only atomic values, no blocking
- **Compile-time calculation**: Type layout calculated at compile time
- **Automatic lifecycle tracking**: No manual operation required
- **Unified data source**: All data from event_store
