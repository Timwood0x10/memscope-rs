//! Allocation types for memory tracking.
//!
//! This module contains types related to memory allocation tracking,
//! including allocation information, borrow tracking, and clone tracking.

use serde::{Deserialize, Serialize};
use std::thread;

// Import related types from other modules
use super::access_tracking::MemoryAccessTrackingInfo;
use super::drop_chain::DropChainAnalysis;
use super::dynamic_type::DynamicTypeInfo;
use super::fragmentation::EnhancedFragmentationAnalysis;
use super::generic::{GenericInstantiationInfo, GenericTypeInfo};
use super::lifecycle::ObjectLifecycleInfo;
use super::memory_layout::MemoryLayoutInfo;
use super::ownership::TypeRelationshipInfo;
use super::runtime_state::RuntimeStateInfo;
use super::smart_pointer::SmartPointerInfo;
use super::stack::StackAllocationInfo;
use super::temporary::TemporaryObjectInfo;
use super::tracking::FunctionCallTrackingInfo;

/// Enhanced borrowing information for allocations.
///
/// Tracks borrow patterns including immutable and mutable borrows,
/// concurrent borrow counts, and timestamp information.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Total number of immutable borrows during lifetime.
    pub immutable_borrows: usize,
    /// Total number of mutable borrows during lifetime.
    pub mutable_borrows: usize,
    /// Peak number of simultaneous borrows observed.
    pub max_concurrent_borrows: usize,
    /// Timestamp of the last borrow event.
    pub last_borrow_timestamp: Option<u64>,

    /// Source of this data (captured or inferred)
    #[serde(default)]
    pub _source: Option<String>,
    /// Confidence level for inferred data
    #[serde(default)]
    pub _confidence: Option<String>,
}

/// Enhanced cloning information for allocations.
///
/// Tracks clone operations including clone counts, original source
/// tracking, and clone relationships.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloneInfo {
    /// Number of times this object was cloned.
    pub clone_count: usize,
    /// Whether this allocation itself is a result of a clone.
    pub is_clone: bool,
    /// If is_clone is true, points to the original object's pointer.
    pub original_ptr: Option<usize>,

    /// Source of this data (captured or inferred)
    #[serde(default)]
    pub _source: Option<String>,
    /// Confidence level for inferred data
    #[serde(default)]
    pub _confidence: Option<String>,
}

/// Information about a memory allocation.
///
/// Comprehensive tracking information for a single memory allocation,
/// including size, type, scope, lifecycle, and various analysis metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct AllocationInfo {
    /// Memory address of the allocation.
    pub ptr: usize,
    /// Size of the allocation in bytes.
    pub size: usize,
    /// Optional variable name associated with this allocation.
    pub var_name: Option<String>,
    /// Optional type name of the allocated data.
    pub type_name: Option<String>,
    /// Optional scope name where the allocation occurred.
    pub scope_name: Option<String>,
    /// Timestamp when the allocation was made.
    pub timestamp_alloc: u64,
    /// Optional timestamp when the allocation was deallocated.
    pub timestamp_dealloc: Option<u64>,
    /// Thread ID where the allocation occurred.
    pub thread_id: thread::ThreadId,
    /// Thread ID as u64 (original value from tracking).
    pub thread_id_u64: u64,
    /// Number of active borrows for this allocation.
    pub borrow_count: usize,
    /// Optional stack trace at the time of allocation.
    pub stack_trace: Option<Vec<String>>,
    /// Whether this allocation is considered leaked.
    pub is_leaked: bool,
    /// Precise lifetime in milliseconds (calculated from creation to destruction).
    pub lifetime_ms: Option<u64>,
    /// Enhanced borrowing information.
    pub borrow_info: Option<BorrowInfo>,
    /// Enhanced cloning information.
    pub clone_info: Option<CloneInfo>,
    /// Flag indicating if detailed ownership history is available in lifetime.json.
    pub ownership_history_available: bool,
    /// Smart pointer specific information.
    pub smart_pointer_info: Option<SmartPointerInfo>,
    /// Detailed memory layout information.
    pub memory_layout: Option<MemoryLayoutInfo>,
    /// Module path (from module_path!())
    pub module_path: Option<String>,
    /// Stack pointer (for StackOwner types like Arc/Rc)
    pub stack_ptr: Option<usize>,
    /// Generic type information.
    pub generic_info: Option<GenericTypeInfo>,
    /// Dynamic type information (trait objects).
    pub dynamic_type_info: Option<DynamicTypeInfo>,
    /// Runtime state information.
    pub runtime_state: Option<RuntimeStateInfo>,
    /// Stack allocation information (if allocated on stack).
    pub stack_allocation: Option<StackAllocationInfo>,
    /// Temporary object information (if this is a temporary).
    pub temporary_object: Option<TemporaryObjectInfo>,
    /// Memory fragmentation analysis.
    pub fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
    /// Enhanced generic instantiation tracking.
    pub generic_instantiation: Option<GenericInstantiationInfo>,
    /// Type relationship information.
    pub type_relationships: Option<TypeRelationshipInfo>,
    /// Type usage information.
    pub type_usage: Option<TypeUsageInfo>,
    /// Function call tracking (if allocation is function-related).
    pub function_call_tracking: Option<FunctionCallTrackingInfo>,
    /// Object lifecycle tracking.
    pub lifecycle_tracking: Option<ObjectLifecycleInfo>,
    /// Memory access pattern tracking.
    pub access_tracking: Option<MemoryAccessTrackingInfo>,
    /// Drop chain analysis (when object is dropped).
    pub drop_chain_analysis: Option<DropChainAnalysis>,
}

