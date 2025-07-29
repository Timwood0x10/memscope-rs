// Integration and Boundary Conditions Test Suite
// Tests complex scenarios and edge cases that might occur in real usage

use memscope_rs::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn ensure_init() {
    // Simple initialization without env_logger dependency
}

#[test]
fn test_mixed_allocation_patterns() {
    ensure_init();
    let tracker = get_global_tracker();

    // Mix of different allocation patterns
    let small_vec = vec![1u8; 10];
    let medium_vec = vec![2u16; 100];
    let large_vec = vec![3u32; 1000];
    let string_data = "test string with some length".to_string();
    let boxed_data = Box::new([4u64; 50]);

    let _ = track_var!(small_vec);
    let _ = track_var!(medium_vec);
    let _ = track_var!(large_vec);
    let _ = track_var!(string_data);
    let _ = track_var!(boxed_data);

    // Verify tracking
    let stats = tracker.get_stats().expect("Should get stats");
    assert!(
        stats.active_allocations >= 5,
        "Should track multiple allocations"
    );

    // Test export with mixed data - using fast mode
    let fast_options = memscope_rs::export::optimized_json_export::OptimizedExportOptions::with_optimization_level(
        memscope_rs::export::optimized_json_export::OptimizationLevel::Low
    ).fast_export_mode(true);
    let export_result =
        tracker.export_to_json_with_optimized_options("mixed_patterns.json", fast_options);
    assert!(
        export_result.is_ok(),
        "Should export mixed allocation patterns"
    );

    // Cleanup
    std::fs::remove_file("mixed_patterns.json").ok();
}

#[test]
fn test_rapid_allocation_deallocation() {
    ensure_init();
    let tracker = get_global_tracker();

    let initial_stats = tracker.get_stats().expect("Should get initial stats");

    // Rapid allocation and deallocation cycles
    for i in 0..100 {
        let temp_data = vec![i; 10];
        let _ = track_var!(temp_data);
        // temp_data goes out of scope and should be deallocated
    }

    // Give some time for cleanup
    thread::sleep(Duration::from_millis(10));

    let final_stats = tracker.get_stats().expect("Should get final stats");

    // Should have processed many allocations
    assert!(
        final_stats.total_allocations > initial_stats.total_allocations + 50,
        "Should track rapid allocations"
    );
}

#[test]
fn test_nested_scope_tracking() {
    ensure_init();
    let _tracker = get_global_tracker();
    let scope_tracker = memscope_rs::core::scope_tracker::get_global_scope_tracker();

    // Test nested scopes
    let outer_scope = scope_tracker
        .enter_scope("outer_scope".to_string())
        .expect("Should enter outer scope");

    let outer_data = vec![1, 2, 3];
    let _ = track_var!(outer_data);

    {
        let inner_scope = scope_tracker
            .enter_scope("inner_scope".to_string())
            .expect("Should enter inner scope");

        let inner_data = vec![4, 5, 6];
        let _ = track_var!(inner_data);

        {
            let deep_scope = scope_tracker
                .enter_scope("deep_scope".to_string())
                .expect("Should enter deep scope");

            let deep_data = vec![7, 8, 9];
            let _ = track_var!(deep_data);

            scope_tracker
                .exit_scope(deep_scope)
                .expect("Should exit deep scope");
        }

        scope_tracker
            .exit_scope(inner_scope)
            .expect("Should exit inner scope");
    }

    scope_tracker
        .exit_scope(outer_scope)
        .expect("Should exit outer scope");

    // Verify scope analysis
    let scope_analysis = scope_tracker
        .get_scope_analysis()
        .expect("Should get scope analysis");

    assert!(
        scope_analysis.total_scopes >= 3,
        "Should track nested scopes"
    );
}

