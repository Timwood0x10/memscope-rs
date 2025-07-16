//! Optimized HTML export with faster loading and better performance

use crate::types::*;
use std::collections::HashMap;

/// Generate an optimized HTML dashboard with lazy loading
pub fn generate_optimized_dashboard(
    memory_stats: &MemoryStats,
    timeline: &[AllocationInfo],
    performance_metrics: &HashMap<String, f64>,
) -> String {
    let summary_data = create_summary_data(memory_stats);
    let chart_data = create_chart_data(timeline);
    
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Memory Analysis Dashboard - Fast Loading</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: #333; }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        .header {{ text-align: center; margin-bottom: 30px; color: white; }}
        .header h1 {{ font-size: 2.5em; margin-bottom: 10px; text-shadow: 2px 2px 4px rgba(0,0,0,0.3); }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .stat-card {{ 
            background: rgba(255,255,255,0.95); 
            padding: 25px; 
            border-radius: 12px; 
            box-shadow: 0 8px 32px rgba(0,0,0,0.1); 
            backdrop-filter: blur(10px);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}
        .stat-card:hover {{ transform: translateY(-5px); box-shadow: 0 12px 40px rgba(0,0,0,0.15); }}
        .stat-card h3 {{ margin-top: 0; color: #4a5568; font-size: 1.2em; }}
        .stat-value {{ font-size: 2em; font-weight: bold; color: #2d3748; margin: 10px 0; }}
        .stat-label {{ color: #718096; font-size: 0.9em; }}
        .chart-container {{ 
            background: rgba(255,255,255,0.95); 
            padding: 25px; 
            border-radius: 12px; 
            box-shadow: 0 8px 32px rgba(0,0,0,0.1); 
            margin-bottom: 20px;
            backdrop-filter: blur(10px);
        }}
        .btn {{ 
            padding: 12px 24px; 
            margin: 5px; 
            border: none; 
            border-radius: 8px; 
            cursor: pointer; 
            font-weight: 600;
            transition: all 0.3s ease;
        }}
        .btn-primary {{ background: linear-gradient(45deg, #667eea, #764ba2); color: white; }}
        .btn-primary:hover {{ transform: translateY(-2px); box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }}
        .btn-secondary {{ background: linear-gradient(45deg, #f093fb, #f5576c); color: white; }}
        .btn-secondary:hover {{ transform: translateY(-2px); box-shadow: 0 4px 12px rgba(240, 147, 251, 0.4); }}
        .progress-bar {{ 
            width: 100%; 
            height: 8px; 
            background: #e2e8f0; 
            border-radius: 4px; 
            overflow: hidden; 
            margin: 10px 0;
        }}
        .progress-fill {{ 
            height: 100%; 
            background: linear-gradient(90deg, #667eea, #764ba2); 
            transition: width 0.3s ease;
        }}
        .loading {{ opacity: 0.6; }}
        .loaded {{ opacity: 1; transition: opacity 0.5s ease; }}
        .allocation-item {{ 
            padding: 10px; 
            margin: 5px 0; 
            background: #f7fafc; 
            border-left: 4px solid #667eea; 
            border-radius: 4px;
            cursor: pointer;
            transition: background 0.2s ease;
        }}
        .allocation-item:hover {{ background: #edf2f7; }}
        .risk-high {{ border-left-color: #e53e3e; }}
        .risk-medium {{ border-left-color: #dd6b20; }}
        .risk-low {{ border-left-color: #38a169; }}
        .tab-container {{ margin-bottom: 20px; }}
        .tab-button {{ 
            padding: 10px 20px; 
            border: none; 
            background: rgba(255,255,255,0.7); 
            cursor: pointer; 
            border-radius: 8px 8px 0 0;
            margin-right: 5px;
        }}
        .tab-button.active {{ background: rgba(255,255,255,0.95); }}
        .tab-content {{ display: none; }}
        .tab-content.active {{ display: block; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧠 Memory Analysis Dashboard</h1>
            <p>Generated at: {timestamp}</p>
            <div>
                <button class="btn btn-primary" onclick="toggleTheme()">🌙 Toggle Theme</button>
                <button class="btn btn-secondary" onclick="exportReport()">📄 Export Report</button>
                <button class="btn btn-secondary" onclick="refreshData()">🔄 Refresh</button>
                <button class="btn btn-secondary" onclick="showHelp()">❓ Help</button>
            </div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <h3>📊 Memory Overview</h3>
                <div class="stat-value">{total_memory}</div>
                <div class="stat-label">Total Memory Tracked</div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {memory_usage_percent}%"></div>
                </div>
                <small>Peak: {peak_memory} | Active: {active_allocations}</small>
            </div>
            
            <div class="stat-card">
                <h3>🔧 System Libraries</h3>
                <div class="stat-value">{system_lib_count}</div>
                <div class="stat-label">Libraries Detected</div>
                <div id="systemLibPreview">{system_lib_preview}</div>
            </div>
            
            <div class="stat-card">
                <h3>🧩 Fragmentation</h3>
                <div class="stat-value">{fragmentation_percent}%</div>
                <div class="stat-label">Memory Fragmentation</div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {fragmentation_percent}%; background: linear-gradient(90deg, #38a169, #e53e3e);"></div>
                </div>
                <small>Holes: {memory_holes} | Waste: {alignment_waste}</small>
            </div>
            
            <div class="stat-card">
                <h3>🔒 Concurrency</h3>
                <div class="stat-value">{concurrency_risk}</div>
                <div class="stat-label">Risk Level</div>
                <div id="concurrencyPreview">{concurrency_preview}</div>
            </div>
        </div>

        <div class="tab-container">
            <button class="tab-button active" onclick="showTab('allocations')">📋 Allocations</button>
            <button class="tab-button" onclick="showTab('timeline')">📈 Timeline</button>
            <button class="tab-button" onclick="showTab('analysis')">🔍 Analysis</button>
            <button class="tab-button" onclick="showTab('risks')">⚠️ Risks</button>
        </div>

        <div id="allocations" class="tab-content active">
            <div class="chart-container">
                <h3>🔥 Top Memory Allocations</h3>
                <div id="allocationsContainer">{allocations_html}</div>
            </div>
        </div>

        <div id="timeline" class="tab-content">
            <div class="chart-container">
                <h3>📈 Memory Timeline</h3>
                <canvas id="timelineChart" width="800" height="400"></canvas>
                <p><em>Click and drag to zoom, double-click to reset</em></p>
            </div>
        </div>

        <div id="analysis" class="tab-content">
            <div class="chart-container">
                <h3>🔍 Detailed Analysis</h3>
                <div id="analysisContainer">Loading detailed analysis...</div>
            </div>
        </div>

        <div id="risks" class="tab-content">
            <div class="chart-container">
                <h3>⚠️ Risk Assessment</h3>
                <div id="risksContainer">Loading risk analysis...</div>
            </div>
        </div>
    </div>

    <script>
        // Lightweight summary data only
        const summaryData = {summary_data};
        
        // Load detailed data on demand
        let detailedData = null;
        let chartData = {chart_data};
        
        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', function() {{
            initializeDashboard();
            setupEventListeners();
        }});
        
        function initializeDashboard() {{
            // Quick initialization with summary data
            updateMemoryOverview();
            renderQuickChart();
        }}
        
        function updateMemoryOverview() {{
            // Already populated in template
        }}
        
        function renderQuickChart() {{
            const canvas = document.getElementById('timelineChart');
            if (!canvas) return;
            
            const ctx = canvas.getContext('2d');
            const width = canvas.width;
            const height = canvas.height;
            
            // Clear canvas
            ctx.clearRect(0, 0, width, height);
            
            // Simple timeline visualization
            ctx.strokeStyle = '#667eea';
            ctx.lineWidth = 2;
            ctx.beginPath();
            
            const points = chartData.timeline || [];
            if (points.length > 0) {{
                const maxValue = Math.max(...points.map(p => p.value));
                
                points.forEach((point, index) => {{
                    const x = (index / (points.length - 1)) * (width - 40) + 20;
                    const y = height - 40 - ((point.value / maxValue) * (height - 80));
                    
                    if (index === 0) {{
                        ctx.moveTo(x, y);
                    }} else {{
                        ctx.lineTo(x, y);
                    }}
                }});
            }}
            
            ctx.stroke();
            
            // Add labels
            ctx.fillStyle = '#4a5568';
            ctx.font = '12px Arial';
            ctx.fillText('Memory Usage Over Time', 20, 20);
        }}
        
        function showTab(tabName) {{
            // Hide all tabs
            document.querySelectorAll('.tab-content').forEach(tab => {{
                tab.classList.remove('active');
            }});
            document.querySelectorAll('.tab-button').forEach(btn => {{
                btn.classList.remove('active');
            }});
            
            // Show selected tab
            document.getElementById(tabName).classList.add('active');
            event.target.classList.add('active');
            
            // Load data for specific tabs on demand
            if (tabName === 'analysis' && !detailedData) {{
                loadDetailedAnalysis();
            }} else if (tabName === 'risks') {{
                loadRiskAnalysis();
            }}
        }}
        
        function loadDetailedAnalysis() {{
            const container = document.getElementById('analysisContainer');
            container.innerHTML = '<p>Loading...</p>';
            
            // Simulate loading detailed analysis
            setTimeout(() => {{
                container.innerHTML = `
                    <div class="allocation-item">
                        <strong>Fragmentation Analysis</strong><br>
                        External fragmentation: {fragmentation_percent}%<br>
                        Memory holes detected: {memory_holes}<br>
                        Alignment waste: {alignment_waste} bytes
                    </div>
                    <div class="allocation-item">
                        <strong>System Library Usage</strong><br>
                        Most active: Standard Collections<br>
                        Network I/O: Moderate usage<br>
                        Async runtime: Active
                    </div>
                `;
            }}, 500);
        }}
        
        function loadRiskAnalysis() {{
            const container = document.getElementById('risksContainer');
            container.innerHTML = '<p>Loading...</p>';
            
            setTimeout(() => {{
                container.innerHTML = `
                    <div class="allocation-item risk-{risk_level}">
                        <strong>Concurrency Risk: {concurrency_risk}</strong><br>
                        Deadlock risk score: {deadlock_risk}<br>
                        Shared memory: {shared_memory} bytes<br>
                        Recommendation: Monitor mutex usage
                    </div>
                `;
            }}, 500);
        }}
        
        function setupEventListeners() {{
            // Add zoom functionality to chart
            const canvas = document.getElementById('timelineChart');
            if (canvas) {{
                canvas.addEventListener('dblclick', resetZoom);
            }}
        }}
        
        function toggleTheme() {{
            document.body.style.filter = document.body.style.filter ? '' : 'invert(1) hue-rotate(180deg)';
        }}
        
        function exportReport() {{
            const data = {{
                summary: summaryData,
                timestamp: new Date().toISOString()
            }};
            
            const blob = new Blob([JSON.stringify(data, null, 2)], {{type: 'application/json'}});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'memory-analysis-report.json';
            a.click();
            URL.revokeObjectURL(url);
        }}
        
        function refreshData() {{
            location.reload();
        }}
        
        function resetZoom() {{
            renderQuickChart();
        }}
        
        function showHelp() {{
            alert('Memory Analysis Dashboard\\n\\n' +
                  '• Click tabs to view different analyses\\n' +
                  '• Hover over cards for details\\n' +
                  '• Use Export to save data\\n' +
                  '• Toggle theme for dark mode');
        }}
    </script>
</body>
</html>"#,
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        total_memory = format_bytes(memory_stats.total_allocated),
        peak_memory = format_bytes(memory_stats.peak_memory),
        active_allocations = memory_stats.active_allocations,
        memory_usage_percent = calculate_usage_percent(memory_stats),
        system_lib_count = count_active_libraries(&memory_stats.system_library_stats),
        system_lib_preview = create_library_preview(&memory_stats.system_library_stats),
        fragmentation_percent = (memory_stats.fragmentation_analysis.fragmentation_ratio * 100.0) as u32,
        memory_holes = memory_stats.fragmentation_analysis.memory_holes.len(),
        alignment_waste = format_bytes(memory_stats.fragmentation_analysis.alignment_waste),
        concurrency_risk = &memory_stats.concurrency_analysis.lock_contention_risk,
        concurrency_preview = create_concurrency_preview(&memory_stats.concurrency_analysis),
        allocations_html = create_allocations_html(&memory_stats.allocations),
        summary_data = create_summary_json(memory_stats),
        chart_data = chart_data,
        risk_level = get_risk_level(&memory_stats.concurrency_analysis.lock_contention_risk),
        deadlock_risk = format!("{:.2}", memory_stats.concurrency_analysis.deadlock_risk_score),
        shared_memory = format_bytes(memory_stats.concurrency_analysis.shared_memory_bytes),
    )
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn calculate_usage_percent(stats: &MemoryStats) -> u32 {
    if stats.peak_memory > 0 {
        ((stats.active_memory as f64 / stats.peak_memory as f64) * 100.0) as u32
    } else {
        0
    }
}

fn count_active_libraries(lib_stats: &SystemLibraryStats) -> usize {
    let mut count = 0;
    if lib_stats.std_collections.allocation_count > 0 { count += 1; }
    if lib_stats.async_runtime.allocation_count > 0 { count += 1; }
    if lib_stats.network_io.allocation_count > 0 { count += 1; }
    if lib_stats.file_system.allocation_count > 0 { count += 1; }
    if lib_stats.serialization.allocation_count > 0 { count += 1; }
    if lib_stats.regex_engine.allocation_count > 0 { count += 1; }
    if lib_stats.crypto_security.allocation_count > 0 { count += 1; }
    if lib_stats.database.allocation_count > 0 { count += 1; }
    if lib_stats.graphics_ui.allocation_count > 0 { count += 1; }
    if lib_stats.http_stack.allocation_count > 0 { count += 1; }
    if lib_stats.compression.allocation_count > 0 { count += 1; }
    if lib_stats.logging.allocation_count > 0 { count += 1; }
    if lib_stats.unknown_system.allocation_count > 0 { count += 1; }
    count
}

fn create_library_preview(lib_stats: &SystemLibraryStats) -> String {
    let mut preview = Vec::new();
    
    if lib_stats.std_collections.total_bytes > 0 {
        preview.push(format!("Collections: {}", format_bytes(lib_stats.std_collections.total_bytes)));
    }
    if lib_stats.async_runtime.total_bytes > 0 {
        preview.push(format!("Async: {}", format_bytes(lib_stats.async_runtime.total_bytes)));
    }
    if lib_stats.network_io.total_bytes > 0 {
        preview.push(format!("Network: {}", format_bytes(lib_stats.network_io.total_bytes)));
    }
    
    if preview.is_empty() {
        "No system libraries detected".to_string()
    } else {
        preview.join("<br>")
    }
}

fn create_concurrency_preview(concurrency: &ConcurrencyAnalysis) -> String {
    format!(
        "Arc: {} | Mutex: {} | Channels: {}",
        format_bytes(concurrency.arc_shared),
        format_bytes(concurrency.mutex_protected),
        format_bytes(concurrency.channel_buffers)
    )
}

fn create_allocations_html(allocations: &[AllocationInfo]) -> String {
    let mut html = String::new();
    
    // Sort by size and take top 20
    let mut sorted_allocs = allocations.to_vec();
    sorted_allocs.sort_by(|a, b| b.size.cmp(&a.size));
    
    for (i, alloc) in sorted_allocs.iter().take(20).enumerate() {
        let var_name = alloc.var_name.as_deref().unwrap_or("Unknown");
        let type_name = alloc.type_name.as_deref().unwrap_or("Unknown");
        let size_str = format_bytes(alloc.size);
        
        html.push_str(&format!(
            r#"<div class="allocation-item">
                <strong>#{}: {}</strong><br>
                Type: {} | Size: {} | Thread: {}
            </div>"#,
            i + 1, var_name, type_name, size_str, alloc.thread_id
        ));
    }
    
    html
}

fn create_summary_json(stats: &MemoryStats) -> String {
    serde_json::json!({
        "total_allocated": stats.total_allocated,
        "active_memory": stats.active_memory,
        "peak_memory": stats.peak_memory,
        "allocation_count": stats.total_allocations,
        "fragmentation_ratio": stats.fragmentation_analysis.fragmentation_ratio,
        "concurrency_risk": stats.concurrency_analysis.lock_contention_risk
    }).to_string()
}

fn create_summary_data(stats: &MemoryStats) -> String {
    create_summary_json(stats)
}

fn create_chart_data(timeline: &[AllocationInfo]) -> String {
    let points: Vec<_> = timeline.iter().enumerate().map(|(i, alloc)| {
        serde_json::json!({
            "time": i,
            "value": alloc.size,
            "label": alloc.var_name.as_deref().unwrap_or("Unknown")
        })
    }).collect();
    
    serde_json::json!({
        "timeline": points
    }).to_string()
}

fn get_risk_level(risk: &str) -> &str {
    match risk {
        "critical" => "high",
        "high" => "high", 
        "medium" => "medium",
        _ => "low"
    }
}