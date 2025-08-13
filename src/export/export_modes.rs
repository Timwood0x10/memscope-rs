//! Export mode system
//!
//! Provides Fast and Normal export modes using Rust Futures for asynchronous export operations

use crate::core::types::TrackingResult;
use crate::export::fast_export_coordinator::{CompleteExportStats, FastExportCoordinator};
use crate::export::quality_validator::{
    ExportConfig, ExportMode, ExportModeManager, QualityValidator, ValidationResult,
    ValidationTiming,
};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

// ExportMode is now defined in quality_validator.rs

/// Fast Future result type
pub type FastExportResult = TrackingResult<CompleteExportStats>;

/// Normal Future result type  
pub type NormalExportResult = TrackingResult<(CompleteExportStats, ValidationResult)>;

/// Export result type
#[derive(Debug)]
pub enum ExportOutcome {
    /// Fast export result (no validation)
    Fast(CompleteExportStats),
    /// Normal export result (with validation)
    WithValidation(CompleteExportStats, ValidationResult),
}

/// Fast Future: pure export, no validation
pub fn export_fast<P: AsRef<Path>>(
    output_path: P,
) -> Pin<Box<dyn Future<Output = FastExportResult> + Send>> {
    let path = output_path.as_ref().to_path_buf();

    Box::pin(async move {
        tracing::info!("🚀 Starting fast export mode (no validation)");

        // create fast mode coordinator
        let mut coordinator = FastExportCoordinator::new_fast_mode();

        // pure export, skip all validation
        let stats = coordinator.export_without_validation(&path).await?;

        tracing::info!(
            "✅ Fast export completed: {} allocations, {:.2} MB",
            stats.parallel_processing.total_allocations,
            stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0
        );

        Ok(stats)
    })
}

/// Export with validation: export first, then validate
pub fn export_with_validation<P: AsRef<Path>>(
    output_path: P,
) -> Pin<Box<dyn Future<Output = NormalExportResult> + Send>> {
    let path = output_path.as_ref().to_path_buf();

    Box::pin(async move {
        tracing::info!("🚀 Starting export with validation mode (export first, then validate)");

        // Step 1: Complete export
        tracing::info!("📝 Step 1: Executing export...");
        let mut coordinator = FastExportCoordinator::new_normal_mode();
        let stats = coordinator.export_without_validation(&path).await?;

        tracing::info!(
            "✅ Export completed: {} allocations, {:.2} MB",
            stats.parallel_processing.total_allocations,
            stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0
        );

        // Step 2: Validate after export completion
        tracing::info!("🔍 Step 2: Executing validation...");
        let mut validator = QualityValidator::new_default();
        let validation_result = validator.validate_file_async(&path).await?;

        tracing::info!(
            "✅ Validation completed: {}",
            if validation_result.is_valid {
                "PASSED"
            } else {
                "FAILED"
            }
        );

        Ok((stats, validation_result))
    })
}

/// Export coordinator - unified export interface with configuration management
pub struct ExportCoordinator {
    /// Export configuration
    config: ExportConfig,
    /// Mode manager for automatic optimizations
    mode_manager: ExportModeManager,
}

impl ExportCoordinator {
    /// Create new export coordinator with configuration
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            mode_manager: ExportModeManager::new(),
        }
    }

    /// Create coordinator with fast mode configuration
    pub fn new_fast() -> Self {
        Self::new(ExportConfig::fast())
    }

    /// Create coordinator with slow mode configuration  
    pub fn new_slow() -> Self {
        Self::new(ExportConfig::slow())
    }

    /// Create coordinator with auto mode configuration
    pub fn new_auto() -> Self {
        Self::new(ExportConfig::auto())
    }

    /// Create coordinator with automatic configuration based on data size
    pub fn new_auto_sized(data_size: usize) -> Self {
        let mode_manager = ExportModeManager::new();
        let config = mode_manager.create_auto_config(data_size);
        Self::new(config)
    }

    /// Execute export with current configuration
    pub async fn export<P: AsRef<Path>>(&self, output_path: P) -> TrackingResult<ExportOutcome> {
        match (&self.config.mode, &self.config.validation_timing) {
            (ExportMode::Fast, _) | (_, ValidationTiming::Disabled) => {
                let stats = export_fast(output_path).await?;
                Ok(ExportOutcome::Fast(stats))
            }
            (ExportMode::Slow, ValidationTiming::Inline) => {
                // For inline validation in slow mode, we still use the normal flow
                // but with more comprehensive validation
                let (stats, validation) = export_with_validation(output_path).await?;
                Ok(ExportOutcome::WithValidation(stats, validation))
            }
            _ => {
                // Default to deferred validation
                let (stats, validation) = export_with_validation(output_path).await?;
                Ok(ExportOutcome::WithValidation(stats, validation))
            }
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &ExportConfig {
        &self.config
    }

    /// Update configuration with validation and optimization
    pub fn update_config(
        &mut self,
        mut config: ExportConfig,
        data_size: Option<usize>,
    ) -> Vec<String> {
        let mut warnings = config.validate_and_fix();

        if let Some(size) = data_size {
            let (optimized_config, optimization_warnings) =
                self.mode_manager.optimize_config(config, size);
            config = optimized_config;
            warnings.extend(optimization_warnings);
        }

        self.config = config;
        warnings
    }

    /// Get mode manager for advanced configuration
    pub fn mode_manager(&self) -> &ExportModeManager {
        &self.mode_manager
    }
}
