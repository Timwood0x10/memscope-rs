# Platform Resource Monitoring

This document describes the new platform-specific resource monitoring capabilities added to the lockfree module, supporting macOS and Linux environments in multi-threaded scenarios.

## Overview

The platform resource monitoring system extends memscope's capabilities beyond memory tracking to include:

- **CPU utilization**: Per-core usage, frequencies, temperatures, and load averages
- **GPU monitoring**: Compute usage, memory utilization, and thermal metrics
- **IO subsystem**: Disk and network throughput monitoring
- **Thread-level metrics**: Resource consumption per thread

## Architecture

The system consists of three main components:

### 1. Platform Resource Collector (`platform_resources.rs`)

Cross-platform abstraction layer that provides unified API for resource collection:

```rust
use memscope::lockfree::PlatformResourceCollector;

let mut collector = PlatformResourceCollector::new()?;
let metrics = collector.collect_metrics()?;
```

**Supported Platforms:**
- **macOS**: Uses mach syscalls for CPU metrics, Metal/IOKit for GPU monitoring
- **Linux**: Uses /proc filesystem for CPU/thread data, nvidia-smi for GPU monitoring

### 2. Resource Integration (`resource_integration.rs`)

Combines memory tracking with system resource monitoring:

```rust
use memscope::lockfree::comprehensive_profile_execution;

let (result, analysis) = comprehensive_profile_execution(&output_dir, || {
    // Your workload here
    perform_work()
})?;
```

**Key Features:**
- Correlation analysis between memory operations and system resources
- Performance bottleneck identification
- Thread efficiency ranking
- Automated performance recommendations

### 3. Integrated Profiling Session

For long-running monitoring scenarios:

```rust
use memscope::lockfree::IntegratedProfilingSession;

let mut session = IntegratedProfilingSession::new(&output_dir)?;
session.start_profiling()?;

// Run your multi-threaded workload
run_workload();

let analysis = session.stop_profiling_and_analyze()?;
```

## Resource Metrics

### CPU Metrics

```rust
pub struct CpuResourceMetrics {
    pub overall_usage_percent: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: Vec<u32>,
    pub temperature_celsius: Vec<f32>,
    pub context_switches_per_sec: u64,
    pub interrupts_per_sec: u64,
    pub load_average: (f64, f64, f64), // 1min, 5min, 15min
}
```

### GPU Metrics

```rust
pub struct GpuResourceMetrics {
    pub device_name: String,
    pub vendor: GpuVendor,
    pub compute_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub temperature_celsius: f32,
    pub power_usage_watts: f32,
    pub frequency_mhz: u32,
}
```

### Thread Metrics

```rust
pub struct ThreadResourceMetrics {
    pub thread_id: u64,
    pub thread_name: Option<String>,
    pub cpu_usage_percent: f32,
    pub memory_resident_bytes: u64,
    pub memory_virtual_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub cpu_time_user_ns: u64,
    pub cpu_time_kernel_ns: u64,
}
```

## Performance Analysis

### Correlation Metrics

The system calculates correlations between memory operations and system resources:

- **Memory-CPU correlation**: Relationship between allocation rate and CPU usage
- **Memory-GPU correlation**: GPU activity during memory-intensive operations
- **Memory-IO correlation**: IO activity correlation with memory operations
- **Allocation-CPU correlation**: CPU usage patterns during allocations
- **Deallocation-pressure correlation**: Memory pressure effects on deallocation patterns

### Bottleneck Identification

Automatically identifies primary performance bottlenecks:

```rust
pub enum BottleneckType {
    CpuBound,           // High CPU correlation
    MemoryBound,        // Memory pressure issues
    IoBound,            // IO throughput limitations
    GpuBound,           // GPU resource constraints
    ContentionBound,    // Lock/synchronization issues
    Balanced,           // Well-balanced resource usage
}
```

### Performance Insights

Provides actionable recommendations:

```rust
pub struct PerformanceInsights {
    pub primary_bottleneck: BottleneckType,
    pub cpu_efficiency_score: f32,
    pub memory_efficiency_score: f32,
    pub io_efficiency_score: f32,
    pub recommendations: Vec<String>,
    pub thread_performance_ranking: Vec<ThreadPerformanceMetric>,
}
```

## Platform-Specific Implementation

### macOS Implementation

Uses native macOS APIs for optimal performance:

- **CPU metrics**: `host_statistics()` and `task_info()` syscalls
- **Memory info**: `mach_task_basic_info` structures
- **Load average**: `getloadavg()` system call
- **Thread info**: `pthread_self()` and task enumeration

**GPU Support:**
- Apple Silicon: Metal Performance Shaders framework
- Intel/AMD: IOKit power management integration

### Linux Implementation

