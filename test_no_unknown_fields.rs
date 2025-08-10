use memscope_rs::{get_global_tracker, track_var};
use memscope_rs::export::binary::BinaryParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Elimination of 'unknown' Fields");
    
    let tracker = get_global_tracker();
    
    // Create test data
    let user_data1 = track_var!(vec![1, 2, 3, 4, 5]);
    let user_data2 = track_var!(String::from("Hello World"));
    let user_data3 = track_var!(Box::new(42u64));
    
    // Export in full mode to test null field elimination
    println!("\n📦 Exporting full binary for unknown field test...");
    tracker.export_full_binary("test_no_unknown")?;
    
    // Convert to JSON
    println!("🔄 Converting to JSON with improved field inference...");
    BinaryParser::parse_full_binary_to_json("MemoryAnalysis/test_no_unknown.memscope", "test_no_unknown")?;
    
    // Read and check the memory analysis JSON file
    let json_file = "MemoryAnalysis/test_no_unknown/test_no_unknown_memory_analysis.json";
    let json_content = fs::read_to_string(json_file)?;
    
    println!("📄 Analyzing JSON content for 'unknown' fields...");
    
    // Count occurrences of "unknown"
    let unknown_count = json_content.matches("\"unknown\"").count();
    
    if unknown_count == 0 {
        println!("✅ SUCCESS: No 'unknown' fields found in JSON output!");
    } else {
        println!("⚠️  Found {} occurrences of 'unknown' fields", unknown_count);
    }
    
    // Show some examples of inferred type names and variable names
    println!("\n🔍 Sample of inferred field values:");
    
    // Extract some type_name and var_name examples
    let lines: Vec<&str> = json_content.lines().collect();
    let mut examples_found = 0;
    
    for line in lines {
        if line.contains("\"type_name\"") && examples_found < 3 {
            let trimmed = line.trim();
            println!("  Type: {}", trimmed);
            examples_found += 1;
        }
        if line.contains("\"var_name\"") && examples_found < 6 {
            let trimmed = line.trim();
            println!("  Var:  {}", trimmed);
            examples_found += 1;
        }
    }
    
    // Check for specific inferred patterns
    let has_inferred_types = json_content.contains("u64_or_f64_or_i64_or_usize") ||
                            json_content.contains("Vec_or_String_header") ||
                            json_content.contains("CustomType_") ||
                            json_content.contains("AlignedStruct_");
    
    let has_inferred_vars = json_content.contains("primitive_var_") ||
                           json_content.contains("small_struct_var_") ||
                           json_content.contains("heap_allocated_var_");
    
    if has_inferred_types {
        println!("✅ Found inferred type names (no more 'unknown' types)");
    }
    
    if has_inferred_vars {
        println!("✅ Found inferred variable names (no more 'unknown' variables)");
    }
    
    // Show file size
    let file_size = fs::metadata(json_file)?.len();
    println!("\n📊 Generated JSON file size: {} bytes", file_size);
    
    // Keep variables alive
    drop(user_data1);
    drop(user_data2);
    drop(user_data3);
    
    if unknown_count == 0 {
        println!("\n🎉 SUCCESS: All 'unknown' fields have been eliminated!");
        println!("   Full-binary mode now provides precise type and variable information.");
    } else {
        println!("\n⚠️  Some 'unknown' fields still remain - further optimization needed.");
    }
    
    Ok(())
}