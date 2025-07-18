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
            TrackingError::AllocationFailed(msg) => write!(f, "Allocation failed: {}", msg),
            TrackingError::DeallocationFailed(msg) => write!(f, "Deallocation failed: {}", msg),
            TrackingError::TrackingDisabled => write!(f, "Memory tracking is disabled"),
            TrackingError::InvalidPointer(msg) => write!(f, "Invalid pointer: {}", msg),
            TrackingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            TrackingError::VisualizationError(msg) => write!(f, "Visualization error: {}", msg),
            TrackingError::ThreadSafetyError(msg) => write!(f, "Thread safety error: {}", msg),
            TrackingError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            TrackingError::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
            TrackingError::ExportError(msg) => write!(f, "Export error: {}", msg),
            TrackingError::MemoryCorruption(msg) => write!(f, "Memory corruption detected: {}", msg),
            TrackingError::UnsafeOperationDetected(msg) => write!(f, "Unsafe operation detected: {}", msg),
            TrackingError::FFIError(msg) => write!(f, "FFI error: {}", msg),
            TrackingError::ScopeError(msg) => write!(f, "Scope error: {}", msg),
            TrackingError::BorrowCheckError(msg) => write!(f, "Borrow check error: {}", msg),
            TrackingError::LifetimeError(msg) => write!(f, "Lifetime error: {}", msg),
            TrackingError::TypeInferenceError(msg) => write!(f, "Type inference error: {}", msg),
            TrackingError::PerformanceError(msg) => write!(f, "Performance error: {}", msg),
            TrackingError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            TrackingError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            TrackingError::IoError(err) => write!(f, "IO error: {}", err),
            TrackingError::LockError(msg) => write!(f, "Lock error: {}", msg),
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
        TrackingError::SerializationError(format!("JSON error: {}", error))
    }
}

/// Information about a memory allocation
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AllocationInfo {
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub scope_name: Option<String>,
    pub timestamp_alloc: u64,
    pub timestamp_dealloc: Option<u64>,
    #[serde(skip)]
    pub thread_id: thread::ThreadId,
    pub borrow_count: usize,
    pub stack_trace: Option<Vec<String>>,
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

    pub fn mark_deallocated(&mut self) {
        self.timestamp_dealloc = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64);
    }

    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }
}

/// Memory statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_allocations: usize,
    pub total_allocated: usize,
    pub active_allocations: usize,
    pub active_memory: usize,
    pub peak_allocations: usize,
    pub peak_memory: usize,
    pub total_deallocations: usize,
    pub total_deallocated: usize,
    pub leaked_allocations: usize,
    pub leaked_memory: usize,
    pub fragmentation_analysis: FragmentationAnalysis,
    pub lifecycle_stats: ScopeLifecycleMetrics,
    pub allocations: Vec<AllocationInfo>,
    pub system_library_stats: SystemLibraryStats,
    pub concurrency_analysis: ConcurrencyAnalysis,
}

impl MemoryStats {
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
#[derive(Debug, Clone)]
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
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub average_size: f64,
    pub peak_size: usize,
    pub current_size: usize,
    pub efficiency_score: f64,
}

/// Fragmentation analysis
#[derive(Debug, Clone, Default)]
pub struct FragmentationAnalysis {
    pub fragmentation_ratio: f64,
    pub largest_free_block: usize,
    pub smallest_free_block: usize,
    pub free_block_count: usize,
    pub total_free_memory: usize,
    pub external_fragmentation: f64,
    pub internal_fragmentation: f64,
}

/// System library usage statistics
#[derive(Debug, Clone, Default)]
pub struct SystemLibraryStats {
    pub std_collections: LibraryUsage,
    pub async_runtime: LibraryUsage,
    pub network_io: LibraryUsage,
    pub file_system: LibraryUsage,
    pub serialization: LibraryUsage,
    pub regex_engine: LibraryUsage,
    pub crypto_security: LibraryUsage,
    pub database: LibraryUsage,
    pub graphics_ui: LibraryUsage,
    pub http_stack: LibraryUsage,
}

