//! Complex type analysis data export optimization
//!
//! This module provides optimized export functionality specifically for complex type analysis data.
//! It separates complex type data into dedicated JSON files to improve performance.

use crate::analysis::ComprehensiveAnalysisReport;
use crate::core::types::{AllocationInfo, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advanced_types::{
        AdvancedTypeAnalysisReport, AdvancedTypeStatistics, LatencyCategory, PerformanceSummary,
    };
    use crate::analysis::{
        AsyncPatternAnalysis, BorrowPatternAnalysis, ClosureAnalysisReport, GenericStatistics,
        LifecycleAnalysisReport,
    };
    use crate::closure_analysis::{ClosureType, CreationContext, DetectedClosure, MemoryImpact};
    use crate::lifecycle_analysis::DropEvent;
    use std::collections::HashMap;

    #[test]
    fn test_complex_type_export_config_default() {
        let config = ComplexTypeExportConfig::default();
        assert!(config.separate_complex_types);
        assert!(!config.compress_data);
        assert_eq!(config.chunk_size, 1000);
        assert!(!config.pretty_format);
    }

    #[test]
    fn test_export_statistics_default_values() {
        let stats = ExportStatistics {
            total_time_ms: 0,
            main_file_size: 0,
            complex_files_size: 0,
            allocations_processed: 0,
            performance_improvement: 0.0,
        };
        assert_eq!(stats.total_time_ms, 0);
        assert_eq!(stats.main_file_size, 0);
        assert_eq!(stats.complex_files_size, 0);
        assert_eq!(stats.allocations_processed, 0);
        assert_eq!(stats.performance_improvement, 0.0);
    }

    #[test]
    fn test_lightweight_export_data_fields() {
        let data = LightweightExportData {
            total_memory: 1000,
            peak_memory: 2000,
            active_allocations: 500,
            allocation_summary: AllocationSummary {
                total_allocations: 100,
                active_allocations: 50,
                total_memory: 1000,
                peak_memory: 2000,
                allocation_count_by_size: HashMap::new(),
            },
            memory_by_type: vec![],
            complex_type_files: ComplexTypeFileReferences {
                complex_types_file: None,
                borrow_analysis_file: None,
                generic_analysis_file: None,
                async_analysis_file: None,
                closure_analysis_file: None,
                lifecycle_analysis_file: None,
            },
            metadata: ExportMetadata {
                export_timestamp: 1234567890,
                export_version: "1.0.0".to_string(),
                data_format_version: "v1".to_string(),
                performance_optimized: true,
            },
        };
        assert_eq!(data.total_memory, 1000);
        assert_eq!(data.peak_memory, 2000);
        assert_eq!(data.active_allocations, 500);
    }

    #[test]
    fn test_allocation_summary_fields() {
        let summary = AllocationSummary {
            total_allocations: 100,
            active_allocations: 50,
            total_memory: 1000,
            peak_memory: 2000,
            allocation_count_by_size: HashMap::new(),
        };
        assert_eq!(summary.total_allocations, 100);
        assert_eq!(summary.active_allocations, 50);
        assert_eq!(summary.total_memory, 1000);
        assert_eq!(summary.peak_memory, 2000);
    }

    #[test]
    fn test_simple_type_usage_fields() {
        let usage = SimpleTypeUsage {
            type_name: "Vec<i32>".to_string(),
            total_size: 1000,
            allocation_count: 10,
            category: "Collection".to_string(),
        };
        assert_eq!(usage.type_name, "Vec<i32>");
        assert_eq!(usage.total_size, 1000);
        assert_eq!(usage.allocation_count, 10);
        assert_eq!(usage.category, "Collection");
    }

    #[test]
    fn test_complex_type_file_references_fields() {
        let references = ComplexTypeFileReferences {
            complex_types_file: Some("complex_types.json".to_string()),
            borrow_analysis_file: Some("borrow_analysis.json".to_string()),
            generic_analysis_file: Some("generic_analysis.json".to_string()),
            async_analysis_file: Some("async_analysis.json".to_string()),
            closure_analysis_file: Some("closure_analysis.json".to_string()),
            lifecycle_analysis_file: Some("lifecycle_analysis.json".to_string()),
        };
        assert_eq!(
            references.complex_types_file,
            Some("complex_types.json".to_string())
        );
        assert_eq!(
            references.borrow_analysis_file,
            Some("borrow_analysis.json".to_string())
        );
        assert_eq!(
            references.generic_analysis_file,
            Some("generic_analysis.json".to_string())
        );
        assert_eq!(
            references.async_analysis_file,
            Some("async_analysis.json".to_string())
        );
        assert_eq!(
            references.closure_analysis_file,
            Some("closure_analysis.json".to_string())
        );
        assert_eq!(
            references.lifecycle_analysis_file,
            Some("lifecycle_analysis.json".to_string())
        );
    }

    #[test]
    fn test_export_metadata_fields() {
        let metadata = ExportMetadata {
            export_timestamp: 1234567890,
            export_version: "1.0.0".to_string(),
            data_format_version: "v1".to_string(),
            performance_optimized: true,
        };
        assert_eq!(metadata.export_timestamp, 1234567890);
        assert_eq!(metadata.export_version, "1.0.0");
        assert_eq!(metadata.data_format_version, "v1");
        assert_eq!(metadata.performance_optimized, true);
    }

    #[test]
    fn test_should_export_complex_types() {
        // Create a mock AdvancedTypeAnalysisReport with some data
        let report = AdvancedTypeAnalysisReport {
            by_category: HashMap::new(),
            all_issues: vec![],
            performance_summary: PerformanceSummary {
                total_overhead_factor: 0.0,
                total_memory_overhead: 0,
                lock_free_percentage: 0.0,
                dominant_latency_category: LatencyCategory::Immediate,
            },
            statistics: AdvancedTypeStatistics {
                by_category: HashMap::new(),
                by_issue_severity: HashMap::new(),
                by_latency_category: HashMap::new(),
                total_advanced_types: 0,
            },
        };
        // With empty data, should not export
        assert_eq!(should_export_complex_types(&report), false);

        // Create a report with some data
        let mut by_category = HashMap::new();
        by_category.insert(
            "InteriorMutability".to_string(),
            vec![], // Using string keys as per the actual implementation
        );
        let report = AdvancedTypeAnalysisReport {
            by_category,
            all_issues: vec![],
            performance_summary: PerformanceSummary {
                total_overhead_factor: 0.0,
                total_memory_overhead: 0,
                lock_free_percentage: 0.0,
                dominant_latency_category: LatencyCategory::Immediate,
            },
            statistics: AdvancedTypeStatistics {
                by_category: HashMap::new(),
                by_issue_severity: HashMap::new(),
                by_latency_category: HashMap::new(),
                total_advanced_types: 1, // With at least one type, should export
            },
        };
        // With data, should export
        assert_eq!(should_export_complex_types(&report), true);
    }

    #[test]
    fn test_should_export_borrow_analysis() {
        let analysis = BorrowPatternAnalysis {
            patterns: vec![], // Using Vec instead of HashMap
            total_events: 0,
            analysis_timestamp: 0,
        };
        // With empty data, should not export
        assert_eq!(should_export_borrow_analysis(&analysis), false);

        // Create analysis with some data
        let analysis = BorrowPatternAnalysis {
            patterns: vec![], // Even with empty vec, if total_events > 0, should export
            total_events: 5,
            analysis_timestamp: 0,
        };
        // With events, should export
        assert_eq!(should_export_borrow_analysis(&analysis), true);
    }

    #[test]
    fn test_should_export_generic_analysis() {
        let analysis = GenericStatistics {
            total_instances: 0,
            unique_base_types: 0,
            total_instantiations: 0,
            constraint_violations: 0,
            most_used_types: vec![],
        };
        // With zero instances, should not export
        assert_eq!(should_export_generic_analysis(&analysis), false);

        let analysis = GenericStatistics {
            total_instances: 5,
            unique_base_types: 2,
            total_instantiations: 10,
            constraint_violations: 1,
            most_used_types: vec![("Vec".to_string(), 3)],
        };
        // With instances, should export
        assert_eq!(should_export_generic_analysis(&analysis), true);
    }

    #[test]
    fn test_should_export_async_analysis() {
        let analysis = AsyncPatternAnalysis {
            patterns: vec![], // Using Vec instead of HashMap
            total_futures_analyzed: 0,
            analysis_timestamp: 0,
        };
        // With empty data, should not export
        assert!(!should_export_async_analysis(&analysis));

        // Create analysis with some data
        let analysis = AsyncPatternAnalysis {
            patterns: vec![], // Even with empty vec, if total_futures_analyzed > 0, should export
            total_futures_analyzed: 5,
            analysis_timestamp: 0,
        };
        // With data, should export
        assert!(should_export_async_analysis(&analysis));
    }

    #[test]
    fn test_should_export_closure_analysis() {
        let analysis = ClosureAnalysisReport {
            capture_statistics: Default::default(),
            lifetime_analysis: Default::default(),
            analysis_timestamp: 0,
            detected_closures: vec![],
            optimization_suggestions: vec![],
        };
        // With empty data, should not export
        assert!(!should_export_closure_analysis(&analysis));

        // Create analysis with some data
        let analysis = ClosureAnalysisReport {
            capture_statistics: Default::default(),
            lifetime_analysis: Default::default(),
            analysis_timestamp: 0,
            detected_closures: vec![DetectedClosure {
                ptr: 0x1234,
                type_name: "test_closure".to_string(),
                size: 32,
                estimated_captures: 1,
                closure_type: ClosureType::Unknown,
                creation_context: CreationContext {
                    scope_name: Some("test_scope".to_string()),
                    thread_id: "main".to_string(),
                    timestamp: 1234567890,
                },
                memory_impact: MemoryImpact::Minimal,
            }], // With at least one closure, should export
            optimization_suggestions: vec![],
        };
        // With data, should export
        assert!(should_export_closure_analysis(&analysis));
    }

    #[test]
    fn test_should_export_lifecycle_analysis() {
        let analysis = LifecycleAnalysisReport {
            borrow_analysis: Default::default(),
            closure_captures: vec![],
            analysis_timestamp: 0,
            drop_events: vec![],
            raii_patterns: vec![],
        };
        // With empty data, should not export
        assert!(!should_export_lifecycle_analysis(&analysis));

        // Create analysis with some data
        let analysis = LifecycleAnalysisReport {
            borrow_analysis: Default::default(),
            closure_captures: vec![],
            analysis_timestamp: 0,
            drop_events: vec![DropEvent {
                ptr: 0x1234,
                type_name: "TestType".to_string(),
                timestamp: 1234567890,
                custom_drop: false,
                thread_id: "main".to_string(),
                call_stack: vec!["frame1".to_string(), "frame2".to_string()],
            }], // With at least one drop event, should export
            raii_patterns: vec![],
        };
        // With data, should export
        assert!(should_export_lifecycle_analysis(&analysis));
    }

    #[test]
    fn test_create_size_distribution() {
        let allocations = vec![
            AllocationInfo {
                ptr: 0x12345678,
                size: 32,
                type_name: Some("i32".to_string()),
                var_name: Some("test_var".to_string()),
                scope_name: Some("test_scope".to_string()),
                timestamp_alloc: 1234567890,
                timestamp_dealloc: Some(1234567900),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: Some(10),
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
            },
            AllocationInfo {
                ptr: 0x23456789,
                size: 512,
                type_name: Some("Vec<i32>".to_string()),
                var_name: Some("test_vec".to_string()),
                scope_name: Some("test_scope".to_string()),
                timestamp_alloc: 1234567890,
                timestamp_dealloc: Some(1234567900),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: Some(10),
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
            },
        ];

        let distribution = create_size_distribution(&allocations);
        assert_eq!(distribution.get("small"), Some(&1));
        assert_eq!(distribution.get("medium"), Some(&1));
    }

    #[test]
    fn test_create_simple_type_usage() {
        let allocations = vec![
            AllocationInfo {
                ptr: 0x12345678,
                size: 32,
                type_name: Some("i32".to_string()),
                var_name: Some("test_var".to_string()),
                scope_name: Some("test_scope".to_string()),
                timestamp_alloc: 1234567890,
                timestamp_dealloc: Some(1234567900),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: Some(10),
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
            },
            AllocationInfo {
                ptr: 0x23456789,
                size: 32,
                type_name: Some("i32".to_string()),
                var_name: Some("test_var2".to_string()),
                scope_name: Some("test_scope".to_string()),
                timestamp_alloc: 1234567890,
                timestamp_dealloc: Some(1234567900),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: Some(10),
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
            },
        ];

        let usage = create_simple_type_usage(&allocations);
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].type_name, "i32");
        assert_eq!(usage[0].total_size, 64);
        assert_eq!(usage[0].allocation_count, 2);
    }

    #[test]
    fn test_categorize_simple_type() {
        assert_eq!(categorize_simple_type("Vec<i32>"), "Collections");
        assert_eq!(
            categorize_simple_type("HashMap<String, i32>"),
            "Collections"
        );
        assert_eq!(categorize_simple_type("String"), "Strings");
        assert_eq!(categorize_simple_type("&str"), "Strings");
        assert_eq!(categorize_simple_type("i32"), "Primitives");
        assert_eq!(categorize_simple_type("u64"), "Primitives");
        assert_eq!(categorize_simple_type("CustomType"), "Other");
    }

    #[test]
    fn test_get_file_size_nonexistent() {
        let path = Path::new("nonexistent_file.txt");
        assert_eq!(get_file_size(path), None);
    }

    #[test]
    fn test_calculate_complex_files_size() {
        let result = ComplexTypeExportResult {
            main_file: "main.json".to_string(),
            complex_types_file: Some("complex_types.json".to_string()),
            borrow_analysis_file: Some("borrow_analysis.json".to_string()),
            generic_analysis_file: Some("generic_analysis.json".to_string()),
            async_analysis_file: Some("async_analysis.json".to_string()),
            closure_analysis_file: Some("closure_analysis.json".to_string()),
            lifecycle_analysis_file: Some("lifecycle_analysis.json".to_string()),
            export_stats: ExportStatistics {
                total_time_ms: 0,
                main_file_size: 0,
                complex_files_size: 0,
                allocations_processed: 0,
                performance_improvement: 0.0,
            },
        };
        // Since files don't exist, size should be 0
        assert_eq!(calculate_complex_files_size(&result), 0);
    }

    #[test]
    fn test_calculate_performance_improvement() {
        let result = ComplexTypeExportResult {
            main_file: "main.json".to_string(),
            complex_types_file: Some("complex_types.json".to_string()),
            borrow_analysis_file: None,
            generic_analysis_file: None,
            async_analysis_file: None,
            closure_analysis_file: None,
            lifecycle_analysis_file: None,
            export_stats: ExportStatistics {
                total_time_ms: 0,
                main_file_size: 500,
                complex_files_size: 500,
                allocations_processed: 0,
                performance_improvement: 0.0,
            },
        };
        // With equal file sizes, improvement should be around 35% (70% * 0.5)
        let improvement = calculate_performance_improvement(&result);
        assert!(improvement > 34.0 && improvement < 36.0);
    }
}

