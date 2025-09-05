//! large active allocations test program
//!
//! create large active allocations to test the true large file export performance

use memscope_rs::{get_global_tracker, init, track_var};
use std::collections::HashMap;

fn main() {
    tracing::info!("🚀 large active allocations test program");
    tracing::info!("======================");
    tracing::info!("");

    init();
    let _keep_alive = run_large_active_allocations();
}

fn run_large_active_allocations() -> Vec<Box<dyn std::any::Any>> {
    
    let mut keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();

    tracing::info!("📦 create 10,000 active allocations...");

    for i in 0..1000 {
        // Reduced from 10000 to 1000
        // create different types of allocations
        let large_vec = vec![i; 100];
        track_var!(large_vec);
        keep_alive.push(Box::new(large_vec) as Box<dyn std::any::Any>);

        let large_string = format!("Large string with data {i}");
        track_var!(large_string);
        keep_alive.push(Box::new(large_string) as Box<dyn std::any::Any>);

        let mut map = HashMap::new();
        map.insert(format!("key_{i}"), i);
        track_var!(map);
        keep_alive.push(Box::new(map) as Box<dyn std::any::Any>);

        if i % 1000 == 0 {
            tracing::info!("  ✅ created {} groups", i);
        }
    }

    tracing::info!("\n📊 final statistics:");
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        tracing::info!("  • total allocations: {}", stats.total_allocations);
        tracing::info!("  • active allocations: {}", stats.active_allocations);
        tracing::info!(
            "  • released allocations: {}",
            stats.total_allocations - stats.active_allocations
        );
        tracing::info!(
            "  • active rate: {:.1}%",
            stats.active_allocations as f64 / stats.total_allocations as f64 * 100.0
        );
        tracing::info!(
            "  • active memory: {:.2} MB",
            stats.active_memory as f64 / 1024.0 / 1024.0
        );
    }

    tracing::info!("\n🎯 now you can test large file export performance!");
    tracing::info!("advise: use fast export to handle so many active allocations.");

    // keep all allocations alive until program ends
    tracing::info!("📌 keep {} variables alive", keep_alive.len());
    keep_alive
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_allocation_creation_logic() {
        // Test the allocation creation logic without actually running the full function
        let test_index = 42;
        
        // Test vector creation
        let large_vec = vec![test_index; 100];
        assert_eq!(large_vec.len(), 100);
        assert_eq!(large_vec[0], test_index);
        assert_eq!(large_vec[99], test_index);
        
        // Test string creation
        let large_string = format!("Large string with data {test_index}");
        assert!(large_string.contains("42"));
        assert!(large_string.starts_with("Large string"));
        
        // Test hashmap creation
        let mut map = HashMap::new();
        map.insert(format!("key_{test_index}"), test_index);
        assert_eq!(map.get("key_42"), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_keep_alive_vector_functionality() {
        // Test the keep_alive vector functionality without global tracker
        let mut keep_alive: Vec<Box<dyn std::any::Any>> = Vec::new();
        
        // Add some test data
        let test_vec = vec![1, 2, 3];
        let test_string = String::from("test");
        let mut test_map = HashMap::new();
        test_map.insert("key".to_string(), 42);
        
        keep_alive.push(Box::new(test_vec) as Box<dyn std::any::Any>);
        keep_alive.push(Box::new(test_string) as Box<dyn std::any::Any>);
        keep_alive.push(Box::new(test_map) as Box<dyn std::any::Any>);
        
        assert_eq!(keep_alive.len(), 3);
        
        // Test that we can drop the vector
        drop(keep_alive);
    }

    #[test]
    fn test_allocation_loop_bounds() {
        // Test the loop bounds and iteration logic
        let expected_iterations = 1000;
        let mut counter = 0;
        
        for i in 0..expected_iterations {
            counter += 1;
            
            // Test the modulo condition for logging
            if i % 1000 == 0 {
                // This should only trigger once (at i=0) for 1000 iterations
                assert_eq!(i, 0);
            }
        }
        
        assert_eq!(counter, expected_iterations);
    }

    #[test]
    fn test_statistics_calculation_logic() {
        // Test the statistics calculation logic without global tracker
        let total_allocations = 3000u64; // 1000 iterations * 3 allocations each
        let active_allocations = 2500u64;
        let released_allocations = total_allocations - active_allocations;
        
        assert_eq!(released_allocations, 500);
        
        let active_rate = active_allocations as f64 / total_allocations as f64 * 100.0;
        assert!((active_rate - 83.33).abs() < 0.01);
        
        let active_memory_bytes = 1024 * 1024 * 10; // 10 MB
        let active_memory_mb = active_memory_bytes as f64 / 1024.0 / 1024.0;
        assert_eq!(active_memory_mb, 10.0);
    }

    #[test]
    fn test_format_string_generation() {
        // Test format string generation used in the allocation loop
        for i in 0..5 {
            let large_string = format!("Large string with data {i}");
            assert!(large_string.contains(&i.to_string()));
            
            let key = format!("key_{i}");
            assert!(key.starts_with("key_"));
            assert!(key.ends_with(&i.to_string()));
        }
    }

    #[test]
    fn test_memory_size_calculations() {
        // Test memory size calculations for different allocation types
        let vec_size = std::mem::size_of::<Vec<i32>>() + (100 * std::mem::size_of::<i32>());
        assert!(vec_size > 400); // At least 400 bytes for 100 i32s
        
        let string_size = std::mem::size_of::<String>() + "Large string with data 42".len();
        assert!(string_size > 20);
        
        let map_size = std::mem::size_of::<HashMap<String, i32>>();
        assert!(map_size > 0);
    }
}
