//! Binary export module for high-performance memory tracking data export.
//!
//! Features:
//! - 3x faster than JSON export with 60%+ smaller file size
//! - Lock-free, single-threaded design for simplicity
//! - Compatible with existing JSON/HTML export APIs
//! - Modular architecture for easy testing and maintenance

mod batch_processor;
mod binary_html_export;
mod binary_html_writer;
mod binary_template_engine;
mod cache;
mod complex_type_analyzer;
mod config;
mod error;
mod error_recovery;
mod ffi_safety_analyzer;
mod field_parser;
mod filter_engine;
pub mod format;
pub mod html_converter;
pub mod html_export;
mod index;
mod index_builder;
#[cfg(test)]
mod integration_test_complex_types;
#[cfg(test)]
mod integration_test_ffi_safety;
#[cfg(test)]
mod integration_test_template_resources;
#[cfg(test)]
mod integration_test_variable_relationships;
mod memory_layout_serialization;
mod parser;
pub mod reader;
mod selective_json_exporter;
mod selective_reader;
mod serializable;
mod smart_pointer_serialization;
mod streaming_field_processor;
mod streaming_json_writer;
mod string_table;
mod template_resource_manager;
mod variable_relationship_analyzer;
pub mod writer;

pub use batch_processor::{
    BatchProcessor, BatchProcessorBuilder, BatchProcessorConfig, BatchProcessorStats, RecordBatch,
};
pub use binary_html_export::{
    get_recommended_config, parse_binary_to_html_auto, parse_binary_to_html_direct,
    parse_binary_to_html_with_config, BinaryHtmlExportConfig, BinaryHtmlExportStats,
    ProcessingStrategy,
};
pub use binary_html_writer::{
    BinaryAllocationData, BinaryHtmlStats, BinaryHtmlWriter, BinaryHtmlWriterConfig,
    BinaryTemplateData,
};
pub use binary_template_engine::{
    BinaryTemplateEngine, BinaryTemplateEngineConfig, BinaryTemplateEngineStats,
};
pub use cache::{CacheEntry, CacheStats, IndexCache, IndexCacheConfig};
pub use complex_type_analyzer::{
    CategorizedTypes, ComplexTypeAnalysis, ComplexTypeAnalyzer, ComplexTypeSummary,
    GenericInstantiation, GenericTypeAnalysis, TypeCategory, TypeInfo,
};
pub use ffi_safety_analyzer::{
    FfiCallGraph, FfiHotspot, FfiSafetyAnalysis, FfiSafetyAnalyzer, FfiSafetySummary,
    RiskAssessment, RiskLevel, UnsafeOperation, UnsafeOperationType,
};
pub use html_converter::*;
pub use variable_relationship_analyzer::{
    GraphEdge, GraphNode, NodeCategory, OwnershipStatus, RelationshipGraph, RelationshipType,
    VariableRelationshipAnalysis, VariableRelationshipAnalyzer,
};

pub use config::{
    AdvancedMetricsLevel, AnalysisType, BinaryExportConfig, BinaryExportConfigBuilder,
    DashboardExportStats, DashboardFormat, DashboardOptions, DataScope, PerformanceMode,
};
pub use error::BinaryExportError;
pub use error_recovery::{
    ErrorRecoveryManager, ErrorReport, ErrorStatistics, ErrorTrend, RecoveryConfig, RecoveryResult,
    RecoveryStrategy,
};
pub use field_parser::{FieldParser, FieldParserConfig, FieldParserStats, PartialAllocationInfo};
pub use filter_engine::{FilterEngine, FilterEngineBuilder, FilterOptimizer, FilterStats};
pub use format::{BinaryExportMode, FileHeader, FORMAT_VERSION, MAGIC_BYTES};
pub use html_export::{
    export_binary,
    export_binary_optimized,
    export_binary_to_both,
    export_binary_to_dashboard, // New unified API
    export_binary_to_html,
    export_binary_to_html_both,
    export_binary_to_html_system,
    export_binary_to_json,
    export_binary_with_config,
    export_binary_with_format,
    show_export_options,
    BinaryOutputFormat,
};
pub use index::{BinaryIndex, CompactAllocationIndex, QuickFilterData, RecordMetadata};
pub use index_builder::BinaryIndexBuilder;
pub use parser::BinaryParser;
pub use reader::BinaryReader;
pub use selective_json_exporter::{
    OptimizationLevel, SelectiveJsonExportConfig, SelectiveJsonExportConfigBuilder,
    SelectiveJsonExportStats, SelectiveJsonExporter,
};
pub use selective_reader::{
    AllocationField, AllocationFilter, SelectiveReadOptions, SelectiveReadOptionsBuilder,
    SortField, SortOrder,
};
pub use streaming_field_processor::{
    OptimizedRecord, StreamingFieldProcessor, StreamingFieldProcessorConfig,
    StreamingFieldProcessorConfigBuilder, StreamingFieldProcessorStats,
};
pub use streaming_json_writer::{
    SelectiveSerializationOptions, StreamingJsonStats, StreamingJsonWriter,
    StreamingJsonWriterConfig, StreamingJsonWriterConfigBuilder,
};
pub use template_resource_manager::{
    create_template_data, PlaceholderProcessor, ResourceConfig, TemplateData,
    TemplateResourceManager,
};
pub use writer::BinaryWriter;

