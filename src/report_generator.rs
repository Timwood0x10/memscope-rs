//! HTML report generation from JSON data

use crate::types::{ComprehensiveReport, TrackingError, TrackingResult};
use std::fs;
use std::path::Path;

/// Generate interactive HTML report from comprehensive JSON data
pub fn generate_interactive_html_report<P: AsRef<Path>>(
    json_data_path: P,
    template_path: P,
    output_path: P,
) -> TrackingResult<()> {
    // Read JSON data
    let json_content = fs::read_to_string(&json_data_path)?;
    let comprehensive_data: ComprehensiveReport = serde_json::from_str(&json_content)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to parse JSON: {e}")))?;

    generate_html_from_data(&comprehensive_data, template_path, output_path)
}

/// Generate HTML report directly from ComprehensiveReport data
pub fn generate_html_from_data<P: AsRef<Path>>(
    data: &ComprehensiveReport,
    template_path: P,
    output_path: P,
) -> TrackingResult<()> {
    // Read template
    let template = fs::read_to_string(&template_path)?;
    
    // Convert data to JSON string for embedding
    let json_string = serde_json::to_string(data)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to serialize data: {e}")))?;
    
    // Create embedded script with comprehensive data
    let embedded_script = format!(
        r#"<script>
window.EMBEDDED_MEMORY_DATA = {};
window.COMPREHENSIVE_DATA = {};
console.log('Comprehensive memory data loaded:', window.COMPREHENSIVE_DATA);
</script>"#, 
        json_string, json_string
    );
    
    // Replace template placeholder with embedded data
    let final_html = template.replace(
        "<!-- DATA_INJECTION_POINT -->", 
        &embedded_script
    );
    
    // Add enhanced JavaScript for comprehensive data handling
    let enhanced_html = add_comprehensive_javascript(&final_html, data)?;
    
    // Write final HTML
    fs::write(&output_path, enhanced_html)?;
    
    println!("✅ Interactive HTML report generated: {}", output_path.as_ref().display());
    Ok(())
}

