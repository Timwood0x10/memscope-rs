// Complex lifecycle showcase demonstrating enhanced tracking capabilities
// This example shows various built-in types, custom types, and complex memory patterns

use memscope_rs::{get_global_tracker, init, track_var};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

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

#[derive(Debug)]
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
    println!("🚀 Complex Multi-Scope Lifecycle Showcase");
    println!("==========================================");
    println!("Demonstrating enhanced lifecycle tracking with:");
    println!("• Multiple nested scopes and function calls");
    println!("• Built-in types (Vec, String, HashMap, etc.)");
    println!("• Smart pointers (Box, Rc, Arc, RefCell)");
    println!("• Custom structs and complex data patterns");
    println!("• Memory growth, ownership transfers, and borrowing");
    println!();

    // Global scope variables
    let global_app_config = String::from("app_config_v2.0");
    track_var!(global_app_config).unwrap();
    let global_session_store = Box::new(HashMap::<String, String>::new());
    track_var!(global_session_store).unwrap();

    // Keep all variables alive until the end by collecting them
    let mut _keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

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

    // Global cleanup
    let global_cleanup_log = Vec::<String>::new();
    track_var!(global_cleanup_log).unwrap();

    // Generate comprehensive analysis with all variables still alive
    generate_final_analysis();

    println!(
        "📌 Keeping {} variables alive for complete lifecycle analysis",
        _keep_alive.len()
    );
}

fn demonstrate_builtin_types() -> Vec<Box<dyn std::any::Any>> {
    println!("📦 Phase 1: Built-in Types Demonstration");
    println!("=========================================");

    let mut keep_alive = Vec::new();

    // Vectors with different growth patterns
    let mut small_vec = Vec::with_capacity(5);
    track_var!(small_vec).unwrap();
    for i in 0..10 {
        small_vec.push(i);
    }
    println!(
        "✓ Small Vec: grew from capacity 5 to {} elements",
        small_vec.len()
    );
    keep_alive.push(Box::new(small_vec) as Box<dyn std::any::Any>);

    let mut large_vec = Vec::with_capacity(1000);
    track_var!(large_vec).unwrap();
    for i in 0..2000 {
        large_vec.push(format!("Item {}", i));
    }
    println!(
        "✓ Large Vec: grew from capacity 1000 to {} elements",
        large_vec.len()
    );
    keep_alive.push(Box::new(large_vec) as Box<dyn std::any::Any>);

    // Strings with different patterns
    let mut growing_string = String::new();
    track_var!(growing_string).unwrap();
    for i in 0..100 {
        growing_string.push_str(&format!("Data chunk {} | ", i));
    }
    println!("✓ Growing String: {} bytes", growing_string.len());
    keep_alive.push(Box::new(growing_string) as Box<dyn std::any::Any>);

    let static_string = String::from("Static content that doesn't grow");
    track_var!(static_string).unwrap();
    println!("✓ Static String: {} bytes", static_string.len());
    keep_alive.push(Box::new(static_string) as Box<dyn std::any::Any>);

    // Collections (using Box to make them trackable)
    let mut hash_map = HashMap::new();
    for i in 0..50 {
        hash_map.insert(format!("key_{}", i), i * 2);
    }
    let boxed_hash_map = Box::new(hash_map);
    track_var!(boxed_hash_map).unwrap();
    println!("✓ Box<HashMap>: {} entries", boxed_hash_map.len());
    keep_alive.push(boxed_hash_map as Box<dyn std::any::Any>);

    let mut hash_set = HashSet::new();
    for i in 0..30 {
        hash_set.insert(format!("unique_{}", i));
    }
    let boxed_hash_set = Box::new(hash_set);
    track_var!(boxed_hash_set).unwrap();
    println!("✓ Box<HashSet>: {} unique items", boxed_hash_set.len());
    keep_alive.push(boxed_hash_set as Box<dyn std::any::Any>);

    let mut btree_map = BTreeMap::new();
    for i in 0..25 {
        btree_map.insert(i, format!("value_{}", i));
    }
    let boxed_btree_map = Box::new(btree_map);
    track_var!(boxed_btree_map).unwrap();
    println!("✓ Box<BTreeMap>: {} sorted entries", boxed_btree_map.len());
    keep_alive.push(boxed_btree_map as Box<dyn std::any::Any>);

    let mut vec_deque = VecDeque::new();
    for i in 0..20 {
        if i % 2 == 0 {
            vec_deque.push_back(i);
        } else {
            vec_deque.push_front(i);
        }
    }
    let boxed_vec_deque = Box::new(vec_deque);
    track_var!(boxed_vec_deque).unwrap();
    println!(
        "✓ Box<VecDeque>: {} items (front/back insertion)",
        boxed_vec_deque.len()
    );
    keep_alive.push(boxed_vec_deque as Box<dyn std::any::Any>);

    println!();
    keep_alive
}

