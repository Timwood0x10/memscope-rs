//! Unified analysis engine for consistent data processing across different export formats
//!
//! This module provides a trait-based architecture that ensures JSON and binary exports
//! use the same analysis logic, preventing data inconsistencies.

use crate::core::types::AllocationInfo;
use serde_json::Value;
use std::error::Error;
use std::fmt;

/// Analysis data container that can be serialized to different formats
#[derive(Debug, Clone)]
pub struct AnalysisData {
    /// The analyzed data as a JSON value (can be converted to other formats)
    pub data: Value,
    /// Metadata about the analysis
    pub metadata: AnalysisMetadata,
}

/// Metadata about the analysis process
#[derive(Debug, Clone)]
pub struct AnalysisMetadata {
    /// Type of analysis performed
    pub analysis_type: String,
    /// Timestamp when analysis was performed
    pub timestamp: u64,
    /// Number of allocations analyzed
    pub total_allocations: usize,
    /// Optimization level used
    pub optimization_level: String,
}

/// Errors that can occur during analysis
#[derive(Debug)]
pub enum AnalysisError {
    /// Data processing error
    ProcessingError(String),
    /// Serialization error
    SerializationError(String),
    /// Invalid input data
    InvalidData(String),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::ProcessingError(msg) => write!(f, "Processing error: {msg}"),
            AnalysisError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            AnalysisError::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
        }
    }
}

impl Error for AnalysisError {}

/// Unified analysis engine trait for consistent data processing
///
/// This trait ensures that all export formats (JSON, binary, etc.) use the same
/// analysis logic, preventing data inconsistencies between different export methods.
pub trait AnalysisEngine {
    /// Create memory analysis data
    ///
    /// Analyzes memory allocation patterns, sizes, and basic statistics
    fn create_memory_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError>;

    /// Create lifetime analysis data
    ///
    /// Analyzes allocation lifetimes, scope information, and lifecycle events
    fn create_lifetime_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError>;

    /// Create performance analysis data
    ///
    /// Analyzes performance metrics, allocation patterns, and optimization opportunities
    fn create_performance_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError>;

    /// Create unsafe FFI analysis data
    ///
    /// Analyzes unsafe operations, FFI boundaries, and potential safety violations
    fn create_unsafe_ffi_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError>;

    /// Create complex types analysis data
    ///
    /// Analyzes complex type usage, generic instantiations, and type relationships
    fn create_complex_types_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError>;

    /// Get analysis engine configuration
    fn get_config(&self) -> &AnalysisConfig;
}

/// Configuration for the analysis engine
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Optimization level to use
    pub optimization_level: OptimizationLevel,
    /// Whether to enable parallel processing
    pub parallel_processing: bool,
    /// Whether to enable enhanced FFI analysis
    pub enhanced_ffi_analysis: bool,
    /// Whether to enable security analysis
    pub security_analysis: bool,
    /// Batch size for processing
    pub batch_size: usize,
}

/// Optimization levels for analysis
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// Low optimization - basic analysis only
    Low,
    /// Medium optimization - standard analysis
    Medium,
    /// High optimization - comprehensive analysis
    High,
    /// Maximum optimization - all features enabled
    Maximum,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::High,
            parallel_processing: true,
            enhanced_ffi_analysis: true,
            security_analysis: false,
            batch_size: 1000,
        }
    }
}

/// Standard implementation of the analysis engine
///
/// This implementation uses the existing optimized analysis functions
/// to ensure consistency with the current JSON export system.
pub struct StandardAnalysisEngine {
    config: AnalysisConfig,
}

impl StandardAnalysisEngine {
    /// Create a new standard analysis engine with default configuration
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }

    /// Create a new standard analysis engine with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }
}

impl Default for StandardAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisEngine for StandardAnalysisEngine {
    fn create_memory_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError> {
        // Create a simple memory analysis since we can't access the private functions
        // This will be improved once we refactor the optimized_json_export module
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let avg_size = if !allocations.is_empty() {
            total_size / allocations.len()
        } else {
            0
        };
        let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(0);
        let min_size = allocations.iter().map(|a| a.size).min().unwrap_or(0);

        let allocations_data: Vec<serde_json::Value> = allocations
            .iter()
            .map(|alloc| {
                serde_json::json!({
                    "ptr": alloc.ptr,
                    "size": alloc.size,
                    "var_name": self.infer_var_name(alloc),
                    "type_name": self.infer_type_name(alloc),
                    "scope_name": alloc.scope_name.as_deref().unwrap_or("global"),
                    "thread_id": alloc.thread_id,
                    "timestamp_alloc": alloc.timestamp_alloc,
                    "timestamp_dealloc": alloc.timestamp_dealloc.unwrap_or(0),
                    "is_leaked": alloc.is_leaked,
                    "borrow_count": alloc.borrow_count,
                    "lifetime_ms": alloc.lifetime_ms.unwrap_or(0),
                    "stack_trace": alloc.stack_trace.as_ref().map(|st| st.join(" -> ")).unwrap_or_else(|| "no_stack_trace".to_string())
                })
            })
            .collect();

        let data = serde_json::json!({
            "allocations": allocations_data,
            "metadata": {
                "analysis_type": "integrated_memory_analysis",
                "export_version": "2.0",
                "optimization_level": format!("{:?}", self.config.optimization_level),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "total_allocations_analyzed": allocations.len(),
                "pipeline_features": {
                    "enhanced_ffi_analysis": self.config.enhanced_ffi_analysis,
                    "parallel_processing": self.config.parallel_processing,
                    "security_analysis": self.config.security_analysis
                }
            },
            "summary": {
                "total_allocations": allocations.len(),
                "total_memory": total_size,
                "average_size": avg_size,
                "max_size": max_size,
                "min_size": min_size,
                "leaked_count": allocations.iter().filter(|a| a.is_leaked).count()
            }
        });

        Ok(AnalysisData {
            data,
            metadata: AnalysisMetadata {
                analysis_type: "integrated_memory_analysis".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                total_allocations: allocations.len(),
                optimization_level: format!("{:?}", self.config.optimization_level),
            },
        })
    }

