//! Binary data parser for reading and converting binary export files
//!
//! This module provides comprehensive parsing capabilities for binary export files,
//! including automatic format detection, version handling, and data conversion.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::Path;
use serde::{Serialize, Deserialize};

use super::*;

/// Binary data parser with format detection and conversion capabilities
pub struct BinaryDataParser {
    /// Parser configuration
    config: ParserConfig,
    /// Format detector
    format_detector: FormatDetector,
    /// Version compatibility handler
    version_handler: VersionHandler,
    /// Data converter
    data_converter: DataConverter,
}

/// Configuration for binary data parsing
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Enable automatic format detection
    pub auto_detect_format: bool,
    /// Enable version migration
    pub enable_version_migration: bool,
    /// Enable data validation during parsing
    pub validate_data: bool,
    /// Maximum file size to parse (bytes)
    pub max_file_size: u64,
    /// Enable streaming parsing for large files
    pub enable_streaming: bool,
    /// Chunk size for streaming parsing
    pub chunk_size: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            auto_detect_format: true,
            enable_version_migration: true,
            validate_data: true,
            max_file_size: 1024 * 1024 * 1024, // 1GB
            enable_streaming: true,
            chunk_size: 256 * 1024, // 256KB
        }
    }
}

/// Parsing result with metadata
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed unified data
    pub data: UnifiedData,
    /// Detected format information
    pub format_info: DetectedFormat,
    /// Parsing statistics
    pub parse_stats: ParseStats,
    /// Validation results
    pub validation_results: ValidationResults,
}

/// Detected format information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFormat {
    /// Detected output format
    pub format: OutputFormat,
    /// Format version
    pub version: u32,
    /// Compression information
    pub compression: Option<CompressionInfo>,
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Parsing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStats {
    /// Total parsing time
    pub parse_time: std::time::Duration,
    /// File size processed
    pub file_size: u64,
    /// Parsing throughput (bytes/second)
    pub throughput: f64,
    /// Memory usage during parsing
    pub peak_memory_usage: usize,
    /// Number of data structures parsed
    pub structures_parsed: u32,
}

/// Version compatibility handler
struct VersionHandler {
    /// Supported version migrations
    migrations: HashMap<u32, Box<dyn VersionMigration>>,
}

/// Version migration trait
trait VersionMigration: Send + Sync {
    /// Migrate data from old version to current version
    fn migrate(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError>;
    /// Get source version
    fn source_version(&self) -> u32;
    /// Get target version
    fn target_version(&self) -> u32;
}

/// Data converter for format transformations
struct DataConverter {
    /// Available converters
    converters: HashMap<(OutputFormat, OutputFormat), Box<dyn FormatConverter>>,
}

/// Format converter trait
trait FormatConverter: Send + Sync {
    /// Convert data from source format to target format
    fn convert(&self, data: &[u8], source: OutputFormat, target: OutputFormat) -> Result<Vec<u8>, BinaryExportError>;
}

impl BinaryDataParser {
    /// Create a new binary data parser
    pub fn new(config: ParserConfig) -> Self {
        Self {
            config,
            format_detector: FormatDetector::new(),
            version_handler: VersionHandler::new(),
            data_converter: DataConverter::new(),
        }
    }

