// Types module - Refactored from the original monolithic types.rs
// This module organizes types into logical groups for better maintainability

// For now, we need to import the original types from the backup file
// Since we removed types_original.rs, we need to recreate the essential types here

// Essential types that are used throughout the codebase
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;

/// Result type for tracking operations
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Comprehensive error type for the tracking system
#[derive(Debug)]
pub enum TrackingError {
    /// Memory allocation operation failed
    AllocationFailed(String),
    /// Memory deallocation operation failed
    DeallocationFailed(String),
    /// Memory tracking is currently disabled
    TrackingDisabled,
    /// The provided pointer is invalid or null
    InvalidPointer(String),
    /// Error occurred during data serialization
    SerializationError(String),
    /// Error occurred during visualization generation
    VisualizationError(String),
    /// Thread safety violation detected
    ThreadSafetyError(String),
    /// Configuration parameter is invalid
    ConfigurationError(String),
    /// Error occurred during memory analysis
    AnalysisError(String),
    /// Error occurred during data export
    ExportError(String),
    /// Memory corruption detected
    MemoryCorruption(String),
    /// Unsafe operation detected and flagged
    UnsafeOperationDetected(String),
    /// Foreign Function Interface error
    FFIError(String),
    /// Scope management error
    ScopeError(String),
    /// Borrow checker violation detected
    BorrowCheckError(String),
    /// Lifetime management error
    LifetimeError(String),
    /// Type inference failed
    TypeInferenceError(String),
    /// Performance threshold exceeded
    PerformanceError(String),
    /// System resources exhausted
    ResourceExhausted(String),
    /// Internal system error
    InternalError(String),
    /// Input/output operation failed
    IoError(String),
    /// Lock acquisition failed
    LockError(String),
    /// Channel communication error
    ChannelError(String),
    /// Thread operation error
    ThreadError(String),
    /// Initialization error
    InitializationError(String),
    /// Feature not implemented
    NotImplemented(String),
    /// Invalid operation
    InvalidOperation(String),
    /// Validation error
    ValidationError(String),
}

impl Clone for TrackingError {
    fn clone(&self) -> Self {
        match self {
            TrackingError::AllocationFailed(s) => TrackingError::AllocationFailed(s.clone()),
            TrackingError::DeallocationFailed(s) => TrackingError::DeallocationFailed(s.clone()),
            TrackingError::TrackingDisabled => TrackingError::TrackingDisabled,
            TrackingError::InvalidPointer(s) => TrackingError::InvalidPointer(s.clone()),
            TrackingError::SerializationError(s) => TrackingError::SerializationError(s.clone()),
            TrackingError::VisualizationError(s) => TrackingError::VisualizationError(s.clone()),
            TrackingError::ThreadSafetyError(s) => TrackingError::ThreadSafetyError(s.clone()),
            TrackingError::ConfigurationError(s) => TrackingError::ConfigurationError(s.clone()),
            TrackingError::AnalysisError(s) => TrackingError::AnalysisError(s.clone()),
            TrackingError::ExportError(s) => TrackingError::ExportError(s.clone()),
            TrackingError::MemoryCorruption(s) => TrackingError::MemoryCorruption(s.clone()),
            TrackingError::UnsafeOperationDetected(s) => {
                TrackingError::UnsafeOperationDetected(s.clone())
            }
            TrackingError::FFIError(s) => TrackingError::FFIError(s.clone()),
            TrackingError::ScopeError(s) => TrackingError::ScopeError(s.clone()),
            TrackingError::BorrowCheckError(s) => TrackingError::BorrowCheckError(s.clone()),
            TrackingError::LifetimeError(s) => TrackingError::LifetimeError(s.clone()),
            TrackingError::TypeInferenceError(s) => TrackingError::TypeInferenceError(s.clone()),
            TrackingError::PerformanceError(s) => TrackingError::PerformanceError(s.clone()),
            TrackingError::ResourceExhausted(s) => TrackingError::ResourceExhausted(s.clone()),
            TrackingError::InternalError(s) => TrackingError::InternalError(s.clone()),
            TrackingError::IoError(s) => TrackingError::IoError(s.clone()),
            TrackingError::LockError(s) => TrackingError::LockError(s.clone()),
            TrackingError::ChannelError(s) => TrackingError::ChannelError(s.clone()),
            TrackingError::ThreadError(s) => TrackingError::ThreadError(s.clone()),
            TrackingError::InitializationError(s) => TrackingError::InitializationError(s.clone()),
            TrackingError::NotImplemented(s) => TrackingError::NotImplemented(s.clone()),
            TrackingError::ValidationError(s) => TrackingError::ValidationError(s.clone()),
            TrackingError::InvalidOperation(s) => TrackingError::InvalidOperation(s.clone()),
        }
    }
}

impl std::fmt::Display for TrackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackingError::AllocationFailed(msg) => write!(f, "Allocation failed: {msg}"),
            TrackingError::DeallocationFailed(msg) => write!(f, "Deallocation failed: {msg}"),
            TrackingError::TrackingDisabled => write!(f, "Memory tracking is disabled"),
            TrackingError::InvalidPointer(msg) => write!(f, "Invalid pointer: {msg}"),
            TrackingError::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            TrackingError::VisualizationError(msg) => write!(f, "Visualization error: {msg}"),
            TrackingError::ThreadSafetyError(msg) => write!(f, "Thread safety error: {msg}"),
            TrackingError::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            TrackingError::AnalysisError(msg) => write!(f, "Analysis error: {msg}"),
            TrackingError::ExportError(msg) => write!(f, "Export error: {msg}"),
            TrackingError::MemoryCorruption(msg) => write!(f, "Memory corruption detected: {msg}"),
            TrackingError::UnsafeOperationDetected(msg) => {
                write!(f, "Unsafe operation detected: {msg}")
            }
            TrackingError::FFIError(msg) => write!(f, "FFI error: {msg}"),
            TrackingError::ScopeError(msg) => write!(f, "Scope error: {msg}"),
            TrackingError::BorrowCheckError(msg) => write!(f, "Borrow check error: {msg}"),
            TrackingError::LifetimeError(msg) => write!(f, "Lifetime error: {msg}"),
            TrackingError::TypeInferenceError(msg) => write!(f, "Type inference error: {msg}"),
            TrackingError::PerformanceError(msg) => write!(f, "Performance error: {msg}"),
            TrackingError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {msg}"),
            TrackingError::InternalError(msg) => write!(f, "Internal error: {msg}"),
            TrackingError::IoError(msg) => write!(f, "IO error: {msg}"),
            TrackingError::LockError(msg) => write!(f, "Lock error: {msg}"),
            TrackingError::ChannelError(msg) => write!(f, "Channel error: {msg}"),
            TrackingError::ThreadError(msg) => write!(f, "Thread error: {msg}"),
            TrackingError::InitializationError(msg) => write!(f, "Initialization error: {msg}"),
            TrackingError::NotImplemented(msg) => write!(f, "Not implemented: {msg}"),
            TrackingError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            TrackingError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
        }
    }
}

impl std::error::Error for TrackingError {}

impl From<std::io::Error> for TrackingError {
    fn from(error: std::io::Error) -> Self {
        TrackingError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for TrackingError {
    fn from(error: serde_json::Error) -> Self {
        TrackingError::SerializationError(format!("JSON error: {error}"))
    }
}

/// Smart pointer specific information for Rc/Arc tracking
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SmartPointerInfo {
    /// Data pointer - points to the actual data being shared
    pub data_ptr: usize,

    /// Clone relationship tracking
    pub cloned_from: Option<usize>,
    /// Clones of this smart pointer
    pub clones: Vec<usize>,

    /// Reference count history (timestamp, count)
    pub ref_count_history: Vec<RefCountSnapshot>,

    /// Weak reference information
    pub weak_count: Option<usize>,
    /// Is this a weak reference?
    pub is_weak_reference: bool,

    /// Lifecycle information
    /// Is this the last strong reference?
    pub is_data_owner: bool,
    /// Was data deallocated when this was dropped?
    pub is_implicitly_deallocated: bool,

    /// Smart pointer type
    pub pointer_type: SmartPointerType,
}

/// Reference count snapshot at a specific time
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RefCountSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: u64,
    /// Strong reference count
    pub strong_count: usize,
    /// Weak reference count
    pub weak_count: usize,
}

/// Type of smart pointer
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SmartPointerType {
    /// Rc smart pointer
    Rc,
    /// Arc smart pointer
    Arc,
    /// RcWeak smart pointer
    RcWeak,
    /// ArcWeak smart pointer
    ArcWeak,
    /// Box smart pointer
    Box,
}

impl SmartPointerInfo {
    /// Create new smart pointer info for Rc/Arc
    pub fn new_rc_arc(
        data_ptr: usize,
        pointer_type: SmartPointerType,
        strong_count: usize,
        weak_count: usize,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            data_ptr,
            cloned_from: None,
            clones: Vec::new(),
            ref_count_history: vec![RefCountSnapshot {
                timestamp,
                strong_count,
                weak_count,
            }],
            weak_count: Some(weak_count),
            is_weak_reference: false,
            is_data_owner: strong_count == 1,
            is_implicitly_deallocated: false,
            pointer_type,
        }
    }

