//! Simple heavy load test for memscope-rs
//! Tests basic functionality under load without external dependencies

use std::sync::{Arc, Barrier, Mutex, Once};
use std::thread;
use std::time::{Duration, Instant};
use memscope_rs::{get_global_tracker, init, track_var};

static INIT: Once = Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn test_simple_concurrent_allocations() {
    ensure_init();
    let tracker = get_global_tracker();

    let num_threads = 8;
    let allocations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                barrier.wait();

                let mut allocations = Vec::new();

                // Create various types of allocations
                for i in 0..allocations_per_thread {
                    // Vec allocations
                    let data = vec![thread_id as u8; 64 + i];
                    if track_var!(data).is_ok() {
                        allocations.push(data);
                    }

                    // String allocations
                    let text = format!("thread_{}_item_{}", thread_id, i);
                    if track_var!(text).is_ok() {
                        allocations.push(text.into_bytes());
                    }

                    // Box allocations
                    let boxed = Box::new(vec![i as u8; 32]);
                    if track_var!(boxed).is_ok() {
                        allocations.push(*boxed);
                    }
                }

                allocations.len()
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all threads completed
    assert_eq!(results.len(), num_threads);

    // Check tracker stats
    let stats = tracker.get_stats();
    println!("Concurrent test stats: {:?}", stats);

    println!("Simple concurrent allocations test passed!");
}

#[test]
fn test_memory_pressure() {
    ensure_init();
    let tracker = get_global_tracker();

    let start_time = Instant::now();

    // Large allocations
    let mut large_allocs = Vec::new();
    for i in 0..10 {
        let data = vec![i as u8; 1024 * 100]; // 100KB each
        if track_var!(data).is_ok() {
            large_allocs.push(data);
        }
    }

    // Many small allocations
    let mut small_allocs = Vec::new();
    for i in 0..1000 {
        let data = vec![i as u8; 64];
        if track_var!(data).is_ok() {
            small_allocs.push(data);
        }
    }

    // Mixed data structures
    let mut mixed_allocs = Vec::new();
    for i in 0..100 {
        match i % 3 {
            0 => {
                let data = format!("string_data_{}", i);
                if track_var!(data).is_ok() {
                    mixed_allocs.push(data.into_bytes());
                }
            }
            1 => {
                let data = vec![i as u8; 128];
                if track_var!(data).is_ok() {
                    mixed_allocs.push(data);
                }
            }
            2 => {
                let data = Box::new(vec![i as u8; 256]);
                if track_var!(data).is_ok() {
                    mixed_allocs.push(*data);
                }
            }
            _ => unreachable!(),
        }
    }

    let duration = start_time.elapsed();
    println!("Memory pressure test completed in {:?}", duration);

    let stats = tracker.get_stats();
    println!("Memory pressure stats: {:?}", stats);

    // Keep allocations alive
    drop(large_allocs);
    drop(small_allocs);
    drop(mixed_allocs);

    println!("Memory pressure test passed!");
}

#[test]
fn test_rapid_cycles() {
    ensure_init();
    let tracker = get_global_tracker();

    let start_time = Instant::now();

    for cycle in 0..50 {
        let mut cycle_allocs = Vec::new();

        // Allocate
        for i in 0..50 {
            let data = vec![(cycle + i) as u8; 64];
            if track_var!(data).is_ok() {
                cycle_allocs.push(data);
            }
        }

        // Partial cleanup
        for _ in 0..25 {
            if !cycle_allocs.is_empty() {
                cycle_allocs.remove(0);
            }
        }

        // More allocations
        for i in 0..25 {
            let data = format!("cycle_{}_item_{}", cycle, i);
            if track_var!(data).is_ok() {
                cycle_allocs.push(data.into_bytes());
            }
        }

        // Complete cleanup
        drop(cycle_allocs);

        if cycle % 10 == 0 {
            let _stats = tracker.get_stats();
            println!("Cycle {} completed", cycle);
        }
    }

    let duration = start_time.elapsed();
    println!("Rapid cycles test completed in {:?}", duration);

    let final_stats = tracker.get_stats();
    println!("Final rapid cycles stats: {:?}", final_stats);

    println!("Rapid cycles test passed!");
}

#[test]
fn test_mixed_operations() {
    ensure_init();
    let tracker = get_global_tracker();

    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            let shared_data = Arc::clone(&shared_data);
            let tracker = Arc::clone(&tracker);

            thread::spawn(move || {
                barrier.wait();

                match thread_id % 4 {
                    0 => {
                        // Allocator thread
                        for _i in 0..200 {
                            let data = vec![thread_id as u8; 128];
                            if track_var!(data).is_ok() {
                                shared_data.lock().unwrap().push(data);
                            }
                        }
                    }
                    1 => {
                        // Stats reader thread
                        for _ in 0..50 {
                            let _stats = tracker.get_stats();
                            thread::sleep(Duration::from_micros(100));
                        }
                    }
                    2 => {
                        // Memory analyzer thread
                        for _ in 0..25 {
                            let _memory_by_type = tracker.get_memory_by_type();
                            let _active = tracker.get_active_allocations();
                            thread::sleep(Duration::from_micros(200));
                        }
                    }
                    3 => {
                        // Mixed operations thread
                        for i in 0..100 {
                            let data = format!("thread_{}_data_{}", thread_id, i);
                            if track_var!(data).is_ok() {
                                shared_data.lock().unwrap().push(data.into_bytes());
                            }

                            if i % 10 == 0 {
                                let _active = tracker.get_active_allocations();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let final_stats = tracker.get_stats();
    println!("Mixed operations test stats: {:?}", final_stats);

    let final_data = shared_data.lock().unwrap();
    println!("Shared data length: {}", final_data.len());

    println!("Mixed operations test passed!");
}
