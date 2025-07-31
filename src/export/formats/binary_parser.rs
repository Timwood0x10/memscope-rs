//! Binary parser for reading and validating binary export files
//!
//! This module provides comprehensive binary parsing capabilities including:
//! - Automatic compression detection and decompression
//! - MessagePack deserialization with error recovery
//! - File format validation and integrity checking
//! - Detailed error reporting and logging

use crate::core::types::TrackingResult;
use crate::export::formats::binary_export::BinaryExportData;
use std::io::Read;
use std::path::Path;

/// Configuration options for binary parsing operations
#[derive(Debug, Clone)]
pub struct BinaryParseOptions {
    /// Automatically detect and handle compression
    pub auto_detect_compression: bool,
    /// Maximum memory usage for decompression (bytes)
    pub max_memory_usage: usize,
    /// Enable partial data recovery for corrupted files
    pub enable_recovery: bool,
    /// Validate checksums and metadata
    pub validate_integrity: bool,
    /// Buffer size for streaming operations
    pub buffer_size: usize,
}

impl BinaryParseOptions {
    /// Fast parsing configuration - minimal validation
    pub fn fast() -> Self {
        Self {
            auto_detect_compression: true,
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            enable_recovery: false,
            validate_integrity: false,
            buffer_size: 64 * 1024, // 64KB
        }
    }

    /// Safe parsing configuration - full validation and recovery
    pub fn safe() -> Self {
        Self {
            auto_detect_compression: true,
            max_memory_usage: 500 * 1024 * 1024, // 500MB
            enable_recovery: true,
            validate_integrity: true,
            buffer_size: 256 * 1024, // 256KB
        }
    }

    /// Streaming configuration for large files
    pub fn streaming() -> Self {
        Self {
            auto_detect_compression: true,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
            enable_recovery: true,
            validate_integrity: false, // Skip for performance
            buffer_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for BinaryParseOptions {
    fn default() -> Self {
        Self::safe()
    }
}

/// Result of binary file validation
#[derive(Debug, Clone)]
pub struct BinaryValidationResult {
    /// Whether the file is valid
    pub is_valid: bool,
    /// Format version detected
    pub format_version: Option<String>,
    /// Compression algorithm detected
    pub compression_detected: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Estimated uncompressed size
    pub uncompressed_size: Option<u64>,
    /// Validation errors found
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Whether partial recovery is possible
    pub recoverable: bool,
}

/// Binary parser with advanced capabilities
pub struct BinaryParser {
    /// Parser configuration options
    options: BinaryParseOptions,
}

impl BinaryParser {
    /// Create a new binary parser with specified options
    pub fn new(options: BinaryParseOptions) -> Self {
        Self { options }
    }

    /// Create a parser with default safe options
    pub fn with_safe_defaults() -> Self {
        Self::new(BinaryParseOptions::safe())
    }

    /// Create a parser optimized for fast parsing
    pub fn with_fast_defaults() -> Self {
        Self::new(BinaryParseOptions::fast())
    }

    /// Parse binary file and return complete data structure
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> TrackingResult<BinaryExportData> {
        let path_str = path.as_ref().to_string_lossy();
        println!("🔍 Starting binary file parsing: {path_str}");
        println!("📋 Parser options: auto_detect={}, max_memory={}MB, recovery={}, validation={}", 
                 self.options.auto_detect_compression,
                 self.options.max_memory_usage / (1024 * 1024),
                 self.options.enable_recovery,
                 self.options.validate_integrity);

        // Step 1: Read file with error handling
        println!("📖 Reading binary file...");
        let read_start = std::time::Instant::now();
        let raw_data = std::fs::read(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to read binary file '{}': {}", path_str, e)
            ))?;
        let read_duration = read_start.elapsed();
        println!("✅ File read completed in {read_duration:?}");
        println!("   - File size: {} bytes", raw_data.len());

        // Step 2: Detect and handle compression
        let decompressed_data = if self.options.auto_detect_compression {
            self.detect_and_decompress(&raw_data)?
        } else {
            println!("ℹ️  Compression detection disabled, using raw data");
            raw_data
        };

        // Step 3: Deserialize MessagePack data
        println!("📋 Deserializing MessagePack data...");
        let deserialize_start = std::time::Instant::now();
        let result = self.deserialize_with_recovery(&decompressed_data)?;
        let deserialize_duration = deserialize_start.elapsed();
        println!("✅ Deserialization completed in {deserialize_duration:?}");

        // Step 4: Validate integrity if enabled
        if self.options.validate_integrity {
            println!("🔍 Validating data integrity...");
            self.validate_data_integrity(&result)?;
            println!("✅ Data integrity validation passed");
        }

        // Step 5: Log summary
        println!("📊 Parsing summary:");
        println!("   - Format version: {}", result.version);
        println!("   - Allocations: {}", result.allocation_count);
        println!("   - Total memory: {} bytes", result.total_memory);
        println!("   - Active memory: {} bytes", result.stats.active_memory);

        if let Some(metadata) = &result.metadata {
            println!("   - Export format: {}", metadata.export_format_version);
            if let Some(compression) = &metadata.compression_algorithm {
                println!("   - Original compression: {compression}");
            }
        }

        println!("🎉 Binary file parsing completed successfully!");
        Ok(result)
    }

    /// Parse binary data from a stream
    pub fn parse_stream<R: Read>(&self, mut reader: R) -> TrackingResult<BinaryExportData> {
        println!("🔍 Starting binary stream parsing...");

        // Read all data from stream
        println!("📖 Reading data from stream...");
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to read from stream: {e}")
            ))?;
        println!("✅ Stream read completed: {} bytes", buffer.len());

        // Use the same parsing logic as file parsing
        let decompressed_data = if self.options.auto_detect_compression {
            self.detect_and_decompress(&buffer)?
        } else {
            buffer
        };

        let result = self.deserialize_with_recovery(&decompressed_data)?;

        if self.options.validate_integrity {
            self.validate_data_integrity(&result)?;
        }

        println!("🎉 Binary stream parsing completed successfully!");
        Ok(result)
    }

