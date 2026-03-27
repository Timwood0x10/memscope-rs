//! Unified render layer
//!
//! This module provides unified rendering capabilities for all tracking strategies
//! through a single interface (Renderer trait).

pub mod binary;
pub mod html;
pub mod json;
pub mod renderer;

// Re-export commonly used types
pub use binary::BinaryRenderer;
pub use html::HtmlRenderer;
pub use json::JsonRenderer;
pub use renderer::Renderer;
