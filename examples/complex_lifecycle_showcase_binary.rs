// Complex lifecycle showcase with binary export - Performance comparison version
// This example measures the performance difference between JSON and binary export

use memscope_rs::{get_global_tracker, init, track_var};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

// Custom data structures to demonstrate tracking
#[derive(Debug, Clone)]
struct User {
    #[allow(dead_code)]
    id: u64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    email: String,
    preferences: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct DatabaseConnection {
    #[allow(dead_code)]
    host: String,
    #[allow(dead_code)]
    port: u16,
    connection_pool: Vec<String>,
    active_queries: VecDeque<String>,
}

#[derive(Debug)]
struct CacheEntry<T> {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    value: T,
    #[allow(dead_code)]
    timestamp: u64,
    #[allow(dead_code)]
    access_count: usize,
}

#[derive(Debug)]
struct GraphNode {
    #[allow(dead_code)]
    id: usize,
    #[allow(dead_code)]
    data: String,
    neighbors: Vec<usize>,
    metadata: BTreeMap<String, i32>,
}

#[derive(Debug)]
struct StreamProcessor {
    buffer: Vec<u8>,
    processed_count: usize,
    error_log: Vec<String>,
}

fn main() {
    init();
    println!("🚀 Complex Lifecycle Showcase - Binary Export Performance Test");
    println!("==============================================================");
    println!("Measuring performance difference between JSON and binary export");
    println!();

    // Start total timing
    let total_start = Instant::now();

    // Global scope variables
    let global_app_config = String::from("app_config_v2.0");
    track_var!(global_app_config);
    let global_session_store = Box::new(HashMap::<String, String>::new());
    track_var!(global_session_store);

    // Keep all variables alive until the end by collecting them
    let mut _keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

    // Data collection phase timing
    let data_collection_start = Instant::now();

    // Phase 1: Basic built-in types with nested scopes
    let vars1 = demonstrate_builtin_types();
    _keep_alive.extend(vars1);

    // Phase 2: Smart pointers and reference counting
    let vars2 = demonstrate_smart_pointers();
    _keep_alive.extend(vars2);

    // Phase 3: Custom data structures
    let vars3 = demonstrate_custom_structures();
    _keep_alive.extend(vars3);

    // Phase 4: Complex memory patterns
    let vars4 = demonstrate_complex_patterns();
    _keep_alive.extend(vars4);

    // Phase 5: Web server simulation
    let vars5 = simulate_web_server_scenario();
    _keep_alive.extend(vars5);

    // Phase 6: Data processing pipeline
    let vars6 = simulate_data_processing_pipeline();
    _keep_alive.extend(vars6);

    let data_collection_time = data_collection_start.elapsed();

    // Global cleanup
    let global_cleanup_log = Vec::<String>::new();
    track_var!(global_cleanup_log);

    // Generate analysis and export with timing
    let export_start = Instant::now();
    generate_final_analysis_with_binary_export();
    let export_time = export_start.elapsed();

    let total_time = total_start.elapsed();

    // Performance summary
    println!("\n⏱️  Performance Summary");
    println!("======================");
    println!(
        "Data Collection Time: {:.2}ms",
        data_collection_time.as_secs_f64() * 1000.0
    );
    println!(
        "Export Processing Time: {:.2}ms",
        export_time.as_secs_f64() * 1000.0
    );
    println!("Total Runtime: {:.2}ms", total_time.as_secs_f64() * 1000.0);
    println!("Variables Tracked: {}", _keep_alive.len());
    println!();
    println!("📊 Binary export provides faster serialization and smaller file sizes");
    println!("🎯 Compare with JSON version to see performance differences");
}

fn demonstrate_builtin_types() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // Vectors with different growth patterns
    let mut small_vec = Vec::with_capacity(5);
    track_var!(small_vec);
    for i in 0..10 {
        small_vec.push(i);
    }
    keep_alive.push(Box::new(small_vec) as Box<dyn std::any::Any>);

