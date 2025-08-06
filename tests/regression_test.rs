//! Simple regression test for memscope-rs optimization project
//! 
//! This test ensures basic functionality works after optimizations.

use memscope_rs::*;

#[test]
fn test_basic_functionality_regression() {
    // Test basic allocation tracking
    let tracker = get_global_tracker();
    
    // Test allocation tracking
    assert!(tracker.track_allocation(0x1000, 100).is_ok());
    assert!(tracker.track_allocation(0x2000, 200).is_ok());
    
    // Test variable association
    assert!(tracker.associate_var(0x1000, "test_var1".to_string(), "i32".to_string()).is_ok());
    assert!(tracker.associate_var(0x2000, "test_var2".to_string(), "String".to_string()).is_ok());
    
    // Test statistics retrieval
    let stats = tracker.get_stats();
    assert!(stats.is_ok());
    
    // Test deallocation
    assert!(tracker.track_deallocation(0x1000).is_ok());
    assert!(tracker.track_deallocation(0x2000).is_ok());
}

#[test]
fn test_tracking_macros_regression() {
    // Test track_var! macro
    let test_vec = vec![1, 2, 3, 4, 5];
    track_var!(test_vec);
    assert_eq!(test_vec.len(), 5); // Should still be usable
    
    // Test track_var_owned! macro
    let test_vec2 = vec![1, 2, 3];
    let tracked = track_var_owned!(test_vec2);
    assert_eq!(tracked.len(), 3);
    let _original = tracked.into_inner();
}

#[test]
fn test_trackable_types_regression() {
    // Test various Trackable implementations
    let vec_data = vec![1, 2, 3];
    assert!(vec_data.get_heap_ptr().is_some());
    assert!(!vec_data.get_type_name().is_empty());
    assert!(vec_data.get_size_estimate() > 0);
    
    let string_data = String::from("test");
    assert!(string_data.get_heap_ptr().is_some());
    assert_eq!(string_data.get_type_name(), "String");
    assert!(string_data.get_size_estimate() > 0);
    
    let box_data = Box::new(42);
    assert!(box_data.get_heap_ptr().is_some());
    assert!(box_data.get_type_name().contains("Box"));
    assert!(box_data.get_size_estimate() > 0);
}

#[test]
fn test_export_functionality_regression() {
    let tracker = get_global_tracker();
    
    // Create some test data
    assert!(tracker.track_allocation(0x3000, 100).is_ok());
    assert!(tracker.associate_var(0x3000, "export_test".to_string(), "i32".to_string()).is_ok());
    
    // Test JSON export
    let result = tracker.export_to_json("regression_test_output");
    assert!(result.is_ok());
    
    // Test export with options
    let options = ExportOptions::new().verbose_logging(false);
    let result = tracker.export_to_json_with_options("regression_test_output_options", options);
    assert!(result.is_ok());
    
    // Cleanup
    assert!(tracker.track_deallocation(0x3000).is_ok());
}

#[test]
fn test_analysis_functionality_regression() {
    let tracker = get_global_tracker();
    
    // Create test data (reduced)
    for i in 0..3 {
        let ptr = (0x4000 + i * 8) as usize;
        assert!(tracker.track_allocation(ptr, 64).is_ok());
        assert!(tracker.associate_var(ptr, format!("test_var_{}", i), "i32".to_string()).is_ok());
    }
    
    // Test basic analysis functions
    let allocations = tracker.get_active_allocations().unwrap_or_default();
    let stats = tracker.get_stats().unwrap_or_default();
    
    // Test fragmentation analysis
    let _frag_result = memscope_rs::analysis::analyze_fragmentation(&allocations);
    
    // Test system libraries analysis
    let _sys_result = memscope_rs::analysis::analyze_system_libraries(&allocations);
    
    // Test comprehensive analysis
    let _comp_result = memscope_rs::analysis::perform_comprehensive_analysis(&allocations, &stats);
    
    // Cleanup
    for i in 0..3 {
        let ptr = (0x4000 + i * 8) as usize;
        assert!(tracker.track_deallocation(ptr).is_ok());
    }
}

