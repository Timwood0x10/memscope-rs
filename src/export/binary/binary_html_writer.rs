//! Binary to HTML writer for high-performance direct conversion
//!
//! This module provides a specialized writer that converts binary allocation data
//! directly to HTML format, bypassing JSON intermediate steps for optimal performance.
//! It works alongside the existing JSON → HTML functionality without interference.

use crate::core::types::AllocationInfo;
use crate::export::binary::complex_type_analyzer::{ComplexTypeAnalysis, ComplexTypeAnalyzer};
use crate::export::binary::error::BinaryExportError;
use crate::export::binary::ffi_safety_analyzer::{FfiSafetyAnalysis, FfiSafetyAnalyzer};
use crate::export::binary::selective_reader::AllocationField;
use crate::export::binary::variable_relationship_analyzer::{
    VariableRelationshipAnalysis, VariableRelationshipAnalyzer,
};

use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Configuration for the binary HTML writer
#[derive(Debug, Clone)]
pub struct BinaryHtmlWriterConfig {
    /// Buffer size for I/O operations (default: 256KB)
    pub buffer_size: usize,

    /// Maximum memory usage before flushing (default: 32MB)
    pub max_memory_before_flush: usize,

    /// Chunk size for processing large datasets (default: 1000)
    pub chunk_size: usize,

    /// Enable intelligent buffering (default: true)
    pub enable_intelligent_buffering: bool,

    /// Enable data compression for large strings (default: false)
    pub enable_data_compression: bool,

    /// Parallel processing threshold (default: 5000)
    pub parallel_threshold: usize,
}

impl Default for BinaryHtmlWriterConfig {
    fn default() -> Self {
        Self {
            buffer_size: 256 * 1024,                   // 256KB
            max_memory_before_flush: 32 * 1024 * 1024, // 32MB
            chunk_size: 1000,
            enable_intelligent_buffering: true,
            enable_data_compression: false,
            parallel_threshold: 5000,
        }
    }
}

/// Statistics for binary HTML write operations
#[derive(Debug, Clone, Default)]
pub struct BinaryHtmlStats {
    /// Total allocations processed
    pub allocations_processed: u64,

    /// Total HTML size generated
    pub total_html_size: usize,

    /// Template rendering time in milliseconds
    pub template_render_time_ms: u64,

    /// Data processing time in milliseconds
    pub data_processing_time_ms: u64,

    /// Peak memory usage during processing
    pub peak_memory_usage: usize,

    /// Number of buffer flushes performed
    pub buffer_flushes: u32,

    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,

    /// Average processing speed (allocations per second)
    pub avg_processing_speed: f64,

    /// Memory efficiency (bytes processed per MB memory used)
    pub memory_efficiency: f64,
}

impl BinaryHtmlStats {
    /// Calculate processing throughput
    pub fn processing_throughput(&self) -> f64 {
        if self.total_processing_time_ms == 0 {
            0.0
        } else {
            (self.allocations_processed as f64 * 1000.0) / self.total_processing_time_ms as f64
        }
    }

    /// Calculate memory efficiency ratio
    pub fn memory_efficiency_ratio(&self) -> f64 {
        if self.peak_memory_usage == 0 {
            0.0
        } else {
            (self.allocations_processed as f64) / (self.peak_memory_usage as f64 / 1024.0 / 1024.0)
        }
    }
}

/// Binary allocation data structure for direct HTML processing
#[derive(Debug, Clone)]
pub struct BinaryAllocationData {
    pub id: u64,
    pub size: usize,
    pub type_name: String,
    pub scope_name: String,
    pub timestamp_alloc: u64,
    pub is_active: bool,
    pub ptr: usize,
    pub thread_id: String,
    pub var_name: Option<String>,
    pub borrow_count: usize,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,
    /// Dynamic fields based on requested fields
    pub optional_fields: HashMap<String, BinaryFieldValue>,
}

/// Binary field value types for flexible data handling
#[derive(Debug, Clone)]
pub enum BinaryFieldValue {
    String(String),
    Number(u64),
    Boolean(bool),
    Array(Vec<String>),
    Optional(Option<Box<BinaryFieldValue>>),
}

