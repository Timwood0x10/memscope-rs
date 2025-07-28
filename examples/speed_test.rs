use memscope_rs::{get_global_tracker, init, track_var};
use std::time::Instant;

fn main() {
    init();
    
    // Create some test data
    for i in 0..1000 {
        let vec = vec![i; 100];
        track_var!(vec);
        
        let string = format!("Test string {}", i);
        track_var!(string);
    }
    
    let tracker = get_global_tracker();
    
    // Test default export (should be fast)
    println!("Testing default export (fast mode)...");
    let start = Instant::now();
    if let Err(e) = tracker.export_to_json("speed_test_default") {
        println!("Export failed: {}", e);
    } else {
        let duration = start.elapsed();
        println!("✅ Default export completed in: {:?}", duration);
    }
    
    // Test explicit fast mode
    println!("\nTesting explicit fast mode...");
    let start = Instant::now();
    if let Err(e) = tracker.export_to_json("speed_test_fast") {
        println!("Export failed: {}", e);
    } else {
        let duration = start.elapsed();
        println!("✅ Fast mode export completed in: {:?}", duration);
    }
    
    println!("\nSpeed test completed!");
}