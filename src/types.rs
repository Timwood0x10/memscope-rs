//! Core types and error handling for the memscope-rs library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for memory tracking operations
#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    /// Failed to acquire a lock
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Invalid pointer for association
    #[error("Invalid pointer association: {ptr:?}")]
    InvalidPointer {
        /// The invalid pointer address
        ptr: usize,
    },

    /// Allocation tracking is disabled
    #[error("Allocation tracking disabled")]
    TrackingDisabled,

    /// Memory corruption detected
    #[error("Memory corruption detected")]
    MemoryCorruption,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error during export
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for tracking operations
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Comprehensive report structure for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    /// Report metadata including generation time and version
    pub metadata: ReportMetadata,
    /// Memory usage overview and statistics
    pub memory_overview: MemoryOverview,
    /// Scope-based analysis of memory usage
    pub scope_analysis: ScopeAnalysis,
    /// Variable lifecycle tracking information
    pub variable_tracking: VariableTracking,
    /// Type-based memory analysis
    pub type_analysis: TypeAnalysis,
    /// Performance metrics and benchmarks
    pub performance_metrics: PerformanceMetrics,
    /// Timeline data for memory events
    pub timeline_data: TimelineData,
    /// Safety analysis and violation detection
    pub safety_analysis: SafetyAnalysis,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Timestamp when the report was generated
    pub generated_at: String,
    /// Version of the memscope-rs library
    pub version: String,
    /// Source of the memory data
    pub source: String,
    /// Total runtime in milliseconds
    pub total_runtime_ms: u128,
    /// Format version of the report
    pub format_version: String,
}

/// Memory overview statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOverview {
    /// Total number of allocations performed
    pub total_allocations: usize,
    /// Total number of deallocations performed
    pub total_deallocations: usize,
    /// Current number of active allocations
    pub active_allocations: usize,
    /// Peak number of concurrent allocations
    pub peak_allocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current active memory in bytes
    pub active_memory: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Memory efficiency as a percentage
    pub memory_efficiency: f64,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: f64,
}

/// Scope analysis data
/// Scope-based memory analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeAnalysis {
    /// List of all scopes and their memory usage
    pub scopes: Vec<ScopeInfo>,
    /// Hierarchical structure of scopes
    pub scope_hierarchy: ScopeHierarchy,
    /// References that cross scope boundaries
    pub cross_scope_references: Vec<CrossScopeReference>,
}

/// Individual scope information
/// Information about a specific scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    /// Name of the scope
    pub name: String,
    /// Nesting depth of the scope
    pub depth: usize,
    /// When the scope was entered
    pub start_time: u128,
    /// When the scope was exited (if applicable)
    pub end_time: Option<u128>,
    /// Current memory usage in this scope
    pub memory_usage: usize,
    /// Peak memory usage in this scope
    pub peak_memory: usize,
    /// Variables defined in this scope
    pub variables: Vec<String>,
    /// Child scopes nested within this scope
    pub child_scopes: Vec<String>,
    /// Parent scope containing this scope
    pub parent_scope: Option<String>,
    /// Number of allocations in this scope
    pub allocation_count: usize,
}

/// Scope hierarchy structure
/// Hierarchical structure of scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeHierarchy {
    /// Top-level scopes with no parent
    pub root_scopes: Vec<String>,
    /// Parent-child relationships between scopes
    pub relationships: HashMap<String, Vec<String>>,
    /// Depth level of each scope
    pub depth_map: HashMap<String, usize>,
}

/// Cross-scope reference tracking
/// Reference that crosses scope boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossScopeReference {
    /// Source scope of the reference
    pub from_scope: String,
    /// Target scope of the reference
    pub to_scope: String,
    /// Name of the referenced variable
    pub variable_name: String,
    /// Type of the cross-scope reference
    pub reference_type: ReferenceType,
    /// When the reference was created
    pub timestamp: u128,
}

/// Variable tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTracking {
    /// List of all tracked variables and their lifecycles
    pub variables: Vec<VariableLifecycle>,
    /// Record of ownership transfers between variables
    pub ownership_transfers: Vec<OwnershipTransfer>,
    /// Record of borrow operations
    pub borrow_events: Vec<BorrowEvent>,
}

