// Core export modules
pub mod adaptive_performance;
pub mod analysis_engine;
pub mod batch_processor;
pub mod binary;
pub mod complex_type_export;
pub mod config_optimizer;
pub mod data_localizer;
pub mod enhanced_json_exporter;
pub mod error_handling;
pub mod error_recovery;
pub mod export_enhanced;
pub mod export_modes;
pub mod fast_export_coordinator;
pub mod fixed_hybrid_template;
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

// Lifecycle exporter for detailed ownership tracking
pub mod lifecycle_exporter;
pub use lifecycle_exporter::{
    export_lifecycle_data, LifecycleExportConfig, LifecycleExportError, LifecycleExporter,
};

// Main public API - Unified entry point for all export operations
pub mod api;
pub use api::{
    export_comprehensive,
    export_fast,
    export_user_variables_binary,
    // High-level convenience functions
    export_user_variables_json,
    // Core export types
    ExportConfig,
    ExportStats,
    Exporter,
};
