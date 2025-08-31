//! Comprehensive memory analysis example
//! 
//! This example demonstrates advanced memory tracking and analysis capabilities:
//! - Complex data structure tracking
//! - Memory lifecycle analysis
//! - Export to multiple formats
//! - Performance monitoring

use memscope_rs::*;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Demonstrates comprehensive memory tracking for complex data structures
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting comprehensive memory analysis example");
    
    // Initialize memory tracker
    let tracker = MemoryTracker::new();
    
    // Demonstrate tracking of various data structures
    demonstrate_basic_tracking(&tracker)?;
    demonstrate_complex_structures(&tracker)?;
    demonstrate_lifecycle_analysis(&tracker)?;
    demonstrate_export_functionality(&tracker)?;
    
    tracing::info!("Comprehensive memory analysis example completed successfully");
    Ok(())
}

/// Demonstrates basic memory tracking operations
fn demonstrate_basic_tracking(tracker: &MemoryTracker) -> TrackingResult<()> {
    tracing::info!("Demonstrating basic memory tracking");
    
    // Track simple allocations
    let simple_int = 42i32;
    let simple_float = std::f64::consts::PI;
    let simple_bool = true;
    
    // Track these allocations
    let int_ptr = &simple_int as *const i32 as usize;
    let float_ptr = &simple_float as *const f64 as usize;
    let bool_ptr = &simple_bool as *const bool as usize;
    
    tracker.track_allocation(int_ptr, std::mem::size_of::<i32>())?;
    tracker.associate_var(int_ptr, "simple_int".to_string(), "i32".to_string())?;
    
    tracker.track_allocation(float_ptr, std::mem::size_of::<f64>())?;
    tracker.associate_var(float_ptr, "simple_float".to_string(), "f64".to_string())?;
    
    tracker.track_allocation(bool_ptr, std::mem::size_of::<bool>())?;
    tracker.associate_var(bool_ptr, "simple_bool".to_string(), "bool".to_string())?;
    
    // Display current statistics
    let stats = tracker.get_stats()?;
    tracing::info!("Basic tracking stats: total={}, active={}, memory={}KB", 
        stats.total_allocations, 
        stats.active_allocations, 
        stats.active_memory / 1024
    );
    
    Ok(())
}

/// Demonstrates tracking of complex data structures
fn demonstrate_complex_structures(tracker: &MemoryTracker) -> TrackingResult<()> {
    tracing::info!("Demonstrating complex data structure tracking");
    
    // Create complex data structures
    let mut user_data = HashMap::new();
    user_data.insert("name".to_string(), "Alice".to_string());
    user_data.insert("email".to_string(), "alice@example.com".to_string());
    user_data.insert("age".to_string(), "30".to_string());
    
    let shared_config = Arc::new(vec![
        "setting1".to_string(),
        "setting2".to_string(),
        "setting3".to_string(),
    ]);
    
    let nested_structure = vec![
        HashMap::from([("key1", vec![1, 2, 3])]),
        HashMap::from([("key2", vec![4, 5, 6])]),
        HashMap::from([("key3", vec![7, 8, 9])]),
    ];
    
    // Track complex structures
    let user_data_ptr = &user_data as *const HashMap<String, String> as usize;
    tracker.track_allocation(user_data_ptr, std::mem::size_of_val(&user_data))?;
    tracker.associate_var(user_data_ptr, "user_data".to_string(), "HashMap<String, String>".to_string())?;
    
    let config_ptr = &*shared_config as *const Vec<String> as usize;
    tracker.track_allocation(config_ptr, std::mem::size_of_val(&*shared_config))?;
    tracker.associate_var(config_ptr, "shared_config".to_string(), "Arc<Vec<String>>".to_string())?;
    
    let nested_ptr = &nested_structure as *const Vec<HashMap<&str, Vec<i32>>> as usize;
    tracker.track_allocation(nested_ptr, std::mem::size_of_val(&nested_structure))?;
    tracker.associate_var(nested_ptr, "nested_structure".to_string(), "Vec<HashMap<&str, Vec<i32>>>".to_string())?;
    
    // Simulate some operations that might affect memory
    let cloned_config = Arc::clone(&shared_config);
    let cloned_ptr = &*cloned_config as *const Vec<String> as usize;
    tracker.track_allocation(cloned_ptr, 0)?; // Shared reference, no additional memory
    tracker.associate_var(cloned_ptr, "cloned_config".to_string(), "Arc<Vec<String>>".to_string())?;
    
    let stats = tracker.get_stats()?;
    tracing::info!("Complex structures stats: total={}, active={}, memory={}KB", 
        stats.total_allocations, 
        stats.active_allocations, 
        stats.active_memory / 1024
    );
    
    Ok(())
}

/// Demonstrates memory lifecycle analysis
fn demonstrate_lifecycle_analysis(tracker: &MemoryTracker) -> TrackingResult<()> {
    tracing::info!("Demonstrating memory lifecycle analysis");
    
    // Create temporary allocations to demonstrate lifecycle
    let temp_allocations = vec![
        ("temp_string", String::from("temporary data")),
        ("temp_vec", format!("vector_{}", 123)),
        ("temp_large", "x".repeat(1000)),
    ];
    
    let mut allocation_ptrs = Vec::new();
    
    // Track temporary allocations
    for (name, data) in &temp_allocations {
        let ptr = data.as_ptr() as usize;
        tracker.track_allocation(ptr, data.len())?;
        tracker.associate_var(ptr, name.to_string(), "String".to_string())?;
        allocation_ptrs.push(ptr);
        
        tracing::info!("Allocated temporary data: {} ({}B)", name, data.len());
    }
    
    // Simulate some work with the data
    let work_simulation = (0..1000).map(|i| i * 2).collect::<Vec<_>>();
    let _work_result = work_simulation.iter().sum::<i32>();
    
    // Track deallocations (simulating drop)
    for (i, ptr) in allocation_ptrs.iter().enumerate() {
        tracker.track_deallocation(*ptr)?;
        tracing::info!("Deallocated temporary data: {}", temp_allocations[i].0);
    }
    
    let stats = tracker.get_stats()?;
    tracing::info!("Lifecycle analysis stats: total={}, active={}, peak={}KB", 
        stats.total_allocations, 
        stats.active_allocations, 
        stats.peak_memory / 1024
    );
    
    Ok(())
}

