//! Direct JSON template generator that uses raw JSON data without complex processing

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Generate HTML directly from raw JSON data
pub fn generate_direct_html(json_data: &HashMap<String, Value>) -> Result<String, Box<dyn Error>> {
    tracing::info!("🎨 Generating enhanced HTML with embedded JSON data...");

    // Validate that we have essential data
    if json_data.is_empty() {
        return Err("No JSON data provided for HTML generation".into());
    }

    // Log what data we have
    for (key, value) in json_data {
        tracing::info!(
            "📊 Found data: {} ({} bytes)",
            key,
            serde_json::to_string(value).unwrap_or_default().len()
        );
    }

    // Transform the data structure to match JavaScript expectations
    let transformed_data = transform_json_data_structure(json_data)?;

    // Serialize the transformed JSON data for embedding with proper escaping
    let json_data_str = serde_json::to_string(&transformed_data)
        .map_err(|e| format!("Failed to serialize JSON data: {e}"))?;

    // Log data structure for debugging
    if let Some(unsafe_ffi_data) = json_data.get("basic_usage_snapshot_unsafe_ffi") {
        if let Some(summary) = unsafe_ffi_data.get("summary") {
            tracing::info!("📊 Unsafe/FFI Summary: {summary}");
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
        "templates/script.js",
        "./templates/script.js",
        "../templates/script.js",
        "../../templates/script.js",
        "templates/dashboard.js",
        "./templates/dashboard.js",
        "../templates/dashboard.js",
        "../../templates/dashboard.js",
    ];

    let template_content = template_paths
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find dashboard template file in any expected location")?;

    // Load CSS content
    let css_content = css_paths
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find CSS template file in any expected location")?;

    // Load JavaScript content
    let js_content = js_paths
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())
        .ok_or("Failed to find JavaScript template file in any expected location")?;

    // Replace placeholders in the template with proper escaping
    let html = template_content
        .replace("{{ json_data }}", &json_data_str) // with spaces
        .replace("{{json_data}}", &json_data_str) // without spaces
        .replace("{{CSS_CONTENT}}", &css_content)
        .replace("{{JS_CONTENT}}", &js_content)
        .replace("{{DATA_PLACEHOLDER}}", &json_data_str)
        .replace(
            "{{\n                {\n                CSS_CONTENT\n            }\n        }",
            &css_content,
        ) // fix CSS format issues
        .replace(
            "{\n                {\n                CSS_CONTENT\n            }\n        }",
            &css_content,
        ); // alternative format

    tracing::info!(
        "✅ Generated HTML with {} bytes of embedded JSON data",
        json_data_str.len()
    );

    Ok(html)
}

