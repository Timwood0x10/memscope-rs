//! Binary template engine for processing binary-specific HTML templates
//!
//! This module provides a specialized template engine that processes the binary_dashboard.html
//! template with data directly from binary sources, independent of the JSON → HTML workflow.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::binary_html_writer::BinaryTemplateData;

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
    /// Template cache for performance
    template_cache: HashMap<String, String>,

    /// CSS content cache
    css_cache: Option<String>,

    /// JavaScript content cache
    js_cache: Option<String>,

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
        let mut engine = Self {
            template_cache: HashMap::new(),
            css_cache: None,
            js_cache: None,
            config,
            last_render_time_ms: 0,
            templates_processed: 0,
            cache_hits: 0,
        };

        // Preload resources if caching is enabled
        if engine.config.enable_cache {
            engine.preload_resources()?;
        }

        Ok(engine)
    }

    /// Render the binary dashboard template with the provided data
    pub fn render_binary_template(
        &mut self,
        template_data: &BinaryTemplateData,
    ) -> Result<String, BinaryExportError> {
        let render_start = Instant::now();

        // Load the binary dashboard template
        let template_content = self.load_binary_template()?;

        // Load CSS and JS resources
        let css_content = self.load_css_resources()?;
        let js_content = self.load_js_resources()?;

        // Convert template data to JSON for injection
        let json_data = self.serialize_template_data(template_data)?;

        // Process template placeholders
        let html_content = self.process_template_placeholders(
            &template_content,
            template_data,
            &json_data,
            &css_content,
            &js_content,
        )?;

        // Update statistics
        self.last_render_time_ms = render_start.elapsed().as_millis() as u64;
        self.templates_processed += 1;

        Ok(html_content)
    }

    /// Load the binary dashboard template
    fn load_binary_template(&mut self) -> Result<String, BinaryExportError> {
        let template_path = "templates/binary_dashboard.html";

        // Check cache first
        if self.config.enable_cache {
            if let Some(cached_template) = self.template_cache.get(template_path) {
                self.cache_hits += 1;
                return Ok(cached_template.clone());
            }
        }

        // Load from file
        let template_content = fs::read_to_string(template_path)
            .map_err(|e| BinaryExportError::Io(e))?;

        // Cache the template if caching is enabled
        if self.config.enable_cache {
            self.template_cache.insert(template_path.to_string(), template_content.clone());
        }

        Ok(template_content)
    }

    /// Load CSS resources for the template
    fn load_css_resources(&mut self) -> Result<String, BinaryExportError> {
        // Check cache first
        if self.config.enable_cache {
            if let Some(ref cached_css) = self.css_cache {
                self.cache_hits += 1;
                return Ok(cached_css.clone());
            }
        }

        // Load CSS content (for now, use embedded styles)
        let css_content = self.get_embedded_css();

        // Cache the CSS if caching is enabled
        if self.config.enable_cache {
            self.css_cache = Some(css_content.clone());
        }

        Ok(css_content)
    }

    /// Load JavaScript resources for the template
    fn load_js_resources(&mut self) -> Result<String, BinaryExportError> {
        // Check cache first
        if self.config.enable_cache {
            if let Some(ref cached_js) = self.js_cache {
                self.cache_hits += 1;
                return Ok(cached_js.clone());
            }
        }

        // Load JS content (for now, use embedded scripts)
        let js_content = self.get_embedded_js();

        // Cache the JS if caching is enabled
        if self.config.enable_cache {
            self.js_cache = Some(js_content.clone());
        }

        Ok(js_content)
    }

    /// Serialize template data to JSON format
    fn serialize_template_data(&self, data: &BinaryTemplateData) -> Result<String, BinaryExportError> {
        use serde_json::json;

        // Convert binary template data to JSON format compatible with the template
        let allocations_json: Vec<serde_json::Value> = data.allocations.iter().map(|alloc| {
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
        }).collect();

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

        serde_json::to_string(&dashboard_data)
            .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))
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
            &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
        );
        html_content = html_content.replace(
            "{{PROCESSING_TIME}}", 
            &template_data.processing_time_ms.to_string()
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
        "#.to_string()
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
            cached_templates: self.template_cache.len(),
        }
    }

    /// Get last render time in milliseconds
    pub fn last_render_time(&self) -> u64 {
        self.last_render_time_ms
    }

    /// Clear template cache
    pub fn clear_cache(&mut self) {
        self.template_cache.clear();
        self.css_cache = None;
        self.js_cache = None;
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
        optional_fields.insert("test_field".to_string(), BinaryFieldValue::String("test_value".to_string()));

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
        
        assert!(css_content.contains("binary-performance-indicator"));
        assert!(js_content.contains("trackBinaryPerformance"));
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
        let mut engine = BinaryTemplateEngine::with_config(
            BinaryTemplateEngineConfig {
                enable_cache: true,
                ..Default::default()
            }
        ).unwrap();
        
        // Note: Resources are preloaded in constructor when caching is enabled
        // So cache should already be populated
        assert!(engine.css_cache.is_some()); // Preloaded
        assert!(engine.js_cache.is_some()); // Preloaded
        
        // Test that subsequent loads return the same content
        let css1 = engine.load_css_resources().unwrap();
        let css2 = engine.load_css_resources().unwrap();
        assert_eq!(css1, css2); // Should be identical
        
        let js1 = engine.load_js_resources().unwrap();
        let js2 = engine.load_js_resources().unwrap();
        assert_eq!(js1, js2); // Should be identical
        
        // Test template caching by manually adding to cache
        let initial_cache_size = engine.template_cache.len();
        engine.template_cache.insert("test_template".to_string(), "test_content".to_string());
        assert_eq!(engine.template_cache.len(), initial_cache_size + 1);
        
        // Verify stats reflect the cache state
        let stats = engine.get_stats();
        assert_eq!(stats.cached_templates, initial_cache_size + 1);
    }

    #[test]
    fn test_cache_hits_tracking() {
        let mut engine = BinaryTemplateEngine::with_config(
            BinaryTemplateEngineConfig {
                enable_cache: true,
                ..Default::default()
            }
        ).unwrap();
        
        // Resources are preloaded in constructor, so cache_hits might already be > 0
        let initial_cache_hits = engine.get_stats().cache_hits;
        
        // Load resources - these should be cache hits since they're preloaded
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();
        assert_eq!(engine.get_stats().cache_hits, initial_cache_hits + 2);
        
        // Load again - more cache hits
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();
        assert_eq!(engine.get_stats().cache_hits, initial_cache_hits + 4);
        
        // One more CSS load
        engine.load_css_resources().unwrap();
        assert_eq!(engine.get_stats().cache_hits, initial_cache_hits + 5);
    }

    #[test]
    fn test_cache_disabled() {
        let mut engine = BinaryTemplateEngine::with_config(
            BinaryTemplateEngineConfig {
                enable_cache: false,
                ..Default::default()
            }
        ).unwrap();
        
        // With caching disabled, resources should not be preloaded
        assert!(engine.css_cache.is_none());
        assert!(engine.js_cache.is_none());
        assert_eq!(engine.get_stats().cache_hits, 0);
        
        // Load resources - should not be cached
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();
        
        // Cache should still be empty
        assert!(engine.css_cache.is_none());
        assert!(engine.js_cache.is_none());
        assert_eq!(engine.get_stats().cache_hits, 0);
    }

    #[test]
    fn test_cache_clearing() {
        let mut engine = BinaryTemplateEngine::with_config(
            BinaryTemplateEngineConfig {
                enable_cache: true,
                ..Default::default()
            }
        ).unwrap();
        
        // Load resources to populate cache
        engine.load_css_resources().unwrap();
        engine.load_js_resources().unwrap();
        
        // Verify cache is populated
        assert!(engine.css_cache.is_some());
        assert!(engine.js_cache.is_some());
        
        // Clear cache
        engine.clear_cache();
        
        // Verify cache is cleared
        assert!(engine.template_cache.is_empty());
        assert!(engine.css_cache.is_none());
        assert!(engine.js_cache.is_none());
    }
}