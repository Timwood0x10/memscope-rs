//! Test the unified binary export API with real data

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Unified Binary Export API");
    println!("====================================");

    // Use an existing binary file
    let binary_file = "MemoryAnalysis/large_scale_user.memscope";
    let base_name = "unified_test";

    if !std::path::Path::new(binary_file).exists() {
        println!("❌ Binary file not found: {}", binary_file);
        return Ok(());
    }

    println!("✅ Found binary file: {}", binary_file);

    // Test 1: JSON Export (should match existing performance)
    println!("\n📊 Test 1: JSON Export (Ultra-Fast)");
    let start = std::time::Instant::now();
    
    match memscope_rs::export::binary::export_binary_to_json(binary_file, base_name) {
        Ok(()) => {
            let json_time = start.elapsed();
            println!("✅ JSON export completed in {}ms", json_time.as_millis());
            
            // Check if files were created
            let json_files = [
                "MemoryAnalysis/unified_test/unified_test_memory_analysis.json",
                "MemoryAnalysis/unified_test/unified_test_lifetime.json",
                "MemoryAnalysis/unified_test/unified_test_performance.json",
                "MemoryAnalysis/unified_test/unified_test_unsafe_ffi.json",
                "MemoryAnalysis/unified_test/unified_test_complex_types.json",
            ];
            
            let mut created_files = 0;
            for file in &json_files {
                if std::path::Path::new(file).exists() {
                    created_files += 1;
                    let size = std::fs::metadata(file)?.len();
                    println!("  ✅ Created: {} ({} bytes)", file, size);
                }
            }
            println!("  📊 Created {} of {} JSON files", created_files, json_files.len());
        }
        Err(e) => {
            println!("❌ JSON export failed: {}", e);
        }
    }

    // Test 2: HTML Export (optimized)
    println!("\n🎨 Test 2: HTML Export (Optimized)");
    let start = std::time::Instant::now();
    
    match memscope_rs::export::binary::export_binary_to_html(binary_file, base_name) {
        Ok(()) => {
            let html_time = start.elapsed();
            println!("✅ HTML user export completed in {}ms", html_time.as_millis());
            
            // Check if HTML file was created
            let html_file = "MemoryAnalysis/unified_test/unified_test_user_dashboard.html";
            if std::path::Path::new(html_file).exists() {
                let size = std::fs::metadata(html_file)?.len();
                println!("  ✅ Created: {} ({} bytes)", html_file, size);
            } else {
                println!("  ❌ HTML file not found: {}", html_file);
            }
        }
        Err(e) => {
            println!("❌ HTML user export failed: {}", e);
        }
    }

    // Test 2.5: HTML System Export
    println!("\n🔧 Test 2.5: HTML Export (System Data)");
    let start = std::time::Instant::now();
    
    match memscope_rs::export::binary::export_binary_to_html_system(binary_file, base_name) {
        Ok(()) => {
            let html_system_time = start.elapsed();
            println!("✅ HTML system export completed in {}ms", html_system_time.as_millis());
            
            // Check if HTML file was created
            let html_file = "MemoryAnalysis/unified_test/unified_test_system_dashboard.html";
            if std::path::Path::new(html_file).exists() {
                let size = std::fs::metadata(html_file)?.len();
                println!("  ✅ Created: {} ({} bytes)", html_file, size);
            } else {
                println!("  ❌ HTML file not found: {}", html_file);
            }
        }
        Err(e) => {
            println!("❌ HTML system export failed: {}", e);
        }
    }

    // Test 3: Both Formats (parallel)
    println!("\n🔄 Test 3: Both Formats (Parallel)");
    let start = std::time::Instant::now();
    
    match memscope_rs::export::binary::export_binary_to_both(binary_file, "parallel_test") {
        Ok(()) => {
            let both_time = start.elapsed();
            println!("✅ Parallel export completed in {}ms", both_time.as_millis());
            
            // Check if both JSON and HTML files were created
            let json_file = "MemoryAnalysis/parallel_test/parallel_test_memory_analysis.json";
            let html_file = "MemoryAnalysis/parallel_test/parallel_test_dashboard.html";
            
            if std::path::Path::new(json_file).exists() {
                println!("  ✅ JSON files created");
            }
            if std::path::Path::new(html_file).exists() {
                println!("  ✅ HTML file created");
            }
        }
        Err(e) => {
            println!("❌ Parallel export failed: {}", e);
        }
    }

    println!("\n🎯 Test completed! Check the MemoryAnalysis directory for output files.");
    
    Ok(())
}