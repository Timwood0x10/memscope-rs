//! HTML converter for binary-to-HTML report generation
//!
//! This module provides comprehensive HTML conversion capabilities including:
//! - Binary data to interactive HTML report conversion
//! - Integration with existing HTML templates and visualization
//! - Performance optimizations for large datasets
//! - Customizable report generation with multiple themes

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use crate::export::formats::binary_export::BinaryExportData;
use std::collections::HashMap;
use std::path::Path;

/// Configuration options for HTML conversion
#[derive(Debug, Clone)]
pub struct HtmlConvertOptions {
    /// Include interactive charts and visualizations
    pub include_charts: bool,
    /// Include detailed allocation tables
    pub include_allocation_tables: bool,
    /// Include timeline visualization
    pub include_timeline: bool,
    /// Include memory statistics dashboard
    pub include_statistics: bool,
    /// Enable dark mode theme
    pub dark_mode: bool,
    /// Maximum number of allocations to display in tables
    pub max_table_rows: usize,
    /// Enable virtual scrolling for large datasets
    pub enable_virtual_scrolling: bool,
    /// Include search and filter functionality
    pub include_search: bool,
    /// Custom CSS styles to inject
    pub custom_css: Option<String>,
    /// Custom JavaScript to inject
    pub custom_js: Option<String>,
    /// Report title
    pub report_title: String,
}

impl HtmlConvertOptions {
    /// Fast HTML generation - minimal features for quick reports
    pub fn fast() -> Self {
        Self {
            include_charts: false,
            include_allocation_tables: true,
            include_timeline: false,
            include_statistics: true,
            dark_mode: false,
            max_table_rows: 1000,
            enable_virtual_scrolling: false,
            include_search: false,
            custom_css: None,
            custom_js: None,
            report_title: "Memory Analysis Report".to_string(),
        }
    }

    /// Complete HTML generation - all features enabled
    pub fn complete() -> Self {
        Self {
            include_charts: true,
            include_allocation_tables: true,
            include_timeline: true,
            include_statistics: true,
            dark_mode: false,
            max_table_rows: 10000,
            enable_virtual_scrolling: true,
            include_search: true,
            custom_css: None,
            custom_js: None,
            report_title: "Comprehensive Memory Analysis Report".to_string(),
        }
    }

    /// Performance optimized - suitable for large datasets
    pub fn performance() -> Self {
        Self {
            include_charts: true,
            include_allocation_tables: true,
            include_timeline: false, // Skip timeline for performance
            include_statistics: true,
            dark_mode: false,
            max_table_rows: 5000,
            enable_virtual_scrolling: true,
            include_search: true,
            custom_css: None,
            custom_js: None,
            report_title: "Performance-Optimized Memory Report".to_string(),
        }
    }

    /// Dashboard style - focused on overview and statistics
    pub fn dashboard() -> Self {
        Self {
            include_charts: true,
            include_allocation_tables: false,
            include_timeline: false,
            include_statistics: true,
            dark_mode: false,
            max_table_rows: 100,
            enable_virtual_scrolling: false,
            include_search: false,
            custom_css: None,
            custom_js: None,
            report_title: "Memory Analysis Dashboard".to_string(),
        }
    }
}

impl Default for HtmlConvertOptions {
    fn default() -> Self {
        Self::complete()
    }
}

/// Statistics collected during HTML conversion
#[derive(Debug, Clone)]
pub struct HtmlConversionStats {
    /// Total conversion time
    pub conversion_time: std::time::Duration,
    /// Number of allocations processed
    pub allocations_processed: usize,
    /// Size of generated HTML in bytes
    pub html_size: u64,
    /// Number of charts generated
    pub charts_generated: usize,
    /// Number of table rows generated
    pub table_rows_generated: usize,
    /// Template processing time
    pub template_processing_time: std::time::Duration,
}

