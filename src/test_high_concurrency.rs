//! High concurrency test for unified tracking system with 20+ threads.
//!
//! This test verifies that our three tracking modes work seamlessly
//! under heavy concurrent load with precise tracking capabilities.

#[cfg(test)]
mod tests {
    use crate::core::thread_registry::{
        collect_unified_tracking_data, enable_precise_tracking, get_registry_stats,
    };
    use crate::core::tracker::get_tracker;
    use crate::track_var;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_20_plus_threads_unified_tracking() {
        println!("🚀 Testing unified tracking with 20+ threads");

        // Enable precise tracking for maximum accuracy
        enable_precise_tracking();

        let thread_count = 25; // Test with 25 threads
        let operations_per_thread = 50;
        let completed_threads = Arc::new(AtomicUsize::new(0));
        let total_operations = Arc::new(AtomicUsize::new(0));

        println!(
            "🔧 Launching {} threads with {} operations each",
            thread_count, operations_per_thread
        );

        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let completed = Arc::clone(&completed_threads);
                let operations = Arc::clone(&total_operations);

                thread::spawn(move || {
                    // Each thread performs multiple tracking operations
                    for op_id in 0..operations_per_thread {
                        // Test track_var! with different data types
                        let vec_data = vec![thread_id; op_id + 1];
                        track_var!(vec_data);

                        let string_data = format!("Thread-{}-Op-{}", thread_id, op_id);
                        track_var!(string_data);

                        let boxed_data = Box::new(thread_id * 1000 + op_id);
                        track_var!(boxed_data);

                        // Test direct tracker access
                        let tracker = get_tracker();
                        if let Ok(stats) = tracker.get_stats() {
                            if stats.total_allocations > 0 && op_id % 10 == 0 {
                                println!(
                                    "Thread-{} Op-{}: {} allocations tracked",
                                    thread_id, op_id, stats.total_allocations
                                );
                            }
                        }

                        operations.fetch_add(1, Ordering::Relaxed);

                        // Small delay to simulate real work
                        if op_id % 5 == 0 {
                            thread::sleep(Duration::from_millis(1));
                        }
                    }

                    completed.fetch_add(1, Ordering::Relaxed);
                    println!(
                        "✅ Thread-{} completed {} operations",
                        thread_id, operations_per_thread
                    );
                    thread_id
                })
            })
            .collect();

        // Monitor progress while threads are running
        let start_time = std::time::Instant::now();
        while completed_threads.load(Ordering::Relaxed) < thread_count {
            thread::sleep(Duration::from_millis(100));
            let current_completed = completed_threads.load(Ordering::Relaxed);
            let current_operations = total_operations.load(Ordering::Relaxed);

            if current_completed > 0 {
                println!(
                    "📊 Progress: {}/{} threads, {} total operations",
                    current_completed, thread_count, current_operations
                );
            }
        }

        // Wait for all threads to complete
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete"))
            .collect();

        let elapsed = start_time.elapsed();
        println!("⏱️  All threads completed in {:?}", elapsed);

        // Verify all threads completed successfully
        assert_eq!(results.len(), thread_count);
        for i in 0..thread_count {
            assert!(results.contains(&i), "Thread {} should have completed", i);
        }

        // Test thread registry statistics
        let registry_stats = get_registry_stats();
        println!(
            "📈 Registry stats: {} active threads, {} total registered",
            registry_stats.active_threads, registry_stats.total_threads_registered
        );

        // Should have registered at least our test threads
        assert!(
            registry_stats.total_threads_registered >= thread_count,
            "Should have registered at least {} threads, got {}",
            thread_count,
            registry_stats.total_threads_registered
        );

        // Test unified data collection under high concurrency
        println!("🔍 Collecting unified tracking data...");
        let unified_data = collect_unified_tracking_data().expect("Should collect unified data");

        println!("📊 Unified data summary:");
        println!("  Trackers: {}", unified_data.tracker_count);
        println!("  Total allocations: {}", unified_data.total_allocations);
        println!("  Total bytes: {}", unified_data.total_bytes_allocated);
        println!("  Active threads: {}", unified_data.active_threads);

        // Final verification
        let final_operations = total_operations.load(Ordering::Relaxed);
        let expected_operations = thread_count * operations_per_thread;

        assert_eq!(
            final_operations, expected_operations,
            "Should have completed all {} operations, got {}",
            expected_operations, final_operations
        );

        println!(
            "🎉 SUCCESS: {} threads completed {} total operations in {:?}",
            thread_count, final_operations, elapsed
        );
    }

    #[test]
    fn test_concurrent_stress_with_mixed_workloads() {
        println!("🔥 Stress testing with mixed concurrent workloads");

        enable_precise_tracking();

        let heavy_threads = 10; // Heavy memory operations
        let light_threads = 15; // Light memory operations
        let total_threads = heavy_threads + light_threads;

        let mut handles = Vec::new();

        // Heavy workload threads
        for thread_id in 0..heavy_threads {
            let handle = thread::spawn(move || {
                for i in 0..30 {
                    // Large allocations
                    let large_vec = vec![thread_id; 1000 + i * 100];
                    track_var!(large_vec);

                    let large_string = "x".repeat(500 + i * 50);
                    track_var!(large_string);

                    // Multiple small allocations
                    for j in 0..10 {
                        let small_data = vec![j; 10];
                        track_var!(small_data);
                    }

                    thread::sleep(Duration::from_millis(2));
                }

                format!("Heavy-{}", thread_id)
            });
            handles.push(handle);
        }

        // Light workload threads
        for thread_id in 0..light_threads {
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    // Small, frequent allocations
                    let small_vec = vec![thread_id; 5];
                    track_var!(small_vec);

                    let short_string = format!("Light-{}-{}", thread_id, i);
                    track_var!(short_string);

                    if i % 20 == 0 {
                        thread::sleep(Duration::from_millis(1));
                    }
                }

                format!("Light-{}", thread_id)
            });
            handles.push(handle);
        }

        println!(
            "🔄 Running {} heavy + {} light threads = {} total",
            heavy_threads, light_threads, total_threads
        );

        // Wait for all threads with timeout monitoring
        let start = std::time::Instant::now();
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete"))
            .collect();

        let elapsed = start.elapsed();

        // Verify completion
        assert_eq!(results.len(), total_threads);

        // Check thread types
        let heavy_completed = results.iter().filter(|s| s.starts_with("Heavy")).count();
        let light_completed = results.iter().filter(|s| s.starts_with("Light")).count();

        assert_eq!(heavy_completed, heavy_threads);
        assert_eq!(light_completed, light_threads);

        // Registry verification
        let registry_stats = get_registry_stats();
        println!("📊 Stress test results:");
        println!("  Elapsed time: {:?}", elapsed);
        println!(
            "  Registry: {} active, {} total registered",
            registry_stats.active_threads, registry_stats.total_threads_registered
        );

        // Data collection verification
        let unified_data = collect_unified_tracking_data().expect("Should collect data");
        println!(
            "  Unified data: {} trackers, {} allocations",
            unified_data.tracker_count, unified_data.total_allocations
        );

        assert!(
            registry_stats.total_threads_registered >= total_threads,
            "Should have registered at least {} threads",
            total_threads
        );

        println!(
            "🎉 Stress test PASSED: Mixed workload with {} threads in {:?}",
            total_threads, elapsed
        );
    }

    #[test]
    fn test_concurrent_registry_cleanup_under_load() {
        println!("🧹 Testing registry cleanup under concurrent load");

        enable_precise_tracking();

        let wave_count = 3;
        let threads_per_wave = 12;

        for wave in 0..wave_count {
            println!(
                "🌊 Wave {} of {}: Launching {} threads",
                wave + 1,
                wave_count,
                threads_per_wave
            );

            let wave_handles: Vec<_> = (0..threads_per_wave)
                .map(|thread_id| {
                    thread::spawn(move || {
                        // Short-lived intense activity
                        for i in 0..20 {
                            let data = vec![wave * 100 + thread_id * 10 + i; 50];
                            track_var!(data);

                            let tracker = get_tracker();
                            let _stats = tracker.get_stats().unwrap_or_default();
                        }

                        thread_id
                    })
                })
                .collect();

            // Wait for this wave to complete
            let wave_results: Vec<_> = wave_handles
                .into_iter()
                .map(|h| h.join().expect("Wave thread should complete"))
                .collect();

            assert_eq!(wave_results.len(), threads_per_wave);

            // Check registry state after each wave
            let stats = get_registry_stats();
            println!(
                "  After wave {}: {} active, {} total registered",
                wave + 1,
                stats.active_threads,
                stats.total_threads_registered
            );

            // Manual cleanup between waves
            crate::core::thread_registry::cleanup_registry();

            // Small delay between waves
            thread::sleep(Duration::from_millis(50));
        }

        // Final verification
        let final_stats = get_registry_stats();
        let unified_data = collect_unified_tracking_data().expect("Should collect final data");

        println!("🏁 Final results:");
        println!(
            "  Registry: {} active, {} total across all waves",
            final_stats.active_threads, final_stats.total_threads_registered
        );
        println!(
            "  Unified: {} trackers, {} allocations",
            unified_data.tracker_count, unified_data.total_allocations
        );

        // Should have handled multiple waves successfully
        assert!(
            final_stats.total_threads_registered >= wave_count * threads_per_wave,
            "Should have registered threads from all waves"
        );

        println!(
            "🎉 Registry cleanup test PASSED: {} waves × {} threads",
            wave_count, threads_per_wave
        );
    }
}