/// Complex type export configuration
#[derive(Debug, Clone)]
pub struct ComplexTypeExportConfig {
    /// Whether to export complex type data to separate files
    pub separate_complex_types: bool,
    /// Whether to compress complex type data
    pub compress_data: bool,
    /// Maximum items per chunk for streaming export
    pub chunk_size: usize,
    /// Whether to use pretty JSON formatting
    pub pretty_format: bool,
}

impl Default for ComplexTypeExportConfig {
    fn default() -> Self {
        Self {
            separate_complex_types: true,
            compress_data: false,
            chunk_size: 1000,
            pretty_format: false, // Disable for performance
        }
    }
}

/// Separated complex type export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeExportResult {
    /// Main export file path
    pub main_file: String,
    /// Complex type analysis file path
    pub complex_types_file: Option<String>,
    /// Borrow analysis file path
    pub borrow_analysis_file: Option<String>,
    /// Generic analysis file path
    pub generic_analysis_file: Option<String>,
    /// Async analysis file path
    pub async_analysis_file: Option<String>,
    /// Closure analysis file path
    pub closure_analysis_file: Option<String>,
    /// Lifecycle analysis file path
    pub lifecycle_analysis_file: Option<String>,
    /// Export statistics
    pub export_stats: ExportStatistics,
}

