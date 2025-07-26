use axum::{
    extract::{Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DataQuery {
    pub format: Option<String>,
    pub limit: Option<usize>,
}

/// Shared state containing the unified memory data
#[derive(Clone)]
pub struct ApiState {
    pub unified_data: Arc<UnifiedMemoryData>,
}

pub fn create_api_router(unified_data: UnifiedMemoryData) -> Router {
    let state = ApiState {
        unified_data: Arc::new(unified_data),
    };

    Router::new()
        .route("/api/health", get(health))
        .route("/api/data/overview", get(get_overview))
        .route("/api/data/allocations", get(get_allocations))
        .route("/api/data/types", get(get_types))
        .route("/api/data/timeline", get(get_timeline))
        .route("/api/data/performance", get(get_performance))
        .route("/api/data/security", get(get_security))
        .route("/api/data/complex-types", get(get_complex_types))
        .route("/api/data/variable-relationships", get(get_variable_relationships))
        .route("/api/data/unified", get(get_unified_data))
        .with_state(state)
}

async fn health() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("MemScope API is running".to_string()),
        error: None,
    })
}

async fn get_overview(State(state): State<ApiState>) -> Json<ApiResponse<HashMap<String, serde_json::Value>>> {
    let data = &state.unified_data;
    let mut overview = HashMap::new();
    
    // 使用与静态HTML相同的数据处理逻辑
    overview.insert("status".to_string(), serde_json::Value::String("active".to_string()));
    overview.insert("total_allocations".to_string(), serde_json::Value::Number(serde_json::Number::from(data.stats.total_allocations)));
    overview.insert("active_allocations".to_string(), serde_json::Value::Number(serde_json::Number::from(data.stats.active_allocations)));
    overview.insert("active_memory".to_string(), serde_json::Value::Number(serde_json::Number::from(data.stats.active_memory)));
    overview.insert("peak_memory".to_string(), serde_json::Value::Number(serde_json::Number::from(data.stats.peak_memory)));
    overview.insert("total_allocated".to_string(), serde_json::Value::Number(serde_json::Number::from(data.stats.total_allocated)));
    overview.insert("memory_efficiency".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(data.stats.memory_efficiency).unwrap_or(serde_json::Number::from(0))));
    
    Json(ApiResponse {
        success: true,
        data: Some(overview),
        error: None,
    })
}

async fn get_allocations(
    Query(params): Query<DataQuery>,
    State(state): State<ApiState>
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let _limit = params.limit.unwrap_or(100);
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的分配数据处理逻辑
    let allocations: Vec<serde_json::Value> = data.allocations
        .iter()
        .map(|alloc| {
            serde_json::json!({
                "ptr": alloc.ptr,
                "size": alloc.size,
                "type_name": alloc.type_name,
                "var_name": alloc.var_name,
                "timestamp_alloc": alloc.timestamp_alloc,
                "timestamp_dealloc": alloc.timestamp_dealloc,
                "scope_name": alloc.scope_name,
                "is_leaked": alloc.is_leaked,
                "lifetime_ms": alloc.lifetime_ms
            })
        })
        .collect();
    
    Json(ApiResponse {
        success: true,
        data: Some(allocations),
        error: None,
    })
}

async fn get_types(State(state): State<ApiState>) -> Json<ApiResponse<HashMap<String, usize>>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的类型统计逻辑
    let mut types = HashMap::new();
    for alloc in &data.allocations {
        if let Some(type_name) = &alloc.type_name {
            *types.entry(type_name.clone()).or_insert(0) += alloc.size;
        }
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(types),
        error: None,
    })
}

async fn get_timeline(State(state): State<ApiState>) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的时间线数据处理逻辑
    let timeline: Vec<serde_json::Value> = data.lifecycle
        .lifecycle_events
        .iter()
        .map(|event| {
            serde_json::json!({
                "event": event,
                "timestamp": event.get("timestamp").unwrap_or(&serde_json::Value::Null),
                "data": event
            })
        })
        .collect();
    
    Json(ApiResponse {
        success: true,
        data: Some(timeline),
        error: None,
    })
}

async fn get_performance(State(state): State<ApiState>) -> Json<ApiResponse<serde_json::Value>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的性能数据处理逻辑
    let performance_data = serde_json::to_value(&data.performance)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    Json(ApiResponse {
        success: true,
        data: Some(performance_data),
        error: None,
    })
}

async fn get_security(State(state): State<ApiState>) -> Json<ApiResponse<serde_json::Value>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的安全数据处理逻辑
    let security_data = serde_json::to_value(&data.security)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    Json(ApiResponse {
        success: true,
        data: Some(security_data),
        error: None,
    })
}

async fn get_complex_types(State(state): State<ApiState>) -> Json<ApiResponse<serde_json::Value>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的复杂类型数据处理逻辑
    let complex_types_data = serde_json::to_value(&data.complex_types)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    Json(ApiResponse {
        success: true,
        data: Some(complex_types_data),
        error: None,
    })
}

async fn get_variable_relationships(State(state): State<ApiState>) -> Json<ApiResponse<serde_json::Value>> {
    let data = &state.unified_data;
    
    // 使用与静态HTML相同的变量关系数据处理逻辑
    let relationships_data = serde_json::to_value(&data.variable_relationships)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    Json(ApiResponse {
        success: true,
        data: Some(relationships_data),
        error: None,
    })
}

async fn get_unified_data(State(state): State<ApiState>) -> Json<ApiResponse<serde_json::Value>> {
    let data = &state.unified_data;
    
    // 返回完整的统一数据结构，与静态HTML使用的完全相同
    let unified_data = serde_json::to_value(data.as_ref())
        .unwrap_or_else(|_| serde_json::json!({}));
    
    Json(ApiResponse {
        success: true,
        data: Some(unified_data),
        error: None,
    })
}