//! Unit tests for the unified tracker API.
//!
//! Kept in a separate file so `src/tracker.rs` stays focused on the public API
//! and within the project line budget.

use super::*;
#[test]
fn test_tracker_creation() {
    let tracker = Tracker::new();
    let _ = tracker;
}

#[test]
fn test_tracker_with_config() {
    let tracker = Tracker::new()
        .with_sampling(SamplingConfig::demo())
        .with_system_monitoring();
    let _ = tracker;
}

#[test]
fn test_track_macro() {
    let tracker = tracker!();
    let my_vec = vec![1, 2, 3];
    track!(tracker, my_vec);
}

#[test]
fn test_analyze() {
    let tracker = tracker!();
    let data = vec![1, 2, 3];
    track!(tracker, data);
    let report = tracker.analyze();
    assert!(report.total_allocations > 0);
}

#[cfg(target_os = "macos")]
#[test]
fn test_system_monitoring() {
    system_monitor::SystemMonitor::global();
    std::thread::sleep(std::time::Duration::from_millis(350));

    let cpu = system_monitor::cpu_usage();
    let mem = system_monitor::memory_used();
    let total = system_monitor::memory_total();

    println!("CPU: {:.2}%", cpu);
    println!("Memory: {} / {} bytes", mem, total);

    assert!((0.0..=100.0).contains(&cpu));
    assert!(total > 0, "total memory should be initialized by now");
}

#[test]
fn test_current_system_snapshot() {
    std::thread::sleep(std::time::Duration::from_millis(150));

    let tracker = tracker!();
    let snapshot = tracker.current_system_snapshot();

    println!(
        "Snapshot: CPU={:.2}%, Mem={:.2}%",
        snapshot.cpu_usage_percent, snapshot.memory_usage_percent
    );

    assert!(snapshot.cpu_usage_percent >= 0.0 && snapshot.cpu_usage_percent <= 100.0);
}

#[test]
fn test_sampling_config_default() {
    let config = SamplingConfig::default();
    assert_eq!(config.sample_rate, 1.0);
    assert!(!config.capture_call_stack);
    assert_eq!(config.max_stack_depth, 10);
}

#[test]
fn test_sampling_config_demo() {
    let config = SamplingConfig::demo();
    assert_eq!(config.sample_rate, 0.1);
    assert!(!config.capture_call_stack);
    assert_eq!(config.max_stack_depth, 5);
}

#[test]
fn test_sampling_config_full() {
    let config = SamplingConfig::full();
    assert_eq!(config.sample_rate, 1.0);
    assert!(config.capture_call_stack);
    assert_eq!(config.max_stack_depth, 20);
}

#[test]
fn test_sampling_config_high_performance() {
    let config = SamplingConfig::high_performance();
    assert_eq!(config.sample_rate, 0.01);
    assert!(!config.capture_call_stack);
    assert_eq!(config.max_stack_depth, 0);
}

#[test]
fn test_sampling_config_clone() {
    let config = SamplingConfig::full();
    let cloned = config.clone();
    assert_eq!(cloned.sample_rate, config.sample_rate);
    assert_eq!(cloned.capture_call_stack, config.capture_call_stack);
}

#[test]
fn test_sampling_config_debug() {
    let config = SamplingConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("SamplingConfig"));
    assert!(debug_str.contains("sample_rate"));
}

#[test]
fn test_system_snapshot_default() {
    let snapshot = SystemSnapshot::default();
    assert_eq!(snapshot.timestamp, 0);
    assert_eq!(snapshot.cpu_usage_percent, 0.0);
    assert_eq!(snapshot.memory_usage_bytes, 0);
    assert_eq!(snapshot.thread_count, 0);
}

#[test]
fn test_system_snapshot_clone() {
    let snapshot = SystemSnapshot {
        timestamp: 1000,
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 1024 * 1024,
        memory_usage_percent: 25.0,
        thread_count: 4,
        disk_read_bps: 1000,
        disk_write_bps: 500,
        network_rx_bps: 2000,
        network_tx_bps: 1000,
        gpu_usage_percent: 30.0,
        gpu_memory_used: 512 * 1024 * 1024,
        gpu_memory_total: 2 * 1024 * 1024 * 1024,
    };

    let cloned = snapshot.clone();
    assert_eq!(cloned.timestamp, 1000);
    assert_eq!(cloned.cpu_usage_percent, 50.0);
}

#[test]
fn test_system_snapshot_debug() {
    let snapshot = SystemSnapshot::default();
    let debug_str = format!("{:?}", snapshot);
    assert!(debug_str.contains("SystemSnapshot"));
}

#[test]
fn test_analysis_report_creation() {
    let report = AnalysisReport {
        total_allocations: 100,
        total_deallocations: 50,
        active_allocations: 50,
        peak_memory_bytes: 1024 * 1024,
        current_memory_bytes: 512 * 1024,
        allocation_rate_per_sec: 10.0,
        deallocation_rate_per_sec: 5.0,
        hotspots: vec![],
        system_snapshots: vec![],
    };

    assert_eq!(report.total_allocations, 100);
    assert_eq!(report.active_allocations, 50);
}

