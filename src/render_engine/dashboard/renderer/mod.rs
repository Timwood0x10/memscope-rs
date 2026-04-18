//! Dashboard renderer using Handlebars templates.
//!
//! This module provides the main dashboard renderer that generates
//! HTML reports from memory tracking data.

mod context;
mod helpers;
mod render_methods;
mod system_info;
mod types;

pub use types::*;

// Re-export for external use
pub use context::rebuild_allocations_from_events;

use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::tracker::Tracker;
use handlebars::Handlebars;
use std::sync::Arc;

/// Dashboard renderer
pub struct DashboardRenderer {
    handlebars: Handlebars<'static>,
}

impl DashboardRenderer {
    /// Create a new dashboard renderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();

        let template_path = format!(
            "{}/src/render_engine/dashboard/templates/dashboard_unified.html",
            env!("CARGO_MANIFEST_DIR")
        );
        handlebars.register_template_file("dashboard_unified", &template_path)?;

        let final_path = format!(
            "{}/src/render_engine/dashboard/templates/dashboard_final.html",
            env!("CARGO_MANIFEST_DIR")
        );
        handlebars.register_template_file("dashboard_final", &final_path)?;

        helpers::register_helpers(&mut handlebars);

        Ok(Self { handlebars })
    }

    /// Build dashboard context from tracker data
    pub fn build_context_from_tracker(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> Result<DashboardContext, Box<dyn std::error::Error>> {
        self.build_context_from_tracker_with_async(tracker, passport_tracker, None)
    }

    /// Build dashboard context from tracker data with async support
    pub fn build_context_from_tracker_with_async(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
        async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
    ) -> Result<DashboardContext, Box<dyn std::error::Error>> {
        context::build_context_from_tracker_with_async(tracker, passport_tracker, async_tracker)
    }

    /// Render dashboard from tracker data (for standalone template)
    pub fn render_from_tracker(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let context = self.build_context_from_tracker(tracker, passport_tracker)?;
        self.render_dashboard(&context)
    }

    /// Render dashboard from context
    pub fn render_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.render_unified_dashboard(context)
    }

    /// Render standalone dashboard (no external dependencies, works with file:// protocol)
    pub fn render_standalone_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.render_unified_dashboard(context)
    }

    /// Render unified dashboard (multi-mode in single HTML)
    pub fn render_unified_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_unified_dashboard(&self.handlebars, context)
    }

    /// Render final dashboard (new investigation console template)
    pub fn render_final_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_final_dashboard(&self.handlebars, context)
    }

    /// Render binary dashboard (legacy template)
    pub fn render_binary_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_binary_dashboard(&self.handlebars, context)
    }

    /// Render clean dashboard (legacy template)
    pub fn render_clean_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_clean_dashboard(&self.handlebars, context)
    }

    /// Render hybrid dashboard (legacy template)
    pub fn render_hybrid_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_hybrid_dashboard(&self.handlebars, context)
    }

    /// Render performance dashboard (legacy template)
    pub fn render_performance_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        render_methods::render_performance_dashboard(&self.handlebars, context)
    }
}

impl Default for DashboardRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create dashboard renderer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_engine::dashboard::renderer::types::{
        AsyncSummary, CircularReferenceReport, DashboardContext, OwnershipGraphInfo,
        SystemResources,
    };

    fn create_empty_context() -> DashboardContext {
        DashboardContext {
            title: "Test".to_string(),
            export_timestamp: "2024-01-01".to_string(),
            total_memory: "0 B".to_string(),
            total_allocations: 0,
            active_allocations: 0,
            peak_memory: "0 B".to_string(),
            thread_count: 0,
            passport_count: 0,
            leak_count: 0,
            unsafe_count: 0,
            ffi_count: 0,
            allocations: vec![],
            relationships: vec![],
            unsafe_reports: vec![],
            passport_details: vec![],
            allocations_count: 0,
            relationships_count: 0,
            unsafe_reports_count: 0,
            json_data: "{}".to_string(),
            os_name: "test".to_string(),
            architecture: "test".to_string(),
            cpu_cores: 1,
            system_resources: SystemResources {
                os_name: "test".to_string(),
                os_version: "1.0".to_string(),
                architecture: "test".to_string(),
                cpu_cores: 1,
                total_physical: "0 B".to_string(),
                available_physical: "0 B".to_string(),
                used_physical: "0 B".to_string(),
                page_size: 4096,
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
            },
            health_score: 100,
            health_status: "Good".to_string(),
            safe_ops_count: 0,
            high_risk_count: 0,
            clean_passport_count: 0,
            active_passport_count: 0,
            leaked_passport_count: 0,
            ffi_tracked_count: 0,
            safe_code_percent: 100,
            ownership_graph: OwnershipGraphInfo {
                total_nodes: 0,
                total_edges: 0,
                total_cycles: 0,
                rc_clone_count: 0,
                arc_clone_count: 0,
                has_issues: false,
                issues: vec![],
                root_cause: None,
            },
            top_allocation_sites: vec![],
            top_leaked_allocations: vec![],
            top_temporary_churn: vec![],
            circular_references: CircularReferenceReport {
                count: 0,
                total_leaked_memory: 0,
                pointers_in_cycles: 0,
                total_smart_pointers: 0,
                has_cycles: false,
            },
        }
    }

    /// Objective: Verify that DashboardRenderer creates successfully.
    /// Invariants: Renderer must be created with valid templates registered.
    #[test]
    fn test_dashboard_renderer_creation() {
        let result = DashboardRenderer::new();
        assert!(
            result.is_ok(),
            "DashboardRenderer should create successfully"
        );
    }

    /// Objective: Verify that DashboardRenderer implements Default.
    /// Invariants: Default should create a valid renderer instance.
    #[test]
    fn test_dashboard_renderer_default() {
        let renderer = DashboardRenderer::default();
        let _ = &renderer;
    }

    /// Objective: Verify that render_unified_dashboard works with minimal context.
    /// Invariants: Should render without errors for valid context.
    #[test]
    fn test_render_unified_dashboard() {
        let renderer = DashboardRenderer::new().expect("Should create renderer");
        let context = create_empty_context();
        let result = renderer.render_unified_dashboard(&context);
        assert!(
            result.is_ok(),
            "Should render unified dashboard successfully"
        );
    }

    /// Objective: Verify that render_final_dashboard works with minimal context.
    /// Invariants: Should render without errors for valid context.
    #[test]
    fn test_render_final_dashboard() {
        let renderer = DashboardRenderer::new().expect("Should create renderer");
        let context = create_empty_context();
        let result = renderer.render_final_dashboard(&context);
        assert!(result.is_ok(), "Should render final dashboard successfully");
    }

    /// Objective: Verify that render_dashboard delegates to unified dashboard.
    /// Invariants: Should produce same output as render_unified_dashboard.
    #[test]
    fn test_render_dashboard() {
        let renderer = DashboardRenderer::new().expect("Should create renderer");
        let context = create_empty_context();
        let result = renderer.render_dashboard(&context);
        assert!(result.is_ok(), "Should render dashboard successfully");
    }

    /// Objective: Verify that render_standalone_dashboard delegates correctly.
    /// Invariants: Should produce same output as unified dashboard.
    #[test]
    fn test_render_standalone_dashboard() {
        let renderer = DashboardRenderer::new().expect("Should create renderer");
        let context = create_empty_context();
        let result = renderer.render_standalone_dashboard(&context);
        assert!(
            result.is_ok(),
            "Should render standalone dashboard successfully"
        );
    }
}
