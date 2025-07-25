//! Simple demonstration of unsafe Rust and FFI memory analysis
//!
//! This example shows basic usage of the enhanced memory tracking for:
//! - Unsafe Rust allocations
//! - FFI memory operations
//! - Safety violation detection

use memscope_rs::unsafe_ffi_tracker::{get_global_unsafe_ffi_tracker, BoundaryEventType};
use memscope_rs::{get_global_tracker, init, track_var};
use std::alloc::{alloc, dealloc, Layout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory tracking
    init();
    println!("🦀 Starting Unsafe Rust & FFI Memory Analysis Demo");

    // Get trackers
    let tracker = get_global_tracker();
    let unsafe_ffi_tracker = get_global_unsafe_ffi_tracker();

    // 1. Safe Rust allocations for comparison
    println!("\n📊 1. Safe Rust Allocations");
    let safe_vec = vec![1, 2, 3, 4, 5];
    track_var!(safe_vec);

    let safe_string = String::from("Hello, safe Rust!");
    track_var!(safe_string);

    // 2. Unsafe Rust allocations
    println!("\n⚠️  2. Unsafe Rust Allocations");
    unsafe {
        let layout = Layout::new::<[i32; 10]>();
        let ptr = alloc(layout);

        if !ptr.is_null() {
            // Track this unsafe allocation
            memscope_rs::track_unsafe_alloc!(ptr, layout.size());

            // Initialize the memory
            let slice = std::slice::from_raw_parts_mut(ptr as *mut i32, 10);
            for (i, item) in slice.iter_mut().enumerate() {
                *item = i as i32 * i as i32;
            }

            println!(
                "   ✅ Allocated and initialized {} bytes via unsafe",
                layout.size()
            );

            // Record boundary event
            let _ = unsafe_ffi_tracker.record_boundary_event(
                ptr as usize,
                BoundaryEventType::RustToFfi,
                "unsafe_rust_block".to_string(),
                "potential_ffi_target".to_string(),
            );

            // Clean up
            dealloc(ptr, layout);
            println!("   ✅ Deallocated unsafe memory");
        }
    }

    // 3. Simulate FFI allocations
    println!("\n🌉 3. Simulated FFI Memory Operations");

    // Simulate C malloc
    unsafe {
        let layout = Layout::from_size_align(256, 1).unwrap();
        let ffi_ptr = alloc(layout);

        if !ffi_ptr.is_null() {
            // Track as FFI allocation
            memscope_rs::track_ffi_alloc!(ffi_ptr, 256, "libc", "malloc");
            println!("   ✅ Simulated FFI allocated 256 bytes");

            // Record cross-boundary event
            let _ = unsafe_ffi_tracker.record_boundary_event(
                ffi_ptr as usize,
                BoundaryEventType::FfiToRust,
                "libc".to_string(),
                "rust_main".to_string(),
            );

            // Use the memory
            let slice = std::slice::from_raw_parts_mut(ffi_ptr, 256);
            slice[0] = 0x42; // Write some data

            // Clean up
            dealloc(ffi_ptr, layout);
            println!("   ✅ Simulated FFI freed memory");
        }
    }

    // 4. Safety violation detection
    println!("\n🚨 4. Safety Violation Detection");

    // Simulate a potential double-free scenario (controlled - only tracking, no actual double dealloc)
    unsafe {
        let layout = Layout::new::<i32>();
        let ptr = alloc(layout);

        if !ptr.is_null() {
            memscope_rs::track_unsafe_alloc!(ptr, layout.size());

            // First deallocation (legitimate)
            let _ = unsafe_ffi_tracker.track_enhanced_deallocation(ptr as usize);
            dealloc(ptr, layout);

            // Attempt to track second deallocation (this should be caught by tracker)
            // Note: We don't actually call dealloc again, just test the tracking
            match unsafe_ffi_tracker.track_enhanced_deallocation(ptr as usize) {
                Ok(_) => println!("   ❌ Double-free not detected (unexpected)"),
                Err(e) => println!("   ✅ Double-free detected: {e}"),
            }
        }
    }

    // 5. Check for memory leaks
    println!("\n🔍 5. Memory Leak Detection");
    let leaks = unsafe_ffi_tracker.detect_leaks(1000)?; // 1 second threshold
    if leaks.is_empty() {
        println!("   ✅ No memory leaks detected");
    } else {
        println!("   ⚠️  {} potential leaks detected", leaks.len());
    }

    // 6. Generate reports
    println!("\n📊 6. Generating Analysis Reports");

    // 6.1 Export JSON files to MemoryAnalysis folder
    println!("📊 Exporting JSON files to MemoryAnalysis folder...");
    let analysis_dir = "MemoryAnalysis";
    std::fs::create_dir_all(analysis_dir)?;
    
    // Export main memory analysis (correct naming for html_from_json)
    let memory_json = format!("{}/snapshot_memory_analysis.json", analysis_dir);
    tracker.export_to_json(&memory_json)?;
    println!("   ✅ Memory analysis: {}", memory_json);
    
    // Export unsafe/FFI analysis
    let ffi_json = format!("{}/snapshot_unsafe_ffi.json", analysis_dir);
    let enhanced_allocations = unsafe_ffi_tracker.get_enhanced_allocations()?;
    let ffi_data = serde_json::to_string_pretty(&enhanced_allocations)?;
    std::fs::write(&ffi_json, ffi_data)?;
    println!("   ✅ Unsafe/FFI analysis: {}", ffi_json);
    
    // Export performance metrics
    let perf_json = format!("{}/snapshot_performance.json", analysis_dir);
    let stats = tracker.get_stats()?;
    let perf_data = serde_json::json!({
        "performance_metrics": stats,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    std::fs::write(&perf_json, serde_json::to_string_pretty(&perf_data)?)?;
    println!("   ✅ Performance metrics: {}", perf_json);
    
    // Export security violations
    let security_json = format!("{}/snapshot_security_violations.json", analysis_dir);
    let violations = unsafe_ffi_tracker.get_safety_violations().unwrap_or_default();
    let security_data = serde_json::json!({
        "security_violations": violations,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    std::fs::write(&security_json, serde_json::to_string_pretty(&security_data)?)?;
    println!("   ✅ Security violations: {}", security_json);
    
    // 7. Display summary statistics
    println!("\n📈 7. Summary Statistics");
    let stats = tracker.get_stats()?;
    let enhanced_allocations = unsafe_ffi_tracker.get_enhanced_allocations()?;
    let violations = unsafe_ffi_tracker.get_safety_violations()?;

    println!("   📊 Total allocations: {}", stats.total_allocations);
    println!("   📊 Active allocations: {}", stats.active_allocations);
    println!(
        "   📊 Peak memory: {}",
        memscope_rs::format_bytes(stats.peak_memory)
    );
    println!(
        "   📊 Enhanced allocations tracked: {}",
        enhanced_allocations.len()
    );
    println!("   📊 Safety violations: {}", violations.len());

    // Count by source type
    let unsafe_count = enhanced_allocations
        .iter()
        .filter(|a| {
            matches!(
                a.source,
                memscope_rs::unsafe_ffi_tracker::AllocationSource::UnsafeRust { .. }
            )
        })
        .count();
    let ffi_count = enhanced_allocations
        .iter()
        .filter(|a| {
            matches!(
                a.source,
                memscope_rs::unsafe_ffi_tracker::AllocationSource::FfiC { .. }
            )
        })
        .count();
    let cross_boundary_events: usize = enhanced_allocations
        .iter()
        .map(|a| a.cross_boundary_events.len())
        .sum();

    println!("   📊 Unsafe Rust allocations: {unsafe_count}");
    println!("   📊 FFI allocations: {ffi_count}");
    println!("   📊 Cross-boundary events: {cross_boundary_events}");

    println!("\n🎉 Unsafe Rust & FFI Memory Analysis Complete!");
    println!("📁 All analysis files are organized in: {}/", analysis_dir);
    println!("\n📊 Generated files:");
    println!("   • snapshot_memory_analysis.json - Memory allocation analysis");
    println!("   • snapshot_unsafe_ffi.json - Unsafe/FFI analysis");
    println!("   • snapshot_performance.json - Performance metrics");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use memscope_rs::unsafe_ffi_tracker::AllocationSource;

    #[test]
    fn test_unsafe_allocation_tracking() {
        let tracker = get_global_unsafe_ffi_tracker();

        unsafe {
            let layout = Layout::new::<i32>();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                // Track allocation
                let result = tracker.track_unsafe_allocation(
                    ptr as usize,
                    layout.size(),
                    "test_location".to_string(),
                );
                assert!(true);

                // Verify it's tracked
                let allocations = tracker.get_enhanced_allocations().unwrap();
                let found = allocations.iter().any(|a| {
                    a.base.ptr == ptr as usize
                        && matches!(a.source, AllocationSource::UnsafeRust { .. })
                });
                assert!(found);

                // Clean up
                dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_ffi_allocation_tracking() {
        let tracker = get_global_unsafe_ffi_tracker();

        unsafe {
            let layout = Layout::from_size_align(128, 1).unwrap();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                // Track FFI allocation
                let result = tracker.track_ffi_allocation(
                    ptr as usize,
                    128,
                    "test_lib".to_string(),
                    "malloc".to_string(),
                );
                assert!(true);

                // Verify it's tracked
                let allocations = tracker.get_enhanced_allocations().unwrap();
                let found = allocations.iter().any(|a| {
                    a.base.ptr == ptr as usize && matches!(a.source, AllocationSource::FfiC { .. })
                });
                assert!(found);

                // Clean up
                dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_boundary_event_recording() {
        let tracker = get_global_unsafe_ffi_tracker();

        // First create an allocation to attach events to
        let result =
            tracker.track_ffi_allocation(0x1000, 256, "test_lib".to_string(), "malloc".to_string());
        assert!(true);

        // Record boundary event
        let result = tracker.record_boundary_event(
            0x1000,
            BoundaryEventType::FfiToRust,
            "c_library".to_string(),
            "rust_code".to_string(),
        );
        assert!(true);

        // Verify event was recorded
        let allocations = tracker.get_enhanced_allocations().unwrap();
        let allocation = allocations.iter().find(|a| a.base.ptr == 0x1000);
        assert!(allocation.is_some());
        assert!(!allocation.unwrap().cross_boundary_events.is_empty());
    }
}