/// Detailed variable lifecycle information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableLifecycle {
    /// Name of the variable
    pub name: String,
    /// Type name of the variable
    pub type_name: String,
    /// Scope where the variable is defined
    pub scope: String,
    /// When the variable was created
    pub birth_time: u128,
    /// When the variable was destroyed (if applicable)
    pub death_time: Option<u128>,
    /// Peak memory usage of this variable
    pub peak_memory: usize,
    /// Current memory usage of this variable
    pub current_memory: usize,
    /// Events where the variable grew in size
    pub growth_events: Vec<GrowthEvent>,
    /// Events where the variable was borrowed
    pub borrow_events: Vec<BorrowEvent>,
    /// Events where the variable was moved
    pub move_events: Vec<MoveEvent>,
    /// Relationships with other variables
    pub relationships: Vec<VariableRelationship>,
    /// Additional metadata tags
    pub metadata_tags: Vec<String>,
}

/// Type analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAnalysis {
    /// Hierarchical structure of types
    pub type_hierarchy: TypeHierarchy,
    /// Analysis of generic types and parameters
    pub generic_analysis: GenericAnalysis,
    /// Analysis of trait usage patterns
    pub trait_usage: TraitUsage,
    /// Memory layout and alignment analysis
    pub memory_layout: MemoryLayout,
    /// Statistical data for each type
    pub type_statistics: HashMap<String, TypeStatistics>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Rate of allocations per second
    pub allocation_rate: f64,
    /// Rate of deallocations per second
    pub deallocation_rate: f64,
    /// Average size of allocations
    pub average_allocation_size: f64,
    /// Median allocation size
    pub median_allocation_size: usize,
    /// 95th percentile allocation size
    pub p95_allocation_size: usize,
    /// Average allocation time in nanoseconds
    pub allocation_time_avg_ns: u128,
    /// Maximum allocation time in nanoseconds
    pub allocation_time_max_ns: u128,
    /// Memory throughput in MB/s
    pub memory_throughput_mb_s: f64,
    /// Garbage collection pressure
    pub gc_pressure: f64,
}

/// Timeline data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    /// Snapshots of memory state over time
    pub memory_snapshots: Vec<MemorySnapshot>,
    /// Individual allocation events
    pub allocation_events: Vec<AllocationEvent>,
    /// Scope entry and exit events
    pub scope_events: Vec<ScopeEvent>,
    /// Time range covered by the timeline
    pub time_range: TimeRange,
    /// Stack trace information for allocations
    pub stack_traces: StackTraceData,
    /// Performance hotspots over time
    pub allocation_hotspots: Vec<AllocationHotspot>,
}

/// Safety analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAnalysis {
    /// Number of unsafe operations detected
    pub unsafe_operations: usize,
    /// Number of FFI calls made
    pub ffi_calls: usize,
    /// List of potential memory leaks
    pub potential_leaks: Vec<PotentialLeak>,
    /// List of safety violations detected
    pub safety_violations: Vec<SafetyViolation>,
    /// Overall risk score (0.0 to 10.0)
    pub risk_score: f64,
}

/// Enhanced information about a memory allocation with lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address of the allocation
    pub ptr: usize,
    /// Size of the allocation in bytes
    pub size: usize,
    /// Timestamp when the allocation occurred (milliseconds since UNIX_EPOCH)
    pub timestamp_alloc: u128,
    /// Timestamp when the deallocation occurred (if applicable)
    pub timestamp_dealloc: Option<u128>,
    /// Optional name of the variable associated with this allocation
    pub var_name: Option<String>,
    /// Optional type name of the variable associated with this allocation
    pub type_name: Option<String>,
    /// Thread ID where the allocation occurred
    pub thread_id: String,
    /// Backtrace information (if available)
    #[cfg(feature = "backtrace")]
    pub backtrace: Option<Vec<String>>,

    // Enhanced lifecycle tracking fields
    /// Peak memory size reached during lifetime (for growable types)
    pub peak_size: Option<usize>,
    /// Number of memory growth events (reallocations)
    pub growth_events: usize,
    /// Scope identifier where this allocation occurred
    pub scope_name: Option<String>,
    /// Ownership pattern for this allocation
    pub ownership_pattern: Option<OwnershipPattern>,
    /// Risk classification for this allocation
    pub risk_level: Option<RiskLevel>,
    /// Memory efficiency score (useful_bytes / allocated_bytes)
    pub efficiency_score: Option<f64>,
    /// Borrowing events count (how many times this was borrowed)
    pub borrow_count: usize,
    /// Mutable borrowing events count
    pub mut_borrow_count: usize,
    /// Ownership transfer events
    pub transfer_count: usize,
    /// Custom metadata tags
    pub metadata_tags: Vec<String>,
}

