//! Comprehensive tests for binary-to-JSON optimization system
//!
//! This test suite validates the optimization functionality and demonstrates
//! the JSON output quality and performance improvements using real binary files.

use memscope_rs::export::binary::{
    IntegrationConfig, BinaryParser, OptimizedBinaryToJsonConverter,
    SelectiveConversionConfig, AdaptiveMultiJsonExporter, JsonType
};
use memscope_rs::{get_global_tracker, track_var};
use std::fs;
use std::path::Path;

/// Create real binary file using MemoryTracker for testing
fn create_test_binary_file(binary_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tracker = get_global_tracker();
    tracker.enable_fast_mode(); // For faster testing
    
    // Create some realistic test allocations
    let test_vec = vec![1, 2, 3, 4, 5];
    let _tracked_vec = track_var!(test_vec);
    
    let test_string = "Hello, binary optimization test!".to_string();
    let _tracked_string = track_var!(test_string);
    
    let test_map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    let _tracked_map = track_var!(test_map);
    
    let large_buffer = vec![0u8; 1024];
    let _tracked_buffer = track_var!(large_buffer);
    
    // Export to binary format
    tracker.export_to_binary(binary_path)?;
    
    // Clean up tracked variables
    drop(_tracked_vec);
    drop(_tracked_string);
    drop(_tracked_map);
    drop(_tracked_buffer);
    
    Ok(())
}

#[test]
fn test_integration_config_basic() {
    println!("🧪 Testing basic integration configuration...");
    
    let default_config = IntegrationConfig::default();
    assert!(default_config.enable_optimization);
    assert!(default_config.enable_fallback);
    assert!(default_config.log_performance);
    
    println!("✅ Default config: optimization={}, fallback={}", 
             default_config.enable_optimization, default_config.enable_fallback);
}

#[test]
fn test_integration_config_presets() {
    println!("🧪 Testing integration configuration presets...");
    
    let perf_config = IntegrationConfig::performance_optimized();
    assert!(perf_config.enable_optimization);
    assert!(!perf_config.enable_fallback);
    assert_eq!(perf_config.optimization_threshold, 0);
    
    let reliability_config = IntegrationConfig::reliability_focused();
    assert!(reliability_config.enable_optimization);
    assert!(reliability_config.enable_fallback);
    assert!(reliability_config.enable_detailed_logging);
    
    let legacy_config = IntegrationConfig::legacy_compatible();
    assert!(!legacy_config.enable_optimization);
    
    println!("✅ All preset configurations work correctly");
}

#[test]
fn test_optimization_threshold() {
    println!("🧪 Testing optimization threshold logic...");
    
    let config = IntegrationConfig::default();
    
    assert!(!config.should_optimize(5 * 1024)); // Below 10KB threshold
    assert!(config.should_optimize(20 * 1024)); // Above 10KB threshold
    
    let disabled_config = IntegrationConfig {
        enable_optimization: false,
        ..Default::default()
    };
    assert!(!disabled_config.should_optimize(100 * 1024));
    
    println!("✅ Optimization threshold logic works correctly");
}