/// Template data structure for HTML generation
#[derive(Debug, Clone)]
pub struct HtmlTemplateData {
    /// Report title
    pub title: String,
    /// Generation timestamp
    pub timestamp: String,
    /// Memory summary statistics
    pub summary: MemorySummary,
    /// Allocation data for tables
    pub allocations: Vec<AllocationInfo>,
    /// Chart data in JSON format
    pub charts_data: String,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Custom CSS styles
    pub custom_css: String,
    /// Custom JavaScript code
    pub custom_js: String,
    /// Configuration flags
    pub config: HtmlConfigFlags,
}

/// Memory summary for dashboard display
#[derive(Debug, Clone)]
pub struct MemorySummary {
    /// Total allocations count
    pub total_allocations: usize,
    /// Active allocations count
    pub active_allocations: usize,
    /// Total memory allocated
    pub total_memory: u64,
    /// Active memory
    pub active_memory: u64,
    /// Peak memory usage
    pub peak_memory: u64,
    /// Memory efficiency percentage
    pub efficiency_percentage: f64,
    /// Top memory consumers
    pub top_consumers: Vec<TopConsumer>,
}

/// Top memory consumer information
#[derive(Debug, Clone)]
pub struct TopConsumer {
    /// Type name
    pub type_name: String,
    /// Total memory used
    pub memory_used: u64,
    /// Number of allocations
    pub allocation_count: usize,
    /// Percentage of total memory
    pub percentage: f64,
}

/// Performance metrics for the report
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average allocation size
    pub avg_allocation_size: f64,
    /// Allocation frequency (allocations per second)
    pub allocation_frequency: f64,
    /// Memory fragmentation score
    pub fragmentation_score: f64,
    /// Lifetime distribution
    pub lifetime_distribution: HashMap<String, usize>,
}

/// Configuration flags for template rendering
#[derive(Debug, Clone)]
pub struct HtmlConfigFlags {
    /// Show charts section
    pub show_charts: bool,
    /// Show allocation tables
    pub show_tables: bool,
    /// Show timeline
    pub show_timeline: bool,
    /// Show statistics
    pub show_statistics: bool,
    /// Enable dark mode
    pub dark_mode: bool,
    /// Enable virtual scrolling
    pub virtual_scrolling: bool,
    /// Enable search functionality
    pub enable_search: bool,
}

/// HTML converter with template engine integration
pub struct HtmlConverter {
    /// Conversion configuration options
    options: HtmlConvertOptions,
    /// Template cache for performance
    template_cache: HashMap<String, String>,
}

impl HtmlConverter {
    /// Create a new HTML converter with specified options
    pub fn new(options: HtmlConvertOptions) -> Self {
        Self {
            options,
            template_cache: HashMap::new(),
        }
    }

    /// Create a converter with fast generation settings
    pub fn with_fast_settings() -> Self {
        Self::new(HtmlConvertOptions::fast())
    }

    /// Create a converter with complete feature set
    pub fn with_complete_settings() -> Self {
        Self::new(HtmlConvertOptions::complete())
    }

    /// Create a converter optimized for performance
    pub fn with_performance_settings() -> Self {
        Self::new(HtmlConvertOptions::performance())
    }

    /// Create a converter for dashboard-style reports
    pub fn with_dashboard_settings() -> Self {
        Self::new(HtmlConvertOptions::dashboard())
    }