// Supporting types for the comprehensive report
/// Type of reference between scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Immutable borrow
    Borrow,
    /// Mutable borrow
    MutableBorrow,
    /// Ownership move
    Move,
    /// Value clone
    Clone,
}

/// Ownership transfer between variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipTransfer {
    /// Source variable transferring ownership
    pub from_variable: String,
    /// Target variable receiving ownership
    pub to_variable: String,
    /// When the transfer occurred
    pub timestamp: u128,
    /// Type of ownership transfer
    pub transfer_type: TransferType,
}

/// Type of ownership transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    /// Ownership moved
    Move,
    /// Value cloned
    Clone,
    /// Reference created
    Reference,
}

/// Borrow operation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowEvent {
    /// Variable that borrows
    pub borrower: String,
    /// Variable being borrowed from
    pub borrowed_from: String,
    /// Type of borrow (mutable/immutable)
    pub borrow_type: BorrowType,
    /// When the borrow occurred
    pub timestamp: u128,
    /// Duration of the borrow
    pub duration: Option<u128>,
}

/// Type of borrow operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorrowType {
    /// Immutable borrow (&T)
    Immutable,
    /// Mutable borrow (&mut T)
    Mutable,
}

/// Memory growth event for a variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthEvent {
    /// When the growth occurred
    pub timestamp: u128,
    /// Previous size before growth
    pub old_size: usize,
    /// New size after growth
    pub new_size: usize,
    /// Reason for the growth
    pub growth_reason: GrowthReason,
}

/// Reason for memory growth or change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthReason {
    /// Initial allocation
    Initial,
    /// Memory reallocation
    Reallocation,
    /// Memory expansion
    Expansion,
    /// Memory shrinkage
    Shrinkage,
}

/// Variable move event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveEvent {
    /// When the move occurred
    pub timestamp: u128,
    /// Source scope of the move
    pub from_scope: String,
    /// Target scope of the move
    pub to_scope: String,
    /// Type of move operation
    pub move_type: MoveType,
}

/// Type of move operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveType {
    /// Ownership transfer
    Ownership,
    /// Reference move
    Reference,
    /// Copy operation
    Copy,
}

/// Relationship between variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRelationship {
    /// Name of the related variable
    pub related_variable: String,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Strength of the relationship (0.0 to 1.0)
    pub strength: f64,
}

/// Type of relationship between variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// One variable contains another
    Contains,
    /// One variable references another
    References,
    /// One variable borrows from another
    Borrows,
    /// One variable is cloned from another
    Cloned,
}

/// Type hierarchy and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeHierarchy {
    /// Categories of types and their information
    pub categories: HashMap<String, TypeCategory>,
    /// Inheritance relationships between types
    pub inheritance_tree: Vec<TypeRelation>,
    /// Composition relationships between types
    pub composition_graph: Vec<CompositionRelation>,
}

/// Category of types with memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCategory {
    /// Name of the type category
    pub name: String,
    /// List of subcategories
    pub subcategories: Vec<String>,
    /// Total memory used by this category
    pub total_memory: usize,
    /// Number of allocations in this category
    pub allocation_count: usize,
}

/// Relationship between types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRelation {
    /// Parent type in the relationship
    pub parent_type: String,
    /// Child type in the relationship
    pub child_type: String,
    /// Strength of the relationship
    pub relation_strength: f64,
}

/// Composition relationship between types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionRelation {
    /// Type that contains other types
    pub container_type: String,
    /// Type that is contained
    pub contained_type: String,
    /// Number of contained instances
    pub count: usize,
}

/// Analysis of generic types and parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericAnalysis {
    /// Information about generic types
    pub generic_types: Vec<GenericTypeInfo>,
    /// Type parameters for each generic type
    pub type_parameters: HashMap<String, Vec<String>>,
    /// Common instantiation patterns
    pub instantiation_patterns: Vec<InstantiationPattern>,
}

