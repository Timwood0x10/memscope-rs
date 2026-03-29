//! Render Engine - Output rendering
//!
//! This module provides the RenderEngine which coordinates rendering
/// of memory data in various formats.
use crate::render_engine::renderer::{OutputFormat, RenderConfig, RenderResult, Renderer};
use crate::snapshot::{MemorySnapshot, SharedSnapshotEngine};
use serde_json;

/// JSON Renderer
struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn format(&self) -> OutputFormat {
        OutputFormat::Json
    }

    fn render(
        &self,
        snapshot: &MemorySnapshot,
        config: &RenderConfig,
    ) -> Result<RenderResult, String> {
        let data = if config.verbose {
            serde_json::to_vec_pretty(&snapshot)
        } else {
            serde_json::to_vec(&snapshot)
        }
        .map_err(|e| e.to_string())?;

        let size = data.len();

        Ok(RenderResult {
            data,
            format: OutputFormat::Json,
            size,
        })
    }
}

/// Render Engine - Coordinates output rendering
///
/// The RenderEngine manages multiple renderers and provides a unified
/// interface for rendering memory data in various formats.
///
/// Key properties:
/// - Pluggable: Supports adding custom renderers
/// - Flexible: Supports multiple output formats
/// - Configurable: Supports various rendering options
pub struct RenderEngine {
    /// Reference to the snapshot engine
    snapshot_engine: SharedSnapshotEngine,
    /// Registered renderers
    renderers: Vec<Box<dyn Renderer>>,
}

impl RenderEngine {
    /// Create a new RenderEngine
    pub fn new(snapshot_engine: SharedSnapshotEngine) -> Self {
        let mut engine = Self {
            snapshot_engine,
            renderers: Vec::new(),
        };

        // Register default renderers
        engine.register_renderer(Box::new(JsonRenderer));

        engine
    }

    /// Register a renderer
    ///
    /// # Arguments
    /// * `renderer` - The renderer to register
    pub fn register_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderers.push(renderer);
    }

    /// Render the current snapshot with the specified configuration
    ///
    /// # Arguments
    /// * `config` - Render configuration
    ///
    /// # Returns
    /// Result containing the rendered data or an error
    pub fn render(&self, config: &RenderConfig) -> Result<RenderResult, String> {
        let snapshot = self.snapshot_engine.build_snapshot();
        self.render_snapshot(&snapshot, config)
    }

    /// Render a specific snapshot with the specified configuration
    ///
    /// # Arguments
    /// * `snapshot` - The snapshot to render
    /// * `config` - Render configuration
    ///
    /// # Returns
    /// Result containing the rendered data or an error
    pub fn render_snapshot(
        &self,
        snapshot: &MemorySnapshot,
        config: &RenderConfig,
    ) -> Result<RenderResult, String> {
        // Find a renderer for the requested format
        for renderer in &self.renderers {
            if renderer.format() == config.format {
                return renderer.render(snapshot, config);
            }
        }

        Err(format!("No renderer found for format: {}", config.format))
    }

    /// Render to JSON format
    pub fn render_json(
        &self,
        snapshot: &MemorySnapshot,
        verbose: bool,
    ) -> Result<RenderResult, String> {
        let config = RenderConfig {
            format: OutputFormat::Json,
            output_path: None,
            verbose,
            include_timestamps: true,
        };
        self.render_snapshot(snapshot, &config)
    }

    /// Check if a renderer is available for the specified format
    ///
    /// # Arguments
    /// * `format` - The output format to check
    pub fn has_renderer(&self, format: OutputFormat) -> bool {
        self.renderers.iter().any(|r| r.format() == format)
    }

    /// Get the number of registered renderers
    pub fn renderer_count(&self) -> usize {
        self.renderers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventStore;
    use crate::snapshot::SnapshotEngine;
    use std::sync::Arc;

    #[test]
    fn test_render_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let engine = RenderEngine::new(snapshot_engine);

        assert!(engine.has_renderer(OutputFormat::Json));
        assert_eq!(engine.renderer_count(), 1);
    }

    #[test]
    fn test_render_json() {
        let event_store = Arc::new(EventStore::new());
        event_store.record(crate::event_store::MemoryEvent::allocate(0x1000, 1024, 1));

        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let engine = RenderEngine::new(snapshot_engine);

        let snapshot = engine.snapshot_engine.build_snapshot();
        let result = engine.render_json(&snapshot, false);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.format, OutputFormat::Json);
        assert!(result.size > 0);
    }

    #[test]
    fn test_render_with_config() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let engine = RenderEngine::new(snapshot_engine);

        let config = RenderConfig::default();
        let result = engine.render(&config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_has_renderer() {
        let event_store = Arc::new(EventStore::new());
        let snapshot_engine = Arc::new(SnapshotEngine::new(event_store));
        let engine = RenderEngine::new(snapshot_engine);

        assert!(engine.has_renderer(OutputFormat::Json));
        assert!(!engine.has_renderer(OutputFormat::Html));
    }
}
