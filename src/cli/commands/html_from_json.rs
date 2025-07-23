//! HTML generation from JSON files command
//!
//! This module provides functionality to generate interactive HTML reports
//! from exported JSON data files, significantly faster than direct tracker export.

use clap::ArgMatches;
use std::error::Error;
use std::fs;
use std::path::Path;
use serde_json::Value;
use std::collections::HashMap;

/// Run the HTML from JSON generation command
pub fn run_html_from_json(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_dir = matches
        .get_one::<String>("input-dir")
        .ok_or("Input directory is required")?;
    let output_file = matches
        .get_one::<String>("output")
        .ok_or("Output HTML file is required")?;
    let base_name = matches
        .get_one::<String>("base-name")
        .map(|s| s.as_str())
        .unwrap_or("snapshot");

    println!("🚀 Generating HTML report from JSON files...");
    println!("📁 Input directory: {}", input_dir);
    println!("📄 Output file: {}", output_file);
    println!("🏷️  Base name: {}", base_name);

    // 🎯 快速加载所有JSON文件
    let json_data = load_json_files(input_dir, base_name)?;
    
    // 🎨 生成HTML报告
    generate_html_from_json_data(&json_data, output_file)?;
    
    println!("✅ HTML report generated successfully!");
    println!("🌐 Open {} in your browser to view the interactive report", output_file);
    
    Ok(())
}

/// Load all JSON files for a given base name
fn load_json_files(input_dir: &str, base_name: &str) -> Result<JsonDataCollection, Box<dyn Error>> {
    let mut data = JsonDataCollection::new();
    
    // 定义要加载的JSON文件类型
    let file_types = [
        ("memory_analysis", "Memory Analysis"),
        ("lifetime", "Lifecycle Analysis"), 
        ("unsafe_ffi", "Unsafe/FFI Analysis"),
        ("performance", "Performance Metrics"),
        ("complex_types", "Complex Types Analysis"),
        ("security_violations", "Security Violations"),
        ("variable_relationships", "Variable Relationships"),
    ];
    
    for (suffix, description) in &file_types {
        let file_path = format!("{}/{}_{}.json", input_dir, base_name, suffix);
        
        if Path::new(&file_path).exists() {
            println!("📊 Loading {}: {}", description, file_path);
            let content = fs::read_to_string(&file_path)?;
            let json_value: Value = serde_json::from_str(&content)?;
            data.insert(suffix.to_string(), json_value);
        } else {
            println!("⚠️  Optional file not found: {} (skipping)", file_path);
        }
    }
    
    if data.is_empty() {
        return Err("No JSON files found! Please check the input directory and base name.".into());
    }
    
    println!("✅ Loaded {} JSON data files", data.len());
    Ok(data)
}

/// Collection of JSON data from different analysis files
type JsonDataCollection = HashMap<String, Value>;

/// Generate HTML report from loaded JSON data
fn generate_html_from_json_data(
    json_data: &JsonDataCollection, 
    output_file: &str
) -> Result<(), Box<dyn Error>> {
    
    // 🎯 读取模板文件
    let css_content = include_str!("../../../templates/styles.css");
    let js_content = include_str!("../../../templates/script.js");
    
    // 🎨 构建HTML内容
    let html_content = build_html_template(css_content, js_content, json_data)?;
    
    // 💾 写入文件
    fs::write(output_file, html_content)?;
    
    Ok(())
}