    /// Parse binary data from file
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<ParseResult, BinaryExportError> {
        let path = path.as_ref();
        let start_time = std::time::Instant::now();

        // Check file size
        let metadata = std::fs::metadata(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        if metadata.len() > self.config.max_file_size {
            return Err(BinaryExportError::ValidationFailed(
                format!("File size {} exceeds maximum {}", metadata.len(), self.config.max_file_size)
            ));
        }

        // Read file data
        let file_data = std::fs::read(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;

        self.parse_data(&file_data, Some(path))
    }
}   
 /// Parse binary data from bytes
    pub fn parse_data(&self, data: &[u8], source_path: Option<&Path>) -> Result<ParseResult, BinaryExportError> {
        let start_time = std::time::Instant::now();
        let file_size = data.len() as u64;

        // Detect format
        let format_info = if self.config.auto_detect_format {
            self.detect_format(data)?
        } else {
            DetectedFormat {
                format: OutputFormat::MessagePack,
                version: BINARY_FORMAT_VERSION,
                compression: None,
                confidence: 0.5,
            }
        };

        // Handle version migration if needed
        let migrated_data = if self.config.enable_version_migration && format_info.version < BINARY_FORMAT_VERSION {
            self.version_handler.migrate_version(data, format_info.version)?
        } else {
            data.to_vec()
        };

        // Decompress if needed
        let decompressed_data = if let Some(ref compression_info) = format_info.compression {
            if compression_info.is_compressed {
                self.decompress_data(&migrated_data, &compression_info.algorithm)?
            } else {
                migrated_data
            }
        } else {
            migrated_data
        };

        // Parse the data based on detected format
        let unified_data = self.parse_format_data(&decompressed_data, format_info.format)?;

        // Validate parsed data if enabled
        let validation_results = if self.config.validate_data {
            self.validate_parsed_data(&unified_data)?
        } else {
            ValidationResults::default()
        };

        let parse_time = start_time.elapsed();
        let throughput = file_size as f64 / parse_time.as_secs_f64();

        let parse_stats = ParseStats {
            parse_time,
            file_size,
            throughput,
            peak_memory_usage: decompressed_data.len(), // Simplified
            structures_parsed: unified_data.allocations.allocations.len() as u32,
        };

        Ok(ParseResult {
            data: unified_data,
            format_info,
            parse_stats,
            validation_results,
        })
    }

    /// Parse data in streaming mode for large files
    pub fn parse_streaming<R: Read + Seek>(&self, mut reader: R) -> Result<ParseResult, BinaryExportError> {
        let start_time = std::time::Instant::now();

        // Read header to detect format
        let mut header_buffer = vec![0u8; 1024];
        let header_size = reader.read(&mut header_buffer)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        header_buffer.truncate(header_size);

        let format_info = self.detect_format(&header_buffer)?;

        // Reset reader to beginning
        reader.seek(SeekFrom::Start(0))
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;

        // Stream parse based on format
        let unified_data = match format_info.format {
            OutputFormat::Chunked { chunk_size } => {
                self.parse_chunked_streaming(reader, chunk_size)?
            }
            _ => {
                // For non-chunked formats, read all data and parse normally
                let mut data = Vec::new();
                reader.read_to_end(&mut data)
                    .map_err(|e| BinaryExportError::IoError(e.kind()))?;
                
                let result = self.parse_data(&data, None)?;
                return Ok(result);
            }
        };

        let parse_time = start_time.elapsed();
        let file_size = reader.seek(SeekFrom::End(0))
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;

        let parse_stats = ParseStats {
            parse_time,
            file_size,
            throughput: file_size as f64 / parse_time.as_secs_f64(),
            peak_memory_usage: self.config.chunk_size,
            structures_parsed: unified_data.allocations.allocations.len() as u32,
        };

        Ok(ParseResult {
            data: unified_data,
            format_info,
            parse_stats,
            validation_results: ValidationResults::default(),
        })
    }

    /// Detect format from data
    fn detect_format(&self, data: &[u8]) -> Result<DetectedFormat, BinaryExportError> {
        let detection_result = self.format_detector.detect_format(data)?;
        
        // Try to detect version
        let version = self.detect_version(data)?;
        
        // Try to detect compression
        let compression = self.detect_compression(data)?;

        Ok(DetectedFormat {
            format: detection_result.format,
            version,
            compression,
            confidence: detection_result.confidence,
        })
    }

    /// Detect version from data
    fn detect_version(&self, data: &[u8]) -> Result<u32, BinaryExportError> {
        if data.len() < 10 {
            return Ok(1); // Default to version 1 for small files
        }

        // Look for version information in different locations based on format
        if data.starts_with(b"MEMBIN") {
            // Custom binary format - version at offset 6
            if data.len() >= 10 {
                let version_bytes = [data[6], data[7], data[8], data[9]];
                return Ok(u32::from_le_bytes(version_bytes));
            }
        }

        // Try to find version in MessagePack data
        if let Ok(parsed) = rmp_serde::from_slice::<serde_json::Value>(data) {
            if let Some(metadata) = parsed.get("metadata") {
                if let Some(version) = metadata.get("format_version") {
                    if let Some(version_num) = version.as_u64() {
                        return Ok(version_num as u32);
                    }
                }
            }
        }

        Ok(1) // Default version
    }

    /// Detect compression from data
    fn detect_compression(&self, data: &[u8]) -> Result<Option<CompressionInfo>, BinaryExportError> {
        if data.len() < 4 {
            return Ok(None);
        }

        // Check for zstd magic number
        if &data[0..4] == &[0x28, 0xb5, 0x2f, 0xfd] {
            return Ok(Some(CompressionInfo {
                is_compressed: true,
                algorithm: Some("zstd".to_string()),
                compression_ratio: None,
                original_size: None,
            }));
        }

        // Check for gzip magic number
        if &data[0..2] == &[0x1f, 0x8b] {
            return Ok(Some(CompressionInfo {
                is_compressed: true,
                algorithm: Some("gzip".to_string()),
                compression_ratio: None,
                original_size: None,
            }));
        }

        Ok(None)
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8], algorithm: &Option<String>) -> Result<Vec<u8>, BinaryExportError> {
        match algorithm.as_ref().map(|s| s.as_str()) {
            Some("zstd") => {
                zstd::bulk::decompress(data, 64 * 1024 * 1024) // 64MB limit
                    .map_err(|e| BinaryExportError::CompressionError(e.to_string()))
            }
            Some("gzip") => {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut result = Vec::new();
                decoder.read_to_end(&mut result)
                    .map_err(|e| BinaryExportError::CompressionError(e.to_string()))?;
                Ok(result)
            }
            _ => Ok(data.to_vec()),
        }
    }

