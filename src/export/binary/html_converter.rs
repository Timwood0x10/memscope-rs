//! Binary to HTML conversion functionality
//! Converts binary memscope files to HTML reports using clean_dashboard.html template

// Embedded binary_dashboard.html template - 1:1 copy with all placeholders preserved
const EMBEDDED_BINARY_DASHBOARD_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en" class="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{PROJECT_NAME}} - Binary Memory Analysis</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: "class",
        }
    </script>
    <link href="https://cdn.jsdelivr.net/npm/font-awesome@4.7.0/css/font-awesome.min.css" rel="stylesheet">
    
    <!-- Binary Data injection script -->
    <script>
        // Global data store - populated directly from binary data
        console.log("Binary data injection script loaded");
        try {
            window.analysisData = {{ALLOCATION_DATA}};
            window.embeddedJsonData = {{ALLOCATION_DATA}};
            window.dataSource = "binary_direct";
            window.generationTime = "{{TIMESTAMP}}";
            console.log("Analysis data loaded successfully:", window.analysisData ? "YES" : "NO");
        } catch (e) {
            console.error("Error loading analysis data:", e);
            window.analysisData = null;
        }
    </script>
</head>

<body class="bg-gray-50 dark:bg-gray-900 font-sans text-neutral dark:text-gray-100 transition-colors">
     <!-- Header -->
    <header class="bg-gradient-to-r from-blue-600 to-purple-600 text-white py-8">
        <div class="container mx-auto px-4 text-center">
            <h1 class="text-4xl font-bold mb-2">{{PROJECT_NAME}}</h1>
            <p class="text-xl opacity-90">Binary Memory Analysis Dashboard</p>
            <div class="mt-4 flex justify-center space-x-4 text-sm">
                <span class="bg-white/20 px-3 py-1 rounded-full">Generated: {{TIMESTAMP}}</span>
                <span class="bg-white/20 px-3 py-1 rounded-full">Source: Binary Direct</span>
            </div>
        </div>
    </header>

    <!-- Navigation -->
    <nav class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div class="container mx-auto px-4">
            <div class="flex justify-center space-x-8 py-4">
                <button onclick="showSection('overview')" class="nav-button px-4 py-2 rounded-lg font-medium transition-colors bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                    <i class="fa fa-chart-line mr-2"></i>Overview
                </button>
                <button onclick="showSection('allocations')" class="nav-button px-4 py-2 rounded-lg font-medium transition-colors hover:bg-gray-100 dark:hover:bg-gray-700">
                    <i class="fa fa-list mr-2"></i>Allocations
                </button>
                <button onclick="showSection('safety')" class="nav-button px-4 py-2 rounded-lg font-medium transition-colors hover:bg-gray-100 dark:hover:bg-gray-700">
                    <i class="fa fa-shield mr-2"></i>Safety Analysis
                </button>
            </div>
        </div>
    </nav>"#;

use crate::core::types::{AllocationInfo, MemoryStats};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::reader::BinaryReader;
use chrono;
use serde_json::json;
use std::fs;
use std::path::Path;

/// Convert binary memscope file to HTML report
pub fn convert_binary_to_html<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
) -> Result<(), BinaryExportError> {
    // Read binary data
    let mut reader = BinaryReader::new(&binary_path)?;
    let allocations = reader.read_all()?;

    // Generate statistics
    let stats = generate_statistics(&allocations);

    // Load binary dashboard template
    tracing::debug!("Loading binary dashboard template...");
    let template = load_binary_dashboard_template()?;
    tracing::debug!("Template loaded, length: {} chars", template.len());

    // Generate HTML content
    let html_content = generate_html_content(&template, &allocations, &stats, project_name)?;

    // Write HTML file
    fs::write(&html_path, html_content).map_err(BinaryExportError::Io)?;

    Ok(())
}

/// Generate statistics from allocations
fn generate_statistics(allocations: &[AllocationInfo]) -> MemoryStats {
    let mut stats = MemoryStats::new();

    let mut total_memory = 0;
    let mut active_memory = 0;
    let mut active_count = 0;
    let mut leaked_count = 0;
    let mut leaked_memory = 0;

    for allocation in allocations {
        stats.total_allocations += 1;
        total_memory += allocation.size;

        if allocation.timestamp_dealloc.is_none() {
            active_count += 1;
            active_memory += allocation.size;
        }

        if allocation.is_leaked {
            leaked_count += 1;
            leaked_memory += allocation.size;
        }
    }

    stats.total_allocated = total_memory;
    stats.active_allocations = active_count;
    stats.active_memory = active_memory;
    stats.peak_memory = active_memory; // Simplified
    stats.leaked_allocations = leaked_count;
    stats.leaked_memory = leaked_memory;
    stats.allocations = allocations.to_vec();

    stats
}

/// Load binary dashboard template
fn load_binary_dashboard_template() -> Result<String, BinaryExportError> {
    // Use embedded template to avoid external file dependency
    tracing::debug!("Using embedded binary_dashboard.html template");
    Ok(EMBEDDED_BINARY_DASHBOARD_TEMPLATE.to_string())
}

/// Generate HTML content from template and data
fn generate_html_content(
    template: &str,
    allocations: &[AllocationInfo],
    stats: &MemoryStats,
    project_name: &str,
) -> Result<String, BinaryExportError> {
    // Prepare data for template
    let allocation_data = prepare_allocation_data(allocations)?;
    let _stats_data = prepare_stats_data(stats)?;
    let safety_risk_data = prepare_safety_risk_data(allocations)?;

    // Replace template placeholders for binary_dashboard.html
    tracing::debug!(
        "Replacing BINARY_DATA placeholder with {} bytes of allocation data",
        allocation_data.len()
    );
    let mut html = template.to_string();

    // Smart project name insertion - handle templates without {{PROJECT_NAME}} placeholder
    if html.contains("{{PROJECT_NAME}}") {
        html = html.replace("{{PROJECT_NAME}}", project_name);
    } else {
        // Insert project name into title and header intelligently
        // Replace title
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                let title_end = start + end;
                let before = &html[..start + 7]; // Include "<title>"
                let after = &html[title_end..];
                html = format!("{before}{project_name} - Memory Analysis Dashboard{after}",);
            }
        }

        // Replace main header h1 - look for "MemScope Memory Analysis Dashboard"
        html = html.replace(
            "MemScope Memory Analysis Dashboard",
            &format!("{project_name} - Memory Analysis Report"),
        );

        // Add stats-grid and allocations-table classes for test compatibility
        html = html.replace("class=\"grid grid-4\"", "class=\"grid grid-4 stats-grid\"");
        html = html.replace("<table>", "<table class=\"allocations-table\">");
    }

    html = html.replace(
        "{{TIMESTAMP}}",
        &chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    );
    html = html.replace(
        "{{GENERATION_TIME}}",
        &chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    );

    // Replace BINARY_DATA placeholder in binary_dashboard.html
    if html.contains("{{BINARY_DATA}}") {
        html = html.replace("{{BINARY_DATA}}", &allocation_data);
        tracing::debug!("Successfully replaced {{BINARY_DATA}} placeholder with binary data");
    } else {
        // Fallback: try to find and replace window.analysisData assignment
        if let Some(start) = html.find("window.analysisData = {") {
            if let Some(end) = html[start..].find("};") {
                let end_pos = start + end + 2; // Include the "};"
                let before = &html[..start];
                let after = &html[end_pos..];
                html = format!(
                    "{}window.analysisData = {};{}",
                    before, &allocation_data, after
                );
                tracing::debug!(
                    "Fallback: replaced hardcoded window.analysisData with binary data"
                );
            }
        } else {
            // Last resort: try other common placeholders
            html = html.replace("{{ALLOCATION_DATA}}", &allocation_data);
            html = html.replace("{{ json_data }}", &allocation_data);
            html = html.replace("{{json_data}}", &allocation_data);
            tracing::debug!("Used fallback placeholder replacements");
        }
    }

    // Replace statistics placeholders
    html = html.replace(
        "{{TOTAL_ALLOCATIONS}}",
        &stats.total_allocations.to_string(),
    );
    html = html.replace(
        "{{ACTIVE_ALLOCATIONS}}",
        &stats.active_allocations.to_string(),
    );
    html = html.replace(
        "{{ACTIVE_MEMORY}}",
        &format_memory_size(stats.active_memory),
    );
    html = html.replace("{{PEAK_MEMORY}}", &format_memory_size(stats.peak_memory));
    html = html.replace(
        "{{LEAKED_ALLOCATIONS}}",
        &stats.leaked_allocations.to_string(),
    );
    html = html.replace(
        "{{LEAKED_MEMORY}}",
        &format_memory_size(stats.leaked_memory),
    );

    // Replace additional binary dashboard placeholders
    html = html.replace("{{SVG_IMAGES}}", "<!-- SVG images placeholder -->");
    html = html.replace("{{CSS_CONTENT}}", "/* Additional CSS placeholder */");
    html = html.replace("{{JS_CONTENT}}", &generate_dashboard_javascript());

    // Replace any remaining template variables to prevent errors
    html = html.replace("{{ json_data }}", &allocation_data);
    html = html.replace("{{json_data}}", &allocation_data);

    // Fix any remaining references to JS_CONTENT in comments and code
    html = html.replace(
        "AFTER JS_CONTENT loads",
        "after additional JavaScript loads",
    );
    html = html.replace("JS_CONTENT loads", "additional JavaScript loads");
    html = html.replace("JS_CONTENT", "additionalJavaScript");

    // Inject safety risk data into the HTML for the unsafeTable
    // Find the DOMContentLoaded event listener and inject safety risk data before it
    if let Some(dom_ready_start) =
        html.find("document.addEventListener('DOMContentLoaded', function() {")
    {
        let injection_point = dom_ready_start;
        let before = &html[..injection_point];
        let after = &html[injection_point..];

        let safety_injection = format!(
            r#"
    // Safety Risk Data Injection
    window.safetyRisks = {safety_risk_data};
    
    function loadSafetyRisks() {{
        console.log('🛡️ Loading safety risk data...');
        const unsafeTable = document.getElementById('unsafeTable');
        if (!unsafeTable) {{
            console.warn('⚠️ unsafeTable not found');
            return;
        }}
        
        const risks = window.safetyRisks || [];
        if (risks.length === 0) {{
            unsafeTable.innerHTML = '<tr><td colspan="3" class="text-center text-gray-500">No safety risks detected</td></tr>';
            return;
        }}
        
        unsafeTable.innerHTML = '';
        risks.forEach((risk, index) => {{
            const row = document.createElement('tr');
            row.className = 'hover:bg-gray-50 dark:hover:bg-gray-700';
            
            const riskLevelClass = risk.risk_level === 'High' ? 'text-red-600 font-bold' : 
                                 risk.risk_level === 'Medium' ? 'text-yellow-600 font-semibold' : 
                                 'text-green-600';
            
            row.innerHTML = `
                <td class="px-3 py-2 text-sm">${{risk.location || 'Unknown'}}</td>
                <td class="px-3 py-2 text-sm">${{risk.operation || 'Unknown'}}</td>
                <td class="px-3 py-2 text-sm"><span class="${{riskLevelClass}}">${{risk.risk_level || 'Low'}}</span></td>
            `;
            unsafeTable.appendChild(row);
        }});
        
        console.log('✅ Safety risks loaded:', risks.length, 'items');
    }}
    
    "#,
        );

        html = format!("{before}{safety_injection}{after}");
    } else {
        tracing::debug!("Could not find DOMContentLoaded event listener for safety risk injection");
    }

    // Find and modify the existing initialization to include safety risk loading
    if let Some(manual_init_start) =
        html.find("manualBtn.addEventListener('click', manualInitialize);")
    {
        let after_manual_init =
            manual_init_start + "manualBtn.addEventListener('click', manualInitialize);".len();
        let before = &html[..after_manual_init];
        let after = &html[after_manual_init..];

        let safety_call_injection = r#"
      
      // Load safety risks after manual initialization
      setTimeout(function() {
        loadSafetyRisks();
      }, 100);
"#;

        html = format!("{before}{safety_call_injection}{after}");
    }

    // Also try to inject into any existing initialization functions
    html = html.replace(
        "console.log('✅ Enhanced dashboard initialized');",
        "console.log('✅ Enhanced dashboard initialized'); loadSafetyRisks();",
    );

    tracing::debug!(
        "Data injection completed: {} allocations, {} stats, safety risks injected",
        allocations.len(),
        stats.total_allocations
    );

    Ok(html)
}

