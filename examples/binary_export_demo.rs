//! Binary export demonstration
//!
//! This example shows how to use the binary export functionality
//! to export memory tracking data in a compact, efficient format.
//!
//! The demo will create files in ./MemoryAnalysis/binary_demo_example/ directory:
//! - binary_demo_example.memscope (binary format)
//! - binary_demo_example.json (converted from binary)
//! - binary_demo_example.html (HTML report from binary)

use memscope_rs::{core::tracker::MemoryTracker, get_global_tracker, track_var};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Binary Export Demo");
    println!("====================");

    // Create output directory
    let output_dir = Path::new("./MemoryAnalysis/binary_demo_example");
    fs::create_dir_all(output_dir)?;
    println!("📁 Created output directory: {}", output_dir.display());

    // Get the global memory tracker
    let tracker = get_global_tracker();
    // Don't enable fast mode to get richer data

    // Create realistic memory allocations with rich data
    println!("\n📊 Creating realistic memory allocations with rich data...");

    // Create various data structures to get rich allocation data
    let demo_buffer = vec![0u8; 1024];
    let _tracked_demo_buffer = track_var!(demo_buffer);

    let large_data = vec![42i32; 512]; // 512 * 4 = 2048 bytes
    let _tracked_large_data = track_var!(large_data);

    let config_string = "Configuration data with detailed settings and parameters".repeat(10);
    let _tracked_config_string = track_var!(config_string);

    let temp_array = vec![1.0f64; 32]; // 32 * 8 = 256 bytes
    let _tracked_temp_array = track_var!(temp_array);

    let image_buffer = vec![255u8; 4096];
    let _tracked_image_buffer = track_var!(image_buffer);

    let mut small_cache = HashMap::new();
    for i in 0..16 {
        small_cache.insert(format!("key_{}", i), format!("value_{}", i));
    }
    let _tracked_small_cache = track_var!(small_cache);

    let network_buffer = vec![0u8; 8192];
    let _tracked_network_buffer = track_var!(network_buffer);

    let metadata = format!(
        "{{\"version\": \"1.0\", \"timestamp\": {}, \"size\": 64}}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let _tracked_metadata = track_var!(metadata);

    // Add some smart pointer examples for richer data
    let boxed_string = Box::new("Boxed string data".to_string());
    let _tracked_boxed_string = track_var!(boxed_string);

    let shared_data = std::rc::Rc::new(vec![1, 2, 3, 4, 5]);
    let _tracked_shared_data = track_var!(shared_data);

    println!("✅ Created 10 realistic allocations with rich metadata");

    // Simulate some deallocations to show complete lifecycle
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = _tracked_demo_buffer; // This will trigger deallocation tracking
    let _ = _tracked_temp_array;

    println!("✅ Simulated 2 deallocations for lifecycle demonstration");

    // Export to binary format using MemoryTracker
    println!("\n💾 Exporting to binary format...");
    let start_time = std::time::Instant::now();
    tracker.export_to_binary("binary_demo_example")?;
    let binary_export_time = start_time.elapsed();

    // Find the created binary file
    let binary_file = find_binary_file("MemoryAnalysis")?;
    let binary_size = fs::metadata(&binary_file)?.len();

    println!("✅ Binary export completed in {:?}", binary_export_time);
    println!(
        "📁 Binary file: {} ({} bytes)",
        binary_file.display(),
        binary_size
    );

    // Define output file paths in the same directory
    let _json_file = output_dir.join("binary_demo_example.json");
    let html_file = output_dir.join("binary_demo_example.html");

    // Convert binary to standard JSON files (5 categorized files)
    println!("\n🔄 Converting binary to standard JSON files...");
    let start_time = std::time::Instant::now();
    MemoryTracker::parse_binary_to_standard_json(&binary_file, "binary_demo_example")?;
    let json_conversion_time = start_time.elapsed();

    // Check the generated JSON files
    let json_files = [
        "binary_demo_example_memory_analysis.json",
        "binary_demo_example_lifetime.json",
        "binary_demo_example_performance.json",
        "binary_demo_example_unsafe_ffi.json",
        "binary_demo_example_complex_types.json",
    ];

    let mut total_json_size = 0;
    println!(
        "✅ Standard JSON conversion completed in {:?}",
        json_conversion_time
    );
    println!("📄 Generated JSON files:");
    for json_file_name in &json_files {
        let json_file_path = output_dir.join(json_file_name);
        if json_file_path.exists() {
            let size = fs::metadata(&json_file_path)?.len();
            total_json_size += size;
            println!("  • {} ({} bytes)", json_file_name, size);
        }
    }

    // Also create a single JSON file for comparison
    let single_json_file = output_dir.join("binary_demo_example_single.json");
    MemoryTracker::parse_binary_to_json(&binary_file, &single_json_file)?;
    let single_json_size = fs::metadata(&single_json_file)?.len();

    // Convert binary to HTML report
    println!("\n🌐 Converting binary to HTML report...");
    let start_time = std::time::Instant::now();
    MemoryTracker::parse_binary_to_html(&binary_file, &html_file)?;
    let html_conversion_time = start_time.elapsed();

    let html_size = fs::metadata(&html_file)?.len();
    println!("✅ HTML report generated in {:?}", html_conversion_time);
    println!(
        "🌐 HTML file: {} ({} bytes)",
        html_file.display(),
        html_size
    );

    // Performance comparison with direct JSON export
    println!("\n� PSerformance Analysis:");
    println!("========================");

    // Export using standard JSON method for comparison
    let start_time = std::time::Instant::now();
    tracker.export_to_json("binary_demo_direct")?;
    let json_direct_time = start_time.elapsed();

    // Calculate total size of direct JSON export (5 files)
    let direct_json_dir = Path::new("MemoryAnalysis/binary_demo_direct");
    let mut json_direct_size = 0;
    if direct_json_dir.exists() {
        for entry in fs::read_dir(direct_json_dir)? {
            let entry = entry?;
            if entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
                json_direct_size += fs::metadata(entry.path())?.len();
            }
        }
    }

    // Calculate performance metrics
    let size_reduction =
        ((json_direct_size as f64 - binary_size as f64) / json_direct_size as f64) * 100.0;
    let speed_improvement =
        json_direct_time.as_nanos() as f64 / binary_export_time.as_nanos() as f64;

    println!("Binary vs Standard JSON Export Performance:");
    println!("  📊 Binary export time:     {:?}", binary_export_time);
    println!("  📊 Standard JSON time:     {:?}", json_direct_time);
    println!(
        "  🚀 Speed improvement:      {:.2}x faster",
        speed_improvement
    );
    println!("  📁 Binary file size:       {} bytes", binary_size);
    println!(
        "  📁 Standard JSON size:     {} bytes (5 files)",
        json_direct_size
    );
    println!(
        "  📁 Converted JSON size:    {} bytes (5 files)",
        total_json_size
    );
    println!(
        "  📁 Single JSON size:       {} bytes (1 file)",
        single_json_size
    );
    println!("  💾 Size reduction:         {:.1}%", size_reduction);

    println!("\nConversion Performance:");
    println!("  🔄 Binary → 5 JSON files:  {:?}", json_conversion_time);
    println!("  🌐 Binary → HTML:          {:?}", html_conversion_time);

    // Show file contents preview
    println!("\n📋 Generated Files Analysis:");
    println!("============================");

    // Analyze JSON content from the first generated file
    let first_json_file = output_dir.join("binary_demo_example_memory_analysis.json");
    let json_content = if first_json_file.exists() {
        fs::read_to_string(&first_json_file)?
    } else {
        fs::read_to_string(&single_json_file)?
    };
    let allocation_count = json_content.matches("\"ptr\":").count();
    println!("📄 JSON Analysis:");
    println!(
        "  • Total JSON files: {} (5 categorized + 1 single)",
        json_files.len() + 1
    );
    println!("  • Combined size: {} bytes", total_json_size);
    println!("  • Allocations found: {}", allocation_count);
    println!(
        "  • Contains structured data: {}",
        json_content.contains("\"size\":")
    );

    // Analyze HTML content
    let html_content = fs::read_to_string(&html_file)?;
    println!("\n🌐 HTML Analysis:");
    println!("  • File size: {} bytes", html_size);
    if html_content.contains("<title>") {
        let title_start = html_content.find("<title>").unwrap() + 7;
        let title_end = html_content.find("</title>").unwrap();
        println!("  • Title: {}", &html_content[title_start..title_end]);
    }
    println!(
        "  • Contains allocation table: {}",
        html_content.contains("<table")
    );
    println!(
        "  • Contains memory addresses: {}",
        html_content.contains("0x")
    );
    println!(
        "  • Interactive dashboard: {}",
        html_content.contains("javascript") || html_content.contains("script")
    );

    // Show sample JSON data
    println!("\n📋 Sample JSON Data (first allocation):");
    if let Some(start) = json_content.find("{") {
        if let Some(end) = json_content[start..].find("},") {
            let sample = &json_content[start..start + end + 1];
            println!("{}", sample);
        }
    }

    println!("\n🎉 Demo completed successfully!");
    println!("📁 All files generated in: {}", output_dir.display());
    println!("📋 Generated files:");
    println!(
        "  • {} (binary format - {} bytes)",
        binary_file.display(),
        binary_size
    );
    for json_file_name in &json_files {
        let json_file_path = output_dir.join(json_file_name);
        if json_file_path.exists() {
            let size = fs::metadata(&json_file_path).map(|m| m.len()).unwrap_or(0);
            println!("  • {} ({} bytes)", json_file_name, size);
        }
    }
    println!(
        "  • {} (HTML report - {} bytes)",
        html_file.display(),
        html_size
    );
    println!(
        "  • {} (single JSON - {} bytes)",
        single_json_file.display(),
        single_json_size
    );

    println!("\n💡 Next steps:");
    println!(
        "  1. Open {} in your browser to view the interactive report",
        html_file.display()
    );
    println!(
        "  2. Examine {} to see the structured allocation data",
        first_json_file.display()
    );
    println!(
        "  3. Compare file sizes: binary ({} bytes) vs JSON ({} bytes)",
        binary_size, total_json_size
    );

    // Create a simple index file for easy access
    let index_file = output_dir.join("README.md");
    let readme_content = format!(
        r#"# Binary Export Demo Results

This directory contains the results of the binary export demonstration.

## Generated Files

- `{}` - Binary format export ({} bytes)
- `{}` - JSON converted from binary ({} bytes) 
- `{}` - HTML report from binary ({} bytes)
- `{}` - Direct JSON export for comparison ({} bytes)

## Performance Results

- **Speed**: Binary export is {:.2}x faster than JSON export
- **Size**: Binary format is {:.1}% smaller than JSON format
- **Export time**: Binary {:?} vs JSON {:?}

## How to View

1. **HTML Report**: Open `{}` in your web browser
2. **JSON Data**: Open `{}` in any text editor or JSON viewer
3. **Binary Data**: Use hex editor or the conversion tools

## Binary Format Benefits

- Compact storage (saves {:.1}% space)
- Fast export/import ({:.2}x speed improvement)
- Preserves all allocation information
- Easy conversion to JSON/HTML formats
"#,
        binary_file.file_name().unwrap().to_string_lossy(),
        binary_size,
        single_json_file.file_name().unwrap().to_string_lossy(),
        single_json_size,
        html_file.file_name().unwrap().to_string_lossy(),
        html_size,
        direct_json_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        json_direct_size,
        speed_improvement,
        size_reduction,
        binary_export_time,
        json_direct_time,
        html_file.file_name().unwrap().to_string_lossy(),
        first_json_file.file_name().unwrap().to_string_lossy(),
        size_reduction,
        speed_improvement
    );

    fs::write(&index_file, readme_content)?;
    println!(
        "  4. Read {} for detailed information",
        index_file.display()
    );

    Ok(())
}

/// Find the binary file in the MemoryAnalysis directory
fn find_binary_file(base_dir: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let memory_analysis_dir = std::path::Path::new(base_dir);

    if !memory_analysis_dir.exists() {
        return Err("MemoryAnalysis directory not found".into());
    }

    // Look for .memscope files
    for entry in fs::read_dir(memory_analysis_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            for sub_entry in fs::read_dir(entry.path())? {
                let sub_entry = sub_entry?;
                if sub_entry.path().extension() == Some(std::ffi::OsStr::new("memscope")) {
                    return Ok(sub_entry.path());
                }
            }
        }
    }

    Err("No .memscope file found".into())
}