/// Add comprehensive JavaScript functionality to handle all data types
fn add_comprehensive_javascript(html: &str, data: &ComprehensiveReport) -> TrackingResult<String> {
    let enhanced_js = format!(r#"
    <script>
        // Enhanced data processing for comprehensive report
        function initializeComprehensiveAnalyzer() {{
            console.log('Initializing comprehensive analyzer...');
            
            if (window.COMPREHENSIVE_DATA) {{
                const data = window.COMPREHENSIVE_DATA;
                
                // Initialize all views with comprehensive data
                initializeScopeAnalysis(data.scope_analysis);
                initializeVariableTracking(data.variable_tracking);
                initializeTypeAnalysis(data.type_analysis);
                initializePerformanceMetrics(data.performance_metrics);
                initializeTimelineData(data.timeline_data);
                initializeSafetyAnalysis(data.safety_analysis);
                
                // Update quick stats with comprehensive metrics
                updateComprehensiveQuickStats(data);
                
                console.log('Comprehensive analyzer initialized successfully');
            }} else {{
                console.warn('No comprehensive data available, falling back to basic mode');
            }}
        }}
        
        function initializeScopeAnalysis(scopeData) {{
            if (!scopeData || !scopeData.scopes) return;
            
            console.log('Initializing scope analysis with', scopeData.scopes.length, 'scopes');
            
            // Create scope hierarchy visualization
            const scopeContainer = document.getElementById('scopeHierarchy');
            if (scopeContainer) {{
                scopeContainer.innerHTML = createScopeHierarchyHTML(scopeData);
            }}
            
            // Add scope metrics to overview
            addScopeMetricsToOverview(scopeData);
        }}
        
        function createScopeHierarchyHTML(scopeData) {{
            let html = '<div class="scope-hierarchy">';
            
            // Group scopes by depth
            const scopesByDepth = {{}};
            scopeData.scopes.forEach(scope => {{
                if (!scopesByDepth[scope.depth]) {{
                    scopesByDepth[scope.depth] = [];
                }}
                scopesByDepth[scope.depth].push(scope);
            }});
            
            // Render each depth level
            Object.keys(scopesByDepth).sort((a, b) => parseInt(a) - parseInt(b)).forEach(depth => {{
                html += `<div class="scope-level" data-depth="${{depth}}">`;
                html += `<h4>Depth ${{depth}}</h4>`;
                
                scopesByDepth[depth].forEach(scope => {{
                    const duration = scope.end_time ? scope.end_time - scope.start_time : 'Active';
                    html += `
                        <div class="scope-card" data-scope="${{scope.name}}">
                            <div class="scope-name">${{scope.name}}</div>
                            <div class="scope-stats">
                                <span>Memory: ${{formatBytes(scope.memory_usage)}}</span>
                                <span>Variables: ${{scope.variables.length}}</span>
                                <span>Duration: ${{duration}}ms</span>
                            </div>
                        </div>
                    `;
                }});
                
                html += '</div>';
            }});
            
            html += '</div>';
            return html;
        }}
        
        function initializeVariableTracking(variableData) {{
            if (!variableData || !variableData.variables) return;
            
            console.log('Initializing variable tracking with', variableData.variables.length, 'variables');
            
            // Create variable lifecycle timeline
            const timelineContainer = document.getElementById('variableLifecycleTimeline');
            if (timelineContainer) {{
                timelineContainer.innerHTML = createVariableTimelineHTML(variableData.variables);
            }}
            
            // Update allocations table with lifecycle data
            updateAllocationsTableWithLifecycle(variableData.variables);
        }}
        
        function createVariableTimelineHTML(variables) {{
            let html = '<div class="variable-timeline">';
            
            // Sort variables by birth time
            const sortedVars = [...variables].sort((a, b) => a.birth_time - b.birth_time);
            
            // Calculate timeline scale
            const minTime = Math.min(...sortedVars.map(v => v.birth_time));
            const maxTime = Math.max(...sortedVars.map(v => v.death_time || Date.now()));
            const timeRange = maxTime - minTime;
            
            sortedVars.forEach((variable, index) => {{
                const startPercent = ((variable.birth_time - minTime) / timeRange) * 100;
                const endPercent = variable.death_time ? 
                    ((variable.death_time - minTime) / timeRange) * 100 : 100;
                const width = endPercent - startPercent;
                
                html += `
                    <div class="timeline-item" style="top: ${{index * 25}}px;">
                        <div class="timeline-bar" 
                             style="left: ${{startPercent}}%; width: ${{width}}%; background: ${{getTypeColor(variable.type_name)}};"
                             title="${{variable.name}} (${{variable.type_name}}) - ${{formatBytes(variable.current_memory)}}">
                            <span class="timeline-label">${{variable.name}}</span>
                        </div>
                    </div>
                `;
            }});
            
            html += '</div>';
            return html;
        }}
        
        function initializeTypeAnalysis(typeData) {{
            if (!typeData) return;
            
            console.log('Initializing type analysis');
            
            // Create type hierarchy visualization
            const typeContainer = document.getElementById('typeHierarchyViz');
            if (typeContainer) {{
                typeContainer.innerHTML = createTypeHierarchyHTML(typeData);
            }}
        }}
        
        function createTypeHierarchyHTML(typeData) {{
            let html = '<div class="type-hierarchy-viz">';
            
            // Display type categories
            if (typeData.type_hierarchy && typeData.type_hierarchy.categories) {{
                Object.entries(typeData.type_hierarchy.categories).forEach(([categoryName, category]) => {{
                    html += `
                        <div class="type-category">
                            <h4>${{categoryName}}</h4>
                            <div class="category-stats">
                                <span>Memory: ${{formatBytes(category.total_memory)}}</span>
                                <span>Allocations: ${{category.allocation_count}}</span>
                            </div>
                        </div>
                    `;
                }});
            }}
            
            html += '</div>';
            return html;
        }}
        
        function initializePerformanceMetrics(perfData) {{
            if (!perfData) return;
            
            console.log('Initializing performance metrics');
            
            // Add performance charts
            const perfContainer = document.getElementById('performanceCharts');
            if (perfContainer) {{
                perfContainer.innerHTML = createPerformanceChartsHTML(perfData);
            }}
        }}
        
        function createPerformanceChartsHTML(perfData) {{
            return `
                <div class="performance-metrics">
                    <div class="perf-metric">
                        <h4>Allocation Rate</h4>
                        <div class="metric-value">${{perfData.allocation_rate.toFixed(2)}} allocs/sec</div>
                    </div>
                    <div class="perf-metric">
                        <h4>Memory Throughput</h4>
                        <div class="metric-value">${{perfData.memory_throughput_mb_s.toFixed(2)}} MB/s</div>
                    </div>
                    <div class="perf-metric">
                        <h4>Average Allocation Size</h4>
                        <div class="metric-value">${{formatBytes(perfData.average_allocation_size)}}</div>
                    </div>
                    <div class="perf-metric">
                        <h4>P95 Allocation Size</h4>
                        <div class="metric-value">${{formatBytes(perfData.p95_allocation_size)}}</div>
                    </div>
                </div>
            `;
        }}
        
        function initializeTimelineData(timelineData) {{
            if (!timelineData) return;
            
            console.log('Initializing timeline data');
            
            // Create memory usage timeline chart
            const chartContainer = document.getElementById('memoryTimelineChart');
            if (chartContainer) {{
                createMemoryTimelineChart(chartContainer, timelineData);
            }}
        }}
        
        function createMemoryTimelineChart(container, timelineData) {{
            // Simple timeline visualization
            let html = '<div class="memory-timeline-chart">';
            
            if (timelineData.allocation_events && timelineData.allocation_events.length > 0) {{
                const events = timelineData.allocation_events;
                const timeRange = timelineData.time_range;
                
                // Group events by time buckets for visualization
                const buckets = 50;
                const bucketSize = timeRange.duration_ms / buckets;
                const bucketData = new Array(buckets).fill(0);
                
                events.forEach(event => {{
                    const bucketIndex = Math.floor((event.timestamp - timeRange.start_time) / bucketSize);
                    if (bucketIndex >= 0 && bucketIndex < buckets) {{
                        bucketData[bucketIndex] += event.size;
                    }}
                }});
                
                const maxBucketValue = Math.max(...bucketData);
                
                html += '<div class="timeline-bars">';
                bucketData.forEach((value, index) => {{
                    const height = (value / maxBucketValue) * 100;
                    html += `<div class="timeline-bar" style="height: ${{height}}%" title="Time bucket ${{index}}: ${{formatBytes(value)}}"></div>`;
                }});
                html += '</div>';
            }}
            
            html += '</div>';
            container.innerHTML = html;
        }}
        
        function initializeSafetyAnalysis(safetyData) {{
            if (!safetyData) return;
            
            console.log('Initializing safety analysis');
            
            // Update safety metrics in overview
            const safetyContainer = document.getElementById('safetyMetrics');
            if (safetyContainer) {{
                safetyContainer.innerHTML = `
                    <div class="safety-metrics">
                        <div class="safety-metric">
                            <span class="metric-label">Unsafe Operations</span>
                            <span class="metric-value">${{safetyData.unsafe_operations}}</span>
                        </div>
                        <div class="safety-metric">
                            <span class="metric-label">FFI Calls</span>
                            <span class="metric-value">${{safetyData.ffi_calls}}</span>
                        </div>
                        <div class="safety-metric">
                            <span class="metric-label">Risk Score</span>
                            <span class="metric-value">${{safetyData.risk_score.toFixed(2)}}</span>
                        </div>
                    </div>
                `;
            }}
        }}
        
        function updateComprehensiveQuickStats(data) {{
            const quickStats = document.getElementById('quickStats');
            if (!quickStats) return;
            
            quickStats.innerHTML = `
                <div class="quick-stat">
                    <div class="stat-value">${{data.memory_overview.active_allocations}}</div>
                    <div class="stat-label">Active Allocations</div>
                </div>
                <div class="quick-stat">
                    <div class="stat-value">${{data.scope_analysis.scopes.length}}</div>
                    <div class="stat-label">Scopes Tracked</div>
                </div>
                <div class="quick-stat">
                    <div class="stat-value">${{data.variable_tracking.variables.length}}</div>
                    <div class="stat-label">Variables</div>
                </div>
                <div class="quick-stat">
                    <div class="stat-value">${{formatBytes(data.memory_overview.peak_memory)}}</div>
                    <div class="stat-label">Peak Memory</div>
                </div>
            `;
        }}
        
        // Enhanced utility functions
        function formatBytes(bytes) {{
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }}
        
        function getTypeColor(typeName) {{
            const colors = {{
                'String': '#2ecc71',
                'Vec': '#e74c3c', 
                'HashMap': '#f39c12',
                'Box': '#9b59b6',
                'i32': '#3498db',
                'u64': '#1abc9c'
            }};
            
            for (const [type, color] of Object.entries(colors)) {{
                if (typeName && typeName.includes(type)) {{
                    return color;
                }}
            }}
            return '#95a5a6';
        }}
        
        // Override the original initialization to use comprehensive data
        document.addEventListener('DOMContentLoaded', function() {{
            if (window.COMPREHENSIVE_DATA) {{
                initializeComprehensiveAnalyzer();
            }} else if (window.EMBEDDED_MEMORY_DATA) {{
                // Fallback to original initialization
                if (typeof initializeAnalyzer === 'function') {{
                    initializeAnalyzer();
                }}
            }} else {{
                console.error('No memory data found');
            }}
        }});
    </script>
    
    <style>
        /* Enhanced styles for comprehensive data visualization */
        .scope-hierarchy {{
            display: flex;
            flex-direction: column;
            gap: 20px;
        }}
        
        .scope-level {{
            border-left: 3px solid #3498db;
            padding-left: 15px;
        }}
        
        .scope-card {{
            background: white;
            border-radius: 8px;
            padding: 15px;
            margin: 10px 0;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            cursor: pointer;
            transition: all 0.3s ease;
        }}
        
        .scope-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
        }}
        
        .scope-name {{
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 8px;
        }}
        
        .scope-stats {{
            display: flex;
            gap: 15px;
            font-size: 12px;
            color: #7f8c8d;
        }}
        
        .variable-timeline {{
            position: relative;
            height: 400px;
            background: #f8f9fa;
            border-radius: 8px;
            overflow-y: auto;
            padding: 10px;
        }}
        
        .timeline-item {{
            position: absolute;
            left: 0;
            right: 0;
            height: 20px;
        }}
        
        .timeline-bar {{
            position: absolute;
            height: 18px;
            border-radius: 9px;
            display: flex;
            align-items: center;
            padding: 0 8px;
            color: white;
            font-size: 11px;
            font-weight: bold;
            cursor: pointer;
            transition: all 0.2s ease;
        }}
        
        .timeline-bar:hover {{
            transform: scaleY(1.2);
            z-index: 10;
        }}
        
        .timeline-label {{
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
        
        .type-hierarchy-viz {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        
        .type-category {{
            background: white;
            border-radius: 8px;
            padding: 15px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        .performance-metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        
        .perf-metric {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        .perf-metric h4 {{
            margin: 0 0 10px 0;
            color: #2c3e50;
            font-size: 14px;
        }}
        
        .metric-value {{
            font-size: 18px;
            font-weight: bold;
            color: #3498db;
        }}
        
        .memory-timeline-chart {{
            background: white;
            border-radius: 8px;
            padding: 20px;
            height: 200px;
        }}
        
        .timeline-bars {{
            display: flex;
            align-items: end;
            height: 150px;
            gap: 2px;
        }}
        
        .timeline-bar {{
            flex: 1;
            background: linear-gradient(to top, #3498db, #2980b9);
            border-radius: 2px 2px 0 0;
            cursor: pointer;
            transition: all 0.2s ease;
        }}
        
        .timeline-bar:hover {{
            background: linear-gradient(to top, #e74c3c, #c0392b);
        }}
        
        .safety-metrics {{
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }}
        
        .safety-metric {{
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 15px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        .quick-stat {{
            background: rgba(255,255,255,0.1);
            border-radius: 8px;
            padding: 15px;
            margin: 10px 0;
            text-align: center;
        }}
        
        .stat-value {{
            font-size: 18px;
            font-weight: bold;
            color: #3498db;
        }}
        
        .stat-label {{
            font-size: 12px;
            color: #bdc3c7;
            margin-top: 5px;
        }}
    </style>
    "#);

    // Insert enhanced JavaScript before closing body tag
    let enhanced_html = html.replace("</body>", &format!("{}\n</body>", enhanced_js));
    
    Ok(enhanced_html)
}

/// Generate a simple HTML report from basic memory data (fallback)
pub fn generate_simple_html_report<P: AsRef<Path>>(
    json_data: &serde_json::Value,
    output_path: P,
) -> TrackingResult<()> {
    let simple_template = include_str!("../template.html");
    
    let json_string = serde_json::to_string(json_data)
        .map_err(|e| TrackingError::SerializationError(format!("Failed to serialize data: {e}")))?;
    
    let embedded_script = format!(
        "<script>window.EMBEDDED_MEMORY_DATA = {};</script>", 
        json_string
    );
    
    let final_html = simple_template.replace(
        "<!-- DATA_INJECTION_POINT -->", 
        &embedded_script
    );
    
    fs::write(&output_path, final_html)?;
    
    println!("✅ Simple HTML report generated: {}", output_path.as_ref().display());
    Ok(())
}