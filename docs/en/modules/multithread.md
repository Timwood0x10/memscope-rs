# Multi-threaded Module: Lock-free High-Concurrency Tracking

The multi-threaded (lockfree) module is designed for **high-concurrency applications** with 20+ threads. It uses sampling-based tracking with zero shared state for maximum performance.

## 🎯 When to Use

**✅ Perfect for:**
- High-concurrency applications (20+ threads)
- Production monitoring systems
- Performance-critical applications
- When approximate data is acceptable
- Web servers, databases, high-throughput systems

**❌ Use single-threaded module for:**
- Development and debugging
- Applications with < 10 threads
- When exact precision is required

## 🔀 Core API

### Quick Start - Simple Tracing

```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start tracking all threads
    trace_all("./MemoryAnalysis")?;
    
    // Your multi-threaded application code
    let handles: Vec<_> = (0..30).map(|i| {
        std::thread::spawn(move || {
            // Thread-local tracking happens automatically
            let data = vec![0u8; 1024 * 1024]; // 1MB allocation
            
            // Simulate work
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            println!("Thread {} completed", i);
        })
    }).collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Stop tracking and export
    stop_tracing()?;
    export_comprehensive_analysis("./MemoryAnalysis", "multi_thread_analysis")?;
    
    println!("🎯 Multi-threaded analysis complete!");
    Ok(())
}
```

### Advanced Configuration

```rust
use memscope_rs::lockfree::{
    SamplingConfig, PlatformResourceCollector, 
    comprehensive_profile_execution
};

fn advanced_multi_threaded_tracking() -> Result<(), Box<dyn std::error::Error>> {
    // Configure sampling for optimal performance
    let sampling_config = SamplingConfig {
        sample_rate: 0.01,        // 1% sampling rate
        min_allocation_size: 1024, // Only track allocations > 1KB
        buffer_size: 1024 * 1024, // 1MB buffer per thread
    };
    
    // Start comprehensive profiling
    let mut session = comprehensive_profile_execution(
        "./HighConcurrencyAnalysis",
        Some(sampling_config)
    )?;
    
    // Your high-concurrency workload
    let handles: Vec<_> = (0..100).map(|thread_id| {
        std::thread::spawn(move || {
            for iteration in 0..1000 {
                // Heavy memory workload
                let data = vec![thread_id; 10000];
                
                // CPU-intensive work
                let sum: usize = data.iter().sum();
                
                // I/O simulation
                if iteration % 100 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                
                // Let data go out of scope
                drop(data);
            }
        })
    }).collect();
    
    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Finalize and export comprehensive analysis
    let analysis_result = session.finalize()?;
    
    println!("📊 Comprehensive Analysis Results:");
    println!("   - Threads analyzed: {}", analysis_result.thread_count);
    println!("   - Total allocations: {}", analysis_result.total_allocations);
    println!("   - Peak memory usage: {:.2} MB", analysis_result.peak_memory_mb);
    println!("   - Performance bottlenecks: {}", analysis_result.bottlenecks.len());
    
    Ok(())
}
```

## 📊 Platform Resource Monitoring

The lockfree module includes comprehensive system resource tracking:

```rust
use memscope_rs::lockfree::{
    PlatformResourceCollector, ThreadResourceMetrics,
    CpuResourceMetrics, IoResourceMetrics
};

fn monitor_system_resources() -> Result<(), Box<dyn std::error::Error>> {
    let mut collector = PlatformResourceCollector::new()?;
    
    // Start monitoring
    collector.start_monitoring()?;
    
    // Run your workload while monitoring
    let handles: Vec<_> = (0..50).map(|i| {
        std::thread::spawn(move || {
            // CPU-intensive task
            let mut data = vec![0u64; 100000];
            for j in 0..data.len() {
                data[j] = (i as u64 * j as u64) % 1000;
            }
            
            // Memory-intensive task
            let large_data = vec![data; 10];
            
            // I/O simulation
            std::thread::sleep(std::time::Duration::from_millis(50));
            
            large_data.len()
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Stop monitoring and get results
    let metrics = collector.stop_monitoring()?;
    
    println!("🖥️  System Resource Usage:");
    println!("   - Peak CPU usage: {:.1}%", metrics.cpu_metrics.peak_usage_percent);
    println!("   - Peak memory usage: {:.2} GB", metrics.memory_metrics.peak_usage_gb);
    println!("   - Total I/O operations: {}", metrics.io_metrics.total_operations);
    println!("   - Thread efficiency: {:.2}%", metrics.thread_metrics.efficiency_percent);
    
    Ok(())
}
```

## ⚡ Performance Characteristics

### Tracking Overhead

| Configuration | CPU Overhead | Memory Overhead | Precision |
|--------------|-------------|----------------|-----------|
| **Default** | < 0.5% | < 1MB/thread | ~95% accuracy |
| **High sampling** | < 2% | < 5MB/thread | ~99% accuracy |
| **Low sampling** | < 0.1% | < 512KB/thread | ~85% accuracy |

