use memscope_rs::{track_var, get_global_tracker};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracking system
    memscope_rs::init();
    
    println!("Testing enhanced JSON export...");
    
    // Create some test variables with different types
    let test_string = String::from("Hello, enhanced JSON!");
    track_var!(test_string)?;
    
    let test_vec: Vec<i32> = vec![1, 2, 3, 4, 5];
    track_var!(test_vec)?;
    
    let test_box = Box::new([42u64; 100]);
    track_var!(test_box)?;
    
    let mut test_hashmap: HashMap<String, i32> = HashMap::new();
    test_hashmap.insert("key1".to_string(), 100);
    test_hashmap.insert("key2".to_string(), 200);
    test_hashmap.insert("key3".to_string(), 300);
    let boxed_hashmap = Box::new(test_hashmap);
    track_var!(boxed_hashmap)?;
    
    // Create some nested scope allocations
    {
        let scoped_vec = vec![1.0, 2.0, 3.0, 4.0];
        track_var!(scoped_vec)?;
        
        let scoped_string = String::from("Scoped allocation");
        track_var!(scoped_string)?;
    }
    
    // Get the global tracker and export enhanced JSON
    let tracker = get_global_tracker();
    
    // Export using the new enhanced JSON function
    tracker.export_to_json("enhanced_test_output.json")?;
    
    // Also export regular JSON for comparison
    // tracker.export_to_json("regular_test_output.json")?;
    tracker.export_memory_analysis("testmemory_analysis.svg")?;
    tracker.export_lifecycle_timeline("testlifecycle_timeline.svg")?;
    
    println!("✅ Enhanced JSON export completed!");
    println!("📄 Files generated:");
    println!("   - enhanced_test_output.json (Enhanced format with complete data)");
    println!("   - memory_analysis.svg (Memory analysis visualization)");
    println!("   - lifecycle_timeline.svg (Lifecycle timeline visualization)");
    
    // Print some statistics
    let stats = tracker.get_stats()?;
    println!("\n📊 Memory Statistics:");
    println!("   Total allocations: {}", stats.total_allocations);
    println!("   Active allocations: {}", stats.active_allocations);
    println!("   Total memory: {} bytes", stats.total_allocated);
    println!("   Active memory: {} bytes", stats.active_memory);
    
    Ok(())
}