/// Information about a generic type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeInfo {
    /// Base type name (e.g., "Vec" for Vec<T>)
    pub base_type: String,
    /// Type parameters used
    pub type_parameters: Vec<String>,
    /// Number of times this type was instantiated
    pub instantiation_count: usize,
    /// Total memory usage of all instances
    pub memory_usage: usize,
}

/// Pattern of type instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiationPattern {
    /// The instantiation pattern (e.g., "Vec<String>")
    pub pattern: String,
    /// How often this pattern occurs
    pub frequency: usize,
    /// Memory impact of this pattern
    pub memory_impact: usize,
}

/// Analysis of trait usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitUsage {
    /// Traits implemented by each type
    pub implemented_traits: HashMap<String, Vec<String>>,
    /// Information about trait objects
    pub trait_objects: Vec<TraitObjectInfo>,
    /// Number of dynamic dispatch calls
    pub dynamic_dispatch_count: usize,
}

/// Information about trait objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitObjectInfo {
    /// Name of the trait
    pub trait_name: String,
    /// Types that implement this trait
    pub implementing_types: Vec<String>,
    /// Number of times used as trait object
    pub usage_count: usize,
}

/// Memory layout and alignment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayout {
    /// Analysis of type alignment
    pub alignment_analysis: AlignmentAnalysis,
    /// Bytes wasted due to padding
    pub padding_waste: usize,
    /// Cache efficiency score
    pub cache_efficiency: f64,
}

/// Analysis of memory alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentAnalysis {
    /// Types with good alignment
    pub well_aligned_types: Vec<String>,
    /// Types with poor alignment
    pub poorly_aligned_types: Vec<String>,
    /// Total bytes wasted due to alignment
    pub alignment_waste: usize,
}

/// Statistical information about a type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatistics {
    /// Total number of instances
    pub total_instances: usize,
    /// Total memory used by all instances
    pub total_memory: usize,
    /// Average size per instance
    pub average_size: f64,
    /// Distribution of instance sizes
    pub size_distribution: SizeDistribution,
}

/// Distribution of allocation sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    /// Minimum allocation size
    pub min_size: usize,
    /// Maximum allocation size
    pub max_size: usize,
    /// Median allocation size
    pub median_size: usize,
    /// Size percentiles (e.g., "P95" -> size)
    pub percentiles: HashMap<String, usize>,
}

/// Snapshot of memory state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// When the snapshot was taken
    pub timestamp: u128,
    /// Total memory in use
    pub total_memory: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Memory breakdown by scope
    pub scope_breakdown: HashMap<String, usize>,
}

/// Individual allocation or deallocation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationEvent {
    /// When the event occurred
    pub timestamp: u128,
    /// Type of allocation event
    pub event_type: AllocationEventType,
    /// Name of the variable involved
    pub variable_name: String,
    /// Size of the allocation
    pub size: usize,
    /// Scope where the event occurred
    pub scope: String,
    /// Stack trace ID for this allocation
    pub stack_trace_id: Option<String>,
    /// Type name of the allocated variable
    pub type_name: Option<String>,
    /// Thread ID where allocation occurred
    pub thread_id: String,
}

/// Type of allocation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationEventType {
    /// Memory allocation event
    Allocate,
    /// Memory deallocation event
    Deallocate,
    /// Memory reallocation event
    Reallocate,
}

/// Event that occurs within a scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeEvent {
    /// When the event occurred
    pub timestamp: u128,
    /// Type of scope event
    pub event_type: ScopeEventType,
    /// Name of the scope where event occurred
    pub scope_name: String,
    /// Memory impact of the event in bytes
    pub memory_impact: usize,
}

/// Type of scope event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeEventType {
    /// Entering a scope
    Enter,
    /// Exiting a scope
    Exit,
}

/// Time range for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time of the range
    pub start_time: u128,
    /// End time of the range
    pub end_time: u128,
    /// Duration in milliseconds
    pub duration_ms: u128,
}

