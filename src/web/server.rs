use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use axum::{
    routing::get,
    Router,
    response::{Html, Json},
    extract::Query,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub static_dir: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            static_dir: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub filter: Option<String>,
    pub sort: Option<String>,
}

pub struct MemScopeServer {
    data: Arc<UnifiedMemoryData>,
    config: ServerConfig,
}

impl MemScopeServer {
    pub fn new(data: UnifiedMemoryData, config: ServerConfig) -> Self {
        Self {
            data: Arc::new(data),
            config,
        }
    }

    pub async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        println!("🌐 Starting MemScope web server at http://{}:{}", self.config.host, self.config.port);
        println!("📊 Serving {} allocations with API endpoints", self.data.allocations.len());

        let app = self.create_app();
        let listener = TcpListener::bind(addr).await?;
        
        println!("✅ Server ready! Open http://{}:{} in your browser", self.config.host, self.config.port);
        println!("🛑 Press Ctrl+C to stop the server");

        axum::serve(listener, app).await?;
        Ok(())
    }

    fn create_app(&self) -> Router {
        let data = Arc::clone(&self.data);
        
        let mut app = Router::new()
            // Main dashboard page
            .route("/", get({
                let data = Arc::clone(&data);
                move |query| serve_dashboard(query, data)
            }))
            
            // Raw JSON data endpoint - matches static HTML embedded data format
            .route("/api/data", get({
                let data = Arc::clone(&data);
                move || get_raw_json_data(data)
            }))
            
            // Legacy API endpoints for backward compatibility and pagination
            .route("/api/overview", get({
                let data = Arc::clone(&data);
                move |query| get_overview(query, data)
            }))
            .route("/api/allocations", get({
                let data = Arc::clone(&data);
                move |query| get_allocations(query, data)
            }))
            .route("/api/types", get({
                let data = Arc::clone(&data);
                move |query| get_types(query, data)
            }))
            .route("/api/timeline", get({
                let data = Arc::clone(&data);
                move |query| get_timeline(query, data)
            }))
            .route("/api/performance", get({
                let data = Arc::clone(&data);
                move |query| get_performance(query, data)
            }))
            .route("/api/security", get({
                let data = Arc::clone(&data);
                move |query| get_security(query, data)
            }))
            
            // Health check
            .route("/health", get(health_check))
            
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            );

        // Serve static files if directory is specified
        if let Some(static_dir) = &self.config.static_dir {
            if static_dir.exists() {
                println!("📂 Serving static files from: {}", static_dir.display());
                app = app.nest_service("/static", ServeDir::new(static_dir));
            }
        }

        app
    }
}

// Handler functions
async fn serve_dashboard() -> Result<Html<String>, StatusCode> {
    // Use the exact same template as static HTML generation
    match std::fs::read_to_string("templates/dashboard.html") {
        Ok(template_content) => {
            // For web server mode, we don't embed JSON data - it will be loaded via API
            let html = template_content.replace("{{json_data}}", "null");
            Ok(Html(html))
        }
        Err(_) => {
            // Fallback to embedded template if file not found
            let dashboard_html = include_str!("../../templates/dashboard.html");
            let html = dashboard_html.replace("{{json_data}}", "null");
            Ok(Html(html))
        }
    }
}

async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("MemScope server is running".to_string()),
        error: None,
        pagination: None,
    })
}

// This endpoint provides the exact same data format as embedded in static HTML
async fn get_raw_json_data(data: Arc<UnifiedMemoryData>) -> Json<serde_json::Value> {
    // Convert UnifiedMemoryData back to the raw JSON format that matches static HTML
    // This ensures 100% compatibility between web server and static modes
    
    let json_data = serde_json::json!({
        "memory_analysis": {
            "allocations": data.allocations,
            "stats": data.stats
        },
        "lifetime": data.lifecycle,
        "performance": data.performance,
        "security_violations": data.security.violation_reports,
        "unsafe_ffi": data.security.violation_reports,
        "complex_types": data.complex_types,
        "variable_relationships": data.variable_relationships
    });
    
    Json(json_data)
}

