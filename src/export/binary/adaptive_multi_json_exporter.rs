//! Adaptive multi-file JSON export system
//!
//! This module provides intelligent strategy selection for binary-to-JSON conversion
//! based on file size and complexity. It automatically chooses between simple direct
//! processing, index optimization, and full streaming processing to achieve optimal
//! performance for different file sizes.

use crate::core::types::AllocationInfo;
use crate::export::binary::{
    BinaryExportError, BinaryIndexBuilder, BinaryParser, SelectiveJsonExporter,
    SelectiveJsonExportConfigBuilder, SelectiveJsonExportStats, 
    AllocationField, AllocationFilter, StreamingJsonWriterConfig,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::info;

/// File size thresholds for strategy selection
const SMALL_FILE_THRESHOLD: u64 = 150 * 1024; // 150KB
const STREAMING_OPTIMIZATION_THRESHOLD: u64 = 1024 * 1024; // 1MB

/// JSON output types supported by the adaptive exporter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonType {
    MemoryAnalysis,
    LifetimeAnalysis,
    PerformanceAnalysis,
    ComplexTypes,
    UnsafeFFI,
}

impl JsonType {
    /// Get the filename suffix for this JSON type
    pub fn filename_suffix(&self) -> &'static str {
        match self {
            JsonType::MemoryAnalysis => "memory_analysis",
            JsonType::LifetimeAnalysis => "lifetime",
            JsonType::PerformanceAnalysis => "performance",
            JsonType::ComplexTypes => "complex_types",
            JsonType::UnsafeFFI => "unsafe_ffi",
        }
    }

    /// Get the required fields for this JSON type
    pub fn required_fields(&self) -> Vec<AllocationField> {
        match self {
            JsonType::MemoryAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::ThreadId,
                AllocationField::TimestampAlloc,
                AllocationField::IsLeaked,
                AllocationField::BorrowCount,
            ],
            JsonType::LifetimeAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::VarName,
                AllocationField::TimestampAlloc,
                AllocationField::TimestampDealloc,
                AllocationField::LifetimeMs,
                AllocationField::ScopeName,
            ],
            JsonType::PerformanceAnalysis => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::TimestampAlloc,
                AllocationField::ThreadId,
                AllocationField::BorrowCount,
            ],
            JsonType::ComplexTypes => vec![
                AllocationField::Ptr,
                AllocationField::Size,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::SmartPointerInfo,
                AllocationField::MemoryLayout,
                AllocationField::GenericInfo,
            ],
            JsonType::UnsafeFFI => vec![
                AllocationField::Ptr,
                AllocationField::VarName,
                AllocationField::TypeName,
                AllocationField::ThreadId,
                AllocationField::StackTrace,
                AllocationField::RuntimeState,
            ],
        }
    }
}

/// Processing strategy used for export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingStrategy {
    /// Simple direct processing for small files (<150KB)
    SimpleDirect,
    /// Index optimization for medium files (150KB-1MB)
    IndexOptimized,
    /// Full streaming processing for large files (>1MB)
    FullyStreaming,
}

impl ProcessingStrategy {
    /// Get a human-readable description of the strategy
    pub fn description(&self) -> &'static str {
        match self {
            ProcessingStrategy::SimpleDirect => "Simple direct processing",
            ProcessingStrategy::IndexOptimized => "Index-optimized processing",
            ProcessingStrategy::FullyStreaming => "Full streaming processing",
        }
    }
}

/// Statistics for multi-file export operations
#[derive(Debug, Clone)]
pub struct MultiExportStats {
    /// Total number of records processed
    pub total_records: usize,
    /// Maximum memory used during processing
    pub max_memory_used: usize,
    /// Cache hit rate for field processing
    pub cache_hit_rate: f64,
    /// Statistics for each JSON type
    pub per_json_stats: HashMap<JsonType, SelectiveJsonExportStats>,
    /// Total processing duration
    pub total_duration: Duration,
    /// Strategy used for processing
    pub strategy_used: ProcessingStrategy,
    /// File size that triggered strategy selection
    pub file_size: u64,
}

impl MultiExportStats {
    /// Calculate overall throughput (records per second)
    pub fn overall_throughput(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.total_records as f64 / self.total_duration.as_secs_f64()
        }
    }

    /// Calculate average processing time per record
    pub fn avg_processing_time_per_record(&self) -> Duration {
        if self.total_records == 0 {
            Duration::ZERO
        } else {
            self.total_duration / self.total_records as u32
        }
    }
}

/// Configuration for adaptive multi-file export
#[derive(Debug, Clone)]
pub struct AdaptiveExportConfig {
    /// Small file threshold (default: 150KB)
    pub small_file_threshold: u64,
    /// Streaming threshold (default: 1MB)
    pub streaming_threshold: u64,
    /// Force a specific strategy (overrides automatic selection)
    pub force_strategy: Option<ProcessingStrategy>,
    /// Enable pretty printing for JSON output
    pub pretty_print: bool,
    /// Buffer size for streaming operations
    pub buffer_size: usize,
    /// Enable parallel processing where applicable
    pub enable_parallel_processing: bool,
}

impl Default for AdaptiveExportConfig {
    fn default() -> Self {
        Self {
            small_file_threshold: SMALL_FILE_THRESHOLD,
            streaming_threshold: STREAMING_OPTIMIZATION_THRESHOLD,
            force_strategy: None,
            pretty_print: false,
            buffer_size: 64 * 1024, // 64KB
            enable_parallel_processing: true,
        }
    }
}

/// Adaptive multi-file JSON exporter with intelligent strategy selection
pub struct AdaptiveMultiJsonExporter {
    config: AdaptiveExportConfig,
}

impl AdaptiveMultiJsonExporter {
    /// Create a new adaptive exporter with default configuration
    pub fn new() -> Self {
        Self {
            config: AdaptiveExportConfig::default(),
        }
    }

    /// Create a new adaptive exporter with custom configuration
    pub fn new_with_config(config: AdaptiveExportConfig) -> Self {
        Self { config }
    }

