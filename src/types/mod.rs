// Types module - Refactored from the original monolithic types.rs
// This module organizes types into logical groups for better maintainability

// For now, we need to import the original types from the backup file
// Since we removed types_original.rs, we need to recreate the essential types here

// Essential types that are used throughout the codebase
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

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation; 
// pub mod visualization;
// pub mod analysis;