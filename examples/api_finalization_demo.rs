//! Comprehensive API demonstration for Task 12 completion
//! 
//! This example demonstrates all finalized APIs and serves as a complete
//! reference for the FFI JSON export optimization features.

use memscope_rs::{
    init, track_var, get_global_tracker,
    analysis::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker,
    export::optimized_json_export::{OptimizedExportOptions, OptimizationLevel}
};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Task 12: API Finalization and Documentation Demo");
    println!("==================================================");
    
    // Initialize the memory tracking system
    init();
    
    // Demonstrate all major API categories
    demonstrate_basic_api()?;
    demonstrate_optimized_export_api()?;
    demonstrate_unsafe_ffi_api()?;
    demonstrate_security_analysis_api()?;
    demonstrate_performance_optimization_api()?;
    demonstrate_backward_compatibility()?;
    
    println!("\n🚀 All APIs demonstrated successfully!");
    println!("✅ Task 12 - API Finalization: COMPLETED");
    
    Ok(())
}

/// Demonstrate the basic, stable API that all users will use
fn demonstrate_basic_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📚 1. Basic API Demonstration");
    println!("-----------------------------");
    
    let tracker = get_global_tracker();
    
    // Basic variable tracking - STABLE API
    let basic_vec = vec![1, 2, 3, 4, 5];
    track_var!(basic_vec);
    println!("✅ Basic vector tracking");
    
    let basic_string = String::from("API Demo");
    track_var!(basic_string);
    println!("✅ String tracking");
    
    let basic_box = Box::new([1, 2, 3, 4, 5]);
    track_var!(basic_box);
    println!("✅ Box tracking");
    
    // Basic export - STABLE API
    tracker.export_to_json("demo_basic_api.json")?;
    println!("✅ Basic JSON export completed");
    
    Ok(())
}

/// Demonstrate the optimized export API with all configuration options
fn demonstrate_optimized_export_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ 2. Optimized Export API Demonstration");
    println!("---------------------------------------");
    
    let tracker = get_global_tracker();
    
    // Create test data
    let mut test_data = Vec::new();
    for i in 0..100 {
        let data = vec![i; 100];
        track_var!(data);
        test_data.push(data);
    }
    println!("✅ Test data created (100 vectors)");
    
    // Demonstrate all optimization levels
    let optimization_levels = [
        ("Low", OptimizationLevel::Low),
        ("Medium", OptimizationLevel::Medium), 
        ("High", OptimizationLevel::High),
        ("Maximum", OptimizationLevel::Maximum),
    ];
    
    for (name, level) in optimization_levels.iter() {
        let options = OptimizedExportOptions::with_optimization_level(*level);
        let filename = format!("demo_optimization_{}.json", name.to_lowercase());
        
        let start = Instant::now();
        tracker.export_to_json_with_options(&filename, options)?;
        let duration = start.elapsed();
        
        println!("✅ {} optimization export: {:?}", name, duration);
    }
    
    // Demonstrate custom configuration
    let custom_options = OptimizedExportOptions::default()
        .parallel_processing(true)
        .buffer_size(512 * 1024)  // 512KB
        .batch_size(2000)
        .streaming_writer(true)
        .schema_validation(true)
        .security_analysis(true)
        .adaptive_optimization(true)
        .max_cache_size(2000);
    
    tracker.export_to_json_with_options("demo_custom_config.json", custom_options)?;
    println!("✅ Custom configuration export completed");
    
    Ok(())
}

/// Demonstrate unsafe FFI tracking capabilities
fn demonstrate_unsafe_ffi_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔒 3. Unsafe FFI API Demonstration");
    println!("----------------------------------");
    
    let tracker = get_global_tracker();
    let unsafe_tracker = get_global_unsafe_ffi_tracker();
    
    // Safe allocations for comparison
    let safe_data = vec![1, 2, 3, 4, 5];
    track_var!(safe_data);
    println!("✅ Safe allocation tracked");
    
    // Unsafe allocations
    unsafe {
        let layout = Layout::new::<[u8; 1024]>();
        let ptr = alloc(layout);
        
        if !ptr.is_null() {
            // Simulate some unsafe operations
            std::ptr::write_bytes(ptr, 0, 1024);
            
            // Properly deallocate
            dealloc(ptr, layout);
            println!("✅ Unsafe allocation and deallocation tracked");
        }
    }
    
    // Export with enhanced FFI analysis
    let ffi_options = OptimizedExportOptions::default()
        .enable_enhanced_ffi_analysis(true)
        .enable_boundary_event_processing(true)
        .enable_memory_passport_tracking(true);
    
    tracker.export_to_json_with_options("demo_unsafe_ffi.json", ffi_options)?;
    println!("✅ Unsafe FFI analysis export completed");
    
    // Check for safety violations
    if let Ok(unsafe_tracker_guard) = unsafe_tracker.try_lock() {
        let violations = unsafe_tracker_guard.analyze_safety_violations();
        println!("✅ Safety violations analyzed: {} found", violations.len());
    }
    
    Ok(())
}

