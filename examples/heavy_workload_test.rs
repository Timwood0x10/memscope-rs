//! Heavy workload stress test
//! Tests memscope-rs stability under large number of variables

use memscope_rs::{track_var, init_for_testing};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting heavy workload stress test...");
    
    // Initialize test mode to enable fast mode and prevent memory explosion
    init_for_testing();
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    // Test 1: Large number of variables
    println!("📊 Test 1: 20,000 variables in simulated async context");
    test_many_variables()?;
    
    // Test 2: Mixed data types stress test
    println!("📊 Test 2: Mixed data types (20,000 variables)");
    test_mixed_types()?;
    
    println!("✅ All stress tests completed successfully!");
    println!("🎉 memscope-rs is stable under heavy workload!");
    
    Ok(())
}

fn test_many_variables() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..20_000 {
        let vec_data = vec![i; 50];
        track_var!(vec_data);
        
        let string_data = format!("heavy_test_string_{}", i);
        track_var!(string_data);
        
        let hash_data = {
            let mut map = HashMap::new();
            map.insert(i, format!("value_{}", i));
            map
        };
        track_var!(hash_data);
        
        // Progress reporting
        if i % 2000 == 0 {
            println!("  Processed {} variables", i);
        }
    }
    
    println!("  ✅ Created 60,000 variables without issues");
    Ok(())
}

fn test_mixed_types() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..20_000 {
        match i % 6 {
            0 => {
                let data = vec![i; 30];
                track_var!(data);
            }
            1 => {
                let data = format!("mixed_string_{}", i);
                track_var!(data);
            }
            2 => {
                let data = HashMap::from([(i, "value"), (i+1, "another")]);
                track_var!(data);
            }
            3 => {
                let data = Box::new(vec![i; 10]);
                track_var!(data);
            }
            4 => {
                let data = std::rc::Rc::new(format!("rc_data_{}", i));
                track_var!(data);
            }
            5 => {
                let data = std::sync::Arc::new(vec![i; 5]);
                track_var!(data);
            }
            _ => unreachable!()
        }
        
        if i % 4000 == 0 {
            println!("  Mixed types: processed {} variables", i);
        }
    }
    
    println!("  ✅ Mixed data types test completed");
    Ok(())
}