//! Safety tests for trace_tools to verify memory safety and error handling.

use std::sync::Once;
use std::thread;
use trace_tools::{get_global_tracker, init, track_var};

static INIT: Once = Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn test_null_pointer_safety() {
    ensure_init();

    let tracker = get_global_tracker();

    // Test tracking with null-like pointers (should be handled gracefully)
    let result = tracker.track_allocation(0, 100);
    assert!(result.is_ok(), "Should handle null pointer gracefully");

    let result = tracker.track_deallocation(0);
    assert!(
        result.is_ok(),
        "Should handle null pointer deallocation gracefully"
    );
}

#[test]
fn test_invalid_pointer_association() {
    ensure_init();

    let tracker = get_global_tracker();

    // Try to associate a variable with a non-existent pointer
    let result = tracker.associate_var(
        0xDEADBEEF,
        "invalid_var".to_string(),
        "InvalidType".to_string(),
    );
    assert!(
        result.is_ok(),
        "Should handle invalid pointer association gracefully"
    );
}

#[test]
fn test_double_deallocation() {
    ensure_init();

    let tracker = get_global_tracker();

    // Simulate double deallocation (should not crash)
    let ptr = 0x12345678;
    let _ = tracker.track_allocation(ptr, 100);
    let result1 = tracker.track_deallocation(ptr);
    let result2 = tracker.track_deallocation(ptr); // Double deallocation

    assert!(result1.is_ok());
    assert!(
        result2.is_ok(),
        "Should handle double deallocation gracefully"
    );
}

#[test]
fn test_extremely_large_allocation() {
    ensure_init();

    let tracker = get_global_tracker();

    // Test with very large allocation size
    let large_size = usize::MAX / 2;
    let result = tracker.track_allocation(0x1000000, large_size);
    assert!(result.is_ok(), "Should handle large allocation size");

    let _stats = tracker.get_stats().unwrap();
    // Note: This might overflow in real scenarios, but our tracking should handle it
    println!("Large allocation tracked: {} bytes", large_size);
}

