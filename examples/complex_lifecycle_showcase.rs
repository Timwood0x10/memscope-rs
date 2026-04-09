//! Complex Lifecycle Showcase - New API
//!
//! This example demonstrates the new unified API with various built-in types,
//! custom types, and complex memory patterns.

use memscope_rs::{global_tracker, init_global_tracking, track, MemScopeResult};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> MemScopeResult<()> {
    println!("Complex Lifecycle Showcase - New API");
    println!("==========================================\n");

    let start_time = Instant::now();
    init_global_tracking()?;
    let tracker = global_tracker()?;

    println!("Phase 1: Built-in Types");
    println!("=========================================");
    demonstrate_builtin_types(&tracker);

    println!("\nPhase 2: Smart Pointers");
    println!("=========================================");
    demonstrate_smart_pointers(&tracker);

    println!("\nPhase 3: Complex Patterns");
    println!("=========================================");
    demonstrate_complex_patterns(&tracker);

    println!("\nPhase 4: Web Server Simulation");
    println!("=========================================");
    simulate_web_server_scenario(&tracker);

    println!("\nPhase 5: Data Processing Pipeline");
    println!("=========================================");
    simulate_data_processing_pipeline(&tracker);

    let duration = start_time.elapsed();

    let stats = tracker.get_stats();
    println!("\nMemory Analysis Results:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!(
        "  Peak memory: {} bytes ({:.2} MB)",
        stats.peak_memory_bytes,
        stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );

    println!("\nExporting memory snapshot...");
    let output_path = "MemoryAnalysis/complex_lifecycle_new_api";
    tracker.export_json(output_path)?;
    println!("  memory_snapshots.json");
    println!("  memory_passports.json");
    println!("  leak_detection.json");
    println!("  unsafe_ffi_analysis.json");
    println!("  system_resources.json");
    println!("  async_analysis.json");

    // Export HTML dashboard
    println!("\nExporting HTML dashboard...");
    tracker.export_html(output_path)?;
    println!("  dashboard.html");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    Ok(())
}

fn demonstrate_builtin_types(tracker: &memscope_rs::GlobalTracker) {
    let mut small_vec = Vec::with_capacity(5);
    for i in 0..10 {
        small_vec.push(i);
    }
    track!(tracker, small_vec);
    println!("✓ Small Vec: 10 elements");

    let mut large_vec = Vec::with_capacity(1000);
    for i in 0..2000 {
        large_vec.push(format!("Item {i}"));
    }
    track!(tracker, large_vec);
    println!("✓ Large Vec: 2000 elements");

    let mut growing_string = String::new();
    for i in 0..100 {
        growing_string.push_str(&format!("Data chunk {i} | "));
    }
    track!(tracker, growing_string);
    println!("✓ Growing String: {} bytes", growing_string.len());

    let mut hash_map = HashMap::new();
    for i in 0..500 {
        hash_map.insert(format!("key_{i}"), i * 2);
    }
    track!(tracker, hash_map);
    println!("✓ HashMap: 500 entries");

    let mut btree_map = BTreeMap::new();
    for i in 0..300 {
        btree_map.insert(i, format!("value_{i}"));
    }
    track!(tracker, btree_map);
    println!("✓ BTreeMap: 300 sorted entries");
}

fn demonstrate_smart_pointers(tracker: &memscope_rs::GlobalTracker) {
    let boxed_large = Box::new(vec![0u8; 1024]);
    track!(tracker, boxed_large);
    println!("✓ Box<Vec<u8>>: 1KB heap allocation");

    let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
    track!(tracker, shared_data);
    let shared_clone1 = Rc::clone(&shared_data);
    track!(tracker, shared_clone1);
    println!(
        "✓ Rc<Vec<i32>>: {} references",
        Rc::strong_count(&shared_data)
    );

    let thread_safe = Arc::new(String::from("Thread-safe shared string"));
    track!(tracker, thread_safe);
    let arc_clone = Arc::clone(&thread_safe);
    track!(tracker, arc_clone);
    println!(
        "✓ Arc<String>: {} references",
        Arc::strong_count(&thread_safe)
    );

    let mutable_data = Rc::new(RefCell::new(vec![10, 20, 30]));
    track!(tracker, mutable_data);
    println!("✓ Rc<RefCell<Vec<i32>>>: interior mutability");
}

fn demonstrate_complex_patterns(tracker: &memscope_rs::GlobalTracker) {
    let mut nested = HashMap::new();
    for i in 0..5 {
        let mut inner = BTreeMap::new();
        for j in 0..10 {
            inner.insert(j, vec![format!("item_{}_{}", i, j); 3]);
        }
        nested.insert(format!("group_{i}"), inner);
    }
    track!(tracker, nested);
    println!("✓ Nested HashMap<BTreeMap>: 3-level nesting");

    let node_a = Rc::new(RefCell::new(vec!["Node A".to_string()]));
    track!(tracker, node_a);
    println!("✓ Circular reference pattern: Rc<RefCell>");

    let mut computation = Vec::new();
    for i in 0..1000 {
        computation.push(format!("Result {}: {}", i, i * i));
    }
    track!(tracker, computation);
    println!("✓ Large computation: 1000 formatted strings");
}

fn simulate_web_server_scenario(tracker: &memscope_rs::GlobalTracker) {
    let mut routes = HashMap::new();
    routes.insert(
        "/api/users".to_string(),
        "UserController::index".to_string(),
    );
    routes.insert(
        "/api/posts".to_string(),
        "PostController::index".to_string(),
    );
    track!(tracker, routes);
    println!("✓ Routes: 2 API endpoints");

    let mut sessions = HashMap::new();
    for i in 0..50 {
        sessions.insert(
            format!("sess_{i:08x}"),
            HashMap::from([
                ("user_id".to_string(), format!("{}", 1000 + i)),
                (
                    "csrf_token".to_string(),
                    format!("token_{:016x}", i * 12345),
                ),
            ]),
        );
    }
    track!(tracker, sessions);
    println!("✓ Sessions: 50 active sessions");

    let mut request_log = VecDeque::new();
    for i in 0..200 {
        request_log.push_back(format!("[{}] GET /api/endpoint_{}", 1640995200 + i, i % 10));
        if request_log.len() > 100 {
            request_log.pop_front();
        }
    }
    track!(tracker, request_log);
    println!("✓ Request log: {} entries", request_log.len());
}

fn simulate_data_processing_pipeline(tracker: &memscope_rs::GlobalTracker) {
    let mut input_queue = VecDeque::new();
    for i in 0..500 {
        input_queue.push_back(format!("data_record_{i:06}"));
    }
    track!(tracker, input_queue);
    println!("✓ Input queue: 500 records");

    let mut stage1 = Vec::new();
    for i in 0..500 {
        stage1.push(format!("validated_{}", i));
    }
    track!(tracker, stage1);
    println!("✓ Stage 1 results: {} processed", stage1.len());

    let mut final_results = HashMap::new();
    for (i, record) in stage1.iter().enumerate() {
        let key = format!("index_{}", i / 10);
        final_results
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record.clone());
    }
    track!(tracker, final_results);
    println!("✓ Final results: {} groups", final_results.len());

    let mut errors = Vec::new();
    for i in 0..25 {
        errors.push(format!("Error {}: Processing failed", i));
    }
    track!(tracker, errors);
    println!("✓ Error tracker: {} errors", errors.len());
}
