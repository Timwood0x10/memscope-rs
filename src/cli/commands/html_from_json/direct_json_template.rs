//! Direct JSON template generator that uses raw JSON data without complex processing

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Generate HTML directly from raw JSON data
pub fn generate_direct_html(json_data: &HashMap<String, Value>) -> Result<String, Box<dyn Error>> {
    println!("🎨 Generating enhanced HTML with embedded JSON data...");
    
    // Validate that we have essential data
    if json_data.is_empty() {
        return Err("No JSON data provided for HTML generation".into());
    }
    
    // Log what data we have
    for (key, value) in json_data {
        println!("📊 Found data: {} ({} bytes)", key, 
            serde_json::to_string(value).unwrap_or_default().len());
    }
    
    // Transform the data structure to match JavaScript expectations
    let transformed_data = transform_json_data_structure(json_data)?;
    
    // Serialize the transformed JSON data for embedding with proper escaping
    let json_data_str = serde_json::to_string(&transformed_data)
        .map_err(|e| format!("Failed to serialize JSON data: {e}"))?;
    
    // Log data structure for debugging
    if let Some(unsafe_ffi_data) = json_data.get("basic_usage_snapshot_unsafe_ffi") {
        if let Some(summary) = unsafe_ffi_data.get("summary") {
            println!("📊 Unsafe/FFI Summary: {summary}");
        }
    }
    

    // Try multiple possible paths for the template files - prioritize the original dashboard.html
    let template_paths = [
        // Primary: Use the original dashboard.html template
        "templates/dashboard.html",
        "./templates/dashboard.html",
        "../templates/dashboard.html",
        "../../templates/dashboard.html",
        // Fallback: Self-contained version if available
        "templates/dashboard_self_contained.html",
        "./templates/dashboard_self_contained.html",
        "../templates/dashboard_self_contained.html",
        "../../templates/dashboard_self_contained.html",
        // Last resort: Simple report (should be avoided)
        "simple_report.html",
        "./simple_report.html",
    ];
    
    let css_paths = [
        "templates/styles.css",
        "./templates/styles.css", 
        "../templates/styles.css",
        "../../templates/styles.css",
    ];
    
    let js_paths = [
        "templates/dashboard.js",
        "./templates/dashboard.js",
        "../templates/dashboard.js",
        "../../templates/dashboard.js",
        "templates/script.js",
        "./templates/script.js",
        "../templates/script.js",
        "../../templates/script.js",
    ];
    
    let template_content = template_paths.iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find dashboard template file in any expected location")?;
    
    // Load CSS content
    let css_content = css_paths.iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find CSS template file in any expected location")?;
    
    // Load JavaScript content
    let js_content = js_paths.iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find JavaScript template file in any expected location")?;
    
    // Replace placeholders in the template
    let html = template_content
        .replace("{{json_data}}", &json_data_str)
        .replace("{{CSS_CONTENT}}", &css_content)
        .replace("{{JS_CONTENT}}", &js_content)
        .replace("{{DATA_PLACEHOLDER}}", &json_data_str);
    
    println!("✅ Generated HTML with {} bytes of embedded JSON data", json_data_str.len());
    
    Ok(html)
}

/// Transform the raw JSON data structure to match JavaScript expectations
fn transform_json_data_structure(json_data: &HashMap<String, Value>) -> Result<serde_json::Map<String, Value>, Box<dyn Error>> {
    let mut transformed = serde_json::Map::new();
    
    // Process each JSON file and map it to the expected structure
    for (file_key, file_data) in json_data {
        // Extract the data type from the filename
        if file_key.contains("memory_analysis") {
            transformed.insert("memory_analysis".to_string(), file_data.clone());
        } else if file_key.contains("lifetime") {
            transformed.insert("lifetime".to_string(), file_data.clone());
        } else if file_key.contains("complex_types") {
            transformed.insert("complex_types".to_string(), file_data.clone());
        } else if file_key.contains("performance") {
            transformed.insert("performance".to_string(), file_data.clone());
        } else if file_key.contains("unsafe_ffi") {
            transformed.insert("unsafe_ffi".to_string(), file_data.clone());
            // Also add it with the specific key that JavaScript expects
            transformed.insert(file_key.clone(), file_data.clone());
        } else if file_key.contains("security_violations") {
            transformed.insert("security_violations".to_string(), file_data.clone());
        } else if file_key.contains("variable_relationships") {
            transformed.insert("variable_relationships".to_string(), file_data.clone());
        } else {
            // Keep any other data with its original key
            transformed.insert(file_key.clone(), file_data.clone());
        }
    }
    
    // Ensure we have all expected data structures, even if empty
    if !transformed.contains_key("memory_analysis") {
        transformed.insert("memory_analysis".to_string(), serde_json::json!({
            "allocations": [],
            "stats": {
                "total_allocations": 0,
                "active_allocations": 0,
                "total_memory": 0,
                "active_memory": 0
            }
        }));
    }
    
    if !transformed.contains_key("lifetime") {
        transformed.insert("lifetime".to_string(), serde_json::json!({
            "lifecycle_events": []
        }));
    }
    
    if !transformed.contains_key("complex_types") {
        transformed.insert("complex_types".to_string(), serde_json::json!({
            "categorized_types": {
                "generic_types": [],
                "collections": [],
                "smart_pointers": [],
                "trait_objects": []
            },
            "summary": {
                "total_complex_types": 0,
                "generic_type_count": 0
            }
        }));
    }
    
    if !transformed.contains_key("performance") {
        transformed.insert("performance".to_string(), serde_json::json!({
            "memory_performance": {
                "active_memory": 0,
                "peak_memory": 0,
                "total_allocated": 0
            },
            "allocation_distribution": {
                "tiny": 0,
                "small": 0,
                "medium": 0,
                "large": 0,
                "massive": 0
            }
        }));
    }
    
    if !transformed.contains_key("unsafe_ffi") {
        transformed.insert("unsafe_ffi".to_string(), serde_json::json!({
            "summary": {
                "total_risk_items": 0,
                "unsafe_count": 0,
                "ffi_count": 0,
                "safety_violations": 0
            },
            "enhanced_ffi_data": [],
            "safety_violations": []
        }));
    }
    
    if !transformed.contains_key("security_violations") {
        transformed.insert("security_violations".to_string(), serde_json::json!({
            "metadata": {
                "total_violations": 0
            },
            "violation_reports": [],
            "security_summary": {
                "security_analysis_summary": {
                    "total_violations": 0,
                    "severity_breakdown": {
                        "critical": 0,
                        "high": 0,
                        "medium": 0,
                        "low": 0,
                        "info": 0
                    }
                }
            }
        }));
    }
    
    println!("🔄 Transformed data structure with keys: {:?}", transformed.keys().collect::<Vec<_>>());
    
    Ok(transformed)
}