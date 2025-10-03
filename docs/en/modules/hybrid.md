# Hybrid Module: Comprehensive Cross-Module Analysis

The hybrid module **combines all tracking strategies** into a unified analysis framework. It automatically detects and adapts to different runtime patterns, providing comprehensive insights across single-threaded, multi-threaded, and async components.

## 🎯 When to Use

**✅ Perfect for:**
- Complex applications with mixed patterns
- Full-stack applications (web servers + async tasks + threads)
- Comprehensive system analysis
- Production monitoring of complex systems
- Cross-component performance analysis

**❌ Overkill for:**
- Simple single-pattern applications
- Development/debugging (use specific modules)
- Resource-constrained environments

## 🔄 Core API

### Unified Tracking

```rust
use memscope_rs::export::fixed_hybrid_template::{FixedHybridTemplate, RenderMode};
use memscope_rs::unified::{
    UnifiedBackend, EnvironmentDetector, TrackingStrategy
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize unified backend - automatically detects environment
    let mut backend = UnifiedBackend::new();
    backend.auto_configure()?;
    
    // The backend will automatically choose optimal strategies:
    // - Single-threaded tracking for main thread
    // - Multi-threaded tracking for thread pools
    // - Async tracking for tokio tasks
    
    // Your mixed application code
    let data = vec![1, 2, 3, 4, 5];
    memscope_rs::track_var!(data);  // Single-threaded tracking
    
    // Multi-threaded work
    let handles: Vec<_> = (0..10).map(|i| {
        std::thread::spawn(move || {
            let thread_data = vec![i; 1000];
            // Automatically uses lockfree tracking
            thread_data.len()
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Generate comprehensive hybrid dashboard
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let hybrid_data = backend.collect_comprehensive_data()?;
    let dashboard = template.generate_hybrid_dashboard(&hybrid_data)?;
    
    std::fs::write("comprehensive_dashboard.html", dashboard)?;
    
    println!("🎯 Comprehensive analysis complete!");
    println!("📊 Dashboard: comprehensive_dashboard.html");
    
    Ok(())
}
```

### Advanced Hybrid Analysis

```rust
use memscope_rs::unified::{
    UnifiedBackend, TrackingDispatcher, HybridStrategy
};
use memscope_rs::export::fixed_hybrid_template::*;

#[tokio::main]
async fn comprehensive_hybrid_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Configure advanced hybrid tracking
    let mut backend = UnifiedBackend::new();
    
    // Set up custom strategy mix
    let hybrid_strategy = HybridStrategy {
        single_thread_threshold: 5,     // Use single-threaded for < 5 threads
        async_task_threshold: 10,       // Switch to async tracking for > 10 tasks
        resource_monitoring: true,      // Enable system resource monitoring
        cross_boundary_tracking: true,  // Track data flow between components
    };
    
    backend.configure_hybrid_strategy(hybrid_strategy)?;
    
    // Complex application simulation
    
    // 1. Single-threaded initialization
    let config_data = vec!["config1", "config2", "config3"];
    memscope_rs::track_var!(config_data);
    
    // 2. Multi-threaded data processing
    let processing_handles: Vec<_> = (0..20).map(|worker_id| {
        std::thread::spawn(move || {
            // Heavy computation
            let mut results = Vec::new();
            for batch in 0..100 {
                let batch_data = vec![worker_id * 1000 + batch; 1000];
                let processed: Vec<_> = batch_data
                    .iter()
                    .map(|&x| x * x)
                    .collect();
                results.extend(processed);
            }
            results.len()
        })
    }).collect();
    
    // 3. Concurrent async tasks
    let async_tasks = (0..50).map(|task_id| {
        tokio::spawn(async move {
            // Simulate async I/O work
            let mut data_chunks = Vec::new();
            for chunk in 0..20 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                let chunk_data = format!("Task-{}-Chunk-{}-{}", task_id, chunk, "X".repeat(100));
                data_chunks.push(chunk_data);
            }
            data_chunks.len()
        })
    });
    
    // 4. Wait for all work to complete
    for handle in processing_handles {
        handle.join().unwrap();
    }
    
    let async_results: Vec<_> = futures::future::join_all(async_tasks).await;
    let total_async_chunks: usize = async_results.into_iter().map(|r| r.unwrap()).sum();
    
    println!("Async tasks processed {} chunks", total_async_chunks);
    
    // 5. Generate comprehensive analysis
    let comprehensive_data = backend.finalize_and_collect()?;
    
    // Create detailed hybrid dashboard
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    let dashboard = template.generate_hybrid_dashboard(&comprehensive_data)?;
    
    // Export multiple formats for different use cases
    std::fs::write("hybrid_comprehensive.html", dashboard)?;
    
    // Generate performance comparison report
    let comparison_data = comprehensive_data.generate_performance_comparison();
    let comparison_report = template.generate_comparison_report(&comparison_data)?;
    std::fs::write("performance_comparison.html", comparison_report)?;
    
    // Generate resource utilization report
    let resource_data = comprehensive_data.get_resource_utilization();
    let resource_report = template.generate_resource_report(&resource_data)?;
    std::fs::write("resource_utilization.html", resource_report)?;
    
    println!("📊 Comprehensive Hybrid Analysis Complete!");
    println!("   📁 Main dashboard: hybrid_comprehensive.html");
    println!("   ⚡ Performance comparison: performance_comparison.html");
    println!("   🖥️  Resource utilization: resource_utilization.html");
    
    // Print summary statistics
    println!("📈 Summary:");
    println!("   - Single-threaded allocations: {}", comprehensive_data.single_thread_stats.allocation_count);
    println!("   - Multi-threaded allocations: {}", comprehensive_data.multi_thread_stats.allocation_count);
    println!("   - Async task allocations: {}", comprehensive_data.async_stats.allocation_count);
    println!("   - Cross-boundary data transfers: {}", comprehensive_data.cross_boundary_transfers);
    println!("   - Peak memory usage: {:.2} MB", comprehensive_data.peak_memory_usage_mb);
    
    Ok(())
}
```