/// Transform the raw JSON data structure to match JavaScript expectations
/// This function preprocesses data in Rust to create visualization-ready structures
fn transform_json_data_structure(
    json_data: &HashMap<String, Value>,
) -> Result<serde_json::Map<String, Value>, Box<dyn Error>> {
    let mut transformed = serde_json::Map::new();

    // Process each JSON file and map it to the expected structure
    for (file_key, file_data) in json_data {
        // Extract the data type from the filename
        if file_key.contains("memory_analysis") {
            let enhanced_memory_data = enhance_memory_analysis_data(file_data)?;
            transformed.insert("memory_analysis".to_string(), enhanced_memory_data);
        } else if file_key.contains("lifetime") {
            let enhanced_lifetime_data = enhance_lifetime_data(file_data)?;
            transformed.insert("lifetime".to_string(), enhanced_lifetime_data);
        } else if file_key.contains("complex_types") {
            transformed.insert("complex_types".to_string(), file_data.clone());
        } else if file_key.contains("performance") {
            transformed.insert("performance".to_string(), file_data.clone());
        } else if file_key.contains("unsafe_ffi") {
            let enhanced_ffi_data = enhance_ffi_data(file_data)?;
            transformed.insert("unsafe_ffi".to_string(), enhanced_ffi_data);
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
        transformed.insert(
            "memory_analysis".to_string(),
            serde_json::json!({
                "allocations": [],
                "stats": {
                    "total_allocations": 0,
                    "active_allocations": 0,
                    "total_memory": 0,
                    "active_memory": 0
                }
            }),
        );
    }

    if !transformed.contains_key("lifetime") {
        transformed.insert(
            "lifetime".to_string(),
            serde_json::json!({
                "lifecycle_events": []
            }),
        );
    }

    if !transformed.contains_key("complex_types") {
        transformed.insert(
            "complex_types".to_string(),
            serde_json::json!({
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
            }),
        );
    }

    if !transformed.contains_key("performance") {
        transformed.insert(
            "performance".to_string(),
            serde_json::json!({
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
            }),
        );
    }

    if !transformed.contains_key("unsafe_ffi") {
        transformed.insert(
            "unsafe_ffi".to_string(),
            serde_json::json!({
                "summary": {
                    "total_risk_items": 0,
                    "unsafe_count": 0,
                    "ffi_count": 0,
                    "safety_violations": 0
                },
                "enhanced_ffi_data": [],
                "safety_violations": []
            }),
        );
    }

    if !transformed.contains_key("security_violations") {
        transformed.insert(
            "security_violations".to_string(),
            serde_json::json!({
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
            }),
        );
    }

    tracing::info!(
        "🔄 Transformed data structure with keys: {:?}",
        transformed.keys().collect::<Vec<_>>()
    );

    Ok(transformed)
}

/// Enhance memory analysis data with visualization-ready structures
fn enhance_memory_analysis_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    if let Some(allocations) = data.get("allocations").and_then(|a| a.as_array()) {
        // Add memory fragmentation analysis
        let fragmentation_data = analyze_memory_fragmentation(allocations);

        // Add memory growth trends
        let growth_trends = analyze_memory_growth_trends(allocations);

        // Create enhanced structure
        if let Some(obj) = enhanced.as_object_mut() {
            obj.insert("fragmentation_analysis".to_string(), fragmentation_data);
            obj.insert("growth_trends".to_string(), growth_trends);
            obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        }
    }

    Ok(enhanced)
}

/// Enhance lifetime data with colorful progress bar information
fn enhance_lifetime_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    if let Some(events) = data.get("lifecycle_events").and_then(|e| e.as_array()) {
        // Filter for user-defined variables
        let user_variables: Vec<&Value> = events
            .iter()
            .filter(|event| {
                event
                    .get("var_name")
                    .and_then(|v| v.as_str())
                    .map_or(false, |s| s != "unknown")
                    && event
                        .get("type_name")
                        .and_then(|v| v.as_str())
                        .map_or(false, |s| s != "unknown")
            })
            .collect();

        // Group by variable name and add color information
        let mut variable_groups = std::collections::HashMap::new();
        for (index, event) in user_variables.iter().enumerate() {
            if let Some(var_name) = event.get("var_name").and_then(|v| v.as_str()) {
                let color_index = index % 10; // 10 colors in palette
                let color = get_progress_color(color_index);

                let group = variable_groups
                    .entry(var_name.to_string())
                    .or_insert_with(|| {
                        serde_json::json!({
                            "var_name": var_name,
                            "type_name": event.get("type_name"),
                            "color": color,
                            "color_index": color_index,
                            "events": []
                        })
                    });

                if let Some(events_array) = group.get_mut("events").and_then(|e| e.as_array_mut()) {
                    events_array.push((*event).clone());
                }
            }
        }

        // Convert to array and add to enhanced data
        let grouped_variables: Vec<Value> = variable_groups.into_values().collect();

        if let Some(obj) = enhanced.as_object_mut() {
            obj.insert(
                "variable_groups".to_string(),
                serde_json::json!(grouped_variables),
            );
            obj.insert(
                "user_variables_count".to_string(),
                serde_json::json!(user_variables.len()),
            );
            obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        }
    }

    Ok(enhanced)
}

