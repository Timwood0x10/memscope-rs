//! Test fast-loading HTML dashboard

use memscope_rs::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Fast-Loading HTML Dashboard...");
    
    // Initialize tracker
    let tracker = get_global_tracker();
    
    // Create some test allocations with simple tracking
    let test_string = String::from("Fast dashboard test with a longer string to see how it performs");
    tracker.track_allocation(test_string.as_ptr() as usize, test_string.capacity())?;
    
    let test_vec: Vec<i32> = (0..1000).collect();
    tracker.track_allocation(test_vec.as_ptr() as usize, test_vec.capacity() * std::mem::size_of::<i32>())?;
    
    let test_box = Box::new([0u64; 1000]);
    tracker.track_allocation(test_box.as_ptr() as usize, std::mem::size_of_val(&*test_box))?;
    
    // Simulate some async work
    thread::spawn(|| {
        let _worker_data = vec![0u8; 32 * 1024]; // 32KB
        thread::sleep(Duration::from_millis(100));
    }).join().unwrap();
    
    // Create more allocations to test performance
    for i in 0..50 {
        let temp_vec: Vec<u8> = vec![i as u8; 1024];
        tracker.track_allocation(temp_vec.as_ptr() as usize, temp_vec.capacity())?;
        if i % 10 == 0 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    println!("📊 Generating optimized dashboard...");
    let start_time = std::time::Instant::now();
    
    // Export optimized dashboard
    tracker.export_interactive_dashboard("fast_dashboard.html")?;
    
    let generation_time = start_time.elapsed();
    println!("✅ Fast dashboard generated in {:?}", generation_time);
    
    // Also export JSON for comparison
    tracker.export_to_json("fast_dashboard_data.json")?;
    
    // Check file sizes
    let html_size = std::fs::metadata("fast_dashboard.html")?.len();
    let json_size = std::fs::metadata("fast_dashboard_data.json")?.len();
    
    println!("📁 File sizes:");
    println!("   HTML: {:.1} KB", html_size as f64 / 1024.0);
    println!("   JSON: {:.1} KB", json_size as f64 / 1024.0);
    
    println!("🌐 Open fast_dashboard.html in your browser to see the optimized dashboard!");
    println!("⚡ This version should load much faster with lazy loading and optimized data structure.");
    
    // Keep allocations alive
    println!("Test data: {} chars, {} items, {} bytes", 
             test_string.len(), test_vec.len(), std::mem::size_of_val(&*test_box));
    
    Ok(())
}