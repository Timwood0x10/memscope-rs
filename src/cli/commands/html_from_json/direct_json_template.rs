use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

use super::html::{get_embedded_styles_css, get_html_template};
use super::js::get_embedded_script_js;

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

    // Generate safety risk data from the transformed allocations
    let safety_risk_data = generate_safety_risk_data_from_json(&transformed_data)?;

    // Serialize the transformed JSON data for embedding with proper escaping
    let json_data_str = serde_json::to_string(&transformed_data)
        .map_err(|e| format!("Failed to serialize JSON data: {e}"))?;

    // Debug: Log data serialization info
    tracing::info!(
        "📊 JSON data serialized: {} characters",
        json_data_str.len()
    );
    if let Some(memory_analysis) = transformed_data.get("memory_analysis") {
        if let Some(allocations) = memory_analysis.get("allocations") {
            if let Some(allocs_array) = allocations.as_array() {
                tracing::info!(
                    "📊 Memory analysis allocations: {} items",
                    allocs_array.len()
                );
            }
        }
    }

    // Log data structure for debugging
    if let Some(unsafe_ffi_data) = json_data.get("basic_usage_snapshot_unsafe_ffi") {
        if let Some(summary) = unsafe_ffi_data.get("summary") {
            tracing::info!("📊 Unsafe/FFI Summary: {summary}");
        }
    }

    // Try multiple possible paths for the template files - prioritize the original dashboard.html

    // Use embedded template to avoid external file dependency
    let template_content = get_html_template().to_string();

    // Use embedded CSS to avoid external file dependency
    let css_content = get_embedded_styles_css().to_string();

    // Use embedded JavaScript to avoid external file dependency
    let js_content = get_embedded_script_js().to_string();

    // Replace placeholders in the template with proper escaping
    let mut html = template_content
        .replace("{{ json_data }}", &json_data_str) // with spaces
        .replace("{{json_data}}", &json_data_str) // without spaces
        .replace("{{CSS_CONTENT}}", &css_content)
        .replace("{{JS_CONTENT}}", &js_content)
        .replace("{{DATA_PLACEHOLDER}}", &json_data_str)
        .replace(
            "{\n        {\n        CSS_CONTENT\n      }\n    }",
            &css_content,
        ); // fix CSS format issues - match exact template spacing

    // Inject safety risk data into the HTML
    html = inject_safety_risk_data_into_html(html, &safety_risk_data)?;

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
                    .is_some_and(|s| s != "unknown")
                    && event
                        .get("type_name")
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s != "unknown")
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