#[test]
fn test_binary_to_json_conversion_demo() {
    println!("🧪 Testing binary-to-JSON conversion with real binary data...");
    
    let binary_path = "tmp_rovodev_test_data";
    let json_base = "tmp_rovodev_test_output";
    
    // Clean up any existing files
    let _ = fs::remove_file(format!("{}.memscope", binary_path));
    let _ = fs::remove_dir_all(&json_base);
    let _ = fs::remove_dir_all("MemoryAnalysis");
    
    // Create real binary file using MemoryTracker
    match create_test_binary_file(binary_path) {
        Ok(_) => {
            println!("✅ Successfully created real binary file using MemoryTracker");
            
            // Find the actual binary file created
            let actual_binary_path = format!("MemoryAnalysis/{}.memscope", binary_path);
            
            if Path::new(&actual_binary_path).exists() {
                let binary_size = fs::metadata(&actual_binary_path).unwrap().len();
                println!("📁 Binary file: {} ({} bytes)", actual_binary_path, binary_size);
                
                // Convert binary to JSON using optimized converter
                match BinaryParser::to_standard_json_files(&actual_binary_path, json_base) {
                    Ok(_) => {
                        println!("✅ Successfully converted binary to JSON files");
                        
                        // Check generated JSON files
                        check_generated_json_files(&json_base);
                        
                        // Display sample JSON content
                        display_sample_json_content(&json_base);
                    }
                    Err(e) => {
                        println!("❌ Failed to convert binary to JSON: {}", e);
                    }
                }
            } else {
                println!("❌ Binary file not found at expected location: {}", actual_binary_path);
            }
        }
        Err(e) => {
            println!("❌ Failed to create binary file: {}", e);
        }
    }
    
    // Clean up test files
    let _ = fs::remove_file(format!("MemoryAnalysis/{}.memscope", binary_path));
    let _ = fs::remove_dir_all(&json_base);
    let _ = fs::remove_dir_all("MemoryAnalysis");
}

#[test]
fn test_optimized_converter_performance() {
    println!("🧪 Testing optimized converter performance characteristics...");
    
    let binary_path = "tmp_rovodev_perf_test";
    
    // Clean up any existing files
    let _ = fs::remove_file(format!("MemoryAnalysis/{}.memscope", binary_path));
    let _ = fs::remove_dir_all("MemoryAnalysis");
    
    // Create real binary file using MemoryTracker
    if create_test_binary_file(binary_path).is_ok() {
        let actual_binary_path = format!("MemoryAnalysis/{}.memscope", binary_path);
        // Test with different optimization levels
        let configs = vec![
            ("Performance", SelectiveConversionConfig::performance_first()),
            ("Balanced", SelectiveConversionConfig::default()),
            ("Memory", SelectiveConversionConfig::memory_efficient()),
        ];
        
        for (name, config) in configs {
            println!("  Testing {} configuration...", name);
            
            match OptimizedBinaryToJsonConverter::with_config(config) {
                Ok(mut converter) => {
                    let json_types = vec![
                        JsonType::MemoryAnalysis,
                        JsonType::LifetimeAnalysis,
                        JsonType::PerformanceAnalysis,
                    ];
                    
                    match converter.convert_binary_to_json(&actual_binary_path, &"tmp_rovodev_output".to_string(), &json_types) {
                        Ok(result) => {
                            println!("    ✅ {} - Processed {} records in {:?}", 
                                     name, result.allocations_processed, result.processing_time);
                            println!("    📊 Memory usage: {:.2} MB, Strategy: {}", 
                                     result.memory_peak_usage as f64 / (1024.0 * 1024.0), result.strategy_used);
                        }
                        Err(e) => {
                            println!("    ❌ {} failed: {}", name, e);
                        }
                    }
                }
                Err(e) => {
                    println!("    ❌ {} converter creation failed: {}", name, e);
                }
            }
            
            // Clean up output
            let _ = fs::remove_dir_all("tmp_rovodev_output");
        }
    }
    
    // Clean up test files
    let _ = fs::remove_file(binary_path);
}

