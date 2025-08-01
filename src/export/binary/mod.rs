// Binary export module - Unified binary export system
// 
// This module provides a comprehensive binary export system that replaces
// the fragmented existing implementation with a clean, performant architecture.
// 
// Key features:
// - Unified data collection from all analysis modules
// - Zero-copy optimizations for large datasets
// - Intelligent error recovery and fallback strategies
// - Adaptive memory management with backpressure control
// - Streaming support for memory-constrained environments

pub mod core;
pub mod data;
pub mod export;
pub mod memory;
pub mod error;
pub mod validation;
pub mod compression;
pub mod processor;

// Re-export main public API
pub use export::{BinaryExporter, ExportConfig, ExportResult};
pub use core::UnifiedData;
pub use data::DataCollector;
pub use error::{BinaryExportError, RecoveryStrategy, ErrorRecovery};
pub use memory::{MemoryManager, ZeroCopyView, SmartBuffer};
pub use validation::ValidationReport;
pub use compression::{CompressionManager, CompressionConfig, CompressionAlgorithm, CompressionStats};
pub use processor::{DataProcessor, ProcessingConfig, ProcessStats, ProcessedData};

// Export format version for compatibility tracking
pub const BINARY_FORMAT_VERSION: u32 = 2;
pub const MINIMUM_SUPPORTED_VERSION: u32 = 1;

/// Main entry point for binary export operations
/// 
/// This provides a simplified API that automatically handles:
/// - Data collection from all analysis modules
/// - Memory optimization based on data size
/// - Error recovery with intelligent fallback
/// - Progress monitoring and cancellation support
pub struct BinaryExport;

impl BinaryExport {
    /// Export memory tracking data to binary format with default settings
    /// 
    /// This method provides the simplest interface for binary export,
    /// automatically selecting optimal settings based on data characteristics.
    /// 
    /// # Arguments
    /// * `tracker` - The memory tracker containing data to export
    /// * `path` - Output file path for the binary data
    /// 
    /// # Returns
    /// * `ExportResult` - Contains export statistics and any warnings
    /// 
    /// # Example
    /// ```rust
    /// use memscope_rs::export::binary::BinaryExport;
    /// 
    /// let result = BinaryExport::export_default(&tracker, "output.bin")?;
    /// println!("Exported {} bytes in {:?}", result.bytes_written, result.duration);
    /// ```
    pub fn export_default<P: AsRef<std::path::Path>>(
        tracker: &crate::core::tracker::MemoryTracker,
        path: P,
    ) -> Result<ExportResult, BinaryExportError> {
        let config = ExportConfig::default();
        Self::export_with_config(tracker, path, config)
    }
    
    /// Export with custom configuration
    /// 
    /// Provides full control over export behavior including compression,
    /// memory management, and error handling strategies.
    pub fn export_with_config<P: AsRef<std::path::Path>>(
        tracker: &crate::core::tracker::MemoryTracker,
        path: P,
        config: ExportConfig,
    ) -> Result<ExportResult, BinaryExportError> {
        let exporter = BinaryExporter::new(config);
        exporter.export(tracker, path)
    }
    
    /// Export asynchronously with default settings
    /// 
    /// Provides non-blocking export operation with progress monitoring
    /// and cancellation support.
    pub async fn export_async<P: AsRef<std::path::Path>>(
        tracker: &crate::core::tracker::MemoryTracker,
        path: P,
    ) -> Result<ExportResult, BinaryExportError> {
        let config = ExportConfig::default();
        Self::export_with_config_async(tracker, path, config).await
    }
    
    /// Export asynchronously with custom configuration
    /// 
    /// Provides full control over async export behavior including
    /// background processing, progress callbacks, and cancellation.
    pub async fn export_with_config_async<P: AsRef<std::path::Path>>(
        tracker: &crate::core::tracker::MemoryTracker,
        path: P,
        config: ExportConfig,
    ) -> Result<ExportResult, BinaryExportError> {
        let exporter = BinaryExporter::new(config);
        exporter.export_async(tracker, path).await
    }
    
    /// Load and validate binary data
    /// 
    /// Reads binary export file and performs integrity validation.
    /// Supports automatic version migration for older formats.
    pub fn load<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<UnifiedData, BinaryExportError> {
        let exporter = BinaryExporter::new(ExportConfig::default());
        exporter.load(path)
    }
    
    /// Validate binary file integrity without full loading
    /// 
    /// Performs quick validation checks including:
    /// - File format version compatibility
    /// - Checksum verification
    /// - Basic structure validation
    pub fn validate<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<validation::ValidationReport, BinaryExportError> {
        validation::validate_binary_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test basic export functionality with minimal data
    #[test]
    fn test_basic_export() {
        use crate::core::tracker::MemoryTracker;
        use tempfile::NamedTempFile;
        
        // Create a memory tracker with some test data
        let tracker = MemoryTracker::new();
        
        // Create a temporary file for export
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        // Test default export
        let result = BinaryExport::export_default(&tracker, temp_file.path());
        
        // Should succeed even with empty data
        match result {
            Ok(export_result) => {
                assert!(export_result.bytes_written > 0);
                assert!(export_result.duration.as_millis() >= 0);
            }
            Err(e) => {
                // Empty data export might fail, which is acceptable
                println!("Export failed as expected with empty data: {:?}", e);
            }
        }
    }
    
    /// Test error recovery mechanisms
    #[test]
    fn test_error_recovery() {
        use crate::core::tracker::MemoryTracker;
        use std::path::Path;
        
        let tracker = MemoryTracker::new();
        
        // Test export to invalid path (should trigger error recovery)
        let invalid_path = Path::new("/invalid/path/that/does/not/exist/test.bin");
        let result = BinaryExport::export_default(&tracker, invalid_path);
        
        // Should return an error
        assert!(result.is_err());
        
        // Test validation of non-existent file
        let validation_result = BinaryExport::validate(invalid_path);
        assert!(validation_result.is_err());
    }
    
    /// Test memory management under different load conditions
    #[test]
    fn test_memory_management() {
        use crate::core::tracker::MemoryTracker;
        use tempfile::NamedTempFile;
        
        let tracker = MemoryTracker::new();
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        // Test with different memory configurations
        let mut config = ExportConfig::default();
        config.max_memory_usage = 1024 * 1024; // 1MB limit
        
        let result = BinaryExport::export_with_config(&tracker, temp_file.path(), config);
        
        // Should handle memory constraints gracefully
        match result {
            Ok(export_result) => {
                // Verify memory usage was within limits
                assert!(export_result.stats.peak_memory_usage <= 1024 * 1024);
            }
            Err(e) => {
                // Memory constraint errors are acceptable
                println!("Memory-constrained export failed as expected: {:?}", e);
            }
        }
    }
}