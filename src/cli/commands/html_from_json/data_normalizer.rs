//! Data normalization module for HTML export
//!
//! This module provides functionality to normalize and standardize data from
//! different JSON sources into a unified format for HTML visualization.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Unified data structure for memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMemoryData {
    /// Core memory statistics
    pub stats: MemoryStatistics,

    /// Memory allocation details
    pub allocations: Vec<AllocationInfo>,

    /// Performance analysis data
    pub performance: PerformanceMetrics,

    /// Lifecycle analysis
    pub lifecycle: LifecycleAnalysis,

    /// Security analysis
    pub security: SecurityAnalysis,

    /// Complex type analysis
    pub complex_types: ComplexTypeAnalysis,

    /// Variable relationships
    pub variable_relationships: VariableRelationships,

    /// Analysis metadata
    pub metadata: AnalysisMetadata,

    /// Original multi-source data (for advanced features)
    #[serde(rename = "_multiSource")]
    pub multi_source: HashMap<String, Value>,
}

/// Core memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    /// Active memory in bytes
    pub active_memory: usize,
    /// Active allocation count
    pub active_allocations: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Total allocations made
    pub total_allocations: usize,
    /// Total memory allocated
    pub total_allocated: usize,
    /// Memory efficiency percentage
    pub memory_efficiency: f64,
}

/// Borrow information for unsafe/FFI tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Number of immutable borrows
    pub immutable_borrows: u32,
    /// Number of mutable borrows
    pub mutable_borrows: u32,
    /// Maximum concurrent borrows
    pub max_concurrent_borrows: u32,
    /// Last borrow timestamp
    pub last_borrow_timestamp: u64,
}

/// Clone information for memory tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Number of clones created
    pub clone_count: u32,
    /// Whether this allocation is a clone
    pub is_clone: bool,
    /// Original pointer if this is a clone
    pub original_ptr: Option<String>,
}

/// Allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory pointer as hex string
    pub ptr: String,
    /// Allocation size in bytes
    pub size: usize,
    /// Variable name if available
    pub var_name: Option<String>,
    /// Type name if available
    pub type_name: Option<String>,
    /// Scope name
    pub scope_name: Option<String>,
    /// Allocation timestamp
    pub timestamp_alloc: u64,
    /// Deallocation timestamp
    pub timestamp_dealloc: Option<u64>,
    /// Thread ID
    pub thread_id: Option<String>,
    /// Borrow count
    pub borrow_count: Option<u32>,
    /// Stack trace
    pub stack_trace: Option<Vec<String>>,
    /// Whether allocation is leaked
    pub is_leaked: bool,
    /// Lifetime in milliseconds
    pub lifetime_ms: Option<u64>,
    /// Borrow information for unsafe/FFI tracking
    pub borrow_info: Option<BorrowInfo>,
    /// Clone information
    pub clone_info: Option<CloneInfo>,
    /// Whether ownership history is available
    pub ownership_history_available: Option<bool>,
    /// Whether FFI tracking is enabled
    pub ffi_tracked: Option<bool>,
    /// Safety violations
    pub safety_violations: Option<Vec<String>>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Allocations per second
    pub allocations_per_second: f64,
    /// Memory efficiency percentage
    pub memory_efficiency: f64,
    /// Optimization status
    pub optimization_status: OptimizationStatus,
    /// Allocation distribution
    pub allocation_distribution: AllocationDistribution,
}

/// Optimization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStatus {
    /// Parallel processing enabled
    pub parallel_processing: bool,
    /// Schema validation enabled
    pub schema_validation: bool,
    /// Streaming enabled
    pub streaming_enabled: bool,
    /// Batch size used
    pub batch_size: Option<usize>,
    /// Buffer size in KB
    pub buffer_size_kb: Option<usize>,
}

/// Allocation distribution by size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDistribution {
    /// Tiny allocations (< 64 bytes)
    pub tiny: usize,
    /// Small allocations (64-1024 bytes)
    pub small: usize,
    /// Medium allocations (1KB-64KB)
    pub medium: usize,
    /// Large allocations (64KB-1MB)
    pub large: usize,
    /// Massive allocations (> 1MB)
    pub massive: usize,
}