    let mut large_vec = Vec::with_capacity(1000);
    track_var!(large_vec);
    for i in 0..2000 {
        large_vec.push(format!("Item {i}"));
    }
    keep_alive.push(Box::new(large_vec) as Box<dyn std::any::Any>);

    // Strings with different patterns
    let mut growing_string = String::new();
    track_var!(growing_string);
    for i in 0..100 {
        growing_string.push_str(&format!("Data chunk {i} | "));
    }
    keep_alive.push(Box::new(growing_string) as Box<dyn std::any::Any>);

    let static_string = String::from("Static content that doesn't grow");
    track_var!(static_string);
    keep_alive.push(Box::new(static_string) as Box<dyn std::any::Any>);

    // Collections (using Box to make them trackable)
    let mut hash_map = HashMap::new();
    for i in 0..500 {
        hash_map.insert(format!("key_with_longer_string_{i}"), i * 2);
    }
    let boxed_hash_map = Box::new(hash_map);
    track_var!(boxed_hash_map);
    keep_alive.push(boxed_hash_map as Box<dyn std::any::Any>);

    let mut hash_set = HashSet::new();
    for i in 0..250 {
        hash_set.insert(format!("unique_item_with_longer_name_{i}"));
    }
    let boxed_hash_set = Box::new(hash_set);
    track_var!(boxed_hash_set);
    keep_alive.push(boxed_hash_set as Box<dyn std::any::Any>);

    let mut btree_map = BTreeMap::new();
    for i in 0..300 {
        btree_map.insert(i, format!("value_with_much_longer_string_data_{i}"));
    }
    let boxed_btree_map = Box::new(btree_map);
    track_var!(boxed_btree_map);
    keep_alive.push(boxed_btree_map as Box<dyn std::any::Any>);

    let mut vec_deque = VecDeque::new();
    for i in 0..400 {
        if i % 2 == 0 {
            vec_deque.push_back(i);
        } else {
            vec_deque.push_front(i);
        }
    }
    let boxed_vec_deque = Box::new(vec_deque);
    track_var!(boxed_vec_deque);
    keep_alive.push(boxed_vec_deque as Box<dyn std::any::Any>);

    keep_alive
}

