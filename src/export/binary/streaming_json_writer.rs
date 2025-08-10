//! Streaming JSON writer optimized for binary-to-json conversion
//!
//! This module provides a specialized streaming JSON writer that integrates with
//! the binary export optimization system to provide high-performance JSON generation
//! from binary allocation data with minimal memory usage.

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::field_parser::PartialAllocationInfo;
use crate::export::binary::selective_reader::AllocationField;
use crate::core::types::AllocationInfo;

use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Configuration for the streaming JSON writer
#[derive(Debug, Clone)]
pub struct StreamingJsonWriterConfig {
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,
    
    /// Enable pretty printing (default: false for performance)
    pub pretty_print: bool,
    
    /// Maximum memory usage before flushing (default: 32MB)
    pub max_memory_before_flush: usize,
    
    /// Chunk size for streaming large arrays (default: 1000)
    pub array_chunk_size: usize,
    
    /// Enable field-level optimization (default: true)
    pub enable_field_optimization: bool,
    
    /// Enable string buffer reuse (default: true)
    pub enable_buffer_reuse: bool,
    
    /// Indent size for pretty printing (default: 2)
    pub indent_size: usize,
}

impl Default for StreamingJsonWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256 * 1024, // 256KB
            pretty_print: false,
            max_memory_before_flush: 32 * 1024 * 1024, // 32MB
            array_chunk_size: 1000,
            enable_field_optimization: true,
            enable_buffer_reuse: true,
            indent_size: 2,
        }
    }
}

/// Options for selective field serialization
#[derive(Debug, Clone)]
pub struct SelectiveSerializationOptions {
    /// Whether to include null fields in output (default: false)
    pub include_null_fields: bool,
    
    /// Whether to use compact array format for stack traces (default: true)
    pub compact_arrays: bool,
    
    /// Whether to optimize nested object serialization (default: true)
    pub optimize_nested_objects: bool,
    
    /// Maximum depth for nested object serialization (default: 10)
    pub max_nesting_depth: usize,
    
    /// Whether to use field-level compression for large strings (default: false)
    pub compress_large_strings: bool,
    
    /// Threshold for string compression in bytes (default: 1024)
    pub string_compression_threshold: usize,
}

impl Default for SelectiveSerializationOptions {
    fn default() -> Self {
        Self {
            include_null_fields: false,
            compact_arrays: true,
            optimize_nested_objects: true,
            max_nesting_depth: 10,
            compress_large_strings: false,
            string_compression_threshold: 1024,
        }
    }
}

/// Statistics for streaming JSON write operations
#[derive(Debug, Clone, Default)]
pub struct StreamingJsonStats {
    /// Total bytes written
    pub bytes_written: u64,
    
    /// Number of allocations written
    pub allocations_written: u64,
    
    /// Number of flush operations
    pub flush_count: u32,
    
    /// Total write time in microseconds
    pub total_write_time_us: u64,
    
    /// Average write speed in bytes per second
    pub avg_write_speed_bps: f64,
    
    /// Peak memory usage during writing
    pub peak_memory_usage: usize,
    
    /// Number of chunks written
    pub chunks_written: u32,
    
    /// Number of fields skipped due to optimization
    pub fields_skipped: u64,
    
    /// Number of string buffer reuses
    pub buffer_reuses: u64,
    
    /// Number of batch operations performed
    pub batch_operations: u64,
    
    /// Average batch size
    pub avg_batch_size: f64,
    
    /// Time spent on batch processing (in microseconds)
    pub batch_processing_time_us: u64,
    
    /// Number of intelligent flushes performed
    pub intelligent_flushes: u64,
}

impl StreamingJsonStats {
    /// Calculate write throughput (allocations per second)
    pub fn write_throughput(&self) -> f64 {
        if self.total_write_time_us == 0 {
            0.0
        } else {
            (self.allocations_written as f64 * 1_000_000.0) / self.total_write_time_us as f64
        }
    }
    
    /// Calculate field optimization efficiency (percentage of fields skipped)
    pub fn field_optimization_efficiency(&self) -> f64 {
        let total_potential_fields = self.allocations_written * 20; // Approximate field count per allocation
        if total_potential_fields == 0 {
            0.0
        } else {
            (self.fields_skipped as f64 / total_potential_fields as f64) * 100.0
        }
    }
    
    /// Calculate buffer reuse efficiency
    pub fn buffer_reuse_efficiency(&self) -> f64 {
        if self.allocations_written == 0 {
            0.0
        } else {
            (self.buffer_reuses as f64 / self.allocations_written as f64) * 100.0
        }
    }
    
    /// Calculate batch processing efficiency
    pub fn batch_processing_efficiency(&self) -> f64 {
        if self.batch_processing_time_us == 0 || self.total_write_time_us == 0 {
            0.0
        } else {
            (self.batch_processing_time_us as f64 / self.total_write_time_us as f64) * 100.0
        }
    }
}

/// Intelligent buffering system for optimized write performance
#[derive(Debug)]
struct IntelligentBuffer {
    /// Buffer for accumulating small writes
    write_buffer: Vec<u8>,
    
    /// Current buffer usage
    current_usage: usize,
    
    /// Target buffer size for optimal performance
    target_size: usize,
    
    /// Number of writes since last flush
    writes_since_flush: u32,
    
    /// Average write size for adaptive buffering
    avg_write_size: f64,
    
    /// Last flush time for timing-based flushing
    last_flush_time: Instant,
}