impl From<crate::core::types::AllocationInfo> for AllocationInfo {
    fn from(old: crate::core::types::AllocationInfo) -> Self {
        // For thread_id, we use a default value since ThreadId cannot be reliably reconstructed from String
        // In a production system, this would require proper thread ID tracking
        let thread_id = thread::current().id();

        // Convert borrow_info
        let borrow_info = old.borrow_info.map(|b| BorrowInfo {
            immutable_borrows: b.immutable_borrows,
            mutable_borrows: b.mutable_borrows,
            max_concurrent_borrows: b.max_concurrent_borrows,
            last_borrow_timestamp: b.last_borrow_timestamp,
            _source: None,
            _confidence: None,
        });

        // Convert clone_info
        let clone_info = old.clone_info.map(|c| CloneInfo {
            clone_count: c.clone_count,
            is_clone: c.is_clone,
            original_ptr: c.original_ptr,
            _source: None,
            _confidence: None,
        });

        // For complex nested types, we set them to None to avoid complex conversions
        // In a production system, these would need proper conversion implementations
        let smart_pointer_info =
            old.smart_pointer_info
                .map(|s| super::smart_pointer::SmartPointerInfo {
                    data_ptr: s.data_ptr,
                    cloned_from: s.cloned_from,
                    clones: s.clones,
                    ref_count_history: s
                        .ref_count_history
                        .iter()
                        .map(|r| super::smart_pointer::RefCountSnapshot {
                            timestamp: r.timestamp,
                            strong_count: r.strong_count,
                            weak_count: r.weak_count,
                        })
                        .collect(),
                    weak_count: s.weak_count,
                    is_weak_reference: s.is_weak_reference,
                    is_data_owner: s.is_data_owner,
                    is_implicitly_deallocated: s.is_implicitly_deallocated,
                    pointer_type: match s.pointer_type {
                        crate::core::types::SmartPointerType::Rc => {
                            super::smart_pointer::SmartPointerType::Rc
                        }
                        crate::core::types::SmartPointerType::Arc => {
                            super::smart_pointer::SmartPointerType::Arc
                        }
                        crate::core::types::SmartPointerType::RcWeak => {
                            super::smart_pointer::SmartPointerType::RcWeak
                        }
                        crate::core::types::SmartPointerType::ArcWeak => {
                            super::smart_pointer::SmartPointerType::ArcWeak
                        }
                        crate::core::types::SmartPointerType::Box => {
                            super::smart_pointer::SmartPointerType::Box
                        }
                    },
                });

        Self {
            ptr: old.ptr,
            size: old.size,
            var_name: old.var_name,
            type_name: old.type_name,
            scope_name: old.scope_name,
            timestamp_alloc: old.timestamp_alloc,
            timestamp_dealloc: old.timestamp_dealloc,
            thread_id,
            thread_id_u64: old.thread_id.parse().unwrap_or_else(|_| {
                tracing::warn!(
                    "Failed to parse thread_id: '{}', defaulting to 0",
                    old.thread_id
                );
                0
            }),
            borrow_count: old.borrow_count,
            stack_trace: old.stack_trace,
            is_leaked: old.is_leaked,
            lifetime_ms: old.lifetime_ms,
            borrow_info,
            clone_info,
            ownership_history_available: old.ownership_history_available,
            smart_pointer_info,
            // Complex nested types - use Into::into for complete conversion
            memory_layout: old.memory_layout.map(Into::into),
            generic_info: old.generic_info.map(Into::into),
            dynamic_type_info: old.dynamic_type_info.map(Into::into),
            runtime_state: old.runtime_state.map(Into::into),
            stack_allocation: old.stack_allocation.map(Into::into),
            temporary_object: old.temporary_object.map(Into::into),
            fragmentation_analysis: old.fragmentation_analysis.map(Into::into),
            generic_instantiation: old.generic_instantiation.map(Into::into),
            type_relationships: old.type_relationships.map(Into::into),
            type_usage: old.type_usage.map(Into::into),
            function_call_tracking: old.function_call_tracking.map(Into::into),
            lifecycle_tracking: old.lifecycle_tracking.map(Into::into),
            access_tracking: old.access_tracking.map(Into::into),
            drop_chain_analysis: old.drop_chain_analysis.map(Into::into),
            module_path: None, // core::types::AllocationInfo doesn't have module_path
            stack_ptr: None,
        }
    }
}

