# Async Memory Module Documentation

## Overview

The `async_memory` module provides comprehensive memory tracking and resource monitoring capabilities for Rust async applications. It offers real-time performance analysis, resource usage tracking, bottleneck identification, and advanced visualization features.

## Architecture

```
src/async_memory/
├── mod.rs                 # Module exports and main API
├── api.rs                 # High-level user API
├── buffer.rs              # Memory buffer management
├── error.rs               # Error types and handling
├── profile.rs             # Task performance profiling
├── resource_monitor.rs    # Resource monitoring engine
├── system_monitor.rs      # System-level monitoring
├── task_id.rs            # Task identification
├── tracker.rs            # Memory tracking implementation
└── visualization.rs      # Report generation and visualization
```

## Core Components

### 1. AsyncResourceMonitor

The main resource monitoring engine that tracks CPU, memory, IO, network, and GPU usage for async tasks.

```rust
use memscope_rs::async_memory::AsyncResourceMonitor;

let mut monitor = AsyncResourceMonitor::new();

// Start monitoring a task
monitor.start_monitoring(task_id, "My Task", TaskType::CpuIntensive);

// Update metrics during execution
monitor.update_metrics(task_id);

// Finish monitoring
monitor.finish_monitoring(task_id);

// Get all profiles
let profiles = monitor.get_all_profiles();
```

### 2. Task Memory Tracker

Tracks memory allocations and deallocations for individual async tasks.

```rust
use memscope_rs::async_memory::{create_tracked, TaskId};

async fn my_task() {
    let task_id = TaskId::new();
    
    // Create tracked future
    let tracked_future = create_tracked(task_id, async {
        // Your async code here
        let data = vec![0u8; 1024]; // Tracked allocation
        process_data(data).await
    });
    
    tracked_future.await
}
```

### 3. Resource Monitoring

Comprehensive system resource tracking with detailed metrics.

#### CPU Metrics
```rust
pub struct CpuMetrics {
    pub usage_percent: f64,           // CPU utilization percentage
    pub time_user_ms: f64,           // User mode execution time
    pub time_kernel_ms: f64,         // Kernel mode execution time
    pub context_switches: u64,       // Number of context switches
    pub cpu_cycles: u64,             // CPU cycles consumed
    pub instructions: u64,           // Instructions executed
    pub cache_misses: u64,           // Cache miss count
    pub branch_misses: u64,          // Branch prediction misses
    pub core_affinity: Vec<u32>,     // CPU cores used
}
```

#### Memory Metrics
```rust
pub struct MemoryMetrics {
    pub current_bytes: u64,          // Current memory usage
    pub peak_bytes: u64,             // Peak memory usage
    pub allocations_count: u64,      // Number of allocations
    pub deallocations_count: u64,    // Number of deallocations
    pub heap_fragmentation: f64,     // Heap fragmentation ratio
    pub memory_bandwidth: f64,       // Memory bandwidth usage
}
```

#### IO Metrics
```rust
pub struct IoMetrics {
    pub bytes_read: u64,             // Total bytes read
    pub bytes_written: u64,          // Total bytes written
    pub operations_read: u64,        // Number of read operations
    pub operations_written: u64,     // Number of write operations
    pub average_latency_ms: f64,     // Average IO latency
    pub bandwidth_mbps: f64,         // IO bandwidth in MB/s
    pub io_wait_percent: f64,        // IO wait percentage
    pub queue_depth: u32,            // IO queue depth
}
```

#### Network Metrics
```rust
pub struct NetworkMetrics {
    pub bytes_sent: u64,             // Total bytes sent
    pub bytes_received: u64,         // Total bytes received
    pub packets_sent: u64,           // Number of packets sent
    pub packets_received: u64,       // Number of packets received
    pub connections_active: u32,     // Active connections
    pub latency_avg_ms: f64,         // Average network latency
    pub throughput_mbps: f64,        // Network throughput
    pub error_count: u64,            // Network errors
}
```