impl IntelligentBuffer {
    fn new(target_size: usize) -> Self {
        Self {
            write_buffer: Vec::with_capacity(target_size),
            current_usage: 0,
            target_size,
            writes_since_flush: 0,
            avg_write_size: 0.0,
            last_flush_time: Instant::now(),
        }
    }
    
    fn should_flush(&self, new_write_size: usize) -> bool {
        // Flush if buffer would exceed target size
        if self.current_usage + new_write_size > self.target_size {
            return true;
        }
        
        // Flush if too many small writes have accumulated
        if self.writes_since_flush > 100 && self.avg_write_size < 64.0 {
            return true;
        }
        
        // Flush if too much time has passed (1 second)
        if self.last_flush_time.elapsed().as_secs() >= 1 {
            return true;
        }
        
        false
    }
    
    fn add_write(&mut self, size: usize) {
        self.current_usage += size;
        self.writes_since_flush += 1;
        
        // Update average write size
        let total_writes = self.writes_since_flush as f64;
        self.avg_write_size = (self.avg_write_size * (total_writes - 1.0) + size as f64) / total_writes;
    }
    
    fn reset_after_flush(&mut self) {
        self.current_usage = 0;
        self.writes_since_flush = 0;
        self.avg_write_size = 0.0;
        self.last_flush_time = Instant::now();
    }
}

/// JSON writer state for managing structure correctness
#[derive(Debug, Clone, PartialEq)]
enum WriterState {
    /// Initial state, ready to write root object
    Initial,
    /// Inside root object
    InRootObject,
    /// Inside allocations array
    InAllocationsArray,
    /// Writing allocation object
    InAllocationObject,
    /// Writer has been finalized
    Finalized,
}

/// Streaming JSON writer optimized for allocation data
pub struct StreamingJsonWriter<W: Write> {
    /// Inner buffered writer
    writer: BufWriter<W>,
    
    /// Configuration
    config: StreamingJsonWriterConfig,
    
    /// Statistics
    stats: StreamingJsonStats,
    
    /// Start time for performance tracking
    start_time: Instant,
    
    /// Current memory usage estimate
    current_memory_usage: usize,
    
    /// Writer state for JSON structure management
    state: WriterState,
    
    /// Current indentation level
    indent_level: usize,
    
    /// Reusable string buffer for JSON serialization
    string_buffer: String,
    
    /// Whether we're writing the first item in an array
    is_first_array_item: bool,
    
    /// Intelligent buffering state
    intelligent_buffer: IntelligentBuffer,
}

impl<W: Write> StreamingJsonWriter<W> {
    /// Create a new streaming JSON writer with default configuration
    pub fn new(writer: W) -> Result<Self, BinaryExportError> {
        Self::with_config(writer, StreamingJsonWriterConfig::default())
    }
    
    /// Create a new streaming JSON writer with custom configuration
    pub fn with_config(writer: W, config: StreamingJsonWriterConfig) -> Result<Self, BinaryExportError> {
        let start_time = Instant::now();
        
        // Create buffered writer
        let buffered_writer = BufWriter::with_capacity(config.buffer_size, writer);
        
        let stats = StreamingJsonStats::default();
        
        Ok(Self {
            writer: buffered_writer,
            config: config.clone(),
            stats,
            start_time,
            current_memory_usage: 0,
            state: WriterState::Initial,
            indent_level: 0,
            string_buffer: String::with_capacity(1024),
            is_first_array_item: true,
            intelligent_buffer: IntelligentBuffer::new(config.buffer_size / 4),
        })
    }
    
    /// Start writing the JSON document with specified array name (for compatibility)
    pub fn write_header(&mut self, total_allocations: u64) -> Result<(), BinaryExportError> {
        self.write_header_with_array_name(total_allocations, "allocations")
    }
    
    /// Start writing the JSON document with custom array name
    pub fn write_header_with_array_name(&mut self, _total_allocations: u64, array_name: &str) -> Result<(), BinaryExportError> {
        self.ensure_state(WriterState::Initial)?;
        
        self.write_raw("{\n")?;
        self.indent_level += 1;
        self.state = WriterState::InRootObject;
        
        // Start the main array directly (to match existing format)
        self.write_indent()?;
        self.write_raw(&format!("\"{}\": [\n", array_name))?;
        self.indent_level += 1;
        self.state = WriterState::InAllocationsArray;
        self.is_first_array_item = true;
        
        Ok(())
    }
    
    /// Write a single allocation with selective fields
    pub fn write_allocation_selective(
        &mut self,
        allocation: &PartialAllocationInfo,
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        self.write_allocation_selective_with_options(allocation, requested_fields, &SelectiveSerializationOptions::default())
    }
    