/// Library usage information
#[derive(Debug, Clone, Default)]
pub struct LibraryUsage {
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub peak_bytes: usize,
    pub average_size: f64,
    pub categories: HashMap<String, usize>,
    pub hotspot_functions: Vec<String>,
}

/// Concurrency safety analysis
#[derive(Debug, Clone, Default)]
pub struct ConcurrencyAnalysis {
    pub thread_safety_allocations: usize,
    pub shared_memory_bytes: usize,
    pub mutex_protected: usize,
    pub arc_shared: usize,
    pub rc_shared: usize,
    pub channel_buffers: usize,
    pub thread_local_storage: usize,
    pub atomic_operations: usize,
    pub lock_contention_risk: String,
}

/// Scope analysis
#[derive(Debug, Clone, Default)]
pub struct ScopeAnalysis {
    pub total_scopes: usize,
    pub active_scopes: usize,
    pub max_depth: usize,
    pub average_lifetime: f64,
    pub memory_efficiency: f64,
    pub scopes: Vec<ScopeInfo>,
    pub scope_hierarchy: ScopeHierarchy,
    pub cross_scope_references: Vec<String>,
}

/// Scope lifecycle metrics
#[derive(Debug, Clone, Default)]
pub struct ScopeLifecycleMetrics {
    pub scope_name: String,
    pub variable_count: usize,
    pub average_lifetime_ms: f64,
    pub total_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub allocation_frequency: f64,
    pub deallocation_efficiency: f64,
    pub completed_allocations: usize,
    pub memory_growth_events: usize,
    pub peak_concurrent_variables: usize,
    pub memory_efficiency_ratio: f64,
    pub ownership_transfer_events: usize,
    pub fragmentation_score: f64,
    pub instant_allocations: usize,
    pub short_term_allocations: usize,
    pub medium_term_allocations: usize,
    pub long_term_allocations: usize,
    pub suspected_leaks: usize,
    pub risk_distribution: RiskDistribution,
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    pub type_lifecycle_patterns: Vec<TypeLifecyclePattern>,
}

/// Scope information
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub name: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub depth: usize,
    pub variables: Vec<String>,
    pub total_memory: usize,
    pub peak_memory: usize,
    pub allocation_count: usize,
    pub lifetime_start: Option<u64>,
    pub lifetime_end: Option<u64>,
    pub is_active: bool,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub memory_usage: usize,
    pub child_scopes: Vec<String>,
    pub parent_scope: Option<String>,
}

/// Scope hierarchy
#[derive(Debug, Clone, Default)]
pub struct ScopeHierarchy {
    pub root_scopes: Vec<String>,
    pub scope_tree: HashMap<String, ScopeInfo>,
    pub max_depth: usize,
    pub total_scopes: usize,
    pub relationships: HashMap<String, Vec<String>>,
    pub depth_map: HashMap<String, usize>,
}

/// Risk distribution analysis for memory allocations
#[derive(Debug, Clone, Default)]
pub struct RiskDistribution {
    pub low_risk: usize,
    pub medium_risk: usize,
    pub high_risk: usize,
    pub critical_risk: usize,
}

/// Type-specific lifecycle pattern analysis
#[derive(Debug, Clone)]
pub struct TypeLifecyclePattern {
    pub type_name: String,
    pub average_lifetime_ms: f64,
    pub typical_size: usize,
    pub growth_pattern: String,
    pub risk_level: String,
    pub instance_count: usize,
}

/// Growth reason for tracking allocation growth
#[derive(Debug, Clone, PartialEq)]
pub enum GrowthReason {
    Initial,
    Expansion,
    Reallocation,
    Optimization,
    UserRequested,
}

/// Type of allocation event
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum AllocationEventType {
    Allocate,
    Deallocate,
    Reallocate,
    Move,
    Borrow,
    Return,
}

/// Type of scope event
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ScopeEventType {
    Enter,
    Exit,
    Create,
    Destroy,
}