    /// Parse data based on format
    fn parse_format_data(&self, data: &[u8], format: OutputFormat) -> Result<UnifiedData, BinaryExportError> {
        match format {
            OutputFormat::MessagePack | OutputFormat::CompressedMessagePack { .. } => {
                rmp_serde::from_slice(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            OutputFormat::CustomBinary => {
                self.parse_custom_binary(data)
            }
            OutputFormat::Raw => {
                bincode::deserialize(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            OutputFormat::Chunked { .. } => {
                self.parse_chunked_format(data)
            }
        }
    }

    /// Parse custom binary format
    fn parse_custom_binary(&self, data: &[u8]) -> Result<UnifiedData, BinaryExportError> {
        if data.len() < 16 {
            return Err(BinaryExportError::InvalidFormat("Data too short for custom binary format".to_string()));
        }

        // Skip magic bytes and version (first 10 bytes)
        let mut offset = 10;

        // Read data length
        if data.len() < offset + 4 {
            return Err(BinaryExportError::InvalidFormat("Missing data length field".to_string()));
        }

        let data_length = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]) as usize;
        offset += 4;

        // Read actual data
        if data.len() < offset + data_length {
            return Err(BinaryExportError::InvalidFormat("Data length mismatch".to_string()));
        }

        let payload = &data[offset..offset + data_length];
        
        // Deserialize payload
        bincode::deserialize(payload)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }

    /// Parse chunked format
    fn parse_chunked_format(&self, data: &[u8]) -> Result<UnifiedData, BinaryExportError> {
        if data.len() < 8 {
            return Err(BinaryExportError::InvalidFormat("Data too short for chunked format".to_string()));
        }

        let mut offset = 4; // Skip magic bytes

        // Read chunk count
        let chunk_count = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]);
        offset += 4;

        let mut all_data = Vec::new();

        // Read each chunk
        for _ in 0..chunk_count {
            if offset + 16 > data.len() {
                return Err(BinaryExportError::InvalidFormat("Incomplete chunk header".to_string()));
            }

            // Read chunk header
            let chunk_header: ChunkHeader = bincode::deserialize(&data[offset..offset + 16])
                .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
            offset += 16;

            // Read chunk data
            if offset + chunk_header.size as usize > data.len() {
                return Err(BinaryExportError::InvalidFormat("Incomplete chunk data".to_string()));
            }

            let chunk_data = &data[offset..offset + chunk_header.size as usize];
            offset += chunk_header.size as usize;

            // Verify checksum
            let calculated_checksum = crc32fast::hash(chunk_data);
            if calculated_checksum != chunk_header.checksum {
                return Err(BinaryExportError::ChecksumMismatch {
                    expected: format!("{:08x}", chunk_header.checksum),
                    actual: format!("{:08x}", calculated_checksum),
                });
            }

            all_data.extend_from_slice(chunk_data);
        }

        // Deserialize combined data
        bincode::deserialize(&all_data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }

    /// Parse chunked format in streaming mode
    fn parse_chunked_streaming<R: Read>(&self, mut reader: R, _chunk_size: usize) -> Result<UnifiedData, BinaryExportError> {
        let mut buffer = vec![0u8; 4];
        
        // Read magic bytes
        reader.read_exact(&mut buffer)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        if &buffer != b"CHUNK" {
            return Err(BinaryExportError::InvalidFormat("Invalid chunked format magic".to_string()));
        }

        // Read chunk count
        reader.read_exact(&mut buffer)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        let chunk_count = u32::from_le_bytes(buffer);

        let mut all_data = Vec::new();

        // Process each chunk
        for _ in 0..chunk_count {
            // Read chunk header
            let mut header_buffer = vec![0u8; 16];
            reader.read_exact(&mut header_buffer)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;

            let chunk_header: ChunkHeader = bincode::deserialize(&header_buffer)
                .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

            // Read chunk data
            let mut chunk_data = vec![0u8; chunk_header.size as usize];
            reader.read_exact(&mut chunk_data)
                .map_err(|e| BinaryExportError::IoError(e.kind()))?;

            // Verify checksum
            let calculated_checksum = crc32fast::hash(&chunk_data);
            if calculated_checksum != chunk_header.checksum {
                return Err(BinaryExportError::ChecksumMismatch {
                    expected: format!("{:08x}", chunk_header.checksum),
                    actual: format!("{:08x}", calculated_checksum),
                });
            }

            all_data.extend_from_slice(&chunk_data);
        }

        // Deserialize combined data
        bincode::deserialize(&all_data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
    }

    /// Validate parsed data
    fn validate_parsed_data(&self, data: &UnifiedData) -> Result<ValidationResults, BinaryExportError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut integrity_score = 1.0;

        // Basic validation
        if data.allocations.allocations.is_empty() {
            warnings.push("No allocation data found".to_string());
            integrity_score -= 0.1;
        }

        // Check for data consistency
        let allocation_count = data.allocations.allocations.len();
        let call_stack_count = data.allocations.call_stacks.len();

        if call_stack_count == 0 && allocation_count > 0 {
            warnings.push("No call stack data found despite having allocations".to_string());
            integrity_score -= 0.1;
        }

        // Check for duplicate allocation IDs
        let mut seen_ids = std::collections::HashSet::new();
        for allocation in &data.allocations.allocations {
            if !seen_ids.insert(allocation.id) {
                errors.push(format!("Duplicate allocation ID: {}", allocation.id));
                integrity_score -= 0.2;
            }
        }

        integrity_score = integrity_score.max(0.0);

        Ok(ValidationResults {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            integrity_score,
        })
    }

    /// Convert between formats
    pub fn convert_format(&self, data: &[u8], source: OutputFormat, target: OutputFormat) -> Result<Vec<u8>, BinaryExportError> {
        if source == target {
            return Ok(data.to_vec());
        }

        self.data_converter.convert(data, source, target)
    }

    /// Get supported input formats
    pub fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary,
            OutputFormat::CompressedMessagePack { level: 6 },
            OutputFormat::Chunked { chunk_size: 256 * 1024 },
            OutputFormat::Raw,
        ]
    }

impl VersionHandler {
    fn new() -> Self {
        let mut migrations = HashMap::new();
        
        // Add version migrations
        migrations.insert(1, Box::new(V1ToV2Migration) as Box<dyn VersionMigration>);
        
        Self { migrations }
    }