/// Enhance FFI data with comprehensive analysis
fn enhance_ffi_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    let empty_vec = vec![];
    let enhanced_data = data
        .get("enhanced_ffi_data")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);
    let boundary_events = data
        .get("boundary_events")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);

    // Calculate comprehensive statistics
    let stats = calculate_ffi_statistics(enhanced_data, boundary_events);

    // Analyze language interactions
    let language_interactions = analyze_language_interactions(boundary_events);

    // Safety analysis
    let safety_analysis = analyze_safety_metrics(enhanced_data);

    if let Some(obj) = enhanced.as_object_mut() {
        obj.insert("comprehensive_stats".to_string(), stats);
        obj.insert("language_interactions".to_string(), language_interactions);
        obj.insert("safety_analysis".to_string(), safety_analysis);
        obj.insert("visualization_ready".to_string(), serde_json::json!(true));
    }

    Ok(enhanced)
}

/// Analyze memory fragmentation from allocations
fn analyze_memory_fragmentation(allocations: &[Value]) -> Value {
    let mut sorted_allocs: Vec<_> = allocations
        .iter()
        .filter_map(|alloc| {
            let ptr_str = alloc.get("ptr")?.as_str()?;
            let size = alloc.get("size")?.as_u64()? as usize;
            let address = u64::from_str_radix(ptr_str.trim_start_matches("0x"), 16).ok()?;
            Some((address, size))
        })
        .collect();

    sorted_allocs.sort_by_key(|&(addr, _)| addr);

    let mut gaps = 0;
    let mut total_gap_size = 0u64;

    for i in 1..sorted_allocs.len() {
        let (prev_addr, prev_size) = sorted_allocs[i - 1];
        let (curr_addr, _) = sorted_allocs[i];
        let prev_end = prev_addr + prev_size as u64;

        if curr_addr > prev_end {
            gaps += 1;
            total_gap_size += curr_addr - prev_end;
        }
    }

    let total_memory: u64 = sorted_allocs.iter().map(|(_, size)| *size as u64).sum();
    let fragmentation_score = if total_memory > 0 {
        ((total_gap_size as f64 / (total_memory + total_gap_size) as f64) * 100.0) as u32
    } else {
        0
    };

    let largest_block = sorted_allocs
        .iter()
        .map(|(_, size)| *size)
        .max()
        .unwrap_or(0);

    serde_json::json!({
        "total_blocks": sorted_allocs.len(),
        "fragmentation_score": fragmentation_score,
        "largest_block": largest_block,
        "gaps": gaps,
        "total_gap_size": total_gap_size,
        "analysis": get_fragmentation_analysis(fragmentation_score)
    })
}

/// Analyze memory growth trends
fn analyze_memory_growth_trends(allocations: &[Value]) -> Value {
    let mut sorted_allocs: Vec<_> = allocations
        .iter()
        .filter_map(|alloc| {
            let timestamp = alloc.get("timestamp_alloc")?.as_u64()?;
            let size = alloc.get("size")?.as_u64()? as usize;
            Some((timestamp, size))
        })
        .collect();

    sorted_allocs.sort_by_key(|&(timestamp, _)| timestamp);

    let mut cumulative_memory = 0;
    let time_points: Vec<_> = sorted_allocs
        .iter()
        .enumerate()
        .map(|(index, &(timestamp, size))| {
            cumulative_memory += size;
            serde_json::json!({
                "timestamp": timestamp,
                "memory": cumulative_memory,
                "index": index
            })
        })
        .take(100) // Limit for performance
        .collect();

    let peak_memory = time_points
        .iter()
        .filter_map(|p| p.get("memory")?.as_u64())
        .max()
        .unwrap_or(0);

    let current_memory = time_points
        .last()
        .and_then(|p| p.get("memory")?.as_u64())
        .unwrap_or(0);

    let start_memory = time_points
        .first()
        .and_then(|p| p.get("memory")?.as_u64())
        .unwrap_or(0);

    let growth_rate = if start_memory > 0 {
        ((current_memory as f64 - start_memory as f64) / start_memory as f64 * 100.0) as i32
    } else {
        0
    };

    let time_span = if time_points.len() > 1 {
        let start_time = time_points[0].get("timestamp").unwrap().as_u64().unwrap();
        let end_time = time_points
            .last()
            .unwrap()
            .get("timestamp")
            .unwrap()
            .as_u64()
            .unwrap();
        (end_time - start_time) / 1_000_000_000 // Convert to seconds
    } else {
        1
    };

    let allocation_rate = if time_span > 0 {
        allocations.len() as u64 / time_span
    } else {
        0
    };

    serde_json::json!({
        "peak_memory": peak_memory,
        "current_memory": current_memory,
        "growth_rate": growth_rate,
        "allocation_rate": allocation_rate,
        "time_points": time_points,
        "analysis": get_trend_analysis(growth_rate)
    })
}

