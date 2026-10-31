//! Dashboard data types for template rendering.

use super::evidence::{EvidenceLevel, RiskConfidence};
use serde::{Deserialize, Serialize};

mod semantics;
pub use semantics::*;

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
    /// Top N temporary churn (short-lived allocations)
    pub top_temporary_churn: Vec<TopTemporaryChurn>,
    /// Circular reference analysis
    pub circular_references: CircularReferenceReport,
    /// Task graph JSON string
    pub task_graph_json: String,

    // ========================================
    // New fields for Kinetic Engineering professional template features
    // ========================================
    /// FFI call mapping topology nodes and edges
    #[serde(default)]
    pub ffi_call_topology: FfiCallTopology,
    /// Symbol table analysis entries
    #[serde(default)]
    pub symbol_table: Vec<SymbolTableEntry>,
    /// Symbol table entry count (template helper, avoids .length in Handlebars)
    #[serde(default)]
    pub symbol_table_count: usize,
    /// Stack integrity metrics
    #[serde(default)]
    pub stack_integrity: StackIntegrityMetrics,
    /// Resource usage bars (bridge pool, serialization overhead)
    #[serde(default)]
    pub resource_bars: Vec<ResourceUsageBar>,
    /// Thread execution timeline rows
    #[serde(default)]
    pub thread_timeline: Vec<ThreadTimelineRow>,
    /// Thread timeline row count (template helper, avoids .length in Handlebars)
    #[serde(default)]
    pub thread_timeline_count: usize,

    /// Waker efficiency grid (opacity values)
    #[serde(default)]
    pub waker_efficiency_grid: Vec<f64>,
    /// Mean completed-task duration in ms; unavailable without completions.
    #[serde(default)]
    pub task_duration_mean_ms: Option<f64>,
    /// Completed-task durations, not individual Future::poll timings.
    #[serde(default)]
    pub task_duration_samples: Vec<f64>,
    /// Async task topology nodes
    #[serde(default)]
    pub task_topology_nodes: Vec<TaskTopologyNode>,
    /// Async task topology node count (template helper, avoids .length in Handlebars)
    #[serde(default)]
    pub task_topology_nodes_count: usize,
    /// Async task topology edges
    #[serde(default)]
    pub task_topology_edges: Vec<TaskTopologyEdge>,
    /// Async task topology edge count (template helper, avoids .length in Handlebars)
    #[serde(default)]
    pub task_topology_edges_count: usize,
    /// Streaming topology stats
    #[serde(default)]
    pub streaming_topology_stats: StreamingTopologyStats,
    /// Trace log entries
    #[serde(default)]
    pub trace_logs: Vec<TraceLogEntry>,

    /// Released binding lifetimes and censored live ages at one capture cut.
    #[serde(default)]
    pub allocation_lifetimes: AllocationLifetimeDistributions,
    /// Dependency graph peripheral nodes
    #[serde(default)]
    pub dependency_graph_nodes: Vec<DependencyNode>,
    /// Selected node detail panel data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_node_detail: Option<NodeDetailPanel>,

    /// Thread affinity hardware grid
    #[serde(default)]
    pub thread_affinity_grid: Vec<String>, // status values
    /// Same-thread inter-allocation interval histogram from recorded Allocate events.
    #[serde(default)]
    pub allocation_interval_histogram: Vec<NeighborDensityBin>,
    /// Mean same-thread interval; missing when fewer than two observations exist.
    #[serde(default)]
    pub allocation_interval_mean_ms: Option<f64>,
    /// Number of intervals included; zero-length observed intervals count too.
    #[serde(default)]
    pub allocation_interval_sample_count: usize,
    /// Share of recorded allocations outside the most frequent allocating thread.
    #[serde(default)]
    pub non_dominant_thread_allocation_share_pct: Option<f64>,
    /// System uptime formatted
    #[serde(default)]
    pub system_uptime_formatted: String,
    /// Thread event log entries
    #[serde(default)]
    pub thread_event_log: Vec<ThreadEventLogEntry>,
    /// Thread policy settings
    #[serde(default)]
    pub thread_policies: Vec<ThreadPolicy>,
    /// Resource limits progress bars
    #[serde(default)]
    pub resource_limits: Vec<ResourceUsageBar>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    /// Actual lifecycle timestamps in milliseconds since the Unix epoch.
    #[serde(default)]
    pub created_at_ms: u64,
    #[serde(default)]
    pub completed_at_ms: Option<u64>,
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
    /// Human-readable status: COMPLETED / LEAKED / RUNNING / WAITING
    #[serde(default)]
    pub status: String,
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
    /// Number of completed tasks (finished without leaking)
    #[serde(default)]
    pub completed: usize,
    /// Number of tasks flagged with potential leaks
    #[serde(default)]
    pub leaked: usize,
    /// Number of zombie tasks (neither completed nor active)
    #[serde(default)]
    pub zombie: usize,
    /// Success rate percentage (completed / total * 100)
    #[serde(default)]
    pub success_rate: f64,
}

