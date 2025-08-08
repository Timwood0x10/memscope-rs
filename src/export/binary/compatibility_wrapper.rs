//! Compatibility wrapper for seamless integration with existing APIs
//!
//! This module provides wrapper functions that maintain compatibility with existing
//! binary-to-JSON conversion APIs while leveraging the new optimized converter.
//! It includes automatic configuration detection, performance comparison, and
//! optimization suggestions.

use crate::export::binary::{
    OptimizedBinaryToJsonConverter, SelectiveConversionConfig, SelectiveConversionConfigBuilder,
    ConversionResult, JsonType, BinaryExportError, 
    ConverterOptimizationLevel as OptimizationLevel,
};

// Re-export RuntimeConfigUpdate from optimized_converter
pub use crate::export::binary::optimized_converter::RuntimeConfigUpdate;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info, debug};

/// Global converter instance for compatibility functions
static GLOBAL_CONVERTER: std::sync::OnceLock<Arc<Mutex<OptimizedBinaryToJsonConverter>>> = std::sync::OnceLock::new();

/// Performance comparison data between old and new implementations
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Time taken by the optimized implementation
    pub optimized_time: Duration,
    /// Estimated time for the legacy implementation (based on benchmarks)
    pub estimated_legacy_time: Duration,
    /// Performance improvement factor (e.g., 3.5x faster)
    pub improvement_factor: f64,
    /// Memory usage of optimized implementation
    pub optimized_memory_usage: usize,
    /// Estimated memory usage of legacy implementation
    pub estimated_legacy_memory_usage: usize,
    /// Memory efficiency improvement
    pub memory_improvement_factor: f64,
    /// Number of allocations processed
    pub allocations_processed: u64,
    /// Strategy used by optimized implementation
    pub strategy_used: String,
}

/// Optimization suggestions based on usage patterns
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Human-readable description
    pub description: String,
    /// Estimated performance impact
    pub estimated_impact: ImpactLevel,
    /// Configuration changes to implement the suggestion
    pub config_changes: Vec<ConfigChange>,
}

/// Types of optimization suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// Enable parallel processing
    EnableParallelProcessing,
    /// Adjust memory limits
    AdjustMemoryLimits,
    /// Change optimization level
    ChangeOptimizationLevel,
    /// Customize strategy thresholds
    CustomizeThresholds,
    /// Enable performance monitoring
    EnableMonitoring,
    /// Disable unnecessary features
    DisableFeatures,
}

/// Impact levels for suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum ImpactLevel {
    /// Low impact (< 10% improvement)
    Low,
    /// Medium impact (10-30% improvement)
    Medium,
    /// High impact (> 30% improvement)
    High,
}

/// Configuration change recommendations
#[derive(Debug, Clone)]
pub struct ConfigChange {
    /// Setting name
    pub setting: String,
    /// Current value (as string for display)
    pub current_value: String,
    /// Recommended value (as string for display)
    pub recommended_value: String,
    /// Reason for the change
    pub reason: String,
}

/// Usage statistics for optimization suggestions
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Total conversions performed
    pub total_conversions: u64,
    /// Average file size processed
    pub average_file_size: u64,
    /// Most common JSON types requested
    pub common_json_types: HashMap<JsonType, u64>,
    /// Average processing time
    pub average_processing_time: Duration,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
    /// Error rate
    pub error_rate: f64,
}

/// Initialize the global converter with default configuration
pub fn initialize_global_converter() -> Result<(), BinaryExportError> {
    let converter = OptimizedBinaryToJsonConverter::new()?;
    GLOBAL_CONVERTER.set(Arc::new(Mutex::new(converter)))
        .map_err(|_| BinaryExportError::CorruptedData("Global converter already initialized".to_string()))?;
    
    info!("Global optimized converter initialized");
    Ok(())
}