/// Enhance FFI data with comprehensive analysis and SVG-inspired visualization data
fn enhance_ffi_data(data: &Value) -> Result<Value, Box<dyn Error>> {
    let mut enhanced = data.clone();

    let empty_vec = vec![];
    // Use the actual allocations field from the JSON data
    let allocations = data
        .get("allocations")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);

    // Fallback to enhanced_ffi_data if allocations is not found
    let enhanced_data = if allocations.is_empty() {
        data.get("enhanced_ffi_data")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec)
    } else {
        allocations
    };

    let boundary_events = data
        .get("boundary_events")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty_vec);

    tracing::info!(
        "🔍 FFI data enhancement - allocations: {}, enhanced_data: {}, boundary_events: {}",
        allocations.len(),
        enhanced_data.len(),
        boundary_events.len()
    );

    // Calculate comprehensive statistics using the actual allocations
    let stats = calculate_ffi_statistics_from_allocations(enhanced_data, boundary_events);

    // Analyze language interactions
    let language_interactions = analyze_language_interactions(boundary_events);

    // Safety analysis using actual allocations
    let safety_analysis = analyze_safety_metrics_from_allocations(enhanced_data);

    // Create SVG-inspired dashboard metrics
    let dashboard_metrics = create_ffi_dashboard_metrics(enhanced_data, boundary_events);

    // Create memory hotspots analysis
    let memory_hotspots = analyze_memory_hotspots(enhanced_data);

    // Create cross-language memory flow analysis
    let memory_flow = analyze_cross_language_memory_flow(enhanced_data, boundary_events);

    // Create risk assessment
    let risk_assessment = create_ffi_risk_assessment(enhanced_data);

    if let Some(obj) = enhanced.as_object_mut() {
        obj.insert("comprehensive_stats".to_string(), stats);
        obj.insert("language_interactions".to_string(), language_interactions);
        obj.insert("safety_analysis".to_string(), safety_analysis);
        obj.insert("dashboard_metrics".to_string(), dashboard_metrics);
        obj.insert("memory_hotspots".to_string(), memory_hotspots);
        obj.insert("memory_flow".to_string(), memory_flow);
        obj.insert("risk_assessment".to_string(), risk_assessment);
        obj.insert("visualization_ready".to_string(), serde_json::json!(true));
        // Ensure allocations are preserved in the enhanced data
        if !allocations.is_empty() {
            obj.insert("allocations".to_string(), serde_json::json!(allocations));
        }
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
        let start_time = time_points[0]
            .get("timestamp")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        let end_time = time_points
            .last()
            .and_then(|p| p.get("timestamp"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0);
        if end_time > start_time {
            (end_time - start_time) / 1_000_000_000 // Convert to seconds
        } else {
            1
        }
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

/// Calculate comprehensive FFI statistics from allocations
fn calculate_ffi_statistics_from_allocations(
    allocations: &[Value],
    boundary_events: &[Value],
) -> Value {
    let ffi_tracked_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let non_ffi_allocations = allocations.len() - ffi_tracked_allocations;

    let boundary_crossings = boundary_events.len();

    // Count safety violations from arrays
    let safety_violations = allocations
        .iter()
        .map(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.len() as u64)
                .unwrap_or(0)
        })
        .sum::<u64>();

    // Count borrow conflicts
    let borrow_conflicts = allocations
        .iter()
        .filter(|item| {
            if let Some(borrow_info) = item.get("borrow_info") {
                let immutable = borrow_info
                    .get("immutable_borrows")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let mutable = borrow_info
                    .get("mutable_borrows")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                immutable > 0 && mutable > 0
            } else {
                false
            }
        })
        .count();

    // Count clones
    let total_clones = allocations
        .iter()
        .map(|item| {
            item.get("clone_info")
                .and_then(|c| c.get("clone_count"))
                .and_then(|cc| cc.as_u64())
                .unwrap_or(0)
        })
        .sum::<u64>();

    let total_memory = allocations
        .iter()
        .map(|item| item.get("size").and_then(|s| s.as_u64()).unwrap_or(0))
        .sum::<u64>();

    serde_json::json!({
        "total_allocations": allocations.len(),
        "ffi_tracked_allocations": ffi_tracked_allocations,
        "non_ffi_allocations": non_ffi_allocations,
        "boundary_crossings": boundary_crossings,
        "safety_violations": safety_violations,
        "borrow_conflicts": borrow_conflicts,
        "total_clones": total_clones,
        "total_memory": total_memory
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
            let key = format!("{from} → {to}");
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

/// Analyze safety metrics from allocations
fn analyze_safety_metrics_from_allocations(allocations: &[Value]) -> Value {
    let safe_operations = allocations
        .iter()
        .filter(|item| {
            // Check if safety_violations array is empty
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.is_empty())
                .unwrap_or(true)
        })
        .count();

    let unsafe_operations = allocations.len() - safe_operations;
    let total_operations = allocations.len();

    let safety_percentage = if total_operations > 0 {
        (safe_operations as f64 / total_operations as f64 * 100.0) as u32
    } else {
        100
    };

    // Count allocations with ownership history
    let with_ownership_history = allocations
        .iter()
        .filter(|item| {
            item.get("ownership_history_available")
                .and_then(|o| o.as_bool())
                .unwrap_or(false)
        })
        .count();

    // Count leaked allocations
    let leaked_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("is_leaked")
                .and_then(|l| l.as_bool())
                .unwrap_or(false)
        })
        .count();

    serde_json::json!({
        "safe_operations": safe_operations,
        "unsafe_operations": unsafe_operations,
        "total_operations": total_operations,
        "safety_percentage": safety_percentage,
        "with_ownership_history": with_ownership_history,
        "leaked_allocations": leaked_allocations
    })
}

