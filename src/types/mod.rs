// Types module - Refactored from the original monolithic types.rs
// This module organizes types into logical groups for better maintainability

// For now, we need to import the original types from the backup file
// Since we removed types_original.rs, we need to recreate the essential types here

// Essential types that are used throughout the codebase
use std::collections::HashMap;
use std::thread;
use serde::{Deserialize, Serialize};

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
    IoError(std::io::Error),
    /// Lock acquisition failed
    LockError(String),
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
            TrackingError::UnsafeOperationDetected(s) => TrackingError::UnsafeOperationDetected(s.clone()),
            TrackingError::FFIError(s) => TrackingError::FFIError(s.clone()),
            TrackingError::ScopeError(s) => TrackingError::ScopeError(s.clone()),
            TrackingError::BorrowCheckError(s) => TrackingError::BorrowCheckError(s.clone()),
            TrackingError::LifetimeError(s) => TrackingError::LifetimeError(s.clone()),
            TrackingError::TypeInferenceError(s) => TrackingError::TypeInferenceError(s.clone()),
            TrackingError::PerformanceError(s) => TrackingError::PerformanceError(s.clone()),
            TrackingError::ResourceExhausted(s) => TrackingError::ResourceExhausted(s.clone()),
            TrackingError::InternalError(s) => TrackingError::InternalError(s.clone()),
            TrackingError::IoError(e) => TrackingError::IoError(std::io::Error::new(e.kind(), e.to_string())),
            TrackingError::LockError(s) => TrackingError::LockError(s.clone()),
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
            TrackingError::UnsafeOperationDetected(msg) => write!(f, "Unsafe operation detected: {msg}"),
            TrackingError::FFIError(msg) => write!(f, "FFI error: {msg}"),
            TrackingError::ScopeError(msg) => write!(f, "Scope error: {msg}"),
            TrackingError::BorrowCheckError(msg) => write!(f, "Borrow check error: {msg}"),
            TrackingError::LifetimeError(msg) => write!(f, "Lifetime error: {msg}"),
            TrackingError::TypeInferenceError(msg) => write!(f, "Type inference error: {msg}"),
            TrackingError::PerformanceError(msg) => write!(f, "Performance error: {msg}"),
            TrackingError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {msg}"),
            TrackingError::InternalError(msg) => write!(f, "Internal error: {msg}"),
            TrackingError::IoError(err) => write!(f, "IO error: {err}"),
            TrackingError::LockError(msg) => write!(f, "Lock error: {msg}"),
        }
    }
}

impl std::error::Error for TrackingError {}

impl From<std::io::Error> for TrackingError {
    fn from(error: std::io::Error) -> Self {
        TrackingError::IoError(error)
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
    pub clones: Vec<usize>,
    
    /// Reference count history (timestamp, count)
    pub ref_count_history: Vec<RefCountSnapshot>,
    
    /// Weak reference information
    pub weak_count: Option<usize>,
    pub is_weak_reference: bool,
    
    /// Lifecycle information
    pub is_data_owner: bool,  // Is this the last strong reference?
    pub is_implicitly_deallocated: bool, // Was data deallocated when this was dropped?
    
    /// Smart pointer type
    pub pointer_type: SmartPointerType,
}

/// Reference count snapshot at a specific time
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RefCountSnapshot {
    pub timestamp: u64,
    pub strong_count: usize,
    pub weak_count: usize,
}

/// Type of smart pointer
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SmartPointerType {
    Rc,
    Arc,
    RcWeak,
    ArcWeak,
    Box,
}

impl SmartPointerInfo {
    /// Create new smart pointer info for Rc/Arc
    pub fn new_rc_arc(data_ptr: usize, pointer_type: SmartPointerType, strong_count: usize, weak_count: usize) -> Self {
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
    pub thread_id: thread::ThreadId,
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
            thread_id: thread::current().id(), // Default to current thread
            borrow_count: helper.borrow_count,
            stack_trace: helper.stack_trace,
            is_leaked: helper.is_leaked,
            lifetime_ms: helper.lifetime_ms,
            smart_pointer_info: None, // Default for deserialization
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
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
            thread_id: thread::current().id(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
        }
    }

    /// Mark this allocation as deallocated with current timestamp
    pub fn mark_deallocated(&mut self) {
        self.timestamp_dealloc = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64);
    }

    /// Check if this allocation is still active (not deallocated)
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }
}

/// Memory statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
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
#[derive(Debug, Clone, serde::Serialize)]
pub struct TypeMemoryUsage {
    /// Name of the type
    pub type_name: String,
    /// Total size allocated for this type
    pub total_size: usize,
    /// Number of allocations for this type
    pub allocation_count: usize,
    /// Average allocation size for this type
    pub average_size: f64,
    /// Peak memory usage for this type
    pub peak_size: usize,
    /// Current memory usage for this type
    pub current_size: usize,
    /// Memory efficiency score for this type
    pub efficiency_score: f64,
}

/// Fragmentation analysis
#[derive(Debug, Clone, Default, serde::Serialize)]
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

/// System library usage statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
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
#[derive(Debug, Clone, Default, serde::Serialize)]
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

/// Concurrency safety analysis
#[derive(Debug, Clone, Default, serde::Serialize)]
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

/// Scope lifecycle metrics
#[derive(Debug, Clone, Default, serde::Serialize)]
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
#[derive(Debug, Clone, Default, serde::Serialize)]
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
#[derive(Debug, Clone, serde::Serialize)]
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
#[derive(Debug, Clone, serde::Serialize)]
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
    Minor { potential_savings: usize },
    /// Moderate optimization
    Moderate { potential_savings: usize, suggestions: Vec<String> },
    /// Major optimization
    Major { potential_savings: usize, suggestions: Vec<String> },
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
    Low,
    Moderate,
    High,
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
    Trait(String),
    Lifetime(String),
    Associated(String),
    Where(String),
}

/// Memory impact
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryImpact {
    None,
    SizeIncrease(usize),
    AlignmentChange(usize),
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
    Negligible,
    Minor,
    Moderate,
    Significant,
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
    Low,
    Moderate,
    High,
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
    Sequential,
    Random,
    Strided { stride: usize },
    Clustered,
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

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation; 
// pub mod visualization;
// pub mod analysis;