    /// Create new smart pointer info for Weak references
    pub fn new_weak(data_ptr: usize, pointer_type: SmartPointerType, weak_count: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            data_ptr,
            cloned_from: None,
            clones: Vec::new(),
            ref_count_history: vec![RefCountSnapshot {
                timestamp,
                strong_count: 0, // Weak references don't contribute to strong count
                weak_count,
            }],
            weak_count: Some(weak_count),
            is_weak_reference: true,
            is_data_owner: false,
            is_implicitly_deallocated: false,
            pointer_type,
        }
    }

    /// Record a clone relationship
    pub fn record_clone(&mut self, clone_ptr: usize, source_ptr: usize) {
        if self.cloned_from.is_none() {
            self.cloned_from = Some(source_ptr);
        }
        self.clones.push(clone_ptr);
    }

    /// Update reference count
    pub fn update_ref_count(&mut self, strong_count: usize, weak_count: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.ref_count_history.push(RefCountSnapshot {
            timestamp,
            strong_count,
            weak_count,
        });

        self.weak_count = Some(weak_count);
        self.is_data_owner = strong_count == 1 && !self.is_weak_reference;
    }

    /// Mark as implicitly deallocated (data was freed when this pointer was dropped)
    pub fn mark_implicitly_deallocated(&mut self) {
        self.is_implicitly_deallocated = true;
    }

    /// Get the latest reference counts
    pub fn latest_ref_counts(&self) -> Option<&RefCountSnapshot> {
        self.ref_count_history.last()
    }
}

/// Information about a memory allocation
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Optional variable name associated with this allocation
    pub var_name: Option<String>,
    /// Optional type name of the allocated data
    pub type_name: Option<String>,
    /// Optional scope name where the allocation occurred
    pub scope_name: Option<String>,
    /// Timestamp when the allocation was made
    pub timestamp_alloc: u64,
    /// Optional timestamp when the allocation was deallocated
    pub timestamp_dealloc: Option<u64>,
    /// Thread ID where the allocation occurred
    #[serde(skip)]
    /// Thread Id
    pub thread_id: String,
    /// Number of active borrows for this allocation
    pub borrow_count: usize,
    /// Optional stack trace at the time of allocation
    pub stack_trace: Option<Vec<String>>,
    /// Whether this allocation is considered leaked
    pub is_leaked: bool,
    /// Precise lifetime in milliseconds (calculated from creation to destruction)
    pub lifetime_ms: Option<u64>,
    /// Smart pointer specific information
    pub smart_pointer_info: Option<SmartPointerInfo>,
    /// Detailed memory layout information
    pub memory_layout: Option<MemoryLayoutInfo>,
    /// Generic type information
    pub generic_info: Option<GenericTypeInfo>,
    /// Dynamic type information (trait objects)
    pub dynamic_type_info: Option<DynamicTypeInfo>,
    /// Runtime state information
    pub runtime_state: Option<RuntimeStateInfo>,
    /// Stack allocation information (if allocated on stack)
    pub stack_allocation: Option<StackAllocationInfo>,
    /// Temporary object information (if this is a temporary)
    pub temporary_object: Option<TemporaryObjectInfo>,
    /// Memory fragmentation analysis
    pub fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
    /// Enhanced generic instantiation tracking
    pub generic_instantiation: Option<GenericInstantiationInfo>,
    /// Type relationship information
    pub type_relationships: Option<TypeRelationshipInfo>,
    /// Type usage information
    pub type_usage: Option<TypeUsageInfo>,
    /// Function call tracking (if allocation is function-related)
    pub function_call_tracking: Option<FunctionCallTrackingInfo>,
    /// Object lifecycle tracking
    pub lifecycle_tracking: Option<ObjectLifecycleInfo>,
    /// Memory access pattern tracking
    pub access_tracking: Option<MemoryAccessTrackingInfo>,
}

impl<'de> serde::Deserialize<'de> for AllocationInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AllocationInfoHelper {
            ptr: usize,
            size: usize,
            var_name: Option<String>,
            type_name: Option<String>,
            scope_name: Option<String>,
            timestamp_alloc: u64,
            timestamp_dealloc: Option<u64>,
            borrow_count: usize,
            stack_trace: Option<Vec<String>>,
            is_leaked: bool,
            lifetime_ms: Option<u64>,
        }

        let helper = AllocationInfoHelper::deserialize(deserializer)?;
        Ok(AllocationInfo {
            ptr: helper.ptr,
            size: helper.size,
            var_name: helper.var_name,
            type_name: helper.type_name,
            scope_name: helper.scope_name,
            timestamp_alloc: helper.timestamp_alloc,
            timestamp_dealloc: helper.timestamp_dealloc,
            thread_id: format!("{:?}", thread::current().id()), // Default to current thread
            borrow_count: helper.borrow_count,
            stack_trace: helper.stack_trace,
            is_leaked: helper.is_leaked,
            lifetime_ms: helper.lifetime_ms,
            smart_pointer_info: None, // Default for deserialization
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
        })
    }
}

impl AllocationInfo {
    /// Create a new AllocationInfo instance
    pub fn new(ptr: usize, size: usize) -> Self {
        Self {
            ptr,
            size,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            timestamp_dealloc: None,
            thread_id: format!("{:?}", thread::current().id()),
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
        }
    }

    /// Mark this allocation as deallocated with current timestamp
    pub fn mark_deallocated(&mut self) {
        self.timestamp_dealloc = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
    }

    /// Check if this allocation is still active (not deallocated)
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }
}

/// Memory statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    /// Total number of allocations made
    pub total_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Number of currently active allocations
    pub active_allocations: usize,
    /// Total bytes in active allocations
    pub active_memory: usize,
    /// Peak number of concurrent allocations
    pub peak_allocations: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Total number of deallocations performed
    pub total_deallocations: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Number of leaked allocations
    pub leaked_allocations: usize,
    /// Total bytes in leaked allocations
    pub leaked_memory: usize,
    /// Analysis of memory fragmentation
    pub fragmentation_analysis: FragmentationAnalysis,
    /// Lifecycle statistics for scopes
    pub lifecycle_stats: ScopeLifecycleMetrics,
    /// List of all allocation information
    pub allocations: Vec<AllocationInfo>,
    /// Statistics for system library allocations
    pub system_library_stats: SystemLibraryStats,
    /// Analysis of concurrent memory operations
    pub concurrency_analysis: ConcurrencyAnalysis,
}

impl MemoryStats {
    /// Create a new empty MemoryStats
    pub fn new() -> Self {
        Self {
            total_allocations: 0,
            total_allocated: 0,
            active_allocations: 0,
            active_memory: 0,
            peak_allocations: 0,
            peak_memory: 0,
            total_deallocations: 0,
            total_deallocated: 0,
            leaked_allocations: 0,
            leaked_memory: 0,
            fragmentation_analysis: FragmentationAnalysis::default(),
            lifecycle_stats: ScopeLifecycleMetrics {
                scope_name: "global".to_string(),
                variable_count: 0,
                average_lifetime_ms: 0.0,
                total_memory_usage: 0,
                peak_memory_usage: 0,
                allocation_frequency: 0.0,
                deallocation_efficiency: 0.0,
                completed_allocations: 0,
                memory_growth_events: 0,
                peak_concurrent_variables: 0,
                memory_efficiency_ratio: 1.0,
                ownership_transfer_events: 0,
                fragmentation_score: 0.0,
                instant_allocations: 0,
                short_term_allocations: 0,
                medium_term_allocations: 0,
                long_term_allocations: 0,
                suspected_leaks: 0,
                risk_distribution: RiskDistribution::default(),
                scope_metrics: Vec::new(),
                type_lifecycle_patterns: Vec::new(),
            },
            allocations: Vec::new(),
            system_library_stats: SystemLibraryStats::default(),
            concurrency_analysis: ConcurrencyAnalysis::default(),
        }
    }
}

/// Memory type analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryTypeInfo {
    /// Name of the memory type
    pub type_name: String,
    /// Total size in bytes for this type
    pub total_size: usize,
    /// Number of allocations of this type
    pub allocation_count: usize,
    /// Average size of allocations for this type
    pub average_size: usize,
    /// Size of the largest allocation for this type
    pub largest_allocation: usize,
    /// Size of the smallest allocation for this type
    pub smallest_allocation: usize,
    /// Number of currently active instances
    pub active_instances: usize,
    /// Number of leaked instances
    pub leaked_instances: usize,
}

/// Type memory usage information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeMemoryUsage {
    /// Name of the type
    pub type_name: String,
    /// Total size allocated for this type
    pub total_size: usize,
    /// Number of allocations for this type
    pub allocation_count: usize,
    /// Average allocation size
    pub average_size: usize,
    /// Current size in memory
    pub current_size: usize,
    /// Memory efficiency score
    pub efficiency_score: f64,
    /// Peak memory usage
    pub peak_size: usize,
}

/// Fragmentation analysis
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FragmentationAnalysis {
    /// Ratio of fragmented to total memory
    pub fragmentation_ratio: f64,
    /// Size of the largest free memory block
    pub largest_free_block: usize,
    /// Size of the smallest free memory block
    pub smallest_free_block: usize,
    /// Total number of free memory blocks
    pub free_block_count: usize,
    /// Total amount of free memory
    pub total_free_memory: usize,
    /// External fragmentation percentage
    pub external_fragmentation: f64,
    /// Internal fragmentation percentage
    pub internal_fragmentation: f64,
}

