//! Renderer trait definition
//!
/// Unified renderer trait
///
/// 所有渲染器必须实现此 trait，提供一致的渲染接口
/// 
/// **关键**：
/// - 不需要知道具体策略
/// - 只需要处理 TrackingSnapshot
/// - 根据 snapshot 字段自动生成不同的 HTML

use crate::data::{TrackingSnapshot, ExportFormat, RenderOutput, RenderResult};

/// Unified renderer trait
///
/// All renderers must implement this trait to provide
/// a consistent interface for rendering tracking data.
pub trait Renderer: Send + Sync {
    /// Get the export format
    fn format(&self) -> ExportFormat;

    /// Render tracking snapshot to output
    ///
    /// # Arguments
    /// * `snapshot` - The tracking snapshot to render
    ///
    /// # Returns
    /// RenderOutput containing the rendered data
    fn render(&self, snapshot: &TrackingSnapshot) -> RenderResult<RenderOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{TrackingSnapshot, TrackingStrategy};

    #[test]
    fn test_renderer_trait_boundaries() {
        // This test just ensures the trait is properly defined
        // Actual implementations will be tested in their respective modules
        // No actual rendering here as we need concrete implementations
    }
}