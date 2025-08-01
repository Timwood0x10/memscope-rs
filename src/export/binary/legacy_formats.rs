//! Legacy format support for existing analysis data formats
//!
//! This module provides support for reading and converting existing JSON
//! analysis data formats into the unified binary export format.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::*;

/// Legacy format adapter for existing analysis data formats
pub struct LegacyFormatAdapter {
    /// Format parsers for different legacy formats
    parsers: HashMap<LegacyFormatType, Box<dyn LegacyFormatParser>>,
}

/// Types of legacy formats supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LegacyFormatType {
    /// Memory analysis JSON format
    MemoryAnalysis,
    /// Lifetime analysis JSON format
    Lifetime,
    /// Performance analysis JSON format
    Performance,
    /// Security violations JSON format
    SecurityViolations,
    /// Unsafe FFI tracking JSON format
    UnsafeFFI,
    /// Complex types analysis JSON format
    ComplexTypes,
}

/// Legacy format parser trait
trait LegacyFormatParser: Send + Sync {
    /// Parse legacy format data into unified format
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError>;
    
    /// Get format type
    fn format_type(&self) -> LegacyFormatType;
    
    /// Detect if data matches this format
    fn can_parse(&self, data: &[u8]) -> bool;
}

/// Parsed legacy format data
#[derive(Debug, Clone)]
pub struct LegacyFormatData {
    /// Format type
    pub format_type: LegacyFormatType,
    /// Parsed allocations (if applicable)
    pub allocations: Vec<LegacyAllocation>,
    /// Performance metrics (if applicable)
    pub performance_metrics: Option<LegacyPerformanceMetrics>,
    /// Security violations (if applicable)
    pub security_violations: Vec<LegacySecurityViolation>,
    /// FFI data (if applicable)
    pub ffi_data: Vec<LegacyFFICall>,
    /// Complex type information (if applicable)
    pub complex_types: Vec<LegacyComplexType>,
    /// Metadata
    pub metadata: HashMap<String, Value>,
}

/// Legacy allocation format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyAllocation {
    /// Memory pointer address
    pub ptr: String,
    /// Scope name (optional)
    pub scope_name: Option<String>,
    /// Allocation size
    pub size: u64,
    /// Allocation timestamp
    pub timestamp_alloc: u64,
    /// Deallocation timestamp (optional)
    pub timestamp_dealloc: Option<u64>,
    /// Type name
    pub type_name: String,
    /// Variable name
    pub var_name: String,
}

/// Legacy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPerformanceMetrics {
    /// Allocation distribution
    pub allocation_distribution: HashMap<String, u32>,
    /// Export performance data
    pub export_performance: LegacyExportPerformance,
    /// Memory performance data
    pub memory_performance: LegacyMemoryPerformance,
    /// Optimization status
    pub optimization_status: HashMap<String, Value>,
}

/// Legacy export performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyExportPerformance {
    /// Number of allocations processed
    pub allocations_processed: u32,
    /// Processing rate information
    pub processing_rate: LegacyProcessingRate,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

/// Legacy processing rate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyProcessingRate {
    /// Allocations per second
    pub allocations_per_second: f64,
    /// Performance class
    pub performance_class: String,
}

/// Legacy memory performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMemoryPerformance {
    /// Active memory usage
    pub active_memory: u64,
    /// Memory efficiency percentage
    pub memory_efficiency: u32,
    /// Peak memory usage
    pub peak_memory: u64,
    /// Total allocated memory
    pub total_allocated: u64,
}

/// Legacy security violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySecurityViolation {
    /// Violation type
    pub violation_type: String,
    /// Description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Location information
    pub location: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

/// Legacy FFI call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyFFICall {
    /// Function name
    pub function_name: String,
    /// Call timestamp
    pub timestamp: u64,
    /// Parameters (if available)
    pub parameters: Option<Vec<String>>,
    /// Return value (if available)
    pub return_value: Option<String>,
    /// Safety level
    pub safety_level: String,
}

/// Legacy complex type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyComplexType {
    /// Type name
    pub type_name: String,
    /// Type category
    pub category: String,
    /// Size information
    pub size: u64,
    /// Complexity score
    pub complexity_score: f64,
    /// Usage count
    pub usage_count: u32,
}

impl LegacyFormatAdapter {
    /// Create a new legacy format adapter
    pub fn new() -> Self {
        let mut parsers: HashMap<LegacyFormatType, Box<dyn LegacyFormatParser>> = HashMap::new();
        
        parsers.insert(LegacyFormatType::MemoryAnalysis, Box::new(MemoryAnalysisParser));
        parsers.insert(LegacyFormatType::Lifetime, Box::new(LifetimeParser));
        parsers.insert(LegacyFormatType::Performance, Box::new(PerformanceParser));
        parsers.insert(LegacyFormatType::SecurityViolations, Box::new(SecurityViolationsParser));
        parsers.insert(LegacyFormatType::UnsafeFFI, Box::new(UnsafeFFIParser));
        parsers.insert(LegacyFormatType::ComplexTypes, Box::new(ComplexTypesParser));
        
        Self { parsers }
    }