/// Demonstrates export functionality to different formats
fn demonstrate_export_functionality(tracker: &MemoryTracker) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Demonstrating export functionality");
    
    // Create temporary directory for exports
    let temp_dir = TempDir::new()?;
    let binary_path = temp_dir.path().join("memory_analysis.bin");
    let json_path = temp_dir.path().join("memory_analysis.json");
    let html_path = temp_dir.path().join("memory_analysis.html");
    
    // Add some final allocations for export demonstration
    let export_data = [
        "Export demonstration data".to_string(),
        "Memory analysis results".to_string(),
        "Performance metrics".to_string(),
    ];
    
    for (i, data) in export_data.iter().enumerate() {
        let ptr = data.as_ptr() as usize;
        tracker.track_allocation(ptr, data.len())?;
        tracker.associate_var(ptr, format!("export_data_{i}"), "String".to_string())?;
    }
    
    // Export to binary format
    tracker.export_to_binary(&binary_path)?;
    tracing::info!("Exported memory data to binary: {}", binary_path.display());
    
    // Export to JSON format
    memscope_rs::export::binary::export_binary_to_json(&binary_path, "comprehensive_analysis")?;
    tracing::info!("Exported memory data to JSON: {}", json_path.display());
    
    // Export to HTML format
    memscope_rs::export::binary::export_binary_to_html(&binary_path, "comprehensive_analysis")?;
    tracing::info!("Exported memory data to HTML: {}", html_path.display());
    
    // Display final statistics
    let stats = tracker.get_stats()?;
    tracing::info!("Final analysis statistics:");
    tracing::info!("  Total allocations: {}", stats.total_allocations);
    tracing::info!("  Active allocations: {}", stats.active_allocations);
    tracing::info!("  Active memory: {}KB", stats.active_memory / 1024);
    tracing::info!("  Peak memory: {}KB", stats.peak_memory / 1024);
    
    // Verify export files exist and have content
    let binary_size = std::fs::metadata(&binary_path)?.len();
    let json_size = std::fs::metadata(&json_path)?.len();
    let html_size = std::fs::metadata(&html_path)?.len();
    
    tracing::info!("Export file sizes:");
    tracing::info!("  Binary: {}B", binary_size);
    tracing::info!("  JSON: {}B", json_size);
    tracing::info!("  HTML: {}B", html_size);
    
    // Verify JSON content structure
    let json_content = std::fs::read_to_string(&json_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&json_content)?;
    
    if let Some(allocations) = parsed.get("allocations").and_then(|v| v.as_array()) {
        tracing::info!("JSON export contains {} allocation records", allocations.len());
        
        // Display sample allocation data
        if let Some(first_alloc) = allocations.first() {
            if let (Some(var_name), Some(type_name), Some(size)) = (
                first_alloc.get("var_name").and_then(|v| v.as_str()),
                first_alloc.get("type_name").and_then(|v| v.as_str()),
                first_alloc.get("size").and_then(|v| v.as_u64())
            ) {
                tracing::info!("Sample allocation: {} ({}) - {}B", var_name, type_name, size);
            }
        }
    }
    
    // Verify HTML content structure
    let html_content = std::fs::read_to_string(&html_path)?;
    let has_visualization = html_content.contains("Variable Lifecycle Visualization");
    let has_statistics = html_content.contains("Enhanced Memory Statistics");
    let has_data = html_content.contains("allocations");
    
    tracing::info!("HTML export verification:");
    tracing::info!("  Contains visualization: {}", has_visualization);
    tracing::info!("  Contains statistics: {}", has_statistics);
    tracing::info!("  Contains data: {}", has_data);
    
    if has_visualization && has_statistics && has_data {
        tracing::info!("HTML export is complete and properly formatted");
    } else {
        tracing::warn!("HTML export may be missing some components");
    }
    
    Ok(())
}

/// Performance monitoring example
#[allow(dead_code)]
fn demonstrate_performance_monitoring(tracker: &MemoryTracker) -> TrackingResult<()> {
    tracing::info!("Demonstrating performance monitoring");
    
    let start_time = std::time::Instant::now();
    let allocation_count = 1000;
    
    // Perform many allocations to test performance
    for i in 0..allocation_count {
        let data = format!("performance_test_{i}");
        let ptr = data.as_ptr() as usize;
        
        tracker.track_allocation(ptr, data.len())?;
        tracker.associate_var(ptr, format!("perf_var_{i}"), "String".to_string())?;
        
        // Deallocate every other allocation
        if i % 2 == 0 {
            tracker.track_deallocation(ptr)?;
        }
    }
    
    let duration = start_time.elapsed();
    let stats = tracker.get_stats()?;
    
    tracing::info!("Performance monitoring results:");
    tracing::info!("  Operations: {}", allocation_count * 2); // alloc + assoc for each
    tracing::info!("  Duration: {:?}", duration);
    tracing::info!("  Ops/sec: {:.2}", (allocation_count * 2) as f64 / duration.as_secs_f64());
    tracing::info!("  Final active allocations: {}", stats.active_allocations);
    tracing::info!("  Memory efficiency: {:.2}%", 
        (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0);
    
    Ok(())
}