/// Export performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatistics {
    /// Total export time in milliseconds
    pub total_time_ms: u64,
    /// Main file size in bytes
    pub main_file_size: u64,
    /// Complex type files total size in bytes
    pub complex_files_size: u64,
    /// Number of allocations processed
    pub allocations_processed: usize,
    /// Performance improvement percentage
    pub performance_improvement: f64,
}

/// Lightweight main export data (without complex type details)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightExportData {
    /// Basic memory statistics (simplified for performance)
    pub total_memory: usize,
    /// peak memory
    pub peak_memory: usize,
    /// active allocations
    pub active_allocations: usize,
    /// Basic allocation summary (without complex type details)
    pub allocation_summary: AllocationSummary,
    /// Memory usage by type (simplified)
    pub memory_by_type: Vec<SimpleTypeUsage>,
    /// References to complex type analysis files
    pub complex_type_files: ComplexTypeFileReferences,
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Simplified allocation summary for main export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSummary {
    /// total allocations
    pub total_allocations: usize,
    /// active allocations
    pub active_allocations: usize,
    /// total memory
    pub total_memory: usize,
    /// peak memory
    pub peak_memory: usize,
    /// allocation count by size
    pub allocation_count_by_size: HashMap<String, usize>,
}

/// Simplified type usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTypeUsage {
    /// type name
    pub type_name: String,
    /// total size
    pub total_size: usize,
    /// allocation count
    pub allocation_count: usize,
    /// category
    pub category: String,
}