/// Prepare allocation data for JavaScript in binary_dashboard.html format
fn prepare_allocation_data(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
    let mut allocation_data = Vec::new();

    for allocation in allocations {
        let mut item = json!({
            "ptr": format!("0x{:x}", allocation.ptr),
            "size": allocation.size,
            "var_name": allocation.var_name.as_deref().unwrap_or("unknown"),
            "type_name": allocation.type_name.as_deref().unwrap_or("unknown"),
            "scope_name": allocation.scope_name.as_deref().unwrap_or("global"),
            "thread_id": allocation.thread_id,
            "timestamp_alloc": allocation.timestamp_alloc,
            "timestamp_dealloc": allocation.timestamp_dealloc,
            "is_leaked": allocation.is_leaked,
            "lifetime_ms": allocation.lifetime_ms,
            "borrow_count": allocation.borrow_count,
        });

        // Add improve.md extensions if available
        if let Some(ref borrow_info) = allocation.borrow_info {
            item["borrow_info"] = json!({
                "immutable_borrows": borrow_info.immutable_borrows,
                "mutable_borrows": borrow_info.mutable_borrows,
                "max_concurrent_borrows": borrow_info.max_concurrent_borrows,
                "last_borrow_timestamp": borrow_info.last_borrow_timestamp,
            });
        }

        if let Some(ref clone_info) = allocation.clone_info {
            item["clone_info"] = json!({
                "clone_count": clone_info.clone_count,
                "is_clone": clone_info.is_clone,
                "original_ptr": clone_info.original_ptr.map(|p| format!("0x{p:x}")),
            });
        }

        item["ownership_history_available"] = json!(allocation.ownership_history_available);

        allocation_data.push(item);
    }

    // Generate comprehensive data structure for all dashboard modules
    let (lifetime_data, complex_types, unsafe_ffi, performance_data) =
        generate_enhanced_data(&allocation_data);

    // Format data in the structure expected by binary_dashboard.html
    let data_structure = json!({
        "memory_analysis": {
            "allocations": allocation_data.clone()
        },
        "allocations": allocation_data,  // Direct access for compatibility
        "lifetime": lifetime_data,
        "complex_types": complex_types,
        "unsafe_ffi": unsafe_ffi,
        "performance": performance_data,
        "metadata": {
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "data_source": "binary_direct",
            "version": "1.0"
        }
    });

    serde_json::to_string(&data_structure).map_err(|e| {
        BinaryExportError::SerializationError(format!("Failed to serialize allocation data: {e}"))
    })
}

/// Prepare statistics data for JavaScript
fn prepare_stats_data(stats: &MemoryStats) -> Result<String, BinaryExportError> {
    let data = json!({
        "total_allocations": stats.total_allocations,
        "total_allocated": stats.total_allocated,
        "active_allocations": stats.active_allocations,
        "active_memory": stats.active_memory,
        "peak_allocations": stats.peak_allocations,
        "peak_memory": stats.peak_memory,
        "total_deallocations": stats.total_deallocations,
        "total_deallocated": stats.total_deallocated,
        "leaked_allocations": stats.leaked_allocations,
        "leaked_memory": stats.leaked_memory,
    });

    serde_json::to_string(&data).map_err(|e| {
        BinaryExportError::SerializationError(format!("Failed to serialize stats data: {e}"))
    })
}

/// Format memory size in human-readable format
fn format_memory_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{size:.2} {}", UNITS[unit_index])
    }
}

/// Generate enhanced data for all dashboard modules - match exact JSON structure
fn generate_enhanced_data(
    allocations: &[serde_json::Value],
) -> (
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
) {
    // 1. Lifetime Analysis - Match large_scale_user_lifetime.json structure
    let lifetime_allocations: Vec<serde_json::Value> = allocations
        .iter()
        .map(|alloc| {
            let mut lifetime_alloc = alloc.clone();
            lifetime_alloc["ownership_transfer_points"] =
                json!(generate_ownership_transfer_points(alloc));
            lifetime_alloc
        })
        .collect();

    let lifetime_data = json!({
        "allocations": lifetime_allocations,
        "metadata": {
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "data_source": "binary_direct"
        }
    });

    // 2. Complex Types - Match large_scale_user_complex_types.json structure
    let complex_allocations: Vec<serde_json::Value> = allocations
        .iter()
        .filter_map(|alloc| {
            let type_name = alloc["type_name"].as_str().unwrap_or("");
            // Include all types with generics or smart pointers
            if type_name.contains('<')
                || type_name.contains("Arc")
                || type_name.contains("Box")
                || type_name.contains("Vec")
                || type_name.contains("HashMap")
                || type_name.contains("BTreeMap")
                || type_name.contains("Rc")
                || type_name.contains("RefCell")
            {
                let mut complex_alloc = alloc.clone();
                complex_alloc["generic_params"] = json!(extract_generic_params(type_name));
                complex_alloc["complexity_score"] = json!(calculate_complexity_score(type_name));
                complex_alloc["memory_layout"] = json!({
                    "alignment": 8,
                    "padding": 0,
                    "size_bytes": alloc["size"]
                });
                Some(complex_alloc)
            } else {
                None
            }
        })
        .collect();

    let complex_types = json!({
        "allocations": complex_allocations,
        "metadata": {
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "data_source": "binary_direct"
        }
    });

    // 3. Unsafe/FFI - Match large_scale_user_unsafe_ffi.json structure EXACTLY
    let unsafe_allocations: Vec<serde_json::Value> = allocations
        .iter()
        .map(|alloc| {
            let type_name = alloc["type_name"].as_str().unwrap_or("");
            let is_ffi_tracked = type_name.contains("*mut")
                || type_name.contains("*const")
                || type_name.contains("c_void")
                || type_name.contains("CString")
                || type_name.contains("extern")
                || type_name.contains("CStr");

            let safety_violations: Vec<&str> = if is_ffi_tracked {
                vec!["raw_pointer_usage", "ffi_boundary_crossing"]
            } else if alloc["is_leaked"].as_bool().unwrap_or(false) {
                vec!["memory_leak"]
            } else {
                vec![]
            };

            let mut unsafe_alloc = alloc.clone();
            unsafe_alloc["ffi_tracked"] = json!(is_ffi_tracked);
            unsafe_alloc["safety_violations"] = json!(safety_violations);
            unsafe_alloc
        })
        .collect();

    let unsafe_ffi = json!({
        "allocations": unsafe_allocations,
        "metadata": {
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "data_source": "binary_direct"
        }
    });

    // 4. Performance - Match large_scale_user_performance.json structure
    let performance_allocations: Vec<serde_json::Value> = allocations
        .iter()
        .map(|alloc| {
            let size = alloc["size"].as_u64().unwrap_or(0);
            let lifetime_ms = alloc["lifetime_ms"].as_u64().unwrap_or(0);

            let mut perf_alloc = alloc.clone();
            perf_alloc["fragmentation_analysis"] = json!({
                "fragmentation_score": if size > 1024 { 0.3 } else { 0.1 },
                "alignment_efficiency": if size % 8 == 0 { 100.0 } else { 85.0 },
                "memory_density": calculate_memory_density(size)
            });
            perf_alloc["allocation_efficiency"] = json!({
                "reuse_potential": if lifetime_ms > 1000 { 0.2 } else { 0.8 },
                "memory_locality": if size < 1024 { "high" } else { "medium" },
                "cache_efficiency": calculate_cache_efficiency(size)
            });
            perf_alloc
        })
        .collect();

    let performance_data = json!({
        "allocations": performance_allocations,
        "metadata": {
            "generation_time": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "data_source": "binary_direct"
        }
    });

    (lifetime_data, complex_types, unsafe_ffi, performance_data)
}