fn demonstrate_smart_pointers() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // Box pointers
    let boxed_large_data = Box::new(vec![0u8; 1024]);
    track_var!(boxed_large_data);

    let boxed_string = Box::new(String::from("Boxed string data"));
    track_var!(boxed_string);

    // Reference counting with Rc
    let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
    track_var!(shared_data);
    let shared_clone1 = Rc::clone(&shared_data);
    track_var!(shared_clone1);
    let shared_clone2 = Rc::clone(&shared_data);
    track_var!(shared_clone2);

    // Thread-safe reference counting with Arc
    let thread_safe_data = Arc::new(String::from("Thread-safe shared string"));
    track_var!(thread_safe_data);
    let arc_clone = Arc::clone(&thread_safe_data);
    track_var!(arc_clone);

    // Interior mutability with RefCell
    let mutable_data = Rc::new(RefCell::new(vec![10, 20, 30]));
    track_var!(mutable_data);
    {
        let mut borrowed = mutable_data.borrow_mut();
        borrowed.push(40);
        borrowed.push(50);
    }

    // Keep smart pointers alive
    keep_alive.push(Box::new(boxed_large_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(boxed_string) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(shared_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(thread_safe_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(mutable_data) as Box<dyn std::any::Any>);

    keep_alive
}

fn demonstrate_custom_structures() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // User struct with nested collections
    let mut user = User {
        id: 12345,
        name: String::from("Alice Johnson"),
        email: String::from("alice.johnson@example.com"),
        preferences: HashMap::new(),
    };
    user.preferences
        .insert("theme".to_string(), "dark".to_string());
    user.preferences
        .insert("language".to_string(), "en".to_string());
    user.preferences
        .insert("notifications".to_string(), "enabled".to_string());
    let boxed_user = Box::new(user);
    track_var!(boxed_user);
    keep_alive.push(boxed_user as Box<dyn std::any::Any>);

    // Database connection simulation
    let mut db_conn = DatabaseConnection {
        host: String::from("localhost"),
        port: 5432,
        connection_pool: Vec::new(),
        active_queries: VecDeque::new(),
    };

    for i in 0..10 {
        db_conn.connection_pool.push(format!("conn_{i}"));
        db_conn
            .active_queries
            .push_back(format!("SELECT * FROM table_{i}"));
    }
    let boxed_db_conn = Box::new(db_conn);
    track_var!(boxed_db_conn);
    keep_alive.push(boxed_db_conn as Box<dyn std::any::Any>);

    // Cache entries with generic types
    let string_cache = CacheEntry {
        key: String::from("user_session_12345"),
        value: String::from("session_data_payload"),
        timestamp: 1640995200,
        access_count: 0,
    };
    let boxed_string_cache = Box::new(string_cache);
    track_var!(boxed_string_cache);
    keep_alive.push(boxed_string_cache as Box<dyn std::any::Any>);

    let vec_cache = CacheEntry {
        key: String::from("computed_results"),
        value: vec![1.0, 2.5, 3.7, 4.2, 5.9],
        timestamp: 1640995300,
        access_count: 0,
    };
    let boxed_vec_cache = Box::new(vec_cache);
    track_var!(boxed_vec_cache);
    keep_alive.push(boxed_vec_cache as Box<dyn std::any::Any>);

    // Graph node with complex relationships
    let mut graph_nodes = Vec::new();
    for i in 0..5 {
        let mut node = GraphNode {
            id: i,
            data: format!("Node {i} data"),
            neighbors: Vec::new(),
            metadata: BTreeMap::new(),
        };

        // Add some neighbors
        for j in 0..3 {
            if (i + j) % 5 != i {
                node.neighbors.push((i + j) % 5);
            }
        }

        // Add metadata
        node.metadata.insert("weight".to_string(), (i as i32) * 10);
        node.metadata.insert("priority".to_string(), 5 - (i as i32));

        graph_nodes.push(node);
    }
    track_var!(graph_nodes);
    keep_alive.push(Box::new(graph_nodes) as Box<dyn std::any::Any>);

    keep_alive
}

fn demonstrate_complex_patterns() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // Nested collections
    let mut nested_structure = HashMap::new();
    for i in 0..5 {
        let mut inner_map = BTreeMap::new();
        for j in 0..10 {
            inner_map.insert(j, vec![format!("item_{}_{}", i, j); 3]);
        }
        nested_structure.insert(format!("group_{i}"), inner_map);
    }
    let boxed_nested = Box::new(nested_structure);
    track_var!(boxed_nested);
    keep_alive.push(boxed_nested as Box<dyn std::any::Any>);

    // Circular reference simulation
    let node_a = Rc::new(RefCell::new(vec!["Node A data".to_string()]));
    let node_b = Rc::new(RefCell::new(vec!["Node B data".to_string()]));
    track_var!(node_a);
    track_var!(node_b);
    keep_alive.push(Box::new(node_a) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(node_b) as Box<dyn std::any::Any>);

    // Memory-intensive computation result
    let mut computation_result = Vec::new();
    for i in 0..1000 {
        computation_result.push(format!("Result {}: {}", i, i * i));
    }
    track_var!(computation_result);
    keep_alive.push(Box::new(computation_result) as Box<dyn std::any::Any>);

    // Stream processing buffer
    let mut stream_processor = StreamProcessor {
        buffer: Vec::with_capacity(2048),
        processed_count: 0,
        error_log: Vec::new(),
    };

    // Simulate processing
    for i in 0..3000 {
        stream_processor.buffer.push((i % 256) as u8);
        if i % 100 == 0 {
            stream_processor.processed_count += 100;
            if i % 500 == 0 {
                stream_processor
                    .error_log
                    .push(format!("Warning at byte {i}"));
            }
        }
    }
    let boxed_stream_processor = Box::new(stream_processor);
    track_var!(boxed_stream_processor);
    keep_alive.push(boxed_stream_processor as Box<dyn std::any::Any>);

    keep_alive
}