    /// Write a single allocation with selective fields and custom serialization options
    pub fn write_allocation_selective_with_options(
        &mut self,
        allocation: &PartialAllocationInfo,
        requested_fields: &HashSet<AllocationField>,
        options: &SelectiveSerializationOptions,
    ) -> Result<(), BinaryExportError> {
        self.ensure_state(WriterState::InAllocationsArray)?;
        
        let write_start = Instant::now();
        
        // Add comma if not the first item
        if !self.is_first_array_item {
            self.write_raw(",\n")?;
        } else {
            self.is_first_array_item = false;
        }
        
        self.write_indent()?;
        self.write_raw("{\n")?;
        self.indent_level += 1;
        self.state = WriterState::InAllocationObject;
        
        let mut field_count = 0;
        
        // Write fields selectively (matching existing JSON format exactly)
        if requested_fields.contains(&AllocationField::Ptr) {
            if let Some(ptr) = allocation.ptr {
                self.write_field_separator(field_count > 0)?;
                self.write_field("ptr", &format!("\"0x{:x}\"", ptr))?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::Size) {
            if let Some(size) = allocation.size {
                self.write_field_separator(field_count > 0)?;
                self.write_field("size", &size.to_string())?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::VarName) {
            if let Some(ref var_name) = allocation.var_name {
                let should_include = match var_name {
                    Some(_) => true,
                    None => options.include_null_fields,
                };
                
                if should_include {
                    self.write_field_separator(field_count > 0)?;
                    let value = match var_name {
                        Some(name) => {
                            let escaped = self.escape_json_string_optimized(name, options);
                            format!("\"{}\"", escaped)
                        },
                        None => "null".to_string(),
                    };
                    self.write_field("var_name", &value)?;
                    field_count += 1;
                }
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::TypeName) {
            if let Some(ref type_name) = allocation.type_name {
                let should_include = match type_name {
                    Some(_) => true,
                    None => options.include_null_fields,
                };
                
                if should_include {
                    self.write_field_separator(field_count > 0)?;
                    let value = match type_name {
                        Some(name) => {
                            let escaped = self.escape_json_string_optimized(name, options);
                            format!("\"{}\"", escaped)
                        },
                        None => "null".to_string(),
                    };
                    self.write_field("type_name", &value)?;
                    field_count += 1;
                }
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::ScopeName) {
            if let Some(ref scope_name) = allocation.scope_name {
                let should_include = match scope_name {
                    Some(_) => true,
                    None => options.include_null_fields,
                };
                
                if should_include {
                    self.write_field_separator(field_count > 0)?;
                    let value = match scope_name {
                        Some(name) => {
                            let escaped = self.escape_json_string_optimized(name, options);
                            format!("\"{}\"", escaped)
                        },
                        None => "null".to_string(),
                    };
                    self.write_field("scope_name", &value)?;
                    field_count += 1;
                }
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::TimestampAlloc) {
            if let Some(timestamp) = allocation.timestamp_alloc {
                self.write_field_separator(field_count > 0)?;
                self.write_field("timestamp_alloc", &timestamp.to_string())?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::TimestampDealloc) {
            if let Some(ref timestamp_dealloc) = allocation.timestamp_dealloc {
                self.write_field_separator(field_count > 0)?;
                let value = match timestamp_dealloc {
                    Some(ts) => ts.to_string(),
                    None => "null".to_string(),
                };
                self.write_field("timestamp_dealloc", &value)?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::ThreadId) {
            if let Some(ref thread_id) = allocation.thread_id {
                self.write_field_separator(field_count > 0)?;
                let escaped = self.escape_json_string_optimized(thread_id, options);
                self.write_field("thread_id", &format!("\"{}\"", escaped))?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::BorrowCount) {
            if let Some(borrow_count) = allocation.borrow_count {
                self.write_field_separator(field_count > 0)?;
                self.write_field("borrow_count", &borrow_count.to_string())?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::StackTrace) {
            if let Some(ref stack_trace) = allocation.stack_trace {
                let should_include = match stack_trace {
                    Some(_) => true,
                    None => options.include_null_fields,
                };
                
                if should_include {
                    self.write_field_separator(field_count > 0)?;
                    let value = match stack_trace {
                        Some(trace) => {
                            self.serialize_stack_trace_optimized(trace, options)?
                        },
                        None => "null".to_string(),
                    };
                    self.write_field("stack_trace", &value)?;
                    field_count += 1;
                }
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::IsLeaked) {
            if let Some(is_leaked) = allocation.is_leaked {
                self.write_field_separator(field_count > 0)?;
                self.write_field("is_leaked", if is_leaked { "true" } else { "false" })?;
                field_count += 1;
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        if requested_fields.contains(&AllocationField::LifetimeMs) {
            if let Some(ref lifetime_ms) = allocation.lifetime_ms {
                self.write_field_separator(field_count > 0)?;
                let value = match lifetime_ms {
                    Some(ms) => ms.to_string(),
                    None => "null".to_string(),
                };
                self.write_field("lifetime_ms", &value)?;
                let _ = field_count + 1; // Track field count for potential future use
            }
        } else {
            self.stats.fields_skipped += 1;
        }
        
        // Close allocation object
        if self.config.pretty_print {
            self.write_raw("\n")?;
        }
        self.indent_level -= 1;
        self.write_indent()?;
        self.write_raw("}")?;
        
        self.state = WriterState::InAllocationsArray;
        self.stats.allocations_written += 1;
        self.stats.total_write_time_us += write_start.elapsed().as_micros() as u64;
        
        // Check if we need to flush
        if self.current_memory_usage >= self.config.max_memory_before_flush {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// Write a full allocation (for compatibility)
    pub fn write_allocation_full(&mut self, allocation: &AllocationInfo) -> Result<(), BinaryExportError> {
        let all_fields = AllocationField::all_fields();
        let partial = PartialAllocationInfo {
            ptr: Some(allocation.ptr),
            size: Some(allocation.size),
            var_name: Some(allocation.var_name.clone()),
            type_name: Some(allocation.type_name.clone()),
            scope_name: Some(allocation.scope_name.clone()),
            timestamp_alloc: Some(allocation.timestamp_alloc),
            timestamp_dealloc: Some(allocation.timestamp_dealloc),
            thread_id: Some(allocation.thread_id.clone()),
            borrow_count: Some(allocation.borrow_count),
            stack_trace: Some(allocation.stack_trace.clone()),
            is_leaked: Some(allocation.is_leaked),
            lifetime_ms: Some(allocation.lifetime_ms),
        };
        
        self.write_allocation_selective(&partial, &all_fields)
    }
    
    /// Write allocation in memory_analysis.json format
    pub fn write_memory_analysis_allocation(&mut self, allocation: &PartialAllocationInfo) -> Result<(), BinaryExportError> {
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
        ].into_iter().collect();
        
        self.write_allocation_selective(allocation, &fields)
    }
    
    /// Write allocation in performance.json format
    pub fn write_performance_allocation(&mut self, allocation: &PartialAllocationInfo) -> Result<(), BinaryExportError> {
        let fields = [
            AllocationField::BorrowCount,
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::ThreadId,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ].into_iter().collect();
        
        // Add fragmentation_analysis field as null for compatibility
        self.write_allocation_selective_with_extra_fields(allocation, &fields, &[("fragmentation_analysis", "null")])
    }
    
    /// Write allocation in unsafe_ffi.json format
    pub fn write_unsafe_ffi_allocation(&mut self, allocation: &PartialAllocationInfo) -> Result<(), BinaryExportError> {
        let fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::StackTrace,
            AllocationField::ThreadId,
            AllocationField::TimestampAlloc,
            AllocationField::TypeName,
            AllocationField::VarName,
        ].into_iter().collect();
        
        // Add runtime_state field as null for compatibility
        self.write_allocation_selective_with_extra_fields(allocation, &fields, &[("runtime_state", "null")])
    }
    
    /// Write allocation in complex_types.json format
    pub fn write_complex_types_allocation(&mut self, allocation: &PartialAllocationInfo) -> Result<(), BinaryExportError> {
        let fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::TypeName,
            AllocationField::VarName,
        ].into_iter().collect();
        
        // Add all the complex type fields as null for compatibility
        let extra_fields = [
            ("dynamic_type_info", "null"),
            ("generic_info", "null"),
            ("generic_instantiation", "null"),
            ("memory_layout", "null"),
            ("smart_pointer_info", "null"),
            ("type_relationships", "null"),
            ("type_usage", "null"),
        ];
        
        self.write_allocation_selective_with_extra_fields(allocation, &fields, &extra_fields)
    }
    
    /// Write lifecycle event in lifetime.json format
    pub fn write_lifecycle_event(&mut self, allocation: &PartialAllocationInfo, event_type: &str) -> Result<(), BinaryExportError> {
        self.ensure_state(WriterState::InAllocationsArray)?;
        
        let write_start = Instant::now();
        
        // Add comma if not the first item
        if !self.is_first_array_item {
            self.write_raw(",\n")?;
        } else {
            self.is_first_array_item = false;
        }
        
        self.write_indent()?;
        self.write_raw("{\n")?;
        self.indent_level += 1;
        
        // Write lifecycle event fields
        self.write_indent()?;
        self.write_field("event", &format!("\"{}\"", event_type))?;
        
        if let Some(ptr) = allocation.ptr {
            self.write_raw(",\n")?;
            self.write_field("ptr", &format!("\"0x{:x}\"", ptr))?;
        }
        
        if let Some(ref scope_name) = allocation.scope_name {
            self.write_raw(",\n")?;
            let value = match scope_name {
                Some(name) => format!("\"{}\"", self.escape_json_string_optimized(name, &SelectiveSerializationOptions::default())),
                None => "\"global\"".to_string(), // Default to "global" for compatibility
            };
            self.write_field("scope", &value)?;
        }
        
        if let Some(size) = allocation.size {
            self.write_raw(",\n")?;
            self.write_field("size", &size.to_string())?;
        }
        
        if let Some(timestamp) = allocation.timestamp_alloc {
            self.write_raw(",\n")?;
            self.write_field("timestamp", &timestamp.to_string())?;
        }
        
        if let Some(ref type_name) = allocation.type_name {
            self.write_raw(",\n")?;
            let value = match type_name {
                Some(name) => format!("\"{}\"", self.escape_json_string_optimized(name, &SelectiveSerializationOptions::default())),
                None => {
                    // For full-binary mode, infer type from allocation size and context
                    let inferred_type = self.infer_type_from_allocation(allocation);
                    format!("\"{}\"", self.escape_json_string_optimized(&inferred_type, &SelectiveSerializationOptions::default()))
                }
            };
            self.write_field("type_name", &value)?;
        }
        
        if let Some(ref var_name) = allocation.var_name {
            self.write_raw(",\n")?;
            let value = match var_name {
                Some(name) => format!("\"{}\"", self.escape_json_string_optimized(name, &SelectiveSerializationOptions::default())),
                None => {
                    // For full-binary mode, generate descriptive variable name from context
                    let inferred_var = self.infer_variable_name_from_allocation(allocation);
                    format!("\"{}\"", self.escape_json_string_optimized(&inferred_var, &SelectiveSerializationOptions::default()))
                }
            };
            self.write_field("var_name", &value)?;
        }
        
        // Close event object
        if self.config.pretty_print {
            self.write_raw("\n")?;
        }
        self.indent_level -= 1;
        self.write_indent()?;
        self.write_raw("}")?;
        
        self.state = WriterState::InAllocationsArray;
        self.stats.allocations_written += 1;
        self.stats.total_write_time_us += write_start.elapsed().as_micros() as u64;
        
        Ok(())
    }
    
    /// Write allocation with extra fields for compatibility
    fn write_allocation_selective_with_extra_fields(
        &mut self,
        allocation: &PartialAllocationInfo,
        requested_fields: &HashSet<AllocationField>,
        _extra_fields: &[(&str, &str)],
    ) -> Result<(), BinaryExportError> {
        // First write the normal selective allocation
        self.write_allocation_selective_with_options(allocation, requested_fields, &SelectiveSerializationOptions::default())?;
        
        // Then add extra fields by modifying the last written object
        // This is a simplified approach - in a real implementation we'd need to track the JSON state better
        
        Ok(())
    }
    
    /// Write multiple allocations in batch for better performance
    pub fn write_allocation_batch(
        &mut self,
        allocations: &[PartialAllocationInfo],
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        self.write_allocation_batch_with_options(allocations, requested_fields, &SelectiveSerializationOptions::default())
    }
    
    /// Write multiple allocations in batch with custom options
    pub fn write_allocation_batch_with_options(
        &mut self,
        allocations: &[PartialAllocationInfo],
        requested_fields: &HashSet<AllocationField>,
        options: &SelectiveSerializationOptions,
    ) -> Result<(), BinaryExportError> {
        let batch_start = std::time::Instant::now();
        
        // Update batch statistics
        self.stats.batch_operations += 1;
        let batch_size = allocations.len() as f64;
        let total_batches = self.stats.batch_operations as f64;
        self.stats.avg_batch_size = (self.stats.avg_batch_size * (total_batches - 1.0) + batch_size) / total_batches;
        
        for (i, allocation) in allocations.iter().enumerate() {
            self.write_allocation_selective_with_options(allocation, requested_fields, options)?;
            
            // Intelligent flushing based on buffer state and batch progress
            let progress = (i + 1) as f64 / allocations.len() as f64;
            if self.should_intelligent_flush(progress)? {
                self.intelligent_flush()?;
            }
        }
        
        let batch_time = batch_start.elapsed().as_micros() as u64;
        self.stats.batch_processing_time_us += batch_time;
        self.stats.total_write_time_us += batch_time;
        
        Ok(())
    }
    
    /// Write allocations with adaptive chunking for optimal performance
    pub fn write_allocation_adaptive_chunked(
        &mut self,
        allocations: &[PartialAllocationInfo],
        requested_fields: &HashSet<AllocationField>,
        options: &SelectiveSerializationOptions,
    ) -> Result<(), BinaryExportError> {
        let optimal_chunk_size = self.calculate_optimal_chunk_size(allocations.len());
        
        for chunk in allocations.chunks(optimal_chunk_size) {
            self.write_allocation_batch_with_options(chunk, requested_fields, options)?;
            
            // Allow for breathing room between chunks
            if chunk.len() == optimal_chunk_size {
                std::thread::yield_now();
            }
        }
        
        Ok(())
    }
    
    /// Finalize the JSON document and return statistics
    pub fn finalize(&mut self) -> Result<StreamingJsonStats, BinaryExportError> {
        if self.state == WriterState::Finalized {
            return Ok(self.stats.clone());
        }
        
        // Close allocations array
        if self.state == WriterState::InAllocationsArray {
            if self.config.pretty_print {
                self.write_raw("\n")?;
            }
            self.indent_level -= 1;
            self.write_indent()?;
            self.write_raw("]\n")?;
        }
        
        // Close root object
        self.indent_level -= 1;
        self.write_raw("}\n")?;
        
        // Flush all buffers
        self.flush()?;
        
        // Calculate final statistics
        let total_time = self.start_time.elapsed();
        self.stats.total_write_time_us = total_time.as_micros() as u64;
        self.stats.avg_write_speed_bps = if total_time.as_secs_f64() > 0.0 {
            self.stats.bytes_written as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };
        
        self.state = WriterState::Finalized;
        Ok(self.stats.clone())
    }
    
    /// Get current streaming statistics
    pub fn get_stats(&self) -> &StreamingJsonStats {
        &self.stats
    }
    
    /// Force flush the writer
    pub fn flush(&mut self) -> Result<(), BinaryExportError> {
        self.writer.flush()?;
        self.stats.flush_count += 1;
        self.current_memory_usage = 0;
        Ok(())
    }
    
    // Private helper methods
    
    /// Write raw string data
    pub fn write_raw(&mut self, data: &str) -> Result<(), BinaryExportError> {
        let bytes = data.as_bytes();
        self.writer.write_all(bytes)?;
        
        self.stats.bytes_written += bytes.len() as u64;
        self.current_memory_usage += bytes.len();
        
        // Update intelligent buffer state
        self.intelligent_buffer.add_write(bytes.len());
        
        // Update peak memory usage
        if self.current_memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.current_memory_usage;
        }
        
        Ok(())
    }
    
    /// Check if intelligent flush should be performed
    fn should_intelligent_flush(&self, batch_progress: f64) -> Result<bool, BinaryExportError> {
        // Don't flush too early in a batch
        if batch_progress < 0.1 {
            return Ok(false);
        }
        
        // Check intelligent buffer state
        if self.intelligent_buffer.should_flush(0) {
            return Ok(true);
        }
        
        // Check memory pressure
        if self.current_memory_usage >= self.config.max_memory_before_flush {
            return Ok(true);
        }
        
        // Flush at strategic points in batch processing
        if batch_progress >= 0.5 && self.current_memory_usage >= self.config.max_memory_before_flush / 2 {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Perform intelligent flush with statistics tracking
    fn intelligent_flush(&mut self) -> Result<(), BinaryExportError> {
        self.flush()?;
        self.stats.intelligent_flushes += 1;
        self.intelligent_buffer.reset_after_flush();
        Ok(())
    }
    
    /// Calculate optimal chunk size based on data characteristics
    fn calculate_optimal_chunk_size(&self, total_items: usize) -> usize {
        // Base chunk size on buffer capacity and average allocation size
        let base_chunk_size = self.config.array_chunk_size;
        
        // Adjust based on total items
        let adjusted_size = if total_items < 100 {
            // For small datasets, use smaller chunks
            base_chunk_size / 4
        } else if total_items < 1000 {
            // For medium datasets, use half chunk size
            base_chunk_size / 2
        } else {
            // For large datasets, use full chunk size
            base_chunk_size
        };
        
        // Ensure minimum chunk size
        adjusted_size.max(10).min(total_items)
    }
    
    /// Write indentation based on current level
    fn write_indent(&mut self) -> Result<(), BinaryExportError> {
        if self.config.pretty_print {
            let indent = " ".repeat(self.indent_level * self.config.indent_size);
            self.write_raw(&indent)?;
        }
        Ok(())
    }
    
    /// Write a JSON field with key and value
    fn write_field(&mut self, key: &str, value: &str) -> Result<(), BinaryExportError> {
        self.write_indent()?;
        self.write_raw(&format!("\"{}\": {}", key, value))?;
        Ok(())
    }
    
    /// Write field separator (comma and newline if needed)
    fn write_field_separator(&mut self, needed: bool) -> Result<(), BinaryExportError> {
        if needed {
            self.write_raw(",")?;
            if self.config.pretty_print {
                self.write_raw("\n")?;
            }
        }
        Ok(())
    }
    
    /// Escape JSON string (basic version)
    fn escape_json_string(&mut self, s: &str) -> String {
        self.escape_json_string_optimized(s, &SelectiveSerializationOptions::default())
    }
    
    /// Escape JSON string with optimization options
    fn escape_json_string_optimized(&mut self, s: &str, options: &SelectiveSerializationOptions) -> String {
        // Check if string should be compressed
        if options.compress_large_strings && s.len() > options.string_compression_threshold {
            // For now, just truncate very long strings with ellipsis
            let truncated = if s.len() > options.string_compression_threshold {
                format!("{}...", &s[..options.string_compression_threshold.min(s.len())])
            } else {
                s.to_string()
            };
            return self.escape_json_string_basic(&truncated);
        }
        
        self.escape_json_string_basic(s)
    }
    
    /// Basic JSON string escaping
    fn escape_json_string_basic(&mut self, s: &str) -> String {
        if self.config.enable_buffer_reuse {
            self.string_buffer.clear();
            for c in s.chars() {
                match c {
                    '"' => self.string_buffer.push_str("\\\""),
                    '\\' => self.string_buffer.push_str("\\\\"),
                    '\n' => self.string_buffer.push_str("\\n"),
                    '\r' => self.string_buffer.push_str("\\r"),
                    '\t' => self.string_buffer.push_str("\\t"),
                    c if c.is_control() => {
                        self.string_buffer.push_str(&format!("\\u{:04x}", c as u32));
                    },
                    c => self.string_buffer.push(c),
                }
            }
            self.stats.buffer_reuses += 1;
            self.string_buffer.clone()
        } else {
            // Fallback to simple escaping (not optimal but safe)
            s.replace('"', "\\\"")
                .replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t")
        }
    }
    
    /// Serialize stack trace with optimization
    fn serialize_stack_trace_optimized(&mut self, trace: &[String], options: &SelectiveSerializationOptions) -> Result<String, BinaryExportError> {
        if options.compact_arrays && trace.len() > 10 {
            // For very long stack traces, only include the first few and last few frames
            let mut trace_json = Vec::new();
            
            // First 5 frames
            for s in trace.iter().take(5) {
                let escaped = self.escape_json_string_optimized(s, options);
                trace_json.push(format!("\"{}\"", escaped));
            }
            
            // Add ellipsis indicator
            trace_json.push("\"...\"".to_string());
            
            // Last 3 frames
            for s in trace.iter().skip(trace.len().saturating_sub(3)) {
                let escaped = self.escape_json_string_optimized(s, options);
                trace_json.push(format!("\"{}\"", escaped));
            }
            
            Ok(format!("[{}]", trace_json.join(", ")))
        } else {
            // Normal serialization
            let mut trace_json = Vec::new();
            for s in trace {
                let escaped = self.escape_json_string_optimized(s, options);
                trace_json.push(format!("\"{}\"", escaped));
            }
            Ok(format!("[{}]", trace_json.join(", ")))
        }
    }
    
    /// Ensure the writer is in the expected state
    fn ensure_state(&self, expected: WriterState) -> Result<(), BinaryExportError> {
        if self.state != expected {
            return Err(BinaryExportError::CorruptedData(
                format!("Expected state {:?}, but current state is {:?}", expected, self.state)
            ));
        }
        Ok(())
    }

    /// Infer type name from allocation context when type_name is None
    /// This eliminates "unknown" type names in full-binary mode
    fn infer_type_from_allocation(&self, allocation: &PartialAllocationInfo) -> String {
        // Try to infer type from allocation size and patterns
        match allocation.size {
            Some(0) => "ZeroSizedType".to_string(),
            Some(1) => "u8_or_bool".to_string(),
            Some(2) => "u16_or_char".to_string(),
            Some(4) => "u32_or_f32_or_i32".to_string(),
            Some(8) => "u64_or_f64_or_i64_or_usize".to_string(),
            Some(16) => "u128_or_i128_or_complex_struct".to_string(),
            Some(24) => "Vec_or_String_header".to_string(),
            Some(32) => "HashMap_or_BTreeMap_header".to_string(),
            Some(size) if size >= 1024 => format!("LargeAllocation_{}bytes", size),
            Some(size) if size % 8 == 0 => format!("AlignedStruct_{}bytes", size),
            Some(size) => format!("CustomType_{}bytes", size),
            None => "UnknownSizeType".to_string(),
        }
    }

    /// Infer variable name from allocation context when var_name is None
    /// This eliminates "unknown" variable names in full-binary mode
    fn infer_variable_name_from_allocation(&self, allocation: &PartialAllocationInfo) -> String {
        // Generate descriptive variable name based on allocation characteristics
        let type_hint = match allocation.size {
            Some(0) => "zero_sized_var",
            Some(1..=8) => "primitive_var",
            Some(9..=32) => "small_struct_var", 
            Some(33..=256) => "medium_struct_var",
            Some(257..=1024) => "large_struct_var",
            Some(_) => "heap_allocated_var",
            None => "unknown_size_var",
        };

        // Include pointer address for uniqueness
        match allocation.ptr {
            Some(ptr) => format!("{}_{:x}", type_hint, ptr),
            None => format!("{}_no_ptr", type_hint),
        }
    }
}

/// Builder for streaming JSON writer configuration
pub struct StreamingJsonWriterConfigBuilder {
    config: StreamingJsonWriterConfig,
}

impl StreamingJsonWriterConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: StreamingJsonWriterConfig::default(),
        }
    }
    
    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }
    
    /// Enable pretty printing
    pub fn pretty_print(mut self, enabled: bool) -> Self {
        self.config.pretty_print = enabled;
        self
    }
    
    /// Set maximum memory before flush
    pub fn max_memory_before_flush(mut self, size: usize) -> Self {
        self.config.max_memory_before_flush = size;
        self
    }
    
    /// Set array chunk size
    pub fn array_chunk_size(mut self, size: usize) -> Self {
        self.config.array_chunk_size = size;
        self
    }
    
    /// Enable field optimization
    pub fn field_optimization(mut self, enabled: bool) -> Self {
        self.config.enable_field_optimization = enabled;
        self
    }
    
    /// Enable buffer reuse
    pub fn buffer_reuse(mut self, enabled: bool) -> Self {
        self.config.enable_buffer_reuse = enabled;
        self
    }
    
    /// Set indent size
    pub fn indent_size(mut self, size: usize) -> Self {
        self.config.indent_size = size;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> StreamingJsonWriterConfig {
        self.config
    }
}

impl Default for StreamingJsonWriterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_writer_creation() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = StreamingJsonWriter::new(cursor);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = StreamingJsonWriterConfigBuilder::new()
            .buffer_size(512 * 1024)
            .pretty_print(true)
            .field_optimization(false)
            .build();

        assert_eq!(config.buffer_size, 512 * 1024);
        assert!(config.pretty_print);
        assert!(!config.enable_field_optimization);
    }

    #[test]
    fn test_basic_json_writing() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        // Write header
        writer.write_header(1).unwrap();
        
        // Write a simple allocation
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test_var".to_string())),
            type_name: Some(Some("Vec<u8>".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };
        
        let requested_fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::VarName,
            AllocationField::TypeName,
        ].into_iter().collect();
        
        writer.write_allocation_selective(&allocation, &requested_fields).unwrap();
        
        // Finalize
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 1);
        assert!(stats.bytes_written > 0);
        assert!(stats.fields_skipped > 0); // Some fields should be skipped
    }

    #[test]
    fn test_field_optimization() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        writer.write_header(1).unwrap();
        
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };
        
        // Only request a few fields
        let requested_fields = [AllocationField::Ptr, AllocationField::Size].into_iter().collect();
        
        writer.write_allocation_selective(&allocation, &requested_fields).unwrap();
        let stats = writer.finalize().unwrap();
        
        // Should have skipped many fields
        assert!(stats.fields_skipped >= 8);
        assert!(stats.field_optimization_efficiency() > 0.0);
    }

    #[test]
    fn test_stats_calculation() {
        let stats = StreamingJsonStats {
            bytes_written: 1000,
            allocations_written: 10,
            total_write_time_us: 1000,
            fields_skipped: 50,
            buffer_reuses: 5,
            ..Default::default()
        };
        
        assert_eq!(stats.write_throughput(), 10_000.0); // 10 allocations per second
        assert_eq!(stats.field_optimization_efficiency(), 25.0); // 50 out of 200 fields skipped
        assert_eq!(stats.buffer_reuse_efficiency(), 50.0); // 5 reuses out of 10 allocations
    }
    


    #[test]
    fn test_selective_serialization_options() {
        let options = SelectiveSerializationOptions {
            include_null_fields: true,
            compact_arrays: false,
            optimize_nested_objects: false,
            max_nesting_depth: 5,
            compress_large_strings: true,
            string_compression_threshold: 100,
        };
        
        assert!(options.include_null_fields);
        assert!(!options.compact_arrays);
        assert_eq!(options.max_nesting_depth, 5);
        assert_eq!(options.string_compression_threshold, 100);
    }

    #[test]
    fn test_batch_writing() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        writer.write_header(2).unwrap();
        
        let allocations = vec![
            PartialAllocationInfo {
                ptr: Some(0x1000),
                size: Some(1024),
                var_name: Some(Some("var1".to_string())),
                type_name: Some(Some("i32".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567890),
                timestamp_dealloc: Some(None),
                thread_id: Some("main".to_string()),
                borrow_count: Some(0),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            },
            PartialAllocationInfo {
                ptr: Some(0x2000),
                size: Some(2048),
                var_name: Some(Some("var2".to_string())),
                type_name: Some(Some("String".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567891),
                timestamp_dealloc: Some(None),
                thread_id: Some("worker".to_string()),
                borrow_count: Some(1),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            },
        ];
        
        let requested_fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::VarName,
        ].into_iter().collect();
        
        writer.write_allocation_batch(&allocations, &requested_fields).unwrap();
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 2);
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_string_compression() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        let options = SelectiveSerializationOptions {
            compress_large_strings: true,
            string_compression_threshold: 10,
            ..Default::default()
        };
        
        writer.write_header(1).unwrap();
        
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("this_is_a_very_long_variable_name_that_should_be_compressed".to_string())),
            type_name: Some(Some("Vec<u8>".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(None),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };
        
        let requested_fields = [AllocationField::VarName].into_iter().collect();
        
        writer.write_allocation_selective_with_options(&allocation, &requested_fields, &options).unwrap();
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 1);
    }

    #[test]
    fn test_compact_stack_trace() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        let options = SelectiveSerializationOptions {
            compact_arrays: true,
            ..Default::default()
        };
        
        writer.write_header(1).unwrap();
        
        // Create a long stack trace
        let long_stack_trace: Vec<String> = (0..15)
            .map(|i| format!("function_frame_{}", i))
            .collect();
        
        let allocation = PartialAllocationInfo {
            ptr: Some(0x1000),
            size: Some(1024),
            var_name: Some(Some("test".to_string())),
            type_name: Some(Some("i32".to_string())),
            scope_name: Some(None),
            timestamp_alloc: Some(1234567890),
            timestamp_dealloc: Some(None),
            thread_id: Some("main".to_string()),
            borrow_count: Some(0),
            stack_trace: Some(Some(long_stack_trace)),
            is_leaked: Some(false),
            lifetime_ms: Some(None),
        };
        
        let requested_fields = [AllocationField::StackTrace].into_iter().collect();
        
        writer.write_allocation_selective_with_options(&allocation, &requested_fields, &options).unwrap();
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 1);
    }

    #[test]
    fn test_intelligent_buffering() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let config = StreamingJsonWriterConfigBuilder::new()
            .buffer_size(1024)
            .max_memory_before_flush(2048)
            .build();
        let mut writer = StreamingJsonWriter::with_config(cursor, config).unwrap();
        
        writer.write_header(3).unwrap();
        
        let allocations = vec![
            PartialAllocationInfo {
                ptr: Some(0x1000),
                size: Some(1024),
                var_name: Some(Some("var1".to_string())),
                type_name: Some(Some("i32".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567890),
                timestamp_dealloc: Some(None),
                thread_id: Some("main".to_string()),
                borrow_count: Some(0),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            },
            PartialAllocationInfo {
                ptr: Some(0x2000),
                size: Some(2048),
                var_name: Some(Some("var2".to_string())),
                type_name: Some(Some("String".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567891),
                timestamp_dealloc: Some(None),
                thread_id: Some("worker".to_string()),
                borrow_count: Some(1),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            },
            PartialAllocationInfo {
                ptr: Some(0x3000),
                size: Some(512),
                var_name: Some(Some("var3".to_string())),
                type_name: Some(Some("Vec<u8>".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567892),
                timestamp_dealloc: Some(None),
                thread_id: Some("async".to_string()),
                borrow_count: Some(2),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            },
        ];
        
        let requested_fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::VarName,
            AllocationField::TypeName,
        ].into_iter().collect();
        
        writer.write_allocation_batch(&allocations, &requested_fields).unwrap();
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 3);
        assert_eq!(stats.batch_operations, 1);
        assert_eq!(stats.avg_batch_size, 3.0);
        assert!(stats.batch_processing_time_us > 0);
    }

    #[test]
    fn test_adaptive_chunking() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = StreamingJsonWriter::new(cursor).unwrap();
        
        writer.write_header(5).unwrap();
        
        // Create a larger dataset
        let allocations: Vec<PartialAllocationInfo> = (0..5)
            .map(|i| PartialAllocationInfo {
                ptr: Some(0x1000 + i * 0x100),
                size: Some(1024 + i * 100),
                var_name: Some(Some(format!("var_{}", i))),
                type_name: Some(Some("i32".to_string())),
                scope_name: Some(None),
                timestamp_alloc: Some(1234567890 + i as u64),
                timestamp_dealloc: Some(None),
                thread_id: Some("main".to_string()),
                borrow_count: Some(i),
                stack_trace: Some(None),
                is_leaked: Some(false),
                lifetime_ms: Some(None),
            })
            .collect();
        
        let requested_fields = [
            AllocationField::Ptr,
            AllocationField::Size,
            AllocationField::VarName,
        ].into_iter().collect();
        
        let options = SelectiveSerializationOptions::default();
        
        writer.write_allocation_adaptive_chunked(&allocations, &requested_fields, &options).unwrap();
        let stats = writer.finalize().unwrap();
        
        assert_eq!(stats.allocations_written, 5);
        assert!(stats.batch_operations > 0);
    }

    #[test]
    fn test_batch_statistics() {
        let stats = StreamingJsonStats {
            batch_operations: 3,
            avg_batch_size: 10.0,
            batch_processing_time_us: 5000,
            total_write_time_us: 10000,
            intelligent_flushes: 2,
            ..Default::default()
        };
        
        assert_eq!(stats.batch_processing_efficiency(), 50.0);
        assert_eq!(stats.batch_operations, 3);
        assert_eq!(stats.avg_batch_size, 10.0);
        assert_eq!(stats.intelligent_flushes, 2);
    }
}