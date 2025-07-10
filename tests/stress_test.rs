//! Stress tests for memscope-rs memory tracking under high load.

use memscope_rs::{get_global_tracker, init, track_var};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn test_high_frequency_allocations() {
    ensure_init();

    let start = Instant::now();
    let mut allocations = Vec::new();

    // Perform many small allocations rapidly
    for i in 0..1000 {
        let data = vec![i; 10];
        if i % 100 == 0 {
            // Only track every 100th allocation to avoid overwhelming the system
            let _ = track_var!(data);
        }
        allocations.push(data);
    }

    let duration = start.elapsed();
    println!("High frequency allocation test completed in {duration:?}");

    // Verify system is still responsive
    let tracker = get_global_tracker();
    let stats = tracker.get_stats().expect("Should be able to get stats");
    println!(
        "Final stats: {} active allocations",
        stats.active_allocations
    );

    // Should complete within reasonable time (adjust based on system)
    assert!(
        duration < Duration::from_secs(5),
        "Test took too long: {duration:?}"
    );
}

#[test]
fn test_concurrent_allocations() {
    ensure_init();

    let num_threads = 4;
    let allocations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                let mut thread_allocations = Vec::new();
                for i in 0..allocations_per_thread {
                    let data = vec![thread_id * 1000 + i; 20];
                    if i % 10 == 0 {
                        // Track some allocations
                        let _ = track_var!(data);
                    }
                    thread_allocations.push(data);
                }

                thread_allocations.len()
            })
        })
        .collect();

    // Wait for all threads to complete
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all threads completed successfully
    for (i, result) in results.iter().enumerate() {
        assert_eq!(*result, allocations_per_thread, "Thread {i} failed");
    }

    // Verify tracker is still functional
    let tracker = get_global_tracker();
    let stats = tracker
        .get_stats()
        .expect("Should be able to get stats after concurrent test");
    println!(
        "Concurrent test stats: {} active allocations",
        stats.active_allocations
    );
}

#[test]
fn test_large_allocations() {
    ensure_init();

    let tracker = get_global_tracker();
    let initial_stats = tracker.get_stats().unwrap();

    // Test with progressively larger allocations
    let sizes = vec![1024, 10_240, 102_400, 1_024_000]; // 1KB to 1MB
    let mut large_allocations = Vec::new();

    for size in sizes {
        let data = vec![0u8; size];
        let _ = track_var!(data);
        large_allocations.push(data);
    }

    let final_stats = tracker.get_stats().unwrap();

    // Verify memory tracking is working for large allocations
    assert!(final_stats.active_memory > initial_stats.active_memory);
    assert!(final_stats.peak_memory >= final_stats.active_memory);

    println!(
        "Large allocation test - Peak memory: {} bytes",
        final_stats.peak_memory
    );
}

#[test]
fn test_allocation_deallocation_cycles() {
    ensure_init();

    let tracker = get_global_tracker();

    // Perform many allocation/deallocation cycles
    for cycle in 0..50 {
        let mut temp_allocations = Vec::new();

        // Allocate
        for i in 0..20 {
            let data = vec![cycle * 100 + i; 50];
            if i % 5 == 0 {
                let _ = track_var!(data);
            }
            temp_allocations.push(data);
        }

        // Let allocations go out of scope (deallocate)
        drop(temp_allocations);

        // Check stats periodically
        if cycle % 10 == 0 {
            let stats = tracker.get_stats().unwrap();
            println!(
                "Cycle {}: {} active allocations",
                cycle, stats.active_allocations
            );
        }
    }

    // Verify system is still responsive
    let final_stats = tracker.get_stats().unwrap();
    println!(
        "Cycle test final stats: {} total allocations, {} total deallocations",
        final_stats.total_allocations, final_stats.total_deallocations
    );
}

#[test]
fn test_mixed_type_allocations() {
    ensure_init();

    let tracker = get_global_tracker();

    // Allocate various types rapidly
    for i in 0..100 {
        match i % 5 {
            0 => {
                let data = vec![i; 10];
                let _ = track_var!(data);
            }
            1 => {
                let data = format!("String number {i}");
                let _ = track_var!(data);
            }
            2 => {
                let data = Box::new(i);
                let _ = track_var!(data);
            }
            3 => {
                let data = std::rc::Rc::new(vec![i; 5]);
                let _ = track_var!(data);
            }
            4 => {
                let data = std::sync::Arc::new(format!("Arc {i}"));
                let _ = track_var!(data);
            }
            _ => unreachable!(),
        }
    }

    // Verify type tracking is working
    let memory_by_type = tracker.get_memory_by_type().unwrap();
    println!(
        "Mixed type test found {} different types",
        memory_by_type.len()
    );

    // Should have multiple types tracked
    assert!(
        !memory_by_type.is_empty(),
        "Should track at least some types"
    );
}

#[test]
fn test_export_under_load() {
    ensure_init();

    let tracker = get_global_tracker();

    // Create some allocations
    let mut allocations = Vec::new();
    for i in 0..50 {
        let data = vec![i; 100];
        let _ = track_var!(data);
        allocations.push(data);
    }

    // Test exports while system is under load
    let export_start = Instant::now();

    // JSON export
    let json_result = tracker.export_to_json("stress_test_output.json");
    assert!(json_result.is_ok(), "JSON export should succeed under load");

    // SVG export
    let svg_result = tracker.export_memory_analysis("stress_test_output.svg");
    assert!(svg_result.is_ok(), "SVG export should succeed under load");

    let export_duration = export_start.elapsed();
    println!("Export under load completed in {export_duration:?}");

    // Cleanup
    std::fs::remove_file("stress_test_output.json").ok();
    std::fs::remove_file("stress_test_output.svg").ok();

    // Should complete within reasonable time
    assert!(
        export_duration < Duration::from_secs(10),
        "Export took too long under load"
    );
}

#[test]
fn test_memory_growth_bounds() {
    ensure_init();

    let tracker = get_global_tracker();
    let initial_stats = tracker.get_stats().unwrap();

    // Perform many allocations to test if tracking memory grows unbounded
    for i in 0..500 {
        let data = vec![i; 10];
        let _ = track_var!(data);

        // Check memory growth every 100 allocations
        if i % 100 == 0 {
            let stats = tracker.get_stats().unwrap();
            println!(
                "Iteration {}: {} active allocations, {} total",
                i, stats.active_allocations, stats.total_allocations
            );
        }
    }

    let final_stats = tracker.get_stats().unwrap();

    // Verify reasonable memory usage (this is a heuristic check)
    let growth_ratio =
        final_stats.total_allocations as f64 / initial_stats.total_allocations.max(1) as f64;
    println!("Memory growth ratio: {growth_ratio:.2}");

    // The tracker itself shouldn't use excessive memory
    // This is a basic sanity check - in production you'd want more sophisticated monitoring
    assert!(
        final_stats.total_allocations < 10000,
        "Suspiciously high allocation count"
    );
}
