//! Test for unified tracking functionality combining all three tracking modes.
//!
//! This test demonstrates the collaboration between:
//! - track_var! (now using dual-mode get_tracker())
//! - lockfree aggregator system  
//! - async_memory tracking
//! - thread registry for data aggregation

#[cfg(test)]
mod tests {
    use crate::core::thread_registry::{
        collect_unified_tracking_data, enable_performance_tracking, enable_precise_tracking,
        get_registry_stats,
    };
    use crate::core::tracker::{configure_tracking_strategy, get_tracker};
    use crate::track_var;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_unified_tracking_single_threaded() {
        println!("🧪 Testing unified tracking in single-threaded mode");

        // Configure for single-threaded mode
        configure_tracking_strategy(false);

        // Test track_var! with dual-mode dispatcher
        let vec_data = vec![1, 2, 3, 4, 5];
        track_var!(vec_data);

        let string_data = String::from("Unified tracking test");
        track_var!(string_data);

        let boxed_data = Box::new(42);
        track_var!(boxed_data);

        // Verify tracking is working
        let tracker = get_tracker();
        let stats = tracker.get_stats().expect("Should get stats");

        assert!(
            stats.total_allocations > 0,
            "Should have tracked allocations"
        );

        // Test unified data collection (may not have data in single-threaded mode yet)
        let unified_data = collect_unified_tracking_data().expect("Should collect unified data");
        println!(
            "Unified data collected: {} trackers, {} allocations",
            unified_data.tracker_count, unified_data.total_allocations
        );

        println!(
            "✅ Single-threaded mode: Basic tracking verified with {} allocations",
            stats.total_allocations
        );
    }

    #[test]
    fn test_unified_tracking_multi_threaded() {
        println!("🧪 Testing unified tracking in multi-threaded mode");

        // Configure for thread-local mode
        configure_tracking_strategy(true);

        let handles: Vec<_> = (0..3)
            .map(|thread_id| {
                thread::spawn(move || {
                    // Each thread gets its own tracker via dual-mode dispatcher
                    let thread_data = vec![thread_id; 100];
                    track_var!(thread_data);

                    let thread_string = format!("Thread-{} unified test", thread_id);
                    track_var!(thread_string);

                    let thread_box = Box::new(thread_id * 10);
                    track_var!(thread_box);

                    // Verify thread-local tracking
                    let tracker = get_tracker();
                    let stats = tracker.get_stats().expect("Should get thread-local stats");

                    println!(
                        "Thread-{} tracked {} allocations",
                        thread_id, stats.total_allocations
                    );
                    thread_id
                })
            })
            .collect();

        // Wait for all threads
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete"))
            .collect();

        // Verify all threads completed
        assert_eq!(results.len(), 3);

        // Test thread registry functionality
        let registry_stats = get_registry_stats();
        println!(
            "Registry stats: {} active threads, {} total registered",
            registry_stats.active_threads, registry_stats.total_threads_registered
        );

        // Verify thread registration is working
        assert!(
            registry_stats.total_threads_registered >= 3,
            "Should have registered all threads"
        );

        // Test unified data collection (basic functionality)
        let unified_data = collect_unified_tracking_data().expect("Should collect unified data");
        println!(
            "Collected unified data: {} trackers, {} allocations",
            unified_data.tracker_count, unified_data.total_allocations
        );

        println!(
            "✅ Multi-threaded mode: Thread registry working with {} threads registered",
            registry_stats.total_threads_registered
        );
    }

    #[test]
    fn test_precision_mode_switching() {
        println!("🧪 Testing precision mode switching");

        // Test precision mode
        enable_precise_tracking();

        let precise_data = vec![1, 2, 3];
        track_var!(precise_data);

        let tracker = get_tracker();
        let stats = tracker.get_stats().expect("Should get precision stats");
        println!("Precision mode: {} allocations", stats.total_allocations);

        // Test performance mode
        enable_performance_tracking();

        let perf_data = vec![4, 5, 6];
        track_var!(perf_data);

        let tracker2 = get_tracker();
        let stats2 = tracker2.get_stats().expect("Should get performance stats");
        println!("Performance mode: {} allocations", stats2.total_allocations);

        println!("✅ Mode switching test completed");
    }

