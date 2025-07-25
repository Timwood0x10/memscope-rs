//! Fixed HTML template generator with rich visualizations similar to SVG reports

use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;
use std::error::Error;

/// Generate a fixed HTML template that immediately renders all data
pub fn generate_fixed_html(data: &UnifiedMemoryData) -> Result<String, Box<dyn Error>> {
    let stats = &data.stats;
    let allocations = &data.allocations;
    
    // Serialize data to JSON
    let json_data = serde_json::to_string(data)?;
    
    // Generate HTML with immediate data rendering
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS - Memory Analysis Report</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #2c3e50;
            line-height: 1.6;
            min-height: 100vh;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }}

        .header {{
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 16px;
            padding: 24px;
            margin-bottom: 24px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
        }}

        .header h1 {{
            font-size: 2.5rem;
            font-weight: 700;
            background: linear-gradient(135deg, #667eea, #764ba2);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }}

        .header-stats {{
            display: flex;
            gap: 16px;
            flex-wrap: wrap;
        }}

        .stat-badge {{
            background: linear-gradient(135deg, #3498db, #2980b9);
            color: white;
            padding: 8px 16px;
            border-radius: 20px;
            font-weight: 600;
            font-size: 0.9rem;
            box-shadow: 0 4px 12px rgba(52, 152, 219, 0.3);
        }}

        .tab-nav {{
            display: flex;
            background: rgba(255, 255, 255, 0.9);
            border-radius: 12px;
            padding: 8px;
            margin-bottom: 24px;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
            gap: 4px;
            overflow-x: auto;
        }}

        .tab-btn {{
            background: transparent;
            border: none;
            padding: 12px 20px;
            border-radius: 8px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.3s ease;
            white-space: nowrap;
            color: #64748b;
        }}

        .tab-btn:hover {{
            background: rgba(102, 126, 234, 0.1);
            color: #667eea;
        }}

        .tab-btn.active {{
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        }}

        .content {{
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 16px;
            padding: 24px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            min-height: 600px;
        }}

        .tab-content {{
            display: none;
        }}

        .tab-content.active {{
            display: block;
        }}

        .overview-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 24px;
        }}

        .overview-card {{
            background: linear-gradient(135deg, #f8fafc, #e2e8f0);
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
        }}

        .overview-card h3 {{
            margin-bottom: 16px;
            color: #1e293b;
            font-size: 1.2rem;
        }}

        .stats-grid {{
            display: grid;
            grid-template-columns: 1fr;
            gap: 12px;
        }}

        .stat-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid rgba(0, 0, 0, 0.1);
        }}

        .stat-label {{
            color: #64748b;
            font-weight: 500;
        }}

        .stat-value {{
            color: #1e293b;
            font-weight: 600;
        }}

        .allocation-item {{
            background: white;
            border-radius: 8px;
            padding: 12px;
            margin-bottom: 8px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }}

        .allocation-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
        }}

        .allocation-ptr {{
            font-family: monospace;
            color: #667eea;
            font-weight: 600;
        }}

        .allocation-size {{
            background: #e1f5fe;
            color: #0277bd;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.9rem;
        }}

        .allocation-details {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 8px;
            font-size: 0.9rem;
            color: #64748b;
        }}

        .error-state {{
            text-align: center;
            padding: 40px;
            color: #e74c3c;
            background: #fdf2f2;
            border-radius: 8px;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧠 MemScope-RS</h1>
            <div class="header-stats">
                <div class="stat-badge">{} bytes</div>
                <div class="stat-badge">{} Active</div>
                <div class="stat-badge">{} bytes Peak</div>
            </div>
        </div>

        <div class="tab-nav">
            <button class="tab-btn active" onclick="showTab('overview')">📊 Overview</button>
            <button class="tab-btn" onclick="showTab('memory')">🧠 Memory Analysis</button>
            <button class="tab-btn" onclick="showTab('timeline')">⏱️ Timeline</button>
            <button class="tab-btn" onclick="showTab('performance')">⚡ Performance</button>
            <button class="tab-btn" onclick="showTab('security')">🔒 Security</button>
        </div>

        <div class="content">
            <div id="overview" class="tab-content active">
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>📈 Memory Statistics</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Active Memory:</span>
                                <span class="stat-value">{}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Peak Memory:</span>
                                <span class="stat-value">{}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Total Allocations:</span>
                                <span class="stat-value">{}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Active Allocations:</span>
                                <span class="stat-value">{}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Memory Efficiency:</span>
                                <span class="stat-value">{}%</span>
                            </div>
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div id="typeDistribution">
                            {}
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Allocations</h3>
                        <div id="recentAllocations">
                            {}
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        <div id="performanceInsights">
                            {}
                        </div>
                    </div>
                </div>
            </div>

            <div id="memory" class="tab-content">
                <h2>🧠 Memory Analysis</h2>
                <div id="allocationsList">
                    {}
                </div>
            </div>

            <div id="timeline" class="tab-content">
                <h2>⏱️ Timeline</h2>
                <div id="timelineContent">
                    {}
                </div>
            </div>

            <div id="performance" class="tab-content">
                <h2>⚡ Performance Metrics</h2>
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>Memory Efficiency</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Efficiency:</span>
                                <span class="stat-value">{}%</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Active/Peak Ratio:</span>
                                <span class="stat-value">{}%</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div id="security" class="tab-content">
                <h2>🔒 Security Analysis</h2>
                <div id="securityContent">
                    {}
                </div>
            </div>
        </div>
    </div>

    <script>
        // Embedded data - no loading needed!
        const DATA = {};
        
        console.log('🚀 MemScope-RS initialized with data:', DATA);

        // Tab switching function
        function showTab(tabName) {{
            // Hide all tabs
            const tabs = document.querySelectorAll('.tab-content');
            tabs.forEach(tab => tab.classList.remove('active'));
            
            // Remove active class from all buttons
            const buttons = document.querySelectorAll('.tab-btn');
            buttons.forEach(btn => btn.classList.remove('active'));
            
            // Show selected tab
            const selectedTab = document.getElementById(tabName);
            if (selectedTab) {{
                selectedTab.classList.add('active');
            }}
            
            // Activate button
            const selectedButton = document.querySelector(`[onclick="showTab('${{tabName}}')"]`);
            if (selectedButton) {{
                selectedButton.classList.add('active');
            }}
            
            console.log(`Switched to tab: ${{tabName}}`);
        }}

        // Format bytes function
        function formatBytes(bytes) {{
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }}

        // Initialize on page load
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('✅ Page loaded, data is immediately available');
            console.log('📊 Stats:', DATA.stats);
            console.log('📋 Allocations:', DATA.allocations.length);
        }});
    </script>