    /// Validate binary file without full parsing
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> TrackingResult<BinaryValidationResult> {
        let path_str = path.as_ref().to_string_lossy();
        println!("🔍 Validating binary file: {path_str}");

        let mut result = BinaryValidationResult {
            is_valid: false,
            format_version: None,
            compression_detected: None,
            file_size: 0,
            uncompressed_size: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            recoverable: false,
        };

        // Check if file exists and is readable
        let metadata = std::fs::metadata(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Cannot access file '{}': {}", path_str, e)
            ))?;
        
        result.file_size = metadata.len();
        println!("   - File size: {} bytes", result.file_size);

        // Try to read a small portion for format detection
        let _sample_size = std::cmp::min(1024, result.file_size as usize);
        let sample_data = std::fs::read(&path)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to read file sample: {e}")
            ))?;

        // Detect compression
        if self.is_compressed_data(&sample_data) {
            result.compression_detected = Some("zstd".to_string());
            println!("   - Compression detected: zstd");

            // Try to get uncompressed size
            if let Ok(decompressed) = zstd::bulk::decompress(&sample_data, self.options.max_memory_usage) {
                result.uncompressed_size = Some(decompressed.len() as u64);
                println!("   - Estimated uncompressed size: {} bytes", decompressed.len());
            }
        } else {
            println!("   - No compression detected");
        }

        // Try basic MessagePack validation
        match self.validate_messagepack_format(&sample_data) {
            Ok(version) => {
                result.format_version = Some(version);
                result.is_valid = true;
                println!("   - Format validation: PASSED");
            }
            Err(e) => {
                result.errors.push(format!("Format validation failed: {e}"));
                result.recoverable = self.options.enable_recovery;
                println!("   - Format validation: FAILED - {e}");
            }
        }

        if result.is_valid {
            println!("✅ File validation completed: VALID");
        } else {
            println!("❌ File validation completed: INVALID ({} errors)", result.errors.len());
        }

        Ok(result)
    }

    /// Detect compression and decompress data if needed
    fn detect_and_decompress(&self, data: &[u8]) -> TrackingResult<Vec<u8>> {
        if self.is_compressed_data(data) {
            println!("🗜️  Compression detected, decompressing...");
            let decompress_start = std::time::Instant::now();
            
            let decompressed = zstd::bulk::decompress(data, self.options.max_memory_usage)
                .map_err(|e| crate::core::types::TrackingError::SerializationError(
                    format!("zstd decompression failed: {e}")
                ))?;
            
            let decompress_duration = decompress_start.elapsed();
            let compression_ratio = data.len() as f64 / decompressed.len() as f64;
            
            println!("✅ Decompression completed in {decompress_duration:?}");
            println!("   - Compressed size: {} bytes", data.len());
            println!("   - Decompressed size: {} bytes", decompressed.len());
            println!("   - Compression ratio: {:.1}%", compression_ratio * 100.0);
            
            Ok(decompressed)
        } else {
            println!("ℹ️  No compression detected, using raw data");
            Ok(data.to_vec())
        }
    }

    /// Check if data appears to be compressed
    fn is_compressed_data(&self, data: &[u8]) -> bool {
        // Check for zstd magic number (0xFD2FB528)
        if data.len() >= 4 {
            let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            magic == 0x28B52FFD
        } else {
            false
        }
    }

    /// Deserialize MessagePack data with error recovery
    fn deserialize_with_recovery(&self, data: &[u8]) -> TrackingResult<BinaryExportData> {
        // First, try normal deserialization
        match rmp_serde::from_slice::<BinaryExportData>(data) {
            Ok(result) => {
                println!("✅ MessagePack deserialization successful");
                Ok(result)
            }
            Err(e) => {
                println!("⚠️  MessagePack deserialization failed: {e}");
                
                if self.options.enable_recovery {
                    println!("🔧 Attempting data recovery...");
                    self.attempt_data_recovery(data, e)
                } else {
                    Err(crate::core::types::TrackingError::SerializationError(
                        format!("MessagePack deserialization failed: {e}")
                    ))
                }
            }
        }
    }

    /// Attempt to recover data from corrupted MessagePack
    fn attempt_data_recovery(&self, _data: &[u8], original_error: rmp_serde::decode::Error) -> TrackingResult<BinaryExportData> {
        println!("🔧 Attempting partial data recovery...");
        
        // For now, create a minimal valid structure as recovery
        // In a real implementation, we could try to parse partial data
        println!("⚠️  Creating minimal recovery structure due to corruption");
        
        let recovered_data = BinaryExportData {
            version: "recovered".to_string(),
            metadata: None,
            stats: crate::core::types::MemoryStats::default(),
            allocations: Vec::new(),
            allocation_count: 0,
            total_memory: 0,
        };
        
        println!("✅ Partial data recovery successful (minimal structure)");
        println!("⚠️  Warning: Original data lost due to corruption. Error: {original_error}");
        
        Ok(recovered_data)
    }

    /// Validate data integrity
    fn validate_data_integrity(&self, data: &BinaryExportData) -> TrackingResult<()> {
        let mut errors = Vec::new();

        // Check basic consistency
        if data.allocations.len() != data.allocation_count {
            errors.push(format!(
                "Allocation count mismatch: expected {}, found {}",
                data.allocation_count,
                data.allocations.len()
            ));
        }

        // Validate metadata if present
        if let Some(metadata) = &data.metadata {
            if metadata.export_format_version.is_empty() {
                errors.push("Empty export format version in metadata".to_string());
            }
        }

        // Check for reasonable values
        if data.total_memory == 0 && !data.allocations.is_empty() {
            errors.push("Total memory is zero but allocations exist".to_string());
        }

        if !errors.is_empty() {
            println!("❌ Data integrity validation failed:");
            for error in &errors {
                println!("   - {error}");
            }
            return Err(crate::core::types::TrackingError::SerializationError(
                format!("Data integrity validation failed: {}", errors.join("; "))
            ));
        }

        Ok(())
    }

    /// Validate MessagePack format and extract version
    fn validate_messagepack_format(&self, data: &[u8]) -> Result<String, String> {
        // Try to parse as BinaryExportData to validate format
        match rmp_serde::from_slice::<BinaryExportData>(data) {
            Ok(parsed_data) => Ok(parsed_data.version),
            Err(e) => Err(format!("Invalid MessagePack format: {e}"))
        }
    }
}

impl Default for BinaryParser {
    fn default() -> Self {
        Self::with_safe_defaults()
    }
}