/// System library usage statistics for tracking external memory usage
///
/// This structure provides insights into memory allocations made by system libraries
/// and external dependencies, helping identify memory usage patterns outside of
/// user code and potential optimization opportunities in library usage.
///
/// # Binary Export Support
/// Fully supports binary serialization and deserialization for efficient export
/// and import operations. The structure is optimized for MessagePack encoding.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemLibraryStats {
    /// Usage statistics for standard collections
    pub std_collections: LibraryUsage,
    /// Usage statistics for async runtime
    pub async_runtime: LibraryUsage,
    /// Usage statistics for network I/O
    pub network_io: LibraryUsage,
    /// Usage statistics for file system operations
    pub file_system: LibraryUsage,
    /// Usage statistics for serialization
    pub serialization: LibraryUsage,
    /// Usage statistics for regex operations
    pub regex_engine: LibraryUsage,
    /// Usage statistics for cryptographic operations
    pub crypto_security: LibraryUsage,
    /// Usage statistics for database operations
    pub database: LibraryUsage,
    /// Usage statistics for graphics and UI
    pub graphics_ui: LibraryUsage,
    /// Usage statistics for HTTP operations
    pub http_stack: LibraryUsage,
}

/// Library usage information
/// Library usage statistics for tracking external dependency memory patterns
///
/// This structure tracks memory allocation patterns for specific libraries
/// or system components, providing insights into external memory usage
/// and potential optimization opportunities.
///
/// # Usage Tracking
/// Monitors allocation frequency, memory consumption, and performance
/// characteristics for library-specific memory operations.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LibraryUsage {
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Categorized usage statistics
    pub categories: HashMap<String, usize>,
    /// Functions with high allocation activity
    pub hotspot_functions: Vec<String>,
}

/// Concurrency analysis for multi-threaded memory operations
///
/// This structure analyzes memory allocation patterns across multiple threads,
/// identifying potential race conditions, contention points, and opportunities
/// for optimization in concurrent memory usage scenarios.
///
/// # Thread Safety
/// All fields are designed to be safely serialized and deserialized across
/// thread boundaries while maintaining data integrity and consistency.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ConcurrencyAnalysis {
    /// Thread Safety Allocations
    pub thread_safety_allocations: usize,
    /// Shared Memory Bytes
    pub shared_memory_bytes: usize,
    /// Mutex Protected
    pub mutex_protected: usize,
    /// Arc Shared
    pub arc_shared: usize,
    /// Rc Shared
    pub rc_shared: usize,
    /// Channel Buffers
    pub channel_buffers: usize,
    /// Thread Local Storage
    pub thread_local_storage: usize,
    /// Atomic Operations
    pub atomic_operations: usize,
    /// Lock Contention Risk
    pub lock_contention_risk: String,
}

/// Scope analysis
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ScopeAnalysis {
    /// Total Scopes
    pub total_scopes: usize,
    /// Active Scopes
    pub active_scopes: usize,
    /// Max Depth
    pub max_depth: usize,
    /// Average Lifetime
    pub average_lifetime: f64,
    /// Memory Efficiency
    pub memory_efficiency: f64,
    /// Scopes
    pub scopes: Vec<ScopeInfo>,
    /// Scope Hierarchy
    pub scope_hierarchy: ScopeHierarchy,
    /// Cross Scope References
    pub cross_scope_references: Vec<String>,
}

/// Scope lifecycle metrics for tracking variable lifetime patterns
///
/// This structure provides detailed analysis of how variables behave within
/// different scopes, including lifetime patterns, memory efficiency, and
/// optimization opportunities for scope-based memory management.
///
/// # Lifecycle Analysis
/// Tracks comprehensive metrics about variable creation, usage, and destruction
/// patterns within specific scopes, enabling detailed performance analysis.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScopeLifecycleMetrics {
    /// Name of the scope
    pub scope_name: String,
    /// Number of variables in scope
    pub variable_count: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Total memory used by scope
    pub total_memory_usage: usize,
    /// Peak memory usage in scope
    pub peak_memory_usage: usize,
    /// Frequency of allocations
    pub allocation_frequency: f64,
    /// Efficiency of deallocations
    pub deallocation_efficiency: f64,
    /// Number of completed allocations
    pub completed_allocations: usize,
    /// Number of memory growth events
    pub memory_growth_events: usize,
    /// Peak number of concurrent variables
    pub peak_concurrent_variables: usize,
    /// Memory efficiency ratio
    pub memory_efficiency_ratio: f64,
    /// Number of ownership transfers
    pub ownership_transfer_events: usize,
    /// Fragmentation score
    pub fragmentation_score: f64,
    /// Number of instant allocations
    pub instant_allocations: usize,
    /// Number of short-term allocations
    pub short_term_allocations: usize,
    /// Number of medium-term allocations
    pub medium_term_allocations: usize,
    /// Number of long-term allocations
    pub long_term_allocations: usize,
    /// Number of suspected memory leaks
    pub suspected_leaks: usize,
    /// Risk distribution analysis
    pub risk_distribution: RiskDistribution,
    /// Metrics for individual scopes
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Lifecycle patterns for types
    pub type_lifecycle_patterns: Vec<TypeLifecyclePattern>,
}

/// Scope information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScopeInfo {
    /// Name
    pub name: String,
    /// Parent
    pub parent: Option<String>,
    /// Children
    pub children: Vec<String>,
    /// Depth
    pub depth: usize,
    /// Variables
    pub variables: Vec<String>,
    /// Total Memory
    pub total_memory: usize,
    /// Peak Memory
    pub peak_memory: usize,
    /// Number of allocations
    pub allocation_count: usize,
    /// Lifetime Start
    pub lifetime_start: Option<u64>,
    /// Lifetime End
    pub lifetime_end: Option<u64>,
    /// Is Active
    pub is_active: bool,
    /// Start Time
    pub start_time: u64,
    /// End Time
    pub end_time: Option<u64>,
    /// Memory Usage
    pub memory_usage: usize,
    /// Child Scopes
    pub child_scopes: Vec<String>,
    /// Parent Scope
    pub parent_scope: Option<String>,
}

/// Scope hierarchy
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ScopeHierarchy {
    /// Root Scopes
    pub root_scopes: Vec<String>,
    /// Scope Tree
    pub scope_tree: HashMap<String, ScopeInfo>,
    /// Max Depth
    pub max_depth: usize,
    /// Total Scopes
    pub total_scopes: usize,
    /// Relationships
    pub relationships: HashMap<String, Vec<String>>,
    /// Depth Map
    pub depth_map: HashMap<String, usize>,
}

/// Risk distribution analysis for memory allocations
/// Risk distribution analysis for memory safety assessment
///
/// This structure analyzes the distribution of memory-related risks across
/// different categories, helping identify potential safety issues and
/// optimization opportunities in memory management patterns.
///
/// # Risk Categories
/// Tracks various types of memory risks including leaks, use-after-free
/// potential, buffer overflows, and other memory safety concerns.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RiskDistribution {
    /// Low Risk
    pub low_risk: usize,
    /// Medium Risk
    pub medium_risk: usize,
    /// High Risk
    pub high_risk: usize,
    /// Critical Risk
    pub critical_risk: usize,
}

/// Type-specific lifecycle pattern analysis
/// Type lifecycle pattern analysis for memory management optimization
///
/// This structure tracks how specific types behave throughout their lifecycle,
/// including allocation patterns, usage frequency, and deallocation timing.
/// This information helps identify optimization opportunities and potential
/// memory management issues for specific data types.
///
/// # Pattern Recognition
/// Analyzes allocation frequency, lifetime distribution, and usage patterns
/// to provide insights into type-specific memory behavior and optimization potential.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeLifecyclePattern {
    /// Type Name
    pub type_name: String,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Typical Size
    pub typical_size: usize,
    /// Growth Pattern
    pub growth_pattern: String,
    /// Risk Level
    pub risk_level: String,
    /// Instance Count
    pub instance_count: usize,
}

/// Growth reason for tracking allocation growth
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum GrowthReason {
    /// Initial allocation
    Initial,
    /// Memory expansion
    Expansion,
    /// Memory reallocation
    Reallocation,
    /// Performance optimization
    Optimization,
    /// User-requested allocation
    UserRequested,
}

/// Type of allocation event
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum AllocationEventType {
    /// Memory allocation event
    Allocate,
    /// Memory deallocation event
    Deallocate,
    /// Memory reallocation event
    Reallocate,
    /// Memory move event
    Move,
    /// Memory borrow event
    Borrow,
    /// Memory return event
    Return,
}

/// Type of scope event
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ScopeEventType {
    /// Enter scope event
    Enter,
    /// Exit scope event
    Exit,
    /// Create scope event
    Create,
    /// Destroy scope event
    Destroy,
}

/// Growth event tracking allocation growth patterns
#[derive(Debug, Clone, serde::Serialize)]
pub struct GrowthEvent {
    /// Timestamp
    pub timestamp: u64,
    /// Old Size
    pub old_size: usize,
    /// New Size
    pub new_size: usize,
    /// Growth Factor
    pub growth_factor: f64,
    /// Reason
    pub reason: GrowthReason,
    /// Var Name
    pub var_name: String,
}

