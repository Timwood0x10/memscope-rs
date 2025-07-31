//! Binary export implementation
//!
//! This module provides the main BinaryExporter implementation that handles
//! the actual export process, including data serialization, compression,
//! and file writing operations.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use serde::{Serialize, Deserialize};

use crate::tracker::MemoryTracker;
use super::core::UnifiedData;
use super::data::DataCollector;
use super::error::{BinaryExportError, ErrorRecovery};
use super::memory::MemoryManager;
use super::validation::ValidationReport;

/// Configuration for binary export operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Enable compression (zstd)
    pub compression_enabled: bool,
    /// Compression level (1-22)
    pub compression_level: i32,
    /// Include metadata for validation
    pub include_metadata: bool,
    /// Chunk size for streaming operations
    pub chunk_size: usize,
    /// Maximum memory usage during export
    pub max_memory_usage: usize,
    /// Enable progress reporting
    pub enable_progress: bool,
    /// Timeout for export operation (seconds)
    pub timeout_secs: u64,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            compression_enabled: true,
            compression_level: 6,
            include_metadata: true,
            chunk_size: 256 * 1024, // 256KB
            max_memory_usage: 512 * 1024 * 1024, // 512MB
            enable_progress: false,
            timeout_secs: 300, // 5 minutes
        }
    }
}

impl ExportConfig {
    /// Fast export configuration - minimal compression, optimized for speed
    pub fn fast() -> Self {
        Self {
            compression_enabled: false,
            compression_level: 1,
            include_metadata: false,
            chunk_size: 64 * 1024,
            max_memory_usage: 256 * 1024 * 1024,
            enable_progress: false,
            timeout_secs: 60,
        }
    }

    /// Compact export configuration - maximum compression
    pub fn compact() -> Self {
        Self {
            compression_enabled: true,
            compression_level: 19,
            include_metadata: true,
            chunk_size: 1024 * 1024,
            max_memory_usage: 1024 * 1024 * 1024,
            enable_progress: true,
            timeout_secs: 1800, // 30 minutes
        }
    }
}

/// Result of a binary export operation
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Number of bytes written to file
    pub bytes_written: u64,
    /// Duration of the export operation
    pub duration: std::time::Duration,
    /// Compression ratio (if compression was used)
    pub compression_ratio: Option<f64>,
    /// Number of allocations exported
    pub allocation_count: usize,
    /// Any warnings encountered during export
    pub warnings: Vec<String>,
    /// Export statistics
    pub stats: ExportStats,
}

/// Detailed statistics about the export operation
#[derive(Debug, Clone)]
pub struct ExportStats {
    /// Time spent collecting data
    pub collection_time: std::time::Duration,
    /// Time spent serializing data
    pub serialization_time: std::time::Duration,
    /// Time spent compressing data (if enabled)
    pub compression_time: Option<std::time::Duration>,
    /// Time spent writing to disk
    pub write_time: std::time::Duration,
    /// Original data size before compression
    pub original_size: u64,
    /// Final file size
    pub final_size: u64,
    /// Memory peak usage during export
    pub peak_memory_usage: usize,
}

/// Main binary exporter implementation
pub struct BinaryExporter {
    config: ExportConfig,
    memory_manager: MemoryManager,
    error_recovery: ErrorRecovery,
}

impl BinaryExporter {
    /// Create a new binary exporter with the given configuration
    pub fn new(config: ExportConfig) -> Self {
        let memory_manager = MemoryManager::new(config.max_memory_usage);
        let error_recovery = ErrorRecovery::new();
        
        Self {
            config,
            memory_manager,
            error_recovery,
        }
    }