/// References to complex type analysis files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeFileReferences {
    /// complex types file path
    pub complex_types_file: Option<String>,
    /// borrow analysis file path
    pub borrow_analysis_file: Option<String>,
    /// generic analysis file path
    pub generic_analysis_file: Option<String>,
    /// async analysis file path
    pub async_analysis_file: Option<String>,
    /// closure analysis file path
    pub closure_analysis_file: Option<String>,
    /// lifecycle analysis file path
    pub lifecycle_analysis_file: Option<String>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// export timestamp
    pub export_timestamp: u64,
    /// export version
    pub export_version: String,
    /// data format version
    pub data_format_version: String,
    /// performance optimized
    pub performance_optimized: bool,
}

/// Export comprehensive analysis with complex type separation
pub fn export_comprehensive_analysis_optimized<P: AsRef<Path>>(
    report: &ComprehensiveAnalysisReport,
    allocations: &[AllocationInfo],
    base_path: P,
    config: &ComplexTypeExportConfig,
) -> TrackingResult<ComplexTypeExportResult> {
    let start_time = std::time::Instant::now();
    let base_path = base_path.as_ref();
    let base_name = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("memory_analysis");

    let mut result = ComplexTypeExportResult {
        main_file: format!("{base_name}.json"),
        complex_types_file: None,
        borrow_analysis_file: None,
        generic_analysis_file: None,
        async_analysis_file: None,
        closure_analysis_file: None,
        lifecycle_analysis_file: None,
        export_stats: ExportStatistics {
            total_time_ms: 0,
            main_file_size: 0,
            complex_files_size: 0,
            allocations_processed: allocations.len(),
            performance_improvement: 0.0,
        },
    };

    // 1. Export main lightweight data
    let main_data = create_lightweight_export_data(report, allocations, &result)?;
    let main_file_path = base_path.with_extension("json");
    export_json_data(&main_data, &main_file_path, config)?;
    result.main_file = main_file_path.to_string_lossy().to_string();

    if config.separate_complex_types {
        // 2. Export complex type analysis
        if should_export_complex_types(&report.advanced_type_analysis) {
            let complex_file_path =
                base_path.with_file_name(format!("{base_name}_complex_types.json"));
            export_json_data(&report.advanced_type_analysis, &complex_file_path, config)?;
            result.complex_types_file = Some(complex_file_path.to_string_lossy().to_string());
        }

        // 3. Export borrow analysis
        if should_export_borrow_analysis(&report.borrow_analysis) {
            let borrow_file_path =
                base_path.with_file_name(format!("{base_name}_borrow_analysis.json"));
            export_json_data(&report.borrow_analysis, &borrow_file_path, config)?;
            result.borrow_analysis_file = Some(borrow_file_path.to_string_lossy().to_string());
        }

        // 4. Export generic analysis
        if should_export_generic_analysis(&report.generic_analysis) {
            let generic_file_path =
                base_path.with_file_name(format!("{base_name}_generic_analysis.json"));
            export_json_data(&report.generic_analysis, &generic_file_path, config)?;
            result.generic_analysis_file = Some(generic_file_path.to_string_lossy().to_string());
        }

        // 5. Export async analysis
        if should_export_async_analysis(&report.async_analysis) {
            let async_file_path =
                base_path.with_file_name(format!("{base_name}_async_analysis.json"));
            export_json_data(&report.async_analysis, &async_file_path, config)?;
            result.async_analysis_file = Some(async_file_path.to_string_lossy().to_string());
        }

        // 6. Export closure analysis
        if should_export_closure_analysis(&report.closure_analysis) {
            let closure_file_path =
                base_path.with_file_name(format!("{base_name}_closure_analysis.json"));
            export_json_data(&report.closure_analysis, &closure_file_path, config)?;
            result.closure_analysis_file = Some(closure_file_path.to_string_lossy().to_string());
        }

        // 7. Export lifecycle analysis
        if should_export_lifecycle_analysis(&report.lifecycle_analysis) {
            let lifecycle_file_path =
                base_path.with_file_name(format!("{base_name}_lifecycle_analysis.json"));
            export_json_data(&report.lifecycle_analysis, &lifecycle_file_path, config)?;
            result.lifecycle_analysis_file =
                Some(lifecycle_file_path.to_string_lossy().to_string());
        }
    }

    // Calculate export statistics
    let total_time = start_time.elapsed();
    result.export_stats.total_time_ms = total_time.as_millis() as u64;
    result.export_stats.main_file_size = get_file_size(&main_file_path).unwrap_or(0);
    result.export_stats.complex_files_size = calculate_complex_files_size(&result);
    result.export_stats.performance_improvement = calculate_performance_improvement(&result);

    tracing::info!(
        "✅ Complex type export completed in {}ms",
        result.export_stats.total_time_ms
    );
    tracing::info!(
        "📊 Main file: {} bytes, Complex files: {} bytes",
        result.export_stats.main_file_size,
        result.export_stats.complex_files_size
    );

    Ok(result)
}

