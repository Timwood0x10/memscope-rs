//! Format management system for binary export
//!
//! This module provides a flexible format management system that supports
//! multiple output formats with automatic format detection and selection.

use std::collections::HashMap;
use std::io::Write;
use serde::{Serialize, Deserialize};

use super::core::UnifiedData;
use super::error::BinaryExportError;
use super::processor::ProcessedData;

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputFormat {
    /// MessagePack format (maximum compatibility)
    MessagePack,
    /// Custom binary format (optimized for speed)
    CustomBinary,
    /// Compressed MessagePack
    CompressedMessagePack { level: i32 },
    /// Chunked format for large datasets
    Chunked { chunk_size: usize },
    /// Raw binary format
    Raw,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::MessagePack
    }
}

impl OutputFormat {
    /// Get format name as string
    pub fn name(&self) -> &'static str {
        match self {
            OutputFormat::MessagePack => "messagepack",
            OutputFormat::CustomBinary => "custom_binary",
            OutputFormat::CompressedMessagePack { .. } => "compressed_messagepack",
            OutputFormat::Chunked { .. } => "chunked",
            OutputFormat::Raw => "raw",
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::MessagePack => "msgpack",
            OutputFormat::CustomBinary => "bin",
            OutputFormat::CompressedMessagePack { .. } => "msgpack.zst",
            OutputFormat::Chunked { .. } => "chunks",
            OutputFormat::Raw => "raw",
        }
    }

    /// Check if format supports streaming
    pub fn supports_streaming(&self) -> bool {
        match self {
            OutputFormat::MessagePack => true,
            OutputFormat::CustomBinary => true,
            OutputFormat::CompressedMessagePack { .. } => true,
            OutputFormat::Chunked { .. } => true,
            OutputFormat::Raw => true,
        }
    }

    /// Get format magic bytes for detection
    pub fn magic_bytes(&self) -> &'static [u8] {
        match self {
            OutputFormat::MessagePack => &[0x82, 0xa7], // MessagePack map with string key
            OutputFormat::CustomBinary => b"MEMBIN",
            OutputFormat::CompressedMessagePack { .. } => &[0x28, 0xb5, 0x2f, 0xfd], // zstd magic
            OutputFormat::Chunked { .. } => b"CHUNK",
            OutputFormat::Raw => &[0x00, 0x01], // Simple raw format marker
        }
    }
}

/// Format writer trait for different output formats
pub trait FormatWriter: Send + Sync {
    /// Write data in the specific format
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize>;
    
    /// Estimate output size for the given data
    fn estimate_size(&self, data: &ProcessedData) -> usize;
    
    /// Check if this writer supports streaming
    fn supports_streaming(&self) -> bool;
    
    /// Get format identifier
    fn format(&self) -> OutputFormat;
    
    /// Write format header (if needed)
    fn write_header(&self, writer: &mut dyn Write) -> FormatResult<usize> {
        // Default implementation writes magic bytes
        let magic = self.format().magic_bytes();
        writer.write_all(magic)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        Ok(magic.len())
    }
    
    /// Write format footer (if needed)
    fn write_footer(&self, writer: &mut dyn Write) -> FormatResult<usize> {
        // Default implementation does nothing
        Ok(0)
    }
}

