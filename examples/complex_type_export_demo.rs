//! Complex Type Export Optimization Demo
//!
//! This example demonstrates the optimized export functionality for complex types,
//! showing how to separate complex type analysis data into dedicated files for better performance.

use memscope_rs::*;
use std::sync::{Arc, Mutex};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Complex Type Export Optimization Demo");
    println!("=========================================");
    
    // Create various complex types to demonstrate the optimization
    create_complex_types_for_demo();
    
    // Get the global tracker
    let tracker = get_global_tracker();
    
    println!("\n📊 Exporting with standard method...");
    let start_standard = std::time::Instant::now();
    tracker.export_to_json("demo_standard.json")?;
    let standard_time = start_standard.elapsed();
    println!("⏱️  Standard export took: {:.2}ms", standard_time.as_millis());
    
    println!("\n🚀 Exporting with optimized method...");
    let start_optimized = std::time::Instant::now();
    let export_result = tracker.export_to_json_optimized("demo_optimized")?;
    let optimized_time = start_optimized.elapsed();
    
    println!("\n✅ Export Optimization Results:");
    println!("================================");
    println!("⏱️  Standard export time: {:.2}ms", standard_time.as_millis());
    println!("⏱️  Optimized export time: {:.2}ms", optimized_time.as_millis());
    
    let speedup = standard_time.as_millis() as f64 / optimized_time.as_millis() as f64;
    println!("🚀 Performance improvement: {:.1}x faster", speedup);
    
    println!("\n📁 Generated Files:");
    println!("==================");
    println!("📄 Main file: {} ({} bytes)", 
             export_result.main_file, 
             export_result.export_stats.main_file_size);
    
    if let Some(ref file) = export_result.complex_types_file {
        println!("📄 Complex types: {}", file);
    }
    if let Some(ref file) = export_result.borrow_analysis_file {
        println!("📄 Borrow analysis: {}", file);
    }
    if let Some(ref file) = export_result.generic_analysis_file {
        println!("📄 Generic analysis: {}", file);
    }
    if let Some(ref file) = export_result.async_analysis_file {
        println!("📄 Async analysis: {}", file);
    }
    if let Some(ref file) = export_result.closure_analysis_file {
        println!("📄 Closure analysis: {}", file);
    }
    if let Some(ref file) = export_result.lifecycle_analysis_file {
        println!("📄 Lifecycle analysis: {}", file);
    }
    
    println!("\n📊 File Size Analysis:");
    println!("======================");
    let standard_size = std::fs::metadata("demo_standard.json")?.len();
    let total_optimized_size = export_result.export_stats.main_file_size + 
                               export_result.export_stats.complex_files_size;
    
    println!("📏 Standard file size: {} bytes", standard_size);
    println!("📏 Optimized main file: {} bytes", export_result.export_stats.main_file_size);
    println!("📏 Complex type files: {} bytes", export_result.export_stats.complex_files_size);
    println!("📏 Total optimized size: {} bytes", total_optimized_size);
    
    let size_ratio = export_result.export_stats.main_file_size as f64 / standard_size as f64;
    println!("📉 Main file is {:.1}% of original size", size_ratio * 100.0);
    
    println!("\n💡 Benefits of Optimized Export:");
    println!("=================================");
    println!("✅ Faster loading of main memory analysis");
    println!("✅ Complex type data loaded on-demand");
    println!("✅ Better performance for large datasets");
    println!("✅ Modular analysis - load only what you need");
    println!("✅ Reduced memory usage during export");
    
    // Demonstrate loading specific analysis files
    demonstrate_selective_loading(&export_result)?;
    
    Ok(())
}

fn create_complex_types_for_demo() {
    println!("🔧 Creating complex types for demonstration...");
    
    // 1. Interior mutability types
    let cell = Cell::new(42);
    let refcell = RefCell::new(vec![1, 2, 3, 4, 5]);
    
    // 2. Concurrency primitives
    let mutex = Arc::new(Mutex::new(String::from("shared data")));
    let mutex_clone = Arc::clone(&mutex);
    
    // 3. Collections with complex generics
    let mut map: HashMap<String, Vec<i32>> = HashMap::new();
    map.insert("numbers".to_string(), vec![1, 2, 3, 4, 5]);
    map.insert("more_numbers".to_string(), vec![6, 7, 8, 9, 10]);
    
    // 4. Nested complex types
    let nested: Vec<HashMap<String, RefCell<Vec<i32>>>> = vec![
        {
            let mut inner_map = HashMap::new();
            inner_map.insert("data1".to_string(), RefCell::new(vec![1, 2, 3]));
            inner_map.insert("data2".to_string(), RefCell::new(vec![4, 5, 6]));
            inner_map
        }
    ];
    
    // 5. Closures that capture environment
    let captured_value = 100;
    let closure = || {
        println!("Captured value: {}", captured_value);
        captured_value * 2
    };
    
    // Use the values to prevent optimization
    println!("📝 Created {} complex types:", 5);
    println!("   - Cell: {}", cell.get());
    println!("   - RefCell: {} items", refcell.borrow().len());
    println!("   - Mutex: protected data");
    println!("   - HashMap: {} entries", map.len());
    println!("   - Nested: {} complex structures", nested.len());
    println!("   - Closure result: {}", closure());
    
    // Keep references alive
    std::mem::forget(cell);
    std::mem::forget(refcell);
    std::mem::forget(mutex);
    std::mem::forget(mutex_clone);
    std::mem::forget(map);
    std::mem::forget(nested);
    // std::mem::forget(closure);
    let _= closure;

}

fn demonstrate_selective_loading(
    export_result: &memscope_rs::export::complex_type_export::ComplexTypeExportResult
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Demonstrating Selective Loading:");
    println!("===================================");
    
    // Load and analyze main file
    if std::path::Path::new(&export_result.main_file).exists() {
        let main_content = std::fs::read_to_string(&export_result.main_file)?;
        let main_size = main_content.len();
        println!("📖 Main file loaded: {} characters", main_size);
        
        // Parse main file to show it contains references
        if let Ok(main_data) = serde_json::from_str::<serde_json::Value>(&main_content) {
            if let Some(complex_files) = main_data.get("complex_type_files") {
                println!("📋 Main file contains references to complex type files:");
                if let Some(complex_types) = complex_files.get("complex_types_file") {
                    println!("   - Complex types: {}", complex_types);
                }
                if let Some(borrow_analysis) = complex_files.get("borrow_analysis_file") {
                    println!("   - Borrow analysis: {}", borrow_analysis);
                }
            }
        }
    }
    
    // Demonstrate loading specific complex type files on demand
    if let Some(ref complex_file) = export_result.complex_types_file {
        if std::path::Path::new(complex_file).exists() {
            let complex_content = std::fs::read_to_string(complex_file)?;
            println!("📖 Complex types file loaded on-demand: {} characters", complex_content.len());
        }
    }
    
    println!("💡 In a real application, you would:");
    println!("   1. Load main file first for overview");
    println!("   2. Load specific analysis files only when needed");
    println!("   3. Implement lazy loading for better performance");
    
    Ok(())
}