    /// Parse legacy format file
    pub fn parse_legacy_file<P: AsRef<Path>>(&self, path: P) -> Result<LegacyFormatData, BinaryExportError> {
        let path = path.as_ref();
        let data = std::fs::read(path)
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        self.parse_legacy_data(&data, Some(path))
    }

    /// Parse legacy format data
    pub fn parse_legacy_data(&self, data: &[u8], source_path: Option<&Path>) -> Result<LegacyFormatData, BinaryExportError> {
        // Try to detect format from filename if available
        if let Some(path) = source_path {
            if let Some(format_type) = self.detect_format_from_filename(path) {
                if let Some(parser) = self.parsers.get(&format_type) {
                    return parser.parse(data);
                }
            }
        }

        // Try each parser to see which one can handle the data
        for parser in self.parsers.values() {
            if parser.can_parse(data) {
                return parser.parse(data);
            }
        }

        Err(BinaryExportError::UnsupportedFeature(
            "Unable to detect legacy format type".to_string()
        ))
    }

    /// Convert legacy format data to unified format
    pub fn convert_to_unified(&self, legacy_data: LegacyFormatData) -> Result<UnifiedData, BinaryExportError> {
        let mut unified_data = UnifiedData::new();

        // Convert allocations
        for legacy_alloc in legacy_data.allocations {
            let allocation_record = AllocationRecord {
                id: self.parse_address(&legacy_alloc.ptr)?,
                address: self.parse_address(&legacy_alloc.ptr)?,
                size: legacy_alloc.size as usize,
                timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_nanos(legacy_alloc.timestamp_alloc),
                call_stack_id: None, // Would need to be derived from other data
                thread_id: 1, // Default thread ID
                allocation_type: legacy_alloc.type_name,
            };
            
            unified_data.allocations.allocations.push(allocation_record);
        }

        // Convert performance metrics if available
        if let Some(perf_metrics) = legacy_data.performance_metrics {
            // Convert to unified performance format
            // This would involve mapping legacy performance data to the new format
        }

        // Convert security violations
        for violation in legacy_data.security_violations {
            // Convert to unified security violation format
        }

        // Convert FFI data
        for ffi_call in legacy_data.ffi_data {
            // Convert to unified FFI format
        }

        // Convert complex types
        for complex_type in legacy_data.complex_types {
            // Convert to unified complex type format
        }

        // Add metadata
        unified_data.metadata.export_metadata.format_version = BINARY_FORMAT_VERSION;
        unified_data.metadata.export_metadata.timestamp = std::time::SystemTime::now();

        Ok(unified_data)
    }

    /// Detect format from filename
    fn detect_format_from_filename(&self, path: &Path) -> Option<LegacyFormatType> {
        let filename = path.file_name()?.to_str()?;
        
        if filename.contains("memory_analysis") {
            Some(LegacyFormatType::MemoryAnalysis)
        } else if filename.contains("lifetime") {
            Some(LegacyFormatType::Lifetime)
        } else if filename.contains("performance") {
            Some(LegacyFormatType::Performance)
        } else if filename.contains("security_violations") {
            Some(LegacyFormatType::SecurityViolations)
        } else if filename.contains("unsafe_ffi") {
            Some(LegacyFormatType::UnsafeFFI)
        } else if filename.contains("complex_types") {
            Some(LegacyFormatType::ComplexTypes)
        } else {
            None
        }
    }

    /// Parse address string to u64
    fn parse_address(&self, addr_str: &str) -> Result<u64, BinaryExportError> {
        if addr_str.starts_with("0x") {
            u64::from_str_radix(&addr_str[2..], 16)
                .map_err(|e| BinaryExportError::InvalidFormat(format!("Invalid address format: {}", e)))
        } else {
            addr_str.parse::<u64>()
                .map_err(|e| BinaryExportError::InvalidFormat(format!("Invalid address format: {}", e)))
        }
    }

    /// Get supported legacy formats
    pub fn supported_formats(&self) -> Vec<LegacyFormatType> {
        self.parsers.keys().cloned().collect()
    }
}

// Parser implementations

/// Memory analysis format parser
struct MemoryAnalysisParser;