/// Analyze safety metrics (legacy function for backward compatibility)
fn _analyze_safety_metrics(enhanced_data: &[Value]) -> Value {
    analyze_safety_metrics_from_allocations(enhanced_data)
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

/// Format memory size for display
fn format_memory_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {unit}", unit = UNITS[unit_index])
    } else {
        format!("{:.1} {unit}", size, unit = UNITS[unit_index])
    }
}

/// Calculate risk level for memory allocation
fn calculate_risk_level(size: u64, is_unsafe: bool, is_ffi: bool) -> String {
    if is_unsafe {
        "HIGH".to_string()
    } else if is_ffi && size > 1024 * 1024 {
        "MEDIUM".to_string()
    } else if is_ffi {
        "LOW".to_string()
    } else {
        "SAFE".to_string()
    }
}

/// Create FFI dashboard metrics inspired by SVG design
fn create_ffi_dashboard_metrics(allocations: &[Value], boundary_events: &[Value]) -> Value {
    let total_allocations = allocations.len();

    // Count unsafe allocations (those with safety violations)
    let unsafe_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        })
        .count();

    // Count FFI-tracked allocations
    let ffi_allocations = allocations
        .iter()
        .filter(|item| {
            item.get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    // Count boundary crossings
    let boundary_crossings = boundary_events.len();

    // Count safety violations
    let safety_violations = allocations
        .iter()
        .map(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0)
        })
        .sum::<usize>();

    // Calculate total unsafe memory
    let unsafe_memory: u64 = allocations
        .iter()
        .filter(|item| {
            item.get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        })
        .map(|item| item.get("size").and_then(|s| s.as_u64()).unwrap_or(0))
        .sum();

    // Calculate safety score
    let safety_score = if total_allocations > 0 {
        ((total_allocations - unsafe_allocations) as f64 / total_allocations as f64 * 100.0) as u32
    } else {
        100
    };

    // Analyze smart pointer types
    let smart_pointer_types = analyze_smart_pointer_types(allocations);

    // Analyze borrow checker metrics
    let borrow_metrics = analyze_borrow_checker_metrics(allocations);

    serde_json::json!({
        "unsafe_allocations": unsafe_allocations,
        "ffi_allocations": ffi_allocations,
        "boundary_crossings": boundary_crossings,
        "safety_violations": safety_violations,
        "unsafe_memory": unsafe_memory,
        "total_allocations": total_allocations,
        "safety_score": safety_score,
        "unsafe_memory_formatted": format_memory_size(unsafe_memory),
        "smart_pointer_types": smart_pointer_types,
        "borrow_metrics": borrow_metrics
    })
}

/// Analyze smart pointer types distribution
fn analyze_smart_pointer_types(allocations: &[Value]) -> Value {
    let mut type_counts = std::collections::HashMap::new();

    for allocation in allocations {
        if let Some(type_name) = allocation.get("type_name").and_then(|t| t.as_str()) {
            if type_name.contains("Arc")
                || type_name.contains("Rc")
                || type_name.contains("Box")
                || type_name.contains("RefCell")
            {
                // Extract the main type name
                let short_type = if type_name.contains("Arc") {
                    "Arc"
                } else if type_name.contains("Rc") {
                    "Rc"
                } else if type_name.contains("Box") {
                    "Box"
                } else if type_name.contains("RefCell") {
                    "RefCell"
                } else {
                    "Other"
                };

                *type_counts.entry(short_type.to_string()).or_insert(0) += 1;
            }
        }
    }

    serde_json::json!(type_counts)
}

/// Analyze borrow checker metrics
fn analyze_borrow_checker_metrics(allocations: &[Value]) -> Value {
    let mut max_concurrent = 0;
    let mut total_borrows = 0;
    let mut conflicts = 0;

    for allocation in allocations {
        if let Some(borrow_info) = allocation.get("borrow_info") {
            if let Some(max_concurrent_borrows) = borrow_info
                .get("max_concurrent_borrows")
                .and_then(|m| m.as_u64())
            {
                max_concurrent = max_concurrent.max(max_concurrent_borrows);
            }

            let immutable = borrow_info
                .get("immutable_borrows")
                .and_then(|i| i.as_u64())
                .unwrap_or(0);
            let mutable = borrow_info
                .get("mutable_borrows")
                .and_then(|m| m.as_u64())
                .unwrap_or(0);

            total_borrows += immutable + mutable;

            // Check for conflicts (both immutable and mutable borrows)
            if immutable > 0 && mutable > 0 {
                conflicts += 1;
            }
        }
    }

    serde_json::json!({
        "max_concurrent_borrows": max_concurrent,
        "total_borrow_operations": total_borrows,
        "borrow_conflicts": conflicts
    })
}

