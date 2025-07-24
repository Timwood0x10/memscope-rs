//! Web page handlers for MemScope server
//!
//! Provides HTML page handlers for the web interface

use axum::{
    extract::State,
    response::Html,
};
use std::sync::Arc;

use super::server::ServerState;

/// Main dashboard page
pub async fn index(State(state): State<Arc<ServerState>>) -> Html<String> {
    let stats = &state.memory_data.stats;
    
    let html = create_enhanced_dashboard_html(stats, state.memory_data.allocations.len(), state.config.port);
    
    Html(html)
}

/// Create enhanced dashboard HTML with modern styling
fn create_enhanced_dashboard_html(stats: &crate::cli::commands::html_from_json::data_normalizer::MemoryStatistics, allocation_count: usize, port: u16) -> String {
    // Read the template file
    let template = include_str!("dashboard_template.html");
    
    // Replace placeholders with actual values
    template
        .replace("{{ACTIVE_MEMORY}}", &stats.active_memory.to_string())
        .replace("{{PEAK_MEMORY}}", &stats.peak_memory.to_string())
        .replace("{{TOTAL_ALLOCATIONS}}", &stats.total_allocations.to_string())
        .replace("{{ACTIVE_ALLOCATIONS}}", &stats.active_allocations.to_string())
        .replace("{{ALLOCATION_COUNT}}", &allocation_count.to_string())
        .replace("{{VERSION}}", env!("CARGO_PKG_VERSION"))
        .replace("{{PORT}}", &port.to_string())
}