## 🏗️ Real-world Example: Full-Stack Web Application

```rust
use memscope_rs::unified::UnifiedBackend;
use memscope_rs::export::fixed_hybrid_template::FixedHybridTemplate;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn fullstack_web_application() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize hybrid tracking for full-stack app
    let mut backend = UnifiedBackend::new();
    backend.auto_configure()?;
    
    // 1. Application Configuration (Single-threaded)
    let app_config = HashMap::from([
        ("database_url", "postgresql://localhost:5432/app"),
        ("redis_url", "redis://localhost:6379"),
        ("port", "8080"),
    ]);
    memscope_rs::track_var!(app_config);
    
    println!("🚀 Starting full-stack application...");
    
    // 2. Database Connection Pool (Multi-threaded)
    let db_pool = Arc::new(DatabasePool::new(10));
    let pool_handles: Vec<_> = (0..10).map(|worker_id| {
        let pool = Arc::clone(&db_pool);
        std::thread::spawn(move || {
            // Simulate database operations
            for query_id in 0..100 {
                let connection = pool.get_connection();
                let query_result = execute_query(&connection, &format!("SELECT * FROM users WHERE id = {}", query_id));
                let processed_result = process_database_result(query_result);
                pool.return_connection(connection);
                
                // Simulate processing time
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            worker_id
        })
    }).collect();
    
    // 3. Web Server (Async tasks)
    let (request_tx, mut request_rx) = mpsc::channel(1000);
    
    // Request handlers
    let handler_tasks = (0..20).map(|handler_id| {
        let tx = request_tx.clone();
        tokio::spawn(async move {
            for request_id in 0..50 {
                // Simulate incoming HTTP requests
                let request = HttpRequest {
                    id: format!("{}-{}", handler_id, request_id),
                    path: format!("/api/users/{}", request_id),
                    method: "GET".to_string(),
                    headers: vec![("Content-Type", "application/json")],
                    body: vec![0u8; 1024], // 1KB request body
                };
                
                // Process request
                let response = process_http_request(request).await;
                
                // Send to response handler
                if tx.send(response).await.is_err() {
                    break;
                }
                
                // Simulate request interval
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
        })
    });
    
    // Response processor
    let response_processor = tokio::spawn(async move {
        let mut responses = Vec::new();
        while let Some(response) = request_rx.recv().await {
            // Simulate response processing (logging, metrics, etc.)
            let processed_response = format!(
                "{{\"status\":{},\"size\":{},\"timestamp\":{}}}",
                response.status_code,
                response.body.len(),
                chrono::Utc::now().timestamp()
            );
            responses.push(processed_response);
            
            if responses.len() >= 1000 {
                break;
            }
        }
        responses.len()
    });
    
    // 4. Background Workers (Mixed async + threading)
    let background_workers = (0..5).map(|worker_id| {
        tokio::spawn(async move {
            // Email service simulation
            for email_batch in 0..20 {
                let mut emails = Vec::new();
                for email_id in 0..50 {
                    let email = Email {
                        id: format!("email-{}-{}-{}", worker_id, email_batch, email_id),
                        to: format!("user{}@example.com", email_id),
                        subject: "Important notification".to_string(),
                        body: "A".repeat(2048), // 2KB email body
                    };
                    emails.push(email);
                }
                
                // Simulate email sending
                send_email_batch(emails).await;
                
                // Wait between batches
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            worker_id
        })
    });
    
    // 5. Wait for all components to complete
    
    // Database workers
    for handle in db_pool_handles {
        handle.join().unwrap();
    }
    
    // Web server tasks
    for task in handler_tasks {
        task.await?;
    }
    
    let total_responses = response_processor.await?;
    println!("📡 Processed {} HTTP responses", total_responses);
    
    // Background workers
    for worker in background_workers {
        let worker_id = worker.await?;
        println!("📧 Background worker {} completed", worker_id);
    }
    
    // 6. Generate comprehensive analysis
    let comprehensive_data = backend.finalize_and_collect()?;
    
    // Create detailed reports
    let template = FixedHybridTemplate::new(RenderMode::Complete);
    
    // Main dashboard
    let main_dashboard = template.generate_hybrid_dashboard(&comprehensive_data)?;
    std::fs::write("fullstack_analysis.html", main_dashboard)?;
    
    // Component-specific reports
    let db_report = template.generate_component_report(&comprehensive_data, "database")?;
    std::fs::write("database_analysis.html", db_report)?;
    
    let web_report = template.generate_component_report(&comprehensive_data, "web_server")?;
    std::fs::write("web_server_analysis.html", web_report)?;
    
    let background_report = template.generate_component_report(&comprehensive_data, "background_workers")?;
    std::fs::write("background_workers_analysis.html", background_report)?;
    
    println!("🎯 Full-stack analysis complete!");
    println!("📊 Reports generated:");
    println!("   - Main: fullstack_analysis.html");
    println!("   - Database: database_analysis.html");
    println!("   - Web Server: web_server_analysis.html");
    println!("   - Background Workers: background_workers_analysis.html");
    
    // Performance summary
    println!("📈 Performance Summary:");
    println!("   - Database throughput: {:.1} queries/sec", comprehensive_data.database_metrics.queries_per_second);
    println!("   - Web server throughput: {:.1} requests/sec", comprehensive_data.web_metrics.requests_per_second);
    println!("   - Background processing: {:.1} jobs/sec", comprehensive_data.background_metrics.jobs_per_second);
    println!("   - Peak memory usage: {:.2} GB", comprehensive_data.peak_memory_usage_mb / 1024.0);
    println!("   - Memory efficiency: {:.1}%", comprehensive_data.memory_efficiency_percent);
    
    Ok(())
}

// Helper structures and functions
struct DatabasePool {
    connections: Vec<String>,
}

impl DatabasePool {
    fn new(size: usize) -> Self {
        let connections = (0..size).map(|i| format!("connection-{}", i)).collect();
        DatabasePool { connections }
    }
    
    fn get_connection(&self) -> &str {
        &self.connections[0] // Simplified
    }
    
    fn return_connection(&self, _conn: &str) {
        // Simplified
    }
}

struct HttpRequest {
    id: String,
    path: String,
    method: String,
    headers: Vec<(&'static str, &'static str)>,
    body: Vec<u8>,
}

struct HttpResponse {
    status_code: u16,
    headers: Vec<(&'static str, &'static str)>,
    body: Vec<u8>,
}

struct Email {
    id: String,
    to: String,
    subject: String,
    body: String,
}

async fn process_http_request(request: HttpRequest) -> HttpResponse {
    // Simulate request processing
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let response_body = format!(
        "{{\"request_id\":\"{}\",\"path\":\"{}\",\"processed\":true}}",
        request.id, request.path
    );
    
    HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type", "application/json")],
        body: response_body.into_bytes(),
    }
}

fn execute_query(_connection: &str, query: &str) -> Vec<String> {
    // Simulate database query
    vec![format!("result for: {}", query)]
}

fn process_database_result(result: Vec<String>) -> Vec<String> {
    result.into_iter().map(|r| format!("processed: {}", r)).collect()
}

async fn send_email_batch(emails: Vec<Email>) {
    // Simulate email sending
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}
```

