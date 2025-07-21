//! HTML Export Demo
//!
//! This example demonstrates how to use the HTML export functionality
//! to generate interactive memory analysis reports.

use memscope_rs::export_interactive_html;
use memscope_rs::{get_global_tracker, get_global_unsafe_ffi_tracker, init, track_var};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the memory tracking system
    init();

    println!("🔍 MemScope-RS HTML Export Demo");
    println!("Generating sample memory allocations...");

    // Create various types of allocations to showcase in the HTML report
    create_sample_allocations();

    // Get the global tracker
    let tracker = get_global_tracker();

    // Get the unsafe FFI tracker (may be empty)
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

    // Export interactive HTML report
    let html_path = "memory_analysis_report.html";
    println!("📊 Exporting interactive HTML report to: {html_path}");

    export_interactive_html(&tracker, Some(&unsafe_ffi_tracker), html_path)?;

    let json_path = "demo_json.json";
    tracker.export_to_json(json_path);
    println!("export json ");

    println!("✅ HTML report generated successfully!");
    println!("📂 Open '{html_path}' in your web browser to view the interactive analysis");
    println!("🌐 The report works offline and includes:");
    println!("   • 📊 Memory analysis dashboard");
    println!("   • ⏱️ Lifecycle timeline visualization");
    println!("   • ⚠️ Unsafe/FFI analysis (if applicable)");
    println!("   • 🎮 Interactive memory explorer");

    // Also demonstrate unsafe FFI tracking
    demonstrate_unsafe_operations();

    // Export another report with unsafe data
    let html_path_with_unsafe = "memory_analysis_with_unsafe.html";
    println!("📊 Exporting report with unsafe operations to: {html_path_with_unsafe}");

    export_interactive_html(&tracker, Some(&unsafe_ffi_tracker), html_path_with_unsafe)?;

    println!("✅ Report with unsafe operations generated!");

    Ok(())
}

fn create_sample_allocations() {
    // Create various types of allocations for demonstration

    // 1. Basic collections
    let numbers = vec![1, 2, 3, 4, 5];
    track_var!(numbers);

    let mut large_vec: Vec<u64> = Vec::with_capacity(1000);
    for i in 0..1000 {
        large_vec.push(i);
    }
    track_var!(large_vec);

    // 2. Strings
    let app_name = String::from("MemScope-RS Demo Application");
    track_var!(app_name);

    let mut config_data = String::with_capacity(2048);
    config_data.push_str("This is a large configuration string that demonstrates string allocation tracking in the memory analysis system.");
    track_var!(config_data);

    // 3. Hash maps (note: HashMap doesn't implement Trackable, so we skip tracking)
    let mut user_data: HashMap<String, i32> = HashMap::new();
    user_data.insert("alice".to_string(), 25);
    user_data.insert("bob".to_string(), 30);
    user_data.insert("charlie".to_string(), 35);
    // HashMap doesn't implement Trackable trait, so we don't track it

    // 4. Nested structures
    let mut matrix: Vec<Vec<f64>> = Vec::new();
    for i in 0..10 {
        let mut row = Vec::new();
        for j in 0..10 {
            row.push((i * j) as f64);
        }
        matrix.push(row);
    }
    track_var!(matrix);

    // 5. Box allocations
    let boxed_data = Box::new([0u8; 1024]); // 1KB allocation
    track_var!(boxed_data);

    let huge_boxed_data = Box::new([0u8; 1024 * 1024]); // 1MB allocation
    track_var!(huge_boxed_data);

    // 6. Different sized allocations to show size distribution
    let small_string = String::from("small");
    track_var!(small_string);

    let medium_vec: Vec<u32> = (0..100).collect();
    track_var!(medium_vec);

    let large_buffer: Vec<u8> = vec![0; 10000];
    track_var!(large_buffer);

    println!("✅ Created {} sample allocations", 9);
}

fn demonstrate_unsafe_operations() {
    println!("⚠️ Demonstrating unsafe operations tracking...");

    let ffi_tracker = get_global_unsafe_ffi_tracker();

    // Simulate some FFI allocations using the actual API
    unsafe {
        // Simulate FFI allocation using std::alloc (safer than libc for demo)
        let layout = std::alloc::Layout::from_size_align(1024, 8).unwrap();
        let ptr1 = std::alloc::alloc(layout) as usize;
        if ptr1 != 0 {
            let _ = ffi_tracker.track_ffi_allocation(
                ptr1,
                1024,
                "demo_lib".to_string(),
                "malloc_simulation".to_string(),
            );

            // Simulate proper cleanup
            std::alloc::dealloc(ptr1 as *mut u8, layout);
            let _ = ffi_tracker.track_enhanced_deallocation(ptr1);
        }

        // Simulate another FFI allocation
        let layout2 = std::alloc::Layout::from_size_align(2048, 8).unwrap();
        let ptr2 = std::alloc::alloc(layout2) as usize;
        if ptr2 != 0 {
            let _ = ffi_tracker.track_ffi_allocation(
                ptr2,
                2048,
                "demo_lib".to_string(),
                "large_malloc_simulation".to_string(),
            );

            std::alloc::dealloc(ptr2 as *mut u8, layout2);
            let _ = ffi_tracker.track_enhanced_deallocation(ptr2);
        }

        // Simulate unsafe Rust allocation
        let layout3 = std::alloc::Layout::from_size_align(512, 8).unwrap();
        let ptr3 = std::alloc::alloc(layout3) as usize;
        if ptr3 != 0 {
            let _ = ffi_tracker.track_unsafe_allocation(
                ptr3,
                512,
                "examples/html_export_demo.rs:demonstrate_unsafe_operations".to_string(),
            );

            std::alloc::dealloc(ptr3 as *mut u8, layout3);
            let _ = ffi_tracker.track_enhanced_deallocation(ptr3);
        }
    }

    println!("✅ Unsafe operations demonstration completed");
}
