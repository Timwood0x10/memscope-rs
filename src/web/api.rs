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
}

/// Type summary for overview
#[derive(Serialize)]
pub struct TypeSummary {
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub average_size: usize,
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
}

/// Timeline point for memory usage over time
#[derive(Serialize)]
pub struct TimelinePoint {
    pub timestamp: u64,
    pub active_memory: usize,
    pub allocation_count: usize,
    pub event_type: String, // "alloc" or "dealloc"
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
        .map(|(type_name, (total_size, count))| TypeSummary {
            type_name,
            total_size,
            allocation_count: count,
            average_size: if count > 0 { total_size / count } else { 0 },
        })
        .collect();
    
    top_types.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    top_types.truncate(10);
    
    // Get recent allocations
    let mut recent_allocations: Vec<_> = memory_data.allocations
        .iter()
        .map(|alloc| AllocationSummary {
            ptr: alloc.ptr.clone(),
            size: alloc.size,
            var_name: alloc.var_name.clone(),
            type_name: alloc.type_name.clone(),
            timestamp: alloc.timestamp_alloc,
            is_active: alloc.timestamp_dealloc.is_none(),
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
            });
            
            // Add deallocation event if present
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                timeline_points.push(TimelinePoint {
                    timestamp: dealloc_time,
                    active_memory: current_memory - allocation.size,
                    allocation_count: current_count - 1,
                    event_type: "dealloc".to_string(),
                });
            }
        }
    }
    
    timeline_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
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
            
            timeline_points.push(TimelinePoint {
                timestamp: *timestamp,
                active_memory: allocation.size, // Simplified - should calculate cumulative
                allocation_count: 1,
                event_type: "alloc".to_string(),
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
    
    let performance_data = serde_json::json!({
        "stats": memory_data.stats,
        "performance": memory_data.performance,
        "allocation_distribution": memory_data.performance.allocation_distribution
    });
    
    Ok(Json(ApiResponse::success(performance_data)))
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