/// Borrow event for tracking borrowing patterns
#[derive(Debug, Clone, serde::Serialize)]
pub struct BorrowEvent {
    /// Timestamp
    pub timestamp: u64,
    /// Memory pointer address
    pub ptr: usize,
    /// Borrow Type
    pub borrow_type: String,
    /// Duration Ms
    pub duration_ms: u64,
    /// Var Name
    pub var_name: String,
}

/// Move event for tracking ownership transfers
#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveEvent {
    /// Timestamp
    pub timestamp: u64,
    /// From Ptr
    pub from_ptr: usize,
    /// To Ptr
    pub to_ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Var Name
    pub var_name: String,
}

/// Variable relationship tracking
#[derive(Debug, Clone, serde::Serialize)]
pub struct VariableRelationship {
    /// Source Var
    pub source_var: String,
    /// Target Var
    pub target_var: String,
    /// Relationship Type
    pub relationship_type: String,
    /// Strength
    pub strength: f64,
}

/// Potential memory leak detection
#[derive(Debug, Clone, serde::Serialize)]
pub struct PotentialLeak {
    /// Memory pointer address
    pub ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Age in milliseconds
    pub age_ms: u64,
    /// Var Name
    pub var_name: Option<String>,
    /// Type Name
    pub type_name: Option<String>,
    /// Severity
    pub severity: String,
}

/// Timeline data for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimelineData {
    /// Time Range
    pub time_range: TimeRange,
    /// Allocation Events
    pub allocation_events: Vec<AllocationEvent>,
    /// Scope Events
    pub scope_events: Vec<ScopeEvent>,
    /// Memory Snapshots
    pub memory_snapshots: Vec<MemorySnapshot>,
}

/// Time range for timeline visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimeRange {
    /// Start Time
    pub start_time: u64,
    /// End Time
    pub end_time: u64,
    /// Duration Ms
    pub duration_ms: u64,
}

/// Memory snapshot at a point in time
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemorySnapshot {
    /// Timestamp
    pub timestamp: u64,
    /// Total Memory
    pub total_memory: usize,
    /// Active Allocations
    pub active_allocations: usize,
    /// Ratio of fragmented to total memory
    pub fragmentation_ratio: f64,
    /// Top Types
    pub top_types: Vec<TypeMemoryUsage>,
}

/// Allocation event for timeline
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationEvent {
    /// Timestamp
    pub timestamp: u64,
    /// Event Type
    pub event_type: AllocationEventType,
    /// Memory pointer address
    pub ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Var Name
    pub var_name: Option<String>,
    /// Type Name
    pub type_name: Option<String>,
}

/// Scope event for timeline
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScopeEvent {
    /// Timestamp
    pub timestamp: u64,
    /// Event Type
    pub event_type: ScopeEventType,
    /// Name of the scope
    pub scope_name: String,
    /// Memory Usage
    pub memory_usage: usize,
    /// Number of variables in scope
    pub variable_count: usize,
}

/// Stack trace data for analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct StackTraceData {
    /// Memory allocation hotspots
    pub hotspots: Vec<StackTraceHotspot>,
    /// Detected allocation patterns
    pub allocation_patterns: Vec<AllocationPattern>,
    /// Total number of samples
    pub total_samples: usize,
}

/// Stack trace hotspot
#[derive(Debug, Clone, serde::Serialize)]
pub struct StackTraceHotspot {
    /// Name of the function
    pub function_name: String,
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Percentage of total allocations
    pub percentage: f64,
}

/// Allocation pattern analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationPattern {
    /// Type of allocation pattern
    pub pattern_type: String,
    /// Frequency of occurrence
    pub frequency: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Description of the item
    pub description: String,
}

/// Stack frame for stack traces
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StackFrame {
    /// Name of the function
    pub function_name: String,
    /// Source file name
    pub file_name: Option<String>,
    /// Line number in source code
    pub line_number: Option<u32>,
    /// Module path
    pub module_path: Option<String>,
}

/// Safety violation types
#[derive(Debug, Clone, serde::Serialize)]
pub enum SafetyViolation {
    /// Potential memory leak detected
    PotentialLeak {
        /// Memory pointer address
        ptr: usize,
        /// Size in bytes
        size: usize,
        /// Age in milliseconds
        age_ms: u64,
        /// Description of the item
        description: String,
    },
    /// Use after free violation detected
    UseAfterFree {
        /// Memory pointer address
        ptr: usize,
        /// Description of the item
        description: String,
    },
    /// Double free violation detected
    DoubleFree {
        /// Memory pointer address
        ptr: usize,
        /// Description of the item
        description: String,
    },
    /// Buffer overflow detected
    BufferOverflow {
        /// Memory pointer address
        ptr: usize,
        /// Size in bytes
        size: usize,
        /// Description of the item
        description: String,
    },
}

/// Allocation hotspot information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationHotspot {
    /// Location information
    pub location: HotspotLocation,
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Frequency of occurrence
    pub frequency: f64,
}

/// Hotspot location information
#[derive(Debug, Clone, serde::Serialize)]
pub struct HotspotLocation {
    /// Name of the function
    pub function_name: String,
    /// Path to source file
    pub file_path: Option<String>,
    /// Line number in source code
    pub line_number: Option<u32>,
    /// Module path
    pub module_path: Option<String>,
}

/// Detailed memory layout analysis information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLayoutInfo {
    /// Total size of the type in bytes
    pub total_size: usize,
    /// Alignment requirement of the type
    pub alignment: usize,
    /// Field layout information
    pub field_layout: Vec<FieldLayoutInfo>,
    /// Padding byte information
    pub padding_info: PaddingAnalysis,
    /// Memory layout efficiency analysis
    pub layout_efficiency: LayoutEfficiency,
}

/// Field layout information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldLayoutInfo {
    /// Field name
    pub field_name: String,
    /// Field type
    pub field_type: String,
    /// Field offset within the struct
    pub offset: usize,
    /// Field size
    pub size: usize,
    /// Field alignment requirement
    pub alignment: usize,
    /// Whether this is a padding field
    pub is_padding: bool,
}

/// Padding byte analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaddingAnalysis {
    /// Total number of padding bytes
    pub total_padding_bytes: usize,
    /// Padding byte locations
    pub padding_locations: Vec<PaddingLocation>,
    /// Padding ratio (padding bytes / total size)
    pub padding_ratio: f64,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<String>,
}

/// Padding byte location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaddingLocation {
    /// Padding start offset
    pub start_offset: usize,
    /// Padding size
    pub size: usize,
    /// Padding reason
    pub reason: PaddingReason,
}

/// Padding reason
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaddingReason {
    /// Field alignment
    FieldAlignment,
    /// Struct tail alignment
    StructAlignment,
    /// Enum discriminant alignment
    EnumDiscriminant,
    /// Other reason
    Other(String),
}

/// Layout efficiency analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutEfficiency {
    /// Memory utilization (useful data / total size)
    pub memory_utilization: f64,
    /// Cache friendliness score (0-100)
    pub cache_friendliness: f64,
    /// Alignment waste in bytes
    pub alignment_waste: usize,
    /// Optimization potential assessment
    pub optimization_potential: OptimizationPotential,
}

/// Optimization potential assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationPotential {
    /// No optimization needed
    None,
    /// Minor optimization
    /// Minor optimization potential
    Minor {
        /// Potential memory savings in bytes
        potential_savings: usize,
    },
    /// Moderate optimization
    Moderate {
        /// Potential savings in bytes
        potential_savings: usize,
        /// Optimization suggestions
        suggestions: Vec<String>,
    },
    /// Major optimization
    Major {
        /// Potential savings in bytes
        potential_savings: usize,
        /// Optimization suggestions
        suggestions: Vec<String>,
    },
}

/// Generic type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericTypeInfo {
    /// Generic base type name
    pub base_type: String,
    /// Generic type parameters
    pub type_parameters: Vec<TypeParameter>,
    /// Monomorphization information
    pub monomorphization_info: MonomorphizationInfo,
    /// Generic constraint information
    pub constraints: Vec<GenericConstraint>,
}

/// Generic type parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParameter {
    /// Parameter name
    pub name: String,
    /// Concrete type
    pub concrete_type: String,
    /// Type size
    pub size: usize,
    /// Type alignment
    pub alignment: usize,
    /// Whether this is a lifetime parameter
    pub is_lifetime: bool,
}

/// Monomorphization information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonomorphizationInfo {
    /// Number of monomorphization instances
    pub instance_count: usize,
    /// Memory usage per instance
    pub per_instance_memory: usize,
    /// Total memory usage
    pub total_memory_usage: usize,
    /// Code bloat assessment
    pub code_bloat_assessment: CodeBloatLevel,
}

/// Code bloat level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CodeBloatLevel {
    /// Low code bloat
    Low,
    /// Moderate code bloat
    Moderate,
    /// High code bloat
    High,
    /// Excessive code bloat
    Excessive,
}

/// Generic constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Constraint description
    pub description: String,
    /// Impact on memory layout
    pub memory_impact: MemoryImpact,
}

/// Constraint type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Trait constraint
    Trait(String),
    /// Lifetime constraint
    Lifetime(String),
    /// Associated type constraint
    Associated(String),
    /// Where clause constraint
    Where(String),
}

/// Memory impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    /// No memory impact
    None,
    /// Size increase
    SizeIncrease(usize),
    /// Alignment change
    AlignmentChange(usize),
    /// Layout change
    LayoutChange(String),
}