    fn create_lifetime_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError> {
        // Create lifecycle events from allocations
        let mut lifecycle_events = Vec::new();
        let mut scope_analysis = std::collections::HashMap::new();

        for alloc in allocations {
            // Add allocation event
            lifecycle_events.push(serde_json::json!({
                "event": "allocation",
                "ptr": format!("0x{:x}", alloc.ptr),
                "size": alloc.size,
                "timestamp": alloc.timestamp_alloc,
                "var_name": self.infer_var_name(alloc),
                "type_name": self.infer_type_name(alloc),
                "scope": alloc.scope_name.as_deref().unwrap_or("global")
            }));

            // Add deallocation event if available
            if let Some(dealloc_time) = alloc.timestamp_dealloc {
                lifecycle_events.push(serde_json::json!({
                    "event": "deallocation",
                    "ptr": format!("0x{:x}", alloc.ptr),
                    "timestamp": dealloc_time,
                    "var_name": self.infer_var_name(alloc),
                    "scope": alloc.scope_name.as_deref().unwrap_or("global")
                }));
            }

            // Update scope analysis
            let scope = alloc.scope_name.as_deref().unwrap_or("global");
            let entry = scope_analysis
                .entry(scope.to_string())
                .or_insert((0, 0, Vec::new()));
            entry.0 += 1; // allocation count
            entry.1 += alloc.size; // total size
            entry.2.push(alloc.size); // individual sizes
        }

        // Convert scope analysis to the expected format
        let scope_stats: std::collections::HashMap<String, serde_json::Value> = scope_analysis
            .into_iter()
            .map(|(scope, (count, total_size, sizes))| {
                let avg_size = if count > 0 { total_size / count } else { 0 };
                (
                    scope,
                    serde_json::json!({
                        "allocation_count": count,
                        "total_size": total_size,
                        "average_size": avg_size,
                        "sizes": sizes
                    }),
                )
            })
            .collect();

        let data = serde_json::json!({
            "lifecycle_events": lifecycle_events,
            "scope_analysis": scope_stats,
            "variable_lifetimes": {},
            "metadata": {
                "analysis_type": "integrated_lifetime_analysis",
                "export_version": "2.0",
                "optimization_level": format!("{:?}", self.config.optimization_level),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "total_allocations_analyzed": allocations.len(),
                "pipeline_features": {
                    "enhanced_ffi_analysis": self.config.enhanced_ffi_analysis,
                    "parallel_processing": self.config.parallel_processing
                }
            },
            "summary": {
                "total_allocations": allocations.len(),
                "unique_scopes": scope_stats.len(),
                "total_events": lifecycle_events.len(),
                "leaked_count": allocations.iter().filter(|a| a.is_leaked).count()
            }
        });

        Ok(AnalysisData {
            data,
            metadata: AnalysisMetadata {
                analysis_type: "integrated_lifetime_analysis".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                total_allocations: allocations.len(),
                optimization_level: format!("{:?}", self.config.optimization_level),
            },
        })
    }

