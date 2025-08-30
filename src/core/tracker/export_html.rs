//! HTML export functionality for memory tracking data.
//!
//! This module contains methods for exporting memory tracking data to HTML format,
//! including interactive dashboards with embedded visualizations.

use super::memory_tracker::MemoryTracker;
use crate::core::types::TrackingResult;
use std::path::Path;

impl MemoryTracker {
    /// Export interactive HTML dashboard with embedded SVG charts.
    ///
    /// This method creates a comprehensive HTML dashboard that includes:
    /// - Memory usage statistics and charts
    /// - Allocation timeline visualization
    /// - Type-based memory analysis
    /// - Interactive filtering and sorting
    /// - Embedded CSS and JavaScript for offline viewing
    ///
    /// # Arguments
    ///
    /// * `path` - The output path for the HTML file
    ///
    /// # Examples
    ///
    /// ```text
    /// // Export to default location
    /// tracker.export_interactive_dashboard("memory_report.html")?;
    ///
    /// // Export to specific directory
    /// tracker.export_interactive_dashboard("reports/detailed_analysis.html")?;
    /// ```
    ///
    /// # Features
    ///
    /// - **Self-contained**: All CSS, JavaScript, and data embedded in single HTML file
    /// - **Interactive**: Click, filter, and drill down into memory data
    /// - **Responsive**: Works on desktop and mobile browsers
    /// - **Offline**: No external dependencies, works without internet connection
    /// - **Comprehensive**: Includes all major memory analysis views
    ///
    /// # Performance
    ///
    /// HTML export is generally fast (1-3 seconds) as it focuses on visualization
    /// rather than comprehensive data processing. The file size depends on the
    /// amount of tracking data but is typically 1-10MB.
    pub fn export_interactive_dashboard<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);

        // Delegate to the specialized HTML export module
        crate::export::html_export::export_interactive_html(self, None, output_path)
    }

    /// Export HTML dashboard with custom unsafe FFI tracker data.
    ///
    /// This method allows including custom unsafe FFI analysis data in the HTML export,
    /// providing enhanced security and safety analysis in the dashboard.
    ///
    /// # Arguments
    ///
    /// * `path` - The output path for the HTML file
    /// * `unsafe_ffi_tracker` - Optional unsafe FFI tracker for enhanced analysis
    ///
    /// # Examples
    ///
    /// ```text
    /// use memscope_rs::get_global_unsafe_ffi_tracker;
    ///
    /// let unsafe_tracker = get_global_unsafe_ffi_tracker();
    /// tracker.export_interactive_dashboard_with_ffi(
    ///     "enhanced_report.html",
    ///     Some(&unsafe_tracker)
    /// )?;
    /// ```
    pub fn export_interactive_dashboard_with_ffi<P: AsRef<Path>>(
        &self,
        path: P,
        unsafe_ffi_tracker: Option<&crate::analysis::unsafe_ffi_tracker::UnsafeFFITracker>,
    ) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);

        // Delegate to the specialized HTML export module with FFI data
        crate::export::html_export::export_interactive_html(self, unsafe_ffi_tracker, output_path)
    }

    /// Generate HTML summary report with key metrics.
    ///
    /// This method creates a lightweight HTML summary that focuses on key metrics
    /// and insights rather than comprehensive data visualization.
    ///
    /// # Arguments
    ///
    /// * `path` - The output path for the HTML summary
    ///
    /// # Examples
    ///
    /// ```text
    /// // Generate quick summary report
    /// tracker.export_html_summary("summary.html")?;
    /// ```
    ///
    /// # Features
    ///
    /// - **Lightweight**: Smaller file size, faster generation
    /// - **Key metrics**: Focus on most important memory statistics
    /// - **Executive summary**: High-level insights and recommendations
    /// - **Quick loading**: Optimized for fast viewing and sharing
    pub fn export_html_summary<P: AsRef<Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(path);

        // Generate summary data
        let stats = self.get_stats()?;
        let memory_by_type = self.get_memory_by_type()?;
        let active_allocations = self.get_active_allocations()?;

        // Create summary HTML content
        let html_content =
            self.generate_summary_html(&stats, &memory_by_type, &active_allocations)?;

        // Write to file
        std::fs::write(&output_path, html_content)
            .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

        tracing::info!("📊 HTML summary exported to: {}", output_path.display());
        Ok(())
    }

    // Private helper methods

    /// Generate HTML content for summary report
    fn generate_summary_html(
        &self,
        stats: &crate::core::types::MemoryStats,
        memory_by_type: &[crate::core::types::TypeMemoryUsage],
        active_allocations: &[crate::core::types::AllocationInfo],
    ) -> TrackingResult<String> {
        // Calculate key metrics
        let total_types = memory_by_type.len();
        let avg_allocation_size = if stats.total_allocations > 0 {
            stats.total_allocated / stats.total_allocations as usize
        } else {
            0
        };

        let memory_efficiency = if stats.peak_memory > 0 {
            (stats.active_memory as f64 / stats.peak_memory as f64) * 100.0
        } else {
            100.0
        };

        // Find top memory consumers
        let mut top_types: Vec<_> = memory_by_type.iter().collect();
        top_types.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        let top_5_types: Vec<_> = top_types.into_iter().take(5).collect();

        // Generate HTML
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Memory Analysis Summary</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 8px 8px 0 0; }}
        .header h1 {{ margin: 0; font-size: 2.5em; font-weight: 300; }}
        .header p {{ margin: 10px 0 0 0; opacity: 0.9; }}
        .content {{ padding: 30px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .metric {{ background: #f8f9fa; padding: 20px; border-radius: 6px; border-left: 4px solid #667eea; }}
        .metric h3 {{ margin: 0 0 10px 0; color: #333; font-size: 0.9em; text-transform: uppercase; letter-spacing: 1px; }}
        .metric .value {{ font-size: 2em; font-weight: bold; color: #667eea; }}
        .metric .unit {{ font-size: 0.8em; color: #666; }}
        .section {{ margin-bottom: 30px; }}
        .section h2 {{ color: #333; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        .type-list {{ list-style: none; padding: 0; }}
        .type-item {{ display: flex; justify-content: space-between; align-items: center; padding: 10px; border-bottom: 1px solid #eee; }}
        .type-name {{ font-weight: 500; }}
        .type-size {{ color: #667eea; font-weight: bold; }}
        .recommendations {{ background: #e8f5e8; border: 1px solid #c3e6c3; border-radius: 6px; padding: 20px; }}
        .recommendations h3 {{ color: #2d5a2d; margin-top: 0; }}
        .recommendations ul {{ margin: 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Memory Analysis Summary</h1>
            <p>Generated on {}</p>
        </div>
        
        <div class="content">
            <div class="metrics">
                <div class="metric">
                    <h3>Total Memory</h3>
                    <div class="value">{}</div>
                    <div class="unit">bytes allocated</div>
                </div>
                <div class="metric">
                    <h3>Active Memory</h3>
                    <div class="value">{}</div>
                    <div class="unit">bytes in use</div>
                </div>
                <div class="metric">
                    <h3>Peak Memory</h3>
                    <div class="value">{}</div>
                    <div class="unit">bytes maximum</div>
                </div>
                <div class="metric">
                    <h3>Memory Efficiency</h3>
                    <div class="value">{:.1}%</div>
                    <div class="unit">active/peak ratio</div>
                </div>
                <div class="metric">
                    <h3>Total Allocations</h3>
                    <div class="value">{}</div>
                    <div class="unit">allocation calls</div>
                </div>
                <div class="metric">
                    <h3>Active Allocations</h3>
                    <div class="value">{}</div>
                    <div class="unit">currently active</div>
                </div>
                <div class="metric">
                    <h3>Unique Types</h3>
                    <div class="value">{}</div>
                    <div class="unit">different types</div>
                </div>
                <div class="metric">
                    <h3>Avg Allocation</h3>
                    <div class="value">{}</div>
                    <div class="unit">bytes average</div>
                </div>
            </div>

            <div class="section">
                <h2>Top Memory Consumers</h2>
                <ul class="type-list">
                    {}
                </ul>
            </div>

            <div class="recommendations">
                <h3>💡 Optimization Recommendations</h3>
                <ul>
                    {}
                </ul>
            </div>
        </div>

        <div class="footer">
            Generated by memscope-rs v{} • {} active allocations analyzed
        </div>
    </div>
</body>
</html>"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.format_bytes(stats.total_allocated),
            self.format_bytes(stats.active_memory),
            self.format_bytes(stats.peak_memory),
            memory_efficiency,
            stats.total_allocations,
            stats.active_allocations,
            total_types,
            self.format_bytes(avg_allocation_size),
            top_5_types.iter().map(|t| format!(
                r#"<li class="type-item"><span class="type-name">{}</span><span class="type-size">{}</span></li>"#,
                t.type_name,
                self.format_bytes(t.total_size)
            )).collect::<Vec<_>>().join(""),
            self.generate_recommendations_html(stats, memory_by_type),
            env!("CARGO_PKG_VERSION"),
            active_allocations.len()
        );

        Ok(html)
    }

    /// Format bytes in human-readable format
    fn format_bytes(&self, bytes: usize) -> String {
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

    /// Generate recommendations HTML
    fn generate_recommendations_html(
        &self,
        stats: &crate::core::types::MemoryStats,
        memory_by_type: &[crate::core::types::TypeMemoryUsage],
    ) -> String {
        let recommendations =
            super::export_json::generate_optimization_recommendations(stats, memory_by_type);

        recommendations
            .iter()
            .map(|rec| format!("<li>{rec}</li>"))
            .collect::<Vec<_>>()
            .join("")
    }
}
