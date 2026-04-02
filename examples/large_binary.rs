//! Large Scale Performance Test - New API
//!
//! This example demonstrates extreme workload testing using the new unified API.

use memscope_rs::analysis::memory_passport_tracker::{
    initialize_global_passport_tracker, PassportTrackerConfig,
};
use memscope_rs::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use memscope_rs::snapshot::MemorySnapshot;
use memscope_rs::{track, tracker};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Large Scale Performance Test - New API");
    println!("========================================\n");

    let total_start = Instant::now();

    // Initialize Memory Passport for FFI tracking
    let config = PassportTrackerConfig {
        detailed_logging: false,
        max_events_per_passport: 1000,
        enable_leak_detection: true,
        enable_validation: false,
        max_passports: 100000,
        track_rust_internal_stack: false,
        user_code_prefixes: vec!["examples/".to_string()],
    };
    let _passport_tracker = initialize_global_passport_tracker(config);

    // Initialize tracker
    let tracker = tracker!();

    // Create large-scale test data
    println!("Creating large-scale test data...");
    let data_creation_start = Instant::now();
    create_large_scale_data(&tracker);
    let data_creation_time = data_creation_start.elapsed();

    // Add unsafe/FFI operations
    simulate_unsafe_ffi_operations();

    println!("Data creation completed in {:.2}ms", data_creation_time.as_secs_f64() * 1000.0);

    // Export using new API
    println!("\nExporting memory snapshot using new API...");
    let export_start = Instant::now();

    let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
    let snapshot = MemorySnapshot::from_allocation_infos(allocations);

    let export_options = ExportJsonOptions::default();
    let output_path = "MemoryAnalysis/large_scale_new_api";

    export_snapshot_to_json(&snapshot, output_path.as_ref(), &export_options)?;

    let export_time = export_start.elapsed();
    let total_time = total_start.elapsed();

    // Performance summary
    println!("\n========================================");
    println!("Performance Summary:");
    println!("  Data Creation: {:.2}ms", data_creation_time.as_secs_f64() * 1000.0);
    println!("  Export Time: {:.2}ms", export_time.as_secs_f64() * 1000.0);
    println!("  Total Runtime: {:.2}ms", total_time.as_secs_f64() * 1000.0);

    // Get analysis report
    let report = tracker.analyze();
    println!("\nMemory Analysis:");
    println!("  Total Allocations: {}", report.total_allocations);
    println!("  Active Allocations: {}", report.active_allocations);
    println!("  Peak Memory: {} bytes ({:.2} MB)",
             report.peak_memory_bytes,
             report.peak_memory_bytes as f64 / 1024.0 / 1024.0);

    // Leak detection
    let leak_result = _passport_tracker.detect_leaks_at_shutdown();
    println!("\nLeak Detection:");
    println!("  Leaks Detected: {}", leak_result.total_leaks);

    println!("\nExport successful to {}/", output_path);

    Ok(())
}

fn create_large_scale_data(tracker: &memscope_rs::tracker::Tracker) {
    // Large vectors
    for i in 0..50 {
        let mut large_vec = Vec::with_capacity(500);
        for j in 0..2000 {
            large_vec.push(format!("Item_{i}_{j}"));
        }
        track!(tracker, large_vec);
    }

    // Large string collections
    for i in 0..30 {
        let mut string_collection = Vec::new();
        for j in 0..500 {
            string_collection.push(format!(
                "String collection item {j} in group {i} with extended content for testing"
            ));
        }
        track!(tracker, string_collection);
    }

    // Large hash maps
    for i in 0..15 {
        let mut large_map = HashMap::new();
        for j in 0..1200 {
            large_map.insert(
                format!("key_with_long_string_{i}_{j}"),
                format!("value_with_even_longer_string_data_{i}_{j}"),
            );
        }
        track!(tracker, large_map);
    }

    // Large byte buffers
    for _i in 0..20 {
        let mut byte_buffer = Vec::with_capacity(5000);
        for j in 0..5000 {
            byte_buffer.push((j % 256) as u8);
        }
        track!(tracker, byte_buffer);
    }

    // Smart pointers
    for i in 0..20 {
        let shared_data = Rc::new(format!("Shared data {i} with reference counting"));
        track!(tracker, shared_data);

        let thread_safe_data = Arc::new(format!("Thread safe data {i}"));
        track!(tracker, thread_safe_data);
    }

    // BTreeMap
    for i in 0..20 {
        let mut nested_btree = BTreeMap::new();
        for j in 0..50 {
            nested_btree.insert(
                format!("btree_key_{i}_{j}"),
                format!("btree_value_{i}_{j}"),
            );
        }
        track!(tracker, nested_btree);
    }

    // VecDeque
    for i in 0..15 {
        let mut queue_data = VecDeque::new();
        for j in 0..100 {
            queue_data.push_back(format!("Queue item {j} in collection {i}"));
        }
        track!(tracker, queue_data);
    }
}

fn simulate_unsafe_ffi_operations() {
    use std::alloc::{alloc, dealloc, Layout};

    println!("Simulating unsafe/FFI operations...");

    // Unsafe allocations
    for i in 0..20 {
        let size = 1024 * (i + 1);
        unsafe {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                std::ptr::write_bytes(ptr, (0x40 + i) as u8, size);
                dealloc(ptr, layout);
            }
        }
    }

    // FFI operations
    extern "C" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
        fn free(ptr: *mut std::ffi::c_void);
    }

    for i in 0..20 {
        let size = 512 * (i + 1);
        let ffi_ptr = unsafe { malloc(size) };

        if !ffi_ptr.is_null() {
            unsafe {
                std::ptr::write_bytes(ffi_ptr as *mut u8, (0x60 + i) as u8, size);
                free(ffi_ptr);
            }
        }
    }
}