fn demonstrate_smart_pointers() -> Vec<Box<dyn std::any::Any>> {
    println!("🔗 Phase 2: Smart Pointers & Reference Counting");
    println!("===============================================");
    let mut keep_alive = Vec::new();

    // Box pointers
    let boxed_large_data = Box::new(vec![0u8; 1024]);
    track_var!(boxed_large_data).unwrap();
    println!("✓ Box<Vec<u8>>: 1KB heap allocation");

    let boxed_string = Box::new(String::from("Boxed string data"));
    track_var!(boxed_string).unwrap();
    println!("✓ Box<String>: heap-allocated string");

    // Reference counting with Rc
    let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
    track_var!(shared_data).unwrap();
    let shared_clone1 = Rc::clone(&shared_data);
    track_var!(shared_clone1).unwrap();
    let shared_clone2 = Rc::clone(&shared_data);
    track_var!(shared_clone2).unwrap();
    println!(
        "✓ Rc<Vec<i32>>: {} references to shared data",
        Rc::strong_count(&shared_data)
    );

    // Thread-safe reference counting with Arc
    let thread_safe_data = Arc::new(String::from("Thread-safe shared string"));
    track_var!(thread_safe_data).unwrap();
    let arc_clone = Arc::clone(&thread_safe_data);
    track_var!(arc_clone).unwrap();
    println!(
        "✓ Arc<String>: {} references to thread-safe data",
        Arc::strong_count(&thread_safe_data)
    );

    // Interior mutability with RefCell
    let mutable_data = Rc::new(RefCell::new(vec![10, 20, 30]));
    track_var!(mutable_data).unwrap();
    {
        let mut borrowed = mutable_data.borrow_mut();
        borrowed.push(40);
        borrowed.push(50);
    }
    println!("✓ Rc<RefCell<Vec<i32>>>: interior mutability pattern");

    // Keep smart pointers alive
    keep_alive.push(Box::new(boxed_large_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(boxed_string) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(shared_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(thread_safe_data) as Box<dyn std::any::Any>);
    keep_alive.push(Box::new(mutable_data) as Box<dyn std::any::Any>);

    println!();
    keep_alive
}

fn demonstrate_custom_structures() -> Vec<Box<dyn std::any::Any>> {
    let keep_alive = Vec::new();
    println!("🏗️  Phase 3: Custom Data Structures");
    println!("===================================");

    // User struct with nested collections (using Box to make trackable)
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
    track_var!(boxed_user).unwrap();
    println!("✓ Box<User>: complex nested data with HashMap");

    // Database connection simulation
    let mut db_conn = DatabaseConnection {
        host: String::from("localhost"),
        port: 5432,
        connection_pool: Vec::new(),
        active_queries: VecDeque::new(),
    };

    for i in 0..10 {
        db_conn.connection_pool.push(format!("conn_{}", i));
        db_conn
            .active_queries
            .push_back(format!("SELECT * FROM table_{}", i));
    }
    let boxed_db_conn = Box::new(db_conn);
    track_var!(boxed_db_conn).unwrap();
    println!(
        "✓ Box<DatabaseConnection>: {} connections, {} active queries",
        boxed_db_conn.connection_pool.len(),
        boxed_db_conn.active_queries.len()
    );

    // Cache entries with generic types
    let string_cache = CacheEntry {
        key: String::from("user_session_12345"),
        value: String::from("session_data_payload"),
        timestamp: 1640995200,
        access_count: 0,
    };
    let boxed_string_cache = Box::new(string_cache);
    track_var!(boxed_string_cache).unwrap();

    let vec_cache = CacheEntry {
        key: String::from("computed_results"),
        value: vec![1.0, 2.5, 3.7, 4.2, 5.9],
        timestamp: 1640995300,
        access_count: 0,
    };
    let boxed_vec_cache = Box::new(vec_cache);
    track_var!(boxed_vec_cache).unwrap();
    println!("✓ Box<CacheEntry<T>>: generic cache structures");

    // Graph node with complex relationships
    let mut graph_nodes = Vec::new();
    for i in 0..5 {
        let mut node = GraphNode {
            id: i,
            data: format!("Node {} data", i),
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
    track_var!(graph_nodes).unwrap();
    println!("✓ Vec<GraphNode>: 5 interconnected nodes with metadata");

    println!();
    keep_alive
}

fn demonstrate_complex_patterns() -> Vec<Box<dyn std::any::Any>> {
    let keep_alive = Vec::new();
    println!("🌀 Phase 4: Complex Memory Patterns");
    println!("===================================");

    // Nested collections (using Box to make trackable)
    let mut nested_structure = HashMap::new();
    for i in 0..5 {
        let mut inner_map = BTreeMap::new();
        for j in 0..10 {
            inner_map.insert(j, vec![format!("item_{}_{}", i, j); 3]);
        }
        nested_structure.insert(format!("group_{}", i), inner_map);
    }
    let boxed_nested = Box::new(nested_structure);
    track_var!(boxed_nested).unwrap();
    println!("✓ Box<HashMap<String, BTreeMap<i32, Vec<String>>>>: 3-level nesting");

    // Circular reference simulation (using Rc)
    let node_a = Rc::new(RefCell::new(vec!["Node A data".to_string()]));
    let node_b = Rc::new(RefCell::new(vec!["Node B data".to_string()]));
    track_var!(node_a).unwrap();
    track_var!(node_b).unwrap();
    println!("✓ Circular reference pattern: Rc<RefCell<Vec<String>>>");

    // Memory-intensive computation result
    let mut computation_result = Vec::new();
    for i in 0..1000 {
        computation_result.push(format!("Result {}: {}", i, i * i));
    }
    track_var!(computation_result).unwrap();
    println!("✓ Large computation result: 1000 formatted strings");

    // Stream processing buffer (using Box to make trackable)
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
                    .push(format!("Warning at byte {}", i));
            }
        }
    }
    let boxed_stream_processor = Box::new(stream_processor);
    track_var!(boxed_stream_processor).unwrap();
    println!(
        "✓ Box<StreamProcessor>: buffer grew from 2KB to {} bytes, {} errors logged",
        boxed_stream_processor.buffer.len(),
        boxed_stream_processor.error_log.len()
    );

    println!();
    keep_alive
}

fn simulate_web_server_scenario() -> Vec<Box<dyn std::any::Any>> {
    let keep_alive = Vec::new();
    println!("🌐 Phase 5a: Web Server Simulation");
    println!("==================================");

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
    track_var!(boxed_routes).unwrap();

    // Session storage
    let mut sessions = HashMap::new();
    for i in 0..50 {
        let session_id = format!("sess_{:08x}", i);
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
    track_var!(boxed_sessions).unwrap();

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
    track_var!(boxed_request_log).unwrap();

    println!(
        "✓ Web server: {} routes, {} sessions, {} log entries",
        boxed_routes.len(),
        boxed_sessions.len(),
        boxed_request_log.len()
    );
    println!();
    keep_alive
}

fn simulate_data_processing_pipeline() -> Vec<Box<dyn std::any::Any>> {
    let keep_alive = Vec::new();
    println!("⚙️  Phase 5b: Data Processing Pipeline");
    println!("=====================================");

    // Input data queue
    let mut input_queue = VecDeque::new();
    for i in 0..500 {
        input_queue.push_back(format!("data_record_{:06}", i));
    }
    let boxed_input_queue = Box::new(input_queue);
    track_var!(boxed_input_queue).unwrap();

    // Processing stages
    let mut stage1_results = Vec::new();
    let mut stage2_results = Vec::new();
    let mut final_results = HashMap::new();

    // Stage 1: Parse and validate (simulate processing)
    for i in 0..500 {
        let processed = format!("validated_data_record_{:06}", i);
        stage1_results.push(processed);
    }
    track_var!(stage1_results).unwrap();

    // Stage 2: Transform and enrich
    for record in &stage1_results {
        let enriched = format!("enriched_{}_with_metadata", record);
        stage2_results.push(enriched);
    }
    track_var!(stage2_results).unwrap();

    // Final stage: Aggregate and index
    for (i, record) in stage2_results.iter().enumerate() {
        let key = format!("index_{}", i / 10); // Group by 10s
        final_results
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record.clone());
    }
    let boxed_final_results = Box::new(final_results);
    track_var!(boxed_final_results).unwrap();

    // Error tracking
    let mut error_tracker = Vec::new();
    for i in 0..25 {
        error_tracker.push(format!(
            "Error {}: Processing failed for record {}",
            i,
            i * 20
        ));
    }
    track_var!(error_tracker).unwrap();

    println!(
        "✓ Data pipeline: {} → {} → {} groups, {} errors tracked",
        500,
        stage1_results.len(),
        boxed_final_results.len(),
        error_tracker.len()
    );
    println!();
    keep_alive
}

fn generate_final_analysis() {
    println!("📊 Final Analysis & Export");
    println!("=========================");

    let tracker = get_global_tracker();

    // Get comprehensive statistics
    if let Ok(stats) = tracker.get_stats() {
        println!("Memory Statistics:");
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

        println!("\nLifetime Distribution:");
        println!("  • Instant (< 1ms): {}", lifecycle.instant_allocations);
        println!(
            "  • Short-term (1-100ms): {}",
            lifecycle.short_term_allocations
        );
        println!(
            "  • Medium-term (100ms-1s): {}",
            lifecycle.medium_term_allocations
        );
        println!("  • Long-term (> 1s): {}", lifecycle.long_term_allocations);
        println!("  • Suspected leaks: {}", lifecycle.suspected_leaks);

        let risk = &lifecycle.risk_distribution;
        println!("\nRisk Assessment:");
        println!("  • Low risk: {}", risk.low_risk);
        println!("  • Potential growth: {}", risk.potential_growth_risk);
        println!("  • High memory: {}", risk.high_memory_risk);
        println!("  • Leak risk: {}", risk.leak_risk);

        if !lifecycle.scope_metrics.is_empty() {
            println!("\nScope Analysis:");
            for scope in &lifecycle.scope_metrics {
                println!(
                    "  • {}: {} vars, {:.1}ms avg, {:.1}% efficiency",
                    scope.scope_name,
                    scope.variable_count,
                    scope.avg_lifetime_ms,
                    scope.efficiency_score * 100.0
                );
            }
        }

        if !lifecycle.type_lifecycle_patterns.is_empty() {
            println!("\nType Patterns:");
            for pattern in &lifecycle.type_lifecycle_patterns {
                println!(
                    "  • {}: {:.1}x growth, {:?} ownership, {:?} risk",
                    pattern.type_name,
                    pattern.memory_growth_factor,
                    pattern.ownership_pattern,
                    pattern.risk_level
                );
            }
        }
    }

    // Export enhanced visualizations and data
    println!("\n🎨 Exporting Visualizations and Data:");

    if let Err(e) = tracker.export_memory_analysis("./complex_memory_analysis.svg") {
        println!("❌ Memory analysis export failed: {}", e);
    } else {
        println!("✅ Memory analysis exported to: ./complex_memory_analysis.svg");
    }

    if let Err(e) = tracker.export_lifecycle_timeline("./complex_lifecycle_timeline.svg") {
        println!("❌ Lifecycle timeline export failed: {}", e);
    } else {
        println!("✅ Enhanced lifecycle timeline exported to: ./complex_lifecycle_timeline.svg");
    }

    if let Err(e) = tracker.export_to_json("./complex_lifecycle_snapshot.json") {
        println!("❌ JSON snapshot export failed: {}", e);
    } else {
        println!("✅ Memory snapshot exported to: ./complex_lifecycle_snapshot.json");
    }


    println!("\n🎯 Showcase Complete!");
    println!("=====================");
    println!("This demonstration showcased:");
    println!("✓ Built-in types: Vec, String, HashMap, HashSet, BTreeMap, VecDeque");
    println!("✓ Smart pointers: Box, Rc, Arc, RefCell");
    println!("✓ Custom structs: User, DatabaseConnection, CacheEntry, GraphNode");
    println!("✓ Complex patterns: Nested collections, circular references, large datasets");
    println!("✓ Real scenarios: Web server simulation, data processing pipeline");
    println!("✓ Enhanced metrics: Growth tracking, ownership analysis, risk assessment");
    println!("✓ Four output formats: Memory analysis SVG, lifecycle timeline SVG, treemap SVG, and JSON snapshot");
    println!("\nGenerated files:");
    println!("  1. complex_memory_analysis.svg - Visual memory usage analysis");
    println!("  2. complex_lifecycle_timeline.svg - Enhanced lifecycle timeline");
    println!("  3. complex_lifecycle_snapshot.json - Hierarchical memory data organized by scopes");
    println!("\nCheck these files for comprehensive memory analysis!");
}