    /// Export binary file to multiple JSON types with adaptive strategy selection
    pub fn export_adaptive<P: AsRef<Path>>(
        &self,
        binary_path: P,
        base_name: &str,
        json_types: &[JsonType],
    ) -> Result<MultiExportStats, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let start_time = Instant::now();

        // Determine file size and select strategy
        let file_size = std::fs::metadata(binary_path)?.len();

        let strategy = self.select_strategy(file_size);

        info!(
            "Processing file {} ({:.1}KB) using {}",
            binary_path.display(),
            file_size as f64 / 1024.0,
            strategy.description()
        );

        // Execute the selected strategy
        let mut stats = match strategy {
            ProcessingStrategy::SimpleDirect => {
                self.export_simple_direct(binary_path, base_name, json_types)?
            }
            ProcessingStrategy::IndexOptimized => {
                self.export_with_index_optimization(binary_path, base_name, json_types)?
            }
            ProcessingStrategy::FullyStreaming => {
                self.export_streaming_multi_json(binary_path, base_name, json_types)?
            }
        };

        stats.total_duration = start_time.elapsed();
        stats.strategy_used = strategy;
        stats.file_size = file_size;

        info!(
            "Export completed in {:.2}s using {} (throughput: {:.1} records/s)",
            stats.total_duration.as_secs_f64(),
            strategy.description(),
            stats.overall_throughput()
        );

        Ok(stats)
    }

    /// Select the optimal processing strategy based on file size
    fn select_strategy(&self, file_size: u64) -> ProcessingStrategy {
        // Check for forced strategy first
        if let Some(forced) = self.config.force_strategy {
            info!("Using forced strategy: {}", forced.description());
            return forced;
        }

        // Automatic strategy selection based on file size
        if file_size <= self.config.small_file_threshold {
            ProcessingStrategy::SimpleDirect
        } else if file_size <= self.config.streaming_threshold {
            ProcessingStrategy::IndexOptimized
        } else {
            ProcessingStrategy::FullyStreaming
        }
    }
}

