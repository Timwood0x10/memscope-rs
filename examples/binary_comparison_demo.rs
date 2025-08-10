use memscope_rs::{get_global_tracker, track_var};
use memscope_rs::export::binary::BinaryParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Binary Export Comparison Demo");
    println!("Comparing user-binary vs full-binary JSON output");
    
    let tracker = get_global_tracker();
    
    // Create some test data with user variables
    println!("\n📦 Creating test data...");
    let user_vector = track_var!(vec![1, 2, 3, 4, 5]);
    let user_string = track_var!(String::from("Hello Binary Comparison"));
    let user_hashmap = track_var!({
        let mut map = std::collections::HashMap::new();
        map.insert("key1", "value1");
        map.insert("key2", "value2");
        map
    });
    
    // Give some time for tracking to register and for system allocations to occur
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Export both user-only and full binary
    println!("📤 Exporting user-only binary...");
    tracker.export_user_binary("comparison_user")?;
    
    println!("📤 Exporting full binary...");
    tracker.export_full_binary("comparison_full")?;
    
    // Parse both to JSON
    println!("🔄 Parsing user binary to JSON...");
    BinaryParser::parse_user_binary_to_json(
        "MemoryAnalysis/comparison_user.memscope", 
        "comparison_user"
    )?;
    
    println!("🔄 Parsing full binary to JSON...");
    BinaryParser::parse_full_binary_to_json(
        "MemoryAnalysis/comparison_full.memscope", 
        "comparison_full"
    )?;
    
    // Compare the results
    println!("\n📊 Comparison Results:");
    println!("{}", "=".repeat(80));
    
    // Check binary file sizes
    let user_binary_size = fs::metadata("MemoryAnalysis/comparison_user.memscope")?.len();
    let full_binary_size = fs::metadata("MemoryAnalysis/comparison_full.memscope")?.len();
    
    println!("📁 Binary File Sizes:");
    println!("  User binary: {} bytes", user_binary_size);
    println!("  Full binary: {} bytes", full_binary_size);
    println!("  Size ratio: {:.1}x larger", full_binary_size as f64 / user_binary_size as f64);
    
    // Compare JSON files
    let json_files = [
        "memory_analysis.json",
        "lifetime.json", 
        "performance.json",
        "unsafe_ffi.json",
        "complex_types.json",
    ];
    
    for file_suffix in &json_files {
        println!("\n📄 Comparing {}:", file_suffix);
        println!("{}", "-".repeat(50));
        
        let user_file = format!("MemoryAnalysis/comparison_user/comparison_user_{}", file_suffix);
        let full_file = format!("MemoryAnalysis/comparison_full/comparison_full_{}", file_suffix);
        
        // Read and parse JSON files
        let user_content = fs::read_to_string(&user_file)?;
        let full_content = fs::read_to_string(&full_file)?;
        
        let user_json: serde_json::Value = serde_json::from_str(&user_content)?;
        let full_json: serde_json::Value = serde_json::from_str(&full_content)?;
        
        // Compare file sizes
        println!("  File sizes:");
        println!("    User: {} bytes", user_content.len());
        println!("    Full: {} bytes", full_content.len());
        
        // Count allocations if it's an object with allocations
        if let (Some(user_obj), Some(full_obj)) = (user_json.as_object(), full_json.as_object()) {
            if let (Some(user_allocs), Some(full_allocs)) = (
                user_obj.get("data").and_then(|d| d.get("allocations")).and_then(|a| a.as_array()),
                full_obj.get("data").and_then(|d| d.get("allocations")).and_then(|a| a.as_array())
            ) {
                println!("  Allocation counts:");
                println!("    User: {} allocations", user_allocs.len());
                println!("    Full: {} allocations", full_allocs.len());
                
                // Check for null values
                let user_nulls = count_null_values(&user_json);
                let full_nulls = count_null_values(&full_json);
                println!("  Null field counts:");
                println!("    User: {} null values", user_nulls);
                println!("    Full: {} null values", full_nulls);
                
                // Show sample allocation data
                if !user_allocs.is_empty() && !full_allocs.is_empty() {
                    println!("  Sample allocation comparison:");
                    println!("    User sample: {}", 
                        serde_json::to_string_pretty(&user_allocs[0]).unwrap_or_default().lines().take(5).collect::<Vec<_>>().join("\n    "));
                    println!("    Full sample: {}", 
                        serde_json::to_string_pretty(&full_allocs[0]).unwrap_or_default().lines().take(5).collect::<Vec<_>>().join("\n    "));
                }
            }
        } else if let (Some(user_arr), Some(full_arr)) = (user_json.as_array(), full_json.as_array()) {
            // Handle array-type files like unsafe_ffi.json
            println!("  Array lengths:");
            println!("    User: {} items", user_arr.len());
            println!("    Full: {} items", full_arr.len());
        }
    }
    
    // Show detailed comparison for memory_analysis.json
    println!("\n🔍 Detailed Memory Analysis Comparison:");
    println!("{}", "=".repeat(80));
    
    let user_memory_file = "MemoryAnalysis/comparison_user/comparison_user_memory_analysis.json";
    let full_memory_file = "MemoryAnalysis/comparison_full/comparison_full_memory_analysis.json";
    
    if let (Ok(user_content), Ok(full_content)) = (
        fs::read_to_string(user_memory_file),
        fs::read_to_string(full_memory_file)
    ) {
        let user_json: serde_json::Value = serde_json::from_str(&user_content)?;
        let full_json: serde_json::Value = serde_json::from_str(&full_content)?;
        
        // Extract allocation data
        if let (Some(user_data), Some(full_data)) = (
            user_json.get("data"),
            full_json.get("data")
        ) {
            println!("User binary data structure:");
            print_json_structure(&user_data, "  ");
            
            println!("\nFull binary data structure:");
            print_json_structure(&full_data, "  ");
            
            // Show metadata comparison
            if let (Some(user_meta), Some(full_meta)) = (
                user_data.get("metadata"),
                full_data.get("metadata")
            ) {
                println!("\nMetadata comparison:");
                println!("  User metadata: {}", serde_json::to_string_pretty(user_meta)?);
                println!("  Full metadata: {}", serde_json::to_string_pretty(full_meta)?);
            }
        }
    }
    
    // Keep variables alive
    drop(user_vector);
    drop(user_string);
    drop(user_hashmap);
    
    println!("\n🎉 Binary comparison demo completed!");
    println!("Check the generated files in MemoryAnalysis/comparison_user/ and MemoryAnalysis/comparison_full/");
    
    Ok(())
}

/// Recursively count null values in a JSON value
fn count_null_values(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Null => 1,
        serde_json::Value::Array(arr) => {
            arr.iter().map(count_null_values).sum()
        },
        serde_json::Value::Object(obj) => {
            obj.values().map(count_null_values).sum()
        },
        _ => 0,
    }
}

/// Print JSON structure overview
fn print_json_structure(value: &serde_json::Value, indent: &str) {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                match val {
                    serde_json::Value::Array(arr) => {
                        println!("{}{}: Array[{}]", indent, key, arr.len());
                    },
                    serde_json::Value::Object(_) => {
                        println!("{}{}: Object", indent, key);
                    },
                    _ => {
                        println!("{}{}: {}", indent, key, 
                            match val {
                                serde_json::Value::String(_) => "String",
                                serde_json::Value::Number(_) => "Number", 
                                serde_json::Value::Bool(_) => "Boolean",
                                serde_json::Value::Null => "Null",
                                _ => "Other"
                            }
                        );
                    }
                }
            }
        },
        serde_json::Value::Array(arr) => {
            println!("{}Array with {} items", indent, arr.len());
        },
        _ => {
            println!("{}Primitive value", indent);
        }
    }
}