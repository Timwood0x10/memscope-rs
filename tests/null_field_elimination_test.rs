use memscope_rs::{get_global_tracker, track_var};
use memscope_rs::export::binary::BinaryParser;
use std::fs;

#[test]
fn test_null_field_elimination_in_full_binary() {
    println!("🧪 Testing Null Field Elimination in Full Binary Mode");
    
    let tracker = get_global_tracker();
    
    // Create test data
    let test_data1 = track_var!(vec![1, 2, 3, 4, 5]);
    let test_data2 = track_var!(String::from("Null Test"));
    
    // Give some time for tracking to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Export full binary (should have no null fields)
    println!("📦 Exporting full binary...");
    tracker.export_full_binary("test_null_elimination").unwrap();
    
    // Parse to JSON using full binary strategy
    println!("🔄 Parsing full binary to JSON...");
    BinaryParser::parse_full_binary_to_json(
        "MemoryAnalysis/test_null_elimination.memscope", 
        "test_null_elimination"
    ).unwrap();
    
    // Check each of the 5 JSON files for null values
    let json_files = [
        "test_null_elimination_memory_analysis.json",
        "test_null_elimination_lifetime.json", 
        "test_null_elimination_performance.json",
        "test_null_elimination_unsafe_ffi.json",
        "test_null_elimination_complex_types.json",
    ];
    
    for file_name in &json_files {
        let file_path = format!("MemoryAnalysis/test_null_elimination/{}", file_name);
        println!("🔍 Checking {} for null fields...", file_name);
        
        // Read the JSON file
        let json_content = fs::read_to_string(&file_path)
            .expect(&format!("Failed to read {}", file_path));
        
        // Parse as JSON to validate structure
        let json_value: serde_json::Value = serde_json::from_str(&json_content)
            .expect(&format!("Failed to parse JSON in {}", file_path));
        
        // Check for null values recursively
        let null_count = count_null_values(&json_value);
        
        println!("  Found {} null values in {}", null_count, file_name);
        
        // For full binary mode, we should have ZERO null values
        assert_eq!(null_count, 0, 
            "Full binary mode should not contain any null fields in {}. Found {} null values.", 
            file_name, null_count);
    }
    
    // Variables are already handled by track_var! macro
    drop(test_data1);
    drop(test_data2);
    
    println!("✅ All JSON files are null-free in full binary mode!");
}

#[test]
fn test_json_format_consistency() {
    println!("🧪 Testing JSON Format Consistency Between User and Full Binary");
    
    let tracker = get_global_tracker();
    
    // Create test data
    let test_data = track_var!(vec![1, 2, 3]);
    
    // Give some time for tracking to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Export both user and full binary
    tracker.export_user_binary("test_consistency_user").unwrap();
    tracker.export_full_binary("test_consistency_full").unwrap();
    
    // Parse both to JSON
    BinaryParser::parse_user_binary_to_json(
        "MemoryAnalysis/test_consistency_user.memscope", 
        "test_consistency_user"
    ).unwrap();
    
    BinaryParser::parse_full_binary_to_json(
        "MemoryAnalysis/test_consistency_full.memscope", 
        "test_consistency_full"
    ).unwrap();
    
    // Check that both modes generate the same 5 files
    let json_files = [
        "memory_analysis.json",
        "lifetime.json", 
        "performance.json",
        "unsafe_ffi.json",
        "complex_types.json",
    ];
    
    for file_suffix in &json_files {
        let user_file = format!("MemoryAnalysis/test_consistency_user/test_consistency_user_{}", file_suffix);
        let full_file = format!("MemoryAnalysis/test_consistency_full/test_consistency_full_{}", file_suffix);
        
        // Both files should exist
        assert!(fs::metadata(&user_file).is_ok(), "User binary should generate {}", file_suffix);
        assert!(fs::metadata(&full_file).is_ok(), "Full binary should generate {}", file_suffix);
        
        // Both should be valid JSON
        let user_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&user_file).unwrap()
        ).expect(&format!("User {} should be valid JSON", file_suffix));
        
        let full_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&full_file).unwrap()
        ).expect(&format!("Full {} should be valid JSON", file_suffix));
        
        // Both should have the same top-level structure type
        match (&user_json, &full_json) {
            (serde_json::Value::Object(user_obj), serde_json::Value::Object(full_obj)) => {
                assert_eq!(
                    user_obj.keys().collect::<std::collections::BTreeSet<_>>(),
                    full_obj.keys().collect::<std::collections::BTreeSet<_>>(),
                    "User and full binary should have same JSON object structure for {}", file_suffix
                );
            },
            (serde_json::Value::Array(_), serde_json::Value::Array(_)) => {
                // Both are arrays, which is fine for some files like unsafe_ffi
            },
            _ => {
                panic!("User and full binary should have same JSON type for {}", file_suffix);
            }
        }
        
        println!("✅ {} has consistent structure between user and full binary", file_suffix);
    }
    
    drop(test_data);
    
    println!("🎉 JSON format consistency test completed successfully!");
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