impl From<crate::capture::backends::core_types::AllocationInfo> for AllocationInfo {
    fn from(info: crate::capture::backends::core_types::AllocationInfo) -> Self {
        Self {
            ptr: info.ptr,
            size: info.size,
            var_name: info.var_name,
            type_name: info.type_name,
            scope_name: None,
            timestamp_alloc: info.allocated_at_ns,
            timestamp_dealloc: None,
            thread_id: std::thread::current().id(),
            thread_id_u64: info.thread_id,
            borrow_count: 0,
            stack_trace: info.stack_trace,
            is_leaked: false,
            lifetime_ms: None,
            module_path: None,
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
            stack_ptr: None, // <--- Added this line
        }
    }
}

/// Type usage information for tracking how types are used across the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeUsageInfo {
    /// Type name.
    pub type_name: String,
    /// Total usage count.
    pub total_usage_count: u64,
    /// Usage contexts.
    pub usage_contexts: Vec<UsageContext>,
    /// Usage patterns over time.
    pub usage_timeline: Vec<UsageTimePoint>,
    /// Hot paths where type is used.
    pub hot_paths: Vec<HotPath>,
    /// Performance impact of usage.
    pub performance_impact: TypePerformanceImpact,
}

/// Usage context information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageContext {
    /// Context type.
    pub context_type: ContextType,
    /// Function or module name.
    pub location: String,
    /// Usage frequency in this context.
    pub frequency: u32,
    /// Performance characteristics in this context.
    pub performance_metrics: ContextPerformanceMetrics,
}

/// Context types where types are used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextType {
    /// Function parameter context.
    FunctionParameter,
    /// Function return context.
    FunctionReturn,
    /// Local variable context.
    LocalVariable,
    /// Struct field context.
    StructField,
    /// Enum variant context.
    EnumVariant,
    /// Trait method context.
    TraitMethod,
    /// Generic constraint context.
    GenericConstraint,
    /// Closure capture context.
    ClosureCapture,
    /// Async context.
    AsyncContext,
    /// Unsafe context.
    UnsafeContext,
}

/// Performance metrics within specific contexts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPerformanceMetrics {
    /// Average execution time in this context.
    pub avg_execution_time_ns: f64,
    /// Memory allocation frequency.
    pub allocation_frequency: f64,
    /// Cache miss rate in this context.
    pub cache_miss_rate: f64,
    /// Branch misprediction rate.
    pub branch_misprediction_rate: f64,
}

/// Usage time point for timeline analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageTimePoint {
    /// Timestamp.
    pub timestamp: u64,
    /// Usage count at this time.
    pub usage_count: u32,
    /// Memory usage at this time.
    pub memory_usage: usize,
    /// Performance metrics at this time.
    pub performance_snapshot: PerformanceSnapshot,
}

/// Performance snapshot at a specific time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// CPU usage percentage.
    pub cpu_usage: f64,
    /// Memory usage percentage.
    pub memory_usage: f64,
    /// Cache hit rate.
    pub cache_hit_rate: f64,
    /// Throughput (operations per second).
    pub throughput: f64,
}

/// Hot path information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotPath {
    /// Path identifier.
    pub path_id: String,
    /// Function call sequence.
    pub call_sequence: Vec<String>,
    /// Execution frequency.
    pub execution_frequency: u64,
    /// Total execution time.
    pub total_execution_time_ns: u64,
    /// Average execution time.
    pub avg_execution_time_ns: f64,
    /// Memory allocations in this path.
    pub memory_allocations: u32,
    /// Performance bottlenecks.
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Performance bottleneck information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck type.
    pub bottleneck_type: BottleneckType,
    /// Location in code.
    pub location: String,
    /// Impact severity.
    pub severity: ImpactLevel,
    /// Description.
    pub description: String,
    /// Suggested optimization.
    pub optimization_suggestion: String,
}

