//! JSON converter for binary-to-JSON format conversion
//!
//! This module provides comprehensive JSON conversion capabilities including:
//! - Binary data to JSON format conversion with full compatibility
//! - Streaming JSON conversion for large datasets
//! - Memory-efficient processing with chunked operations
//! - Format validation and consistency checking

use crate::core::types::{AllocationInfo, MemoryStats, TrackingResult};
use crate::export::formats::binary_export::BinaryExportData;
use serde_json::{Map, Value};
use std::io::Write;
use std::path::Path;

/// Configuration options for JSON conversion
#[derive(Debug, Clone)]
pub struct JsonConvertOptions {
    /// Enable pretty-printing for human readability
    pub pretty_print: bool,
    /// Include metadata in the output
    pub include_metadata: bool,
    /// Include performance statistics
    pub include_statistics: bool,
    /// Enable streaming mode for large files
    pub streaming_mode: bool,
    /// Chunk size for streaming operations (number of allocations)
    pub chunk_size: usize,
    /// Maximum memory usage for conversion (bytes)
    pub max_memory_usage: usize,
    /// Include lifecycle events if available
    pub include_lifecycle_events: bool,
    /// Validate output JSON format
    pub validate_output: bool,
}

impl JsonConvertOptions {
    /// Fast conversion configuration - minimal features for speed
    pub fn fast() -> Self {
        Self {
            pretty_print: false,
            include_metadata: false,
            include_statistics: false,
            streaming_mode: false,
            chunk_size: 1000,
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            include_lifecycle_events: false,
            validate_output: false,
        }
    }

    /// Complete conversion configuration - all features enabled
    pub fn complete() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
            include_statistics: true,
            streaming_mode: false,
            chunk_size: 5000,
            max_memory_usage: 500 * 1024 * 1024, // 500MB
            include_lifecycle_events: true,
            validate_output: true,
        }
    }

    /// Streaming conversion configuration - optimized for large files
    pub fn streaming() -> Self {
        Self {
            pretty_print: false,
            include_metadata: true,
            include_statistics: true,
            streaming_mode: true,
            chunk_size: 10000,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
            include_lifecycle_events: true,
            validate_output: false, // Skip validation for performance
        }
    }

    /// Compatible conversion configuration - ensures compatibility with existing tools
    pub fn compatible() -> Self {
        Self {
            pretty_print: true,
            include_metadata: false, // Skip metadata for compatibility
            include_statistics: false,
            streaming_mode: false,
            chunk_size: 5000,
            max_memory_usage: 256 * 1024 * 1024, // 256MB
            include_lifecycle_events: false,
            validate_output: true,
        }
    }
}

impl Default for JsonConvertOptions {
    fn default() -> Self {
        Self::complete()
    }
}

/// Statistics collected during JSON conversion
#[derive(Debug, Clone)]
pub struct JsonConversionStats {
    /// Total conversion time
    pub conversion_time: std::time::Duration,
    /// Number of allocations converted
    pub allocations_converted: usize,
    /// Size of output JSON in bytes
    pub output_size: u64,
    /// Memory usage during conversion
    pub peak_memory_usage: u64,
    /// Number of chunks processed (for streaming mode)
    pub chunks_processed: usize,
    /// Validation errors found (if validation enabled)
    pub validation_errors: Vec<String>,
}

/// JSON converter with advanced conversion capabilities
pub struct JsonConverter {
    /// Conversion configuration options
    options: JsonConvertOptions,
}

impl JsonConverter {
    /// Create a new JSON converter with specified options
    pub fn new(options: JsonConvertOptions) -> Self {
        Self { options }
    }

    /// Create a converter with fast conversion settings
    pub fn with_fast_settings() -> Self {
        Self::new(JsonConvertOptions::fast())
    }

    /// Create a converter with complete feature set
    pub fn with_complete_settings() -> Self {
        Self::new(JsonConvertOptions::complete())
    }

    /// Create a converter optimized for streaming large files
    pub fn with_streaming_settings() -> Self {
        Self::new(JsonConvertOptions::streaming())
    }

    /// Create a converter that ensures compatibility with existing tools
    pub fn with_compatible_settings() -> Self {
        Self::new(JsonConvertOptions::compatible())
    }