impl LegacyFormatParser for MemoryAnalysisParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::MemoryAnalysis,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse allocations array
        if let Some(allocations) = json_data.get("allocations").and_then(|v| v.as_array()) {
            for alloc_value in allocations {
                if let Ok(allocation) = serde_json::from_value::<LegacyAllocation>(alloc_value.clone()) {
                    legacy_data.allocations.push(allocation);
                }
            }
        }

        // Extract metadata
        if let Some(metadata) = json_data.get("metadata") {
            if let Ok(meta_map) = serde_json::from_value::<HashMap<String, Value>>(metadata.clone()) {
                legacy_data.metadata = meta_map;
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::MemoryAnalysis
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            json_data.get("allocations").is_some()
        } else {
            false
        }
    }
}

/// Performance format parser
struct PerformanceParser;

impl LegacyFormatParser for PerformanceParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::Performance,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse performance metrics
        if let Ok(perf_metrics) = serde_json::from_value::<LegacyPerformanceMetrics>(json_data.clone()) {
            legacy_data.performance_metrics = Some(perf_metrics);
        }

        // Extract metadata
        if let Some(metadata) = json_data.get("metadata") {
            if let Ok(meta_map) = serde_json::from_value::<HashMap<String, Value>>(metadata.clone()) {
                legacy_data.metadata = meta_map;
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::Performance
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            json_data.get("export_performance").is_some() || 
            json_data.get("memory_performance").is_some()
        } else {
            false
        }
    }
}

/// Lifetime format parser
struct LifetimeParser;

impl LegacyFormatParser for LifetimeParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        // Similar to MemoryAnalysisParser but focused on lifetime data
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::Lifetime,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse lifetime-specific data
        if let Some(allocations) = json_data.get("allocations").and_then(|v| v.as_array()) {
            for alloc_value in allocations {
                if let Ok(allocation) = serde_json::from_value::<LegacyAllocation>(alloc_value.clone()) {
                    legacy_data.allocations.push(allocation);
                }
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::Lifetime
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            // Check for lifetime-specific indicators
            json_data.get("lifetime_analysis").is_some() ||
            (json_data.get("allocations").is_some() && 
             json_data.get("allocations").and_then(|v| v.as_array())
                .map(|arr| arr.iter().any(|item| 
                    item.get("timestamp_dealloc").is_some()))
                .unwrap_or(false))
        } else {
            false
        }
    }
}

/// Security violations format parser
struct SecurityViolationsParser;

impl LegacyFormatParser for SecurityViolationsParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::SecurityViolations,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse security violations
        if let Some(violations) = json_data.get("violations").and_then(|v| v.as_array()) {
            for violation_value in violations {
                if let Ok(violation) = serde_json::from_value::<LegacySecurityViolation>(violation_value.clone()) {
                    legacy_data.security_violations.push(violation);
                }
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::SecurityViolations
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            json_data.get("violations").is_some() ||
            json_data.get("security_violations").is_some()
        } else {
            false
        }
    }
}

/// Unsafe FFI format parser
struct UnsafeFFIParser;

impl LegacyFormatParser for UnsafeFFIParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::UnsafeFFI,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse FFI calls
        if let Some(ffi_calls) = json_data.get("ffi_calls").and_then(|v| v.as_array()) {
            for ffi_value in ffi_calls {
                if let Ok(ffi_call) = serde_json::from_value::<LegacyFFICall>(ffi_value.clone()) {
                    legacy_data.ffi_data.push(ffi_call);
                }
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::UnsafeFFI
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            json_data.get("ffi_calls").is_some() ||
            json_data.get("unsafe_operations").is_some()
        } else {
            false
        }
    }
}

/// Complex types format parser
struct ComplexTypesParser;

impl LegacyFormatParser for ComplexTypesParser {
    fn parse(&self, data: &[u8]) -> Result<LegacyFormatData, BinaryExportError> {
        let json_data: Value = serde_json::from_slice(data)
            .map_err(|e| BinaryExportError::SerializationError(e.to_string()))?;

        let mut legacy_data = LegacyFormatData {
            format_type: LegacyFormatType::ComplexTypes,
            allocations: Vec::new(),
            performance_metrics: None,
            security_violations: Vec::new(),
            ffi_data: Vec::new(),
            complex_types: Vec::new(),
            metadata: HashMap::new(),
        };

        // Parse complex types
        if let Some(types) = json_data.get("complex_types").and_then(|v| v.as_array()) {
            for type_value in types {
                if let Ok(complex_type) = serde_json::from_value::<LegacyComplexType>(type_value.clone()) {
                    legacy_data.complex_types.push(complex_type);
                }
            }
        }

        Ok(legacy_data)
    }

    fn format_type(&self) -> LegacyFormatType {
        LegacyFormatType::ComplexTypes
    }