    /// Export memory tracking data to binary format
    pub fn export<P: AsRef<Path>>(
        &self,
        tracker: &MemoryTracker,
        path: P,
    ) -> Result<ExportResult, BinaryExportError> {
        let start_time = Instant::now();
        let path = path.as_ref();
        
        // Validate input parameters
        self.validate_export_params(tracker, path)?;
        
        // Collect data from tracker
        let collection_start = Instant::now();
        let data = self.collect_data(tracker)?;
        let collection_time = collection_start.elapsed();
        
        // Serialize data
        let serialization_start = Instant::now();
        let serialized_data = self.serialize_data(&data)?;
        let serialization_time = serialization_start.elapsed();
        
        // Compress data if enabled
        let (final_data, compression_time, compression_ratio) = if self.config.compression_enabled {
            let compression_start = Instant::now();
            let compressed = self.compress_data(&serialized_data)?;
            let compression_time = compression_start.elapsed();
            let ratio = compressed.len() as f64 / serialized_data.len() as f64;
            (compressed, Some(compression_time), Some(ratio))
        } else {
            (serialized_data, None, None)
        };
        
        // Write to file
        let write_start = Instant::now();
        let bytes_written = self.write_to_file(&final_data, path)?;
        let write_time = write_start.elapsed();
        
        let total_duration = start_time.elapsed();
        
        // Build result
        Ok(ExportResult {
            bytes_written,
            duration: total_duration,
            compression_ratio,
            allocation_count: data.allocations.allocations.len(),
            warnings: Vec::new(),
            stats: ExportStats {
                collection_time,
                serialization_time,
                compression_time,
                write_time,
                original_size: serialized_data.len() as u64,
                final_size: bytes_written,
                peak_memory_usage: self.memory_manager.peak_usage(),
            },
        })
    }

    /// Load binary data from file
    pub fn load<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<UnifiedData, BinaryExportError> {
        let path = path.as_ref();
        
        // Validate file exists and is readable
        if !path.exists() {
            return Err(BinaryExportError::FileNotFound(path.to_path_buf()));
        }
        
        // Read file data
        let file_data = std::fs::read(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        // Decompress if needed
        let data = if self.is_compressed(&file_data)? {
            self.decompress_data(&file_data)?
        } else {
            file_data
        };
        
        // Deserialize
        self.deserialize_data(&data)
    }

    /// Validate export parameters
    fn validate_export_params(
        &self,
        tracker: &MemoryTracker,
        path: &Path,
    ) -> Result<(), BinaryExportError> {
        // Check if tracker has data
        if tracker.allocation_count() == 0 {
            return Err(BinaryExportError::NoDataToExport);
        }
        
        // Check if parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| BinaryExportError::IoError(e))?;
            }
        }
        
