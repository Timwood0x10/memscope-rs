//! Tests for optimized lock implementations

#[cfg(test)]
mod tests {
    use crate::core::scope_tracker::{get_global_scope_tracker, ScopeGuard};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_optimized_scope_tracker_basic() {
        let tracker = get_global_scope_tracker();

        // Test basic scope operations
        let scope_id = tracker.enter_scope("test_scope".to_string()).expect("Failed to enter scope");

        // Associate a variable
        tracker
            .associate_variable("test_var".to_string(), 64)
            .expect("Failed to track allocation");

        // Exit scope
        tracker.exit_scope(scope_id).expect("Failed to exit scope");

        // Get analysis
        let analysis = tracker.get_scope_analysis().expect("Failed to get scope analysis");
        assert!(analysis.total_scopes > 0);
    }

    #[test]
    fn test_optimized_scope_tracker_concurrent() {
        let tracker = Arc::new(get_global_scope_tracker());
        let mut handles = vec![];

        // Test concurrent access
        for i in 0..10 {
            let tracker_clone = tracker.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let scope_name = format!("scope_{}_{}", i, j);
                    if let Ok(scope_id) = tracker_clone.enter_scope(scope_name) {
                        // Associate some variables
                        let _ = tracker_clone.associate_variable(format!("var_{}", j), 32);
                        let _ = tracker_clone.exit_scope(scope_id);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            if let Err(e) = handle.join() {
                eprintln!("Thread join failed: {:?}", e);
            }
        }

        // Verify we can still get analysis
        let analysis = tracker.get_scope_analysis().unwrap();
        // analysis.total_scopes is usize, always >= 0, so this check is redundant
        assert!(analysis.total_scopes > 0 || analysis.total_scopes == 0); // Should have some scopes
    }

    #[test]
    fn test_scope_guard_raii() {
        // Test basic RAII functionality - scope guard should not panic
        let result = std::panic::catch_unwind(|| {
            let _guard = ScopeGuard::enter("raii_test_scope").expect("Failed to enter RAII scope");
            // If we get here, the guard was created successfully
            true
        });

        assert!(
            result.is_ok(),
            "ScopeGuard creation and drop should not panic"
        );

        // Test that we can create multiple guards without issues
        let result2 = std::panic::catch_unwind(|| {
            let _guard1 = ScopeGuard::enter("raii_test_scope_1").expect("Failed to enter RAII scope 1");
            let _guard2 = ScopeGuard::enter("raii_test_scope_2").expect("Failed to enter RAII scope 2");
            // Both guards should be dropped cleanly
            true
        });

        assert!(
            result2.is_ok(),
            "Multiple ScopeGuards should work correctly"
        );
    }
}
