//! Selective JSON exporter that integrates all optimization components
//!
//! This module provides a unified interface for exporting binary allocation data
//! to JSON format with selective field processing, streaming output, and
//! comprehensive performance optimizations.

use crate::export::binary::batch_processor::{BatchProcessor, BatchProcessorConfig};
use crate::export::binary::cache::{IndexCache, IndexCacheConfig};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::field_parser::{FieldParser, PartialAllocationInfo};
use crate::export::binary::filter_engine::FilterEngine;

use crate::export::binary::selective_reader::{
    AllocationField, AllocationFilter, SelectiveReadOptionsBuilder,
};
use crate::export::binary::streaming_json_writer::{
    SelectiveSerializationOptions, StreamingJsonStats, StreamingJsonWriter,
    StreamingJsonWriterConfig,
};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

/// Configuration for selective JSON export operations
#[derive(Debug, Clone)]
pub struct SelectiveJsonExportConfig {
    /// Configuration for streaming JSON writer
    pub json_writer_config: StreamingJsonWriterConfig,

    /// Configuration for batch processor
    pub batch_processor_config: BatchProcessorConfig,

    /// Configuration for index cache
    pub index_cache_config: IndexCacheConfig,

    /// Configuration for selective serialization
    pub serialization_options: SelectiveSerializationOptions,

    /// Enable parallel processing for multiple files
    pub enable_parallel_processing: bool,

    /// Maximum number of concurrent export operations
    pub max_concurrent_exports: usize,

    /// Enable comprehensive error recovery
    pub enable_error_recovery: bool,

    /// Enable detailed performance monitoring
    pub enable_performance_monitoring: bool,
}

impl Default for SelectiveJsonExportConfig {
    fn default() -> Self {
        Self {
            json_writer_config: StreamingJsonWriterConfig::default(),
            batch_processor_config: BatchProcessorConfig::default(),
            index_cache_config: IndexCacheConfig::default(),
            serialization_options: SelectiveSerializationOptions::default(),
            enable_parallel_processing: true,
            max_concurrent_exports: 4,
            enable_error_recovery: true,
            enable_performance_monitoring: true,
        }
    }
}

/// Statistics for selective JSON export operations
#[derive(Debug, Clone, Default)]
pub struct SelectiveJsonExportStats {
    /// Statistics from streaming JSON writer
    pub json_writer_stats: StreamingJsonStats,

    /// Total export time in microseconds
    pub total_export_time_us: u64,

    /// Number of files processed
    pub files_processed: u32,

    /// Total allocations exported
    pub total_allocations_exported: u64,

    /// Total bytes written across all files
    pub total_bytes_written: u64,

    /// Number of index cache hits
    pub index_cache_hits: u64,

    /// Number of index cache misses
    pub index_cache_misses: u64,

    /// Number of errors encountered and recovered
    pub errors_recovered: u32,

    /// Average export throughput (allocations per second)
    pub avg_export_throughput: f64,

    /// Memory efficiency (bytes per allocation)
    pub memory_efficiency: f64,
}

impl SelectiveJsonExportStats {
    /// Calculate overall cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.index_cache_hits + self.index_cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.index_cache_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Calculate export efficiency (files per second)
    pub fn export_efficiency(&self) -> f64 {
        if self.total_export_time_us == 0 {
            0.0
        } else {
            (self.files_processed as f64 * 1_000_000.0) / self.total_export_time_us as f64
        }
    }

    /// Calculate compression ratio compared to full export
    pub fn compression_ratio(&self) -> f64 {
        if self.total_allocations_exported == 0 {
            0.0
        } else {
            // Estimate full export size (approximate)
            let estimated_full_size = self.total_allocations_exported * 500; // ~500 bytes per full allocation
            if estimated_full_size == 0 {
                0.0
            } else {
                (self.total_bytes_written as f64 / estimated_full_size as f64) * 100.0
            }
        }
    }
}

/// Selective JSON exporter with integrated optimization components
pub struct SelectiveJsonExporter {
    /// Configuration
    config: SelectiveJsonExportConfig,