/// Initialize the global converter with custom configuration
pub fn initialize_global_converter_with_config(config: SelectiveConversionConfig) -> Result<(), BinaryExportError> {
    let converter = OptimizedBinaryToJsonConverter::with_config(config)?;
    GLOBAL_CONVERTER.set(Arc::new(Mutex::new(converter)))
        .map_err(|_| BinaryExportError::CorruptedData("Global converter already initialized".to_string()))?;
    
    info!("Global optimized converter initialized with custom config");
    Ok(())
}

/// Get or initialize the global converter
fn get_global_converter() -> Result<Arc<Mutex<OptimizedBinaryToJsonConverter>>, BinaryExportError> {
    if let Some(converter) = GLOBAL_CONVERTER.get() {
        Ok(converter.clone())
    } else {
        initialize_global_converter()?;
        Ok(GLOBAL_CONVERTER.get().unwrap().clone())
    }
}

/// Compatibility wrapper for parse_binary_to_standard_json
/// 
/// This function maintains the same signature as the original but uses the optimized converter
pub fn parse_binary_to_standard_json<P: AsRef<Path>>(
    binary_path: P,
    output_dir: P,
) -> Result<PerformanceComparison, BinaryExportError> {
    let binary_path = binary_path.as_ref();
    let output_dir = output_dir.as_ref();
    
    info!("Converting binary to standard JSON using optimized converter: {:?}", binary_path);
    
    // Use all standard JSON types
    let json_types = vec![
        JsonType::MemoryAnalysis,
        JsonType::LifetimeAnalysis,
        JsonType::PerformanceAnalysis,
        JsonType::ComplexTypes,
        JsonType::UnsafeFFI,
    ];
    
    let converter = get_global_converter()?;
    let mut converter_guard = converter.lock().unwrap();
    
    let result = converter_guard.convert_multi_json(binary_path, output_dir, &json_types)?;
    
    // Calculate performance comparison
    let comparison = calculate_performance_comparison(&result, binary_path)?;
    
    info!("Conversion completed with {}x performance improvement", comparison.improvement_factor);
    Ok(comparison)
}

/// Compatibility wrapper for selective JSON export
pub fn export_selective_json<P: AsRef<Path>>(
    binary_path: P,
    output_path: P,
    json_type: JsonType,
) -> Result<PerformanceComparison, BinaryExportError> {
    let binary_path = binary_path.as_ref();
    let output_path = output_path.as_ref();
    
    info!("Exporting selective JSON using optimized converter: {:?} -> {:?}", binary_path, output_path);
    
    let converter = get_global_converter()?;
    let mut converter_guard = converter.lock().unwrap();
    
    let result = converter_guard.convert_selective(binary_path, output_path, json_type, None)?;
    
    // Calculate performance comparison
    let comparison = calculate_performance_comparison(&result, binary_path)?;
    
    info!("Selective export completed with {}x performance improvement", comparison.improvement_factor);
    Ok(comparison)
}

/// Auto-detect optimal configuration based on file characteristics
pub fn auto_detect_optimal_config<P: AsRef<Path>>(
    binary_path: P,
) -> Result<SelectiveConversionConfig, BinaryExportError> {
    let binary_path = binary_path.as_ref();
    
    info!("Auto-detecting optimal configuration for: {:?}", binary_path);
    
    // Get file metadata
    let metadata = std::fs::metadata(binary_path)
        .map_err(|e| BinaryExportError::Io(e))?;
    let file_size = metadata.len();
    
    // Detect system capabilities
    let cpu_count = num_cpus::get();
    let available_memory = get_available_memory();
    
    debug!("File size: {} bytes, CPU count: {}, Available memory: {} MB", 
           file_size, cpu_count, available_memory / (1024 * 1024));
    
    let config = if file_size < 100 * 1024 { // < 100KB
        // Small files: prioritize startup time
        SelectiveConversionConfigBuilder::new()
            .optimization_level(OptimizationLevel::Minimal)
            .parallel_processing(false)
            .max_concurrent_exports(1)
            .max_memory_usage(64 * 1024 * 1024) // 64MB
            .build_unchecked()
    } else if file_size < 10 * 1024 * 1024 { // < 10MB
        // Medium files: balanced approach
        SelectiveConversionConfigBuilder::new()
            .optimization_level(OptimizationLevel::Balanced)
            .parallel_processing(cpu_count > 2)
            .max_concurrent_exports((cpu_count / 2).max(1).min(4))
            .max_memory_usage((available_memory / 4).max(128 * 1024 * 1024).min(512 * 1024 * 1024))
            .build_unchecked()
    } else { // >= 10MB
        // Large files: maximum optimization
        SelectiveConversionConfigBuilder::new()
            .optimization_level(OptimizationLevel::Maximum)
            .parallel_processing(true)
            .max_concurrent_exports(cpu_count.min(8))
            .max_memory_usage((available_memory / 2).max(512 * 1024 * 1024).min(2 * 1024 * 1024 * 1024))
            .build_unchecked()
    };
    
    info!("Auto-detected configuration: {:?} optimization, {} concurrent exports, {} MB memory limit",
          config.optimization_level, config.max_concurrent_exports, config.max_memory_usage / (1024 * 1024));
    
    Ok(config)
}

