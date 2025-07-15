//! Web Dashboard Demo - Complete FFI/Unsafe Memory Analysis with Interactive Web Interface
//! 
//! This example demonstrates the complete flow:
//! 1. Rust program with unsafe/FFI operations
//! 2. Memory tracking and data collection
//! 3. Export to JSON format
//! 4. Web server serving interactive dashboard
//! 5. Real-time data visualization

use memscope_rs::{init, track_var, get_global_tracker};
use memscope_rs::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, BoundaryEventType};
use memscope_rs::web_export::export_web_dashboard_data;
use memscope_rs::web_server::MemoryAnalysisWebServer;
use std::alloc::{alloc, dealloc, Layout};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory tracking
    init();
    println!("🦀 Starting Web Dashboard Demo - FFI/Unsafe Memory Analysis");
    println!("{}", "=".repeat(70));

    // Get trackers
    let tracker = get_global_tracker();
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

    println!("\n📊 Phase 1: Generating Memory Activity");
    
    // 1. Safe Rust allocations for baseline
    println!("   Creating safe Rust allocations...");
    let safe_vec = vec![1, 2, 3, 4, 5];
    track_var!(safe_vec);
    
    let safe_string = String::from("Hello, memscope web dashboard!");
    track_var!(safe_string);
    
    let safe_hashmap = std::collections::HashMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);
    track_var!(safe_hashmap);

    // 2. Unsafe Rust allocations
    println!("   Creating unsafe Rust allocations...");
    unsafe {
        // Track unsafe allocation
        unsafe_ffi_tracker.track_unsafe_allocation(
            0x1000 as usize, // Mock pointer
            1024,
            "examples/web_dashboard_demo.rs:45".to_string(),
            vec![], // Empty call stack for demo
        )?;

        // Simulate manual memory management
        let layout = Layout::new::<[u8; 1024]>();
        let ptr = alloc(layout);
        if !ptr.is_null() {
            // Track this allocation
            unsafe_ffi_tracker.track_unsafe_allocation(
                ptr as usize,
                layout.size(),
                "examples/web_dashboard_demo.rs:55".to_string(),
                vec![], // Empty call stack for demo
            )?;

            // Use the memory briefly
            std::ptr::write_bytes(ptr, 0x42, layout.size());
            
            // Clean up
            dealloc(ptr, layout);
            unsafe_ffi_tracker.track_unsafe_deallocation(ptr as usize)?;
        }
    }

    // 3. Simulate FFI allocations
    println!("   Simulating FFI allocations...");
    // Mock FFI allocation tracking
    unsafe_ffi_tracker.track_ffi_allocation(
        0x2000 as usize, // Mock pointer
        2048,
        "libc".to_string(),
        "malloc".to_string(),
        vec![], // Empty call stack for demo
    )?;

    // 4. Simulate cross-boundary operations
    println!("   Simulating cross-boundary operations...");
    unsafe_ffi_tracker.track_boundary_event(
        0x1000 as usize,
        BoundaryEventType::RustToC,
        "rust_context".to_string(),
        "c_context".to_string(),
        1024,
    )?;

    // 5. Create some complex data structures
    println!("   Creating complex data structures...");
    let boxed_data = Box::new(vec![0u64; 1000]);
    track_var!(boxed_data);
    
    let rc_data = std::rc::Rc::new(String::from("Reference counted data"));
    track_var!(rc_data);
    
    let arc_data = std::sync::Arc::new(vec![1.0f64; 500]);
    track_var!(arc_data);

    println!("\n📈 Phase 2: Exporting Data");
    
    // Export comprehensive data to JSON
    let json_filename = "web_dashboard/data.json";
    export_web_dashboard_data(&tracker, &unsafe_ffi_tracker, json_filename)?;
    println!("   ✓ Exported dashboard data to: {}", json_filename);

    // Also export traditional formats for comparison
    tracker.export_to_json("web_dashboard/memory_snapshot.json")?;
    tracker.export_memory_analysis("web_dashboard/memory_analysis.svg")?;
    println!("   ✓ Exported traditional formats");

    println!("\n🌐 Phase 3: Starting Web Server");
    
    // Start the web server
    let mut web_server = MemoryAnalysisWebServer::new(8080, "web_dashboard".to_string());
    
    println!("   🚀 Starting web server on http://localhost:8080");
    println!("   📊 Dashboard will be available at: http://localhost:8080/");
    println!("   🔗 API endpoints:");
    println!("      GET  /api/data     - Current memory data");
    println!("      GET  /api/refresh  - Refresh data cache");
    println!("      GET  /api/stats    - Memory statistics");
    
    // Start server in a separate thread for demo
    let server_handle = thread::spawn(move || {
        if let Err(e) = web_server.start() {
            eprintln!("Web server error: {}", e);
        }
    });

    println!("\n🔄 Phase 4: Simulating Real-time Updates");
    
    // Simulate ongoing memory activity for real-time updates
    for i in 0..10 {
        thread::sleep(Duration::from_secs(2));
        
        println!("   Update {}: Creating new allocations...", i + 1);
        
        // Create some new allocations
        let dynamic_vec = vec![i; (i + 1) * 100];
        track_var!(dynamic_vec);
        
        let dynamic_string = format!("Dynamic string update #{}", i);
        track_var!(dynamic_string);
        
        // Update the JSON data file
        export_web_dashboard_data(&tracker, &unsafe_ffi_tracker, json_filename)?;
        
        if i == 2 {
            println!("\n   🎯 Try opening the dashboard now:");
            println!("      http://localhost:8080/");
            println!("      The data will update every 2 seconds");
        }
    }

    println!("\n📊 Phase 5: Final Statistics");
    
    // Print final statistics
    let stats = tracker.get_stats()?;
    println!("   Final Memory Statistics:");
    println!("     Active Allocations: {}", stats.active_allocations);
    println!("     Active Memory: {:.2} KB", stats.active_memory as f64 / 1024.0);
    println!("     Peak Memory: {:.2} KB", stats.peak_memory as f64 / 1024.0);
    println!("     Total Allocations: {}", stats.total_allocations);

    let enhanced_allocations = unsafe_ffi_tracker.get_enhanced_allocations()?;
    let violations = unsafe_ffi_tracker.get_safety_violations()?;
    
    println!("   Unsafe/FFI Statistics:");
    println!("     Enhanced Allocations: {}", enhanced_allocations.len());
    println!("     Safety Violations: {}", violations.len());
    
    let unsafe_count = enhanced_allocations.iter()
        .filter(|a| matches!(a.source, memscope_rs::unsafe_ffi_tracker::AllocationSource::UnsafeRust { .. }))
        .count();
    let ffi_count = enhanced_allocations.iter()
        .filter(|a| matches!(a.source, memscope_rs::unsafe_ffi_tracker::AllocationSource::FfiC { .. }))
        .count();
        
    println!("     Unsafe Rust Allocations: {}", unsafe_count);
    println!("     FFI Allocations: {}", ffi_count);

    println!("\n🎉 Demo Complete!");
    println!("{}", "=".repeat(70));
    println!("The web server is still running. You can:");
    println!("  1. Open http://localhost:8080/ to view the dashboard");
    println!("  2. Check the generated files:");
    println!("     - web_dashboard/data.json (Dashboard data)");
    println!("     - web_dashboard/memory_snapshot.json (Traditional format)");
    println!("     - web_dashboard/memory_analysis.svg (SVG visualization)");
    println!("  3. Press Ctrl+C to stop the server");

    // Keep the main thread alive so the server continues running
    server_handle.join().unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_dashboard_data_generation() {
        init();
        
        let tracker = get_global_tracker();
        let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();
        
        // Create some test data
        let test_vec = vec![1, 2, 3];
        track_var!(test_vec).unwrap();
        
        // Test data export
        let result = export_web_dashboard_data(
            &tracker, 
            &unsafe_ffi_tracker, 
            "test_dashboard_data.json"
        );
        
        assert!(result.is_ok());
        
        // Clean up
        let _ = std::fs::remove_file("test_dashboard_data.json");
    }

    #[test]
    fn test_web_server_creation() {
        let web_server = MemoryAnalysisWebServer::new(8081, "web_dashboard".to_string());
        // Just test that we can create the server without errors
        assert_eq!(web_server.port, 8081);
    }
}