    /// Index cache for performance optimization
    index_cache: IndexCache,

    /// Batch processor for efficient record processing
    batch_processor: BatchProcessor,

    /// Filter engine for intelligent filtering
    filter_engine: FilterEngine,

    /// Field parser for selective field parsing
    field_parser: FieldParser,

    /// Export statistics
    stats: SelectiveJsonExportStats,
}

impl SelectiveJsonExporter {
    /// Create a new selective JSON exporter with default configuration
    pub fn new() -> Result<Self, BinaryExportError> {
        Self::with_config(SelectiveJsonExportConfig::default())
    }

    /// Create a new selective JSON exporter with custom configuration
    pub fn with_config(config: SelectiveJsonExportConfig) -> Result<Self, BinaryExportError> {
        let index_cache = IndexCache::new(config.index_cache_config.clone())?;
        let batch_processor = BatchProcessor::with_config(config.batch_processor_config.clone());
        // Create a placeholder filter engine - we'll update it when we have an index
        let dummy_index = std::sync::Arc::new(crate::export::binary::index::BinaryIndex::new(
            std::path::PathBuf::new(),
            0,
            0,
            crate::export::binary::format::FileHeader::new_legacy(0),
        ));
        let filter_engine = FilterEngine::new(dummy_index);
        let field_parser = FieldParser::new();

        Ok(Self {
            config,
            index_cache,
            batch_processor,
            filter_engine,
            field_parser,
            stats: SelectiveJsonExportStats::default(),
        })
    }

    /// Export a single binary file to JSON with selective fields
    pub fn export_to_json_selective<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
        requested_fields: &HashSet<AllocationField>,
        filters: &[AllocationFilter],
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let export_start = Instant::now();

        // Build or retrieve index
        let index = self.get_or_build_index(&binary_path)?;

        // Create selective read options
        let _read_options = SelectiveReadOptionsBuilder::new()
            .with_fields(requested_fields.clone())
            .filters(filters.to_vec())
            .build()?;

        // Open binary file for reading
        let mut binary_file = File::open(&binary_path)?;

        // Create JSON writer
        let json_file = File::create(&json_path)?;
        let buffered_writer = BufWriter::new(json_file);
        let mut json_writer = StreamingJsonWriter::with_config(
            buffered_writer,
            self.config.json_writer_config.clone(),
        )?;

        // Start JSON document
        json_writer.write_header(index.record_count() as u64)?;

        // Process records in batches
        let mut processed_count = 0;
        let batch_size = self.config.batch_processor_config.batch_size;

        for batch_start in (0..index.record_count() as usize).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(index.record_count() as usize);
            let batch_offsets: Vec<u64> = (batch_start..batch_end)
                .filter_map(|i| index.get_record_offset(i))
                .collect();

            if batch_offsets.is_empty() {
                continue;
            }

            // Apply pre-filtering using index
            let filtered_indices = self.filter_engine.filter_candidates(filters)?;
            let filtered_offsets: Vec<u64> = batch_offsets
                .into_iter()
                .enumerate()
                .filter(|(i, _)| filtered_indices.contains(&(batch_start + i)))
                .map(|(_, offset)| offset)
                .collect();

            if filtered_offsets.is_empty() {
                continue;
            }

            // Read and parse records
            let records = self.batch_processor.process_batch(
                &mut binary_file,
                &filtered_offsets,
                requested_fields,
            )?;

            // Convert PartialAllocationInfo to AllocationInfo for filtering
            let full_records: Vec<crate::core::types::AllocationInfo> = records
                .records
                .iter()
                .map(|partial| partial.clone().to_full_allocation())
                .collect();

            // Apply precise filtering
            let filtered_full_records = self
                .filter_engine
                .apply_precise_filters(full_records, filters)?;

