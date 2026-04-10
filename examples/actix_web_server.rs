//! Actix-web server memory tracking demonstration for memscope-rs.
//!
//! This example demonstrates memory tracking in a real web server scenario:
//! - Starting an actix-web server with memory tracking
//! - Handling HTTP requests with tracked allocations
//! - Graceful shutdown and report generation
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example actix_web_server
//! ```

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use memscope_rs::{global_tracker, init_global_tracking, track, MemScopeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Global request counter for tracking traffic.
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

/// In-memory data store for the server.
#[derive(Debug, Default)]
struct DataStore {
    /// Key-value storage.
    data: HashMap<String, Vec<u8>>,
    /// Total bytes stored.
    total_bytes: usize,
}

impl DataStore {
    /// Creates a new empty data store.
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            total_bytes: 0,
        }
    }

    /// Inserts a key-value pair into the store.
    fn insert(&mut self, key: String, value: Vec<u8>) {
        self.total_bytes += value.len();
        self.data.insert(key, value);
    }

    /// Gets a value by key.
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.data.get(key)
    }

    /// Returns the total bytes stored.
    fn memory_usage(&self) -> usize {
        self.total_bytes
    }

    /// Returns the number of items stored.
    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Request payload for data insertion.
#[derive(Debug, Deserialize)]
struct InsertRequest {
    /// Key for the data.
    key: String,
    /// Value to store (base64 encoded or raw bytes).
    value: String,
}

/// Response for data retrieval.
#[derive(Debug, Serialize)]
struct GetResponse {
    /// Whether the key was found.
    found: bool,
    /// The value if found.
    value: Option<String>,
    /// Size of the value in bytes.
    size: Option<usize>,
}

/// Response for server statistics.
#[derive(Debug, Serialize)]
struct StatsResponse {
    /// Total requests processed.
    total_requests: u64,
    /// Number of items in store.
    items_count: usize,
    /// Memory usage in bytes.
    memory_usage: usize,
}

/// Health check endpoint.
#[get("/health")]
async fn health() -> HttpResponse {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get server statistics.
#[get("/stats")]
async fn server_stats(store: web::Data<Arc<parking_lot::Mutex<DataStore>>>) -> HttpResponse {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    let store = store.lock();
    let response = StatsResponse {
        total_requests: REQUEST_COUNT.load(Ordering::Relaxed),
        items_count: store.len(),
        memory_usage: store.memory_usage(),
    };

    HttpResponse::Ok().json(response)
}

/// Insert data into the store.
#[post("/insert")]
async fn insert(
    store: web::Data<Arc<parking_lot::Mutex<DataStore>>>,
    payload: web::Json<InsertRequest>,
    tracker: web::Data<Arc<memscope_rs::GlobalTracker>>,
) -> HttpResponse {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    // Create the value buffer.
    let value = payload.value.as_bytes().to_vec();
    let value_size = value.len();

    // Track the allocation.
    track!(tracker.as_ref(), value);

    // Store the data.
    {
        let mut store = store.lock();
        store.insert(payload.key.clone(), value);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "status": "inserted",
        "key": payload.key,
        "size": value_size
    }))
}

/// Get data from the store.
#[get("/get/{key}")]
async fn get(
    store: web::Data<Arc<parking_lot::Mutex<DataStore>>>,
    path: web::Path<String>,
) -> HttpResponse {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);

    let key = path.into_inner();
    let store = store.lock();

    match store.get(&key) {
        Some(value) => {
            let response = GetResponse {
                found: true,
                value: Some(String::from_utf8_lossy(value).to_string()),
                size: Some(value.len()),
            };
            HttpResponse::Ok().json(response)
        }
        None => {
            let response = GetResponse {
                found: false,
                value: None,
                size: None,
            };
            HttpResponse::NotFound().json(response)
        }
    }
}

/// Simulates client requests to the server.
async fn simulate_client_requests(
    tracker: Arc<memscope_rs::GlobalTracker>,
    store: Arc<parking_lot::Mutex<DataStore>>,
) -> MemScopeResult<()> {
    println!("\n=== Simulating Client Requests ===\n");

    // Simulate various request patterns.
    let test_data = vec![
        ("user_1", "Alice's profile data with some extra information"),
        ("user_2", "Bob's profile data"),
        ("session_1", "Session token and metadata"),
        ("cache_1", "Cached computation result with large payload"),
        ("config", "Application configuration settings"),
    ];

    // Insert data items.
    for (key, value) in &test_data {
        let mut store = store.lock();
        let data = value.as_bytes().to_vec();
        track!(tracker, data);
        store.insert(key.to_string(), data);

        println!("  Inserted {}: {} bytes", key, value.len());
    }

    // Simulate repeated access patterns.
    println!("\n  Simulating repeated access patterns...");
    for i in 0..20 {
        let key = format!("temp_{}", i);
        let value = vec![i as u8; 1024]; // 1 KB per temp item

        let mut store = store.lock();
        track!(tracker, value);
        store.insert(key, value);

        if i % 5 == 0 {
            println!("  Processed {} temporary items", i + 1);
        }
    }

    // Show final statistics.
    let store = store.lock();
    println!(
        "\n  Final store: {} items, {} bytes",
        store.len(),
        store.memory_usage()
    );

    Ok(())
}

