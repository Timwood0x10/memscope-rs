use memscope_rs::export::binary::BinaryParser;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = "MemoryAnalysis/complex_lifecycle_binary.memscope";
    let base_name = "complex_lifecycle_test";
    
    println!("🚀 Testing binary-to-JSON conversion performance");
    println!("Binary file: {}", binary_path);
    println!("Target: <300ms for 5 JSON files");
    println!("=====================================");
    
    let start = Instant::now();
    
    // Test the conversion
    BinaryParser::to_standard_json_files(binary_path, base_name)?;
    
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    println!("✅ Conversion completed in {}ms", elapsed_ms);
    
    if elapsed_ms <= 300 {
        println!("🎯 SUCCESS: Performance target achieved! ({}ms <= 300ms)", elapsed_ms);
    } else {
        println!("❌ FAILED: Performance target missed ({}ms > 300ms)", elapsed_ms);
    }
    
    // Check generated files
    let output_dir = std::path::Path::new("MemoryAnalysis").join(base_name);
    let expected_files = [
        "memory_analysis",
        "lifetime", 
        "performance",
        "unsafe_ffi",
        "complex_types"
    ];
    
    println!("\n📁 Generated files:");
    for file_type in &expected_files {
        let file_path = output_dir.join(format!("{}_{}.json", base_name, file_type));
        if file_path.exists() {
            let size = std::fs::metadata(&file_path)?.len();
            println!("  ✅ {} ({:.1}KB)", file_path.display(), size as f64 / 1024.0);
        } else {
            println!("  ❌ {} (missing)", file_path.display());
        }
    }
    
    Ok(())
}