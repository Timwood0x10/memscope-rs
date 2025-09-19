# Multi-Thread Memory Tracing API Documentation

## Overview

The memscope-rs multi-thread memory tracing system provides comprehensive memory allocation tracking, performance monitoring, and interactive visualization for Rust applications. It features selective thread tracking, real-time resource monitoring, and advanced visualization with correlation analysis.

## Key Features

### 🧵 **Selective Thread Tracking**
- Track specific threads based on custom criteria (e.g., even thread IDs)
- Zero-overhead for untracked threads
- Real-time thread activity monitoring

### 📊 **Comprehensive Resource Monitoring**
- CPU usage tracking per core
- Memory allocation/deallocation patterns
- I/O operations monitoring  
- GPU resource utilization (when available)

### 🎯 **Interactive Visualization**
- Role-based thread classification (Memory Intensive, CPU Intensive, I/O Intensive, Balanced, Lightweight)
- Focus mode with visual transitions
- Resource correlation scatter plots
- Cross-tab intelligent linking

### 📈 **Performance Analysis**
- Thread performance rankings
- Memory usage patterns
- CPU-Memory correlation analysis
- Bottleneck identification

## Trackable Data Types

### 🧵 **Per-Thread Memory Data**
- **Allocation Events**: Memory allocation size, timestamp, call stack
- **Deallocation Events**: Memory release operations and patterns
- **Peak Memory Usage**: Maximum memory consumption per thread (e.g., 15.9MB - 22.8MB for tracked threads)
- **Allocation Frequency**: Number of operations per second (e.g., 1020-1480 allocations per thread)
- **Memory Efficiency**: Allocation/deallocation ratio (50-80% typical range)
- **Call Stack Traces**: Function call hierarchy for memory operations (up to 10 levels deep)

### 💻 **System Resource Metrics**
- **CPU Usage**: Per-core utilization (0-100% per core, 14 cores typical)
- **Overall CPU Load**: System-wide CPU consumption (10.6% average in verified demo)
- **System Load Average**: 1-minute, 5-minute, 15-minute load averages
- **Memory Pressure**: Total system memory usage and available memory
- **Thread Count**: Active thread monitoring (25 tracked + 25 untracked in demo)

### ⏱️ **Temporal Performance Data**
- **Resource Timeline**: Real-time sampling at 10Hz (100ms intervals)
- **Allocation Patterns**: Memory allocation distribution over time
- **CPU Usage Trends**: CPU utilization changes during execution
- **Thread Lifecycle**: Thread creation, execution, and termination phases
- **Performance Bottlenecks**: Identification of resource contention points

### 🎮 **Hardware Resource Utilization**
- **GPU Metrics**: GPU device detection and utilization (when available)
- **I/O Operations**: Estimated I/O activity based on memory patterns
- **Network Activity**: TCP/UDP data transfer detection
- **Disk Activity**: File system read/write operations

### 📊 **Advanced Analytics Data**
- **Thread Role Classification**: Automatic categorization based on resource usage patterns
- **Resource Correlation**: CPU vs Memory allocation rate relationships
- **Anomaly Detection**: High resource usage identification with visual alerts
- **Performance Efficiency Scoring**: Multi-dimensional performance evaluation

## Visualization Capabilities

### 🎯 **Interactive Multi-Thread Overview**
- **Thread Cards Display**: Visual representation of all tracked threads
- **Role-Based Color Coding**: 
  - 💾 Memory Intensive (Red): >18MB peak memory + >1200 allocations
  - 🔥 CPU Intensive (Orange): >25% CPU usage
  - ⚡ I/O Intensive (Blue): >2000 I/O operations
  - 🧵 Balanced (Green): 1000+ allocations, moderate resource usage
  - 💤 Lightweight (Gray): Minimal resource consumption
- **Alert System**: 
  - High Alert (Red pulse): >20MB memory or >30% CPU
  - Medium Alert (Orange): >15MB memory or >20% CPU
  - Normal (Green): Standard resource usage

### 📈 **Performance Analytics Dashboard**
- **Memory-Sorted Rankings**: Threads ordered by peak memory usage (highest to lowest)
- **Resource Timeline**: 32 real-time samples showing CPU, memory, and I/O over time
- **Correlation Scatter Plot**: 
  - X-axis: CPU Usage (0-40%)
  - Y-axis: Memory Allocation Rate (0-50 MB/s)
  - Color intensity: I/O operation frequency
  - Trend lines: Automatic correlation analysis with Pearson coefficient

### 🔍 **Focus Mode Analysis**
- **Thread Isolation**: Click any thread to enter focused analysis mode
- **Visual Transitions**: Selected thread scales 115%, others fade to 30% opacity
- **Cross-Tab Filtering**: All dashboard tabs automatically filter to selected thread
- **Deep Correlation Analysis**: Dedicated scatter plot for CPU-Memory-I/O relationships