/// Lifecycle analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAnalysis {
    /// Lifecycle events
    pub lifecycle_events: Vec<Value>,
    /// Scope analysis
    pub scope_analysis: HashMap<String, Value>,
    /// Variable lifetimes
    pub variable_lifetimes: HashMap<String, Value>,
}

/// Security analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Total violations count
    pub total_violations: usize,
    /// Risk level
    pub risk_level: String,
    /// Severity breakdown
    pub severity_breakdown: SeverityBreakdown,
    /// Violation reports
    pub violation_reports: Vec<Value>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Severity breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityBreakdown {
    /// Critical violations
    pub critical: usize,
    /// High severity violations
    pub high: usize,
    /// Medium severity violations
    pub medium: usize,
    /// Low severity violations
    pub low: usize,
    /// Info level violations
    pub info: usize,
}

/// Complex type analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeAnalysis {
    /// Categorized types
    pub categorized_types: CategorizedTypes,
    /// Complex type analysis details
    pub complex_type_analysis: Vec<Value>,
    /// Summary information
    pub summary: ComplexTypeSummary,
}

/// Categorized types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizedTypes {
    /// Collection types
    pub collections: Vec<Value>,
    /// Generic types
    pub generic_types: Vec<Value>,
    /// Smart pointers
    pub smart_pointers: Vec<Value>,
    /// Trait objects
    pub trait_objects: Vec<Value>,
}

/// Complex type summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTypeSummary {
    /// Total complex types
    pub total_complex_types: usize,
    /// Complexity distribution
    pub complexity_distribution: ComplexityDistribution,
}

/// Complexity distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityDistribution {
    /// Low complexity types
    pub low_complexity: usize,
    /// Medium complexity types
    pub medium_complexity: usize,
    /// High complexity types
    pub high_complexity: usize,
    /// Very high complexity types
    pub very_high_complexity: usize,
}

/// Variable relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRelationships {
    /// Relationship data
    pub relationships: Vec<Value>,
    /// Variable registry
    pub registry: HashMap<String, Value>,
    /// Dependency graph
    pub dependency_graph: HashMap<String, Value>,
    /// Scope hierarchy
    pub scope_hierarchy: HashMap<String, Value>,
}

/// Analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Analysis timestamp
    pub timestamp: u64,
    /// Export version
    pub export_version: String,
    /// Analysis type
    pub analysis_type: String,
    /// Data integrity hash
    pub data_integrity_hash: Option<String>,
}

/// Data normalization error
#[derive(Debug)]
pub enum NormalizationError {
    /// Missing required field
    MissingField(String),
    /// Invalid data type
    InvalidType(String),
    /// Data validation error
    ValidationError(String),
    /// JSON parsing error
    JsonError(serde_json::Error),
}

impl fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalizationError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            NormalizationError::InvalidType(msg) => {
                write!(f, "Invalid data type: {}", msg)
            }
            NormalizationError::ValidationError(msg) => {
                write!(f, "Data validation error: {}", msg)
            }
            NormalizationError::JsonError(err) => {
                write!(f, "JSON error: {}", err)
            }
        }
    }
}

impl Error for NormalizationError {}

impl From<serde_json::Error> for NormalizationError {
    fn from(err: serde_json::Error) -> Self {
        NormalizationError::JsonError(err)
    }
}

/// Data normalizer for converting multi-source JSON to unified format
pub struct DataNormalizer {
    /// Validation enabled
    validation_enabled: bool,
    /// Default values for missing fields
    #[allow(dead_code)]
    default_values: HashMap<String, Value>,
}

impl DataNormalizer {
    /// Create a new data normalizer
    pub fn new() -> Self {
        Self {
            validation_enabled: true,
            default_values: Self::create_default_values(),
        }
    }

    /// Create normalizer with validation disabled
    pub fn without_validation() -> Self {
        Self {
            validation_enabled: false,
            default_values: Self::create_default_values(),
        }
    }

