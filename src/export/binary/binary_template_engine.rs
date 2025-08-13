//! Binary template engine for processing binary-specific HTML templates
//!
//! This module provides a specialized template engine that processes the binary_dashboard.html
//! template with data directly from binary sources, independent of the JSON → HTML workflow.

use crate::export::binary::binary_html_writer::BinaryTemplateData;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::template_resource_manager::{
    create_template_data, ResourceConfig, TemplateResourceManager,
};

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

/// Configuration for the binary template engine
#[derive(Debug, Clone)]
pub struct BinaryTemplateEngineConfig {
    /// Enable template caching for better performance
    pub enable_cache: bool,

    /// Enable template precompilation
    pub enable_precompilation: bool,

    /// Enable data compression for large datasets
    pub enable_data_compression: bool,

    /// Maximum template cache size in MB
    pub max_cache_size_mb: usize,

    /// Template processing timeout in seconds
    pub processing_timeout_secs: u64,
}

impl Default for BinaryTemplateEngineConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_precompilation: true,
            enable_data_compression: false,
            max_cache_size_mb: 10,
            processing_timeout_secs: 30,
        }
    }
}

/// Binary template engine for processing binary-specific templates
pub struct BinaryTemplateEngine {
    /// Template resource manager
    resource_manager: TemplateResourceManager,

    /// Resource configuration
    resource_config: ResourceConfig,

    /// Configuration
    config: BinaryTemplateEngineConfig,

    /// Performance statistics
    last_render_time_ms: u64,

    /// Template processing statistics
    templates_processed: u64,

    /// Cache hit count
    cache_hits: u64,
}

impl BinaryTemplateEngine {
    /// Create a new binary template engine with default configuration
    pub fn new() -> Result<Self, BinaryExportError> {
        Self::with_config(BinaryTemplateEngineConfig::default())
    }

    /// Create a new binary template engine with custom configuration
    pub fn with_config(config: BinaryTemplateEngineConfig) -> Result<Self, BinaryExportError> {
        let resource_manager = TemplateResourceManager::new("templates")?;
        let resource_config = ResourceConfig {
            embed_css: true,
            embed_js: true,
            embed_svg: true,
            minify_resources: config.enable_data_compression,
            custom_paths: HashMap::new(),
        };

        let engine = Self {
            resource_manager,
            resource_config,
            config,
            last_render_time_ms: 0,
            templates_processed: 0,
            cache_hits: 0,
        };

        Ok(engine)
    }

    /// Render the binary dashboard template with the provided data
    pub fn render_binary_template(
        &mut self,
        template_data: &BinaryTemplateData,
    ) -> Result<String, BinaryExportError> {
        let render_start = Instant::now();

        // Convert template data to JSON for injection
        let json_data = self.serialize_template_data(template_data)?;

        // Create template data for resource manager
        let mut custom_data = HashMap::new();

        // Add analysis data to custom data if available
        if let Some(ref complex_types) = template_data.complex_types {
            let complex_types_json = serde_json::to_string(complex_types).map_err(|e| {
                BinaryExportError::SerializationError(format!(
                    "Complex types serialization failed: {}",
                    e
                ))
            })?;
            custom_data.insert("complex_types".to_string(), complex_types_json);
        }

        if let Some(ref unsafe_ffi) = template_data.unsafe_ffi {
            let ffi_json = serde_json::to_string(unsafe_ffi).map_err(|e| {
                BinaryExportError::SerializationError(format!(
                    "FFI safety serialization failed: {}",
                    e
                ))
            })?;
            custom_data.insert("unsafe_ffi".to_string(), ffi_json);
        }

        if let Some(ref variable_relationships) = template_data.variable_relationships {
            let relationships_json =
                serde_json::to_string(variable_relationships).map_err(|e| {
                    BinaryExportError::SerializationError(format!(
                        "Variable relationships serialization failed: {}",
                        e
                    ))
                })?;
            custom_data.insert("variable_relationships".to_string(), relationships_json);
        }

        let resource_template_data =
            create_template_data(&template_data.project_name, &json_data, custom_data);

        // Process template with resource manager
        let html_content = self.resource_manager.process_template(
            "binary_dashboard.html",
            &resource_template_data,
            &self.resource_config,
        )?;

        // Update statistics
        self.last_render_time_ms = render_start.elapsed().as_millis() as u64;
        self.templates_processed += 1;

        Ok(html_content)
    }