/// Dynamic type information (trait objects)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicTypeInfo {
    /// Trait name
    pub trait_name: String,
    /// Virtual function table information
    pub vtable_info: VTableInfo,
    /// Concrete object type (if determinable)
    pub concrete_type: Option<String>,
    /// Dynamic dispatch overhead
    pub dispatch_overhead: DispatchOverhead,
    /// Type erasure information
    pub type_erasure_info: TypeErasureInfo,
}

/// Virtual function table information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VTableInfo {
    /// VTable size
    pub vtable_size: usize,
    /// Number of methods
    pub method_count: usize,
    /// VTable pointer offset
    pub vtable_ptr_offset: usize,
    /// Method list
    pub methods: Vec<VTableMethod>,
}

/// Virtual function table method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VTableMethod {
    /// Method name
    pub name: String,
    /// Method signature
    pub signature: String,
    /// Offset in vtable
    pub vtable_offset: usize,
}

/// Dynamic dispatch overhead
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchOverhead {
    /// Indirect call overhead in nanoseconds
    pub indirect_call_overhead_ns: f64,
    /// Cache miss probability
    pub cache_miss_probability: f64,
    /// Branch misprediction rate
    pub branch_misprediction_rate: f64,
    /// Overall performance impact assessment
    pub performance_impact: PerformanceImpact,
}

/// Performance impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceImpact {
    /// Negligible performance impact
    Negligible,
    /// Minor performance impact
    Minor,
    /// Moderate performance impact
    Moderate,
    /// Significant performance impact
    Significant,
    /// Severe performance impact
    Severe,
}

/// Type erasure information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeErasureInfo {
    /// Whether original type information is recoverable
    pub type_info_recoverable: bool,
    /// Type size information
    pub size_info: Option<usize>,
    /// Alignment information
    pub alignment_info: Option<usize>,
    /// Destructor information
    pub destructor_info: Option<String>,
}

/// Runtime state information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStateInfo {
    /// CPU usage information
    pub cpu_usage: CpuUsageInfo,
    /// Memory pressure
    pub memory_pressure: MemoryPressureInfo,
    /// Cache performance
    pub cache_performance: CachePerformanceInfo,
    /// Allocator state
    pub allocator_state: AllocatorStateInfo,
    /// Garbage collection information (if applicable)
    pub gc_info: Option<GcInfo>,
}

/// CPU usage information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuUsageInfo {
    /// Current CPU usage percentage
    pub current_usage_percent: f64,
    /// Average CPU usage percentage
    pub average_usage_percent: f64,
    /// Peak CPU usage percentage
    pub peak_usage_percent: f64,
    /// CPU intensive operations count
    pub intensive_operations_count: usize,
}

/// Memory pressure information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPressureInfo {
    /// Current memory pressure level
    pub pressure_level: MemoryPressureLevel,
    /// Available memory percentage
    pub available_memory_percent: f64,
    /// Memory allocation failures count
    pub allocation_failures: usize,
    /// Memory fragmentation level
    pub fragmentation_level: f64,
}

/// Memory pressure level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    /// Low memory pressure
    Low,
    /// Moderate memory pressure
    Moderate,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

/// Cache performance information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachePerformanceInfo {
    /// L1 cache hit rate
    pub l1_hit_rate: f64,
    /// L2 cache hit rate
    pub l2_hit_rate: f64,
    /// L3 cache hit rate
    pub l3_hit_rate: f64,
    /// Cache miss penalty in nanoseconds
    pub cache_miss_penalty_ns: f64,
    /// Memory access pattern analysis
    pub access_pattern: MemoryAccessPattern,
}

/// Memory access pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAccessPattern {
    /// Sequential access pattern
    Sequential,
    /// Random access pattern
    Random,
    /// Strided access pattern
    Strided {
        /// Stride size
        stride: usize,
    },
    /// Clustered access pattern
    Clustered,
    /// Mixed access pattern
    Mixed,
}

/// Allocator state information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocatorStateInfo {
    /// Allocator type
    pub allocator_type: String,
    /// Heap size
    pub heap_size: usize,
    /// Used heap space
    pub heap_used: usize,
    /// Free blocks count
    pub free_blocks_count: usize,
    /// Largest free block size
    pub largest_free_block: usize,
    /// Allocator efficiency score
    pub efficiency_score: f64,
}

/// Garbage collection information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GcInfo {
    /// GC type
    pub gc_type: String,
    /// GC runs count
    pub gc_runs: usize,
    /// Total GC time in milliseconds
    pub total_gc_time_ms: u64,
    /// Average GC pause time
    pub average_pause_time_ms: f64,
    /// Memory reclaimed
    pub memory_reclaimed: usize,
}

/// Stack allocation tracking information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackAllocationInfo {
    /// Stack frame identifier
    pub frame_id: usize,
    /// Variable name on stack
    pub var_name: String,
    /// Stack offset from frame pointer
    pub stack_offset: isize,
    /// Size of stack allocation
    pub size: usize,
    /// Function name where allocated
    pub function_name: String,
    /// Stack depth level
    pub stack_depth: usize,
    /// Lifetime scope information
    pub scope_info: StackScopeInfo,
}

/// Stack scope information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackScopeInfo {
    /// Scope type (function, block, loop, etc.)
    pub scope_type: ScopeType,
    /// Scope start line number
    pub start_line: Option<u32>,
    /// Scope end line number
    pub end_line: Option<u32>,
    /// Parent scope identifier
    pub parent_scope: Option<usize>,
    /// Nested scope level
    pub nesting_level: usize,
}

/// Scope type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    /// Function scope
    Function,
    /// Block scope
    Block,
    /// Loop scope
    Loop,
    /// Conditional scope
    Conditional,
    /// Match scope
    Match,
    /// Closure scope
    Async,
    /// Unsafe scope
    Unsafe,
}

/// Temporary object tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporaryObjectInfo {
    /// Temporary object identifier
    pub temp_id: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Destruction timestamp
    pub destroyed_at: Option<u64>,
    /// Lifetime in nanoseconds
    pub lifetime_ns: Option<u64>,
    /// Creation context
    pub creation_context: CreationContext,
    /// Usage pattern
    pub usage_pattern: TemporaryUsagePattern,
    /// Memory location type
    pub location_type: MemoryLocationType,
}

/// Creation context for temporary objects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreationContext {
    /// Function where created
    pub function_name: String,
    /// Expression type that created the temporary
    pub expression_type: ExpressionType,
    /// Source location
    pub source_location: Option<SourceLocation>,
    /// Call stack at creation
    pub call_stack: Vec<String>,
}

/// Expression type that creates temporaries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionType {
    /// Function call expression
    FunctionCall,
    /// Method call expression
    MethodCall,
    /// Operator overload expression
    OperatorOverload,
    /// Conversion expression
    Conversion,
    /// Literal expression
    Literal,
    /// Aggregate expression
    Conditional,
    /// Match expression
    Match,
}

/// Source location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

/// Temporary usage pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemporaryUsagePattern {
    /// Used immediately and discarded
    Immediate,
    /// Passed to function
    FunctionArgument,
    /// Used in expression chain
    ExpressionChain,
    /// Stored temporarily
    TemporaryStorage,
    /// Moved to permanent location
    MovedToPermanent,
}

/// Memory location type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryLocationType {
    /// Stack memory location
    Stack,
    /// Heap memory location
    Heap,
    /// Register memory location
    Register,
    /// Static memory location
    Static,
    /// Thread-local memory location
    ThreadLocal,
}

/// Enhanced memory fragmentation analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnhancedFragmentationAnalysis {
    /// Total heap size
    pub total_heap_size: usize,
    /// Used heap size
    pub used_heap_size: usize,
    /// Free heap size
    pub free_heap_size: usize,
    /// Number of free blocks
    pub free_block_count: usize,
    /// Free block size distribution
    pub free_block_distribution: Vec<BlockSizeRange>,
    /// Fragmentation metrics
    pub fragmentation_metrics: FragmentationMetrics,
    /// Allocation patterns causing fragmentation
    pub fragmentation_causes: Vec<FragmentationCause>,
}

/// Block size range for distribution analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSizeRange {
    /// Minimum size in range
    pub min_size: usize,
    /// Maximum size in range
    pub max_size: usize,
    /// Number of blocks in this range
    pub block_count: usize,
    /// Total size of blocks in this range
    pub total_size: usize,
}

/// Fragmentation metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentationMetrics {
    /// External fragmentation ratio
    pub external_fragmentation: f64,
    /// Internal fragmentation ratio
    pub internal_fragmentation: f64,
    /// Largest free block size
    pub largest_free_block: usize,
    /// Average free block size
    pub average_free_block_size: f64,
    /// Fragmentation severity level
    pub severity_level: FragmentationSeverity,
}

/// Fragmentation severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentationSeverity {
    /// Low fragmentation severity
    Low,
    /// Moderate fragmentation severity
    Moderate,
    /// High fragmentation severity
    High,
    /// Critical fragmentation severity
    Critical,
}

/// Fragmentation cause analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentationCause {
    /// Cause type
    pub cause_type: FragmentationCauseType,
    /// Description of the cause
    pub description: String,
    /// Impact on fragmentation
    pub impact_level: ImpactLevel,
    /// Suggested mitigation
    pub mitigation_suggestion: String,
}

