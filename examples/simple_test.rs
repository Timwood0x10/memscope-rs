//! Simple test for the separated export functionality

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::export::export_separated_json_simple;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Simple Separated JSON Export");

    // Initialize tracker
    let tracker = MemoryTracker::new();

    // Export to separated JSON files
    let output_path = Path::new("test_output/simple_analysis");

    match export_separated_json_simple(&tracker, output_path) {
        Ok(result) => {
            println!("✅ Export successful!");
            println!("📁 Files generated:");
            println!(
                "  🔗 Relationships: {}",
                result.variable_relationships_path.display()
            );
            println!("  📊 Memory: {}", result.memory_analysis_path.display());
            println!("  ⏱️ Lifetime: {}", result.lifetime_analysis_path.display());
            println!("  ⚠️ Safety: {}", result.unsafe_ffi_analysis_path.display());
            println!("⚡ Performance:");
            println!(
                "  📈 Allocations processed: {}",
                result.total_allocations_processed
            );
            println!("  ⏱️ Total time: {:?}", result.export_time);
        }
        Err(e) => {
            eprintln!("❌ Export failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