/// Types of performance bottlenecks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BottleneckType {
    /// Memory allocation bottleneck.
    MemoryAllocation,
    /// Memory deallocation bottleneck.
    MemoryDeallocation,
    /// Cache miss bottleneck.
    CacheMiss,
    /// Branch misprediction bottleneck.
    BranchMisprediction,
    /// Function call bottleneck.
    FunctionCall,
    /// Data movement bottleneck.
    DataMovement,
    /// Synchronization bottleneck.
    Synchronization,
    /// I/O bottleneck.
    IO,
}

/// Impact level enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Low impact level.
    Low,
    /// Medium impact level.
    Medium,
    /// High impact level.
    High,
    /// Critical impact level.
    Critical,
}

/// Type performance impact assessment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypePerformanceImpact {
    /// Overall performance score (0-100).
    pub performance_score: f64,
    /// Memory efficiency score (0-100).
    pub memory_efficiency_score: f64,
    /// CPU efficiency score (0-100).
    pub cpu_efficiency_score: f64,
    /// Cache efficiency score (0-100).
    pub cache_efficiency_score: f64,
    /// Optimization recommendations.
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}

/// Optimization recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type.
    pub recommendation_type: RecommendationType,
    /// Priority level.
    pub priority: Priority,
    /// Description.
    pub description: String,
    /// Expected performance improvement.
    pub expected_improvement: f64,
    /// Implementation difficulty.
    pub implementation_difficulty: ImplementationDifficulty,
}

/// Types of optimization recommendations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Memory layout optimization.
    MemoryLayout,
    /// Algorithm change recommendation.
    AlgorithmChange,
    /// Data structure change recommendation.
    DataStructureChange,
    /// Caching strategy recommendation.
    CachingStrategy,
    /// Memory pooling recommendation.
    MemoryPooling,
    /// Lazy initialization recommendation.
    LazyInitialization,
    /// Inlining recommendation.
    Inlining,
    /// Vectorization recommendation.
    Vectorization,
    /// Parallelization recommendation.
    Parallelization,
}

/// Priority levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority.
    Low,
    /// Medium priority.
    Medium,
    /// High priority.
    High,
    /// Critical priority.
    Critical,
}

/// Implementation difficulty levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    /// Easy implementation difficulty.
    Easy,
    /// Medium implementation difficulty.
    Medium,
    /// Hard implementation difficulty.
    Hard,
    /// Very hard implementation difficulty.
    VeryHard,
}

// Custom Serialize implementation for AllocationInfo to handle thread::ThreadId
impl Serialize for AllocationInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AllocationInfo", 25)?;
        state.serialize_field("ptr", &self.ptr)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("var_name", &self.var_name)?;
        state.serialize_field("type_name", &self.type_name)?;
        state.serialize_field("scope_name", &self.scope_name)?;
        state.serialize_field("timestamp_alloc", &self.timestamp_alloc)?;
        state.serialize_field("timestamp_dealloc", &self.timestamp_dealloc)?;
        state.serialize_field("thread_id", &format!("{:?}", self.thread_id))?;
        state.serialize_field("borrow_count", &self.borrow_count)?;
        state.serialize_field("stack_trace", &self.stack_trace)?;
        state.serialize_field("is_leaked", &self.is_leaked)?;
        state.serialize_field("lifetime_ms", &self.lifetime_ms)?;
        state.serialize_field("borrow_info", &self.borrow_info)?;
        state.serialize_field("clone_info", &self.clone_info)?;
        state.serialize_field(
            "ownership_history_available",
            &self.ownership_history_available,
        )?;
        state.serialize_field("smart_pointer_info", &self.smart_pointer_info)?;
        state.serialize_field("memory_layout", &self.memory_layout)?;
        state.serialize_field("generic_info", &self.generic_info)?;
        state.serialize_field("dynamic_type_info", &self.dynamic_type_info)?;
        state.serialize_field("runtime_state", &self.runtime_state)?;
        state.serialize_field("stack_allocation", &self.stack_allocation)?;
        state.serialize_field("temporary_object", &self.temporary_object)?;
        state.serialize_field("fragmentation_analysis", &self.fragmentation_analysis)?;
        state.serialize_field("generic_instantiation", &self.generic_instantiation)?;
        state.serialize_field("type_relationships", &self.type_relationships)?;
        state.serialize_field("type_usage", &self.type_usage)?;
        state.serialize_field("function_call_tracking", &self.function_call_tracking)?;
        state.serialize_field("lifecycle_tracking", &self.lifecycle_tracking)?;
        state.serialize_field("access_tracking", &self.access_tracking)?;
        state.serialize_field("drop_chain_analysis", &self.drop_chain_analysis)?;
        state.serialize_field("module_path", &self.module_path)?;
        state.end()
    }
}

