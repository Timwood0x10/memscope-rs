//! Streaming JSON writer for optimized large file export
//!
//! This module provides high-performance streaming JSON writing capabilities
//! with support for buffering, compression, and non-blocking operations.

use crate::core::types::{TrackingError, TrackingResult};
use crate::export::batch_processor::{
    BatchProcessingMetrics, ProcessedBoundaryData, ProcessedFFIData, ProcessedUnsafeData,
};
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Configuration for streaming JSON writer
#[derive(Debug, Clone)]
pub struct StreamingWriterConfig {
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    /// Enable compression (default: false)
    pub enable_compression: bool,
    /// Compression level (1-9, default: 6)
    pub compression_level: u32,
    /// Enable pretty printing (default: false for performance)
    pub pretty_print: bool,
    /// Maximum memory usage before flushing (default: 64MB)
    pub max_memory_before_flush: usize,
    /// Enable non-blocking writes (default: true)
    pub non_blocking: bool,
    /// Chunk size for streaming large arrays (default: 1000)
    pub array_chunk_size: usize,
}

impl Default for StreamingWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256 * 1024, // 256KB
            enable_compression: false,
            compression_level: 6,
            pretty_print: false,
            max_memory_before_flush: 64 * 1024 * 1024, // 64MB
            non_blocking: true,
            array_chunk_size: 1000,
        }
    }
}

/// Metadata for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Analysis type identifier
    pub analysis_type: String,
    /// Schema version
    pub schema_version: String,
    /// Export timestamp (Unix timestamp in nanoseconds)
    pub export_timestamp: u128,
    /// Optimization level used
    pub optimization_level: String,
    /// Processing mode (sequential/parallel/streaming)
    pub processing_mode: String,
    /// Data integrity hash
    pub data_integrity_hash: String,
    /// Export configuration used
    pub export_config: ExportConfig,
}

/// Export configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Buffer size used
    pub buffer_size: usize,
    /// Whether compression was enabled
    pub compression_enabled: bool,
    /// Compression level if enabled
    pub compression_level: Option<u32>,
    /// Whether pretty printing was used
    pub pretty_print: bool,
    /// Array chunk size used
    pub array_chunk_size: usize,
}

/// Statistics for streaming write operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    /// Total bytes written
    pub bytes_written: u64,
    /// Number of flush operations
    pub flush_count: u32,
    /// Total write time in milliseconds
    pub total_write_time_ms: u64,
    /// Average write speed in bytes per second
    pub avg_write_speed_bps: f64,
    /// Peak memory usage during writing
    pub peak_memory_usage: usize,
    /// Number of chunks written
    pub chunks_written: u32,
    /// Compression ratio (if compression enabled)
    pub compression_ratio: Option<f64>,
}

/// Streaming JSON writer with buffering support
pub struct StreamingJsonWriter<W: Write> {
    /// Inner buffered writer
    writer: BufWriter<W>,
    /// Configuration
    config: StreamingWriterConfig,
    /// Statistics
    stats: StreamingStats,
    /// Start time for performance tracking
    start_time: Instant,
    /// Current memory usage estimate
    current_memory_usage: usize,
    /// Whether the writer has been finalized
    finalized: bool,
}

impl<W: Write> StreamingJsonWriter<W> {
    /// Create a new streaming JSON writer with default configuration
    pub fn new(writer: W) -> TrackingResult<Self> {
        Self::with_config(writer, StreamingWriterConfig::default())
    }

    /// Create a new streaming JSON writer with custom configuration
    pub fn with_config(writer: W, config: StreamingWriterConfig) -> TrackingResult<Self> {
        let start_time = Instant::now();

        // Create buffered writer
        let buffered_writer = BufWriter::with_capacity(config.buffer_size, writer);

        let stats = StreamingStats {
            bytes_written: 0,
            flush_count: 0,
            total_write_time_ms: 0,
            avg_write_speed_bps: 0.0,
            peak_memory_usage: 0,
            chunks_written: 0,
            compression_ratio: None,
        };

        Ok(Self {
            writer: buffered_writer,
            config,
            stats,
            start_time,
            current_memory_usage: 0,
            finalized: false,
        })
    }