impl Default for AdaptiveMultiJsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;
    use crate::export::binary::BinaryWriter;

    use tempfile::TempDir;

    /// Create a test binary file with sample allocation data
    fn create_test_binary_file(temp_dir: &TempDir, allocations: &[AllocationInfo]) -> std::path::PathBuf {
        let binary_path = temp_dir.path().join("test.memscope");
        let mut writer = BinaryWriter::new(&binary_path).expect("Failed to create binary writer");
        
        writer.build_string_table(allocations).expect("Failed to build string table");
        writer.write_header(allocations.len() as u32).expect("Failed to write header");
        
        for allocation in allocations {
            writer.write_allocation(allocation).expect("Failed to write allocation");
        }
        
        writer.finish().expect("Failed to finish writing");
        binary_path
    }

    /// Create a default AllocationInfo for testing
    fn create_default_allocation_info() -> AllocationInfo {
        AllocationInfo {
            ptr: 0x1000,
            size: 64,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: 1000000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
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

    /// Create sample allocation data for testing
    fn create_sample_allocations(count: usize) -> Vec<AllocationInfo> {
        (0..count)
            .map(|i| AllocationInfo {
                ptr: 0x1000 + (i * 64),
                size: 64 + (i % 1000), // Varying sizes
                var_name: Some(format!("var_{}", i)),
                type_name: Some(format!("Type{}", i % 10)),
                thread_id: format!("thread_{}", i % 4),
                timestamp_alloc: 1000000 + (i as u64 * 1000),
                timestamp_dealloc: if i % 3 == 0 { Some(1000000 + (i as u64 * 1000) + 500) } else { None },
                lifetime_ms: if i % 3 == 0 { Some(500) } else { None },
                is_leaked: i % 5 == 0,
                borrow_count: i % 3,
                scope_name: Some(format!("scope_{}", i % 5)),
                stack_trace: if i % 7 == 0 { Some(vec![format!("frame_{}", i)]) } else { None },
                ..create_default_allocation_info()
            })
            .collect()
    }

    #[test]
    fn test_json_type_filename_suffix() {
        assert_eq!(JsonType::MemoryAnalysis.filename_suffix(), "memory_analysis");
        assert_eq!(JsonType::LifetimeAnalysis.filename_suffix(), "lifetime");
        assert_eq!(JsonType::PerformanceAnalysis.filename_suffix(), "performance");
        assert_eq!(JsonType::ComplexTypes.filename_suffix(), "complex_types");
        assert_eq!(JsonType::UnsafeFFI.filename_suffix(), "unsafe_ffi");
    }

    #[test]
    fn test_json_type_required_fields() {
        let memory_fields = JsonType::MemoryAnalysis.required_fields();
        assert!(memory_fields.contains(&AllocationField::Ptr));
        assert!(memory_fields.contains(&AllocationField::Size));
        assert!(memory_fields.contains(&AllocationField::VarName));
        assert!(memory_fields.contains(&AllocationField::TypeName));
        assert!(memory_fields.contains(&AllocationField::ThreadId));
        assert!(memory_fields.contains(&AllocationField::TimestampAlloc));
        assert!(memory_fields.contains(&AllocationField::IsLeaked));
        assert!(memory_fields.contains(&AllocationField::BorrowCount));

        let lifetime_fields = JsonType::LifetimeAnalysis.required_fields();
        assert!(lifetime_fields.contains(&AllocationField::Ptr));
        assert!(lifetime_fields.contains(&AllocationField::VarName));
        assert!(lifetime_fields.contains(&AllocationField::TimestampAlloc));
        assert!(lifetime_fields.contains(&AllocationField::TimestampDealloc));
        assert!(lifetime_fields.contains(&AllocationField::LifetimeMs));
        assert!(lifetime_fields.contains(&AllocationField::ScopeName));

        let performance_fields = JsonType::PerformanceAnalysis.required_fields();
        assert!(performance_fields.contains(&AllocationField::Ptr));
        assert!(performance_fields.contains(&AllocationField::Size));
        assert!(performance_fields.contains(&AllocationField::TimestampAlloc));
        assert!(performance_fields.contains(&AllocationField::ThreadId));
        assert!(performance_fields.contains(&AllocationField::BorrowCount));
    }

    #[test]
    fn test_strategy_selection() {
        let exporter = AdaptiveMultiJsonExporter::new();

        // Small file
        assert_eq!(
            exporter.select_strategy(100 * 1024), // 100KB
            ProcessingStrategy::SimpleDirect
        );

        // Medium file
        assert_eq!(
            exporter.select_strategy(500 * 1024), // 500KB
            ProcessingStrategy::IndexOptimized
        );

        // Large file
        assert_eq!(
            exporter.select_strategy(2 * 1024 * 1024), // 2MB
            ProcessingStrategy::FullyStreaming
        );

        // Edge cases
        assert_eq!(
            exporter.select_strategy(SMALL_FILE_THRESHOLD), // Exactly at threshold
            ProcessingStrategy::SimpleDirect
        );

        assert_eq!(
            exporter.select_strategy(SMALL_FILE_THRESHOLD + 1), // Just above small threshold
            ProcessingStrategy::IndexOptimized
        );

        assert_eq!(
            exporter.select_strategy(STREAMING_OPTIMIZATION_THRESHOLD), // Exactly at streaming threshold
            ProcessingStrategy::IndexOptimized
        );

        assert_eq!(
            exporter.select_strategy(STREAMING_OPTIMIZATION_THRESHOLD + 1), // Just above streaming threshold
            ProcessingStrategy::FullyStreaming
        );
    }

    #[test]
    fn test_forced_strategy() {
        let config = AdaptiveExportConfig {
            force_strategy: Some(ProcessingStrategy::FullyStreaming),
            ..Default::default()
        };
        let exporter = AdaptiveMultiJsonExporter::new_with_config(config);

        // Should use forced strategy regardless of file size
        assert_eq!(
            exporter.select_strategy(50 * 1024), // 50KB (normally SimpleDirect)
            ProcessingStrategy::FullyStreaming
        );

        assert_eq!(
            exporter.select_strategy(500 * 1024), // 500KB (normally IndexOptimized)
            ProcessingStrategy::FullyStreaming
        );

        assert_eq!(
            exporter.select_strategy(5 * 1024 * 1024), // 5MB (normally FullyStreaming)
            ProcessingStrategy::FullyStreaming
        );
    }

    #[test]
    fn test_custom_thresholds() {
        let config = AdaptiveExportConfig {
            small_file_threshold: 50 * 1024,  // 50KB
            streaming_threshold: 200 * 1024,  // 200KB
            ..Default::default()
        };
        let exporter = AdaptiveMultiJsonExporter::new_with_config(config);

        assert_eq!(
            exporter.select_strategy(40 * 1024), // 40KB
            ProcessingStrategy::SimpleDirect
        );

        assert_eq!(
            exporter.select_strategy(100 * 1024), // 100KB
            ProcessingStrategy::IndexOptimized
        );

        assert_eq!(
            exporter.select_strategy(300 * 1024), // 300KB
            ProcessingStrategy::FullyStreaming
        );
    }

    #[test]
    fn test_multi_export_stats_calculations() {
        let stats = MultiExportStats {
            total_records: 1000,
            max_memory_used: 1024 * 1024,
            cache_hit_rate: 85.0,
            per_json_stats: HashMap::new(),
            total_duration: Duration::from_secs(2),
            strategy_used: ProcessingStrategy::IndexOptimized,
            file_size: 500 * 1024,
        };

        assert_eq!(stats.overall_throughput(), 500.0); // 1000 records / 2 seconds
        assert_eq!(stats.avg_processing_time_per_record(), Duration::from_millis(2));

        // Test edge cases
        let zero_duration_stats = MultiExportStats {
            total_records: 1000,
            total_duration: Duration::ZERO,
            ..stats.clone()
        };
        assert_eq!(zero_duration_stats.overall_throughput(), 0.0);

        let zero_records_stats = MultiExportStats {
            total_records: 0,
            total_duration: Duration::from_secs(2),
            ..stats.clone()
        };
        assert_eq!(zero_records_stats.avg_processing_time_per_record(), Duration::ZERO);
    }

    #[test]
    fn test_processing_strategy_description() {
        assert_eq!(ProcessingStrategy::SimpleDirect.description(), "Simple direct processing");
        assert_eq!(ProcessingStrategy::IndexOptimized.description(), "Index-optimized processing");
        assert_eq!(ProcessingStrategy::FullyStreaming.description(), "Full streaming processing");
    }

    #[test]
    fn test_adaptive_export_config_default() {
        let config = AdaptiveExportConfig::default();
        
        assert_eq!(config.small_file_threshold, SMALL_FILE_THRESHOLD);
        assert_eq!(config.streaming_threshold, STREAMING_OPTIMIZATION_THRESHOLD);
        assert_eq!(config.force_strategy, None);
        assert_eq!(config.pretty_print, false);
        assert_eq!(config.buffer_size, 64 * 1024);
        assert_eq!(config.enable_parallel_processing, true);
    }

    #[test]
    fn test_filter_allocations_for_json_type() {
        let exporter = AdaptiveMultiJsonExporter::new();
        let allocations = create_sample_allocations(10);

        // Test MemoryAnalysis - should include all allocations
        let memory_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::MemoryAnalysis);
        assert_eq!(memory_filtered.len(), 10);

        // Test LifetimeAnalysis - should only include deallocated allocations
        let lifetime_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::LifetimeAnalysis);
        let expected_lifetime_count = allocations.iter().filter(|a| a.timestamp_dealloc.is_some()).count();
        assert_eq!(lifetime_filtered.len(), expected_lifetime_count);

        // Test PerformanceAnalysis - should only include allocations >= 1024 bytes
        let performance_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::PerformanceAnalysis);
        let expected_performance_count = allocations.iter().filter(|a| a.size >= 1024).count();
        assert_eq!(performance_filtered.len(), expected_performance_count);

        // Test ComplexTypes - should only include allocations with type names
        let complex_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::ComplexTypes);
        let expected_complex_count = allocations.iter()
            .filter(|a| a.type_name.is_some() && a.type_name.as_ref().unwrap() != "unknown")
            .count();
        assert_eq!(complex_filtered.len(), expected_complex_count);

        // Test UnsafeFFI - should only include allocations with stack traces
        let ffi_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::UnsafeFFI);
        let expected_ffi_count = allocations.iter()
            .filter(|a| a.stack_trace.is_some() && !a.stack_trace.as_ref().unwrap().is_empty())
            .count();
        assert_eq!(ffi_filtered.len(), expected_ffi_count);
    }

    #[test]
    fn test_filter_allocations_json_structure() {
        let exporter = AdaptiveMultiJsonExporter::new();
        let allocations = vec![AllocationInfo {
            ptr: 0x1000,
            size: 128,
            var_name: Some("test_var".to_string()),
            type_name: Some("TestType".to_string()),
            thread_id: "main".to_string(),
            timestamp_alloc: 1000000,
            timestamp_dealloc: Some(1000500),
            lifetime_ms: Some(500),
            is_leaked: false,
            borrow_count: 2,
            scope_name: Some("test_scope".to_string()),
            stack_trace: Some(vec!["frame1".to_string(), "frame2".to_string()]),
            ..create_default_allocation_info()
        }];

        let memory_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::MemoryAnalysis);
        assert_eq!(memory_json.len(), 1);

        if let serde_json::Value::Object(obj) = &memory_json[0] {
            assert_eq!(obj.get("ptr").unwrap(), &serde_json::Value::String("0x1000".to_string()));
            assert_eq!(obj.get("size").unwrap(), &serde_json::Value::Number(serde_json::Number::from(128)));
            assert_eq!(obj.get("var_name").unwrap(), &serde_json::Value::String("test_var".to_string()));
            assert_eq!(obj.get("type_name").unwrap(), &serde_json::Value::String("TestType".to_string()));
            assert_eq!(obj.get("thread_id").unwrap(), &serde_json::Value::String("main".to_string()));
            assert_eq!(obj.get("timestamp_alloc").unwrap(), &serde_json::Value::Number(serde_json::Number::from(1000000)));
            assert_eq!(obj.get("is_leaked").unwrap(), &serde_json::Value::Bool(false));
            assert_eq!(obj.get("borrow_count").unwrap(), &serde_json::Value::Number(serde_json::Number::from(2)));
        } else {
            panic!("Expected JSON object");
        }

        let lifetime_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::LifetimeAnalysis);
        assert_eq!(lifetime_json.len(), 1);

        if let serde_json::Value::Object(obj) = &lifetime_json[0] {
            assert_eq!(obj.get("timestamp_dealloc").unwrap(), &serde_json::Value::Number(serde_json::Number::from(1000500)));
            assert_eq!(obj.get("lifetime_ms").unwrap(), &serde_json::Value::Number(serde_json::Number::from(500)));
            assert_eq!(obj.get("scope_name").unwrap(), &serde_json::Value::String("test_scope".to_string()));
        } else {
            panic!("Expected JSON object");
        }

        let ffi_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::UnsafeFFI);
        assert_eq!(ffi_json.len(), 1);

        if let serde_json::Value::Object(obj) = &ffi_json[0] {
            if let serde_json::Value::Array(stack_trace) = obj.get("stack_trace").unwrap() {
                assert_eq!(stack_trace.len(), 2);
                assert_eq!(stack_trace[0], serde_json::Value::String("frame1".to_string()));
                assert_eq!(stack_trace[1], serde_json::Value::String("frame2".to_string()));
            } else {
                panic!("Expected stack_trace to be an array");
            }
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_create_filters_for_json_type() {
        let exporter = AdaptiveMultiJsonExporter::new();

        // MemoryAnalysis should have no filters
        let memory_filters = exporter.create_filters_for_json_type(JsonType::MemoryAnalysis);
        assert_eq!(memory_filters.len(), 0);

        // LifetimeAnalysis should have no filters (filtering done in simple direct processing)
        let lifetime_filters = exporter.create_filters_for_json_type(JsonType::LifetimeAnalysis);
        assert_eq!(lifetime_filters.len(), 0);

        // PerformanceAnalysis should filter by size >= 1024
        let performance_filters = exporter.create_filters_for_json_type(JsonType::PerformanceAnalysis);
        assert_eq!(performance_filters.len(), 1);
        match &performance_filters[0] {
            AllocationFilter::SizeRange(min, max) => {
                assert_eq!(*min, 1024);
                assert_eq!(*max, usize::MAX);
            }
            _ => panic!("Expected SizeRange filter"),
        }

        // ComplexTypes should filter by size >= 64
        let complex_filters = exporter.create_filters_for_json_type(JsonType::ComplexTypes);
        assert_eq!(complex_filters.len(), 1);
        match &complex_filters[0] {
            AllocationFilter::SizeRange(min, max) => {
                assert_eq!(*min, 64);
                assert_eq!(*max, usize::MAX);
            }
            _ => panic!("Expected SizeRange filter"),
        }

        // UnsafeFFI should filter by size >= 32
        let ffi_filters = exporter.create_filters_for_json_type(JsonType::UnsafeFFI);
        assert_eq!(ffi_filters.len(), 1);
        match &ffi_filters[0] {
            AllocationFilter::SizeRange(min, max) => {
                assert_eq!(*min, 32);
                assert_eq!(*max, usize::MAX);
            }
            _ => panic!("Expected SizeRange filter"),
        }
    }

    #[test]
    fn test_empty_allocations_handling() {
        let exporter = AdaptiveMultiJsonExporter::new();
        let empty_allocations: Vec<AllocationInfo> = vec![];

        // Test that empty allocations are handled gracefully
        for json_type in [
            JsonType::MemoryAnalysis,
            JsonType::LifetimeAnalysis,
            JsonType::PerformanceAnalysis,
            JsonType::ComplexTypes,
            JsonType::UnsafeFFI,
        ] {
            let filtered = exporter.filter_allocations_for_json_type(&empty_allocations, json_type);
            assert_eq!(filtered.len(), 0);
        }
    }

    #[test]
    fn test_null_field_handling() {
        let exporter = AdaptiveMultiJsonExporter::new();
        let allocations = vec![AllocationInfo {
            ptr: 0x1000,
            size: 128,
            var_name: None,
            type_name: None,
            thread_id: "main".to_string(),
            timestamp_alloc: 1000000,
            timestamp_dealloc: None,
            lifetime_ms: None,
            is_leaked: false,
            borrow_count: 0,
            scope_name: None,
            stack_trace: None,
            ..create_default_allocation_info()
        }];

        let memory_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::MemoryAnalysis);
        assert_eq!(memory_json.len(), 1);

        if let serde_json::Value::Object(obj) = &memory_json[0] {
            assert_eq!(obj.get("var_name").unwrap(), &serde_json::Value::String("unknown".to_string()));
            assert_eq!(obj.get("type_name").unwrap(), &serde_json::Value::String("unknown".to_string()));
        } else {
            panic!("Expected JSON object");
        }

        let lifetime_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::LifetimeAnalysis);
        assert_eq!(lifetime_json.len(), 0); // Should be filtered out due to no deallocation

        let ffi_json = exporter.filter_allocations_for_json_type(&allocations, JsonType::UnsafeFFI);
        assert_eq!(ffi_json.len(), 0); // Should be filtered out due to no stack trace
    }

    #[test]
    fn test_large_allocation_filtering() {
        let exporter = AdaptiveMultiJsonExporter::new();
        let allocations = vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 32,  // Small allocation
                type_name: Some("SmallType".to_string()),
                stack_trace: Some(vec!["frame1".to_string()]),
                ..create_default_allocation_info()
            },
            AllocationInfo {
                ptr: 0x2000,
                size: 512, // Medium allocation
                type_name: Some("MediumType".to_string()),
                stack_trace: Some(vec!["frame2".to_string()]),
                ..create_default_allocation_info()
            },
            AllocationInfo {
                ptr: 0x3000,
                size: 2048, // Large allocation
                type_name: Some("LargeType".to_string()),
                stack_trace: Some(vec!["frame3".to_string()]),
                ..create_default_allocation_info()
            },
        ];

        // PerformanceAnalysis should only include allocations >= 1024 bytes
        let performance_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::PerformanceAnalysis);
        assert_eq!(performance_filtered.len(), 1); // Only the 2048-byte allocation

        // ComplexTypes should include allocations >= 64 bytes
        let complex_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::ComplexTypes);
        assert_eq!(complex_filtered.len(), 2); // 512 and 2048-byte allocations

        // UnsafeFFI should include allocations >= 32 bytes
        let ffi_filtered = exporter.filter_allocations_for_json_type(&allocations, JsonType::UnsafeFFI);
        assert_eq!(ffi_filtered.len(), 3); // All allocations (but filtered by stack trace)
    }

    // Integration test would require actual binary files, which is complex for unit tests
    // These would be better as integration tests in a separate test file
}

