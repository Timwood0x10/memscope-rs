use memscope_rs::{track_var, get_global_tracker};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Data Availability in Binary ===");
    
    // Create various types of variables to test data capture
    let my_string = "Hello World".to_string();
    track_var!(my_string);
    
    let my_vector = vec![1, 2, 3, 4, 5];
    track_var!(my_vector);
    
    let mut my_map = HashMap::new();
    my_map.insert("key1".to_string(), "value1".to_string());
    my_map.insert("key2".to_string(), "value2".to_string());
    track_var!(my_map);
    
    let my_box = Box::new("boxed data".to_string());
    track_var!(my_box);
    
    // Get tracker and examine the data
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    
    println!("\n=== Examining Captured Data ===");
    println!("Total allocations: {}", stats.allocations.len());
    
    for (i, alloc) in stats.allocations.iter().enumerate() {
        println!("\nAllocation {}:", i + 1);
        println!("  ptr: 0x{:x}", alloc.ptr);
        println!("  size: {} bytes", alloc.size);
        
        // Check if we have var_name
        match &alloc.var_name {
            Some(name) => println!("  var_name: '{}'", name),
            None => println!("  var_name: None (需要推断)"),
        }
        
        // Check if we have type_name
        match &alloc.type_name {
            Some(type_name) => println!("  type_name: '{}'", type_name),
            None => println!("  type_name: None (需要推断)"),
        }
        
        // Check scope_name
        match &alloc.scope_name {
            Some(scope) => println!("  scope_name: '{}'", scope),
            None => println!("  scope_name: None"),
        }
        
        println!("  thread_id: '{}'", alloc.thread_id);
        println!("  timestamp_alloc: {}", alloc.timestamp_alloc);
    }
    
    // Export to binary and test parsing
    println!("\n=== Testing Binary Export/Parse ===");
    tracker.export_full_binary("test_data_availability")?;
    
    // Parse back and check if data is preserved
    use memscope_rs::export::binary::BinaryParser;
    let allocations = BinaryParser::load_allocations("MemoryAnalysis/test_data_availability.memscope")?;
    
    println!("\nAfter binary round-trip:");
    println!("Total allocations loaded: {}", allocations.len());
    
    for (i, alloc) in allocations.iter().enumerate() {
        println!("\nLoaded Allocation {}:", i + 1);
        println!("  ptr: 0x{:x}", alloc.ptr);
        println!("  size: {} bytes", alloc.size);
        
        // Check if data is preserved after binary round-trip
        match &alloc.var_name {
            Some(name) => println!("  var_name: '{}' ✅", name),
            None => println!("  var_name: None ❌ (数据丢失)"),
        }
        
        match &alloc.type_name {
            Some(type_name) => println!("  type_name: '{}' ✅", type_name),
            None => println!("  type_name: None ❌ (数据丢失)"),
        }
    }
    
    // Summary
    let has_var_names = stats.allocations.iter().any(|a| a.var_name.is_some());
    let has_type_names = stats.allocations.iter().any(|a| a.type_name.is_some());
    let loaded_has_var_names = allocations.iter().any(|a| a.var_name.is_some());
    let loaded_has_type_names = allocations.iter().any(|a| a.type_name.is_some());
    
    println!("\n=== Summary ===");
    println!("Original data has var_names: {}", if has_var_names { "✅ YES" } else { "❌ NO" });
    println!("Original data has type_names: {}", if has_type_names { "✅ YES" } else { "❌ NO" });
    println!("Loaded data has var_names: {}", if loaded_has_var_names { "✅ YES" } else { "❌ NO" });
    println!("Loaded data has type_names: {}", if loaded_has_type_names { "✅ YES" } else { "❌ NO" });
    
    if has_var_names && has_type_names {
        println!("\n🎉 结论: 程序确实能获取到var_name和type_name数据！");
        println!("   不需要推断，可以直接使用存储的数据。");
    } else {
        println!("\n⚠️  结论: 程序无法获取完整的var_name和type_name数据");
        println!("   需要推断函数来填补缺失的信息。");
    }
    
    Ok(())
}