    fn can_parse(&self, data: &[u8]) -> bool {
        if let Ok(json_data) = serde_json::from_slice::<Value>(data) {
            json_data.get("complex_types").is_some() ||
            json_data.get("type_analysis").is_some()
        } else {
            false
        }
    }
}

/// Batch convert multiple legacy files to unified binary format
pub fn convert_legacy_directory<P: AsRef<Path>>(
    input_dir: P,
    output_path: P,
) -> Result<IntegratedExportResult, BinaryExportError> {
    let input_dir = input_dir.as_ref();
    let adapter = LegacyFormatAdapter::new();
    let mut combined_data = UnifiedData::new();

    // Find all JSON files in the directory
    let entries = std::fs::read_dir(input_dir)
        .map_err(|e| BinaryExportError::IoError(e.kind()))?;

    for entry in entries {
        let entry = entry.map_err(|e| BinaryExportError::IoError(e.kind()))?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match adapter.parse_legacy_file(&path) {
                Ok(legacy_data) => {
                    let unified_data = adapter.convert_to_unified(legacy_data)?;
                    // Merge data into combined_data
                    combined_data.allocations.allocations.extend(unified_data.allocations.allocations);
                    // Merge other data types as needed
                }
                Err(e) => {
                    println!("Warning: Failed to parse {}: {:?}", path.display(), e);
                }
            }
        }
    }

    // Export combined data using integrated exporter
    let config = IntegratedConfig::balanced();
    let mut exporter = IntegratedBinaryExporter::new(config);
    
    // Create a mock tracker with the combined data
    let tracker = crate::core::tracker::MemoryTracker::new();
    // In a real implementation, we would populate the tracker with the combined data
    
    exporter.export(&tracker, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_legacy_format_adapter_creation() {
        let adapter = LegacyFormatAdapter::new();
        let supported = adapter.supported_formats();
        
        assert!(supported.contains(&LegacyFormatType::MemoryAnalysis));
        assert!(supported.contains(&LegacyFormatType::Performance));
        assert!(supported.contains(&LegacyFormatType::SecurityViolations));
    }

    #[test]
    fn test_memory_analysis_parser() {
        let parser = MemoryAnalysisParser;
        
        let test_data = r#"{
            "allocations": [
                {
                    "ptr": "0x1520046a0",
                    "scope_name": null,
                    "size": 15,
                    "timestamp_alloc": 1753865197333169000,
                    "timestamp_dealloc": null,
                    "type_name": "String",
                    "var_name": "test_var"
                }
            ]
        }"#;
        
        assert!(parser.can_parse(test_data.as_bytes()));
        
        let result = parser.parse(test_data.as_bytes());
        assert!(result.is_ok());
        
        let legacy_data = result.unwrap();
        assert_eq!(legacy_data.format_type, LegacyFormatType::MemoryAnalysis);
        assert_eq!(legacy_data.allocations.len(), 1);
        assert_eq!(legacy_data.allocations[0].var_name, "test_var");
    }

    #[test]
    fn test_performance_parser() {
        let parser = PerformanceParser;
        
        let test_data = r#"{
            "export_performance": {
                "allocations_processed": 30,
                "processing_rate": {
                    "allocations_per_second": 30000.0,
                    "performance_class": "excellent"
                },
                "total_processing_time_ms": 4
            },
            "memory_performance": {
                "active_memory": 3660964,
                "memory_efficiency": 100,
                "peak_memory": 3660964,
                "total_allocated": 3735544
            }
        }"#;
        
        assert!(parser.can_parse(test_data.as_bytes()));
        
        let result = parser.parse(test_data.as_bytes());
        assert!(result.is_ok());
        
        let legacy_data = result.unwrap();
        assert_eq!(legacy_data.format_type, LegacyFormatType::Performance);
        assert!(legacy_data.performance_metrics.is_some());
    }

    #[test]
    fn test_format_detection_from_filename() {
        let adapter = LegacyFormatAdapter::new();
        
        assert_eq!(
            adapter.detect_format_from_filename(Path::new("test_memory_analysis.json")),
            Some(LegacyFormatType::MemoryAnalysis)
        );
        
        assert_eq!(
            adapter.detect_format_from_filename(Path::new("test_performance.json")),
            Some(LegacyFormatType::Performance)
        );
        
        assert_eq!(
            adapter.detect_format_from_filename(Path::new("test_security_violations.json")),
            Some(LegacyFormatType::SecurityViolations)
        );
    }

    #[test]
    fn test_address_parsing() {
        let adapter = LegacyFormatAdapter::new();
        
        assert_eq!(adapter.parse_address("0x1520046a0").unwrap(), 0x1520046a0);
        assert_eq!(adapter.parse_address("123456").unwrap(), 123456);
        
        assert!(adapter.parse_address("invalid").is_err());
    }
}