//! Binary to HTML conversion functionality
//! Converts binary memscope files to HTML reports using clean_dashboard.html template

use crate::core::types::{AllocationInfo, MemoryStats};
use crate::export::binary::reader::BinaryReader;
use crate::export::binary::error::BinaryExportError;
use chrono;
use std::fs;
use std::path::Path;
use serde_json::json;

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
    println!("🔄 Loading binary dashboard template...");
    let template = load_binary_dashboard_template()?;
    println!("📄 Template loaded, length: {} chars", template.len());
    
    // Generate HTML content
    let html_content = generate_html_content(&template, &allocations, &stats, project_name)?;
    
    // Write HTML file
    fs::write(&html_path, html_content)
        .map_err(|e| BinaryExportError::Io(e))?;
    
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
    // Try to load from templates directory
    let template_paths = [
        "templates/binary_dashboard.html",
        "../templates/binary_dashboard.html", 
        "../../templates/binary_dashboard.html",
    ];
    
    for path in &template_paths {
        println!("🔍 Trying to load template from: {}", path);
        if let Ok(content) = fs::read_to_string(path) {
            println!("✅ Successfully loaded template from: {} ({} chars)", path, content.len());
            return Ok(content);
        } else {
            println!("❌ Failed to load from: {}", path);
        }
    }
    
    println!("⚠️ Using fallback embedded template");
    // Fallback to embedded template
    Ok(get_embedded_binary_template())
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
    let stats_data = prepare_stats_data(stats)?;
    
    // Replace template placeholders for binary_dashboard.html
    let mut html = template.to_string();
    
    // Replace basic placeholders
    html = html.replace("{{PROJECT_NAME}}", project_name);
    html = html.replace("{{TIMESTAMP}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    html = html.replace("{{GENERATION_TIME}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    
    // Replace data placeholders - binary_dashboard.html uses different placeholders
    html = html.replace("{{BINARY_DATA}}", &allocation_data);
    html = html.replace("{{ALLOCATION_DATA}}", &allocation_data);
    html = html.replace("{{STATS_DATA}}", &stats_data);
    html = html.replace("{{ json_data }}", &allocation_data);  // Template uses this format
    html = html.replace("{{json_data}}", &allocation_data);    // Alternative format
    
    // Replace statistics placeholders
    html = html.replace("{{TOTAL_ALLOCATIONS}}", &stats.total_allocations.to_string());
    html = html.replace("{{ACTIVE_ALLOCATIONS}}", &stats.active_allocations.to_string());
    html = html.replace("{{ACTIVE_MEMORY}}", &format_memory_size(stats.active_memory));
    html = html.replace("{{PEAK_MEMORY}}", &format_memory_size(stats.peak_memory));
    html = html.replace("{{LEAKED_ALLOCATIONS}}", &stats.leaked_allocations.to_string());
    html = html.replace("{{LEAKED_MEMORY}}", &format_memory_size(stats.leaked_memory));
    
    // Replace additional binary dashboard placeholders
    html = html.replace("{{SVG_IMAGES}}", "<!-- SVG images placeholder -->");
    html = html.replace("{{CSS_CONTENT}}", "/* Additional CSS placeholder */");
    html = html.replace("{{JS_CONTENT}}", "// Additional JavaScript placeholder");
    
    // Replace any remaining template variables to prevent errors
    html = html.replace("{{ json_data }}", &allocation_data);
    html = html.replace("{{json_data}}", &allocation_data);
    
    // Fix any remaining references to JS_CONTENT in comments and code
    html = html.replace("AFTER JS_CONTENT loads", "after additional JavaScript loads");
    html = html.replace("JS_CONTENT loads", "additional JavaScript loads");
    html = html.replace("JS_CONTENT", "additionalJavaScript");
    
    println!("📊 Data injection completed: {} allocations, {} stats", allocations.len(), stats.total_allocations);
    
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
                "original_ptr": clone_info.original_ptr.map(|p| format!("0x{:x}", p)),
            });
        }
        
        item["ownership_history_available"] = json!(allocation.ownership_history_available);
        
        allocation_data.push(item);
    }
    
    // Format data in the structure expected by binary_dashboard.html
    let data_structure = json!({
        "memory_analysis": {
            "allocations": allocation_data
        },
        "allocations": allocation_data  // Also provide direct access
    });
    
    serde_json::to_string(&data_structure)
        .map_err(|e| BinaryExportError::SerializationError(format!("Failed to serialize allocation data: {}", e)))
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
    
    serde_json::to_string(&data)
        .map_err(|e| BinaryExportError::SerializationError(format!("Failed to serialize stats data: {}", e)))
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
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Get embedded binary dashboard template - use actual binary_dashboard.html content
fn get_embedded_binary_template() -> String {
    // Force read the actual binary_dashboard.html template
    match std::fs::read_to_string("templates/binary_dashboard.html") {
        Ok(content) => {
            println!("✅ Successfully loaded binary_dashboard.html template ({} chars)", content.len());
            return content;
        },
        Err(e) => {
            println!("❌ Failed to load binary_dashboard.html: {}", e);
        }
    }
    
    // Fallback to a simplified version if file not found
    r#"<!DOCTYPE html>
<html lang="en" class="light">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{PROJECT_NAME}} - Binary Memory Analysis</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            darkMode: 'class',
        }
    </script>
    <link href="https://cdn.jsdelivr.net/npm/font-awesome@4.7.0/css/font-awesome.min.css" rel="stylesheet">
    
    <!-- Binary Data injection script -->
    <script>
        // Global data store - populated directly from binary data
        console.log('Binary data injection script loaded');
        try {
            window.analysisData = {{ALLOCATION_DATA}};
            window.embeddedJsonData = {{ALLOCATION_DATA}};
            window.dataSource = "binary_direct";
            window.generationTime = "{{TIMESTAMP}}";
            console.log('Analysis data loaded successfully:', window.analysisData ? 'YES' : 'NO');
        } catch (e) {
            console.error('Error loading analysis data:', e);
            window.analysisData = null;
        }
    </script>
