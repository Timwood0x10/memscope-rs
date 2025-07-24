//! MemScope web server implementation
//!
//! Provides HTTP API and web interface for interactive memory analysis

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::CorsLayer;

use crate::cli::commands::html_from_json::data_normalizer::UnifiedMemoryData;
use super::handlers;
use super::api;

/// MemScope web server state
#[derive(Debug)]
pub struct ServerState {
    /// Unified memory analysis data
    pub memory_data: Arc<UnifiedMemoryData>,
    /// Pre-computed indexes for fast queries
    pub indexes: Arc<DataIndexes>,
    /// Server configuration
    pub config: ServerConfig,
}

/// Pre-computed data indexes for fast queries
#[derive(Debug)]
pub struct DataIndexes {
    /// Variable name to allocation indices mapping
    pub variable_index: HashMap<String, Vec<usize>>,
    /// Type name to allocation indices mapping
    pub type_index: HashMap<String, Vec<usize>>,
    /// Timeline index sorted by timestamp
    pub timeline_index: std::collections::BTreeMap<u64, Vec<usize>>,
    /// Scope to allocation indices mapping
    pub scope_index: HashMap<String, Vec<usize>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Enable CORS for development
    pub enable_cors: bool,
    /// Static files directory
    pub static_dir: Option<String>,
    /// Enable request logging
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            enable_cors: true,
            static_dir: None,
            enable_logging: true,
        }
    }
}

/// Main MemScope web server
pub struct MemScopeServer {
    state: Arc<ServerState>,
}

impl MemScopeServer {
    /// Create a new MemScope server
    pub fn new(memory_data: UnifiedMemoryData, config: ServerConfig) -> Self {
        println!("🔧 Building data indexes for fast queries...");
        let indexes = Self::build_indexes(&memory_data);
        
        let state = Arc::new(ServerState {
            memory_data: Arc::new(memory_data),
            indexes: Arc::new(indexes),
            config,
        });
        
        Self { state }
    }
    
    /// Start the web server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let port = self.state.config.port;
        let app = self.create_router();
        
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await?;
        
        println!("🌐 MemScope server starting at http://localhost:{}", port);
        println!("📊 Loaded {} allocations", self.state.memory_data.allocations.len());
        println!("🔍 Variable index: {} entries", self.state.indexes.variable_index.len());
        println!("🏷️  Type index: {} entries", self.state.indexes.type_index.len());
        println!("⏱️  Timeline index: {} entries", self.state.indexes.timeline_index.len());
        println!();
        println!("🎯 Available endpoints:");
        println!("   GET  /                              - Main dashboard");
        println!("   GET  /api/overview                  - Memory overview");
        println!("   GET  /api/variables                 - Variables list");
        println!("   GET  /api/variables/:name           - Variable details");
        println!("   GET  /api/variables/:name/timeline  - Variable timeline");
        println!("   GET  /api/timeline                  - Timeline events");
        println!("   GET  /api/allocation-distribution   - Allocation size distribution");
        println!("   GET  /api/unsafe-ffi                - Unsafe/FFI analysis");
        println!("   GET  /api/performance               - Performance metrics");
        println!("   GET  /api/search                    - Search variables/types");
        println!();
        println!("Press Ctrl+C to stop the server");
        
        axum::serve(listener, app).await?;
        Ok(())
    }
    
    /// Create the router with all routes
    fn create_router(&self) -> Router {
        let mut router = Router::new()
            // Main dashboard
            .route("/", get(handlers::index))
            
            // API routes
            .route("/api/overview", get(api::overview))
            .route("/api/variables", get(api::variables_list))
            .route("/api/variables/:name", get(api::variable_details))
            .route("/api/variables/:name/timeline", get(api::variable_timeline))
            .route("/api/variables/:name/relationships", get(api::variable_relationships))
            .route("/api/timeline", get(api::timeline_events))
            .route("/api/unsafe-ffi", get(api::unsafe_ffi_analysis))
            .route("/api/performance", get(api::performance_metrics))
            .route("/api/allocation-distribution", get(api::allocation_distribution))
            .route("/api/search", get(api::search))
            
            // Health check
            .route("/health", get(health_check))
            
            .with_state(self.state.clone());
        
        // Add CORS if enabled
        if self.state.config.enable_cors {
            router = router.layer(CorsLayer::permissive());
        }
        
        // Serve static files if directory is specified
        if let Some(static_dir) = &self.state.config.static_dir {
            router = router.nest_service("/static", ServeDir::new(static_dir));
        }
        
        router
    }
    
    /// Build data indexes for fast queries
    fn build_indexes(memory_data: &UnifiedMemoryData) -> DataIndexes {
        let mut variable_index = HashMap::new();
        let mut type_index = HashMap::new();
        let mut timeline_index = std::collections::BTreeMap::new();
        let mut scope_index = HashMap::new();
        
        for (i, allocation) in memory_data.allocations.iter().enumerate() {
            // Index by variable name
            if let Some(var_name) = &allocation.var_name {
                variable_index
                    .entry(var_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
            
            // Index by type name
            if let Some(type_name) = &allocation.type_name {
                type_index
                    .entry(type_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
            
            // Index by timestamp
            timeline_index
                .entry(allocation.timestamp_alloc)
                .or_insert_with(Vec::new)
                .push(i);
            
            // Index by scope
            if let Some(scope_name) = &allocation.scope_name {
                scope_index
                    .entry(scope_name.clone())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }
        
        DataIndexes {
            variable_index,
            type_index,
            timeline_index,
            scope_index,
        }
    }
}

/// Health check endpoint
async fn health_check(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let stats = &state.memory_data.stats;
    
    (
        StatusCode::OK,
        format!(
            "MemScope Server OK\nAllocations: {}\nActive Memory: {} bytes\nPeak Memory: {} bytes",
            state.memory_data.allocations.len(),
            stats.active_memory,
            stats.peak_memory
        ),
    )
}