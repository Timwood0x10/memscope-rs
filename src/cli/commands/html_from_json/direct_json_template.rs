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
    
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS - Direct JSON Memory Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
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
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 16px;
            padding: 24px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
        }}

        .header h1 {{
            font-size: 28px;
            font-weight: 300;
            letter-spacing: 1px;
            color: #2c3e50;
            margin-bottom: 10px;
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
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}

        .metric-card {{
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
        }}

        .metric-card h3 {{
            color: #2c3e50;
            margin-bottom: 16px;
            font-size: 1.1rem;
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .metric-value {{
            font-size: 2rem;
            font-weight: bold;
            color: #3498db;
            margin-bottom: 8px;
        }}

        .metric-label {{
            color: #7f8c8d;
            font-size: 0.9rem;
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

        .chart-container {{
            position: relative;
            height: 400px;
            margin: 20px 0;
        }}

        .data-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}

        .data-card {{
            background: #f8fafc;
            border-radius: 8px;
            padding: 16px;
            border-left: 4px solid #3498db;
        }}

        .data-card h4 {{
            color: #2c3e50;
            margin-bottom: 12px;
        }}

        .data-item {{
            display: flex;
            justify-content: space-between;
            padding: 4px 0;
            border-bottom: 1px solid #e2e8f0;
        }}

        .data-item:last-child {{
            border-bottom: none;
        }}

        .data-label {{
            color: #64748b;
        }}

        .data-value {{
            font-weight: 600;
            color: #2c3e50;
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

        .json-viewer {{
            background: #2d3748;
            color: #e2e8f0;
            padding: 16px;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 0.9rem;
            overflow-x: auto;
            max-height: 400px;
            overflow-y: auto;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧠 MemScope-RS Memory Analysis</h1>
            <div class="subtitle">Direct JSON Data Visualization</div>
        </div>

        <div class="metrics-grid">
            <div class="metric-card">
                <h3>💾 Active Memory</h3>
                <div class="metric-value" id="activeMemory">Loading...</div>
                <div class="metric-label">Currently allocated</div>
            </div>
            <div class="metric-card">
                <h3>📊 Peak Memory</h3>
                <div class="metric-value" id="peakMemory">Loading...</div>
                <div class="metric-label">Maximum usage</div>
            </div>
            <div class="metric-card">
                <h3>🔢 Total Allocations</h3>
                <div class="metric-value" id="totalAllocations">Loading...</div>
                <div class="metric-label">All time</div>
            </div>
            <div class="metric-card">
                <h3>⚡ Memory Efficiency</h3>
                <div class="metric-value" id="memoryEfficiency">Loading...</div>
                <div class="metric-label">Performance rating</div>
            </div>
        </div>

        <div class="tab-nav">
            <button class="tab-btn active" onclick="showTab('overview')">📊 Overview</button>
            <button class="tab-btn" onclick="showTab('allocations')">🧠 Allocations</button>
            <button class="tab-btn" onclick="showTab('performance')">⚡ Performance</button>
            <button class="tab-btn" onclick="showTab('timeline')">⏱️ Timeline</button>
            <button class="tab-btn" onclick="showTab('security')">🔒 Security</button>
            <button class="tab-btn" onclick="showTab('raw-data')">📄 Raw Data</button>
        </div>

        <div class="content">
            <div id="overview" class="tab-content active">
                <h2>📊 Memory Overview</h2>
                <div class="chart-container">
                    <canvas id="allocationChart"></canvas>
                </div>
                <div id="overviewData" class="data-grid">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>

            <div id="allocations" class="tab-content">
                <h2>🧠 Memory Allocations</h2>
                <div id="allocationsList">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>

            <div id="performance" class="tab-content">
                <h2>⚡ Performance Analysis</h2>
                <div class="chart-container">
                    <canvas id="performanceChart"></canvas>
                </div>
                <div id="performanceData" class="data-grid">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>

            <div id="timeline" class="tab-content">
                <h2>⏱️ Allocation Timeline</h2>
                <div class="chart-container">
                    <canvas id="timelineChart"></canvas>
                </div>
                <div id="timelineData">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>

            <div id="security" class="tab-content">
                <h2>🔒 Security Analysis</h2>
                <div id="securityData" class="data-grid">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>

            <div id="raw-data" class="tab-content">
                <h2>📄 Raw JSON Data</h2>
                <div class="json-viewer" id="rawJsonData">
                    <!-- Will be populated by JavaScript -->
                </div>
            </div>
        </div>
    </div>

    <script>
        // 🎯 Raw JSON data - directly from files!
        let RAW_JSON_DATA;
        try {{
            RAW_JSON_DATA = {json_data_str};
            console.log('🚀 MemScope-RS initialized with direct JSON data');
            console.log('📊 Raw data loaded:', RAW_JSON_DATA);
            
            // Validate data structure
            if (!RAW_JSON_DATA || typeof RAW_JSON_DATA !== 'object') {{
                throw new Error('Invalid JSON data structure');
            }}
        }} catch (error) {{
            console.error('❌ Failed to load JSON data:', error);
            RAW_JSON_DATA = {{}};
            
            // Show error message to user
            document.addEventListener('DOMContentLoaded', function() {{
                document.body.innerHTML = `
                    <div style="padding: 40px; text-align: center; font-family: Arial, sans-serif;">
                        <h1 style="color: #e74c3c;">⚠️ Data Loading Error</h1>
                        <p>Failed to load memory analysis data. Please check the JSON files and try again.</p>
                        <p style="color: #7f8c8d; font-size: 0.9rem;">Error: ${{error.message}}</p>
                    </div>
                `;
            }});
        }}

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

        // Initialize the interface
        function initializeInterface() {{
            console.log('🎯 Initializing interface with direct JSON data...');
            
            // Update metrics from performance data
            updateMetrics();
            
            // Render different sections
            renderOverview();
            renderAllocations();
            renderPerformance();
            renderTimeline();
            renderSecurity();
            renderRawData();
            
            console.log('✅ Interface initialized successfully');
        }}

        function updateMetrics() {{
            console.log('📊 Updating metrics from data:', RAW_JSON_DATA);
            
            const performance = RAW_JSON_DATA.performance;
            const memoryAnalysis = RAW_JSON_DATA.memory_analysis;
            
            // Update active memory
            const activeMemoryEl = document.getElementById('activeMemory');
            if (activeMemoryEl) {{
                if (performance && performance.memory_performance && performance.memory_performance.active_memory) {{
                    activeMemoryEl.textContent = formatBytes(performance.memory_performance.active_memory);
                }} else {{
                    activeMemoryEl.textContent = 'N/A';
                }}
            }}
            
            // Update peak memory
            const peakMemoryEl = document.getElementById('peakMemory');
            if (peakMemoryEl) {{
                if (performance && performance.memory_performance && performance.memory_performance.peak_memory) {{
                    peakMemoryEl.textContent = formatBytes(performance.memory_performance.peak_memory);
                }} else {{
                    peakMemoryEl.textContent = 'N/A';
                }}
            }}
            
            // Update memory efficiency
            const memoryEfficiencyEl = document.getElementById('memoryEfficiency');
            if (memoryEfficiencyEl) {{
                if (performance && performance.memory_performance && typeof performance.memory_performance.memory_efficiency === 'number') {{
                    memoryEfficiencyEl.textContent = performance.memory_performance.memory_efficiency + '%';
                }} else {{
                    memoryEfficiencyEl.textContent = 'N/A';
                }}
            }}
            
            // Update total allocations
            const totalAllocationsEl = document.getElementById('totalAllocations');
            if (totalAllocationsEl) {{
                if (memoryAnalysis && memoryAnalysis.allocations && Array.isArray(memoryAnalysis.allocations)) {{
                    totalAllocationsEl.textContent = memoryAnalysis.allocations.length.toLocaleString();
                }} else {{
                    totalAllocationsEl.textContent = '0';
                }}
            }}
            
            console.log('✅ Metrics updated successfully');
        }}

        function renderOverview() {{
            console.log('📊 Rendering overview...');
            const performance = RAW_JSON_DATA.performance;
            const container = document.getElementById('overviewData');
            
            if (!container) {{
                console.error('❌ Overview container not found');
                return;
            }}
            
            if (performance && performance.allocation_distribution) {{
                const dist = performance.allocation_distribution;
                console.log('📊 Found allocation distribution:', dist);
                
                container.innerHTML = `
                    <div class="data-card">
                        <h4>📊 Allocation Distribution</h4>
                        <div class="data-item">
                            <span class="data-label">Tiny (&lt;64B):</span>
                            <span class="data-value">${{dist.tiny || 0}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Small (64B-1KB):</span>
                            <span class="data-value">${{dist.small || 0}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Medium (1KB-64KB):</span>
                            <span class="data-value">${{dist.medium || 0}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Large (64KB-1MB):</span>
                            <span class="data-value">${{dist.large || 0}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Massive (&gt;1MB):</span>
                            <span class="data-value">${{dist.massive || 0}}</span>
                        </div>
                    </div>
                `;
                
                // Create pie chart
                try {{
                    createAllocationChart(dist);
                }} catch (error) {{
                    console.error('❌ Failed to create allocation chart:', error);
                }}
            }} else {{
                console.warn('⚠️ No allocation distribution data found');
                container.innerHTML = `
                    <div class="data-card">
                        <h4>📊 Allocation Distribution</h4>
                        <p style="color: #7f8c8d; text-align: center; padding: 20px;">
                            No allocation distribution data available
                        </p>
                    </div>
                `;
            }}
        }}

        function createAllocationChart(distribution) {{
            console.log('📊 Creating allocation chart with data:', distribution);
            
            // Check if Chart.js is loaded
            if (typeof Chart === 'undefined') {{
                console.error('❌ Chart.js not loaded');
                const container = document.querySelector('.chart-container');
                if (container) {{
                    container.innerHTML = '<p style="text-align: center; color: #7f8c8d; padding: 40px;">Chart.js failed to load. Please check your internet connection.</p>';
                }}
                return;
            }}
            
            const canvas = document.getElementById('allocationChart');
            if (!canvas) {{
                console.error('❌ Chart canvas not found');
                return;
            }}
            
            try {{
                const ctx = canvas.getContext('2d');
                
                // Prepare data
                const chartData = [
                    distribution.tiny || 0,
                    distribution.small || 0,
                    distribution.medium || 0,
                    distribution.large || 0,
                    distribution.massive || 0
                ];
                
                // Only create chart if we have data
                const hasData = chartData.some(value => value > 0);
                if (!hasData) {{
                    ctx.fillStyle = '#7f8c8d';
                    ctx.font = '16px Arial';
                    ctx.textAlign = 'center';
                    ctx.fillText('No allocation data available', canvas.width / 2, canvas.height / 2);
                    return;
                }}
                
                new Chart(ctx, {{
                    type: 'pie',
                    data: {{
                        labels: ['Tiny (<64B)', 'Small (64B-1KB)', 'Medium (1KB-64KB)', 'Large (64KB-1MB)', 'Massive (>1MB)'],
                        datasets: [{{
                            data: chartData,
                            backgroundColor: [
                                '#2ecc71',  // Green for tiny
                                '#3498db',  // Blue for small
                                '#f39c12',  // Orange for medium
                                '#e74c3c',  // Red for large
                                '#9b59b6'   // Purple for massive
                            ],
                            borderWidth: 2,
                            borderColor: '#ffffff'
                        }}]
                    }},
                    options: {{
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: {{
                            title: {{
                                display: true,
                                text: 'Memory Allocation Size Distribution',
                                font: {{
                                    size: 16,
                                    weight: 'bold'
                                }}
                            }},
                            legend: {{
                                position: 'bottom',
                                labels: {{
                                    padding: 20,
                                    usePointStyle: true
                                }}
                            }}
                        }}
                    }}
                }});
                
                console.log('✅ Allocation chart created successfully');
            }} catch (error) {{
                console.error('❌ Failed to create chart:', error);
                const container = canvas.parentElement;
                if (container) {{
                    container.innerHTML = `<p style="text-align: center; color: #e74c3c; padding: 40px;">Failed to create chart: ${{error.message}}</p>`;
                }}
            }}
        }}

        function renderAllocations() {{
            console.log('🧠 Rendering allocations...');
            const memoryAnalysis = RAW_JSON_DATA.memory_analysis;
            const container = document.getElementById('allocationsList');
            
            if (!container) {{
                console.error('❌ Allocations container not found');
                return;
            }}
            
            if (memoryAnalysis && memoryAnalysis.allocations && Array.isArray(memoryAnalysis.allocations)) {{
                const allocations = memoryAnalysis.allocations.slice(0, 50); // Show first 50
                console.log(`📊 Rendering ${{allocations.length}} of ${{memoryAnalysis.allocations.length}} allocations`);
                
                if (allocations.length === 0) {{
                    container.innerHTML = '<p style="text-align: center; color: #7f8c8d; padding: 40px;">No allocations found</p>';
                    return;
                }}
                
                const html = allocations.map(alloc => `
                    <div class="allocation-item">
                        <div class="allocation-header">
                            <span class="allocation-ptr">${{alloc.ptr || 'Unknown'}}</span>
                            <span class="allocation-size">${{formatBytes(alloc.size || 0)}}</span>
                        </div>
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 8px; font-size: 0.9rem; color: #64748b;">
                            <div>Variable: ${{alloc.var_name || 'Unknown'}}</div>
                            <div>Type: ${{alloc.type_name || 'Unknown'}}</div>
                            <div>Status: ${{alloc.timestamp_dealloc ? 'Deallocated' : 'Active'}}</div>
                        </div>
                    </div>
                `).join('');
                
                container.innerHTML = `
                    <p style="margin-bottom: 16px; color: #64748b;">Showing ${{allocations.length}} of ${{memoryAnalysis.allocations.length}} total allocations</p>
                    ${{html}}
                `;
            }} else {{
                console.warn('⚠️ No allocation data found');
                container.innerHTML = '<p style="text-align: center; color: #7f8c8d; padding: 40px;">No allocation data available</p>';
            }}
        }}

        function renderPerformance() {{
            const performance = RAW_JSON_DATA.performance;
            const container = document.getElementById('performanceData');
            
            if (performance) {{
                container.innerHTML = `
                    <div class="data-card">
                        <h4>⚡ Processing Performance</h4>
                        <div class="data-item">
                            <span class="data-label">Allocations Processed:</span>
                            <span class="data-value">${{performance.export_performance?.allocations_processed || 0}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Processing Time:</span>
                            <span class="data-value">${{performance.export_performance?.total_processing_time_ms || 0}}ms</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Rate:</span>
                            <span class="data-value">${{(performance.export_performance?.processing_rate?.allocations_per_second || 0).toFixed(1)}} allocs/sec</span>
                        </div>
                    </div>
                    <div class="data-card">
                        <h4>💾 Memory Performance</h4>
                        <div class="data-item">
                            <span class="data-label">Active Memory:</span>
                            <span class="data-value">${{formatBytes(performance.memory_performance?.active_memory || 0)}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Peak Memory:</span>
                            <span class="data-value">${{formatBytes(performance.memory_performance?.peak_memory || 0)}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Total Allocated:</span>
                            <span class="data-value">${{formatBytes(performance.memory_performance?.total_allocated || 0)}}</span>
                        </div>
                        <div class="data-item">
                            <span class="data-label">Efficiency:</span>
                            <span class="data-value">${{performance.memory_performance?.memory_efficiency || 0}}%</span>
                        </div>
                    </div>
                `;
            }}
        }}

        function renderTimeline() {{
            const memoryAnalysis = RAW_JSON_DATA.memory_analysis;
            const container = document.getElementById('timelineData');
            
            if (memoryAnalysis && memoryAnalysis.allocations) {{
                const allocations = memoryAnalysis.allocations
                    .sort((a, b) => a.timestamp_alloc - b.timestamp_alloc)
                    .slice(0, 20);
                
                const html = allocations.map((alloc, index) => `
                    <div class="allocation-item">
                        <div class="allocation-header">
                            <span class="allocation-ptr">Event #${{index + 1}} - ${{alloc.ptr}}</span>
                            <span class="allocation-size">${{formatBytes(alloc.size)}}</span>
                        </div>
                        <div style="color: #64748b; font-size: 0.9rem;">
                            Time: ${{new Date(alloc.timestamp_alloc / 1000000).toLocaleTimeString()}} | 
                            Variable: ${{alloc.var_name || 'Unknown'}} | 
                            Type: ${{alloc.type_name || 'Unknown'}}
                        </div>
                    </div>
                `).join('');
                
                container.innerHTML = `
                    <p>Timeline of first 20 allocation events (sorted by time)</p>
                    ${{html}}
                `;
            }}
        }}

        function renderSecurity() {{
            const securityViolations = RAW_JSON_DATA.security_violations;
            const unsafeFfi = RAW_JSON_DATA.unsafe_ffi;
            const container = document.getElementById('securityData');
            
            let html = '';
            
            if (securityViolations) {{
                html += `
                    <div class="data-card">
                        <h4>🔒 Security Violations</h4>
                        <pre style="background: #f8fafc; padding: 12px; border-radius: 4px; font-size: 0.9rem;">${{JSON.stringify(securityViolations, null, 2)}}</pre>
                    </div>
                `;
            }}
            
            if (unsafeFfi) {{
                html += `
                    <div class="data-card">
                        <h4>⚠️ Unsafe/FFI Analysis</h4>
                        <pre style="background: #f8fafc; padding: 12px; border-radius: 4px; font-size: 0.9rem;">${{JSON.stringify(unsafeFfi, null, 2)}}</pre>
                    </div>
                `;
            }}
            
            container.innerHTML = html || '<p>No security data available</p>';
        }}

        function renderRawData() {{
            const container = document.getElementById('rawJsonData');
            container.textContent = JSON.stringify(RAW_JSON_DATA, null, 2);
        }}

        // Initialize when page loads
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('✅ Page loaded, initializing interface...');
            initializeInterface();
        }});
    </script>
</body>
</html>"#, json_data_str = json_data_str);

    Ok(html)
}