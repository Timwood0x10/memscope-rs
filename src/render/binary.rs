//! Binary renderer implementation

use super::Renderer;
use crate::data::{ExportFormat, RenderOutput, RenderResult, TrackingSnapshot};
use crate::error::types::{ErrorKind, ErrorSeverity, MemScopeError};

/// Binary renderer
pub struct BinaryRenderer;

impl Renderer for BinaryRenderer {
    fn format(&self) -> ExportFormat {
        ExportFormat::Binary
    }

    fn render(&self, snapshot: &TrackingSnapshot) -> RenderResult<RenderOutput> {
        // Use serde_json to serialize as bytes for simplicity
        let json_bytes = serde_json::to_vec(snapshot).map_err(|e| {
            MemScopeError::with_context(
                ErrorKind::InternalError,
                ErrorSeverity::Error,
                &format!("Binary serialization failed: {}", e),
                "BinaryRenderer",
            )
        })?;

        Ok(RenderOutput::Bytes(json_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AllocationRecord, TrackingSnapshot, TrackingStrategy};

    #[test]
    fn test_binary_renderer_format() {
        let renderer = BinaryRenderer;
        assert_eq!(renderer.format(), ExportFormat::Binary);
    }

    #[test]
    fn test_binary_renderer_render_empty() {
        let renderer = BinaryRenderer;
        let snapshot = TrackingSnapshot::new(TrackingStrategy::Core);

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());

        if let Ok(RenderOutput::Bytes(bytes)) = result {
            assert!(!bytes.is_empty());
        } else {
            panic!("Expected Bytes output");
        }
    }

    #[test]
    fn test_binary_renderer_render_with_data() {
        let renderer = BinaryRenderer;
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);

        let alloc = AllocationRecord::new(0x1000, 1024);
        snapshot.add_allocation(alloc);

        let result = renderer.render(&snapshot);
        assert!(result.is_ok());

        if let Ok(RenderOutput::Bytes(bytes)) = result {
            // Binary should be compact
            assert!(!bytes.is_empty());
        } else {
            panic!("Expected Bytes output");
        }
    }

    #[test]
    fn test_binary_roundtrip() {
        let renderer = BinaryRenderer;
        let mut original = TrackingSnapshot::new(TrackingStrategy::Core);

        let alloc = AllocationRecord::new(0x1000, 1024);
        original.add_allocation(alloc);

        // Serialize
        let result = renderer.render(&original);
        assert!(result.is_ok());

        // Deserialize
        if let Ok(RenderOutput::Bytes(bytes)) = result {
            let deserialized: TrackingSnapshot = serde_json::from_slice(&bytes)
                .map_err(|e| {
                    MemScopeError::with_context(
                        ErrorKind::InternalError,
                        ErrorSeverity::Error,
                        &format!("Binary deserialization failed: {}", e),
                        "BinaryRenderer",
                    )
                })
                .expect("Deserialization failed");

            // Verify data integrity
            assert_eq!(deserialized.strategy, original.strategy);
            assert_eq!(deserialized.allocations.len(), original.allocations.len());
            assert_eq!(
                deserialized.stats.total_allocations,
                original.stats.total_allocations
            );
        } else {
            panic!("Expected Bytes output");
        }
    }
}