#[test]
fn test_analysis_report_clone() {
    let report = AnalysisReport {
        total_allocations: 10,
        total_deallocations: 5,
        active_allocations: 5,
        peak_memory_bytes: 1024,
        current_memory_bytes: 512,
        allocation_rate_per_sec: 1.0,
        deallocation_rate_per_sec: 0.5,
        hotspots: vec![],
        system_snapshots: vec![],
    };

    let cloned = report.clone();
    assert_eq!(cloned.total_allocations, 10);
}

#[test]
fn test_analysis_report_debug() {
    let report = AnalysisReport {
        total_allocations: 0,
        total_deallocations: 0,
        active_allocations: 0,
        peak_memory_bytes: 0,
        current_memory_bytes: 0,
        allocation_rate_per_sec: 0.0,
        deallocation_rate_per_sec: 0.0,
        hotspots: vec![],
        system_snapshots: vec![],
    };

    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("AnalysisReport"));
}

#[test]
fn test_allocation_hotspot_creation() {
    let hotspot = AllocationHotspot {
        var_name: "my_vec".to_string(),
        type_name: "Vec<u8>".to_string(),
        total_size: 1024,
        allocation_count: 10,
        location: Some("main.rs:42".to_string()),
    };

    assert_eq!(hotspot.var_name, "my_vec");
    assert_eq!(hotspot.total_size, 1024);
}

#[test]
fn test_allocation_hotspot_clone() {
    let hotspot = AllocationHotspot {
        var_name: "data".to_string(),
        type_name: "String".to_string(),
        total_size: 100,
        allocation_count: 5,
        location: None,
    };

    let cloned = hotspot.clone();
    assert_eq!(cloned.var_name, "data");
}

#[test]
fn test_allocation_hotspot_debug() {
    let hotspot = AllocationHotspot {
        var_name: "test".to_string(),
        type_name: "i32".to_string(),
        total_size: 4,
        allocation_count: 1,
        location: None,
    };

    let debug_str = format!("{:?}", hotspot);
    assert!(debug_str.contains("AllocationHotspot"));
}

#[test]
fn test_tracker_clone() {
    let tracker = Tracker::new();
    let cloned = tracker.clone();

    let report1 = tracker.analyze();
    let report2 = cloned.analyze();

    // Both should have the same underlying data
    assert_eq!(report1.total_allocations, report2.total_allocations);
}

#[test]
fn test_tracker_with_sampling() {
    let tracker = Tracker::new().with_sampling(SamplingConfig::high_performance());
    let data = vec![1, 2, 3];
    tracker.track_as(&data, "data", "test.rs", 1, "test_module");
}

#[test]
fn test_tracker_elapsed() {
    let tracker = Tracker::new();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = tracker.elapsed();
    assert!(elapsed >= std::time::Duration::from_millis(10));
}

#[test]
fn test_tracker_with_system_monitoring() {
    let tracker = Tracker::new().with_system_monitoring();
    let _ = tracker.current_system_snapshot();
}

#[test]
fn test_tracker_track_as_multiple() {
    let tracker = Tracker::new();
    let data = vec![1, 2, 3, 4, 5];

    tracker.track_as(&data, "my_vec", "test.rs", 10, "test_module");
    tracker.track_as(&data, "my_vec", "test.rs", 20, "test_module");

    let report = tracker.analyze();
    let _ = report.total_allocations;
}

#[test]
fn test_sampling_config_custom() {
    let config = SamplingConfig {
        sample_rate: 0.5,
        capture_call_stack: true,
        max_stack_depth: 15,
    };

    assert!((config.sample_rate - 0.5).abs() < 0.001);
    assert!(config.capture_call_stack);
    assert_eq!(config.max_stack_depth, 15);
}

#[test]
fn test_analysis_report_with_hotspots() {
    let report = AnalysisReport {
        total_allocations: 100,
        total_deallocations: 50,
        active_allocations: 50,
        peak_memory_bytes: 1024 * 1024,
        current_memory_bytes: 512 * 1024,
        allocation_rate_per_sec: 10.0,
        deallocation_rate_per_sec: 5.0,
        hotspots: vec![AllocationHotspot {
            var_name: "test".to_string(),
            type_name: "Vec<u8>".to_string(),
            total_size: 1024,
            allocation_count: 10,
            location: Some("test.rs:1".to_string()),
        }],
        system_snapshots: vec![],
    };

    assert_eq!(report.hotspots.len(), 1);
}

#[test]
fn test_tracker_with_sampling_and_monitoring() {
    let tracker = Tracker::new()
        .with_sampling(SamplingConfig::demo())
        .with_system_monitoring();

    let data = vec![1, 2, 3];
    tracker.track_as(&data, "data", "test.rs", 1, "test_module");

    let snapshot = tracker.current_system_snapshot();
    assert!(snapshot.cpu_usage_percent >= 0.0);
}

#[test]
fn test_tracker_events() {
    let tracker = Tracker::new();
    let data = vec![1, 2, 3];
    tracker.track_as(&data, "test_data", "test.rs", 1, "test_module");

    let events = tracker.events();
    assert!(!events.is_empty());
}