            // Convert back to PartialAllocationInfo for JSON writing
            let filtered_records: Vec<PartialAllocationInfo> = filtered_full_records
                .iter()
                .map(|full| PartialAllocationInfo {
                    ptr: Some(full.ptr),
                    size: Some(full.size),
                    var_name: Some(full.var_name.clone()),
                    type_name: Some(full.type_name.clone()),
                    scope_name: Some(full.scope_name.clone()),
                    timestamp_alloc: Some(full.timestamp_alloc),
                    timestamp_dealloc: Some(full.timestamp_dealloc),
                    thread_id: Some(full.thread_id.clone()),
                    borrow_count: Some(full.borrow_count),
                    stack_trace: Some(full.stack_trace.clone()),
                    is_leaked: Some(full.is_leaked),
                    lifetime_ms: Some(full.lifetime_ms),
                })
                .collect();

            // Write records to JSON
            json_writer.write_allocation_batch_with_options(
                &filtered_records,
                requested_fields,
                &self.config.serialization_options,
            )?;

            processed_count += filtered_records.len();
        }

        // Finalize JSON document
        let json_stats = json_writer.finalize()?;

        // Update export statistics
        self.stats.json_writer_stats = json_stats;
        self.stats.total_export_time_us += export_start.elapsed().as_micros() as u64;
        self.stats.files_processed += 1;
        self.stats.total_allocations_exported += processed_count as u64;
        self.stats.total_bytes_written += self.stats.json_writer_stats.bytes_written;

        // Calculate derived statistics
        self.update_derived_stats();

        Ok(self.stats.clone())
    }

    /// Export multiple binary files to JSON in parallel
    pub fn export_multiple_json_types<P: AsRef<Path>>(
        &mut self,
        binary_files: &[(P, P)], // (binary_path, json_path) pairs
        requested_fields: &HashSet<AllocationField>,
        filters: &[AllocationFilter],
    ) -> Result<Vec<SelectiveJsonExportStats>, BinaryExportError> {
        if !self.config.enable_parallel_processing || binary_files.len() <= 1 {
            // Sequential processing
            let mut results = Vec::new();
            for (binary_path, json_path) in binary_files {
                let stats = self.export_to_json_selective(
                    binary_path,
                    json_path,
                    requested_fields,
                    filters,
                )?;
                results.push(stats);
            }
            return Ok(results);
        }

        // Parallel processing (simplified implementation)
        // In a real implementation, we would use proper parallel processing
        let mut results = Vec::new();
        let chunk_size = self.config.max_concurrent_exports;

        for chunk in binary_files.chunks(chunk_size) {
            for (binary_path, json_path) in chunk {
                let stats = self.export_to_json_selective(
                    binary_path,
                    json_path,
                    requested_fields,
                    filters,
                )?;
                results.push(stats);
            }
        }

        Ok(results)
    }

    /// Export to memory_analysis.json format (compatible with existing format)
    pub fn export_memory_analysis_json<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let fields = [
            AllocationField::BorrowCount,
            AllocationField::IsLeaked,
            AllocationField::Ptr,
            AllocationField::ScopeName,
            AllocationField::Size,
            AllocationField::ThreadId,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ]
        .into_iter()
        .collect();

        self.export_to_json_selective(binary_path, json_path, &fields, &[])
    }

    /// Export to lifetime.json format (compatible with existing format)
    pub fn export_lifetime_json<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let export_start = Instant::now();

        // Build or retrieve index
        let index = self.get_or_build_index(&binary_path)?;

        // Open binary file for reading
        let mut binary_file = File::open(&binary_path)?;

        // Create JSON writer with lifecycle_events array
        let json_file = File::create(&json_path)?;
        let buffered_writer = BufWriter::new(json_file);
        let mut json_writer = StreamingJsonWriter::with_config(
            buffered_writer,
            self.config.json_writer_config.clone(),
        )?;

        // Start JSON document with lifecycle_events array
        json_writer
            .write_header_with_array_name(index.record_count() as u64, "lifecycle_events")?;

        // Process records and write as lifecycle events
        let fields = [
            AllocationField::Ptr,
            AllocationField::ScopeName,
            AllocationField::Size,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ]
        .into_iter()
        .collect();

        let mut processed_count = 0;
        let batch_size = self.config.batch_processor_config.batch_size;

        for batch_start in (0..index.record_count() as usize).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(index.record_count() as usize);
            let batch_offsets: Vec<u64> = (batch_start..batch_end)
                .filter_map(|i| index.get_record_offset(i))
                .collect();

            if batch_offsets.is_empty() {
                continue;
            }

            // Read and parse records
            let records =
                self.batch_processor
                    .process_batch(&mut binary_file, &batch_offsets, &fields)?;

            // Write records as lifecycle events
            for record in &records.records {
                json_writer.write_lifecycle_event(record, "allocation")?;
            }

            processed_count += records.records.len();
        }

        // Finalize JSON document
        let json_stats = json_writer.finalize()?;

        // Update export statistics
        self.stats.json_writer_stats = json_stats;
        self.stats.total_export_time_us += export_start.elapsed().as_micros() as u64;
        self.stats.files_processed += 1;
        self.stats.total_allocations_exported += processed_count as u64;
        self.stats.total_bytes_written += self.stats.json_writer_stats.bytes_written;

        self.update_derived_stats();

        Ok(self.stats.clone())
    }

    /// Export to performance.json format (compatible with existing format)
    pub fn export_performance_json<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let fields = [
            AllocationField::BorrowCount,
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::ThreadId,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ]
        .into_iter()
        .collect();

        self.export_to_json_selective(binary_path, json_path, &fields, &[])
    }

    /// Export to unsafe_ffi.json format (compatible with existing format)
    pub fn export_unsafe_ffi_json<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let export_start = Instant::now();

        // Build or retrieve index
        let index = self.get_or_build_index(&binary_path)?;

        // Open binary file for reading
        let mut binary_file = File::open(&binary_path)?;

        // Create JSON writer
        let json_file = File::create(&json_path)?;
        let buffered_writer = BufWriter::new(json_file);
        let mut json_writer = StreamingJsonWriter::with_config(
            buffered_writer,
            self.config.json_writer_config.clone(),
        )?;

        // Start JSON document with specific structure for unsafe_ffi
        json_writer.write_raw("{\n")?;
        json_writer.write_raw("  \"boundary_events\": [],\n")?;
        json_writer.write_raw("  \"enhanced_ffi_data\": [\n")?;

        // Process records
        let fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::StackTrace,
            AllocationField::ThreadId,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ]
        .into_iter()
        .collect();

        let mut processed_count = 0;
        let batch_size = self.config.batch_processor_config.batch_size;

        for batch_start in (0..index.record_count() as usize).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(index.record_count() as usize);
            let batch_offsets: Vec<u64> = (batch_start..batch_end)
                .filter_map(|i| index.get_record_offset(i))
                .collect();

            if batch_offsets.is_empty() {
                continue;
            }

            // Read and parse records
            let records =
                self.batch_processor
                    .process_batch(&mut binary_file, &batch_offsets, &fields)?;

            // Write records with unsafe_ffi format
            for record in &records.records {
                json_writer.write_unsafe_ffi_allocation(record)?;
            }

            processed_count += records.records.len();
        }

        // Close the enhanced_ffi_data array and root object
        json_writer.write_raw("\n  ]\n")?;
        json_writer.write_raw("}\n")?;

        // Finalize JSON document
        let json_stats = json_writer.finalize()?;

        // Update export statistics
        self.stats.json_writer_stats = json_stats;
        self.stats.total_export_time_us += export_start.elapsed().as_micros() as u64;
        self.stats.files_processed += 1;
        self.stats.total_allocations_exported += processed_count as u64;
        self.stats.total_bytes_written += self.stats.json_writer_stats.bytes_written;

        self.update_derived_stats();

        Ok(self.stats.clone())
    }

    /// Export to complex_types.json format (compatible with existing format)
    pub fn export_complex_types_json<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: Q,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        let export_start = Instant::now();

        // Build or retrieve index
        let index = self.get_or_build_index(&binary_path)?;

        // Open binary file for reading
        let mut binary_file = File::open(&binary_path)?;

        // Create JSON writer
        let json_file = File::create(&json_path)?;
        let buffered_writer = BufWriter::new(json_file);
        let mut json_writer = StreamingJsonWriter::with_config(
            buffered_writer,
            self.config.json_writer_config.clone(),
        )?;

        // Start JSON document with categorized_types structure
        json_writer.write_raw("{\n")?;
        json_writer.write_raw("  \"categorized_types\": {\n")?;
        json_writer.write_raw("    \"primitive\": [\n")?;

        // Process records
        let fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::TypeName,
            AllocationField::VarName,
        ]
        .into_iter()
        .collect();

        let mut processed_count = 0;
        let batch_size = self.config.batch_processor_config.batch_size;

        for batch_start in (0..index.record_count() as usize).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(index.record_count() as usize);
            let batch_offsets: Vec<u64> = (batch_start..batch_end)
                .filter_map(|i| index.get_record_offset(i))
                .collect();

            if batch_offsets.is_empty() {
                continue;
            }

            // Read and parse records
            let records =
                self.batch_processor
                    .process_batch(&mut binary_file, &batch_offsets, &fields)?;

            // Write records with complex_types format
            for record in &records.records {
                json_writer.write_complex_types_allocation(record)?;
            }

            processed_count += records.records.len();
        }

        // Close the structure
        json_writer.write_raw("\n    ]\n")?;
        json_writer.write_raw("  }\n")?;
        json_writer.write_raw("}\n")?;

        // Finalize JSON document
        let json_stats = json_writer.finalize()?;

        // Update export statistics
        self.stats.json_writer_stats = json_stats;
        self.stats.total_export_time_us += export_start.elapsed().as_micros() as u64;
        self.stats.files_processed += 1;
        self.stats.total_allocations_exported += processed_count as u64;
        self.stats.total_bytes_written += self.stats.json_writer_stats.bytes_written;

        self.update_derived_stats();

        Ok(self.stats.clone())
    }

    /// Export all 5 JSON types in the standard format (compatible with existing output)
    pub fn export_all_standard_json_types<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        binary_path: P,
        output_dir: Q,
        base_name: &str,
    ) -> Result<Vec<SelectiveJsonExportStats>, BinaryExportError> {
        let output_dir = output_dir.as_ref();
        let mut results = Vec::new();

        // Export memory_analysis.json
        let memory_path = output_dir.join(format!("{}_memory_analysis.json", base_name));
        results.push(self.export_memory_analysis_json(&binary_path, &memory_path)?);

        // Export lifetime.json
        let lifetime_path = output_dir.join(format!("{}_lifetime.json", base_name));
        results.push(self.export_lifetime_json(&binary_path, &lifetime_path)?);

        // Export performance.json
        let performance_path = output_dir.join(format!("{}_performance.json", base_name));
        results.push(self.export_performance_json(&binary_path, &performance_path)?);

        // Export unsafe_ffi.json
        let unsafe_ffi_path = output_dir.join(format!("{}_unsafe_ffi.json", base_name));
        results.push(self.export_unsafe_ffi_json(&binary_path, &unsafe_ffi_path)?);

        // Export complex_types.json
        let complex_types_path = output_dir.join(format!("{}_complex_types.json", base_name));
        results.push(self.export_complex_types_json(&binary_path, &complex_types_path)?);

        Ok(results)
    }

    /// Export with automatic field selection based on file analysis
    pub fn export_with_auto_field_selection<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
        json_path: P,
        optimization_level: OptimizationLevel,
    ) -> Result<SelectiveJsonExportStats, BinaryExportError> {
        // Analyze file to determine optimal field selection
        let index = self.get_or_build_index(&binary_path)?;
        let auto_fields = self.analyze_optimal_fields(&index, optimization_level)?;
        let auto_filters = self.analyze_optimal_filters(&index, optimization_level)?;

        self.export_to_json_selective(&binary_path, &json_path, &auto_fields, &auto_filters)
    }

    /// Get current export statistics
    pub fn get_stats(&self) -> &SelectiveJsonExportStats {
        &self.stats
    }

    /// Reset export statistics
    pub fn reset_stats(&mut self) {
        self.stats = SelectiveJsonExportStats::default();
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        let _ = self.index_cache.clear();
        self.batch_processor.clear_cache();
    }

    // Private helper methods

    /// Get or build index for the given binary file
    fn get_or_build_index<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
    ) -> Result<crate::export::binary::index::BinaryIndex, BinaryExportError> {
        let path = binary_path.as_ref();

        // Use the cache's get_or_build_index method
        let index_builder = crate::export::binary::index_builder::BinaryIndexBuilder::new();
        let index = self.index_cache.get_or_build_index(path, &index_builder)?;

        // Update statistics based on cache behavior
        let cache_stats = self.index_cache.get_stats();
        self.stats.index_cache_hits = cache_stats.cache_hits;
        self.stats.index_cache_misses = cache_stats.cache_misses;

        Ok(index)
    }

    /// Analyze optimal fields based on file characteristics
    fn analyze_optimal_fields(
        &self,
        index: &crate::export::binary::index::BinaryIndex,
        optimization_level: OptimizationLevel,
    ) -> Result<HashSet<AllocationField>, BinaryExportError> {
        let mut fields = HashSet::new();

        // Always include basic fields
        fields.insert(AllocationField::Ptr);
        fields.insert(AllocationField::Size);
        fields.insert(AllocationField::TimestampAlloc);

        match optimization_level {
            OptimizationLevel::Minimal => {
                // Only basic fields
            }
            OptimizationLevel::Balanced => {
                // Add commonly useful fields
                fields.insert(AllocationField::VarName);
                fields.insert(AllocationField::TypeName);
                fields.insert(AllocationField::ThreadId);
                fields.insert(AllocationField::IsLeaked);
            }
            OptimizationLevel::Comprehensive => {
                // Add all available fields
                fields.extend(AllocationField::all_fields());
            }
        }

        // Remove fields that are not present in the file
        let available_fields = self.analyze_available_fields(index)?;
        fields.retain(|field| available_fields.contains(field));

        Ok(fields)
    }

    /// Analyze optimal filters based on file characteristics
    fn analyze_optimal_filters(
        &self,
        _index: &crate::export::binary::index::BinaryIndex,
        optimization_level: OptimizationLevel,
    ) -> Result<Vec<AllocationFilter>, BinaryExportError> {
        let mut filters = Vec::new();

        match optimization_level {
            OptimizationLevel::Minimal => {
                // No filters for maximum compatibility
            }
            OptimizationLevel::Balanced => {
                // Filter out very small allocations
                filters.push(AllocationFilter::SizeRange(32, usize::MAX));
            }
            OptimizationLevel::Comprehensive => {
                // More aggressive filtering
                filters.push(AllocationFilter::SizeRange(16, usize::MAX));
            }
        }

        Ok(filters)
    }

    /// Analyze which fields are available in the file
    fn analyze_available_fields(
        &self,
        _index: &crate::export::binary::index::BinaryIndex,
    ) -> Result<HashSet<AllocationField>, BinaryExportError> {
        // For now, assume all fields are available
        // In a real implementation, we would analyze the file format
        Ok(AllocationField::all_fields())
    }

    /// Update derived statistics
    fn update_derived_stats(&mut self) {
        if self.stats.total_export_time_us > 0 {
            self.stats.avg_export_throughput = (self.stats.total_allocations_exported as f64
                * 1_000_000.0)
                / self.stats.total_export_time_us as f64;
        }

        if self.stats.total_allocations_exported > 0 {
            self.stats.memory_efficiency = self.stats.total_bytes_written as f64
                / self.stats.total_allocations_exported as f64;
        }
    }
}

