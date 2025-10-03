# Async Module: Task-Centric Memory Analysis

The async module provides **task-aware memory tracking** for async/await applications. Unlike thread-based tracking, this system tracks memory at the granularity of individual async tasks (Futures).

## 🎯 When to Use

**✅ Perfect for:**
- async/await applications
- Tokio, async-std, smol runtimes
- Task-level memory analysis
- Async service monitoring
- Microservices and async web servers

**❌ Use other modules for:**
- Synchronous applications
- Thread-pool based concurrency
- When task-level granularity isn't needed

## ⚡ Core API

### Quick Start

```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, get_memory_snapshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize async memory tracking
    initialize().await?;
    
    // Create tracked async task
    let task = spawn_tracked(async {
        let data = vec![0u8; 1024 * 1024]; // 1MB allocation
        
        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Process data
        let processed = data.iter().map(|&x| x as u64).sum::<u64>();
        processed
    });
    
    // Await result
    let result = task.await?;
    println!("Processed {} bytes, sum: {}", 1024 * 1024, result);
    
    // Get memory snapshot
    let snapshot = get_memory_snapshot();
    println!("Active tasks: {}", snapshot.active_task_count());
    println!("Total memory tracked: {} bytes", snapshot.total_memory_bytes());
    
    Ok(())
}
```

### Advanced Task Tracking

```rust
use memscope_rs::async_memory::{
    initialize, create_tracked, TaskMemoryProfile, 
    AsyncResourceMonitor, TaskType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    // Create resource monitor for detailed analysis
    let mut monitor = AsyncResourceMonitor::new();
    monitor.start_monitoring().await?;
    
    // CPU-intensive task
    let cpu_task = create_tracked(
        async {
            let mut data = vec![0u64; 1_000_000];
            for i in 0..data.len() {
                data[i] = (i as u64).pow(2) % 1000;
            }
            data.iter().sum::<u64>()
        },
        TaskType::CpuIntensive
    );
    
    // I/O-intensive task
    let io_task = create_tracked(
        async {
            let mut results = Vec::new();
            for i in 0..100 {
                // Simulate file I/O
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                results.push(format!("Result-{}", i));
            }
            results.len()
        },
        TaskType::IoIntensive
    );
    
    // Network-intensive task
    let network_task = create_tracked(
        async {
            let mut responses = Vec::new();
            for i in 0..50 {
                // Simulate network request
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                let response = format!("Response-{}: {}", i, "A".repeat(1024));
                responses.push(response);
            }
            responses.len()
        },
        TaskType::NetworkIntensive
    );
    
    // Wait for all tasks
    let (cpu_result, io_result, network_result) = tokio::try_join!(
        cpu_task,
        io_task,
        network_task
    )?;
    
    println!("CPU task result: {}", cpu_result);
    println!("I/O task result: {}", io_result);
    println!("Network task result: {}", network_result);
    
    // Stop monitoring and get detailed analysis
    let analysis = monitor.stop_monitoring().await?;
    
    println!("📊 Task Analysis:");
    println!("   - CPU-intensive tasks: {}", analysis.cpu_task_count);
    println!("   - I/O-intensive tasks: {}", analysis.io_task_count);
    println!("   - Network-intensive tasks: {}", analysis.network_task_count);
    println!("   - Peak memory per task: {:.2} MB", analysis.peak_memory_per_task_mb);
    
    Ok(())
}
```

## 🎮 Real-world Examples

### Async Web Server