/// Calculate comprehensive FFI statistics
fn calculate_ffi_statistics(enhanced_data: &[Value], boundary_events: &[Value]) -> Value {
    let unsafe_allocations = enhanced_data
        .iter()
        .filter(|item| {
            !item
                .get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let ffi_allocations = enhanced_data
        .iter()
        .filter(|item| {
            item.get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let boundary_crossings = boundary_events.len();

    let safety_violations = enhanced_data
        .iter()
        .map(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_u64())
                .unwrap_or(0)
        })
        .sum::<u64>();

    let unsafe_memory = enhanced_data
        .iter()
        .map(|item| item.get("size").and_then(|s| s.as_u64()).unwrap_or(0))
        .sum::<u64>();

    serde_json::json!({
        "unsafe_allocations": unsafe_allocations,
        "ffi_allocations": ffi_allocations,
        "boundary_crossings": boundary_crossings,
        "safety_violations": safety_violations,
        "unsafe_memory": unsafe_memory
    })
}

/// Analyze language interactions from boundary events
fn analyze_language_interactions(boundary_events: &[Value]) -> Value {
    let mut interactions = std::collections::HashMap::new();

    for event in boundary_events {
        if let (Some(from), Some(to)) = (
            event.get("from_context").and_then(|f| f.as_str()),
            event.get("to_context").and_then(|t| t.as_str()),
        ) {
            let key = format!("{} → {}", from, to);
            *interactions.entry(key).or_insert(0) += 1;
        }
    }

    let interactions_vec: Vec<_> = interactions
        .into_iter()
        .map(|(interaction, count)| {
            serde_json::json!({
                "interaction": interaction,
                "count": count
            })
        })
        .collect();

    serde_json::json!(interactions_vec)
}

/// Analyze safety metrics
fn analyze_safety_metrics(enhanced_data: &[Value]) -> Value {
    let safe_operations = enhanced_data
        .iter()
        .filter(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_u64())
                .unwrap_or(0)
                == 0
        })
        .count();

    let unsafe_operations = enhanced_data.len() - safe_operations;
    let total_operations = enhanced_data.len();

    let safety_percentage = if total_operations > 0 {
        (safe_operations as f64 / total_operations as f64 * 100.0) as u32
    } else {
        100
    };

    serde_json::json!({
        "safe_operations": safe_operations,
        "unsafe_operations": unsafe_operations,
        "total_operations": total_operations,
        "safety_percentage": safety_percentage
    })
}

/// Get progress bar color by index
fn get_progress_color(index: usize) -> &'static str {
    const COLORS: &[&str] = &[
        "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#feca57", "#ff9ff3", "#54a0ff", "#5f27cd",
        "#00d2d3", "#ff9f43",
    ];
    COLORS[index % COLORS.len()]
}

/// Get fragmentation analysis text
fn get_fragmentation_analysis(score: u32) -> &'static str {
    match score {
        0..=9 => "Excellent memory layout with minimal fragmentation.",
        10..=24 => "Good memory layout with low fragmentation.",
        25..=49 => "Moderate fragmentation detected. Consider memory pool allocation.",
        _ => "High fragmentation detected. Memory layout optimization recommended.",
    }
}

/// Get trend analysis text
fn get_trend_analysis(growth_rate: i32) -> &'static str {
    match growth_rate {
        i32::MIN..=-1 => "Memory usage is decreasing - good memory management.",
        0..=9 => "Stable memory usage with minimal growth.",
        10..=49 => "Moderate memory growth - monitor for potential leaks.",
        _ => "High memory growth detected - investigate for memory leaks.",
    }
}