/// Potential memory leak detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialLeak {
    /// Name of the potentially leaked variable
    pub variable_name: String,
    /// Size of the leaked memory in bytes
    pub size: usize,
    /// Age of the allocation in milliseconds
    pub age_ms: u128,
    /// Scope where the leak was detected
    pub scope: String,
    /// Confidence level of leak detection (0.0 to 1.0)
    pub confidence: f64,
    /// Stack trace where the leak was allocated
    pub allocation_stack: Option<Vec<StackFrame>>,
    /// Last access time (if available)
    pub last_access_time: Option<u128>,
}

/// Stack trace data for allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTraceData {
    /// Map of stack trace ID to actual stack frames
    pub traces: HashMap<String, Vec<StackFrame>>,
    /// Allocation hotspots by stack trace
    pub hotspots: Vec<StackTraceHotspot>,
    /// Most common allocation patterns
    pub common_patterns: Vec<AllocationPattern>,
}

/// Individual stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name
    pub function: String,
    /// File name
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Module path
    pub module: Option<String>,
}

/// Stack trace hotspot analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTraceHotspot {
    /// Stack trace pattern
    pub stack_pattern: Vec<StackFrame>,
    /// Number of allocations from this stack
    pub allocation_count: usize,
    /// Total memory allocated from this stack
    pub total_memory: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Frequency per second
    pub frequency_per_second: f64,
}

/// Common allocation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    /// Pattern description
    pub pattern: String,
    /// How often this pattern occurs
    pub frequency: usize,
    /// Total memory impact
    pub total_memory_impact: usize,
    /// Example stack traces
    pub example_stacks: Vec<Vec<StackFrame>>,
}

/// Allocation hotspot over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationHotspot {
    /// Time window start
    pub timestamp: u128,
    /// Location information
    pub location: HotspotLocation,
    /// Number of allocations in this time window
    pub allocation_count: usize,
    /// Total memory allocated in this window
    pub total_memory: usize,
    /// Average allocation rate (allocations per second)
    pub allocation_rate: f64,
    /// Memory pressure score (0.0 to 1.0)
    pub memory_pressure: f64,
}

/// Location information for hotspots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotLocation {
    /// Function name
    pub function: String,
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Scope name
    pub scope: String,
}

/// Memory safety violation detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyViolation {
    /// Type of safety violation
    pub violation_type: ViolationType,
    /// Description of the violation
    pub description: String,
    /// Location where violation occurred
    pub location: String,
    /// Severity level of the violation
    pub severity: Severity,
    /// When the violation was detected
    pub timestamp: u128,
}

/// Type of memory safety violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// Double free violation
    DoubleFree,
    /// Use after free violation
    UseAfterFree,
    /// Buffer overflow violation
    BufferOverflow,
    /// Null pointer dereference
    NullPointerDereference,
    /// Data race condition
    DataRace,
}

/// Severity level of violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity issue
    Low,
    /// Medium severity issue
    Medium,
    /// High severity issue
    High,
    /// Critical severity issue
    Critical,
}

impl AllocationInfo {
    /// Create a new allocation info with enhanced lifecycle tracking
    pub fn new(ptr: usize, size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let thread_id = format!("{:?}", std::thread::current().id());

        Self {
            ptr,
            size,
            timestamp_alloc: timestamp,
            timestamp_dealloc: None,
            var_name: None,
            type_name: None,
            thread_id,
            #[cfg(feature = "backtrace")]
            backtrace: None,

            // Initialize enhanced lifecycle fields
            peak_size: Some(size), // Initially same as size
            growth_events: 0,
            scope_name: None,
            ownership_pattern: None,
            risk_level: None,
            efficiency_score: Some(1.0), // Initially 100% efficient
            borrow_count: 0,
            mut_borrow_count: 0,
            transfer_count: 0,
            metadata_tags: Vec::new(),
        }
    }

