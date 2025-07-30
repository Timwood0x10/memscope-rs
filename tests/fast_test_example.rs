//! Example of fast testing with memscope-rs
//!
//! This test demonstrates how to use the optimized testing mode
//! which reduces overhead and eliminates lock contention warnings.

use memscope_rs::{get_global_tracker, init_test, track_var};

#[test]
fn test_fast_tracking() {
    // Initialize with fast mode
    init_test!();

    // Create and track variables - no warnings in fast mode
    let vec1 = vec![1, 2, 3, 4, 5];
    track_var!(vec1);

    let string1 = String::from("Hello, fast testing!");
    track_var!(string1);

    let boxed_data = Box::new(42);
    track_var!(boxed_data);

    // Variables remain usable
    assert_eq!(vec1.len(), 5);
    assert_eq!(*boxed_data, 42);
    assert!(string1.contains("fast"));

    // Basic stats should still work
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        assert!(stats.total_allocations > 0);
        assert!(stats.active_memory > 0);
    }
}

#[test]
fn test_concurrent_fast_tracking() {
    init_test!();

    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let data = vec![i; 100];
                track_var!(data);

                let string_data = format!("Thread {}", i);
                track_var!(string_data);

                // Return some data to verify it works
                (data.len(), string_data.len())
            })
        })
        .collect();

    // Wait for all threads and verify results
    for handle in handles {
        let (vec_len, string_len) = handle.join().unwrap();
        assert_eq!(vec_len, 100);
        assert!(string_len > 0);
    }

    // Verify tracker stats
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        // Should have tracked multiple allocations
        assert!(stats.total_allocations >= 20); // At least 2 per thread
    }
}

#[test]
fn test_smart_pointer_fast_tracking() {
    init_test!();

    use std::rc::Rc;
    use std::sync::Arc;

    // Test Rc tracking
    let rc_data = Rc::new(vec![1, 2, 3]);
    track_var!(rc_data);

    let rc_clone = Rc::clone(&rc_data);
    track_var!(rc_clone);

    // Test Arc tracking
    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data);

    let arc_clone = Arc::clone(&arc_data);
    track_var!(arc_clone);

    // Verify functionality
    assert_eq!(rc_data.len(), 3);
    assert_eq!(Rc::strong_count(&rc_data), 2);
    assert_eq!(Arc::strong_count(&arc_data), 2);
    assert!(arc_data.contains("Shared"));
}

#[cfg(feature = "test")]
#[test]
fn test_feature_flag_fast_mode() {
    // This test only runs when the test feature is enabled
    let tracker = get_global_tracker();

    // Should be in fast mode when test feature is enabled
    assert!(tracker.is_fast_mode());
}