/// Analyze memory hotspots for visualization
fn analyze_memory_hotspots(allocations: &[Value]) -> Value {
    let mut hotspots = Vec::new();

    for allocation in allocations {
        if let (Some(size), Some(ptr), Some(type_name)) = (
            allocation.get("size").and_then(|s| s.as_u64()),
            allocation.get("ptr").and_then(|p| p.as_str()),
            allocation.get("type_name").and_then(|t| t.as_str()),
        ) {
            let is_unsafe = allocation
                .get("safety_violations")
                .and_then(|s| s.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);

            let is_ffi = allocation
                .get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false);

            hotspots.push(serde_json::json!({
                "ptr": ptr,
                "size": size,
                "type_name": type_name,
                "is_unsafe": is_unsafe,
                "is_ffi": is_ffi,
                "category": if is_unsafe { "UNSAFE" } else { "FFI" },
                "size_formatted": format_memory_size(size),
                "risk_level": calculate_risk_level(size, is_unsafe, is_ffi)
            }));
        }
    }

    // Sort by size descending
    hotspots.sort_by(|a, b| {
        let size_a = a.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
        let size_b = b.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
        size_b.cmp(&size_a)
    });

    serde_json::json!(hotspots)
}

/// Analyze cross-language memory flow
fn analyze_cross_language_memory_flow(allocations: &[Value], boundary_events: &[Value]) -> Value {
    let rust_allocations = allocations
        .iter()
        .filter(|item| {
            !item
                .get("ffi_tracked")
                .and_then(|f| f.as_bool())
                .unwrap_or(false)
        })
        .count();

    let ffi_allocations = allocations.len() - rust_allocations;

    // Analyze flow directions from boundary events
    let mut rust_to_ffi = 0;
    let mut ffi_to_rust = 0;

    for event in boundary_events {
        if let (Some(from), Some(to)) = (
            event.get("from_context").and_then(|f| f.as_str()),
            event.get("to_context").and_then(|t| t.as_str()),
        ) {
            match (from, to) {
                ("rust", "ffi") | ("rust", "c") => rust_to_ffi += 1,
                ("ffi", "rust") | ("c", "rust") => ffi_to_rust += 1,
                _ => {}
            }
        }
    }

    serde_json::json!({
        "rust_allocations": rust_allocations,
        "ffi_allocations": ffi_allocations,
        "rust_to_ffi_flow": rust_to_ffi,
        "ffi_to_rust_flow": ffi_to_rust,
        "total_boundary_crossings": boundary_events.len()
    })
}

/// Create FFI risk assessment
fn create_ffi_risk_assessment(allocations: &[Value]) -> Value {
    let mut risk_items = Vec::new();

    for allocation in allocations {
        let empty_vec = vec![];
        let safety_violations = allocation
            .get("safety_violations")
            .and_then(|s| s.as_array())
            .unwrap_or(&empty_vec);

        if !safety_violations.is_empty() {
            for violation in safety_violations {
                if let Some(violation_str) = violation.as_str() {
                    risk_items.push(serde_json::json!({
                        "type": "safety_violation",
                        "description": violation_str,
                        "severity": get_violation_severity(violation_str),
                        "ptr": allocation.get("ptr"),
                        "size": allocation.get("size")
                    }));
                }
            }
        }

        // Check for potential risks based on borrow patterns
        if let Some(borrow_info) = allocation.get("borrow_info") {
            let immutable = borrow_info
                .get("immutable_borrows")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let mutable = borrow_info
                .get("mutable_borrows")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if immutable > 0 && mutable > 0 {
                risk_items.push(serde_json::json!({
                    "type": "borrow_conflict",
                    "description": "Concurrent immutable and mutable borrows detected",
                    "severity": "medium",
                    "ptr": allocation.get("ptr"),
                    "immutable_borrows": immutable,
                    "mutable_borrows": mutable
                }));
            }
        }
    }

    // Calculate risk summary
    let critical_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("critical"))
        .count();
    let high_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("high"))
        .count();
    let medium_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("medium"))
        .count();
    let low_risks = risk_items
        .iter()
        .filter(|r| r.get("severity").and_then(|s| s.as_str()) == Some("low"))
        .count();

    serde_json::json!({
        "risk_items": risk_items,
        "summary": {
            "total_risks": risk_items.len(),
            "critical": critical_risks,
            "high": high_risks,
            "medium": medium_risks,
            "low": low_risks
        }
    })
}