    /// Mark this allocation as deallocated
    pub fn mark_deallocated(&mut self) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        self.timestamp_dealloc = Some(timestamp);
    }

    /// Check if this allocation is still active
    pub fn is_active(&self) -> bool {
        self.timestamp_dealloc.is_none()
    }

    /// Get the lifetime of this allocation in milliseconds
    pub fn lifetime_ms(&self) -> Option<u128> {
        self.timestamp_dealloc
            .map(|dealloc| dealloc - self.timestamp_alloc)
    }

    /// Record a memory growth event (reallocation)
    pub fn record_growth(&mut self, new_size: usize) {
        self.growth_events += 1;
        if let Some(peak) = self.peak_size {
            self.peak_size = Some(peak.max(new_size));
        } else {
            self.peak_size = Some(new_size);
        }

        // Update efficiency score
        if let Some(peak) = self.peak_size {
            self.efficiency_score = Some(self.size as f64 / peak as f64);
        }
    }

    /// Record a borrowing event
    pub fn record_borrow(&mut self, is_mutable: bool) {
        if is_mutable {
            self.mut_borrow_count += 1;
        } else {
            self.borrow_count += 1;
        }
    }

    /// Record an ownership transfer
    pub fn record_transfer(&mut self) {
        self.transfer_count += 1;
    }

    /// Add a metadata tag
    pub fn add_metadata_tag(&mut self, tag: String) {
        if !self.metadata_tags.contains(&tag) {
            self.metadata_tags.push(tag);
        }
    }

    /// Calculate memory growth factor
    pub fn memory_growth_factor(&self) -> f64 {
        if let Some(peak) = self.peak_size {
            peak as f64 / self.size.max(1) as f64
        } else {
            1.0
        }
    }

    /// Classify the risk level of this allocation
    pub fn classify_risk(&mut self) {
        let growth_factor = self.memory_growth_factor();
        let lifetime = self.lifetime_ms().unwrap_or(0) as f64;

        self.risk_level = Some(if self.size > 1024 * 1024 || growth_factor > 3.0 {
            RiskLevel::Critical
        } else if self.size > 1024 || growth_factor > 2.0 || lifetime > 10000.0 {
            RiskLevel::High
        } else if self.size > 256 || growth_factor > 1.5 || lifetime > 1000.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        });
    }

    /// Determine ownership pattern based on type
    pub fn determine_ownership_pattern(&mut self) {
        if let Some(type_name) = &self.type_name {
            self.ownership_pattern =
                Some(if type_name.contains("Rc") || type_name.contains("Arc") {
                    OwnershipPattern::Shared
                } else if type_name.starts_with('&') {
                    OwnershipPattern::Borrowed
                } else if self.transfer_count > 0 && self.borrow_count > 0 {
                    OwnershipPattern::Mixed
                } else {
                    OwnershipPattern::Owned
                });
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of allocations tracked
    pub total_allocations: usize,
    /// Total number of deallocations tracked
    pub total_deallocations: usize,
    /// Total bytes allocated
    pub total_allocated: usize,
    /// Total bytes deallocated
    pub total_deallocated: usize,
    /// Current number of active allocations
    pub active_allocations: usize,
    /// Current bytes in active allocations
    pub active_memory: usize,
    /// Peak number of active allocations
    pub peak_allocations: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Lifecycle statistics
    pub lifecycle_stats: LifecycleStats,
    /// List of all allocations for detailed analysis
    pub allocations: Vec<AllocationInfo>,
    /// Memory fragmentation analysis
    pub fragmentation_analysis: FragmentationAnalysis,
    /// System library usage statistics
    pub system_library_stats: SystemLibraryStats,
    /// Concurrency safety analysis
    pub concurrency_analysis: ConcurrencyAnalysis,
}

/// Memory usage by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMemoryUsage {
    /// The name of the data type
    pub type_name: String,
    /// Total size in bytes for this type
    pub total_size: usize,
    /// Number of allocations for this type
    pub allocation_count: usize,
}

/// Allocation hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotInfo {
    /// Location identifier (could be function name, file:line, etc.)
    pub location: String,
    /// Number of allocations from this location
    pub count: usize,
    /// Total size of allocations from this location
    pub total_size: usize,
    /// Average allocation size
    pub average_size: f64,
}