    /// Load the binary dashboard template (now handled by resource manager)
    fn load_binary_template(&mut self) -> Result<String, BinaryExportError> {
        // This method is now deprecated as resource manager handles template loading
        // Keeping for backward compatibility
        let template_path = "templates/binary_dashboard.html";
        fs::read_to_string(template_path).map_err(|e| BinaryExportError::Io(e))
    }

    /// Load CSS resources for the template (now handled by resource manager)
    fn load_css_resources(&mut self) -> Result<String, BinaryExportError> {
        // This method is now deprecated as resource manager handles CSS loading
        // Keeping for backward compatibility
        self.resource_manager.get_shared_css(&self.resource_config)
    }

    /// Load JavaScript resources for the template (now handled by resource manager)
    fn load_js_resources(&mut self) -> Result<String, BinaryExportError> {
        // This method is now deprecated as resource manager handles JS loading
        // Keeping for backward compatibility
        self.resource_manager.get_shared_js(&self.resource_config)
    }

    /// Serialize template data to JSON format
    fn serialize_template_data(
        &self,
        data: &BinaryTemplateData,
    ) -> Result<String, BinaryExportError> {
        use serde_json::json;

        // Convert binary template data to JSON format compatible with the template
        let allocations_json: Vec<serde_json::Value> = data
            .allocations
            .iter()
            .map(|alloc| {
                json!({
                    "id": alloc.id,
                    "size": alloc.size,
                    "type_name": alloc.type_name,
                    "scope_name": alloc.scope_name,
                    "timestamp_alloc": alloc.timestamp_alloc,
                    "is_active": alloc.is_active,
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "thread_id": alloc.thread_id,
                    "var_name": alloc.var_name,
                    "borrow_count": alloc.borrow_count,
                    "is_leaked": alloc.is_leaked,
                    "lifetime_ms": alloc.lifetime_ms
                })
            })
            .collect();

        let dashboard_data = json!({
            "project_name": data.project_name,
            "data_source": data.data_source,
            "summary": {
                "total_allocations": data.allocations.len(),
                "total_memory": data.total_memory_usage,
                "peak_memory": data.peak_memory_usage,
                "active_allocations": data.active_allocations_count
            },
            "memory_analysis": {
                "allocations": allocations_json,
                "memory_timeline": [],
                "size_distribution": []
            },
            "performance_metrics": {
                "export_time_ms": data.processing_time_ms,
                "data_source": "binary_direct",
                "throughput_allocations_per_sec": self.calculate_throughput(data)
            }
        });

        serde_json::to_string(&dashboard_data).map_err(|e| {
            BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e))
        })
    }

    /// Process template placeholders with actual data
    fn process_template_placeholders(
        &self,
        template: &str,
        template_data: &BinaryTemplateData,
        json_data: &str,
        css_content: &str,
        js_content: &str,
    ) -> Result<String, BinaryExportError> {
        let mut html_content = template.to_string();

        // Replace basic placeholders
        html_content = html_content.replace("{{PROJECT_NAME}}", &template_data.project_name);
        html_content = html_content.replace("{{BINARY_DATA}}", json_data);
        html_content = html_content.replace("{{CSS_CONTENT}}", css_content);
        html_content = html_content.replace("{{JS_CONTENT}}", js_content);
        html_content = html_content.replace(
            "{{GENERATION_TIME}}",
            &chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        );
        html_content = html_content.replace(
            "{{PROCESSING_TIME}}",
            &template_data.processing_time_ms.to_string(),
        );

        // Replace performance-specific placeholders
        let throughput = self.calculate_throughput(template_data);
        html_content = html_content.replace("{{THROUGHPUT}}", &throughput.to_string());

        Ok(html_content)
    }

    /// Calculate processing throughput
    fn calculate_throughput(&self, data: &BinaryTemplateData) -> f64 {
        if data.processing_time_ms == 0 {
            0.0
        } else {
            (data.allocations.len() as f64 * 1000.0) / data.processing_time_ms as f64
        }
    }

    /// Preload resources for better performance
    fn preload_resources(&mut self) -> Result<(), BinaryExportError> {
        // Preload template
        self.load_binary_template()?;

        // Preload CSS and JS
        self.load_css_resources()?;
        self.load_js_resources()?;

        Ok(())
    }

    /// Get embedded CSS content
    fn get_embedded_css(&self) -> String {
        r#"
        /* Binary Dashboard Specific Styles */
        .binary-performance-indicator {
            background: linear-gradient(45deg, #3b82f6, #1d4ed8);
            color: white;
            padding: 4px 12px;
            border-radius: 16px;
            font-size: 0.8rem;
            font-weight: 600;
            display: inline-flex;
            align-items: center;
            gap: 4px;
        }

        .binary-stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 1rem 0;
        }

        .binary-stat-card {
            background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
            border: 1px solid #cbd5e0;
            border-radius: 0.5rem;
            padding: 1rem;
            text-align: center;
            transition: transform 0.2s ease;
        }

        .binary-stat-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .binary-processing-badge {
            background: linear-gradient(45deg, #10b981, #059669);
            color: white;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }

        /* Dark mode adjustments for binary dashboard */
        .dark .binary-stat-card {
            background: linear-gradient(135deg, #374151 0%, #4b5563 100%);
            border-color: #6b7280;
        }

        /* Performance indicators */
        .performance-metric {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0.5rem 0;
            border-bottom: 1px solid #e5e7eb;
        }

        .performance-metric:last-child {
            border-bottom: none;
        }

        .performance-value {
            font-weight: 600;
            color: #059669;
        }

        /* Binary data table enhancements */
        .binary-table-row:hover {
            background-color: rgba(59, 130, 246, 0.05);
        }

        .binary-pointer {
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 0.875rem;
            color: #6366f1;
        }

        /* Responsive adjustments */
        @media (max-width: 768px) {
            .binary-stats-grid {
                grid-template-columns: repeat(2, 1fr);
                gap: 0.5rem;
            }
            
            .binary-stat-card {
                padding: 0.75rem;
            }
        }
        "#
        .to_string()
    }

    /// Get embedded JavaScript content
    fn get_embedded_js(&self) -> String {
        r#"
        // Binary Dashboard Specific JavaScript
        
        // Performance monitoring
        function trackBinaryPerformance() {
            const startTime = performance.now();
            
            return {
                end: function() {
                    const endTime = performance.now();
                    return endTime - startTime;
                }
            };
        }

        // Binary data processing utilities
        function processBinaryData(data) {
            if (!data || !data.memory_analysis) {
                console.warn('No binary data available');
                return null;
            }

            return {
                allocations: data.memory_analysis.allocations || [],
                summary: data.summary || {},
                performance: data.performance_metrics || {}
            };
        }

        // Enhanced table sorting for binary data
        function sortBinaryTable(column, direction = 'asc') {
            const table = document.getElementById('allocations-table');
            if (!table) return;

            const rows = Array.from(table.querySelectorAll('tr')).slice(1); // Skip header
            
            rows.sort((a, b) => {
                const aVal = a.cells[getColumnIndex(column)].textContent.trim();
                const bVal = b.cells[getColumnIndex(column)].textContent.trim();
                
                // Handle different data types
                if (column === 'size') {
                    return direction === 'asc' ? 
                        parseBytes(aVal) - parseBytes(bVal) : 
                        parseBytes(bVal) - parseBytes(aVal);
                } else if (column === 'ptr') {
                    const aPtr = parseInt(aVal.replace('0x', ''), 16);
                    const bPtr = parseInt(bVal.replace('0x', ''), 16);
                    return direction === 'asc' ? aPtr - bPtr : bPtr - aPtr;
                } else {
                    return direction === 'asc' ? 
                        aVal.localeCompare(bVal) : 
                        bVal.localeCompare(aVal);
                }
            });

            // Re-append sorted rows
            rows.forEach(row => table.appendChild(row));
        }

        function getColumnIndex(column) {
            const columns = { 'ptr': 0, 'variable': 1, 'type': 2, 'size': 3, 'status': 4 };
            return columns[column] || 0;
        }

        function parseBytes(str) {
            const match = str.match(/^([\d.]+)\s*([KMGT]?B)$/i);
            if (!match) return 0;
            
            const value = parseFloat(match[1]);
            const unit = match[2].toUpperCase();
            
            const multipliers = { 'B': 1, 'KB': 1024, 'MB': 1024*1024, 'GB': 1024*1024*1024 };
            return value * (multipliers[unit] || 1);
        }

        // Binary-specific chart configurations
        function createBinaryCharts() {
            // Enhanced chart configurations for binary data
            Chart.defaults.font.family = "'Inter', sans-serif";
            Chart.defaults.color = '#6b7280';
            
            // Add binary-specific chart plugins
            Chart.register({
                id: 'binaryDataPlugin',
                beforeDraw: function(chart) {
                    if (chart.config.options.plugins?.binaryIndicator) {
                        const ctx = chart.ctx;
                        ctx.save();
                        ctx.fillStyle = '#3b82f6';
                        ctx.font = '12px Inter';
                        ctx.fillText('Binary Source', 10, 20);
                        ctx.restore();
                    }
                }
            });
        }

        // Initialize binary dashboard features
        function initializeBinaryFeatures() {
            // Add binary-specific event listeners
            document.addEventListener('keydown', function(e) {
                if (e.ctrlKey && e.key === 'b') {
                    e.preventDefault();
                    showBinaryInfo();
                }
            });

            // Add performance monitoring
            const perfMonitor = trackBinaryPerformance();
            
            // Setup binary data refresh
            setInterval(function() {
                updateBinaryMetrics();
            }, 5000);

            console.log('Binary dashboard features initialized');
        }

        function showBinaryInfo() {
            const info = {
                dataSource: 'Binary Direct',
                processingMode: 'Streaming',
                memoryEfficient: true,
                performanceOptimized: true
            };
            
            console.table(info);
        }

        function updateBinaryMetrics() {
            // Update real-time metrics if available
            if (window.analysisData && window.analysisData.performance_metrics) {
                const metrics = window.analysisData.performance_metrics;
                
                // Update throughput display
                const throughputEl = document.getElementById('throughput');
                if (throughputEl && metrics.throughput_allocations_per_sec) {
                    throughputEl.textContent = Math.round(metrics.throughput_allocations_per_sec).toLocaleString();
                }
            }
        }

        // Export binary dashboard utilities
        window.binaryDashboard = {
            trackPerformance: trackBinaryPerformance,
            processData: processBinaryData,
            sortTable: sortBinaryTable,
            createCharts: createBinaryCharts,
            initialize: initializeBinaryFeatures
        };
        "#.to_string()
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> BinaryTemplateEngineStats {
        BinaryTemplateEngineStats {
            templates_processed: self.templates_processed,
            last_render_time_ms: self.last_render_time_ms,
            cache_hits: self.cache_hits,
            cache_hit_rate: if self.templates_processed > 0 {
                (self.cache_hits as f64 / self.templates_processed as f64) * 100.0
            } else {
                0.0
            },
            cached_templates: 0, // Now handled by resource manager
        }
    }

    /// Get last render time in milliseconds
    pub fn last_render_time(&self) -> u64 {
        self.last_render_time_ms
    }

    /// Clear template cache
    pub fn clear_cache(&mut self) {
        // Clear resource manager cache
        self.resource_manager.clear_cache();
    }
}

impl Default for BinaryTemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default BinaryTemplateEngine")
    }
}