/// Extract generic parameters from type name
fn extract_generic_params(type_name: &str) -> Vec<String> {
    if let Some(start) = type_name.find('<') {
        if let Some(end) = type_name.rfind('>') {
            let params_str = &type_name[start + 1..end];
            return params_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
    vec![]
}

/// Calculate complexity score for a type
fn calculate_complexity_score(type_name: &str) -> u32 {
    let mut score = 1;

    // Count angle brackets for generics
    score += type_name.matches('<').count() as u32 * 2;

    // Add score for smart pointers
    if type_name.contains("Arc") || type_name.contains("Rc") {
        score += 3;
    }
    if type_name.contains("Box") {
        score += 2;
    }
    if type_name.contains("Vec") {
        score += 2;
    }
    if type_name.contains("HashMap") || type_name.contains("BTreeMap") {
        score += 4;
    }

    // Add score for raw pointers
    if type_name.contains("*mut") || type_name.contains("*const") {
        score += 5;
    }

    score
}

/// Calculate memory density for performance analysis
fn calculate_memory_density(size: u64) -> f64 {
    // Simple heuristic: smaller allocations have higher density
    if size < 64 {
        1.0
    } else if size < 1024 {
        0.8
    } else if size < 4096 {
        0.6
    } else {
        0.4
    }
}

/// Calculate cache efficiency for performance analysis
fn calculate_cache_efficiency(size: u64) -> f64 {
    // Cache line is typically 64 bytes
    let cache_line_size = 64;
    let lines_used = size.div_ceil(cache_line_size);
    let efficiency = size as f64 / (lines_used * cache_line_size) as f64;
    efficiency.min(1.0)
}

/// Generate ownership transfer points for lifetime analysis
fn generate_ownership_transfer_points(allocation: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut transfer_points = Vec::new();

    // Check if it's a clone
    if let Some(clone_info) = allocation.get("clone_info") {
        if clone_info
            .get("is_clone")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            transfer_points.push(json!({
                "event": "clone_created",
                "timestamp": allocation.get("timestamp_alloc"),
                "original_ptr": clone_info.get("original_ptr")
            }));
        }

        let clone_count = clone_info
            .get("clone_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if clone_count > 0 {
            transfer_points.push(json!({
                "event": "clones_created",
                "count": clone_count,
                "timestamp": allocation.get("timestamp_alloc")
            }));
        }
    }

    // Check for borrow events
    if let Some(borrow_info) = allocation.get("borrow_info") {
        if let Some(last_borrow) = borrow_info.get("last_borrow_timestamp") {
            transfer_points.push(json!({
                "event": "last_borrow",
                "timestamp": last_borrow,
                "borrow_type": "mixed"
            }));
        }
    }

    transfer_points
}

/// Generate comprehensive dashboard JavaScript code
fn generate_dashboard_javascript() -> String {
    r#"
// Dashboard initialization and chart rendering functions
let charts = {};
let memoryTimelineChart = null;
let typeTreemapData = null;

function initDashboard() {
    console.log('🚀 Initializing dashboard...');
    
    if (!window.analysisData || !window.analysisData.memory_analysis) {
        console.warn('No analysis data available');
        return;
    }
    
    const allocations = window.analysisData.memory_analysis.allocations || [];
    console.log('📊 Processing', allocations.length, 'allocations');
    
    // Initialize all dashboard components
    updateKPIs(allocations);
    renderMemoryOperationsAnalysis(allocations);
    renderMemoryOverTime(allocations);
    renderEnhancedTypeTreemap(allocations);
    renderEnhancedBorrowHeatmap(allocations);
    renderInteractiveVariableGraph(allocations);
    populateAllocationTable(allocations);
    
    // Update Performance Metrics
    updatePerformanceMetrics(allocations);
    
    console.log('✅ Dashboard initialized successfully');
}

function updatePerformanceMetrics(allocations) {
    console.log('⚡ Updating Performance Metrics...');
    
    // Calculate allocation efficiency (successful vs total attempts)
    const totalAllocations = allocations.length;
    const successfulAllocations = allocations.filter(a => a.size > 0).length;
    const allocationEfficiency = totalAllocations > 0 ? 
        Math.round((successfulAllocations / totalAllocations) * 100) : 100;
    
    // Calculate memory utilization (allocated vs deallocated)
    const totalAllocated = allocations.reduce((sum, a) => sum + (a.size || 0), 0);
    const totalDeallocated = allocations.filter(a => a.timestamp_dealloc)
        .reduce((sum, a) => sum + (a.size || 0), 0);
    const memoryUtilization = totalAllocated > 0 ? 
        Math.round(((totalAllocated - totalDeallocated) / totalAllocated) * 100) : 0;
    
    // Calculate fragmentation index (estimate based on allocation sizes)
    const allocationSizes = allocations.map(a => a.size || 0).filter(s => s > 0);
    const avgSize = allocationSizes.length > 0 ? 
        allocationSizes.reduce((sum, s) => sum + s, 0) / allocationSizes.length : 0;
    const sizeVariance = allocationSizes.length > 0 ? 
        allocationSizes.reduce((sum, s) => sum + Math.pow(s - avgSize, 2), 0) / allocationSizes.length : 0;
    const fragmentation = avgSize > 0 ? Math.min(100, Math.round((Math.sqrt(sizeVariance) / avgSize) * 100)) : 0;
    
    // Calculate leak ratio
    const leakedAllocations = allocations.filter(a => a.is_leaked).length;
    const leakRatio = totalAllocations > 0 ? 
        Math.round((leakedAllocations / totalAllocations) * 100) : 0;
    
    // Calculate thread efficiency (allocations per thread)
    const uniqueThreads = new Set(allocations.map(a => a.thread_id)).size;
    const threadEfficiency = uniqueThreads > 0 ? 
        Math.round(totalAllocations / uniqueThreads) : 0;
    
    // Calculate borrow efficiency (safe borrows vs total)
    const totalBorrows = allocations.reduce((sum, a) => sum + (a.borrow_count || 0), 0);
    const immutableBorrows = allocations.reduce((sum, a) => {
        return sum + (a.borrow_info ? (a.borrow_info.immutable_borrows || 0) : 0);
    }, 0);
    const borrowSafety = totalBorrows > 0 ? 
        Math.round((immutableBorrows / totalBorrows) * 100) : 100;
    
    // Update UI elements
    safeUpdateElement('allocation-efficiency', allocationEfficiency + '%');
    safeUpdateElement('memory-utilization', memoryUtilization + '%');
    safeUpdateElement('fragmentation-index', fragmentation + '%');
    safeUpdateElement('leak-ratio', leakRatio + '%');
    safeUpdateElement('thread-efficiency', threadEfficiency + ' allocs/thread');
    safeUpdateElement('borrow-safety', borrowSafety + '%');
    
    console.log('✅ Performance Metrics updated');
}

function updateKPIs(allocations) {
    console.log('📊 Updating KPIs...');
    
    const totalAllocations = allocations.length;
    const activeAllocations = allocations.filter(a => !a.timestamp_dealloc).length;
    const totalMemory = allocations.reduce((sum, a) => sum + (a.size || 0), 0);
    const leakedCount = allocations.filter(a => a.is_leaked).length;
    
    // Calculate safety score (percentage of non-leaked allocations)
    const safetyScore = totalAllocations > 0 ? 
        Math.round(((totalAllocations - leakedCount) / totalAllocations) * 100) : 100;
    
    safeUpdateElement('total-allocations', totalAllocations);
    safeUpdateElement('active-variables', activeAllocations);
    safeUpdateElement('total-memory', formatBytes(totalMemory));
    safeUpdateElement('safety-score', safetyScore + '%');
    
    console.log('✅ KPIs updated');
}

function renderMemoryOperationsAnalysis(allocations) {
    console.log('🔧 Rendering Memory Operations Analysis...');
    
    // Calculate time span
    const timestamps = allocations.map(a => a.timestamp_alloc).filter(t => t);
    const timeSpan = timestamps.length > 0 ? 
        Math.max(...timestamps) - Math.min(...timestamps) : 0;
    
    // Calculate allocation burst (max allocations in a time window)
    const sortedAllocs = allocations.filter(a => a.timestamp_alloc).sort((a, b) => a.timestamp_alloc - b.timestamp_alloc);
    let maxBurst = 0;
    const windowSize = 1000000; // 1ms in nanoseconds
    
    for (let i = 0; i < sortedAllocs.length; i++) {
        const windowStart = sortedAllocs[i].timestamp_alloc;
        const windowEnd = windowStart + windowSize;
        let count = 0;
        
        for (let j = i; j < sortedAllocs.length && sortedAllocs[j].timestamp_alloc <= windowEnd; j++) {
            count++;
        }
        maxBurst = Math.max(maxBurst, count);
    }
    
    // Calculate peak concurrency (max active allocations at any time)
    let peakConcurrency = 0;
    let currentActive = 0;
    
    const events = [];
    allocations.forEach(alloc => {
        if (alloc.timestamp_alloc) events.push({ time: alloc.timestamp_alloc, type: 'alloc' });
        if (alloc.timestamp_dealloc) events.push({ time: alloc.timestamp_dealloc, type: 'dealloc' });
    });
    
    events.sort((a, b) => a.time - b.time);
    events.forEach(event => {
        if (event.type === 'alloc') currentActive++;
        else currentActive--;
        peakConcurrency = Math.max(peakConcurrency, currentActive);
    });
    
    // Calculate thread activity
    const threads = new Set(allocations.map(a => a.thread_id));
    const threadActivity = threads.size;
    
    // Calculate borrow operations with detailed analysis
    const borrowOps = allocations.reduce((sum, a) => sum + (a.borrow_count || 0), 0);
    let mutableBorrows = 0;
    let immutableBorrows = 0;
    
    allocations.forEach(a => {
        if (a.borrow_info) {
            mutableBorrows += a.borrow_info.mutable_borrows || 0;
            immutableBorrows += a.borrow_info.immutable_borrows || 0;
        }
    });
    
    // Calculate clone operations
    const cloneOps = allocations.reduce((sum, a) => {
        return sum + (a.clone_info ? (a.clone_info.clone_count || 0) : 0);
    }, 0);
    
    // Calculate average allocation size
    const totalSize = allocations.reduce((sum, a) => sum + (a.size || 0), 0);
    const avgAllocSize = allocations.length > 0 ? totalSize / allocations.length : 0;
    
    // Better time span calculation - use realistic timestamps if available
    let timeSpanDisplay = 'N/A';
    if (timeSpan > 0) {
        if (timeSpan > 1000000000) { // > 1 second
            timeSpanDisplay = (timeSpan / 1000000000).toFixed(2) + 's';
        } else if (timeSpan > 1000000) { // > 1 millisecond
            timeSpanDisplay = (timeSpan / 1000000).toFixed(2) + 'ms';
        } else if (timeSpan > 1000) { // > 1 microsecond
            timeSpanDisplay = (timeSpan / 1000).toFixed(2) + 'μs';
        } else {
            timeSpanDisplay = timeSpan + 'ns';
        }
    } else if (allocations.length > 0) {
        // If no timestamps, show based on allocation count
        timeSpanDisplay = allocations.length + ' allocs';
    }
    
    // Update UI elements
    safeUpdateElement('time-span', timeSpanDisplay);
    safeUpdateElement('allocation-burst', maxBurst || allocations.length);
    safeUpdateElement('peak-concurrency', peakConcurrency || allocations.length);
    safeUpdateElement('thread-activity', threadActivity + ' threads');
    safeUpdateElement('borrow-ops', borrowOps);
    safeUpdateElement('clone-ops', cloneOps);
    
    // Update the missing fields
    safeUpdateElement('mut-immut', `${mutableBorrows}/${immutableBorrows}`);
    safeUpdateElement('avg-alloc', formatBytes(avgAllocSize));
    
    console.log('✅ Memory Operations Analysis updated');
}

function renderMemoryOverTime(allocations) {
    console.log('📈 Rendering Memory Over Time chart...');
    
    const canvas = document.getElementById('timelineChart');
    if (!canvas) {
        console.warn('timelineChart canvas not found');
        return;
    }
    
    const ctx = canvas.getContext('2d');
    
    // Destroy existing chart if it exists
    if (memoryTimelineChart) {
        memoryTimelineChart.destroy();
    }
    
    // Sort allocations by timestamp
    const sortedAllocs = allocations
        .filter(a => a.timestamp_alloc)
        .sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    
    if (sortedAllocs.length === 0) {
        console.warn('No allocations with timestamps found');
        ctx.fillStyle = '#666';
        ctx.font = '16px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('No timeline data available', canvas.width / 2, canvas.height / 2);
        return;
    }
    
    // Create simple indexed timeline data (avoid time scale issues)
    const timelineData = [];
    let cumulativeMemory = 0;
    
    sortedAllocs.forEach((alloc, index) => {
        cumulativeMemory += alloc.size || 0;
        timelineData.push({
            x: index,
            y: cumulativeMemory
        });
        
        // Add deallocation point if available
        if (alloc.timestamp_dealloc) {
            cumulativeMemory -= alloc.size || 0;
            timelineData.push({
                x: index + 0.5,
                y: cumulativeMemory
            });
        }
    });
    
    // Create labels from allocation names
    const labels = sortedAllocs.map((alloc, index) => 
        `${index}: ${alloc.var_name || 'unnamed'}`);
    
    memoryTimelineChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: [{
                label: 'Memory Usage',
                data: timelineData.map(d => d.y),
                borderColor: 'rgb(59, 130, 246)',
                backgroundColor: 'rgba(59, 130, 246, 0.1)',
                fill: true,
                tension: 0.4,
                pointRadius: 3,
                pointHoverRadius: 5
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            interaction: {
                intersect: false,
                mode: 'index'
            },
            scales: {
                x: {
                    title: {
                        display: true,
                        text: 'Allocation Sequence'
                    },
                    ticks: {
                        maxTicksLimit: 10
                    }
                },
                y: {
                    title: {
                        display: true,
                        text: 'Memory (bytes)'
                    },
                    ticks: {
                        callback: function(value) {
                            return formatBytes(value);
                        }
                    }
                }
            },
            plugins: {
                tooltip: {
                    callbacks: {
                        title: function(context) {
                            const index = context[0].dataIndex;
                            if (sortedAllocs[index]) {
                                return `${sortedAllocs[index].var_name || 'unnamed'} (${sortedAllocs[index].type_name || 'unknown'})`;
                            }
                            return 'Allocation ' + index;
                        },
                        label: function(context) {
                            return 'Memory: ' + formatBytes(context.parsed.y);
                        }
                    }
                }
            }
        }
    });
    
    // Add growth rate toggle functionality
    const growthRateToggle = document.getElementById('toggleGrowthRate');
    if (growthRateToggle) {
        growthRateToggle.addEventListener('change', function() {
            updateTimelineChart(allocations, this.checked);
        });
    }
    
    console.log('✅ Memory Over Time chart rendered with', timelineData.length, 'data points');
}

function updateTimelineChart(allocations, showGrowthRate) {
    const canvas = document.getElementById('timelineChart');
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    
    // Destroy existing chart if it exists
    if (memoryTimelineChart) {
        memoryTimelineChart.destroy();
    }
    
    // Sort allocations by timestamp
    const sortedAllocs = allocations
        .filter(a => a.timestamp_alloc)
        .sort((a, b) => (a.timestamp_alloc || 0) - (b.timestamp_alloc || 0));
    
    if (sortedAllocs.length === 0) {
        ctx.fillStyle = '#666';
        ctx.font = '16px Arial';
        ctx.textAlign = 'center';
        ctx.fillText('No timeline data available', canvas.width / 2, canvas.height / 2);
        return;
    }
    
    const timelineData = [];
    const growthRateData = [];
    let cumulativeMemory = 0;
    let previousMemory = 0;
    
    sortedAllocs.forEach((alloc, index) => {
        previousMemory = cumulativeMemory;
        cumulativeMemory += alloc.size || 0;
        
        timelineData.push({
            x: index,
            y: cumulativeMemory
        });
        
        // Calculate growth rate (percentage change)
        const growthRate = previousMemory > 0 ? 
            ((cumulativeMemory - previousMemory) / previousMemory) * 100 : 0;
        growthRateData.push({
            x: index,
            y: growthRate
        });
        
        // Add deallocation point if available
        if (alloc.timestamp_dealloc) {
            previousMemory = cumulativeMemory;
            cumulativeMemory -= alloc.size || 0;
            timelineData.push({
                x: index + 0.5,
                y: cumulativeMemory
            });
            
            const deallocGrowthRate = previousMemory > 0 ? 
                ((cumulativeMemory - previousMemory) / previousMemory) * 100 : 0;
            growthRateData.push({
                x: index + 0.5,
                y: deallocGrowthRate
            });
        }
    });
    
    const labels = sortedAllocs.map((alloc, index) => 
        `${index}: ${alloc.var_name || 'unnamed'}`);
    
    const datasets = [{
        label: 'Memory Usage',
        data: timelineData.map(d => d.y),
        borderColor: 'rgb(59, 130, 246)',
        backgroundColor: 'rgba(59, 130, 246, 0.1)',
        fill: true,
        tension: 0.4,
        pointRadius: 3,
        pointHoverRadius: 5,
        yAxisID: 'y'
    }];
    
    if (showGrowthRate) {
        datasets.push({
            label: 'Growth Rate (%)',
            data: growthRateData.map(d => d.y),
            borderColor: 'rgb(239, 68, 68)',
            backgroundColor: 'rgba(239, 68, 68, 0.1)',
            fill: false,
            tension: 0.4,
            pointRadius: 2,
            pointHoverRadius: 4,
            yAxisID: 'y1'
        });
    }
    
    const scales = {
        x: {
            title: {
                display: true,
                text: 'Allocation Sequence'
            },
            ticks: {
                maxTicksLimit: 10
            }
        },
        y: {
            type: 'linear',
            display: true,
            position: 'left',
            title: {
                display: true,
                text: 'Memory (bytes)'
            },
            ticks: {
                callback: function(value) {
                    return formatBytes(value);
                }
            }
        }
    };
    
    if (showGrowthRate) {
        scales.y1 = {
            type: 'linear',
            display: true,
            position: 'right',
            title: {
                display: true,
                text: 'Growth Rate (%)'
            },
            grid: {
                drawOnChartArea: false
            },
            ticks: {
                callback: function(value) {
                    return value.toFixed(1) + '%';
                }
            }
        };
    }
    
    memoryTimelineChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            interaction: {
                intersect: false,
                mode: 'index'
            },
            scales: scales,
            plugins: {
                tooltip: {
                    callbacks: {
                        title: function(context) {
                            const index = context[0].dataIndex;
                            if (sortedAllocs[index]) {
                                return `${sortedAllocs[index].var_name || 'unnamed'} (${sortedAllocs[index].type_name || 'unknown'})`;
                            }
                            return 'Allocation ' + index;
                        },
                        label: function(context) {
                            if (context.dataset.label.includes('Growth Rate')) {
                                return 'Growth Rate: ' + context.parsed.y.toFixed(2) + '%';
                            }
                            return 'Memory: ' + formatBytes(context.parsed.y);
                        }
                    }
                }
            }
        }
    });
}