</body>
</html>"#,
        // Header stats
        format_bytes(stats.active_memory),
        stats.active_allocations,
        format_bytes(stats.peak_memory),
        
        // Overview stats
        format_bytes(stats.active_memory),
        format_bytes(stats.peak_memory),
        stats.total_allocations,
        stats.active_allocations,
        stats.memory_efficiency,
        
        // Type distribution
        generate_type_distribution(data),
        
        // Recent allocations
        generate_recent_allocations(allocations),
        
        // Performance insights
        generate_performance_insights(stats),
        
        // Memory analysis
        generate_memory_analysis(allocations),
        
        // Timeline
        generate_timeline(allocations),
        
        // Performance metrics
        stats.memory_efficiency,
        if stats.peak_memory > 0 { (stats.active_memory as f64 / stats.peak_memory as f64 * 100.0) as u32 } else { 0 },
        
        // Security content
        generate_security_content(data),
        
        // JSON data
        json_data
    );

    Ok(html)
}

fn format_bytes(bytes: usize) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let units = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, units[unit_index])
    } else {
        format!("{:.1} {}", size, units[unit_index])
    }
}

fn generate_type_distribution(data: &UnifiedMemoryData) -> String {
    let mut type_counts: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
    
    for allocation in &data.allocations {
        let type_name = allocation.type_name.as_deref().unwrap_or("Unknown").to_string();
        let entry = type_counts.entry(type_name).or_insert((0, 0));
        entry.0 += allocation.size;
        entry.1 += 1;
    }
    
    let mut types: Vec<_> = type_counts.into_iter().collect();
    types.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Sort by total size
    
    if types.is_empty() {
        return "<p>No type information available</p>".to_string();
    }
    
    types.into_iter().take(5).map(|(type_name, (total_size, count))| {
        format!(r#"
            <div class="stat-item">
                <span class="stat-label">{}:</span>
                <span class="stat-value">{} ({} allocs)</span>
            </div>
        "#, type_name, format_bytes(total_size), count)
    }).collect::<Vec<_>>().join("")
}

fn generate_recent_allocations(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "<p>No allocations found</p>".to_string();
    }
    
    allocations.iter().take(5).map(|alloc| {
        let var_name = alloc.var_name.as_deref().unwrap_or("Unknown");
        let type_name = alloc.type_name.as_deref().unwrap_or("Unknown");
        
        format!(r#"
            <div class="allocation-item">
                <div class="allocation-header">
                    <span class="allocation-ptr">{}</span>
                    <span class="allocation-size">{}</span>
                </div>
                <div class="allocation-details">
                    <div>Variable: {}</div>
                    <div>Type: {}</div>
                </div>
            </div>
        "#, alloc.ptr, format_bytes(alloc.size), var_name, type_name)
    }).collect::<Vec<_>>().join("")
}

fn generate_performance_insights(stats: &crate::cli::commands::html_from_json::data_normalizer::MemoryStatistics) -> String {
    let mut insights = Vec::new();
    
    if stats.memory_efficiency < 50.0 {
        insights.push("⚠️ Low memory efficiency detected");
    } else {
        insights.push("✅ Good memory efficiency");
    }
    
    if stats.active_allocations > 1000 {
        insights.push("📊 High number of active allocations");
    }
    
    if stats.peak_memory > stats.active_memory * 2 {
        insights.push("📈 Significant memory peak detected");
    }
    
    if insights.is_empty() {
        insights.push("✅ Memory usage looks healthy");
    }
    
    insights.into_iter().map(|insight| {
        format!(r#"<div class="stat-item"><span class="stat-value">{}</span></div>"#, insight)
    }).collect::<Vec<_>>().join("")
}

fn generate_memory_analysis(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "<p>No allocations to analyze</p>".to_string();
    }
    
    let mut html = String::new();
    html.push_str(&format!("<p>Total allocations: {}</p>", allocations.len()));
    
    for (i, alloc) in allocations.iter().take(20).enumerate() {
        let var_name = alloc.var_name.as_deref().unwrap_or("Unknown");
        let type_name = alloc.type_name.as_deref().unwrap_or("Unknown");
        
        html.push_str(&format!(r#"
            <div class="allocation-item">
                <div class="allocation-header">
                    <span class="allocation-ptr">#{} {}</span>
                    <span class="allocation-size">{}</span>
                </div>
                <div class="allocation-details">
                    <div>Variable: {}</div>
                    <div>Type: {}</div>
                    <div>Status: {}</div>
                </div>
            </div>
        "#, i + 1, alloc.ptr, format_bytes(alloc.size), var_name, type_name, 
        if alloc.timestamp_dealloc.is_none() { "Active" } else { "Deallocated" }));
    }
    
    if allocations.len() > 20 {
        html.push_str(&format!("<p>... and {} more allocations</p>", allocations.len() - 20));
    }
    
    html
}

fn generate_timeline(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "<p>No timeline events</p>".to_string();
    }
    
    let mut events: Vec<_> = allocations.iter().enumerate().collect();
    events.sort_by(|a, b| a.1.timestamp_alloc.cmp(&b.1.timestamp_alloc));
    
    let mut html = String::new();
    html.push_str(&format!("<p>Timeline of {} allocation events:</p>", events.len()));
    
    for (i, alloc) in events.iter().take(15) {
        let var_name = alloc.var_name.as_deref().unwrap_or("Unknown");
        let timestamp = alloc.timestamp_alloc / 1_000_000; // Convert to milliseconds
        
        html.push_str(&format!(r#"
            <div class="allocation-item">
                <div class="allocation-header">
                    <span class="allocation-ptr">Event #{}</span>
                    <span class="allocation-size">{}</span>
                </div>
                <div class="allocation-details">
                    <div>Variable: {}</div>
                    <div>Time: {}ms</div>
                    <div>Pointer: {}</div>
                </div>
            </div>
        "#, i + 1, format_bytes(alloc.size), var_name, timestamp, alloc.ptr));
    }
    
    if events.len() > 15 {
        html.push_str(&format!("<p>... and {} more events</p>", events.len() - 15));
    }
    
    html
}

fn generate_security_content(data: &UnifiedMemoryData) -> String {
    let mut html = String::new();
    
    // Check for potential security issues
    let large_allocations = data.allocations.iter().filter(|a| a.size > 1024 * 1024).count();
    let unknown_types = data.allocations.iter().filter(|a| a.type_name.is_none()).count();
    
    html.push_str("<div class=\"overview-grid\">");
    
    html.push_str(&format!(r#"
        <div class="overview-card">
            <h3>Security Overview</h3>
            <div class="stats-grid">
                <div class="stat-item">
                    <span class="stat-label">Large Allocations (>1MB):</span>
                    <span class="stat-value">{}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">Unknown Types:</span>
                    <span class="stat-value">{}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">Total Allocations:</span>
                    <span class="stat-value">{}</span>
                </div>
            </div>
        </div>
    "#, large_allocations, unknown_types, data.allocations.len()));
    
    html.push_str("</div>");
    
    if large_allocations > 0 {
        html.push_str("<h3>Large Allocations:</h3>");
        for alloc in data.allocations.iter().filter(|a| a.size > 1024 * 1024).take(5) {
            html.push_str(&format!(r#"
                <div class="allocation-item">
                    <div class="allocation-header">
                        <span class="allocation-ptr">{}</span>
                        <span class="allocation-size">{}</span>
                    </div>
                </div>
            "#, alloc.ptr, format_bytes(alloc.size)));
        }
    }
    
    html
}