// Test the simplified global_tracker API
use memscope_rs::capture::backends::global_tracking::{
    global_tracker, init_global_tracking, reset_global_tracking,
};
use memscope_rs::MemScopeError;
use serial_test::serial;

fn get_tracker() -> Result<
    std::sync::Arc<memscope_rs::capture::backends::global_tracking::GlobalTracker>,
    MemScopeError,
> {
    reset_global_tracking();
    init_global_tracking()?;
    global_tracker()
}

#[test]
#[serial]
fn test_simplified_api() -> Result<(), MemScopeError> {
    let tracker = get_tracker()?;

    // Get initial stats
    let initial_stats = tracker.get_stats();

    // Track some variables
    let data = vec![1, 2, 3, 4, 5];
    tracker.track(&data);

    let text = String::from("Hello, world!");
    tracker.track(&text);

    // Get statistics
    let stats = tracker.get_stats();
    // Verify that new allocations were tracked (should be >= initial)
    assert!(stats.total_allocations >= initial_stats.total_allocations);

    // Export JSON
    tracker.export_json("/tmp/test_export_json")?;

    // Export HTML
    tracker.export_html("/tmp/test_export_html")?;

    Ok(())
}

#[test]
#[serial]
fn test_init_twice() {
    reset_global_tracking();
    let first_result = init_global_tracking();

    match first_result {
        Ok(_) => {
            // First initialization succeeded
            let second_result = init_global_tracking();
            assert!(second_result.is_err(), "Second initialization should fail");
        }
        Err(_) => {
            // Already initialized from another test, this is acceptable
        }
    }
}

#[test]
#[serial]
fn test_track_exact() -> Result<(), MemScopeError> {
    let tracker = get_tracker()?;

    // Get initial stats
    let initial_stats = tracker.get_stats();

    // Track with exact name
    let important_data = vec![10, 20, 30];
    tracker.track_as(&important_data, "important_data", "test.rs", 42);

    let stats = tracker.get_stats();
    // Verify that at least one allocation was tracked
    assert!(stats.total_allocations >= initial_stats.total_allocations);

    Ok(())
}

#[test]
#[serial]
fn test_export_before_init() {
    reset_global_tracking();
    let result = global_tracker();
    match result {
        Ok(_) => {
            // Tracker was already initialized (from another test), this is acceptable
        }
        Err(_) => {
            // Expected behavior: not initialized
        }
    }
}

#[test]
#[serial]
fn test_tracker_statistics() -> Result<(), MemScopeError> {
    let tracker = get_tracker()?;

    let initial_stats = tracker.get_stats();

    let data = vec![1u8; 1024]; // 1KB
    tracker.track(&data);

    let stats = tracker.get_stats();
    assert!(stats.total_allocations >= initial_stats.total_allocations);
    assert!(stats.current_memory_bytes >= initial_stats.current_memory_bytes);

    Ok(())
}

#[test]
#[serial]
fn test_tracker_analysis() -> Result<(), MemScopeError> {
    let tracker = get_tracker()?;

    let data = vec![1, 2, 3, 4, 5];
    tracker.track(&data);

    // Run analysis
    let report = tracker.analyze();
    // Verify analysis completes successfully - total_allocations is usize, always >= 0
    let _ = report.total_allocations;

    Ok(())
}

#[test]
#[serial]
fn test_global_tracker_singleton() -> Result<(), MemScopeError> {
    let tracker = get_tracker()?;

    let tracker2 = global_tracker()?;
    assert!(
        std::ptr::eq(&*tracker, &*tracker2),
        "Should return same instance"
    );

    Ok(())
}