/// Get violation severity
fn get_violation_severity(violation: &str) -> &'static str {
    match violation.to_lowercase().as_str() {
        v if v.contains("double free") || v.contains("use after free") => "critical",
        v if v.contains("invalid free") || v.contains("buffer overflow") => "high",
        v if v.contains("memory leak") || v.contains("uninitialized") => "medium",
        _ => "low",
    }
}

/// Generate safety risk data from JSON data structure
fn generate_safety_risk_data_from_json(
    transformed_data: &serde_json::Map<String, Value>,
) -> Result<String, Box<dyn Error>> {
    let mut safety_risks = Vec::new();

    // Extract allocations from memory_analysis
    if let Some(memory_analysis) = transformed_data.get("memory_analysis") {
        if let Some(allocations) = memory_analysis
            .get("allocations")
            .and_then(|a| a.as_array())
        {
            for allocation in allocations {
                // Check for potential unsafe operations based on allocation patterns

                // 1. Large allocations that might indicate unsafe buffer operations
                if let Some(size) = allocation.get("size").and_then(|s| s.as_u64()) {
                    if size > 1024 * 1024 {
                        // > 1MB
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Large Memory Allocation",
                            "risk_level": "Medium",
                            "description": format!("Large allocation of {} bytes may indicate unsafe buffer operations", size)
                        }));
                    }
                }

                // 2. Leaked memory indicates potential unsafe operations
                if let Some(is_leaked) = allocation.get("is_leaked").and_then(|l| l.as_bool()) {
                    if is_leaked {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Memory Leak",
                            "risk_level": "High",
                            "description": "Memory leak detected - potential unsafe memory management"
                        }));
                    }
                }

                // 3. High borrow count might indicate unsafe sharing
                if let Some(borrow_count) = allocation.get("borrow_count").and_then(|b| b.as_u64())
                {
                    if borrow_count > 10 {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "High Borrow Count",
                            "risk_level": "Medium",
                            "description": format!("High borrow count ({}) may indicate unsafe sharing patterns", borrow_count)
                        }));
                    }
                }

                // 4. Raw pointer types indicate direct unsafe operations
                if let Some(type_name) = allocation.get("type_name").and_then(|t| t.as_str()) {
                    if type_name.contains("*mut") || type_name.contains("*const") {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Raw Pointer Usage",
                            "risk_level": "High",
                            "description": format!("Raw pointer type '{}' requires unsafe operations", type_name)
                        }));
                    }

                    // 5. FFI-related types
                    if type_name.contains("CString")
                        || type_name.contains("CStr")
                        || type_name.contains("c_void")
                        || type_name.contains("extern")
                    {
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "FFI Boundary Crossing",
                            "risk_level": "Medium",
                            "description": format!("FFI type '{}' crosses safety boundaries", type_name)
                        }));
                    }
                }

                // 6. Very short-lived allocations might indicate unsafe temporary operations
                if let Some(lifetime_ms) = allocation.get("lifetime_ms").and_then(|l| l.as_u64()) {
                    if lifetime_ms < 1 {
                        // Less than 1ms
                        safety_risks.push(serde_json::json!({
                            "location": format!("{}::{}",
                                allocation.get("scope_name").and_then(|s| s.as_str()).unwrap_or("unknown"),
                                allocation.get("var_name").and_then(|s| s.as_str()).unwrap_or("unnamed")),
                            "operation": "Short-lived Allocation",
                            "risk_level": "Low",
                            "description": format!("Very short lifetime ({}ms) may indicate unsafe temporary operations", lifetime_ms)
                        }));
                    }
                }
            }
        }
    }

    // Check unsafe_ffi data for additional risks
    if let Some(unsafe_ffi) = transformed_data.get("unsafe_ffi") {
        if let Some(safety_violations) = unsafe_ffi
            .get("safety_violations")
            .and_then(|sv| sv.as_array())
        {
            for violation in safety_violations {
                if let Some(violation_type) =
                    violation.get("violation_type").and_then(|vt| vt.as_str())
                {
                    let severity = get_violation_severity(violation_type);
                    let risk_level = match severity {
                        "critical" => "High",
                        "high" => "High",
                        "medium" => "Medium",
                        _ => "Low",
                    };

                    safety_risks.push(serde_json::json!({
                        "location": violation.get("location").and_then(|l| l.as_str()).unwrap_or("Unknown"),
                        "operation": format!("Safety Violation: {violation_type}"),
                        "risk_level": risk_level,
                        "description": violation.get("description").and_then(|d| d.as_str()).unwrap_or("Safety violation detected")
                    }));
                }
            }
        }
    }

    // If no risks found, add a placeholder to show the system is working
    if safety_risks.is_empty() {
        safety_risks.push(serde_json::json!({
            "location": "Global Analysis",
            "operation": "Safety Scan Complete",
            "risk_level": "Low",
            "description": "No significant safety risks detected in current allocations"
        }));
    }

    serde_json::to_string(&safety_risks)
        .map_err(|e| format!("Failed to serialize safety risk data: {e}").into())
}