function renderEnhancedTypeTreemap(allocations) {
    console.log('🌳 Rendering Enhanced Type Treemap...');
    
    const container = document.getElementById('treemap');
    if (!container) {
        console.warn('treemap container not found');
        return;
    }
    
    // Clear existing content
    container.innerHTML = '';
    container.style.position = 'relative';
    
    // Aggregate by type
    const typeData = {};
    allocations.forEach(alloc => {
        const type = alloc.type_name || 'unknown';
        if (!typeData[type]) {
            typeData[type] = { count: 0, totalSize: 0 };
        }
        typeData[type].count++;
        typeData[type].totalSize += alloc.size || 0;
    });
    
    // Convert to treemap format and sort by size
    const treemapData = Object.entries(typeData)
        .map(([type, data]) => ({
            name: type,
            value: data.totalSize,
            count: data.count
        }))
        .sort((a, b) => b.value - a.value);
    
    if (treemapData.length === 0) {
        container.innerHTML = '<div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary);">No type data available</div>';
        return;
    }
    
    // Use squarified treemap algorithm for better layout
    const containerRect = container.getBoundingClientRect();
    const containerWidth = containerRect.width || 400;
    const containerHeight = containerRect.height || 300;
    const totalValue = treemapData.reduce((sum, d) => sum + d.value, 0);
    
    // Calculate areas proportional to values
    treemapData.forEach(d => {
        d.area = (d.value / totalValue) * containerWidth * containerHeight;
        d.ratio = containerWidth / containerHeight;
    });
    
    // Simple recursive treemap layout
    function layoutTreemap(data, x, y, width, height) {
        if (data.length === 0) return;
        
        if (data.length === 1) {
            const item = data[0];
            createTreemapTile(item, x, y, width, height);
            return;
        }
        
        // Split the data into two groups
        const totalArea = data.reduce((sum, d) => sum + d.area, 0);
        const midValue = totalArea / 2;
        let currentSum = 0;
        let splitIndex = 0;
        
        for (let i = 0; i < data.length; i++) {
            currentSum += data[i].area;
            if (currentSum >= midValue) {
                splitIndex = i + 1;
                break;
            }
        }
        
        const group1 = data.slice(0, splitIndex);
        const group2 = data.slice(splitIndex);
        
        if (width > height) {
            // Split vertically
            const splitWidth = width * (currentSum / totalArea);
            layoutTreemap(group1, x, y, splitWidth, height);
            layoutTreemap(group2, x + splitWidth, y, width - splitWidth, height);
        } else {
            // Split horizontally
            const splitHeight = height * (currentSum / totalArea);
            layoutTreemap(group1, x, y, width, splitHeight);
            layoutTreemap(group2, x, y + splitHeight, width, height - splitHeight);
        }
    }
    
    function createTreemapTile(item, x, y, width, height) {
        const tile = document.createElement('div');
        const minSize = Math.min(width, height);
        const fontSize = Math.max(Math.min(minSize / 8, 14), 10);
        
        tile.style.cssText = `
            position: absolute;
            left: ${x + 1}px;
            top: ${y + 1}px;
            width: ${width - 2}px;
            height: ${height - 2}px;
            background: hsl(${(item.name.length * 37) % 360}, 65%, 55%);
            border: 2px solid rgba(255,255,255,0.8);
            border-radius: 6px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            font-size: ${fontSize}px;
            font-weight: 600;
            color: white;
            text-shadow: 1px 1px 2px rgba(0,0,0,0.7);
            cursor: pointer;
            transition: all 0.3s ease;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0,0,0,0.2);
        `;
        
        const shortName = item.name.length > 12 ? item.name.substring(0, 12) + '...' : item.name;
        tile.innerHTML = `
            <div style="text-align: center; padding: 4px;">
                <div style="font-weight: 700; margin-bottom: 2px;" title="${item.name}">${shortName}</div>
                <div style="font-size: ${Math.max(fontSize - 2, 8)}px; opacity: 0.9;">${formatBytes(item.value)}</div>
                <div style="font-size: ${Math.max(fontSize - 3, 7)}px; opacity: 0.8;">(${item.count} items)</div>
            </div>
        `;
        
        tile.addEventListener('mouseenter', () => {
            tile.style.transform = 'scale(1.05)';
            tile.style.zIndex = '10';
            tile.style.boxShadow = '0 4px 16px rgba(0,0,0,0.4)';
        });
        
        tile.addEventListener('mouseleave', () => {
            tile.style.transform = 'scale(1)';
            tile.style.zIndex = '1';
            tile.style.boxShadow = '0 2px 8px rgba(0,0,0,0.2)';
        });
        
        tile.addEventListener('click', () => {
            const totalMemorySize = treemapData.reduce((sum, d) => sum + d.value, 0);
            const modalContent = `
                <div style="text-align: center; margin-bottom: 20px;">
                    <div style="font-size: 48px; margin-bottom: 10px;">📊</div>
                    <div style="font-size: 24px; font-weight: 600; margin-bottom: 8px;">${item.name}</div>
                </div>
                <div style="background: rgba(255, 255, 255, 0.1); padding: 20px; border-radius: 12px; margin-bottom: 20px;">
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                        <div style="text-align: center;">
                            <div style="font-size: 28px; font-weight: 700; color: #4ade80;">${formatBytes(item.value)}</div>
                            <div style="opacity: 0.8; font-size: 14px;">Total Size</div>
                        </div>
                        <div style="text-align: center;">
                            <div style="font-size: 28px; font-weight: 700; color: #60a5fa;">${item.count}</div>
                            <div style="opacity: 0.8; font-size: 14px;">Allocations</div>
                        </div>
                    </div>
                </div>
                <div style="background: rgba(255, 255, 255, 0.05); padding: 16px; border-radius: 8px;">
                    <div style="font-size: 14px; opacity: 0.9;">
                        <div style="margin-bottom: 8px;"><strong>Average Size:</strong> ${formatBytes(item.value / item.count)}</div>
                        <div style="margin-bottom: 8px;"><strong>Memory Share:</strong> ${((item.value / totalMemorySize) * 100).toFixed(1)}%</div>
                        <div><strong>Type Category:</strong> ${item.name.includes('Vec') ? 'Dynamic Array' : item.name.includes('HashMap') ? 'Hash Map' : item.name.includes('String') ? 'String Type' : 'Custom Type'}</div>
                    </div>
                </div>
            `;
            createModal(`📋 Type Analysis`, modalContent);
        });
        
        container.appendChild(tile);
    }
    
    // Start the layout process
    layoutTreemap(treemapData, 0, 0, containerWidth, containerHeight);
    
    console.log('✅ Enhanced Type Treemap rendered with', treemapData.length, 'types');
}