### 📊 **System Resource Overview**
- **CPU Performance Metrics**: 
  - Average CPU: 10.6% (verified demo result)
  - Peak CPU: Individual core utilization tracking
  - Core count: 14 cores (example system)
  - CPU efficiency scoring: 40% (calculated metric)
- **Memory Performance Summary**:
  - Active tracked threads: 25 threads
  - Total allocations: 31,000 operations
  - Total peak memory: 248.2 MB
  - Memory efficiency: 50% average
- **Bottleneck Analysis**: Automatic identification of system constraints

## Core API

### Initialization

```rust
use memscope_rs::lockfree::{
    init_thread_tracker, 
    finalize_thread_tracker,
    track_allocation_lockfree,
    track_deallocation_lockfree,
    SamplingConfig
};

// Initialize tracking for current thread
init_thread_tracker(&output_dir, Some(SamplingConfig::demo()))?;
```

### Memory Tracking

```rust
// Track memory allocation
track_allocation_lockfree(ptr_address, size, &call_stack)?;

// Track memory deallocation  
track_deallocation_lockfree(ptr_address, &call_stack)?;

// Finalize tracking
finalize_thread_tracker()?;
```

### Platform Resource Monitoring

```rust
use memscope_rs::lockfree::{
    PlatformResourceCollector,
    IntegratedProfilingSession
};

// Initialize platform monitoring
let resource_collector = PlatformResourceCollector::new()?;
let mut session = IntegratedProfilingSession::new(
    output_dir.clone(), 
    resource_collector
)?;

// Start monitoring
session.start_monitoring()?;

// Stop and collect data
session.stop_monitoring()?;
```

### Data Export and Visualization

```rust
use memscope_rs::lockfree::export_comprehensive_analysis;

// Export comprehensive analysis with interactive dashboard
export_comprehensive_analysis(
    &output_dir,
    "platform_demo",  // prefix
    None              // custom config
)?;
```

## Generated Outputs

### JSON Data Files
- **`platform_demo_comprehensive.json`**: Complete analysis data (8.9MB)
- **`platform_demo_resource_rankings.json`**: Thread performance rankings (28KB)

### HTML Dashboard
- **`platform_demo_dashboard.html`**: Interactive visualization dashboard (117KB)

### Binary Trace Files
- **`memscope_thread_*.bin`**: Raw allocation data per thread
- **`memscope_thread_*.freq`**: Allocation frequency data

## Dashboard Features

### Multi-Thread Overview
- **25 tracked threads** displayed as interactive cards
- **Role classification**: Memory Intensive 💾, CPU Intensive 🔥, I/O Intensive ⚡, Balanced 🧵, Lightweight 💤
- **Visual alerts**: Color-coded performance indicators with pulse animations for high-usage threads
- **Click-to-focus**: Select any thread card to enter focus mode

### Thread Performance Details  
- **Memory-sorted rankings**: Threads ordered by peak memory usage
- **Performance metrics**: CPU usage, memory allocation rate, I/O operations
- **Efficiency scoring**: Based on allocation/deallocation ratios

### Resource Timeline
- **32 real-time samples** at 10Hz sampling rate
- **CPU usage tracking**: Per-core utilization and system load
- **Memory activity**: Allocation patterns over time
- **Thread activity**: Active thread count monitoring

### System Summary
- **Performance insights**: CPU efficiency (40%), Memory efficiency (50%)
- **Bottleneck analysis**: Automatic bottleneck type detection
- **Resource correlation**: CPU vs Memory allocation analysis

## Advanced Interaction Features

### Focus Mode
```javascript
// Click any thread card to enter focus mode
selectThread(threadId);

// Background click to exit focus mode
handleBackgroundClick();
```

**Visual Effects:**
- Selected thread scales to 115% with elevation
- Other threads fade to 30% opacity
- Page background dims for focus
- Cross-tab content filtering

### Correlation Analysis
- **Scatter Plot**: CPU usage vs Memory allocation rate
- **I/O Mapping**: Point color intensity represents I/O operations
- **Trend Analysis**: Automatic correlation coefficient calculation
- **Pattern Recognition**: Compute-intensive vs Data-movement patterns

## Example Usage

