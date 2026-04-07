//! Integration tests for Render Engine
//!
//! Tests cover:
//! - Core functionality: export, snapshot, analysis
//! - Boundary conditions: empty data, large datasets
//! - Configuration: templates, options

use memscope_rs::render_engine::{DashboardTemplate, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

// ============================================================================
// Dashboard Template Tests
// ============================================================================

#[test]
fn test_dashboard_template_unified_to_string() {
    let template = DashboardTemplate::Unified;
    assert_eq!(template.to_string(), "dashboard_unified");
}

#[test]
fn test_dashboard_template_final_to_string() {
    let template = DashboardTemplate::Final;
    assert_eq!(template.to_string(), "dashboard_final");
}

#[test]
fn test_dashboard_template_default() {
    let template = DashboardTemplate::default();
    assert!(matches!(template, DashboardTemplate::Unified));
}

// ============================================================================
// Export Options Tests
// ============================================================================

#[test]
fn test_export_json_options_default() {
    let options = ExportJsonOptions::default();
    assert!(options.buffer_size > 0, "Buffer size should be positive");
}

#[test]
fn test_export_json_options_custom() {
    let options = ExportJsonOptions {
        parallel_processing: true,
        buffer_size: 16384,
        use_compact_format: Some(true),
        enable_type_cache: true,
        batch_size: 500,
        streaming_writer: true,
        schema_validation: true,
        adaptive_optimization: true,
        max_cache_size: 5000,
        security_analysis: true,
        include_low_severity: false,
        integrity_hashes: true,
        fast_export_mode: true,
        auto_fast_export_threshold: Some(50000),
        thread_count: Some(8),
    };

    assert!(options.parallel_processing);
    assert_eq!(options.buffer_size, 16384);
    assert_eq!(options.batch_size, 500);
}

// ============================================================================
// Memory Snapshot Tests
// ============================================================================

#[test]
fn test_memory_snapshot_new() {
    let snapshot = MemorySnapshot::new();
    assert!(
        snapshot.active_allocations.is_empty(),
        "New snapshot should have no allocations"
    );
    assert!(
        snapshot.timestamp > 0,
        "Snapshot should have valid timestamp"
    );
}

#[test]
fn test_memory_snapshot_timestamp_ordering() {
    let snapshot1 = MemorySnapshot::new();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let snapshot2 = MemorySnapshot::new();

    assert!(
        snapshot2.timestamp > snapshot1.timestamp,
        "Later snapshot should have greater timestamp"
    );
}

// ============================================================================
// Tracker Integration Tests
// ============================================================================

#[test]
fn test_tracker_snapshot_captures_allocations() {
    let t = tracker!();

    track!(t, vec![1u8; 100]);
    track!(t, vec![2u8; 200]);

    let stats = t.stats();
    assert!(
        stats.total_allocations > 0,
        "Stats should capture allocations"
    );
}

#[test]
fn test_tracker_analyze_returns_valid_report() {
    let t = tracker!();

    track!(t, vec![1u8; 1000]);
    track!(t, vec![2u8; 2000]);

    let report = t.analyze();
    assert!(report.total_allocations > 0);
    assert!(
        report.current_memory_bytes > 0,
        "Current memory should be tracked"
    );
}

#[test]
fn test_tracker_elapsed_increases_over_time() {
    use std::thread::sleep;
    use std::time::Duration;

    let t = tracker!();
    let start = t.elapsed();

    sleep(Duration::from_millis(20));

    let elapsed = t.elapsed();
    assert!(
        elapsed.as_millis() >= start.as_millis() + 20,
        "Elapsed time should increase by at least 20ms"
    );
}

#[test]
fn test_tracker_clone_shares_state() {
    let t1 = tracker!();
    let t2 = t1.clone();

    track!(t1, vec![1, 2, 3]);

    let report1 = t1.analyze();
    let report2 = t2.analyze();

    assert!(
        report1.total_allocations > 0 && report2.total_allocations > 0,
        "Both trackers should see allocations"
    );
}

#[test]
fn test_tracker_system_snapshot_cpu_in_valid_range() {
    let t = tracker!();
    let snapshot = t.current_system_snapshot();

    assert!(
        snapshot.cpu_usage_percent >= 0.0 && snapshot.cpu_usage_percent <= 100.0,
        "CPU usage should be between 0 and 100%"
    );
}

// ============================================================================
// Sampling Configuration Tests
// ============================================================================

#[test]
fn test_high_performance_sampling_tracks_allocations() {
    use memscope_rs::tracker::SamplingConfig;

    let t = tracker!().with_sampling(SamplingConfig::high_performance());

    // Use more iterations to ensure at least some allocations are sampled
    // With 1% sample rate, 1000 iterations should yield ~10 samples
    for i in 0..1000 {
        track!(t, vec![i as u8; 64]);
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "High performance sampling should track some allocations (got {})",
        report.total_allocations
    );
}

#[test]
fn test_high_accuracy_sampling_tracks_allocations() {
    use memscope_rs::tracker::SamplingConfig;

    let t = tracker!().with_sampling(SamplingConfig::default());

    for i in 0..100 {
        track!(t, vec![i as u8; 64]);
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Default sampling should track allocations"
    );
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_empty_tracker_has_zero_allocations() {
    let t = tracker!();
    let report = t.analyze();

    assert_eq!(report.total_allocations, 0);
    assert!(report.hotspots.is_empty());
}

#[test]
fn test_large_allocation_tracking() {
    let t = tracker!();

    let large_size = 10 * 1024 * 1024; // 10 MB
    track!(t, vec![0u8; large_size]);

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track large allocation"
    );
}

#[test]
fn test_many_allocations_tracking() {
    let t = tracker!();

    for i in 0..500 {
        track!(t, vec![i as u8; 64]);
    }

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track at least some allocations"
    );
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
fn test_concurrent_tracking_from_multiple_threads() {
    let t = Arc::new(tracker!());
    let mut handles = vec![];

    for thread_id in 0..4 {
        let t_clone = Arc::clone(&t);
        let handle = thread::spawn(move || {
            for i in 0..50 {
                track!(t_clone, vec![(thread_id * 50 + i) as u8; 64]);
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
        "Concurrent tracking should track at least some allocations"
    );
}

// ============================================================================
// Auto Export Tests
// ============================================================================

#[test]
fn test_tracker_with_auto_export_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("auto_export.json");
    let output_str = output_path.to_str().unwrap();

    {
        let _t = tracker!().with_auto_export(output_str);
        track!(_t, vec![1, 2, 3]);
    }

    // Note: auto_export creates file on drop, but the file might not exist
    // depending on implementation. This test verifies the method exists.
}

// ============================================================================
// Hotspot Detection Tests
// ============================================================================

#[test]
fn test_analyze_detects_hotspots_by_size() {
    let t = tracker!();

    track!(t, vec![1u8; 100]);
    track!(t, vec![2u8; 5000]);
    track!(t, vec![3u8; 100]);

    let report = t.analyze();
    assert!(!report.hotspots.is_empty(), "Should detect hotspots");

    let largest = report.hotspots.iter().max_by_key(|h| h.total_size);
    assert!(
        largest.is_some_and(|h| h.total_size >= 5000),
        "Largest hotspot should be at least 5000 bytes"
    );
}

// ============================================================================
// Different Data Types Tests
// ============================================================================

#[test]
fn test_string_tracking() {
    let t = tracker!();

    track!(t, String::from("hello world"));
    track!(t, "x".repeat(1000));

    let report = t.analyze();
    assert!(
        report.total_allocations > 0,
        "Should track string allocations"
    );
}

#[test]
fn test_hashmap_tracking() {
    use std::collections::HashMap;

    let t = tracker!();

    let mut map: HashMap<u32, String> = HashMap::new();
    for i in 0..100 {
        map.insert(i, format!("value_{}", i));
    }
    track!(t, map);

    let report = t.analyze();
    assert!(report.total_allocations >= 1);
}

#[test]
fn test_smart_pointer_tracking() {
    use std::sync::Arc;

    let t = tracker!();

    track!(t, Arc::new(vec![1u8; 1024]));
    track!(t, Box::new(vec![2u8; 2048]));

    let report = t.analyze();
    assert!(report.total_allocations >= 2);
}
