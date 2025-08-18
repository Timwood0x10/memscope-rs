pub mod adaptive_performance;
pub mod analysis_engine;
pub mod batch_processor;
pub mod binary;

// Unified API - the new clean interface
pub mod unified_export_api;


pub use clean_unified_api::{
    MemScopeExporter,
    export_json, export_binary, export_html, export_auto
};


pub use unified_export_api::{
    export_user_variables_json, export_user_variables_binary
};


// pub use unified_export_api::{export_fast, export_comprehensive}; // 太混乱，已隐藏
// pub use new_unified_api::{...}; 
pub mod complex_type_export;
pub mod config_optimizer;
pub mod data_localizer;
pub mod error_handling;
pub mod error_recovery;
pub mod fast_export_coordinator;
pub mod export_enhanced;
pub mod export_modes;
pub mod high_speed_buffered_writer;
pub mod html_export;
pub mod optimized_json_export;
pub mod parallel_shard_processor;
pub mod progress_monitor;
pub mod quality_validator;
pub mod schema_validator;
pub mod streaming_json_writer;
pub mod system_optimizer;
pub mod visualization;
pub mod new_unified_api;
pub mod clean_unified_api;

// New lifecycle exporter for detailed ownership tracking
pub mod lifecycle_exporter;
pub use lifecycle_exporter::{
    LifecycleExporter,
    LifecycleExportConfig,
    export_lifecycle_data,
    LifecycleExportError,
    ExportStats,
};
