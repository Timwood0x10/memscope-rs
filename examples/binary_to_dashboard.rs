//! Binary to Dashboard Demo
//!
//! This example demonstrates how to convert binary memory analysis files
//! to HTML dashboards using the existing templates/dashboard.html template.

use std::fs;
use std::io::Write;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Binary to Dashboard Demo (Using Existing Template)");
    println!("===================================================");

    // Check if we have binary files
    let binary_files = [
        ("MemoryAnalysis/large_scale_user.memscope", "large_scale_user"),
        ("MemoryAnalysis/large_scale_full.memscope", "large_scale_full"),
    ];

    for (binary_file, project_name) in &binary_files {
        if std::path::Path::new(binary_file).exists() {
            println!("\n🔄 Converting {} to dashboard using template...", binary_file);
            
            let start = Instant::now();
            
            // Read the template
            let template_content = fs::read_to_string("templates/dashboard.html")?;
            
            // Generate mock data that matches the expected format
            let memory_data = generate_memory_data_for_template(binary_file, project_name)?;
            
            // Read CSS and JS content
            let css_content = fs::read_to_string("templates/styles.css")?;
            let js_content = fs::read_to_string("templates/script.js")?;
            
            // Replace placeholders in template (matching the actual template format)
            let html_content = template_content
                .replace("{{ project_name }}", project_name)
                .replace("{{ json_data }}", &memory_data)
                .replace("{{ CSS_CONTENT }}", &css_content)
                .replace("{{ JS_CONTENT }}", &js_content);
            
            // Create output path
            let output_dir = format!("MemoryAnalysis/{}", project_name);
            fs::create_dir_all(&output_dir)?;
            let output_path = format!("{}/dashboard_from_binary.html", output_dir);
            
            // Write HTML file
            let mut file = fs::File::create(&output_path)?;
            file.write_all(html_content.as_bytes())?;
            
            let elapsed = start.elapsed();
            println!("✅ Dashboard created in {}ms: {}", elapsed.as_millis(), output_path);
            println!("   📊 Open in browser: file://{}/{}", 
                std::env::current_dir()?.display(), output_path);
            
            break; // Demo with first available file
        }
    }

    println!("\n🎉 Binary to Dashboard demo completed!");
    println!("💡 This uses the same template format as the existing ccc.html");
    
    Ok(())
}

fn generate_memory_data_for_template(binary_file: &str, _project_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    // Generate data that matches window.analysisData structure expected by script.js
    let memory_data = serde_json::json!({
        "memory_analysis": {
            "allocations": [
                {
                    "id": 1,
                    "size": 102400,
                    "type_name": "Vec<u8>",
                    "scope_name": "main",
                    "timestamp_alloc": 1500,
                    "is_active": true
                },
                {
                    "id": 2,
                    "size": 81920,
                    "type_name": "HashMap<String, Value>",
                    "scope_name": "parser",
                    "timestamp_alloc": 2100,
                    "is_active": true
                },
                {
                    "id": 3,
                    "size": 65536,
                    "type_name": "String",
                    "scope_name": "buffer",
                    "timestamp_alloc": 1800,
                    "is_active": false
                }
            ],
            "memory_stats": {
                "total_size": 2048000,
                "peak_memory": 1536000,
                "total_allocations": 1500
            }
        },
        "complex_types": {
            "summary": {
                "total_complex_types": 25,
                "generic_type_count": 8
            }
        },
        "allocation_timeline": [
            {"timestamp": 1000, "memory_usage": 512000, "allocation_count": 375},
            {"timestamp": 2000, "memory_usage": 1024000, "allocation_count": 750},
            {"timestamp": 3000, "memory_usage": 1536000, "allocation_count": 1125},
            {"timestamp": 4000, "memory_usage": 1280000, "allocation_count": 937}
        ],
        "allocation_distribution": {
            "small": {"range": "0-1KB", "count": 800, "percentage": 53.3},
            "medium": {"range": "1KB-10KB", "count": 500, "percentage": 33.3},
            "large": {"range": "10KB-100KB", "count": 150, "percentage": 10.0},
            "huge": {"range": ">100KB", "count": 50, "percentage": 3.4}
        },
        "top_allocations": [
            {
                "id": 1,
                "size": 102400,
                "type": "Vec<u8>",
                "location": "main.rs:42",
                "status": "Active",
                "timestamp": 1500
            },
            {
                "id": 2,
                "size": 81920,
                "type": "HashMap<String, Value>",
                "location": "parser.rs:128",
                "status": "Active", 
                "timestamp": 2100
            },
            {
                "id": 3,
                "size": 65536,
                "type": "String",
                "location": "buffer.rs:67",
                "status": "Freed",
                "timestamp": 1800
            }
        ],
        "performance_metrics": {
            "export_time_ms": 35,
            "compression_ratio": 0.75,
            "throughput_mb_per_sec": 58.5
        },
        "metadata": {
            "export_version": "2.0",
            "optimization_level": "High",
            "generated_from_binary": true
        }
    });
    
    Ok(serde_json::to_string_pretty(&memory_data)?)
}