#[test]
fn test_tracker_event_store() {
    let tracker = Tracker::new();
    let _store = tracker.event_store();
}

#[test]
fn test_tracker_stats() {
    let tracker = Tracker::new();
    let stats = tracker.stats();

    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.active_allocations, 0);
}

#[test]
fn test_tracker_stats_with_data() {
    let tracker = Tracker::new();
    let data = vec![1u8; 1024];
    tracker.track_as(&data, "buffer", "test.rs", 1, "test_module");

    let stats = tracker.stats();
    assert!(
        stats.total_allocations >= 1,
        "Should have at least one allocation after tracking data"
    );
}

#[test]
fn test_tracker_system_snapshots() {
    let tracker = Tracker::new().with_system_monitoring();
    let snapshots = tracker.system_snapshots();
    assert!(!snapshots.is_empty());
}

#[test]
fn test_tracker_inner() {
    let tracker = Tracker::new();
    let _inner = tracker.inner();
}

#[test]
fn test_tracker_with_auto_export() {
    let tracker = Tracker::new().with_auto_export("/tmp/test_export");
    let data = vec![1, 2, 3];
    tracker.track_as(&data, "test", "test.rs", 1, "test_module");
}

#[test]
fn test_sampling_config_zero_rate() {
    let config = SamplingConfig {
        sample_rate: 0.0,
        capture_call_stack: false,
        max_stack_depth: 0,
    };

    let tracker = Tracker::new().with_sampling(config);
    let data = vec![1, 2, 3];
    tracker.track_as(&data, "test", "test.rs", 1, "test_module");
}

#[test]
fn test_analysis_report_serialization() {
    let report = AnalysisReport {
        total_allocations: 100,
        total_deallocations: 50,
        active_allocations: 50,
        peak_memory_bytes: 1024,
        current_memory_bytes: 512,
        allocation_rate_per_sec: 10.0,
        deallocation_rate_per_sec: 5.0,
        hotspots: vec![],
        system_snapshots: vec![],
    };

    let json = serde_json::to_string(&report);
    assert!(json.is_ok());
}

#[test]
fn test_allocation_hotspot_serialization() {
    let hotspot = AllocationHotspot {
        var_name: "test".to_string(),
        type_name: "Vec<u8>".to_string(),
        total_size: 1024,
        allocation_count: 10,
        location: Some("test.rs:1".to_string()),
    };

    let json = serde_json::to_string(&hotspot);
    assert!(json.is_ok());
}

#[test]
fn test_tracker_multiple_system_snapshots() {
    let tracker = Tracker::new().with_system_monitoring();
    std::thread::sleep(std::time::Duration::from_millis(10));
    tracker.current_system_snapshot();

    let snapshots = tracker.system_snapshots();
    assert!(
        !snapshots.is_empty(),
        "Should have at least one system snapshot"
    );
}

#[test]
fn test_tracker_analyze_with_hotspots() {
    let tracker = Tracker::new();
    let data1 = vec![1u8; 100];
    let data2 = vec![2u8; 200];
    let data3 = vec![3u8; 300];

    tracker.track_as(&data1, "buffer1", "test.rs", 1, "test_module");
    tracker.track_as(&data2, "buffer2", "test.rs", 2, "test_module");
    tracker.track_as(&data3, "buffer3", "test.rs", 3, "test_module");

    let report = tracker.analyze();
    assert!(
        report.total_allocations >= 3,
        "Should have at least 3 allocations after tracking"
    );
}

#[test]
fn test_tracker_default() {
    let tracker = Tracker::default();
    let report = tracker.analyze();
    assert_eq!(report.total_allocations, 0);
}

/// Objective: Verify the dedupe index does not retain one slot per address ever
/// allocated: releasing a block must reclaim its slot (the profiler must not grow
/// without bound in a long-running process).
/// Invariants: after N allocate/release cycles on distinct addresses the index is
/// empty again, while live addresses keep their slot.
#[test]
fn released_addresses_do_not_retain_dedupe_slots() {
    let tracker = Tracker::new();
    let base = 0x1000_0000usize;

    for step in 0..256usize {
        let ptr = base + step * 64;
        tracker
            .inner
            .track_allocation(ptr, 64)
            .expect("registering the block must succeed");
        assert!(
            tracker
                .track_deallocation(ptr)
                .expect("releasing the block must succeed"),
            "the block must have been tracked before it is released"
        );
    }

    assert_eq!(
        tracker.recorded_bindings.len(),
        0,
        "released addresses must not keep a dedupe slot"
    );

    // A slot that still describes a live binding must survive the reclamation sweep.
    let live = base + 4096;
    tracker
        .inner
        .track_allocation(live, 32)
        .expect("registering the live block must succeed");
    let slot = tracker.recorded_slot(live);
    *slot.lock().expect("slot lock must be acquirable") = Some(RecordedBinding {
        size: 32,
        serial: tracker.inner.binding_serial(live).unwrap_or_default(),
        typed: true,
    });
    drop(slot);
    tracker.reclaim_slot(live);
    assert_eq!(
        tracker.recorded_bindings.len(),
        1,
        "a live address must keep its slot"
    );
}
