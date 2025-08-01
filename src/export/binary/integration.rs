//! Integration module for binary export system
//!
//! This module provides high-level integration of all binary export components
//! into a unified, easy-to-use interface that handles component coordination,
//! data flow optimization, and system-wide configuration.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use crate::core::tracker::MemoryTracker;
use super::*;

/// Integrated binary export system
/// 
/// This is the main entry point for the binary export system that coordinates
/// all components and provides a simplified interface for users.
pub struct IntegratedBinaryExporter {
    /// Data collector for gathering information from all analysis modules
    data_collector: DataCollector,
    /// Data processor for handling transformations and optimizations
    data_processor: DataProcessor,
    /// Format manager for handling different output formats
    format_manager: FormatManager,
    /// Compression manager for data compression
    compression_manager: CompressionManager,
    /// Parallel processor for high-performance operations
    parallel_processor: Option<ParallelProcessor>,
    /// Memory manager for efficient memory usage
    memory_manager: Arc<MemoryManager>,
    /// Error recovery system
    error_recovery: ErrorRecovery,
    /// System configuration
    config: IntegratedConfig,
    /// Performance monitor
    performance_monitor: PerformanceMonitor,
}

/// Comprehensive configuration for the integrated system
#[derive(Debug, Clone)]
pub struct IntegratedConfig {
    /// Data collection configuration
    pub collection: crate::export::binary::data::CollectionConfig,
    /// Processing configuration
    pub processing: crate::export::binary::processor::ProcessingConfig,
    /// Compression configuration
    pub compression: CompressionConfig,
    /// Parallel processing configuration
    pub parallel: Option<crate::export::binary::parallel::ParallelConfig>,
    /// Output format
    pub output_format: OutputFormat,
    /// Export configuration
    pub export: ExportConfig,
    /// Performance monitoring enabled
    pub enable_monitoring: bool,
    /// Automatic optimization enabled
    pub auto_optimize: bool,
}

impl Default for IntegratedConfig {
    fn default() -> Self {
        Self {
            collection: crate::export::binary::data::CollectionConfig::default(),
            processing: crate::export::binary::processor::ProcessingConfig::default(),
            compression: CompressionConfig::default(),
            parallel: Some(crate::export::binary::parallel::ParallelConfig::default()),
            output_format: OutputFormat::MessagePack,
            export: ExportConfig::default(),
            enable_monitoring: true,
            auto_optimize: true,
        }
    }
}

impl IntegratedConfig {
    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            collection: crate::export::binary::data::CollectionConfig {
                max_memory_usage: 1024 * 1024 * 1024, // 1GB
                include_call_stacks: true,
                max_call_stack_depth: 64,
                enable_expensive_analysis: false,
                enable_parallel_collection: true,
                collection_timeout: Duration::from_secs(60),
                chunk_size: 1024 * 1024, // 1MB chunks
            },
            processing: crate::export::binary::processor::ProcessingConfig::fast(),
            compression: CompressionConfig::fast(),
            parallel: Some(crate::export::binary::parallel::ParallelConfig {
                worker_threads: num_cpus::get() * 2,
                enable_work_stealing: true,
                load_balancing: crate::export::binary::parallel::LoadBalancingStrategy::WorkStealing,
                ..Default::default()
            }),
            output_format: OutputFormat::CustomBinary,
            export: ExportConfig::fast(),
            enable_monitoring: true,
            auto_optimize: true,
        }
    }

    /// Create a memory-efficient configuration
    pub fn memory_efficient() -> Self {
        Self {
            collection: crate::export::binary::data::CollectionConfig {
                max_memory_usage: 64 * 1024 * 1024, // 64MB
                include_call_stacks: false,
                max_call_stack_depth: 16,
                enable_expensive_analysis: false,
                enable_parallel_collection: false,
                collection_timeout: Duration::from_secs(300),
                chunk_size: 64 * 1024, // 64KB chunks
            },
            processing: crate::export::binary::processor::ProcessingConfig::memory_efficient(),
            compression: CompressionConfig::max_compression(),
            parallel: None, // Disable parallel processing to save memory
            output_format: OutputFormat::CompressedMessagePack { level: 19 },
            export: ExportConfig::compact(),
            enable_monitoring: false,
            auto_optimize: false,
        }
    }

    /// Create a balanced configuration
    pub fn balanced() -> Self {
        Self {
            collection: crate::export::binary::data::CollectionConfig::default(),
            processing: crate::export::binary::processor::ProcessingConfig::default(),
            compression: CompressionConfig::balanced(),
            parallel: Some(crate::export::binary::parallel::ParallelConfig::default()),
            output_format: OutputFormat::MessagePack,
            export: ExportConfig::default(),
            enable_monitoring: true,
            auto_optimize: true,
        }
    }
}

