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
#[derive(Debug, Clone)]
pub enum TrackingError {
    AllocationFailed(String),
    DeallocationFailed(String),
    TrackingDisabled,
    InvalidPointer(String),
    SerializationError(String),
    VisualizationError(String),
    ThreadSafetyError(String),
    ConfigurationError(String),
    AnalysisError(String),
    ExportError(String),
    MemoryCorruption(String),
    UnsafeOperationDetected(String),
    FFIError(String),
    ScopeError(String),
    BorrowCheckError(String),
    LifetimeError(String),
    TypeInferenceError(String),
    PerformanceError(String),
    ResourceExhausted(String),
    InternalError(String),
    IoError(std::io::Error),
    LockError(String),
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
}

/// Memory type analysis
#[derive(Debug, Clone)]
pub struct MemoryTypeInfo {
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub average_size: usize,
    pub largest_allocation: usize,
    pub smallest_allocation: usize,
    pub active_instances: usize,
    pub leaked_instances: usize,
}

/// Type memory usage information
#[derive(Debug, Clone)]
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
}

/// Scope lifecycle metrics
#[derive(Debug, Clone)]
pub struct ScopeLifecycleMetrics {
    pub scope_name: String,
    pub variable_count: usize,
    pub average_lifetime_ms: f64,
    pub total_memory_usage: usize,
    pub peak_memory_usage: usize,
    pub allocation_frequency: f64,
    pub deallocation_efficiency: f64,
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
}

/// Scope hierarchy
#[derive(Debug, Clone)]
pub struct ScopeHierarchy {
    pub root_scopes: Vec<String>,
    pub scope_tree: HashMap<String, ScopeInfo>,
    pub max_depth: usize,
    pub total_scopes: usize,
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
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationEventType {
    Allocate,
    Deallocate,
    Reallocate,
    Move,
    Borrow,
    Return,
}

/// Type of scope event
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeEventType {
    Enter,
    Exit,
    Create,
    Destroy,
}

// TODO: Gradually move types to these modules:
// pub mod core;
// pub mod allocation; 
// pub mod visualization;
// pub mod analysis;