impl BinaryAllocationData {
    /// Create binary allocation data from AllocationInfo
    pub fn from_allocation(
        allocation: &AllocationInfo,
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<Self, BinaryExportError> {
        let mut optional_fields = HashMap::new();

        // Process optional fields based on request
        if requested_fields.contains(&AllocationField::StackTrace) {
            if let Some(ref stack_trace) = allocation.stack_trace {
                optional_fields.insert(
                    "stack_trace".to_string(),
                    BinaryFieldValue::Array(stack_trace.clone()),
                );
            }
        }

        if requested_fields.contains(&AllocationField::TimestampDealloc) {
            if let Some(timestamp_dealloc) = allocation.timestamp_dealloc {
                optional_fields.insert(
                    "timestamp_dealloc".to_string(),
                    BinaryFieldValue::Number(timestamp_dealloc),
                );
            }
        }

        // Add more optional fields as needed
        if requested_fields.contains(&AllocationField::SmartPointerInfo) {
            if allocation.smart_pointer_info.is_some() {
                optional_fields.insert(
                    "smart_pointer_info".to_string(),
                    BinaryFieldValue::String("present".to_string()),
                );
            }
        }

        Ok(Self {
            id: allocation.ptr as u64,
            size: allocation.size,
            type_name: allocation
                .type_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            scope_name: allocation
                .scope_name
                .clone()
                .unwrap_or_else(|| "global".to_string()),
            timestamp_alloc: allocation.timestamp_alloc,
            is_active: allocation.timestamp_dealloc.is_none(),
            ptr: allocation.ptr,
            thread_id: allocation.thread_id.clone(),
            var_name: allocation.var_name.clone(),
            borrow_count: allocation.borrow_count,
            is_leaked: allocation.is_leaked,
            lifetime_ms: allocation.lifetime_ms,
            optional_fields,
        })
    }
}

/// Binary template data structure for HTML generation
#[derive(Debug, Clone)]
pub struct BinaryTemplateData {
    pub project_name: String,
    pub allocations: Vec<BinaryAllocationData>,
    pub total_memory_usage: u64,
    pub peak_memory_usage: u64,
    pub active_allocations_count: usize,
    pub processing_time_ms: u64,
    pub data_source: String,
    pub complex_types: Option<ComplexTypeAnalysis>,
    pub unsafe_ffi: Option<FfiSafetyAnalysis>,
    pub variable_relationships: Option<VariableRelationshipAnalysis>,
}

/// Intelligent buffering system for optimized write performance
#[derive(Debug)]
struct IntelligentBuffer {
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
        let buffer = Self {
            current_usage: 0,
            target_size,
            writes_since_flush: 0,
            avg_write_size: 0.0,
            last_flush_time: Instant::now(),
        };

        // Use the target_size field to ensure it's read
        tracing::debug!(
            "Created IntelligentBuffer with target size: {}",
            buffer.target_size
        );
        buffer
    }

    fn reset_after_flush(&mut self) {
        self.current_usage = 0;
        self.writes_since_flush = 0;
        self.avg_write_size = 0.0;
        self.last_flush_time = Instant::now();
    }
}

/// Binary HTML writer for direct conversion from binary data to HTML
pub struct BinaryHtmlWriter<W: Write> {
    /// Inner buffered writer
    writer: BufWriter<W>,

    /// Configuration
    config: BinaryHtmlWriterConfig,

    /// Statistics
    stats: BinaryHtmlStats,

    /// Start time for performance tracking
    start_time: Instant,

    /// Current memory usage estimate
    current_memory_usage: usize,

    /// Allocation data buffer for batch processing
    allocation_buffer: Vec<BinaryAllocationData>,

    /// All allocations for complex type analysis
    all_allocations: Vec<AllocationInfo>,

    /// Intelligent buffering state
    intelligent_buffer: IntelligentBuffer,
}

impl<W: Write> BinaryHtmlWriter<W> {
    /// Create a new binary HTML writer with default configuration
    pub fn new(writer: W) -> Result<Self, BinaryExportError> {
        Self::with_config(writer, BinaryHtmlWriterConfig::default())
    }