</head>

<body class="bg-gray-50 dark:bg-gray-900 font-sans text-neutral dark:text-gray-100 transition-colors">
    <!-- Header -->
    <header class="bg-gradient-to-r from-blue-600 to-purple-600 text-white py-8">
        <div class="container mx-auto px-4 text-center">
            <h1 class="text-4xl font-bold mb-2">{{PROJECT_NAME}}</h1>
            <p class="text-xl opacity-90">Binary Memory Analysis Report - {{TIMESTAMP}}</p>
        </div>
    </header>

    <!-- Main Content -->
    <main class="container mx-auto px-4 py-8">
        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                <h3 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">Total Allocations</h3>
                <p class="text-3xl font-bold text-gray-900 dark:text-white">{{TOTAL_ALLOCATIONS}}</p>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                <h3 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">Active Memory</h3>
                <p class="text-3xl font-bold text-green-600">{{ACTIVE_MEMORY}}</p>
            </div>
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                <h3 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">Leaked Memory</h3>
                <p class="text-3xl font-bold text-red-600">{{LEAKED_MEMORY}}</p>
            </div>
        </div>

        <!-- Allocations Table -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden">
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 class="text-xl font-semibold text-gray-900 dark:text-white">Memory Allocations</h2>
            </div>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-700">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Pointer</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Size</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Variable</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Type</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Status</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Borrow Info</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Clone Info</th>
                        </tr>
                    </thead>
                    <tbody id="allocations-tbody" class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                        <tr><td colspan="7" class="px-6 py-4 text-center">Loading allocation data...</td></tr>
                    </tbody>
                </table>
            </div>
        </div>
    </main>

    <script>
        console.log('🚀 Binary Dashboard Loading...');
        
        // Use the injected data
        const allocations = window.analysisData || [];
        
        function formatSize(bytes) {
            if (!bytes || bytes === 0) return '0 B';
            const units = ['B', 'KB', 'MB', 'GB'];
            let size = bytes;
            let unitIndex = 0;
            
            while (size >= 1024 && unitIndex < units.length - 1) {
                size /= 1024;
                unitIndex++;
            }
            
            return unitIndex === 0 ? 
                `${bytes} ${units[unitIndex]}` : 
                `${size.toFixed(2)} ${units[unitIndex]}`;
        }
        
        function loadBinaryAllocations() {
            console.log('📊 Loading', allocations.length, 'allocations from binary data');
            
            const tbody = document.getElementById('allocations-tbody');
            if (!tbody) {
                console.error('❌ Table body not found');
                return;
            }
            
            if (!allocations || allocations.length === 0) {
                tbody.innerHTML = '<tr><td colspan="7" class="px-6 py-4 text-center">No allocation data available</td></tr>';
                return;
            }
            
            tbody.innerHTML = '';
            
            allocations.forEach((alloc, index) => {
                try {
                    const row = document.createElement('tr');
                    row.className = 'hover:bg-gray-50 dark:hover:bg-gray-700';
                    
                    const status = alloc.timestamp_dealloc ? 
                        '<span class="text-gray-600">Deallocated</span>' : 
                        (alloc.is_leaked ? '<span class="text-red-600 font-semibold">Leaked</span>' : '<span class="text-green-600 font-semibold">Active</span>');
                    
                    const borrowInfo = alloc.borrow_info ? 
                        `<div class="text-xs text-gray-600">I:${alloc.borrow_info.immutable_borrows || 0} M:${alloc.borrow_info.mutable_borrows || 0}</div>` : 
                        '<span class="text-gray-400">-</span>';
                    
                    const cloneInfo = alloc.clone_info ? 
                        `<div class="text-xs text-blue-600">Count:${alloc.clone_info.clone_count || 0}</div>` : 
                        '<span class="text-gray-400">-</span>';
                    
                    row.innerHTML = `
                        <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900 dark:text-gray-100">${alloc.ptr || 'N/A'}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm font-semibold text-green-600">${formatSize(alloc.size || 0)}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-orange-600 font-medium">${alloc.var_name || 'unnamed'}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-purple-600 font-medium">${alloc.type_name || 'unknown'}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm">${status}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm">${borrowInfo}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm">${cloneInfo}</td>
                    `;
                    
                    tbody.appendChild(row);
                } catch (e) {
                    console.error('❌ Error processing allocation', index, ':', e);
                }
            });
            
            console.log('✅ Binary allocations loaded successfully');
        }
        
        // Initialize when DOM is ready
        document.addEventListener('DOMContentLoaded', function() {
            console.log('📋 Binary Dashboard DOM ready');
            try {
                loadBinaryAllocations();
                console.log('✅ Binary dashboard initialized successfully');
            } catch (error) {
                console.error('❌ Failed to initialize binary dashboard:', error);
            }
        });
    </script>
</body>
</html>"#.to_string()
}

/// Public API function for binary to HTML conversion
pub fn parse_binary_to_html_direct<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
    project_name: &str,
) -> Result<(), BinaryExportError> {
    convert_binary_to_html(binary_path, html_path, project_name)
}