// Custom Deserialize implementation for AllocationInfo due to complex field handling
impl<'de> Deserialize<'de> for AllocationInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AllocationInfoHelper {
            ptr: usize,
            size: usize,
            var_name: Option<String>,
            type_name: Option<String>,
            scope_name: Option<String>,
            timestamp_alloc: u64,
            timestamp_dealloc: Option<u64>,
            #[serde(default)]
            thread_id: String,
            #[serde(default)]
            thread_id_u64: u64,
            borrow_count: usize,
            stack_trace: Option<Vec<String>>,
            is_leaked: bool,
            lifetime_ms: Option<u64>,
            borrow_info: Option<BorrowInfo>,
            clone_info: Option<CloneInfo>,
            ownership_history_available: Option<bool>,
            smart_pointer_info: Option<SmartPointerInfo>,
            memory_layout: Option<MemoryLayoutInfo>,
            generic_info: Option<GenericTypeInfo>,
            dynamic_type_info: Option<DynamicTypeInfo>,
            runtime_state: Option<RuntimeStateInfo>,
            stack_allocation: Option<StackAllocationInfo>,
            temporary_object: Option<TemporaryObjectInfo>,
            fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
            generic_instantiation: Option<GenericInstantiationInfo>,
            type_relationships: Option<TypeRelationshipInfo>,
            type_usage: Option<TypeUsageInfo>,
            function_call_tracking: Option<FunctionCallTrackingInfo>,
            lifecycle_tracking: Option<ObjectLifecycleInfo>,
            access_tracking: Option<MemoryAccessTrackingInfo>,
            drop_chain_analysis: Option<DropChainAnalysis>,
            module_path: Option<String>,
        }

        let helper = AllocationInfoHelper::deserialize(deserializer)?;
        // Parse thread_id from string format, default to current thread if parsing fails
        let thread_id = if !helper.thread_id.is_empty() {
            // Try to parse the thread_id from format like "ThreadId(1234567890)"
            // Since we can't reconstruct the actual ThreadId, we use the current thread
            thread::current().id()
        } else {
            thread::current().id()
        };

        Ok(AllocationInfo {
            ptr: helper.ptr,
            size: helper.size,
            var_name: helper.var_name,
            type_name: helper.type_name,
            scope_name: helper.scope_name,
            timestamp_alloc: helper.timestamp_alloc,
            timestamp_dealloc: helper.timestamp_dealloc,
            thread_id,
            thread_id_u64: helper.thread_id_u64,
            borrow_count: helper.borrow_count,
            stack_trace: helper.stack_trace,
            is_leaked: helper.is_leaked,
            lifetime_ms: helper.lifetime_ms,
            borrow_info: helper.borrow_info,
            clone_info: helper.clone_info,
            ownership_history_available: helper.ownership_history_available.unwrap_or(false),
            smart_pointer_info: helper.smart_pointer_info,
            memory_layout: helper.memory_layout,
            generic_info: helper.generic_info,
            dynamic_type_info: helper.dynamic_type_info,
            runtime_state: helper.runtime_state,
            stack_allocation: helper.stack_allocation,
            temporary_object: helper.temporary_object,
            fragmentation_analysis: helper.fragmentation_analysis,
            generic_instantiation: helper.generic_instantiation,
            type_relationships: helper.type_relationships,
            type_usage: helper.type_usage,
            function_call_tracking: helper.function_call_tracking,
            lifecycle_tracking: helper.lifecycle_tracking,
            access_tracking: helper.access_tracking,
            drop_chain_analysis: helper.drop_chain_analysis,
            module_path: helper.module_path,
            stack_ptr: None,
        })
    }
}