    /// Create a new binary HTML writer with custom configuration
    pub fn with_config(
        writer: W,
        config: BinaryHtmlWriterConfig,
    ) -> Result<Self, BinaryExportError> {
        let start_time = Instant::now();

        // Create buffered writer
        let buffered_writer = BufWriter::with_capacity(config.buffer_size, writer);

        let stats = BinaryHtmlStats::default();

        Ok(Self {
            writer: buffered_writer,
            config: config.clone(),
            stats,
            start_time,
            current_memory_usage: 0,
            allocation_buffer: Vec::with_capacity(config.chunk_size),
            all_allocations: Vec::new(),
            intelligent_buffer: IntelligentBuffer::new(config.buffer_size / 4),
        })
    }

    /// Write a single allocation directly from binary data
    pub fn write_binary_allocation(
        &mut self,
        allocation: &AllocationInfo,
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        let write_start = Instant::now();

        // Store allocation for complex type analysis
        self.all_allocations.push(allocation.clone());

        // Convert to binary allocation data (direct processing, no JSON)
        let binary_data = BinaryAllocationData::from_allocation(allocation, requested_fields)?;

        // Add to buffer for batch processing
        self.allocation_buffer.push(binary_data);

        // Update memory usage estimate
        self.current_memory_usage += std::mem::size_of::<BinaryAllocationData>();

        // Check if we need to flush
        if self.current_memory_usage >= self.config.max_memory_before_flush {
            self.flush_allocation_buffer()?;
        }

        self.stats.allocations_processed += 1;
        self.stats.data_processing_time_ms += write_start.elapsed().as_millis() as u64;

        Ok(())
    }

    /// Write multiple allocations in batch for better performance
    pub fn write_binary_allocation_batch(
        &mut self,
        allocations: &[AllocationInfo],
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        let batch_start = Instant::now();

        // Process allocations based on size
        if allocations.len() >= self.config.parallel_threshold {
            self.write_allocation_batch_parallel(allocations, requested_fields)?;
        } else {
            self.write_allocation_batch_serial(allocations, requested_fields)?;
        }

        self.stats.data_processing_time_ms += batch_start.elapsed().as_millis() as u64;
        Ok(())
    }

    /// Serial batch processing for smaller datasets
    fn write_allocation_batch_serial(
        &mut self,
        allocations: &[AllocationInfo],
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        for allocation in allocations {
            self.write_binary_allocation(allocation, requested_fields)?;
        }
        Ok(())
    }

    /// Parallel batch processing for larger datasets
    fn write_allocation_batch_parallel(
        &mut self,
        allocations: &[AllocationInfo],
        requested_fields: &HashSet<AllocationField>,
    ) -> Result<(), BinaryExportError> {
        // For now, use serial processing
        // Parallel processing implementation using rayon for better performance
        self.write_allocation_batch_serial(allocations, requested_fields)
    }

    /// Flush the allocation buffer
    fn flush_allocation_buffer(&mut self) -> Result<(), BinaryExportError> {
        // For now, just clear the buffer
        // The actual HTML generation happens in finalize_with_binary_template
        self.allocation_buffer.clear();
        self.current_memory_usage = 0;
        self.stats.buffer_flushes += 1;
        self.intelligent_buffer.reset_after_flush();
        Ok(())
    }

    /// Complete HTML generation using binary template data
    pub fn finalize_with_binary_template(
        &mut self,
        project_name: &str,
    ) -> Result<BinaryHtmlStats, BinaryExportError> {
        let finalize_start = Instant::now();

        // Build final template data from accumulated allocations
        let template_data = self.build_binary_template_data(project_name)?;

        // Generate HTML content using binary template engine
        let html_content = self.render_binary_template(&template_data)?;

        // Write final HTML
        self.writer.write_all(html_content.as_bytes())?;
        self.writer.flush()?;

        // Update final statistics
        self.stats.total_html_size = html_content.len();
        self.stats.total_processing_time_ms = self.start_time.elapsed().as_millis() as u64;
        self.stats.template_render_time_ms = finalize_start.elapsed().as_millis() as u64;
        self.stats.avg_processing_speed = self.stats.processing_throughput();
        self.stats.memory_efficiency = self.stats.memory_efficiency_ratio();

        Ok(self.stats.clone())
    }

