//! Integration tests for Tracker module
//!
//! Tests cover:
//! - Core functionality: tracking, analysis, snapshots
//! - Boundary conditions: empty data, large allocations
//! - Edge cases: concurrent access, memory pressure

use memscope_rs::{track, tracker};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

// ============================================================================
// Core Functionality Tests
// ============================================================================

#[test]
fn test_track_records_allocation() {
    let t = tracker!();

    let data = vec![0u8; 1024];
    track!(t, data);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track at least one allocation"
    );
}

#[test]
fn test_track_multiple_variables() {
    let t = tracker!();

    track!(t, vec![1u8; 100]);
    track!(t, vec![2u8; 200]);
    track!(t, vec![3u8; 300]);

    let report = t.analyze();
    assert!(
        report.total_allocations >= 3,
        "Should track at least 3 allocations"
    );
}

#[test]
fn test_snapshot_captures_state() {
    let t = tracker!();

    track!(t, vec![1u8; 100]);

    let stats1 = t.stats();
    assert!(stats1.total_allocations > 0);

    track!(t, vec![2u8; 200]);

    let stats2 = t.stats();
    assert!(stats2.total_allocations >= stats1.total_allocations);
}

#[test]
fn test_analyze_detects_hotspots() {
    let t = tracker!();

    track!(t, vec![1u8; 100]);
    track!(t, vec![2u8; 5000]);
    track!(t, vec![3u8; 100]);

    let report = t.analyze();
    assert!(!report.hotspots.is_empty(), "Should detect hotspots");
}

#[test]
fn test_elapsed_time_increases() {
    use std::thread::sleep;
    use std::time::Duration;

    let t = tracker!();
    let start = t.elapsed();

    sleep(Duration::from_millis(50));

    let elapsed = t.elapsed();
    assert!(
        elapsed.as_millis() >= start.as_millis() + 50,
        "Elapsed time should increase by at least 50ms"
    );
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_empty_tracker_state() {
    let t = tracker!();
    let report = t.analyze();

    assert_eq!(
        report.total_allocations, 0,
        "Empty tracker should have zero allocations"
    );
}

#[test]
fn test_small_allocation_tracking() {
    let t = tracker!();

    let small_data = vec![0u8; 1];
    track!(t, small_data);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track small allocation"
    );
}

#[test]
fn test_large_allocation_tracking() {
    let t = tracker!();

    let large_size = 10 * 1024 * 1024; // 10 MB
    let data = vec![0u8; large_size];
    track!(t, data);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track large allocation"
    );
}

#[test]
fn test_many_allocations() {
    let t = tracker!();

    for i in 0..100 {
        let data = vec![i as u8; 64];
        track!(t, data);
    }

    let report = t.analyze();
    assert!(report.total_allocations > 0, "Should track allocations");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_tracker_clone_works() {
    let t1 = tracker!();
    let t2 = t1.clone();

    track!(t1, vec![1, 2, 3]);

    let report1 = t1.analyze();
    let report2 = t2.analyze();

    assert!(report1.total_allocations > 0 || report2.total_allocations > 0);
}

#[test]
fn test_concurrent_tracking() {
    let t = Arc::new(tracker!());
    let mut handles = vec![];

    for thread_id in 0..4 {
        let t_clone = Arc::clone(&t);
        let handle = thread::spawn(move || {
            for i in 0..50 {
                let data = vec![(thread_id * 50 + i) as u8; 64];
                track!(t_clone, data);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Concurrent tracking should work"
    );
}

#[test]
fn test_different_collection_types() {
    let t = tracker!();

    let mut hashmap: HashMap<u32, String> = HashMap::new();
    for i in 0..10 {
        hashmap.insert(i, format!("value_{}", i));
    }
    track!(t, hashmap);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track HashMap allocation"
    );
}

#[test]
fn test_smart_pointer_types() {
    let t = tracker!();

    let arc_data = Arc::new(vec![1u8; 1024]);
    let box_data = Box::new(vec![2u8; 2048]);

    track!(t, arc_data);
    track!(t, box_data);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track Arc and Box allocations"
    );
}

#[test]
fn test_string_tracking() {
    let t = tracker!();

    let short_string = String::from("hello");
    let long_string = "x".repeat(1000);

    track!(t, short_string);
    track!(t, long_string);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track string allocations"
    );
}

// ============================================================================
// Sampling Configuration Tests
// ============================================================================

#[test]
fn test_high_performance_sampling() {
    use memscope_rs::tracker::SamplingConfig;

    let t = tracker!().with_sampling(SamplingConfig::high_performance());

    // Use more iterations to ensure at least some allocations are sampled
    // With 1% sample rate, 1000 iterations should yield ~10 samples
    for i in 0..1000 {
        let data = vec![i as u8; 64];
        track!(t, data);
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "High performance sampling should track allocations (got {})",
        report.total_allocations
    );
}

#[test]
fn test_default_sampling() {
    use memscope_rs::tracker::SamplingConfig;

    let t = tracker!().with_sampling(SamplingConfig::default());

    for i in 0..50 {
        let data = vec![i as u8; 64];
        track!(t, data);
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Default sampling should track allocations"
    );
}

// ============================================================================
// System Integration Tests
// ============================================================================

#[test]
fn test_system_snapshot_returns_valid_data() {
    let t = tracker!();
    let snapshot = t.current_system_snapshot();

    assert!(
        snapshot.cpu_usage_percent <= 100.0,
        "CPU usage should not exceed 100%"
    );
}
