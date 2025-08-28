//! Binary data writer for efficient allocation record serialization

use crate::core::types::AllocationInfo;
use crate::export::binary::config::BinaryExportConfig;
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::format::{
    AdvancedMetricsHeader, BinaryExportMode, FileHeader, MetricsBitmapFlags, ALLOCATION_RECORD_TYPE,
};
use crate::export::binary::serializable::BinarySerializable;
use crate::export::binary::string_table::{StringTable, StringTableBuilder};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Binary writer for allocation records using buffered I/O
pub struct BinaryWriter {
    writer: BufWriter<File>,
    config: BinaryExportConfig,
    string_table: Option<StringTable>,
}

impl BinaryWriter {
    /// Create new binary writer for the specified file path with default config
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BinaryExportError> {
        Self::new_with_config(path, &BinaryExportConfig::default())
    }

    /// Create new binary writer with custom configuration
    pub fn new_with_config<P: AsRef<Path>>(
        path: P,
        config: &BinaryExportConfig,
    ) -> Result<Self, BinaryExportError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        Ok(Self {
            writer,
            config: config.clone(),
            string_table: None,
        })
    }

    /// Build string table from allocation data for optimization
    pub fn build_string_table(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        if !self.config.string_table_optimization {
            return Ok(()); // String table optimization disabled
        }

        // Use frequency threshold based on data size
        let min_frequency = if allocations.len() > 1000 { 3 } else { 2 };
        let mut builder = StringTableBuilder::new(min_frequency);

        // Collect string frequencies from all allocations
        for alloc in allocations {
            // Record frequently repeated strings
            if let Some(ref type_name) = alloc.type_name {
                builder.record_string(type_name);
            }
            if let Some(ref var_name) = alloc.var_name {
                builder.record_string(var_name);
            }
            if let Some(ref scope_name) = alloc.scope_name {
                builder.record_string(scope_name);
            }
            builder.record_string(&alloc.thread_id);

            // Record stack trace strings (function names are often repeated)
            if let Some(ref stack_trace) = alloc.stack_trace {
                for frame in stack_trace {
                    builder.record_string(frame);
                }
            }
        }

        let table = builder.build()?;
        let stats = table.compression_stats();

        // Only use string table if it provides meaningful compression
        if stats.space_saved() > 0 && table.len() > 0 {
            tracing::debug!(
                "String table built: {} strings, {:.1}% space savings",
                table.len(),
                stats.space_saved_percent()
            );
            self.string_table = Some(table);
        }

        Ok(())
    }

    /// Write file header with allocation count and optional string table (legacy compatibility)
    pub fn write_header(&mut self, count: u32) -> Result<(), BinaryExportError> {
        let header = FileHeader::new_legacy(count);
        let header_bytes = header.to_bytes();

        self.writer.write_all(&header_bytes)?;

        self.write_string_table_if_present()
    }

    /// Write enhanced file header with export mode and allocation counts
    pub fn write_enhanced_header(
        &mut self,
        total_count: u32,
        export_mode: BinaryExportMode,
        user_count: u16,
        system_count: u16,
    ) -> Result<(), BinaryExportError> {
        let header = FileHeader::new(total_count, export_mode, user_count, system_count);
        let header_bytes = header.to_bytes();

        self.writer.write_all(&header_bytes)?;

        self.write_string_table_if_present()
    }

    /// Write string table if present (helper method)
    fn write_string_table_if_present(&mut self) -> Result<(), BinaryExportError> {
        // Write string table if present
        if let Some(ref string_table) = self.string_table {
            // Write string table marker and size
            self.writer.write_all(b"STBL")?; // String table marker
            let table_size = string_table.serialized_size() as u32;
            self.writer.write_all(&table_size.to_le_bytes())?;

            // Write the string table
            string_table.write_binary(&mut self.writer)?;
        } else {
            // Write empty string table marker
            self.writer.write_all(b"NONE")?; // No string table
            self.writer.write_all(&0u32.to_le_bytes())?; // Size 0
        }

        Ok(())
    }

    /// Write single allocation record in TLV format
    pub fn write_allocation(&mut self, alloc: &AllocationInfo) -> Result<(), BinaryExportError> {
        // Calculate value size (excluding Type and Length fields)
        let value_size = self.calculate_value_size(alloc);

        // Write Type (1 byte)
        self.writer.write_all(&[ALLOCATION_RECORD_TYPE])?;

        // Write Length (4 bytes, Little Endian) - only the Value part
        self.writer.write_all(&(value_size as u32).to_le_bytes())?;

        // Write Value: basic fields (ptr, size, timestamps)
        self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?;
        self.writer.write_all(&(alloc.size as u64).to_le_bytes())?;
        self.writer
            .write_all(&alloc.timestamp_alloc.to_le_bytes())?;

        // Write optional timestamp_dealloc
        match alloc.timestamp_dealloc {
            Some(ts) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&ts.to_le_bytes())?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }

        // Write string fields
        self.write_optional_string(&alloc.var_name)?;
        self.write_optional_string(&alloc.type_name)?;
        self.write_optional_string(&alloc.scope_name)?;
        self.write_string(&alloc.thread_id)?;

        // Write stack trace
        self.write_optional_string_vec(&alloc.stack_trace)?;

        // Write numeric fields
        self.writer
            .write_all(&(alloc.borrow_count as u32).to_le_bytes())?;
        self.writer
            .write_all(&(alloc.is_leaked as u8).to_le_bytes())?;

        // Write optional lifetime_ms
        match alloc.lifetime_ms {
            Some(ms) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&ms.to_le_bytes())?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }

        // Write improve.md extensions: borrow_info
        match &alloc.borrow_info {
            Some(borrow_info) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&(borrow_info.immutable_borrows as u32).to_le_bytes())?;
                self.writer.write_all(&(borrow_info.mutable_borrows as u32).to_le_bytes())?;
                self.writer.write_all(&(borrow_info.max_concurrent_borrows as u32).to_le_bytes())?;
                match borrow_info.last_borrow_timestamp {
                    Some(ts) => {
                        self.writer.write_all(&1u8.to_le_bytes())?; // has timestamp
                        self.writer.write_all(&ts.to_le_bytes())?;
                    }
                    None => {
                        self.writer.write_all(&0u8.to_le_bytes())?; // no timestamp
                    }
                }
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no borrow_info
            }
        }

        // Write improve.md extensions: clone_info
        match &alloc.clone_info {
            Some(clone_info) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.writer.write_all(&(clone_info.clone_count as u32).to_le_bytes())?;
                self.writer.write_all(&(clone_info.is_clone as u8).to_le_bytes())?;
                match clone_info.original_ptr {
                    Some(ptr) => {
                        self.writer.write_all(&1u8.to_le_bytes())?; // has original_ptr
                        self.writer.write_all(&(ptr as u64).to_le_bytes())?;
                    }
                    None => {
                        self.writer.write_all(&0u8.to_le_bytes())?; // no original_ptr
                    }
                }
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no clone_info
            }
        }

        // Write improve.md extensions: ownership_history_available
        self.writer.write_all(&(alloc.ownership_history_available as u8).to_le_bytes())?;

        // Write complex fields using binary serialization
        self.write_optional_binary_field(&alloc.smart_pointer_info)?;
        self.write_optional_binary_field(&alloc.memory_layout)?;
        self.write_optional_json_field(&alloc.generic_info)?;
        self.write_optional_json_field(&alloc.dynamic_type_info)?;
        self.write_optional_json_field(&alloc.runtime_state)?;
        self.write_optional_json_field(&alloc.stack_allocation)?;
        self.write_optional_json_field(&alloc.temporary_object)?;
        self.write_optional_json_field(&alloc.fragmentation_analysis)?;
        self.write_optional_json_field(&alloc.generic_instantiation)?;
        self.write_optional_json_field(&alloc.type_relationships)?;
        self.write_optional_json_field(&alloc.type_usage)?;
        self.write_optional_json_field(&alloc.function_call_tracking)?;
        self.write_optional_json_field(&alloc.lifecycle_tracking)?;
        self.write_optional_json_field(&alloc.access_tracking)?;

        Ok(())
    }

    /// Write advanced metrics segment if enabled in config
    pub fn write_advanced_metrics_segment(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        if !self.config.has_advanced_metrics() {
            return Ok(()); // Skip if no advanced metrics enabled
        }

        // Calculate metrics bitmap based on config
        let mut metrics_bitmap = 0u32;

        if self.config.lifecycle_timeline {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::LifecycleAnalysis);
        }
        if self.config.container_analysis {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::ContainerAnalysis);
        }
        if self.config.source_analysis {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::SourceAnalysis);
        }
        if self.config.fragmentation_analysis {
            metrics_bitmap = MetricsBitmapFlags::enable(
                metrics_bitmap,
                MetricsBitmapFlags::FragmentationAnalysis,
            );
        }
        if self.config.thread_context_tracking {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::ThreadContext);
        }
        if self.config.drop_chain_analysis {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::DropChainAnalysis);
        }
        if self.config.zst_analysis {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::ZstAnalysis);
        }
        if self.config.health_scoring {
            metrics_bitmap =
                MetricsBitmapFlags::enable(metrics_bitmap, MetricsBitmapFlags::HealthScoring);
        }
        if self.config.performance_benchmarking {
            metrics_bitmap = MetricsBitmapFlags::enable(
                metrics_bitmap,
                MetricsBitmapFlags::PerformanceBenchmarks,
            );
        }

        // Calculate segment size (header + data)
        let data_size = self.calculate_advanced_metrics_data_size(allocations, metrics_bitmap);
        let segment_size = 16 + data_size; // 16 bytes for AdvancedMetricsHeader

        // Write advanced metrics header
        let header = AdvancedMetricsHeader::new(segment_size as u32, metrics_bitmap);
        let header_bytes = header.to_bytes();
        self.writer.write_all(&header_bytes)?;

        // Write metrics data based on enabled flags
        self.write_metrics_data(allocations, metrics_bitmap)?;

        Ok(())
    }

    /// Calculate size needed for advanced metrics data
    fn calculate_advanced_metrics_data_size(
        &self,
        allocations: &[AllocationInfo],
        metrics_bitmap: u32,
    ) -> usize {
        let mut size = 0;

        // For each enabled metric, calculate its data size
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::LifecycleAnalysis) {
            size += self.calculate_lifecycle_data_size(allocations);
        }
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::ContainerAnalysis) {
            size += self.calculate_container_data_size(allocations);
        }
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::TypeUsageStats) {
            size += self.calculate_type_usage_data_size(allocations);
        }
        // Add other metrics as needed...

        size
    }

    /// Write metrics data based on enabled bitmap flags
    fn write_metrics_data(
        &mut self,
        allocations: &[AllocationInfo],
        metrics_bitmap: u32,
    ) -> Result<(), BinaryExportError> {
        // Write lifecycle analysis data if enabled
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::LifecycleAnalysis) {
            self.write_lifecycle_metrics(allocations)?;
        }

        // Write container analysis data if enabled
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::ContainerAnalysis) {
            self.write_container_metrics(allocations)?;
        }

        // Write type usage statistics if enabled
        if MetricsBitmapFlags::is_enabled(metrics_bitmap, MetricsBitmapFlags::TypeUsageStats) {
            self.write_type_usage_metrics(allocations)?;
        }

        // Add other metrics as they are implemented...

        Ok(())
    }

    /// Write lifecycle analysis metrics
    fn write_lifecycle_metrics(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        // Write count of allocations with lifecycle data
        let lifecycle_count = allocations
            .iter()
            .filter(|a| a.lifetime_ms.is_some())
            .count();

        self.writer
            .write_all(&(lifecycle_count as u32).to_le_bytes())?;

        // Write lifecycle data for each allocation
        for alloc in allocations {
            if let Some(lifetime) = alloc.lifetime_ms {
                self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?; // allocation ID
                self.writer.write_all(&lifetime.to_le_bytes())?; // lifetime in ms

                // Write lifecycle phase information if available
                if let Some(ref lifecycle_tracking) = alloc.lifecycle_tracking {
                    self.writer.write_all(&1u8.to_le_bytes())?; // has lifecycle tracking
                    let json_str = serde_json::to_string(lifecycle_tracking).map_err(|e| {
                        BinaryExportError::CorruptedData(format!(
                            "Lifecycle JSON serialization failed: {e}"
                        ))
                    })?;
                    self.write_string(&json_str)?;
                } else {
                    self.writer.write_all(&0u8.to_le_bytes())?; // no lifecycle tracking
                }
            }
        }

        Ok(())
    }

    /// Write container analysis metrics
    fn write_container_metrics(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        // Write count of allocations with container data
        let container_count = allocations
            .iter()
            .filter(|a| a.memory_layout.is_some())
            .count();

        self.writer
            .write_all(&(container_count as u32).to_le_bytes())?;

        // Write container data for each allocation
        for alloc in allocations {
            if let Some(ref memory_layout) = alloc.memory_layout {
                self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?; // allocation ID

                // Serialize memory layout as JSON for now (can be optimized later)
                let json_str = serde_json::to_string(memory_layout).map_err(|e| {
                    BinaryExportError::CorruptedData(format!(
                        "Memory layout JSON serialization failed: {e}"
                    ))
                })?;
                self.write_string(&json_str)?;
            }
        }

        Ok(())
    }

    /// Write type usage statistics
    fn write_type_usage_metrics(
        &mut self,
        allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        // Write count of allocations with type usage data
        let type_usage_count = allocations
            .iter()
            .filter(|a| a.type_usage.is_some())
            .count();

        self.writer
            .write_all(&(type_usage_count as u32).to_le_bytes())?;

        // Write type usage data for each allocation
        for alloc in allocations {
            if let Some(ref type_usage) = alloc.type_usage {
                self.writer.write_all(&(alloc.ptr as u64).to_le_bytes())?; // allocation ID

                // Serialize type usage as JSON for now (can be optimized later)
                let json_str = serde_json::to_string(type_usage).map_err(|e| {
                    BinaryExportError::CorruptedData(format!(
                        "Type usage JSON serialization failed: {e}"
                    ))
                })?;
                self.write_string(&json_str)?;
            }
        }

        Ok(())
    }

    /// Calculate size for lifecycle data
    fn calculate_lifecycle_data_size(&self, allocations: &[AllocationInfo]) -> usize {
        let mut size = 4; // count field

        for alloc in allocations {
            if alloc.lifetime_ms.is_some() {
                size += 8; // ptr
                size += 8; // lifetime_ms
                size += 1; // lifecycle_tracking flag

                if let Some(ref lifecycle_tracking) = alloc.lifecycle_tracking {
                    if let Ok(json_str) = serde_json::to_string(lifecycle_tracking) {
                        size += 4 + json_str.len(); // string length + content
                    }
                }
            }
        }

        size
    }

    /// Calculate size for container data
    fn calculate_container_data_size(&self, allocations: &[AllocationInfo]) -> usize {
        let mut size = 4; // count field

        for alloc in allocations {
            if let Some(ref memory_layout) = alloc.memory_layout {
                size += 8; // ptr
                if let Ok(json_str) = serde_json::to_string(memory_layout) {
                    size += 4 + json_str.len(); // string length + content
                }
            }
        }

        size
    }

    /// Calculate size for type usage data
    fn calculate_type_usage_data_size(&self, allocations: &[AllocationInfo]) -> usize {
        let mut size = 4; // count field

        for alloc in allocations {
            if let Some(ref type_usage) = alloc.type_usage {
                size += 8; // ptr
                if let Ok(json_str) = serde_json::to_string(type_usage) {
                    size += 4 + json_str.len(); // string length + content
                }
            }
        }

        size
    }

    /// Finish writing and flush all data to disk
    pub fn finish(mut self) -> Result<(), BinaryExportError> {
        self.writer.flush()?;
        Ok(())
    }

    /// Calculate size needed for the Value part of TLV (excluding Type and Length)
    fn calculate_value_size(&self, alloc: &AllocationInfo) -> usize {
        let mut size = 8 + 8 + 8; // ptr + size + timestamp_alloc

        // timestamp_dealloc: 1 byte flag + optional 8 bytes
        size += 1;
        if alloc.timestamp_dealloc.is_some() {
            size += 8;
        }

        // String fields: length (4 bytes) + content
        size += 4; // var_name length
        if let Some(ref name) = alloc.var_name {
            size += name.len();
        }

        size += 4; // type_name length
        if let Some(ref name) = alloc.type_name {
            size += name.len();
        }

        size += 4; // scope_name length
        if let Some(ref name) = alloc.scope_name {
            size += name.len();
        }

        size += 4 + alloc.thread_id.len(); // thread_id

        // Stack trace
        size += 4; // stack_trace count
        if let Some(ref stack_trace) = alloc.stack_trace {
            for frame in stack_trace {
                size += 4 + frame.len(); // length + content for each frame
            }
        }

        // Numeric fields
        size += 4; // borrow_count
        size += 1; // is_leaked

        // lifetime_ms: 1 byte flag + optional 8 bytes
        size += 1;
        if alloc.lifetime_ms.is_some() {
            size += 8;
        }

        // improve.md extensions: borrow_info
        size += 1; // presence flag
        if let Some(ref borrow_info) = alloc.borrow_info {
            size += 4 + 4 + 4; // immutable_borrows + mutable_borrows + max_concurrent_borrows
            size += 1; // timestamp presence flag
            if borrow_info.last_borrow_timestamp.is_some() {
                size += 8; // timestamp
            }
        }

        // improve.md extensions: clone_info
        size += 1; // presence flag
        if let Some(ref clone_info) = alloc.clone_info {
            size += 4 + 1; // clone_count + is_clone
            size += 1; // original_ptr presence flag
            if clone_info.original_ptr.is_some() {
                size += 8; // original_ptr
            }
        }

        // improve.md extensions: ownership_history_available
        size += 1; // boolean flag

        // Binary fields
        size += self.calculate_binary_field_size(&alloc.smart_pointer_info);
        size += self.calculate_binary_field_size(&alloc.memory_layout);
        size += self.calculate_json_field_size(&alloc.generic_info);
        size += self.calculate_json_field_size(&alloc.dynamic_type_info);
        size += self.calculate_json_field_size(&alloc.runtime_state);
        size += self.calculate_json_field_size(&alloc.stack_allocation);
        size += self.calculate_json_field_size(&alloc.temporary_object);
        size += self.calculate_json_field_size(&alloc.fragmentation_analysis);
        size += self.calculate_json_field_size(&alloc.generic_instantiation);
        size += self.calculate_json_field_size(&alloc.type_relationships);
        size += self.calculate_json_field_size(&alloc.type_usage);
        size += self.calculate_json_field_size(&alloc.function_call_tracking);
        size += self.calculate_json_field_size(&alloc.lifecycle_tracking);
        size += self.calculate_json_field_size(&alloc.access_tracking);

        size
    }

    /// Calculate size needed for optional binary field
    fn calculate_binary_field_size<T: BinarySerializable>(&self, field: &Option<T>) -> usize {
        let mut size = 1; // flag byte
        if let Some(value) = field {
            size += value.binary_size();
        }
        size
    }

    /// Calculate size needed for optional JSON field
    fn calculate_json_field_size<T: serde::Serialize>(&self, field: &Option<T>) -> usize {
        let mut size = 1; // flag byte
        if let Some(value) = field {
            // Pre-serialize to get exact size
            if let Ok(json_str) = serde_json::to_string(value) {
                size += 4 + json_str.len(); // length + content
            } else {
                // If serialization fails, just account for the flag
                size = 1;
            }
        }
        size
    }

    /// Write optional string field, using string table if available
    fn write_optional_string(&mut self, opt_str: &Option<String>) -> Result<(), BinaryExportError> {
        match opt_str {
            Some(s) => {
                if let Some(ref string_table) = self.string_table {
                    // Try to find string in table
                    if let Some(index) = self.find_string_index(s) {
                        // Write as string table reference: 0xFFFF followed by compressed index
                        self.writer.write_all(&0xFFFFu32.to_le_bytes())?;
                        string_table.write_compressed_index(&mut self.writer, index)?;
                    } else {
                        // Write as inline string
                        self.write_inline_string(s)?;
                    }
                } else {
                    // No string table, write inline
                    self.write_inline_string(s)?;
                }
            }
            None => {
                // Use 0xFFFFFFFE as None marker to distinguish from empty string (length 0)
                self.writer.write_all(&0xFFFFFFFEu32.to_le_bytes())?;
            }
        }
        Ok(())
    }

    /// Write string field, using string table if available
    fn write_string(&mut self, s: &str) -> Result<(), BinaryExportError> {
        if let Some(ref string_table) = self.string_table {
            // Try to find string in table
            if let Some(index) = self.find_string_index(s) {
                // Write as string table reference: 0xFFFF followed by compressed index
                self.writer.write_all(&0xFFFFu32.to_le_bytes())?;
                string_table.write_compressed_index(&mut self.writer, index)?;
            } else {
                // Write as inline string
                self.write_inline_string(s)?;
            }
        } else {
            // No string table, write inline
            self.write_inline_string(s)?;
        }
        Ok(())
    }

    /// Write string inline with length prefix
    fn write_inline_string(&mut self, s: &str) -> Result<(), BinaryExportError> {
        self.writer.write_all(&(s.len() as u32).to_le_bytes())?;
        self.writer.write_all(s.as_bytes())?;
        Ok(())
    }

    /// Find string index in string table
    fn find_string_index(&self, s: &str) -> Option<u16> {
        // Don't use string table for empty strings
        if s.is_empty() {
            return None;
        }

        if let Some(ref string_table) = self.string_table {
            string_table.get_index(s)
        } else {
            None
        }
    }

    /// Write an optional vector of strings, using string table if available
    fn write_optional_string_vec(
        &mut self,
        vec: &Option<Vec<String>>,
    ) -> Result<(), BinaryExportError> {
        match vec {
            Some(strings) => {
                // Write count
                self.writer
                    .write_all(&(strings.len() as u32).to_le_bytes())?;
                // Write each string using string table optimization
                for string in strings {
                    self.write_string(string)?;
                }
            }
            None => {
                self.writer.write_all(&0u32.to_le_bytes())?; // 0 count indicates None
            }
        }
        Ok(())
    }

    /// Write optional binary field using BinarySerializable trait
    fn write_optional_binary_field<T: BinarySerializable>(
        &mut self,
        field: &Option<T>,
    ) -> Result<(), BinaryExportError> {
        match field {
            Some(value) => {
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                value.write_binary(&mut self.writer)?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }
        Ok(())
    }

    /// Write optional JSON field (serialize to JSON string)
    fn write_optional_json_field<T: serde::Serialize>(
        &mut self,
        field: &Option<T>,
    ) -> Result<(), BinaryExportError> {
        match field {
            Some(value) => {
                let json_str = serde_json::to_string(value).map_err(|e| {
                    BinaryExportError::CorruptedData(format!("JSON serialization failed: {}", e))
                })?;
                self.writer.write_all(&1u8.to_le_bytes())?; // has value
                self.write_string(&json_str)?;
            }
            None => {
                self.writer.write_all(&0u8.to_le_bytes())?; // no value
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_test_allocation() -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("i32".to_string()),
            scope_name: None,
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    #[test]
    fn test_writer_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let writer = BinaryWriter::new(temp_file.path());
        assert!(writer.is_ok());
    }

    #[test]
    fn test_header_writing() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let mut writer = BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");

        let result = writer.write_header(42);
        assert!(result.is_ok());

        writer.finish().expect("Failed to finish writing");

        // Verify file size is at least header size
        let metadata = fs::metadata(temp_file.path()).expect("Failed to create temp file");
        assert!(metadata.len() >= 16);
    }

    #[test]
    fn test_allocation_writing() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let mut writer = BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");

        writer.write_header(1).expect("Failed to write header");

        let alloc = create_test_allocation();
        let result = writer.write_allocation(&alloc);
        assert!(result.is_ok());

        writer.finish().expect("Failed to finish writing");

        // Verify file has content beyond header
        let metadata = fs::metadata(temp_file.path()).expect("Failed to create temp file");
        assert!(metadata.len() > 16);
    }

    #[test]
    fn test_record_size_calculation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let writer = BinaryWriter::new(temp_file.path()).expect("Failed to create temp file");

        let alloc = create_test_allocation();
        let size = writer.calculate_value_size(&alloc);

        // Basic fields: 8 + 8 + 8 = 24 (ptr + size + timestamp_alloc)
        // timestamp_dealloc: 1 byte flag (None) = 1
        // var_name: 4 + 8 = 12 ("test_var")
        // type_name: 4 + 3 = 7 ("i32")
        // scope_name: 4 + 0 = 4 (None)
        // thread_id: 4 + 4 = 8 ("main")
        // stack_trace: 4 + 0 = 4 (None)
        // borrow_count: 4
        // is_leaked: 1
        // lifetime_ms: 1 byte flag (None) = 1
        // borrow_info: 1 byte flag (None) = 1
        // clone_info: 1 byte flag (None) = 1
        // ownership_history_available: 1 byte = 1
        // JSON fields (14 fields * 1 byte flag each): 14
        // Total: 24 + 1 + 12 + 7 + 4 + 8 + 4 + 4 + 1 + 1 + 1 + 1 + 1 + 14 = 83
        assert_eq!(size, 83);
    }

    #[test]
    fn test_advanced_metrics_segment_writing() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config = BinaryExportConfig::debug_comprehensive();
        let mut writer = BinaryWriter::new_with_config(temp_file.path(), &config)
            .expect("Failed to create temp file");

        writer.write_header(1).expect("Failed to write header");

        let mut alloc = create_test_allocation();
        alloc.lifetime_ms = Some(1500); // Add some lifecycle data

        writer
            .write_allocation(&alloc)
            .expect("Failed to write allocation");
        writer
            .write_advanced_metrics_segment(&[alloc])
            .expect("Failed to write metrics segment");
        writer.finish().expect("Failed to finish writing");

        // Verify file has content beyond basic allocation data
        let metadata = std::fs::metadata(temp_file.path()).expect("Failed to create temp file");
        assert!(metadata.len() > 100); // Should be larger with advanced metrics
    }

    #[test]
    fn test_advanced_metrics_disabled() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config = BinaryExportConfig::minimal(); // No advanced metrics
        let mut writer = BinaryWriter::new_with_config(temp_file.path(), &config)
            .expect("Failed to create temp file");

        writer.write_header(1).expect("Failed to write header");
        let alloc = create_test_allocation();
        writer
            .write_allocation(&alloc)
            .expect("Failed to write allocation");

        // Should not write advanced metrics segment
        writer
            .write_advanced_metrics_segment(&[alloc])
            .expect("Failed to write metrics segment");
        writer.finish().expect("Failed to finish writing");

        // File should be smaller without advanced metrics
        let metadata = std::fs::metadata(temp_file.path()).expect("Failed to create temp file");
        assert!(metadata.len() < 150); // Should be smaller without advanced metrics
    }
}
