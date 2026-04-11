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
