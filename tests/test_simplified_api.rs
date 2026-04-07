// Test the simplified global_tracker API
use memscope_rs::capture::backends::global_tracking::{
    global_tracker, init_global_tracking, reset_global_tracking, GlobalTrackerError,
};

#[test]
fn test_simplified_api() -> Result<(), GlobalTrackerError> {
    reset_global_tracking();

    // Initialize global tracking
    init_global_tracking()?;

    // Get the global tracker
    let tracker = global_tracker()?;

    // Track some variables
    let data = vec![1, 2, 3, 4, 5];
    tracker.track(&data);

    let text = String::from("Hello, world!");
    tracker.track(&text);

    // Get statistics
    let stats = tracker.get_stats();
    assert_eq!(stats.total_allocations, 2);

    // Export JSON
    tracker.export_json("/tmp/test_export_json")?;

    // Export HTML
    tracker.export_html("/tmp/test_export_html")?;

    Ok(())
}

#[test]
fn test_init_twice() {
    reset_global_tracking();

    init_global_tracking().unwrap();

    // Second initialization should fail
    let result = init_global_tracking();
    assert!(result.is_err());
}

#[test]
fn test_track_exact() -> Result<(), GlobalTrackerError> {
    reset_global_tracking();

    init_global_tracking()?;

    let tracker = global_tracker()?;

    // Track with exact name
    let important_data = vec![10, 20, 30];
    tracker.track_as(&important_data, "important_data", "test.rs", 42);

    let stats = tracker.get_stats();
    assert_eq!(stats.total_allocations, 1);

    Ok(())
}

#[test]
fn test_export_before_init() {
    reset_global_tracking();

    // Should fail if not initialized
    let result = global_tracker();
    assert!(result.is_err());
}