/// One of several same-named `AllocationInfo` types in this crate: this one is the
/// the dashboard/JSON DTO; never a source of truth for the ledger.
///
/// Convert at the module boundary instead of passing it across; a global rename is
/// deliberately avoided (see DATA_ACCURACY_DEVELOPMENT_PLAN §14.2).
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
    pub lifetime_ms: Option<f64>,
    /// Censored age at the report cut, separate from completed lifetime.
    #[serde(default)]
    pub observed_age_ms: Option<f64>,
    /// Whether memory is leaked.
    ///
    /// Compatibility projection of [`Self::leak_status`]: only `suspected` or
    /// `confirmed` allow `true`. A missing deallocation timestamp is NOT a leak.
    pub is_leaked: bool,
    /// Physical allocation state observed at the snapshot cut.
    ///
    /// This is a fact, not a conclusion: `live` only means "no release event was
    /// recorded inside the capture window".
    #[serde(default)]
    pub allocation_state: crate::snapshot::AllocationState,
    /// Leak conclusion for this allocation at the snapshot cut.
    #[serde(default)]
    pub leak_status: crate::snapshot::LeakStatus,
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
    /// Allocation generation id (incremented on pointer address reuse)
    pub generation_id: usize,
    /// Pointer provenance (`allocator`, `clone`, `smart_pointer`, `reused_address`).
    ///
    /// Leakage is not provenance: it is reported through `leak_status`.
    pub provenance: String,
    /// Evidence level for this allocation's analysis
    #[serde(default)]
    pub evidence: EvidenceLevel,
    /// Risk confidence for this allocation
    #[serde(default)]
    pub confidence: RiskConfidence,
    /// Type layout snapshot for this allocation
    #[serde(default)]
    pub layout_snapshot: Option<crate::capture::types::TypeLayoutSnapshot>,
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
    /// Generation at the address, when supplied by the report.
    #[serde(default)]
    pub generation: u64,
    /// Evidence retained from a superseded address binding.
    #[serde(default)]
    pub superseded: bool,
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
    /// Human-readable description joining risk factors for display
    #[serde(default)]
    pub description: String,
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
    /// Generation at the address, when supplied by the report.
    #[serde(default)]
    pub generation: u64,
    /// Evidence retained from a superseded address binding.
    #[serde(default)]
    pub superseded: bool,
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
    /// Whether the passport is currently active (not leaked, not freed)
    #[serde(default)]
    pub is_active: bool,
    /// Best-effort source location string (file:line or allocation context)
    #[serde(default)]
    pub source_location: String,
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
    /// Process CPU usage percentage (0.0-100.0), derived from getrusage
    #[serde(default)]
    pub cpu_usage_pct: f64,
    /// Total physical memory in bytes (raw, for percentage computation)
    #[serde(default)]
    pub total_physical_bytes: u64,
    /// Used physical memory in bytes (raw, for percentage computation)
    #[serde(default)]
    pub used_physical_bytes: u64,
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
    /// Whether the thread currently holds active allocations
    #[serde(default)]
    pub is_active: bool,
    /// Human-readable status: ACTIVE / IDLE
    #[serde(default)]
    pub status: String,
    /// Logical CPU core the thread was last observed running on.
    #[serde(default)]
    pub cpu_core: Option<u32>,
}

/// Thread aggregator for internal use
#[derive(Debug, Clone, Default)]
pub struct ThreadAggregator {
    pub allocation_count: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub total_allocated: usize,
    /// Logical CPU core the thread was last observed running on.
    pub cpu_core: Option<u32>,
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

/// Circular reference report for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularReferenceReport {
    /// Number of circular references detected
    pub count: usize,
    /// Total estimated leaked memory
    pub total_leaked_memory: usize,
    /// Number of smart pointers involved in cycles
    pub pointers_in_cycles: usize,
    /// Total number of smart pointers analyzed
    pub total_smart_pointers: usize,
    /// Whether any circular references were detected
    pub has_cycles: bool,
}

// ============================================================
// New types for Kinetic Engineering professional template features
// ============================================================

/// FFI call mapping node (for Call Mapping Topology visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiCallNode {
    /// Symbol or function name
    pub name: String,
    /// Memory address
    pub address: Option<String>,
    /// Node type: root, bridge, target, source
    pub node_type: String,
    /// Status: active, idle, hot
    pub status: String,
}

/// FFI call mapping edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiCallEdge {
    /// Source node index
    pub source: usize,
    /// Target node index
    pub target: usize,
    /// Resolved source node name (for table display)
    #[serde(default)]
    pub from_name: String,
    /// Resolved target node name (for table display)
    #[serde(default)]
    pub to_name: String,
    /// Human-readable label describing the crossing
    #[serde(default)]
    pub label: String,
}

/// Symbol table entry with status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTableEntry {
    /// Hexadecimal address
    pub hex_addr: String,
    /// Symbol/function name
    pub symbol_name: String,
    /// Status: PINNED, HOT, IDLE
    pub status: String,
    /// Total call count
    pub call_count: Option<u64>,
    /// Average time per call (microseconds)
    pub time_avg_us: Option<f64>,
    /// Whether the symbol is hot (high risk) — drives row color in template
    #[serde(default)]
    pub is_hot: bool,
}