    fn migrate_version(&self, data: &[u8], from_version: u32) -> Result<Vec<u8>, BinaryExportError> {
        let mut current_data = data.to_vec();
        let mut current_version = from_version;

        while current_version < BINARY_FORMAT_VERSION {
            if let Some(migration) = self.migrations.get(&current_version) {
                current_data = migration.migrate(&current_data)?;
                current_version = migration.target_version();
            } else {
                return Err(BinaryExportError::UnsupportedVersion {
                    found: current_version,
                    supported: BINARY_FORMAT_VERSION,
                });
            }
        }

        Ok(current_data)
    }
}

/// Migration from version 1 to version 2
struct V1ToV2Migration;

impl VersionMigration for V1ToV2Migration {
    fn migrate(&self, data: &[u8]) -> Result<Vec<u8>, BinaryExportError> {
        // Placeholder migration - would implement actual data transformation
        // For now, just return the data as-is
        Ok(data.to_vec())
    }

    fn source_version(&self) -> u32 { 1 }
    fn target_version(&self) -> u32 { 2 }
}

impl DataConverter {
    fn new() -> Self {
        let mut converters = HashMap::new();
        
        // Add format converters
        converters.insert(
            (OutputFormat::MessagePack, OutputFormat::CustomBinary),
            Box::new(MessagePackToCustomBinaryConverter) as Box<dyn FormatConverter>
        );
        
        Self { converters }
    }