/// Types of fragmentation causes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FragmentationCauseType {
    /// Mixed allocation sizes
    MixedAllocationSizes,
    /// Frequent allocation/deallocation
    FrequentAllocDealloc,
    /// Long-lived allocations blocking coalescing
    LongLivedAllocations,
    /// Poor allocation strategy
    PoorAllocationStrategy,
    /// Memory leaks
    MemoryLeaks,
}

/// Impact level enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Low impact level
    Low,
    /// Medium impact level
    Medium,
    /// High impact level
    High,
    /// Critical impact level
    Critical,
}

/// Enhanced generic instantiation tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericInstantiationInfo {
    /// Base generic type name
    pub base_type: String,
    /// Concrete type parameters
    pub concrete_parameters: Vec<ConcreteTypeParameter>,
    /// Instantiation location
    pub instantiation_location: SourceLocation,
    /// Instantiation frequency
    pub instantiation_count: usize,
    /// Memory usage per instantiation
    pub memory_per_instance: usize,
    /// Total memory usage across all instances
    pub total_memory_usage: usize,
    /// Compilation time impact
    pub compilation_impact: CompilationImpact,
    /// Runtime performance characteristics
    pub performance_characteristics: PerformanceCharacteristics,
}

/// Concrete type parameter with detailed information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcreteTypeParameter {
    /// Parameter name
    pub name: String,
    /// Concrete type
    pub concrete_type: String,
    /// Type complexity score
    pub complexity_score: u32,
    /// Memory footprint
    pub memory_footprint: usize,
    /// Alignment requirements
    pub alignment: usize,
    /// Whether type implements common traits
    pub trait_implementations: Vec<String>,
    /// Type category
    pub type_category: TypeCategory,
}

/// Type category classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeCategory {
    /// Primitive type
    Primitive,
    /// Struct type
    Struct,
    /// Enum type
    Enum,
    /// Union type
    Union,
    /// Tuple type
    Tuple,
    /// Array type
    Array,
    /// Slice type
    Slice,
    /// Reference type
    Reference,
    /// Pointer type
    Pointer,
    /// Function type
    Function,
    /// Closure type
    TraitObject,
    /// Generic type
    Generic,
    /// Associated type
    Associated,
}

/// Compilation impact assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationImpact {
    /// Estimated compilation time increase (milliseconds)
    pub compilation_time_ms: u64,
    /// Code size increase (bytes)
    pub code_size_increase: usize,
    /// LLVM IR complexity score
    pub ir_complexity_score: u32,
    /// Optimization difficulty level
    pub optimization_difficulty: OptimizationDifficulty,
}

/// Optimization difficulty levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationDifficulty {
    /// Easy optimization difficulty
    Easy,
    /// Moderate optimization difficulty
    Moderate,
    /// Hard optimization difficulty
    Hard,
    /// Very hard optimization difficulty
    VeryHard,
}

/// Performance characteristics of instantiated types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceCharacteristics {
    /// Average allocation time (nanoseconds)
    pub avg_allocation_time_ns: f64,
    /// Average deallocation time (nanoseconds)
    pub avg_deallocation_time_ns: f64,
    /// Memory access pattern
    pub access_pattern: MemoryAccessPattern,
    /// Cache performance impact
    pub cache_impact: CacheImpact,
    /// Branch prediction impact
    pub branch_prediction_impact: BranchPredictionImpact,
}

/// Cache impact assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheImpact {
    /// L1 cache impact score
    pub l1_impact_score: f64,
    /// L2 cache impact score
    pub l2_impact_score: f64,
    /// L3 cache impact score
    pub l3_impact_score: f64,
    /// Cache line utilization efficiency
    pub cache_line_efficiency: f64,
}

/// Branch prediction impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchPredictionImpact {
    /// Branch misprediction rate
    pub misprediction_rate: f64,
    /// Impact on pipeline stalls
    pub pipeline_stall_impact: f64,
    /// Predictability score
    pub predictability_score: f64,
}

/// Type inheritance and composition relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeRelationshipInfo {
    /// Type name
    pub type_name: String,
    /// Parent types (traits, base structs)
    pub parent_types: Vec<ParentTypeInfo>,
    /// Child types (implementors, derived types)
    pub child_types: Vec<ChildTypeInfo>,
    /// Composed types (fields, associated types)
    pub composed_types: Vec<ComposedTypeInfo>,
    /// Relationship complexity score
    pub complexity_score: u32,
    /// Inheritance depth
    pub inheritance_depth: u32,
    /// Composition breadth
    pub composition_breadth: u32,
}

/// Parent type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentTypeInfo {
    /// Parent type name
    pub type_name: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Inheritance level
    pub inheritance_level: u32,
    /// Memory layout impact
    pub memory_impact: MemoryImpact,
}

/// Child type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChildTypeInfo {
    /// Child type name
    pub type_name: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Specialization level
    pub specialization_level: u32,
    /// Usage frequency
    pub usage_frequency: u32,
}

/// Composed type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComposedTypeInfo {
    /// Composed type name
    pub type_name: String,
    /// Field or association name
    pub field_name: String,
    /// Composition type
    pub composition_type: CompositionType,
    /// Memory offset (if applicable)
    pub memory_offset: Option<usize>,
    /// Access frequency
    pub access_frequency: u32,
}

/// Type relationship types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Trait implementation relationship
    TraitImplementation,
    /// Trait bound relationship
    TraitBound,
    /// Inheritance relationship
    Inheritance,
    /// Association relationship
    Association,
    /// Aggregation relationship
    Composition,
    /// Dependency relationship
    Dependency,
}

/// Composition types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompositionType {
    /// Field composition type
    Field,
    /// Associated type composition type
    AssociatedType,
    /// Generic parameter composition type
    GenericParameter,
    /// Nested type composition type
    NestedType,
    /// Reference composition type
    Reference,
    /// Smart pointer composition type
    SmartPointer,
}

/// Type usage frequency and context tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeUsageInfo {
    /// Type name
    pub type_name: String,
    /// Total usage count
    pub total_usage_count: u64,
    /// Usage contexts
    pub usage_contexts: Vec<UsageContext>,
    /// Usage patterns over time
    pub usage_timeline: Vec<UsageTimePoint>,
    /// Hot paths where type is used
    pub hot_paths: Vec<HotPath>,
    /// Performance impact of usage
    pub performance_impact: TypePerformanceImpact,
}

/// Usage context information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageContext {
    /// Context type
    pub context_type: ContextType,
    /// Function or module name
    pub location: String,
    /// Usage frequency in this context
    pub frequency: u32,
    /// Performance characteristics in this context
    pub performance_metrics: ContextPerformanceMetrics,
}

/// Context types where types are used
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContextType {
    /// Function parameter context
    FunctionParameter,
    /// Function return context
    FunctionReturn,
    /// Local variable context
    LocalVariable,
    /// Struct field context
    StructField,
    /// Enum variant context
    EnumVariant,
    /// Trait method context
    TraitMethod,
    /// Generic constraint context
    GenericConstraint,
    /// Closure capture context
    ClosureCapture,
    /// Async context
    AsyncContext,
    /// Unsafe context
    UnsafeContext,
}

/// Performance metrics within specific contexts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPerformanceMetrics {
    /// Average execution time in this context
    pub avg_execution_time_ns: f64,
    /// Memory allocation frequency
    pub allocation_frequency: f64,
    /// Cache miss rate in this context
    pub cache_miss_rate: f64,
    /// Branch misprediction rate
    pub branch_misprediction_rate: f64,
}

/// Usage time point for timeline analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageTimePoint {
    /// Timestamp
    pub timestamp: u64,
    /// Usage count at this time
    pub usage_count: u32,
    /// Memory usage at this time
    pub memory_usage: usize,
    /// Performance metrics at this time
    pub performance_snapshot: PerformanceSnapshot,
}

/// Performance snapshot at a specific time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Throughput (operations per second)
    pub throughput: f64,
}

/// Hot path information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotPath {
    /// Path identifier
    pub path_id: String,
    /// Function call sequence
    pub call_sequence: Vec<String>,
    /// Execution frequency
    pub execution_frequency: u64,
    /// Total execution time
    pub total_execution_time_ns: u64,
    /// Average execution time
    pub avg_execution_time_ns: f64,
    /// Memory allocations in this path
    pub memory_allocations: u32,
    /// Performance bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Performance bottleneck information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck type
    pub bottleneck_type: BottleneckType,
    /// Location in code
    pub location: String,
    /// Impact severity
    pub severity: ImpactLevel,
    /// Description
    pub description: String,
    /// Suggested optimization
    pub optimization_suggestion: String,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BottleneckType {
    /// Memory allocation bottleneck
    MemoryAllocation,
    /// Memory deallocation bottleneck
    MemoryDeallocation,
    /// Cache miss bottleneck
    CacheMiss,
    /// Branch misprediction bottleneck
    BranchMisprediction,
    /// Function call bottleneck
    FunctionCall,
    /// Data movement bottleneck
    DataMovement,
    /// Synchronization bottleneck
    Synchronization,
    /// I/O bottleneck
    IO,
}

/// Type performance impact assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypePerformanceImpact {
    /// Overall performance score (0-100)
    pub performance_score: f64,
    /// Memory efficiency score (0-100)
    pub memory_efficiency_score: f64,
    /// CPU efficiency score (0-100)
    pub cpu_efficiency_score: f64,
    /// Cache efficiency score (0-100)
    pub cache_efficiency_score: f64,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}