Leverages /proc filesystem and Linux-specific interfaces:

- **CPU metrics**: `/proc/stat`, `/proc/cpuinfo`, `/sys/devices/system/cpu/`
- **Thread metrics**: `/proc/self/task/*/stat`, `/proc/self/task/*/status`
- **GPU metrics**: nvidia-smi, ROCm tools, i915 driver interfaces
- **Temperature**: `/sys/class/thermal/thermal_zone*/temp`

**GPU Support:**
- NVIDIA: nvidia-smi command-line interface
- AMD: ROCm system management interface
- Intel: i915 driver sysfs interface

## Usage Examples

### Basic Resource Monitoring

```rust
use memscope::lockfree::PlatformResourceCollector;

fn monitor_resources() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = PlatformResourceCollector::new()?;
    
    for _i in 0..10 {
        let metrics = collector.collect_metrics()?;
        
        println!("CPU: {:.1}%", metrics.cpu_metrics.overall_usage_percent);
        if let Some(gpu) = &metrics.gpu_metrics {
            println!("GPU: {:.1}%", gpu.compute_usage_percent);
        }
        
        std::thread::sleep(collector.get_optimal_collection_interval());
    }
    
    Ok(())
}
```

### Comprehensive Analysis

```rust
use memscope::lockfree::comprehensive_profile_execution;

fn analyze_workload() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = std::path::Path::new("./analysis_output");
    
    let (result, analysis) = comprehensive_profile_execution(output_dir, || {
        // Your compute-intensive workload
        perform_matrix_multiplication();
        perform_memory_operations();
        perform_io_operations();
        
        "workload_completed"
    })?;
    
    println!("Workload result: {}", result);
    println!("Primary bottleneck: {:?}", analysis.performance_insights.primary_bottleneck);
    
    for recommendation in &analysis.performance_insights.recommendations {
        println!("Recommendation: {}", recommendation);
    }
    
    Ok(())
}
```

### Multi-threaded Monitoring

```rust
use memscope::lockfree::IntegratedProfilingSession;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

fn monitor_multithreaded_app() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = std::path::Path::new("./multithread_analysis");
    let mut session = IntegratedProfilingSession::new(output_dir)?;
    
    session.start_profiling()?;
    
    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    
    // Launch worker threads
    for i in 0..4 {
        let stop = stop_signal.clone();
        let handle = thread::Builder::new()
            .name(format!("worker_{}", i))
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    // Simulate work
                    perform_cpu_intensive_task();
                    perform_memory_allocations();
                    
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            })?;
        handles.push(handle);
    }
    
    // Let it run for analysis period
    thread::sleep(std::time::Duration::from_secs(30));
    
    // Stop threads
    stop_signal.store(true, Ordering::Relaxed);
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Analyze results
    let analysis = session.stop_profiling_and_analyze()?;
    
    println!("Thread performance ranking:");
    for (i, thread_perf) in analysis.performance_insights.thread_performance_ranking.iter().enumerate() {
        println!("{}. Thread {} - Efficiency: {:.1}%", 
                i + 1, thread_perf.thread_id, thread_perf.efficiency_score);
    }
    
    Ok(())
}
```

## Integration with Existing Features

The platform resource monitoring integrates seamlessly with existing memscope features:

- **Binary export**: Resource metrics included in binary format
- **JSON export**: Comprehensive analysis with resource correlations
- **HTML visualization**: Enhanced dashboards showing resource usage
- **Lockfree tracking**: Zero-overhead resource collection in multi-threaded scenarios

## Performance Considerations

- **Collection frequency**: Default 10Hz (100ms interval) for real-time monitoring
- **Adaptive sampling**: Automatically adjusts based on system load
- **Memory overhead**: < 1MB per 1000 samples
- **CPU overhead**: < 1% on modern systems
- **Thread safety**: Complete lockfree design for concurrent access

## Error Handling

The system gracefully handles platform limitations:

- Missing GPU drivers return `None` for GPU metrics
- Insufficient permissions default to basic CPU metrics
- Unsupported platforms return default/empty metrics
- All operations use `Result<T, Box<dyn std::error::Error>>` for proper error propagation

## Testing

Run the demonstration example:

```bash
cargo run --example platform_resource_monitoring_demo
```

Expected output includes:
- Platform capability detection
- Resource collection samples
- Comprehensive analysis results
- Multi-threaded performance rankings

## Future Enhancements

Planned improvements:

1. **Windows support**: WMI and Performance Counters integration
2. **Network monitoring**: Detailed network interface metrics
3. **Power monitoring**: Battery and thermal throttling detection
4. **Container awareness**: Docker/Kubernetes resource limits
5. **Real-time visualization**: Live dashboard for resource monitoring