fn simulate_web_server_scenario() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // Request routing table
    let mut routes = HashMap::new();
    routes.insert(
        "/api/users".to_string(),
        "UserController::index".to_string(),
    );
    routes.insert(
        "/api/users/{id}".to_string(),
        "UserController::show".to_string(),
    );
    routes.insert(
        "/api/posts".to_string(),
        "PostController::index".to_string(),
    );
    routes.insert(
        "/api/auth/login".to_string(),
        "AuthController::login".to_string(),
    );
    let boxed_routes = Box::new(routes);
    track_var!(boxed_routes);
    keep_alive.push(boxed_routes as Box<dyn std::any::Any>);

    // Session storage
    let mut sessions = HashMap::new();
    for i in 0..50 {
        let session_id = format!("sess_{i:08x}");
        let session_data = HashMap::from([
            ("user_id".to_string(), format!("{}", 1000 + i)),
            (
                "csrf_token".to_string(),
                format!("token_{:016x}", i * 12345),
            ),
            (
                "last_activity".to_string(),
                format!("{}", 1640995200 + i * 60),
            ),
        ]);
        sessions.insert(session_id, session_data);
    }
    let boxed_sessions = Box::new(sessions);
    track_var!(boxed_sessions);
    keep_alive.push(boxed_sessions as Box<dyn std::any::Any>);

    // Request log buffer
    let mut request_log = VecDeque::new();
    for i in 0..200 {
        let log_entry = format!(
            "[{}] GET /api/endpoint_{} - 200 OK - {}ms",
            1640995200 + i,
            i % 10,
            10 + (i % 50)
        );
        request_log.push_back(log_entry);

        // Keep only last 100 entries
        if request_log.len() > 100 {
            request_log.pop_front();
        }
    }
    let boxed_request_log = Box::new(request_log);
    track_var!(boxed_request_log);
    keep_alive.push(boxed_request_log as Box<dyn std::any::Any>);

    keep_alive
}

fn simulate_data_processing_pipeline() -> Vec<Box<dyn std::any::Any>> {
    let mut keep_alive = Vec::new();

    // Input data queue
    let mut input_queue = VecDeque::new();
    for i in 0..500 {
        input_queue.push_back(format!("data_record_{i:06}"));
    }
    let boxed_input_queue = Box::new(input_queue);
    track_var!(boxed_input_queue);
    keep_alive.push(boxed_input_queue as Box<dyn std::any::Any>);

    // Processing stages
    let mut stage1_results = Vec::new();
    let mut stage2_results = Vec::new();
    let mut final_results = HashMap::new();

    // Stage 1: Parse and validate
    for i in 0..500 {
        let processed = format!("validated_data_record_{i:06}");
        stage1_results.push(processed);
    }
    track_var!(stage1_results);
    keep_alive.push(Box::new(stage1_results.clone()) as Box<dyn std::any::Any>);

    // Stage 2: Transform and enrich
    for record in &stage1_results {
        let enriched = format!("enriched_{record}_with_metadata");
        stage2_results.push(enriched);
    }
    track_var!(stage2_results);
    keep_alive.push(Box::new(stage2_results.clone()) as Box<dyn std::any::Any>);

    // Final stage: Aggregate and index
    for (i, record) in stage2_results.iter().enumerate() {
        let key = format!("index_{}", i / 10);
        final_results
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record.clone());
    }
    let boxed_final_results = Box::new(final_results);
    track_var!(boxed_final_results);
    keep_alive.push(boxed_final_results as Box<dyn std::any::Any>);

    // Error tracking
    let mut error_tracker = Vec::new();
    for i in 0..25 {
        error_tracker.push(format!(
            "Error {}: Processing failed for record {}",
            i,
            i * 20
        ));
    }
    track_var!(error_tracker);
    keep_alive.push(Box::new(error_tracker) as Box<dyn std::any::Any>);

    keep_alive
}