/// Optimization recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority level
    pub priority: Priority,
    /// Description
    pub description: String,
    /// Expected performance improvement
    pub expected_improvement: f64,
    /// Implementation difficulty
    pub implementation_difficulty: ImplementationDifficulty,
}

/// Types of optimization recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Memory layout optimization
    MemoryLayout,
    /// Algorithm change recommendation
    AlgorithmChange,
    /// Data structure change recommendation
    DataStructureChange,
    /// Caching strategy recommendation
    CachingStrategy,
    /// Memory pooling recommendation
    MemoryPooling,
    /// Lazy initialization recommendation
    LazyInitialization,
    /// Inlining recommendation
    Inlining,
    /// Vectorization recommendation
    Vectorization,
    /// Parallelization recommendation
    Parallelization,
}

/// Priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Implementation difficulty levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    /// Easy implementation difficulty
    Easy,
    /// Medium implementation difficulty
    Medium,
    /// Hard implementation difficulty
    Hard,
    /// Very hard implementation difficulty
    VeryHard,
}

/// Function call frequency and call stack tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallTrackingInfo {
    /// Function name
    pub function_name: String,
    /// Module path
    pub module_path: String,
    /// Total call count
    pub total_call_count: u64,
    /// Call frequency per second
    pub call_frequency_per_sec: f64,
    /// Average execution time per call
    pub avg_execution_time_ns: f64,
    /// Total execution time
    pub total_execution_time_ns: u64,
    /// Call stack information
    pub call_stack_info: CallStackInfo,
    /// Memory allocations per call
    pub memory_allocations_per_call: f64,
    /// Performance characteristics
    pub performance_characteristics: FunctionPerformanceCharacteristics,
    /// Call patterns
    pub call_patterns: Vec<CallPattern>,
}

/// Call stack information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallStackInfo {
    /// Maximum call stack depth
    pub max_stack_depth: u32,
    /// Average call stack depth
    pub avg_stack_depth: f64,
    /// Most common call sequences
    pub common_call_sequences: Vec<CallSequence>,
    /// Recursive call detection
    pub recursive_calls: Vec<RecursiveCallInfo>,
    /// Stack overflow risk assessment
    pub stack_overflow_risk: StackOverflowRisk,
}

/// Call sequence information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallSequence {
    /// Sequence of function names
    pub function_sequence: Vec<String>,
    /// Frequency of this sequence
    pub frequency: u32,
    /// Average execution time for this sequence
    pub avg_execution_time_ns: f64,
    /// Memory usage pattern for this sequence
    pub memory_usage_pattern: MemoryUsagePattern,
}

/// Memory usage pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUsagePattern {
    /// Peak memory usage in sequence
    pub peak_memory_usage: usize,
    /// Average memory usage
    pub avg_memory_usage: usize,
    /// Memory allocation frequency
    pub allocation_frequency: f64,
    /// Memory deallocation frequency
    pub deallocation_frequency: f64,
    /// Memory leak potential
    pub leak_potential: LeakPotential,
}

/// Memory leak potential assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LeakPotential {
    /// Low memory leak potential
    Low,
    /// Medium memory leak potential
    Medium,
    /// High memory leak potential
    High,
    /// Critical memory leak potential
    Critical,
}

/// Recursive call information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecursiveCallInfo {
    /// Function name
    pub function_name: String,
    /// Maximum recursion depth
    pub max_recursion_depth: u32,
    /// Average recursion depth
    pub avg_recursion_depth: f64,
    /// Tail recursion optimization potential
    pub tail_recursion_potential: bool,
    /// Stack usage per recursion level
    pub stack_usage_per_level: usize,
    /// Performance impact of recursion
    pub recursion_performance_impact: RecursionPerformanceImpact,
}

/// Recursion performance impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecursionPerformanceImpact {
    /// Stack overhead per call
    pub stack_overhead_per_call: usize,
    /// Function call overhead
    pub call_overhead_ns: f64,
    /// Cache impact of deep recursion
    pub cache_impact: f64,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<String>,
}

/// Stack overflow risk assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StackOverflowRisk {
    /// Low stack overflow risk
    Low,
    /// Medium stack overflow risk
    Medium,
    /// High stack overflow risk
    High,
    /// Critical stack overflow risk
    Critical,
}

/// Function performance characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionPerformanceCharacteristics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage characteristics
    pub memory_characteristics: FunctionMemoryCharacteristics,
    /// I/O characteristics
    pub io_characteristics: IOCharacteristics,
    /// Concurrency characteristics
    pub concurrency_characteristics: ConcurrencyCharacteristics,
    /// Performance bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Function memory characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMemoryCharacteristics {
    /// Stack memory usage
    pub stack_memory_usage: usize,
    /// Heap memory allocations
    pub heap_allocations: u32,
    /// Memory access pattern
    pub access_pattern: MemoryAccessPattern,
    /// Cache efficiency
    pub cache_efficiency: f64,
    /// Memory bandwidth utilization
    pub memory_bandwidth_utilization: f64,
}

/// I/O characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IOCharacteristics {
    /// File I/O operations
    pub file_io_operations: u32,
    /// Network I/O operations
    pub network_io_operations: u32,
    /// Average I/O wait time
    pub avg_io_wait_time_ns: f64,
    /// I/O throughput
    pub io_throughput_bytes_per_sec: f64,
    /// I/O efficiency score
    pub io_efficiency_score: f64,
}

/// Concurrency characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConcurrencyCharacteristics {
    /// Thread safety level
    pub thread_safety_level: ThreadSafetyLevel,
    /// Lock contention frequency
    pub lock_contention_frequency: f64,
    /// Parallel execution potential
    pub parallel_execution_potential: f64,
    /// Synchronization overhead
    pub synchronization_overhead_ns: f64,
    /// Deadlock risk assessment
    pub deadlock_risk: DeadlockRisk,
}

/// Thread safety levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreadSafetyLevel {
    /// Thread safe
    ThreadSafe,
    /// Conditionally thread safe
    ConditionallyThreadSafe,
    /// Not thread safe
    NotThreadSafe,
    /// Unknown thread safety
    Unknown,
}

/// Deadlock risk assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeadlockRisk {
    /// No deadlock risk
    None,
    /// Low deadlock risk
    Low,
    /// Medium deadlock risk
    Medium,
    /// High deadlock risk
    High,
}

/// Call pattern information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallPattern {
    /// Pattern type
    pub pattern_type: CallPatternType,
    /// Pattern description
    pub description: String,
    /// Frequency of this pattern
    pub frequency: u32,
    /// Performance impact
    pub performance_impact: f64,
    /// Optimization potential
    pub optimization_potential: f64,
}

/// Types of call patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallPatternType {
    /// Sequential call pattern
    Sequential,
    /// Recursive call pattern
    Recursive,
    /// Iterative call pattern
    Iterative,
    /// Conditional call pattern
    Conditional,
    /// Parallel call pattern
    Parallel,
    /// Asynchronous call pattern
    Asynchronous,
    /// Callback call pattern
    Callback,
    /// Event-driven call pattern
    EventDriven,
}

/// Object lifecycle event tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectLifecycleInfo {
    /// Object identifier
    pub object_id: usize,
    /// Object type name
    pub type_name: String,
    /// Lifecycle events
    pub lifecycle_events: Vec<LifecycleEvent>,
    /// Total lifetime duration
    pub total_lifetime_ns: Option<u64>,
    /// Lifecycle stage durations
    pub stage_durations: LifecycleStageDurations,
    /// Lifecycle efficiency metrics
    pub efficiency_metrics: LifecycleEfficiencyMetrics,
    /// Lifecycle patterns
    pub lifecycle_patterns: Vec<LifecyclePattern>,
}

/// Lifecycle event information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Event type
    pub event_type: LifecycleEventType,
    /// Timestamp when event occurred
    pub timestamp: u64,
    /// Location where event occurred
    pub location: SourceLocation,
    /// Memory state at event time
    pub memory_state: MemoryState,
    /// Performance metrics at event time
    pub performance_metrics: EventPerformanceMetrics,
    /// Call stack at event time
    pub call_stack: Vec<String>,
}

/// Types of lifecycle events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Object creation
    Creation,
    /// Object initialization
    Initialization,
    /// Object first use
    FirstUse,
    /// Object move
    Move,
    /// Object copy
    Copy,
    /// Object clone
    Clone,
    /// Object borrow
    Borrow,
    /// Object mutable borrow
    MutableBorrow,
    /// Object borrow release
    BorrowRelease,
    /// Object modification
    Modification,
    /// Object last use
    LastUse,
    /// Object drop
    Drop,
    /// Object destruction
    Destruction,
    /// Object memory reclaim
    MemoryReclaim,
}

/// Memory state at event time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryState {
    /// Memory location
    pub memory_location: MemoryLocationType,
    /// Memory address
    pub memory_address: usize,
    /// Object size
    pub object_size: usize,
    /// Reference count (if applicable)
    pub reference_count: Option<u32>,
    /// Borrow state
    pub borrow_state: BorrowState,
}

/// Borrow state information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowState {
    /// Object is not borrowed
    NotBorrowed,
    /// Object is shared borrowed
    SharedBorrow {
        /// Number of shared borrows
        count: u32,
    },
    /// Object is mutably borrowed
    MutableBorrow,
    /// Object has been moved out
    MovedOut,
}

