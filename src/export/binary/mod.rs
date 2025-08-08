//! Binary export module for high-performance memory tracking data export.
//!
//! Features:
//! - 3x faster than JSON export with 60%+ smaller file size
//! - Lock-free, single-threaded design for simplicity
//! - Compatible with existing JSON/HTML export APIs
//! - Modular architecture for easy testing and maintenance

mod batch_processor;
mod cache;
mod config;
mod error;
mod field_parser;
mod filter_engine;
mod format;
mod index;
mod index_builder;

mod memory_layout_serialization;
mod parser;
#[cfg(test)]
mod performance_tests;
mod reader;
mod selective_reader;
mod serializable;
mod smart_pointer_serialization;
mod string_table;
mod writer;

pub use batch_processor::{BatchProcessor, BatchProcessorBuilder, BatchProcessorConfig, BatchProcessorStats, RecordBatch};
pub use cache::{CacheEntry, CacheStats, IndexCache, IndexCacheConfig};
pub use config::{AdvancedMetricsLevel, BinaryExportConfig, BinaryExportConfigBuilder};
pub use error::BinaryExportError;
pub use field_parser::{FieldParser, FieldParserConfig, FieldParserStats, PartialAllocationInfo};
pub use filter_engine::{FilterEngine, FilterEngineBuilder, FilterOptimizer, FilterStats};
pub use format::{FileHeader, FORMAT_VERSION, MAGIC_BYTES};
pub use index::{BinaryIndex, CompactAllocationIndex, QuickFilterData, RecordMetadata};
pub use index_builder::BinaryIndexBuilder;
pub use parser::BinaryParser;
pub use reader::BinaryReader;
pub use selective_reader::{
    AllocationField, AllocationFilter, SelectiveReadOptions, SelectiveReadOptionsBuilder,
    SortField, SortOrder,
};
pub use writer::BinaryWriter;

use crate::core::types::AllocationInfo;
use std::path::Path;

/// Export allocation information to binary format with default configuration
pub fn export_to_binary<P: AsRef<Path>>(
    allocations: &[AllocationInfo],
    path: P,
) -> Result<(), BinaryExportError> {
    export_to_binary_with_config(allocations, path, &BinaryExportConfig::default())
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