        Ok(())
    }

    /// Collect data from memory tracker
    fn collect_data(&self, tracker: &MemoryTracker) -> Result<UnifiedData, BinaryExportError> {
        let collector = DataCollector::new(self.config.clone().into());
        collector.collect_from_tracker(tracker)
    }

    /// Serialize data to bytes
    fn serialize_data(&self, data: &UnifiedData) -> Result<Vec<u8>, BinaryExportError> {
        bincode::serialize(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }

    /// Compress data using zstd
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        zstd::bulk::compress(data, self.config.compression_level)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    /// Write data to file
    fn write_to_file(&self, data: &[u8], path: &Path) -> Result<u64, BinaryExportError> {
        let file = File::create(path)
            .map_err(|e| BinaryExportError::IoError(e))?;
        
        let mut writer = BufWriter::new(file);
        writer.write_all(data)
            .map_err(|e| BinaryExportError::IoError(e))?;
        
        writer.flush()
            .map_err(|e| BinaryExportError::IoError(e))?;
        
        Ok(data.len() as u64)
    }

    /// Export memory tracking data asynchronously
    pub async fn export_async<P: AsRef<Path>>(
        &self,
        tracker: &MemoryTracker,
        path: P,
    ) -> Result<ExportResult, BinaryExportError> {
        let start_time = Instant::now();
        let path = path.as_ref();
        
        // Validate input parameters
        self.validate_export_params(tracker, path)?;
        
        // Collect data from tracker asynchronously
        let collection_start = Instant::now();
        let data = self.collect_data_async(tracker).await?;
        let collection_time = collection_start.elapsed();
        
        // Serialize data in background task
        let serialization_start = Instant::now();
        let serialized_data = tokio::task::spawn_blocking({
            let data = data.clone();
            move || bincode::serialize(&data)
        }).await
        .map_err(|e| BinaryExportError::InternalError(e.to_string()))?
        .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        let serialization_time = serialization_start.elapsed();
        
        // Compress data if enabled (in background)
        let (final_data, compression_time, compression_ratio) = if self.config.compression_enabled {
            let compression_start = Instant::now();
            let compressed = tokio::task::spawn_blocking({
                let data = serialized_data.clone();
                let level = self.config.compression_level;
                move || zstd::bulk::compress(&data, level)
            }).await
            .map_err(|e| BinaryExportError::InternalError(e.to_string()))?
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
            
            let compression_time = compression_start.elapsed();
            let ratio = compressed.len() as f64 / serialized_data.len() as f64;
            (compressed, Some(compression_time), Some(ratio))
        } else {
            (serialized_data, None, None)
        };
        
        // Write to file asynchronously
        let write_start = Instant::now();
        let bytes_written = self.write_to_file_async(&final_data, path).await?;
        let write_time = write_start.elapsed();
        
        let total_duration = start_time.elapsed();
        
        // Build result
        Ok(ExportResult {
            bytes_written,
            duration: total_duration,
            compression_ratio,
            allocation_count: data.allocations.allocations.len(),
            warnings: Vec::new(),
            stats: ExportStats {
                collection_time,
                serialization_time,
                compression_time,
                write_time,
                original_size: serialized_data.len() as u64,
                final_size: bytes_written,
                peak_memory_usage: self.memory_manager.peak_usage(),
            },
        })
    }

    /// Check if data is compressed
    fn is_compressed(&self, data: &[u8]) -> Result<bool, BinaryExportError> {
        // Simple heuristic: check for zstd magic number
        Ok(data.len() >= 4 && &data[0..4] == b"\x28\xb5\x2f\xfd")
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        zstd::bulk::decompress(data, self.config.max_memory_usage)
            .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
    }

    /// Deserialize data from bytes
    fn deserialize_data(&self, data: &[u8]) -> Result<UnifiedData, BinaryExportError> {
        bincode::deserialize(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }
    
    /// Collect data from memory tracker asynchronously
    async fn collect_data_async(&self, tracker: &MemoryTracker) -> Result<UnifiedData, BinaryExportError> {
        // Run data collection in a background task to avoid blocking
        let collector = crate::export::binary::data::DataCollector::new(self.config.clone().into());
        
        tokio::task::spawn_blocking(move || {
            collector.collect_from_tracker(tracker)
        }).await
        .map_err(|e| BinaryExportError::InternalError(e.to_string()))?
    }
    
    /// Write data to file asynchronously
    async fn write_to_file_async(&self, data: &[u8], path: &Path) -> Result<u64, BinaryExportError> {
        tokio::fs::write(path, data).await
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        Ok(data.len() as u64)
    }
}

impl From<ExportConfig> for crate::export::binary::data::CollectionConfig {
    fn from(config: ExportConfig) -> Self {
        Self {
            max_memory_usage: config.max_memory_usage,
            include_call_stacks: true,
            max_call_stack_depth: 32,
            enable_expensive_analysis: !config.compression_enabled, // Trade-off
            enable_parallel_collection: true,
            collection_timeout: std::time::Duration::from_secs(config.timeout_secs),
            chunk_size: config.chunk_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::MemoryTracker;
    use tempfile::NamedTempFile;

    #[test]
    fn test_export_config_presets() {
        let fast = ExportConfig::fast();
        assert!(!fast.compression_enabled);
        assert_eq!(fast.compression_level, 1);
        
        let compact = ExportConfig::compact();
        assert!(compact.compression_enabled);
        assert_eq!(compact.compression_level, 19);
    }

    #[test]
    fn test_binary_exporter_creation() {
        let config = ExportConfig::default();
        let exporter = BinaryExporter::new(config);
        assert!(exporter.config.compression_enabled);
    }

    #[test]
    fn test_export_validation() {
        let config = ExportConfig::default();
        let exporter = BinaryExporter::new(config);
        let tracker = MemoryTracker::new();
        
        // Should fail with no data
        let temp_file = NamedTempFile::new().unwrap();
        let result = exporter.export(&tracker, temp_file.path());
        assert!(result.is_err());
    }
}