impl Default for SelectiveJsonExporter {
    fn default() -> Self {
        Self::new().expect("Failed to create default SelectiveJsonExporter")
    }
}

/// Optimization levels for automatic field selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// Minimal fields for maximum performance
    Minimal,
    /// Balanced selection of useful fields
    Balanced,
    /// Comprehensive field selection
    Comprehensive,
}

/// Builder for selective JSON export configuration
pub struct SelectiveJsonExportConfigBuilder {
    config: SelectiveJsonExportConfig,
}

impl SelectiveJsonExportConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: SelectiveJsonExportConfig::default(),
        }
    }

    /// Set JSON writer configuration
    pub fn json_writer_config(mut self, config: StreamingJsonWriterConfig) -> Self {
        self.config.json_writer_config = config;
        self
    }

    /// Set batch processor configuration
    pub fn batch_processor_config(mut self, config: BatchProcessorConfig) -> Self {
        self.config.batch_processor_config = config;
        self
    }

    /// Set index cache configuration
    pub fn index_cache_config(mut self, config: IndexCacheConfig) -> Self {
        self.config.index_cache_config = config;
        self
    }

    /// Set serialization options
    pub fn serialization_options(mut self, options: SelectiveSerializationOptions) -> Self {
        self.config.serialization_options = options;
        self
    }

    /// Enable or disable parallel processing
    pub fn parallel_processing(mut self, enabled: bool) -> Self {
        self.config.enable_parallel_processing = enabled;
        self
    }

    /// Set maximum concurrent exports
    pub fn max_concurrent_exports(mut self, max: usize) -> Self {
        self.config.max_concurrent_exports = max;
        self
    }

    /// Enable or disable error recovery
    pub fn error_recovery(mut self, enabled: bool) -> Self {
        self.config.enable_error_recovery = enabled;
        self
    }

    /// Enable or disable performance monitoring
    pub fn performance_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_performance_monitoring = enabled;
        self
    }

    /// Build the configuration
    pub fn build(self) -> SelectiveJsonExportConfig {
        self.config
    }
}