```rust
use memscope_rs::async_memory::{initialize, spawn_tracked, TaskType};
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn async_web_server_simulation() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    let request_counter = Arc::new(AtomicUsize::new(0));
    let (tx, mut rx) = mpsc::channel(1000);
    
    // Spawn request handler tasks
    let mut handlers = Vec::new();
    for worker_id in 0..10 {
        let tx = tx.clone();
        let counter = Arc::clone(&request_counter);
        
        let handler = spawn_tracked(
            async move {
                for request_id in 0..100 {
                    // Simulate incoming request
                    let request = format!("Request-{}-{}", worker_id, request_id);
                    
                    // Process request
                    let response = process_request(request).await;
                    
                    // Send response
                    if tx.send(response).await.is_err() {
                        break;
                    }
                    
                    counter.fetch_add(1, Ordering::Relaxed);
                    
                    // Simulate request interval
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            },
            TaskType::NetworkIntensive
        );
        
        handlers.push(handler);
    }
    
    // Response collector
    let collector = spawn_tracked(
        async move {
            let mut responses = Vec::new();
            while let Some(response) = rx.recv().await {
                responses.push(response);
                if responses.len() >= 1000 {
                    break;
                }
            }
            responses.len()
        },
        TaskType::IoIntensive
    );
    
    // Wait for all handlers
    for handler in handlers {
        handler.await?;
    }
    
    let response_count = collector.await?;
    let total_requests = request_counter.load(Ordering::Relaxed);
    
    println!("🌐 Async web server simulation complete!");
    println!("   - Requests processed: {}", total_requests);
    println!("   - Responses collected: {}", response_count);
    
    // Get final memory snapshot
    let snapshot = get_memory_snapshot();
    println!("   - Peak concurrent tasks: {}", snapshot.peak_concurrent_tasks());
    println!("   - Total memory allocated: {:.2} MB", snapshot.total_memory_bytes() as f64 / (1024.0 * 1024.0));
    
    Ok(())
}

async fn process_request(request: String) -> String {
    // Simulate database query
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    let query_result = vec![request.clone(); 10];
    
    // Simulate business logic
    let processed_data = query_result
        .iter()
        .map(|r| format!("Processed: {}", r))
        .collect::<Vec<_>>();
    
    // Simulate response serialization
    format!("{{\"status\":\"ok\",\"data\":{:?}}}", processed_data)
}
```

### Microservice Communication

