//! HTML export functionality for interactive memory visualization
//! Generates self-contained HTML files with embedded CSS/JS for offline viewing

use crate::tracker::MemoryTracker;
use crate::types::{AllocationInfo, MemoryStats, TrackingResult, TrackingError};
use crate::unsafe_ffi_tracker::UnsafeFFITracker;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde_json;
use std::io::Read;

// Embed CSS and JS content at compile time
const CSS_CONTENT: &str = include_str!("../templates/styles.css");
const JS_CONTENT: &str = include_str!("../templates/script.js");

/// Export comprehensive interactive HTML report
pub fn export_interactive_html<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    unsafe_ffi_tracker: Option<&UnsafeFFITracker>,
    path: P,
) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting interactive HTML report to: {}", path.display());

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Collect all data
    let active_allocations = tracker.get_active_allocations()?;
    let stats = tracker.get_stats()?;
    let memory_by_type = tracker.get_memory_by_type().unwrap_or_default();

    // Generate SVGs as base64 embedded data
    let memory_analysis_svg = generate_memory_analysis_svg_data(tracker)?;
    let lifecycle_timeline_svg = generate_lifecycle_timeline_svg_data(tracker)?;
    let unsafe_ffi_svg = if let Some(ffi_tracker) = unsafe_ffi_tracker {
        generate_unsafe_ffi_svg_data(ffi_tracker)?
    } else {
        String::new()
    };

    // Prepare JSON data for JavaScript
    let json_data = prepare_json_data(&active_allocations, &stats, &memory_by_type, unsafe_ffi_tracker)?;

    // Generate complete HTML
    let html_content = generate_html_template(&memory_analysis_svg, &lifecycle_timeline_svg, &unsafe_ffi_svg, &json_data)?;

    let mut file = File::create(path)?;
    file.write_all(html_content.as_bytes())
        .map_err(|e| TrackingError::SerializationError(format!("Failed to write HTML: {e}")))?;

    tracing::info!("Successfully exported interactive HTML report");
    Ok(())
}

/// Generate memory analysis SVG as base64 data URL
fn generate_memory_analysis_svg_data(tracker: &MemoryTracker) -> TrackingResult<String> {
    use crate::visualization::export_memory_analysis;
    
    // FIXED: Also create the main SVG file with correct peak memory values
    export_memory_analysis(tracker, "moderate_unsafe_ffi_memory_analysis.svg")?;
    
    // Create temporary file for SVG
    let temp_path = "tmp_rovodev_memory_analysis.svg";
    export_memory_analysis(tracker, temp_path)?;
    
    let mut file = File::open(temp_path)?;
    let mut svg_content = String::new();
    file.read_to_string(&mut svg_content)?;
    
    // Clean up temp file
    std::fs::remove_file(temp_path).ok();
    
    // Convert to base64 data URL (simple base64 encoding)
    let encoded = base64_encode(svg_content.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
}

/// Simple base64 encoding function
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in input.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(CHARS[((b >> 18) & 63) as usize] as char);
        result.push(CHARS[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(b & 63) as usize] as char } else { '=' });
    }
    
    result
}