/// Main entry point for the actix-web server demonstration.
#[actix_web::main]
async fn main() -> MemScopeResult<()> {
    println!("==============================================");
    println!("  Actix-Web Server Memory Tracking Demo      ");
    println!("==============================================\n");

    // Initialize memory tracking.
    init_global_tracking()?;
    let tracker = global_tracker()?;

    println!("Memory tracking initialized.\n");

    // Create shared data store.
    let store = Arc::new(parking_lot::Mutex::new(DataStore::new()));
    let store_clone = Arc::clone(&store);
    let tracker_clone = Arc::clone(&tracker);

    // Track the data store.
    {
        let store_guard = store.lock();
        track!(tracker, store_guard.data);
    }

    // Start the server in a background task.
    println!("Starting actix-web server on http://127.0.0.1:8080...\n");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&store_clone)))
            .app_data(web::Data::new(Arc::clone(&tracker_clone)))
            .service(health)
            .service(server_stats)
            .service(insert)
            .service(get)
    })
    .bind("127.0.0.1:8080")?
    .run();

    // Run server in background.
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for server to start.
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Simulate client requests.
    simulate_client_requests(Arc::clone(&tracker), Arc::clone(&store)).await?;

    // Make some HTTP requests.
    println!("\n=== Making HTTP Requests ===\n");

    let client = reqwest::Client::new();

    // Health check.
    match client.get("http://127.0.0.1:8080/health").send().await {
        Ok(resp) => {
            println!("  GET /health: {}", resp.status());
        }
        Err(e) => {
            println!("  Request failed: {}", e);
        }
    }

    // Get stats.
    match client.get("http://127.0.0.1:8080/stats").send().await {
        Ok(resp) => {
            println!("  GET /stats: {}", resp.status());
            if let Ok(text) = resp.text().await {
                println!("  Response: {}", text);
            }
        }
        Err(e) => {
            println!("  Request failed: {}", e);
        }
    }

    // Insert data via HTTP.
    match client
        .post("http://127.0.0.1:8080/insert")
        .json(&serde_json::json!({
            "key": "http_data",
            "value": "Data inserted via HTTP request"
        }))
        .send()
        .await
    {
        Ok(resp) => {
            println!("  POST /insert: {}", resp.status());
        }
        Err(e) => {
            println!("  Request failed: {}", e);
        }
    }

    // Get data via HTTP.
    match client
        .get("http://127.0.0.1:8080/get/http_data")
        .send()
        .await
    {
        Ok(resp) => {
            println!("  GET /get/http_data: {}", resp.status());
            if let Ok(text) = resp.text().await {
                println!("  Response: {}", text);
            }
        }
        Err(e) => {
            println!("  Request failed: {}", e);
        }
    }

    // Graceful shutdown.
    println!("\n=== Shutting Down Server ===\n");
    server_handle.abort();

    // Wait a moment for cleanup.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Generate memory analysis report.
    println!("=== Memory Analysis Report ===\n");

    let mem_stats = tracker.get_stats();
    println!("  Total allocations: {}", mem_stats.total_allocations);
    println!("  Active allocations: {}", mem_stats.active_allocations);
    println!("  Peak memory usage: {} bytes", mem_stats.peak_memory_bytes);
    println!("  Current memory: {} bytes", mem_stats.current_memory_bytes);

    // Export reports.
    println!("\n=== Exporting Reports ===\n");

    let output_path = "MemoryAnalysis/actix_web_server";
    tracker.export_json(output_path)?;
    println!("  JSON report: {}/memory_snapshots.json", output_path);

    tracker.export_html(output_path)?;
    println!("  HTML dashboard: {}/dashboard.html", output_path);

    println!("\n==============================================");
    println!("  Demo Complete!                              ");
    println!("==============================================");

    println!("\nOpen the HTML dashboard to visualize server memory usage.");
    println!("Dashboard location: {}/dashboard.html", output_path);

    Ok(())
}