```rust
use memscope_rs::async_memory::{
    initialize, create_tracked, TaskType, 
    TaskMemoryProfile, get_memory_snapshot
};
use tokio::sync::oneshot;
use std::collections::HashMap;

struct ServiceRegistry {
    services: HashMap<String, oneshot::Sender<String>>,
}

#[tokio::main]
async fn microservice_simulation() -> Result<(), Box<dyn std::error::Error>> {
    initialize().await?;
    
    // Service A: User Service
    let user_service = create_tracked(
        async {
            let mut users = Vec::new();
            for i in 0..1000 {
                // Simulate user data processing
                let user = format!("{{\"id\":{},\"name\":\"User{}\",\"email\":\"user{}@example.com\"}}", i, i, i);
                users.push(user);
                
                // Simulate database write
                if i % 100 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
            users.len()
        },
        TaskType::IoIntensive
    );
    
    // Service B: Payment Service
    let payment_service = create_tracked(
        async {
            let mut transactions = Vec::new();
            for i in 0..500 {
                // Simulate payment processing
                let transaction = format!("{{\"id\":{},\"amount\":{:.2},\"status\":\"completed\"}}", i, i as f64 * 10.50);
                transactions.push(transaction);
                
                // Simulate external API call
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
            transactions.len()
        },
        TaskType::NetworkIntensive
    );
    
    // Service C: Analytics Service
    let analytics_service = create_tracked(
        async {
            let mut metrics = Vec::new();
            for batch in 0..50 {
                // Simulate data aggregation
                let mut batch_data = Vec::new();
                for i in 0..1000 {
                    let value = (batch * 1000 + i) as f64;
                    batch_data.push(value.sin().abs());
                }
                
                // Calculate statistics
                let sum: f64 = batch_data.iter().sum();
                let avg = sum / batch_data.len() as f64;
                let metric = format!("{{\"batch\":{},\"avg\":{:.4},\"count\":{}}}", batch, avg, batch_data.len());
                metrics.push(metric);
                
                // Simulate computation delay
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
            metrics.len()
        },
        TaskType::CpuIntensive
    );
    
    // API Gateway - orchestrates all services
    let gateway = create_tracked(
        async {
            let mut responses = Vec::new();
            
            // Simulate 100 composite requests
            for request_id in 0..100 {
                // Simulate parallel service calls
                let user_data = format!("User data for request {}", request_id);
                let payment_data = format!("Payment data for request {}", request_id);
                let analytics_data = format!("Analytics for request {}", request_id);
                
                // Compose response
                let composite_response = format!(
                    "{{\"request_id\":{},\"user\":\"{}\",\"payment\":\"{}\",\"analytics\":\"{}\"}}",
                    request_id, user_data, payment_data, analytics_data
                );
                responses.push(composite_response);
                
                // Simulate response time
                tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
            }
            
            responses.len()
        },
        TaskType::NetworkIntensive
    );
    
    // Wait for all services
    let (user_count, payment_count, analytics_count, gateway_count) = tokio::try_join!(
        user_service,
        payment_service,
        analytics_service,
        gateway
    )?;
    
    println!("🏗️  Microservice simulation complete!");
    println!("   - User service processed: {} users", user_count);
    println!("   - Payment service processed: {} transactions", payment_count);
    println!("   - Analytics service processed: {} metrics", analytics_count);
    println!("   - Gateway handled: {} composite requests", gateway_count);
    
    // Final memory analysis
    let snapshot = get_memory_snapshot();
    println!("📊 Memory Analysis:");
    println!("   - Peak concurrent tasks: {}", snapshot.peak_concurrent_tasks());
    println!("   - Total allocations tracked: {}", snapshot.total_allocations());
    println!("   - Memory efficiency: {:.1}%", snapshot.memory_efficiency_percent());
    
    Ok(())
}
```

## 📊 Performance Characteristics

### Tracking Overhead

| Feature | Overhead | Description |
|---------|----------|-------------|
| **Task identification** | < 5ns | Zero-overhead task ID extraction |
| **Memory tracking** | < 0.1% CPU | Lock-free event buffering |
| **Data collection** | < 1MB/thread | Efficient ring buffers |

### Task Analysis Metrics

```rust
use memscope_rs::async_memory::{TaskMemoryProfile, TaskPerformanceMetrics};

async fn analyze_task_performance() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = get_memory_snapshot();
    
    // Get performance metrics for each task type
    let cpu_metrics = snapshot.get_task_metrics(TaskType::CpuIntensive);
    let io_metrics = snapshot.get_task_metrics(TaskType::IoIntensive);
    let network_metrics = snapshot.get_task_metrics(TaskType::NetworkIntensive);
    
    println!("🎯 Task Performance Analysis:");
    println!("CPU Tasks:");
    println!("   - Average memory: {:.2} MB", cpu_metrics.avg_memory_mb);
    println!("   - Peak memory: {:.2} MB", cpu_metrics.peak_memory_mb);
    println!("   - Completion rate: {:.1}%", cpu_metrics.completion_rate_percent);
    
    println!("I/O Tasks:");
    println!("   - Average memory: {:.2} MB", io_metrics.avg_memory_mb);
    println!("   - Wait time: {:.2}ms", io_metrics.avg_wait_time_ms);
    
    println!("Network Tasks:");
    println!("   - Average memory: {:.2} MB", network_metrics.avg_memory_mb);
    println!("   - Throughput: {:.1} requests/sec", network_metrics.throughput_per_sec);
    
    Ok(())
}
```

## 🔗 Next Steps

- **[Hybrid Module](hybrid.md)** - Cross-module comprehensive analysis
- **[API Reference](api-reference/analysis-api.md)** - Complete async API documentation
- **[Examples](examples/async-usage.md)** - More async examples