/// Generate lifecycle timeline SVG as base64 data URL
fn generate_lifecycle_timeline_svg_data(tracker: &MemoryTracker) -> TrackingResult<String> {
    use crate::visualization::export_lifecycle_timeline;
    
    let temp_path = "tmp_rovodev_lifecycle_timeline.svg";
    export_lifecycle_timeline(tracker, temp_path)?;
    
    let mut file = File::open(temp_path)?;
    let mut svg_content = String::new();
    file.read_to_string(&mut svg_content)?;
    
    std::fs::remove_file(temp_path).ok();
    
    let encoded = base64_encode(svg_content.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
}

/// Generate unsafe FFI SVG as base64 data URL
fn generate_unsafe_ffi_svg_data(unsafe_ffi_tracker: &UnsafeFFITracker) -> TrackingResult<String> {
    use crate::visualization::export_unsafe_ffi_dashboard;
    
    let temp_path = "tmp_rovodev_unsafe_ffi.svg";
    export_unsafe_ffi_dashboard(unsafe_ffi_tracker, temp_path)?;
    
    let mut file = File::open(temp_path)?;
    let mut svg_content = String::new();
    file.read_to_string(&mut svg_content)?;
    
    std::fs::remove_file(temp_path).ok();
    
    let encoded = base64_encode(svg_content.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
}

/// Prepare JSON data for JavaScript consumption
fn prepare_json_data(
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
    memory_by_type: &[crate::types::TypeMemoryUsage],
    unsafe_ffi_tracker: Option<&UnsafeFFITracker>,
) -> TrackingResult<String> {
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // 转换分配数据为正确的格式
    let formatted_allocations: Vec<serde_json::Value> = allocations.iter().map(|alloc| {
        json!({
            "ptr": alloc.ptr,
            "size": alloc.size,
            "timestamp": alloc.timestamp_alloc,
            "var_name": alloc.var_name.as_ref().unwrap_or(&format!("ptr_{:x}", alloc.ptr)),
            "type_name": alloc.type_name.as_ref().unwrap_or(&"Unknown".to_string()),
            "call_stack": alloc.stack_trace.as_ref().map(|stack| {
                stack.iter().map(|frame| {
                    json!({
                        "function_name": frame,
                        "file_name": "unknown",
                        "line_number": 0
                    })
                }).collect::<Vec<_>>()
            }).unwrap_or_default()
        })
    }).collect();
    
    let json_obj = json!({
        "allocations": formatted_allocations,
        "stats": stats,
        "memoryByType": memory_by_type,
        "unsafeFFI": unsafe_ffi_tracker.map(|t| {
            json!({
                "allocations": t.get_enhanced_allocations().unwrap_or_default(),
                "violations": t.get_safety_violations().unwrap_or_default(),
                "boundaryEvents": Vec::<serde_json::Value>::new(),
                "stats": serde_json::json!({})
            })
        }),
        "timestamp": timestamp,
        "version": env!("CARGO_PKG_VERSION")
    });
    
    serde_json::to_string_pretty(&json_obj)
        .map_err(|e| TrackingError::SerializationError(format!("JSON serialization failed: {e}")))
}

/// Generate complete HTML template with embedded CSS and JavaScript
fn generate_html_template(
    memory_analysis_svg: &str,
    lifecycle_timeline_svg: &str,
    unsafe_ffi_svg: &str,
    json_data: &str,
) -> TrackingResult<String> {
    let unsafe_ffi_html = if unsafe_ffi_svg.is_empty() {
        r#"<div class="empty-state">
            <h3>⚠️ No Unsafe/FFI Data Available</h3>
            <p>This analysis did not detect any unsafe Rust code or FFI operations.</p>
            <p>This is generally a good sign for memory safety! 🎉</p>
        </div>"#.to_string()
    } else {
        format!(r#"<div class="svg-container">
            <img src="{}" alt="Unsafe FFI Dashboard" class="svg-image" />
        </div>"#, unsafe_ffi_svg)
    };

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS Interactive Memory Analysis</title>
    <style>
        {css}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>🔍 MemScope-RS Interactive Memory Analysis</h1>
            <div class="header-stats">
                <span class="stat-badge" id="totalMemory">Loading...</span>
                <span class="stat-badge" id="activeAllocs">Loading...</span>
                <span class="stat-badge" id="peakMemory">Loading...</span>
            </div>
        </header>

        <nav class="tab-nav">
            <button class="tab-btn active" data-tab="overview">📊 Overview</button>
            <button class="tab-btn" data-tab="memory-analysis">🧠 Memory Analysis</button>
            <button class="tab-btn" data-tab="lifecycle">⏱️ Lifecycle Timeline</button>
            <button class="tab-btn" data-tab="unsafe-ffi">⚠️ Unsafe/FFI</button>
            <button class="tab-btn" data-tab="interactive">🎮 Interactive Explorer</button>
        </nav>

        <main class="content">
            <!-- Overview Tab -->
            <div class="tab-content active" id="overview">
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>📈 Memory Statistics</h3>
                        <div id="memoryStats">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div id="typeDistribution">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Allocations</h3>
                        <div id="recentAllocations">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        <div id="performanceInsights">Loading...</div>
                    </div>
                </div>
            </div>

            <!-- Memory Analysis Tab -->
            <div class="tab-content" id="memory-analysis">
                <!-- Dynamic visualization will be rendered here by JavaScript -->
            </div>

            <!-- Lifecycle Timeline Tab -->
            <div class="tab-content" id="lifecycle">
                <!-- Dynamic visualization will be rendered here by JavaScript -->
            </div>

            <!-- Unsafe/FFI Tab -->
            <div class="tab-content" id="unsafe-ffi">
                {unsafe_ffi_html}
            </div>

            <!-- Interactive Explorer Tab -->
            <div class="tab-content" id="interactive">
                <div class="explorer-controls">
                    <div class="control-group">
                        <label for="filterType">Filter by Type:</label>
                        <select id="filterType">
                            <option value="">All Types</option>
                        </select>
                    </div>
                    <div class="control-group">
                        <label for="sizeRange">Size Range:</label>
                        <input type="range" id="sizeRange" min="0" max="100" value="100">
                        <span id="sizeRangeValue">All sizes</span>
                    </div>
                    <div class="control-group">
                        <label for="sortBy">Sort by:</label>
                        <select id="sortBy">
                            <option value="size">Size</option>
                            <option value="timestamp">Timestamp</option>
                            <option value="type">Type</option>
                        </select>
                    </div>
                </div>
                <div class="explorer-content">
                    <div class="allocation-grid" id="allocationGrid">
                        Loading allocations...
                    </div>
                </div>
            </div>
        </main>
    </div>

    <script>
        // Embedded data
        const MEMORY_DATA = {json_data};
        
        {javascript}
    </script>
</body>
</html>"#,
        css = CSS_CONTENT,
        javascript = JS_CONTENT,
        unsafe_ffi_html = unsafe_ffi_html,
        json_data = json_data
    );

    Ok(html)
}