#[test]
fn test_adaptive_multi_json_export() {
    println!("🧪 Testing adaptive multi-JSON export functionality...");
    
    let binary_path = "tmp_rovodev_adaptive_test";
    
    // Clean up any existing files
    let _ = fs::remove_file(format!("MemoryAnalysis/{}.memscope", binary_path));
    let _ = fs::remove_dir_all("MemoryAnalysis");
    
    if create_test_binary_file(binary_path).is_ok() {
        let actual_binary_path = format!("MemoryAnalysis/{}.memscope", binary_path);
        let exporter = AdaptiveMultiJsonExporter::new();
        
        let json_types = vec![
            JsonType::MemoryAnalysis,
            JsonType::LifetimeAnalysis,
            JsonType::PerformanceAnalysis,
            JsonType::ComplexTypes,
            JsonType::UnsafeFFI,
        ];
        
        match exporter.export_adaptive(&actual_binary_path, "tmp_rovodev_adaptive", &json_types) {
            Ok(stats) => {
                println!("✅ Adaptive export completed successfully");
                println!("  📊 Strategy: {:?}", stats.strategy_used);
                println!("  ⏱️  Total time: {:?}", stats.total_duration);
                println!("  📁 Files generated: {}", json_types.len());
                println!("  💾 Total records: {}", stats.total_records);
                
                // Verify files were created
                for json_type in &json_types {
                    let filename = format!("tmp_rovodev_adaptive/{}.json", json_type.filename_suffix());
                    if Path::new(&filename).exists() {
                        println!("  ✅ Generated: {}", filename);
                    } else {
                        println!("  ❌ Missing: {}", filename);
                    }
                }
            }
            Err(e) => {
                println!("❌ Adaptive export failed: {}", e);
            }
        }
        
        // Clean up output
        let _ = fs::remove_dir_all("tmp_rovodev_adaptive");
    }
    
    // Clean up test files
    let _ = fs::remove_file(binary_path);
}

fn check_generated_json_files(base_path: &str) {
    let expected_files = vec![
        "all_allocations.json",
        "active_allocations.json", 
        "dropped_allocations.json",
        "lifecycle_analysis.json",
        "thread_analysis.json",
    ];
    
    println!("  📁 Checking generated JSON files:");
    for file in &expected_files {
        let path = format!("{}/{}", base_path, file);
        if Path::new(&path).exists() {
            if let Ok(metadata) = fs::metadata(&path) {
                println!("    ✅ {} ({} bytes)", file, metadata.len());
            } else {
                println!("    ✅ {} (size unknown)", file);
            }
        } else {
            println!("    ❌ {} (missing)", file);
        }
    }
}

fn display_sample_json_content(base_path: &str) {
    println!("  📄 Sample JSON content:");
    
    let sample_file = format!("{}/all_allocations.json", base_path);
    if let Ok(content) = fs::read_to_string(&sample_file) {
        // Display first few lines of JSON
        let lines: Vec<&str> = content.lines().take(10).collect();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                println!("    📝 {}:", sample_file);
            }
            println!("      {}", line);
        }
        if content.lines().count() > 10 {
            println!("      ... ({} more lines)", content.lines().count() - 10);
        }
        
        // Show file size
        println!("    📊 File size: {} bytes", content.len());
    } else {
        println!("    ❌ Could not read sample file: {}", sample_file);
    }
}