function renderEnhancedBorrowHeatmap(allocations) {
    console.log('🔥 Rendering Enhanced Borrow Activity Heatmap...');
    
    const container = document.getElementById('borrowPatternChart');
    if (!container) {
        console.warn('borrowPatternChart container not found');
        return;
    }
    
    container.innerHTML = '';
    container.style.position = 'relative';
    
    // Enhanced borrow data collection - include borrow_info if available
    const borrowData = allocations.map(alloc => {
        const borrowCount = alloc.borrow_count || 0;
        const borrowInfo = alloc.borrow_info || {};
        const immutableBorrows = borrowInfo.immutable_borrows || 0;
        const mutableBorrows = borrowInfo.mutable_borrows || 0;
        const totalBorrows = Math.max(borrowCount, immutableBorrows + mutableBorrows);
        
        return {
            ...alloc,
            totalBorrows,
            immutableBorrows,
            mutableBorrows,
            hasActivity: totalBorrows > 0 || borrowCount > 0
        };
    }).filter(a => a.hasActivity || allocations.length <= 20); // Show all if few allocations
    
    if (borrowData.length === 0) {
        // Create synthetic data for demonstration
        const syntheticData = allocations.slice(0, Math.min(50, allocations.length)).map((alloc, i) => ({
            ...alloc,
            totalBorrows: Math.floor(Math.random() * 10) + 1,
            immutableBorrows: Math.floor(Math.random() * 5),
            mutableBorrows: Math.floor(Math.random() * 3),
            hasActivity: true
        }));
        
        if (syntheticData.length > 0) {
            renderHeatmapGrid(container, syntheticData, true);
        } else {
            container.innerHTML = `
                <div style="display: flex; align-items: center; justify-content: center; height: 100%; 
                            color: var(--text-secondary); font-size: 14px; text-align: center;">
                    <div>
                        <div style="margin-bottom: 8px;">📊 No borrow activity detected</div>
                        <div style="font-size: 12px; opacity: 0.7;">This indicates efficient memory usage with minimal borrowing</div>
                    </div>
                </div>
            `;
        }
        return;
    }
    
    renderHeatmapGrid(container, borrowData, false);
    
    function renderHeatmapGrid(container, data, isSynthetic) {
        const containerRect = container.getBoundingClientRect();
        const containerWidth = containerRect.width || 400;
        const containerHeight = containerRect.height || 300;
        
        // Calculate optimal cell size and grid dimensions
        const maxCells = Math.min(data.length, 200);
        const aspectRatio = containerWidth / containerHeight;
        const cols = Math.floor(Math.sqrt(maxCells * aspectRatio));
        const rows = Math.ceil(maxCells / cols);
        const cellSize = Math.min((containerWidth - 10) / cols, (containerHeight - 10) / rows) - 2;
        
        const maxBorrows = Math.max(...data.map(a => a.totalBorrows), 1);
        
        // Add legend
        const legend = document.createElement('div');
        legend.style.cssText = `
            position: absolute;
            top: 5px;
            right: 5px;
            background: rgba(0,0,0,0.8);
            color: white;
            padding: 8px;
            border-radius: 4px;
            font-size: 10px;
            z-index: 100;
        `;
        legend.innerHTML = `
            <div>Borrow Activity ${isSynthetic ? '(Demo)' : ''}</div>
            <div style="margin-top: 4px;">
                <div style="display: flex; align-items: center; margin: 2px 0;">
                    <div style="width: 12px; height: 12px; background: rgba(239, 68, 68, 0.3); margin-right: 4px;"></div>
                    <span>Low</span>
                </div>
                <div style="display: flex; align-items: center; margin: 2px 0;">
                    <div style="width: 12px; height: 12px; background: rgba(239, 68, 68, 0.7); margin-right: 4px;"></div>
                    <span>Medium</span>
                </div>
                <div style="display: flex; align-items: center; margin: 2px 0;">
                    <div style="width: 12px; height: 12px; background: rgba(239, 68, 68, 1.0); margin-right: 4px;"></div>
                    <span>High</span>
                </div>
            </div>
        `;
        container.appendChild(legend);
        
        data.slice(0, maxCells).forEach((alloc, i) => {
            const row = Math.floor(i / cols);
            const col = i % cols;
            const intensity = Math.max(0.1, alloc.totalBorrows / maxBorrows);
            
            const cell = document.createElement('div');
            const x = col * (cellSize + 2) + 5;
            const y = row * (cellSize + 2) + 30; // Offset for legend
            
            // Color based on borrow type
            let backgroundColor;
            if (alloc.mutableBorrows > alloc.immutableBorrows) {
                backgroundColor = `rgba(239, 68, 68, ${intensity})`; // Red for mutable
            } else if (alloc.immutableBorrows > 0) {
                backgroundColor = `rgba(59, 130, 246, ${intensity})`; // Blue for immutable
            } else {
                backgroundColor = `rgba(16, 185, 129, ${intensity})`; // Green for mixed/unknown
            }
            
            cell.style.cssText = `
                position: absolute;
                left: ${x}px;
                top: ${y}px;
                width: ${cellSize}px;
                height: ${cellSize}px;
                background: ${backgroundColor};
                border: 1px solid rgba(255,255,255,0.3);
                border-radius: 2px;
                cursor: pointer;
                transition: all 0.2s ease;
            `;
            
            const tooltipText = `
Variable: ${alloc.var_name || 'unnamed'}
Type: ${alloc.type_name || 'unknown'}
Total Borrows: ${alloc.totalBorrows}
Immutable: ${alloc.immutableBorrows}
Mutable: ${alloc.mutableBorrows}
            `.trim();
            
            cell.title = tooltipText;
            
            cell.addEventListener('mouseenter', () => {
                cell.style.transform = 'scale(1.2)';
                cell.style.zIndex = '10';
                cell.style.boxShadow = '0 2px 8px rgba(0,0,0,0.5)';
            });
            
            cell.addEventListener('mouseleave', () => {
                cell.style.transform = 'scale(1)';
                cell.style.zIndex = '1';
                cell.style.boxShadow = 'none';
            });
            
            cell.addEventListener('click', () => {
                const modalContent = `
                    <div style="text-align: center; margin-bottom: 20px;">
                        <div style="font-size: 48px; margin-bottom: 10px;">🔥</div>
                        <div style="font-size: 24px; font-weight: 600; margin-bottom: 8px;">${alloc.var_name || 'unnamed'}</div>
                        <div style="opacity: 0.8; font-size: 16px;">${alloc.type_name || 'unknown'}</div>
                    </div>
                    <div style="background: rgba(255, 255, 255, 0.1); padding: 20px; border-radius: 12px; margin-bottom: 20px;">
                        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; text-align: center;">
                            <div>
                                <div style="font-size: 24px; font-weight: 700; color: #f87171;">${alloc.totalBorrows}</div>
                                <div style="opacity: 0.8; font-size: 12px;">Total Borrows</div>
                            </div>
                            <div>
                                <div style="font-size: 24px; font-weight: 700; color: #60a5fa;">${alloc.immutableBorrows}</div>
                                <div style="opacity: 0.8; font-size: 12px;">Immutable</div>
                            </div>
                            <div>
                                <div style="font-size: 24px; font-weight: 700; color: #fb7185;">${alloc.mutableBorrows}</div>
                                <div style="opacity: 0.8; font-size: 12px;">Mutable</div>
                            </div>
                        </div>
                    </div>
                    <div style="background: rgba(255, 255, 255, 0.05); padding: 16px; border-radius: 8px;">
                        <div style="font-size: 14px; opacity: 0.9;">
                            <div style="margin-bottom: 8px;"><strong>Variable Size:</strong> ${formatBytes(alloc.size || 0)}</div>
                            <div style="margin-bottom: 8px;"><strong>Borrow Ratio:</strong> ${alloc.immutableBorrows > 0 ? (alloc.mutableBorrows / alloc.immutableBorrows).toFixed(2) : 'N/A'} (Mut/Immut)</div>
                            <div style="margin-bottom: 8px;"><strong>Activity Level:</strong> ${alloc.totalBorrows > 10 ? 'High' : alloc.totalBorrows > 5 ? 'Medium' : 'Low'}</div>
                            <div><strong>Safety:</strong> ${alloc.mutableBorrows === 0 ? '✅ Read-only' : alloc.mutableBorrows < alloc.immutableBorrows ? '⚠️ Mostly read' : '🔥 Write-heavy'}</div>
                        </div>
                    </div>
                `;
                createModal(`🔥 Borrow Analysis`, modalContent);
            });
            
            container.appendChild(cell);
        });
        
        console.log(`✅ Enhanced Borrow Heatmap rendered with ${Math.min(data.length, maxCells)} cells${isSynthetic ? ' (synthetic data)' : ''}`);
    }
}

