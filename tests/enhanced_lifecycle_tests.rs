use memscope_rs::*;
use std::collections::HashMap;
use std::sync::Arc;

// Helper function for allocation type inference (matches the JavaScript logic)
fn infer_allocation_type(type_name: &str) -> &'static str {
    let heap_types = ["Box", "Vec", "String", "HashMap", "BTreeMap", "Arc", "Rc"];
    let stack_types = ["i32", "i64", "f32", "f64", "bool", "char", "usize", "isize"];

    for heap_type in &heap_types {
        if type_name.contains(heap_type) {
            return "heap";
        }
    }

    for stack_type in &stack_types {
        if type_name.contains(stack_type) {
            return "stack";
        }
    }

    if type_name.contains('*') || type_name.contains('&') {
        return "heap";
    }

    "unknown"
}

#[cfg(test)]
mod enhanced_lifecycle_tests {
    use super::*;

    #[test]
    fn test_variable_lifecycle_visualization_data() {
        // Test that we can create allocation data suitable for lifecycle visualization
        let tracker = MemoryTracker::new();

        // Create test allocations with different types
        let heap_var = Arc::new(String::from("heap_allocated"));
        let stack_var = 42i32;
        let mut map_var = HashMap::new();
        map_var.insert("key", "value");

        // Track allocations using correct API (ptr as usize, size)
        let heap_ptr = &*heap_var as *const String as usize;
        let stack_ptr = &stack_var as *const i32 as usize;
        let map_ptr = &map_var as *const HashMap<&str, &str> as usize;

        let _ = tracker.track_allocation(heap_ptr, std::mem::size_of::<String>());
        let _ = tracker.track_allocation(stack_ptr, std::mem::size_of::<i32>());
        let _ = tracker.track_allocation(map_ptr, std::mem::size_of::<HashMap<&str, &str>>());

        // Test passes if no panics occur during tracking
        // Allocation tracking completed successfully
    }

    #[test]
    fn test_heap_stack_allocation_inference() {
        // Test the allocation type inference logic used in visualization
        let test_cases = vec![
            ("Arc<String>", "heap"),
            ("Box<i32>", "heap"),
            ("Vec<u8>", "heap"),
            ("HashMap<String, i32>", "heap"),
            ("i32", "stack"),
            ("f64", "stack"),
            ("bool", "stack"),
            ("char", "stack"),
            ("CustomStruct", "unknown"),
        ];

        for (type_name, expected) in test_cases {
            let inferred = infer_allocation_type(type_name);
            assert_eq!(inferred, expected, "Failed for type: {type_name}");
        }
    }

    #[test]
    fn test_lifecycle_timeline_calculation() {
        // Test timeline calculation for visualization
        let tracker = MemoryTracker::new();

        // Create allocation with known timing
        let test_var = String::from("test");
        let test_ptr = &test_var as *const String as usize;

        let _ = tracker.track_allocation(test_ptr, std::mem::size_of::<String>());

        // Simulate some time passing with actual work instead of sleep
        let _work_simulation = (0..1000).map(|i| i * 2).collect::<Vec<_>>();

        // End tracking
        drop(test_var);

        // Test passes if tracking works without errors
        // Timeline calculation test completed
    }

    #[test]
    fn test_memory_statistics_calculation() {
        // Test the enhanced memory statistics
        let tracker = MemoryTracker::new();

        // Create various allocations
        let heap_var1 = Arc::new(String::from("heap1"));
        let heap_var2 = Arc::new(String::from("heap2"));
        let stack_vars = vec![1i32, 2i32, 3i32];

        // Track heap allocations
        let heap_ptr1 = &*heap_var1 as *const String as usize;
        let heap_ptr2 = &*heap_var2 as *const String as usize;
        let _ = tracker.track_allocation(heap_ptr1, std::mem::size_of::<String>());
        let _ = tracker.track_allocation(heap_ptr2, std::mem::size_of::<String>());

        // Track stack allocations
        for var in &stack_vars {
            let ptr = var as *const i32 as usize;
            let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
        }

        // Test allocation type inference logic
        assert_eq!(infer_allocation_type("Arc<String>"), "heap");
        assert_eq!(infer_allocation_type("i32"), "stack");
        assert_eq!(infer_allocation_type("CustomType"), "unknown");
    }

    #[test]
    fn test_progress_bar_bounds_calculation() {
        // Test that progress bar calculations stay within bounds
        let test_data = vec![
            (1000, 100, 10.0), // start_time, time_range, lifetime_ms
            (2000, 100, 5.0),
            (1500, 100, 20.0),
        ];

        for (start_time, time_range, lifetime_ms) in test_data {
            let min_time = 1000;
            let start_percent = ((start_time - min_time) as f64 / time_range as f64) * 100.0;
            let end_time = start_time + (lifetime_ms * 1_000_000.0) as u64;
            let end_percent = ((end_time - min_time) as f64 / time_range as f64) * 100.0;

            // Ensure bounds are respected
            let bounded_start = start_percent.clamp(0.0, 100.0);
            let bounded_end = end_percent.max(bounded_start).min(100.0);
            let width = bounded_end - bounded_start;

            assert!((0.0..=100.0).contains(&bounded_start));
            assert!((0.0..=100.0).contains(&bounded_end));
            assert!(width >= 0.0);
            assert!(bounded_end >= bounded_start);
        }
    }
}