/// Stack integrity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StackIntegrityMetrics {
    /// Percentage of pointers checked
    pub pointers_checked_pct: Option<f64>,
    /// Number of memory violations
    pub memory_violations: Option<usize>,
    /// Unwinding strategy: PANIC_ABORT, UNWIND, etc.
    pub unwinding_strategy: String,
}

/// Resource usage bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageBar {
    /// Label (e.g., "BRIDGE_POOL_ALLOC")
    pub label: String,
    /// Usage percentage (0-100)
    pub pct: f64,
    /// Color class: primary, secondary, error
    pub color_class: String,
}

/// Thread execution timeline segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSegment {
    /// Start percentage (0-100)
    pub start_pct: f64,
    /// Width percentage (0-100)
    pub width_pct: f64,
    /// Color class
    pub color: String,
}

/// Thread execution timeline row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadTimelineRow {
    /// Thread name
    pub thread_name: String,
    /// Segments representing execution phases
    pub segments: Vec<TimelineSegment>,
}

/// Waker efficiency grid cell value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakerEfficiencyGrid(pub Vec<f64>); // opacity values 0.0-1.0

/// Task topology node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTopologyNode {
    /// Task ID (hex)
    pub task_id: String,
    /// Task name
    pub name: String,
    /// Parent task ID (none if root)
    pub parent_id: Option<String>,
    /// Status: RUNNING, IDLE, WAITING, POLLING, COMPLETED
    pub status: String,
    /// Duration in ms
    pub duration_ms: Option<f64>,
    /// X position percentage for layout
    pub x_pct: f64,
    /// Y position percentage for layout
    pub y_pct: f64,
}

/// Task topology edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTopologyEdge {
    /// Source task ID
    pub source: String,
    /// Target task ID
    pub target: String,
    /// Whether this edge is active (animated)
    pub is_active: Option<bool>,
}

/// Streaming topology stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamingTopologyStats {
    /// Number of graph edges
    pub graph_edges: usize,
    /// Sampling rate in ms
    pub sampling_rate_ms: Option<u64>,
    /// Waker locks status: NONE, LOW, HIGH
    pub waker_locks_status: String,
}

/// Trace log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLogEntry {
    /// Numeric ordering key; formatted clock text is display-only.
    pub timestamp_ns: u64,
    /// Timestamp string
    pub timestamp: String,
    /// Log level: INFO, WARN, ERR, TASK
    pub level: String,
    /// Log message
    pub message: String,
}

/// Neighbor density histogram bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborDensityBin {
    /// Count of neighbors in this bin
    pub count: usize,
    /// Range label (e.g., "500ms")
    pub range_label: String,
}

/// Dependency graph peripheral node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Node identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Position: upstream_left, upstream_center, upstream_right, downstream_left, downstream_right
    pub position: String,
    /// Status tag (IDLE, SYNC, HOT, etc.)
    pub status: Option<String>,
    /// Opacity multiplier (0.0-1.0)
    pub opacity: f64,
}

/// Selected node detail panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetailPanel {
    /// Node name/identifier
    pub node_name: String,
    /// Status badge (HOT, ACTIVE, IDLE, etc.)
    pub status_badge: String,
    /// UUID
    pub uuid: String,
    /// Current status text
    pub current_status: String,
    /// Execution time in ms
    pub execution_time_ms: u64,
    /// Upstream dependency count
    pub upstream_deps: usize,
    /// Execution trace entries
    pub exec_trace: Vec<String>,
}

/// Hardware thread allocation pip status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAffinityPip {
    /// Status: PROCESSING, IO_WAIT, IDLE, OVERLOAD
    pub status: String,
}

/// Scheduler lag mini bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerLagBar {
    /// Height percentage
    pub height_pct: f64,
}

/// Thread event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadEventLogEntry {
    /// Numeric ordering key; formatted clock text is display-only.
    pub timestamp_ns: u64,
    /// Timestamp
    pub time: String,
    /// Level: INFO, WARN, TASK, ERR
    pub level: String,
    /// Main message
    pub message: String,
    /// Optional stack traces
    pub stack_traces: Vec<String>,
}

/// Thread policy setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPolicy {
    /// Policy name
    pub name: String,
    /// Enabled status
    pub enabled: Option<bool>,
}

/// Thread affinity grid data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAffinityGrid {
    /// Grid dimensions: rows x cols
    pub rows: usize,
    pub cols: usize,
    /// PIP status matrix (flattened)
    pub pips: Vec<ThreadAffinityPip>,
}

/// FFI call mapping topology
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FfiCallTopology {
    /// Nodes in the topology
    pub nodes: Vec<FfiCallNode>,
    /// Edges between nodes
    pub edges: Vec<FfiCallEdge>,
}

/// Completed lifetime and censored age populations must never share a histogram.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationLifetimeDistributions {
    pub capture_at_ns: u64,
    pub released: Vec<NeighborDensityBin>,
    pub live: Vec<NeighborDensityBin>,
    /// Observations with inverted or post-cut timestamps are excluded explicitly.
    pub invalid_timestamps: usize,
}
