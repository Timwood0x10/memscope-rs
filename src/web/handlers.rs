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
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        
        .header {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            margin-bottom: 30px;
            text-align: center;
        }}
        
        .header h1 {{
            color: #2563eb;
            font-size: 2.5rem;
            margin-bottom: 10px;
        }}
        
        .header p {{
            color: #666;
            font-size: 1.1rem;
        }}
        
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        
        .stat-card {{
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
        }}
        
        .stat-card h3 {{
            color: #374151;
            font-size: 0.9rem;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 10px;
        }}
        
        .stat-card .value {{
            font-size: 2rem;
            font-weight: bold;
            color: #2563eb;
            margin-bottom: 5px;
        }}
        
        .stat-card .unit {{
            color: #666;
            font-size: 0.9rem;
        }}
        
        .nav-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        
        .nav-card {{
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
            cursor: pointer;
            text-decoration: none;
            color: inherit;
        }}
        
        .nav-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 20px rgba(0,0,0,0.15);
        }}
        
        .nav-card h3 {{
            color: #2563eb;
            font-size: 1.3rem;
            margin-bottom: 10px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .nav-card p {{
            color: #666;
            line-height: 1.5;
        }}
        
        .api-section {{
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        
        .api-section h2 {{
            color: #374151;
            margin-bottom: 20px;
            font-size: 1.5rem;
        }}
        
        .api-endpoint {{
            background: #f8fafc;
            padding: 15px;
            border-radius: 5px;
            margin-bottom: 10px;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 0.9rem;
            border-left: 4px solid #2563eb;
        }}
        
        .method {{
            color: #059669;
            font-weight: bold;
            margin-right: 10px;
        }}
        
        .endpoint {{
            color: #1f2937;
        }}
        
        .description {{
            color: #6b7280;
            margin-left: 50px;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }}
        
        .footer {{
            text-align: center;
            margin-top: 50px;
            padding: 20px;
            color: #666;
            border-top: 1px solid #e5e7eb;
        }}
        
        @media (max-width: 768px) {{
            .container {{
                padding: 10px;
            }}
            
            .header h1 {{
                font-size: 2rem;
            }}
            
            .stats-grid {{
                grid-template-columns: 1fr;
            }}
            
            .nav-grid {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 MemScope-RS</h1>
            <p>Interactive Rust Memory Analysis Dashboard</p>
        </div>
        
        <div class="stats-grid">
            <div class="stat-card">
                <h3>Active Memory</h3>
                <div class="value">{active_memory}</div>
                <div class="unit">bytes</div>
            </div>
            <div class="stat-card">
                <h3>Peak Memory</h3>
                <div class="value">{peak_memory}</div>
                <div class="unit">bytes</div>
            </div>
            <div class="stat-card">
                <h3>Total Allocations</h3>
                <div class="value">{total_allocations}</div>
                <div class="unit">allocations</div>
            </div>
            <div class="stat-card">
                <h3>Active Allocations</h3>
                <div class="value">{active_allocations}</div>
                <div class="unit">active</div>
            </div>
        </div>
        
        <div class="nav-grid">
            <div class="nav-card" onclick="window.open('/api/overview', '_blank')">
                <h3>📊 Memory Overview</h3>
                <p>Get a comprehensive overview of memory usage, top variables, and recent allocations</p>
            </div>
            <div class="nav-card" onclick="window.open('/api/variables', '_blank')">
                <h3>🔍 Variables Explorer</h3>
                <p>Browse all tracked variables with filtering, sorting, and detailed analysis</p>
            </div>
            <div class="nav-card" onclick="window.open('/api/timeline', '_blank')">
                <h3>⏱️ Memory Timeline</h3>
                <p>Explore memory allocation and deallocation events over time</p>
            </div>
            <div class="nav-card" onclick="window.open('/api/unsafe-ffi', '_blank')">
                <h3>⚠️ Unsafe/FFI Analysis</h3>
                <p>Analyze unsafe code blocks and FFI operations for potential issues</p>
            </div>
            <div class="nav-card" onclick="window.open('/api/performance', '_blank')">
                <h3>⚡ Performance Metrics</h3>
                <p>View performance statistics and optimization recommendations</p>
            </div>
            <div class="nav-card" onclick="showSearchDemo()">
                <h3>🔎 Search</h3>
                <p>Search variables, types, and other memory-related information</p>
            </div>
        </div>
        
        <div class="api-section">
            <h2>🚀 API Endpoints</h2>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/overview</span>
                <div class="description">Memory analysis overview with top variables and types</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/variables</span>
                <div class="description">List all variables with pagination and filtering</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/variables/:name</span>
                <div class="description">Detailed information about a specific variable</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/variables/:name/timeline</span>
                <div class="description">Timeline events for a specific variable</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/variables/:name/relationships</span>
                <div class="description">Related variables and dependencies</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/timeline</span>
                <div class="description">Memory timeline events with time range filtering</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/unsafe-ffi</span>
                <div class="description">Unsafe code and FFI analysis results</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/performance</span>
                <div class="description">Performance metrics and statistics</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/api/search?q=query</span>
                <div class="description">Search variables, types, and other data</div>
            </div>
            <div class="api-endpoint">
                <span class="method">GET</span>
                <span class="endpoint">/health</span>
                <div class="description">Server health check</div>
            </div>
        </div>
        
        <div class="footer">
            <p>MemScope-RS v{version} • Generated from {allocation_count} allocations • Server running on port {port}</p>
        </div>
    </div>
    
    <script>
        function showSearchDemo() {{
            const query = prompt('Enter search query:');
            if (query) {{
                window.open(`/api/search?q=${{encodeURIComponent(query)}}`, '_blank');
            }}
        }}
        
        // Format numbers with commas
        function formatNumber(num) {{
            return num.toString().replace(/\B(?=(\d{{3}})+(?!\d))/g, ",");
        }}
        
        // Format bytes to human readable
        function formatBytes(bytes) {{
            const units = ['B', 'KB', 'MB', 'GB'];
            let size = bytes;
            let unitIndex = 0;
            while (size >= 1024 && unitIndex < units.length - 1) {{
                size /= 1024;
                unitIndex++;
            }}
            return unitIndex === 0 ? `${{bytes}} ${{units[unitIndex]}}` : `${{size.toFixed(1)}} ${{units[unitIndex]}}`;
        }}
        
        // Update displayed values with formatting
        document.addEventListener('DOMContentLoaded', function() {{
            const activeMemoryEl = document.querySelector('.stat-card:nth-child(1) .value');
            const peakMemoryEl = document.querySelector('.stat-card:nth-child(2) .value');
            const totalAllocsEl = document.querySelector('.stat-card:nth-child(3) .value');
            const activeAllocsEl = document.querySelector('.stat-card:nth-child(4) .value');
            
            if (activeMemoryEl) activeMemoryEl.textContent = formatBytes({active_memory});
            if (peakMemoryEl) peakMemoryEl.textContent = formatBytes({peak_memory});
            if (totalAllocsEl) totalAllocsEl.textContent = formatNumber({total_allocations});
            if (activeAllocsEl) activeAllocsEl.textContent = formatNumber({active_allocations});
        }});
    </script>
</body>
</html>"#,
        active_memory = stats.active_memory,
        peak_memory = stats.peak_memory,
        total_allocations = stats.total_allocations,
        active_allocations = stats.active_allocations,
        allocation_count = state.memory_data.allocations.len(),
        version = env!("CARGO_PKG_VERSION"),
        port = state.config.port,
    );
    
    Html(html)
}