/// Create lightweight export data without complex type details
fn create_lightweight_export_data(
    report: &ComprehensiveAnalysisReport,
    allocations: &[AllocationInfo],
    result: &ComplexTypeExportResult,
) -> TrackingResult<LightweightExportData> {
    // Create simplified allocation summary
    let allocation_summary = AllocationSummary {
        total_allocations: allocations.len(),
        active_allocations: allocations
            .iter()
            .filter(|a| a.timestamp_dealloc.is_none())
            .count(),
        total_memory: allocations.iter().map(|a| a.size).sum(),
        peak_memory: report.memory_stats.peak_memory,
        allocation_count_by_size: create_size_distribution(allocations),
    };

    // Create simplified type usage
    let memory_by_type = create_simple_type_usage(allocations);

    // Create file references
    let complex_type_files = ComplexTypeFileReferences {
        complex_types_file: result.complex_types_file.clone(),
        borrow_analysis_file: result.borrow_analysis_file.clone(),
        generic_analysis_file: result.generic_analysis_file.clone(),
        async_analysis_file: result.async_analysis_file.clone(),
        closure_analysis_file: result.closure_analysis_file.clone(),
        lifecycle_analysis_file: result.lifecycle_analysis_file.clone(),
    };

    // Create metadata
    let metadata = ExportMetadata {
        export_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        export_version: "1.0.0".to_string(),
        data_format_version: "complex_type_optimized_v1".to_string(),
        performance_optimized: true,
    };

    Ok(LightweightExportData {
        total_memory: report.memory_stats.total_allocated,
        peak_memory: report.memory_stats.peak_memory,
        active_allocations: report.memory_stats.active_allocations,
        allocation_summary,
        memory_by_type,
        complex_type_files,
        metadata,
    })
}

