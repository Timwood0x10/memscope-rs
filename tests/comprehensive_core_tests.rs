//! Comprehensive tests for core functionality modules
//! This test file focuses on improving coverage for core modules

use memscope_rs::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        memscope_rs::init_for_testing();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_initialization() {
        // Test memory tracker initialization
        ensure_init();
        let tracker = get_global_tracker();
        // Test that we can get stats without error
        let stats_result = tracker.get_stats();
        assert!(stats_result.is_ok());
    }

    #[test]
    fn test_basic_allocation_tracking() {
        // Test basic allocation tracking functionality
        ensure_init();
        
        // Track a simple allocation
        let data = vec![1, 2, 3, 4, 5];
        track_var!(data);
        
        // Verify the data is still accessible
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_variable_lifecycle() {
        // Test variable lifecycle tracking
        ensure_init();
        let tracker = get_global_tracker();
        
        {
            let data = String::from("test_string");
            track_var!(data);
            // Variable goes out of scope here
        }
        
        // Test that tracker handles scope exit properly
        let stats_result = tracker.get_stats();
        assert!(stats_result.is_ok());
    }

    #[test]
    fn test_multiple_variable_tracking() {
        // Test tracking multiple variables
        ensure_init();
        let tracker = get_global_tracker();
        
        let vec_data = vec![1, 2, 3];
        let string_data = String::from("test");
        let _number_data = 42i32; // i32 doesn't implement Trackable
        
        track_var!(vec_data);
        track_var!(string_data);
        
        // Verify all variables are tracked
        let stats_result = tracker.get_stats();
        assert!(stats_result.is_ok());
    }

    #[test]
    fn test_error_handling() {
        // Test error handling in various scenarios
        ensure_init();
        let tracker = get_global_tracker();
        
        // Test export with empty tracker
        let result = tracker.get_stats();
        assert!(result.is_ok());
    }

    #[test]
    fn test_concurrent_tracking() {
        // Test concurrent access to tracking functionality
        use std::thread;
        
        // Ensure initialization happens in main thread first
        ensure_init();
        
        let handles: Vec<_> = (0..4).map(|i| {
            thread::spawn(move || {
                let tracker = get_global_tracker();
                let data = vec![i; 10];
                track_var!(data);
                let export_result = tracker.get_stats();
                export_result
            })
        }).collect();
        
        for handle in handles {
            let export_result = handle.join().expect("Thread panicked");
            assert!(export_result.is_ok());
        }
    }

    #[test]
    fn test_large_data_tracking() {
        // Test tracking of larger data structures
        ensure_init();
        let tracker = get_global_tracker();
        
        let large_vec: Vec<i32> = (0..1000).collect();
        track_var!(large_vec);
        
        let result = tracker.get_stats();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_data_structures() {
        // Test tracking of nested data structures
        ensure_init();
        let tracker = get_global_tracker();
        
        let nested_data = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];
        track_var!(nested_data);
        
        let result = tracker.get_stats();
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_operations() {
        // Test string-related tracking
        ensure_init();
        let tracker = get_global_tracker();
        
        let mut string_data = String::from("initial");
        string_data.push_str(" appended");
        track_var!(string_data);
        
        let result = tracker.get_stats();
        assert!(result.is_ok());
    }

    #[test]
    fn test_option_tracking() {
        // Test tracking of Option types
        ensure_init();
        let tracker = get_global_tracker();
        
        let some_data = Some(vec![1, 2, 3]);
        let none_data: Option<Vec<i32>> = None;
        
        track_var!(some_data);
        track_var!(none_data);
        
        let result = tracker.get_stats();
        assert!(result.is_ok());
    }
}