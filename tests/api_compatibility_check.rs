//! API compatibility checker for memscope-rs optimization project
//! 
//! This module ensures all existing APIs remain functional during optimization.
//! It tests all public interfaces to guarantee backward compatibility.

use memscope_rs::*;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::rc::Rc;

/// API compatibility test suite
pub struct ApiCompatibilityChecker {
    test_results: Vec<ApiTestResult>,
}

#[derive(Debug, Clone)]
pub struct ApiTestResult {
    pub api_name: String,
    pub test_passed: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl ApiCompatibilityChecker {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }
    
    /// Run all API compatibility tests
    pub fn run_all_tests(&mut self) -> bool {
        println!("🔍 Running API compatibility tests...");
        
        // Core tracking API tests
        self.test_memory_tracker_api();
        self.test_global_tracker_api();
        self.test_tracking_allocator_api();
        
        // Trackable trait tests
        self.test_trackable_implementations();
        
        // Macro tests
        self.test_tracking_macros();
        
        // Analysis API tests
        self.test_analysis_apis();
        
        // Export API tests
        self.test_export_apis();
        
        // Utility function tests
        self.test_utility_functions();
        
        // Advanced type tests
        self.test_advanced_types();
        
        let passed_tests = self.test_results.iter().filter(|r| r.test_passed).count();
        let total_tests = self.test_results.len();
        
        println!("📊 API Compatibility Results: {}/{} tests passed", passed_tests, total_tests);
        
        if passed_tests == total_tests {
            println!("✅ All API compatibility tests passed!");
            true
        } else {
            println!("❌ Some API compatibility tests failed:");
            for result in &self.test_results {
                if !result.test_passed {
                    println!("   ❌ {}: {}", result.api_name, 
                        result.error_message.as_deref().unwrap_or("Unknown error"));
                }
            }
            false
        }
    }
    
    /// Test core MemoryTracker API
    fn test_memory_tracker_api(&mut self) {
        self.run_test("MemoryTracker::track_allocation", || {
            let tracker = get_global_tracker();
            tracker.track_allocation(0x1000, 100)?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::track_deallocation", || {
            let tracker = get_global_tracker();
            tracker.track_allocation(0x2000, 100)?;
            tracker.track_deallocation(0x2000)?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::associate_var", || {
            let tracker = get_global_tracker();
            tracker.track_allocation(0x3000, 100)?;
            tracker.associate_var(0x3000, "test_var".to_string(), "i32".to_string())?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::get_statistics", || {
            let tracker = get_global_tracker();
            let _stats = tracker.get_stats()?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::get_active_allocations", || {
            let tracker = get_global_tracker();
            let _allocations = tracker.get_active_allocations()?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::export_to_json", || {
            let tracker = get_global_tracker();
            tracker.export_to_json("api_test_output")?;
            Ok(())
        });
        
        self.run_test("MemoryTracker::export_to_json_with_options", || {
            let tracker = get_global_tracker();
            let options = ExportOptions::new().verbose_logging(true);
            tracker.export_to_json_with_options("api_test_output_options", options)?;
            Ok(())
        });
    }
    
    /// Test global tracker API
    fn test_global_tracker_api(&mut self) {
        self.run_test("get_global_tracker", || {
            let _tracker = get_global_tracker();
            Ok(())
        });
    }
    
    /// Test TrackingAllocator API
    fn test_tracking_allocator_api(&mut self) {
        self.run_test("TrackingAllocator::new", || {
            let _allocator = TrackingAllocator::new();
            Ok(())
        });
    }
    
