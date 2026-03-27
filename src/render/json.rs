//! JSON renderer implementation

use super::Renderer;
use crate::data::{ExportFormat, RenderOutput, RenderResult, TrackingSnapshot};
use crate::error::types::{ErrorKind, ErrorSeverity, MemScopeError};

/// JSON renderer
pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn format(&self) -> ExportFormat {
        ExportFormat::Json
    }

    fn render(&self, snapshot: &TrackingSnapshot) -> RenderResult<RenderOutput> {
        // 直接序列化整个 snapshot
        let json = serde_json::to_string_pretty(snapshot).map_err(|e| {
            MemScopeError::with_context(
                ErrorKind::InternalError,
                ErrorSeverity::Error,
                &format!("JSON serialization failed: {}", e),
                "JsonRenderer",
            )
        })?;

        Ok(RenderOutput::String(json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AllocationRecord, TrackingSnapshot, TrackingStrategy};

    #[test]
    fn test_json_renderer_format() {
        let renderer = JsonRenderer;
        assert_eq!(renderer.format(), ExportFormat::Json);
    }

    #[test]
    fn test_json_renderer_render_empty() {
        let renderer = JsonRenderer;
        let snapshot = TrackingSnapshot::new(TrackingStrategy::Core);

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());

        if let Ok(RenderOutput::String(json)) = result {
            assert!(json.contains("strategy"));
            assert!(json.contains("allocations"));
        } else {
            panic!("Expected String output");
        }
    }

    #[test]
    fn test_json_renderer_render_with_data() {
        let renderer = JsonRenderer;
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);

        let alloc = AllocationRecord::new(0x1000, 1024);
        snapshot.add_allocation(alloc);

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());

        if let Ok(RenderOutput::String(json)) = result {
            assert!(json.contains("1024"));
            assert!(json.contains("0x1000"));
        } else {
            panic!("Expected String output");
        }
    }

    #[test]
    fn test_json_renderer_pretty() {
        let renderer = JsonRenderer;
        let snapshot = TrackingSnapshot::new(TrackingStrategy::Core);

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());

        if let Ok(RenderOutput::String(json)) = result {
            // Pretty printed JSON should have newlines
            assert!(json.contains('\n'));
        } else {
            panic!("Expected String output");
        }
    }
}
