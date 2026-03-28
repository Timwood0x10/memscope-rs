//! Render Engine - Output rendering
//!
//! This module provides the RenderEngine which is responsible for
//! rendering memory data in various formats (JSON, HTML, Binary, etc.).

pub mod engine;
pub mod renderer;

pub use engine::RenderEngine;
pub use renderer::{OutputFormat, RenderConfig, RenderResult, Renderer};