#### GPU Metrics (Optional)
```rust
pub struct GpuMetrics {
    pub gpu_utilization: f64,        // GPU utilization percentage
    pub memory_used: u64,            // GPU memory used
    pub memory_total: u64,           // Total GPU memory
    pub compute_units: u32,          // Active compute units
    pub memory_bandwidth: f64,       // GPU memory bandwidth
    pub temperature: f32,            // GPU temperature
    pub power_draw: f32,             // Power consumption
    pub clock_speed: u32,            // GPU clock speed
}
```

### 4. Performance Analysis

Advanced performance analysis with bottleneck detection and efficiency scoring.

#### Task Resource Profile
```rust
pub struct TaskResourceProfile {
    pub task_id: TaskId,
    pub task_name: String,
    pub task_type: TaskType,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<f64>,
    
    // Resource metrics
    pub cpu_metrics: CpuMetrics,
    pub memory_metrics: MemoryMetrics,
    pub io_metrics: IoMetrics,
    pub network_metrics: NetworkMetrics,
    pub gpu_metrics: Option<GpuMetrics>,
    
    // Performance analysis
    pub efficiency_score: f64,
    pub resource_balance: f64,
    pub bottleneck_type: BottleneckType,
    
    // Enhanced features
    pub source_location: SourceLocation,
    pub hot_metrics: HotMetrics,
    pub efficiency_explanation: EfficiencyExplanation,
}
```

#### Bottleneck Detection
```rust
pub enum BottleneckType {
    Cpu,        // CPU-bound workload
    Memory,     // Memory-bound workload
    Io,         // IO-bound workload
    Network,    // Network-bound workload
    Gpu,        // GPU-bound workload
    Balanced,   // Well-balanced resource usage
    Unknown,    // Insufficient data for analysis
}
```

#### Task Types
```rust
pub enum TaskType {
    CpuIntensive,      // CPU-heavy computations
    IoIntensive,       // File and storage operations
    NetworkIntensive,  // Network communications
    MemoryIntensive,   // Memory-heavy operations
    GpuCompute,        // GPU computations
    Mixed,             // Mixed workload
    Streaming,         // Real-time streaming
    Background,        // Background processing
}
```

### 5. Source Location Tracking

Track the source code location where tasks are defined for debugging and optimization.

```rust
pub struct SourceLocation {
    pub file_path: String,           // Source file path
    pub line_number: u32,            // Line number
    pub function_name: String,       // Function name
    pub module_path: String,         // Module path
    pub crate_name: String,          // Crate name
}
```

### 6. Hot Metrics Analysis

Detailed hotspot analysis for performance optimization.

```rust
pub struct HotMetrics {
    pub cpu_hotspots: Vec<CpuHotspot>,
    pub memory_hotspots: Vec<MemoryHotspot>,
    pub io_hotspots: Vec<IoHotspot>,
    pub network_hotspots: Vec<NetworkHotspot>,
    pub critical_path_analysis: CriticalPathAnalysis,
}
```

### 7. Visualization and Reporting

Generate comprehensive HTML reports with charts and analysis.

```rust
use memscope_rs::async_memory::{VisualizationGenerator, VisualizationConfig, Theme};

let config = VisualizationConfig {
    title: "Performance Analysis Report".to_string(),
    theme: Theme::Dark,
    include_charts: true,
    include_baselines: true,
    include_rankings: true,
    include_efficiency_breakdown: true,
};

let visualizer = VisualizationGenerator::with_config(config);
let html_report = visualizer.generate_html_report(&profiles)?;
```

## Data Collection Logic

### 1. Initialization

```rust
use memscope_rs::async_memory;

// Initialize the async memory tracking system
async_memory::initialize()?;
```

### 2. Task Registration

```rust
// Method 1: Manual registration
let monitor = AsyncResourceMonitor::new();
monitor.start_monitoring(task_id, "Task Name", TaskType::CpuIntensive);

// Method 2: With source location
let source_location = SourceLocation {
    file_path: "src/main.rs".to_string(),
    line_number: 42,
    function_name: "process_data".to_string(),
    module_path: "myapp::processor".to_string(),
    crate_name: "myapp".to_string(),
};
monitor.start_monitoring_with_location(task_id, "Task Name", TaskType::CpuIntensive, Some(source_location));
```