    fn create_performance_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError> {
        // Calculate performance metrics
        let total_size: usize = allocations.iter().map(|a| a.size).sum();
        let avg_size = if !allocations.is_empty() {
            total_size / allocations.len()
        } else {
            0
        };
        let max_size = allocations.iter().map(|a| a.size).max().unwrap_or(0);
        let min_size = allocations.iter().map(|a| a.size).min().unwrap_or(0);

        // Group by thread for thread analysis
        let mut thread_stats = std::collections::HashMap::new();
        for alloc in allocations {
            let entry = thread_stats
                .entry(alloc.thread_id.clone())
                .or_insert((0, 0));
            entry.0 += 1; // count
            entry.1 += alloc.size; // total size
        }

        let thread_analysis: std::collections::HashMap<String, serde_json::Value> = thread_stats
            .into_iter()
            .map(|(thread_id, (count, total_size))| {
                (
                    thread_id,
                    serde_json::json!({
                        "allocation_count": count,
                        "total_size": total_size,
                        "average_size": if count > 0 { total_size / count } else { 0 }
                    }),
                )
            })
            .collect();

        let allocations_data: Vec<serde_json::Value> = allocations
            .iter()
            .map(|alloc| {
                serde_json::json!({
                    "ptr": alloc.ptr,
                    "size": alloc.size,
                    "timestamp_alloc": alloc.timestamp_alloc,
                    "thread_id": alloc.thread_id,
                    "borrow_count": alloc.borrow_count,
                    "var_name": self.infer_var_name(alloc),
                    "type_name": self.infer_type_name(alloc),
                    "scope_name": alloc.scope_name.as_deref().unwrap_or("global"),
                    "is_leaked": alloc.is_leaked,
                    "lifetime_ms": alloc.lifetime_ms.unwrap_or(0),
                    "fragmentation_score": 0.0 // Default fragmentation score when analysis is not available
                })
            })
            .collect();

        let data = serde_json::json!({
            "allocations": allocations_data,
            "thread_analysis": thread_analysis,
            "metadata": {
                "analysis_type": "integrated_performance_analysis",
                "export_version": "2.0",
                "optimization_level": format!("{:?}", self.config.optimization_level),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "total_allocations_analyzed": allocations.len(),
                "pipeline_features": {
                    "parallel_processing": self.config.parallel_processing,
                    "batch_size": self.config.batch_size
                }
            },
            "summary": {
                "total_allocations": allocations.len(),
                "total_memory": total_size,
                "average_size": avg_size,
                "max_size": max_size,
                "min_size": min_size,
                "unique_threads": thread_analysis.len()
            }
        });

        Ok(AnalysisData {
            data,
            metadata: AnalysisMetadata {
                analysis_type: "integrated_performance_analysis".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                total_allocations: allocations.len(),
                optimization_level: format!("{:?}", self.config.optimization_level),
            },
        })
    }

    fn create_unsafe_ffi_analysis(
        &self,
        _allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError> {
        use crate::analysis::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker;

        // Get enhanced allocations with real unsafe/FFI data
        let tracker = get_global_unsafe_ffi_tracker();
        let enhanced_allocations = tracker.get_enhanced_allocations().map_err(|e| {
            AnalysisError::ProcessingError(format!("Failed to get enhanced allocations: {e}"))
        })?;

        // For unsafe/FFI analysis, we want ALL enhanced allocations, not just user-defined ones
        // because unsafe/FFI operations often don't have variable names
        let user_enhanced_allocations = enhanced_allocations;

        // Convert to the expected JSON format matching snapshot_unsafe_ffi.json
        let data = serde_json::to_value(&user_enhanced_allocations).map_err(|e| {
            AnalysisError::SerializationError(format!(
                "Failed to serialize enhanced allocations: {e}"
            ))
        })?;

        Ok(AnalysisData {
            data,
            metadata: AnalysisMetadata {
                analysis_type: "unsafe_ffi_analysis".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                total_allocations: user_enhanced_allocations.len(),
                optimization_level: format!("{:?}", self.config.optimization_level),
            },
        })
    }

    fn create_complex_types_analysis(
        &self,
        allocations: &[AllocationInfo],
    ) -> Result<AnalysisData, AnalysisError> {
        // Categorize types
        let mut categorized_types = std::collections::HashMap::new();
        let mut generic_types = std::collections::HashMap::new();

        for alloc in allocations {
            let type_name = self.infer_type_name(alloc);

            // Categorize the type
            let category = if type_name.contains('<') && type_name.contains('>') {
                "generic"
            } else if type_name.starts_with("Vec") || type_name.starts_with("HashMap") {
                "collection"
            } else if type_name.contains("::") {
                "module_type"
            } else {
                "primitive"
            };

            let entry = categorized_types
                .entry(category.to_string())
                .or_insert(Vec::new());
            // Debug: Check if we have the enhanced data
            if alloc.memory_layout.is_some() {
                tracing::debug!(
                    "AllocationInfo has memory_layout for {}",
                    self.infer_var_name(alloc)
                );
            }
            if alloc.generic_info.is_some() {
                tracing::debug!(
                    "AllocationInfo has generic_info for {}",
                    self.infer_var_name(alloc)
                );
            }

            // Manually serialize to avoid potential serde issues
            let mut json_obj = serde_json::Map::new();
            json_obj.insert(
                "ptr".to_string(),
                serde_json::Value::String(format!("0x{:x}", alloc.ptr)),
            );
            json_obj.insert(
                "size".to_string(),
                serde_json::Value::Number(serde_json::Number::from(alloc.size)),
            );
            json_obj.insert(
                "var_name".to_string(),
                serde_json::Value::String(self.infer_var_name(alloc)),
            );
            json_obj.insert(
                "type_name".to_string(),
                serde_json::Value::String(type_name.to_string()),
            );

            // Serialize complex fields with default values instead of null
            json_obj.insert(
                "smart_pointer_info".to_string(),
                if let Some(ref info) = alloc.smart_pointer_info {
                    serde_json::to_value(info)
                        .unwrap_or_else(|_| serde_json::json!({"type": "unknown", "ref_count": 0}))
                } else {
                    serde_json::json!({"type": "none", "ref_count": 0})
                },
            );

            json_obj.insert(
                "memory_layout".to_string(),
                if let Some(ref layout) = alloc.memory_layout {
                    match serde_json::to_value(layout) {
                        Ok(value) => {
                            tracing::debug!(
                                "Successfully serialized memory_layout for {}",
                                self.infer_var_name(alloc)
                            );
                            value
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Failed to serialize memory_layout for {}: {}",
                                self.infer_var_name(alloc),
                                e
                            );
                            serde_json::json!({"layout_type": "unknown", "size": alloc.size, "alignment": 1})
                        }
                    }
                } else {
                    serde_json::json!({"layout_type": "default", "size": alloc.size, "alignment": 1})
                },
            );

            json_obj.insert(
                "generic_info".to_string(),
                if let Some(ref info) = alloc.generic_info {
                    match serde_json::to_value(info) {
                        Ok(value) => {
                            tracing::debug!(
                                "Successfully serialized generic_info for {}",
                                self.infer_var_name(alloc)
                            );
                            value
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Failed to serialize generic_info for {}: {}",
                                self.infer_var_name(alloc),
                                e
                            );
                            serde_json::json!({"generic_type": "none", "type_parameters": []})
                        }
                    }
                } else {
                    serde_json::json!({"generic_type": "none", "type_parameters": []})
                },
            );

            json_obj.insert(
                "dynamic_type_info".to_string(),
                if let Some(ref info) = alloc.dynamic_type_info {
                    serde_json::to_value(info).unwrap_or_else(|_| serde_json::json!({"type": "static", "runtime_type": "unknown"}))
                } else {
                    serde_json::json!({"type": "static", "runtime_type": self.infer_type_name(alloc)})
                },
            );

            json_obj.insert(
                "generic_instantiation".to_string(),
                if let Some(ref info) = alloc.generic_instantiation {
                    serde_json::to_value(info).unwrap_or_else(
                        |_| serde_json::json!({"instantiation": "none", "parameters": []}),
                    )
                } else {
                    serde_json::json!({"instantiation": "none", "parameters": []})
                },
            );

            json_obj.insert(
                "type_relationships".to_string(),
                if let Some(ref info) = alloc.type_relationships {
                    serde_json::to_value(info).unwrap_or_else(
                        |_| serde_json::json!({"relationships": [], "inheritance": "none"}),
                    )
                } else {
                    serde_json::json!({"relationships": [], "inheritance": "none"})
                },
            );

            json_obj.insert(
                "type_usage".to_string(),
                if let Some(ref info) = alloc.type_usage {
                    serde_json::to_value(info).unwrap_or_else(
                        |_| serde_json::json!({"usage_count": 1, "usage_pattern": "single"}),
                    )
                } else {
                    serde_json::json!({"usage_count": 1, "usage_pattern": "single"})
                },
            );

            entry.push(serde_json::Value::Object(json_obj));

            // Track generic types separately
            if category == "generic" {
                let entry = generic_types.entry(type_name.to_string()).or_insert((0, 0));
                entry.0 += 1; // count
                entry.1 += alloc.size; // total size
            }
        }