/// Comprehensive export result with detailed statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedExportResult {
    /// Basic export result
    pub export_result: ExportResult,
    /// Data collection statistics
    pub collection_stats: CollectionStatistics,
    /// Processing statistics
    pub processing_stats: crate::export::binary::processor::ProcessStats,
    /// Compression statistics
    pub compression_stats: Option<CompressionStats>,
    /// Parallel processing statistics
    pub parallel_stats: Option<crate::export::binary::parallel::ParallelStats>,
    /// Overall performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Validation report
    pub validation_report: ValidationReport,
}

/// Performance metrics for the integrated system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total export time
    pub total_time: Duration,
    /// Time breakdown by component
    pub component_times: ComponentTimes,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
    /// Efficiency scores
    pub efficiency: EfficiencyMetrics,
}

/// Time breakdown by system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTimes {
    /// Data collection time
    pub collection_time: Duration,
    /// Processing time
    pub processing_time: Duration,
    /// Compression time
    pub compression_time: Duration,
    /// Format writing time
    pub format_time: Duration,
    /// I/O time
    pub io_time: Duration,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage
    pub peak_usage: usize,
    /// Average memory usage
    pub average_usage: usize,
    /// Memory efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Overall throughput (bytes/second)
    pub overall_throughput: f64,
    /// Collection throughput
    pub collection_throughput: f64,
    /// Processing throughput
    pub processing_throughput: f64,
    /// Compression throughput
    pub compression_throughput: f64,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Overall efficiency score (0.0 to 1.0)
    pub overall_efficiency: f64,
    /// CPU utilization efficiency
    pub cpu_efficiency: f64,
    /// Memory utilization efficiency
    pub memory_efficiency: f64,
    /// I/O efficiency
    pub io_efficiency: f64,
}

/// Performance monitoring system
struct PerformanceMonitor {
    /// Monitoring enabled
    enabled: bool,
    /// Start time
    start_time: Instant,
    /// Component timers
    component_timers: std::collections::HashMap<String, Instant>,
    /// Memory usage samples
    memory_samples: Vec<(Instant, usize)>,
}

impl IntegratedBinaryExporter {
    /// Create a new integrated binary exporter
    pub fn new(config: IntegratedConfig) -> Self {
        let memory_manager = Arc::new(MemoryManager::new(config.processing.max_memory_usage));
        
        let data_collector = DataCollector::new(config.collection.clone());
        let data_processor = DataProcessor::new(config.processing.clone());
        let format_manager = FormatManager::new();
        let compression_manager = CompressionManager::new(config.compression.clone());
        let parallel_processor = config.parallel.as_ref().map(|cfg| ParallelProcessor::new(cfg.clone()));
        let error_recovery = ErrorRecovery::new();
        let performance_monitor = PerformanceMonitor::new(config.enable_monitoring);

        Self {
            data_collector,
            data_processor,
            format_manager,
            compression_manager,
            parallel_processor,
            memory_manager,
            error_recovery,
            config,
            performance_monitor,
        }
    }