### 3. Metric Collection

```rust
// Periodic metric updates during task execution
loop {
    monitor.update_metrics(task_id);
    
    // Your task work here
    do_work().await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### 4. Task Completion

```rust
// Finish monitoring when task completes
monitor.finish_monitoring(task_id);
```

### 5. Data Retrieval

```rust
// Get all task profiles
let profiles = monitor.get_all_profiles();

// Get specific task profile
if let Some(profile) = monitor.get_profile(task_id) {
    println!("Task efficiency: {:.1}%", profile.efficiency_score * 100.0);
}
```

## Display Content

### 1. Performance Overview

- **Summary Statistics**: Total tasks, average resource usage, overall efficiency
- **Performance Trends**: Visual charts showing resource distribution
- **System Health**: Overall system resource utilization

### 2. Task Analysis

- **Individual Task Cards**: Detailed metrics for each monitored task
- **Baseline Comparisons**: How each task compares to average performance
- **Category Rankings**: Ranking within task type categories
- **Efficiency Breakdown**: Component-wise efficiency analysis

### 3. Resource Monitoring

- **CPU Usage**: Utilization percentage, core distribution, context switches
- **Memory Usage**: Current/peak usage, allocation patterns, fragmentation
- **IO Performance**: Bandwidth, latency, operation counts
- **Network Activity**: Throughput, connection counts, error rates
- **GPU Utilization**: GPU usage, memory consumption, compute units

### 4. Optimization Insights

- **Bottleneck Identification**: Primary performance bottlenecks
- **Hot Spots**: Performance-critical code sections
- **Optimization Recommendations**: Actionable improvement suggestions
- **Critical Path Analysis**: Execution flow analysis

## Examples

### Basic Usage

```rust
use memscope_rs::async_memory::{self, AsyncResourceMonitor, TaskType, TaskId};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    async_memory::initialize()?;
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Start tasks with monitoring
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let task_id = i as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        
        // Register task
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring(task_id, format!("Task-{}", i), TaskType::CpuIntensive);
        }
        
        // Spawn monitored task
        let handle = tokio::spawn(async move {
            execute_monitored_task(task_id, monitor_clone).await
        });
        
        handles.push((task_id, handle));
    }
    
    // Wait for completion
    for (task_id, handle) in handles {
        handle.await?;
        
        let mut mon = monitor.lock().unwrap();
        mon.finish_monitoring(task_id);
    }
    
    // Generate report
    let profiles = {
        let mon = monitor.lock().unwrap();
        mon.get_all_profiles().clone()
    };
    
    let visualizer = VisualizationGenerator::new();
    let html_report = visualizer.generate_html_report(&profiles)?;
    std::fs::write("performance_report.html", html_report)?;
    
    println!("Report generated: performance_report.html");
    Ok(())
}

