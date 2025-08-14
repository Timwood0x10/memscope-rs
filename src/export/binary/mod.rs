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
mod ffi_safety_analyzer;
mod variable_relationship_analyzer;
#[cfg(test)]
mod integration_test_complex_types;
#[cfg(test)]
mod integration_test_ffi_safety;
#[cfg(test)]
mod integration_test_template_resources;
#[cfg(test)]
mod integration_test_variable_relationships;
mod error;
mod error_recovery;
mod field_parser;
mod filter_engine;
pub mod format;
mod html_export;
mod index;
mod index_builder;
mod memory_layout_serialization;
mod parser;
mod reader;
mod selective_json_exporter;
mod selective_reader;
mod serializable;
mod smart_pointer_serialization;
mod streaming_field_processor;
mod streaming_json_writer;
mod string_table;
mod template_resource_manager;
mod writer;

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
    ComplexTypeAnalysis, ComplexTypeAnalyzer, ComplexTypeSummary, CategorizedTypes,
    TypeInfo, GenericTypeAnalysis, GenericInstantiation, TypeCategory,
};
pub use ffi_safety_analyzer::{
    FfiSafetyAnalysis, FfiSafetyAnalyzer, FfiSafetySummary, UnsafeOperation,
    FfiHotspot, RiskAssessment, FfiCallGraph, UnsafeOperationType, RiskLevel,
};
pub use variable_relationship_analyzer::{
    VariableRelationshipAnalysis, VariableRelationshipAnalyzer, RelationshipGraph,
    GraphNode, GraphEdge, RelationshipType, NodeCategory, OwnershipStatus,
};

pub use config::{AdvancedMetricsLevel, BinaryExportConfig, BinaryExportConfigBuilder};
pub use error::BinaryExportError;
pub use error_recovery::{
    ErrorRecoveryManager, ErrorReport, ErrorStatistics, ErrorTrend, RecoveryConfig, RecoveryResult,
    RecoveryStrategy,
};
pub use field_parser::{FieldParser, FieldParserConfig, FieldParserStats, PartialAllocationInfo};
pub use filter_engine::{FilterEngine, FilterEngineBuilder, FilterOptimizer, FilterStats};
pub use format::{BinaryExportMode, FileHeader, FORMAT_VERSION, MAGIC_BYTES};
pub use html_export::{
    export_binary, export_binary_optimized, export_binary_with_format,
    export_binary_to_json, export_binary_to_html, export_binary_to_html_system, 
    export_binary_to_html_both, export_binary_to_both,
    export_binary_with_config, show_export_options,
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
    TemplateResourceManager, TemplateData, ResourceConfig, PlaceholderProcessor,
    create_template_data,
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
pub fn export_to_binary_with_mode<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    path: P,
    export_mode: BinaryExportMode,
    config: &BinaryExportConfig,
) -> Result<(), BinaryExportError> {
    let mut writer = BinaryWriter::new_with_config(path, config)?;

    // Build string table for optimization if enabled
    writer.build_string_table(allocations)?;

    // Calculate user vs system allocation counts
    let user_count = allocations.iter().filter(|a| a.var_name.is_some()).count() as u16;
    let system_count = (allocations.len() - user_count as usize) as u16;
    let total_count = allocations.len() as u32;

    // Write enhanced header with mode and counts
    writer.write_enhanced_header(total_count, export_mode, user_count, system_count)?;

    // Write allocation records
    for allocation in allocations {
        writer.write_allocation(allocation)?;
    }

    writer.finish()?;
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
    let metadata = std::fs::metadata(path).map_err(|e| BinaryExportError::Io(e))?;
    let file_size = metadata.len();

    // Open file and read header
    let mut file = File::open(path).map_err(|e| BinaryExportError::Io(e))?;

    let mut header_bytes = [0u8; format::HEADER_SIZE];
    file.read_exact(&mut header_bytes)
        .map_err(|e| BinaryExportError::Io(e))?;

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
