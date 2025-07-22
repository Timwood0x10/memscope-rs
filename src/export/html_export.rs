//! HTML export functionality for interactive memory visualization
//! Generates self-contained HTML files with embedded CSS/JS for offline viewing

use crate::analysis::unsafe_ffi_tracker::UnsafeFFITracker;
use crate::core::tracker::MemoryTracker;
use crate::core::types::{AllocationInfo, MemoryStats, TrackingError, TrackingResult};
use serde_json;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

// Embed CSS and JS content at compile time
const CSS_CONTENT: &str = include_str!("../../templates/styles.css");
const JS_CONTENT: &str = include_str!("../../templates/script.js");

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

    // Use registry to get comprehensive data with variable names
    let comprehensive_data =
        crate::variable_registry::VariableRegistry::generate_comprehensive_export(tracker)?;

    // Extract data from comprehensive structure
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

    // Convert memory_by_type to HashMap format for optimization
    let memory_by_type_map: std::collections::HashMap<String, (usize, usize)> = memory_by_type
        .iter()
        .map(|usage| {
            (
                usage.type_name.clone(),
                (usage.total_size, usage.allocation_count),
            )
        })
        .collect();

    // Prepare optimized JSON data for JavaScript with comprehensive registry data
    let json_data = prepare_comprehensive_json_data(
        &comprehensive_data,
        &active_allocations,
        &stats,
        &memory_by_type_map,
        unsafe_ffi_tracker,
    )?;

    // Generate complete HTML
    let html_content = generate_html_template(
        &memory_analysis_svg,
        &lifecycle_timeline_svg,
        &unsafe_ffi_svg,
        &json_data,
    )?;

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
    Ok(format!("data:image/svg+xml;base64,{encoded}"))
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
        result.push(if chunk.len() > 1 {
            CHARS[((b >> 6) & 63) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b & 63) as usize] as char
        } else {
            '='
        });
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
    Ok(format!("data:image/svg+xml;base64,{encoded}"))
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
    Ok(format!("data:image/svg+xml;base64,{encoded}"))
}

/// Prepare comprehensive JSON data for frontend consumption with registry-based variable names
fn prepare_comprehensive_json_data(
    _comprehensive_data: &serde_json::Value,
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
    memory_by_type: &std::collections::HashMap<String, (usize, usize)>,
    unsafe_ffi_tracker: Option<&UnsafeFFITracker>,
) -> TrackingResult<String> {
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 预处理数据，减少前端计算负担
    let processed_allocations = if allocations.len() > 1000 {
        // 大数据集：智能采样 + 代表性样本
        let mut sampled = sample_allocations(allocations, 500);
        sampled.extend(get_representative_allocations(allocations, 100));
        sampled
    } else {
        allocations.to_vec()
    };

    // 预计算类型分布和性能指标
    let type_distribution = precompute_type_distribution(&processed_allocations);
    let performance_metrics = precompute_performance_metrics(stats, &processed_allocations);

    // 转换分配数据为正确的格式
    let formatted_allocations: Vec<serde_json::Value> = processed_allocations
        .iter()
        .map(|alloc| {
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
        })
        .collect();

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
        "version": env!("CARGO_PKG_VERSION"),
        // 预处理的数据，前端直接使用，减少计算时间
        "precomputed": {
            "type_distribution": type_distribution,
            "performance_metrics": performance_metrics,
            "original_data_size": allocations.len(),
            "processed_data_size": processed_allocations.len(),
            "is_sampled": allocations.len() > 1000,
            "optimization_info": {
                "sampling_ratio": if allocations.len() > 1000 {
                    format!("{:.1}%", (processed_allocations.len() as f64 / allocations.len() as f64) * 100.0)
                } else {
                    "100%".to_string()
                },
                "load_time_estimate": if allocations.len() > 1000 { "Fast" } else { "Instant" }
            }
        }
    });

    serde_json::to_string(&json_obj)
        .map_err(|e| TrackingError::SerializationError(format!("JSON serialization failed: {e}")))
}

