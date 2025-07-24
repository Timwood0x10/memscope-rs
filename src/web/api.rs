//! API handlers for MemScope web server
//!
//! Provides RESTful API endpoints for memory analysis data

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::server::ServerState;
use crate::cli::commands::html_from_json::data_normalizer::{AllocationInfo, MemoryStatistics};

/// Utility functions for data formatting and visualization hints
pub struct DataFormatter;

impl DataFormatter {
    /// Format bytes into human-readable string
    pub fn format_bytes(bytes: usize) -> String {
        if bytes == 0 {
            return "0 B".to_string();
        }
        
        let units = ["B", "KB", "MB", "GB", "TB"];
        let base = 1024_f64;
        let log = (bytes as f64).log(base).floor() as usize;
        let unit_index = log.min(units.len() - 1);
        let value = bytes as f64 / base.powi(unit_index as i32);
        
        format!("{:.1} {}", value, units[unit_index])
    }
    
    /// Format number with K/M suffixes
    pub fn format_number(num: usize) -> String {
        if num >= 1_000_000 {
            format!("{:.1}M", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K", num as f64 / 1_000.0)
        } else {
            num.to_string()
        }
    }
    
    /// Format timestamp as relative time
    pub fn format_timestamp(timestamp: u64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        if timestamp > now {
            return "Future".to_string();
        }
        
        let diff = now - timestamp;
        
        if diff < 60 {
            "Just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
    
    /// Get size category for visualization
    pub fn get_size_category(size: usize) -> String {
        match size {
            0..=64 => "tiny".to_string(),
            65..=1024 => "small".to_string(),
            1025..=1048576 => "medium".to_string(),
            1048577..=10485760 => "large".to_string(),
            _ => "massive".to_string(),
        }
    }
    
    /// Get color hint based on size category
    pub fn get_size_color(size: usize) -> String {
        match Self::get_size_category(size).as_str() {
            "tiny" => "#3b82f6".to_string(),
            "small" => "#10b981".to_string(),
            "medium" => "#f59e0b".to_string(),
            "large" => "#ef4444".to_string(),
            "massive" => "#8b5cf6".to_string(),
            _ => "#64748b".to_string(),
        }
    }
    
    /// Get event type icon
    pub fn get_event_icon(event_type: &str) -> String {
        match event_type {
            "alloc" | "allocation" => "🟢".to_string(),
            "dealloc" | "deallocation" => "🔴".to_string(),
            "resize" => "🟡".to_string(),
            _ => "⚪".to_string(),
        }
    }
    
    /// Get event type color
    pub fn get_event_color(event_type: &str) -> String {
        match event_type {
            "alloc" | "allocation" => "#10b981".to_string(),
            "dealloc" | "deallocation" => "#ef4444".to_string(),
            "resize" => "#f59e0b".to_string(),
            _ => "#64748b".to_string(),
        }
    }
    
    /// Get age category for allocations
    pub fn get_age_category(timestamp: u64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        if timestamp > now {
            return "future".to_string();
        }
        
        let age = now - timestamp;
        
        match age {
            0..=300 => "recent".to_string(),      // 5 minutes
            301..=3600 => "moderate".to_string(), // 1 hour
            3601..=86400 => "old".to_string(),    // 1 day
            _ => "ancient".to_string(),
        }
    }
    
    /// Calculate percentage of total
    pub fn calculate_percentage(value: usize, total: usize) -> f64 {
        if total == 0 {
            0.0
        } else {
            (value as f64 / total as f64) * 100.0
        }
    }
    
}

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Overview data structure
#[derive(Serialize)]
pub struct OverviewData {
    pub stats: MemoryStatistics,
    pub top_variables: Vec<VariableSummary>,
    pub top_types: Vec<TypeSummary>,
    pub recent_allocations: Vec<AllocationSummary>,
    pub memory_timeline: Vec<TimelinePoint>,
}

/// Variable summary for overview
#[derive(Serialize, Clone)]
pub struct VariableSummary {
    pub name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub type_name: Option<String>,
    pub scope: Option<String>,
    // Visualization hints
    pub size_formatted: String,
    pub count_formatted: String,
    pub size_category: String,
    pub color_hint: String,
    pub percentage_of_total: f64,
}

/// Type summary for overview
#[derive(Serialize)]
pub struct TypeSummary {
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub average_size: usize,
    // Visualization hints
    pub size_formatted: String,
    pub count_formatted: String,
    pub average_size_formatted: String,
    pub size_category: String,
    pub color_hint: String,
    pub percentage_of_total: f64,
}

/// Allocation summary
#[derive(Serialize)]
pub struct AllocationSummary {
    pub ptr: String,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub timestamp: u64,
    pub is_active: bool,
    // Visualization hints
    pub size_formatted: String,
    pub timestamp_formatted: String,
    pub size_category: String,
    pub color_hint: String,
    pub status_icon: String,
    pub age_category: String, // "recent", "old", etc.
}

/// Timeline point for memory usage over time
#[derive(Serialize)]
pub struct TimelinePoint {
    pub timestamp: u64,
    pub active_memory: usize,
    pub allocation_count: usize,
    pub event_type: String, // "alloc" or "dealloc"
    // Visualization hints
    pub timestamp_formatted: String,
    pub active_memory_formatted: String,
    pub count_formatted: String,
    pub event_icon: String,
    pub color_hint: String,
    pub chart_position: f64, // 0.0 to 1.0 for timeline positioning
}

/// Variable details response
#[derive(Serialize)]
pub struct VariableDetails {
    pub name: String,
    pub allocations: Vec<AllocationInfo>,
    pub total_size: usize,
    pub allocation_count: usize,
    pub timeline: Vec<VariableTimelineEvent>,
    pub related_variables: Vec<String>,
}

/// Variable timeline event
#[derive(Serialize)]
pub struct VariableTimelineEvent {
    pub timestamp: u64,
    pub event_type: String, // "allocated", "deallocated", "resized"
    pub size: usize,
    pub ptr: String,
}

/// Query parameters for variables list
#[derive(Deserialize)]
pub struct VariablesQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub filter_type: Option<String>,
    pub filter_scope: Option<String>,
    pub sort_by: Option<String>, // "name", "size", "count"
    pub sort_order: Option<String>, // "asc", "desc"
}

/// Query parameters for timeline
#[derive(Deserialize)]
pub struct TimelineQuery {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
}

/// Search query parameters
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category: Option<String>, // "variables", "types", "all"
    pub limit: Option<usize>,
}

/// Search result
#[derive(Serialize)]
pub struct SearchResult {
    pub category: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub relevance: f64,
}

/// GET /api/overview - Memory analysis overview
pub async fn overview(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<OverviewData>>, StatusCode> {
    let memory_data = &state.memory_data;
    let indexes = &state.indexes;
    
    // Calculate top variables by total size
    let mut variable_sizes: HashMap<String, (usize, usize)> = HashMap::new();
    for allocation in &memory_data.allocations {
        if let Some(var_name) = &allocation.var_name {
            let entry = variable_sizes.entry(var_name.clone()).or_insert((0, 0));
            entry.0 += allocation.size;
            entry.1 += 1;
        }
    }
    
    let total_memory = memory_data.stats.peak_memory;
    let mut top_variables: Vec<_> = variable_sizes
        .into_iter()
        .map(|(name, (total_size, count))| {
            let first_alloc = memory_data.allocations
                .iter()
                .find(|a| a.var_name.as_ref() == Some(&name));
            
            VariableSummary {
                name,
                total_size,
                allocation_count: count,
                type_name: first_alloc.and_then(|a| a.type_name.clone()),
                scope: first_alloc.and_then(|a| a.scope_name.clone()),
                // Add visualization hints
                size_formatted: DataFormatter::format_bytes(total_size),
                count_formatted: DataFormatter::format_number(count),
                size_category: DataFormatter::get_size_category(total_size),
                color_hint: DataFormatter::get_size_color(total_size),
                percentage_of_total: DataFormatter::calculate_percentage(total_size, total_memory),
            }
        })
        .collect();
    
    top_variables.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    top_variables.truncate(10);
    
    // Calculate top types
    let mut type_sizes: HashMap<String, (usize, usize)> = HashMap::new();
    for allocation in &memory_data.allocations {
        if let Some(type_name) = &allocation.type_name {
            let entry = type_sizes.entry(type_name.clone()).or_insert((0, 0));
            entry.0 += allocation.size;
            entry.1 += 1;
        }
    }
    
    let mut top_types: Vec<_> = type_sizes
        .into_iter()
        .map(|(type_name, (total_size, count))| {
            let average_size = if count > 0 { total_size / count } else { 0 };
            TypeSummary {
                type_name,
                total_size,
                allocation_count: count,
                average_size,
                // Add visualization hints
                size_formatted: DataFormatter::format_bytes(total_size),
                count_formatted: DataFormatter::format_number(count),
                average_size_formatted: DataFormatter::format_bytes(average_size),
                size_category: DataFormatter::get_size_category(total_size),
                color_hint: DataFormatter::get_size_color(total_size),
                percentage_of_total: DataFormatter::calculate_percentage(total_size, total_memory),
            }
        })
        .collect();
    
    top_types.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    top_types.truncate(10);
    
    // Get recent allocations
    let mut recent_allocations: Vec<_> = memory_data.allocations
        .iter()
        .map(|alloc| {
            let is_active = alloc.timestamp_dealloc.is_none();
            AllocationSummary {
                ptr: alloc.ptr.clone(),
                size: alloc.size,
                var_name: alloc.var_name.clone(),
                type_name: alloc.type_name.clone(),
                timestamp: alloc.timestamp_alloc,
                is_active,
                // Add visualization hints
                size_formatted: DataFormatter::format_bytes(alloc.size),
                timestamp_formatted: DataFormatter::format_timestamp(alloc.timestamp_alloc),
                size_category: DataFormatter::get_size_category(alloc.size),
                color_hint: if is_active { 
                    DataFormatter::get_size_color(alloc.size) 
                } else { 
                    "#9ca3af".to_string() 
                },
                status_icon: if is_active { "🟢".to_string() } else { "🔴".to_string() },
                age_category: DataFormatter::get_age_category(alloc.timestamp_alloc),
            }
        })
        .collect();
    
    recent_allocations.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    recent_allocations.truncate(20);
    
    // Generate memory timeline
    let mut timeline_points = Vec::new();
    let mut current_memory = 0usize;
    let mut current_count = 0usize;
    
    for (timestamp, allocation_indices) in &indexes.timeline_index {
        for &idx in allocation_indices {
            let allocation = &memory_data.allocations[idx];
            current_memory += allocation.size;
            current_count += 1;
            
            timeline_points.push(TimelinePoint {
                timestamp: *timestamp,
                active_memory: current_memory,
                allocation_count: current_count,
                event_type: "alloc".to_string(),
                // Add visualization hints
                timestamp_formatted: DataFormatter::format_timestamp(*timestamp),
                active_memory_formatted: DataFormatter::format_bytes(current_memory),
                count_formatted: DataFormatter::format_number(current_count),
                event_icon: DataFormatter::get_event_icon("alloc"),
                color_hint: DataFormatter::get_event_color("alloc"),
                chart_position: 0.0, // Will be calculated later
            });
            
            // Add deallocation event if present
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                let dealloc_memory = current_memory - allocation.size;
                let dealloc_count = current_count - 1;
                
                timeline_points.push(TimelinePoint {
                    timestamp: dealloc_time,
                    active_memory: dealloc_memory,
                    allocation_count: dealloc_count,
                    event_type: "dealloc".to_string(),
                    // Add visualization hints
                    timestamp_formatted: DataFormatter::format_timestamp(dealloc_time),
                    active_memory_formatted: DataFormatter::format_bytes(dealloc_memory),
                    count_formatted: DataFormatter::format_number(dealloc_count),
                    event_icon: DataFormatter::get_event_icon("dealloc"),
                    color_hint: DataFormatter::get_event_color("dealloc"),
                    chart_position: 0.0, // Will be calculated later
                });
            }
        }
    }
    
    timeline_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    // Calculate chart positions (0.0 to 1.0)
    if !timeline_points.is_empty() {
        let min_time = timeline_points.first().unwrap().timestamp;
        let max_time = timeline_points.last().unwrap().timestamp;
        let time_range = if max_time > min_time { max_time - min_time } else { 1 };
        
        for point in &mut timeline_points {
            point.chart_position = (point.timestamp - min_time) as f64 / time_range as f64;
        }
    }
    
    timeline_points.truncate(100); // Limit to 100 points for performance
    
    let overview = OverviewData {
        stats: memory_data.stats.clone(),
        top_variables,
        top_types,
        recent_allocations,
        memory_timeline: timeline_points,
    };
    
    Ok(Json(ApiResponse::success(overview)))
}

/// GET /api/variables - List all variables with pagination
pub async fn variables_list(
    Query(params): Query<VariablesQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<VariableSummary>>>, StatusCode> {
    let indexes = &state.indexes;
    let memory_data = &state.memory_data;
    
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(50).min(1000); // Max 1000 items per page
    
    // Collect all variables with their stats
    let mut variables: Vec<VariableSummary> = indexes.variable_index
        .iter()
        .map(|(name, allocation_indices)| {
            let total_size: usize = allocation_indices
                .iter()
                .map(|&idx| memory_data.allocations[idx].size)
                .sum();
            
            let first_alloc = &memory_data.allocations[allocation_indices[0]];
            
            VariableSummary {
                name: name.clone(),
                total_size,
                allocation_count: allocation_indices.len(),
                type_name: first_alloc.type_name.clone(),
                scope: first_alloc.scope_name.clone(),
                // Add visualization hints
                size_formatted: DataFormatter::format_bytes(total_size),
                count_formatted: DataFormatter::format_number(allocation_indices.len()),
                size_category: DataFormatter::get_size_category(total_size),
                color_hint: DataFormatter::get_size_color(total_size),
                percentage_of_total: DataFormatter::calculate_percentage(total_size, memory_data.stats.peak_memory),
            }
        })
        .collect();
    
    // Apply filters
    if let Some(filter_type) = &params.filter_type {
        variables.retain(|v| {
            v.type_name.as_ref().map_or(false, |t| t.contains(filter_type))
        });
    }
    
    if let Some(filter_scope) = &params.filter_scope {
        variables.retain(|v| {
            v.scope.as_ref().map_or(false, |s| s.contains(filter_scope))
        });
    }
    
    // Apply sorting
    match params.sort_by.as_deref().unwrap_or("size") {
        "name" => variables.sort_by(|a, b| a.name.cmp(&b.name)),
        "size" => variables.sort_by(|a, b| b.total_size.cmp(&a.total_size)),
        "count" => variables.sort_by(|a, b| b.allocation_count.cmp(&a.allocation_count)),
        _ => variables.sort_by(|a, b| b.total_size.cmp(&a.total_size)),
    }
    
    if params.sort_order.as_deref() == Some("asc") {
        variables.reverse();
    }
    
    // Apply pagination
    let start = page * limit;
    let end = (start + limit).min(variables.len());
    let page_variables = if start < variables.len() {
        variables[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    Ok(Json(ApiResponse::success(page_variables)))
}

/// GET /api/variables/:name - Get detailed information about a specific variable
pub async fn variable_details(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<VariableDetails>>, StatusCode> {
    let indexes = &state.indexes;
    let memory_data = &state.memory_data;
    
    if let Some(allocation_indices) = indexes.variable_index.get(&name) {
        let allocations: Vec<AllocationInfo> = allocation_indices
            .iter()
            .map(|&idx| memory_data.allocations[idx].clone())
            .collect();
        
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        
        // Generate timeline events
        let mut timeline_events: Vec<VariableTimelineEvent> = Vec::new();
        for allocation in &allocations {
            timeline_events.push(VariableTimelineEvent {
                timestamp: allocation.timestamp_alloc,
                event_type: "allocated".to_string(),
                size: allocation.size,
                ptr: allocation.ptr.clone(),
            });
            
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                timeline_events.push(VariableTimelineEvent {
                    timestamp: dealloc_time,
                    event_type: "deallocated".to_string(),
                    size: allocation.size,
                    ptr: allocation.ptr.clone(),
                });
            }
        }
        
        timeline_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Find related variables (same type or scope)
        let first_alloc = &allocations[0];
        let mut related_variables = Vec::new();
        
        if let Some(type_name) = &first_alloc.type_name {
            if let Some(type_indices) = indexes.type_index.get(type_name) {
                for &idx in type_indices.iter().take(10) {
                    if let Some(var_name) = &memory_data.allocations[idx].var_name {
                        if var_name != &name && !related_variables.contains(var_name) {
                            related_variables.push(var_name.clone());
                        }
                    }
                }
            }
        }
        
        let details = VariableDetails {
            name,
            allocations,
            total_size,
            allocation_count: allocation_indices.len(),
            timeline: timeline_events,
            related_variables,
        };
        
        Ok(Json(ApiResponse::success(details)))
    } else {
        Ok(Json(ApiResponse::error(format!("Variable '{}' not found", name))))
    }
}

/// GET /api/variables/:name/timeline - Get timeline events for a specific variable
pub async fn variable_timeline(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<VariableTimelineEvent>>>, StatusCode> {
    let indexes = &state.indexes;
    let memory_data = &state.memory_data;
    
    if let Some(allocation_indices) = indexes.variable_index.get(&name) {
        let mut timeline_events = Vec::new();
        
        for &idx in allocation_indices {
            let allocation = &memory_data.allocations[idx];
            
            timeline_events.push(VariableTimelineEvent {
                timestamp: allocation.timestamp_alloc,
                event_type: "allocated".to_string(),
                size: allocation.size,
                ptr: allocation.ptr.clone(),
            });
            
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                timeline_events.push(VariableTimelineEvent {
                    timestamp: dealloc_time,
                    event_type: "deallocated".to_string(),
                    size: allocation.size,
                    ptr: allocation.ptr.clone(),
                });
            }
        }
        
        timeline_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(Json(ApiResponse::success(timeline_events)))
    } else {
        Ok(Json(ApiResponse::error(format!("Variable '{}' not found", name))))
    }
}

/// GET /api/variables/:name/relationships - Get relationships for a specific variable
pub async fn variable_relationships(
    Path(name): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let indexes = &state.indexes;
    let memory_data = &state.memory_data;
    
    if let Some(allocation_indices) = indexes.variable_index.get(&name) {
        let mut related_variables = Vec::new();
        
        // Find variables with same type
        if let Some(&first_idx) = allocation_indices.first() {
            let first_alloc = &memory_data.allocations[first_idx];
            
            if let Some(type_name) = &first_alloc.type_name {
                if let Some(type_indices) = indexes.type_index.get(type_name) {
                    for &idx in type_indices.iter().take(20) {
                        if let Some(var_name) = &memory_data.allocations[idx].var_name {
                            if var_name != &name && !related_variables.contains(var_name) {
                                related_variables.push(var_name.clone());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Json(ApiResponse::success(related_variables)))
    } else {
        Ok(Json(ApiResponse::error(format!("Variable '{}' not found", name))))
    }
}

/// GET /api/timeline - Get timeline events
pub async fn timeline_events(
    Query(params): Query<TimelineQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<TimelinePoint>>>, StatusCode> {
    let indexes = &state.indexes;
    let memory_data = &state.memory_data;
    
    let limit = params.limit.unwrap_or(1000).min(10000);
    let mut timeline_points = Vec::new();
    let mut cumulative_memory = 0usize;
    let mut cumulative_count = 0usize;
    
    // Get min and max timestamps for chart positioning
    let min_timestamp = indexes.timeline_index.keys().min().copied().unwrap_or(0);
    let max_timestamp = indexes.timeline_index.keys().max().copied().unwrap_or(0);
    let time_range = max_timestamp.saturating_sub(min_timestamp).max(1);
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    
    for (timestamp, allocation_indices) in &indexes.timeline_index {
        // Apply time range filter
        if let Some(start_time) = params.start_time {
            if *timestamp < start_time {
                continue;
            }
        }
        if let Some(end_time) = params.end_time {
            if *timestamp > end_time {
                break;
            }
        }
        
        for &idx in allocation_indices {
            let allocation = &memory_data.allocations[idx];
            let is_active = allocation.timestamp_dealloc.is_none();
            
            if is_active {
                cumulative_memory += allocation.size;
                cumulative_count += 1;
            }
            
            // Calculate chart position (0.0 to 1.0)
            let chart_position = if time_range > 0 {
                (timestamp.saturating_sub(min_timestamp) as f64) / (time_range as f64)
            } else {
                0.0
            };
            
            timeline_points.push(TimelinePoint {
                timestamp: *timestamp,
                active_memory: cumulative_memory,
                allocation_count: cumulative_count,
                event_type: if is_active { "alloc".to_string() } else { "dealloc".to_string() },
                timestamp_formatted: DataFormatter::format_timestamp(*timestamp / 1_000_000_000), // Convert nanoseconds to seconds
                active_memory_formatted: DataFormatter::format_bytes(cumulative_memory),
                count_formatted: DataFormatter::format_number(cumulative_count),
                event_icon: if is_active { "📈".to_string() } else { "📉".to_string() },
                color_hint: if is_active { "#10b981".to_string() } else { "#ef4444".to_string() },
                chart_position,
            });
            
            if timeline_points.len() >= limit {
                break;
            }
        }
        
        if timeline_points.len() >= limit {
            break;
        }
    }
    
    Ok(Json(ApiResponse::success(timeline_points)))
}

/// GET /api/unsafe-ffi - Get unsafe/FFI analysis
pub async fn unsafe_ffi_analysis(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let memory_data = &state.memory_data;
    
    // Return the unsafe/FFI analysis data
    let unsafe_ffi_data = &memory_data.multi_source.get("unsafe_ffi")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    
    Ok(Json(ApiResponse::success(unsafe_ffi_data.clone())))
}

/// GET /api/performance - Get performance metrics
pub async fn performance_metrics(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let memory_data = &state.memory_data;
    
    // Calculate additional performance metrics
    let total_allocations = memory_data.allocations.len();
    let active_allocations = memory_data.allocations.iter()
        .filter(|a| a.timestamp_dealloc.is_none())
        .count();
    let deallocated_allocations = total_allocations - active_allocations;
    
    // Calculate memory efficiency
    let memory_efficiency = if memory_data.stats.peak_memory > 0 {
        (memory_data.stats.active_memory as f64 / memory_data.stats.peak_memory as f64 * 100.0) as u32
    } else {
        0
    };
    
    // Calculate allocation rate (use existing field)
    let allocation_rate = memory_data.performance.allocations_per_second as u32;
    
    // Get performance class
    let performance_class = if allocation_rate > 1000 {
        "excellent"
    } else if allocation_rate > 500 {
        "good"
    } else if allocation_rate > 100 {
        "fair"
    } else {
        "needs_optimization"
    };
    
    // Calculate fragmentation estimate
    let fragmentation_score = if total_allocations > 0 {
        let avg_allocation_size = memory_data.stats.peak_memory / total_allocations;
        if avg_allocation_size < 64 {
            "high"
        } else if avg_allocation_size < 1024 {
            "medium"
        } else {
            "low"
        }
    } else {
        "unknown"
    };
    
    let enhanced_performance_data = serde_json::json!({
        "overview": {
            "total_allocations": total_allocations,
            "active_allocations": active_allocations,
            "deallocated_allocations": deallocated_allocations,
            "peak_memory": memory_data.stats.peak_memory,
            "active_memory": memory_data.stats.active_memory,
            "memory_efficiency": memory_efficiency,
            "allocation_rate": allocation_rate,
            "performance_class": performance_class,
            "fragmentation_score": fragmentation_score,
            // Formatted values for display
            "peak_memory_formatted": DataFormatter::format_bytes(memory_data.stats.peak_memory),
            "active_memory_formatted": DataFormatter::format_bytes(memory_data.stats.active_memory),
            "total_allocations_formatted": DataFormatter::format_number(total_allocations),
            "active_allocations_formatted": DataFormatter::format_number(active_allocations),
            "allocation_rate_formatted": format!("{}/s", DataFormatter::format_number(allocation_rate as usize))
        },
        "metrics": {
            "memory_utilization": {
                "current": memory_data.stats.active_memory,
                "peak": memory_data.stats.peak_memory,
                "efficiency_percentage": memory_efficiency,
                "waste": memory_data.stats.peak_memory.saturating_sub(memory_data.stats.active_memory),
                "waste_formatted": DataFormatter::format_bytes(memory_data.stats.peak_memory.saturating_sub(memory_data.stats.active_memory)),
                "color_hint": if memory_efficiency > 80 { "#10b981" } else if memory_efficiency > 60 { "#f59e0b" } else { "#ef4444" }
            },
            "allocation_performance": {
                "rate": allocation_rate,
                "total_count": total_allocations,
                "active_count": active_allocations,
                "deallocation_rate": if total_allocations > 0 { (deallocated_allocations as f64 / total_allocations as f64 * 100.0) as u32 } else { 0 },
                "performance_class": performance_class,
                "color_hint": match performance_class {
                    "excellent" => "#10b981",
                    "good" => "#3b82f6", 
                    "fair" => "#f59e0b",
                    _ => "#ef4444"
                }
            },
            "fragmentation": {
                "score": fragmentation_score,
                "avg_allocation_size": if total_allocations > 0 { memory_data.stats.peak_memory / total_allocations } else { 0 },
                "avg_allocation_size_formatted": DataFormatter::format_bytes(if total_allocations > 0 { memory_data.stats.peak_memory / total_allocations } else { 0 }),
                "small_allocations": memory_data.allocations.iter().filter(|a| a.size <= 64).count(),
                "large_allocations": memory_data.allocations.iter().filter(|a| a.size > 1048576).count(),
                "color_hint": match fragmentation_score {
                    "low" => "#10b981",
                    "medium" => "#f59e0b",
                    _ => "#ef4444"
                }
            }
        },
        "trends": {
            "memory_timeline": memory_data.allocations.iter()
                .take(100) // Sample for timeline
                .map(|alloc| serde_json::json!({
                    "timestamp": alloc.timestamp_alloc,
                    "size": alloc.size,
                    "cumulative": 0, // Will be calculated on frontend
                    "event_type": if alloc.timestamp_dealloc.is_none() { "alloc" } else { "dealloc" }
                }))
                .collect::<Vec<_>>(),
            "size_distribution": {
                "tiny": memory_data.allocations.iter().filter(|a| a.size <= 64).count(),
                "small": memory_data.allocations.iter().filter(|a| a.size > 64 && a.size <= 1024).count(),
                "medium": memory_data.allocations.iter().filter(|a| a.size > 1024 && a.size <= 1048576).count(),
                "large": memory_data.allocations.iter().filter(|a| a.size > 1048576 && a.size <= 10485760).count(),
                "massive": memory_data.allocations.iter().filter(|a| a.size > 10485760).count()
            }
        },
        "raw_data": {
            "stats": memory_data.stats,
            "performance": memory_data.performance
        }
    });
    
    Ok(Json(ApiResponse::success(enhanced_performance_data)))
}

/// GET /api/search - Search variables, types, etc.
pub async fn search(
    Query(params): Query<SearchQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<Vec<SearchResult>>>, StatusCode> {
    let indexes = &state.indexes;
    let query = params.q.to_lowercase();
    let limit = params.limit.unwrap_or(50).min(500);
    let mut results = Vec::new();
    
    // Search variables
    if params.category.as_deref().unwrap_or("all") == "all" || params.category.as_deref() == Some("variables") {
        for (var_name, _) in &indexes.variable_index {
            if var_name.to_lowercase().contains(&query) {
                results.push(SearchResult {
                    category: "variable".to_string(),
                    name: var_name.clone(),
                    description: format!("Variable: {}", var_name),
                    url: format!("/api/variables/{}", var_name),
                    relevance: calculate_relevance(&query, var_name),
                });
            }
        }
    }
    
    // Search types
    if params.category.as_deref().unwrap_or("all") == "all" || params.category.as_deref() == Some("types") {
        for (type_name, _) in &indexes.type_index {
            if type_name.to_lowercase().contains(&query) {
                results.push(SearchResult {
                    category: "type".to_string(),
                    name: type_name.clone(),
                    description: format!("Type: {}", type_name),
                    url: format!("/api/types/{}", type_name),
                    relevance: calculate_relevance(&query, type_name),
                });
            }
        }
    }
    
    // Sort by relevance
    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    
    Ok(Json(ApiResponse::success(results)))
}

/// GET /api/allocation-distribution - Get allocation size distribution for charts
pub async fn allocation_distribution(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let memory_data = &state.memory_data;
    
    // Calculate distribution by size categories
    let mut distribution = std::collections::HashMap::new();
    distribution.insert("tiny", 0);
    distribution.insert("small", 0);
    distribution.insert("medium", 0);
    distribution.insert("large", 0);
    distribution.insert("massive", 0);
    
    let mut total_memory_by_category = std::collections::HashMap::new();
    total_memory_by_category.insert("tiny", 0usize);
    total_memory_by_category.insert("small", 0usize);
    total_memory_by_category.insert("medium", 0usize);
    total_memory_by_category.insert("large", 0usize);
    total_memory_by_category.insert("massive", 0usize);
    
    for allocation in &memory_data.allocations {
        let category = match allocation.size {
            0..=64 => "tiny",
            65..=1024 => "small", 
            1025..=1048576 => "medium",
            1048577..=10485760 => "large",
            _ => "massive",
        };
        
        *distribution.get_mut(category).unwrap() += 1;
        *total_memory_by_category.get_mut(category).unwrap() += allocation.size;
    }
    
    // Create visualization-ready data
    let chart_data = serde_json::json!({
        "distribution": {
            "tiny": {
                "count": distribution["tiny"],
                "total_size": total_memory_by_category["tiny"],
                "color": "#10b981",
                "label": "Tiny (≤64B)",
                "size_formatted": DataFormatter::format_bytes(total_memory_by_category["tiny"]),
                "count_formatted": DataFormatter::format_number(distribution["tiny"]),
                "percentage": DataFormatter::calculate_percentage(distribution["tiny"], memory_data.allocations.len())
            },
            "small": {
                "count": distribution["small"],
                "total_size": total_memory_by_category["small"],
                "color": "#3b82f6",
                "label": "Small (65B-1KB)",
                "size_formatted": DataFormatter::format_bytes(total_memory_by_category["small"]),
                "count_formatted": DataFormatter::format_number(distribution["small"]),
                "percentage": DataFormatter::calculate_percentage(distribution["small"], memory_data.allocations.len())
            },
            "medium": {
                "count": distribution["medium"],
                "total_size": total_memory_by_category["medium"],
                "color": "#f59e0b",
                "label": "Medium (1KB-1MB)",
                "size_formatted": DataFormatter::format_bytes(total_memory_by_category["medium"]),
                "count_formatted": DataFormatter::format_number(distribution["medium"]),
                "percentage": DataFormatter::calculate_percentage(distribution["medium"], memory_data.allocations.len())
            },
            "large": {
                "count": distribution["large"],
                "total_size": total_memory_by_category["large"],
                "color": "#ef4444",
                "label": "Large (1MB-10MB)",
                "size_formatted": DataFormatter::format_bytes(total_memory_by_category["large"]),
                "count_formatted": DataFormatter::format_number(distribution["large"]),
                "percentage": DataFormatter::calculate_percentage(distribution["large"], memory_data.allocations.len())
            },
            "massive": {
                "count": distribution["massive"],
                "total_size": total_memory_by_category["massive"],
                "color": "#7c3aed",
                "label": "Massive (>10MB)",
                "size_formatted": DataFormatter::format_bytes(total_memory_by_category["massive"]),
                "count_formatted": DataFormatter::format_number(distribution["massive"]),
                "percentage": DataFormatter::calculate_percentage(distribution["massive"], memory_data.allocations.len())
            }
        },
        "summary": {
            "total_allocations": memory_data.allocations.len(),
            "total_memory": memory_data.stats.peak_memory,
            "active_memory": memory_data.stats.active_memory,
            "total_memory_formatted": DataFormatter::format_bytes(memory_data.stats.peak_memory),
            "active_memory_formatted": DataFormatter::format_bytes(memory_data.stats.active_memory)
        }
    });
    
    Ok(Json(ApiResponse::success(chart_data)))
}

/// Calculate search relevance score
fn calculate_relevance(query: &str, text: &str) -> f64 {
    let text_lower = text.to_lowercase();
    
    if text_lower == query {
        1.0 // Exact match
    } else if text_lower.starts_with(query) {
        0.8 // Starts with query
    } else if text_lower.contains(query) {
        0.6 // Contains query
    } else {
        0.0 // No match
    }
}