    /// Normalize multi-source JSON data to unified format
    pub fn normalize(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<UnifiedMemoryData, NormalizationError> {
        tracing::info!("🔄 Starting data normalization...");

        // Extract and normalize each data source
        let stats = self.normalize_memory_stats(multi_source)?;
        let allocations = self.normalize_allocations(multi_source)?;
        let performance = self.normalize_performance(multi_source)?;
        let lifecycle = self.normalize_lifecycle(multi_source)?;
        let security = self.normalize_security(multi_source)?;
        let complex_types = self.normalize_complex_types(multi_source)?;
        let variable_relationships = self.normalize_variable_relationships(multi_source)?;
        let metadata = self.normalize_metadata(multi_source)?;

        let unified = UnifiedMemoryData {
            stats,
            allocations,
            performance,
            lifecycle,
            security,
            complex_types,
            variable_relationships,
            metadata,
            multi_source: multi_source.clone(),
        };

        // Validate the unified data if validation is enabled
        if self.validation_enabled {
            self.validate_unified_data(&unified)?;
        }

        tracing::info!("✅ Data normalization completed successfully");
        Ok(unified)
    }

    /// Create default values for missing fields
    fn create_default_values() -> HashMap<String, Value> {
        let mut defaults = HashMap::new();
        defaults.insert("active_memory".to_string(), Value::Number(0.into()));
        defaults.insert("active_allocations".to_string(), Value::Number(0.into()));
        defaults.insert("peak_memory".to_string(), Value::Number(0.into()));
        defaults.insert("total_allocations".to_string(), Value::Number(0.into()));
        defaults.insert("total_allocated".to_string(), Value::Number(0.into()));
        defaults.insert(
            "memory_efficiency".to_string(),
            Value::Number(serde_json::Number::from_f64(0.0).expect("Failed to create JSON number")),
        );
        defaults
    }

    /// Normalize memory statistics
    fn normalize_memory_stats(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<MemoryStatistics, NormalizationError> {
        // Try to get stats from memory_analysis first, then performance
        let memory_data = multi_source.get("memory_analysis");
        let performance_data = multi_source.get("performance");

        let memory_stats = memory_data
            .and_then(|data| data.get("memory_stats"))
            .or_else(|| memory_data.and_then(|data| data.get("stats")));

        let perf_memory = performance_data.and_then(|data| data.get("memory_performance"));

        let metadata = memory_data.and_then(|data| data.get("metadata"));

        Ok(MemoryStatistics {
            active_memory: self
                .extract_usize(memory_stats, "active_memory")
                .or_else(|| self.extract_usize(perf_memory, "active_memory"))
                .unwrap_or(0),
            active_allocations: self.count_active_allocations(memory_data),
            peak_memory: self
                .extract_usize(memory_stats, "peak_memory")
                .or_else(|| self.extract_usize(perf_memory, "peak_memory"))
                .unwrap_or(0),
            total_allocations: self
                .extract_usize(memory_stats, "total_allocations")
                .or_else(|| self.extract_usize(perf_memory, "total_allocated"))
                .or_else(|| self.extract_usize(metadata, "total_allocations"))
                .unwrap_or(0),
            total_allocated: self
                .extract_usize(memory_stats, "total_allocated")
                .or_else(|| self.extract_usize(perf_memory, "total_allocated"))
                .unwrap_or(0),
            memory_efficiency: self
                .extract_f64(perf_memory, "memory_efficiency")
                .unwrap_or(0.0),
        })
    }

    /// Count active allocations from allocation array
    fn count_active_allocations(&self, memory_data: Option<&Value>) -> usize {
        memory_data
            .and_then(|data| data.get("allocations"))
            .and_then(|allocs| allocs.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0)
    }

    /// Normalize allocations data
    fn normalize_allocations(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<Vec<AllocationInfo>, NormalizationError> {
        // Try to get allocations from memory_analysis or unsafe_ffi
        let memory_data = multi_source.get("memory_analysis");
        let unsafe_ffi_data = multi_source.get("unsafe_ffi");
        
        let empty_vec = vec![];
        let allocations_array = memory_data
            .and_then(|data| data.get("allocations"))
            .and_then(|allocs| allocs.as_array())
            .or_else(|| unsafe_ffi_data
                .and_then(|data| data.get("allocations"))
                .and_then(|allocs| allocs.as_array()))
            .unwrap_or(&empty_vec);

        let mut normalized_allocations = Vec::new();

        for (index, alloc) in allocations_array.iter().enumerate() {
            if let Some(_alloc_obj) = alloc.as_object() {
                // Extract borrow_info if present
                let borrow_info = alloc.get("borrow_info").and_then(|bi| {
                    Some(BorrowInfo {
                        immutable_borrows: self.extract_u32(Some(bi), "immutable_borrows").unwrap_or(0),
                        mutable_borrows: self.extract_u32(Some(bi), "mutable_borrows").unwrap_or(0),
                        max_concurrent_borrows: self.extract_u32(Some(bi), "max_concurrent_borrows").unwrap_or(0),
                        last_borrow_timestamp: self.extract_u64(Some(bi), "last_borrow_timestamp").unwrap_or(0),
                    })
                });

                // Extract clone_info if present
                let clone_info = alloc.get("clone_info").and_then(|ci| {
                    Some(CloneInfo {
                        clone_count: self.extract_u32(Some(ci), "clone_count").unwrap_or(0),
                        is_clone: self.extract_bool(Some(ci), "is_clone").unwrap_or(false),
                        original_ptr: self.extract_string(Some(ci), "original_ptr"),
                    })
                });

                // Extract safety_violations if present
                let safety_violations = alloc.get("safety_violations")
                    .and_then(|sv| sv.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect());

                let allocation_info = AllocationInfo {
                    ptr: self
                        .extract_string(Some(alloc), "ptr")
                        .unwrap_or_else(|| format!("0x{:x}", index)),
                    size: self.extract_usize(Some(alloc), "size").unwrap_or(0),
                    var_name: self.extract_string(Some(alloc), "var_name"),
                    type_name: self.extract_string(Some(alloc), "type_name"),
                    scope_name: self
                        .extract_string(Some(alloc), "scope_name")
                        .or_else(|| self.extract_string(Some(alloc), "scope")),
                    timestamp_alloc: self
                        .extract_u64(Some(alloc), "timestamp_alloc")
                        .or_else(|| self.extract_u64(Some(alloc), "timestamp"))
                        .unwrap_or(0),
                    timestamp_dealloc: self.extract_u64(Some(alloc), "timestamp_dealloc"),
                    thread_id: self.extract_string(Some(alloc), "thread_id"),
                    borrow_count: self.extract_u32(Some(alloc), "borrow_count"),
                    stack_trace: self.extract_string_array(Some(alloc), "stack_trace"),
                    is_leaked: self.extract_bool(Some(alloc), "is_leaked").unwrap_or(false),
                    lifetime_ms: self.extract_u64(Some(alloc), "lifetime_ms"),
                    borrow_info,
                    clone_info,
                    ownership_history_available: self.extract_bool(Some(alloc), "ownership_history_available"),
                    ffi_tracked: self.extract_bool(Some(alloc), "ffi_tracked"),
                    safety_violations,
                };
                normalized_allocations.push(allocation_info);
            }
        }

        tracing::info!("📊 Normalized {} allocations", normalized_allocations.len());
        Ok(normalized_allocations)
    }

    /// Normalize performance data
    fn normalize_performance(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<PerformanceMetrics, NormalizationError> {
        let performance_data = multi_source.get("performance");
        let export_perf = performance_data.and_then(|data| data.get("export_performance"));
        let memory_perf = performance_data.and_then(|data| data.get("memory_performance"));
        let alloc_dist = performance_data.and_then(|data| data.get("allocation_distribution"));
        let opt_status = performance_data.and_then(|data| data.get("optimization_status"));

        Ok(PerformanceMetrics {
            processing_time_ms: self
                .extract_u64(export_perf, "total_processing_time_ms")
                .unwrap_or(0),
            allocations_per_second: export_perf
                .and_then(|data| data.get("processing_rate"))
                .and_then(|rate| self.extract_f64(Some(rate), "allocations_per_second"))
                .unwrap_or(0.0),
            memory_efficiency: self
                .extract_f64(memory_perf, "memory_efficiency")
                .unwrap_or(0.0),
            optimization_status: OptimizationStatus {
                parallel_processing: self
                    .extract_bool(opt_status, "parallel_processing")
                    .unwrap_or(false),
                schema_validation: self
                    .extract_bool(opt_status, "schema_validation")
                    .unwrap_or(false),
                streaming_enabled: self
                    .extract_bool(opt_status, "streaming_enabled")
                    .unwrap_or(false),
                batch_size: self.extract_usize(opt_status, "batch_size"),
                buffer_size_kb: self.extract_usize(opt_status, "buffer_size_kb"),
            },
            allocation_distribution: AllocationDistribution {
                tiny: self.extract_usize(alloc_dist, "tiny").unwrap_or(0),
                small: self.extract_usize(alloc_dist, "small").unwrap_or(0),
                medium: self.extract_usize(alloc_dist, "medium").unwrap_or(0),
                large: self.extract_usize(alloc_dist, "large").unwrap_or(0),
                massive: self.extract_usize(alloc_dist, "massive").unwrap_or(0),
            },
        })
    }

    /// Normalize lifecycle data
    fn normalize_lifecycle(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<LifecycleAnalysis, NormalizationError> {
        let empty_object = Value::Object(serde_json::Map::new());
        let lifecycle_data = multi_source.get("lifetime").unwrap_or(&empty_object);

        Ok(LifecycleAnalysis {
            lifecycle_events: lifecycle_data
                .get("lifecycle_events")
                .and_then(|events| events.as_array())
                .cloned()
                .unwrap_or_default(),
            scope_analysis: lifecycle_data
                .get("scope_analysis")
                .and_then(|scope| scope.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            variable_lifetimes: lifecycle_data
                .get("variable_lifetimes")
                .and_then(|lifetimes| lifetimes.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        })
    }

    /// Normalize security data
    fn normalize_security(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<SecurityAnalysis, NormalizationError> {
        let security_data = multi_source.get("security_violations");
        let security_summary = security_data
            .and_then(|data| data.get("security_summary"))
            .and_then(|summary| summary.get("security_analysis_summary"));
        let severity = security_summary.and_then(|summary| summary.get("severity_breakdown"));

        Ok(SecurityAnalysis {
            total_violations: self
                .extract_usize(security_summary, "total_violations")
                .unwrap_or(0),
            risk_level: security_summary
                .and_then(|summary| summary.get("risk_assessment"))
                .and_then(|risk| self.extract_string(Some(risk), "risk_level"))
                .unwrap_or_else(|| "Unknown".to_string()),
            severity_breakdown: SeverityBreakdown {
                critical: self.extract_usize(severity, "critical").unwrap_or(0),
                high: self.extract_usize(severity, "high").unwrap_or(0),
                medium: self.extract_usize(severity, "medium").unwrap_or(0),
                low: self.extract_usize(severity, "low").unwrap_or(0),
                info: self.extract_usize(severity, "info").unwrap_or(0),
            },
            violation_reports: security_data
                .and_then(|data| data.get("violation_reports"))
                .and_then(|reports| reports.as_array())
                .cloned()
                .unwrap_or_default(),
            recommendations: security_data
                .and_then(|data| data.get("analysis_recommendations"))
                .and_then(|recs| recs.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// Normalize complex types data
    fn normalize_complex_types(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<ComplexTypeAnalysis, NormalizationError> {
        let empty_object = Value::Object(serde_json::Map::new());
        let complex_data = multi_source.get("complex_types").unwrap_or(&empty_object);
        let categorized = complex_data.get("categorized_types");
        let summary = complex_data.get("summary");
        let complexity_dist = summary.and_then(|s| s.get("complexity_distribution"));

        Ok(ComplexTypeAnalysis {
            categorized_types: CategorizedTypes {
                collections: categorized
                    .and_then(|cat| cat.get("collections"))
                    .and_then(|coll| coll.as_array())
                    .cloned()
                    .unwrap_or_default(),
                generic_types: categorized
                    .and_then(|cat| cat.get("generic_types"))
                    .and_then(|gen| gen.as_array())
                    .cloned()
                    .unwrap_or_default(),
                smart_pointers: categorized
                    .and_then(|cat| cat.get("smart_pointers"))
                    .and_then(|smart| smart.as_array())
                    .cloned()
                    .unwrap_or_default(),
                trait_objects: categorized
                    .and_then(|cat| cat.get("trait_objects"))
                    .and_then(|traits| traits.as_array())
                    .cloned()
                    .unwrap_or_default(),
            },
            complex_type_analysis: complex_data
                .get("complex_type_analysis")
                .and_then(|analysis| analysis.as_array())
                .cloned()
                .unwrap_or_default(),
            summary: ComplexTypeSummary {
                total_complex_types: self
                    .extract_usize(summary, "total_complex_types")
                    .unwrap_or(0),
                complexity_distribution: ComplexityDistribution {
                    low_complexity: self
                        .extract_usize(complexity_dist, "low_complexity")
                        .unwrap_or(0),
                    medium_complexity: self
                        .extract_usize(complexity_dist, "medium_complexity")
                        .unwrap_or(0),
                    high_complexity: self
                        .extract_usize(complexity_dist, "high_complexity")
                        .unwrap_or(0),
                    very_high_complexity: self
                        .extract_usize(complexity_dist, "very_high_complexity")
                        .unwrap_or(0),
                },
            },
        })
    }

    /// Normalize variable relationships data
    fn normalize_variable_relationships(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<VariableRelationships, NormalizationError> {
        let empty_object = Value::Object(serde_json::Map::new());
        let var_data = multi_source
            .get("variable_relationships")
            .unwrap_or(&empty_object);

        Ok(VariableRelationships {
            relationships: var_data
                .get("variable_relationships")
                .and_then(|rels| rels.as_array())
                .cloned()
                .unwrap_or_default(),
            registry: var_data
                .get("variable_registry")
                .and_then(|reg| reg.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            dependency_graph: var_data
                .get("dependency_graph")
                .and_then(|graph| graph.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            scope_hierarchy: var_data
                .get("scope_hierarchy")
                .and_then(|hierarchy| hierarchy.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
        })
    }

    /// Normalize metadata
    fn normalize_metadata(
        &self,
        multi_source: &HashMap<String, Value>,
    ) -> Result<AnalysisMetadata, NormalizationError> {
        let memory_data = multi_source.get("memory_analysis");
        let metadata = memory_data.and_then(|data| data.get("metadata"));

        Ok(AnalysisMetadata {
            timestamp: self.extract_u64(metadata, "timestamp").unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
            export_version: self
                .extract_string(metadata, "export_version")
                .unwrap_or_else(|| "2.0".to_string()),
            analysis_type: self
                .extract_string(metadata, "analysis_type")
                .unwrap_or_else(|| "integrated_analysis".to_string()),
            data_integrity_hash: self.extract_string(metadata, "data_integrity_hash"),
        })
    }

    /// Validate unified data structure
    fn validate_unified_data(&self, data: &UnifiedMemoryData) -> Result<(), NormalizationError> {
        // Basic validation checks
        if data.stats.active_memory > data.stats.peak_memory && data.stats.peak_memory > 0 {
            return Err(NormalizationError::ValidationError(
                "Active memory cannot exceed peak memory".to_string(),
            ));
        }

        if data.stats.active_allocations > data.stats.total_allocations
            && data.stats.total_allocations > 0
        {
            return Err(NormalizationError::ValidationError(
                "Active allocations cannot exceed total allocations".to_string(),
            ));
        }

        // Validate allocation data consistency
        let actual_active_count = data
            .allocations
            .iter()
            .filter(|alloc| alloc.timestamp_dealloc.is_none())
            .count();

        if actual_active_count != data.stats.active_allocations && data.stats.active_allocations > 0
        {
            tracing::info!(
                "⚠️  Warning: Active allocation count mismatch (stats: {}, actual: {})",
                data.stats.active_allocations,
                actual_active_count
            );
        }

        tracing::info!("✅ Data validation passed");
        Ok(())
    }

    // Helper methods for data extraction

    fn extract_usize(&self, data: Option<&Value>, field: &str) -> Option<usize> {
        data?.get(field)?.as_u64().map(|v| v as usize)
    }

    fn extract_u64(&self, data: Option<&Value>, field: &str) -> Option<u64> {
        data?.get(field)?.as_u64()
    }

    fn extract_f64(&self, data: Option<&Value>, field: &str) -> Option<f64> {
        data?.get(field)?.as_f64()
    }

    fn extract_bool(&self, data: Option<&Value>, field: &str) -> Option<bool> {
        data?.get(field)?.as_bool()
    }

    fn extract_string(&self, data: Option<&Value>, field: &str) -> Option<String> {
        data?.get(field)?.as_str().map(|s| s.to_string())
    }

    fn extract_string_array(&self, data: Option<&Value>, field: &str) -> Option<Vec<String>> {
        data?
            .get(field)?
            .as_array()?
            .iter()
            .map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    }

    fn extract_u32(&self, data: Option<&Value>, field: &str) -> Option<u32> {
        data?.get(field)?.as_u64().map(|v| v as u32)
    }
}

impl Default for DataNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