/// Growth event tracking allocation growth patterns
#[derive(Debug, Clone)]
pub struct GrowthEvent {
    pub timestamp: u64,
    pub old_size: usize,
    pub new_size: usize,
    pub growth_factor: f64,
    pub reason: GrowthReason,
    pub var_name: String,
}

/// Borrow event for tracking borrowing patterns
#[derive(Debug, Clone)]
pub struct BorrowEvent {
    pub timestamp: u64,
    pub ptr: usize,
    pub borrow_type: String,
    pub duration_ms: u64,
    pub var_name: String,
}

/// Move event for tracking ownership transfers
#[derive(Debug, Clone)]
pub struct MoveEvent {
    pub timestamp: u64,
    pub from_ptr: usize,
    pub to_ptr: usize,
    pub size: usize,
    pub var_name: String,
}

/// Variable relationship tracking
#[derive(Debug, Clone)]
pub struct VariableRelationship {
    pub source_var: String,
    pub target_var: String,
    pub relationship_type: String,
    pub strength: f64,
}

/// Potential memory leak detection
#[derive(Debug, Clone)]
pub struct PotentialLeak {
    pub ptr: usize,
    pub size: usize,
    pub age_ms: u64,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub severity: String,
}

/// Timeline data for visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimelineData {
    pub time_range: TimeRange,
    pub allocation_events: Vec<AllocationEvent>,
    pub scope_events: Vec<ScopeEvent>,
    pub memory_snapshots: Vec<MemorySnapshot>,
}

/// Time range for timeline visualization
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimeRange {
    pub start_time: u64,
    pub end_time: u64,
    pub duration_ms: u64,
}

/// Memory snapshot at a point in time
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemorySnapshot {
    pub timestamp: u64,
    pub total_memory: usize,
    pub active_allocations: usize,
    pub fragmentation_ratio: f64,
    pub top_types: Vec<TypeMemoryUsage>,
}

/// Allocation event for timeline
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationEvent {
    pub timestamp: u64,
    pub event_type: AllocationEventType,
    pub ptr: usize,
    pub size: usize,
    pub var_name: Option<String>,
    pub type_name: Option<String>,
}

/// Scope event for timeline
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScopeEvent {
    pub timestamp: u64,
    pub event_type: ScopeEventType,
    pub scope_name: String,
    pub memory_usage: usize,
    pub variable_count: usize,
}

/// Stack trace data for analysis
#[derive(Debug, Clone)]
pub struct StackTraceData {
    pub hotspots: Vec<StackTraceHotspot>,
    pub allocation_patterns: Vec<AllocationPattern>,
    pub total_samples: usize,
}

/// Stack trace hotspot
#[derive(Debug, Clone)]
pub struct StackTraceHotspot {
    pub function_name: String,
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub average_size: f64,
    pub percentage: f64,
}

/// Allocation pattern analysis
#[derive(Debug, Clone)]
pub struct AllocationPattern {
    pub pattern_type: String,
    pub frequency: usize,
    pub total_bytes: usize,
    pub description: String,
}

/// Stack frame for stack traces
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub module_path: Option<String>,
}

/// Safety violation types
#[derive(Debug, Clone)]
pub enum SafetyViolation {
    PotentialLeak {
        ptr: usize,
        size: usize,
        age_ms: u64,
        description: String,
    },
    UseAfterFree {
        ptr: usize,
        description: String,
    },
    DoubleFree {
        ptr: usize,
        description: String,
    },
    BufferOverflow {
        ptr: usize,
        size: usize,
        description: String,
    },
}

/// Allocation hotspot information
#[derive(Debug, Clone)]
pub struct AllocationHotspot {
    pub location: HotspotLocation,
    pub allocation_count: usize,
    pub total_bytes: usize,
    pub average_size: f64,
    pub frequency: f64,
}

/// Hotspot location information
#[derive(Debug, Clone)]
pub struct HotspotLocation {
    pub function_name: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub module_path: Option<String>,
}

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation; 
// pub mod visualization;
// pub mod analysis;