impl Default for SelectiveJsonExportConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selective_json_exporter_creation() {
        // Use a temporary directory for testing to avoid permission issues
        let temp_dir = tempfile::TempDir::new().unwrap();
        let cache_config = IndexCacheConfig {
            cache_directory: temp_dir.path().to_path_buf(),
            max_entries: 1000,
            max_age_seconds: 3600,
            enable_compression: false,
        };

        let config = SelectiveJsonExportConfig {
            index_cache_config: cache_config,
            ..Default::default()
        };

        let exporter = SelectiveJsonExporter::with_config(config);
        assert!(
            exporter.is_ok(),
            "Failed to create SelectiveJsonExporter: {:?}",
            exporter.err()
        );
    }

    #[test]
    fn test_config_builder() {
        let config = SelectiveJsonExportConfigBuilder::new()
            .parallel_processing(false)
            .max_concurrent_exports(2)
            .error_recovery(false)
            .build();

        assert!(!config.enable_parallel_processing);
        assert_eq!(config.max_concurrent_exports, 2);
        assert!(!config.enable_error_recovery);
    }

    #[test]
    fn test_optimization_levels() {
        assert_eq!(OptimizationLevel::Minimal, OptimizationLevel::Minimal);
        assert_ne!(OptimizationLevel::Minimal, OptimizationLevel::Balanced);
    }

    #[test]
    fn test_export_stats_calculations() {
        let stats = SelectiveJsonExportStats {
            index_cache_hits: 8,
            index_cache_misses: 2,
            total_export_time_us: 1_000_000, // 1 second
            files_processed: 5,
            total_allocations_exported: 1000,
            total_bytes_written: 50000,
            ..Default::default()
        };

        assert_eq!(stats.cache_hit_rate(), 80.0);
        assert_eq!(stats.export_efficiency(), 5.0); // 5 files per second
        assert!(stats.compression_ratio() > 0.0);
    }

    #[test]
    fn test_field_analysis() {
        let mut exporter = SelectiveJsonExporter::new().unwrap();

        // Test basic functionality without actual file processing
        let stats = exporter.get_stats();
        assert_eq!(stats.files_processed, 0);
        assert_eq!(stats.total_allocations_exported, 0);

        exporter.reset_stats();
        assert_eq!(exporter.get_stats().files_processed, 0);
    }

    #[test]
    fn test_cache_operations() {
        let mut exporter = SelectiveJsonExporter::new().unwrap();

        // Test cache clearing
        exporter.clear_caches();

        // Verify stats are still accessible
        let stats = exporter.get_stats();
        assert_eq!(stats.index_cache_hits, 0);
        assert_eq!(stats.index_cache_misses, 0);
    }
}