/// Export JSON data with configuration options
fn export_json_data<T: Serialize>(
    data: &T,
    path: &Path,
    config: &ComplexTypeExportConfig,
) -> TrackingResult<()> {
    let json_string = if config.pretty_format {
        serde_json::to_string_pretty(data)
    } else {
        serde_json::to_string(data)
    }
    .map_err(|e| crate::core::types::TrackingError::SerializationError(e.to_string()))?;

    std::fs::write(path, json_string)
        .map_err(|e| crate::core::types::TrackingError::IoError(e.to_string()))?;

    Ok(())
}

// Helper functions for determining what to export
fn should_export_complex_types(
    analysis: &crate::advanced_types::AdvancedTypeAnalysisReport,
) -> bool {
    analysis.statistics.total_advanced_types > 0
}

fn should_export_borrow_analysis(analysis: &crate::analysis::BorrowPatternAnalysis) -> bool {
    analysis.total_events > 0
}

fn should_export_generic_analysis(analysis: &crate::analysis::GenericStatistics) -> bool {
    analysis.total_instances > 0
}

fn should_export_async_analysis(analysis: &crate::analysis::AsyncPatternAnalysis) -> bool {
    analysis.total_futures_analyzed > 0
}

fn should_export_closure_analysis(analysis: &crate::analysis::ClosureAnalysisReport) -> bool {
    !analysis.detected_closures.is_empty()
}

