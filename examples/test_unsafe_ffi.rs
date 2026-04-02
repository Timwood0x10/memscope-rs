//! Unsafe/FFI Tracking Test with Memory Passport
//!
//! This example tests unsafe Rust and FFI memory tracking with memory passport system.
//! Demonstrates cross-boundary memory lifecycle tracking and leak detection.

use memscope_rs::{track, tracker};
use memscope_rs::analysis::memory_passport_tracker::{
    initialize_global_passport_tracker, PassportTrackerConfig,
};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Unsafe/FFI Tracking Test with Memory Passport");
    println!("═══════════════════════════════════════════════════════\n");

    // Initialize memory passport tracker with custom configuration
    let config = PassportTrackerConfig {
        detailed_logging: true,
        max_events_per_passport: 100,
        enable_leak_detection: true,
        enable_validation: true,
        max_passports: 10000,
        track_rust_internal_stack: false, // Only track user code call stacks
        user_code_prefixes: vec![
            "src/".to_string(),
            "examples/".to_string(),
            "tests/".to_string(),
            "benches/".to_string(),
        ],
    };
    
    let passport_tracker = initialize_global_passport_tracker(config);
    println!("📋 Memory Passport Tracker initialized");
    println!("   • Detailed logging: enabled");
    println!("   • Leak detection: enabled");
    println!("   • Validation: enabled");
    println!("   • Track Rust internal stack: disabled (user code only)");
    println!("   • User code prefixes: src/, examples/, tests/, benches/\n");

    let tracker = tracker!();
    let num_unsafe_allocations = 1_000;
    let num_ffi_allocations = 1_000;
    let num_leaked_allocations = 100; // Intentionally leaked for demonstration

    println!("📊 Configuration:");
    println!("  Mode: Unsafe/FFI Tracking with Memory Passport");
    println!("  Unsafe Allocations: {}", num_unsafe_allocations);
    println!("  FFI Allocations: {}", num_ffi_allocations);
    println!("  Intentionally Leaked: {} (for demonstration)\n", num_leaked_allocations);

    let start_time = Instant::now();

    // Test 1: Normal unsafe Rust allocations (properly managed)
    println!("⚠️  Testing Normal Unsafe Rust Allocations...");
    for i in 0..num_unsafe_allocations {
        let size = 64 + (i % 4033);

        unsafe {
            let layout = Layout::from_size_align(size, 1).unwrap();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                // Create memory passport for this allocation
                // passport_id is used internally by the tracker for lifecycle management
                let _passport_id = passport_tracker.create_passport(
                    ptr as usize,
                    size,
                    "unsafe_rust_allocation".to_string(),
                )?;

                // Use the memory
                let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
                for (j, byte) in slice.iter_mut().enumerate() {
                    *byte = (i * j) as u8;
                }

                // Track with memscope
                let data = Vec::from_raw_parts(ptr as *mut u8, size, size);
                track!(tracker, data);

                // Memory will be freed when data goes out of scope
                // Passport status: FreedByRust
            }
        }
    }

    // Test 2: FFI allocations with proper handover and reclaim
    println!("🌉 Testing FFI Allocations with Handover/Reclaim...");
    for i in 0..num_ffi_allocations {
        let size = 128 + (i % 1024);

        unsafe {
            let layout = Layout::from_size_align(size, 1).unwrap();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                // Create memory passport
                // passport_id is used internally by the tracker for lifecycle management
                let _passport_id = passport_tracker.create_passport(
                    ptr as usize,
                    size,
                    "ffi_handover_test".to_string(),
                )?;

                // Simulate handover to FFI
                passport_tracker.record_handover_to_ffi(
                    ptr as usize,
                    "foreign_function_call".to_string(),
                    format!("ffi_function_{}", i % 10),
                )?;

                // Use the memory (simulating FFI usage)
                let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
                for (j, byte) in slice.iter_mut().enumerate() {
                    *byte = (i + j) as u8;
                }

                // Simulate reclaim by Rust
                passport_tracker.record_reclaimed_by_rust(
                    ptr as usize,
                    "rust_cleanup".to_string(),
                    "ffi_returned_memory".to_string(),
                )?;

                // Track with memscope
                let data = Vec::from_raw_parts(ptr as *mut u8, size, size);
                track!(tracker, data);

                // Memory will be freed when data goes out of scope
                // Passport status: ReclaimedByRust
            }
        }
    }

    // Test 3: Intentionally leaked allocations (for demonstration)
    println!("🚨 Testing Intentionally Leaked Allocations (for demonstration)...");
    let mut leaked_ptrs = Vec::new();
    for i in 0..num_leaked_allocations {
        let size = 256;

        unsafe {
            let layout = Layout::from_size_align(size, 1).unwrap();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                // Create memory passport
                // passport_id is used internally by the tracker for lifecycle management
                let _passport_id = passport_tracker.create_passport(
                    ptr as usize,
                    size,
                    "intentionally_leaked".to_string(),
                )?;

                // Handover to FFI but never reclaim
                passport_tracker.record_handover_to_ffi(
                    ptr as usize,
                    "leaked_ffi_call".to_string(),
                    format!("leaky_function_{}", i),
                )?;

                // Use the memory
                let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
                for (j, byte) in slice.iter_mut().enumerate() {
                    *byte = (i + j) as u8;
                }

                // Store ptr to prevent compiler optimization
                leaked_ptrs.push(ptr);

                // DO NOT FREE - This simulates a memory leak
                // Passport status: InForeignCustody (confirmed leak)
            }
        }
    }

    let duration = start_time.elapsed();

    // Detect leaks at shutdown
    println!("\n🔍 Detecting Memory Leaks at Shutdown...");
    let leak_result = passport_tracker.detect_leaks_at_shutdown();

    println!("\n📊 Leak Detection Results:");
    println!("  Total Passports Created: {}", passport_tracker.get_stats().total_passports_created);
    println!("  Active Passports: {}", passport_tracker.get_stats().active_passports);
    println!("  Leaks Detected: {}", leak_result.total_leaks);
    println!("  Expected Leaks: {}", num_leaked_allocations);
    
    if leak_result.total_leaks > 0 {
        println!("\n🚨 Leaked Memory Details:");
        for (i, detail) in leak_result.leak_details.iter().take(5).enumerate() {
            println!("  {}. Passport ID: {}", i + 1, detail.passport_id);
            println!("     Memory Address: 0x{:x}", detail.memory_address);
            println!("     Size: {} bytes", detail.size_bytes);
            println!("     Last Context: {}", detail.last_context);
            println!("     Lifecycle: {}", detail.lifecycle_summary);
            println!();
        }
        if leak_result.leak_details.len() > 5 {
            println!("  ... and {} more leaks", leak_result.leak_details.len() - 5);
        }
    }

    // Get analysis report
    let report = tracker.analyze();

    println!("\n📊 Memory Analysis Results:");
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Total Allocations: {}", report.total_allocations);
    println!("  Active Allocations: {}", report.active_allocations);
    println!("  Current Memory: {} bytes ({:.2} MB)", 
             report.current_memory_bytes, 
             report.current_memory_bytes as f64 / 1024.0 / 1024.0);
    println!("  Peak Memory: {} bytes ({:.2} MB)", 
             report.peak_memory_bytes, 
             report.peak_memory_bytes as f64 / 1024.0 / 1024.0);

    // Export detailed JSON using tracker.export_json
    println!("\n📄 Exporting detailed JSON...");
    match tracker.export_json("unsafe_ffi_passport_test") {
        Ok(_) => {
            println!("✅ Detailed JSON exported successfully!");
            println!("   📁 Files saved to MemoryAnalysis/unsafe_ffi_passport_test/");
            println!("   📄 memory_analysis.json - with borrow_info, clone_info, ownership_history");
            println!("   📄 lifetime.json - detailed ownership events");
            println!("   📄 unsafe_ffi.json - unsafe operations analysis");
            println!("   📄 variable_relationships.json - variable relationships");
            println!("   📄 type_analysis.json - type analysis");
        }
        Err(e) => {
            eprintln!("❌ JSON export failed: {}", e);
        }
    }

    // Export analysis report
    println!("\n📊 Exporting analysis report...");
    match tracker.export_analysis("unsafe_ffi_passport_analysis") {
        Ok(_) => {
            println!("✅ Analysis report exported successfully!");
            println!("   📁 File saved to MemoryAnalysis/unsafe_ffi_passport_analysis_analysis.json");
            println!("   📊 Contains: total allocations, peak memory, hotspots, system snapshots");
        }
        Err(e) => eprintln!("❌ Analysis report export failed: {}", e),
    }

    // Export passport data
    println!("\n📋 Exporting Memory Passport Data...");
    let passports = passport_tracker.get_all_passports();
    let passport_data = serde_json::to_string_pretty(&serde_json::json!({
        "metadata": {
            "export_timestamp": chrono::Utc::now().to_rfc3339(),
            "total_passports": passports.len(),
            "leaks_detected": leak_result.total_leaks,
        },
        "passports": passports.values().collect::<Vec<_>>(),
        "leak_detection_result": leak_result,
    }))?;

    std::fs::create_dir_all("./MemoryAnalysis/unsafe_ffi_passport_test")?;
    std::fs::write("./MemoryAnalysis/unsafe_ffi_passport_test/memory_passports.json", passport_data)?;
    println!("✅ Memory passport data exported successfully!");
    println!("   📁 File saved to MemoryAnalysis/unsafe_ffi_passport_test/memory_passports.json");
    println!("   📋 Contains: all memory passports, lifecycle events, leak detection results");

    println!("\n🎉 Test completed successfully!");
    println!("═══════════════════════════════════════════════════════\n");

    // Clean up leaked memory (in real code, you'd want to fix the leaks!)
    // For this test, we intentionally leave them leaked to demonstrate detection
    println!("💡 Note: {} leaks were intentionally created for demonstration.", num_leaked_allocations);
    println!("   In production code, these would need to be fixed by:");
    println!("   1. Properly reclaiming memory from FFI");
    println!("   2. Ensuring all handovers have matching reclaims");
    println!("   3. Using RAII patterns for FFI boundary management\n");

    Ok(())
}
