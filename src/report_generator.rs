use std::fs;
use std::path::Path;
use crate::{TrackingResult, TrackingError};

/// Generate a self-contained HTML report with embedded JSON data
pub fn generate_html_report<P: AsRef<Path>>(
    json_data_path: P,
    template_path: P,
    output_path: P,
) -> TrackingResult<()> {
    // Read the JSON data
    let json_content = fs::read_to_string(&json_data_path)?;
    
    // Read the HTML template
    let template_content = fs::read_to_string(&template_path)?;
    
    // Embed JSON data into the template
    let embedded_html = embed_json_data(&template_content, &json_content)?;
    
    // Write the self-contained HTML report
    fs::write(&output_path, embedded_html)?;
    
    println!("✅ Generated self-contained HTML report: {}", output_path.as_ref().display());
    Ok(())
}

/// Embed JSON data as inline script in HTML template
fn embed_json_data(template: &str, json_data: &str) -> TrackingResult<String> {
    // Find the placeholder for data injection
    let data_placeholder = "<!-- DATA_INJECTION_POINT -->";
    
    if !template.contains(data_placeholder) {
        return Err(TrackingError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Template must contain <!-- DATA_INJECTION_POINT --> placeholder"
        )));
    }
    
    // Create the inline script with JSON data
    let inline_script = format!(
        r#"<script type="text/javascript">
// Embedded JSON data for offline analysis
window.EMBEDDED_MEMORY_DATA = {};
console.log('📊 Loaded embedded memory analysis data');
</script>"#,
        json_data
    );
    
    // Replace placeholder with inline script
    let result = template.replace(data_placeholder, &inline_script);
    
    Ok(result)
}

/// Generate HTML report from tracker data
pub fn generate_report_from_tracker(
    tracker: &crate::MemoryTracker,
    unsafe_tracker: &crate::UnsafeFFITracker,
    output_path: &str,
) -> TrackingResult<()> {
    // Export JSON data to temporary file
    let temp_json = "temp_report_data.json";
    tracker.export_to_json(temp_json)?;
    
    // Use the built-in template
    let template_path = "report_template.html";
    
    // Generate the report
    generate_html_report(temp_json, template_path, output_path)?;
    
    // Clean up temporary file
    let _ = fs::remove_file(temp_json);
    
    Ok(())
}

