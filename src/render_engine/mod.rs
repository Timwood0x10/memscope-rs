//! Render Engine - Output rendering
//!
//! This module provides the RenderEngine which is responsible for
//! rendering memory data in various formats (JSON, HTML, Binary, etc.).

pub mod dashboard;
pub mod engine;
pub mod export;
pub mod renderer;

pub use dashboard::{DashboardContext, DashboardRenderer};
pub use engine::RenderEngine;
pub use export::{
    export_all_json, export_dashboard_html, export_dashboard_html_with_template,
    export_leak_detection_json, export_memory_passports_json, export_ownership_graph_json,
    export_snapshot_to_json, export_unsafe_ffi_json, DashboardTemplate, ExportError,
    ExportJsonOptions, OptimizationLevel, SchemaValidator,
};
pub use renderer::{OutputFormat, RenderConfig, RenderResult, Renderer};