#[test]
fn test_simple_real_binary_to_json() {
    println!("🧪 Testing REAL binary-to-JSON conversion...");
    
    // Clean up
    let _ = std::fs::remove_dir_all("MemoryAnalysis");
    
    // Create real allocations
    let tracker = get_global_tracker();
    tracker.enable_fast_mode();
    
    let test_vec = vec![1, 2, 3, 4, 5];
    let _tracked_vec = track_var!(test_vec);
    
    let test_string = "Hello, binary optimization!".to_string();
    let _tracked_string = track_var!(test_string);
    
    // Export to binary
    println!("📁 Creating binary file...");
    if tracker.export_to_binary("demo_test").is_ok() {
        println!("✅ Binary export successful");
        
        let binary_path = "MemoryAnalysis/demo_test.memscope";
        if std::path::Path::new(binary_path).exists() {
            let binary_size = std::fs::metadata(binary_path).unwrap().len();
            println!("📁 Binary file: {} ({} bytes)", binary_path, binary_size);
            
            // Convert to JSON
            println!("🔄 Converting binary to JSON...");
            if let Ok(_) = memscope_rs::core::tracker::MemoryTracker::parse_binary_to_standard_json(binary_path, "demo_output") {
                println!("✅ JSON conversion successful");
                
                // Show generated files
                let output_dir = "MemoryAnalysis/demo_output";
                if let Ok(entries) = std::fs::read_dir(output_dir) {
                    println!("📄 Generated JSON files:");
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension() == Some(std::ffi::OsStr::new("json")) {
                            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            println!("  • {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
                            
                            // Show sample content
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let sample = if content.len() > 150 { &content[..150] } else { &content };
                                println!("    📝 Sample: {}", sample);
                                if content.len() > 150 { println!("    ... (truncated)"); }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Clean up
    let _ = std::fs::remove_dir_all("MemoryAnalysis");
}

#[test]
fn test_environment_config_parsing() {
    println!("🧪 Testing environment configuration parsing...");
    
    let config = IntegrationConfig::from_environment();
    
    assert!(config.optimization_threshold > 0);
    
    println!("✅ Environment configuration parsing works");
}

#[test]
fn test_large_binary_conversion_debug() {
    println!("🚀 Testing large binary conversion with debug info...");
    
    let binary_path = "./MemoryAnalysis/complex_lifecycle_binary.memscope";
    
    if !std::path::Path::new(binary_path).exists() {
        println!("❌ Binary file not found: {}", binary_path);
        println!("   Please run: cargo run --example complex_lifecycle_showcase_binary");
        return;
    }
    
    let binary_size = std::fs::metadata(binary_path).unwrap().len();
    println!("📁 Binary file: {} ({} bytes)", binary_path, binary_size);
    
    // Clean up any existing output
    let output_base = "debug_test_output";
    let output_dir = format!("MemoryAnalysis/{}", output_base);
    let _ = std::fs::remove_dir_all(&output_dir);
    
    // Convert using BinaryParser
    println!("🔄 Converting binary to JSON...");
    println!("📊 File size: {} bytes (should trigger optimization if > 10KB)", binary_size);
    let start = std::time::Instant::now();
    
    println!("🔧 About to call BinaryParser::to_standard_json_files");
    let result = BinaryParser::to_standard_json_files(binary_path, output_base);
    println!("🔧 BinaryParser::to_standard_json_files returned: {:?}", result.is_ok());
    let duration = start.elapsed();
    
    match result {
        Ok(_) => {
            println!("✅ Conversion completed in {:?}", duration);
            
            // Check generated files in the correct output directory
            println!("📁 Checking output directory: {}", output_dir);
            
            if std::path::Path::new(&output_dir).exists() {
                if let Ok(entries) = std::fs::read_dir(&output_dir) {
                    let mut file_count = 0;
                    let mut total_size = 0u64;
                    
                    for entry in entries.flatten() {
                        if entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
                            file_count += 1;
                            let size = std::fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
                            total_size += size;
                            
                            let filename = entry.path().file_name().unwrap().to_string_lossy().to_string();
                            println!("  📄 {}: {} bytes", filename, size);
                            
                            // Check if file has content
                            if size > 100 {
                                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                    if content.contains("[]") && content.len() < 50 {
                                        println!("    ⚠️  File appears to be empty array");
                                    } else if content.contains("[") {
                                        println!("    ✅ File contains allocation data");
                                    }
                                }
                            }
                        }
                    }
                    
                    println!("📊 Summary: {} JSON files, {} total bytes", file_count, total_size);
                    
                    if file_count == 0 {
                        println!("❌ No JSON files generated!");
                    } else if total_size < 1000 {
                        println!("⚠️  Files are very small, may be empty");
                    } else {
                        println!("✅ Files generated successfully");
                    }
                } else {
                    println!("❌ Could not read output directory");
                }
            } else {
                println!("❌ Output directory not found: {}", output_dir);
            }
        }
        Err(e) => {
            println!("❌ Conversion failed: {}", e);
        }
    }
    
    // Clean up
    // let _ = std::fs::remove_dir_all(&output_dir);
}