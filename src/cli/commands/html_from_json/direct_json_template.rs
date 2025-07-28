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
    
    // Serialize the raw JSON data for embedding with proper escaping
    let json_data_str = serde_json::to_string(json_data)
        .map_err(|e| format!("Failed to serialize JSON data: {e}"))?;
    
    // Log data structure for debugging
    if let Some(unsafe_ffi_data) = json_data.get("basic_usage_snapshot_unsafe_ffi") {
        if let Some(summary) = unsafe_ffi_data.get("summary") {
            println!("📊 Unsafe/FFI Summary: {summary}");
        }
    }
    

    // Try multiple possible paths for the template files
    let template_paths = [
        "templates/dashboard.html",
        "./templates/dashboard.html",
        "../templates/dashboard.html",
        "../../templates/dashboard.html",
    ];
    
    let css_paths = [
        "templates/styles.css",
        "./templates/styles.css", 
        "../templates/styles.css",
        "../../templates/styles.css",
    ];
    
    let template_content = template_paths.iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find dashboard template file in any expected location")?;
    
    // Load CSS content
    let css_content = css_paths.iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find CSS template file in any expected location")?;
    
    // Replace placeholders in the template
    let html = template_content
        .replace("{{json_data}}", &json_data_str)
        .replace("{{CSS_CONTENT}}", &css_content);
    
    println!("✅ Generated HTML with {} bytes of embedded JSON data", json_data_str.len());
    
    Ok(html)
}