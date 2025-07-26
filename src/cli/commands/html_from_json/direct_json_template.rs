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
        .map_err(|e| format!("Failed to serialize JSON data: {}", e))?;
    

    let template_content = std::fs::read_to_string("templates/dashboard.html")
        .map_err(|e| format!("Failed to read dashboard template: {}", e))?;
    
    // Embed the JSON data into the template
    let html = template_content.replace("{{json_data}}", &json_data_str);
    
    println!("✅ Generated HTML with {} bytes of embedded JSON data", json_data_str.len());
    
    Ok(html)
}