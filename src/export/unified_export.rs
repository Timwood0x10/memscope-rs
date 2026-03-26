//! Unified export system
//!
//! This module provides a unified export interface that consolidates the 28 existing
//! export modules into 5 core exporters while preserving all functionality.

use crate::types::internal_types::{Snapshot, TrackingError};
use serde::{Deserialize, Serialize};
use std::io::Write;

// ============================================================================
// Export Configuration
// ============================================================================

/// Export format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// HTML format
    Html,
    /// CSV format
    Csv,
    /// Custom format
    Custom,
}

/// Export configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,
    /// Include analysis results
    pub include_analysis: bool,
    /// Include 3D visualization
    pub include_3d: bool,
    /// Include timeline data
    pub include_timeline: bool,
    /// Optimization level
    pub optimization: ExportOptimization,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Html,
            include_analysis: true,
            include_3d: false,
            include_timeline: false,
            optimization: ExportOptimization::default(),
        }
    }
}

/// Export optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOptimization {
    /// No optimization
    None,
    /// Size optimization (smaller output)
    Size,
    /// Speed optimization (faster export)
    Speed,
    /// Balanced optimization
    Balanced,
}

impl Default for ExportOptimization {
    fn default() -> Self {
        Self::Balanced
    }
}

// ============================================================================
// Export Result Types
// ============================================================================

/// Export output
#[derive(Debug, Clone)]
pub enum ExportOutput {
    /// String output
    String(String),
    /// Binary output
    Binary(Vec<u8>),
    /// File output
    File(std::path::PathBuf),
}

/// Export error type
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

/// Export result type
pub type ExportResult<T> = Result<T, ExportError>;

// ============================================================================
// Export Backend Trait
// ============================================================================

/// Core export backend trait
pub trait ExportBackend: Send + Sync {
    /// Get the name of this exporter
    fn name(&self) -> &str;

    /// Export snapshot to output
    fn export(&self, snapshot: &Snapshot, config: &ExportConfig) -> ExportResult<ExportOutput>;

    /// Check if this exporter supports the given format
    fn supports_format(&self, format: ExportFormat) -> bool {
        match format {
            ExportFormat::Custom => false,
            _ => true,
        }
    }
}

// ============================================================================
// JSON Exporter
// ============================================================================

/// JSON exporter
pub struct JsonExporter;

impl JsonExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportBackend for JsonExporter {
    fn name(&self) -> &str {
        "JSON Exporter"
    }

    fn export(&self, snapshot: &Snapshot, _config: &ExportConfig) -> ExportResult<ExportOutput> {
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        Ok(ExportOutput::String(json))
    }

    fn supports_format(&self, format: ExportFormat) -> bool {
        format == ExportFormat::Json
    }
}

// ============================================================================
// Binary Exporter
// ============================================================================

/// Binary exporter
pub struct BinaryExporter;

impl BinaryExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinaryExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportBackend for BinaryExporter {
    fn name(&self) -> &str {
        "Binary Exporter"
    }

    fn export(&self, snapshot: &Snapshot, _config: &ExportConfig) -> ExportResult<ExportOutput> {
        let binary = bincode::serialize(snapshot)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        Ok(ExportOutput::Binary(binary))
    }

    fn supports_format(&self, format: ExportFormat) -> bool {
        format == ExportFormat::Binary
    }
}

// ============================================================================
// HTML Generator
// ============================================================================

/// HTML generator
pub struct HtmlGenerator {
    /// Include 3D visualization
    include_3d: bool,
    /// Include timeline
    include_timeline: bool,
}

impl HtmlGenerator {
    pub fn new() -> Self {
        Self {
            include_3d: false,
            include_timeline: false,
        }
    }

    pub fn with_3d(mut self, include_3d: bool) -> Self {
        self.include_3d = include_3d;
        self
    }