    /// Export memory tracking data with full integration
    pub fn export<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> Result<IntegratedExportResult, BinaryExportError> {
        let start_time = Instant::now();
        self.performance_monitor.start_monitoring();

        // Auto-optimize configuration if enabled
        if self.config.auto_optimize {
            self.auto_optimize_config(tracker)?;
        }

        // Phase 1: Data Collection
        self.performance_monitor.start_component_timer("collection");
        let unified_data = self.collect_data_with_recovery(tracker)?;
        let collection_time = self.performance_monitor.end_component_timer("collection");

        // Phase 2: Data Processing
        self.performance_monitor.start_component_timer("processing");
        let processed_data = self.process_data_with_optimization(&unified_data)?;
        let processing_time = self.performance_monitor.end_component_timer("processing");

        // Phase 3: Compression (if enabled)
        let (final_data, compression_stats, compression_time) = if self.config.compression.algorithm != CompressionAlgorithm::None {
            self.performance_monitor.start_component_timer("compression");
            let compressed = self.compress_data_with_recovery(&processed_data.data)?;
            let comp_time = self.performance_monitor.end_component_timer("compression");
            
            let stats = CompressionStats {
                original_size: processed_data.data.len() as u64,
                compressed_size: compressed.len() as u64,
                compression_ratio: compressed.len() as f64 / processed_data.data.len() as f64,
                compression_time: comp_time,
                throughput: processed_data.data.len() as f64 / comp_time.as_secs_f64(),
                algorithm: self.config.compression.algorithm,
                level: self.config.compression.level,
            };
            
            (compressed, Some(stats), comp_time)
        } else {
            (processed_data.data.clone(), None, Duration::from_millis(0))
        };

        // Phase 4: Format Writing
        self.performance_monitor.start_component_timer("format");
        let bytes_written = self.write_formatted_data(&processed_data, &final_data, &path)?;
        let format_time = self.performance_monitor.end_component_timer("format");

        // Phase 5: Validation
        let validation_report = self.validate_output(&path)?;

        let total_time = start_time.elapsed();

        // Build comprehensive result
        let result = IntegratedExportResult {
            export_result: ExportResult {
                bytes_written,
                duration: total_time,
                compression_ratio: compression_stats.as_ref().map(|s| s.compression_ratio),
                allocation_count: unified_data.allocations.allocations.len(),
                warnings: Vec::new(),
                stats: crate::export::binary::export::ExportStats {
                    collection_time,
                    serialization_time: processing_time,
                    compression_time: Some(compression_time),
                    write_time: format_time,
                    original_size: processed_data.data.len() as u64,
                    final_size: bytes_written,
                    peak_memory_usage: self.memory_manager.peak_usage(),
                },
            },
            collection_stats: self.get_collection_statistics(),
            processing_stats: self.data_processor.get_stats().into_iter().last().unwrap_or_default(),
            compression_stats,
            parallel_stats: self.parallel_processor.as_ref().map(|p| p.get_stats()),
            performance_metrics: self.calculate_performance_metrics(total_time),
            validation_report,
        };

        Ok(result)
    }

    /// Export asynchronously with full integration
    pub async fn export_async<P: AsRef<Path>>(
        &mut self,
        tracker: &MemoryTracker,
        path: P,
    ) -> Result<IntegratedExportResult, BinaryExportError> {
        // For now, delegate to synchronous version
        // In a full implementation, this would use async/await throughout
        tokio::task::spawn_blocking({
            let path = path.as_ref().to_path_buf();
            move || {
                // This would need proper async integration
                // For now, return a placeholder error
                Err(BinaryExportError::UnsupportedFeature("Async export not fully implemented".to_string()))
            }
        }).await
        .map_err(|e| BinaryExportError::InternalError(e.to_string()))?
    }