        let generic_stats: std::collections::HashMap<String, serde_json::Value> = generic_types
            .into_iter()
            .map(|(type_name, (count, total_size))| {
                (
                    type_name,
                    serde_json::json!({
                        "instantiation_count": count,
                        "total_size": total_size,
                        "average_size": if count > 0 { total_size / count } else { 0 }
                    }),
                )
            })
            .collect();

        let data = serde_json::json!({
            "categorized_types": categorized_types,
            "generic_types": generic_stats,
            "metadata": {
                "analysis_type": "integrated_complex_types_analysis",
                "export_version": "2.0",
                "optimization_level": format!("{:?}", self.config.optimization_level),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "total_allocations_analyzed": allocations.len(),
                "pipeline_features": {
                    "type_categorization": true,
                    "generic_analysis": true,
                    "memory_layout_analysis": true
                }
            },
            "summary": {
                "total_allocations": allocations.len(),
                "type_categories": categorized_types.len(),
                "generic_types": generic_stats.len(),
                "complex_type_ratio": if !allocations.is_empty() {
                    (categorized_types.get("generic").map(|v| v.len()).unwrap_or(0) as f64 / allocations.len() as f64) * 100.0
                } else { 0.0 }
            }
        });

        Ok(AnalysisData {
            data,
            metadata: AnalysisMetadata {
                analysis_type: "integrated_complex_types_analysis".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                total_allocations: allocations.len(),
                optimization_level: format!("{:?}", self.config.optimization_level),
            },
        })
    }

    fn get_config(&self) -> &AnalysisConfig {
        &self.config
    }
}