async fn get_overview(
    Query(_query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<serde_json::Value>> {
    let overview = serde_json::json!({
        "stats": data.stats,
        "summary": {
            "total_allocations": data.allocations.len(),
            "total_types": data.allocations.iter()
                .filter_map(|a| a.type_name.as_ref())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "memory_efficiency": data.stats.memory_efficiency,
            "peak_memory": data.stats.peak_memory,
            "active_memory": data.stats.active_memory
        }
    });

    Json(ApiResponse {
        success: true,
        data: Some(overview),
        error: None,
        pagination: None,
    })
}

async fn get_allocations(
    Query(query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50).min(1000); // Max 1000 per page
    
    let mut allocations = data.allocations.clone();
    
    // Apply filtering
    if let Some(filter) = &query.filter {
        allocations.retain(|alloc| {
            alloc.type_name.as_ref()
                .map(|t| t.contains(filter))
                .unwrap_or(false)
        });
    }
    
    // Apply sorting
    if let Some(sort) = &query.sort {
        match sort.as_str() {
            "size" => allocations.sort_by(|a, b| b.size.cmp(&a.size)),
            "timestamp" => allocations.sort_by(|a, b| b.timestamp_alloc.cmp(&a.timestamp_alloc)),
            "type" => allocations.sort_by(|a, b| {
                a.type_name.as_ref().unwrap_or(&String::new())
                    .cmp(b.type_name.as_ref().unwrap_or(&String::new()))
            }),
            _ => {} // No sorting
        }
    }
    
    let total = allocations.len();
    let total_pages = (total + per_page - 1) / per_page;
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    
    let page_allocations: Vec<serde_json::Value> = allocations[start..end]
        .iter()
        .map(|alloc| serde_json::to_value(alloc).unwrap_or_default())
        .collect();

    Json(ApiResponse {
        success: true,
        data: Some(page_allocations),
        error: None,
        pagination: Some(PaginationInfo {
            page,
            per_page,
            total,
            total_pages,
        }),
    })
}

async fn get_types(
    Query(_query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<serde_json::Value>> {
    let mut type_stats = std::collections::HashMap::new();
    
    for alloc in &data.allocations {
        if let Some(type_name) = &alloc.type_name {
            let entry = type_stats.entry(type_name.clone()).or_insert((0usize, 0usize));
            entry.0 += alloc.size;
            entry.1 += 1;
        }
    }
    
    let types_data: Vec<serde_json::Value> = type_stats
        .into_iter()
        .map(|(type_name, (total_size, count))| {
            serde_json::json!({
                "type_name": type_name,
                "total_size": total_size,
                "count": count,
                "average_size": if count > 0 { total_size / count } else { 0 }
            })
        })
        .collect();

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "types": types_data,
            "total_types": types_data.len()
        })),
        error: None,
        pagination: None,
    })
}

async fn get_timeline(
    Query(query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(100).min(1000);
    
    let mut timeline_events: Vec<serde_json::Value> = Vec::new();
    
    // Create timeline events from allocations
    for alloc in &data.allocations {
        timeline_events.push(serde_json::json!({
            "event": "allocation",
            "timestamp": alloc.timestamp_alloc,
            "ptr": alloc.ptr,
            "size": alloc.size,
            "type_name": alloc.type_name,
            "var_name": alloc.var_name
        }));
        
        if let Some(dealloc_time) = alloc.timestamp_dealloc {
            timeline_events.push(serde_json::json!({
                "event": "deallocation",
                "timestamp": dealloc_time,
                "ptr": alloc.ptr,
                "size": alloc.size,
                "type_name": alloc.type_name,
                "var_name": alloc.var_name
            }));
        }
    }
    
    // Sort by timestamp
    timeline_events.sort_by(|a, b| {
        let a_ts = a["timestamp"].as_u64().unwrap_or(0);
        let b_ts = b["timestamp"].as_u64().unwrap_or(0);
        a_ts.cmp(&b_ts)
    });
    
    let total = timeline_events.len();
    let total_pages = (total + per_page - 1) / per_page;
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    
    let page_events = timeline_events[start..end].to_vec();

    Json(ApiResponse {
        success: true,
        data: Some(page_events),
        error: None,
        pagination: Some(PaginationInfo {
            page,
            per_page,
            total,
            total_pages,
        }),
    })
}

async fn get_performance(
    Query(_query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::to_value(&data.performance).unwrap_or_default()),
        error: None,
        pagination: None,
    })
}

async fn get_security(
    Query(_query): Query<PaginationQuery>,
    data: Arc<UnifiedMemoryData>
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::to_value(&data.security).unwrap_or_default()),
        error: None,
        pagination: None,
    })
}