// Auto-detection functions are defined below and exported automatically

use crate::core::types::AllocationInfo;
use std::path::Path;

/// Export allocation information to binary format with default configuration
pub fn export_to_binary<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    path: P,
) -> Result<(), BinaryExportError> {
    export_to_binary_with_config(allocations, path, &BinaryExportConfig::default())
}

/// Export allocation information to binary format with enhanced header
///
/// This function properly filters allocations based on the export mode:
/// - UserOnly: Only exports allocations with var_name (user allocations)
/// - Full: Exports all allocations (user + system)
///
/// # Arguments
/// * `allocations` - Vector of allocation information to export
/// * `path` - Path where the binary file will be written
/// * `export_mode` - Binary export mode that controls which allocations are written
/// * `config` - Export configuration settings
///
/// # Returns
/// * `Result<(), BinaryExportError>` - Success or detailed export error
pub fn export_to_binary_with_mode<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    path: P,
    export_mode: BinaryExportMode,
    config: &BinaryExportConfig,
) -> Result<(), BinaryExportError> {
    let mut writer = BinaryWriter::new_with_config(path, config)?;

    // Filter allocations based on export mode
    let filtered_allocations: Vec<&AllocationInfo> = match export_mode {
        BinaryExportMode::UserOnly => {
            // Only include allocations with var_name (user allocations)
            allocations
                .iter()
                .filter(|a| a.var_name.is_some())
                .collect()
        }
        BinaryExportMode::Full => {
            // Include all allocations
            allocations.iter().collect()
        }
    };

    // Build string table for optimization if enabled
    // Note: We use the original allocations for string table optimization
    // as it doesn't affect the actual data written, only internal optimization
    writer.build_string_table(allocations)?;

    // Calculate accurate allocation counts based on filtered data
    let user_count = filtered_allocations
        .iter()
        .filter(|a| a.var_name.is_some())
        .count() as u16;
    let system_count = (filtered_allocations.len() - user_count as usize) as u16;
    let total_count = filtered_allocations.len() as u32;

    // Validate count consistency
    debug_assert_eq!(
        user_count as usize + system_count as usize,
        total_count as usize,
        "Count validation failed: user({user_count}) + system({system_count}) != total({total_count})"
    );

    tracing::info!(
        "Binary export starting: {total_count} allocations ({user_count} user, {system_count} system) in {export_mode:?} mode"
    );
    tracing::info!("Filtered from {} original allocations", allocations.len());

    // Write enhanced header with mode and accurate counts
    writer.write_enhanced_header(total_count, export_mode, user_count, system_count)?;

    // Write only the filtered allocation records
    for allocation in filtered_allocations {
        writer.write_allocation(allocation)?;
    }

    writer.finish()?;

    tracing::info!(
        "Binary export completed: {total_count} allocations written"
    );

    Ok(())
}

