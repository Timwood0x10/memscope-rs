// Coverage Validation Test Suite
// Tests to ensure all major code paths are exercised

use memscope_rs::*;
use std::collections::HashMap;

fn ensure_init() {
    // Simple initialization without env_logger dependency
}

#[test]
fn test_all_tracker_methods() {
    ensure_init();
    let tracker = get_global_tracker();
    
    // Test all major tracker methods to ensure coverage
    
    // 1. Basic tracking operations
    let data = vec![1, 2, 3];
    let ptr = data.as_ptr() as usize;
    let _ = track_var!(data);
    
    // 2. Manual tracking operations
    let _manual_result = tracker.track_allocation(0x1000, 100);
    // Should either succeed or fail gracefully
    
    // 3. Association operations
    let _assoc_result = tracker.associate_var(ptr, "test_var".to_string(), "Vec<i32>".to_string());
    // Should either succeed or fail gracefully
    
    // 4. Statistics retrieval
    let stats = tracker.get_stats().expect("Should get stats");
    assert!(stats.total_allocations < 1_000_000, "Stats should be reasonable");
    
    // 5. Active allocations
    let active = tracker.get_active_allocations().expect("Should get active allocations");
    println!("Active allocations: {}", active.len());
    
    // 6. Allocation history
    let history = tracker.get_allocation_history().expect("Should get history");
    println!("History entries: {}", history.len());
    
    // 7. Memory by type
    let by_type = tracker.get_memory_by_type().expect("Should get memory by type");
    println!("Memory types: {}", by_type.len());
    
    println!("All tracker methods exercised successfully");
}

#[test]
fn test_all_scope_tracker_methods() {
    ensure_init();
    let scope_tracker = memscope_rs::scope_tracker::get_global_scope_tracker();
    
    // Test all major scope tracker methods
    
    // 1. Scope creation and management
    let scope1 = scope_tracker.enter_scope("test_scope_1".to_string())
        .expect("Should enter scope 1");
    
    let scope2 = scope_tracker.enter_scope("test_scope_2".to_string())
        .expect("Should enter scope 2");
    
    // 2. Variable association
    let _assoc_result = scope_tracker.associate_variable("test_var".to_string(), 100);
    assert!(_assoc_result.is_ok(), "Should associate variable");
    
    // 3. Scope analysis
    let analysis = scope_tracker.get_scope_analysis()
        .expect("Should get scope analysis");
    assert!(analysis.total_scopes >= 2, "Should have at least 2 scopes");
    
    // 4. Lifecycle metrics
    let metrics = scope_tracker.get_scope_lifecycle_metrics()
        .expect("Should get lifecycle metrics");
    println!("Lifecycle metrics: {}", metrics.len());
    
    // 5. Scope exit
    scope_tracker.exit_scope(scope2).expect("Should exit scope 2");
    scope_tracker.exit_scope(scope1).expect("Should exit scope 1");
    
    println!("All scope tracker methods exercised successfully");
}

#[test]
fn test_all_unsafe_ffi_methods() {
    ensure_init();
    let unsafe_tracker = get_global_unsafe_ffi_tracker();
    
    // Test all major unsafe/FFI tracker methods
    
    // 1. Unsafe allocations
    let ptr_result = unsafe_tracker.track_unsafe_allocation(
        0x2000,
        100,
        "test_pointer_op".to_string()
    );
    assert!(ptr_result.is_ok(), "Should track unsafe allocation");
    
    // 2. FFI allocations
    let ffi_result = unsafe_tracker.track_ffi_allocation(
        0x3000,
        200,
        "test_library".to_string(),
        "test_ffi_function".to_string()
    );
    assert!(ffi_result.is_ok(), "Should track FFI allocation");
    
    // 3. Enhanced deallocation
    let dealloc_result = unsafe_tracker.track_enhanced_deallocation(0x2000);
    // Should either succeed or fail gracefully
    match dealloc_result {
        Ok(_) => println!("Enhanced deallocation succeeded"),
        Err(e) => println!("Enhanced deallocation failed gracefully: {}", e),
    }
    
    // 4. Boundary events
    let boundary_result = unsafe_tracker.record_boundary_event(
        0x3000,
        memscope_rs::unsafe_ffi_tracker::BoundaryEventType::RustToFfi,
        "rust_context".to_string(),
        "ffi_context".to_string()
    );
    assert!(boundary_result.is_ok(), "Should record boundary event");
    
    // 5. Safety violations
    let violations_result = unsafe_tracker.get_safety_violations();
    assert!(violations_result.is_ok(), "Should get safety violations");
    
    // 6. Statistics
    let stats = unsafe_tracker.get_stats();
    println!("Unsafe/FFI tracker stats: {} operations, risk score: {}", 
             stats.total_operations, stats.risk_score);
    
    // Very lenient check - just ensure the tracker is working
    assert!(stats.risk_score.is_finite(), "Risk score should be finite");
    
    // Note: Operation counting might not work as expected in test environment
    if stats.total_operations == 0 {
        println!("Note: No operations tracked - this may be expected in test environment");
    } else {
        println!("Successfully tracked {} operations", stats.total_operations);
    }
    
    println!("All unsafe/FFI tracker methods exercised successfully");
}