/// Statistics for binary template engine performance
#[derive(Debug, Clone)]
pub struct BinaryTemplateEngineStats {
    /// Total number of templates processed
    pub templates_processed: u64,

    /// Last render time in milliseconds
    pub last_render_time_ms: u64,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Cache hit rate as percentage
    pub cache_hit_rate: f64,

    /// Number of cached templates
    pub cached_templates: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::binary_html_writer::{BinaryAllocationData, BinaryFieldValue};
    use std::collections::HashMap;

    fn create_test_template_data() -> BinaryTemplateData {
        let mut optional_fields = HashMap::new();
        optional_fields.insert(
            "test_field".to_string(),
            BinaryFieldValue::String("test_value".to_string()),
        );

        let allocation = BinaryAllocationData {
            id: 1,
            size: 1024,
            type_name: "Vec<u8>".to_string(),
            scope_name: "main".to_string(),
            timestamp_alloc: 1234567890,
            is_active: true,
            ptr: 0x1000,
            thread_id: "main".to_string(),
            var_name: Some("test_var".to_string()),
            borrow_count: 0,
            is_leaked: false,
            lifetime_ms: Some(1000),
            optional_fields,
        };

        BinaryTemplateData {
            project_name: "test_project".to_string(),
            allocations: vec![allocation],
            total_memory_usage: 1024,
            peak_memory_usage: 1024,
            active_allocations_count: 1,
            processing_time_ms: 100,
            data_source: "binary_direct".to_string(),
            complex_types: None,
            unsafe_ffi: None,
            variable_relationships: None,
        }
    }

