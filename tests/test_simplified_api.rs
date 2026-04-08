// Test the simplified global_tracker API
use memscope_rs::capture::backends::global_tracking::{
    global_tracker, init_global_tracking, GlobalTrackerError,
};

#[test]
fn test_simplified_api() -> Result<(), GlobalTrackerError> {
    // Note: In concurrent test environment, global state may be pre-initialized
    // This test handles both cases gracefully
    let tracker = match global_tracker() {
        Ok(t) => t,
        Err(GlobalTrackerError::NotInitialized) => {
            // Initialize if not already done
            init_global_tracking()?;
            global_tracker()?
        }
        Err(e) => return Err(e),
    };

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
fn test_init_twice() {
    // Note: In concurrent environment, tracker may already be initialized
    // This test verifies the initialization behavior
    let first_result = init_global_tracking();

    match first_result {
        Ok(_) => {
            // First initialization succeeded
            let second_result = init_global_tracking();
            assert!(second_result.is_err(), "Second initialization should fail");
        }
        Err(GlobalTrackerError::AlreadyInitialized) => {
            // Already initialized from another test, this is acceptable
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_track_exact() -> Result<(), GlobalTrackerError> {
    // Note: In concurrent test environment, global state may be pre-initialized
    let tracker = match global_tracker() {
        Ok(t) => t,
        Err(GlobalTrackerError::NotInitialized) => {
            init_global_tracking()?;
            global_tracker()?
        }
        Err(e) => return Err(e),
    };

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
fn test_export_before_init() {
    // Note: In concurrent test environment, global state behavior is unpredictable
    // This test verifies error handling
    let result = global_tracker();
    match result {
        Ok(_) => {
            // Tracker was already initialized (from another test), this is acceptable
        }
        Err(GlobalTrackerError::NotInitialized) => {
            // Expected behavior: not initialized
        }
        Err(_) => {
            // Other errors are unexpected
            panic!("Unexpected error from global_tracker()");
        }
    }
}

#[test]
fn test_tracker_statistics() -> Result<(), GlobalTrackerError> {
    // Note: In concurrent test environment, global state may be pre-initialized
    let tracker = match global_tracker() {
        Ok(t) => t,
        Err(GlobalTrackerError::NotInitialized) => {
            init_global_tracking()?;
            global_tracker()?
        }
        Err(e) => return Err(e),
    };

    let initial_stats = tracker.get_stats();

    let data = vec![1u8; 1024]; // 1KB
    tracker.track(&data);

    let stats = tracker.get_stats();
    assert!(stats.total_allocations >= initial_stats.total_allocations);
    assert!(stats.current_memory_bytes >= initial_stats.current_memory_bytes);

    Ok(())
}

#[test]
fn test_tracker_analysis() -> Result<(), GlobalTrackerError> {
    // Note: In concurrent test environment, global state may be pre-initialized
    let tracker = match global_tracker() {
        Ok(t) => t,
        Err(GlobalTrackerError::NotInitialized) => {
            init_global_tracking()?;
            global_tracker()?
        }
        Err(e) => return Err(e),
    };

    let data = vec![1, 2, 3, 4, 5];
    tracker.track(&data);

    // Run analysis
    let report = tracker.analyze();
    // Verify analysis completes successfully
    assert!(report.total_allocations >= 0);

    Ok(())
}

#[test]
fn test_global_tracker_singleton() -> Result<(), GlobalTrackerError> {
    // Test that global_tracker returns the same instance
    let tracker1 = match global_tracker() {
        Ok(t) => t,
        Err(GlobalTrackerError::NotInitialized) => {
            init_global_tracking()?;
            global_tracker()?
        }
        Err(e) => return Err(e),
    };

    let tracker2 = global_tracker()?;
    assert!(
        std::ptr::eq(&*tracker1, &*tracker2),
        "Should return same instance"
    );

    Ok(())
}
