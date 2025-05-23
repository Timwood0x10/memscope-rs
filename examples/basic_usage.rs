
use trace_tools::tracker::get_global_tracker;
#[allow(warnings)]
fn main() {
    // Get the global memory tracker
    let tracker = get_global_tracker();
    
    // Create some heap-allocated values
    let s = String::from("Hello, world!");
    let mut numbers = Vec::new();
    for i in 0..10 {
        numbers.push(i);
    }
    let boxed = Box::new(42);
    
    // Print some debug info
    println!("String: {}", s);
    println!("Numbers: {:?}", numbers);
    println!("Boxed value: {}", boxed);
    
    // Manually track the allocations (this is a simplified example)
    // In a real implementation, you would use a custom allocator or proc macro
    if let Err(e) = tracker.associate_var(
        s.as_ptr() as *const () as usize,
        "s".to_string(),
        "String".to_string()
    ) {
        eprintln!("Failed to track string: {}", e);
    }
    
    if let Err(e) = tracker.associate_var(
        numbers.as_ptr() as *const () as usize,
        "numbers".to_string(),
        "Vec<i32>".to_string()
    ) {
        eprintln!("Failed to track vector: {}", e);
    }
    
    if let Err(e) = tracker.associate_var(
        Box::<_>::as_ptr(&boxed) as *const () as usize,
        "boxed".to_string(),
        "Box<i32>".to_string()
    ) {
        eprintln!("Failed to track boxed value: {}", e);
    }
    
    // Export the memory snapshot
    if let Err(e) = tracker.export_to_json("memory_snapshot.json") {
        eprintln!("Failed to export to JSON: {}", e);
    } else {
        println!("\nSuccessfully exported memory snapshot to memory_snapshot.json");
    }
    
    if let Err(e) = tracker.export_to_svg("memory_usage.svg") {
        eprintln!("Failed to export to SVG: {}", e);
    } else {
        println!("Successfully exported memory usage visualization to memory_usage.svg");
    }
    
    // Print some stats
    let stats = tracker.get_stats();
    println!("\nMemory Stats:");
    println!("  Total allocations: {}", stats.total_allocations);
    
    // The tracked variables will be automatically dropped here
}