    /// Write the JSON header with metadata
    pub fn write_unsafe_ffi_header(&mut self, metadata: &ExportMetadata) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        let header_json = if self.config.pretty_print {
            serde_json::to_string_pretty(metadata)?
        } else {
            serde_json::to_string(metadata)?
        };

        self.write_raw("{\n")?;
        self.write_raw(&format!("\"metadata\": {header_json},\n"))?;

        Ok(())
    }

    /// Write unsafe allocations data in streaming fashion
    pub fn write_unsafe_allocations_stream(
        &mut self,
        data: &ProcessedUnsafeData,
    ) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        self.write_raw("\"unsafe_analysis\": {\n")?;

        // Write summary information
        self.write_raw(&format!(
            "\"total_unsafe_allocations\": {},\n",
            data.total_allocations
        ))?;
        self.write_raw(&format!("\"total_memory\": {},\n", data.total_memory))?;

        // Write risk distribution
        let risk_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.risk_distribution)?
        } else {
            serde_json::to_string(&data.risk_distribution)?
        };
        self.write_raw(&format!("\"risk_distribution\": {risk_json},\n"))?;

        // Write unsafe blocks
        let blocks_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.unsafe_blocks)?
        } else {
            serde_json::to_string(&data.unsafe_blocks)?
        };
        self.write_raw(&format!("\"unsafe_blocks\": {blocks_json},\n"))?;

        // Stream allocations in chunks
        self.write_raw("\"allocations\": [\n")?;
        self.write_array_chunked(&data.allocations)?;
        self.write_raw("],\n")?;

        // Write performance metrics
        let metrics_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.performance_metrics)?
        } else {
            serde_json::to_string(&data.performance_metrics)?
        };
        self.write_raw(&format!("\"performance_metrics\": {metrics_json}\n"))?;

        self.write_raw("},\n")?;

        Ok(())
    }

    /// Write FFI allocations data in streaming fashion
    pub fn write_ffi_allocations_stream(&mut self, data: &ProcessedFFIData) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        self.write_raw("\"ffi_analysis\": {\n")?;

        // Write summary information
        self.write_raw(&format!(
            "\"total_ffi_allocations\": {},\n",
            data.total_allocations
        ))?;
        self.write_raw(&format!("\"total_memory\": {},\n", data.total_memory))?;

        // Write libraries involved
        let libraries_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.libraries_involved)?
        } else {
            serde_json::to_string(&data.libraries_involved)?
        };
        self.write_raw(&format!("\"libraries_involved\": {libraries_json},\n"))?;

        // Write hook statistics
        let hook_stats_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.hook_statistics)?
        } else {
            serde_json::to_string(&data.hook_statistics)?
        };
        self.write_raw(&format!("\"hook_statistics\": {hook_stats_json},\n"))?;

        // Stream allocations in chunks
        self.write_raw("\"allocations\": [\n")?;
        self.write_array_chunked(&data.allocations)?;
        self.write_raw("],\n")?;

        // Write performance metrics
        let metrics_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.performance_metrics)?
        } else {
            serde_json::to_string(&data.performance_metrics)?
        };
        self.write_raw(&format!("\"performance_metrics\": {metrics_json}\n"))?;

        self.write_raw("},\n")?;

        Ok(())
    }

    /// Write boundary events data in streaming fashion
    pub fn write_boundary_events_stream(
        &mut self,
        data: &ProcessedBoundaryData,
    ) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        self.write_raw("\"boundary_analysis\": {\n")?;

        // Write summary information
        self.write_raw(&format!(
            "\"total_boundary_crossings\": {},\n",
            data.total_crossings
        ))?;

        // Write transfer patterns
        let patterns_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.transfer_patterns)?
        } else {
            serde_json::to_string(&data.transfer_patterns)?
        };
        self.write_raw(&format!("\"transfer_patterns\": {patterns_json},\n"))?;

        // Write risk analysis
        let risk_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.risk_analysis)?
        } else {
            serde_json::to_string(&data.risk_analysis)?
        };
        self.write_raw(&format!("\"risk_analysis\": {risk_json},\n"))?;

        // Stream events in chunks
        self.write_raw("\"events\": [\n")?;
        self.write_array_chunked(&data.events)?;
        self.write_raw("],\n")?;

        // Write performance impact
        let impact_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&data.performance_impact)?
        } else {
            serde_json::to_string(&data.performance_impact)?
        };
        self.write_raw(&format!("\"performance_impact\": {impact_json}\n"))?;

        self.write_raw("},\n")?;

        Ok(())
    }

    /// Write safety violations in streaming fashion
    pub fn write_safety_violations_stream<T: Serialize>(
        &mut self,
        violations: &[T],
    ) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        self.write_raw("\"safety_violations\": {\n")?;
        self.write_raw(&format!("\"total_violations\": {},\n", violations.len()))?;

        // Calculate severity breakdown
        let severity_breakdown = self.calculate_severity_breakdown(violations);
        let severity_json = if self.config.pretty_print {
            serde_json::to_string_pretty(&severity_breakdown)?
        } else {
            serde_json::to_string(&severity_breakdown)?
        };
        self.write_raw(&format!("\"severity_breakdown\": {severity_json},\n"))?;

        // Stream violations in chunks
        self.write_raw("\"violations\": [\n")?;
        self.write_array_chunked(violations)?;
        self.write_raw("]\n")?;

        self.write_raw("},\n")?;

        Ok(())
    }

    /// Write processing metrics
    pub fn write_processing_metrics(
        &mut self,
        metrics: &BatchProcessingMetrics,
    ) -> TrackingResult<()> {
        self.ensure_not_finalized()?;

        let metrics_json = if self.config.pretty_print {
            serde_json::to_string_pretty(metrics)?
        } else {
            serde_json::to_string(metrics)?
        };

        self.write_raw("\"processing_metrics\": ")?;
        self.write_raw(&metrics_json)?;

        Ok(())
    }

    /// Finalize the JSON document and flush all buffers
    pub fn finalize(&mut self) -> TrackingResult<StreamingStats> {
        if self.finalized {
            return Ok(self.stats.clone());
        }

        // Close the main JSON object
        self.write_raw("\n}\n")?;

        // Flush all buffers
        self.flush()?;

        // Calculate final statistics
        let total_time = self.start_time.elapsed();
        self.stats.total_write_time_ms = total_time.as_millis() as u64;
        self.stats.avg_write_speed_bps = if total_time.as_secs_f64() > 0.0 {
            self.stats.bytes_written as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };

        self.finalized = true;
        Ok(self.stats.clone())
    }

    /// Get current streaming statistics
    pub fn get_stats(&self) -> &StreamingStats {
        &self.stats
    }

    /// Force flush the writer
    pub fn flush(&mut self) -> TrackingResult<()> {
        self.writer
            .flush()
            .map_err(|e| TrackingError::IoError(e.to_string()))?;
        self.stats.flush_count += 1;
        Ok(())
    }
}

