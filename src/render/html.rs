//! HTML rendering implementation
//!
//! Provides HTML rendering for all tracking strategies with
//! strategy-specific dashboards using templates from ./templates/.

use super::renderer::Renderer;
use crate::data::{RenderOutput, RenderResult, TrackingSnapshot, TrackingStrategy};
use crate::error::types::{ErrorKind, MemScopeError};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// HTML renderer
///
/// Generates strategy-specific HTML dashboards for memory tracking data
/// using templates from the templates directory.
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        Self
    }

    /// Load template from templates directory
    ///
    /// Template path resolution order:
    /// 1. Environment variable `MEMSCOPE_TEMPLATES_DIR` if set
    /// 2. `templates/` relative to current working directory
    /// 3. `./templates/` relative to the executable
    fn load_template(&self, template_name: &str) -> RenderResult<String> {
        // Try environment variable first
        if let Ok(custom_dir) = std::env::var("MEMSCOPE_TEMPLATES_DIR") {
            let template_path = std::path::PathBuf::from(custom_dir).join(template_name);
            if let Ok(content) = std::fs::read_to_string(&template_path) {
                return Ok(content);
            }
        }

        // Try current working directory
        if let Ok(cwd) = std::env::current_dir() {
            let template_path = cwd.join("templates").join(template_name);
            if let Ok(content) = std::fs::read_to_string(&template_path) {
                return Ok(content);
            }
        }

        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let template_path = exe_dir.join("templates").join(template_name);
                if let Ok(content) = std::fs::read_to_string(&template_path) {
                    return Ok(content);
                }
            }
        }

        // If all attempts failed, return a detailed error
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let template_path = cwd.join("templates").join(template_name);
        
        let mut error_msg = format!("Failed to load template '{}'. Tried:\n", template_name);
        error_msg.push_str(&format!("  1. Environment variable MEMSCOPE_TEMPLATES_DIR\n"));
        error_msg.push_str(&format!("  2. {}/templates/\n", cwd.display()));
        
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                error_msg.push_str(&format!("  3. {}/templates/\n", exe_dir.display()));
            }
        }
        
        error_msg.push_str(&format!("\nExpected path: {}", template_path.display()));
        
        Err(MemScopeError::new(ErrorKind::InternalError, &error_msg))
    }

    /// Common template rendering logic
    ///
    /// This function handles the common pattern of:
    /// 1. Loading a template
    /// 2. Preparing analysis data
    /// 3. Replacing placeholders
    /// 4. Injecting data script
    /// 5. Generating error page on failure (instead of complex fallback)
    fn render_with_template(
        &self,
        template_name: &str,
        strategy_name: &str,
        snapshot: &TrackingSnapshot,
    ) -> String {
        // Load template
        let template = match self.load_template(template_name) {
            Ok(t) => t,
            Err(e) => {
                return self.generate_error_html(
                    strategy_name,
                    &format!("Failed to load template '{}': {}", template_name, e),
                    snapshot,
                );
            }
        };

        // Prepare data for template
        let analysis_data = match self.prepare_analysis_data(snapshot) {
            Ok(data) => data,
            Err(e) => {
                return self.generate_error_html(
                    strategy_name,
                    &format!("Failed to prepare analysis data: {}", e),
                    snapshot,
                );
            }
        };

        // Replace placeholders with actual data
        let mut html = template.clone();
        html = html.replace("{{BINARY_DATA}}", &analysis_data);
        html = self.replace_placeholders(&html, snapshot);

        // Ensure data injection
        if !html.contains("window.analysisData") {
            html = self.inject_analysis_data_script(&html, &analysis_data);
        }

        html
    }

    /// Prepare analysis data in the format expected by the templates
    ///
    /// This converts TrackingSnapshot to the complex JSON structure that
    /// the JavaScript in the templates expects.
    fn prepare_analysis_data(&self, snapshot: &TrackingSnapshot) -> Result<String, MemScopeError> {
        // Convert allocations to the expected format
        let allocations: Vec<Value> = snapshot
            .allocations
            .iter()
            .map(|alloc| {
                let lifetime_ms = alloc.lifetime_ms().unwrap_or(0) as f64;
                let timestamp_dealloc = alloc.dealloc_timestamp.map(|t| t as i64);
                
                // Handle borrow_info
                let borrow_info_json = if let Some(ref borrow_info) = alloc.borrow_info {
                    json!({
                        "immutable_borrows": borrow_info.immutable_borrows,
                        "mutable_borrows": borrow_info.mutable_borrows,
                        "max_concurrent_borrows": borrow_info.max_concurrent_borrows,
                        "last_borrow_timestamp": borrow_info.last_borrow_timestamp
                    })
                } else {
                    Value::Null
                };

                // Handle clone_info
                let clone_info_json = if let Some(ref clone_info) = alloc.clone_info {
                    json!({
                        "clone_count": clone_info.clone_count,
                        "is_clone": clone_info.is_clone,
                        "original_ptr": clone_info.original_ptr.map(|p| format!("0x{:x}", p))
                    })
                } else {
                    Value::Null
                };

                json!({
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "size": alloc.size,
                    "var_name": alloc.var_name.as_deref().unwrap_or("unknown"),
                    "type_name": alloc.type_name.as_deref().unwrap_or("unknown"),
                    "thread_id": alloc.thread_id,
                    "timestamp_alloc": alloc.timestamp as i64,
                    "timestamp_dealloc": timestamp_dealloc,
                    "is_leaked": !alloc.is_active && timestamp_dealloc.is_none(),
                    "is_active": alloc.is_active,
                    "lifetime_ms": lifetime_ms,
                    "borrow_count": alloc.borrow_info.as_ref()
                        .map(|b| b.immutable_borrows + b.mutable_borrows)
                        .unwrap_or(0),
                    "borrow_info": borrow_info_json,
                    "clone_info": clone_info_json,
                    "ownership_history_available": alloc.ownership_history_available
                })
            })
            .collect();

        // Generate enhanced data sections
        let (lifetime_data, complex_types, unsafe_ffi, performance_data) =
            self.generate_enhanced_data(&allocations);

        // Build the complete data structure expected by templates
        let data_structure = json!({
            "memory_analysis": {
                "allocations": allocations.clone()
            },
            "allocations": allocations, // Direct access for compatibility
            "lifetime": lifetime_data,
            "complex_types": complex_types,
            "unsafe_ffi": unsafe_ffi,
            "performance": performance_data,
            "metadata": {
                "generation_time": format_timestamp(snapshot.timestamp),
                "data_source": "unified_tracker",
                "version": "2.0",
                "strategy": format!("{:?}", snapshot.strategy)
            }
        });

        serde_json::to_string(&data_structure).map_err(|e| {
            MemScopeError::new(ErrorKind::InternalError, &e.to_string())
        })
    }

    /// Generate enhanced data sections for the dashboard
    fn generate_enhanced_data(&self, allocations: &[serde_json::Value]) -> (Value, Value, Value, Value) {
        // Generate lifetime data
        let lifetime_data = self.generate_lifetime_data(allocations);

        // Generate complex types data
        let complex_types = self.generate_complex_types_data(allocations);

        // Generate unsafe FFI data
        let unsafe_ffi = self.generate_unsafe_ffi_data(allocations);

        // Generate performance data
        let performance = self.generate_performance_data(allocations);

        (lifetime_data, complex_types, unsafe_ffi, performance)
    }

    /// Generate lifetime analysis data
    fn generate_lifetime_data(&self, allocations: &[serde_json::Value]) -> Value {
        let mut total_lifetime = 0.0;
        let mut allocation_count = 0;
        let mut lifetime_buckets = HashMap::new();

        for alloc in allocations {
            if let Some(lifetime_ms) = alloc.get("lifetime_ms").and_then(|v| v.as_f64()) {
                total_lifetime += lifetime_ms;
                allocation_count += 1;

                // Group by lifetime buckets
                let bucket = self.get_lifetime_bucket(lifetime_ms);
                *lifetime_buckets.entry(bucket).or_insert(0) += 1;
            }
        }

        let avg_lifetime = if allocation_count > 0 {
            total_lifetime / allocation_count as f64
        } else {
            0.0
        };

        json!({
            "total_lifetime_avg": avg_lifetime,
            "allocation_count": allocation_count,
            "lifetime_distribution": lifetime_buckets
        })
    }

    /// Get lifetime bucket for categorization
    fn get_lifetime_bucket(&self, lifetime_ms: f64) -> String {
        match lifetime_ms {
            t if t < 1.0 => "sub-millisecond".to_string(),
            t if t < 10.0 => "1-10ms".to_string(),
            t if t < 100.0 => "10-100ms".to_string(),
            t if t < 1000.0 => "100ms-1s".to_string(),
            t if t < 10000.0 => "1s-10s".to_string(),
            _ => "10s+".to_string(),
        }
    }

    /// Generate complex types analysis data
    fn generate_complex_types_data(&self, allocations: &[serde_json::Value]) -> Value {
        let mut type_distribution: HashMap<String, usize> = HashMap::new();
        let mut smart_pointer_usage = HashMap::new();

        for alloc in allocations {
            if let Some(type_name) = alloc.get("type_name").and_then(|v| v.as_str()) {
                *type_distribution.entry(type_name.to_string()).or_insert(0) += 1;

                // Detect smart pointers
                if type_name.contains("Arc<") || type_name.contains("Rc<") {
                    *smart_pointer_usage.entry("reference_counted".to_string()).or_insert(0) += 1;
                }
                if type_name.contains("Box<") {
                    *smart_pointer_usage.entry("boxed".to_string()).or_insert(0) += 1;
                }
                if type_name.contains("Vec<") || type_name.contains("HashMap<") {
                    *smart_pointer_usage.entry("collection".to_string()).or_insert(0) += 1;
                }
            }
        }

        json!({
            "type_distribution": type_distribution,
            "smart_pointer_usage": smart_pointer_usage
        })
    }

    /// Generate unsafe FFI data
    fn generate_unsafe_ffi_data(&self, allocations: &[serde_json::Value]) -> Value {
        // Count allocations that might be from unsafe operations
        let unsafe_allocations = allocations
            .iter()
            .filter(|alloc| {
                alloc.get("type_name")
                    .and_then(|v| v.as_str())
                    .map(|t| t.contains("CString") || t.contains("CStr") || t.contains("*mut") || t.contains("*const"))
                    .unwrap_or(false)
            })
            .count();

        json!({
            "unsafe_allocations": unsafe_allocations,
            "ffi_calls": 0, // Would need additional tracking
            "safety_violations": []
        })
    }

    /// Generate performance metrics data
    fn generate_performance_data(&self, allocations: &[serde_json::Value]) -> Value {
        let allocation_count = allocations.len() as u64;
        let total_memory: u64 = allocations
            .iter()
            .filter_map(|alloc| alloc.get("size").and_then(|v| v.as_u64()))
            .sum();

        // Calculate allocation rate (simplified)
        let allocation_rate = if allocation_count > 0 {
            allocation_count // Rate per second would need timestamp data
        } else {
            0
        };

        json!({
            "allocation_rate": allocation_rate,
            "deallocation_rate": 0, // Would need deallocation tracking
            "peak_concurrency": allocation_count,
            "efficiency_score": 100 // Placeholder calculation
        })
    }

    /// Replace placeholders in template with actual data
    fn replace_placeholders(&self, template: &str, snapshot: &TrackingSnapshot) -> String {
        let allocations_json = serde_json::to_string(&snapshot.allocations).unwrap_or_default();
        let events_json = serde_json::to_string(&snapshot.events).unwrap_or_default();
        let tasks_json = serde_json::to_string(&snapshot.tasks).unwrap_or_default();
        let stats_json = serde_json::to_string(&snapshot.stats).unwrap_or_default();
        let snapshot_json = serde_json::to_string(snapshot).unwrap_or_default();

        let mut html = template.to_string();

        // Replace basic placeholders
        html = html.replace("{{TITLE}}", &format!("Memory Analysis - {:?}", snapshot.strategy));
        html = html.replace("{{title}}", &format!("Memory Analysis - {:?}", snapshot.strategy));
        html = html.replace("{{timestamp}}", &format_timestamp(snapshot.timestamp));
        html = html.replace("{{strategy}}", &format!("{:?}", snapshot.strategy));
        html = html.replace("{{STRATEGY}}", &format!("{:?}", snapshot.strategy));

        // Replace JSON data placeholders
        html = html.replace("{{allocations_json}}", &allocations_json);
        html = html.replace("{{ALLOCATIONS_DATA}}", &allocations_json);
        html = html.replace("{{events_json}}", &events_json);
        html = html.replace("{{EVENTS_DATA}}", &events_json);
        html = html.replace("{{tasks_json}}", &tasks_json);
        html = html.replace("{{TASKS_DATA}}", &tasks_json);
        html = html.replace("{{stats_json}}", &stats_json);
        html = html.replace("{{STATS_DATA}}", &stats_json);
        html = html.replace("{{snapshot_json}}", &snapshot_json);

        // Replace statistics placeholders
        html = html.replace("{{total_allocations}}", &snapshot.stats.total_allocations.to_string());
        html = html.replace("{{TOTAL_ALLOCATIONS}}", &snapshot.stats.total_allocations.to_string());
        html = html.replace("{{total_deallocations}}", &snapshot.stats.total_deallocations.to_string());
        html = html.replace("{{TOTAL_DEALLOCATIONS}}", &snapshot.stats.total_deallocations.to_string());
        html = html.replace("{{peak_memory}}", &format_bytes(snapshot.stats.peak_memory as usize));
        html = html.replace("{{PEAK_MEMORY}}", &format_bytes(snapshot.stats.peak_memory as usize));
        html = html.replace("{{active_memory}}", &format_bytes(snapshot.stats.active_memory as usize));
        html = html.replace("{{ACTIVE_MEMORY}}", &format_bytes(snapshot.stats.active_memory as usize));
        html = html.replace("{{fragmentation}}", &format!("{:.2}", snapshot.stats.fragmentation));
        html = html.replace("{{FRAGMENTATION}}", &format!("{:.2}", snapshot.stats.fragmentation));

        // Replace memory placeholders (in MB)
        html = html.replace("{{TOTAL_MEMORY}}", &format!("{:.1}MB", snapshot.stats.peak_memory as f64 / 1024.0 / 1024.0));
        html = html.replace("{{totalMemory}}", &format!("{:.1}MB", snapshot.stats.peak_memory as f64 / 1024.0 / 1024.0));
        html = html.replace("{{CURRENT_MEMORY}}", &format!("{:.1}MB", snapshot.stats.active_memory as f64 / 1024.0 / 1024.0));

        // Replace allocation count placeholders
        html = html.replace("{{TOTAL_VARIABLES}}", &snapshot.allocations.len().to_string());
        html = html.replace("{{totalVariables}}", &snapshot.allocations.len().to_string());

        // Replace thread count placeholder (for lockfree strategy)
        html = html.replace("{{THREAD_COUNT}}", &format!("{}", snapshot.allocations.len()));
        html = html.replace("{{threadCount}}", &format!("{}", snapshot.allocations.len()));

        // Replace efficiency placeholder
        html = html.replace("{{EFFICIENCY}}", &format!("{:.1}", 100.0 * (1.0 - snapshot.stats.fragmentation)));
        html = html.replace("{{efficiency}}", &format!("{:.1}", 100.0 * (1.0 - snapshot.stats.fragmentation)));

        html
    }

    /// Replace fallback placeholders for compatibility
    fn replace_fallback_placeholders(&self, html: &mut String, snapshot: &TrackingSnapshot) {
        // Handle various placeholder variations
        let project_name = String::from("MemScope");
        let generation_time = format_timestamp(snapshot.timestamp);
        let title = format!("Memory Analysis - {:?}", snapshot.strategy);
        
        let placeholders = [
            ("{{BINARY_DATA}}", &self.prepare_analysis_data(snapshot).unwrap_or_default()),
            ("{{ json_data }}", &self.prepare_analysis_data(snapshot).unwrap_or_default()),
            ("{{json_data}}", &self.prepare_analysis_data(snapshot).unwrap_or_default()),
            ("{{ALLOCATION_DATA}}", &self.prepare_allocation_json(snapshot)),
            ("{{PROJECT_NAME}}", &project_name),
            ("{{TITLE}}", &title),
            ("{{GENERATION_TIME}}", &generation_time),
        ];

        for (placeholder, value) in placeholders {
            *html = html.replace(placeholder, value);
        }

        // Handle hardcoded window.analysisData assignments
        if let Some(start) = html.find("window.analysisData = {") {
            if let Some(end) = html[start..].find("};") {
                let end_pos = start + end + 2;
                let analysis_data = self.prepare_analysis_data(snapshot).unwrap_or_default();
                let before = &html[..start];
                let after = &html[end_pos..];
                *html = format!("{}window.analysisData = {};{}", before, analysis_data, after);
            }
        }
    }

    /// Inject analysis data script into HTML
    fn inject_analysis_data_script(&self, html: &str, data: &str) -> String {
        let script = format!(
            r#"
    <script>
    window.analysisData = {};
    console.log('Data injection successful:', window.analysisData);
    </script>
    "#,
            data
        );

        if let Some(head_end) = html.find("</head>") {
            format!("{}{}{}", &html[..head_end], script, &html[head_end..])
        } else {
            format!("{}{}", html, script)
        }
    }

    /// Prepare allocation JSON data
    fn prepare_allocation_json(&self, snapshot: &TrackingSnapshot) -> String {
        serde_json::to_string(&snapshot.allocations).unwrap_or_default()
    }

    /// Generate Core strategy HTML
    fn render_core(&self, snapshot: &TrackingSnapshot) -> String {
        self.render_with_template(
            "clean_dashboard.html",
            "Core",
            snapshot,
        )
    }

    /// Generate simple error page (when template loading fails)
    fn generate_error_html(&self, strategy_name: &str, error_msg: &str, snapshot: &TrackingSnapshot) -> String {
        let timestamp = format_timestamp(snapshot.timestamp);
        let data_json = serde_json::to_string(&snapshot).unwrap_or_default();
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Memory Tracking Error - {}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .error-container {{
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{ color: #e74c3c; }}
        .error-details {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 4px;
            margin: 20px 0;
            font-family: monospace;
            white-space: pre-wrap;
        }}
        .stats-summary {{
            margin-top: 20px;
            padding: 15px;
            background: #e8f5e9;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <h1>⚠️ Template Loading Failed</h1>
        <p><strong>Strategy:</strong> {}</p>
        <p><strong>Timestamp:</strong> {}</p>
        
        <div class="error-details">
            <strong>Error:</strong>
            {}
        </div>
        
        <div class="stats-summary">
            <h3>Statistics:</h3>
            <ul>
                <li>Total allocations: {}</li>
                <li>Total deallocations: {}</li>
                <li>Peak memory: {} bytes</li>
                <li>Active memory: {} bytes</li>
            </ul>
        </div>
        
        <p style="margin-top: 20px; font-size: 0.9em; color: #666;">
            Raw data is available below for manual inspection.
        </p>
        
        <div class="error-details">
            <h3>Raw Data (JSON):</h3>
            <pre>{}</pre>
        </div>
    </div>
</body>
</html>"#,
            strategy_name,
            strategy_name,
            timestamp,
            error_msg,
            snapshot.stats.total_allocations,
            snapshot.stats.total_deallocations,
            snapshot.stats.peak_memory,
            snapshot.stats.active_memory,
            data_json
        )
    }

    /// Get strategy name for error messages
    fn get_strategy_name(&self, strategy: TrackingStrategy) -> &'static str {
        match strategy {
            TrackingStrategy::Core => "Core",
            TrackingStrategy::Lockfree => "Lockfree",
            TrackingStrategy::Async => "Async",
            TrackingStrategy::Unified => "Unified",
        }
    }

    /// Generate Lockfree strategy HTML
    fn render_lockfree(&self, snapshot: &TrackingSnapshot) -> String {
        self.render_with_template(
            "multithread_template.html",
            "Lockfree",
            snapshot,
        )
    }

    /// Generate Async strategy HTML
    fn render_async(&self, snapshot: &TrackingSnapshot) -> String {
        self.render_with_template(
            "async_template.html",
            "Async",
            snapshot,
        )
    }

    /// Generate Unified strategy HTML
    fn render_unified(&self, snapshot: &TrackingSnapshot) -> String {
        self.render_with_template(
            "hybrid_dashboard.html",
            "Unified",
            snapshot,
        )
    }
}

impl Renderer for HtmlRenderer {
    fn format(&self) -> crate::data::ExportFormat {
        crate::data::ExportFormat::Html
    }

    fn render(&self, snapshot: &TrackingSnapshot) -> RenderResult<RenderOutput> {
        let html = match snapshot.strategy {
            TrackingStrategy::Core => self.render_core(snapshot),
            TrackingStrategy::Lockfree => self.render_lockfree(snapshot),
            TrackingStrategy::Async => self.render_async(snapshot),
            TrackingStrategy::Unified => self.render_unified(snapshot),
        };
        Ok(RenderOutput::String(html))
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format timestamp to human-readable string
fn format_timestamp(timestamp: u64) -> String {
    let secs = timestamp / 1_000_000;
    let millis = (timestamp % 1_000_000) / 1_000;
    let micros = timestamp % 1_000;
    format!("{}.{:03}.{:03}s", secs, millis, micros)
}

/// Format bytes to human-readable string
fn format_bytes(bytes: usize) -> String {
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
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{TrackingStats, TrackingStrategy};

    #[test]
    fn test_html_renderer_creation() {
        let renderer = HtmlRenderer::new();
        assert_eq!(renderer.format(), crate::data::ExportFormat::Html);
    }

    #[test]
    fn test_html_render_core() {
        let renderer = HtmlRenderer::new();
        let snapshot = TrackingSnapshot {
            strategy: TrackingStrategy::Core,
            allocations: vec![],
            events: vec![],
            tasks: vec![],
            stats: TrackingStats::default(),
            timestamp: 0,
        };

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());
        let html = result.unwrap();
        if let RenderOutput::String(content) = html {
            assert!(content.contains("MemScope Memory Analysis Dashboard"));
            assert!(content.contains("<!DOCTYPE html>") || content.contains("<html"));
        } else {
            panic!("Expected String output");
        }
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "0.000.000s");
        assert_eq!(format_timestamp(1_000_000), "1.000.000s");
        assert_eq!(format_timestamp(1_500_500), "1.500.500s");
    }
}