impl StandardAnalysisEngine {
    /// Convert our analysis config to the existing OptimizedExportOptions
    #[allow(dead_code)]
    fn convert_to_export_options(
        &self,
    ) -> crate::export::optimized_json_export::OptimizedExportOptions {
        use crate::export::optimized_json_export::{
            OptimizationLevel as ExportOptLevel, OptimizedExportOptions,
        };

        let export_opt_level = match self.config.optimization_level {
            OptimizationLevel::Low => ExportOptLevel::Low,
            OptimizationLevel::Medium => ExportOptLevel::Medium,
            OptimizationLevel::High => ExportOptLevel::High,
            OptimizationLevel::Maximum => ExportOptLevel::Maximum,
        };

        OptimizedExportOptions::with_optimization_level(export_opt_level)
            .parallel_processing(self.config.parallel_processing)
            .batch_size(self.config.batch_size)
    }

    /// Infer type name from allocation when type_name is None
    /// This eliminates "unknown" type names in full-binary mode
    fn infer_type_name(&self, alloc: &AllocationInfo) -> String {
        match alloc.type_name.as_deref() {
            Some(name) => name.to_string(),
            None => {
                // Infer type from allocation size and patterns
                match alloc.size {
                    0 => "ZeroSizedType".to_string(),
                    1 => "u8_or_bool".to_string(),
                    2 => "u16_or_char".to_string(),
                    4 => "u32_or_f32_or_i32".to_string(),
                    8 => "u64_or_f64_or_i64_or_usize".to_string(),
                    16 => "u128_or_i128_or_complex_struct".to_string(),
                    24 => "Vec_or_String_header".to_string(),
                    32 => "HashMap_or_BTreeMap_header".to_string(),
                    size if size >= 1024 => format!("LargeAllocation_{size}bytes"),
                    size if size % 8 == 0 => format!("AlignedStruct_{size}bytes"),
                    size => format!("CustomType_{size}bytes"),
                }
            }
        }
    }

    /// Infer variable name from allocation when var_name is None
    /// This eliminates "unknown" variable names in full-binary mode
    fn infer_var_name(&self, alloc: &AllocationInfo) -> String {
        match alloc.var_name.as_deref() {
            Some(name) => name.to_string(),
            None => {
                // Generate descriptive variable name based on allocation characteristics
                let type_hint = match alloc.size {
                    0 => "zero_sized_var",
                    1..=8 => "primitive_var",
                    9..=32 => "small_struct_var",
                    33..=256 => "medium_struct_var",
                    257..=1024 => "large_struct_var",
                    _ => "heap_allocated_var",
                };

                // Include pointer address for uniqueness
                format!("{}_{:x}", type_hint, alloc.ptr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    fn create_test_allocations() -> Vec<AllocationInfo> {
        vec![AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("buffer".to_string()),
            type_name: Some("Vec<u8>".to_string()),
            scope_name: Some("main".to_string()),
            timestamp_alloc: 1234567890,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(100),
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
        }]
    }

    #[test]
    fn test_standard_analysis_engine_creation() {
        let engine = StandardAnalysisEngine::new();
        assert_eq!(
            engine.get_config().optimization_level,
            OptimizationLevel::High
        );
    }

    #[test]
    fn test_memory_analysis() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_test_allocations();

        let result = engine.create_memory_analysis(&allocations);
        assert!(result.is_ok());

        let analysis_data = result.expect("Failed to get analysis data");
        assert_eq!(
            analysis_data.metadata.analysis_type,
            "integrated_memory_analysis"
        );
        assert_eq!(analysis_data.metadata.total_allocations, 1);
    }

    #[test]
    fn test_standard_analysis_engine_with_config() {
        let config = AnalysisConfig {
            optimization_level: OptimizationLevel::Low,
            parallel_processing: false,
            enhanced_ffi_analysis: false,
            security_analysis: true,
            batch_size: 500,
        };
        let engine = StandardAnalysisEngine::with_config(config);

        assert_eq!(
            engine.get_config().optimization_level,
            OptimizationLevel::Low
        );
        assert!(!engine.get_config().parallel_processing);
        assert!(!engine.get_config().enhanced_ffi_analysis);
        assert!(engine.get_config().security_analysis);
        assert_eq!(engine.get_config().batch_size, 500);
    }

    #[test]
    fn test_standard_analysis_engine_default() {
        let engine1 = StandardAnalysisEngine::new();
        let engine2 = StandardAnalysisEngine::default();

        assert_eq!(
            engine1.get_config().optimization_level,
            engine2.get_config().optimization_level
        );
        assert_eq!(
            engine1.get_config().parallel_processing,
            engine2.get_config().parallel_processing
        );
        assert_eq!(
            engine1.get_config().batch_size,
            engine2.get_config().batch_size
        );
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();

        assert_eq!(config.optimization_level, OptimizationLevel::High);
        assert!(config.parallel_processing);
        assert!(config.enhanced_ffi_analysis);
        assert!(!config.security_analysis);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_lifetime_analysis() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_test_allocations();

        let result = engine.create_lifetime_analysis(&allocations);
        assert!(result.is_ok());

        let analysis_data = result.expect("Lifetime analysis should succeed with valid test data");
        assert_eq!(
            analysis_data.metadata.analysis_type,
            "integrated_lifetime_analysis"
        );
        assert_eq!(analysis_data.metadata.total_allocations, 1);

        // Check that the data contains expected fields
        let data = &analysis_data.data;
        assert!(data.get("lifecycle_events").is_some());
        assert!(data.get("scope_analysis").is_some());
        assert!(data.get("summary").is_some());
    }

    #[test]
    fn test_performance_analysis() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_test_allocations();

        let result = engine.create_performance_analysis(&allocations);
        assert!(result.is_ok());

        let analysis_data = result.unwrap();
        assert_eq!(
            analysis_data.metadata.analysis_type,
            "integrated_performance_analysis"
        );
        assert_eq!(analysis_data.metadata.total_allocations, 1);

        // Check that the data contains expected fields
        let data = &analysis_data.data;
        assert!(data.get("allocations").is_some());
        assert!(data.get("thread_analysis").is_some());
        assert!(data.get("summary").is_some());
    }

    #[test]
    fn test_complex_types_analysis() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_test_allocations();

        let result = engine.create_complex_types_analysis(&allocations);
        assert!(result.is_ok());

        let analysis_data = result.unwrap();
        assert_eq!(
            analysis_data.metadata.analysis_type,
            "integrated_complex_types_analysis"
        );
        assert_eq!(analysis_data.metadata.total_allocations, 1);

        // Check that the data contains expected fields
        let data = &analysis_data.data;
        assert!(data.get("categorized_types").is_some());
        assert!(data.get("generic_types").is_some());
        assert!(data.get("summary").is_some());
    }