    fn convert(&self, data: &[u8], source: OutputFormat, target: OutputFormat) -> Result<Vec<u8>, BinaryExportError> {
        if let Some(converter) = self.converters.get(&(source, target)) {
            converter.convert(data, source, target)
        } else {
            // Generic conversion through UnifiedData
            self.generic_convert(data, source, target)
        }
    }

    fn generic_convert(&self, data: &[u8], source: OutputFormat, target: OutputFormat) -> Result<Vec<u8>, BinaryExportError> {
        // Parse with source format
        let unified_data: UnifiedData = match source {
            OutputFormat::MessagePack => {
                rmp_serde::from_slice(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?
            }
            OutputFormat::CustomBinary => {
                bincode::deserialize(data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?
            }
            _ => {
                return Err(BinaryExportError::UnsupportedFeature(
                    format!("Generic conversion from {:?} not supported", source)
                ));
            }
        };

        // Serialize with target format
        match target {
            OutputFormat::MessagePack => {
                rmp_serde::to_vec(&unified_data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            OutputFormat::CustomBinary => {
                bincode::serialize(&unified_data)
                    .map_err(|e| BinaryExportError::SerializationError(e.to_string()))
            }
            _ => {
                Err(BinaryExportError::UnsupportedFeature(
                    format!("Generic conversion to {:?} not supported", target)
                ))
            }
        }
    }
}

/// Converter from MessagePack to Custom Binary
struct MessagePackToCustomBinaryConverter;

impl FormatConverter for MessagePackToCustomBinaryConverter {
    fn convert(&self, data: &[u8], _source: OutputFormat, _target: OutputFormat) -> Result<Vec<u8>, BinaryExportError> {
        // Parse MessagePack
        let unified_data: UnifiedData = rmp_serde::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        // Serialize to custom binary format
        let mut result = Vec::new();
        
        // Add magic bytes
        result.extend_from_slice(b"MEMBIN");
        
        // Add version
        result.extend_from_slice(&BINARY_FORMAT_VERSION.to_le_bytes());
        
        // Serialize data
        let serialized = bincode::serialize(&unified_data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;
        
        // Add data length
        result.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
        
        // Add data
        result.extend_from_slice(&serialized);
        
        // Add checksum
        let checksum = crc32fast::hash(&serialized);
        result.extend_from_slice(&checksum.to_le_bytes());
        
        Ok(result)
    }
}

/// Chunk header structure (reused from format.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunkHeader {
    index: u32,
    size: u32,
    checksum: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_creation() {
        let config = ParserConfig::default();
        let parser = BinaryDataParser::new(config);
        
        let supported = parser.supported_formats();
        assert!(!supported.is_empty());
        assert!(supported.contains(&OutputFormat::MessagePack));
    }

    #[test]
    fn test_format_detection() {
        let config = ParserConfig::default();
        let parser = BinaryDataParser::new(config);
        
        // Test MessagePack detection
        let msgpack_data = [0x82, 0xa7, 0x74, 0x65, 0x73, 0x74];
        let result = parser.detect_format(&msgpack_data);
        assert!(result.is_ok());
        
        let format_info = result.unwrap();
        assert_eq!(format_info.format, OutputFormat::MessagePack);
    }

    #[test]
    fn test_version_detection() {
        let config = ParserConfig::default();
        let parser = BinaryDataParser::new(config);
        
        // Test custom binary format version detection
        let mut test_data = Vec::new();
        test_data.extend_from_slice(b"MEMBIN");
        test_data.extend_from_slice(&2u32.to_le_bytes());
        test_data.extend_from_slice(b"test data");
        
        let version = parser.detect_version(&test_data).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_compression_detection() {
        let config = ParserConfig::default();
        let parser = BinaryDataParser::new(config);
        
        // Test zstd detection
        let zstd_data = [0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x01, 0x02, 0x03];
        let result = parser.detect_compression(&zstd_data).unwrap();
        
        assert!(result.is_some());
        let compression_info = result.unwrap();
        assert!(compression_info.is_compressed);
        assert_eq!(compression_info.algorithm, Some("zstd".to_string()));
    }

    #[test]
    fn test_format_conversion() {
        let config = ParserConfig::default();
        let parser = BinaryDataParser::new(config);
        
        // Create test data
        let test_data = UnifiedData::new();
        let msgpack_data = rmp_serde::to_vec(&test_data).unwrap();
        
        // Convert to custom binary
        let result = parser.convert_format(
            &msgpack_data,
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary
        );
        
        assert!(result.is_ok());
        let converted = result.unwrap();
        assert!(converted.starts_with(b"MEMBIN"));
    }
}