//! Render Engine - Output rendering
//!
//! This module provides the RenderEngine which is responsible for
//! rendering memory data in various formats (JSON, HTML, Binary, etc.).

pub mod engine;
pub mod export;
pub mod renderer;

pub use engine::RenderEngine;
pub use export::{
    export_all_json, export_leak_detection_json, export_memory_passports_json,
    export_snapshot_to_json, export_unsafe_ffi_json, ExportError, ExportJsonOptions,
};
pub use renderer::{OutputFormat, RenderConfig, RenderResult, Renderer};