fn should_export_lifecycle_analysis(analysis: &crate::analysis::LifecycleAnalysisReport) -> bool {
    !analysis.drop_events.is_empty() || !analysis.raii_patterns.is_empty()
}

// Helper functions for statistics
fn create_size_distribution(allocations: &[AllocationInfo]) -> HashMap<String, usize> {
    let mut distribution = HashMap::new();
    for alloc in allocations {
        let size_category = match alloc.size {
            0..=64 => "small",
            65..=1024 => "medium",
            1025..=65536 => "large",
            _ => "huge",
        };
        *distribution.entry(size_category.to_string()).or_insert(0) += 1;
    }
    distribution
}

fn create_simple_type_usage(allocations: &[AllocationInfo]) -> Vec<SimpleTypeUsage> {
    let mut type_usage: HashMap<String, (usize, usize)> = HashMap::new();

    for alloc in allocations {
        let type_name = alloc.type_name.as_deref().unwrap_or("Unknown").to_string();
        let entry = type_usage.entry(type_name.clone()).or_insert((0, 0));
        entry.0 += alloc.size;
        entry.1 += 1;
    }

    type_usage
        .into_iter()
        .map(|(type_name, (total_size, count))| SimpleTypeUsage {
            type_name: type_name.clone(),
            total_size,
            allocation_count: count,
            category: categorize_simple_type(&type_name),
        })
        .collect()
}

fn categorize_simple_type(type_name: &str) -> String {
    if type_name.contains("Vec") || type_name.contains("HashMap") {
        "Collections".to_string()
    } else if type_name.contains("String") || type_name.contains("str") {
        "Strings".to_string()
    } else if type_name.contains("i32") || type_name.contains("u64") {
        "Primitives".to_string()
    } else {
        "Other".to_string()
    }
}

fn get_file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}

fn calculate_complex_files_size(result: &ComplexTypeExportResult) -> u64 {
    let mut total = 0;

    if let Some(ref path) = result.complex_types_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }
    if let Some(ref path) = result.borrow_analysis_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }
    if let Some(ref path) = result.generic_analysis_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }
    if let Some(ref path) = result.async_analysis_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }
    if let Some(ref path) = result.closure_analysis_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }
    if let Some(ref path) = result.lifecycle_analysis_file {
        total += get_file_size(Path::new(path)).unwrap_or(0);
    }

    total
}

fn calculate_performance_improvement(result: &ComplexTypeExportResult) -> f64 {
    // Estimate performance improvement based on file size reduction
    let total_size = result.export_stats.main_file_size + result.export_stats.complex_files_size;
    if total_size > 0 {
        let main_ratio = result.export_stats.main_file_size as f64 / total_size as f64;
        // Assume 60-80% improvement when main file is much smaller
        (1.0 - main_ratio) * 70.0
    } else {
        0.0
    }
}