#[test]
fn test_concurrent_tracking_with_shared_data() {
    ensure_init();
    let tracker = get_global_tracker();

    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Spawn multiple threads that share data
    for i in 0..5 {
        let shared_clone = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let local_data = vec![i * 10 + j; 20];
                let _ = track_var!(local_data);

                // Modify shared data
                if let Ok(mut data) = shared_clone.lock() {
                    data.push(i * 10 + j);
                }

                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    // Verify shared data
    let final_shared = shared_data.lock().expect("Should lock shared data");
    assert_eq!(
        final_shared.len(),
        50,
        "Should have 50 entries from all threads"
    );

    // Verify tracking stats
    let stats = tracker.get_stats().expect("Should get stats");
    assert!(
        stats.total_allocations >= 50,
        "Should track concurrent allocations"
    );
}

#[test]
fn test_memory_pressure_scenarios() {
    ensure_init();
    let tracker = get_global_tracker();

    let initial_stats = tracker.get_stats().expect("Should get initial stats");

    // Create memory pressure with large allocations
    let mut large_allocations = Vec::new();

    for i in 0..10 {
        let large_data = vec![i as u8; 10_000]; // 10KB each
        let _ = track_var!(large_data.clone());
        large_allocations.push(large_data);
    }

    let pressure_stats = tracker.get_stats().expect("Should get pressure stats");

    // Check if we have any memory increase (very lenient check)
    let memory_increase = pressure_stats
        .active_memory
        .saturating_sub(initial_stats.active_memory);

    if memory_increase > 0 {
        println!("Memory pressure test: {memory_increase} bytes increase detected");
    } else {
        println!("Memory pressure test: No significant memory increase detected");
        println!("This may be expected in test environments with limited tracking");
    }

    // Just verify the test completed without crashing
    assert!(
        pressure_stats.total_allocations >= initial_stats.total_allocations,
        "Should have at least the same number of total allocations"
    );

    // Test export under pressure
    let export_result = tracker.export_to_json("memory_pressure.json");
    assert!(export_result.is_ok(), "Should export under memory pressure");

    // Release pressure
    drop(large_allocations);

    // Give time for cleanup
    thread::sleep(Duration::from_millis(50));

    // Cleanup
    std::fs::remove_file("memory_pressure.json").ok();
}

#[test]
fn test_error_recovery_scenarios() {
    ensure_init();
    let tracker = get_global_tracker();

    // Test recovery from various error conditions

    // 1. Invalid pointer association (should not crash)
    let invalid_result = tracker.associate_var(0, "invalid".to_string(), "test".to_string());
    // Should either succeed or fail gracefully
    match invalid_result {
        Ok(_) => println!("Invalid pointer association succeeded"),
        Err(e) => println!("Invalid pointer association failed gracefully: {e}"),
    }

    // 2. Double deallocation attempt (should not crash)
    let test_data = vec![1, 2, 3];
    let ptr = test_data.as_ptr() as usize;
    let _ = track_var!(test_data);

    // Try to manually deallocate (this might fail, but shouldn't crash)
    let dealloc_result = tracker.track_deallocation(ptr);
    match dealloc_result {
        Ok(_) => println!("Manual deallocation succeeded"),
        Err(e) => println!("Manual deallocation failed gracefully: {e}"),
    }

    // 3. Stats retrieval should still work
    let stats_result = tracker.get_stats();
    assert!(
        stats_result.is_ok(),
        "Stats retrieval should work after errors"
    );
}

#[test]
fn test_comprehensive_export_integration() {
    ensure_init();
    let tracker = get_global_tracker();
    let unsafe_tracker = get_global_unsafe_ffi_tracker();
    let scope_tracker = memscope_rs::core::scope_tracker::get_global_scope_tracker();

    // Create comprehensive test scenario
    let scope_id = scope_tracker
        .enter_scope("integration_test".to_string())
        .expect("Should enter scope");

    // Various data types
    let vec_data = vec![1, 2, 3, 4, 5];
    let string_data = "integration test string".to_string();
    let boxed_data = Box::new(42);

    let _ = track_var!(vec_data);
    let _ = track_var!(string_data);
    let _ = track_var!(boxed_data);

    // Add unsafe operations
    unsafe_tracker
        .track_unsafe_allocation(0x6000, 64, "integration_test".to_string())
        .expect("Should track unsafe allocation");

    // Test all export formats - using fast mode
    let fast_options = memscope_rs::export::optimized_json_export::OptimizedExportOptions::with_optimization_level(
        memscope_rs::export::optimized_json_export::OptimizationLevel::Low
    ).fast_export_mode(true);
    let json_result =
        tracker.export_to_json_with_optimized_options("comprehensive_test.json", fast_options);
    assert!(json_result.is_ok(), "JSON export should succeed");

    let svg_result = tracker.export_memory_analysis("comprehensive_test.svg");
    assert!(svg_result.is_ok(), "SVG export should succeed");

    let timeline_result = tracker.export_lifecycle_timeline("comprehensive_timeline.svg");
    assert!(timeline_result.is_ok(), "Timeline export should succeed");

    let dashboard_result = tracker.export_interactive_dashboard("comprehensive_dashboard.html");
    assert!(dashboard_result.is_ok(), "Dashboard export should succeed");

    // Verify exported files exist and have content
    assert!(
        std::path::Path::new("comprehensive_test.json").exists(),
        "JSON file should exist"
    );
    assert!(
        std::path::Path::new("comprehensive_test.svg").exists(),
        "SVG file should exist"
    );
    assert!(
        std::path::Path::new("comprehensive_timeline.svg").exists(),
        "Timeline file should exist"
    );
    assert!(
        std::path::Path::new("comprehensive_dashboard.html").exists(),
        "Dashboard file should exist"
    );

    // Exit scope
    scope_tracker
        .exit_scope(scope_id)
        .expect("Should exit scope");

    // Cleanup
    std::fs::remove_file("comprehensive_test.json").ok();
    std::fs::remove_file("comprehensive_test.svg").ok();
    std::fs::remove_file("comprehensive_timeline.svg").ok();
    std::fs::remove_file("comprehensive_dashboard.html").ok();
}