    pub fn with_timeline(mut self, include_timeline: bool) -> Self {
        self.include_timeline = include_timeline;
        self
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportBackend for HtmlGenerator {
    fn name(&self) -> &str {
        "HTML Generator"
    }

    fn export(&self, snapshot: &Snapshot, config: &ExportConfig) -> ExportResult<ExportOutput> {
        let html = self.generate_html(snapshot, config)?;

        Ok(ExportOutput::String(html))
    }

    fn supports_format(&self, format: ExportFormat) -> bool {
        format == ExportFormat::Html
    }
}

impl HtmlGenerator {
    fn generate_html(&self, snapshot: &Snapshot, _config: &ExportConfig) -> ExportResult<String> {
        let data = serde_json::to_string(snapshot)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>MemScope Memory Analysis</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        * { margin: 0; padding: 0; box-sizing: border-box; }\n");
        html.push_str("        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 20px; }\n");
        html.push_str("        .container { max-width: 1400px; margin: 0 auto; }\n");
        html.push_str("        .header { background: white; padding: 30px; border-radius: 10px; margin-bottom: 30px; box-shadow: 0 10px 30px rgba(0,0,0,0.1); }\n");
        html.push_str("        .header h1 { color: #667eea; font-size: 2.5em; margin-bottom: 10px; }\n");
        html.push_str("        .kpi-container { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px; }\n");
        html.push_str("        .kpi-card { background: white; padding: 25px; border-radius: 10px; box-shadow: 0 5px 15px rgba(0,0,0,0.1); }\n");
        html.push_str("        .kpi-value { font-size: 2.5em; font-weight: bold; color: #667eea; margin-bottom: 10px; }\n");
        html.push_str("        .kpi-label { color: #666; font-size: 1em; text-transform: uppercase; letter-spacing: 1px; }\n");
        html.push_str("        .dashboard-grid { display: grid; grid-template-columns: 2fr 1fr; gap: 20px; margin-bottom: 30px; }\n");
        html.push_str("        .card { background: white; padding: 25px; border-radius: 10px; box-shadow: 0 5px 15px rgba(0,0,0,0.1); }\n");
        html.push_str("        .card h2 { color: #667eea; margin-bottom: 20px; font-size: 1.5em; }\n");
        html.push_str("        table { width: 100%; border-collapse: collapse; }\n");
        html.push_str("        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #eee; }\n");
        html.push_str("        th { background: #f8f9fa; color: #667eea; font-weight: 600; }\n");
        html.push_str("        tr:hover { background: #f8f9fa; }\n");
        html.push_str("        .status-active { color: #28a745; font-weight: bold; }\n");
        html.push_str("        .status-freed { color: #6c757d; }\n");
        html.push_str("        @media (max-width: 768px) { .dashboard-grid { grid-template-columns: 1fr; } .kpi-container { grid-template-columns: 1fr; } }\n");
        html.push_str("    </style>\n");

        // Add 3D visualization scripts if enabled
        if self.include_3d {
            html.push_str("    <script src=\"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js\"></script>\n");
        }

        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Header
        html.push_str("    <div class=\"container\">\n");
        html.push_str("        <div class=\"header\">\n");
        html.push_str("            <h1>🧠 MemScope Memory Analysis</h1>\n");
        html.push_str("            <p>Comprehensive memory tracking and visualization</p>\n");
        html.push_str("        </div>\n");

        // KPI container
        html.push_str("        <div class=\"kpi-container\" id=\"kpi-container\">\n");
        html.push_str("            <!-- KPIs will be inserted here -->\n");
        html.push_str("        </div>\n");

        // Dashboard grid
        html.push_str("        <div class=\"dashboard-grid\">\n");
        html.push_str("            <div class=\"card\">\n");
        html.push_str("                <h2>📊 Memory Allocations</h2>\n");
        html.push_str("                <div class=\"table-container\">\n");
        html.push_str("                    <table id=\"allocations-table\">\n");
        html.push_str("                        <thead>\n");
        html.push_str("                            <tr>\n");
        html.push_str("                                <th>Address</th>\n");
        html.push_str("                                <th>Size</th>\n");
        html.push_str("                                <th>Thread</th>\n");
        html.push_str("                                <th>Status</th>\n");
        html.push_str("                                <th>Lifetime (ms)</th>\n");
        html.push_str("                            </tr>\n");
        html.push_str("                        </thead>\n");
        html.push_str("                        <tbody>\n");
        html.push_str("                            <!-- Allocation rows will be inserted here -->\n");
        html.push_str("                        </tbody>\n");
        html.push_str("                    </table>\n");
        html.push_str("                </div>\n");
        html.push_str("            </div>\n");

        html.push_str("            <div class=\"card\">\n");
        html.push_str("                <h2>📈 Statistics</h2>\n");
        html.push_str("                <div id=\"stats-container\">\n");
        html.push_str("                    <!-- Stats will be inserted here -->\n");
        html.push_str("                </div>\n");
        html.push_str("            </div>\n");
        html.push_str("        </div>\n");

        // 3D visualization container if enabled
        if self.include_3d {
            html.push_str("        <div class=\"card\">\n");
            html.push_str("            <h2>🎮 3D Memory Visualization</h2>\n");
            html.push_str("            <div id=\"3d-container\" style=\"width: 100%; height: 500px;\"></div>\n");
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");

        // Data script
        html.push_str("    <script id=\"data-script\" type=\"application/json\">\n");
        html.push_str(&data);
        html.push_str("    </script>\n");

        // JavaScript
        html.push_str("    <script>\n");
        html.push_str("        const data = JSON.parse(document.getElementById('data-script').textContent);\n");

        html.push_str("        function formatBytes(bytes) {\n");
        html.push_str("            if (bytes === 0) return '0 B';\n");
        html.push_str("            const k = 1024;\n");
        html.push_str("            const sizes = ['B', 'KB', 'MB', 'GB'];\n");
        html.push_str("            const i = Math.floor(Math.log(bytes) / Math.log(k));\n");
        html.push_str("            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];\n");
        html.push_str("        }\n");

        html.push_str("        function renderKPIs() {\n");
        html.push_str("            const kpis = {\n");
        html.push_str("                totalAllocations: data.allocations.length,\n");
        html.push_str("                totalSize: data.allocations.reduce((sum, a) => sum + a.size, 0),\n");
        html.push_str("                activeAllocations: data.allocations.filter(a => !a.free_ts).length,\n");
        html.push_str("                leakedAllocations: data.allocations.filter(a => !a.free_ts).length,\n");
        html.push_str("            };\n");

        html.push_str("            document.getElementById('kpi-container').innerHTML = `\n");
        html.push_str("                <div class=\"kpi-card\">\n");
        html.push_str("                    <div class=\"kpi-value\">${kpis.totalAllocations}</div>\n");
        html.push_str("                    <div class=\"kpi-label\">Total Allocations</div>\n");
        html.push_str("                </div>\n");
        html.push_str("                <div class=\"kpi-card\">\n");
        html.push_str("                    <div class=\"kpi-value\">${formatBytes(kpis.totalSize)}</div>\n");
        html.push_str("                    <div class=\"kpi-label\">Total Size</div>\n");
        html.push_str("                </div>\n");
        html.push_str("                <div class=\"kpi-card\">\n");
        html.push_str("                    <div class=\"kpi-value\">${kpis.activeAllocations}</div>\n");
        html.push_str("                    <div class=\"kpi-label\">Active Allocations</div>\n");
        html.push_str("                </div>\n");
        html.push_str("                <div class=\"kpi-card\">\n");
        html.push_str("                    <div class=\"kpi-value\">${kpis.leakedAllocations}</div>\n");
        html.push_str("                    <div class=\"kpi-label\">Leaked Allocations</div>\n");
        html.push_str("                </div>\n");
        html.push_str("            `;\n");
        html.push_str("        }\n");

        html.push_str("        function renderAllocations() {\n");
        html.push_str("            const tbody = document.querySelector('#allocations-table tbody');\n");
        html.push_str("            const sortedAllocations = [...data.allocations].sort((a, b) => b.size - a.size).slice(0, 100);\n");

        html.push_str("            tbody.innerHTML = sortedAllocations.map(alloc => `\n");
        html.push_str("                <tr>\n");
        html.push_str("                    <td><code>0x${alloc.ptr.toString(16).padStart(12, '0')}</code></td>\n");
        html.push_str("                    <td>${formatBytes(alloc.size)}</td>\n");
        html.push_str("                    <td>${alloc.thread}</td>\n");
        html.push_str("                    <td class=\"${alloc.free_ts ? 'status-freed' : 'status-active'}\">\n");
        html.push_str("                        ${alloc.free_ts ? 'Freed' : 'Active'}\n");
        html.push_str("                    </td>\n");
        html.push_str("                    <td>${alloc.free_ts ? alloc.free_ts - alloc.alloc_ts : 'N/A'}</td>\n");
        html.push_str("                </tr>\n");
        html.push_str("            `).join('');\n");
        html.push_str("        }\n");

        html.push_str("        function renderStats() {\n");
        html.push_str("            const container = document.getElementById('stats-container');\n");
        html.push_str("            container.innerHTML = `\n");
        html.push_str("                <div style=\"margin-bottom: 15px;\"><strong>Total Threads:</strong> ${data.threads.length}</div>\n");
        html.push_str("                <div style=\"margin-bottom: 15px;\"><strong>Total Tasks:</strong> ${data.tasks.length}</div>\n");
        html.push_str("                <div style=\"margin-bottom: 15px;\"><strong>Passports:</strong> ${data.passports.length}</div>\n");
        html.push_str("                <div><strong>Timestamp:</strong> ${data.timestamp}</div>\n");
        html.push_str("            `;\n");
        html.push_str("        }\n");

        // 3D visualization if enabled
        if self.include_3d {
            html.push_str("        function render3D() {\n");
            html.push_str("            const container = document.getElementById('3d-container');\n");
            html.push_str("            const scene = new THREE.Scene();\n");
            html.push_str("            const camera = new THREE.PerspectiveCamera(75, container.clientWidth / container.clientHeight, 0.1, 1000);\n");
            html.push_str("            const renderer = new THREE.WebGLRenderer();\n");
            html.push_str("            renderer.setSize(container.clientWidth, container.clientHeight);\n");
            html.push_str("            container.appendChild(renderer.domElement);\n");

            html.push_str("            const allocations = data.allocations.slice(0, 100);\n");
            html.push_str("            const geometry = new THREE.BoxGeometry(1, 1, 1);\n");

            html.push_str("            allocations.forEach((alloc, index) => {\n");
            html.push_str("                const material = new THREE.MeshPhongMaterial({\n");
            html.push_str("                    color: alloc.free_ts ? 0x00ff00 : 0xff0000,\n");
            html.push_str("                });\n");
            html.push_str("                const block = new THREE.Mesh(geometry, material);\n");
            html.push_str("                block.position.x = (index % 10) * 2;\n");
            html.push_str("                block.position.y = Math.floor(index / 10) * 2;\n");
            html.push_str("                block.scale.z = Math.min(alloc.size / 1000, 5);\n");
            html.push_str("                scene.add(block);\n");
            html.push_str("            });\n");

            html.push_str("            const light = new THREE.DirectionalLight(0xffffff, 1);\n");
            html.push_str("            light.position.set(1, 1, 1);\n");
            html.push_str("            scene.add(light);\n");

            html.push_str("            camera.position.z = 20;\n");

            html.push_str("            function animate() {\n");
            html.push_str("                requestAnimationFrame(animate);\n");
            html.push_str("                renderer.render(scene, camera);\n");
            html.push_str("            }\n");
            html.push_str("            animate();\n");
            html.push_str("        }\n");
        }

        html.push_str("        // Initialize\n");
        html.push_str("        renderKPIs();\n");
        html.push_str("        renderAllocations();\n");
        html.push_str("        renderStats();\n");

        if self.include_3d {
            html.push_str("        render3D();\n");
        }

        html.push_str("    </script>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }
}

// ============================================================================
// CSV Exporter
// ============================================================================

/// CSV exporter
pub struct CsvExporter;

impl CsvExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportBackend for CsvExporter {
    fn name(&self) -> &str {
        "CSV Exporter"
    }

    fn export(&self, snapshot: &Snapshot, _config: &ExportConfig) -> ExportResult<ExportOutput> {
        let mut csv = String::new();
        csv.push_str("ptr,size,alloc_ts,free_ts,thread,var_name,type_name\n");

        for alloc in &snapshot.allocations {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                alloc.ptr,
                alloc.size,
                alloc.alloc_ts,
                alloc.free_ts.map_or("".to_string(), |t| t.to_string()),
                alloc.thread,
                alloc.meta.var_name.as_deref().unwrap_or(""),
                alloc.meta.type_name.as_deref().unwrap_or("")
            ));
        }

        Ok(ExportOutput::String(csv))
    }

    fn supports_format(&self, format: ExportFormat) -> bool {
        format == ExportFormat::Csv
    }
}

// ============================================================================
// Composite Exporter
// ============================================================================

/// Composite exporter that can use multiple backends
pub struct CompositeExporter {
    exporters: Vec<Box<dyn ExportBackend>>,
}

impl CompositeExporter {
    pub fn new() -> Self {
        Self {
            exporters: vec![
                Box::new(JsonExporter::new()),
                Box::new(BinaryExporter::new()),
                Box::new(HtmlGenerator::new()),
                Box::new(CsvExporter::new()),
            ],
        }
    }

    pub fn with_exporters(exporters: Vec<Box<dyn ExportBackend>>) -> Self {
        Self { exporters }
    }

    pub fn add_exporter(&mut self, exporter: Box<dyn ExportBackend>) {
        self.exporters.push(exporter);
    }

    pub fn find_exporter(&self, format: ExportFormat) -> Option<&dyn ExportBackend> {
        self.exporters
            .iter()
            .find(|e| e.supports_format(format))
            .map(|e| e.as_ref())
    }
}

impl Default for CompositeExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportBackend for CompositeExporter {
    fn name(&self) -> &str {
        "Composite Exporter"
    }

    fn export(&self, snapshot: &Snapshot, config: &ExportConfig) -> ExportResult<ExportOutput> {
        // Find the appropriate exporter for the requested format
        if let Some(exporter) = self.find_exporter(config.format) {
            exporter.export(snapshot, config)
        } else {
            Err(ExportError::UnsupportedFormat(format!(
                "{:?}",
                config.format
            )))
        }
    }

    fn supports_format(&self, format: ExportFormat) -> bool {
        self.exporters.iter().any(|e| e.supports_format(format))
    }
}