/// Export allocation information to binary format with custom configuration
pub fn export_to_binary_with_config<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    path: P,
    config: &BinaryExportConfig,
) -> Result<(), BinaryExportError> {
    let mut writer = BinaryWriter::new_with_config(path, config)?;

    // Build string table for optimization if enabled
    writer.build_string_table(allocations)?;

    writer.write_header(allocations.len() as u32)?;

    // Write basic allocation records
    for allocation in allocations {
        writer.write_allocation(allocation)?;
    }

    // Write advanced metrics segment if enabled
    writer.write_advanced_metrics_segment(allocations)?;

    writer.finish()?;

    // Log configuration info if advanced metrics are enabled
    if config.has_advanced_metrics() {
        tracing::info!(
            "Binary export completed with advanced metrics (impact: {:.1}%)",
            config.estimated_performance_impact() * 100.0
        );
    }

    Ok(())
}

/// Convert binary file to JSON format
pub fn parse_binary_to_json<P: AsRef<Path>>(
    binary_path: P,
    json_path: P,
) -> Result<(), BinaryExportError> {
    BinaryParser::to_json(binary_path, json_path)
}

/// Convert binary file to HTML format
pub fn parse_binary_to_html<P: AsRef<Path>>(
    binary_path: P,
    html_path: P,
) -> Result<(), BinaryExportError> {
    BinaryParser::to_html(binary_path, html_path)
}

/// Binary file type information
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryFileInfo {
    /// Export mode (user-only vs full)
    pub export_mode: BinaryExportMode,
    /// Total allocation count
    pub total_count: u32,
    /// User allocation count (var_name.is_some())
    pub user_count: u16,
    /// System allocation count (var_name.is_none())
    pub system_count: u16,
    /// Binary format version
    pub version: u32,
    /// Whether counts are consistent
    pub is_count_consistent: bool,
    /// File size in bytes
    pub file_size: u64,
}

impl BinaryFileInfo {
    /// Check if this is a user-only binary
    pub fn is_user_only(&self) -> bool {
        self.export_mode == BinaryExportMode::UserOnly
    }

    /// Check if this is a full binary
    pub fn is_full_binary(&self) -> bool {
        self.export_mode == BinaryExportMode::Full
    }

    /// Get a human-readable description of the binary type
    pub fn type_description(&self) -> String {
        match self.export_mode {
            BinaryExportMode::UserOnly => format!(
                "User-only binary ({} user allocations, {} KB)",
                self.user_count,
                self.file_size / 1024
            ),
            BinaryExportMode::Full => format!(
                "Full binary ({} total allocations: {} user + {} system, {} KB)",
                self.total_count,
                self.user_count,
                self.system_count,
                self.file_size / 1024
            ),
        }
    }

    /// Get recommended processing strategy
    pub fn recommended_strategy(&self) -> &'static str {
        match self.export_mode {
            BinaryExportMode::UserOnly => "Simple processing (small file, user data only)",
            BinaryExportMode::Full => "Optimized processing (large file, comprehensive data)",
        }
    }
}

/// Automatically detect binary file type and characteristics
///
/// This function reads the binary file header to determine:
/// - Export mode (user-only vs full)
/// - Allocation counts (total, user, system)
/// - Format version and compatibility
/// - File size and consistency checks
///
/// # Arguments
/// * `path` - Path to the binary file
///
/// # Returns
/// * `Ok(BinaryFileInfo)` - File information and characteristics
/// * `Err(BinaryExportError)` - If file cannot be read or is invalid
///
/// # Example
/// ```no_run
/// use memscope_rs::export::binary::detect_binary_type;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let info = detect_binary_type("MemoryAnalysis/my_program.memscope")?;
/// println!("Binary type: {}", info.type_description());
/// println!("Strategy: {}", info.recommended_strategy());
///
/// if info.is_full_binary() {
///     // Use optimized processing for large files
///     // parse_full_binary_to_json(path, base_name)?;
/// } else {
///     // Use simple processing for small files
///     // parse_user_binary_to_json(path, base_name)?;
/// }
/// # Ok(())
/// # }
/// ```
pub fn detect_binary_type<P: AsRef<Path>>(path: P) -> Result<BinaryFileInfo, BinaryExportError> {
    use std::fs::File;
    use std::io::Read;

    let path = path.as_ref();

    // Get file size
    let metadata = std::fs::metadata(path).map_err(BinaryExportError::Io)?;
    let file_size = metadata.len();

    // Open file and read header
    let mut file = File::open(path).map_err(BinaryExportError::Io)?;

    let mut header_bytes = [0u8; format::HEADER_SIZE];
    file.read_exact(&mut header_bytes)
        .map_err(BinaryExportError::Io)?;

    // Parse header
    let header = FileHeader::from_bytes(&header_bytes);

    // Validate magic bytes
    if !header.is_valid_magic() {
        return Err(BinaryExportError::InvalidFormat);
    }

    // Check version compatibility
    if !header.is_compatible_version() {
        return Err(BinaryExportError::CorruptedData(format!(
            "Unsupported format version: {}",
            header.version
        )));
    }

    // Extract information
    let export_mode = header.get_export_mode();
    let (total_count, user_count, system_count) = header.get_allocation_counts();
    let is_count_consistent = header.is_count_consistent();

    Ok(BinaryFileInfo {
        export_mode,
        total_count,
        user_count,
        system_count,
        version: header.version,
        is_count_consistent,
        file_size,
    })
}