impl AllocationInfo {
    /// Create a new AllocationInfo instance with improved field enhancements.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Memory address of the allocation
    /// * `size` - Size of the allocation in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use memscope_rs::capture::types::AllocationInfo;
    ///
    /// let info = AllocationInfo::new(0x1000, 1024);
    /// assert_eq!(info.ptr, 0x1000);
    /// assert_eq!(info.size, 1024);
    /// assert!(info.is_active());
    /// ```
    pub fn new(ptr: usize, size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            ptr,
            size,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: timestamp,
            timestamp_dealloc: None,
            thread_id: thread::current().id(),
            thread_id_u64: {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                thread::current().id().hash(&mut hasher);
                hasher.finish()
            },
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(1), // Default: 1ms (just allocated)
            borrow_info: None,    // No borrow info available by default
            clone_info: Some(CloneInfo {
                clone_count: 0,
                is_clone: false,
                original_ptr: None,
                _source: None,
                _confidence: None,
            }),
            ownership_history_available: false, // Not available without tracking
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
            module_path: None,
            stack_ptr: None,
        }
    }

    /// Set source location (file and line) for this allocation.
    pub fn set_source_location(&mut self, file: &str, line: u32) {
        let frame = format!("{}:{}", file, line);
        self.stack_trace.get_or_insert_with(Vec::new).push(frame);
    }

    /// Set smart pointer information for this allocation.
    pub fn set_smart_pointer_info(&mut self, info: SmartPointerInfo) {
        self.smart_pointer_info = Some(info);
    }

    /// Mark this allocation as deallocated with current timestamp.
    pub fn mark_deallocated(&mut self) {
        self.timestamp_dealloc = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
    }

    /// Check if this allocation is still active (not deallocated).
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }

    /// Update allocation with type-specific enhancements using inference engine.
    ///
    /// ⚠️ WARNING: This method uses inference/heuristics. Data may be WRONG.
    /// All inferred data is marked with `_source: "inferred"` and confidence level.
    ///
    /// Detects common patterns for Rc/Arc, Vec, String, HashMap, and Box types.
    pub fn enhance_with_type_info(&mut self, type_name: &str) {
        if self.timestamp_dealloc.is_none() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let elapsed_ns = current_time.saturating_sub(self.timestamp_alloc);
            let elapsed_ms = elapsed_ns / 1_000_000;
            self.lifetime_ms = Some(if elapsed_ms == 0 { 1 } else { elapsed_ms });
        }

        // Use inference engine for borrow and clone info
        // ⚠️ All data below is INFERRED, not captured
        if type_name.contains("Rc<") || type_name.contains("Arc<") {
            self.clone_info = Some(CloneInfo {
                clone_count: 2,
                is_clone: false,
                original_ptr: None,
                _source: Some("inferred".to_string()),
                _confidence: Some("low".to_string()),
            });

            self.borrow_info = Some(BorrowInfo {
                immutable_borrows: 5,
                mutable_borrows: 0,
                max_concurrent_borrows: 5,
                last_borrow_timestamp: Some(self.timestamp_alloc + 1000000),
                _source: Some("inferred".to_string()),
                _confidence: Some("low".to_string()),
            });
        } else if type_name.contains("Vec<")
            || type_name.contains("String")
            || type_name.contains("HashMap")
        {
            self.borrow_info = Some(BorrowInfo {
                immutable_borrows: 4,
                mutable_borrows: 2,
                max_concurrent_borrows: 3,
                last_borrow_timestamp: Some(self.timestamp_alloc + 800000),
                _source: Some("inferred".to_string()),
                _confidence: Some("low".to_string()),
            });
        } else if type_name.contains("Box<") {
            self.clone_info = Some(CloneInfo {
                clone_count: 0,
                is_clone: false,
                original_ptr: None,
                _source: Some("inferred".to_string()),
                _confidence: Some("low".to_string()),
            });

            self.borrow_info = Some(BorrowInfo {
                immutable_borrows: 2,
                mutable_borrows: 1,
                max_concurrent_borrows: 1,
                last_borrow_timestamp: Some(self.timestamp_alloc + 300000),
                _source: Some("inferred".to_string()),
                _confidence: Some("low".to_string()),
            });
        }
    }

    /// Enhance with inference engine (explicit inference)
    ///
    /// ⚠️ WARNING: All data from this method is INFERRED.
    pub fn enhance_with_inference(&mut self, engine: &crate::capture::inference::InferenceEngine) {
        let type_name = self.type_name.as_deref();

        let inferred_borrow = engine.infer_borrow_info(type_name);
        self.borrow_info = Some(BorrowInfo {
            immutable_borrows: inferred_borrow.immutable_borrows,
            mutable_borrows: inferred_borrow.mutable_borrows,
            max_concurrent_borrows: inferred_borrow.max_concurrent_borrows,
            last_borrow_timestamp: None,
            _source: Some("inferred".to_string()),
            _confidence: Some(format!("{:?}", inferred_borrow._confidence).to_lowercase()),
        });

        let inferred_sp = engine.infer_smart_pointer(type_name);
        if inferred_sp.pointer_type != crate::capture::inference::SmartPointerType::Unknown {
            self.clone_info = Some(CloneInfo {
                clone_count: inferred_sp.ref_count.unwrap_or(0),
                is_clone: false,
                original_ptr: None,
                _source: Some("inferred".to_string()),
                _confidence: Some(format!("{:?}", inferred_sp._confidence).to_lowercase()),
            });
        }
    }
}

/// Risk distribution analysis for memory allocations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskDistribution {
    /// Low risk allocations count.
    pub low_risk: usize,
    /// Medium risk allocations count.
    pub medium_risk: usize,
    /// High risk allocations count.
    pub high_risk: usize,
    /// Critical risk allocations count.
    pub critical_risk: usize,
}