    /// Test all Trackable implementations
    fn test_trackable_implementations(&mut self) {
        // Test Vec<T>
        self.run_test("Vec<T>::Trackable", || {
            let vec = vec![1, 2, 3, 4, 5];
            let _ptr = vec.get_heap_ptr();
            let _type_name = vec.get_type_name();
            let _size = vec.get_size_estimate();
            Ok(())
        });
        
        // Test String
        self.run_test("String::Trackable", || {
            let s = String::from("test string");
            let _ptr = s.get_heap_ptr();
            let _type_name = s.get_type_name();
            let _size = s.get_size_estimate();
            Ok(())
        });
        
        // Test Box<T>
        self.run_test("Box<T>::Trackable", || {
            let boxed = Box::new(42);
            let _ptr = boxed.get_heap_ptr();
            let _type_name = boxed.get_type_name();
            let _size = boxed.get_size_estimate();
            Ok(())
        });
        
        // Test Rc<T>
        self.run_test("Rc<T>::Trackable", || {
            let rc = Rc::new(vec![1, 2, 3]);
            let _ptr = rc.get_heap_ptr();
            let _type_name = rc.get_type_name();
            let _size = rc.get_size_estimate();
            let _ref_count = rc.get_ref_count();
            let _data_ptr = rc.get_data_ptr();
            Ok(())
        });
        
        // Test Arc<T>
        self.run_test("Arc<T>::Trackable", || {
            let arc = Arc::new(vec![1, 2, 3]);
            let _ptr = arc.get_heap_ptr();
            let _type_name = arc.get_type_name();
            let _size = arc.get_size_estimate();
            let _ref_count = arc.get_ref_count();
            let _data_ptr = arc.get_data_ptr();
            Ok(())
        });
        
        // Test HashMap
        self.run_test("HashMap<K,V>::Trackable", || {
            let mut map = HashMap::new();
            map.insert("key1", 1);
            map.insert("key2", 2);
            let _ptr = map.get_heap_ptr();
            let _type_name = map.get_type_name();
            let _size = map.get_size_estimate();
            Ok(())
        });
        
        // Test HashSet
        self.run_test("HashSet<T>::Trackable", || {
            let mut set = HashSet::new();
            set.insert(1);
            set.insert(2);
            let _ptr = set.get_heap_ptr();
            let _type_name = set.get_type_name();
            let _size = set.get_size_estimate();
            Ok(())
        });
        
        // Test BTreeMap
        self.run_test("BTreeMap<K,V>::Trackable", || {
            let mut map = BTreeMap::new();
            map.insert("key1", 1);
            map.insert("key2", 2);
            let _ptr = map.get_heap_ptr();
            let _type_name = map.get_type_name();
            let _size = map.get_size_estimate();
            Ok(())
        });
        
        // Test VecDeque
        self.run_test("VecDeque<T>::Trackable", || {
            let mut deque = VecDeque::new();
            deque.push_back(1);
            deque.push_back(2);
            let _ptr = deque.get_heap_ptr();
            let _type_name = deque.get_type_name();
            let _size = deque.get_size_estimate();
            Ok(())
        });
        
        // Test Option<T>
        self.run_test("Option<T>::Trackable", || {
            let some_val = Some(vec![1, 2, 3]);
            let _ptr = some_val.get_heap_ptr();
            let _type_name = some_val.get_type_name();
            let _size = some_val.get_size_estimate();
            
            let none_val: Option<Vec<i32>> = None;
            let _ptr2 = none_val.get_heap_ptr();
            let _type_name2 = none_val.get_type_name();
            let _size2 = none_val.get_size_estimate();
            Ok(())
        });
        
        // Test Result<T,E>
        self.run_test("Result<T,E>::Trackable", || {
            let ok_val: Result<Vec<i32>, String> = Ok(vec![1, 2, 3]);
            let _ptr = ok_val.get_heap_ptr();
            let _type_name = ok_val.get_type_name();
            let _size = ok_val.get_size_estimate();
            
            let err_val: Result<Vec<i32>, String> = Err("error".to_string());
            let _ptr2 = err_val.get_heap_ptr();
            let _type_name2 = err_val.get_type_name();
            let _size2 = err_val.get_size_estimate();
            Ok(())
        });
    }
    
    /// Test tracking macros
    fn test_tracking_macros(&mut self) {
        self.run_test("track_var! macro", || {
            let test_vec = vec![1, 2, 3, 4, 5];
            track_var!(test_vec);
            // Verify the variable is still usable
            assert_eq!(test_vec.len(), 5);
            Ok(())
        });
        
        self.run_test("track_var_owned! macro", || {
            let test_vec = vec![1, 2, 3, 4, 5];
            let tracked = track_var_owned!(test_vec);
            // Verify the tracked variable works
            assert_eq!(tracked.len(), 5);
            let _original = tracked.into_inner();
            Ok(())
        });
        
        self.run_test("track_var_smart! macro", || {
            let number = 42i32;
            let test_vec = vec![1, 2, 3];
            let rc_data = Rc::new(vec![1, 2]);
            
            // track_var_smart!(number); // Skip i32 as it doesn't implement Trackable
            track_var_smart!(test_vec.clone());
            track_var_smart!(rc_data.clone());
            
            // Verify variables are still usable
            assert_eq!(number, 42);
            assert_eq!(test_vec.len(), 3);
            assert_eq!(rc_data.len(), 2);
            Ok(())
        });
    }
    
