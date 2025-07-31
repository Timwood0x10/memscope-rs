//! Optimized JSON export functionality
//! This provides high-performance JSON export with various optimization options

use crate::core::types::TrackingResult;

/// Export memory data to JSON format
pub fn export_memory_to_json<P: AsRef<std::path::Path>>(
    tracker: &crate::core::tracker::MemoryTracker,
    path: P,
) -> TrackingResult<()> {
    let allocations = tracker.get_all_active_allocations()?;
    let json = serde_json::to_string_pretty(&allocations)
        .map_err(|e| crate::core::types::TrackingError::SerializationError(e.to_string()))?;
    
    std::fs::write(path, json)
        .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;
    
    Ok(())
}

/// Optimization levels for export
#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    Low,
    Medium,
    High,
}

/// Export options for optimized JSON export
#[derive(Debug, Clone)]
pub struct OptimizedExportOptions {
    pub include_system_allocations: bool,
    pub verbose_logging: bool,
    pub buffer_size: usize,
    pub compress_output: bool,
    // Additional fields that were missing
    pub enable_fast_export_mode: bool,
    pub parallel_processing: bool,
    pub use_streaming_writer: bool,
    pub enable_schema_validation: bool,
    pub enable_enhanced_ffi_analysis: bool,
    pub enable_boundary_event_processing: bool,
    pub enable_memory_passport_tracking: bool,
    pub enable_security_analysis: bool,
    pub enable_adaptive_optimization: bool,
    pub batch_size: usize,
}

impl Default for OptimizedExportOptions {
    fn default() -> Self {
        Self {
            include_system_allocations: false,
            verbose_logging: false,
            buffer_size: 64 * 1024,
            compress_output: false,
            enable_fast_export_mode: false,
            parallel_processing: false,
            use_streaming_writer: false,
            enable_schema_validation: true,
            enable_enhanced_ffi_analysis: true,
            enable_boundary_event_processing: true,
            enable_memory_passport_tracking: true,
            enable_security_analysis: true,
            enable_adaptive_optimization: true,
            batch_size: 1000,
        }
    }
}

impl OptimizedExportOptions {
    /// Create options with specific optimization level
    pub fn with_optimization_level(level: OptimizationLevel) -> Self {
        let mut options = Self::default();
        match level {
            OptimizationLevel::Low => {
                options.enable_fast_export_mode = true;
                options.parallel_processing = false;
                options.use_streaming_writer = false;
                options.enable_schema_validation = false;
                options.enable_enhanced_ffi_analysis = false;
                options.enable_boundary_event_processing = false;
                options.enable_memory_passport_tracking = false;
                options.enable_security_analysis = false;
                options.enable_adaptive_optimization = false;
                options.batch_size = 10000;
            }
            OptimizationLevel::Medium => {
                options.enable_fast_export_mode = true;
                options.parallel_processing = true;
                options.use_streaming_writer = true;
                options.enable_schema_validation = false;
                options.batch_size = 5000;
            }
            OptimizationLevel::High => {
                // Use default values (all features enabled)
            }
        }
        options
    }

    /// Enable fast export mode
    pub fn fast_export_mode(mut self, enabled: bool) -> Self {
        self.enable_fast_export_mode = enabled;
        self
    }
}