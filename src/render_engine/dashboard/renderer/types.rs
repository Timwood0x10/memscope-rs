//! Dashboard data types for template rendering.

use serde::{Deserialize, Serialize};

/// Risk score penalty per high-risk operation.
/// Each high-risk operation reduces the health score by this amount.
pub const HIGH_RISK_PENALTY: f64 = 10.0;

/// Dashboard context for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardContext {
    /// Page title
    pub title: String,
    /// Export timestamp
    pub export_timestamp: String,
    /// Total memory allocated (formatted)
    pub total_memory: String,
    /// Total number of allocations
    pub total_allocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Peak memory usage (formatted)
    pub peak_memory: String,
    /// Number of threads
    pub thread_count: usize,
    /// Number of memory passports
    pub passport_count: usize,
    /// Number of memory leaks detected
    pub leak_count: usize,
    /// Number of unsafe operations
    pub unsafe_count: usize,
    /// Number of FFI operations
    pub ffi_count: usize,
    /// Allocation information
    pub allocations: Vec<AllocationInfo>,
    /// Variable relationships
    pub relationships: Vec<RelationshipInfo>,
    /// Unsafe/FFI reports
    pub unsafe_reports: Vec<UnsafeReport>,
    /// Detailed passport information
    pub passport_details: Vec<PassportDetail>,
    /// Count helper for template
    pub allocations_count: usize,
    /// Count helper for template
    pub relationships_count: usize,
    /// Count helper for template
    pub unsafe_reports_count: usize,
    /// JSON data string for injection (performance optimization)
    pub json_data: String,
    /// OS name
    pub os_name: String,
    /// Architecture
    pub architecture: String,
    /// CPU cores
    pub cpu_cores: usize,
    /// System resources
    pub system_resources: SystemResources,
    /// Thread analysis data
    pub threads: Vec<ThreadInfo>,
    /// Async task analysis data
    pub async_tasks: Vec<AsyncTaskInfo>,
    /// Async summary
    pub async_summary: AsyncSummary,
    /// Health score (0-100)
    pub health_score: u32,
    /// Health status text
    pub health_status: String,
    /// Safe operations count
    pub safe_ops_count: usize,
    /// High risk issues count
    pub high_risk_count: usize,
    /// Clean passports count
    pub clean_passport_count: usize,
    /// Active passports count
    pub active_passport_count: usize,
    /// Leaked passports count
    pub leaked_passport_count: usize,
    /// FFI tracked passports count
    pub ffi_tracked_count: usize,
    /// Safe code percentage
    pub safe_code_percent: u32,
    /// Ownership graph information
    pub ownership_graph: OwnershipGraphInfo,
    /// Top N allocation sites
    pub top_allocation_sites: Vec<TopAllocationSite>,
    /// Top N leaked allocations
    pub top_leaked_allocations: Vec<TopLeakedAllocation>,
    /// Top N temporary churn
    pub top_temporary_churn: Vec<TopTemporaryChurn>,
}

/// Ownership graph information for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraphInfo {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Number of detected cycles
    pub total_cycles: usize,
    /// Rc clone count
    pub rc_clone_count: usize,
    /// Arc clone count
    pub arc_clone_count: usize,
    /// Whether there are issues
    pub has_issues: bool,
    /// Detected issues
    pub issues: Vec<OwnershipIssue>,
    /// Root cause if any
    pub root_cause: Option<RootCauseInfo>,
}

/// Ownership issue for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipIssue {
    /// Issue type
    pub issue_type: String,
    /// Severity (error, warning)
    pub severity: String,
    /// Description
    pub description: String,
}

/// Root cause information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseInfo {
    /// Cause type
    pub cause: String,
    /// Description
    pub description: String,
    /// Impact
    pub impact: String,
}

/// Async task information for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTaskInfo {
    /// Task ID
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Task type
    pub task_type: String,
    /// Total bytes allocated
    pub total_bytes: u64,
    /// Current memory usage
    pub current_memory: u64,
    /// Peak memory usage
    pub peak_memory: u64,
    /// Number of allocations
    pub total_allocations: u64,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Efficiency score (0.0 - 1.0)
    pub efficiency_score: f64,
    /// Whether task is completed
    pub is_completed: bool,
    /// Whether task has potential leak
    pub has_potential_leak: bool,
}

/// Async summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncSummary {
    /// Total number of async tasks
    pub total_tasks: usize,
    /// Number of active tasks
    pub active_tasks: usize,
    /// Total allocations across all tasks
    pub total_allocations: usize,
    /// Total memory bytes
    pub total_memory_bytes: usize,
    /// Peak memory bytes
    pub peak_memory_bytes: usize,
}

/// Allocation information for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address
    pub address: String,
    /// Type name
    pub type_name: String,
    /// Allocation size in bytes
    pub size: usize,
    /// Variable name
    pub var_name: String,
    /// Timestamp
    pub timestamp: String,
    /// Thread ID
    pub thread_id: String,
    /// Borrow information
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    /// Clone information
    pub is_clone: bool,
    pub clone_count: usize,
    /// Allocation timestamp (nanoseconds)
    pub timestamp_alloc: u64,
    /// Deallocation timestamp (nanoseconds, 0 if not freed)
    pub timestamp_dealloc: u64,
    /// Lifetime in milliseconds
    pub lifetime_ms: f64,
    /// Whether memory is leaked
    pub is_leaked: bool,
    /// Allocation type (stack, heap, etc.)
    pub allocation_type: String,
    /// Whether this is a smart pointer
    pub is_smart_pointer: bool,
    /// Smart pointer type (Arc, Rc, Box, etc.)
    pub smart_pointer_type: String,
    /// Source file where allocation occurred
    pub source_file: Option<String>,
    /// Source line where allocation occurred
    pub source_line: Option<u32>,
    /// Module path where allocation occurred
    pub module_path: Option<String>,
}