/// Enhanced lifecycle statistics for memory allocations per lifecycle.md specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleStats {
    /// Number of completed allocations (with deallocation timestamps)
    pub completed_allocations: usize,
    /// Average lifetime in milliseconds
    pub average_lifetime_ms: f64,
    /// Median lifetime in milliseconds
    pub median_lifetime_ms: f64,
    /// Lifecycle percentiles
    pub lifetime_percentiles: LifecyclePercentiles,
    /// Shortest lifetime in milliseconds
    pub min_lifetime_ms: u128,
    /// Longest lifetime in milliseconds
    pub max_lifetime_ms: u128,
    /// Number of instant allocations (< 1ms)
    pub instant_allocations: usize,
    /// Number of short-term allocations (1ms - 100ms)
    pub short_term_allocations: usize,
    /// Number of medium-term allocations (100ms - 1s)
    pub medium_term_allocations: usize,
    /// Number of long-term allocations (> 1s)
    pub long_term_allocations: usize,
    /// Number of suspected memory leaks (active > 10s)
    pub suspected_leaks: usize,

    // Enhanced metrics per lifecycle.md
    /// Memory growth events (reallocations, expansions)
    pub memory_growth_events: usize,
    /// Peak concurrent variables at any point in time
    pub peak_concurrent_variables: usize,
    /// Memory efficiency ratio (useful_memory / total_allocated)
    pub memory_efficiency_ratio: f64,
    /// Ownership transfer events detected
    pub ownership_transfer_events: usize,
    /// Borrowing relationship violations
    pub borrowing_violations: usize,
    /// Memory fragmentation score (0.0 = perfect, 1.0 = highly fragmented)
    pub fragmentation_score: f64,
    /// Risk classification distribution
    pub risk_distribution: RiskDistribution,
    /// Scope-based lifecycle metrics
    pub scope_metrics: Vec<ScopeLifecycleMetrics>,
    /// Type-specific lifecycle patterns
    pub type_lifecycle_patterns: Vec<TypeLifecyclePattern>,
}

/// Lifecycle percentile statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecyclePercentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Lifecycle statistics by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecycleStats {
    /// Type name
    pub type_name: String,
    /// Average lifetime for this type
    pub average_lifetime_ms: f64,
    /// Number of allocations for this type
    pub allocation_count: usize,
    /// Lifecycle category
    pub category: LifecycleCategory,
}

/// Categories for lifecycle duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleCategory {
    /// Very short-lived (< 1ms)
    Instant,
    /// Short-lived (1ms - 100ms)
    ShortTerm,
    /// Medium-lived (100ms - 1s)
    MediumTerm,
    /// Long-lived (> 1s)
    LongTerm,
}

/// Risk classification distribution for memory allocations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskDistribution {
    /// High memory risk allocations (large size or high growth)
    pub high_memory_risk: usize,
    /// Potential growth risk allocations
    pub potential_growth_risk: usize,
    /// Short lifecycle risk allocations
    pub short_lifecycle_risk: usize,
    /// Low risk allocations
    pub low_risk: usize,
    /// Memory leak risk allocations (long-lived without deallocation)
    pub leak_risk: usize,
}

/// Memory fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FragmentationAnalysis {
    /// Total number of memory fragments
    pub total_fragments: usize,
    /// Largest available contiguous block size
    pub largest_free_block: usize,
    /// Smallest allocation size
    pub smallest_allocation: usize,
    /// Fragmentation ratio (0.0 = no fragmentation, 1.0 = highly fragmented)
    pub fragmentation_ratio: f64,
    /// List of memory holes
    pub memory_holes: Vec<MemoryHole>,
    /// Size distribution statistics
    pub size_distribution: HashMap<String, usize>,
    /// Bytes wasted due to alignment
    pub alignment_waste: usize,
    /// External fragmentation degree
    pub external_fragmentation: f64,
    /// Internal fragmentation degree
    pub internal_fragmentation: f64,
}

/// Memory hole information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHole {
    /// Hole start address
    pub start_address: usize,
    /// Hole size in bytes
    pub size: usize,
    /// Hole duration in milliseconds
    pub duration_ms: u64,
    /// Type of hole: "gap", "alignment_padding", "freed_space"
    pub hole_type: String,
    /// Cause of the hole
    pub cause: String,
}

/// System library usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemLibraryStats {
    /// Standard library collections
    pub std_collections: LibraryUsage,
    /// Async runtime (tokio, async-std)
    pub async_runtime: LibraryUsage,
    /// Network I/O libraries
    pub network_io: LibraryUsage,
    /// File system operations
    pub file_system: LibraryUsage,
    /// Serialization/deserialization
    pub serialization: LibraryUsage,
    /// Regular expression engine
    pub regex_engine: LibraryUsage,
    /// Cryptography and security
    pub crypto_security: LibraryUsage,
    /// Database libraries
    pub database: LibraryUsage,
    /// Graphics and UI libraries
    pub graphics_ui: LibraryUsage,
    /// HTTP client/server stack
    pub http_stack: LibraryUsage,
    /// Compression and encoding
    pub compression: LibraryUsage,
    /// Logging systems
    pub logging: LibraryUsage,
    /// Unknown system allocations
    pub unknown_system: LibraryUsage,
}