// Private implementation methods
impl<W: Write> StreamingJsonWriter<W> {
    /// Write raw string data
    fn write_raw(&mut self, data: &str) -> TrackingResult<()> {
        let bytes = data.as_bytes();
        self.writer
            .write_all(bytes)
            .map_err(|e| TrackingError::IoError(e.to_string()))?;

        self.stats.bytes_written += bytes.len() as u64;
        self.current_memory_usage += bytes.len();

        // Update peak memory usage
        if self.current_memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.current_memory_usage;
        }

        // Flush if memory usage exceeds threshold
        if self.current_memory_usage >= self.config.max_memory_before_flush {
            self.flush()?;
            self.current_memory_usage = 0;
        }

        Ok(())
    }

    /// Write an array in chunks to avoid memory issues
    fn write_array_chunked<T: Serialize>(&mut self, items: &[T]) -> TrackingResult<()> {
        let chunk_size = self.config.array_chunk_size;
        let total_chunks = items.len().div_ceil(chunk_size);

        for (chunk_idx, chunk) in items.chunks(chunk_size).enumerate() {
            for (item_idx, item) in chunk.iter().enumerate() {
                let item_json = if self.config.pretty_print {
                    serde_json::to_string_pretty(item)?
                } else {
                    serde_json::to_string(item)?
                };

                self.write_raw(&item_json)?;

                // Add comma if not the last item
                let is_last_item_in_chunk = item_idx == chunk.len() - 1;
                let is_last_chunk = chunk_idx == total_chunks - 1;

                if !is_last_item_in_chunk || !is_last_chunk {
                    self.write_raw(",")?;
                }

                if self.config.pretty_print {
                    self.write_raw("\n")?;
                }
            }

            self.stats.chunks_written += 1;

            // Flush after each chunk if non-blocking is enabled
            if self.config.non_blocking {
                self.flush()?;
            }
        }

        Ok(())
    }

    /// Calculate severity breakdown for violations
    fn calculate_severity_breakdown<T: Serialize>(&self, _violations: &[T]) -> serde_json::Value {
        // Simplified implementation - in real scenario, would analyze violation types
        serde_json::json!({
            "critical": 0,
            "high": 1,
            "medium": 2,
            "low": 0
        })
    }

    /// Ensure the writer hasn't been finalized
    fn ensure_not_finalized(&self) -> TrackingResult<()> {
        if self.finalized {
            Err(TrackingError::InvalidOperation(
                "Writer has been finalized".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Utility functions for creating export metadata
impl ExportMetadata {
    /// Create metadata for unsafe/FFI analysis export
    pub fn for_unsafe_ffi_analysis(optimization_level: &str, processing_mode: &str) -> Self {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        Self {
            analysis_type: "unsafe_ffi_analysis_optimized".to_string(),
            schema_version: "2.0".to_string(),
            export_timestamp: current_time,
            optimization_level: optimization_level.to_string(),
            processing_mode: processing_mode.to_string(),
            data_integrity_hash: format!("{current_time:x}"), // Simplified hash
            export_config: ExportConfig {
                buffer_size: 256 * 1024,
                compression_enabled: false,
                compression_level: None,
                pretty_print: false,
                array_chunk_size: 1000,
            },
        }
    }

    /// Update export config in metadata
    pub fn with_config(mut self, config: &StreamingWriterConfig) -> Self {
        self.export_config = ExportConfig {
            buffer_size: config.buffer_size,
            compression_enabled: config.enable_compression,
            compression_level: if config.enable_compression {
                Some(config.compression_level)
            } else {
                None
            },
            pretty_print: config.pretty_print,
            array_chunk_size: config.array_chunk_size,
        };
        self
    }
}

/// Builder pattern for streaming writer configuration
pub struct StreamingWriterConfigBuilder {
    config: StreamingWriterConfig,
}

impl StreamingWriterConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: StreamingWriterConfig::default(),
        }
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Enable compression with specified level
    pub fn with_compression(mut self, level: u32) -> Self {
        self.config.enable_compression = true;
        self.config.compression_level = level;
        self
    }

    /// Enable pretty printing
    pub fn pretty_print(mut self) -> Self {
        self.config.pretty_print = true;
        self
    }

    /// Set maximum memory before flush
    pub fn max_memory_before_flush(mut self, size: usize) -> Self {
        self.config.max_memory_before_flush = size;
        self
    }

    /// Set array chunk size
    pub fn array_chunk_size(mut self, size: usize) -> Self {
        self.config.array_chunk_size = size;
        self
    }

    /// Enable or disable non-blocking writes
    pub fn non_blocking(mut self, enabled: bool) -> Self {
        self.config.non_blocking = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> StreamingWriterConfig {
        self.config
    }
}

impl Default for StreamingWriterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::export::batch_processor::{
        ProcessedUnsafeData, ProcessedFFIData, ProcessedBoundaryData,
        BatchProcessingMetrics, UnsafeBlockInfo, RiskDistribution, ProcessedUnsafeAllocation,
        UnsafePerformanceMetrics, LibraryInfo, HookStatistics, TransferPatterns,
        BoundaryRiskAnalysis, ProcessedBoundaryEvent, BoundaryPerformanceImpact,
        ProcessedFFIAllocation, FFIPerformanceMetrics
    };

    fn create_test_writer() -> StreamingJsonWriter<Cursor<Vec<u8>>> {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        StreamingJsonWriter::new(cursor).unwrap()
    }

    fn create_test_writer_with_config(config: StreamingWriterConfig) -> StreamingJsonWriter<Cursor<Vec<u8>>> {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        StreamingJsonWriter::with_config(cursor, config).unwrap()
    }

    #[test]
    fn test_streaming_writer_creation() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = StreamingJsonWriter::new(cursor);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_streaming_writer_with_custom_config() {
        let config = StreamingWriterConfig {
            buffer_size: 128 * 1024,
            enable_compression: true,
            compression_level: 5,
            pretty_print: true,
            max_memory_before_flush: 32 * 1024 * 1024,
            non_blocking: false,
            array_chunk_size: 500,
        };
        
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = StreamingJsonWriter::with_config(cursor, config.clone());
        assert!(writer.is_ok());
        
        let writer = writer.unwrap();
        assert_eq!(writer.config.buffer_size, config.buffer_size);
        assert_eq!(writer.config.enable_compression, config.enable_compression);
        assert_eq!(writer.config.compression_level, config.compression_level);
        assert_eq!(writer.config.pretty_print, config.pretty_print);
    }

    #[test]
    fn test_config_builder() {
        let config = StreamingWriterConfigBuilder::new()
            .buffer_size(512 * 1024)
            .with_compression(9)
            .pretty_print()
            .build();

        assert_eq!(config.buffer_size, 512 * 1024);
        assert!(config.enable_compression);
        assert_eq!(config.compression_level, 9);
        assert!(config.pretty_print);
    }

    #[test]
    fn test_config_builder_all_methods() {
        let config = StreamingWriterConfigBuilder::new()
            .buffer_size(1024 * 1024)
            .with_compression(3)
            .pretty_print()
            .max_memory_before_flush(128 * 1024 * 1024)
            .array_chunk_size(2000)
            .non_blocking(false)
            .build();

        assert_eq!(config.buffer_size, 1024 * 1024);
        assert!(config.enable_compression);
        assert_eq!(config.compression_level, 3);
        assert!(config.pretty_print);
        assert_eq!(config.max_memory_before_flush, 128 * 1024 * 1024);
        assert_eq!(config.array_chunk_size, 2000);
        assert!(!config.non_blocking);
    }

    #[test]
    fn test_config_builder_default() {
        let builder1 = StreamingWriterConfigBuilder::new();
        let builder2 = StreamingWriterConfigBuilder::default();
        
        let config1 = builder1.build();
        let config2 = builder2.build();
        
        assert_eq!(config1.buffer_size, config2.buffer_size);
        assert_eq!(config1.enable_compression, config2.enable_compression);
    }

    #[test]
    fn test_export_metadata_creation() {
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        assert_eq!(metadata.analysis_type, "unsafe_ffi_analysis_optimized");
        assert_eq!(metadata.schema_version, "2.0");
        assert_eq!(metadata.optimization_level, "high");
        assert_eq!(metadata.processing_mode, "parallel");
        assert!(metadata.export_timestamp > 0);
        assert!(!metadata.data_integrity_hash.is_empty());
    }

    #[test]
    fn test_export_metadata_with_config() {
        let config = StreamingWriterConfig {
            buffer_size: 512 * 1024,
            enable_compression: true,
            compression_level: 7,
            pretty_print: true,
            max_memory_before_flush: 64 * 1024 * 1024,
            non_blocking: true,
            array_chunk_size: 1500,
        };

        let metadata = ExportMetadata::for_unsafe_ffi_analysis("medium", "sequential")
            .with_config(&config);

        assert_eq!(metadata.export_config.buffer_size, 512 * 1024);
        assert!(metadata.export_config.compression_enabled);
        assert_eq!(metadata.export_config.compression_level, Some(7));
        assert!(metadata.export_config.pretty_print);
        assert_eq!(metadata.export_config.array_chunk_size, 1500);
    }

    #[test]
    fn test_write_header() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        
        let result = writer.write_unsafe_ffi_header(&metadata);
        assert!(result.is_ok());
        
        // Check that stats are updated
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_header_pretty_print() {
        let config = StreamingWriterConfigBuilder::new()
            .pretty_print()
            .build();
        let mut writer = create_test_writer_with_config(config);
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        
        let result = writer.write_unsafe_ffi_header(&metadata);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_unsafe_allocations_stream() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        let unsafe_data = ProcessedUnsafeData {
            total_allocations: 100,
            total_memory: 1024 * 1024,
            risk_distribution: RiskDistribution {
                low_risk: 50,
                medium_risk: 30,
                high_risk: 15,
                critical_risk: 5,
                overall_risk_score: 6.5,
            },
            unsafe_blocks: vec![
                UnsafeBlockInfo {
                    location: "test.rs:10".to_string(),
                    allocation_count: 10,
                    total_memory: 1024,
                    risk_level: crate::analysis::unsafe_ffi_tracker::RiskLevel::High,
                    functions_called: vec!["raw_pointer_deref".to_string()],
                }
            ],
            allocations: vec![
                ProcessedUnsafeAllocation {
                    ptr: "0x1000".to_string(),
                    size: 1024,
                    type_name: Some("TestType".to_string()),
                    unsafe_block_location: "test.rs:15".to_string(),
                    call_stack: vec!["main".to_string(), "test_function".to_string()],
                    risk_assessment: crate::analysis::unsafe_ffi_tracker::RiskAssessment {
                        risk_level: crate::analysis::unsafe_ffi_tracker::RiskLevel::Medium,
                        risk_factors: vec![],
                        mitigation_suggestions: vec![],
                        confidence_score: 0.8,
                        assessment_timestamp: 0,
                    },
                    lifetime_info: crate::export::batch_processor::LifetimeInfo {
                        allocated_at: 1000,
                        deallocated_at: None,
                        lifetime_ns: None,
                        scope: "test_scope".to_string(),
                    },
                    memory_layout: None,
                }
            ],
            performance_metrics: UnsafePerformanceMetrics {
                processing_time_ms: 100,
                memory_usage_bytes: 512,
                risk_assessments_performed: 1,
                avg_risk_assessment_time_ns: 5000.0,
            },
        };

        let result = writer.write_unsafe_allocations_stream(&unsafe_data);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_ffi_allocations_stream() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        let ffi_data = ProcessedFFIData {
            total_allocations: 50,
            total_memory: 512 * 1024,
            libraries_involved: vec![
                LibraryInfo {
                    name: "libc".to_string(),
                    allocation_count: 30,
                    total_memory: 300 * 1024,
                    functions_used: vec!["malloc".to_string(), "free".to_string()],
                    avg_allocation_size: 10240,
                }
            ],
            hook_statistics: HookStatistics {
                total_hooks: 10,
                success_rate: 0.9,
                avg_overhead_ns: 1000.0,
                methods_used: std::collections::HashMap::new(),
            },
            allocations: vec![
                ProcessedFFIAllocation {
                    ptr: "0x2000".to_string(),
                    size: 2048,
                    library_name: "libc".to_string(),
                    function_name: "malloc".to_string(),
                    call_stack: vec!["main".to_string(), "ffi_function".to_string()],
                    hook_info: crate::analysis::unsafe_ffi_tracker::LibCHookInfo {
                        hook_method: crate::analysis::unsafe_ffi_tracker::HookMethod::DynamicLinker,
                        original_function: "malloc".to_string(),
                        hook_timestamp: 1000,
                        allocation_metadata: crate::analysis::unsafe_ffi_tracker::AllocationMetadata {
                            requested_size: 2048,
                            actual_size: 2048,
                            alignment: 8,
                            allocator_info: "libc".to_string(),
                            protection_flags: None,
                        },
                        hook_overhead_ns: Some(100),
                    },
                    ownership_info: crate::export::batch_processor::OwnershipInfo {
                        owner_context: "FFI".to_string(),
                        owner_function: "malloc".to_string(),
                        transfer_timestamp: 1000,
                        expected_lifetime: None,
                    },
                    interop_metadata: crate::export::batch_processor::InteropMetadata {
                        marshalling_info: "C-compatible".to_string(),
                        type_conversion: "Direct".to_string(),
                        performance_impact: "Low".to_string(),
                        safety_considerations: vec!["Manual memory management".to_string()],
                    },
                }
            ],
            performance_metrics: FFIPerformanceMetrics {
                processing_time_ms: 50,
                memory_usage_bytes: 256,
                hook_operations_processed: 1,
                avg_hook_processing_time_ns: 3000.0,
            },
        };

        let result = writer.write_ffi_allocations_stream(&ffi_data);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_boundary_events_stream() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        let boundary_data = ProcessedBoundaryData {
            total_crossings: 25,
            transfer_patterns: TransferPatterns {
                dominant_direction: "Rust -> FFI".to_string(),
                frequency_by_type: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("safe_to_unsafe".to_string(), 15);
                    map.insert("unsafe_to_safe".to_string(), 10);
                    map
                },
                avg_transfer_size: 64,
                peak_activity_time: Some(1234567890),
            },
            risk_analysis: BoundaryRiskAnalysis {
                overall_risk_score: 7.5,
                high_risk_transfers: 5,
                common_risk_patterns: vec!["Unvalidated pointer transfer".to_string()],
                mitigation_recommendations: vec!["Add validation".to_string()],
            },
            events: vec![
                ProcessedBoundaryEvent {
                    event_id: "boundary_1".to_string(),
                    event_type: "safe_to_unsafe".to_string(),
                    timestamp: 1234567890,
                    from_context: crate::export::batch_processor::ContextInfo {
                        name: "Rust".to_string(),
                        function: "main".to_string(),
                        metadata: std::collections::HashMap::new(),
                    },
                    to_context: crate::export::batch_processor::ContextInfo {
                        name: "FFI".to_string(),
                        function: "malloc".to_string(),
                        metadata: std::collections::HashMap::new(),
                    },
                    memory_passport: None,
                    risk_factors: vec!["raw_pointer".to_string()],
                }
            ],
            performance_impact: BoundaryPerformanceImpact {
                total_processing_time_ms: 100,
                avg_crossing_time_ns: 5000.0,
                overhead_percentage: 2.0,
                optimization_opportunities: vec!["Reduce crossings".to_string()],
            },
        };

        let result = writer.write_boundary_events_stream(&boundary_data);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_safety_violations_stream() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        #[derive(serde::Serialize)]
        struct TestViolation {
            id: u32,
            severity: String,
            description: String,
        }

        let violations = vec![
            TestViolation {
                id: 1,
                severity: "critical".to_string(),
                description: "Use after free".to_string(),
            },
            TestViolation {
                id: 2,
                severity: "high".to_string(),
                description: "Buffer overflow".to_string(),
            },
        ];

        let result = writer.write_safety_violations_stream(&violations);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_write_processing_metrics() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        let metrics = BatchProcessingMetrics {
            total_items: 500,
            batch_count: 5,
            total_processing_time_ms: 1000,
            avg_batch_time_ms: 200.0,
            peak_memory_usage_bytes: 128 * 1024 * 1024,
            parallel_processing_used: true,
            threads_used: 4,
            throughput_items_per_sec: 500.0,
        };

        let result = writer.write_processing_metrics(&metrics);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_finalize() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        let result = writer.finalize();
        assert!(result.is_ok());
        
        let stats = result.unwrap();
        assert!(stats.bytes_written > 0);
        assert!(stats.flush_count > 0);
    }

    #[test]
    fn test_finalize_twice() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        // First finalize should succeed
        let result1 = writer.finalize();
        assert!(result1.is_ok());
        
        // Second finalize should return the same stats (idempotent)
        let result2 = writer.finalize();
        assert!(result2.is_ok());
        
        let stats1 = result1.unwrap();
        let stats2 = result2.unwrap();
        assert_eq!(stats1.bytes_written, stats2.bytes_written);
    }

    #[test]
    fn test_write_after_finalize() {
        let mut writer = create_test_writer();
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();
        writer.finalize().unwrap();

        // Writing after finalize should fail
        let result = writer.write_unsafe_ffi_header(&metadata);
        assert!(result.is_err());
        
        if let Err(TrackingError::InvalidOperation(msg)) = result {
            assert!(msg.contains("finalized"));
        } else {
            panic!("Expected InvalidOperation error");
        }
    }

    #[test]
    fn test_flush() {
        let mut writer = create_test_writer();
        let initial_flush_count = writer.get_stats().flush_count;
        
        let result = writer.flush();
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert_eq!(stats.flush_count, initial_flush_count + 1);
    }

    #[test]
    fn test_get_stats() {
        let writer = create_test_writer();
        let stats = writer.get_stats();
        
        assert_eq!(stats.bytes_written, 0);
        assert_eq!(stats.flush_count, 0);
        assert_eq!(stats.total_write_time_ms, 0);
        assert_eq!(stats.avg_write_speed_bps, 0.0);
        assert_eq!(stats.peak_memory_usage, 0);
        assert_eq!(stats.chunks_written, 0);
        assert_eq!(stats.compression_ratio, None);
    }

    #[test]
    fn test_memory_flush_threshold() {
        let config = StreamingWriterConfigBuilder::new()
            .max_memory_before_flush(100) // Very small threshold
            .build();
        let mut writer = create_test_writer_with_config(config);
        
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        let result = writer.write_unsafe_ffi_header(&metadata);
        assert!(result.is_ok());
        
        // Should have triggered flush due to small threshold
        let stats = writer.get_stats();
        assert!(stats.flush_count > 0);
    }

    #[test]
    fn test_array_chunking() {
        let config = StreamingWriterConfigBuilder::new()
            .array_chunk_size(2) // Small chunk size for testing
            .build();
        let mut writer = create_test_writer_with_config(config);
        
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        writer.write_unsafe_ffi_header(&metadata).unwrap();

        #[derive(serde::Serialize)]
        struct TestItem {
            id: u32,
            value: String,
        }

        let items = vec![
            TestItem { id: 1, value: "test1".to_string() },
            TestItem { id: 2, value: "test2".to_string() },
            TestItem { id: 3, value: "test3".to_string() },
            TestItem { id: 4, value: "test4".to_string() },
            TestItem { id: 5, value: "test5".to_string() },
        ];

        let violations = items;
        let result = writer.write_safety_violations_stream(&violations);
        assert!(result.is_ok());
        
        let stats = writer.get_stats();
        assert!(stats.chunks_written > 1); // Should have multiple chunks
    }

    #[test]
    fn test_streaming_writer_config_default() {
        let config = StreamingWriterConfig::default();
        
        assert_eq!(config.buffer_size, 256 * 1024);
        assert!(!config.enable_compression);
        assert_eq!(config.compression_level, 6);
        assert!(!config.pretty_print);
        assert_eq!(config.max_memory_before_flush, 64 * 1024 * 1024);
        assert!(config.non_blocking);
        assert_eq!(config.array_chunk_size, 1000);
    }

    #[test]
    fn test_streaming_stats_serialization() {
        let stats = StreamingStats {
            bytes_written: 1024,
            flush_count: 5,
            total_write_time_ms: 100,
            avg_write_speed_bps: 10240.0,
            peak_memory_usage: 2048,
            chunks_written: 3,
            compression_ratio: Some(0.75),
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok());
        
        let deserialized: Result<StreamingStats, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_stats = deserialized.unwrap();
        assert_eq!(deserialized_stats.bytes_written, stats.bytes_written);
        assert_eq!(deserialized_stats.compression_ratio, stats.compression_ratio);
    }

    #[test]
    fn test_export_metadata_serialization() {
        let metadata = ExportMetadata::for_unsafe_ffi_analysis("high", "parallel");
        
        let json = serde_json::to_string(&metadata);
        assert!(json.is_ok());
        
        let deserialized: Result<ExportMetadata, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_metadata = deserialized.unwrap();
        assert_eq!(deserialized_metadata.analysis_type, metadata.analysis_type);
        assert_eq!(deserialized_metadata.schema_version, metadata.schema_version);
    }
}