/// Build complete HTML template with embedded data
fn build_html_template(
    css_content: &str,
    js_content: &str, 
    json_data: &JsonDataCollection
) -> Result<String, Box<dyn Error>> {
    
    // 🎯 准备数据摘要用于header统计
    let stats_summary = extract_stats_summary(json_data);
    
    // 🎨 构建完整的HTML
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope-RS Interactive Memory Analysis</title>
    <style>
        {css_content}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1>🔍 MemScope-RS Interactive Memory Analysis</h1>
            <div class="header-stats">
                <span class="stat-badge" id="totalMemory">{total_memory}</span>
                <span class="stat-badge" id="activeAllocs">{active_allocs} Active</span>
                <span class="stat-badge" id="peakMemory">{peak_memory}</span>
            </div>
        </header>

        <nav class="tab-nav">
            <button class="tab-btn active" data-tab="overview">📊 Overview</button>
            <button class="tab-btn" data-tab="memory-analysis">🧠 Memory Analysis</button>
            <button class="tab-btn" data-tab="lifecycle">⏱️ Lifecycle Timeline</button>
            <button class="tab-btn" data-tab="unsafe-ffi">⚠️ Unsafe/FFI</button>
            <button class="tab-btn" data-tab="performance">⚡ Performance</button>
            <button class="tab-btn" data-tab="security">🔒 Security</button>
            <button class="tab-btn" data-tab="variables">🔗 Variable Relationships</button>
            <button class="tab-btn" data-tab="interactive">🎮 Interactive Explorer</button>
        </nav>

        <main class="content">
            <!-- Overview Tab -->
            <div class="tab-content active" id="overview">
                <div class="overview-grid">
                    <div class="overview-card">
                        <h3>📈 Memory Statistics</h3>
                        <div id="memoryStats">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>🏷️ Type Distribution</h3>
                        <div id="typeDistribution">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>📋 Recent Allocations</h3>
                        <div id="recentAllocations">Loading...</div>
                    </div>
                    <div class="overview-card">
                        <h3>⚡ Performance Insights</h3>
                        <div id="performanceInsights">Loading...</div>
                    </div>
                </div>
            </div>

            <!-- Memory Analysis Tab -->
            <div class="tab-content" id="memory-analysis">
                <div id="memoryAnalysisContent">Loading memory analysis...</div>
            </div>

            <!-- Lifecycle Timeline Tab -->
            <div class="tab-content" id="lifecycle">
                <div id="lifecycleContent">Loading lifecycle analysis...</div>
            </div>

            <!-- Unsafe/FFI Tab -->
            <div class="tab-content" id="unsafe-ffi">
                <div id="unsafeFfiContent">Loading unsafe/FFI analysis...</div>
            </div>

            <!-- Performance Tab -->
            <div class="tab-content" id="performance">
                <div id="performanceContent">Loading performance analysis...</div>
            </div>

            <!-- Security Tab -->
            <div class="tab-content" id="security">
                <div id="securityContent">Loading security analysis...</div>
            </div>

            <!-- Variable Relationships Tab -->
            <div class="tab-content" id="variables">
                <div id="variableContent">Loading variable relationships...</div>
            </div>

            <!-- Interactive Explorer Tab -->
            <div class="tab-content" id="interactive">
                <div class="explorer-controls">
                    <div class="control-group">
                        <label for="filterType">Filter by Type:</label>
                        <select id="filterType">
                            <option value="">All Types</option>
                        </select>
                    </div>
                    <div class="control-group">
                        <label for="sizeRange">Size Range:</label>
                        <input type="range" id="sizeRange" min="0" max="100" value="100">
                        <span id="sizeRangeValue">All sizes</span>
                    </div>
                    <div class="control-group">
                        <label for="sortBy">Sort by:</label>
                        <select id="sortBy">
                            <option value="size">Size</option>
                            <option value="timestamp">Timestamp</option>
                            <option value="type">Type</option>
                        </select>
                    </div>
                </div>
                <div class="explorer-content">
                    <div class="allocation-grid" id="allocationGrid">
                        Loading allocations...
                    </div>
                </div>
            </div>
        </main>
    </div>

    <script>
        // 🎯 嵌入的JSON数据 - 从多个文件合并
        const MEMORY_DATA = {json_data_str};
        
        // 🚀 增强的JavaScript功能
        {js_content}
        
        // 🎨 初始化多数据源支持
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('🎯 Initializing multi-source memory analysis...');
            console.log('📊 Available data sources:', Object.keys(MEMORY_DATA));
            
            // 转换多数据源为兼容的单一数据结构
            const compatibleData = convertMultiSourceData(MEMORY_DATA);
            
            // 初始化可视化器
            if (typeof MemScopeVisualizer !== 'undefined') {{
                window.memscope = new MemScopeVisualizer(compatibleData);
                console.log('✅ MemScope visualizer initialized with multi-source data');
            }} else {{
                console.warn('⚠️ MemScopeVisualizer not found, falling back to basic initialization');
                initializeBasicView(compatibleData);
            }}
        }});
        
        // 转换多数据源为兼容格式
        function convertMultiSourceData(multiData) {{
            const memoryData = multiData.memory_analysis || {{}};
            const lifetimeData = multiData.lifetime || {{}};
            const performanceData = multiData.performance || {{}};
            const unsafeFfiData = multiData.unsafe_ffi || {{}};
            const securityData = multiData.security_violations || {{}};
            const complexTypesData = multiData.complex_types || {{}};
            const variableRelationshipsData = multiData.variable_relationships || {{}};
            
            // 从memory_analysis的memory_stats提取统计信息
            const memoryStats = memoryData.memory_stats || {{}};
            const metadata = memoryData.metadata || {{}};
            
            // 从performance数据提取性能指标
            const perfMemory = performanceData.memory_performance || {{}};
            const perfAllocation = performanceData.allocation_distribution || {{}};
            const perfExport = performanceData.export_performance || {{}};
            
            // 计算活跃分配数量（从allocations数组长度推算）
            const activeAllocations = memoryData.allocations ? memoryData.allocations.length : 0;
            
            return {{
                // 从memory_stats和performance综合提取统计信息
                stats: {{
                    active_memory: memoryStats.active_memory || perfMemory.active_memory || 0,
                    active_allocations: activeAllocations || metadata.total_allocations || 0,
                    peak_memory: memoryStats.peak_memory || perfMemory.peak_memory || 0,
                    total_allocations: memoryStats.total_allocations || perfMemory.total_allocated || metadata.total_allocations || 0,
                    total_allocated: memoryStats.total_allocated || perfMemory.total_allocated || 0
                }},
                
                // 从memory_analysis提取分配信息
                allocations: memoryData.allocations || [],
                
                // 从performance提取分配分布
                allocation_distribution: perfAllocation || {{}},
                
                // 性能指标
                performance: {{
                    processing_time_ms: perfExport?.total_processing_time_ms || 0,
                    allocations_per_second: perfExport?.processing_rate?.allocations_per_second || 0,
                    memory_efficiency: perfMemory?.memory_efficiency || 0,
                    optimization_status: performanceData.optimization_status || {{}}
                }},
                
                // 生命周期数据
                lifecycle: lifetimeData,
                
                // Unsafe/FFI数据
                unsafeFFI: unsafeFfiData,
                
                // 安全分析数据
                security: {{
                    total_violations: securityData.security_summary?.security_analysis_summary?.total_violations || 0,
                    risk_level: securityData.security_summary?.security_analysis_summary?.risk_assessment?.risk_level || 'Unknown',
                    severity_breakdown: securityData.security_summary?.security_analysis_summary?.severity_breakdown || {{}},
                    violation_reports: securityData.violation_reports || [],
                    recommendations: securityData.analysis_recommendations || []
                }},
                
                // 复杂类型数据
                complex_types: complexTypesData,
                
                // 变量关系数据
                variable_relationships: {{
                    relationships: variableRelationshipsData.variable_relationships || [],
                    registry: variableRelationshipsData.variable_registry || {{}}
                }},
                
                // 元数据
                metadata: {{
                    timestamp: metadata.timestamp || Math.floor(Date.now() / 1000),
                    export_version: metadata.export_version || '2.0',
                    analysis_type: metadata.analysis_type || 'integrated_analysis'
                }},
                
                // 原始多数据源（用于高级功能）
                _multiSource: multiData
            }};
        }}
        
        // 基础视图初始化（当MemScopeVisualizer不可用时）
        function initializeBasicView(data) {{
            console.log('🎯 Initializing basic view with data:', data);
            
            // 基础格式化函数
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
            
            // 更新header统计
            const totalMemoryEl = document.getElementById('totalMemory');
            const activeAllocsEl = document.getElementById('activeAllocs');
            const peakMemoryEl = document.getElementById('peakMemory');
            
            if (totalMemoryEl) totalMemoryEl.textContent = formatBytes(data.stats.active_memory);
            if (activeAllocsEl) activeAllocsEl.textContent = data.stats.active_allocations + ' Active';
            if (peakMemoryEl) peakMemoryEl.textContent = formatBytes(data.stats.peak_memory);
            
            // 填充Overview内容
            const memoryStatsEl = document.getElementById('memoryStats');
            if (memoryStatsEl) {{
                memoryStatsEl.innerHTML = `
                    <div class="stats-grid">
                        <div class="stat-item">
                            <span class="stat-label">Active Memory:</span>
                            <span class="stat-value">${{formatBytes(data.stats.active_memory)}}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Peak Memory:</span>
                            <span class="stat-value">${{formatBytes(data.stats.peak_memory)}}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Total Allocations:</span>
                            <span class="stat-value">${{data.stats.total_allocations.toLocaleString()}}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Active Allocations:</span>
                            <span class="stat-value">${{data.stats.active_allocations.toLocaleString()}}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Total Allocated:</span>
                            <span class="stat-value">${{formatBytes(data.stats.total_allocated)}}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Memory Efficiency:</span>
                            <span class="stat-value">${{data.performance.memory_efficiency}}%</span>
                        </div>
                    </div>
                `;
            }}
            
            // 初始化Performance Analysis标签页
            initializePerformanceAnalysis(data);
            
            // 初始化Security Analysis标签页
            initializeSecurityAnalysis(data);
            
            // 初始化Memory Analysis详细信息
            initializeMemoryAnalysisDetails(data);
            
            // 初始化Variable Relationships
            initializeVariableRelationships(data);
            
            // 初始化Lifecycle Analysis
            initializeLifecycleAnalysis(data);
            
            console.log('✅ Basic view initialized with all data');
        }}
        
        // 初始化Performance Analysis
        function initializePerformanceAnalysis(data) {{
            const perfContent = document.getElementById('performanceContent');
            if (perfContent && data.performance) {{
                const perfData = data.performance;
                const allocationDist = perfData.allocation_distribution || {{}};
                const memoryPerf = perfData.memory_performance || {{}};
                const exportPerf = perfData.export_performance || {{}};
                const optimizationStatus = perfData.optimization_status || {{}};
                
                perfContent.innerHTML = `
                    <div class="performance-overview">
                        <h3>📊 Performance Metrics</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Processing Time:</span>
                                <span class="stat-value">${{exportPerf.total_processing_time_ms || 0}}ms</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Allocations/Second:</span>
                                <span class="stat-value">${{exportPerf.processing_rate?.allocations_per_second?.toFixed(1) || 'N/A'}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Memory Efficiency:</span>
                                <span class="stat-value">${{memoryPerf.memory_efficiency || 0}}%</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Allocations Processed:</span>
                                <span class="stat-value">${{exportPerf.allocations_processed || 0}}</span>
                            </div>
                        </div>
                        
                        <h3>💾 Memory Performance</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Active Memory:</span>
                                <span class="stat-value">${{formatBytes(memoryPerf.active_memory || 0)}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Peak Memory:</span>
                                <span class="stat-value">${{formatBytes(memoryPerf.peak_memory || 0)}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Total Allocated:</span>
                                <span class="stat-value">${{formatBytes(memoryPerf.total_allocated || 0)}}</span>
                            </div>
                        </div>
                        
                        <h3>📈 Allocation Distribution</h3>
                        <div class="allocation-distribution">
                            <div class="dist-item">Tiny: ${{allocationDist.tiny || 0}}</div>
                            <div class="dist-item">Small: ${{allocationDist.small || 0}}</div>
                            <div class="dist-item">Medium: ${{allocationDist.medium || 0}}</div>
                            <div class="dist-item">Large: ${{allocationDist.large || 0}}</div>
                            <div class="dist-item">Massive: ${{allocationDist.massive || 0}}</div>
                        </div>
                        
                        <h3>⚙️ Optimization Status</h3>
                        <div class="optimization-status">
                            <div>Parallel Processing: ${{optimizationStatus.parallel_processing ? '✅' : '❌'}}</div>
                            <div>Schema Validation: ${{optimizationStatus.schema_validation ? '✅' : '❌'}}</div>
                            <div>Streaming Enabled: ${{optimizationStatus.streaming_enabled ? '✅' : '❌'}}</div>
                            <div>Batch Size: ${{optimizationStatus.batch_size || 'N/A'}}</div>
                            <div>Buffer Size: ${{optimizationStatus.buffer_size_kb || 'N/A'}} KB</div>
                        </div>
                    </div>
                `;
                console.log('✅ Performance analysis initialized');
            }} else {{
                const perfContent = document.getElementById('performanceContent');
                if (perfContent) {{
                    perfContent.innerHTML = `
                        <div class="no-data">
                            <h3>⚡ Performance Analysis</h3>
                            <p>No performance data available in the current analysis.</p>
                        </div>
                    `;
                }}
            }}
        }}
        
        // 初始化Security Analysis
        function initializeSecurityAnalysis(data) {{
            const secContent = document.getElementById('securityContent');
            if (secContent && data.security) {{
                const secData = data.security;
                const severity = secData.severity_breakdown || {{}};
                
                secContent.innerHTML = `
                    <div class="security-overview">
                        <h3>🔒 Security Analysis Summary</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Total Violations:</span>
                                <span class="stat-value">${{secData.total_violations}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Risk Level:</span>
                                <span class="stat-value risk-${{secData.risk_level.toLowerCase()}}">${{secData.risk_level}}</span>
                            </div>
                        </div>
                        
                        <h3>📊 Severity Breakdown</h3>
                        <div class="severity-breakdown">
                            <div class="severity-item critical">Critical: ${{severity.critical || 0}}</div>
                            <div class="severity-item high">High: ${{severity.high || 0}}</div>
                            <div class="severity-item medium">Medium: ${{severity.medium || 0}}</div>
                            <div class="severity-item low">Low: ${{severity.low || 0}}</div>
                            <div class="severity-item info">Info: ${{severity.info || 0}}</div>
                        </div>
                        
                        <h3>💡 Recommendations</h3>
                        <div class="recommendations">
                            ${{secData.recommendations.map(rec => `<div class="recommendation">• ${{rec}}</div>`).join('')}}
                        </div>
                    </div>
                `;
                console.log('✅ Security analysis initialized');
            }}
        }}
        
        // 初始化Memory Analysis详细信息
        function initializeMemoryAnalysisDetails(data) {{
            const memContent = document.getElementById('memoryAnalysisContent');
            if (memContent && data.allocations) {{
                const allocations = data.allocations.slice(0, 100); // 显示前100个分配
                
                memContent.innerHTML = `
                    <div class="memory-details">
                        <h3>🧠 Memory Allocations (Showing first 100 of ${{data.allocations.length}})</h3>
                        <div class="allocations-table">
                            <table>
                                <thead>
                                    <tr>
                                        <th>Address</th>
                                        <th>Size</th>
                                        <th>Variable</th>
                                        <th>Scope</th>
                                        <th>Status</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    ${{allocations.map(alloc => `
                                        <tr>
                                            <td><code>${{alloc.ptr}}</code></td>
                                            <td>${{formatBytes(alloc.size)}}</td>
                                            <td>${{alloc.var_name || 'Unknown'}}</td>
                                            <td>${{alloc.scope_name || 'Global'}}</td>
                                            <td>${{alloc.timestamp_dealloc ? 'Freed' : 'Active'}}</td>
                                        </tr>
                                    `).join('')}}
                                </tbody>
                            </table>
                        </div>
                    </div>
                `;
                console.log('✅ Memory analysis details initialized');
            }}
        }}
        
        // 初始化Variable Relationships
        function initializeVariableRelationships(data) {{
            const varContent = document.getElementById('variableContent');
            if (varContent && data.variable_relationships) {{
                const relationships = data.variable_relationships.relationships || [];
                const registry = data.variable_relationships.registry || {{}};
                
                varContent.innerHTML = `
                    <div class="variable-relationships">
                        <h3>🔗 Variable Relationships Overview</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Total Variables:</span>
                                <span class="stat-value">${{relationships.length}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Registry Entries:</span>
                                <span class="stat-value">${{Object.keys(registry).length}}</span>
                            </div>
                        </div>
                        
                        <h3>📋 Variable Details</h3>
                        <div class="variables-table">
                            <table>
                                <thead>
                                    <tr>
                                        <th>Variable Name</th>
                                        <th>Type</th>
                                        <th>Size</th>
                                        <th>Allocation Address</th>
                                        <th>Timestamp</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    ${{relationships.map(rel => {{
                                        const regEntry = registry[rel.allocation_ptr] || {{}};
                                        return `
                                            <tr>
                                                <td><strong>${{rel.variable_name}}</strong></td>
                                                <td><code>${{rel.type_name}}</code></td>
                                                <td>${{formatBytes(rel.size)}}</td>
                                                <td><code>0x${{rel.allocation_ptr.toString(16)}}</code></td>
                                                <td>${{regEntry.timestamp ? new Date(regEntry.timestamp / 1000000).toLocaleString() : 'N/A'}}</td>
                                            </tr>
                                        `;
                                    }}).join('')}}
                                </tbody>
                            </table>
                        </div>
                        
                        <h3>🔍 Type Distribution</h3>
                        <div class="type-distribution">
                            ${{(() => {{
                                const typeCount = {{}};
                                relationships.forEach(rel => {{
                                    const baseType = rel.type_name.split('<')[0].split('::').pop();
                                    typeCount[baseType] = (typeCount[baseType] || 0) + 1;
                                }});
                                return Object.entries(typeCount)
                                    .sort((a, b) => b[1] - a[1])
                                    .map(([type, count]) => `
                                        <div class="type-item">
                                            <span class="type-name">${{type}}</span>
                                            <span class="type-count">${{count}}</span>
                                        </div>
                                    `).join('');
                            }})()}}
                        </div>
                        
                        <h3>📊 Memory Usage by Variable</h3>
                        <div class="memory-usage">
                            ${{relationships
                                .sort((a, b) => b.size - a.size)
                                .slice(0, 10)
                                .map(rel => `
                                    <div class="usage-item">
                                        <span class="var-name">${{rel.variable_name}}</span>
                                        <span class="var-size">${{formatBytes(rel.size)}}</span>
                                        <div class="usage-bar" style="width: ${{(rel.size / Math.max(...relationships.map(r => r.size))) * 100}}%"></div>
                                    </div>
                                `).join('')}}
                        </div>
                    </div>
                `;
                console.log('✅ Variable relationships initialized');
            }} else {{
                const varContent = document.getElementById('variableContent');
                if (varContent) {{
                    varContent.innerHTML = `
                        <div class="no-data-section">
                            <h3>🔗 Variable Relationships</h3>
                            <p>No variable relationship data available in the current analysis.</p>
                            <p>Make sure the variable_relationships.json file is included in your export.</p>
                        </div>
                    `;
                }}
            }}
        }}
        
        // 初始化Lifecycle Analysis
        function initializeLifecycleAnalysis(data) {{
            const lifecycleContent = document.getElementById('lifecycleContent');
            if (lifecycleContent && data.lifecycle) {{
                const lifecycleData = data.lifecycle;
                const events = lifecycleData.lifecycle_events || [];
                const summary = lifecycleData.summary || {{}};
                const scopeAnalysis = lifecycleData.scope_analysis || {{}};
                
                lifecycleContent.innerHTML = `
                    <div class="lifecycle-analysis">
                        <h3>⏱️ Lifecycle Analysis Overview</h3>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Total Events:</span>
                                <span class="stat-value">${{events.length}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Allocation Events:</span>
                                <span class="stat-value">${{events.filter(e => e.event === 'allocation').length}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Deallocation Events:</span>
                                <span class="stat-value">${{events.filter(e => e.event === 'deallocation').length}}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Active Scopes:</span>
                                <span class="stat-value">${{Object.keys(scopeAnalysis).length}}</span>
                            </div>
                        </div>
                        
                        <h3>📊 Scope Analysis</h3>
                        <div class="scope-analysis">
                            ${{Object.entries(scopeAnalysis).map(([scope, info]) => `
                                <div class="scope-item">
                                    <strong>${{scope}}</strong>: ${{info.allocation_count || 0}} allocations, 
                                    ${{formatBytes(info.total_memory || 0)}} total memory
                                </div>
                            `).join('')}}
                        </div>
                        
                        <h3>📈 Recent Lifecycle Events (Last 20)</h3>
                        <div class="timeline-events">
                            <table>
                                <thead>
                                    <tr>
                                        <th>Time</th>
                                        <th>Event</th>
                                        <th>Pointer</th>
                                        <th>Size</th>
                                        <th>Scope</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    ${{events.slice(0, 20).map(event => `
                                        <tr>
                                            <td>${{new Date(event.timestamp / 1000000).toLocaleTimeString()}}</td>
                                            <td><span class="event-${{event.event}}">${{event.event}}</span></td>
                                            <td><code>${{event.ptr}}</code></td>
                                            <td>${{formatBytes(event.size || 0)}}</td>
                                            <td>${{event.scope || 'unknown'}}</td>
                                        </tr>
                                    `).join('')}}
                                </tbody>
                            </table>
                        </div>
                        
                        <div class="variable-relationships-integration">
                            <h3>🔗 Variable Relationships (Integrated View)</h3>
                            <p>This section shows variable relationship data integrated within the lifecycle context.</p>
                            ${{data.variable_relationships && data.variable_relationships.relationships ? `
                                <div class="integrated-relationships">
                                    ${{data.variable_relationships.relationships.slice(0, 5).map(rel => `
                                        <div class="relationship-item">
                                            <strong>${{rel.variable_name}}</strong> 
                                            (<code>${{rel.type_name}}</code>) 
                                            - ${{formatBytes(rel.size)}}
                                        </div>
                                    `).join('')}}
                                    ${{data.variable_relationships.relationships.length > 5 ? `
                                        <div class="more-relationships">
                                            ... and ${{data.variable_relationships.relationships.length - 5}} more variables. 
                                            <span style="color: blue; cursor: pointer;">
                                                View all in Variable Relationships tab
                                            </span>
                                        </div>
                                    ` : ''}}
                                </div>
                            ` : `
                                <p>No variable relationship data available. Variable relationships are tracked separately.</p>
                            `}}
                        </div>
                    </div>
                `;
                console.log('✅ Lifecycle analysis initialized');
            }} else {{
                const lifecycleContent = document.getElementById('lifecycleContent');
                if (lifecycleContent) {{
                    lifecycleContent.innerHTML = `
                        <div class="no-data">
                            <h3>⏱️ Lifecycle Analysis</h3>
                            <p>No lifecycle data available in the current analysis.</p>
                            <p>Make sure the lifetime.json file is included in your export.</p>
                        </div>
                    `;
                }}
            }}
        }}
        
        // Helper function to switch to variables tab
        function switchToVariablesTab() {{
            const variablesTab = document.querySelector('[data-tab="variables"]');
            if (variablesTab) {{
                variablesTab.click();
            }}
        }}
        
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
    </script>