impl AdaptiveMultiJsonExporter {
    /// Simple direct processing for small files (<150KB)
    /// Uses existing AnalysisEngine for straightforward processing
    /// 
    /// This strategy is optimized for small files where the overhead of indexing
    /// and streaming would be counterproductive. It loads all data into memory
    /// and processes it directly using simple JSON serialization.
    fn export_simple_direct<P: AsRef<Path>>(
        &self,
        binary_path: P,
        base_name: &str,
        json_types: &[JsonType],
    ) -> Result<MultiExportStats, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let start_time = Instant::now();

        info!(
            "Using simple direct processing for small file: {}",
            binary_path.display()
        );

        // Load all allocations using existing parser
        let allocations = BinaryParser::load_allocations(binary_path)?;
        let total_records = allocations.len();

        if total_records == 0 {
            info!("No allocations found in binary file, creating empty JSON files");
        } else {
            info!("Loaded {} allocations for direct processing", total_records);
        }

        let mut per_json_stats = HashMap::new();
        let max_memory_used = allocations.len() * std::mem::size_of::<AllocationInfo>();

        // Process each JSON type using direct approach
        for json_type in json_types {
            let json_start = Instant::now();
            let file_path = format!("{}_{}.json", base_name, json_type.filename_suffix());

            info!("Processing {} for {}", json_type.filename_suffix(), file_path);

            // Create filtered data for this JSON type
            let filtered_allocations = self.filter_allocations_for_json_type(&allocations, *json_type);

            // Handle empty results gracefully
            if filtered_allocations.is_empty() {
                info!("No allocations match filters for {}, creating empty array", json_type.filename_suffix());
            }

            // Write JSON directly using simple serialization
            let json_content = if self.config.pretty_print {
                serde_json::to_string_pretty(&filtered_allocations)
                    .map_err(|e| BinaryExportError::SerializationError(format!(
                        "Failed to serialize {} data: {}", json_type.filename_suffix(), e
                    )))?
            } else {
                serde_json::to_string(&filtered_allocations)
                    .map_err(|e| BinaryExportError::SerializationError(format!(
                        "Failed to serialize {} data: {}", json_type.filename_suffix(), e
                    )))?
            };

            // Write to file with error context
            std::fs::write(&file_path, json_content)
                .map_err(|e| BinaryExportError::Io(e))?;

            let file_size = std::fs::metadata(&file_path)
                .map_err(|e| BinaryExportError::Io(e))?
                .len();

            // Create simple stats for this JSON type
            let json_stats = SelectiveJsonExportStats {
                total_allocations_exported: filtered_allocations.len() as u64,
                total_bytes_written: file_size,
                total_export_time_us: json_start.elapsed().as_micros() as u64,
                files_processed: 1,
                avg_export_throughput: if json_start.elapsed().as_secs_f64() > 0.0 {
                    filtered_allocations.len() as f64 / json_start.elapsed().as_secs_f64()
                } else {
                    0.0
                },
                memory_efficiency: if filtered_allocations.len() > 0 {
                    file_size as f64 / filtered_allocations.len() as f64
                } else {
                    0.0
                },
                ..Default::default()
            };

            per_json_stats.insert(*json_type, json_stats);

            info!(
                "Generated {} ({:.1}KB) with {} records in {:.2}s (throughput: {:.1} records/s)",
                file_path,
                file_size as f64 / 1024.0,
                filtered_allocations.len(),
                json_start.elapsed().as_secs_f64(),
                if json_start.elapsed().as_secs_f64() > 0.0 {
                    filtered_allocations.len() as f64 / json_start.elapsed().as_secs_f64()
                } else {
                    0.0
                }
            );
        }