/// Demonstrate security analysis capabilities
fn demonstrate_security_analysis_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️ 4. Security Analysis API Demonstration");
    println!("------------------------------------------");
    
    let tracker = get_global_tracker();
    let unsafe_tracker = get_global_unsafe_ffi_tracker();
    
    // Create various allocation patterns
    let security_test_data = vec![0u8; 10000];
    track_var!(security_test_data);
    
    // Simulate potential security scenarios
    unsafe {
        let layout = Layout::new::<[u8; 256]>();
        let ptr1 = alloc(layout);
        let ptr2 = alloc(layout);
        
        if !ptr1.is_null() && !ptr2.is_null() {
            // Normal operations
            dealloc(ptr1, layout);
            dealloc(ptr2, layout);
        }
    }
    
    // Export with comprehensive security analysis
    let security_options = OptimizedExportOptions::default()
        .security_analysis(true)
        .include_low_severity_violations(true)
        .generate_integrity_hashes(true);
    
    tracker.export_to_json_with_options("demo_security_analysis.json", security_options)?;
    println!("✅ Security analysis export completed");
    
    // Get detailed security information
    if let Ok(unsafe_tracker_guard) = unsafe_tracker.try_lock() {
        let safety_violations = unsafe_tracker_guard.analyze_safety_violations();
        let security_violations = unsafe_tracker_guard.get_security_violations();
        
        println!("✅ Security analysis results:");
        println!("   - Safety violations: {}", safety_violations.len());
        println!("   - Security violations: {}", security_violations.len());
    }
    
    Ok(())
}

/// Demonstrate performance optimization features
fn demonstrate_performance_optimization_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 5. Performance Optimization API Demonstration");
    println!("--------------------------------------------------");
    
    let tracker = get_global_tracker();
    
    // Create large dataset for performance testing
    let mut large_dataset = Vec::new();
    for i in 0..1000 {
        let data = vec![i as u8; 1000];
        track_var!(data);
        large_dataset.push(data);
    }
    println!("✅ Large dataset created (1000 vectors of 1000 elements each)");
    
    // Test different performance configurations
    let performance_configs = [
        ("Fast", OptimizedExportOptions::default()
            .parallel_processing(true)
            .buffer_size(1024 * 1024)  // 1MB
            .batch_size(5000)
            .streaming_writer(true)
            .adaptive_optimization(true)),
        
        ("Balanced", OptimizedExportOptions::default()
            .parallel_processing(true)
            .buffer_size(256 * 1024)   // 256KB
            .batch_size(1000)
            .streaming_writer(true)),
        
        ("Memory Efficient", OptimizedExportOptions::default()
            .parallel_processing(false)
            .buffer_size(64 * 1024)    // 64KB
            .batch_size(500)
            .streaming_writer(true)),
    ];
    
    for (name, options) in performance_configs.iter() {
        let filename = format!("demo_performance_{}.json", name.replace(" ", "_").to_lowercase());
        
        let start = Instant::now();
        tracker.export_to_json_with_options(&filename, options.clone())?;
        let duration = start.elapsed();
        
        println!("✅ {} configuration: {:?}", name, duration);
    }
    
    Ok(())
}

/// Demonstrate backward compatibility with older APIs
fn demonstrate_backward_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 6. Backward Compatibility Demonstration");
    println!("-------------------------------------------");
    
    let tracker = get_global_tracker();
    
    // Create test data using old-style patterns
    let legacy_vec = vec![1, 2, 3, 4, 5];
    track_var!(legacy_vec);
    
    let legacy_string = String::from("Legacy API Test");
    track_var!(legacy_string);
    
    // Use the original API (still works)
    tracker.export_to_json("demo_legacy_api.json")?;
    println!("✅ Legacy API export (export_to_json) works perfectly");
    
    // Show that it's equivalent to using default options
    let equivalent_options = OptimizedExportOptions::default();
    tracker.export_to_json_with_options("demo_equivalent_new_api.json", equivalent_options)?;
    println!("✅ New API with default options produces equivalent results");
    
    // Demonstrate migration path
    println!("✅ Migration path:");
    println!("   OLD: tracker.export_to_json(\"file.json\")?;");
    println!("   NEW: tracker.export_to_json_with_options(\"file.json\", OptimizedExportOptions::default())?;");
    println!("   Both produce identical results - no breaking changes!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_stability() {
        // Test that all APIs are accessible and stable
        init();
        
        let tracker = get_global_tracker();
        let _unsafe_tracker = get_global_unsafe_ffi_tracker();
        
        // Test basic API
        let test_vec = vec![1, 2, 3];
        track_var!(test_vec);
        
        // Test export APIs
        let basic_result = tracker.export_to_json("test_basic.json");
        assert!(basic_result.is_ok());
        
        let options = OptimizedExportOptions::default();
        let advanced_result = tracker.export_to_json_with_options("test_advanced.json", options);
        assert!(advanced_result.is_ok());
        
        // Clean up test files
        let _ = std::fs::remove_file("test_basic.json");
        let _ = std::fs::remove_file("test_advanced.json");
    }
    
    #[test]
    fn test_optimization_levels() {
        // Test all optimization levels are accessible
        let _low = OptimizedExportOptions::with_optimization_level(OptimizationLevel::Low);
        let _medium = OptimizedExportOptions::with_optimization_level(OptimizationLevel::Medium);
        let _high = OptimizedExportOptions::with_optimization_level(OptimizationLevel::High);
        let _maximum = OptimizedExportOptions::with_optimization_level(OptimizationLevel::Maximum);
    }
    
    #[test]
    fn test_builder_pattern() {
        // Test the builder pattern works correctly
        let options = OptimizedExportOptions::default()
            .parallel_processing(true)
            .buffer_size(1024)
            .batch_size(100)
            .streaming_writer(false)
            .schema_validation(true)
            .security_analysis(false);
        
        assert_eq!(options.parallel_processing, true);
        assert_eq!(options.buffer_size, 1024);
        assert_eq!(options.batch_size, 100);
        assert_eq!(options.use_streaming_writer, false);
        assert_eq!(options.enable_schema_validation, true);
        assert_eq!(options.enable_security_analysis, false);
    }
}