</body>
</html>"#,
        css_content = css_content,
        js_content = js_content,
        json_data_str = serde_json::to_string_pretty(json_data)?,
        total_memory = stats_summary.total_memory,
        active_allocs = stats_summary.active_allocs,
        peak_memory = stats_summary.peak_memory,
    );
    
    Ok(html)
}

/// Extract summary statistics for header display
fn extract_stats_summary(json_data: &JsonDataCollection) -> StatsSummary {
    let mut summary = StatsSummary::default();
    
    // 从memory_analysis中提取统计信息
    if let Some(memory_data) = json_data.get("memory_analysis") {
        if let Some(stats) = memory_data.get("statistics") {
            if let Some(overall) = stats.get("overall") {
                summary.total_memory = format_bytes(
                    overall.get("active_memory").and_then(|v| v.as_u64()).unwrap_or(0)
                );
                summary.peak_memory = format_bytes(
                    overall.get("peak_memory").and_then(|v| v.as_u64()).unwrap_or(0)
                );
                summary.active_allocs = overall.get("active_allocations")
                    .and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }
    }
    
    // 从performance中提取额外信息
    if let Some(_perf_data) = json_data.get("performance") {
        // 可以添加更多性能指标
    }
    
    summary
}

/// Statistics summary for header display
#[derive(Default)]
struct StatsSummary {
    total_memory: String,
    active_allocs: u64,
    peak_memory: String,
}

/// Format bytes in human readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}