/// Inject safety risk data into HTML template
fn inject_safety_risk_data_into_html(
    mut html: String,
    safety_risk_data: &str,
) -> Result<String, Box<dyn Error>> {
    // Replace the safety risk data in the existing template
    html = html.replace(
        "window.safetyRisks = [];",
        &format!("window.safetyRisks = {safety_risk_data};"),
    );

    // Always ensure loadSafetyRisks function is available
    if !html.contains("function loadSafetyRisks") {
        // Find a good injection point - before the closing </script> tag
        if let Some(script_end) = html.rfind("</script>") {
            let before = &html[..script_end];
            let after = &html[script_end..];

            let safety_function_injection = r#"
    // Safety Risk Data Management Function
    function loadSafetyRisks() {
        console.log("🛡️ Loading safety risk data...");
        const unsafeTable = document.getElementById('unsafeTable');
        if (!unsafeTable) {
            console.warn('⚠️ unsafeTable not found');
            return;
        }
        
        const risks = window.safetyRisks || [];
        if (risks.length === 0) {
            unsafeTable.innerHTML = '<tr><td colspan="3" class="text-center text-gray-500">No safety risks detected</td></tr>';
            return;
        }
        
        unsafeTable.innerHTML = '';
        risks.forEach((risk, index) => {
            const row = document.createElement('tr');
            row.className = "hover:bg-gray-50 dark:hover:bg-gray-700";
            
            const riskLevelClass = risk.risk_level === 'High' ? 'text-red-600 font-bold' : 
                                 risk.risk_level === 'Medium' ? 'text-yellow-600 font-semibold' : 
                                 'text-green-600';
            
            row.innerHTML = `
                <td class="px-3 py-2 text-sm">${risk.location || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm">${risk.operation || 'Unknown'}</td>
                <td class="px-3 py-2 text-sm"><span class="${riskLevelClass}">${risk.risk_level || 'Low'}</span></td>
            `;
            unsafeTable.appendChild(row);
        });
        
        console.log("✅ Safety risks loaded:", risks.length, 'items');
    }
    
    "#;

            html = format!("{before}{safety_function_injection}{after}");
        }
    }

    // Ensure safety risks are loaded after initialization - but only call if function exists
    html = html.replace("console.log('✅ Enhanced dashboard initialized');", 
                       "console.log('✅ Enhanced dashboard initialized'); setTimeout(() => { if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); } }, 100);");

    // Also add to manual initialization if it exists - with safer replacement
    if html.contains("manualBtn.addEventListener('click', manualInitialize);") {
        html = html.replace("manualBtn.addEventListener('click', manualInitialize);", 
                           "manualBtn.addEventListener('click', function() { manualInitialize(); setTimeout(() => { if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); } }, 100); });");
    }

    // Remove any standalone loadSafetyRisks calls that might cause errors
    html = html.replace(
        "loadSafetyRisks();",
        "if (typeof loadSafetyRisks === 'function') { loadSafetyRisks(); }",
    );

    tracing::info!("📊 Safety risk data and function injected into HTML template");

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_direct_html_with_empty_data() {
        let json_data = HashMap::new();
        let result = generate_direct_html(&json_data);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No JSON data provided for HTML generation"
        );
    }

    #[test]
    fn test_transform_json_data_structure_with_empty_input() {
        let json_data = HashMap::new();
        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("memory_analysis"));
        assert!(transformed.contains_key("lifetime"));
        assert!(transformed.contains_key("complex_types"));
        assert!(transformed.contains_key("performance"));
        assert!(transformed.contains_key("unsafe_ffi"));
        assert!(transformed.contains_key("security_violations"));
    }

    #[test]
    fn test_transform_json_data_structure_with_memory_analysis() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_memory_analysis".to_string(),
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

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("memory_analysis"));
    }

    #[test]
    fn test_transform_json_data_structure_with_lifetime_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_lifetime".to_string(),
            serde_json::json!({
                "lifecycle_events": []
            }),
        );

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("lifetime"));
    }

    #[test]
    fn test_transform_json_data_structure_with_complex_types() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_complex_types".to_string(),
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

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("complex_types"));
    }

    #[test]
    fn test_transform_json_data_structure_with_performance_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_performance".to_string(),
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

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("performance"));
    }

    #[test]
    fn test_transform_json_data_structure_with_ffi_data() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_unsafe_ffi".to_string(),
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

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("unsafe_ffi"));
    }

    #[test]
    fn test_transform_json_data_structure_with_security_violations() {
        let mut json_data = HashMap::new();
        json_data.insert(
            "test_security_violations".to_string(),
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

        let result = transform_json_data_structure(&json_data);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert!(transformed.contains_key("security_violations"));
    }

    #[test]
    fn test_enhance_memory_analysis_data() {
        let data = serde_json::json!({
            "allocations": []
        });

        let result = enhance_memory_analysis_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhance_lifetime_data() {
        let data = serde_json::json!({
            "lifecycle_events": []
        });

        let result = enhance_lifetime_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhance_ffi_data() {
        let data = serde_json::json!({
            "allocations": [],
            "boundary_events": []
        });

        let result = enhance_ffi_data(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_memory_fragmentation() {
        let allocations = vec![];
        let result = analyze_memory_fragmentation(&allocations);
        assert_eq!(result["total_blocks"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_memory_growth_trends() {
        let allocations = vec![];
        let result = analyze_memory_growth_trends(&allocations);
        assert_eq!(result["peak_memory"], serde_json::json!(0));
    }

    #[test]
    fn test_calculate_ffi_statistics_from_allocations() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = calculate_ffi_statistics_from_allocations(&allocations, &boundary_events);
        assert_eq!(result["total_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_language_interactions() {
        let boundary_events = vec![];
        let result = analyze_language_interactions(&boundary_events);
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn test_analyze_safety_metrics_from_allocations() {
        let allocations = vec![];
        let result = analyze_safety_metrics_from_allocations(&allocations);
        assert_eq!(result["total_operations"], serde_json::json!(0));
    }

    #[test]
    fn test_get_progress_color() {
        let color = get_progress_color(0);
        assert_eq!(color, "#ff6b6b");

        let color = get_progress_color(10);
        assert_eq!(color, "#ff6b6b"); // Should wrap around
    }

    #[test]
    fn test_get_fragmentation_analysis() {
        let analysis = get_fragmentation_analysis(5);
        assert_eq!(
            analysis,
            "Excellent memory layout with minimal fragmentation."
        );

        let analysis = get_fragmentation_analysis(15);
        assert_eq!(analysis, "Good memory layout with low fragmentation.");

        let analysis = get_fragmentation_analysis(35);
        assert_eq!(
            analysis,
            "Moderate fragmentation detected. Consider memory pool allocation."
        );

        let analysis = get_fragmentation_analysis(60);
        assert_eq!(
            analysis,
            "High fragmentation detected. Memory layout optimization recommended."
        );
    }

    #[test]
    fn test_get_trend_analysis() {
        let analysis = get_trend_analysis(-5);
        assert_eq!(
            analysis,
            "Memory usage is decreasing - good memory management."
        );

        let analysis = get_trend_analysis(5);
        assert_eq!(analysis, "Stable memory usage with minimal growth.");

        let analysis = get_trend_analysis(25);
        assert_eq!(
            analysis,
            "Moderate memory growth - monitor for potential leaks."
        );

        let analysis = get_trend_analysis(75);
        assert_eq!(
            analysis,
            "High memory growth detected - investigate for memory leaks."
        );
    }

    #[test]
    fn test_format_memory_size() {
        let formatted = format_memory_size(1023);
        assert_eq!(formatted, "1023 B");

        let formatted = format_memory_size(1024);
        assert_eq!(formatted, "1.0 KB");

        let formatted = format_memory_size(1024 * 1024);
        assert_eq!(formatted, "1.0 MB");

        let formatted = format_memory_size(1024 * 1024 * 1024);
        assert_eq!(formatted, "1.0 GB");
    }

    #[test]
    fn test_calculate_risk_level() {
        let risk = calculate_risk_level(100, true, false);
        assert_eq!(risk, "HIGH");

        let risk = calculate_risk_level(1024 * 1024 + 1, false, true);
        assert_eq!(risk, "MEDIUM");

        let risk = calculate_risk_level(100, false, true);
        assert_eq!(risk, "LOW");

        let risk = calculate_risk_level(100, false, false);
        assert_eq!(risk, "SAFE");
    }

    #[test]
    fn test_create_ffi_dashboard_metrics() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = create_ffi_dashboard_metrics(&allocations, &boundary_events);
        assert_eq!(result["total_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_smart_pointer_types() {
        let allocations = vec![];
        let result = analyze_smart_pointer_types(&allocations);
        assert_eq!(result, serde_json::json!({}));
    }

    #[test]
    fn test_analyze_borrow_checker_metrics() {
        let allocations = vec![];
        let result = analyze_borrow_checker_metrics(&allocations);
        assert_eq!(result["max_concurrent_borrows"], serde_json::json!(0));
    }

    #[test]
    fn test_analyze_memory_hotspots() {
        let allocations = vec![];
        let result = analyze_memory_hotspots(&allocations);
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn test_analyze_cross_language_memory_flow() {
        let allocations = vec![];
        let boundary_events = vec![];
        let result = analyze_cross_language_memory_flow(&allocations, &boundary_events);
        assert_eq!(result["rust_allocations"], serde_json::json!(0));
    }

    #[test]
    fn test_create_ffi_risk_assessment() {
        let allocations = vec![];
        let result = create_ffi_risk_assessment(&allocations);
        assert_eq!(result["summary"]["total_risks"], serde_json::json!(0));
    }

    #[test]
    fn test_get_violation_severity() {
        let severity = get_violation_severity("double free detected");
        assert_eq!(severity, "critical");

        let severity = get_violation_severity("invalid free operation");
        assert_eq!(severity, "high");

        let severity = get_violation_severity("memory leak detected");
        assert_eq!(severity, "medium");

        let severity = get_violation_severity("unknown issue");
        assert_eq!(severity, "low");
    }

    #[test]
    fn test_generate_safety_risk_data_from_json() {
        let transformed_data = serde_json::Map::new();
        let result = generate_safety_risk_data_from_json(&transformed_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inject_safety_risk_data_into_html() {
        let html = "<script>window.safetyRisks = [];</script>".to_string();
        let safety_risk_data = "[]";
        let result = inject_safety_risk_data_into_html(html, safety_risk_data);
        assert!(result.is_ok());
    }
}