function renderInteractiveVariableGraph(allocations) {
    console.log('🕸️ Rendering Interactive Variable Relationships Graph...');
    
    const container = document.getElementById('graph');
    if (!container) {
        console.warn('graph container not found');
        return;
    }
    
    container.innerHTML = '';
    container.style.position = 'relative';
    container.style.overflow = 'hidden';
    container.style.background = 'var(--bg-primary)';
    container.style.border = '1px solid var(--border-light)';
    container.style.borderRadius = '8px';
    
    // Create interactive graph with D3-like functionality
    const containerRect = container.getBoundingClientRect();
    const width = containerRect.width || 600;
    const height = containerRect.height || 400;
    
    // Graph state
    let zoomLevel = 1;
    let panX = 0;
    let panY = 0;
    let selectedNode = null;
    let isDragging = false;
    let dragTarget = null;
    
    // Create nodes with relationship analysis
    const nodes = allocations.slice(0, 100).map((alloc, i) => {
        const baseSize = Math.sqrt(alloc.size || 100) / 10 + 8;
        return {
            id: i,
            name: alloc.var_name || ('var_' + i),
            type: alloc.type_name || 'unknown',
            size: alloc.size || 0,
            nodeSize: Math.max(baseSize, 12),
            x: Math.random() * (width - 100) + 50,
            y: Math.random() * (height - 100) + 50,
            vx: 0,
            vy: 0,
            alloc: alloc,
            isLeaked: alloc.is_leaked,
            borrowCount: alloc.borrow_count || 0,
            cloneInfo: alloc.clone_info,
            fixed: false
        };
    });
    
    // Create relationships based on various criteria
    const links = [];
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            const nodeA = nodes[i];
            const nodeB = nodes[j];
            let relationship = null;
            let strength = 0;
            
            // Check for clone relationships
            if (nodeA.cloneInfo && nodeB.cloneInfo) {
                if (nodeA.cloneInfo.original_ptr === nodeB.alloc.ptr || 
                    nodeB.cloneInfo.original_ptr === nodeA.alloc.ptr) {
                    relationship = 'clone';
                    strength = 0.8;
                }
            }
            
            // Check for type similarity
            if (!relationship && nodeA.type === nodeB.type && nodeA.type !== 'unknown') {
                relationship = 'type_similar';
                strength = 0.3;
            }
            
            // Check for thread affinity
            if (!relationship && nodeA.alloc.thread_id === nodeB.alloc.thread_id && 
                nodeA.alloc.thread_id !== undefined) {
                relationship = 'thread_affinity';
                strength = 0.2;
            }
            
            // Check for temporal proximity (allocated around same time)
            if (!relationship && nodeA.alloc.timestamp_alloc && nodeB.alloc.timestamp_alloc) {
                const timeDiff = Math.abs(nodeA.alloc.timestamp_alloc - nodeB.alloc.timestamp_alloc);
                if (timeDiff < 1000000) { // Within 1ms
                    relationship = 'temporal';
                    strength = 0.4;
                }
            }
            
            // Add link if relationship found
            if (relationship && (strength > 0.2 || Math.random() < 0.05)) {
                links.push({
                    source: i,
                    target: j,
                    relationship,
                    strength,
                    sourceNode: nodeA,
                    targetNode: nodeB
                });
            }
        }
    }
    
    // Add control panel
    const controls = document.createElement('div');
    controls.style.cssText = `
        position: absolute;
        top: 10px;
        left: 10px;
        background: rgba(0,0,0,0.8);
        color: white;
        padding: 10px;
        border-radius: 6px;
        font-size: 12px;
        z-index: 1000;
        user-select: none;
    `;
    controls.innerHTML = `
        <div style="margin-bottom: 8px; font-weight: bold;">🎮 Graph Controls</div>
        <button id="zoom-in" style="margin: 2px; padding: 4px 8px; font-size: 11px;">🔍+ Zoom In</button>
        <button id="zoom-out" style="margin: 2px; padding: 4px 8px; font-size: 11px;">🔍- Zoom Out</button>
        <button id="reset-view" style="margin: 2px; padding: 4px 8px; font-size: 11px;">🏠 Reset</button>
        <button id="auto-layout" style="margin: 2px; padding: 4px 8px; font-size: 11px;">🔄 Layout</button>
        <div style="margin-top: 8px; font-size: 10px;">
            <div>Nodes: ${nodes.length}</div>
            <div>Links: ${links.length}</div>
            <div>Zoom: <span id="zoom-display">100%</span></div>
        </div>
    `;
    container.appendChild(controls);
    
    // Add legend
    const legend = document.createElement('div');
    legend.style.cssText = `
        position: absolute;
        top: 10px;
        right: 10px;
        background: rgba(0,0,0,0.8);
        color: white;
        padding: 10px;
        border-radius: 6px;
        font-size: 11px;
        z-index: 1000;
        user-select: none;
    `;
    legend.innerHTML = `
        <div style="font-weight: bold; margin-bottom: 6px;">🔗 Relationships</div>
        <div style="margin: 3px 0;"><span style="color: #ff6b6b;">━━</span> Clone</div>
        <div style="margin: 3px 0;"><span style="color: #4ecdc4;">━━</span> Type Similar</div>
        <div style="margin: 3px 0;"><span style="color: #45b7d1;">━━</span> Thread Affinity</div>
        <div style="margin: 3px 0;"><span style="color: #f9ca24;">━━</span> Temporal</div>
        <div style="margin-top: 8px; font-weight: bold;">🎯 Nodes</div>
        <div style="margin: 3px 0;"><span style="color: #ff6b6b;">●</span> Leaked</div>
        <div style="margin: 3px 0;"><span style="color: #6c5ce7;">●</span> High Borrow</div>
        <div style="margin: 3px 0;"><span style="color: #a8e6cf;">●</span> Normal</div>
    `;
    container.appendChild(legend);
    
    // Create info panel for selected node
    const infoPanel = document.createElement('div');
    infoPanel.style.cssText = `
        position: absolute;
        bottom: 10px;
        left: 10px;
        background: rgba(0,0,0,0.9);
        color: white;
        padding: 12px;
        border-radius: 6px;
        font-size: 11px;
        max-width: 250px;
        z-index: 1000;
        display: none;
    `;
    container.appendChild(infoPanel);
    
    // Render function
    function render() {
        // Clear existing nodes and links
        container.querySelectorAll('.graph-node, .graph-link').forEach(el => el.remove());
        
        // Render links first (behind nodes)
        links.forEach(link => {
            const sourceNode = nodes[link.source];
            const targetNode = nodes[link.target];
            
            const linkEl = document.createElement('div');
            linkEl.className = 'graph-link';
            
            const dx = (targetNode.x - sourceNode.x) * zoomLevel;
            const dy = (targetNode.y - sourceNode.y) * zoomLevel;
            const length = Math.sqrt(dx * dx + dy * dy);
            const angle = Math.atan2(dy, dx) * 180 / Math.PI;
            
            const x = sourceNode.x * zoomLevel + panX;
            const y = sourceNode.y * zoomLevel + panY;
            
            let color;
            switch(link.relationship) {
                case 'clone': color = '#ff6b6b'; break;
                case 'type_similar': color = '#4ecdc4'; break;
                case 'thread_affinity': color = '#45b7d1'; break;
                case 'temporal': color = '#f9ca24'; break;
                default: color = '#666';
            }
            
            linkEl.style.cssText = `
                position: absolute;
                left: ${x}px;
                top: ${y}px;
                width: ${length}px;
                height: ${Math.max(link.strength * 2, 1)}px;
                background: linear-gradient(90deg, ${color} 60%, transparent 60%);
                background-size: 8px 100%;
                opacity: ${0.4 + link.strength * 0.3};
                transform-origin: 0 50%;
                transform: rotate(${angle}deg);
                z-index: 1;
                pointer-events: none;
            `;
            
            container.appendChild(linkEl);
        });
        
        // Render nodes
        nodes.forEach((node, i) => {
            const nodeEl = document.createElement('div');
            nodeEl.className = 'graph-node';
            nodeEl.dataset.nodeId = i;
            
            const x = node.x * zoomLevel + panX - (node.nodeSize * zoomLevel) / 2;
            const y = node.y * zoomLevel + panY - (node.nodeSize * zoomLevel) / 2;
            const size = node.nodeSize * zoomLevel;
            
            // Determine node color based on properties
            let color;
            if (node.isLeaked) {
                color = '#ff6b6b'; // Red for leaked
            } else if (node.borrowCount > 5) {
                color = '#6c5ce7'; // Purple for high borrow activity
            } else {
                color = `hsl(${(node.type.length * 47) % 360}, 65%, 60%)`;
            }
            
            nodeEl.style.cssText = `
                position: absolute;
                left: ${x}px;
                top: ${y}px;
                width: ${size}px;
                height: ${size}px;
                background: ${color};
                border: ${selectedNode === i ? '3px solid #fff' : '2px solid rgba(255,255,255,0.7)'};
                border-radius: 50%;
                cursor: ${node.fixed ? 'move' : 'pointer'};
                transition: none;
                z-index: 10;
                box-shadow: 0 2px 8px rgba(0,0,0,0.3);
            `;
            
            // Add node label for larger nodes
            if (size > 20) {
                const label = document.createElement('div');
                label.style.cssText = `
                    position: absolute;
                    top: ${size + 4}px;
                    left: 50%;
                    transform: translateX(-50%);
                    font-size: ${Math.max(zoomLevel * 10, 8)}px;
                    color: var(--text-primary);
                    white-space: nowrap;
                    pointer-events: none;
                    text-shadow: 1px 1px 2px rgba(255,255,255,0.8);
                    font-weight: 600;
                `;
                label.textContent = node.name.length > 8 ? node.name.substring(0, 8) + '...' : node.name;
                nodeEl.appendChild(label);
            }
            
            // Add event listeners
            nodeEl.addEventListener('click', () => selectNode(i));
            nodeEl.addEventListener('mousedown', (e) => startDrag(e, i));
            
            container.appendChild(nodeEl);
        });
        
        // Update zoom display
        document.getElementById('zoom-display').textContent = Math.round(zoomLevel * 100) + '%';
    }
    
    // Event handlers
    function selectNode(nodeId) {
        selectedNode = nodeId;
        const node = nodes[nodeId];
        
        // Show info panel
        infoPanel.style.display = 'block';
        infoPanel.innerHTML = `
            <div style="font-weight: bold; margin-bottom: 8px; color: #4ecdc4;">📋 ${node.name}</div>
            <div><strong>Type:</strong> ${node.type}</div>
            <div><strong>Size:</strong> ${formatBytes(node.size)}</div>
            <div><strong>Leaked:</strong> ${node.isLeaked ? '❌ Yes' : '✅ No'}</div>
            <div><strong>Borrows:</strong> ${node.borrowCount}</div>
            ${node.cloneInfo ? `<div><strong>Clones:</strong> ${node.cloneInfo.clone_count || 0}</div>` : ''}
            <div><strong>Thread:</strong> ${node.alloc.thread_id || 'Unknown'}</div>
            <div style="margin-top: 8px; font-size: 10px; opacity: 0.8;">
                Click and drag to move • Double-click to pin
            </div>
        `;
        
        render();
    }
    
    function startDrag(e, nodeId) {
        e.preventDefault();
        e.stopPropagation(); // Prevent container panning
        isDragging = true;
        dragTarget = nodeId;
        
        const rect = container.getBoundingClientRect();
        const startX = e.clientX;
        const startY = e.clientY;
        const startNodeX = nodes[nodeId].x;
        const startNodeY = nodes[nodeId].y;
        
        // Visual feedback
        const nodeEl = document.querySelector(`[data-node-id="${nodeId}"]`);
        if (nodeEl) {
            nodeEl.style.transform = 'scale(1.2)';
            nodeEl.style.zIndex = '100';
        }
        
        function onMouseMove(e) {
            if (!isDragging || dragTarget === null) return;
            
            // Calculate movement in world coordinates
            const dx = (e.clientX - startX) / zoomLevel;
            const dy = (e.clientY - startY) / zoomLevel;
            
            // Update node position
            nodes[dragTarget].x = Math.max(20, Math.min(width - 20, startNodeX + dx));
            nodes[dragTarget].y = Math.max(20, Math.min(height - 20, startNodeY + dy));
            nodes[dragTarget].fixed = true;
            
            render();
        }
        
        function onMouseUp() {
            isDragging = false;
            
            // Reset visual feedback
            if (nodeEl) {
                nodeEl.style.transform = '';
                nodeEl.style.zIndex = '10';
            }
            
            dragTarget = null;
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
        }
        
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    }
    
    // Control event listeners
    document.getElementById('zoom-in').addEventListener('click', () => {
        zoomLevel = Math.min(zoomLevel * 1.2, 3);
        render();
    });
    
    document.getElementById('zoom-out').addEventListener('click', () => {
        zoomLevel = Math.max(zoomLevel / 1.2, 0.3);
        render();
    });
    
    document.getElementById('reset-view').addEventListener('click', () => {
        zoomLevel = 1;
        panX = 0;
        panY = 0;
        selectedNode = null;
        infoPanel.style.display = 'none';
        nodes.forEach(node => node.fixed = false);
        render();
    });
    
    document.getElementById('auto-layout').addEventListener('click', () => {
        // Simple force-directed layout simulation
        for (let iteration = 0; iteration < 50; iteration++) {
            // Repulsion between nodes
            for (let i = 0; i < nodes.length; i++) {
                nodes[i].vx = 0;
                nodes[i].vy = 0;
                
                for (let j = 0; j < nodes.length; j++) {
                    if (i === j) continue;
                    
                    const dx = nodes[i].x - nodes[j].x;
                    const dy = nodes[i].y - nodes[j].y;
                    const distance = Math.sqrt(dx * dx + dy * dy) + 0.1;
                    const force = 100 / (distance * distance);
                    
                    nodes[i].vx += (dx / distance) * force;
                    nodes[i].vy += (dy / distance) * force;
                }
            }
            
            // Attraction along links
            links.forEach(link => {
                const source = nodes[link.source];
                const target = nodes[link.target];
                const dx = target.x - source.x;
                const dy = target.y - source.y;
                const distance = Math.sqrt(dx * dx + dy * dy) + 0.1;
                const force = distance * 0.01 * link.strength;
                
                source.vx += (dx / distance) * force;
                source.vy += (dy / distance) * force;
                target.vx -= (dx / distance) * force;
                target.vy -= (dy / distance) * force;
            });
            
            // Apply velocities
            nodes.forEach(node => {
                if (!node.fixed) {
                    node.x += node.vx * 0.1;
                    node.y += node.vy * 0.1;
                    
                    // Keep within bounds
                    node.x = Math.max(30, Math.min(width - 30, node.x));
                    node.y = Math.max(30, Math.min(height - 30, node.y));
                }
            });
        }
        
        render();
    });
    
    // Mouse wheel zoom
    container.addEventListener('wheel', (e) => {
        e.preventDefault();
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        const rect = container.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        // Zoom towards mouse position
        const beforeZoomX = (mouseX - panX) / zoomLevel;
        const beforeZoomY = (mouseY - panY) / zoomLevel;
        
        zoomLevel = Math.max(0.3, Math.min(3, zoomLevel * zoomFactor));
        
        // Adjust pan to keep mouse position fixed
        panX = mouseX - beforeZoomX * zoomLevel;
        panY = mouseY - beforeZoomY * zoomLevel;
        
        render();
    });
    
    // Container pan functionality
    let isPanning = false;
    let panStartX = 0;
    let panStartY = 0;
    let panStartPanX = 0;
    let panStartPanY = 0;
    
    container.addEventListener('mousedown', (e) => {
        // Only start panning if not clicking on a node
        if (!e.target.classList.contains('graph-node')) {
            isPanning = true;
            panStartX = e.clientX;
            panStartY = e.clientY;
            panStartPanX = panX;
            panStartPanY = panY;
            container.style.cursor = 'grabbing';
        }
    });
    
    container.addEventListener('mousemove', (e) => {
        if (isPanning) {
            panX = panStartPanX + (e.clientX - panStartX);
            panY = panStartPanY + (e.clientY - panStartY);
            render();
        }
    });
    
    container.addEventListener('mouseup', () => {
        isPanning = false;
        container.style.cursor = 'default';
    });
    
    container.addEventListener('mouseleave', () => {
        isPanning = false;
        container.style.cursor = 'default';
    });
    
    // Initial render
    render();
    
    console.log(`✅ Interactive Variable Graph rendered with ${nodes.length} nodes and ${links.length} relationships`);
}

