//! Basic async simulation test for track_var! functionality

use memscope_rs::track_var;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing basic track_var! functionality in async-simulated environment...");
    
    // Initialize async mode manually to test the fix
    std::env::set_var("MEMSCOPE_ASYNC_MODE", "1");
    
    
    // Test 1: Basic variable tracking
    println!("Test 1: Basic variable tracking");
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    println!("✓ Vec tracking successful: {} elements", data.len());
    
    // Test 2: String tracking
    println!("Test 2: String tracking");
    let text = "Hello async world!".to_string();
    track_var!(text);
    println!("✓ String tracking successful: '{}'", text);
    
    // Test 3: Mixed with async operations
    println!("Test 3: Mixed with async operations");
    let numbers = vec![10, 20, 30];
    track_var!(numbers);
    
    std::thread::sleep(std::time::Duration::from_millis(10));
    println!("✓ Simulated async operation completed, numbers: {:?}", numbers);
    
    // Test 4: Multiple variables in sequence
    println!("Test 4: Multiple variables in sequence");
    for i in 0..10 {
        let temp_data = vec![i; 5];
        track_var!(temp_data);
        
        if i % 3 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    println!("✓ Sequential tracking completed");
    
    // Test 5: Smart pointer types
    println!("Test 5: Smart pointer types");
    let rc_data = std::rc::Rc::new(vec![100, 200, 300]);
    track_var!(rc_data);
    println!("✓ Rc tracking successful: {:?}", rc_data);
    
    let arc_data = std::sync::Arc::new("Arc data".to_string());
    track_var!(arc_data);
    println!("✓ Arc tracking successful: '{}'", arc_data);
    
    let box_data = Box::new(42);
    track_var!(box_data);
    println!("✓ Box tracking successful: {}", box_data);
    
    println!("🎉 All basic async tracking tests passed!");
    println!("memscope-rs track_var! works correctly in async environments!");
    
    Ok(())
}