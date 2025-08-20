//! Simple Binary Test - Check if binary export works at all

use memscope_rs::{get_global_tracker, init, track_var};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the memory tracking system
    init();
    println!("🧪 Simple Binary Export Test");
    println!("============================");

    // Create some tracked variables
    let data = vec![1, 2, 3, 4, 5];
    track_var!(data);
    println!("✓ Created and tracked 'data' vector");

    let text = String::from("Hello, binary export!");
    track_var!(text);
    println!("✓ Created and tracked 'text' string");

    // Get tracker and export
    let tracker = get_global_tracker();
    
    match (tracker.get_active_allocations(), tracker.get_stats()) {
        (Ok(allocations), Ok(stats)) => {
            println!("📊 Found {} allocations to export", allocations.len());
            
            // Print allocation details
            for (i, alloc) in allocations.iter().enumerate() {
                println!("   Allocation {}: ptr=0x{:x}, size={}, type={:?}, var={:?}", 
                        i, alloc.ptr, alloc.size, alloc.type_name, alloc.var_name);
                println!("      borrow_info: {:?}", alloc.borrow_info);
                println!("      clone_info: {:?}", alloc.clone_info);
                println!("      ownership_history_available: {}", alloc.ownership_history_available);
                println!("      lifetime_ms: {:?}", alloc.lifetime_ms);
            }

            // Try binary export
            println!("\n💾 Attempting binary export...");
            match memscope_rs::export::export_user_variables_binary(
                allocations.clone(),
                stats.clone(),
                "simple_binary_test"
            ) {
                Ok(export_stats) => {
                    println!("✅ Binary export successful!");
                    println!("   📊 Processed {} allocations in {}ms",
                            export_stats.allocations_processed, export_stats.processing_time_ms);
                    
                    // Check if file was created
                    let binary_file = "simple_binary_test.memscope";
                    if std::path::Path::new(binary_file).exists() {
                        let file_size = std::fs::metadata(binary_file)?.len();
                        println!("   📁 Binary file created: {} ({} bytes)", binary_file, file_size);
                        
                        if file_size > 40 {
                            println!("   ✅ File has content beyond header");
                        } else {
                            println!("   ⚠️ File seems to only contain header (40 bytes)");
                        }
                    } else {
                        println!("   ❌ Binary file not found");
                    }
                }
                Err(e) => {
                    println!("❌ Binary export failed: {}", e);
                }
            }
        }
        (Err(e), _) => println!("❌ Failed to get allocations: {}", e),
        (_, Err(e)) => println!("❌ Failed to get stats: {}", e),
    }

    Ok(())
}