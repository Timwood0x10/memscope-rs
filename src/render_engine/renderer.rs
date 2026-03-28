//! Renderer trait for pluggable renderers
//!
//! This module defines the Renderer trait and output formats.

use crate::snapshot::MemorySnapshot;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Output format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// Binary format
    Binary,
    /// CSV format
    Csv,
    /// SVG format
    Svg,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Binary => write!(f, "binary"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Svg => write!(f, "svg"),
        }
    }
}

/// Render configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Output format
    pub format: OutputFormat,
    /// Output path (if writing to file)
    pub output_path: Option<String>,
    /// Whether to include detailed information
    pub verbose: bool,
    /// Whether to include timestamps
    pub include_timestamps: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            output_path: None,
            verbose: false,
            include_timestamps: true,
        }
    }
}

/// Render result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    /// The rendered data as bytes
    pub data: Vec<u8>,
    /// Output format used
    pub format: OutputFormat,
    /// Number of bytes rendered
    pub size: usize,
}

/// Renderer trait for pluggable output formats
///
/// All renderers must implement this trait to be used with the RenderEngine.
pub trait Renderer: Send + Sync {
    /// Get the output format of this renderer
    fn format(&self) -> OutputFormat;

    /// Render a memory snapshot
    ///
    /// # Arguments
    /// * `snapshot` - The memory snapshot to render
    /// * `config` - Render configuration
    ///
    /// # Returns
    /// Result containing the rendered data or an error
    fn render(&self, snapshot: &MemorySnapshot, config: &RenderConfig) -> Result<RenderResult, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Binary.to_string(), "binary");
    }

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.format, OutputFormat::Json);
        assert!(!config.verbose);
        assert!(config.include_timestamps);
    }
}