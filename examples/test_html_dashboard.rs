use memscope_rs::{get_global_tracker, track_var};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracking system
    memscope_rs::init();
    
    println!("🚀 Testing HTML Dashboard Export");
    
    // Create some test data
    let mut test_vec = Vec::with_capacity(1000);
    track_var!(test_vec);
    
    for i in 0..500 {
        test_vec.push(i);
    }
    
    let test_string = "Hello, Memory Tracking World!".repeat(100);
    track_var!(test_string);
    
    let mut test_map = HashMap::new();
    // Note: HashMap doesn't implement Trackable, so we track individual Vec values
    
    for i in 0..50 {
        let value_vec = vec![i; 10];
        test_map.insert(format!("key_{}", i), value_vec);
    }
    
    let test_box = Box::new([42u64; 1000]);
    track_var!(test_box);
    
    // Get the global tracker
    let tracker = get_global_tracker();
    
    // First export JSON with full data (correct order!)
    println!("📄 Generating JSON report with full data...");
    tracker.export_to_json("dashboard_data.json")?;
    
    // Then generate HTML dashboard based on JSON
    println!("📊 Generating interactive HTML dashboard from JSON...");
    tracker.export_interactive_dashboard("interactive_dashboard.html")?;

    // Print summary
    let stats = tracker.get_stats()?;
    println!("\n✅ Dashboard Export Complete!");
    println!("📈 Memory Statistics:");
    println!("   Total Allocations: {}", stats.total_allocations);
    println!("   Active Memory: {} bytes", stats.active_memory);
    println!("   Peak Memory: {} bytes", stats.peak_memory);
    
    println!("\n🎯 Files Generated:");
    println!("   📱 interactive_dashboard.html - Open in browser for interactive analysis");
    println!("   📊 dashboard_data.json - Raw data for further analysis");
    
    println!("\n🎉 Test completed successfully!");
    println!("   Open 'interactive_dashboard.html' in your browser to see the results!");
    
    Ok(())
}