    #[test]
    fn test_empty_allocations() {
        let engine = StandardAnalysisEngine::new();
        let empty_allocations = vec![];

        // Test all analysis methods with empty data
        let memory_result = engine.create_memory_analysis(&empty_allocations);
        assert!(memory_result.is_ok());
        let memory_data = memory_result.expect("Memory analysis should succeed with empty data");
        assert_eq!(memory_data.metadata.total_allocations, 0);

        let lifetime_result = engine.create_lifetime_analysis(&empty_allocations);
        assert!(lifetime_result.is_ok());
        let lifetime_data =
            lifetime_result.expect("Lifetime analysis should succeed with empty data");
        assert_eq!(lifetime_data.metadata.total_allocations, 0);

        let performance_result = engine.create_performance_analysis(&empty_allocations);
        assert!(performance_result.is_ok());
        let performance_data =
            performance_result.expect("Performance analysis should succeed with empty data");
        assert_eq!(performance_data.metadata.total_allocations, 0);

        let complex_types_result = engine.create_complex_types_analysis(&empty_allocations);
        assert!(complex_types_result.is_ok());
        let complex_types_data =
            complex_types_result.expect("Complex types analysis should succeed with empty data");
        assert_eq!(complex_types_data.metadata.total_allocations, 0);
    }