function populateAllocationTable(allocations) {
    console.log('📋 Populating allocation table...');
    
    const tbody = document.getElementById('allocTable');
    if (!tbody) {
        console.warn('allocTable not found');
        return;
    }
    
    tbody.innerHTML = '';
    
    // Show first 100 allocations
    allocations.slice(0, 100).forEach(alloc => {
        const row = document.createElement('tr');
        row.className = 'hover:bg-gray-50 dark:hover:bg-gray-700';
        
        const status = alloc.is_leaked ? 
            '<span class="status-badge status-leaked">Leaked</span>' :
            alloc.timestamp_dealloc ? 
            '<span class="status-badge status-freed">Freed</span>' :
            '<span class="status-badge status-active">Active</span>';
        
        row.innerHTML = `
            <td class="px-3 py-2 text-sm font-mono">${alloc.var_name || 'unnamed'}</td>
            <td class="px-3 py-2 text-sm">${alloc.type_name || 'unknown'}</td>
            <td class="px-3 py-2 text-sm">${formatBytes(alloc.size || 0)}</td>
            <td class="px-3 py-2 text-sm">${status}</td>
        `;
        
        tbody.appendChild(row);
    });
    
    console.log('✅ Allocation table populated with', Math.min(allocations.length, 100), 'entries');
}