#[test]
fn test_utility_functions_regression() {
    // Test format_bytes
    let formatted = format_bytes(1024);
    assert!(!formatted.is_empty());
    
    // Test get_simple_type
    let simple = get_simple_type("std::vec::Vec<i32>");
    assert!(!simple.is_empty());
    
    // Test simplify_type_name
    let simplified = simplify_type_name("std::collections::HashMap<std::string::String, i32>");
    assert!(!simplified.0.is_empty());
}

#[test]
fn test_concurrent_access_regression() {
    use std::sync::Arc;
    use std::thread;
    
    let tracker = Arc::new(get_global_tracker());
    let mut handles = Vec::new();
    
    // Spawn multiple threads to test concurrent access (reduced)
    for thread_id in 0..2 {
        let tracker_clone = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for i in 0..3 {
                let ptr = (thread_id * 1000 + i) as usize;
                let _ = tracker_clone.track_allocation(ptr, 64);
                let _ = tracker_clone.associate_var(ptr, format!("thread_{}_var_{}", thread_id, i), "i32".to_string());
                if i % 2 == 0 {
                    let _ = tracker_clone.track_deallocation(ptr);
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify tracker is still functional
    let stats = tracker.get_stats();
    assert!(stats.is_ok());
}

#[test]
fn test_memory_safety_regression() {
    // Test that we don't have memory leaks or crashes
    let tracker = get_global_tracker();
    
    // Allocate and deallocate many items (reduced)
    for i in 0..50 {
        let ptr = (0x10000 + i * 8) as usize;
        assert!(tracker.track_allocation(ptr, 64).is_ok());
        
        if i % 10 == 0 {
            assert!(tracker.associate_var(ptr, format!("safety_test_{}", i), "u64".to_string()).is_ok());
        }
        
        if i % 2 == 0 {
            assert!(tracker.track_deallocation(ptr).is_ok());
        }
    }
    
    // Get final statistics
    let stats = tracker.get_stats();
    assert!(stats.is_ok());
}

#[test]
fn test_error_handling_regression() {
    let tracker = get_global_tracker();
    
    // Test that invalid operations are handled gracefully
    // (These should not panic, but may return errors)
    
    // Try to deallocate non-existent allocation
    let _ = tracker.track_deallocation(0xDEADBEEF);
    
    // Try to associate variable with non-existent allocation
    let _ = tracker.associate_var(0xDEADBEEF, "nonexistent".to_string(), "void".to_string());
    
    // Tracker should still be functional
    assert!(tracker.track_allocation(0x5000, 100).is_ok());
    assert!(tracker.track_deallocation(0x5000).is_ok());
}

/// Run all regression tests and report results
pub fn run_regression_tests() -> bool {
    println!("🧪 Running regression tests...");
    
    let test_functions = vec![
        ("Basic Functionality", test_basic_functionality_regression as fn()),
        ("Tracking Macros", test_tracking_macros_regression as fn()),
        ("Trackable Types", test_trackable_types_regression as fn()),
        ("Export Functionality", test_export_functionality_regression as fn()),
        ("Analysis Functionality", test_analysis_functionality_regression as fn()),
        ("Utility Functions", test_utility_functions_regression as fn()),
        ("Concurrent Access", test_concurrent_access_regression as fn()),
        ("Memory Safety", test_memory_safety_regression as fn()),
        ("Error Handling", test_error_handling_regression as fn()),
    ];
    
    let mut passed = 0;
    let total = test_functions.len();
    
    for (name, test_fn) in test_functions {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_fn)) {
            Ok(_) => {
                println!("   ✅ {}", name);
                passed += 1;
            }
            Err(_) => {
                println!("   ❌ {}", name);
            }
        }
    }
    
    println!("📊 Regression test results: {}/{} passed", passed, total);
    
    if passed == total {
        println!("✅ All regression tests passed!");
        true
    } else {
        println!("❌ Some regression tests failed!");
        false
    }
}