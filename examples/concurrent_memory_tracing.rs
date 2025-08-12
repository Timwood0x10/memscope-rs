//! 🚀 Data Structure Demo
//! 
//! Demonstrates memory tracking with various data structures

use memscope_rs::{track_var, init};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    println!("🚀 Data Structure Demo Started!");
    println!("{}", "=".repeat(40));
    
    let total_start = Instant::now();
    
    // Test different data structures
    println!("\n📊 Creating various data structures...");
    
    // Vector operations
    let mut vec_data = Vec::new();
    for i in 0..100 {
        vec_data.push(format!("item_{}", i));
    }
    track_var!(vec_data);
    
    // HashMap operations
    let mut map_data = HashMap::new();
    for i in 0..50 {
        map_data.insert(format!("key_{}", i), i * 2);
    }
    track_var!(map_data);
    
    // VecDeque operations
    let mut deque_data = VecDeque::new();
    for i in 0..75 {
        deque_data.push_back(vec![i as u8; 32]);
    }
    track_var!(deque_data);
    
    // Nested structures
    let mut nested_data = Vec::new();
    for i in 0..25 {
        let inner_vec = vec![i; 10];
        track_var!(inner_vec);
        nested_data.push(inner_vec);
    }
    track_var!(nested_data);
    
    let total_time = total_start.elapsed();
    
    println!("\n📊 Data Structure Demo Report");
    println!("{}", "=".repeat(40));
    println!("Total execution time: {:.2}ms", total_time.as_millis());
    println!("Created multiple data structure types");
    
    println!("\n🏁 Data Structure Demo Completed!");
    
    Ok(())
}