### Scalability

| Thread Count | Export Time | Analysis Time | File Size |
|-------------|-------------|---------------|-----------|
| **30 threads** | 211ms | 150ms | 480KB |
| **100 threads** | 450ms | 300ms | 1.2MB |
| **500 threads** | 1.1s | 800ms | 4.8MB |

## 🎮 Real-world Examples

### Web Server Monitoring

```rust
use memscope_rs::lockfree::{trace_all, stop_tracing, export_comprehensive_analysis};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn simulate_web_server() -> Result<(), Box<dyn std::error::Error>> {
    // Start comprehensive tracking
    trace_all("./WebServerAnalysis")?;
    
    let request_counter = Arc::new(AtomicUsize::new(0));
    
    // Simulate web server with multiple worker threads
    let handles: Vec<_> = (0..20).map(|worker_id| {
        let counter = Arc::clone(&request_counter);
        
        std::thread::spawn(move || {
            for request_id in 0..1000 {
                // Simulate request processing
                let request_data = format!("Request-{}-{}", worker_id, request_id);
                let response_buffer = vec![0u8; 4096]; // 4KB response
                
                // Simulate database query
                let query_result = vec![request_data.as_bytes(); 10];
                
                // Simulate JSON serialization
                let json_response = format!(
                    "{{\"worker\":{},\"request\":{},\"data\":{:?}}}",
                    worker_id, request_id, query_result.len()
                );
                
                // Update metrics
                counter.fetch_add(1, Ordering::Relaxed);
                
                // Simulate response time
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
    }).collect();
    
    // Wait for all workers to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Stop tracking and analyze
    stop_tracing()?;
    export_comprehensive_analysis("./WebServerAnalysis", "web_server_performance")?;
    
    let total_requests = request_counter.load(Ordering::Relaxed);
    println!("🌐 Web server simulation complete!");
    println!("   - Total requests processed: {}", total_requests);
    println!("   - Analysis exported to: web_server_performance.html");
    
    Ok(())
}
```

### Database Connection Pool

```rust
use memscope_rs::lockfree::{comprehensive_profile_execution, SamplingConfig};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<String>>>,
}

impl ConnectionPool {
    fn new(size: usize) -> Self {
        let mut connections = VecDeque::new();
        for i in 0..size {
            connections.push_back(format!("Connection-{}", i));
        }
        
        ConnectionPool {
            connections: Arc::new(Mutex::new(connections)),
        }
    }
    
    fn get_connection(&self) -> Option<String> {
        self.connections.lock().unwrap().pop_front()
    }
    
    fn return_connection(&self, conn: String) {
        self.connections.lock().unwrap().push_back(conn);
    }
}

fn database_workload_simulation() -> Result<(), Box<dyn std::error::Error>> {
    // Configure for database-like workload
    let config = SamplingConfig {
        sample_rate: 0.05,        // 5% sampling for database operations
        min_allocation_size: 512, // Track allocations > 512 bytes
        buffer_size: 2 * 1024 * 1024, // 2MB buffer for high-frequency operations
    };
    
    let mut session = comprehensive_profile_execution(
        "./DatabaseAnalysis",
        Some(config)
    )?;
    
    let pool = Arc::new(ConnectionPool::new(10));
    
    // Simulate concurrent database operations
    let handles: Vec<_> = (0..50).map(|thread_id| {
        let pool = Arc::clone(&pool);
        
        std::thread::spawn(move || {
            for query_id in 0..200 {
                // Get connection from pool
                let connection = loop {
                    if let Some(conn) = pool.get_connection() {
                        break conn;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                };
                
                // Simulate query execution
                let query = format!("SELECT * FROM table WHERE id = {}", query_id);
                let result_set = vec![format!("Row-{}-{}", thread_id, query_id); 100];
                
                // Simulate result processing
                let processed_data: Vec<String> = result_set
                    .iter()
                    .map(|row| format!("Processed: {}", row))
                    .collect();
                
                // Simulate serialization
                let serialized = format!("{:?}", processed_data);
                
                // Return connection to pool
                pool.return_connection(connection);
                
                // Simulate network delay
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let analysis = session.finalize()?;
    
    println!("🗄️  Database simulation complete!");
    println!("   - Connection pool efficiency: {:.1}%", analysis.resource_efficiency);
    println!("   - Memory hotspots detected: {}", analysis.bottlenecks.len());
    
    Ok(())
}
```

## 🔗 Next Steps

- **[Async Module](async.md)** - Task-centric memory analysis
- **[Hybrid Module](hybrid.md)** - Cross-module comprehensive analysis
- **[Examples](examples/concurrent-analysis.md)** - More multi-threading examples