async fn execute_monitored_task(
    task_id: TaskId,
    monitor: Arc<Mutex<AsyncResourceMonitor>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Periodic metric updates
    let update_handle = {
        let monitor_clone = Arc::clone(&monitor);
        tokio::spawn(async move {
            for _ in 0..10 {
                {
                    let mut mon = monitor_clone.lock().unwrap();
                    mon.update_metrics(task_id);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
    };
    
    // Simulate work
    for i in 0..1000000u32 {
        let _ = i.wrapping_mul(i) % 12345;
        if i % 100000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    update_handle.await?;
    Ok(())
}
```

### Advanced Usage with Custom Configuration

```rust
use memscope_rs::async_memory::{
    VisualizationGenerator, VisualizationConfig, Theme,
    resource_monitor::SourceLocation
};

// Create custom visualization config
let viz_config = VisualizationConfig {
    title: "Production Performance Analysis".to_string(),
    theme: Theme::Dark,
    include_charts: true,
    include_baselines: true,
    include_rankings: true,
    include_efficiency_breakdown: true,
};

// Monitor tasks with source tracking
let source_location = SourceLocation {
    file_path: file!().to_string(),
    line_number: line!(),
    function_name: "advanced_task".to_string(),
    module_path: module_path!().to_string(),
    crate_name: env!("CARGO_PKG_NAME").to_string(),
};

monitor.start_monitoring_with_location(
    task_id, 
    "Advanced Task", 
    TaskType::Mixed, 
    Some(source_location)
);
```

### Memory Tracking Example

```rust
use memscope_rs::async_memory::{create_tracked, TaskId};

async fn memory_intensive_task() -> Result<(), Box<dyn std::error::Error>> {
    let task_id = TaskId::new();
    
    let result = create_tracked(task_id, async {
        // Memory allocations are tracked
        let large_buffer = vec![0u8; 10 * 1024 * 1024]; // 10MB
        
        // Process data
        process_buffer(large_buffer).await?;
        
        // Allocations and deallocations are monitored
        let cache = build_cache().await?;
        
        Ok::<_, Box<dyn std::error::Error>>("Task completed")
    }).await?;
    
    println!("Result: {}", result);
    Ok(())
}
```

### Integration with Existing Applications

```rust
// In your existing async application
async fn integrate_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize monitoring
    async_memory::initialize()?;
    let monitor = Arc::new(Mutex::new(AsyncResourceMonitor::new()));
    
    // Your existing task spawning logic
    let mut task_handles = Vec::new();
    
    for (i, config) in task_configs.iter().enumerate() {
        let task_id = i as TaskId;
        let monitor_clone = Arc::clone(&monitor);
        
        // Add monitoring to existing tasks
        {
            let mut mon = monitor_clone.lock().unwrap();
            mon.start_monitoring(task_id, config.name.clone(), config.task_type);
        }
        
        // Spawn with monitoring wrapper
        let handle = tokio::spawn(async move {
            let result = execute_existing_task(config).await;
            
            // Finish monitoring
            {
                let mut mon = monitor_clone.lock().unwrap();
                mon.finish_monitoring(task_id);
            }
            
            result
        });
        
        task_handles.push(handle);
    }
    
    // Wait for completion and generate report
    for handle in task_handles {
        handle.await?;
    }
    
    generate_performance_report(&monitor).await?;
    Ok(())
}
```

## Performance Characteristics

### Memory Overhead
- **Per task**: ~1-5KB metadata
- **Global state**: ~10-50KB for monitoring infrastructure
- **Scaling**: Linear O(n) with number of tasks

### CPU Overhead
- **Metric collection**: <1% CPU overhead
- **Report generation**: O(n) where n is number of tasks
- **Real-time updates**: Configurable frequency (default 100ms)

### Storage Requirements
- **In-memory profiles**: ~2-10KB per task
- **HTML reports**: ~2-5KB per task in final output
- **Minimal disk usage**: Only for generated reports

## Best Practices

### 1. Task Naming and Categorization
```rust
// Use descriptive task names
monitor.start_monitoring(task_id, "Image Processing Pipeline", TaskType::CpuIntensive);

// Group related tasks by type
monitor.start_monitoring(task_id, "Database Query", TaskType::IoIntensive);
monitor.start_monitoring(task_id, "API Request", TaskType::NetworkIntensive);
```

### 2. Monitoring Lifecycle
```rust
// Always pair start/finish calls
monitor.start_monitoring(task_id, name, task_type);
// ... task execution ...
monitor.finish_monitoring(task_id); // Important: don't forget this!
```

### 3. Source Location Tracking
```rust
// Use macros for automatic source tracking
macro_rules! track_task {
    ($monitor:expr, $task_id:expr, $name:expr, $task_type:expr) => {
        $monitor.start_monitoring_with_location(
            $task_id,
            $name,
            $task_type,
            Some(SourceLocation {
                file_path: file!().to_string(),
                line_number: line!(),
                function_name: "unknown".to_string(), // Can be enhanced with more macros
                module_path: module_path!().to_string(),
                crate_name: env!("CARGO_PKG_NAME").to_string(),
            })
        );
    };
}
```

### 4. Error Handling
```rust
use memscope_rs::async_memory::AsyncError;

match monitor.start_monitoring(task_id, name, task_type) {
    Ok(_) => { /* monitoring started */ }
    Err(AsyncError::TaskAlreadyExists) => {
        // Handle duplicate task ID
    }
    Err(e) => {
        eprintln!("Monitoring error: {}", e);
    }
}
```

### 5. Performance Optimization
```rust
// Batch metric updates for efficiency
let update_interval = Duration::from_millis(500); // Adjust based on needs

// Use periodic updates rather than continuous
tokio::spawn(async move {
    let mut interval = tokio::time::interval(update_interval);
    loop {
        interval.tick().await;
        monitor.update_all_metrics(); // Batch update all tasks
    }
});
```

## Troubleshooting

### Common Issues

#### 1. Missing Task Data
```rust
// Problem: No data in reports
// Solution: Ensure monitoring lifecycle is complete
monitor.start_monitoring(task_id, name, task_type);
// ... execute task ...
monitor.finish_monitoring(task_id); // <- Don't forget this!
```

#### 2. Memory Leaks
```rust
// Problem: Memory usage grows over time
// Solution: Properly clean up finished tasks
monitor.cleanup_finished_tasks(); // Periodic cleanup
```

#### 3. Performance Impact
```rust
// Problem: Monitoring overhead too high
// Solution: Reduce update frequency
let config = MonitoringConfig {
    update_interval_ms: 1000, // Increase from default 100ms
    batch_updates: true,       // Enable batching
};
```

#### 4. Large Report Files
```rust
// Problem: HTML reports too large
// Solution: Filter or paginate data
let filtered_profiles: HashMap<_, _> = profiles
    .into_iter()
    .filter(|(_, profile)| profile.efficiency_score < 0.8) // Only inefficient tasks
    .collect();

let html_report = visualizer.generate_html_report(&filtered_profiles)?;
```

### Debug Mode

```rust
// Enable debug logging
env_logger::init();

// Use debug methods
monitor.debug_print_all_tasks();
monitor.validate_consistency();
```

## Migration Guide

### From Basic Monitoring to Async Memory

```rust
// Before: Basic timing
let start = Instant::now();
execute_task().await;
let duration = start.elapsed();
println!("Task took: {:?}", duration);

// After: Comprehensive monitoring
let task_id = TaskId::new();
monitor.start_monitoring(task_id, "My Task", TaskType::CpuIntensive);

let update_handle = tokio::spawn({
    let monitor = monitor.clone();
    async move {
        loop {
            monitor.update_metrics(task_id);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
});

execute_task().await;

update_handle.abort();
monitor.finish_monitoring(task_id);

// Get detailed analysis
let profile = monitor.get_profile(task_id).unwrap();
println!("Efficiency: {:.1}%", profile.efficiency_score * 100.0);
println!("Bottleneck: {:?}", profile.bottleneck_type);
```

## API Summary

### Main Exports
```rust
pub use async_memory::{
    // Core monitoring
    AsyncResourceMonitor,
    TaskResourceProfile,
    TaskType,
    BottleneckType,
    
    // Metrics
    CpuMetrics,
    MemoryMetrics,
    IoMetrics,
    NetworkMetrics,
    GpuMetrics,
    
    // Visualization
    VisualizationGenerator,
    VisualizationConfig,
    Theme,
    
    // Tracking
    create_tracked,
    TaskId,
    
    // Analysis
    PerformanceBaselines,
    CategoryRanking,
    PerformanceComparison,
    
    // Errors
    AsyncError,
    VisualizationError,
};
```

This comprehensive module provides everything needed for enterprise-grade async task monitoring, performance analysis, and optimization in Rust applications.