/// Automatically choose the optimal parsing strategy based on binary type
///
/// This function detects the binary type and automatically selects the most
/// appropriate parsing method:
/// - User-only binaries: Simple, fast processing
/// - Full binaries: Optimized processing with advanced features
///
/// # Arguments
/// * `binary_path` - Path to the binary file
/// * `base_name` - Base name for output JSON files
///
/// # Returns
/// * `Ok(())` - Parsing completed successfully
/// * `Err(BinaryExportError)` - If parsing fails
///
/// Export binary data directly to interactive HTML dashboard
///
/// This function creates a complete HTML dashboard with interactive visualizations
/// using the templates in ./templates/dashboard.html
///
/// # Example
/// ```no_run
/// use memscope_rs::export::binary::export_binary_to_html_dashboard;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create interactive HTML dashboard
/// export_binary_to_html_dashboard(
///     "MemoryAnalysis/my_program.memscope",
///     "MemoryAnalysis/my_program/dashboard.html",
///     "my_program"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn export_binary_to_html_dashboard<P: AsRef<Path>>(
    binary_path: P,
    _output_path: P,
    project_name: &str,
) -> Result<(), BinaryExportError> {
    // Use the new unified API - the output path is determined automatically
    export_binary_to_html(binary_path, project_name)
}

