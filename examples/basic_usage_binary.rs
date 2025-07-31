//! Basic usage example for memscope-rs memory visualizer with binary export.

use memscope_rs::{get_global_tracker, init, track_var};
use memscope_rs::export::formats::binary_export::{BinaryExportOptions, export_memory_to_binary};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    // Initialize the memory tracking system
    init();
    println!("memscope-rs initialized. Tracking memory allocations...");

    // Allocate and track simple types
    println!("\nAllocating and tracking variables...");

    let numbers_vec = vec![1, 2, 3, 4, 5];
    track_var!(numbers_vec);
    println!("Tracked 'numbers_vec'");

    let text_string = String::from("Hello, Trace Tools!");
    track_var!(text_string);
    println!("Tracked 'text_string'");

    let boxed_value = Box::new(100i32);
    track_var!(boxed_value);
    println!("Tracked 'boxed_value'");

    let boxed_value2 = Box::new(200i32);
    track_var!(boxed_value2);
    println!("Tracked 'boxed_value2'");

    // Track reference-counted types
    let rc_data = Rc::new(vec![10, 20, 30]);
    track_var!(rc_data);
    println!("Tracked 'rc_data'");

    let arc_data = Arc::new(String::from("Shared data"));
    track_var!(arc_data);
    println!("Tracked 'arc_data'");

    // Clone Rc to show shared ownership
    let rc_data_clone = Rc::clone(&rc_data);
    track_var!(rc_data_clone);
    println!("Tracked 'rc_data_clone' (shares allocation with 'rc_data')");

    // Perform some operations (variables remain fully usable)
    let sum_of_vec = numbers_vec.iter().sum::<i32>();
    println!("\nSum of 'numbers_vec': {sum_of_vec}");
    println!("Length of 'text_string': {}", text_string.len());
    println!("Value in 'boxed_value': {}", *boxed_value);
    println!("Value in 'boxed_value2': {}", *boxed_value2);
    println!("First element of 'rc_data': {}", rc_data[0]);
    println!("Content of 'arc_data': {}", *arc_data);

    // Get memory statistics
    let tracker = get_global_tracker();
    if let Ok(stats) = tracker.get_stats() {
        println!("\nMemory Statistics:");
        println!("  Active allocations: {}", stats.active_allocations);
        println!("  Active memory: {} bytes", stats.active_memory);
        println!("  Total allocations: {}", stats.total_allocations);
        println!("  Peak memory: {} bytes", stats.peak_memory);
    }

    // Export memory snapshot to binary format (will be saved to MemoryAnalysis/basic_usage/ directory)
    println!("\nExporting memory snapshot to binary format...");
    
    // Create output directory if it doesn't exist
    let output_dir = "MemoryAnalysis/basic_usage";
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {e}");
        return;
    }
    
    let binary_path = format!("{}/basic_usage_snapshot.ms", output_dir);
    
    // Configure binary export options
    let export_options = BinaryExportOptions::balanced(); // Use balanced compression and features
    
    // Export to binary format
    match export_memory_to_binary(&tracker, &binary_path, export_options) {
        Ok(stats) => {
            println!("✅ Successfully exported binary to: {}", binary_path);
            println!("📊 Export Statistics:");
            println!("   - Export time: {:?}", stats.export_time);
            println!("   - File size: {} bytes", stats.file_size);
            println!("   - Original size: {} bytes", stats.original_size);
            println!("   - Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
            println!("   - Allocations exported: {}", stats.allocation_count);
            println!("   - Total memory tracked: {} bytes", stats.total_memory);
        }
        Err(e) => {
            eprintln!("❌ Failed to export binary: {e}");
        }
    }

    // Also demonstrate different export options
    println!("\nDemonstrating different binary export options...");
    
    // Fast export (no compression)
    let fast_path = format!("{}/basic_usage_fast.ms", output_dir);
    let fast_options = BinaryExportOptions::fast();
    
    match export_memory_to_binary(&tracker, &fast_path, fast_options) {
        Ok(stats) => {
            println!("✅ Fast export completed: {}", fast_path);
            println!("   - File size: {} bytes (no compression)", stats.file_size);
            println!("   - Export time: {:?}", stats.export_time);
        }
        Err(e) => {
            eprintln!("❌ Fast export failed: {e}");
        }
    }
    
    // Compact export (maximum compression)
    let compact_path = format!("{}/basic_usage_compact.ms", output_dir);
    let compact_options = BinaryExportOptions::compact();
    
    match export_memory_to_binary(&tracker, &compact_path, compact_options) {
        Ok(stats) => {
            println!("✅ Compact export completed: {}", compact_path);
            println!("   - File size: {} bytes (max compression)", stats.file_size);
            println!("   - Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
            println!("   - Export time: {:?}", stats.export_time);
        }
        Err(e) => {
            eprintln!("❌ Compact export failed: {e}");
        }
    }

    // Provide usage instructions
    println!("\n💡 Usage Instructions:");
    println!("   Convert to JSON: memscope export -i {} -f json -o basic_usage.json", binary_path);
    println!("   Convert to HTML: memscope export -i {} -f html -o basic_usage.html", binary_path);
    println!("   Validate file:   memscope export -i {} --validate-only", binary_path);
    println!("   Stream convert:  memscope export -i {} -f json --streaming -o basic_usage_stream.json", binary_path);

    println!("\nExample finished. Check the binary files in '{}':", output_dir);
    println!("  - basic_usage_snapshot.ms (balanced compression)");
    println!("  - basic_usage_fast.ms (no compression)");
    println!("  - basic_usage_compact.ms (maximum compression)");
    println!("\nUse the 'memscope export' command to convert these binary files to JSON or HTML format.");
}