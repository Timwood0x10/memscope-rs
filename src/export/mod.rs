pub mod adaptive_performance;
pub mod analysis_engine;
pub mod batch_processor;
pub mod binary;

// Unified API - the new clean interface
pub mod unified_export_api;

// Re-export the unified API for easy access
pub use unified_export_api::{
    UnifiedExporter, ExportConfig as LegacyExportConfig, ExportStats as LegacyExportStats,
    export_user_variables_json, export_user_variables_binary, 
    export_fast, export_comprehensive
};

// New unified API - recommended for new code
pub use new_unified_api::{
    MemScopeExporter, ExportConfig, ExportStats, OptimizationLevel,
    quick_export_json, quick_export_binary
};
pub mod complex_type_export;
pub mod config_optimizer;
pub mod data_localizer;
pub mod error_handling;
pub mod error_recovery;
pub mod export_enhanced;
pub mod export_modes;
pub mod fast_export_coordinator;
pub mod high_speed_buffered_writer;
pub mod html_export;
pub mod optimized_json_export;
pub mod parallel_shard_processor;
// pub mod performance_benchmark; // Removed - test code should not be in main library
// pub mod performance_comparison; // Removed - test code should not be in main library
// pub mod performance_testing; // Removed - test code should not be in main library
pub mod progress_monitor;
pub mod quality_validator;
pub mod schema_validator;
pub mod streaming_json_writer;
pub mod system_optimizer;
pub mod visualization;
pub mod new_unified_api;