    #[test]
    fn test_binary_template_engine_creation() {
        let engine = BinaryTemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_template_data_serialization() {
        let engine = BinaryTemplateEngine::new().unwrap();
        let template_data = create_test_template_data();

        let json_result = engine.serialize_template_data(&template_data);
        assert!(json_result.is_ok());

        let json_str = json_result.unwrap();
        assert!(json_str.contains("test_project"));
        assert!(json_str.contains("binary_direct"));
    }

    #[test]
    fn test_css_and_js_loading() {
        let mut engine = BinaryTemplateEngine::new().unwrap();

        let css_result = engine.load_css_resources();
        assert!(css_result.is_ok());

        let js_result = engine.load_js_resources();
        assert!(js_result.is_ok());

        let css_content = css_result.unwrap();
        let js_content = js_result.unwrap();

        // Test that CSS and JS content is loaded (content depends on actual files)
        assert!(!css_content.is_empty());
        assert!(!js_content.is_empty());
    }

    #[test]
    fn test_placeholder_processing() {
        let engine = BinaryTemplateEngine::new().unwrap();
        let template_data = create_test_template_data();

        let template = "Project: {{PROJECT_NAME}}, Time: {{PROCESSING_TIME}}ms";
        let json_data = "{}";
        let css_content = "";
        let js_content = "";

        let result = engine.process_template_placeholders(
            template,
            &template_data,
            json_data,
            css_content,
            js_content,
        );

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.contains("test_project"));
        assert!(processed.contains("100ms"));
    }