/// Performance metrics at event time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventPerformanceMetrics {
    /// CPU cycles consumed by event
    pub cpu_cycles: u64,
    /// Memory bandwidth used
    pub memory_bandwidth_bytes: usize,
    /// Cache misses caused by event
    pub cache_misses: u32,
    /// Event processing time
    pub processing_time_ns: u64,
}

/// Lifecycle stage durations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleStageDurations {
    /// Time from creation to first use
    pub creation_to_first_use_ns: Option<u64>,
    /// Time spent in active use
    pub active_use_duration_ns: Option<u64>,
    /// Time from last use to destruction
    pub last_use_to_destruction_ns: Option<u64>,
    /// Time spent borrowed
    pub borrowed_duration_ns: u64,
    /// Time spent idle
    pub idle_duration_ns: u64,
}

/// Lifecycle efficiency metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleEfficiencyMetrics {
    /// Utilization ratio (active time / total time)
    pub utilization_ratio: f64,
    /// Memory efficiency (useful operations / memory usage)
    pub memory_efficiency: f64,
    /// Performance efficiency score
    pub performance_efficiency: f64,
    /// Resource waste assessment
    pub resource_waste: ResourceWasteAssessment,
}

/// Resource waste assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceWasteAssessment {
    /// Wasted memory percentage
    pub wasted_memory_percent: f64,
    /// Wasted CPU cycles percentage
    pub wasted_cpu_percent: f64,
    /// Premature destruction events
    pub premature_destructions: u32,
    /// Unused object instances
    pub unused_instances: u32,
    /// Optimization opportunities
    pub optimization_opportunities: Vec<String>,
}

/// Lifecycle pattern information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecyclePattern {
    /// Pattern type
    pub pattern_type: LifecyclePatternType,
    /// Pattern frequency
    pub frequency: u32,
    /// Pattern efficiency
    pub efficiency_score: f64,
    /// Associated performance impact
    pub performance_impact: f64,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<String>,
}

/// Types of lifecycle patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecyclePatternType {
    /// Short-lived objects
    ShortLived,
    /// Long-lived objects
    LongLived,
    /// Cyclical objects
    Cyclical,
    /// On-demand objects
    OnDemand,
    /// Cached objects
    Cached,
    /// Pooled objects
    Pooled,
    /// Singleton objects
    Singleton,
    /// Factory objects
    Factory,
    /// RAII objects
    RAII,
}

/// Memory access pattern tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessTrackingInfo {
    /// Memory region identifier
    pub region_id: usize,
    /// Memory address range
    pub address_range: AddressRange,
    /// Access events
    pub access_events: Vec<MemoryAccessEvent>,
    /// Access statistics
    pub access_statistics: MemoryAccessStatistics,
    /// Access patterns
    pub access_patterns: Vec<AccessPattern>,
    /// Performance impact
    pub performance_impact: MemoryAccessPerformanceImpact,
}

/// Memory address range
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddressRange {
    /// Start address
    pub start_address: usize,
    /// End address
    pub end_address: usize,
    /// Size in bytes
    pub size: usize,
}

/// Memory access event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessEvent {
    /// Access type
    pub access_type: MemoryAccessType,
    /// Timestamp
    pub timestamp: u64,
    /// Memory address
    pub address: usize,
    /// Access size
    pub size: usize,
    /// Function that performed the access
    pub function_name: String,
    /// Access latency
    pub latency_ns: u64,
    /// Cache hit/miss information
    pub cache_info: CacheAccessInfo,
}

/// Types of memory access
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAccessType {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Read-modify-write access
    ReadModifyWrite,
    /// Prefetch access
    Prefetch,
    /// Flush access
    Flush,
}

/// Cache access information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheAccessInfo {
    /// L1 cache hit
    pub l1_hit: bool,
    /// L2 cache hit
    pub l2_hit: bool,
    /// L3 cache hit
    pub l3_hit: bool,
    /// Memory access required
    pub memory_access: bool,
    /// Access latency breakdown
    pub latency_breakdown: CacheLatencyBreakdown,
}

/// Cache latency breakdown
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheLatencyBreakdown {
    /// L1 cache latency
    pub l1_latency_ns: f64,
    /// L2 cache latency
    pub l2_latency_ns: f64,
    /// L3 cache latency
    pub l3_latency_ns: f64,
    /// Main memory latency
    pub memory_latency_ns: f64,
}

/// Memory access statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessStatistics {
    /// Total read operations
    pub total_reads: u64,
    /// Total write operations
    pub total_writes: u64,
    /// Read/write ratio
    pub read_write_ratio: f64,
    /// Average access frequency per second
    pub avg_access_frequency: f64,
    /// Peak access frequency
    pub peak_access_frequency: f64,
    /// Access locality metrics
    pub locality_metrics: LocalityMetrics,
    /// Bandwidth utilization
    pub bandwidth_utilization: BandwidthUtilization,
}

/// Memory locality metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalityMetrics {
    /// Temporal locality score (0-1)
    pub temporal_locality: f64,
    /// Spatial locality score (0-1)
    pub spatial_locality: f64,
    /// Sequential access percentage
    pub sequential_access_percent: f64,
    /// Random access percentage
    pub random_access_percent: f64,
    /// Stride pattern detection
    pub stride_patterns: Vec<StridePattern>,
}

/// Stride pattern information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StridePattern {
    /// Stride size in bytes
    pub stride_size: usize,
    /// Pattern frequency
    pub frequency: u32,
    /// Pattern efficiency
    pub efficiency_score: f64,
    /// Cache friendliness
    pub cache_friendliness: f64,
}

/// Bandwidth utilization information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandwidthUtilization {
    /// Peak bandwidth usage (bytes/sec)
    pub peak_bandwidth: f64,
    /// Average bandwidth usage
    pub avg_bandwidth: f64,
    /// Bandwidth efficiency percentage
    pub efficiency_percent: f64,
    /// Bottleneck identification
    pub bottlenecks: Vec<BandwidthBottleneck>,
}

/// Bandwidth bottleneck information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BandwidthBottleneck {
    /// Bottleneck location
    pub location: BandwidthBottleneckLocation,
    /// Impact severity
    pub severity: ImpactLevel,
    /// Description
    pub description: String,
    /// Mitigation suggestions
    pub mitigation_suggestions: Vec<String>,
}

/// Bandwidth bottleneck locations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BandwidthBottleneckLocation {
    /// L1 cache bottleneck
    L1Cache,
    /// L2 cache bottleneck
    L2Cache,
    /// L3 cache bottleneck
    L3Cache,
    /// Main memory bottleneck
    MainMemory,
    /// Memory controller bottleneck
    SystemBus,
    /// PCIe bottleneck
    PCIe,
    /// Network bottleneck
    Network,
}

/// Access pattern information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessPattern {
    /// Pattern type
    pub pattern_type: AccessPatternType,
    /// Pattern description
    pub description: String,
    /// Frequency of this pattern
    pub frequency: u32,
    /// Performance characteristics
    pub performance_characteristics: AccessPatternPerformance,
    /// Optimization potential
    pub optimization_potential: f64,
}

/// Types of access patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccessPatternType {
    /// Sequential access pattern
    Sequential,
    /// Random access pattern
    Random,
    /// Strided access pattern
    Strided,
    /// Hotspot access pattern
    Hotspot,
    /// Sparse access pattern
    Sparse,
    /// Dense access pattern
    Dense,
    /// Temporal access pattern
    Temporal,
    /// Spatial access pattern
    Spatial,
}

/// Access pattern performance characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessPatternPerformance {
    /// Cache hit rate for this pattern
    pub cache_hit_rate: f64,
    /// Average latency for this pattern
    pub avg_latency_ns: f64,
    /// Bandwidth efficiency
    pub bandwidth_efficiency: f64,
    /// Prefetcher effectiveness
    pub prefetcher_effectiveness: f64,
}

/// Memory access performance impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryAccessPerformanceImpact {
    /// Overall performance score
    pub performance_score: f64,
    /// Cache efficiency impact
    pub cache_efficiency_impact: f64,
    /// Memory bandwidth impact
    pub bandwidth_impact: f64,
    /// CPU pipeline impact
    pub pipeline_impact: f64,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<MemoryOptimizationRecommendation>,
}

/// Memory optimization recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryOptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: MemoryOptimizationType,
    /// Priority
    pub priority: Priority,
    /// Expected improvement
    pub expected_improvement: f64,
    /// Implementation effort
    pub implementation_effort: ImplementationDifficulty,
    /// Description
    pub description: String,
}

/// Types of memory optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryOptimizationType {
    /// Data layout optimization
    DataLayout,
    /// Access pattern optimization
    AccessPattern,
    /// Prefetching optimization
    Prefetching,
    /// Caching optimization
    Caching,
    /// Memory pooling optimization
    MemoryPooling,
    /// NUMA optimization
    NUMA,
    /// Vectorization optimization
    Vectorization,
    /// Compression optimization
    Compression,
}

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation;
// pub mod visualization;
// pub mod analysis;

/// Timeline event for memory visualization (simple version)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
}

/// Timeline data structure for visualization (simple version)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimpleTimelineData {
    pub events: Vec<TimelineEvent>,
    pub start_time: u64,
    pub end_time: u64,
}