/// Thread statistics for multithread dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadStats {
    /// Thread ID
    pub id: u64,
    /// Number of allocations
    pub allocations: usize,
    /// Total memory used
    pub memory: usize,
    /// Peak memory usage
    pub peak: usize,
    /// Thread status
    pub status: String,
}

/// Timeline allocation for multithread dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineAllocation {
    /// Timestamp
    pub timestamp: u64,
    /// Thread ID
    pub thread_id: u64,
    /// Allocation size
    pub size: usize,
    /// Variable name
    pub var_name: Option<String>,
}

/// Thread conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadConflict {
    /// Description of the conflict
    pub description: String,
    /// Threads involved
    pub threads: String,
    /// Conflict type
    #[serde(rename = "type")]
    pub conflict_type: String,
}

/// Variable relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInfo {
    /// Source pointer
    pub source_ptr: String,
    /// Source variable name
    pub source_var_name: String,
    /// Target pointer
    pub target_ptr: String,
    /// Target variable name
    pub target_var_name: String,
    /// Relationship type (reference, borrow, clone, copy, move, ownership_transfer)
    pub relationship_type: String,
    /// Relationship strength (0.0 to 1.0)
    pub strength: f64,
    /// Type name
    pub type_name: String,
    /// Color for visualization
    pub color: String,
    /// Whether this relationship is part of a detected cycle (true) or not (false)
    pub is_part_of_cycle: bool,
    /// Whether source is a Container type (no heap pointer)
    pub is_container_source: bool,
    /// Whether target is a Container type (no heap pointer)
    pub is_container_target: bool,
}

/// Unsafe/FFI report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeReport {
    /// Passport ID
    pub passport_id: String,
    /// Allocation pointer
    pub allocation_ptr: String,
    /// Variable name
    pub var_name: String,
    /// Type name
    pub type_name: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Created at timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Status at shutdown
    pub status: String,
    /// Lifecycle events
    pub lifecycle_events: Vec<LifecycleEventInfo>,
    /// Cross-boundary events
    pub cross_boundary_events: Vec<BoundaryEventInfo>,
    /// Whether this is a memory leak
    pub is_leaked: bool,
    /// Risk level (low, medium, high)
    pub risk_level: String,
    /// Risk factors
    pub risk_factors: Vec<String>,
}

/// Lifecycle event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEventInfo {
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: u64,
    /// Context
    pub context: String,
    /// Event icon
    pub icon: String,
    /// Event color
    pub color: String,
}

/// Boundary event information (FFI/Rust crossings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryEventInfo {
    /// Event type (RustToFfi, FfiToRust, etc.)
    pub event_type: String,
    /// Source context
    pub from_context: String,
    /// Target context
    pub to_context: String,
    /// Timestamp
    pub timestamp: u64,
    /// Direction icon
    pub icon: String,
    /// Direction color
    pub color: String,
}

/// Detailed passport information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportDetail {
    /// Passport ID
    pub passport_id: String,
    /// Allocation pointer
    pub allocation_ptr: String,
    /// Variable name
    pub var_name: String,
    /// Type name
    pub type_name: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Status at shutdown
    pub status: String,
    /// Created at
    pub created_at: u64,
    /// Updated at
    pub updated_at: u64,
    /// Whether leaked
    pub is_leaked: bool,
    /// Whether FFI tracked
    pub ffi_tracked: bool,
    /// Lifecycle events
    pub lifecycle_events: Vec<LifecycleEventInfo>,
    /// Cross-boundary events
    pub cross_boundary_events: Vec<BoundaryEventInfo>,
    /// Risk level
    pub risk_level: String,
    /// Risk confidence
    pub risk_confidence: f64,
}

/// System resources information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// OS name
    pub os_name: String,
    /// OS version
    pub os_version: String,
    /// CPU architecture
    pub architecture: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Total physical memory (formatted)
    pub total_physical: String,
    /// Available physical memory (formatted)
    pub available_physical: String,
    /// Used physical memory (formatted)
    pub used_physical: String,
    /// Page size
    pub page_size: u64,
}

/// Thread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID (formatted as "Thread-N" instead of "ThreadId(N)")
    pub thread_id: String,
    /// Thread summary (e.g., "5 allocs, 1.2KB")
    pub thread_summary: String,
    /// Number of allocations
    pub allocation_count: usize,
    /// Current memory usage
    pub current_memory: String,
    /// Peak memory usage
    pub peak_memory: String,
    /// Total allocated
    pub total_allocated: String,
    /// Raw current memory in bytes for sorting
    pub current_memory_bytes: usize,
    /// Raw peak memory in bytes for sorting
    pub peak_memory_bytes: usize,
    /// Raw total allocated in bytes for sorting
    pub total_allocated_bytes: usize,
}

/// Thread aggregator for internal use
#[derive(Debug, Clone, Default)]
pub struct ThreadAggregator {
    pub allocation_count: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub total_allocated: usize,
}

/// Top N allocation site for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAllocationSite {
    /// Site name (usually from stack trace)
    pub name: String,
    /// Total bytes allocated at this site
    pub total_bytes: usize,
    /// Number of allocations at this site
    pub allocation_count: usize,
}

/// Top N leaked allocation for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLeakedAllocation {
    /// Memory address
    pub address: String,
    /// Size in bytes
    pub size: usize,
    /// Type name
    pub type_name: String,
    /// Allocation timestamp
    pub timestamp_alloc: u64,
    /// Stack trace
    pub stack_trace: Option<Vec<String>>,
}

/// Top N temporary churn for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTemporaryChurn {
    /// Site name
    pub name: String,
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
}