## 📊 Hybrid Analysis Features

### Cross-Component Data Flow

The hybrid module tracks data movement between different components:

```rust
use memscope_rs::unified::CrossBoundaryTracker;

fn track_data_flow() -> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = CrossBoundaryTracker::new();
    
    // Track data moving from single-threaded to multi-threaded
    let data = vec![1, 2, 3, 4, 5];
    let data_id = tracker.register_data(&data, "main_thread_data")?;
    
    let handle = std::thread::spawn(move || {
        // Data crosses thread boundary
        tracker.track_boundary_cross(data_id, "thread_worker")?;
        
        // Process in worker thread
        let processed = data.into_iter().map(|x| x * 2).collect::<Vec<_>>();
        Ok(processed)
    });
    
    let result = handle.join().unwrap()?;
    tracker.track_completion(data_id, result.len())?;
    
    Ok(())
}
```

### Performance Comparison Dashboard

The hybrid module generates comparative analysis across all tracking modes:

```rust
// Generated dashboard includes:
// - Memory usage comparison (single vs multi vs async)
// - Performance bottleneck identification
// - Resource utilization across components
// - Cross-boundary transfer efficiency
// - Scalability analysis recommendations
```

## 🔗 Next Steps

- **[API Reference](api-reference/)** - Complete API documentation
- **[Examples](examples/integration-examples.md)** - Full integration examples
- **[Performance Optimization](advanced/performance-optimization.md)** - Optimization tips