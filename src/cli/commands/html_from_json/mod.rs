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
use std::time::Instant;
use rayon::prelude::*;

use crate::web::{MemScopeServer, server::ServerConfig};

pub mod data_normalizer;
mod data_integrator;
pub mod template_generator;

use data_normalizer::{DataNormalizer, UnifiedMemoryData};
use data_integrator::DataIntegrator;

mod direct_json_template;

/// Run the HTML from JSON generation command
pub async fn run_html_from_json(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
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

    // 🎯 Load JSON files
    let json_data = load_json_files(input_dir, base_name)?;
    
    // 🔄 Normalize data
    let normalizer = DataNormalizer::new();
    let mut unified_data = normalizer.normalize(&json_data)?;
    
    // 🔗 Integrate multiple data sources
    let integrator = DataIntegrator::new();
    let integration_stats = integrator.integrate(&mut unified_data)?;
    
    println!("📊 Integration Statistics:");
    println!("   Cross-references found: {}", integration_stats.cross_references_found);
    println!("   Conflicts resolved: {}", integration_stats.conflicts_resolved);
    println!("   Data enrichments: {}", integration_stats.enrichments_performed);
    println!("   Index build time: {}ms", integration_stats.index_build_time_ms);
    println!("   Total integration time: {}ms", integration_stats.integration_time_ms);
    
    // Check if web server should be started
    if matches.get_flag("serve") {
        let port = matches.get_one::<u16>("port").copied().unwrap_or(8080);
        
        println!("🚀 Starting MemScope web server...");
        println!("📊 Data loaded: {} allocations, {} variables", 
            unified_data.allocations.len(),
            integration_stats.cross_references_found);
        
        // Start web server
        let config = ServerConfig {
            port,
            host: "127.0.0.1".to_string(),
            static_dir: None,
        };
        
        let server = MemScopeServer::new(unified_data, config);
        server.serve().await?;
    } else {
        // 🎨 Generate static HTML report - using direct JSON data template
        println!("🎨 Using direct JSON data template with charts...");
        let html_content = direct_json_template::generate_direct_html(&json_data)?;
        
        let template_stats = crate::cli::commands::html_from_json::template_generator::TemplateStats {
            template_size_bytes: html_content.len(),
            css_processing_time_ms: 0,
            js_processing_time_ms: 0,
            serialization_time_ms: 0,
            generation_time_ms: 1,
            cache_hit_rate: 0.0,
            compression_ratio: Some(1.0),
        };
        
        println!("🎨 Template Generation Statistics:");
        println!("   Template size: {:.1} KB", template_stats.template_size_bytes as f64 / 1024.0);
        println!("   CSS processing: {}ms", template_stats.css_processing_time_ms);
        println!("   JS processing: {}ms", template_stats.js_processing_time_ms);
        println!("   Data serialization: {}ms", template_stats.serialization_time_ms);
        println!("   Total generation time: {}ms", template_stats.generation_time_ms);
        println!("   Cache hit rate: {:.1}%", template_stats.cache_hit_rate);
        
        // Write HTML file
        fs::write(output_file, html_content)?;
    }
    
    println!("✅ HTML report generated successfully!");
    println!("🌐 Open {} in your browser to view the interactive report", output_file);
    
    Ok(())
}

/// Configuration for JSON file loading
#[derive(Debug, Clone)]
pub struct JsonFileConfig {
    /// File suffix
    pub suffix: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Whether the file is required
    pub required: bool,
    /// Maximum file size in MB
    pub max_size_mb: Option<usize>,
}

impl JsonFileConfig {
    /// Create new file configuration
    pub fn new(suffix: &'static str, description: &'static str) -> Self {
        Self {
            suffix,
            description,
            required: false,
            max_size_mb: Some(100), // Default 100MB limit
        }
    }
    
    /// Mark file as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Set maximum file size
    pub fn max_size_mb(mut self, size: usize) -> Self {
        self.max_size_mb = Some(size);
        self
    }
}

/// Result of loading a single JSON file
#[derive(Debug)]
pub struct JsonLoadResult {
    /// File suffix
    pub suffix: String,
    /// Whether loading was successful
    pub success: bool,
    /// Loaded JSON data
    pub data: Option<Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// File size in bytes
    pub file_size: usize,
    /// Load time in milliseconds
    pub load_time_ms: u64,
}

/// Statistics for the JSON loading process
#[derive(Debug)]
pub struct JsonLoadStats {
    /// Total files attempted
    pub total_files_attempted: usize,
    /// Files successfully loaded
    pub files_loaded: usize,
    /// Files skipped
    pub files_skipped: usize,
    /// Files failed to load
    pub files_failed: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Total load time in milliseconds
    pub total_load_time_ms: u64,
    /// Whether parallel loading was used
    pub parallel_loading_used: bool,
}

