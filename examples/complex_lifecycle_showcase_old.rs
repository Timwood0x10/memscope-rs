//! Complex example demonstrating memscope-rs capabilities
//!
//! This example simulates a realistic application with multiple components:
//! - Web server with request handling
//! - Database connection pool
//! - Cache system with TTL
//! - Background task processing
//! - Memory-intensive data processing pipeline

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Simulates a user session with associated data
#[derive(Debug, Clone)]
struct UserSession {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    user_data: Vec<u8>,
    #[allow(dead_code)]
    preferences: HashMap<String, String>,
    cache: Vec<String>,
}

impl UserSession {
    fn new(id: String, data_size: usize) -> Self {
        let mut preferences = HashMap::new();
        preferences.insert("theme".to_string(), "dark".to_string());
        preferences.insert("language".to_string(), "en".to_string());
        preferences.insert("timezone".to_string(), "UTC".to_string());

        Self {
            id,
            user_data: vec![0u8; data_size],
            preferences,
            cache: Vec::new(),
        }
    }

    fn add_to_cache(&mut self, item: String) {
        self.cache.push(item);
        // Limit cache size
        if self.cache.len() > 100 {
            self.cache.remove(0);
        }
    }
}

/// Simulates a database connection pool
struct DatabasePool {
    #[allow(dead_code)]
    connections: Vec<DatabaseConnection>,
    query_cache: HashMap<String, Vec<u8>>,
}

struct DatabaseConnection {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    buffer: Vec<u8>,
    #[allow(dead_code)]
    is_active: bool,
}

impl DatabasePool {
    fn new(pool_size: usize) -> Self {
        let connections = (0..pool_size)
            .map(|id| DatabaseConnection {
                id,
                buffer: vec![0u8; 4096], // 4KB buffer per connection
                is_active: false,
            })
            .collect();

        Self {
            connections,
            query_cache: HashMap::new(),
        }
    }

    fn execute_query(&mut self, query: &str) -> Vec<u8> {
        // Simulate query execution with result caching
        if let Some(cached_result) = self.query_cache.get(query) {
            return cached_result.clone();
        }

        // Simulate query result
        let result = format!("Result for: {query}").into_bytes();
        self.query_cache.insert(query.to_string(), result.clone());
        result
    }
}

/// Simulates a web server handling requests
struct WebServer {
    sessions: HashMap<String, UserSession>,
    request_log: Vec<String>,
    response_cache: HashMap<String, Vec<u8>>,
}

impl WebServer {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            request_log: Vec::new(),
            response_cache: HashMap::new(),
        }
    }

    fn handle_request(&mut self, user_id: &str, endpoint: &str) -> Vec<u8> {
        // Log the request
        let log_entry = format!("User {user_id} accessed {endpoint}");
        self.request_log.push(log_entry);

        // Get or create user session
        let session = self
            .sessions
            .entry(user_id.to_string())
            .or_insert_with(|| UserSession::new(user_id.to_string(), 2048));

        // Add to user cache
        session.add_to_cache(endpoint.to_string());

        // Generate response
        let response_key = format!("{user_id}:{endpoint}");
        if let Some(cached_response) = self.response_cache.get(&response_key) {
            return cached_response.clone();
        }

        let response = match endpoint {
            "/api/profile" => format!("Profile data for {user_id}").into_bytes(),
            "/api/dashboard" => vec![1u8; 8192], // 8KB dashboard data
            "/api/analytics" => vec![2u8; 16384], // 16KB analytics data
            _ => b"404 Not Found".to_vec(),
        };

        self.response_cache.insert(response_key, response.clone());
        response
    }
}

/// Simulates background task processing
fn background_task_processor() -> Vec<String> {
    let mut results = Vec::new();

    // Simulate processing multiple tasks
    for task_id in 0..50 {
        // Each task processes some data
        let task_data = vec![task_id as u8; 1024]; // 1KB per task
        let result = format!("Task {} completed with {} bytes", task_id, task_data.len());
        results.push(result);

        // Simulate some processing time
        thread::sleep(Duration::from_millis(1));
    }

    results
}

/// Simulates data processing pipeline
fn data_processing_pipeline() -> Vec<Vec<f64>> {
    let mut pipeline_results = Vec::new();

    // Stage 1: Generate raw data
    let raw_data: Vec<i32> = (0..10000).map(|i| i * 2).collect();

    // Stage 2: Transform to floating point
    let float_data: Vec<f64> = raw_data.iter().map(|&x| x as f64 / 100.0).collect();

    // Stage 3: Apply statistical operations
    let chunks: Vec<Vec<f64>> = float_data
        .chunks(100)
        .map(|chunk| {
            let mean = chunk.iter().sum::<f64>() / chunk.len() as f64;
            let variance =
                chunk.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / chunk.len() as f64;
            vec![mean, variance, chunk.len() as f64]
        })
        .collect();

    pipeline_results.extend(chunks);
    pipeline_results
}