/// Generate complete HTML template with embedded CSS and JavaScript
fn generate_html_template(
    _memory_analysis_svg: &str,
    _lifecycle_timeline_svg: &str,
    unsafe_ffi_svg: &str,
    json_data: &str,
) -> TrackingResult<String> {
    let unsafe_ffi_html = if unsafe_ffi_svg.is_empty() {
        r#"<div class="empty-state">
            <h3>⚠️ No Unsafe/FFI Data Available</h3>
            <p>This analysis did not detect any unsafe Rust code or FFI operations.</p>
            <p>This is generally a good sign for memory safety! 🎉</p>
        </div>"#
            .to_string()
    } else {
        format!(
            r#"<div class="svg-container">
            <img src="{unsafe_ffi_svg}" alt="Unsafe FFI Dashboard" class="svg-image" />
        </div>"#
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS Interactive Memory Analysis</title>
    <style>
        {CSS_CONTENT}
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
        
        {JS_CONTENT}
    </script>
</body>
</html>"#
    );

    Ok(html)
}

/// 智能采样算法 - 保持数据代表性
fn sample_allocations(allocations: &[AllocationInfo], max_count: usize) -> Vec<AllocationInfo> {
    if allocations.len() <= max_count {
        return allocations.to_vec();
    }

    let step = allocations.len() / max_count;
    let mut sampled = Vec::new();

    for i in (0..allocations.len()).step_by(step) {
        if sampled.len() < max_count {
            sampled.push(allocations[i].clone());
        }
    }

    sampled
}

/// 获取代表性分配（最大、最小、中位数等）
fn get_representative_allocations(
    allocations: &[AllocationInfo],
    count: usize,
) -> Vec<AllocationInfo> {
    let mut sorted = allocations.to_vec();
    sorted.sort_by(|a, b| b.size.cmp(&a.size));

    let mut representatives = Vec::new();
    let step = sorted.len().max(1) / count.min(sorted.len());

    for i in (0..sorted.len()).step_by(step.max(1)) {
        if representatives.len() < count {
            representatives.push(sorted[i].clone());
        }
    }

    representatives
}

/// 预计算类型分布
fn precompute_type_distribution(allocations: &[AllocationInfo]) -> serde_json::Value {
    use std::collections::HashMap;

    let mut type_map: HashMap<String, (usize, usize)> = HashMap::new();

    for alloc in allocations {
        let type_name = alloc.type_name.clone().unwrap_or_else(|| {
            // 智能类型推断
            if alloc.size <= 8 {
                "Small Primitive".to_string()
            } else if alloc.size <= 32 {
                "Medium Object".to_string()
            } else if alloc.size <= 1024 {
                "Large Structure".to_string()
            } else {
                "Buffer/Collection".to_string()
            }
        });

        let entry = type_map.entry(type_name).or_insert((0, 0));
        entry.0 += alloc.size;
        entry.1 += 1;
    }

    let mut sorted_types: Vec<_> = type_map.into_iter().collect();
    sorted_types.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    sorted_types.truncate(10); // 只保留前10个类型

    serde_json::json!(sorted_types)
}

/// 预计算性能指标
fn precompute_performance_metrics(
    stats: &MemoryStats,
    allocations: &[AllocationInfo],
) -> serde_json::Value {
    let current_memory = stats.active_memory;
    let peak_memory = stats.peak_memory;
    let utilization = if peak_memory > 0 {
        (current_memory as f64 / peak_memory as f64 * 100.0) as u32
    } else {
        0
    };

    let total_size: usize = allocations.iter().map(|a| a.size).sum();
    let avg_size = if !allocations.is_empty() {
        total_size / allocations.len()
    } else {
        0
    };

    let large_allocs = allocations.iter().filter(|a| a.size > 1024 * 1024).count();

    serde_json::json!({
        "utilization_percent": utilization,
        "avg_allocation_size": avg_size,
        "large_allocations_count": large_allocs,
        "efficiency_score": if utilization > 80 { "HIGH" } else if utilization > 50 { "MEDIUM" } else { "LOW" },
        "fragmentation_score": if allocations.len() > 100 { "HIGH" } else { "LOW" }
    })
}
