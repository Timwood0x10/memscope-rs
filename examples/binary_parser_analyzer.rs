//! Binary Parser Analyzer
//! 
//! This example analyzes the binary file to understand:
//! 1. What exactly is stored in the binary file
//! 2. Why the user-binary is so large
//! 3. Detailed breakdown of the binary content
//! 
//! Based on large_scale_binary_comparison.rs pattern

use memscope_rs::export::binary::detect_binary_type;
use memscope_rs::MemoryTracker;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Binary Parser Analyzer for comprehensive_rust_types.memscope");
    println!("================================================================");

    let binary_path = "MemoryAnalysis/comprehensive_rust_types.memscope";
    
    if !std::path::Path::new(binary_path).exists() {
        println!("❌ Binary file not found: {}", binary_path);
        println!("Please run comprehensive_rust_types_with_json example first to generate the binary file");
        return Ok(());
    }

    let file_size = std::fs::metadata(binary_path)?.len();
    println!("📊 Binary file: {} ({:.2} KB, {} bytes)", binary_path, file_size as f64 / 1024.0, file_size);

    // ===== STEP 1: Detect Binary Type =====
    println!("\n📋 STEP 1: Detecting Binary Type...");
    
    match detect_binary_type(binary_path) {
        Ok(binary_type) => {
            println!("✅ Binary type detected: {:?}", binary_type);
        },
        Err(e) => {
            println!("⚠️ Failed to detect binary type: {}", e);
        }
    }

    // ===== STEP 2: Parse Binary using MemoryTracker =====
    println!("\n📋 STEP 2: Parsing Binary File...");
    
    // Use the same method as large_scale_binary_comparison.rs
    let output_base = "comprehensive_rust_types_analysis";
    match MemoryTracker::parse_binary_to_json(binary_path, output_base) {
        Ok(_) => {
            println!("✅ Binary parsed successfully to {}", output_base);
        },
        Err(e) => {
            println!("❌ Failed to parse binary: {}", e);
            return Ok(());
        }
    }

    // ===== STEP 3: Analyze Generated JSON File =====
    println!("\n📋 STEP 3: Analyzing Generated JSON File...");
    
    let json_file_path = format!("./{}", output_base);
    if !Path::new(&json_file_path).exists() {
        println!("❌ JSON file not found: {}", json_file_path);
        return Ok(());
    }

    let json_file_size = fs::metadata(&json_file_path)?.len();
    println!("  📄 {}: {:.2} KB ({} bytes)", json_file_path, json_file_size as f64 / 1024.0, json_file_size);
    
    println!("📊 Total JSON size: {:.2} KB ({} bytes)", json_file_size as f64 / 1024.0, json_file_size);

    // ===== STEP 4: Analyze JSON Content =====
    println!("\n📋 STEP 4: Analyzing JSON Content...");
    
    let json_content = fs::read_to_string(&json_file_path)?;
    if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&json_content) {
        // Analyze top-level structure
        println!("✅ JSON structure analysis:");
        for (key, value) in json_data.as_object().unwrap_or(&serde_json::Map::new()) {
            match value {
                serde_json::Value::Array(arr) => {
                    println!("   • {}: {} items", key, arr.len());
                },
                serde_json::Value::Object(obj) => {
                    println!("   • {}: {} fields", key, obj.len());
                },
                _ => {
                    println!("   • {}: {}", key, value);
                }
            }
        }
        
        // Analyze allocations if present
        if let Some(allocations) = json_data.get("allocations").and_then(|a| a.as_array()) {
            println!("\n📊 Allocation Analysis:");
            println!("   • Total allocations: {}", allocations.len());
            
            // Analyze allocation data
            let mut size_distribution = std::collections::HashMap::new();
            let mut type_distribution = std::collections::HashMap::new();
            let mut var_name_distribution = std::collections::HashMap::new();
            
            for (i, alloc) in allocations.iter().enumerate() {
                // Size distribution
                if let Some(size) = alloc.get("size").and_then(|s| s.as_u64()) {
                    let size_category = match size {
                        0..=100 => "tiny (0-100)",
                        101..=1000 => "small (101-1K)",
                        1001..=10000 => "medium (1K-10K)",
                        10001..=100000 => "large (10K-100K)",
                        _ => "huge (>100K)",
                    };
                    *size_distribution.entry(size_category).or_insert(0) += 1;
                }
                
                // Type distribution
                if let Some(type_name) = alloc.get("type_name").and_then(|t| t.as_str()) {
                    *type_distribution.entry(type_name).or_insert(0) += 1;
                }
                
                // Variable name distribution
                let var_name = alloc.get("var_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no_var_name");
                *var_name_distribution.entry(var_name).or_insert(0) += 1;
                
                // Show first few allocations in detail
                if i < 5 {
                    println!("\n📄 Allocation {}:", i);
                    if let Some(ptr) = alloc.get("ptr") {
                        println!("   • Ptr: {}", ptr);
                    }
                    if let Some(size) = alloc.get("size") {
                        println!("   • Size: {} bytes", size);
                    }
                    if let Some(type_name) = alloc.get("type_name") {
                        println!("   • Type: {}", type_name);
                    }
                    if let Some(var_name) = alloc.get("var_name") {
                        println!("   • Var name: {}", var_name);
                    }
                    if let Some(timestamp) = alloc.get("timestamp_alloc") {
                        println!("   • Timestamp alloc: {}", timestamp);
                    }
                    if let Some(lifetime) = alloc.get("lifetime_ms") {
                        println!("   • Lifetime ms: {}", lifetime);
                    }
                }
            }
            
            println!("\n📊 Size Distribution:");
            for (category, count) in &size_distribution {
                println!("   • {}: {} allocations", category, count);
            }
            
            println!("\n📊 Type Distribution (top 10):");
            let mut type_vec: Vec<_> = type_distribution.iter().collect();
            type_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (type_name, count) in type_vec.iter().take(10) {
                println!("   • {}: {} allocations", type_name, count);
            }
            
            println!("\n📊 Variable Name Distribution (top 10):");
            let mut var_vec: Vec<_> = var_name_distribution.iter().collect();
            var_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (var_name, count) in var_vec.iter().take(10) {
                println!("   • {}: {} allocations", var_name, count);
            }
        } else {
            println!("⚠️ No allocations array found in JSON");
        }
    } else {
        println!("❌ Failed to parse JSON file");
    }

    // ===== STEP 5: Summary and Recommendations =====
    println!("\n🎯 Analysis Summary");
    println!("==================");
    println!("📊 Binary file size: {:.2} KB ({} bytes)", file_size as f64 / 1024.0, file_size);
    println!("📊 JSON file size: {:.2} KB ({} bytes)", json_file_size as f64 / 1024.0, json_file_size);
    if json_file_size > 0 {
        println!("📊 Compression ratio: {:.1}x (binary is {:.1}% of JSON size)", 
                 json_file_size as f64 / file_size as f64,
                 (file_size as f64 / json_file_size as f64) * 100.0);
    }
    
    if file_size > 100_000 {
        println!("⚠️ Large file size detected! Possible reasons:");
        println!("   • Large number of allocations being tracked");
        println!("   • Detailed call stack information for each allocation");
        println!("   • String data (variable names, type names, call stack frames)");
        println!("   • Optional fields containing additional metadata");
        println!("   • System allocations included (check export mode)");
    }
    
    println!("\n💡 Recommendations:");
    println!("   • Use UserOnly mode to exclude system allocations");
    println!("   • Consider filtering allocations by size or type");
    println!("   • Limit call stack depth if not needed");
    println!("   • Use compression for storage if needed");

    println!("\n🎉 Binary Analysis COMPLETED!");

    Ok(())
}