/// Main application simulation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memscope-rs
    init();
    println!("Starting Complex Application Memory Analysis Demo");
    println!("{}", "=".repeat(60));

    let start_time = Instant::now();

    // 1. Initialize core components
    println!("\nPhase 1: Initializing Core Components");

    let mut web_server = WebServer::new();
    // Note: WebServer doesn't implement Trackable, so we track its components instead

    let mut db_pool = DatabasePool::new(10);
    // Note: DatabasePool doesn't implement Trackable, so we track its components instead

    let shared_config = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    // Add some configuration
    {
        let mut config = shared_config.lock().unwrap();
        config.insert("max_connections".to_string(), "100".to_string());
        config.insert("cache_size".to_string(), "1024".to_string());
        config.insert("log_level".to_string(), "info".to_string());
    }

    let _tracked_shared_config = track_var!(shared_config.clone());

    println!("Core components initialized");

    // 2. Simulate user traffic
    println!("\nPhase 2: Simulating User Traffic");

    let users = vec!["alice", "bob", "charlie", "diana", "eve"];
    let endpoints = vec!["/api/profile", "/api/dashboard", "/api/analytics"];

    for user in &users {
        for endpoint in &endpoints {
            let response = web_server.handle_request(user, endpoint);
            let _tracked_response = track_var!(response);

            // Simulate database queries
            let query = format!(
                "SELECT * FROM {} WHERE user = '{}'",
                endpoint.replace("/api/", ""),
                user
            );
            let db_result = db_pool.execute_query(&query);
            let _tracked_db_result = track_var!(db_result);

            // Track the query string as well
            let query_string = query;
            let _tracked_query_string = track_var!(query_string);
        }
    }

    println!("Processed {} user requests", users.len() * endpoints.len());

    // 3. Background processing
    println!("\nPhase 3: Background Task Processing");

    let task_results = background_task_processor();
    let _tracked_task_results = track_var!(task_results);

    println!("Completed background task processing");

    // 4. Data processing pipeline
    println!("\nPhase 4: Data Processing Pipeline");

    let pipeline_results = data_processing_pipeline();
    let _tracked_pipeline_results = track_var!(pipeline_results);

    println!("Data processing pipeline completed");

    // 5. Memory-intensive operations
    println!("\nPhase 5: Memory-Intensive Operations");

    // Large data structures
    let large_dataset = vec![0u8; 1024 * 1024]; // 1MB
    let _tracked_large_dataset = track_var!(large_dataset);

    let string_collection: Vec<String> = (0..1000)
        .map(|i| format!("String item number {i} with some additional content"))
        .collect();
    let _tracked_string_collection = track_var!(string_collection);

    // Reference counted data
    let shared_data = std::rc::Rc::new(vec![0u64; 10000]);

    let shared_clone = std::rc::Rc::clone(&shared_data);
    let _tracked_shared_data = track_var!(shared_data);
    let _tracked_shared_clone = track_var!(shared_clone);

    // Thread-safe shared data
    let arc_data = Arc::new(String::from("Shared across threads"));
    let _tracked_arc_data = track_var!(arc_data);

    println!("Memory-intensive operations completed");

    // 6. Multi-threaded processing
    println!("\nPhase 6: Multi-threaded Processing");

    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            let config_clone = Arc::clone(&shared_config);
            thread::spawn(move || {
                let thread_data = vec![thread_id as u8; 4096];
                // Note: track_var! doesn't work across thread boundaries
                // This is expected behavior for the current implementation

                // Simulate some work
                let _result: Vec<_> = thread_data.iter().map(|x| x * 2).collect();

                // Access shared config
                let _config = config_clone.lock().unwrap();

                format!("Thread {thread_id} completed")
            })
        })
        .collect();

    let thread_results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let _tracked_thread_results = track_var!(thread_results);

    println!("Multi-threaded processing completed");

    // 7. Memory analysis and reporting
    println!("\nPhase 7: Memory Analysis & Reporting");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    let memory_by_type = tracker.get_memory_by_type()?;

    println!("\nMemory Usage Statistics:");
    println!("  Active Allocations: {}", stats.active_allocations);
    println!(
        "  Active Memory: {:.2} MB",
        stats.active_memory as f64 / 1024.0 / 1024.0
    );
    println!(
        "  Peak Memory: {:.2} MB",
        stats.peak_memory as f64 / 1024.0 / 1024.0
    );
    println!("  Total Allocations: {}", stats.total_allocations);
    println!("  Total Deallocations: {}", stats.total_deallocations);

    println!("\nMemory Usage by Type:");
    for (i, type_info) in memory_by_type.iter().take(10).enumerate() {
        println!(
            "  {}. {}: {:.2} KB ({} allocations)",
            i + 1,
            type_info.type_name,
            type_info.total_size as f64 / 1024.0,
            type_info.allocation_count
        );
    }

    // 8. Export comprehensive analysis
    println!("\nPhase 8: Exporting Analysis");

    let json_filename = "complex_app_analysis.json";
    let svg_filename = "complex_app_visualization.svg";

    tracker.export_to_json(json_filename)?;
    tracker.export_memory_analysis(svg_filename)?;

    println!("Exported detailed analysis to:");
    println!("   JSON: {json_filename}");
    println!("   SVG:  {svg_filename}");

    // 9. Performance summary
    let elapsed = start_time.elapsed();
    println!("\nPerformance Summary:");
    println!("  Total execution time: {:.2}s", elapsed.as_secs_f64());
    println!("  Memory tracking overhead: Minimal");
    println!("  Application phases: 8/8 completed successfully");

    println!("\nComplex Application Demo Completed Successfully!");
    println!("{}", "=".repeat(60));
    println!("Check the exported files for detailed memory analysis:");
    println!("   • {json_filename} - Detailed JSON data for programmatic analysis");
    println!("   • {svg_filename} - Visual SVG charts for human analysis");
    println!("\nThis demo showcased:");
    println!("   ✓ Web server simulation with user sessions");
    println!("   ✓ Database connection pooling");
    println!("   ✓ Caching mechanisms");
    println!("   ✓ Background task processing");
    println!("   ✓ Data processing pipelines");
    println!("   ✓ Memory-intensive operations");
    println!("   ✓ Multi-threaded processing");
    println!("   ✓ Comprehensive memory analysis");

    Ok(())
}