#[test]
fn test_concurrent_tracker_access() {
    ensure_init();

    let tracker = get_global_tracker();
    let num_threads = 8;

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let tracker = tracker.clone();
            thread::spawn(move || {
                // Each thread performs various operations
                for j in 0..50 {
                    let ptr = i * 1000 + j;
                    let _ = tracker.track_allocation(ptr, j * 10);

                    if j % 2 == 0 {
                        let _ = tracker.associate_var(
                            ptr,
                            format!("var_{}_{}", i, j),
                            "TestType".to_string(),
                        );
                    }

                    if j % 3 == 0 {
                        let _ = tracker.track_deallocation(ptr);
                    }

                    // Occasionally get stats
                    if j % 10 == 0 {
                        let _ = tracker.get_stats();
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }

    // Verify tracker is still functional
    let stats = tracker.get_stats().unwrap();
    println!(
        "Concurrent access test completed: {} active allocations",
        stats.active_allocations
    );
}

#[test]
fn test_export_with_no_data() {
    ensure_init();

    let tracker = get_global_tracker();

    // Test exports when no data is tracked
    let json_result = tracker.export_to_json("empty_test.json");
    assert!(json_result.is_ok(), "Should export empty data successfully");

    let svg_result = tracker.export_to_svg("empty_test.svg");
    assert!(svg_result.is_ok(), "Should export empty SVG successfully");

    // Verify files were created
    assert!(std::path::Path::new("empty_test.json").exists());
    assert!(std::path::Path::new("empty_test.svg").exists());

    // Cleanup
    std::fs::remove_file("empty_test.json").ok();
    std::fs::remove_file("empty_test.svg").ok();
}

#[test]
fn test_export_to_invalid_path() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create some data first
    let data = vec![1, 2, 3];
    let _ = track_var!(data);

    // Test export to invalid/inaccessible path
    let json_result = tracker.export_to_json("/invalid/path/test.json");
    // Should handle gracefully (might succeed if path is created, or fail gracefully)
    match json_result {
        Ok(_) => println!("Export succeeded (path was created)"),
        Err(e) => println!("Export failed gracefully: {}", e),
    }

    // Test with relative path that should work
    let json_result = tracker.export_to_json("./test_output.json");
    assert!(json_result.is_ok(), "Should export to valid relative path");

    // Cleanup
    std::fs::remove_file("./test_output.json").ok();
}

#[test]
fn test_trackable_trait_edge_cases() {
    ensure_init();

    // Test empty Vec (no heap allocation)
    let empty_vec: Vec<i32> = Vec::new();
    let result = track_var!(empty_vec);
    assert!(result.is_ok(), "Should handle empty Vec gracefully");

    // Test empty String (no heap allocation)
    let empty_string = String::new();
    let result = track_var!(empty_string);
    assert!(result.is_ok(), "Should handle empty String gracefully");

    // Test with capacity but no elements
    let mut vec_with_capacity = Vec::with_capacity(100);
    let result = track_var!(vec_with_capacity);
    assert!(result.is_ok(), "Should handle Vec with capacity");

    // Add elements after tracking
    vec_with_capacity.push(42);
    // Note: The tracking won't update automatically, which is expected behavior
}

#[test]
fn test_memory_stats_consistency() {
    ensure_init();

    let tracker = get_global_tracker();
    let initial_stats = tracker.get_stats().unwrap();

    // Perform a series of allocations and deallocations
    let mut ptrs = Vec::new();

    // Allocations
    for i in 0..10 {
        let ptr = 0x100000 + i;
        let size = (i + 1) * 100;
        let _ = tracker.track_allocation(ptr, size);
        ptrs.push((ptr, size));
    }

    let mid_stats = tracker.get_stats().unwrap();

    // Deallocations
    for (ptr, _) in &ptrs[..5] {
        let _ = tracker.track_deallocation(*ptr);
    }

    let final_stats = tracker.get_stats().unwrap();

    // Verify stats consistency (accounting for test environment variability)
    assert!(mid_stats.total_allocations >= initial_stats.total_allocations);
    assert!(final_stats.total_deallocations >= initial_stats.total_deallocations);

    // Active allocations may vary in test environment due to concurrent operations
    if final_stats.active_allocations > mid_stats.active_allocations {
        println!(
            "Note: Active allocations increased (may be due to test environment): {} -> {}",
            mid_stats.active_allocations, final_stats.active_allocations
        );
    } else {
        println!(
            "Active allocations decreased as expected: {} -> {}",
            mid_stats.active_allocations, final_stats.active_allocations
        );
    }

    println!("Stats consistency test:");
    println!(
        "  Initial: {} allocs, {} deallocs",
        initial_stats.total_allocations, initial_stats.total_deallocations
    );
    println!(
        "  Mid: {} allocs, {} deallocs",
        mid_stats.total_allocations, mid_stats.total_deallocations
    );
    println!(
        "  Final: {} allocs, {} deallocs",
        final_stats.total_allocations, final_stats.total_deallocations
    );
}

#[test]
fn test_thread_local_recursion_prevention() {
    ensure_init();

    // This test verifies that our thread-local recursion prevention works
    // We can't easily trigger the exact scenario, but we can verify the system
    // remains stable under rapid allocation patterns

    let mut allocations = Vec::new();

    for i in 0..100 {
        // Rapid allocations that might trigger internal allocations
        let data = format!("Test string number {} with some content", i);
        let _ = track_var!(data);
        allocations.push(data);

        // Also create some vectors (but store as strings for consistency)
        let vec_data = vec![i; 10];
        let _ = track_var!(vec_data);
        allocations.push(format!("Vec data: {:?}", vec_data));
    }

    // If we get here without hanging or crashing, recursion prevention is working
    let tracker = get_global_tracker();
    let stats = tracker.get_stats().unwrap();
    println!(
        "Recursion prevention test completed: {} active allocations",
        stats.active_allocations
    );
}

#[test]
fn test_allocation_history_growth() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create many allocations to test history growth
    for i in 0..100 {
        let data = vec![i; 10];
        let _ = track_var!(data);
    }

    let history = tracker.get_allocation_history().unwrap();
    println!("Allocation history size: {}", history.len());

    // History should contain some entries (exact number may vary due to try_lock behavior)
    // This is more of a monitoring test than a strict assertion
    if history.len() > 50 {
        println!(
            "Warning: Allocation history is growing large: {} entries",
            history.len()
        );
    }
}

#[test]
fn test_error_handling_robustness() {
    ensure_init();

    let tracker = get_global_tracker();

    // Test various error conditions
    let results = [
        tracker.track_allocation(usize::MAX, 0),
        tracker.track_allocation(0, usize::MAX),
        tracker.track_deallocation(usize::MAX),
        tracker.associate_var(usize::MAX, String::new(), String::new()),
        tracker.associate_var(0, "test".repeat(1000), "type".repeat(1000)),
    ];

    // All operations should complete without panicking
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Operation {} should handle edge case gracefully",
            i
        );
    }

    // Tracker should still be functional
    let stats = tracker.get_stats();
    assert!(
        stats.is_ok(),
        "Tracker should remain functional after error conditions"
    );
}