fn generate_final_analysis_with_binary_export() {
    println!("📊 Final Analysis & Binary Export");
    println!("=================================");

    let tracker = get_global_tracker();

    // Get comprehensive statistics with timing
    let stats_start = Instant::now();
    if let Ok(stats) = tracker.get_stats() {
        let stats_time = stats_start.elapsed();

        println!(
            "Memory Statistics (collected in {:.2}ms):",
            stats_time.as_secs_f64() * 1000.0
        );
        println!("  • Total allocations: {}", stats.total_allocations);
        println!("  • Active allocations: {}", stats.active_allocations);
        println!(
            "  • Active memory: {} bytes ({:.2} MB)",
            stats.active_memory,
            stats.active_memory as f64 / 1024.0 / 1024.0
        );
        println!(
            "  • Peak memory: {} bytes ({:.2} MB)",
            stats.peak_memory,
            stats.peak_memory as f64 / 1024.0 / 1024.0
        );

        let lifecycle = &stats.lifecycle_stats;
        println!("\nLifecycle Analysis:");
        println!(
            "  • Average lifetime: {:.2}ms",
            lifecycle.average_lifetime_ms
        );
        println!(
            "  • Completed allocations: {}",
            lifecycle.completed_allocations
        );
        println!(
            "  • Memory growth events: {}",
            lifecycle.memory_growth_events
        );
        println!(
            "  • Peak concurrent variables: {}",
            lifecycle.peak_concurrent_variables
        );
        println!(
            "  • Memory efficiency: {:.2}%",
            lifecycle.memory_efficiency_ratio * 100.0
        );
        println!(
            "  • Ownership transfers: {}",
            lifecycle.ownership_transfer_events
        );
        println!(
            "  • Fragmentation score: {:.3}",
            lifecycle.fragmentation_score
        );
    }

    // Export with detailed timing measurements
    println!("\n🎨 Binary Export and Conversion Performance:");

    // Step 1: Binary export timing
    let binary_export_start = Instant::now();
    if let Err(e) = tracker.export_to_binary("complex_lifecycle_binary") {
        println!("❌ Binary export failed: {e}");
        return;
    }
    let binary_export_time = binary_export_start.elapsed();
    println!(
        "✅ Binary export completed in {:.2}ms",
        binary_export_time.as_secs_f64() * 1000.0
    );

    // Step 2: Binary -> JSON conversion timing
    let conversion_start = Instant::now();
    let binary_path = "MemoryAnalysis/complex_lifecycle_binary.memscope";
    let json_path = "MemoryAnalysis/complex_lifecycle_from_binary.json";

    let conversion_time = if let Err(e) =
        memscope_rs::core::tracker::MemoryTracker::parse_binary_to_json(binary_path, json_path)
    {
        println!("❌ Binary to JSON conversion failed: {e}");
        conversion_start.elapsed()
    } else {
        let conversion_time = conversion_start.elapsed();
        println!(
            "✅ Binary -> JSON conversion completed in {:.2}ms",
            conversion_time.as_secs_f64() * 1000.0
        );
        conversion_time
    };

    println!("\n📊 Performance Summary:");
    println!(
        "  Binary export: {:.2}ms",
        binary_export_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Binary -> JSON: {:.2}ms",
        conversion_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Total workflow: {:.2}ms",
        (binary_export_time + conversion_time).as_secs_f64() * 1000.0
    );

    println!("\n🎯 Binary -> JSON Workflow Analysis:");
    println!("====================================");
    println!("Workflow: Data Collection -> Binary Export -> JSON Conversion");
    println!("This approach provides:");
    println!("✓ Fast binary storage for archival");
    println!("✓ On-demand JSON conversion for compatibility");
    println!("✓ Efficient two-step data processing pipeline");
    println!("\nGenerated files:");
    println!("  1. complex_lifecycle_binary.memscope - Binary format data");
    println!("  2. complex_lifecycle_from_binary.json - JSON converted from binary");
}
