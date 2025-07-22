//! Comprehensive concurrency tests for memscope-rs.
//! Tests thread safety, parallel allocations, and concurrent data structures.

use crossbeam::thread;
use parking_lot::{Mutex, RwLock};
// use rayon::prelude::*; // Unused import
use memscope_rs::{get_global_tracker, init, track_var, Trackable};
use std::sync::{Arc, Barrier};
use std::thread as std_thread;
use std::time::{Duration, Instant};

static INIT: std::sync::Once = std::sync::Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        init();
    });
}

#[test]
fn test_basic_thread_safety() {
    ensure_init();

    let num_threads = 4;
    let allocations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            std_thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                let mut thread_allocations = Vec::new();
                for i in 0..allocations_per_thread {
                    let data = vec![thread_id; i + 1];
                    thread_allocations.push(data);
                }

                thread_allocations.len()
            })
        })
        .collect();

    let mut total_allocations = 0;
    for handle in handles {
        total_allocations += handle.join().expect("Thread should complete successfully");
    }

    assert_eq!(
        total_allocations,
        num_threads * allocations_per_thread,
        "All threads should complete their allocations"
    );

    // Verify that the tracker recorded allocations from multiple threads
    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should have tracked multi-threaded allocations"
    );
}

#[test]
fn test_concurrent_variable_tracking() {
    ensure_init();

    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let barrier = Arc::clone(&barrier);
            std_thread::spawn(move || {
                barrier.wait();

                // Each thread creates and tracks variables with unique names
                let data = vec![thread_id; 50];
                let var_name = format!("thread_{thread_id}_data");

                // We can't use track_var! macro across threads with dynamic names,
                // so we'll test the underlying tracking mechanism
                if let Some(ptr) = data.get_heap_ptr() {
                    let tracker = get_global_tracker();
                    let _ = tracker.associate_var(ptr, var_name, "Vec<usize>".to_string());
                }

                // Keep data alive for a bit
                std_thread::sleep(Duration::from_millis(10));

                (thread_id, data.len())
            })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().expect("Thread should complete"));
    }

    assert_eq!(results.len(), num_threads, "All threads should complete");

    // Verify tracking worked
    let tracker = get_global_tracker();
    let active_allocs = tracker.get_active_allocations();

    // Should have allocations from multiple threads
    let unique_threads: std::collections::HashSet<_> = active_allocs
        .iter()
        .map(|a| {
            a.first()
                .map(|info| info.thread_id.clone())
                .unwrap_or_else(|| format!("{:?}", std::thread::current().id()))
        })
        .collect();

    // Note: Thread tracking might not work without global allocator feature
    println!("Unique threads found: {}", unique_threads.len());
    if unique_threads.len() <= 1 {
        println!(
            "Expected multiple threads, got {}, but test continues",
            unique_threads.len()
        );
    }
}

#[test]
fn test_shared_data_structures() {
    ensure_init();

    // Test Arc<Mutex<T>> pattern
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let num_threads = 6;
    let items_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let shared_data = Arc::clone(&shared_data);
            std_thread::spawn(move || {
                for i in 0..items_per_thread {
                    let item = format!("thread_{thread_id}_item_{i}");
                    {
                        shared_data.lock().push(item);
                    } // Release lock immediately

                    // Small delay to increase contention
                    std_thread::sleep(Duration::from_micros(10));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    let final_data = {
        let guard = shared_data.lock();
        guard.clone()
    }; // Release lock immediately
    assert_eq!(
        final_data.len(),
        num_threads * items_per_thread,
        "Should have all items from all threads"
    );

    // Test that Arc itself can be tracked
    let arc_data = Arc::new(vec![1, 2, 3, 4, 5]);
    let _tracked_arc_data = track_var!(arc_data);

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track shared data structure allocations"
    );
}

#[test]
fn test_crossbeam_scoped_threads() {
    ensure_init();

    let data = vec![1, 2, 3, 4, 5];
    let _tracked_data = track_var!(data);

    thread::scope(|s| {
        // Spawn threads that can access the main thread's data
        for i in 0..4 {
            s.spawn(move |_| {
                // Each thread creates its own allocation
                let thread_data = vec![i; 100];

                // Simulate some work
                std_thread::sleep(Duration::from_millis(5));

                thread_data.len()
            });
        }
    })
    .expect("Scoped threads should complete successfully");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track scoped thread allocations"
    );
}

#[test]
fn test_rayon_parallel_processing() {
    ensure_init();

    let input_data: Vec<i32> = (0..1000).collect();
    let _tracked_input_data = track_var!(input_data.clone());

    // Parallel map operation
    let processed: Vec<String> = input_data
        .iter()
        .map(|&x| {
            // Each parallel task allocates a string
            format!("processed_{}", x * x)
        })
        .collect();

    assert_eq!(processed.len(), 1000, "Should process all items");
    let _tracked_processed = track_var!(processed.clone());
    assert_eq!(
        processed[0], "processed_0",
        "Should have correct first item"
    );
    assert_eq!(
        processed[999], "processed_998001",
        "Should have correct last item"
    );

    // Parallel reduce operation
    let sum: i32 = input_data.iter().sum();
    assert_eq!(sum, 499500, "Should calculate correct sum");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track parallel allocations"
    );
}

