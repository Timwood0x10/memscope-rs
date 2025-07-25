//! Web page handlers for MemScope server
//!
//! Provides HTML page handlers for the web interface

use axum::{
    extract::State,
    response::Html,
};
use std::sync::Arc;

use super::server::ServerState;

/// Main dashboard page
pub async fn index(State(state): State<Arc<ServerState>>) -> Html<String> {
    let stats = &state.memory_data.stats;
    
    let html = create_enhanced_dashboard_html(stats, state.memory_data.allocations.len(), state.config.port);
    
    Html(html)
}

/// Create enhanced dashboard HTML with modern styling
fn create_enhanced_dashboard_html(stats: &crate::cli::commands::html_from_json::data_normalizer::MemoryStatistics, allocation_count: usize, port: u16) -> String {
    // Create a simple interactive dashboard that loads data from API endpoints
    format!(r#"<!DOCTYPE html>
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
            font-family: 'Segoe UI', 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
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
            transition: transform 0.2s ease;
        }}

        .stat-badge:hover {{
            transform: translateY(-2px);
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
            animation: fadeIn 0.3s ease;
        }}

        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
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
            border: 1px solid rgba(255, 255, 255, 0.2);
        }}

        .overview-card h3 {{
            margin-bottom: 16px;
            color: #1e293b;
            font-size: 1.2rem;
        }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 16px;
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

        .loading {{
            text-align: center;
            padding: 40px;
            color: #64748b;
        }}

        .error {{
            text-align: center;
            padding: 40px;
            color: #e74c3c;
        }}

        .api-endpoints {{
            background: #f8fafc;
            border-radius: 8px;
            padding: 20px;
            margin-top: 20px;
        }}

        .api-endpoints h3 {{
            margin-bottom: 16px;
            color: #1e293b;
        }}

        .endpoint {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 8px 0;
            border-bottom: 1px solid rgba(0, 0, 0, 0.1);
        }}

        .method {{
            background: #3498db;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: 600;
            min-width: 40px;
            text-align: center;
        }}

        .path {{
            font-family: monospace;
            color: #2c3e50;
        }}

        .description {{
            color: #64748b;
            font-size: 0.9rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <!-- Header -->
        <div class="header">
            <h1>🧠 MemScope-RS</h1>
            <div class="header-stats">
                <div class="stat-badge" id="totalMemory">{} bytes</div>
                <div class="stat-badge" id="activeAllocs">{} Active</div>
                <div class="stat-badge" id="peakMemory">{} bytes</div>
            </div>
        </div>

        <!-- Navigation Tabs -->
        <div class="tab-nav">
            <button class="tab-btn active" data-tab="overview">📊 Overview</button>
            <button class="tab-btn" data-tab="memory-analysis">🧠 Memory Analysis</button>
            <button class="tab-btn" data-tab="variables">📋 Variables</button>
            <button class="tab-btn" data-tab="timeline">⏱️ Timeline</button>
            <button class="tab-btn" data-tab="performance">⚡ Performance</button>
            <button class="tab-btn" data-tab="api">🔗 API</button>
        </div>

        <!-- Content Area -->
        <div class="content">
            <!-- Overview Tab -->
            <div class="tab-content active" id="overview">
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>📈 Memory Statistics</h3>
                        <div id="memoryStats">
                            <div class="stats-grid">
                                <div class="stat-item">
                                    <span class="stat-label">Active Memory:</span>
                                    <span class="stat-value">{} bytes</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Peak Memory:</span>
                                    <span class="stat-value">{} bytes</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Total Allocations:</span>
                                    <span class="stat-value">{}</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Active Allocations:</span>
                                    <span class="stat-value">{}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div id="typeDistribution" class="loading">Loading type distribution...</div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Variables</h3>
                        <div id="recentVariables" class="loading">Loading variables...</div>
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        <div id="performanceInsights" class="loading">Loading performance data...</div>
                    </div>
                </div>
            </div>

            <!-- Memory Analysis Tab -->
            <div class="tab-content" id="memory-analysis">
                <div id="memoryAnalysisContent" class="loading">Loading memory analysis...</div>
            </div>

            <!-- Variables Tab -->
            <div class="tab-content" id="variables">
                <div id="variablesContent" class="loading">Loading variables...</div>
            </div>

            <!-- Timeline Tab -->
            <div class="tab-content" id="timeline">
                <div id="timelineContent" class="loading">Loading timeline...</div>
            </div>

            <!-- Performance Tab -->
            <div class="tab-content" id="performance">
                <div id="performanceContent" class="loading">Loading performance metrics...</div>
            </div>

            <!-- API Tab -->
            <div class="tab-content" id="api">
                <div class="api-endpoints">
                    <h3>🔗 Available API Endpoints</h3>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/overview</span>
                        <span class="description">Memory overview and statistics</span>
                    </div>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/variables</span>
                        <span class="description">List all variables with pagination</span>
                    </div>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/variables/:name</span>
                        <span class="description">Get detailed information about a specific variable</span>
                    </div>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/timeline</span>
                        <span class="description">Get timeline events</span>
                    </div>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/performance</span>
                        <span class="description">Get performance metrics</span>
                    </div>
                    <div class="endpoint">
                        <span class="method">GET</span>
                        <span class="path">/api/search</span>
                        <span class="description">Search variables and types</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Initialize the application
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('🚀 Initializing MemScope-RS Web Interface...');
            
            // Initialize tabs
            initializeTabs();
            
            // Load data for different tabs
            loadOverviewData();
            
            console.log('✅ MemScope-RS Web Interface initialized');
        }});

        // Initialize tab navigation
        function initializeTabs() {{
            const tabButtons = document.querySelectorAll('.tab-btn');
            const tabContents = document.querySelectorAll('.tab-content');
            
            tabButtons.forEach(button => {{
                button.addEventListener('click', () => {{
                    const targetTab = button.getAttribute('data-tab');
                    
                    // Remove all active states
                    tabButtons.forEach(btn => btn.classList.remove('active'));
                    tabContents.forEach(content => content.classList.remove('active'));
                    
                    // Activate current tab
                    button.classList.add('active');
                    const targetContent = document.getElementById(targetTab);
                    if (targetContent) {{
                        targetContent.classList.add('active');
                        
                        // Load data for the tab if needed
                        loadTabData(targetTab);
                    }}
                }});
            }});
        }}

        // Load data for specific tabs
        function loadTabData(tabName) {{
            switch(tabName) {{
                case 'memory-analysis':
                    loadMemoryAnalysis();
                    break;
                case 'variables':
                    loadVariables();
                    break;
                case 'timeline':
                    loadTimeline();
                    break;
                case 'performance':
                    loadPerformance();
                    break;
            }}
        }}

        // Load overview data with performance optimization
        async function loadOverviewData() {{
            try {{
                // Parallel loading for better performance
                const [overviewResponse, variablesResponse] = await Promise.all([
                    fetch('/api/overview'),
                    fetch('/api/variables?limit=5')
                ]);
                
                const [overviewData, variablesData] = await Promise.all([
                    overviewResponse.json(),
                    variablesResponse.json()
                ]);
                
                console.log('Overview API response:', overviewData);
                console.log('Variables API response:', variablesData);
                
                const overview = overviewData.data || overviewData;
                const variables = variablesData.data || variablesData;
                
                updateTypeDistribution(overview.type_distribution || {{}});
                updatePerformanceInsights(overview.performance_insights || []);
                updateRecentVariables(variables.variables || variables.top_variables || []);
                
            }} catch (error) {{
                console.error('Failed to load overview data:', error);
                document.getElementById('typeDistribution').innerHTML = '<div class="error">❌ Failed to load type distribution</div>';
                document.getElementById('performanceInsights').innerHTML = '<div class="error">❌ Failed to load performance insights</div>';
                document.getElementById('recentVariables').innerHTML = '<div class="error">❌ Failed to load variables</div>';
            }}
        }}

        // Update type distribution
        function updateTypeDistribution(typeDistribution) {{
            const element = document.getElementById('typeDistribution');
            const types = Object.entries(typeDistribution).slice(0, 5);
            
            if (types.length === 0) {{
                element.innerHTML = '<p>No type information available</p>';
                return;
            }}
            
            const html = types.map(([typeName, data]) => {{
                const size = data.total_size || 0;
                const count = data.allocation_count || 0;
                return `
                    <div class="stat-item">
                        <span class="stat-label">${{typeName}}:</span>
                        <span class="stat-value">${{formatBytes(size)}} (${{count}} allocs)</span>
                    </div>
                `;
            }}).join('');
            
            element.innerHTML = html;
        }}

        // Update recent variables
        function updateRecentVariables(variables) {{
            const element = document.getElementById('recentVariables');
            const recent = variables.slice(0, 5);
            
            if (recent.length === 0) {{
                element.innerHTML = '<p>No variables found</p>';
                return;
            }}
            
            const html = recent.map(variable => `
                <div class="stat-item">
                    <span class="stat-label">${{variable.name || 'Unknown'}}:</span>
                    <span class="stat-value">${{formatBytes(variable.total_size || 0)}}</span>
                </div>
            `).join('');
            
            element.innerHTML = html;
        }}

        // Update performance insights
        function updatePerformanceInsights(insights) {{
            const element = document.getElementById('performanceInsights');
            
            if (insights.length === 0) {{
                element.innerHTML = '<div class="stat-item"><span class="stat-value">✅ Memory usage looks healthy</span></div>';
                return;
            }}
            
            const html = insights.map(insight => `
                <div class="stat-item">
                    <span class="stat-value">${{insight}}</span>
                </div>
            `).join('');
            
            element.innerHTML = html;
        }}

        // Load memory analysis with faster loading
        async function loadMemoryAnalysis() {{
            const element = document.getElementById('memoryAnalysisContent');
            element.innerHTML = '<div class="loading">⚡ Loading memory analysis...</div>';
            
            try {{
                const response = await fetch('/api/overview');
                const apiResponse = await response.json();
                const data = apiResponse.data || apiResponse;
                
                element.innerHTML = `
                    <h3>📊 Memory Analysis Dashboard</h3>
                    <div class="overview-grid">
                        <div class="overview-card">
                            <h4>💾 Memory Usage</h4>
                            <div class="stats-grid">
                                <div class="stat-item">
                                    <span class="stat-label">Active:</span>
                                    <span class="stat-value">${{formatBytes(data.stats?.active_memory || 0)}}</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Peak:</span>
                                    <span class="stat-value">${{formatBytes(data.stats?.peak_memory || 0)}}</span>
                                </div>
                            </div>
                        </div>
                        <div class="overview-card">
                            <h4>📈 Allocations</h4>
                            <div class="stats-grid">
                                <div class="stat-item">
                                    <span class="stat-label">Total:</span>
                                    <span class="stat-value">${{data.stats?.total_allocations || 0}}</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-label">Active:</span>
                                    <span class="stat-value">${{data.stats?.active_allocations || 0}}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            }} catch (error) {{
                console.error('Memory analysis load error:', error);
                element.innerHTML = '<div class="error">❌ Failed to load memory analysis</div>';
            }}
        }}

        // Load variables with optimized loading
        async function loadVariables() {{
            const element = document.getElementById('variablesContent');
            element.innerHTML = '<div class="loading">🔍 Loading variables...</div>';
            
            try {{
                const response = await fetch('/api/variables?limit=20');
                const apiResponse = await response.json();
                const data = apiResponse.data || apiResponse;
                const variables = data.variables || data.top_variables || [];
                
                if (variables.length === 0) {{
                    element.innerHTML = '<p>📭 No variables found</p>';
                    return;
                }}
                
                const html = `
                    <h3>📋 Variables List (Top ${{variables.length}})</h3>
                    <div class="overview-grid">
                        ${{variables.slice(0, 15).map(variable => `
                            <div class="overview-card">
                                <h4>🔧 ${{variable.name || 'Unknown'}}</h4>
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <span class="stat-label">Type:</span>
                                        <span class="stat-value">${{variable.type_name || 'Unknown'}}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Size:</span>
                                        <span class="stat-value">${{formatBytes(variable.total_size || 0)}}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Allocations:</span>
                                        <span class="stat-value">${{variable.allocation_count || 0}}</span>
                                    </div>
                                </div>
                            </div>
                        `).join('')}}
                    </div>
                `;
                element.innerHTML = html;
            }} catch (error) {{
                console.error('Variables load error:', error);
                element.innerHTML = '<div class="error">❌ Failed to load variables</div>';
            }}
        }}

        // Load timeline
        async function loadTimeline() {{
            const element = document.getElementById('timelineContent');
            element.innerHTML = '<div class="loading">⏱️ Loading timeline...</div>';
            
            try {{
                const response = await fetch('/api/timeline?limit=25');
                const apiResponse = await response.json();
                const data = apiResponse.data || apiResponse;
                const events = data.events || [];
                
                if (events.length === 0) {{
                    element.innerHTML = '<p>📅 No timeline events found</p>';
                    return;
                }}
                
                const html = `
                    <h3>⏱️ Timeline Events (Latest ${{events.length}})</h3>
                    <div class="overview-grid">
                        ${{events.slice(0, 20).map(event => `
                            <div class="overview-card">
                                <h4>📌 ${{event.event_type || 'Unknown'}} Event</h4>
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <span class="stat-label">Variable:</span>
                                        <span class="stat-value">${{event.variable_name || 'Unknown'}}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Size:</span>
                                        <span class="stat-value">${{formatBytes(event.size || 0)}}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Time:</span>
                                        <span class="stat-value">🕐 ${{new Date(event.timestamp * 1000).toLocaleTimeString()}}</span>
                                    </div>
                                </div>
                            </div>
                        `).join('')}}
                    </div>
                `;
                element.innerHTML = html;
            }} catch (error) {{
                console.error('Timeline load error:', error);
                element.innerHTML = '<div class="error">❌ Failed to load timeline</div>';
            }}
        }}

        // Load performance
        function loadPerformance() {{
            const element = document.getElementById('performanceContent');
            element.innerHTML = '<div class="loading">Loading performance metrics...</div>';
            
            fetch('/api/performance')
                .then(response => response.json())
                .then(apiResponse => {{
                    const data = apiResponse.data || apiResponse;
                    element.innerHTML = `
                        <h3>⚡ Performance Metrics</h3>
                        <div class="overview-grid">
                            <div class="overview-card">
                                <h4>Memory Efficiency</h4>
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <span class="stat-label">Utilization:</span>
                                        <span class="stat-value">${{(data.memory_utilization * 100).toFixed(1)}}%</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Fragmentation:</span>
                                        <span class="stat-value">${{(data.fragmentation_ratio * 100).toFixed(1)}}%</span>
                                    </div>
                                </div>
                            </div>
                            <div class="overview-card">
                                <h4>Allocation Patterns</h4>
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <span class="stat-label">Avg Size:</span>
                                        <span class="stat-value">${{formatBytes(data.average_allocation_size || 0)}}</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Lifetime:</span>
                                        <span class="stat-value">${{(data.average_lifetime || 0).toFixed(2)}}ms</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    `;
                }})
                .catch(error => {{
                    element.innerHTML = '<div class="error">Failed to load performance metrics</div>';
                }});
        }}

        // Format bytes into human-readable string
        function formatBytes(bytes) {{
            if (bytes === 0) return '0 B';
            
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }}
    </script>
</body>
</html>"#, 
        stats.active_memory,
        stats.active_allocations,
        stats.peak_memory,
        stats.active_memory,
        stats.peak_memory,
        stats.total_allocations,
        stats.active_allocations
    )
}