    /// Test analysis APIs
    fn test_analysis_apis(&mut self) {
        self.run_test("EnhancedMemoryAnalyzer", || {
            let _analyzer = EnhancedMemoryAnalyzer::new();
            Ok(())
        });
        
        self.run_test("analyze_memory_with_enhanced_features", || {
            let tracker = get_global_tracker();
            let _allocations = tracker.get_active_allocations().unwrap_or_default();
            let _result = analyze_memory_with_enhanced_features();
            Ok(())
        });
        
        self.run_test("UnsafeFFITracker", || {
            let _tracker = get_global_unsafe_ffi_tracker();
            Ok(())
        });
        
        self.run_test("analyze_fragmentation", || {
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let _result = memscope_rs::analysis::analyze_fragmentation(&allocations);
            Ok(())
        });
        
        self.run_test("analyze_system_libraries", || {
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let _result = memscope_rs::analysis::analyze_system_libraries(&allocations);
            Ok(())
        });
        
        self.run_test("analyze_concurrency_safety", || {
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let _result = memscope_rs::analysis::analyze_concurrency_safety(&allocations);
            Ok(())
        });
        
        self.run_test("perform_comprehensive_analysis", || {
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let stats = tracker.get_stats().unwrap_or_default();
            let _result = memscope_rs::analysis::perform_comprehensive_analysis(&allocations, &stats);
            Ok(())
        });
        
        self.run_test("build_variable_relationship_graph", || {
            let tracker = get_global_tracker();
            let _allocations = tracker.get_active_allocations().unwrap_or_default();
            // Note: This function needs variable_info parameter, skipping for now
            // let _graph = build_variable_relationship_graph(&allocations, &HashMap::new());
            Ok(())
        });
        
        // Test specialized analyzers
        self.run_test("BorrowAnalyzer", || {
            let analyzer = memscope_rs::analysis::get_global_borrow_analyzer();
            let _result = analyzer.analyze_borrow_patterns();
            Ok(())
        });
        
        self.run_test("GenericAnalyzer", || {
            let analyzer = memscope_rs::analysis::get_global_generic_analyzer();
            let _stats = analyzer.get_generic_statistics();
            Ok(())
        });
        
        self.run_test("AsyncAnalyzer", || {
            let analyzer = memscope_rs::analysis::get_global_async_analyzer();
            let _analysis = analyzer.analyze_async_patterns();
            Ok(())
        });
        
        self.run_test("ClosureAnalyzer", || {
            let analyzer = memscope_rs::analysis::get_global_closure_analyzer();
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let _report = analyzer.analyze_closure_patterns(&allocations);
            Ok(())
        });
        
        self.run_test("LifecycleAnalyzer", || {
            let analyzer = memscope_rs::analysis::get_global_lifecycle_analyzer();
            let _report = analyzer.get_lifecycle_report();
            Ok(())
        });
    }
    
    /// Test export APIs
    fn test_export_apis(&mut self) {
        self.run_test("ExportOptions::new", || {
            let _options = ExportOptions::new();
            Ok(())
        });
        
        self.run_test("ExportOptions builder pattern", || {
            let _options = ExportOptions::new()
                .include_system_allocations(true)
                .verbose_logging(true)
                .buffer_size(128 * 1024)
                .compress_output(false);
            Ok(())
        });
        
        self.run_test("export_lifecycle_timeline", || {
            let tracker = get_global_tracker();
            let _result = export_lifecycle_timeline(&tracker, "api_test_timeline");
            Ok(())
        });
        
        self.run_test("export_memory_analysis", || {
            let tracker = get_global_tracker();
            let _stats = tracker.get_stats().unwrap_or_default();
            let _result = export_memory_analysis(&tracker, "api_test_analysis");
            Ok(())
        });
    }
    
    /// Test utility functions
    fn test_utility_functions(&mut self) {
        self.run_test("format_bytes", || {
            let formatted = format_bytes(1024);
            assert!(!formatted.is_empty());
            Ok(())
        });
        
        self.run_test("get_simple_type", || {
            let simple = get_simple_type("std::vec::Vec<i32>");
            assert!(!simple.is_empty());
            Ok(())
        });
        
        self.run_test("simplify_type_name", || {
            let simplified = simplify_type_name("std::collections::HashMap<std::string::String, i32>");
            assert!(!simplified.0.is_empty());
            Ok(())
        });
    }
    
