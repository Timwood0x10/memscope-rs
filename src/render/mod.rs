//! Unified render layer
//!
//! This module provides unified rendering capabilities for all tracking strategies
//! through a single interface (Renderer trait).

pub mod renderer;
pub mod json;
pub mod binary;
pub mod html;

// Re-export commonly used types
pub use renderer::Renderer;
pub use json::JsonRenderer;
pub use binary::BinaryRenderer;
pub use html::HtmlRenderer;