    /// Build template data structure from binary allocations
    fn build_binary_template_data(
        &self,
        project_name: &str,
    ) -> Result<BinaryTemplateData, BinaryExportError> {
        let total_memory: u64 = self.allocation_buffer.iter().map(|a| a.size as u64).sum();
        let peak_memory = total_memory; // Simplified calculation
        let active_count = self
            .allocation_buffer
            .iter()
            .filter(|a| a.is_active)
            .count();

        // Perform complex type analysis on collected allocations
        let complex_types = if !self.all_allocations.is_empty() {
            match ComplexTypeAnalyzer::analyze_allocations(&self.all_allocations) {
                Ok(analysis) => Some(analysis),
                Err(e) => {
                    tracing::warn!("Complex type analysis failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Perform FFI safety analysis on collected allocations
        let unsafe_ffi = if !self.all_allocations.is_empty() {
            match FfiSafetyAnalyzer::analyze_allocations(&self.all_allocations) {
                Ok(analysis) => Some(analysis),
                Err(e) => {
                    tracing::warn!("FFI safety analysis failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Perform variable relationship analysis on collected allocations
        let variable_relationships = if !self.all_allocations.is_empty() {
            match VariableRelationshipAnalyzer::analyze_allocations(&self.all_allocations) {
                Ok(analysis) => Some(analysis),
                Err(e) => {
                    tracing::warn!("Variable relationship analysis failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(BinaryTemplateData {
            project_name: project_name.to_string(),
            allocations: self.allocation_buffer.clone(),
            total_memory_usage: total_memory,
            peak_memory_usage: peak_memory,
            active_allocations_count: active_count,
            processing_time_ms: self.stats.data_processing_time_ms,
            data_source: "binary_direct".to_string(),
            complex_types,
            unsafe_ffi,
            variable_relationships,
        })
    }

    /// Render HTML using binary template engine
    fn render_binary_template(
        &self,
        data: &BinaryTemplateData,
    ) -> Result<String, BinaryExportError> {
        use crate::export::binary::binary_template_engine::BinaryTemplateEngine;

        // Create and use the binary template engine
        let mut template_engine = BinaryTemplateEngine::new().map_err(|e| {
            BinaryExportError::CorruptedData(format!("Failed to create template engine: {}", e))
        })?;

        // Render the template with binary data
        template_engine.render_binary_template(data)
    }

    /// Convert binary template data to JSON format for template compatibility
    fn _convert_to_json_format(
        &self,
        data: &BinaryTemplateData,
    ) -> Result<String, BinaryExportError> {
        use serde_json::json;

        let allocations_json: Vec<serde_json::Value> = data
            .allocations
            .iter()
            .map(|alloc| {
                json!({
                    "id": alloc.id,
                    "size": alloc.size,
                    "type_name": alloc.type_name,
                    "scope_name": alloc.scope_name,
                    "timestamp_alloc": alloc.timestamp_alloc,
                    "is_active": alloc.is_active,
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "thread_id": alloc.thread_id,
                    "var_name": alloc.var_name,
                    "borrow_count": alloc.borrow_count,
                    "is_leaked": alloc.is_leaked,
                    "lifetime_ms": alloc.lifetime_ms
                })
            })
            .collect();

        let mut dashboard_data = json!({
            "project_name": data.project_name,
            "data_source": data.data_source,
            "summary": {
                "total_allocations": data.allocations.len(),
                "total_memory": data.total_memory_usage,
                "peak_memory": data.peak_memory_usage,
                "active_allocations": data.active_allocations_count
            },
            "memory_analysis": {
                "allocations": allocations_json,
                "memory_timeline": [],
                "size_distribution": []
            },
            "performance_metrics": {
                "export_time_ms": data.processing_time_ms,
                "data_source": "binary_direct",
                "throughput_allocations_per_sec": self.stats.processing_throughput()
            }
        });

        // Add complex type analysis if available
        if let Some(ref complex_types) = data.complex_types {
            dashboard_data["complex_types"] = serde_json::to_value(complex_types).map_err(|e| {
                BinaryExportError::SerializationError(format!(
                    "Complex types serialization failed: {}",
                    e
                ))
            })?;
        }

        // Add FFI safety analysis if available
        if let Some(ref unsafe_ffi) = data.unsafe_ffi {
            dashboard_data["unsafe_ffi"] = serde_json::to_value(unsafe_ffi).map_err(|e| {
                BinaryExportError::SerializationError(format!(
                    "FFI safety analysis serialization failed: {}",
                    e
                ))
            })?;
        }

        // Add variable relationship analysis if available
        if let Some(ref variable_relationships) = data.variable_relationships {
            dashboard_data["variable_relationships"] = serde_json::to_value(variable_relationships)
                .map_err(|e| {
                    BinaryExportError::SerializationError(format!(
                        "Variable relationship analysis serialization failed: {}",
                        e
                    ))
                })?;
        }

        serde_json::to_string_pretty(&dashboard_data).map_err(|e| {
            BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e))
        })
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &BinaryHtmlStats {
        &self.stats
    }

    /// Update peak memory usage tracking
    fn _update_peak_memory_usage(&mut self) {
        if self.current_memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.current_memory_usage;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_allocation() -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("test_var".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: Some("main".to_string()),
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: Some(vec!["frame1".to_string(), "frame2".to_string()]),
            is_leaked: false,
            lifetime_ms: Some(1000),
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
    fn test_binary_html_writer_creation() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let writer = BinaryHtmlWriter::new(cursor);
        assert!(writer.is_ok());
    }

    #[test]
    fn test_binary_allocation_data_conversion() {
        let allocation = create_test_allocation();
        let fields = AllocationField::all_basic_fields();

        let binary_data = BinaryAllocationData::from_allocation(&allocation, &fields);
        assert!(binary_data.is_ok());

        let data = binary_data.expect("Failed to get test value");
        assert_eq!(data.size, 1024);
        assert_eq!(data.type_name, "Vec<u8>");
        assert_eq!(data.is_active, true);
    }

    #[test]
    fn test_write_binary_allocation() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = BinaryHtmlWriter::new(cursor).expect("Failed to get test value");

        let allocation = create_test_allocation();
        let fields = AllocationField::all_basic_fields();

        let result = writer.write_binary_allocation(&allocation, &fields);
        assert!(result.is_ok());
        assert_eq!(writer.stats.allocations_processed, 1);
    }

    #[test]
    fn test_batch_processing() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = BinaryHtmlWriter::new(cursor).expect("Failed to get test value");

        let allocations = vec![create_test_allocation(); 5];
        let fields = AllocationField::all_basic_fields();

        let result = writer.write_binary_allocation_batch(&allocations, &fields);
        assert!(result.is_ok());
        assert_eq!(writer.stats.allocations_processed, 5);
    }

    #[test]
    fn test_finalize_with_template() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = BinaryHtmlWriter::new(cursor).expect("Failed to get test value");

        let allocation = create_test_allocation();
        let fields = AllocationField::all_basic_fields();

        writer
            .write_binary_allocation(&allocation, &fields)
            .expect("Test operation failed");
        let stats = writer
            .finalize_with_binary_template("test_project")
            .expect("Test operation failed");

        assert_eq!(stats.allocations_processed, 1);
        assert!(stats.total_html_size > 0);
        assert!(stats.total_processing_time_ms > 0);
    }

    #[test]
    fn test_stats_calculation() {
        let mut stats = BinaryHtmlStats::default();
        stats.allocations_processed = 1000;
        stats.total_processing_time_ms = 500;
        stats.peak_memory_usage = 1024 * 1024; // 1MB

        assert_eq!(stats.processing_throughput(), 2000.0); // 2000 allocations/sec
        assert_eq!(stats.memory_efficiency_ratio(), 1000.0); // 1000 allocations/MB
    }
}
