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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::error_handling::ValidationType;

    #[test]
    fn test_export_outcome_debug() {
        let stats = create_test_stats();
        let outcome = ExportOutcome::Fast(stats.clone());
        let debug_str = format!("{:?}", outcome);
        assert!(debug_str.contains("Fast"));

        let validation = create_test_validation_result();
        let outcome = ExportOutcome::WithValidation(stats, validation);
        let debug_str = format!("{:?}", outcome);
        assert!(debug_str.contains("WithValidation"));
    }

    #[test]
    fn test_fast_export_result_type() {
        let _result: FastExportResult = Ok(create_test_stats());
        // Just testing that the type is correctly defined
    }

    #[test]
    fn test_normal_export_result_type() {
        let _result: NormalExportResult =
            Ok((create_test_stats(), create_test_validation_result()));
        // Just testing that the type is correctly defined
    }

    #[test]
    fn test_export_coordinator_new() {
        let config = ExportConfig::fast();
        let coordinator = ExportCoordinator::new(config.clone());

        assert_eq!(coordinator.config().mode, config.mode);
        assert_eq!(
            coordinator.config().validation_timing,
            config.validation_timing
        );
    }

    #[test]
    fn test_export_coordinator_new_fast() {
        let coordinator = ExportCoordinator::new_fast();

        assert_eq!(coordinator.config().mode, ExportMode::Fast);
        assert_eq!(
            coordinator.config().validation_timing,
            ValidationTiming::Deferred
        );
    }

    #[test]
    fn test_export_coordinator_new_slow() {
        let coordinator = ExportCoordinator::new_slow();

        assert_eq!(coordinator.config().mode, ExportMode::Slow);
        assert_eq!(
            coordinator.config().validation_timing,
            ValidationTiming::Inline
        );
    }

    #[test]
    fn test_export_coordinator_new_auto() {
        let coordinator = ExportCoordinator::new_auto();

        assert_eq!(coordinator.config().mode, ExportMode::Auto);
        assert_eq!(
            coordinator.config().validation_timing,
            ValidationTiming::Deferred
        );
    }

    #[test]
    fn test_export_coordinator_new_auto_sized() {
        let coordinator = ExportCoordinator::new_auto_sized(1000);

        // The mode should be determined based on data size
        // Since the default mode manager uses Fast as default mode, it will always return Fast
        assert_eq!(coordinator.config().mode, ExportMode::Fast);
    }

    #[test]
    fn test_export_coordinator_update_config() {
        let mut coordinator = ExportCoordinator::new_fast();
        let _old_config = coordinator.config().clone();

        let new_config = ExportConfig::slow();
        let warnings = coordinator.update_config(new_config.clone(), None);

        assert_eq!(coordinator.config().mode, new_config.mode);
        assert_eq!(
            coordinator.config().validation_timing,
            new_config.validation_timing
        );
        // Should have no warnings for valid config
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_export_coordinator_update_config_with_optimization() {
        let mut coordinator = ExportCoordinator::new_fast();

        let new_config = ExportConfig::slow();
        let _warnings = coordinator.update_config(new_config.clone(), Some(1000000)); // Large data size

        assert_eq!(coordinator.config().mode, new_config.mode);
        assert_eq!(
            coordinator.config().validation_timing,
            new_config.validation_timing
        );
        // May have optimization warnings
        // Just check that the function works without panicking
    }

    #[test]
    fn test_export_coordinator_mode_manager() {
        let coordinator = ExportCoordinator::new_fast();
        let mode_manager = coordinator.mode_manager();

        // Just check that we can get the mode manager
        let (mode, threshold, _perf_threshold) = mode_manager.get_settings();
        assert_eq!(mode, ExportMode::Fast);
        assert!(threshold > 0);
    }

    // Helper functions for creating test data
    fn create_test_stats() -> CompleteExportStats {
        use crate::export::data_localizer::DataGatheringStats;
        use crate::export::high_speed_buffered_writer::WritePerformanceStats;
        use crate::export::parallel_shard_processor::ParallelProcessingStats;

        CompleteExportStats {
            data_gathering: DataGatheringStats {
                total_time_ms: 50,
                basic_data_time_ms: 30,
                ffi_data_time_ms: 10,
                scope_data_time_ms: 10,
                allocation_count: 100,
                ffi_allocation_count: 10,
                scope_count: 50,
            },
            parallel_processing: ParallelProcessingStats {
                total_allocations: 100,
                shard_count: 4,
                threads_used: 2,
                total_processing_time_ms: 100,
                avg_shard_processing_time_ms: 25.0,
                parallel_efficiency: 1.8,
                throughput_allocations_per_sec: 1000.0,
                used_parallel_processing: true,
                total_output_size_bytes: 10240,
            },
            write_performance: WritePerformanceStats {
                total_bytes_written: 10240,
                shards_written: 4,
                total_write_time_ms: 50,
                avg_write_speed_bps: 204800.0,
                flush_count: 2,
                preallocation_effective: true,
                buffer_utilization: 0.8,
            },
            total_export_time_ms: 200,
            total_allocations_processed: 100,
            total_output_size_bytes: 10240,
            overall_throughput_allocations_per_sec: 500.0,
            overall_write_speed_mbps: 0.05,
            data_gathering_percentage: 25.0,
            processing_percentage: 50.0,
            writing_percentage: 25.0,
            estimated_traditional_time_ms: 1000,
            performance_improvement_factor: 5.0,
        }
    }

    fn create_test_validation_result() -> ValidationResult {
        use crate::export::quality_validator::{IssueSeverity, IssueType, ValidationIssue};

        ValidationResult {
            is_valid: true,
            validation_type: ValidationType::DataIntegrity,
            message: "Test validation".to_string(),
            issues: vec![ValidationIssue {
                issue_type: IssueType::InconsistentData,
                description: "Test issue".to_string(),
                severity: IssueSeverity::Medium,
                affected_data: "test_data".to_string(),
                suggested_fix: Some("Fix the data".to_string()),
                auto_fixable: false,
            }],
            validation_time_ms: 10,
            data_size: 10240,
        }
    }
}