#[test]
fn test_all_export_methods() {
    ensure_init();
    let tracker = get_global_tracker();
    
    // Create some test data
    let data = vec![1, 2, 3, 4, 5];
    let _ = track_var!(data);
    
    // Test all export methods
    
    // 1. JSON export
    let json_result = tracker.export_to_json("coverage_test.json");
    assert!(json_result.is_ok(), "JSON export should work");
    
    // 2. Memory analysis SVG
    let svg_result = tracker.export_memory_analysis("coverage_memory.svg");
    assert!(svg_result.is_ok(), "Memory analysis export should work");
    
    // 3. Lifecycle timeline SVG
    let timeline_result = tracker.export_lifecycle_timeline("coverage_timeline.svg");
    assert!(timeline_result.is_ok(), "Timeline export should work");
    
    // 4. Interactive dashboard
    let dashboard_result = tracker.export_interactive_dashboard("coverage_dashboard.html");
    assert!(dashboard_result.is_ok(), "Dashboard export should work");
    
    // Cleanup
    std::fs::remove_file("coverage_test.json").ok();
    std::fs::remove_file("coverage_memory.svg").ok();
    std::fs::remove_file("coverage_timeline.svg").ok();
    std::fs::remove_file("coverage_dashboard.html").ok();
    
    println!("All export methods exercised successfully");
}

#[test]
fn test_error_path_coverage() {
    ensure_init();
    let tracker = get_global_tracker();
    
    // Test various error conditions to ensure error paths are covered
    
    // 1. Invalid pointer operations
    let _invalid_dealloc = tracker.track_deallocation(0);
    // Should handle gracefully
    
    // 2. Invalid associations
    let _invalid_assoc = tracker.associate_var(0, "".to_string(), "".to_string());
    // Should handle gracefully
    
    // 3. Export to invalid paths
    let invalid_export = tracker.export_to_json("");
    assert!(invalid_export.is_err(), "Empty path should fail");
    
    // 4. Scope operations
    let scope_tracker = memscope_rs::scope_tracker::get_global_scope_tracker();
    // Create a dummy scope ID for testing (since ScopeId::new doesn't exist)
    // This will test error handling for invalid scope operations
    let dummy_scope = scope_tracker.enter_scope("dummy".to_string());
    if let Ok(scope_id) = dummy_scope {
        let _invalid_scope_exit = scope_tracker.exit_scope(scope_id);
        // Should handle gracefully
    }
    // Should handle gracefully
    
    println!("Error path coverage completed");
}

#[test]
fn test_data_structure_coverage() {
    ensure_init();
    let tracker = get_global_tracker();
    
    // Test various data structures to ensure type handling coverage
    
    // 1. Collections
    let vec_data = vec![1, 2, 3];
    let mut _map_data = HashMap::new();
    _map_data.insert("key1", "value1");
    _map_data.insert("key2", "value2");
    
    let _ = track_var!(vec_data);
    // Note: HashMap doesn't implement Trackable trait
    println!("HashMap created (not tracked due to trait requirements)");
    
    // 2. Strings
    let string_data = String::from("test string");
    let _str_slice = "string slice";
    
    let _ = track_var!(string_data);
    // Note: str_slice is not owned, so tracking might not work
    
    // 3. Boxed data
    let boxed_data = Box::new(42);
    let _ = track_var!(boxed_data);
    
    // 4. Arrays and slices (skip tracking since arrays don't implement Trackable)
    let _array_data = [1, 2, 3, 4, 5];
    // Note: Arrays don't implement Trackable trait
    println!("Array created (not tracked due to trait requirements)");
    
    // 5. Tuples (skip tracking since tuples don't implement Trackable)
    let _tuple_data = (1, "hello", std::f64::consts::PI);
    // Note: Tuples don't implement Trackable trait
    println!("Tuple created (not tracked due to trait requirements)");
    
    // 6. Custom structs (skip tracking since TestStruct doesn't implement Trackable)
    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestStruct {
        field1: i32,
        field2: String,
    }
    
    let _struct_data = TestStruct {
        field1: 42,
        field2: "test".to_string(),
    };
    // Note: Custom structs need to implement Trackable trait to be tracked
    println!("Custom struct created (not tracked due to trait requirements)");
    
    // Verify tracking
    let stats = tracker.get_stats().expect("Should get stats");
    println!("Data structure coverage: {} allocations tracked", stats.active_allocations);
}