    fn create_multiple_test_allocations() -> Vec<AllocationInfo> {
        vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 1024,
                var_name: Some("buffer".to_string()),
                type_name: Some("Vec<u8>".to_string()),
                scope_name: Some("main".to_string()),
                timestamp_alloc: 1234567890,
                timestamp_dealloc: Some(1234567990),
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: Some(vec!["main".to_string(), "allocate".to_string()]),
                is_leaked: false,
                lifetime_ms: Some(100),
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
                ptr: 0x2000,
                size: 512,
                var_name: None,  // Test inference
                type_name: None, // Test inference
                scope_name: Some("function".to_string()),
                timestamp_alloc: 1234567900,
                timestamp_dealloc: None,
                thread_id: "worker".to_string(),
                borrow_count: 2,
                stack_trace: None,
                is_leaked: true,
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
            },
            AllocationInfo {
                ptr: 0x3000,
                size: 8,
                var_name: Some("counter".to_string()),
                type_name: Some("HashMap<String, i32>".to_string()),
                scope_name: None, // Test global scope
                timestamp_alloc: 1234567910,
                timestamp_dealloc: Some(1234567950),
                thread_id: "main".to_string(),
                borrow_count: 1,
                stack_trace: Some(vec!["main".to_string()]),
                is_leaked: false,
                lifetime_ms: Some(40),
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
        ]
    }

    #[test]
    fn test_multiple_allocations_analysis() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_multiple_test_allocations();

        // Test memory analysis with multiple allocations
        let memory_result = engine.create_memory_analysis(&allocations);
        assert!(memory_result.is_ok());
        let memory_data = memory_result.unwrap();
        assert_eq!(memory_data.metadata.total_allocations, 3);

        // Check summary calculations
        let summary = memory_data.data.get("summary").unwrap();
        assert_eq!(
            summary.get("total_allocations").unwrap().as_u64().unwrap(),
            3
        );
        assert_eq!(summary.get("total_memory").unwrap().as_u64().unwrap(), 1544); // 1024 + 512 + 8
        assert_eq!(summary.get("leaked_count").unwrap().as_u64().unwrap(), 1);
        assert_eq!(summary.get("max_size").unwrap().as_u64().unwrap(), 1024);
        assert_eq!(summary.get("min_size").unwrap().as_u64().unwrap(), 8);
    }

    #[test]
    fn test_lifetime_analysis_with_multiple_allocations() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_multiple_test_allocations();

        let result = engine.create_lifetime_analysis(&allocations);
        assert!(result.is_ok());
        let analysis_data = result.unwrap();

        // Check lifecycle events
        let events = analysis_data
            .data
            .get("lifecycle_events")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(events.len(), 5); // 3 allocations + 2 deallocations

        // Check scope analysis
        let scope_analysis = analysis_data
            .data
            .get("scope_analysis")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(scope_analysis.contains_key("main"));
        assert!(scope_analysis.contains_key("function"));
        assert!(scope_analysis.contains_key("global"));
    }

    #[test]
    fn test_performance_analysis_with_multiple_threads() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_multiple_test_allocations();

        let result = engine.create_performance_analysis(&allocations);
        assert!(result.is_ok());
        let analysis_data = result.unwrap();

        // Check thread analysis
        let thread_analysis = analysis_data
            .data
            .get("thread_analysis")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(thread_analysis.contains_key("main"));
        assert!(thread_analysis.contains_key("worker"));

        // Check main thread stats
        let main_stats = thread_analysis.get("main").unwrap();
        assert_eq!(
            main_stats
                .get("allocation_count")
                .unwrap()
                .as_u64()
                .unwrap(),
            2
        );
        assert_eq!(
            main_stats.get("total_size").unwrap().as_u64().unwrap(),
            1032
        ); // 1024 + 8

        // Check worker thread stats
        let worker_stats = thread_analysis.get("worker").unwrap();
        assert_eq!(
            worker_stats
                .get("allocation_count")
                .unwrap()
                .as_u64()
                .unwrap(),
            1
        );
        assert_eq!(
            worker_stats.get("total_size").unwrap().as_u64().unwrap(),
            512
        );
    }

    #[test]
    fn test_complex_types_categorization() {
        let engine = StandardAnalysisEngine::new();
        let allocations = create_multiple_test_allocations();

        let result = engine.create_complex_types_analysis(&allocations);
        assert!(result.is_ok());
        let analysis_data = result.unwrap();

        // Check categorized types
        let categorized = analysis_data
            .data
            .get("categorized_types")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(categorized.contains_key("generic")); // Vec<u8> and HashMap<String, i32>
        assert!(categorized.contains_key("primitive")); // Inferred type for size 512

        // Check generic types analysis
        let generic_types = analysis_data
            .data
            .get("generic_types")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(generic_types.contains_key("Vec<u8>"));
        assert!(generic_types.contains_key("HashMap<String, i32>"));
    }

    #[test]
    fn test_type_name_inference() {
        let engine = StandardAnalysisEngine::new();

        // Test different size patterns
        let test_cases = vec![
            (0, "ZeroSizedType"),
            (1, "u8_or_bool"),
            (2, "u16_or_char"),
            (4, "u32_or_f32_or_i32"),
            (8, "u64_or_f64_or_i64_or_usize"),
            (16, "u128_or_i128_or_complex_struct"),
            (24, "Vec_or_String_header"),
            (32, "HashMap_or_BTreeMap_header"),
            (1024, "LargeAllocation_1024bytes"),
            (48, "AlignedStruct_48bytes"), // 48 % 8 == 0
            (33, "CustomType_33bytes"),    // 33 % 8 != 0
        ];

        for (size, expected_prefix) in test_cases {
            let alloc = AllocationInfo {
                ptr: 0x1000,
                size,
                var_name: None,
                type_name: None, // Force inference
                scope_name: None,
                timestamp_alloc: 0,
                timestamp_dealloc: None,
                thread_id: "test".to_string(),
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
            };

            let inferred_type = engine.infer_type_name(&alloc);
            assert_eq!(inferred_type, expected_prefix);
        }
    }

    #[test]
    fn test_var_name_inference() {
        let engine = StandardAnalysisEngine::new();

        let test_cases = vec![
            (0, "zero_sized_var"),
            (4, "primitive_var"),
            (16, "small_struct_var"),
            (128, "medium_struct_var"),
            (512, "large_struct_var"),
            (2048, "heap_allocated_var"),
        ];

        for (size, expected_prefix) in test_cases {
            let alloc = AllocationInfo {
                ptr: 0x1234,
                size,
                var_name: None, // Force inference
                type_name: Some("TestType".to_string()),
                scope_name: None,
                timestamp_alloc: 0,
                timestamp_dealloc: None,
                thread_id: "test".to_string(),
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
            };

            let inferred_var = engine.infer_var_name(&alloc);
            assert!(inferred_var.starts_with(expected_prefix));
            assert!(inferred_var.contains("1234")); // Should contain pointer address
        }
    }

    #[test]
    fn test_optimization_levels() {
        // Test all optimization levels
        let levels = vec![
            OptimizationLevel::Low,
            OptimizationLevel::Medium,
            OptimizationLevel::High,
            OptimizationLevel::Maximum,
        ];

        for level in levels {
            let config = AnalysisConfig {
                optimization_level: level.clone(),
                parallel_processing: true,
                enhanced_ffi_analysis: true,
                security_analysis: false,
                batch_size: 1000,
            };

            let engine = StandardAnalysisEngine::with_config(config);
            let allocations = create_test_allocations();

            // Test that all analysis methods work with different optimization levels
            let memory_result = engine.create_memory_analysis(&allocations);
            assert!(memory_result.is_ok());
            let memory_data = memory_result.unwrap();
            assert_eq!(
                memory_data.metadata.optimization_level,
                format!("{:?}", level)
            );

            let lifetime_result = engine.create_lifetime_analysis(&allocations);
            assert!(lifetime_result.is_ok());

            let performance_result = engine.create_performance_analysis(&allocations);
            assert!(performance_result.is_ok());

            let complex_types_result = engine.create_complex_types_analysis(&allocations);
            assert!(complex_types_result.is_ok());
        }
    }

    #[test]
    fn test_analysis_error_display() {
        let processing_error = AnalysisError::ProcessingError("Test processing error".to_string());
        assert_eq!(
            processing_error.to_string(),
            "Processing error: Test processing error"
        );

        let serialization_error =
            AnalysisError::SerializationError("Test serialization error".to_string());
        assert_eq!(
            serialization_error.to_string(),
            "Serialization error: Test serialization error"
        );

        let invalid_data_error = AnalysisError::InvalidData("Test invalid data".to_string());
        assert_eq!(
            invalid_data_error.to_string(),
            "Invalid data: Test invalid data"
        );
    }

    #[test]
    fn test_analysis_error_debug() {
        let error = AnalysisError::ProcessingError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ProcessingError"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_analysis_data_debug_and_clone() {
        let metadata = AnalysisMetadata {
            analysis_type: "test_analysis".to_string(),
            timestamp: 1234567890,
            total_allocations: 10,
            optimization_level: "High".to_string(),
        };

        let data = AnalysisData {
            data: serde_json::json!({"test": "value"}),
            metadata: metadata.clone(),
        };

        // Test Debug implementation
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("AnalysisData"));
        assert!(debug_str.contains("test_analysis"));

        // Test Clone implementation
        let cloned_data = data.clone();
        assert_eq!(
            cloned_data.metadata.analysis_type,
            data.metadata.analysis_type
        );
        assert_eq!(cloned_data.metadata.timestamp, data.metadata.timestamp);
        assert_eq!(
            cloned_data.metadata.total_allocations,
            data.metadata.total_allocations
        );
    }

    #[test]
    fn test_analysis_metadata_debug_and_clone() {
        let metadata = AnalysisMetadata {
            analysis_type: "test_metadata".to_string(),
            timestamp: 9876543210,
            total_allocations: 42,
            optimization_level: "Maximum".to_string(),
        };

        // Test Debug implementation
        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("AnalysisMetadata"));
        assert!(debug_str.contains("test_metadata"));
        assert!(debug_str.contains("42"));

        // Test Clone implementation
        let cloned_metadata = metadata.clone();
        assert_eq!(cloned_metadata.analysis_type, metadata.analysis_type);
        assert_eq!(cloned_metadata.timestamp, metadata.timestamp);
        assert_eq!(
            cloned_metadata.total_allocations,
            metadata.total_allocations
        );
        assert_eq!(
            cloned_metadata.optimization_level,
            metadata.optimization_level
        );
    }

    #[test]
    fn test_analysis_config_debug_and_clone() {
        let config = AnalysisConfig {
            optimization_level: OptimizationLevel::Medium,
            parallel_processing: false,
            enhanced_ffi_analysis: true,
            security_analysis: true,
            batch_size: 2000,
        };

        // Test Debug implementation
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("AnalysisConfig"));
        assert!(debug_str.contains("Medium"));
        assert!(debug_str.contains("2000"));

        // Test Clone implementation
        let cloned_config = config.clone();
        assert_eq!(cloned_config.optimization_level, config.optimization_level);
        assert_eq!(
            cloned_config.parallel_processing,
            config.parallel_processing
        );
        assert_eq!(
            cloned_config.enhanced_ffi_analysis,
            config.enhanced_ffi_analysis
        );
        assert_eq!(cloned_config.security_analysis, config.security_analysis);
        assert_eq!(cloned_config.batch_size, config.batch_size);
    }

    #[test]
    fn test_optimization_level_equality() {
        assert_eq!(OptimizationLevel::Low, OptimizationLevel::Low);
        assert_eq!(OptimizationLevel::Medium, OptimizationLevel::Medium);
        assert_eq!(OptimizationLevel::High, OptimizationLevel::High);
        assert_eq!(OptimizationLevel::Maximum, OptimizationLevel::Maximum);

        assert_ne!(OptimizationLevel::Low, OptimizationLevel::Medium);
        assert_ne!(OptimizationLevel::Medium, OptimizationLevel::High);
        assert_ne!(OptimizationLevel::High, OptimizationLevel::Maximum);
    }

    #[test]
    fn test_convert_to_export_options() {
        let engine = StandardAnalysisEngine::new();
        let export_options = engine.convert_to_export_options();

        // This tests the conversion function exists and works
        // The actual values depend on the implementation
        assert_eq!(export_options.batch_size, 1000);
        assert!(export_options.parallel_processing);
    }
}