/// Library usage details
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryUsage {
    /// Number of allocations
    pub allocation_count: usize,
    /// Total bytes allocated
    pub total_bytes: usize,
    /// Peak bytes at any time
    pub peak_bytes: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Detailed categories
    pub categories: HashMap<String, usize>,
    /// Hotspot functions
    pub hotspot_functions: Vec<String>,
}

/// Concurrency safety analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcurrencyAnalysis {
    /// Thread-safe allocations count
    pub thread_safety_allocations: usize,
    /// Shared memory bytes
    pub shared_memory_bytes: usize,
    /// Mutex-protected memory
    pub mutex_protected: usize,
    /// Arc-shared memory
    pub arc_shared: usize,
    /// Rc reference-counted memory
    pub rc_shared: usize,
    /// Channel buffer memory
    pub channel_buffers: usize,
    /// Thread-local storage
    pub thread_local_storage: usize,
    /// Atomic operations memory
    pub atomic_operations: usize,
    /// Lock contention risk: "low", "medium", "high", "critical"
    pub lock_contention_risk: String,
    /// Concurrency patterns detected
    pub concurrency_patterns: Vec<ConcurrencyPattern>,
    /// Data race risks
    pub data_race_risks: Vec<DataRaceRisk>,
    /// Deadlock risk score (0.0 to 1.0)
    pub deadlock_risk_score: f64,
}

/// Concurrency pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyPattern {
    /// Pattern type: "producer_consumer", "shared_state", "message_passing", "actor_model"
    pub pattern_type: String,
    /// Number of threads involved
    pub thread_count: usize,
    /// Memory usage for this pattern
    pub memory_usage: usize,
    /// Safety level: "safe", "unsafe", "mixed"
    pub safety_level: String,
    /// Performance impact: "low", "medium", "high"
    pub performance_impact: String,
    /// Detected locations
    pub locations: Vec<String>,
}

/// Data race risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRaceRisk {
    /// Risk type
    pub risk_type: String,
    /// Memory address involved
    pub memory_address: usize,
    /// Severity: "low", "medium", "high", "critical"
    pub severity: String,
    /// Risk description
    pub description: String,
    /// Suggested fix
    pub suggested_fix: String,
}

/// Scope-based lifecycle metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeLifecycleMetrics {
    /// Scope identifier (function name, block, etc.)
    pub scope_name: String,
    /// Number of variables in this scope
    pub variable_count: usize,
    /// Average lifetime of variables in this scope
    pub avg_lifetime_ms: f64,
    /// Total memory usage in this scope
    pub total_memory_bytes: usize,
    /// Peak concurrent variables in this scope
    pub peak_concurrent_vars: usize,
    /// Scope efficiency score (0.0 = poor, 1.0 = excellent)
    pub efficiency_score: f64,
}

/// Type-specific lifecycle patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeLifecyclePattern {
    /// Type name (String, Vec, Box, etc.)
    pub type_name: String,
    /// Average allocation count per variable of this type
    pub avg_allocations_per_var: f64,
    /// Memory growth factor (peak_size / initial_size)
    pub memory_growth_factor: f64,
    /// Typical lifetime range for this type
    pub typical_lifetime_range: (u64, u64), // (min_ms, max_ms)
    /// Ownership pattern (owned, borrowed, shared)
    pub ownership_pattern: OwnershipPattern,
    /// Risk level for this type
    pub risk_level: RiskLevel,
}

/// Ownership patterns for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipPattern {
    /// Exclusively owned (Box, Vec, String)
    Owned,
    /// Shared ownership (Rc, Arc)
    Shared,
    /// Borrowed references (&T, &mut T)
    Borrowed,
    /// Mixed ownership patterns
    Mixed,
}

/// Risk levels for memory allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - small, predictable allocations
    Low,
    /// Medium risk - moderate size or some growth
    Medium,
    /// High risk - large allocations or significant growth
    High,
    /// Critical risk - potential memory leaks or excessive growth
    Critical,
}