    /// Load and validate binary data with full integration
    pub fn load<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<UnifiedData, BinaryExportError> {
        let path = path.as_ref();

        // Validate file first
        let validation_report = validate_binary_file(path)?;
        if !validation_report.is_valid {
            return Err(BinaryExportError::ValidationFailed(
                format!("File validation failed: {}", validation_report.summary())
            ));
        }

        // Read and decompress if needed
        let file_data = std::fs::read(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;

        let decompressed_data = if validation_report.compression_info
            .as_ref()
            .map_or(false, |info| info.is_compressed) {
            self.compression_manager.decompress(&file_data, self.config.compression.algorithm)?
        } else {
            file_data
        };

        // Deserialize based on format
        let unified_data = match self.config.output_format {
            OutputFormat::MessagePack | OutputFormat::CompressedMessagePack { .. } => {
                rmp_serde::from_slice(&decompressed_data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?
            }
            OutputFormat::CustomBinary => {
                // Custom binary format deserialization
                bincode::deserialize(&decompressed_data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?
            }
            OutputFormat::Raw => {
                // Raw format deserialization
                bincode::deserialize(&decompressed_data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?
            }
            OutputFormat::Chunked { .. } => {
                // Chunked format would need special handling
                return Err(BinaryExportError::UnsupportedFeature("Chunked format loading not implemented".to_string()));
            }
        };

        Ok(unified_data)
    }

    /// Auto-optimize configuration based on data characteristics
    fn auto_optimize_config(&mut self, tracker: &MemoryTracker) -> Result<(), BinaryExportError> {
        let allocation_count = tracker.allocation_count();
        let estimated_data_size = allocation_count * 128; // Rough estimate

        // Optimize based on data size
        if estimated_data_size > 100 * 1024 * 1024 { // > 100MB
            // Large dataset - optimize for memory efficiency
            self.config.processing.chunk_size = 64 * 1024; // Smaller chunks
            self.config.compression.algorithm = CompressionAlgorithm::Zstd;
            self.config.compression.level = 6; // Balanced compression
            
            if self.config.parallel.is_none() {
                self.config.parallel = Some(crate::export::binary::parallel::ParallelConfig::default());
            }
        } else if estimated_data_size < 1024 * 1024 { // < 1MB
            // Small dataset - optimize for speed
            self.config.processing.chunk_size = 256 * 1024; // Larger chunks
            self.config.compression.algorithm = CompressionAlgorithm::Lz4;
            self.config.compression.level = 1; // Fast compression
            
            // Disable parallel processing for small datasets
            self.config.parallel = None;
        }

        Ok(())
    }

    /// Collect data with error recovery
    fn collect_data_with_recovery(&self, tracker: &MemoryTracker) -> Result<UnifiedData, BinaryExportError> {
        self.error_recovery.execute_with_recovery(
            || self.data_collector.collect_from_tracker(tracker),
            "data_collection"
        )
    }

    /// Process data with optimization
    fn process_data_with_optimization(&self, data: &UnifiedData) -> Result<crate::export::binary::processor::ProcessedData, BinaryExportError> {
        if let Some(ref parallel_processor) = self.parallel_processor {
            // Use parallel processing for large datasets
            if data.allocations.allocations.len() > 1000 {
                return self.data_processor.process_parallel_with_work_stealing(data);
            }
        }

        // Use batch processing for smaller datasets
        self.data_processor.process_batch(data)
    }

    /// Compress data with error recovery
    fn compress_data_with_recovery(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        self.error_recovery.execute_with_recovery(
            || {
                let mut compression_manager = self.compression_manager.clone();
                compression_manager.compress(data)
                    .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
            },
            "compression"
        )
    }

    /// Write formatted data to file
    fn write_formatted_data(
        &self,
        processed_data: &crate::export::binary::processor::ProcessedData,
        final_data: &[u8],
        path: &Path,
    ) -> Result<u64, BinaryExportError> {
        let mut file = std::fs::File::create(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;

        // Create a modified processed data with final compressed data
        let output_data = crate::export::binary::processor::ProcessedData {
            data: final_data.to_vec(),
            metadata: processed_data.metadata.clone(),
            validation_results: processed_data.validation_results.clone(),
        };

        self.format_manager.write_data(&output_data, self.config.output_format, &mut file)
            .map_err(|e| e.into())
            .map(|bytes| bytes as u64)
    }

    /// Validate output file
    fn validate_output(&self, path: &Path) -> Result<ValidationReport, BinaryExportError> {
        validate_binary_file(path)
    }

    /// Get collection statistics
    fn get_collection_statistics(&self) -> CollectionStatistics {
        // This would get actual statistics from the data collector
        // For now, return default values
        CollectionStatistics::default()
    }

    /// Calculate comprehensive performance metrics
    fn calculate_performance_metrics(&self, total_time: Duration) -> PerformanceMetrics {
        let component_times = self.performance_monitor.get_component_times();
        let memory_stats = self.calculate_memory_stats();
        let throughput = self.calculate_throughput_metrics(total_time);
        let efficiency = self.calculate_efficiency_metrics(&component_times, &memory_stats);

        PerformanceMetrics {
            total_time,
            component_times,
            memory_stats,
            throughput,
            efficiency,
        }
    }

    /// Calculate memory statistics
    fn calculate_memory_stats(&self) -> MemoryStats {
        let peak_usage = self.memory_manager.peak_usage();
        let samples = self.performance_monitor.get_memory_samples();
        
        let average_usage = if samples.is_empty() {
            0
        } else {
            samples.iter().map(|(_, usage)| usage).sum::<usize>() / samples.len()
        };

        let efficiency_score = if peak_usage > 0 {
            average_usage as f64 / peak_usage as f64
        } else {
            1.0
        };

        MemoryStats {
            peak_usage,
            average_usage,
            efficiency_score,
        }
    }

    /// Calculate throughput metrics
    fn calculate_throughput_metrics(&self, total_time: Duration) -> ThroughputMetrics {
        let total_secs = total_time.as_secs_f64();
        let data_size = self.memory_manager.peak_usage() as f64;

        ThroughputMetrics {
            overall_throughput: data_size / total_secs,
            collection_throughput: data_size / total_secs, // Placeholder
            processing_throughput: data_size / total_secs, // Placeholder
            compression_throughput: data_size / total_secs, // Placeholder
        }
    }

    /// Calculate efficiency metrics
    fn calculate_efficiency_metrics(&self, component_times: &ComponentTimes, memory_stats: &MemoryStats) -> EfficiencyMetrics {
        // Calculate CPU efficiency based on parallel utilization
        let cpu_efficiency = if let Some(ref parallel_stats) = self.parallel_processor.as_ref().map(|p| p.get_stats()) {
            parallel_stats.parallel_efficiency
        } else {
            0.8 // Assume reasonable efficiency for non-parallel processing
        };

        EfficiencyMetrics {
            overall_efficiency: (cpu_efficiency + memory_stats.efficiency_score) / 2.0,
            cpu_efficiency,
            memory_efficiency: memory_stats.efficiency_score,
            io_efficiency: 0.9, // Placeholder - would calculate based on I/O patterns
        }
    }

    /// Get system status and health information
    pub fn get_system_status(&self) -> SystemStatus {
        SystemStatus {
            memory_usage: self.memory_manager.current_usage(),
            peak_memory_usage: self.memory_manager.peak_usage(),
            parallel_workers_active: self.parallel_processor.as_ref()
                .map(|p| p.get_stats().items_per_worker.len())
                .unwrap_or(0),
            compression_enabled: self.config.compression.algorithm != CompressionAlgorithm::None,
            monitoring_enabled: self.config.enable_monitoring,
            auto_optimization_enabled: self.config.auto_optimize,
        }
    }
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Current memory usage
    pub memory_usage: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Number of active parallel workers
    pub parallel_workers_active: usize,
    /// Whether compression is enabled
    pub compression_enabled: bool,
    /// Whether monitoring is enabled
    pub monitoring_enabled: bool,
    /// Whether auto-optimization is enabled
    pub auto_optimization_enabled: bool,
}

impl PerformanceMonitor {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            start_time: Instant::now(),
            component_timers: std::collections::HashMap::new(),
            memory_samples: Vec::new(),
        }
    }

    fn start_monitoring(&mut self) {
        if self.enabled {
            self.start_time = Instant::now();
            self.component_timers.clear();
            self.memory_samples.clear();
        }
    }

    fn start_component_timer(&mut self, component: &str) {
        if self.enabled {
            self.component_timers.insert(component.to_string(), Instant::now());
        }
    }

    fn end_component_timer(&mut self, component: &str) -> Duration {
        if self.enabled {
            if let Some(start_time) = self.component_timers.remove(component) {
                return start_time.elapsed();
            }
        }
        Duration::from_millis(0)
    }

    fn get_component_times(&self) -> ComponentTimes {
        // This would return actual measured times
        // For now, return default values
        ComponentTimes {
            collection_time: Duration::from_millis(100),
            processing_time: Duration::from_millis(50),
            compression_time: Duration::from_millis(25),
            format_time: Duration::from_millis(10),
            io_time: Duration::from_millis(15),
        }
    }

    fn get_memory_samples(&self) -> &[(Instant, usize)] {
        &self.memory_samples
    }
}

impl Default for crate::export::binary::processor::ProcessStats {
    fn default() -> Self {
        Self {
            bytes_processed: 0,
            duration: Duration::from_millis(0),
            throughput: 0.0,
            peak_memory_usage: 0,
            chunks_processed: 0,
            validation_errors: 0,
            efficiency: 0.0,
        }
    }
}

impl Default for CollectionStatistics {
    fn default() -> Self {
        Self {
            total_duration: Duration::from_millis(0),
            phase_durations: std::collections::HashMap::new(),
            allocations_processed: 0,
            call_stacks_collected: 0,
            peak_memory_usage: 0,
            cache_hits: 0,
            cache_misses: 0,
            items_skipped: 0,
            errors_encountered: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_integrated_config_presets() {
        let high_perf = IntegratedConfig::high_performance();
        assert!(high_perf.auto_optimize);
        assert_eq!(high_perf.output_format, OutputFormat::CustomBinary);

        let memory_eff = IntegratedConfig::memory_efficient();
        assert!(memory_eff.parallel.is_none());
        assert_eq!(memory_eff.processing.max_memory_usage, 64 * 1024 * 1024);

        let balanced = IntegratedConfig::balanced();
        assert!(balanced.parallel.is_some());
        assert_eq!(balanced.output_format, OutputFormat::MessagePack);
    }

    #[test]
    fn test_integrated_exporter_creation() {
        let config = IntegratedConfig::default();
        let exporter = IntegratedBinaryExporter::new(config);
        
        let status = exporter.get_system_status();
        assert_eq!(status.memory_usage, 0);
        assert!(status.compression_enabled);
    }

    #[test]
    fn test_system_status() {
        let config = IntegratedConfig::high_performance();
        let exporter = IntegratedBinaryExporter::new(config);
        
        let status = exporter.get_system_status();
        assert!(status.auto_optimization_enabled);
        assert!(status.monitoring_enabled);
        assert!(status.compression_enabled);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(true);
        monitor.start_monitoring();
        
        monitor.start_component_timer("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = monitor.end_component_timer("test");
        
        assert!(elapsed >= Duration::from_millis(10));
    }
}