use axum::{
    extract::Query,
    response::{Html, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub theme: Option<String>,
    pub debug: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

pub async fn serve_dashboard(Query(params): Query<PageQuery>, data: Arc<UnifiedMemoryData>) -> Result<Html<String>, StatusCode> {
    let _theme = params.theme.unwrap_or_else(|| "default".to_string());
    let _debug_mode = params.debug.unwrap_or(false);
    
    // Use the same template system as static HTML generation with real data
    let html_content = generate_web_dashboard_html_with_data(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Html(html_content))
}

/// Generate HTML content using the same template system as static generation with real data
fn generate_web_dashboard_html_with_data(unified_data: &UnifiedMemoryData) -> Result<String, Box<dyn std::error::Error>> {
    // Convert UnifiedMemoryData back to the JSON format expected by the template
    let mut json_data = HashMap::new();
    
    // Convert complex types data
    let complex_types_json = serde_json::json!({
        "categorized_types": {
            "generic_types": unified_data.complex_types.categorized_types.generic_types,
            "collections": unified_data.complex_types.categorized_types.collections,
            "smart_pointers": unified_data.complex_types.categorized_types.smart_pointers,
            "trait_objects": unified_data.complex_types.categorized_types.trait_objects
        },
        "complex_type_analysis": unified_data.complex_types.complex_type_analysis,
        "summary": {
            "total_complex_types": unified_data.complex_types.summary.total_complex_types,
            "generic_type_count": unified_data.complex_types.categorized_types.generic_types.len()
        }
    });
    json_data.insert("complex_types".to_string(), complex_types_json);
    
    // Convert memory analysis data
    let memory_analysis_json = serde_json::json!({
        "allocations": unified_data.allocations.iter().map(|alloc| {
            serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "var_name": alloc.var_name,
                "type_name": alloc.type_name,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "scope_name": alloc.scope_name
            })
        }).collect::<Vec<_>>()
    });
    json_data.insert("memory_analysis".to_string(), memory_analysis_json);
    
    // Convert lifetime data
    let lifetime_json = serde_json::json!({
        "lifecycle_events": unified_data.lifecycle.lifecycle_events
    });
    json_data.insert("lifetime".to_string(), lifetime_json);
    
    // Convert performance data
    let performance_json = serde_json::json!({
        "memory_performance": {
            "active_memory": unified_data.performance.active_memory,
            "peak_memory": unified_data.performance.peak_memory,
            "total_allocated": unified_data.performance.total_allocated
        }
    });
    json_data.insert("performance".to_string(), performance_json);
    
    // Convert unsafe FFI data
    let unsafe_ffi_json = serde_json::json!({
        "summary": {
            "unsafe_count": 0,
            "ffi_count": 0
        },
        "enhanced_ffi_data": []
    });
    json_data.insert("unsafe_ffi".to_string(), unsafe_ffi_json);
    
    // Use the same direct template generation as static HTML
    crate::cli::commands::html_from_json::direct_json_template::generate_direct_html(&json_data)
}

pub async fn handle_not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Resource not found".to_string(),
            code: 404,
        }),
    )
}

pub async fn handle_error(error: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error,
            code: 500,
        }),
    )
}