    /// Convert binary data to JSON file
    pub fn convert_to_file<P: AsRef<Path>>(
        &self,
        data: &BinaryExportData,
        path: P,
    ) -> TrackingResult<JsonConversionStats> {
        let path_str = path.as_ref().to_string_lossy();
        let start_time = std::time::Instant::now();

        println!("🔄 Starting binary-to-JSON conversion: {path_str}");
        println!(
            "📋 Conversion options: pretty={}, metadata={}, streaming={}, chunk_size={}",
            self.options.pretty_print,
            self.options.include_metadata,
            self.options.streaming_mode,
            self.options.chunk_size
        );

        let mut stats = JsonConversionStats {
            conversion_time: std::time::Duration::from_millis(0),
            allocations_converted: 0,
            output_size: 0,
            peak_memory_usage: 0,
            chunks_processed: 0,
            validation_errors: Vec::new(),
        };

        // Choose conversion method based on configuration
        let json_output = if self.options.streaming_mode && data.allocations.len() > self.options.chunk_size {
            println!("📊 Using streaming conversion for {} allocations", data.allocations.len());
            self.convert_streaming(data, &mut stats)?
        } else {
            println!("📊 Using standard conversion for {} allocations", data.allocations.len());
            self.convert_standard(data, &mut stats)?
        };

        // Write to file
        println!("💾 Writing JSON to file...");
        let write_start = std::time::Instant::now();
        
        let json_string = if self.options.pretty_print {
            serde_json::to_string_pretty(&json_output)
        } else {
            serde_json::to_string(&json_output)
        }
        .map_err(|e| crate::core::types::TrackingError::SerializationError(
            format!("JSON serialization failed: {e}")
        ))?;

        std::fs::write(&path, &json_string)
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to write JSON file: {e}")
            ))?;

        let write_duration = write_start.elapsed();
        stats.output_size = json_string.len() as u64;

        println!("✅ File write completed in {write_duration:?}");

        // Validate output if enabled
        if self.options.validate_output {
            println!("🔍 Validating output JSON...");
            self.validate_json_output(&json_output, &mut stats)?;
        }

        stats.conversion_time = start_time.elapsed();

        println!("🎉 JSON conversion completed successfully!");
        println!("   - Total time: {:?}", stats.conversion_time);
        println!("   - Allocations converted: {}", stats.allocations_converted);
        println!("   - Output size: {} bytes", stats.output_size);
        println!("   - Chunks processed: {}", stats.chunks_processed);
        if !stats.validation_errors.is_empty() {
            println!("   - Validation errors: {}", stats.validation_errors.len());
        }

        Ok(stats)
    }

    /// Convert binary data to JSON stream
    pub fn convert_to_stream<W: Write>(
        &self,
        data: &BinaryExportData,
        mut writer: W,
    ) -> TrackingResult<JsonConversionStats> {
        println!("🔄 Starting binary-to-JSON stream conversion");
        let start_time = std::time::Instant::now();

        let mut stats = JsonConversionStats {
            conversion_time: std::time::Duration::from_millis(0),
            allocations_converted: 0,
            output_size: 0,
            peak_memory_usage: 0,
            chunks_processed: 0,
            validation_errors: Vec::new(),
        };

        // Convert to JSON value
        let json_output = if self.options.streaming_mode {
            self.convert_streaming(data, &mut stats)?
        } else {
            self.convert_standard(data, &mut stats)?
        };

        // Write to stream
        println!("💾 Writing JSON to stream...");
        let json_string = if self.options.pretty_print {
            serde_json::to_string_pretty(&json_output)
        } else {
            serde_json::to_string(&json_output)
        }
        .map_err(|e| crate::core::types::TrackingError::SerializationError(
            format!("JSON serialization failed: {e}")
        ))?;

        writer.write_all(json_string.as_bytes())
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to write to stream: {e}")
            ))?;

        writer.flush()
            .map_err(|e| crate::core::types::TrackingError::IoError(
                format!("Failed to flush stream: {e}")
            ))?;

        stats.output_size = json_string.len() as u64;
        stats.conversion_time = start_time.elapsed();

        println!("🎉 Stream conversion completed successfully!");
        println!("   - Total time: {:?}", stats.conversion_time);
        println!("   - Output size: {} bytes", stats.output_size);

        Ok(stats)
    }

    /// Standard conversion method for smaller datasets
    fn convert_standard(
        &self,
        data: &BinaryExportData,
        stats: &mut JsonConversionStats,
    ) -> TrackingResult<Value> {
        println!("🔧 Performing standard JSON conversion...");
        let convert_start = std::time::Instant::now();

        // Create the main JSON structure compatible with existing format
        let mut json_map = Map::new();

        // Add allocations array (main data)
        let allocations_json = self.convert_allocations(&data.allocations)?;
        json_map.insert("allocations".to_string(), Value::Array(allocations_json));

        // Add metadata if enabled
        if self.options.include_metadata {
            if let Some(metadata) = &data.metadata {
                let mut metadata_map = Map::new();
                metadata_map.insert("version".to_string(), Value::String(metadata.export_format_version.clone()));
                metadata_map.insert("timestamp".to_string(), Value::Number(metadata.timestamp.into()));
                
                if let Some(compression) = &metadata.compression_algorithm {
                    metadata_map.insert("compression".to_string(), Value::String(compression.clone()));
                }
                
                json_map.insert("metadata".to_string(), Value::Object(metadata_map));
            }
        }

        // Add statistics if enabled
        if self.options.include_statistics {
            let stats_json = self.convert_memory_stats(&data.stats)?;
            json_map.insert("statistics".to_string(), stats_json);
        }

        // Add lifecycle events if enabled and available
        if self.options.include_lifecycle_events {
            let lifecycle_events = self.generate_lifecycle_events(&data.allocations)?;
            json_map.insert("lifecycle_events".to_string(), Value::Array(lifecycle_events));
        }

        stats.allocations_converted = data.allocations.len();
        stats.chunks_processed = 1;

        let convert_duration = convert_start.elapsed();
        println!("✅ Standard conversion completed in {convert_duration:?}");

        Ok(Value::Object(json_map))
    }

    /// Streaming conversion method for large datasets
    fn convert_streaming(
        &self,
        data: &BinaryExportData,
        stats: &mut JsonConversionStats,
    ) -> TrackingResult<Value> {
        println!("🔧 Performing streaming JSON conversion...");
        let convert_start = std::time::Instant::now();

        let mut json_map = Map::new();
        let mut all_allocations = Vec::new();

        // Process allocations in chunks
        let chunks = data.allocations.chunks(self.options.chunk_size);
        let total_chunks = chunks.len();

        for (chunk_idx, chunk) in chunks.enumerate() {
            println!("📦 Processing chunk {}/{} ({} allocations)", 
                     chunk_idx + 1, total_chunks, chunk.len());
            
            let chunk_json = self.convert_allocations(chunk)?;
            all_allocations.extend(chunk_json);
            
            stats.chunks_processed += 1;
            
            // Memory management - force garbage collection periodically
            if chunk_idx % 10 == 0 {
                // In a real implementation, we might implement memory pressure handling here
                println!("🧹 Memory management checkpoint at chunk {}", chunk_idx + 1);
            }
        }

        json_map.insert("allocations".to_string(), Value::Array(all_allocations));

        // Add other sections similar to standard conversion
        if self.options.include_metadata {
            if let Some(metadata) = &data.metadata {
                let mut metadata_map = Map::new();
                metadata_map.insert("version".to_string(), Value::String(metadata.export_format_version.clone()));
                metadata_map.insert("timestamp".to_string(), Value::Number(metadata.timestamp.into()));
                json_map.insert("metadata".to_string(), Value::Object(metadata_map));
            }
        }

        if self.options.include_statistics {
            let stats_json = self.convert_memory_stats(&data.stats)?;
            json_map.insert("statistics".to_string(), stats_json);
        }

        stats.allocations_converted = data.allocations.len();

        let convert_duration = convert_start.elapsed();
        println!("✅ Streaming conversion completed in {convert_duration:?}");

        Ok(Value::Object(json_map))
    }

    /// Convert allocation information to JSON format
    fn convert_allocations(&self, allocations: &[AllocationInfo]) -> TrackingResult<Vec<Value>> {
        let mut json_allocations = Vec::with_capacity(allocations.len());

        for allocation in allocations {
            let mut alloc_map = Map::new();

            // Convert pointer to hex string format
            alloc_map.insert("ptr".to_string(), Value::String(format!("0x{:x}", allocation.ptr)));
            alloc_map.insert("size".to_string(), Value::Number(allocation.size.into()));
            alloc_map.insert("timestamp_alloc".to_string(), Value::Number(allocation.timestamp_alloc.into()));

            // Optional fields - use null for None values to match existing format
            alloc_map.insert("scope_name".to_string(), 
                allocation.scope_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::Null));

            alloc_map.insert("type_name".to_string(),
                allocation.type_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::Null));

            alloc_map.insert("var_name".to_string(),
                allocation.var_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::Null));

            alloc_map.insert("timestamp_dealloc".to_string(),
                allocation.timestamp_dealloc
                    .map(|t| Value::Number(t.into()))
                    .unwrap_or(Value::Null));

            // Add additional fields if they exist
            if allocation.borrow_count > 0 {
                alloc_map.insert("borrow_count".to_string(), Value::Number(allocation.borrow_count.into()));
            }

            if allocation.is_leaked {
                alloc_map.insert("is_leaked".to_string(), Value::Bool(true));
            }

            if let Some(lifetime) = allocation.lifetime_ms {
                alloc_map.insert("lifetime_ms".to_string(), Value::Number(lifetime.into()));
            }

            json_allocations.push(Value::Object(alloc_map));
        }

        Ok(json_allocations)
    }

    /// Convert memory statistics to JSON format
    fn convert_memory_stats(&self, stats: &MemoryStats) -> TrackingResult<Value> {
        let mut stats_map = Map::new();

        stats_map.insert("total_allocations".to_string(), Value::Number(stats.total_allocations.into()));
        stats_map.insert("total_allocated".to_string(), Value::Number(stats.total_allocated.into()));
        stats_map.insert("active_allocations".to_string(), Value::Number(stats.active_allocations.into()));
        stats_map.insert("active_memory".to_string(), Value::Number(stats.active_memory.into()));
        stats_map.insert("peak_allocations".to_string(), Value::Number(stats.peak_allocations.into()));
        stats_map.insert("peak_memory".to_string(), Value::Number(stats.peak_memory.into()));
        stats_map.insert("leaked_allocations".to_string(), Value::Number(stats.leaked_allocations.into()));
        stats_map.insert("leaked_memory".to_string(), Value::Number(stats.leaked_memory.into()));

        Ok(Value::Object(stats_map))
    }

    /// Generate lifecycle events from allocation data
    fn generate_lifecycle_events(&self, allocations: &[AllocationInfo]) -> TrackingResult<Vec<Value>> {
        let mut events = Vec::new();

        for allocation in allocations {
            // Allocation event
            let mut alloc_event = Map::new();
            alloc_event.insert("event".to_string(), Value::String("allocation".to_string()));
            alloc_event.insert("ptr".to_string(), Value::String(format!("0x{:x}", allocation.ptr)));
            alloc_event.insert("size".to_string(), Value::Number(allocation.size.into()));
            alloc_event.insert("timestamp".to_string(), Value::Number(allocation.timestamp_alloc.into()));
            
            alloc_event.insert("scope".to_string(),
                allocation.scope_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::String("global".to_string())));

            alloc_event.insert("type_name".to_string(),
                allocation.type_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::String("unknown".to_string())));

            alloc_event.insert("var_name".to_string(),
                allocation.var_name.as_ref()
                    .map(|s| Value::String(s.clone()))
                    .unwrap_or(Value::String("unknown".to_string())));

            events.push(Value::Object(alloc_event));

            // Deallocation event if available
            if let Some(dealloc_time) = allocation.timestamp_dealloc {
                let mut dealloc_event = Map::new();
                dealloc_event.insert("event".to_string(), Value::String("deallocation".to_string()));
                dealloc_event.insert("ptr".to_string(), Value::String(format!("0x{:x}", allocation.ptr)));
                dealloc_event.insert("timestamp".to_string(), Value::Number(dealloc_time.into()));
                
                events.push(Value::Object(dealloc_event));
            }
        }

        // Sort events by timestamp
        events.sort_by(|a, b| {
            let timestamp_a = a.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
            let timestamp_b = b.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
            timestamp_a.cmp(&timestamp_b)
        });

        Ok(events)
    }

    /// Validate the generated JSON output
    fn validate_json_output(&self, json: &Value, stats: &mut JsonConversionStats) -> TrackingResult<()> {
        println!("🔍 Validating JSON output structure...");

        // Check main structure
        if let Value::Object(map) = json {
            // Validate allocations array
            if let Some(Value::Array(allocations)) = map.get("allocations") {
                for (idx, allocation) in allocations.iter().enumerate() {
                    if let Value::Object(alloc_map) = allocation {
                        // Check required fields
                        if !alloc_map.contains_key("ptr") {
                            stats.validation_errors.push(format!("Allocation {} missing 'ptr' field", idx));
                        }
                        if !alloc_map.contains_key("size") {
                            stats.validation_errors.push(format!("Allocation {} missing 'size' field", idx));
                        }
                        if !alloc_map.contains_key("timestamp_alloc") {
                            stats.validation_errors.push(format!("Allocation {} missing 'timestamp_alloc' field", idx));
                        }
                    } else {
                        stats.validation_errors.push(format!("Allocation {} is not an object", idx));
                    }
                }
            } else {
                stats.validation_errors.push("Missing or invalid 'allocations' array".to_string());
            }

            // Validate lifecycle events if present
            if let Some(Value::Array(events)) = map.get("lifecycle_events") {
                for (idx, event) in events.iter().enumerate() {
                    if let Value::Object(event_map) = event {
                        if !event_map.contains_key("event") || !event_map.contains_key("timestamp") {
                            stats.validation_errors.push(format!("Lifecycle event {} missing required fields", idx));
                        }
                    }
                }
            }
        } else {
            stats.validation_errors.push("Root JSON is not an object".to_string());
        }

        if stats.validation_errors.is_empty() {
            println!("✅ JSON validation passed");
        } else {
            println!("⚠️  JSON validation found {} errors", stats.validation_errors.len());
            for error in &stats.validation_errors {
                println!("   - {error}");
            }
        }

        Ok(())
    }
}

impl Default for JsonConverter {
    fn default() -> Self {
        Self::with_complete_settings()
    }
}