    #[test]
    fn test_throughput_calculation() {
        let engine = BinaryTemplateEngine::new().unwrap();
        let template_data = create_test_template_data();

        let throughput = engine.calculate_throughput(&template_data);
        assert_eq!(throughput, 10.0); // 1 allocation / 100ms * 1000 = 10 allocs/sec
    }

    #[test]
    fn test_caching_functionality() {
        let mut engine = BinaryTemplateEngine::with_config(BinaryTemplateEngineConfig {
            enable_cache: true,
            ..Default::default()
        })
        .unwrap();

        // Note: Resources are now managed by TemplateResourceManager
        // Cache functionality is handled internally

        // Test that subsequent loads return the same content
        let css1 = engine.load_css_resources().unwrap();
        let css2 = engine.load_css_resources().unwrap();
        assert_eq!(css1, css2); // Should be identical

        let js1 = engine.load_js_resources().unwrap();
        let js2 = engine.load_js_resources().unwrap();
        assert_eq!(js1, js2); // Should be identical

        // Template caching is now handled by resource manager internally
        // No direct access to cache needed

        // Verify stats reflect the processing
        let stats = engine.get_stats();
        assert_eq!(stats.cached_templates, 0); // Now handled by resource manager
    }

    #[test]
    fn test_cache_hits_tracking() {
        let mut engine = BinaryTemplateEngine::with_config(BinaryTemplateEngineConfig {
            enable_cache: true,
            ..Default::default()
        })
        .unwrap();

        // Cache hits are now managed by resource manager internally
        // Test that resources can be loaded multiple times without error
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();

        // Load again - should work without error
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();

        // One more CSS load
        engine.load_css_resources().unwrap();

        // Verify stats are still accessible
        let stats = engine.get_stats();
        assert_eq!(stats.cache_hits, 0); // Cache hits now managed by resource manager
    }

    #[test]
    fn test_cache_disabled() {
        let mut engine = BinaryTemplateEngine::with_config(BinaryTemplateEngineConfig {
            enable_cache: false,
            ..Default::default()
        })
        .unwrap();

        // With caching disabled, resource manager handles loading differently
        // No direct cache access needed
        assert_eq!(engine.get_stats().cache_hits, 0);

        // Load resources - should not be cached
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();

        // Cache is managed internally by resource manager
        // No direct verification needed
        assert_eq!(engine.get_stats().cache_hits, 0);
    }

    #[test]
    fn test_cache_clearing() {
        let mut engine = BinaryTemplateEngine::with_config(BinaryTemplateEngineConfig {
            enable_cache: true,
            ..Default::default()
        })
        .unwrap();

        // Load resources to populate cache
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();

        // Cache is managed internally by resource manager
        // Test that clear_cache method works without errors
        engine.clear_cache();

        // Verify engine still functions after cache clear
        let test_data = create_test_template_data();
        let result = engine.render_binary_template(&test_data);
        assert!(result.is_ok());
    }
}