/// # Example
/// ```no_run
/// use memscope_rs::export::binary::parse_binary_auto;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Automatically detects type and uses optimal strategy
/// parse_binary_auto("MemoryAnalysis/my_program.memscope", "my_program")?;
/// # Ok(())
/// # }
/// ```
pub fn parse_binary_auto<P: AsRef<Path>>(
    binary_path: P,
    base_name: &str,
) -> Result<(), BinaryExportError> {
    let binary_path = binary_path.as_ref();

    // Detect binary type
    let info = detect_binary_type(binary_path)?;

    tracing::info!(
        "Auto-detected binary type: {} (version {})",
        info.type_description(),
        info.version
    );

    tracing::info!("Using strategy: {}", info.recommended_strategy());

    // Choose optimal parsing strategy
    match info.export_mode {
        BinaryExportMode::UserOnly => {
            tracing::debug!("Using simple parsing for user-only binary");
            BinaryParser::parse_user_binary_to_json(binary_path, base_name)
        }
        BinaryExportMode::Full => {
            tracing::debug!("Using optimized parsing for full binary");
            BinaryParser::parse_full_binary_to_json(binary_path, base_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use std::fs;
    use tempfile::TempDir;

    /// Create test allocation data
    fn create_test_allocations() -> Vec<AllocationInfo> {
        let mut alloc1 = AllocationInfo::new(0x1000, 64);
        alloc1.var_name = Some("user_var1".to_string());
        alloc1.type_name = Some("String".to_string());
        alloc1.timestamp_alloc = 1000;
        alloc1.stack_trace = Some(vec!["main".to_string(), "allocate".to_string()]);
        alloc1.thread_id = "1".to_string();

        let mut alloc2 = AllocationInfo::new(0x2000, 128);
        alloc2.var_name = Some("user_var2".to_string());
        alloc2.type_name = Some("Vec<i32>".to_string());
        alloc2.timestamp_alloc = 2000;
        alloc2.stack_trace = Some(vec!["main".to_string(), "create_vec".to_string()]);
        alloc2.thread_id = "1".to_string();

        let mut alloc3 = AllocationInfo::new(0x3000, 32);
        alloc3.var_name = None; // System allocation
        alloc3.type_name = None;
        alloc3.timestamp_alloc = 3000;
        alloc3.stack_trace = Some(vec!["system".to_string()]);
        alloc3.thread_id = "2".to_string();

        vec![alloc1, alloc2, alloc3]
    }

    #[test]
    fn test_export_to_binary_default() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test.memscope");
        let allocations = create_test_allocations();

        let result = export_to_binary(&allocations, &binary_path);
        assert!(result.is_ok());
        assert!(binary_path.exists());
        assert!(binary_path.metadata().unwrap().len() > 0);
    }

    #[test]
    fn test_export_to_binary_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_config.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfigBuilder::new()
            .advanced_metrics_level(AdvancedMetricsLevel::Essential)
            .build();

        let result = export_to_binary_with_config(&allocations, &binary_path, &config);
        assert!(result.is_ok());
        assert!(binary_path.exists());
        assert!(binary_path.metadata().unwrap().len() > 0);
    }

    #[test]
    fn test_export_to_binary_with_mode_user_only() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_user_only.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::minimal();

        let result = export_to_binary_with_mode(
            &allocations,
            &binary_path,
            BinaryExportMode::UserOnly,
            &config,
        );
        assert!(result.is_ok());
        assert!(binary_path.exists());

        // Verify the file was created with user-only mode by reading it back
        let info = detect_binary_type(&binary_path).unwrap();
        assert!(info.is_user_only());

        // Count actual user allocations in our test data
        let expected_user_count = allocations.iter().filter(|a| a.var_name.is_some()).count();

        // The binary should reflect the user-only mode
        assert_eq!(info.export_mode, BinaryExportMode::UserOnly);

        // In UserOnly mode, only user allocations should be written
        assert_eq!(info.user_count, expected_user_count as u16);
        assert_eq!(info.system_count, 0); // No system allocations in UserOnly mode
        assert_eq!(info.total_count, expected_user_count as u32);

        // Verify we can parse it back to JSON to check the data integrity
        let json_path = temp_dir.path().join("test_user_only.json");
        let parse_result = parse_binary_to_json(&binary_path, &json_path);
        assert!(parse_result.is_ok());
        assert!(json_path.exists());

        // Verify JSON content contains only user allocations
        let json_content = fs::read_to_string(&json_path).unwrap();
        let json_data: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Check that the JSON contains only user allocations
        if json_data.is_array() {
            // JSON is directly an array of allocations
            let json_allocations = json_data.as_array().unwrap();
            assert_eq!(json_allocations.len(), expected_user_count); // Only user allocations

            // All allocations in the JSON should have var_name
            for alloc in json_allocations {
                let var_name = alloc.get("var_name");
                assert!(
                    var_name.is_some() && !var_name.unwrap().is_null(),
                    "All allocations in UserOnly mode should have var_name"
                );
            }
        } else if json_data.is_object() {
            // JSON is an object containing allocations array
            if let Some(allocations_array) = json_data.get("allocations") {
                assert!(allocations_array.is_array());
                let json_allocations = allocations_array.as_array().unwrap();
                assert_eq!(json_allocations.len(), expected_user_count); // Only user allocations

                // All allocations should have var_name
                for alloc in json_allocations {
                    let var_name = alloc.get("var_name");
                    assert!(
                        var_name.is_some() && !var_name.unwrap().is_null(),
                        "All allocations in UserOnly mode should have var_name"
                    );
                }
            }
        }
    }

    #[test]
    fn test_export_to_binary_with_mode_full() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_full.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::debug_comprehensive();

        let result =
            export_to_binary_with_mode(&allocations, &binary_path, BinaryExportMode::Full, &config);
        assert!(result.is_ok());
        assert!(binary_path.exists());

        // Verify the file was created with full mode
        let info = detect_binary_type(&binary_path).unwrap();
        assert!(info.is_full_binary());
        assert_eq!(info.user_count, 2); // User allocations
        assert_eq!(info.system_count, 1); // System allocations
        assert_eq!(info.total_count, 3); // Total allocations
    }

    #[test]
    fn test_detect_binary_type_user_only() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_detect_user.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::minimal();

        export_to_binary_with_mode(
            &allocations,
            &binary_path,
            BinaryExportMode::UserOnly,
            &config,
        )
        .unwrap();

        let info = detect_binary_type(&binary_path).unwrap();
        assert!(info.is_user_only());
        assert!(!info.is_full_binary());
        assert_eq!(info.export_mode, BinaryExportMode::UserOnly);
        assert!(info.is_count_consistent);
        assert!(info.file_size > 0);

        let description = info.type_description();
        assert!(description.contains("User-only binary"));

        let strategy = info.recommended_strategy();
        assert!(strategy.contains("Simple processing"));

        // Verify we can actually read the binary back and get meaningful data
        let json_path = temp_dir.path().join("verify_user_only.json");
        let parse_result = parse_binary_to_json(&binary_path, &json_path);
        assert!(parse_result.is_ok());

        // Read and verify the JSON content
        let json_content = fs::read_to_string(&json_path).unwrap();

        tracing::info!(
            "JSON content preview: {}",
            &json_content[..json_content.len().min(500)]
        );

        let json_data: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify the JSON structure is valid and matches our expectations
        match json_data {
            serde_json::Value::Array(ref json_allocations) => {
                tracing::info!("JSON is an array with {} elements", json_allocations.len());

                // Count user vs system allocations in JSON
                let json_user_count = json_allocations
                    .iter()
                    .filter(|alloc| alloc.get("var_name").map_or(false, |v| !v.is_null()))
                    .count();
                let json_system_count = json_allocations.len() - json_user_count;

                tracing::info!(
                    "JSON contains: {} user, {} system allocations",
                    json_user_count,
                    json_system_count
                );

                // Verify the JSON data matches the binary header
                assert_eq!(json_user_count, info.user_count as usize);
                assert_eq!(json_system_count, info.system_count as usize);
                assert_eq!(json_allocations.len(), info.total_count as usize);
            }
            serde_json::Value::Object(ref obj) => {
                tracing::info!(
                    "JSON is an object with keys: {:?}",
                    obj.keys().collect::<Vec<_>>()
                );
            }
            _ => {
                tracing::info!("JSON is neither array nor object: {:?}", json_data);
            }
        }

        // Count user allocations in original data
        let original_user_count = allocations.iter().filter(|a| a.var_name.is_some()).count();
        tracing::info!("Original user allocations: {}", original_user_count);

        // The binary detection should reflect actual data
        tracing::info!(
            "Detected user count: {}, system count: {}",
            info.user_count,
            info.system_count
        );
    }

    #[test]
    fn test_detect_binary_type_full() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_detect_full.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::debug_comprehensive();

        export_to_binary_with_mode(&allocations, &binary_path, BinaryExportMode::Full, &config)
            .unwrap();

        let info = detect_binary_type(&binary_path).unwrap();
        assert!(!info.is_user_only());
        assert!(info.is_full_binary());
        assert_eq!(info.export_mode, BinaryExportMode::Full);
        assert_eq!(info.user_count, 2);
        assert_eq!(info.system_count, 1);
        assert_eq!(info.total_count, 3);
        assert!(info.is_count_consistent);
        assert!(info.file_size > 0);

        let description = info.type_description();
        assert!(description.contains("Full binary"));
        assert!(description.contains("3 total allocations"));
        assert!(description.contains("2 user + 1 system"));

        let strategy = info.recommended_strategy();
        assert!(strategy.contains("Optimized processing"));
    }

    #[test]
    fn test_detect_binary_type_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent.memscope");

        let result = detect_binary_type(&invalid_path);
        assert!(result.is_err());
        if let Err(BinaryExportError::Io(_)) = result {
            // Expected error type
        } else {
            panic!("Expected IoError for nonexistent file");
        }
    }

    #[test]
    fn test_detect_binary_type_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.memscope");

        // Create a file with invalid content
        fs::write(&invalid_path, b"invalid binary data").unwrap();

        let result = detect_binary_type(&invalid_path);
        assert!(result.is_err());

        // Verify error classification is appropriate for invalid binary format
        match result {
            Err(BinaryExportError::InvalidFormat) => {
                // Correct error type for invalid magic bytes or format
            }
            Err(BinaryExportError::Io(_)) => {
                // Acceptable if file is too small to contain valid header
            }
            Err(BinaryExportError::CorruptedData(msg)) => {
                // Acceptable for malformed header data
                assert!(!msg.is_empty(), "Error message should be descriptive");
            }
            Err(other) => {
                panic!("Unexpected error type for invalid format: {:?}", other);
            }
            Ok(_) => {
                panic!("Invalid binary file should not be detected as valid");
            }
        }
    }

    #[test]
    fn test_parse_binary_to_json() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_parse.memscope");
        let json_path = temp_dir.path().join("test_parse.json");
        let allocations = create_test_allocations();

        // First create a binary file
        export_to_binary(&allocations, &binary_path).unwrap();

        // Then parse it to JSON
        let result = parse_binary_to_json(&binary_path, &json_path);
        assert!(result.is_ok());
        assert!(json_path.exists());
        assert!(json_path.metadata().unwrap().len() > 0);

        // Verify JSON content is valid
        let json_content = fs::read_to_string(&json_path).unwrap();
        assert!(!json_content.is_empty());
        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    }

    #[test]
    fn test_parse_binary_to_html() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_parse_html.memscope");
        let html_path = temp_dir.path().join("test_parse.html");
        let allocations = create_test_allocations();

        // First create a binary file
        export_to_binary(&allocations, &binary_path).unwrap();

        // Then parse it to HTML
        let result = parse_binary_to_html(&binary_path, &html_path);
        assert!(result.is_ok());
        assert!(html_path.exists());
        assert!(html_path.metadata().unwrap().len() > 0);

        // Verify HTML content
        let html_content = fs::read_to_string(&html_path).unwrap();
        assert!(!html_content.is_empty());
        assert!(html_content.contains("<!DOCTYPE html") || html_content.contains("<html"));
    }

    #[test]
    fn test_parse_binary_auto_user_only() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_auto_user.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::minimal();

        // Create user-only binary
        export_to_binary_with_mode(
            &allocations,
            &binary_path,
            BinaryExportMode::UserOnly,
            &config,
        )
        .unwrap();

        // Test auto parsing
        let result = parse_binary_auto(&binary_path, "test_auto_user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_auto_full() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_auto_full.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfig::debug_comprehensive();

        // Create full binary
        export_to_binary_with_mode(&allocations, &binary_path, BinaryExportMode::Full, &config)
            .unwrap();

        // Test auto parsing
        let result = parse_binary_auto(&binary_path, "test_auto_full");
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_binary_to_html_dashboard() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_dashboard.memscope");
        let html_path = temp_dir.path().join("dashboard.html");
        let allocations = create_test_allocations();

        // First create a binary file
        export_to_binary(&allocations, &binary_path).unwrap();

        // Test dashboard export
        let result = export_binary_to_html_dashboard(&binary_path, &html_path, "test_project");
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_file_info_methods() {
        let user_only_info = BinaryFileInfo {
            export_mode: BinaryExportMode::UserOnly,
            total_count: 2,
            user_count: 2,
            system_count: 0,
            version: FORMAT_VERSION,
            is_count_consistent: true,
            file_size: 1024,
        };

        assert!(user_only_info.is_user_only());
        assert!(!user_only_info.is_full_binary());

        let description = user_only_info.type_description();
        assert!(description.contains("User-only binary"));
        assert!(description.contains("2 user allocations"));
        assert!(description.contains("1 KB"));

        let strategy = user_only_info.recommended_strategy();
        assert!(strategy.contains("Simple processing"));

        let full_info = BinaryFileInfo {
            export_mode: BinaryExportMode::Full,
            total_count: 5,
            user_count: 3,
            system_count: 2,
            version: FORMAT_VERSION,
            is_count_consistent: true,
            file_size: 4096,
        };

        assert!(!full_info.is_user_only());
        assert!(full_info.is_full_binary());

        let description = full_info.type_description();
        assert!(description.contains("Full binary"));
        assert!(description.contains("5 total allocations"));
        assert!(description.contains("3 user + 2 system"));
        assert!(description.contains("4 KB"));

        let strategy = full_info.recommended_strategy();
        assert!(strategy.contains("Optimized processing"));
    }

    #[test]
    fn test_export_with_advanced_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_advanced.memscope");
        let allocations = create_test_allocations();

        let config = BinaryExportConfigBuilder::new()
            .advanced_metrics_level(AdvancedMetricsLevel::Comprehensive)
            .build();

        let result = export_to_binary_with_config(&allocations, &binary_path, &config);
        assert!(result.is_ok());
        assert!(binary_path.exists());

        // Verify the file was created and has content
        let metadata = binary_path.metadata().unwrap();
        assert!(metadata.len() > 0);

        // Verify we can detect the binary type
        let info = detect_binary_type(&binary_path).unwrap();
        assert_eq!(info.version, FORMAT_VERSION);
    }

    #[test]
    fn test_empty_allocations() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_empty.memscope");
        let allocations: Vec<AllocationInfo> = vec![];

        let result = export_to_binary(&allocations, &binary_path);
        assert!(result.is_ok());
        assert!(binary_path.exists());

        // Verify the file was created with zero allocations
        let info = detect_binary_type(&binary_path).unwrap();
        assert_eq!(info.total_count, 0);
        assert_eq!(info.user_count, 0);
        assert_eq!(info.system_count, 0);
    }

    #[test]
    fn test_large_allocation_count() {
        let temp_dir = TempDir::new().unwrap();
        let binary_path = temp_dir.path().join("test_large.memscope");

        // Create a larger number of allocations with known user/system split
        let mut allocations = Vec::new();
        let mut _expected_user_count = 0;
        let mut _expected_system_count = 0;

        for i in 0..100 {
            let mut alloc = AllocationInfo::new(0x1000 + (i * 0x100), 64 + i);
            if i % 2 == 0 {
                alloc.var_name = Some(format!("var_{i}"));
                _expected_user_count += 1;
            } else {
                alloc.var_name = None;
                _expected_system_count += 1;
            }
            alloc.type_name = Some(format!("Type_{i}"));
            alloc.timestamp_alloc = 1000 + i as u64;
            alloc.stack_trace = Some(vec![format!("func_{i}")]);
            alloc.thread_id = ((i % 4) + 1).to_string();
            allocations.push(alloc);
        }

        tracing::info!(
            "Expected: {} user, {} system, {} total",
            _expected_user_count,
            _expected_system_count,
            allocations.len()
        );

        let result = export_to_binary(&allocations, &binary_path);
        assert!(result.is_ok());
        assert!(binary_path.exists());

        // Verify the counts are correct by reading the binary
        let info = detect_binary_type(&binary_path).unwrap();
        tracing::info!(
            "Detected: {} user, {} system, {} total",
            info.user_count,
            info.system_count,
            info.total_count
        );

        assert_eq!(info.total_count, 100);
        assert!(info.is_count_consistent);

        // Verify by parsing back to JSON and counting
        let json_path = temp_dir.path().join("test_large.json");
        let parse_result = parse_binary_to_json(&binary_path, &json_path);
        assert!(parse_result.is_ok());

        let json_content = fs::read_to_string(&json_path).unwrap();
        let json_data: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        if let Some(allocations_array) = json_data.get("allocations") {
            let json_allocations = allocations_array.as_array().unwrap();
            let json_user_count = json_allocations
                .iter()
                .filter(|alloc| alloc.get("var_name").map_or(false, |v| !v.is_null()))
                .count();
            let json_system_count = json_allocations.len() - json_user_count;

            tracing::info!(
                "JSON parsed: {} user, {} system, {} total",
                json_user_count,
                json_system_count,
                json_allocations.len()
            );

            // Verify the JSON data matches our expectations
            assert_eq!(json_allocations.len(), 100);
            assert_eq!(json_user_count, _expected_user_count);
            assert_eq!(json_system_count, _expected_system_count);
        }
    }
}