/// Collection of JSON data from different analysis files
type JsonDataCollection = HashMap<String, Value>;

/// Optimized JSON file loader with parallel processing and monitoring
fn load_json_files(input_dir: &str, base_name: &str) -> Result<JsonDataCollection, Box<dyn Error>> {
    let start_time = Instant::now();
    
    // 定义要加载的JSON文件类型配置
    let file_configs = vec![
        JsonFileConfig::new("memory_analysis", "Memory Analysis").required(),
        JsonFileConfig::new("lifetime", "Lifecycle Analysis"),
        JsonFileConfig::new("unsafe_ffi", "Unsafe/FFI Analysis"),
        JsonFileConfig::new("performance", "Performance Metrics"),
        JsonFileConfig::new("complex_types", "Complex Types Analysis"),
        JsonFileConfig::new("security_violations", "Security Violations"),
        JsonFileConfig::new("variable_relationships", "Variable Relationships"),
    ];
    
    println!("🚀 Starting optimized JSON file loading...");
    println!("📁 Directory: {}", input_dir);
    println!("🏷️  Base name: {}", base_name);
    
    // check file exist and size
    let mut valid_files = Vec::new();
    let mut total_size = 0usize;
    
    for config in &file_configs {
        // try multiple file name formats
        let _possible_paths = vec![
            format!("{}/{}_{}.json", input_dir, base_name, config.suffix),
            format!("{}/*{}*.json", input_dir, config.suffix),
        ];
        
        let mut found_path = None;
        
        // try exact match first
        let exact_path = format!("{}/{}_{}.json", input_dir, base_name, config.suffix);
        if Path::new(&exact_path).exists() {
            found_path = Some(exact_path);
        } else {
            // if exact match fails, search for files containing keywords
            if let Ok(entries) = fs::read_dir(input_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if file_name.contains(&config.suffix) && file_name.ends_with(".json") {
                                found_path = Some(path.to_string_lossy().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(file_path) = found_path {
            let path = Path::new(&file_path);
        
            if path.exists() {
            match fs::metadata(&file_path) {
                Ok(metadata) => {
                    let file_size = metadata.len() as usize;
                    
                    // check file size limit
                    if let Some(max_size) = config.max_size_mb {
                        let max_bytes = max_size * 1024 * 1024;
                        if file_size > max_bytes {
                            println!("⚠️  File {} ({:.1} MB) exceeds size limit ({} MB), skipping", 
                                file_path, file_size as f64 / 1024.0 / 1024.0, max_size);
                            continue;
                        }
                    }
                    
                    total_size += file_size;
                    println!("✓ Found {}: {} ({:.1} KB)", 
                        config.description, file_path, file_size as f64 / 1024.0);
                    valid_files.push((config.clone(), file_path, file_size));
                }
                Err(e) => {
                    println!("⚠️  Cannot read metadata for {}: {}", file_path, e);
                }
            }
            } else if config.required {
                return Err(format!("Required file not found: {}_{}*.json", base_name, config.suffix).into());
            } else {
                println!("⚠️  Optional file not found: {}_{}*.json (skipping)", base_name, config.suffix);
            }
        }
    }
    
    if valid_files.is_empty() {
        return Err("No valid JSON files found! Please check the input directory and base name.".into());
    }
    
    println!("📊 Found {} valid files, total size: {:.1} MB", 
        valid_files.len(), total_size as f64 / 1024.0 / 1024.0);
    
    // decide whether to use parallel loading (>= 3 files or >= 10MB)
    let use_parallel = valid_files.len() >= 3 || total_size >= 10 * 1024 * 1024;
    
    if use_parallel {
        println!("⚡ Using parallel loading for {} files", valid_files.len());
    } else {
        println!("📝 Using sequential loading for {} files", valid_files.len());
    }
    
    // load files
    let results = if use_parallel {
        load_files_parallel(&valid_files)?
    } else {
        load_files_sequential(&valid_files)?
    };
    
    // process results
    let mut data = JsonDataCollection::new();
    let mut stats = JsonLoadStats {
        total_files_attempted: valid_files.len(),
        files_loaded: 0,
        files_skipped: 0,
        files_failed: 0,
        total_size_bytes: 0,
        total_load_time_ms: start_time.elapsed().as_millis() as u64,
        parallel_loading_used: use_parallel,
    };
    
    for result in results {
        if result.success {
            if let Some(json_data) = result.data {
                data.insert(result.suffix.clone(), json_data);
                stats.files_loaded += 1;
                stats.total_size_bytes += result.file_size;
                println!("✅ Loaded {} ({:.1} KB in {}ms)", 
                    result.suffix, result.file_size as f64 / 1024.0, result.load_time_ms);
            }
        } else {
            stats.files_failed += 1;
            println!("❌ Failed to load {}: {}", 
                result.suffix, result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }
    }
    
    // print statistics
    print_load_statistics(&stats);
    
    if data.is_empty() {
        return Err("No JSON files were successfully loaded!".into());
    }
    
    Ok(data)
}

/// Load files in parallel using rayon
fn load_files_parallel(files: &[(JsonFileConfig, String, usize)]) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    let results: Vec<JsonLoadResult> = files
        .par_iter()
        .map(|(config, file_path, file_size)| {
            load_single_file(config, file_path, *file_size)
        })
        .collect();
    
    Ok(results)
}

/// Load files sequentially
fn load_files_sequential(files: &[(JsonFileConfig, String, usize)]) -> Result<Vec<JsonLoadResult>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for (config, file_path, file_size) in files {
        results.push(load_single_file(config, file_path, *file_size));
    }
    
    Ok(results)
}

/// Load a single JSON file with error handling and timing
fn load_single_file(config: &JsonFileConfig, file_path: &str, file_size: usize) -> JsonLoadResult {
    let start_time = Instant::now();
    
    let result = match fs::read_to_string(file_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json_value) => {
                    // validate JSON structure
                    if let Err(validation_error) = validate_json_structure(&json_value, config.suffix) {
                        JsonLoadResult {
                            suffix: config.suffix.to_string(),
                            success: false,
                            data: None,
                            error: Some(format!("Validation error: {}", validation_error)),
                            file_size,
                            load_time_ms: start_time.elapsed().as_millis() as u64,
                        }
                    } else {
                        JsonLoadResult {
                            suffix: config.suffix.to_string(),
                            success: true,
                            data: Some(json_value),
                            error: None,
                            file_size,
                            load_time_ms: start_time.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => JsonLoadResult {
                    suffix: config.suffix.to_string(),
                    success: false,
                    data: None,
                    error: Some(format!("JSON parsing error: {}", e)),
                    file_size,
                    load_time_ms: start_time.elapsed().as_millis() as u64,
                }
            }
        }
        Err(e) => JsonLoadResult {
            suffix: config.suffix.to_string(),
            success: false,
            data: None,
            error: Some(format!("File read error: {}", e)),
            file_size,
            load_time_ms: start_time.elapsed().as_millis() as u64,
        }
    };
    
    result
}

/// Validate JSON structure based on file type
fn validate_json_structure(json: &Value, file_type: &str) -> Result<(), String> {
    match file_type {
        "memory_analysis" => {
            if !json.is_object() {
                return Err("Memory analysis JSON must be an object".to_string());
            }
            // can add more specific validation
        }
        "performance" => {
            if !json.is_object() {
                return Err("Performance JSON must be an object".to_string());
            }
        }
        _ => {
            // basic validation: ensure it's a valid JSON object or array
            if !json.is_object() && !json.is_array() {
                return Err("JSON must be an object or array".to_string());
            }
        }
    }
    Ok(())
}

/// Print loading statistics
fn print_load_statistics(stats: &JsonLoadStats) {
    println!("\n📈 JSON Loading Statistics:");
    println!("   Files attempted: {}", stats.total_files_attempted);
    println!("   Files loaded: {}", stats.files_loaded);
    println!("   Files failed: {}", stats.files_failed);
    println!("   Total size: {:.1} MB", stats.total_size_bytes as f64 / 1024.0 / 1024.0);
    println!("   Total time: {}ms", stats.total_load_time_ms);
    println!("   Parallel loading: {}", if stats.parallel_loading_used { "Yes" } else { "No" });
    
    if stats.files_loaded > 0 {
        let avg_time = stats.total_load_time_ms / stats.files_loaded as u64;
        let throughput = if stats.total_load_time_ms > 0 {
            (stats.total_size_bytes as f64 / 1024.0 / 1024.0) / (stats.total_load_time_ms as f64 / 1000.0)
        } else {
            0.0
        };
        println!("   Average time per file: {}ms", avg_time);
        println!("   Throughput: {:.1} MB/s", throughput);
    }
    println!();
}

/// Generate HTML report from unified data
fn generate_html_from_unified_data(
    unified_data: &UnifiedMemoryData, 
    output_file: &str
) -> Result<(), Box<dyn Error>> {
    
    // read template files
    let css_content = include_str!("../../../../templates/styles.css");
    let js_content = include_str!("../../../../templates/script.js");
    
    // build HTML content
    let html_content = build_html_template_unified(css_content, js_content, unified_data)?;
    
    // write to file
    fs::write(output_file, html_content)?;
    
    Ok(())
}

/// Build complete HTML template with unified data
fn build_html_template_unified(
    css_content: &str,
    js_content: &str, 
    unified_data: &UnifiedMemoryData
) -> Result<String, Box<dyn Error>> {
    
    // prepare data summary for header statistics
    let stats = &unified_data.stats;
    
    // format statistics information
    let total_memory = format_bytes(stats.active_memory);
    let active_allocs = format!("{} Active", stats.active_allocations);
    let peak_memory = format_bytes(stats.peak_memory);
    
    // serialize unified data to JSON
    let json_data_str = serde_json::to_string(unified_data)?;
    
    // build complete HTML
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
                <span class="stat-badge" id="activeAllocs">{active_allocs}</span>
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
            <button class="tab-btn" data-tab="complex-types">🔧 Complex Types</button>
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

            <!-- Complex Types Tab -->
            <div class="tab-content" id="complex-types">
                <div id="complexTypesContent">Loading complex types analysis...</div>
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
        // 🎯 统一的数据结构
        const UNIFIED_DATA = {json_data_str};
        
        // 🚀 增强的JavaScript功能
        {js_content}
        
        // 🎨 初始化统一数据支持
        document.addEventListener('DOMContentLoaded', function() {{
            console.log('🎯 Initializing unified memory analysis...');
            console.log('📊 Unified data structure loaded:', UNIFIED_DATA);
            
            // 初始化可视化器
            if (typeof MemScopeVisualizer !== 'undefined') {{
                window.memscope = new MemScopeVisualizer(UNIFIED_DATA);
                console.log('✅ MemScope visualizer initialized with unified data');
            }} else {{
                console.warn('⚠️ MemScopeVisualizer not found, falling back to basic initialization');
                initializeBasicViewUnified(UNIFIED_DATA);
            }}
        }});
        
        // 基础视图初始化（当MemScopeVisualizer不可用时）
        function initializeBasicViewUnified(data) {{
            console.log('🎯 Initializing basic view with unified data:', data);
            
            // 更新header统计
            updateHeaderStats(data.stats);
            
            // 填充Overview内容
            initializeOverviewUnified(data);
            
            // 初始化各个标签页
            initializePerformanceAnalysisUnified(data.performance);
            initializeSecurityAnalysisUnified(data.security);
            initializeMemoryAnalysisDetailsUnified(data.allocations);
            initializeVariableRelationshipsUnified(data.variable_relationships);
            initializeLifecycleAnalysisUnified(data.lifecycle);
            initializeComplexTypesAnalysisUnified(data.complex_types);
            
            console.log('✅ Basic unified view initialized');
        }}
        
        // 更新header统计信息
        function updateHeaderStats(stats) {{
            const totalMemoryEl = document.getElementById('totalMemory');
            const activeAllocsEl = document.getElementById('activeAllocs');
            const peakMemoryEl = document.getElementById('peakMemory');
            
            if (totalMemoryEl) totalMemoryEl.textContent = formatBytes(stats.active_memory);
            if (activeAllocsEl) activeAllocsEl.textContent = stats.active_allocations + ' Active';
            if (peakMemoryEl) peakMemoryEl.textContent = formatBytes(stats.peak_memory);
        }}
        
        // 初始化Overview
        function initializeOverviewUnified(data) {{
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
                            <span class="stat-value">${{data.stats.memory_efficiency.toFixed(1)}}%</span>
                        </div>
                    </div>
                `;
            }}
        }}
        
        // 其他初始化函数的占位符
        function initializePerformanceAnalysisUnified(performance) {{
            console.log('Initializing performance analysis:', performance);
        }}
        
        function initializeSecurityAnalysisUnified(security) {{
            console.log('Initializing security analysis:', security);
        }}
        
        function initializeMemoryAnalysisDetailsUnified(allocations) {{
            console.log('Initializing memory analysis details:', allocations.length, 'allocations');
        }}
        
        function initializeVariableRelationshipsUnified(relationships) {{
            console.log('Initializing variable relationships:', relationships);
        }}
        
        function initializeLifecycleAnalysisUnified(lifecycle) {{
            console.log('Initializing lifecycle analysis:', lifecycle);
        }}
        
        function initializeComplexTypesAnalysisUnified(complexTypes) {{
            console.log('Initializing complex types analysis:', complexTypes);
        }}
        
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
    </script>
</body>
</html>"#);

    Ok(html)
}

/// Format bytes into human-readable string
fn format_bytes(bytes: usize) -> String {
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
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}