/// Create a standalone HTML template optimized for embedded data
pub fn create_standalone_template(output_path: &str) -> TrackingResult<()> {
    let template_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Memory Analysis Report</title>
    <style>
        /* Embedded CSS for self-contained report */
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.95);
            border-radius: 15px;
            padding: 30px;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
            padding-bottom: 20px;
            border-bottom: 3px solid #667eea;
        }
        
        .header h1 {
            color: #2c3e50;
            font-size: 2.5em;
            margin: 0;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
        }
        
        .header .subtitle {
            color: #7f8c8d;
            font-size: 1.2em;
            margin-top: 10px;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }
        
        .metric-card {
            background: linear-gradient(135deg, #3498db, #2980b9);
            color: white;
            padding: 25px;
            border-radius: 12px;
            text-align: center;
            box-shadow: 0 8px 16px rgba(52, 152, 219, 0.3);
            transition: transform 0.3s ease;
        }
        
        .metric-card:hover {
            transform: translateY(-5px);
        }
        
        .metric-card .value {
            font-size: 2.5em;
            font-weight: bold;
            margin-bottom: 10px;
        }
        
        .metric-card .label {
            font-size: 1.1em;
            opacity: 0.9;
        }
        
        .visualization-section {
            margin-bottom: 40px;
            background: #f8f9fa;
            border-radius: 12px;
            padding: 25px;
            border-left: 5px solid #667eea;
        }
        
        .visualization-section h2 {
            color: #2c3e50;
            margin-top: 0;
            font-size: 1.8em;
        }
        
        .svg-container {
            width: 100%;
            overflow-x: auto;
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        }
        
        .no-data {
            text-align: center;
            color: #7f8c8d;
            font-style: italic;
            padding: 40px;
            background: #ecf0f1;
            border-radius: 8px;
        }
        
        .timestamp {
            text-align: center;
            color: #95a5a6;
            font-size: 0.9em;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #ecf0f1;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Memory Analysis Report</h1>
            <div class="subtitle">Comprehensive Rust Memory Usage Analysis</div>
        </div>
        
        <div class="metrics-grid" id="metricsGrid">
            <!-- Metrics will be populated by JavaScript -->
        </div>
        
        <div class="visualization-section">
            <h2>📊 Lifecycle Timeline Visualization</h2>
            <div class="svg-container" id="lifecycleVisualization">
                <!-- SVG content will be generated here -->
            </div>
        </div>
        
        <div class="visualization-section">
            <h2>🛡️ Unsafe/FFI Analysis Dashboard</h2>
            <div class="svg-container" id="unsafeFFIVisualization">
                <!-- SVG content will be generated here -->
            </div>
        </div>
        
        <div class="visualization-section">
            <h2>💾 Memory Analysis</h2>
            <div class="svg-container" id="memoryAnalysisVisualization">
                <!-- SVG content will be generated here -->
            </div>
        </div>
        
        <div class="timestamp" id="timestamp">
            <!-- Timestamp will be populated -->
        </div>
    </div>

    <!-- DATA_INJECTION_POINT -->

    <script type="text/javascript">
        // Initialize the report when page loads
        document.addEventListener('DOMContentLoaded', function() {
            if (window.EMBEDDED_MEMORY_DATA) {
                initializeReport(window.EMBEDDED_MEMORY_DATA);
            } else {
                console.error('No embedded data found');
                showNoDataMessage();
            }
        });
        
        function initializeReport(data) {
            console.log('🚀 Initializing memory analysis report...');
            
            // Populate metrics
            populateMetrics(data);
            
            // Generate visualizations
            generateLifecycleVisualization(data);
            generateUnsafeFFIVisualization(data);
            generateMemoryAnalysisVisualization(data);
            
            // Set timestamp
            setTimestamp(data);
            
            console.log('✅ Report initialization complete');
        }
        
        function populateMetrics(data) {
            const metricsGrid = document.getElementById('metricsGrid');
            const memoryStats = data.memory_stats || {};
            const unsafeStats = extractUnsafeStats(data);
            
            const metrics = [
                {
                    label: 'Total Memory',
                    value: formatBytes(memoryStats.total_allocated_bytes || 0),
                    color: '#3498db'
                },
                {
                    label: 'Active Allocations',
                    value: memoryStats.total_allocations || 0,
                    color: '#2ecc71'
                },
                {
                    label: 'Unsafe Operations',
                    value: unsafeStats.total_operations || 0,
                    color: '#e74c3c'
                },
                {
                    label: 'Memory Efficiency',
                    value: ((memoryStats.memory_efficiency || 0) * 100).toFixed(1) + '%',
                    color: '#f39c12'
                }
            ];
            
            metricsGrid.innerHTML = metrics.map(metric => `
                <div class="metric-card" style="background: linear-gradient(135deg, ${metric.color}, ${adjustColor(metric.color, -20)});">
                    <div class="value">${metric.value}</div>
                    <div class="label">${metric.label}</div>
                </div>
            `).join('');
        }
        
        function generateLifecycleVisualization(data) {
            const container = document.getElementById('lifecycleVisualization');
            const svgContent = createLifecycleTimelineSVG(data);
            container.innerHTML = svgContent;
        }
        
        function generateUnsafeFFIVisualization(data) {
            const container = document.getElementById('unsafeFFIVisualization');
            const svgContent = createUnsafeFFIDashboardSVG(data);
            container.innerHTML = svgContent;
        }
        
        function generateMemoryAnalysisVisualization(data) {
            const container = document.getElementById('memoryAnalysisVisualization');
            const svgContent = createMemoryAnalysisSVG(data);
            container.innerHTML = svgContent;
        }
        
        function setTimestamp(data) {
            const timestampEl = document.getElementById('timestamp');
            const timestamp = data.timestamp || new Date().toISOString();
            timestampEl.textContent = `Report generated: ${new Date(timestamp).toLocaleString()}`;
        }
        
        function showNoDataMessage() {
            document.querySelector('.container').innerHTML = `
                <div class="header">
                    <h1>❌ No Data Available</h1>
                    <div class="subtitle">Unable to load embedded memory analysis data</div>
                </div>
                <div class="no-data">
                    <p>This report requires embedded JSON data to function.</p>
                    <p>Please regenerate the report with valid memory tracking data.</p>
                </div>
            `;
        }
        
        // Utility functions
        function formatBytes(bytes) {
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }
        
        function adjustColor(color, amount) {
            const usePound = color[0] === '#';
            const col = usePound ? color.slice(1) : color;
            const num = parseInt(col, 16);
            let r = (num >> 16) + amount;
            let g = (num >> 8 & 0x00FF) + amount;
            let b = (num & 0x0000FF) + amount;
            r = r > 255 ? 255 : r < 0 ? 0 : r;
            g = g > 255 ? 255 : g < 0 ? 0 : g;
            b = b > 255 ? 255 : b < 0 ? 0 : b;
            return (usePound ? '#' : '') + (r << 16 | g << 8 | b).toString(16).padStart(6, '0');
        }
        
        // Include all the SVG rendering functions from dashboard.js
        // (These would be copied from the existing dashboard.js file)
        
        // Placeholder for SVG rendering functions
        function createLifecycleTimelineSVG(data) {
            return '<div class="no-data">Lifecycle visualization will be rendered here</div>';
        }
        
        function createUnsafeFFIDashboardSVG(data) {
            return '<div class="no-data">Unsafe/FFI analysis will be rendered here</div>';
        }
        
        function createMemoryAnalysisSVG(data) {
            return '<div class="no-data">Memory analysis will be rendered here</div>';
        }
        
        function extractUnsafeStats(data) {
            if (data.unsafe_ffi_stats) {
                return {
                    total_operations: data.unsafe_ffi_stats.total_operations || 0,
                    ffi_calls: data.unsafe_ffi_stats.ffi_calls || 0,
                    memory_violations: data.unsafe_ffi_stats.memory_violations || 0,
                    risk_score: data.unsafe_ffi_stats.risk_score || 0
                };
            }
            return { total_operations: 0, ffi_calls: 0, memory_violations: 0, risk_score: 0 };
        }
    </script>
</body>
</html>"#;

    fs::write(output_path, template_content)?;
    
    println!("✅ Created standalone HTML template: {}", output_path);
    Ok(())
}