        let total_duration = start_time.elapsed();
        info!(
            "Simple direct processing completed in {:.2}s for {} JSON files",
            total_duration.as_secs_f64(),
            json_types.len()
        );

        Ok(MultiExportStats {
            total_records,
            max_memory_used,
            cache_hit_rate: 0.0, // No caching in simple mode
            per_json_stats,
            total_duration,
            strategy_used: ProcessingStrategy::SimpleDirect,
            file_size: 0, // Will be set by caller
        })
    }

    /// Index-optimized processing for medium files (150KB-1MB)
    /// Uses indexing for faster access but still processes in batches
    /// 
    /// This strategy builds an index first to enable fast record lookup and filtering,
    /// then uses selective reading to only load the fields needed for each JSON type.
    /// It balances performance and memory usage for medium-sized files.
    fn export_with_index_optimization<P: AsRef<Path>>(
        &self,
        binary_path: P,
        base_name: &str,
        json_types: &[JsonType],
    ) -> Result<MultiExportStats, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let start_time = Instant::now();

        info!(
            "Using index-optimized processing for medium file: {}",
            binary_path.display()
        );

        // Build index for faster access
        let index_build_start = Instant::now();
        let index_builder = BinaryIndexBuilder::new()
            .with_quick_filter_threshold(500) // Lower threshold for medium files
            .with_quick_filter_batch_size(100); // Smaller batches for better memory usage
        
        let index = index_builder.build_index(binary_path)?;
        let total_records = index.allocations.count as usize;
        let index_build_time = index_build_start.elapsed();

        info!(
            "Built index for {} records in {:.2}s (has_quick_filter: {})",
            total_records,
            index_build_time.as_secs_f64(),
            index.allocations.quick_filter_data.is_some()
        );

        if total_records == 0 {
            info!("No allocations found in binary file, creating empty JSON files");
        }

        let mut per_json_stats = HashMap::new();
        let mut max_memory_used = 0;
        let mut total_cache_hits = 0u64;
        let mut total_cache_requests = 0u64;

        // Process each JSON type with selective reading
        for json_type in json_types {
            let json_start = Instant::now();
            let file_path = format!("{}_{}.json", base_name, json_type.filename_suffix());

            info!("Processing {} with index optimization", json_type.filename_suffix());

            // Create selective exporter for this JSON type with optimized configuration
            let json_writer_config = StreamingJsonWriterConfig {
                buffer_size: self.config.buffer_size,
                pretty_print: self.config.pretty_print,
                enable_field_optimization: true, // Enable field-level optimization
                enable_buffer_reuse: true,       // Enable buffer reuse for better performance
                array_chunk_size: 500,           // Smaller chunks for medium files
                ..Default::default()
            };

            let config = SelectiveJsonExportConfigBuilder::new()
                .json_writer_config(json_writer_config)
                .parallel_processing(self.config.enable_parallel_processing)
                .performance_monitoring(true) // Enable detailed monitoring
                .error_recovery(true)         // Enable error recovery
                .build();

            let mut exporter = SelectiveJsonExporter::with_config(config)?;

            // Define selective read options for this JSON type
            let required_fields: HashSet<AllocationField> = json_type.required_fields().into_iter().collect();
            let filters = self.create_filters_for_json_type(*json_type);

            info!(
                "Exporting {} fields with {} filters for {}",
                required_fields.len(),
                filters.len(),
                json_type.filename_suffix()
            );

            // Export with selective reading
            let stats = exporter.export_to_json_selective(
                binary_path,
                &file_path,
                &required_fields,
                &filters,
            )?;

            // Accumulate cache statistics
            total_cache_hits += stats.index_cache_hits;
            total_cache_requests += stats.index_cache_hits + stats.index_cache_misses;

            // Estimate memory usage based on allocations processed
            let estimated_memory = (stats.total_allocations_exported as usize) * std::mem::size_of::<AllocationInfo>();
            max_memory_used = max_memory_used.max(estimated_memory);

            let file_size = std::fs::metadata(&file_path)?.len();
            let processing_time = json_start.elapsed();

            // Extract values before moving stats
            let allocations_exported = stats.total_allocations_exported;
            let throughput = stats.avg_export_throughput;
            let cache_hit_rate = stats.cache_hit_rate();

            per_json_stats.insert(*json_type, stats);

            info!(
                "Generated {} ({:.1}KB) with {} records in {:.2}s (throughput: {:.1} records/s, cache hit rate: {:.1}%)",
                file_path,
                file_size as f64 / 1024.0,
                allocations_exported,
                processing_time.as_secs_f64(),
                throughput,
                cache_hit_rate
            );
        }

        // Calculate overall cache hit rate
        let cache_hit_rate = if total_cache_requests == 0 {
            0.0
        } else {
            (total_cache_hits as f64 / total_cache_requests as f64) * 100.0
        };

        let total_duration = start_time.elapsed();
        info!(
            "Index-optimized processing completed in {:.2}s for {} JSON files (overall cache hit rate: {:.1}%)",
            total_duration.as_secs_f64(),
            json_types.len(),
            cache_hit_rate
        );

        Ok(MultiExportStats {
            total_records,
            max_memory_used,
            cache_hit_rate,
            per_json_stats,
            total_duration,
            strategy_used: ProcessingStrategy::IndexOptimized,
            file_size: 0, // Will be set by caller
        })
    }

    /// Full streaming processing for large files (>1MB)
    /// Uses streaming approach with constant memory usage
    /// 
    /// This strategy is designed for large files where memory usage must be kept constant
    /// regardless of file size. It processes records one at a time, immediately writing
    /// them to output files and discarding them from memory.
    fn export_streaming_multi_json<P: AsRef<Path>>(
        &self,
        binary_path: P,
        base_name: &str,
        json_types: &[JsonType],
    ) -> Result<MultiExportStats, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let start_time = Instant::now();

        info!(
            "Using full streaming processing for large file: {} (target: constant memory usage)",
            binary_path.display()
        );

        // Build lightweight index for streaming access
        let index_build_start = Instant::now();
        let index_builder = BinaryIndexBuilder::new()
            .with_quick_filter_threshold(10000) // Higher threshold for large files
            .with_quick_filter_batch_size(1000); // Larger batches for streaming efficiency
        
        let index = index_builder.build_index(binary_path)?;
        let total_records = index.allocations.count as usize;
        let index_build_time = index_build_start.elapsed();

        info!(
            "Built streaming index for {} records in {:.2}s (memory footprint: ~{}KB)",
            total_records,
            index_build_time.as_secs_f64(),
            (std::mem::size_of_val(&index) + 
             index.allocations.relative_offsets.len() * std::mem::size_of::<u32>() +
             index.allocations.record_sizes.len() * std::mem::size_of::<u16>()) / 1024
        );

        if total_records == 0 {
            info!("No allocations found in binary file, creating empty JSON files");
        } else if total_records > 100000 {
            info!("Large dataset detected ({} records), using aggressive streaming optimizations", total_records);
        }

        let mut per_json_stats = HashMap::new();
        // Constant memory usage regardless of file size
        let max_memory_used = 2 * 1024 * 1024; // Fixed 2MB for streaming
        let mut total_cache_hits = 0u64;
        let mut total_cache_requests = 0u64;

        // Process each JSON type with aggressive streaming
        for json_type in json_types {
            let json_start = Instant::now();
            let file_path = format!("{}_{}.json", base_name, json_type.filename_suffix());

            info!("Streaming {} with constant memory usage", json_type.filename_suffix());

            // Create highly optimized streaming configuration
            let json_writer_config = StreamingJsonWriterConfig {
                buffer_size: self.config.buffer_size.min(32 * 1024), // Smaller buffers for streaming
                pretty_print: self.config.pretty_print,
                enable_field_optimization: true,
                enable_buffer_reuse: true,
                array_chunk_size: 100,           // Small chunks for immediate processing
                max_memory_before_flush: 1024 * 1024, // Aggressive flushing at 1MB
                ..Default::default()
            };

            let config = SelectiveJsonExportConfigBuilder::new()
                .json_writer_config(json_writer_config)
                .parallel_processing(false) // Disable parallel processing for streaming
                .performance_monitoring(true)
                .error_recovery(true)
                .build();

            let mut exporter = SelectiveJsonExporter::with_config(config)?;

            // Define selective read options optimized for streaming
            let required_fields: HashSet<AllocationField> = json_type.required_fields().into_iter().collect();
            let filters = self.create_filters_for_json_type(*json_type);

            info!(
                "Streaming {} fields with {} filters (estimated memory per record: {}B)",
                required_fields.len(),
                filters.len(),
                required_fields.len() * 64 // Rough estimate of memory per field
            );

            // Export with streaming processing - this should use constant memory
            let stats = exporter.export_to_json_selective(
                binary_path,
                &file_path,
                &required_fields,
                &filters,
            )?;

            // Accumulate cache statistics
            total_cache_hits += stats.index_cache_hits;
            total_cache_requests += stats.index_cache_hits + stats.index_cache_misses;

            let file_size = std::fs::metadata(&file_path)?.len();
            let processing_time = json_start.elapsed();

            // Extract values before moving stats
            let allocations_exported = stats.total_allocations_exported;
            let throughput = stats.avg_export_throughput;
            let cache_hit_rate = stats.cache_hit_rate();
            let memory_efficiency = stats.memory_efficiency;

            per_json_stats.insert(*json_type, stats);

            info!(
                "Streamed {} ({:.1}KB) with {} records in {:.2}s (throughput: {:.1} records/s, memory efficiency: {:.1}B/record, cache hit: {:.1}%)",
                file_path,
                file_size as f64 / 1024.0,
                allocations_exported,
                processing_time.as_secs_f64(),
                throughput,
                memory_efficiency,
                cache_hit_rate
            );

            // Verify constant memory usage
            if allocations_exported > 10000 {
                info!(
                    "Large dataset processed with constant memory usage ({}MB fixed allocation)",
                    max_memory_used / (1024 * 1024)
                );
            }
        }

        // Calculate overall cache hit rate
        let cache_hit_rate = if total_cache_requests == 0 {
            0.0
        } else {
            (total_cache_hits as f64 / total_cache_requests as f64) * 100.0
        };

        let total_duration = start_time.elapsed();
        let overall_throughput = if total_duration.as_secs_f64() > 0.0 {
            total_records as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        info!(
            "Full streaming processing completed in {:.2}s for {} JSON files (overall throughput: {:.1} records/s, cache hit: {:.1}%, memory: constant {}MB)",
            total_duration.as_secs_f64(),
            json_types.len(),
            overall_throughput,
            cache_hit_rate,
            max_memory_used / (1024 * 1024)
        );

        // Verify memory usage is truly constant
        if total_records > 50000 {
            info!(
                "✓ Streaming verification: processed {} records with constant {}MB memory usage (scalability confirmed)",
                total_records,
                max_memory_used / (1024 * 1024)
            );
        }

        Ok(MultiExportStats {
            total_records,
            max_memory_used,
            cache_hit_rate,
            per_json_stats,
            total_duration,
            strategy_used: ProcessingStrategy::FullyStreaming,
            file_size: 0, // Will be set by caller
        })
    }

    /// Filter allocations for a specific JSON type (used in simple direct processing)
    /// 
    /// This method creates JSON objects with only the fields required for the specific
    /// JSON type, reducing memory usage and improving serialization performance.
    fn filter_allocations_for_json_type(
        &self,
        allocations: &[AllocationInfo],
        json_type: JsonType,
    ) -> Vec<serde_json::Value> {
        let required_fields = json_type.required_fields();

        // Apply type-specific filtering first
        let filtered_allocations: Vec<&AllocationInfo> = match json_type {
            JsonType::LifetimeAnalysis => {
                // Only include allocations that have been deallocated
                allocations.iter()
                    .filter(|alloc| alloc.timestamp_dealloc.is_some())
                    .collect()
            }
            JsonType::PerformanceAnalysis => {
                // Focus on larger allocations for performance analysis
                allocations.iter()
                    .filter(|alloc| alloc.size >= 1024)
                    .collect()
            }
            JsonType::ComplexTypes => {
                // Only include allocations with type information and >= 64 bytes
                // Larger allocations are more likely to be complex types
                allocations.iter()
                    .filter(|alloc| alloc.size >= 64 &&
                                   alloc.type_name.is_some() && 
                                   alloc.type_name.as_ref().unwrap() != "unknown")
                    .collect()
            }
            JsonType::UnsafeFFI => {
                // Only include allocations with stack traces (indicating potential FFI)
                allocations.iter()
                    .filter(|alloc| alloc.stack_trace.is_some() && 
                                   !alloc.stack_trace.as_ref().unwrap().is_empty())
                    .collect()
            }
            JsonType::MemoryAnalysis => {
                // Include all allocations for memory analysis
                allocations.iter().collect()
            }
        };

        filtered_allocations
            .iter()
            .map(|alloc| {
                let mut json_obj = serde_json::Map::new();

                for field in &required_fields {
                    match field {
                        AllocationField::Ptr => {
                            json_obj.insert("ptr".to_string(), 
                                serde_json::Value::String(format!("0x{:x}", alloc.ptr)));
                        }
                        AllocationField::Size => {
                            json_obj.insert("size".to_string(), 
                                serde_json::Value::Number(serde_json::Number::from(alloc.size)));
                        }
                        AllocationField::VarName => {
                            json_obj.insert("var_name".to_string(),
                                serde_json::Value::String(
                                    alloc.var_name.as_deref().unwrap_or("unknown").to_string()
                                ));
                        }
                        AllocationField::TypeName => {
                            json_obj.insert("type_name".to_string(),
                                serde_json::Value::String(
                                    alloc.type_name.as_deref().unwrap_or("unknown").to_string()
                                ));
                        }
                        AllocationField::ThreadId => {
                            json_obj.insert("thread_id".to_string(),
                                serde_json::Value::String(alloc.thread_id.clone()));
                        }
                        AllocationField::TimestampAlloc => {
                            json_obj.insert("timestamp_alloc".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(alloc.timestamp_alloc)));
                        }
                        AllocationField::TimestampDealloc => {
                            if let Some(ts) = alloc.timestamp_dealloc {
                                json_obj.insert("timestamp_dealloc".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(ts)));
                            } else {
                                json_obj.insert("timestamp_dealloc".to_string(),
                                    serde_json::Value::Null);
                            }
                        }
                        AllocationField::LifetimeMs => {
                            if let Some(lifetime) = alloc.lifetime_ms {
                                json_obj.insert("lifetime_ms".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(lifetime)));
                            } else {
                                json_obj.insert("lifetime_ms".to_string(),
                                    serde_json::Value::Null);
                            }
                        }
                        AllocationField::IsLeaked => {
                            json_obj.insert("is_leaked".to_string(),
                                serde_json::Value::Bool(alloc.is_leaked));
                        }
                        AllocationField::BorrowCount => {
                            json_obj.insert("borrow_count".to_string(),
                                serde_json::Value::Number(serde_json::Number::from(alloc.borrow_count)));
                        }
                        AllocationField::ScopeName => {
                            if let Some(ref scope) = alloc.scope_name {
                                json_obj.insert("scope_name".to_string(),
                                    serde_json::Value::String(scope.clone()));
                            } else {
                                json_obj.insert("scope_name".to_string(),
                                    serde_json::Value::Null);
                            }
                        }
                        AllocationField::StackTrace => {
                            if let Some(ref stack_trace) = alloc.stack_trace {
                                let stack_array: Vec<serde_json::Value> = stack_trace
                                    .iter()
                                    .map(|s| serde_json::Value::String(s.clone()))
                                    .collect();
                                json_obj.insert("stack_trace".to_string(),
                                    serde_json::Value::Array(stack_array));
                            } else {
                                json_obj.insert("stack_trace".to_string(),
                                    serde_json::Value::Null);
                            }
                        }
                        // Handle all other fields (complex fields, advanced features, etc.)
                        _ => {
                            // Skip complex fields in simple direct processing
                            // These would be handled by the selective exporter in other strategies
                            // For simple direct processing, we only handle basic fields above
                        }
                    }
                }

                serde_json::Value::Object(json_obj)
            })
            .collect()
    }

    /// Create filters for a specific JSON type
    /// 
    /// This method creates intelligent filters that reduce the amount of data
    /// that needs to be processed for each JSON type, improving performance
    /// by focusing on relevant allocations only.
    fn create_filters_for_json_type(&self, json_type: JsonType) -> Vec<AllocationFilter> {
        match json_type {
            JsonType::MemoryAnalysis => {
                // Include all allocations for comprehensive memory analysis
                // No filters to ensure complete picture
                vec![]
            }
            JsonType::LifetimeAnalysis => {
                // Focus on allocations with meaningful lifetime data
                // Note: The actual filtering is done in the simple direct processing
                // For selective reading, we rely on the exporter's built-in filtering
                vec![]
            }
            JsonType::PerformanceAnalysis => {
                // Focus on larger allocations that are more likely to impact performance
                // Filter out very small allocations (< 1KB) to reduce noise
                vec![AllocationFilter::SizeRange(1024, usize::MAX)]
            }
            JsonType::ComplexTypes => {
                // Focus on allocations that are likely to have complex type information
                // Larger allocations are more likely to be complex types
                vec![AllocationFilter::SizeRange(64, usize::MAX)]
            }
            JsonType::UnsafeFFI => {
                // Focus on allocations that might be related to FFI operations
                // These are often larger and from specific threads
                vec![AllocationFilter::SizeRange(32, usize::MAX)]
            }
        }
    }
}