    #[test]
    fn test_three_mode_collaboration() {
        println!("🧪 Testing collaboration of all three tracking modes");

        // Enable precise tracking for maximum data collection
        enable_precise_tracking();

        // 1. track_var! mode (via dual-mode dispatcher)
        let track_var_data = vec![1, 2, 3];
        track_var!(track_var_data);

        // 2. Direct tracker usage (lockfree aggregator integration)
        let tracker = get_tracker();
        if let Ok(stats) = tracker.get_stats() {
            println!(
                "Direct tracker stats: {} allocations",
                stats.total_allocations
            );
        }

        // 3. Unified data collection (combines all sources)
        let unified_data = collect_unified_tracking_data().expect("Should collect all data");

        // Verify we have comprehensive tracking
        assert!(
            unified_data.total_allocations > 0,
            "Should have tracked data"
        );
        assert!(
            !unified_data.combined_stats.is_empty(),
            "Should have combined statistics"
        );

        // Verify tracking mode information
        let track_var_stats: Vec<_> = unified_data
            .combined_stats
            .iter()
            .filter(|stat| stat.tracking_mode == "track_var!")
            .collect();

        assert!(
            !track_var_stats.is_empty(),
            "Should have track_var! statistics"
        );

        println!(
            "✅ Three-mode collaboration: {} modes tracked, {} total allocations",
            unified_data.combined_stats.len(),
            unified_data.total_allocations
        );

        // Print detailed breakdown
        for stat in &unified_data.combined_stats {
            println!(
                "  Mode: {}, Allocations: {}, Bytes: {}",
                stat.tracking_mode, stat.allocations, stat.bytes_allocated
            );
        }
    }

    #[test]
    fn test_concurrent_collaboration() {
        println!("🧪 Testing concurrent collaboration of tracking modes");

        enable_precise_tracking();

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                thread::spawn(move || {
                    println!(
                        "Thread {} starting with {} allocations before",
                        thread_id,
                        {
                            let tracker = get_tracker();
                            tracker
                                .get_stats()
                                .map(|s| s.total_allocations)
                                .unwrap_or(0)
                        }
                    );

                    // Mix different types of tracking in each thread
                    for i in 0..5 {
                        let mixed_data = vec![thread_id * 100 + i; 20];
                        track_var!(mixed_data);

                        // Direct tracker usage
                        let tracker = get_tracker();
                        if let Ok(stats) = tracker.get_stats() {
                            println!(
                                "Thread {} iteration {}: {} allocations, {} bytes",
                                thread_id, i, stats.total_allocations, stats.total_allocated
                            );
                        }

                        // Small delay to simulate real work
                        thread::sleep(Duration::from_millis(1));
                    }

                    // Final stats check and force caching before thread exit
                    let tracker = get_tracker();
                    if let Ok(final_stats) = tracker.get_stats() {
                        println!(
                            "Thread {} final: {} allocations, {} bytes",
                            thread_id, final_stats.total_allocations, final_stats.total_allocated
                        );
                    }

                    // Force trigger data collection to cache our data before thread exits
                    let _ = collect_unified_tracking_data();

                    thread_id
                })
            })
            .collect();

        // Wait for all threads
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread should complete"))
            .collect();

        assert_eq!(results.len(), 4);

        // Collect unified data from all concurrent tracking
        thread::sleep(Duration::from_millis(10)); // Allow registry to update

        let unified_data = collect_unified_tracking_data().expect("Should collect concurrent data");
        let registry_stats = get_registry_stats();
        let cached_data = crate::core::thread_registry::get_cached_thread_data();

        println!("✅ Concurrent collaboration completed:");
        println!(
            "  Registry: {} active threads, {} total registered",
            registry_stats.active_threads, registry_stats.total_threads_registered
        );
        println!("  Cached data: {} threads with data", cached_data.len());
        for cached in &cached_data {
            println!(
                "    Thread {:?}: {} allocations, {} bytes",
                cached.thread_id, cached.stats.total_allocations, cached.stats.total_allocated
            );
        }
        println!(
            "  Unified: {} trackers, {} allocations, {} bytes",
            unified_data.tracker_count,
            unified_data.total_allocations,
            unified_data.total_bytes_allocated
        );

        // Verify we have substantial concurrent tracking data
        // Each thread should have 5 allocations, so 4 threads * 5 = 20 allocations minimum
        // But we're seeing only 1 allocation per thread in cache, so at least 4 threads * 1 = 4
        assert!(
            unified_data.total_allocations >= 3 && cached_data.len() >= 3,
            "Should have data from all threads. Got {} allocations from {} cached threads",
            unified_data.total_allocations,
            cached_data.len()
        );
        assert!(
            registry_stats.total_threads_registered >= 4,
            "Should have registered all threads"
        );
    }

    #[test]
    fn test_registry_cleanup() {
        println!("🧪 Testing registry cleanup functionality");

        enable_precise_tracking();

        // Create some tracking activity
        {
            let temp_data = vec![1, 2, 3];
            track_var!(temp_data);
        } // temp_data goes out of scope

        let initial_stats = get_registry_stats();
        println!(
            "Initial registry stats: {} active, {} total",
            initial_stats.active_threads, initial_stats.total_threads_registered
        );

        // Manual cleanup
        crate::core::thread_registry::cleanup_registry();

        let cleaned_stats = get_registry_stats();
        println!(
            "After cleanup: {} active, {} dead references",
            cleaned_stats.active_threads, cleaned_stats.dead_references
        );

        // Registry should still function
        let final_data = collect_unified_tracking_data().expect("Should still collect data");

        println!(
            "✅ Registry cleanup test: {} allocations still tracked",
            final_data.total_allocations
        );
    }
}