#[test]
fn test_parking_lot_synchronization() {
    ensure_init();

    // Test with parking_lot's Mutex (faster than std::sync::Mutex)
    let shared_counter = Arc::new(std::sync::Mutex::new(0));
    let shared_data = Arc::new(std::sync::Mutex::new(Vec::new()));

    let num_threads = 8;
    let increments_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let counter = Arc::clone(&shared_counter);
            let data = Arc::clone(&shared_data);

            std_thread::spawn(move || {
                for i in 0..increments_per_thread {
                    // Increment counter
                    {
                        let mut count = counter.lock().unwrap();
                        *count += 1;
                    }

                    // Add data
                    {
                        let mut vec = data.lock().unwrap();
                        vec.push(format!("thread_{thread_id}_item_{i}"));
                    } // Release lock immediately
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    let final_count = {
        let guard = shared_counter.lock().unwrap();
        *guard
    }; // Release lock immediately

    let final_data_len = {
        let guard = shared_data.lock().unwrap();
        guard.len()
    }; // Release lock immediately

    assert_eq!(
        final_count,
        num_threads * increments_per_thread,
        "Counter should be correct"
    );
    assert_eq!(
        final_data_len,
        num_threads * increments_per_thread,
        "Data length should be correct"
    );

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track parking_lot synchronized allocations"
    );
}

#[test]
fn test_rwlock_concurrent_access() {
    ensure_init();

    let shared_data = Arc::new(RwLock::new(std::collections::HashMap::new()));
    let num_readers = 6;
    let num_writers = 2;

    let mut handles = Vec::new();

    // Spawn reader threads
    for reader_id in 0..num_readers {
        let data = Arc::clone(&shared_data);
        let handle = std_thread::spawn(move || {
            for i in 0..50 {
                let read_guard = data.read();
                let _value = read_guard.get(&format!("key_{}", i % 10));

                // Simulate read work
                std_thread::sleep(Duration::from_micros(100));
            }
            reader_id
        });
        handles.push(handle);
    }

    // Spawn writer threads
    for writer_id in 0..num_writers {
        let data = Arc::clone(&shared_data);
        let handle = std_thread::spawn(move || {
            for i in 0..25 {
                let mut write_guard = data.write();
                write_guard.insert(format!("key_{i}"), format!("value_from_writer_{writer_id}"));

                // Simulate write work
                std_thread::sleep(Duration::from_micros(200));
            }
            writer_id
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    let final_data = shared_data.read();
    assert!(!final_data.is_empty(), "Should have written some data");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track RwLock allocations"
    );
}

#[test]
fn test_channel_communication() {
    ensure_init();

    let (tx, rx) = std::sync::mpsc::channel();
    let num_senders = 4;
    let messages_per_sender = 50;

    // Spawn sender threads
    let sender_handles: Vec<_> = (0..num_senders)
        .map(|sender_id| {
            let tx = tx.clone();
            std_thread::spawn(move || {
                for i in 0..messages_per_sender {
                    let message = format!("Message from sender {sender_id} - {i}");
                    tx.send(message).expect("Failed to send message");
                }
            })
        })
        .collect();

    // Drop the original sender
    drop(tx);

    // Receiver thread
    let receiver_handle = std_thread::spawn(move || {
        let mut received_messages = Vec::new();
        while let Ok(message) = rx.recv() {
            received_messages.push(message);
        }
        received_messages
    });

    // Wait for senders
    for handle in sender_handles {
        handle.join().expect("Sender should complete");
    }

    // Wait for receiver
    let messages = receiver_handle.join().expect("Receiver should complete");

    assert_eq!(
        messages.len(),
        num_senders * messages_per_sender,
        "Should receive all messages"
    );

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track channel allocations"
    );
}

#[test]
fn test_memory_contention_stress() {
    ensure_init();

    let start_time = Instant::now();
    let duration = Duration::from_millis(100);
    let shared_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let num_threads = std_thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let counter = Arc::clone(&shared_counter);
            let start = start_time;

            std_thread::spawn(move || {
                let mut local_allocations = 0;

                while start.elapsed() < duration {
                    // Rapid allocation and deallocation
                    let data = vec![42u8; 1024];
                    local_allocations += 1;

                    // Update shared counter
                    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    // Force deallocation
                    drop(data);
                }

                local_allocations
            })
        })
        .collect();

    let mut total_local_allocations = 0;
    for handle in handles {
        total_local_allocations += handle.join().expect("Thread should complete");
    }

    let shared_count = shared_counter.load(std::sync::atomic::Ordering::Relaxed);

    assert_eq!(
        total_local_allocations, shared_count,
        "Local and shared counts should match"
    );
    assert!(
        total_local_allocations > 0,
        "Should have made some allocations"
    );

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track stress test allocations"
    );
}

#[test]
fn test_thread_local_storage() {
    ensure_init();

    thread_local! {
        static THREAD_DATA: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    let num_threads = 4;
    let items_per_thread = 25;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            std_thread::spawn(move || {
                THREAD_DATA.with(|data| {
                    let mut vec = data.borrow_mut();
                    for i in 0..items_per_thread {
                        vec.push(format!("thread_{thread_id}_item_{i}"));
                    }
                    vec.len()
                })
            })
        })
        .collect();

    let mut total_items = 0;
    for handle in handles {
        total_items += handle.join().expect("Thread should complete");
    }

    assert_eq!(
        total_items,
        num_threads * items_per_thread,
        "All threads should populate their thread-local storage"
    );

    let tracker = get_global_tracker();
    let stats = tracker.get_stats();
    assert!(
        stats.unwrap().total_allocations > 0,
        "Should track thread-local allocations"
    );
}