### Complete Example (verified_selective_demo.rs)

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = PathBuf::from("./Memoryanalysis");
    
    // Initialize platform monitoring
    let resource_collector = PlatformResourceCollector::new()?;
    let mut session = IntegratedProfilingSession::new(
        output_dir.clone(),
        resource_collector
    )?;
    
    session.start_monitoring()?;
    
    // Launch 50 threads with selective tracking
    let handles: Vec<_> = (0..50).map(|i| {
        let output_dir = output_dir.clone();
        thread::spawn(move || {
            run_enhanced_verified_worker(i, &output_dir)
        })
    }).collect();
    
    // Wait for completion
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    session.stop_monitoring()?;
    
    // Export comprehensive analysis
    export_comprehensive_analysis(&output_dir, "platform_demo", None)?;
    
    println!("🌐 Open ./Memoryanalysis/platform_demo_dashboard.html");
    Ok(())
}
```

### Thread Worker Implementation

```rust
fn run_enhanced_verified_worker(
    thread_idx: usize, 
    output_dir: &Path
) -> Result<(), String> {
    let should_track = thread_idx % 2 == 0; // Track even threads only
    
    if should_track {
        // Initialize tracking
        init_thread_tracker(output_dir, Some(SamplingConfig::demo()))?;
        
        // Perform tracked allocations
        for i in 0..operation_count {
            let size = calculate_allocation_size(thread_idx, i);
            let ptr = generate_pointer_address(thread_idx, i);
            let call_stack = capture_call_stack();
            
            track_allocation_lockfree(ptr, size, &call_stack)?;
            
            // Simulate work
            perform_memory_intensive_work(size);
            
            // Periodic deallocation
            if should_deallocate(i) {
                track_deallocation_lockfree(old_ptr, &old_stack)?;
            }
        }
        
        finalize_thread_tracker()?;
    } else {
        // Untracked thread - baseline performance
        simulate_untracked_work(thread_idx);
    }
    
    Ok(())
}
```

## Configuration Options

### SamplingConfig

```rust
pub struct SamplingConfig {
    pub allocation_sampling_rate: f64,    // 0.0-1.0
    pub stack_depth: usize,               // Call stack depth
    pub enable_frequency_tracking: bool,   // Track allocation frequencies
}

impl SamplingConfig {
    pub fn demo() -> Self {
        Self {
            allocation_sampling_rate: 1.0,  // Track all allocations
            stack_depth: 10,
            enable_frequency_tracking: true,
        }
    }
}
```

## Performance Characteristics

### Tracking Overhead
- **Tracked threads**: ~5-10% overhead per allocation
- **Untracked threads**: Zero overhead
- **Memory usage**: ~50-80KB per tracked thread

### Scalability
- **Thread capacity**: 100+ concurrent tracked threads
- **Allocation rate**: 10,000+ allocations/second per thread
- **Data export**: Supports GB-scale datasets

### System Requirements
- **OS**: macOS, Linux (with platform-specific optimizations)
- **Memory**: 100MB+ for comprehensive analysis
- **Storage**: ~1MB per 1000 allocations

## Best Practices

### 1. Selective Tracking Strategy
```rust
// Example: Track only performance-critical threads
let should_track = thread_name.contains("worker") || 
                   thread_name.contains("compute");
```

### 2. Sampling Configuration
```rust
// For production: reduce sampling rate
let config = SamplingConfig {
    allocation_sampling_rate: 0.1,  // Sample 10% of allocations
    stack_depth: 5,                 // Shallow stack traces
    enable_frequency_tracking: false,
};
```

### 3. Data Export Timing
```rust
// Export after thread completion for accuracy
for handle in handles {
    handle.join().unwrap()?;
}
// All threads completed - safe to export
export_comprehensive_analysis(&output_dir, "analysis", None)?;
```

## Troubleshooting

### Common Issues

**1. Missing thread data in dashboard**
- Ensure `finalize_thread_tracker()` is called before export
- Check that thread completed before export
- Verify output directory permissions

**2. High memory usage during tracking**
- Reduce `allocation_sampling_rate` 
- Decrease `stack_depth`
- Implement periodic data flushing

**3. Dashboard performance issues**
- Large datasets (>10MB JSON) may cause browser lag
- Consider data filtering or pagination for massive datasets

### Debug Mode
```rust
// Enable detailed logging
std::env::set_var("MEMSCOPE_DEBUG", "1");
init_thread_tracker(&output_dir, Some(config))?;
```

## API Reference Summary

| Function | Purpose | Parameters |
|----------|---------|------------|
| `init_thread_tracker` | Initialize tracking | `output_dir`, `config` |
| `track_allocation_lockfree` | Track allocation | `ptr`, `size`, `call_stack` |
| `track_deallocation_lockfree` | Track deallocation | `ptr`, `call_stack` |
| `finalize_thread_tracker` | Finalize tracking | None |
| `export_comprehensive_analysis` | Generate reports | `output_dir`, `prefix`, `config` |
| `PlatformResourceCollector::new` | Initialize monitoring | None |
| `IntegratedProfilingSession::new` | Create session | `output_dir`, `collector` |

## Version Information

- **Version**: 0.1.5
- **Rust Edition**: 2021
- **Dependencies**: sysinfo, serde, chrono
- **License**: MIT/Apache-2.0

---

For more examples and advanced usage, see the `examples/` directory in the repository.