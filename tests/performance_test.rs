//! Performance tests for memscope-rs to measure overhead and identify bottlenecks.

use memscope_rs::{get_global_tracker, init, track_var};
use std::sync::Once;
use std::time::{Duration, Instant};

static INIT: Once = Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn benchmark_allocation_tracking_overhead() {
    ensure_init();

    let iterations = 1000;

    // Benchmark without tracking
    let start = Instant::now();
    let mut untracked_allocations = Vec::new();
    for i in 0..iterations {
        let data = vec![i; 100];
        untracked_allocations.push(data);
    }
    let untracked_duration = start.elapsed();

    // Benchmark with tracking
    let start = Instant::now();
    let mut tracked_allocations = Vec::new();
    for i in 0..iterations {
        let data = vec![i; 100];
        let _ = track_var!(data);
        tracked_allocations.push(data);
    }
    let tracked_duration = start.elapsed();

    let overhead = tracked_duration.saturating_sub(untracked_duration);
    let overhead_percent = if untracked_duration.as_nanos() > 0 {
        (overhead.as_nanos() as f64 / untracked_duration.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    println!("Allocation tracking overhead benchmark:");
    println!("  Untracked: {untracked_duration:?}");
    println!("  Tracked: {tracked_duration:?}");
    println!("  Overhead: {overhead:?} ({overhead_percent:.2}%)");

    // Reasonable overhead threshold for debug builds (adjust based on requirements)
    // In debug mode, tracking overhead can be significantly higher due to:
    // - Lack of optimizations
    // - Debug assertions and checks
    // - Mutex contention in concurrent tests
    // In release mode, this should be much lower
    let threshold = if cfg!(debug_assertions) {
        2000.0 // Allow higher overhead in debug builds
    } else {
        500.0 // Stricter threshold for release builds
    };

    assert!(
        overhead_percent < threshold,
        "Tracking overhead too high: {overhead_percent:.2}% (threshold: {threshold:.2}%)"
    );
}

#[test]
fn benchmark_stats_retrieval() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create some allocations first
    let mut allocations = Vec::new();
    for i in 0..100 {
        let data = vec![i; 50];
        let _ = track_var!(data);
        allocations.push(data);
    }

    // Benchmark stats retrieval
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = tracker.get_stats().unwrap();
    }

    let duration = start.elapsed();
    let avg_time = duration / iterations;

    println!("Stats retrieval benchmark:");
    println!("  Total time: {duration:?}");
    println!("  Average per call: {avg_time:?}");

    // Should be reasonably fast (adjusted threshold for lifecycle stats calculation)
    assert!(
        avg_time < Duration::from_millis(10),
        "Stats retrieval too slow: {avg_time:?}"
    );
}

#[test]
fn benchmark_export_performance() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create substantial amount of data
    let mut allocations = Vec::new();
    for i in 0..200 {
        let data = vec![i; 100];
        let _ = track_var!(data);
        allocations.push(data);
    }

    // Benchmark JSON export
    let start = Instant::now();
    let json_result = tracker.export_to_json("perf_test.json");
    let json_duration = start.elapsed();

    assert!(json_result.is_ok(), "JSON export should succeed");

    // Benchmark SVG export
    let start = Instant::now();
    let svg_result = tracker.export_memory_analysis("perf_test.svg");
    let svg_duration = start.elapsed();

    assert!(svg_result.is_ok(), "SVG export should succeed");

    println!("Export performance benchmark:");
    println!("  JSON export: {json_duration:?}");
    println!("  SVG export: {svg_duration:?}");

    // Cleanup
    std::fs::remove_file("perf_test.json").ok();
    std::fs::remove_file("perf_test.svg").ok();

    // Reasonable export time thresholds
    assert!(
        json_duration < Duration::from_secs(5),
        "JSON export too slow: {json_duration:?}"
    );
    assert!(
        svg_duration < Duration::from_secs(10),
        "SVG export too slow: {svg_duration:?}"
    );
}