// Implement From trait for converting from core::types to capture::types
impl From<crate::core::types::TypeUsageInfo> for TypeUsageInfo {
    fn from(old: crate::core::types::TypeUsageInfo) -> Self {
        Self {
            type_name: old.type_name,
            total_usage_count: old.total_usage_count,
            usage_contexts: old
                .usage_contexts
                .into_iter()
                .map(|c| UsageContext {
                    context_type: match c.context_type {
                        crate::core::types::ContextType::FunctionParameter => {
                            ContextType::FunctionParameter
                        }
                        crate::core::types::ContextType::FunctionReturn => {
                            ContextType::FunctionReturn
                        }
                        crate::core::types::ContextType::LocalVariable => {
                            ContextType::LocalVariable
                        }
                        crate::core::types::ContextType::StructField => ContextType::StructField,
                        crate::core::types::ContextType::EnumVariant => ContextType::EnumVariant,
                        crate::core::types::ContextType::TraitMethod => ContextType::TraitMethod,
                        crate::core::types::ContextType::GenericConstraint => {
                            ContextType::GenericConstraint
                        }
                        crate::core::types::ContextType::ClosureCapture => {
                            ContextType::ClosureCapture
                        }
                        crate::core::types::ContextType::AsyncContext => ContextType::AsyncContext,
                        crate::core::types::ContextType::UnsafeContext => {
                            ContextType::UnsafeContext
                        }
                    },
                    location: c.location,
                    frequency: c.frequency,
                    performance_metrics: ContextPerformanceMetrics {
                        avg_execution_time_ns: c.performance_metrics.avg_execution_time_ns,
                        allocation_frequency: c.performance_metrics.allocation_frequency,
                        cache_miss_rate: c.performance_metrics.cache_miss_rate,
                        branch_misprediction_rate: c.performance_metrics.branch_misprediction_rate,
                    },
                })
                .collect(),
            usage_timeline: old
                .usage_timeline
                .into_iter()
                .map(|t| UsageTimePoint {
                    timestamp: t.timestamp,
                    usage_count: t.usage_count,
                    memory_usage: t.memory_usage,
                    performance_snapshot: PerformanceSnapshot {
                        cpu_usage: t.performance_snapshot.cpu_usage,
                        memory_usage: t.performance_snapshot.memory_usage,
                        cache_hit_rate: t.performance_snapshot.cache_hit_rate,
                        throughput: t.performance_snapshot.throughput,
                    },
                })
                .collect(),
            hot_paths: old
                .hot_paths
                .into_iter()
                .map(|h| HotPath {
                    path_id: h.path_id,
                    call_sequence: h.call_sequence,
                    execution_frequency: h.execution_frequency,
                    total_execution_time_ns: h.total_execution_time_ns,
                    avg_execution_time_ns: h.avg_execution_time_ns,
                    memory_allocations: h.memory_allocations,
                    bottlenecks: h
                        .bottlenecks
                        .into_iter()
                        .map(|b| PerformanceBottleneck {
                            bottleneck_type: match b.bottleneck_type {
                                crate::core::types::BottleneckType::MemoryAllocation => {
                                    BottleneckType::MemoryAllocation
                                }
                                crate::core::types::BottleneckType::MemoryDeallocation => {
                                    BottleneckType::MemoryDeallocation
                                }
                                crate::core::types::BottleneckType::CacheMiss => {
                                    BottleneckType::CacheMiss
                                }
                                crate::core::types::BottleneckType::BranchMisprediction => {
                                    BottleneckType::BranchMisprediction
                                }
                                crate::core::types::BottleneckType::FunctionCall => {
                                    BottleneckType::FunctionCall
                                }
                                crate::core::types::BottleneckType::DataMovement => {
                                    BottleneckType::DataMovement
                                }
                                crate::core::types::BottleneckType::Synchronization => {
                                    BottleneckType::Synchronization
                                }
                                crate::core::types::BottleneckType::IO => BottleneckType::IO,
                            },
                            location: b.location,
                            severity: match b.severity {
                                crate::core::types::ImpactLevel::Low => ImpactLevel::Low,
                                crate::core::types::ImpactLevel::Medium => ImpactLevel::Medium,
                                crate::core::types::ImpactLevel::High => ImpactLevel::High,
                                crate::core::types::ImpactLevel::Critical => ImpactLevel::Critical,
                            },
                            description: b.description,
                            optimization_suggestion: b.optimization_suggestion,
                        })
                        .collect(),
                })
                .collect(),
            performance_impact: TypePerformanceImpact {
                performance_score: old.performance_impact.performance_score,
                memory_efficiency_score: old.performance_impact.memory_efficiency_score,
                cpu_efficiency_score: old.performance_impact.cpu_efficiency_score,
                cache_efficiency_score: old.performance_impact.cache_efficiency_score,
                optimization_recommendations: old
                    .performance_impact
                    .optimization_recommendations
                    .into_iter()
                    .map(|r| OptimizationRecommendation {
                        recommendation_type: match r.recommendation_type {
                            crate::core::types::RecommendationType::MemoryLayout => {
                                RecommendationType::MemoryLayout
                            }
                            crate::core::types::RecommendationType::AlgorithmChange => {
                                RecommendationType::AlgorithmChange
                            }
                            crate::core::types::RecommendationType::DataStructureChange => {
                                RecommendationType::DataStructureChange
                            }
                            crate::core::types::RecommendationType::CachingStrategy => {
                                RecommendationType::CachingStrategy
                            }
                            crate::core::types::RecommendationType::MemoryPooling => {
                                RecommendationType::MemoryPooling
                            }
                            crate::core::types::RecommendationType::LazyInitialization => {
                                RecommendationType::LazyInitialization
                            }
                            crate::core::types::RecommendationType::Inlining => {
                                RecommendationType::Inlining
                            }
                            crate::core::types::RecommendationType::Vectorization => {
                                RecommendationType::Vectorization
                            }
                            crate::core::types::RecommendationType::Parallelization => {
                                RecommendationType::Parallelization
                            }
                        },
                        priority: match r.priority {
                            crate::core::types::Priority::Low => Priority::Low,
                            crate::core::types::Priority::Medium => Priority::Medium,
                            crate::core::types::Priority::High => Priority::High,
                            crate::core::types::Priority::Critical => Priority::Critical,
                        },
                        description: r.description,
                        expected_improvement: r.expected_improvement,
                        implementation_difficulty: match r.implementation_difficulty {
                            crate::core::types::ImplementationDifficulty::Easy => {
                                ImplementationDifficulty::Easy
                            }
                            crate::core::types::ImplementationDifficulty::Medium => {
                                ImplementationDifficulty::Medium
                            }
                            crate::core::types::ImplementationDifficulty::Hard => {
                                ImplementationDifficulty::Hard
                            }
                            crate::core::types::ImplementationDifficulty::VeryHard => {
                                ImplementationDifficulty::VeryHard
                            }
                        },
                    })
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_info_creation() {
        let info = AllocationInfo::new(0x12345678, 1024);

        assert_eq!(info.ptr, 0x12345678);
        assert_eq!(info.size, 1024);
        assert!(info.is_active());
        assert!(info.borrow_info.is_none());
        assert!(info.clone_info.is_some());
        assert!(!info.ownership_history_available);
    }

    #[test]
    fn test_allocation_info_mark_deallocated() {
        let mut info = AllocationInfo::new(0x1000, 512);
        assert!(info.is_active());

        info.mark_deallocated();
        assert!(!info.is_active());
        assert!(info.timestamp_dealloc.is_some());
    }

    #[test]
    fn test_allocation_info_enhance_with_type_info() {
        let mut info = AllocationInfo::new(0x1000, 512);

        // Test with Rc type
        info.enhance_with_type_info("std::rc::Rc<String>");
        if let Some(clone_info) = &info.clone_info {
            assert_eq!(clone_info.clone_count, 2);
        }

        // Test with Vec type
        info.enhance_with_type_info("Vec<i32>");
        if let Some(borrow_info) = &info.borrow_info {
            assert_eq!(borrow_info.immutable_borrows, 4);
            assert_eq!(borrow_info.mutable_borrows, 2);
        }
    }

    #[test]
    fn test_borrow_info_default() {
        let borrow_info = BorrowInfo::default();

        assert_eq!(borrow_info.immutable_borrows, 0);
        assert_eq!(borrow_info.mutable_borrows, 0);
        assert_eq!(borrow_info.max_concurrent_borrows, 0);
        assert_eq!(borrow_info.last_borrow_timestamp, None);
    }

    #[test]
    fn test_clone_info_default() {
        let clone_info = CloneInfo::default();

        assert_eq!(clone_info.clone_count, 0);
        assert!(!clone_info.is_clone);
        assert_eq!(clone_info.original_ptr, None);
    }

    #[test]
    fn test_allocation_info_serialization() {
        let info = AllocationInfo::new(0x1000, 512);

        // Test that it can be serialized
        let serialized = serde_json::to_string(&info);
        assert!(serialized.is_ok());

        // Test that serialized data contains expected fields
        let json_str = serialized.expect("Failed to serialize AllocationInfo to JSON");
        assert!(json_str.contains("ptr"));
        assert!(json_str.contains("size"));
    }
}