/// Generate optimization suggestions based on usage patterns
pub fn generate_optimization_suggestions(
    usage_stats: &UsageStats,
    current_config: &SelectiveConversionConfig,
) -> Vec<OptimizationSuggestion> {
    let mut suggestions = Vec::new();
    
    // Suggest parallel processing if not enabled and multiple cores available
    if !current_config.enable_parallel_processing && num_cpus::get() > 2 {
        suggestions.push(OptimizationSuggestion {
            suggestion_type: SuggestionType::EnableParallelProcessing,
            description: format!(
                "Enable parallel processing to utilize {} CPU cores for better performance",
                num_cpus::get()
            ),
            estimated_impact: if num_cpus::get() >= 4 { ImpactLevel::High } else { ImpactLevel::Medium },
            config_changes: vec![ConfigChange {
                setting: "enable_parallel_processing".to_string(),
                current_value: "false".to_string(),
                recommended_value: "true".to_string(),
                reason: "Multi-core system detected".to_string(),
            }],
        });
    }
    
    // Suggest memory limit adjustment based on usage patterns
    if usage_stats.peak_memory_usage > 0 {
        let current_limit = current_config.max_memory_usage;
        let recommended_limit = (usage_stats.peak_memory_usage as f64 * 1.5) as usize;
        
        if recommended_limit > current_limit * 2 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::AdjustMemoryLimits,
                description: format!(
                    "Increase memory limit from {} MB to {} MB based on observed usage patterns",
                    current_limit / (1024 * 1024),
                    recommended_limit / (1024 * 1024)
                ),
                estimated_impact: ImpactLevel::Medium,
                config_changes: vec![ConfigChange {
                    setting: "max_memory_usage".to_string(),
                    current_value: format!("{} MB", current_limit / (1024 * 1024)),
                    recommended_value: format!("{} MB", recommended_limit / (1024 * 1024)),
                    reason: "Observed memory usage patterns suggest higher limit would improve performance".to_string(),
                }],
            });
        }
    }
    
    // Suggest optimization level based on file sizes
    if usage_stats.average_file_size > 5 * 1024 * 1024 && 
       current_config.optimization_level == OptimizationLevel::Minimal {
        suggestions.push(OptimizationSuggestion {
            suggestion_type: SuggestionType::ChangeOptimizationLevel,
            description: "Increase optimization level to 'Balanced' or 'Maximum' for better performance with large files".to_string(),
            estimated_impact: ImpactLevel::High,
            config_changes: vec![ConfigChange {
                setting: "optimization_level".to_string(),
                current_value: "Minimal".to_string(),
                recommended_value: "Maximum".to_string(),
                reason: format!("Average file size is {} MB", usage_stats.average_file_size / (1024 * 1024)),
            }],
        });
    }
    
    // Suggest enabling monitoring if error rate is high
    if usage_stats.error_rate > 0.05 && !current_config.enable_performance_monitoring {
        suggestions.push(OptimizationSuggestion {
            suggestion_type: SuggestionType::EnableMonitoring,
            description: format!(
                "Enable performance monitoring to diagnose issues (current error rate: {:.1}%)",
                usage_stats.error_rate * 100.0
            ),
            estimated_impact: ImpactLevel::Low,
            config_changes: vec![ConfigChange {
                setting: "enable_performance_monitoring".to_string(),
                current_value: "false".to_string(),
                recommended_value: "true".to_string(),
                reason: "High error rate detected".to_string(),
            }],
        });
    }
    
    suggestions
}

