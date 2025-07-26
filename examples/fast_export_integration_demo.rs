//! Fast Export Integration Demo
//!
//! This example demonstrates the integration of the fast export coordinator
//! into the existing export system, showing automatic mode selection and
//! performance improvements for large datasets.

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::export::optimized_json_export::{OptimizedExportOptions, OptimizationLevel};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Fast Export Integration Demo");
    println!("===============================");
    
    // Create a memory tracker
    let tracker = MemoryTracker::new();
    
    // Simulate some memory allocations to create test data
    println!("📊 Creating test data...");
    create_test_allocations(&tracker, 8000); // Create enough data to trigger fast export
    
    // Demo 1: Traditional export (for comparison)
    println!("\n1️⃣ Traditional Export (for comparison):");
    let start_time = Instant::now();
    let traditional_options = OptimizedExportOptions::default()
        .fast_export_mode(false)  // Force traditional export
        .auto_fast_export_threshold(None); // Disable auto mode
    
    tracker.export_to_json_with_optimized_options("demo_traditional", traditional_options)?;
    let traditional_time = start_time.elapsed();
    println!("   Traditional export completed in: {:?}", traditional_time);
    
    // Demo 2: Fast export mode (explicit)
    println!("\n2️⃣ Fast Export Mode (explicit):");
    let start_time = Instant::now();
    let fast_options = OptimizedExportOptions::default()
        .fast_export_mode(true)  // Explicitly enable fast export
        .thread_count(Some(4))   // Use 4 threads
        .buffer_size(512 * 1024); // 512KB buffer
    
    tracker.export_to_json_with_optimized_options("demo_fast_explicit", fast_options)?;
    let fast_time = start_time.elapsed();
    println!("   Fast export completed in: {:?}", fast_time);
    
    // Demo 3: Auto mode selection (recommended)
    println!("\n3️⃣ Auto Mode Selection (recommended):");
    let start_time = Instant::now();
    let auto_options = OptimizedExportOptions::default()
        .auto_fast_export_threshold(Some(5000)); // Auto-enable for >5000 allocations
    
    tracker.export_to_json_with_optimized_options("demo_auto", auto_options)?;
    let auto_time = start_time.elapsed();
    println!("   Auto mode export completed in: {:?}", auto_time);
    
    // Demo 4: Convenience method
    println!("\n4️⃣ Convenience Method (export_to_json_fast):");
    let start_time = Instant::now();
    tracker.export_to_json_fast("demo_convenience")?;
    let convenience_time = start_time.elapsed();
    println!("   Convenience method completed in: {:?}", convenience_time);
    
    // Performance comparison
    println!("\n📈 Performance Comparison:");
    println!("   Traditional: {:?}", traditional_time);
    println!("   Fast (explicit): {:?}", fast_time);
    println!("   Auto mode: {:?}", auto_time);
    println!("   Convenience: {:?}", convenience_time);
    
    if fast_time < traditional_time {
        let improvement = traditional_time.as_millis() as f64 / fast_time.as_millis() as f64;
        println!("   🎯 Fast export is {:.2}x faster than traditional!", improvement);
    }
    
    // Demo 5: Backward compatibility test
    println!("\n5️⃣ Backward Compatibility Test:");
    match tracker.test_export_backward_compatibility() {
        Ok(results) => {
            let summary = &results["backward_compatibility_test"]["summary"];
            println!("   Tests passed: {}/{}", 
                summary["passed_tests"], 
                summary["total_tests"]);
            println!("   Success rate: {:.1}%", 
                summary["success_rate"].as_f64().unwrap_or(0.0));
            println!("   Status: {}", 
                results["backward_compatibility_test"]["compatibility_status"]
                    .as_str().unwrap_or("unknown"));
        }
        Err(e) => {
            println!("   ❌ Compatibility test failed: {}", e);
        }
    }
    
    // Demo 6: Show upgrade path
    println!("\n6️⃣ API Upgrade Guide:");
    tracker.show_export_upgrade_path();
    
    println!("\n✅ Fast Export Integration Demo completed!");
    println!("   Check the MemoryAnalysis/ directory for generated files.");
    
    Ok(())
}

/// Create test allocations to demonstrate the export system
fn create_test_allocations(tracker: &MemoryTracker, count: usize) {
    use std::alloc::{alloc, dealloc, Layout};
    
    let mut allocations = Vec::new();
    
    for i in 0..count {
        let size = 64 + (i % 1000); // Variable sizes
        let layout = Layout::from_size_align(size, 8).unwrap();
        
        unsafe {
            let ptr = alloc(layout);
            if !ptr.is_null() {
                // Track the allocation
                let type_name = match i % 5 {
                    0 => Some("Vec<u8>".to_string()),
                    1 => Some("HashMap<String, i32>".to_string()),
                    2 => Some("String".to_string()),
                    3 => Some("CustomStruct".to_string()),
                    _ => Some("Buffer".to_string()),
                };
                
                let var_name = Some(format!("test_var_{}", i));
                let scope_name = Some(format!("test_scope_{}", i % 10));
                
                // Store for cleanup
                allocations.push((ptr, layout));
                
                // Simulate some deallocations
                if i % 3 == 0 && !allocations.is_empty() {
                    let (old_ptr, old_layout) = allocations.remove(0);
                    dealloc(old_ptr, old_layout);
                }
            }
        }
    }
    
    // Cleanup remaining allocations
    for (ptr, layout) in allocations {
        unsafe {
            dealloc(ptr, layout);
        }
    }
    
    println!("   Created {} test allocations", count);
}