    /// Convert binary data to HTML file
    pub fn convert_to_file<P: AsRef<Path>>(
        &mut self,
        data: &BinaryExportData,
        path: P,
    ) -> TrackingResult<HtmlConversionStats> {
        let path_str = path.as_ref().to_string_lossy();
        let start_time = std::time::Instant::now();

        println!("🌐 Starting binary-to-HTML conversion: {path_str}");
        println!(
            "📋 Conversion options: charts={}, tables={}, timeline={}, stats={}",
            self.options.include_charts,
            self.options.include_allocation_tables,
            self.options.include_timeline,
            self.options.include_statistics
        );

        let mut stats = HtmlConversionStats {
            conversion_time: std::time::Duration::from_millis(0),
            allocations_processed: 0,
            html_size: 0,
            charts_generated: 0,
            table_rows_generated: 0,
            template_processing_time: std::time::Duration::from_millis(0),
        };

        // Generate HTML content
        let html_content = self.generate_html_report(data, &mut stats)?;

        // Write to file
        println!("💾 Writing HTML to file...");
        let write_start = std::time::Instant::now();
        
        std::fs::write(&path, &html_content)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to write HTML file: {e}")
            ))?;

        let write_duration = write_start.elapsed();
        stats.html_size = html_content.len() as u64;

        println!("✅ File write completed in {write_duration:?}");

        stats.conversion_time = start_time.elapsed();

        println!("🎉 HTML conversion completed successfully!");
        println!("   - Total time: {:?}", stats.conversion_time);
        println!("   - Allocations processed: {}", stats.allocations_processed);
        println!("   - HTML size: {} bytes", stats.html_size);
        println!("   - Charts generated: {}", stats.charts_generated);
        println!("   - Table rows: {}", stats.table_rows_generated);

        Ok(stats)
    }

    /// Generate interactive HTML report from binary data
    pub fn generate_interactive_report(&mut self, data: &BinaryExportData) -> TrackingResult<String> {
        println!("🌐 Generating interactive HTML report...");
        
        let mut stats = HtmlConversionStats {
            conversion_time: std::time::Duration::from_millis(0),
            allocations_processed: 0,
            html_size: 0,
            charts_generated: 0,
            table_rows_generated: 0,
            template_processing_time: std::time::Duration::from_millis(0),
        };

        self.generate_html_report(data, &mut stats)
    }

    /// Generate complete HTML report
    fn generate_html_report(
        &mut self,
        data: &BinaryExportData,
        stats: &mut HtmlConversionStats,
    ) -> TrackingResult<String> {
        let template_start = std::time::Instant::now();

        // Prepare template data
        println!("🔧 Preparing template data...");
        let template_data = self.prepare_template_data(data, stats)?;

        // Load and process template
        println!("📄 Loading HTML template...");
        let template = self.load_template()?;

        // Render template with data
        println!("🎨 Rendering HTML template...");
        let html_content = self.render_template(&template, &template_data)?;

        stats.template_processing_time = template_start.elapsed();
        println!("✅ Template processing completed in {:?}", stats.template_processing_time);

        Ok(html_content)
    }

    /// Prepare data for template rendering
    fn prepare_template_data(
        &self,
        data: &BinaryExportData,
        stats: &mut HtmlConversionStats,
    ) -> TrackingResult<HtmlTemplateData> {
        println!("📊 Analyzing data for template...");

        // Generate memory summary
        let summary = self.generate_memory_summary(&data.stats, &data.allocations)?;

        // Prepare allocations for display (limit if necessary)
        let display_allocations = if data.allocations.len() > self.options.max_table_rows {
            println!("⚠️  Limiting allocations to {} rows for performance", self.options.max_table_rows);
            data.allocations[..self.options.max_table_rows].to_vec()
        } else {
            data.allocations.clone()
        };

        stats.allocations_processed = data.allocations.len();
        stats.table_rows_generated = display_allocations.len();

        // Generate charts data
        let charts_data = if self.options.include_charts {
            stats.charts_generated = 3; // Memory usage, allocation sizes, timeline
            self.generate_charts_data(&data.allocations)?
        } else {
            "{}".to_string()
        };

        // Generate performance metrics
        let performance_metrics = self.calculate_performance_metrics(&data.allocations)?;

        // Create template data
        let template_data = HtmlTemplateData {
            title: self.options.report_title.clone(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            summary,
            allocations: display_allocations,
            charts_data,
            performance_metrics,
            custom_css: self.options.custom_css.clone().unwrap_or_default(),
            custom_js: self.options.custom_js.clone().unwrap_or_default(),
            config: HtmlConfigFlags {
                show_charts: self.options.include_charts,
                show_tables: self.options.include_allocation_tables,
                show_timeline: self.options.include_timeline,
                show_statistics: self.options.include_statistics,
                dark_mode: self.options.dark_mode,
                virtual_scrolling: self.options.enable_virtual_scrolling,
                enable_search: self.options.include_search,
            },
        };

        Ok(template_data)
    }

    /// Generate memory summary statistics
    fn generate_memory_summary(
        &self,
        stats: &MemoryStats,
        allocations: &[AllocationInfo],
    ) -> TrackingResult<MemorySummary> {
        // Calculate top consumers
        let mut type_usage: HashMap<String, (u64, usize)> = HashMap::new();
        
        for allocation in allocations {
            let type_name = allocation.type_name.clone()
                .unwrap_or_else(|| "unknown".to_string());
            let entry = type_usage.entry(type_name).or_insert((0, 0));
            entry.0 += allocation.size as u64;
            entry.1 += 1;
        }

        let mut top_consumers: Vec<TopConsumer> = type_usage
            .into_iter()
            .map(|(type_name, (memory_used, allocation_count))| TopConsumer {
                type_name,
                memory_used,
                allocation_count,
                percentage: (memory_used as f64 / stats.active_memory as f64) * 100.0,
            })
            .collect();

        top_consumers.sort_by(|a, b| b.memory_used.cmp(&a.memory_used));
        top_consumers.truncate(10); // Top 10 consumers

        let efficiency_percentage = if stats.peak_memory > 0 {
            (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
        } else {
            100.0
        };

        Ok(MemorySummary {
            total_allocations: stats.total_allocations,
            active_allocations: stats.active_allocations,
            total_memory: stats.total_allocated as u64,
            active_memory: stats.active_memory as u64,
            peak_memory: stats.peak_memory as u64,
            efficiency_percentage,
            top_consumers,
        })
    }

    /// Generate charts data in JSON format
    fn generate_charts_data(&self, allocations: &[AllocationInfo]) -> TrackingResult<String> {
        let mut charts_data = serde_json::Map::new();

        // Memory usage over time chart
        let mut memory_timeline = Vec::new();
        let mut cumulative_memory = 0u64;
        
        for allocation in allocations {
            cumulative_memory += allocation.size as u64;
            memory_timeline.push(serde_json::json!({
                "timestamp": allocation.timestamp_alloc,
                "memory": cumulative_memory
            }));
        }
        
        charts_data.insert("memoryTimeline".to_string(), serde_json::Value::Array(memory_timeline));

        // Allocation size distribution
        let mut size_buckets = HashMap::new();
        for allocation in allocations {
            let bucket = match allocation.size {
                0..=64 => "0-64B",
                65..=1024 => "65B-1KB",
                1025..=65536 => "1KB-64KB",
                65537..=1048576 => "64KB-1MB",
                _ => ">1MB",
            };
            *size_buckets.entry(bucket).or_insert(0) += 1;
        }

        let size_distribution: Vec<_> = size_buckets
            .into_iter()
            .map(|(bucket, count)| serde_json::json!({
                "bucket": bucket,
                "count": count
            }))
            .collect();

        charts_data.insert("sizeDistribution".to_string(), serde_json::Value::Array(size_distribution));

        // Type usage chart
        let mut type_usage = HashMap::new();
        for allocation in allocations {
            let type_name = allocation.type_name.clone()
                .unwrap_or_else(|| "unknown".to_string());
            *type_usage.entry(type_name).or_insert(0) += allocation.size;
        }

        let mut type_data: Vec<_> = type_usage
            .into_iter()
            .map(|(type_name, total_size)| serde_json::json!({
                "type": type_name,
                "size": total_size
            }))
            .collect();

        type_data.sort_by(|a, b| {
            b.get("size").unwrap().as_u64().unwrap()
                .cmp(&a.get("size").unwrap().as_u64().unwrap())
        });
        type_data.truncate(15); // Top 15 types

        charts_data.insert("typeUsage".to_string(), serde_json::Value::Array(type_data));

        serde_json::to_string(&charts_data)
            .map_err(|e| crate::core::types::TrackingError::SerializationError(
                format!("Failed to serialize charts data: {e}")
            ))
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, allocations: &[AllocationInfo]) -> TrackingResult<PerformanceMetrics> {
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let avg_allocation_size = if !allocations.is_empty() {
            total_size as f64 / allocations.len() as f64
        } else {
            0.0
        };

        // Calculate allocation frequency (simplified)
        let allocation_frequency = if !allocations.is_empty() {
            let time_span = allocations.last().unwrap().timestamp_alloc - allocations.first().unwrap().timestamp_alloc;
            if time_span > 0 {
                (allocations.len() as f64) / (time_span as f64 / 1_000_000_000.0) // Convert nanoseconds to seconds
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Simple fragmentation score (could be more sophisticated)
        let fragmentation_score = if !allocations.is_empty() {
            let size_variance = self.calculate_size_variance(allocations);
            (size_variance / avg_allocation_size).min(1.0).max(0.0)
        } else {
            0.0
        };

        // Lifetime distribution
        let mut lifetime_distribution = HashMap::new();
        for allocation in allocations {
            if let Some(lifetime) = allocation.lifetime_ms {
                let bucket = match lifetime {
                    0..=100 => "0-100ms",
                    101..=1000 => "100ms-1s",
                    1001..=10000 => "1s-10s",
                    10001..=60000 => "10s-1min",
                    _ => ">1min",
                };
                *lifetime_distribution.entry(bucket.to_string()).or_insert(0) += 1;
            }
        }

        Ok(PerformanceMetrics {
            avg_allocation_size,
            allocation_frequency,
            fragmentation_score,
            lifetime_distribution,
        })
    }

    /// Calculate variance in allocation sizes
    fn calculate_size_variance(&self, allocations: &[AllocationInfo]) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let mean = allocations.iter().map(|a| a.size as f64).sum::<f64>() / allocations.len() as f64;
        let variance = allocations
            .iter()
            .map(|a| {
                let diff = a.size as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / allocations.len() as f64;

        variance.sqrt()
    }

    /// Load HTML template from embedded or file system
    fn load_template(&mut self) -> TrackingResult<String> {
        // Check cache first
        if let Some(cached_template) = self.template_cache.get("main") {
            return Ok(cached_template.clone());
        }

        // Try to load from templates directory
        let template_path = "templates/dashboard.html";
        let template_content = if std::path::Path::new(template_path).exists() {
            std::fs::read_to_string(template_path)
                .map_err(|e| crate::core::types::TrackingError::IoError(
                    format!("Failed to read template file: {e}")
                ))?
        } else {
            // Use embedded default template
            self.get_default_template()
        };

        // Cache the template
        self.template_cache.insert("main".to_string(), template_content.clone());

        Ok(template_content)
    }

    /// Render template with data
    fn render_template(&self, template: &str, data: &HtmlTemplateData) -> TrackingResult<String> {
        let mut rendered = template.to_string();

        // Replace template variables
        rendered = rendered.replace("{{ title }}", &data.title);
        rendered = rendered.replace("{{ timestamp }}", &data.timestamp);
        
        // Serialize data for JavaScript
        let json_data = serde_json::to_string(&data.allocations)
            .map_err(|e| crate::core::types::TrackingError::SerializationError(
                format!("Failed to serialize allocation data: {e}")
            ))?;
        
        rendered = rendered.replace("{{ json_data }}", &json_data);
        rendered = rendered.replace("{{ charts_data }}", &data.charts_data);
        
        // Replace configuration flags
        rendered = rendered.replace("{{ show_charts }}", &data.config.show_charts.to_string());
        rendered = rendered.replace("{{ show_tables }}", &data.config.show_tables.to_string());
        rendered = rendered.replace("{{ show_timeline }}", &data.config.show_timeline.to_string());
        rendered = rendered.replace("{{ show_statistics }}", &data.config.show_statistics.to_string());
        
        // Add custom CSS and JS
        if !data.custom_css.is_empty() {
            rendered = rendered.replace("{{ custom_css }}", &format!("<style>{}</style>", data.custom_css));
        } else {
            rendered = rendered.replace("{{ custom_css }}", "");
        }
        
        if !data.custom_js.is_empty() {
            rendered = rendered.replace("{{ custom_js }}", &format!("<script>{}</script>", data.custom_js));
        } else {
            rendered = rendered.replace("{{ custom_js }}", "");
        }

        Ok(rendered)
    }

    /// Get default embedded HTML template
    fn get_default_template(&self) -> String {
        // Simplified default template
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    {{ custom_css }}
</head>
<body class="bg-gray-100">
    <div class="container mx-auto px-4 py-8">
        <header class="mb-8">
            <h1 class="text-3xl font-bold text-gray-800">{{ title }}</h1>
            <p class="text-gray-600">Generated on {{ timestamp }}</p>
        </header>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-lg font-semibold text-gray-700">Total Allocations</h3>
                <p class="text-2xl font-bold text-blue-600" id="total-allocations">-</p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-lg font-semibold text-gray-700">Active Memory</h3>
                <p class="text-2xl font-bold text-green-600" id="active-memory">-</p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-lg font-semibold text-gray-700">Peak Memory</h3>
                <p class="text-2xl font-bold text-red-600" id="peak-memory">-</p>
            </div>
            <div class="bg-white p-6 rounded-lg shadow">
                <h3 class="text-lg font-semibold text-gray-700">Efficiency</h3>
                <p class="text-2xl font-bold text-purple-600" id="efficiency">-</p>
            </div>
        </div>

        <div class="bg-white p-6 rounded-lg shadow mb-8">
            <h2 class="text-xl font-semibold text-gray-800 mb-4">Memory Allocations</h2>
            <div class="overflow-x-auto">
                <table class="min-w-full table-auto">
                    <thead>
                        <tr class="bg-gray-50">
                            <th class="px-4 py-2 text-left">Pointer</th>
                            <th class="px-4 py-2 text-left">Size</th>
                            <th class="px-4 py-2 text-left">Type</th>
                            <th class="px-4 py-2 text-left">Status</th>
                        </tr>
                    </thead>
                    <tbody id="allocations-table">
                        <!-- Populated by JavaScript -->
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <script>
        // Data from Rust
        const analysisData = {{ json_data }};
        const chartsData = {{ charts_data }};
        
        // Populate statistics
        document.getElementById('total-allocations').textContent = analysisData.length;
        
        // Populate table
        const tableBody = document.getElementById('allocations-table');
        analysisData.slice(0, 100).forEach(allocation => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td class="px-4 py-2 font-mono text-sm">${allocation.ptr}</td>
                <td class="px-4 py-2">${allocation.size} bytes</td>
                <td class="px-4 py-2">${allocation.type_name || 'unknown'}</td>
                <td class="px-4 py-2">
                    <span class="px-2 py-1 text-xs rounded ${allocation.timestamp_dealloc ? 'bg-red-100 text-red-800' : 'bg-green-100 text-green-800'}">
                        ${allocation.timestamp_dealloc ? 'Deallocated' : 'Active'}
                    </span>
                </td>
            `;
            tableBody.appendChild(row);
        });
    </script>
    {{ custom_js }}
</body>
</html>"#.to_string()
    }
}

impl Default for HtmlConverter {
    fn default() -> Self {
        Self::with_complete_settings()
    }
}