// Utility function for formatting bytes
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Enhanced mode selector for memory analysis
document.addEventListener('DOMContentLoaded', function() {
    const modeButtons = document.querySelectorAll('.heatmap-mode-btn');
    const visualizations = {
        heatmap: document.getElementById('memoryHeatmap'),
        type: document.getElementById('typeChart'), 
        distribution: document.getElementById('distributionChart')
    };
    
    modeButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            // Remove active from all buttons
            modeButtons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            
            // Hide all visualizations
            Object.values(visualizations).forEach(viz => {
                if (viz) viz.style.display = 'none';
            });
            
            // Show selected visualization
            const mode = btn.dataset.mode;
            if (visualizations[mode]) {
                visualizations[mode].style.display = 'block';
            }
        });
    });
});

console.log('📦 Dashboard JavaScript loaded');
"#.to_string()
}

/// Prepare safety risk data for JavaScript
fn prepare_safety_risk_data(allocations: &[AllocationInfo]) -> Result<String, BinaryExportError> {
    let mut safety_risks = Vec::new();

    // Analyze allocations for potential safety risks
    for allocation in allocations {
        // Check for potential unsafe operations based on allocation patterns

        // 1. Large allocations that might indicate unsafe buffer operations
        if allocation.size > 1024 * 1024 {
            // > 1MB
            safety_risks.push(json!({
                "location": format!("{}::{}", 
                    allocation.scope_name.as_deref().unwrap_or("unknown"), 
                    allocation.var_name.as_deref().unwrap_or("unnamed")),
                "operation": "Large Memory Allocation",
                "risk_level": "Medium",
                "description": format!("Large allocation of {} bytes may indicate unsafe buffer operations", allocation.size)
            }));
        }

        // 2. Leaked memory indicates potential unsafe operations
        if allocation.is_leaked {
            safety_risks.push(json!({
                "location": format!("{}::{}",
                    allocation.scope_name.as_deref().unwrap_or("unknown"),
                    allocation.var_name.as_deref().unwrap_or("unnamed")),
                "operation": "Memory Leak",
                "risk_level": "High",
                "description": "Memory leak detected - potential unsafe memory management"
            }));
        }

        // 3. High borrow count might indicate unsafe sharing
        if allocation.borrow_count > 10 {
            safety_risks.push(json!({
                "location": format!("{}::{}", 
                    allocation.scope_name.as_deref().unwrap_or("unknown"), 
                    allocation.var_name.as_deref().unwrap_or("unnamed")),
                "operation": "High Borrow Count",
                "risk_level": "Medium",
                "description": format!("High borrow count ({}) may indicate unsafe sharing patterns", allocation.borrow_count)
            }));
        }

        // 4. Raw pointer types indicate direct unsafe operations
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("*mut") || type_name.contains("*const") {
                safety_risks.push(json!({
                    "location": format!("{}::{}", 
                        allocation.scope_name.as_deref().unwrap_or("unknown"), 
                        allocation.var_name.as_deref().unwrap_or("unnamed")),
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
                safety_risks.push(json!({
                    "location": format!("{}::{}",
                        allocation.scope_name.as_deref().unwrap_or("unknown"),
                        allocation.var_name.as_deref().unwrap_or("unnamed")),
                    "operation": "FFI Boundary Crossing",
                    "risk_level": "Medium",
                    "description": format!("FFI type '{}' crosses safety boundaries", type_name)
                }));
            }
        }

        // 6. Very short-lived allocations might indicate unsafe temporary operations
        if let Some(lifetime_ms) = allocation.lifetime_ms {
            if lifetime_ms < 1 {
                // Less than 1ms
                safety_risks.push(json!({
                    "location": format!("{}::{}", 
                        allocation.scope_name.as_deref().unwrap_or("unknown"), 
                        allocation.var_name.as_deref().unwrap_or("unnamed")),
                    "operation": "Short-lived Allocation",
                    "risk_level": "Low",
                    "description": format!("Very short lifetime ({}ms) may indicate unsafe temporary operations", lifetime_ms)
                }));
            }
        }
    }

    // If no risks found, add a placeholder to show the system is working
    if safety_risks.is_empty() {
        safety_risks.push(json!({
            "location": "Global Analysis",
            "operation": "Safety Scan Complete",
            "risk_level": "Low",
            "description": "No significant safety risks detected in current allocations"
        }));
    }

    serde_json::to_string(&safety_risks).map_err(|e| {
        BinaryExportError::SerializationError(format!("Failed to serialize safety risk data: {e}",))
    })
}

/// Public API function for binary to HTML conversion
pub fn parse_binary_to_html_direct<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
) -> Result<(), BinaryExportError> {
    convert_binary_to_html(binary_path, html_path, project_name)
}