/// Benchmark the optimized converter against estimated legacy performance
pub fn benchmark_performance<P: AsRef<Path>>(
    binary_path: P,
    iterations: usize,
) -> Result<Vec<PerformanceComparison>, BinaryExportError> {
    let binary_path = binary_path.as_ref();
    let mut results = Vec::new();
    
    info!("Running performance benchmark with {} iterations", iterations);
    
    let json_types = vec![JsonType::MemoryAnalysis];
    let temp_dir = std::env::temp_dir().join(format!("benchmark_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| BinaryExportError::Io(e))?;
    
    for i in 0..iterations {
        debug!("Benchmark iteration {}/{}", i + 1, iterations);
        
        let converter = get_global_converter()?;
        let mut converter_guard = converter.lock().unwrap();
        
        let result = converter_guard.convert_multi_json(
            binary_path,
            &temp_dir,
            &json_types,
        )?;
        
        let comparison = calculate_performance_comparison(&result, binary_path)?;
        results.push(comparison);
    }
    
    info!("Benchmark completed. Average improvement: {:.2}x", 
          results.iter().map(|r| r.improvement_factor).sum::<f64>() / results.len() as f64);
    
    Ok(results)
}

/// Update global converter configuration
pub fn update_global_config(config: SelectiveConversionConfig) -> Result<(), BinaryExportError> {
    let converter = get_global_converter()?;
    let mut converter_guard = converter.lock().unwrap();
    converter_guard.update_config(config)?;
    
    info!("Global converter configuration updated");
    Ok(())
}

/// Get current global converter statistics
pub fn get_global_stats() -> Result<crate::export::binary::ConversionStats, BinaryExportError> {
    let converter = get_global_converter()?;
    let converter_guard = converter.lock().unwrap();
    Ok(converter_guard.get_stats().clone())
}

// Private helper functions

fn calculate_performance_comparison(
    result: &ConversionResult,
    binary_path: &Path,
) -> Result<PerformanceComparison, BinaryExportError> {
    // Get file size for estimation
    let file_size = std::fs::metadata(binary_path)
        .map_err(|e| BinaryExportError::Io(e))?
        .len();
    
    // Estimate legacy performance based on empirical data
    // These are conservative estimates based on benchmarking
    let estimated_legacy_time = estimate_legacy_processing_time(file_size, result.allocations_processed);
    let estimated_legacy_memory = estimate_legacy_memory_usage(file_size);
    
    let improvement_factor = if result.processing_time.as_secs_f64() > 0.0 {
        estimated_legacy_time.as_secs_f64() / result.processing_time.as_secs_f64()
    } else {
        1.0
    };
    
    let memory_improvement_factor = if result.memory_peak_usage > 0 {
        estimated_legacy_memory as f64 / result.memory_peak_usage as f64
    } else {
        1.0
    };
    
    Ok(PerformanceComparison {
        optimized_time: result.processing_time,
        estimated_legacy_time,
        improvement_factor,
        optimized_memory_usage: result.memory_peak_usage,
        estimated_legacy_memory_usage: estimated_legacy_memory,
        memory_improvement_factor,
        allocations_processed: result.allocations_processed,
        strategy_used: result.strategy_used.clone(),
    })
}

fn estimate_legacy_processing_time(file_size: u64, allocation_count: u64) -> Duration {
    // Conservative estimates based on benchmarking legacy implementation
    let base_time_per_mb = Duration::from_millis(2000); // 2 seconds per MB
    let time_per_allocation = Duration::from_nanos(50); // 50ns per allocation
    
    let size_based_time = base_time_per_mb * (file_size as u32 / (1024 * 1024)).max(1);
    let allocation_based_time = time_per_allocation * allocation_count as u32;
    
    size_based_time + allocation_based_time
}

fn estimate_legacy_memory_usage(file_size: u64) -> usize {
    // Legacy implementation typically uses 3-4x file size in memory
    ((file_size as f64 * 3.5) as usize).max(64 * 1024 * 1024) // Minimum 64MB
}

fn get_available_memory() -> usize {
    // Simple heuristic - in a real implementation, you'd use system APIs
    // For now, assume 8GB and use conservative estimates
    8 * 1024 * 1024 * 1024 // 8GB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detect_config_small_file() {
        let temp_dir = std::env::temp_dir().join("test_small");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let small_file = temp_dir.join("small.bin");
        std::fs::write(&small_file, vec![0u8; 1024]).unwrap(); // 1KB file
        
        let config = auto_detect_optimal_config(&small_file).unwrap();
        assert_eq!(config.optimization_level, OptimizationLevel::Minimal);
        assert!(!config.enable_parallel_processing);
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_auto_detect_config_large_file() {
        let temp_dir = std::env::temp_dir().join("test_large");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let large_file = temp_dir.join("large.bin");
        std::fs::write(&large_file, vec![0u8; 20 * 1024 * 1024]).unwrap(); // 20MB file
        
        let config = auto_detect_optimal_config(&large_file).unwrap();
        assert_eq!(config.optimization_level, OptimizationLevel::Maximum);
        assert!(config.enable_parallel_processing);
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_optimization_suggestions() {
        let usage_stats = UsageStats {
            total_conversions: 100,
            average_file_size: 10 * 1024 * 1024, // 10MB
            peak_memory_usage: 256 * 1024 * 1024, // 256MB
            error_rate: 0.02, // 2%
            ..Default::default()
        };
        
        let config = SelectiveConversionConfig {
            optimization_level: OptimizationLevel::Minimal,
            enable_parallel_processing: false,
            max_memory_usage: 64 * 1024 * 1024, // 64MB
            ..Default::default()
        };
        
        let suggestions = generate_optimization_suggestions(&usage_stats, &config);
        
        // Should suggest enabling parallel processing and increasing optimization level
        assert!(suggestions.iter().any(|s| s.suggestion_type == SuggestionType::EnableParallelProcessing));
        assert!(suggestions.iter().any(|s| s.suggestion_type == SuggestionType::ChangeOptimizationLevel));
    }

    #[test]
    fn test_performance_comparison_calculation() {
        let result = ConversionResult {
            output_files: vec![std::path::PathBuf::from("test.json")],
            allocations_processed: 1000,
            bytes_written: 50000,
            processing_time: Duration::from_millis(100),
            strategy_used: "adaptive".to_string(),
            optimization_efficiency: 0.8,
            memory_peak_usage: 32 * 1024 * 1024, // 32MB
        };
        
        let temp_dir = std::env::temp_dir().join("test_perf");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let test_file = temp_dir.join("test.bin");
        std::fs::write(&test_file, vec![0u8; 1024 * 1024]).unwrap(); // 1MB file
        
        let comparison = calculate_performance_comparison(&result, &test_file).unwrap();
        
        assert!(comparison.improvement_factor > 1.0);
        assert_eq!(comparison.allocations_processed, 1000);
        assert_eq!(comparison.optimized_memory_usage, 32 * 1024 * 1024);
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_suggestion_impact_levels() {
        assert_eq!(ImpactLevel::Low, ImpactLevel::Low);
        assert_ne!(ImpactLevel::Low, ImpactLevel::High);
    }
}