/// Format-specific error types
#[derive(Debug, Clone)]
pub enum FormatError {
    /// I/O error during format operations
    IoError(std::io::ErrorKind),
    /// Serialization error
    SerializationError(String),
    /// Unsupported format feature
    UnsupportedFeature(String),
    /// Format validation error
    ValidationError(String),
    /// Compression error
    CompressionError(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::IoError(kind) => write!(f, "I/O error: {:?}", kind),
            FormatError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            FormatError::UnsupportedFeature(feature) => write!(f, "Unsupported feature: {}", feature),
            FormatError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            FormatError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<FormatError> for BinaryExportError {
    fn from(error: FormatError) -> Self {
        match error {
            FormatError::IoError(kind) => BinaryExportError::IoError(kind),
            FormatError::SerializationError(msg) => BinaryExportError::SerializationError(msg),
            FormatError::CompressionError(msg) => BinaryExportError::CompressionError(msg),
            FormatError::UnsupportedFeature(feature) => BinaryExportError::UnsupportedFeature(feature),
            FormatError::ValidationError(msg) => BinaryExportError::ValidationFailed(msg),
        }
    }
}

/// Result type for format operations
pub type FormatResult<T> = Result<T, FormatError>;

/// Format detection result
#[derive(Debug, Clone)]
pub struct FormatDetectionResult {
    /// Detected format
    pub format: OutputFormat,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Additional format metadata
    pub metadata: HashMap<String, String>,
}

/// Format detector for automatic format recognition
pub struct FormatDetector {
    /// Known format signatures
    signatures: HashMap<Vec<u8>, OutputFormat>,
}

impl FormatDetector {
    /// Create a new format detector
    pub fn new() -> Self {
        let mut signatures = HashMap::new();
        
        // Register format signatures
        signatures.insert(OutputFormat::MessagePack.magic_bytes().to_vec(), OutputFormat::MessagePack);
        signatures.insert(OutputFormat::CustomBinary.magic_bytes().to_vec(), OutputFormat::CustomBinary);
        signatures.insert(OutputFormat::CompressedMessagePack { level: 6 }.magic_bytes().to_vec(), 
                         OutputFormat::CompressedMessagePack { level: 6 });
        signatures.insert(OutputFormat::Chunked { chunk_size: 256 * 1024 }.magic_bytes().to_vec(), 
                         OutputFormat::Chunked { chunk_size: 256 * 1024 });
        signatures.insert(OutputFormat::Raw.magic_bytes().to_vec(), OutputFormat::Raw);
        
        Self { signatures }
    }

    /// Detect format from data header
    pub fn detect_format(&self, data: &[u8]) -> FormatResult<FormatDetectionResult> {
        if data.len() < 4 {
            return Err(FormatError::ValidationError("Data too short for format detection".to_string()));
        }

        // Try exact magic byte matches first
        for (signature, format) in &self.signatures {
            if data.starts_with(signature) {
                return Ok(FormatDetectionResult {
                    format: *format,
                    confidence: 1.0,
                    metadata: HashMap::new(),
                });
            }
        }

        // Try heuristic detection
        self.detect_format_heuristic(data)
    }

    /// Heuristic format detection based on data patterns
    fn detect_format_heuristic(&self, data: &[u8]) -> FormatResult<FormatDetectionResult> {
        let mut metadata = HashMap::new();
        
        // Check for MessagePack patterns
        if data[0] >= 0x80 && data[0] <= 0x8f {
            // MessagePack fixmap
            metadata.insert("detection_method".to_string(), "heuristic_msgpack".to_string());
            return Ok(FormatDetectionResult {
                format: OutputFormat::MessagePack,
                confidence: 0.8,
                metadata,
            });
        }

        // Check for zstd compression
        if data.len() >= 4 && &data[0..4] == &[0x28, 0xb5, 0x2f, 0xfd] {
            metadata.insert("detection_method".to_string(), "heuristic_zstd".to_string());
            return Ok(FormatDetectionResult {
                format: OutputFormat::CompressedMessagePack { level: 6 },
                confidence: 0.9,
                metadata,
            });
        }

        // Default to raw format with low confidence
        metadata.insert("detection_method".to_string(), "fallback".to_string());
        Ok(FormatDetectionResult {
            format: OutputFormat::Raw,
            confidence: 0.1,
            metadata,
        })
    }
}

/// Main format manager
pub struct FormatManager {
    /// Registered format writers
    writers: HashMap<OutputFormat, Box<dyn FormatWriter>>,
    /// Format detector
    detector: FormatDetector,
    /// Format compatibility matrix
    compatibility: HashMap<OutputFormat, Vec<OutputFormat>>,
}

impl FormatManager {
    /// Create a new format manager
    pub fn new() -> Self {
        let mut manager = Self {
            writers: HashMap::new(),
            detector: FormatDetector::new(),
            compatibility: HashMap::new(),
        };

        // Register default format writers
        manager.register_default_writers();
        manager.setup_compatibility_matrix();
        
        manager
    }

    /// Register a format writer
    pub fn register_writer(&mut self, writer: Box<dyn FormatWriter>) {
        let format = writer.format();
        self.writers.insert(format, writer);
    }

    /// Get format writer for the specified format
    pub fn get_writer(&self, format: OutputFormat) -> Option<&dyn FormatWriter> {
        self.writers.get(&format).map(|w| w.as_ref())
    }

    /// Write data in the specified format
    pub fn write_data(
        &self,
        data: &ProcessedData,
        format: OutputFormat,
        writer: &mut dyn Write,
    ) -> FormatResult<usize> {
        let format_writer = self.get_writer(format)
            .ok_or_else(|| FormatError::UnsupportedFeature(format!("Format {:?} not supported", format)))?;

        let mut bytes_written = 0;
        
        // Write header
        bytes_written += format_writer.write_header(writer)?;
        
        // Write data
        bytes_written += format_writer.write_data(data, writer)?;
        
        // Write footer
        bytes_written += format_writer.write_footer(writer)?;
        
        Ok(bytes_written)
    }

    /// Detect format from data
    pub fn detect_format(&self, data: &[u8]) -> FormatResult<FormatDetectionResult> {
        self.detector.detect_format(data)
    }

    /// Get compatible formats for conversion
    pub fn get_compatible_formats(&self, source_format: OutputFormat) -> Vec<OutputFormat> {
        self.compatibility.get(&source_format)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Estimate output size for format
    pub fn estimate_size(&self, data: &ProcessedData, format: OutputFormat) -> Option<usize> {
        self.get_writer(format).map(|writer| writer.estimate_size(data))
    }

    /// Check if format supports streaming
    pub fn supports_streaming(&self, format: OutputFormat) -> bool {
        self.get_writer(format)
            .map(|writer| writer.supports_streaming())
            .unwrap_or(false)
    }

    /// Get all supported formats
    pub fn supported_formats(&self) -> Vec<OutputFormat> {
        self.writers.keys().cloned().collect()
    }

    /// Register default format writers
    fn register_default_writers(&mut self) {
        // Register built-in format writers
        self.register_writer(Box::new(MessagePackWriter::new()));
        self.register_writer(Box::new(CustomBinaryWriter::new()));
        self.register_writer(Box::new(CompressedMessagePackWriter::new(6)));
        self.register_writer(Box::new(ChunkedWriter::new(256 * 1024)));
        self.register_writer(Box::new(RawWriter::new()));
    }

    /// Setup format compatibility matrix
    fn setup_compatibility_matrix(&mut self) {
        // MessagePack can be converted to most formats
        self.compatibility.insert(OutputFormat::MessagePack, vec![
            OutputFormat::CustomBinary,
            OutputFormat::CompressedMessagePack { level: 6 },
            OutputFormat::Raw,
        ]);

        // Custom binary can be converted to MessagePack and Raw
        self.compatibility.insert(OutputFormat::CustomBinary, vec![
            OutputFormat::MessagePack,
            OutputFormat::Raw,
        ]);

        // Compressed formats can be decompressed to their base formats
        self.compatibility.insert(OutputFormat::CompressedMessagePack { level: 6 }, vec![
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary,
            OutputFormat::Raw,
        ]);

        // Chunked format can be reassembled to other formats
        self.compatibility.insert(OutputFormat::Chunked { chunk_size: 256 * 1024 }, vec![
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary,
            OutputFormat::Raw,
        ]);

        // Raw format is the most basic and can be converted to structured formats
        self.compatibility.insert(OutputFormat::Raw, vec![
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary,
        ]);
    }
}

impl Default for FormatManager {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in format writers

/// MessagePack format writer
pub struct MessagePackWriter {
    /// Writer configuration
    config: MessagePackConfig,
}

#[derive(Debug, Clone)]
struct MessagePackConfig {
    /// Use compact representation
    compact: bool,
    /// Include type information
    include_types: bool,
}

impl MessagePackWriter {
    pub fn new() -> Self {
        Self {
            config: MessagePackConfig {
                compact: true,
                include_types: false,
            },
        }
    }
}

impl FormatWriter for MessagePackWriter {
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize> {
        // Serialize to MessagePack format
        let msgpack_data = rmp_serde::to_vec(&data.data)
            .map_err(|e| FormatError::SerializationError(e.to_string()))?;
        
        writer.write_all(&msgpack_data)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        
        Ok(msgpack_data.len())
    }

    fn estimate_size(&self, data: &ProcessedData) -> usize {
        // MessagePack is typically 10-20% smaller than JSON
        (data.data.len() as f64 * 0.85) as usize
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::MessagePack
    }
}

/// Custom binary format writer
pub struct CustomBinaryWriter {
    /// Writer configuration
    config: CustomBinaryConfig,
}

#[derive(Debug, Clone)]
struct CustomBinaryConfig {
    /// Use little-endian byte order
    little_endian: bool,
    /// Include checksums
    include_checksums: bool,
}

impl CustomBinaryWriter {
    pub fn new() -> Self {
        Self {
            config: CustomBinaryConfig {
                little_endian: true,
                include_checksums: true,
            },
        }
    }
}

impl FormatWriter for CustomBinaryWriter {
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize> {
        let mut bytes_written = 0;
        
        // Write data length (4 bytes, little-endian)
        let data_len = data.data.len() as u32;
        let len_bytes = if self.config.little_endian {
            data_len.to_le_bytes()
        } else {
            data_len.to_be_bytes()
        };
        
        writer.write_all(&len_bytes)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        bytes_written += 4;
        
        // Write data
        writer.write_all(&data.data)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        bytes_written += data.data.len();
        
        // Write checksum if enabled
        if self.config.include_checksums {
            let checksum = crc32fast::hash(&data.data);
            let checksum_bytes = if self.config.little_endian {
                checksum.to_le_bytes()
            } else {
                checksum.to_be_bytes()
            };
            
            writer.write_all(&checksum_bytes)
                .map_err(|e| FormatError::IoError(e.kind()))?;
            bytes_written += 4;
        }
        
        Ok(bytes_written)
    }

    fn estimate_size(&self, data: &ProcessedData) -> usize {
        let base_size = data.data.len() + 4; // data + length field
        if self.config.include_checksums {
            base_size + 4 // + checksum
        } else {
            base_size
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::CustomBinary
    }
}

/// Compressed MessagePack writer
pub struct CompressedMessagePackWriter {
    /// Compression level
    compression_level: i32,
    /// Base MessagePack writer
    msgpack_writer: MessagePackWriter,
}

impl CompressedMessagePackWriter {
    pub fn new(compression_level: i32) -> Self {
        Self {
            compression_level,
            msgpack_writer: MessagePackWriter::new(),
        }
    }
}

impl FormatWriter for CompressedMessagePackWriter {
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize> {
        // First serialize to MessagePack
        let mut msgpack_buffer = Vec::new();
        self.msgpack_writer.write_data(data, &mut msgpack_buffer)?;
        
        // Then compress
        let compressed_data = zstd::bulk::compress(&msgpack_buffer, self.compression_level)
            .map_err(|e| FormatError::CompressionError(e.to_string()))?;
        
        writer.write_all(&compressed_data)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        
        Ok(compressed_data.len())
    }

    fn estimate_size(&self, data: &ProcessedData) -> usize {
        let msgpack_size = self.msgpack_writer.estimate_size(data);
        // Estimate compression ratio based on level
        let compression_ratio = match self.compression_level {
            1..=3 => 0.7,
            4..=6 => 0.5,
            7..=9 => 0.4,
            _ => 0.3,
        };
        (msgpack_size as f64 * compression_ratio) as usize
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::CompressedMessagePack { level: self.compression_level }
    }
}

/// Chunked format writer for large datasets
pub struct ChunkedWriter {
    /// Chunk size in bytes
    chunk_size: usize,
}

impl ChunkedWriter {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl FormatWriter for ChunkedWriter {
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize> {
        let mut bytes_written = 0;
        let total_chunks = (data.data.len() + self.chunk_size - 1) / self.chunk_size;
        
        // Write chunk count (4 bytes)
        let chunk_count_bytes = (total_chunks as u32).to_le_bytes();
        writer.write_all(&chunk_count_bytes)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        bytes_written += 4;
        
        // Write chunks
        for (chunk_index, chunk) in data.data.chunks(self.chunk_size).enumerate() {
            // Write chunk header (index + size)
            let chunk_header = ChunkHeader {
                index: chunk_index as u32,
                size: chunk.len() as u32,
                checksum: crc32fast::hash(chunk),
            };
            
            let header_bytes = bincode::serialize(&chunk_header)
                .map_err(|e| FormatError::SerializationError(e.to_string()))?;
            
            writer.write_all(&header_bytes)
                .map_err(|e| FormatError::IoError(e.kind()))?;
            bytes_written += header_bytes.len();
            
            // Write chunk data
            writer.write_all(chunk)
                .map_err(|e| FormatError::IoError(e.kind()))?;
            bytes_written += chunk.len();
        }
        
        Ok(bytes_written)
    }

    fn estimate_size(&self, data: &ProcessedData) -> usize {
        let total_chunks = (data.data.len() + self.chunk_size - 1) / self.chunk_size;
        let header_overhead = 4 + total_chunks * 16; // chunk count + chunk headers
        data.data.len() + header_overhead
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Chunked { chunk_size: self.chunk_size }
    }
}

/// Raw format writer (minimal overhead)
pub struct RawWriter;

impl RawWriter {
    pub fn new() -> Self {
        Self
    }
}

impl FormatWriter for RawWriter {
    fn write_data(&self, data: &ProcessedData, writer: &mut dyn Write) -> FormatResult<usize> {
        writer.write_all(&data.data)
            .map_err(|e| FormatError::IoError(e.kind()))?;
        Ok(data.data.len())
    }

    fn estimate_size(&self, data: &ProcessedData) -> usize {
        data.data.len()
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn format(&self) -> OutputFormat {
        OutputFormat::Raw
    }
}

/// Chunk header for chunked format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunkHeader {
    /// Chunk index
    index: u32,
    /// Chunk size in bytes
    size: u32,
    /// Chunk checksum
    checksum: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::binary::processor::{ProcessedData, ProcessingMetadata, ProcessingMethod, DataFormat, ValidationResults};

    fn create_test_data() -> ProcessedData {
        ProcessedData {
            data: b"Hello, world! This is test data.".to_vec(),
            metadata: ProcessingMetadata {
                timestamp: std::time::SystemTime::now(),
                method: ProcessingMethod::Batch,
                format: DataFormat::Bincode,
                compression: None,
                config_hash: 12345,
            },
            validation_results: ValidationResults::default(),
        }
    }

    #[test]
    fn test_format_manager_creation() {
        let manager = FormatManager::new();
        let supported = manager.supported_formats();
        assert!(!supported.is_empty());
        assert!(supported.contains(&OutputFormat::MessagePack));
        assert!(supported.contains(&OutputFormat::CustomBinary));
    }

    #[test]
    fn test_messagepack_writer() {
        let writer = MessagePackWriter::new();
        let test_data = create_test_data();
        let mut output = Vec::new();
        
        let result = writer.write_data(&test_data, &mut output);
        assert!(result.is_ok());
        assert!(!output.is_empty());
        
        let estimated_size = writer.estimate_size(&test_data);
        assert!(estimated_size > 0);
    }

    #[test]
    fn test_custom_binary_writer() {
        let writer = CustomBinaryWriter::new();
        let test_data = create_test_data();
        let mut output = Vec::new();
        
        let result = writer.write_data(&test_data, &mut output);
        assert!(result.is_ok());
        assert!(!output.is_empty());
        
        // Check that output starts with magic bytes
        assert!(output.starts_with(OutputFormat::CustomBinary.magic_bytes()));
    }

    #[test]
    fn test_format_detection() {
        let detector = FormatDetector::new();
        
        // Test MessagePack detection
        let msgpack_data = [0x82, 0xa7, 0x74, 0x65, 0x73, 0x74]; // MessagePack data
        let result = detector.detect_format(&msgpack_data);
        assert!(result.is_ok());
        
        let detection = result.unwrap();
        assert_eq!(detection.format, OutputFormat::MessagePack);
        assert!(detection.confidence > 0.5);
    }

    #[test]
    fn test_chunked_writer() {
        let writer = ChunkedWriter::new(10); // Small chunks for testing
        let test_data = create_test_data();
        let mut output = Vec::new();
        
        let result = writer.write_data(&test_data, &mut output);
        assert!(result.is_ok());
        assert!(!output.is_empty());
        
        // Should have chunk count at the beginning
        assert!(output.len() > 4);
    }

    #[test]
    fn test_format_compatibility() {
        let manager = FormatManager::new();
        let compatible = manager.get_compatible_formats(OutputFormat::MessagePack);
        assert!(!compatible.is_empty());
        assert!(compatible.contains(&OutputFormat::CustomBinary));
    }
}