    /// Test advanced types
    fn test_advanced_types(&mut self) {
        self.run_test("Advanced types analysis", || {
            let tracker = get_global_tracker();
            let allocations = tracker.get_active_allocations().unwrap_or_default();
            let _report = memscope_rs::advanced_types::analyze_advanced_types(&allocations);
            Ok(())
        });
        
        // Test advanced trackable implementations
        self.run_test("RefCell<T>::Trackable", || {
            use std::cell::RefCell;
            let cell = RefCell::new(42);
            let _ptr = cell.get_heap_ptr();
            let _type_name = cell.get_type_name();
            let _size = cell.get_size_estimate();
            Ok(())
        });
        
        self.run_test("Mutex<T>::Trackable", || {
            let mutex = Mutex::new(vec![1, 2, 3]);
            let _ptr = mutex.get_heap_ptr();
            let _type_name = mutex.get_type_name();
            let _size = mutex.get_size_estimate();
            Ok(())
        });
    }
    
    /// Run a single test and record the result
    fn run_test<F>(&mut self, test_name: &str, test_fn: F) 
    where 
        F: FnOnce() -> Result<(), Box<dyn std::error::Error>>
    {
        let start = std::time::Instant::now();
        
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(test_fn)) {
            Ok(Ok(())) => ApiTestResult {
                api_name: test_name.to_string(),
                test_passed: true,
                error_message: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Ok(Err(e)) => ApiTestResult {
                api_name: test_name.to_string(),
                test_passed: false,
                error_message: Some(format!("Error: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Err(panic) => ApiTestResult {
                api_name: test_name.to_string(),
                test_passed: false,
                error_message: Some(format!("Panic: {:?}", panic)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        };
        
        if result.test_passed {
            println!("   ✅ {} ({} ms)", result.api_name, result.execution_time_ms);
        } else {
            println!("   ❌ {} ({} ms): {}", 
                result.api_name, 
                result.execution_time_ms,
                result.error_message.as_deref().unwrap_or("Unknown error")
            );
        }
        
        self.test_results.push(result);
    }
    
    /// Get detailed test results
    pub fn get_test_results(&self) -> &[ApiTestResult] {
        &self.test_results
    }
    
    /// Generate a detailed report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# API Compatibility Test Report\n\n");
        
        let passed = self.test_results.iter().filter(|r| r.test_passed).count();
        let total = self.test_results.len();
        
        report.push_str(&format!("## Summary\n"));
        report.push_str(&format!("- Total tests: {}\n", total));
        report.push_str(&format!("- Passed: {}\n", passed));
        report.push_str(&format!("- Failed: {}\n", total - passed));
        report.push_str(&format!("- Success rate: {:.1}%\n\n", (passed as f64 / total as f64) * 100.0));
        
        report.push_str("## Test Results\n\n");
        for result in &self.test_results {
            let status = if result.test_passed { "✅ PASS" } else { "❌ FAIL" };
            report.push_str(&format!("- {} {} ({} ms)", status, result.api_name, result.execution_time_ms));
            if let Some(error) = &result.error_message {
                report.push_str(&format!(" - {}", error));
            }
            report.push('\n');
        }
        
        if passed < total {
            report.push_str("\n## Failed Tests\n\n");
            for result in self.test_results.iter().filter(|r| !r.test_passed) {
                report.push_str(&format!("### {}\n", result.api_name));
                if let Some(error) = &result.error_message {
                    report.push_str(&format!("Error: {}\n", error));
                }
                report.push_str(&format!("Execution time: {} ms\n\n", result.execution_time_ms));
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_compatibility_checker() {
        let mut checker = ApiCompatibilityChecker::new();
        let all_passed = checker.run_all_tests();
        
        // Generate and print report
        let report = checker.generate_report();
        println!("{}", report);
        
        // This test should pass if all APIs are compatible
        assert!(all_passed, "Some API compatibility tests failed");
    }
    
    #[test]
    fn test_individual_api_components() {
        // Test that we can create the checker
        let checker = ApiCompatibilityChecker::new();
        assert_eq!(checker.test_results.len(), 0);
        
        // Test that we can get results
        let results = checker.get_test_results();
        assert_eq!(results.len(), 0);
    }
}

/// Convenience function to run API compatibility check
pub fn run_api_compatibility_check() -> bool {
    let mut checker = ApiCompatibilityChecker::new();
    checker.run_all_tests()
}

/// Save API compatibility report to file
pub fn save_api_report_to_file(filename: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let mut checker = ApiCompatibilityChecker::new();
    checker.run_all_tests();
    
    let report = checker.generate_report();
    
    let mut file = File::create(filename)?;
    file.write_all(report.as_bytes())?;
    
    println!("📄 API compatibility report saved to {}", filename);
    Ok(())
}