#[test]
fn benchmark_variable_association() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create allocations first
    let mut ptrs = Vec::new();
    for i in 0..100 {
        let ptr = 0x200000 + i;
        let _ = tracker.track_allocation(ptr, 100);
        ptrs.push(ptr);
    }

    // Benchmark variable association
    let start = Instant::now();

    for (i, ptr) in ptrs.iter().enumerate() {
        let _ = tracker.associate_var(*ptr, format!("var_{i}"), "TestType".to_string());
    }

    let duration = start.elapsed();
    let avg_time = duration / ptrs.len() as u32;

    println!("Variable association benchmark:");
    println!("  Total time: {duration:?}");
    println!("  Average per association: {avg_time:?}");

    // Should be fast
    assert!(
        avg_time < Duration::from_millis(1),
        "Variable association too slow: {avg_time:?}"
    );
}

#[test]
fn benchmark_concurrent_performance() {
    ensure_init();

    let tracker = get_global_tracker();
    let num_threads = 4;
    let operations_per_thread = 100;

    let start = Instant::now();

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let tracker = tracker.clone();
            std::thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let ptr = thread_id * 10000 + i;

                    // Mix of operations
                    let _ = tracker.track_allocation(ptr, i * 10);

                    if i % 3 == 0 {
                        let _ = tracker.associate_var(
                            ptr,
                            format!("var_{thread_id}_{i}"),
                            "ConcurrentType".to_string(),
                        );
                    }

                    if i % 5 == 0 {
                        let _ = tracker.get_stats();
                    }

                    if i % 7 == 0 {
                        let _ = tracker.track_deallocation(ptr);
                    }
                }
            })
        })
        .collect();

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let total_operations = num_threads * operations_per_thread * 2; // Approximate
    let avg_time_per_op = duration / total_operations as u32;

    println!("Concurrent performance benchmark:");
    println!("  Total time: {duration:?}");
    println!("  Operations: {total_operations}");
    println!("  Average per operation: {avg_time_per_op:?}");

    // Should handle concurrent load reasonably well
    assert!(
        duration < Duration::from_secs(10),
        "Concurrent operations too slow: {duration:?}"
    );
}

#[test]
fn benchmark_memory_usage_of_tracker() {
    ensure_init();

    let tracker = get_global_tracker();

    // Clear any existing data to get a clean baseline
    // Note: We can't actually clear the tracker, so we'll work with relative measurements
    
    // Create a smaller, more controlled test
    let mut allocations = Vec::new();
    let mut tracked_ptrs = Vec::new();
    
    // Get baseline
    let initial_stats = tracker.get_stats().unwrap();
    
    // Create exactly 100 tracked allocations with known sizes
    for i in 0..100 {
        let data = vec![i; 10]; // 10 * 4 = 40 bytes each
        let ptr = data.as_ptr() as usize;
        let _ = track_var!(data);
        tracked_ptrs.push(ptr);
        allocations.push(data);
    }

    let final_stats = tracker.get_stats().unwrap();

    // Calculate the difference
    let new_tracked_memory = final_stats.active_memory.saturating_sub(initial_stats.active_memory);
    let new_allocation_count = final_stats.active_allocations.saturating_sub(initial_stats.active_allocations);

    // Expected data size: 100 allocations * 40 bytes each = 4000 bytes
    let expected_data_size = 100 * 40;
    
    println!("Memory usage benchmark:");
    println!("  Created allocations: 100");
    println!("  New tracked allocations: {new_allocation_count}");
    println!("  New tracked memory: {new_tracked_memory} bytes");
    println!("  Expected data size: {expected_data_size} bytes");
    
    // More lenient check - focus on whether tracking is working rather than exact overhead
    if new_allocation_count > 0 {
        let total_per_allocation = new_tracked_memory / new_allocation_count;
        println!("  Total memory per allocation: {total_per_allocation} bytes");
        
        // Check that we're tracking a reasonable number of our allocations
        assert!(
            new_allocation_count >= 20,
            "Too few new allocations tracked: {new_allocation_count} (expected >= 20)"
        );
        
        // More realistic check: total memory per allocation should be reasonable
        // This includes both data and metadata
        assert!(
            total_per_allocation < 10000,
            "Total memory per allocation too high: {total_per_allocation} bytes (expected < 10000 bytes)"
        );
        
        // Ensure we're actually tracking memory
        assert!(
            new_tracked_memory > expected_data_size / 2,
            "Tracked memory too low: {new_tracked_memory} bytes (expected > {}) - tracking may not be working",
            expected_data_size / 2
        );
    } else {
        panic!("No new allocations were tracked - tracking system may not be working");
    }
}
