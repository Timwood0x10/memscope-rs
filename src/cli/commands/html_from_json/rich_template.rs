//! Rich HTML template generator with SVG-quality visualizations

use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;
use std::error::Error;

/// Generate a rich HTML template with SVG-quality visualizations
pub fn generate_rich_html(data: &UnifiedMemoryData) -> Result<String, Box<dyn Error>> {
    let stats = &data.stats;
    let allocations = &data.allocations;
    
    // Calculate metrics for visualizations
    let memory_efficiency = stats.memory_efficiency;
    let memory_usage_percent = if stats.peak_memory > 0 {
        (stats.active_memory as f64 / stats.peak_memory as f64 * 100.0) as u32
    } else {
        0
    };
    
    // Serialize data to JSON
    let json_data = serde_json::to_string(data)?;
    
    // Generate HTML with rich visualizations
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS - Interactive Memory Analysis</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: 'Segoe UI', Arial, sans-serif;
            background: linear-gradient(135deg, #ecf0f1 0%, #bdc3c7 100%);
            color: #2c3e50;
            line-height: 1.6;
            min-height: 100vh;
        }}

        .container {{
            max-width: 1800px;
            margin: 0 auto;
            padding: 20px;
        }}

        .header {{
            text-align: center;
            margin-bottom: 40px;
        }}

        .header h1 {{
            font-size: 28px;
            font-weight: 300;
            letter-spacing: 1px;
            color: #2c3e50;
            margin-bottom: 20px;
        }}

        .header .subtitle {{
            font-size: 11px;
            font-weight: 600;
            letter-spacing: 2px;
            color: #7f8c8d;
            text-transform: uppercase;
        }}

        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}

        .metric-card {{
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            display: flex;
            align-items: center;
            gap: 20px;
        }}

        .circular-progress {{
            position: relative;
            width: 50px;
            height: 50px;
        }}

        .circular-progress svg {{
            width: 50px;
            height: 50px;
            transform: rotate(-90deg);
        }}

        .circular-progress .background {{
            fill: none;
            stroke: #ecf0f1;
            stroke-width: 6;
        }}

        .circular-progress .progress {{
            fill: none;
            stroke-width: 6;
            stroke-linecap: round;
            transition: stroke-dashoffset 0.5s ease;
        }}

        .circular-progress .percentage {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 12px;
            font-weight: bold;
        }}

        .metric-info h3 {{
            font-size: 12px;
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 4px;
        }}

        .metric-info .value {{
            font-size: 16px;
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 8px;
        }}

        .metric-info .status {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .status-dot {{
            width: 8px;
            height: 8px;
            border-radius: 50%;
        }}

        .status-text {{
            font-size: 9px;
            font-weight: 600;
            text-transform: uppercase;
        }}

        .tab-nav {{
            display: flex;
            background: white;
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
            background: linear-gradient(135deg, #3498db, #2980b9);
            color: white;
            box-shadow: 0 4px 12px rgba(52, 152, 219, 0.3);
        }}

        .content {{
            background: white;
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

        .allocation-item {{
            background: white;
            border-radius: 8px;
            padding: 12px;
            margin-bottom: 8px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            border-left: 4px solid #3498db;
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
            font-size: 0.9rem;
        }}

        .allocation-size {{
            background: #e1f5fe;
            color: #0277bd;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: 600;
        }}

        .allocation-details {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 8px;
            font-size: 0.85rem;
            color: #64748b;
        }}

        .type-distribution {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
        }}

        .type-item {{
            background: white;
            border-radius: 8px;
            padding: 12px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .type-name {{
            font-weight: 600;
            color: #2c3e50;
        }}

        .type-stats {{
            color: #7f8c8d;
            font-size: 0.9rem;
        }}

        .timeline-container {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
        }}

        .timeline-event {{
            display: flex;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #ecf0f1;
        }}

        .timeline-event:last-child {{
            border-bottom: none;
        }}

        .timeline-dot {{
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: #3498db;
            margin-right: 16px;
            flex-shrink: 0;
        }}

        .timeline-content {{
            flex: 1;
        }}

        .timeline-title {{
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 4px;
        }}

        .timeline-details {{
            color: #7f8c8d;
            font-size: 0.9rem;
        }}

        .performance-metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
        }}

        .performance-card {{
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
        }}

        .performance-card h4 {{
            color: #2c3e50;
            margin-bottom: 16px;
            font-size: 1.1rem;
        }}

        .performance-stat {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #ecf0f1;
        }}

        .performance-stat:last-child {{
            border-bottom: none;
        }}

        .stat-label {{
            color: #64748b;
            font-weight: 500;
        }}

        .stat-value {{
            color: #2c3e50;
            font-weight: 600;
        }}

        .error-state {{
            text-align: center;
            padding: 40px;
            color: #e74c3c;
            background: #fdf2f2;
            border-radius: 8px;
            margin: 20px 0;
        }}

        /* Color classes for different metrics */
        .color-high {{ color: #e74c3c; }}
        .color-medium {{ color: #f39c12; }}
        .color-low {{ color: #2ecc71; }}
        .color-primary {{ color: #3498db; }}

        .stroke-high {{ stroke: #e74c3c; }}
        .stroke-medium {{ stroke: #f39c12; }}
        .stroke-low {{ stroke: #2ecc71; }}
        .stroke-primary {{ stroke: #3498db; }}

        .bg-high {{ background-color: #e74c3c; }}
        .bg-medium {{ background-color: #f39c12; }}
        .bg-low {{ background-color: #2ecc71; }}
        .bg-primary {{ background-color: #3498db; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Rust Memory Usage Analysis</h1>
            <div class="subtitle">Key Performance Metrics</div>
        </div>

        <div class="metrics-grid">
            <div class="metric-card">
                <div class="circular-progress">
                    <svg>
                        <circle class="background" cx="25" cy="25" r="20"/>
                        <circle class="progress stroke-primary" cx="25" cy="25" r="20" 
                                stroke-dasharray="125.66" 
                                stroke-dashoffset="{}"
                                style="stroke-dasharray: 125.66 125.66;"/>
                    </svg>
                    <div class="percentage color-primary">{}%</div>
                </div>
                <div class="metric-info">
                    <h3>Active Memory</h3>
                    <div class="value">{}</div>
                    <div class="status">
                        <div class="status-dot bg-medium"></div>
                        <div class="status-text color-medium">MEDIUM</div>
                    </div>
                </div>
            </div>

            <div class="metric-card">
                <div class="circular-progress">
                    <svg>
                        <circle class="background" cx="25" cy="25" r="20"/>
                        <circle class="progress stroke-high" cx="25" cy="25" r="20" 
                                stroke-dasharray="125.66" 
                                stroke-dashoffset="0"/>
                    </svg>
                    <div class="percentage color-high">100%</div>
                </div>
                <div class="metric-info">
                    <h3>Peak Memory</h3>
                    <div class="value">{}</div>
                    <div class="status">
                        <div class="status-dot bg-high"></div>
                        <div class="status-text color-high">HIGH</div>
                    </div>
                </div>
            </div>

            <div class="metric-card">
                <div class="circular-progress">
                    <svg>
                        <circle class="background" cx="25" cy="25" r="20"/>
                        <circle class="progress stroke-low" cx="25" cy="25" r="20" 
                                stroke-dasharray="125.66" 
                                stroke-dashoffset="0"/>
                    </svg>
                    <div class="percentage color-low">100%</div>
                </div>
                <div class="metric-info">
                    <h3>Active Allocs</h3>
                    <div class="value">{}</div>
                    <div class="status">
                        <div class="status-dot bg-low"></div>
                        <div class="status-text color-low">GOOD</div>
                    </div>
                </div>
            </div>

            <div class="metric-card">
                <div class="circular-progress">
                    <svg>
                        <circle class="background" cx="25" cy="25" r="20"/>
                        <circle class="progress stroke-{}color" cx="25" cy="25" r="20" 
                                stroke-dasharray="125.66" 
                                stroke-dashoffset="{}"/>
                    </svg>
                    <div class="percentage color-{}color">{:.0}%</div>
                </div>
                <div class="metric-info">
                    <h3>Memory Efficiency</h3>
                    <div class="value">{:.1}%</div>
                    <div class="status">
                        <div class="status-dot bg-{}color"></div>
                        <div class="status-text color-{}color">{}</div>
                    </div>
                </div>
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
                        <div class="performance-stat">
                            <span class="stat-label">Active Memory:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Peak Memory:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Total Allocations:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Active Allocations:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Memory Efficiency:</span>
                            <span class="stat-value">{:.1}%</span>
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div class="type-distribution">
                            {}
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Allocations</h3>
                        {}
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        {}
                    </div>
                </div>
            </div>

            <div id="memory" class="tab-content">
                <h2>🧠 Memory Analysis</h2>
                <p style="margin-bottom: 20px;">Detailed analysis of {} memory allocations</p>
                {}
            </div>

            <div id="timeline" class="tab-content">
                <h2>⏱️ Allocation Timeline</h2>
                <div class="timeline-container">
                    {}
                </div>
            </div>

            <div id="performance" class="tab-content">
                <h2>⚡ Performance Metrics</h2>
                <div class="performance-metrics">
                    <div class="performance-card">
                        <h4>Memory Efficiency</h4>
                        <div class="performance-stat">
                            <span class="stat-label">Efficiency:</span>
                            <span class="stat-value">{:.1}%</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Active/Peak Ratio:</span>
                            <span class="stat-value">{}%</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Total Allocated:</span>
                            <span class="stat-value">{}</span>
                        </div>
                    </div>
                    <div class="performance-card">
                        <h4>Allocation Patterns</h4>
                        <div class="performance-stat">
                            <span class="stat-label">Average Size:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Largest Allocation:</span>
                            <span class="stat-value">{}</span>
                        </div>
                        <div class="performance-stat">
                            <span class="stat-label">Smallest Allocation:</span>
                            <span class="stat-value">{}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div id="security" class="tab-content">
                <h2>🔒 Security Analysis</h2>
                {}
            </div>
        </div>
    </div>

    <script>
        // Embedded data - no loading needed!
        const DATA = {};
        
        console.log('🚀 MemScope-RS initialized with rich visualizations');
        console.log('📊 Data loaded:', DATA);

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
            return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
        }}

        // Initialize on page load
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('✅ Rich visualization interface loaded');
            console.log('📊 Stats:', DATA.stats);
            console.log('📋 Allocations:', DATA.allocations.length);
        }});
    </script>
</body>
</html>"#,
        // Circular progress calculations
        calculate_stroke_dashoffset(memory_usage_percent),
        memory_usage_percent,
        format_bytes(stats.active_memory),
        format_bytes(stats.peak_memory),
        stats.active_allocations,
        
        // Memory efficiency color and calculations
        get_efficiency_color(memory_efficiency),
        calculate_stroke_dashoffset(memory_efficiency as u32),
        get_efficiency_color(memory_efficiency),
        memory_efficiency,
        memory_efficiency,
        get_efficiency_color(memory_efficiency),
        get_efficiency_color(memory_efficiency),
        get_efficiency_status(memory_efficiency),
        
        // Overview stats
        format_bytes(stats.active_memory),
        format_bytes(stats.peak_memory),
        stats.total_allocations,
        stats.active_allocations,
        stats.memory_efficiency,
        
        // Type distribution
        generate_type_distribution_rich(data),
        
        // Recent allocations
        generate_recent_allocations_rich(allocations),
        
        // Performance insights
        generate_performance_insights_rich(stats),
        
        // Memory analysis
        allocations.len(),
        generate_memory_analysis_rich(allocations),
        
        // Timeline
        generate_timeline_rich(allocations),
        
        // Performance metrics
        stats.memory_efficiency,
        memory_usage_percent,
        format_bytes(stats.total_allocated),
        
        // Allocation patterns
        if stats.total_allocations > 0 { format_bytes(stats.total_allocated / stats.total_allocations) } else { "0 B".to_string() },
        format_bytes(allocations.iter().map(|a| a.size).max().unwrap_or(0)),
        format_bytes(allocations.iter().map(|a| a.size).min().unwrap_or(0)),
        
        // Security content
        generate_security_content_rich(data),
        
        // JSON data
        json_data
    );

    Ok(html)
}

fn calculate_stroke_dashoffset(percentage: u32) -> f64 {
    let circumference = 125.66; // 2 * π * 20
    circumference - (percentage as f64 / 100.0 * circumference)
}

fn get_efficiency_color(efficiency: f64) -> &'static str {
    if efficiency >= 70.0 {
        "low"
    } else if efficiency >= 40.0 {
        "medium"
    } else {
        "high"
    }
}

fn get_efficiency_status(efficiency: f64) -> &'static str {
    if efficiency >= 70.0 {
        "GOOD"
    } else if efficiency >= 40.0 {
        "MEDIUM"
    } else {
        "LOW"
    }
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

fn generate_type_distribution_rich(data: &UnifiedMemoryData) -> String {
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
            <div class="type-item">
                <span class="type-name">{}</span>
                <span class="type-stats">{} ({} allocs)</span>
            </div>
        "#, type_name, format_bytes(total_size), count)
    }).collect::<Vec<_>>().join("")
}

fn generate_recent_allocations_rich(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
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

fn generate_performance_insights_rich(stats: &crate::cli::commands::html_from_json::data_normalizer::MemoryStatistics) -> String {
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
        format!(r#"<div class="performance-stat"><span class="stat-value">{}</span></div>"#, insight)
    }).collect::<Vec<_>>().join("")
}

fn generate_memory_analysis_rich(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "<p>No allocations to analyze</p>".to_string();
    }
    
    let mut html = String::new();
    
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
        html.push_str(&format!("<p style=\"margin-top: 20px; text-align: center; color: #7f8c8d;\">... and {} more allocations</p>", allocations.len() - 20));
    }
    
    html
}

fn generate_timeline_rich(allocations: &[crate::cli::commands::html_from_json::data_normalizer::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "<p>No timeline events</p>".to_string();
    }
    
    let mut events: Vec<_> = allocations.iter().enumerate().collect();
    events.sort_by(|a, b| a.1.timestamp_alloc.cmp(&b.1.timestamp_alloc));
    
    let mut html = String::new();
    
    for (i, alloc) in events.iter().take(15) {
        let var_name = alloc.var_name.as_deref().unwrap_or("Unknown");
        let timestamp = alloc.timestamp_alloc / 1_000_000; // Convert to milliseconds
        
        html.push_str(&format!(r#"
            <div class="timeline-event">
                <div class="timeline-dot"></div>
                <div class="timeline-content">
                    <div class="timeline-title">Allocation #{} - {}</div>
                    <div class="timeline-details">
                        Variable: {} | Size: {} | Time: {}ms | Pointer: {}
                    </div>
                </div>
            </div>
        "#, i + 1, format_bytes(alloc.size), var_name, format_bytes(alloc.size), timestamp, alloc.ptr));
    }
    
    if events.len() > 15 {
        html.push_str(&format!("<p style=\"text-align: center; color: #7f8c8d; margin-top: 16px;\">... and {} more events</p>", events.len() - 15));
    }
    
    html
}

fn generate_security_content_rich(data: &UnifiedMemoryData) -> String {
    let mut html = String::new();
    
    // Check for potential security issues
    let large_allocations = data.allocations.iter().filter(|a| a.size > 1024 * 1024).count();
    let unknown_types = data.allocations.iter().filter(|a| a.type_name.is_none()).count();
    
    html.push_str("<div class=\"performance-metrics\">");
    
    html.push_str(&format!(r#"
        <div class="performance-card">
            <h4>Security Overview</h4>
            <div class="performance-stat">
                <span class="stat-label">Large Allocations (>1MB):</span>
                <span class="stat-value">{}</span>
            </div>
            <div class="performance-stat">
                <span class="stat-label">Unknown Types:</span>
                <span class="stat-value">{}</span>
            </div>
            <div class="performance-stat">
                <span class="stat-label">Total Allocations:</span>
                <span class="stat-value">{}</span>
            </div>
        </div>
    "#, large_allocations, unknown_types, data.allocations.len()));
    
    if large_allocations > 0 {
        html.push_str(r#"
        <div class="performance-card">
            <h4>Large Allocations</h4>
        "#);
        
        for alloc in data.allocations.iter().filter(|a| a.size > 1024 * 1024).take(5) {
            html.push_str(&format!(r#"
                <div class="performance-stat">
                    <span class="stat-label">{}</span>
                    <span class="stat-value">{}</span>
                </div>
            "#, alloc.ptr, format_bytes(alloc.size)));
        }
        
        html.push_str("</div>");
    }
    
    html.push_str("</div>");
    
    html
}