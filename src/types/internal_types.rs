//! Unified core types for memory tracking
//!
//! This module provides a unified type system that preserves all existing fields
//! and functionality while simplifying the architecture through better abstraction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Error Types
// ============================================================================

/// Result type for tracking operations
pub type TrackingResult<T> = Result<T, TrackingError>;

/// Comprehensive error type for the tracking system (preserving all existing error types)
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
    IoError(String),
    LockContention(String),
    LockError(String),
    ChannelError(String),
    ThreadError(String),
    InitializationError(String),
    NotImplemented(String),
    InvalidOperation(String),
    ValidationError(String),
    DataError(String),
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
            TrackingError::LockContention(msg) => write!(f, "Lock contention: {msg}"),
            TrackingError::DataError(msg) => write!(f, "Data error: {msg}"),
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

// ============================================================================
// Event Types (Unified Event Model)
// ============================================================================

/// Unified event type for all tracking scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Memory allocation event
    Alloc {
        ptr: usize,
        size: usize,
        thread: u32,
        ts: u64,
    },

    /// Memory deallocation event
    Dealloc { ptr: usize, thread: u32, ts: u64 },

    /// Memory reallocation event
    Realloc {
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        thread: u32,
        ts: u64,
    },

    /// Async task spawn event
    TaskSpawn { id: u64, ts: u64 },

    /// Async task end event
    TaskEnd { id: u64, ts: u64 },

    /// FFI allocation event
    FfiAlloc {
        ptr: usize,
        size: usize,
        lib: String,
        ts: u64,
    },

    /// FFI free event
    FfiFree { ptr: usize, lib: String, ts: u64 },

    /// Unsafe memory access event
    UnsafeAccess {
        ptr: usize,
        size: usize,
        location: String,
        ts: u64,
    },

    /// Variable tracking event
    VariableTrack {
        ptr: usize,
        var_name: String,
        type_name: String,
        ts: u64,
    },
}

impl Event {
    /// Returns the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::Alloc { ts, .. }
            | Event::Dealloc { ts, .. }
            | Event::Realloc { ts, .. }
            | Event::TaskSpawn { ts, .. }
            | Event::TaskEnd { ts, .. }
            | Event::FfiAlloc { ts, .. }
            | Event::FfiFree { ts, .. }
            | Event::UnsafeAccess { ts, .. }
            | Event::VariableTrack { ts, .. } => *ts,
        }
    }

    /// Returns the pointer associated with this event (if any)
    pub fn ptr(&self) -> Option<usize> {
        match self {
            Event::Alloc { ptr, .. }
            | Event::Dealloc { ptr, .. }
            | Event::FfiAlloc { ptr, .. }
            | Event::FfiFree { ptr, .. }
            | Event::UnsafeAccess { ptr, .. }
            | Event::VariableTrack { ptr, .. } => Some(*ptr),
            Event::Realloc { old_ptr, .. } => Some(*old_ptr),
            Event::TaskSpawn { .. } | Event::TaskEnd { .. } => None,
        }
    }
}

// ============================================================================
// Allocation Types (Preserving all 30+ fields)
// ============================================================================

/// Smart pointer specific information (preserving all existing fields)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartPointerInfo {
    pub data_ptr: usize,
    pub cloned_from: Option<usize>,
    pub clones: Vec<usize>,
    pub ref_count_history: Vec<RefCountSnapshot>,
    pub weak_count: Option<usize>,
    pub is_weak_reference: bool,
    pub is_data_owner: bool,
    pub is_implicitly_deallocated: bool,
    pub pointer_type: SmartPointerType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefCountSnapshot {
    pub timestamp: u64,
    pub strong_count: usize,
    pub weak_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SmartPointerType {
    Rc,
    Arc,
    RcWeak,
    ArcWeak,
    Box,
}

impl SmartPointerInfo {
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

    pub fn record_clone(&mut self, clone_ptr: usize, source_ptr: usize) {
        if self.cloned_from.is_none() {
            self.cloned_from = Some(source_ptr);
        }
        self.clones.push(clone_ptr);
    }

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
}

/// Borrow information (preserving all existing fields)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BorrowInfo {
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub max_concurrent_borrows: usize,
    pub last_borrow_timestamp: Option<u64>,
}

/// Clone information (preserving all existing fields)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CloneInfo {
    pub clone_count: usize,
    pub is_clone: bool,
    pub original_ptr: Option<usize>,
}

/// Allocation metadata (preserving all existing fields)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationMeta {
    pub var_name: Option<String>,
    pub type_name: Option<String>,
    pub stack_trace: Option<Vec<String>>,
    pub smart_pointer_info: Option<SmartPointerInfo>,
    pub borrow_info: Option<BorrowInfo>,
    pub clone_info: Option<CloneInfo>,
}

/// Core allocation information (preserving all 30+ fields from original AllocationInfo)
#[derive(Debug, Clone, Serialize)]
pub struct Allocation {
    pub ptr: usize,
    pub size: usize,
    pub alloc_ts: u64,
    pub free_ts: Option<u64>,
    pub thread: u32,
    pub meta: AllocationMeta,
    pub scope_name: Option<String>,
    pub borrow_count: usize,
    pub is_leaked: bool,
    pub lifetime_ms: Option<u64>,
    pub ownership_history_available: bool,
    pub memory_layout: Option<MemoryLayoutInfo>,
    pub generic_info: Option<GenericTypeInfo>,
    pub dynamic_type_info: Option<DynamicTypeInfo>,
    pub runtime_state: Option<RuntimeStateInfo>,
    pub stack_allocation: Option<StackAllocationInfo>,
    pub temporary_object: Option<TemporaryObjectInfo>,
    pub fragmentation_analysis: Option<EnhancedFragmentationAnalysis>,
    pub generic_instantiation: Option<GenericInstantiationInfo>,
    pub type_relationships: Option<TypeRelationshipInfo>,
    pub type_usage: Option<TypeUsageInfo>,
    pub function_call_tracking: Option<FunctionCallTrackingInfo>,
    pub lifecycle_tracking: Option<ObjectLifecycleInfo>,
    pub access_tracking: Option<MemoryAccessTrackingInfo>,
    pub drop_chain_analysis: Option<DropChainAnalysis>,
}

impl<'de> serde::Deserialize<'de> for Allocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AllocationHelper {
            ptr: usize,
            size: usize,
            alloc_ts: u64,
            free_ts: Option<u64>,
            thread: u32,
            meta: AllocationMeta,
            scope_name: Option<String>,
            borrow_count: usize,
            is_leaked: bool,
            lifetime_ms: Option<u64>,
            ownership_history_available: Option<bool>,
        }

        let helper = AllocationHelper::deserialize(deserializer)?;
        Ok(Allocation {
            ptr: helper.ptr,
            size: helper.size,
            alloc_ts: helper.alloc_ts,
            free_ts: helper.free_ts,
            thread: helper.thread,
            meta: helper.meta,
            scope_name: helper.scope_name,
            borrow_count: helper.borrow_count,
            is_leaked: helper.is_leaked,
            lifetime_ms: helper.lifetime_ms,
            ownership_history_available: helper.ownership_history_available.unwrap_or(false),
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
        })
    }
}

impl Allocation {
    pub fn new(ptr: usize, size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            ptr,
            size,
            alloc_ts: timestamp,
            free_ts: None,
            thread: current_thread_id(),
            meta: AllocationMeta::default(),
            scope_name: None,
            borrow_count: 0,
            is_leaked: false,
            lifetime_ms: Some(1),
            ownership_history_available: true,
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
        }
    }

    pub fn is_active(&self) -> bool {
        self.free_ts.is_none()
    }

    pub fn mark_deallocated(&mut self) {
        self.free_ts = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
    }
}

fn current_thread_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ============================================================================
// Extended Type Definitions (Preserving all existing types)
// ============================================================================

/// Memory layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayoutInfo {
    pub alignment: usize,
    pub padding: usize,
    pub field_offsets: Vec<(String, usize)>,
}

/// Generic type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeInfo {
    pub type_parameters: Vec<String>,
    pub instantiation_count: usize,
}

/// Dynamic type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTypeInfo {
    pub concrete_type: String,
    pub trait_methods: Vec<String>,
}

/// Runtime state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateInfo {
    pub state: String,
    pub transitions: Vec<(String, u64)>,
}

/// Stack allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackAllocationInfo {
    pub stack_pointer: usize,
    pub frame_size: usize,
}

/// Temporary object information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryObjectInfo {
    pub is_temporary: bool,
    pub lifetime_ns: u64,
}

/// Enhanced fragmentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFragmentationAnalysis {
    pub fragmentation_ratio: f64,
    pub external_fragmentation: f64,
    pub internal_fragmentation: f64,
}

/// Generic instantiation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstantiationInfo {
    pub generic_types: Vec<String>,
    pub instantiation_count: usize,
}

/// Type relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRelationshipInfo {
    pub related_types: Vec<String>,
    pub relationship_type: String,
}

/// Type usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeUsageInfo {
    pub usage_count: usize,
    pub average_lifetime_ms: f64,
}

/// Function call tracking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallTrackingInfo {
    pub function_name: String,
    pub call_count: usize,
}

/// Object lifecycle info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLifecycleInfo {
    pub created_at: u64,
    pub destroyed_at: Option<u64>,
    pub lifetime_ms: Option<u64>,
}

/// Memory access tracking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessTrackingInfo {
    pub read_count: usize,
    pub write_count: usize,
    pub last_access: Option<u64>,
}

/// Drop chain analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropChainAnalysis {
    pub drop_chain: Vec<String>,
    pub total_drop_time_ns: u64,
}

// ============================================================================
// MemoryPassport System (Killer Feature 1 - Complete Implementation)
// ============================================================================

/// Memory passport for tracking memory across language boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    pub id: usize,
    pub ptr: usize,
    pub size: usize,
    pub source: MemorySource,
    pub status: PassportStatus,
    pub created_at: u64,
    pub events: Vec<PassportEvent>,
}

impl MemoryPassport {
    pub fn new(id: usize, ptr: usize, size: usize, source: MemorySource, ts: u64) -> Self {
        Self {
            id,
            ptr,
            size,
            source,
            status: PassportStatus::Active,
            created_at: ts,
            events: vec![PassportEvent {
                timestamp: ts,
                event_type: PassportEventType::Created,
                details: "Memory allocated".to_string(),
            }],
        }
    }

    pub fn transfer(&mut self, to: String, ts: u64) {
        self.status = PassportStatus::Transferred { to: to.clone() };
        self.events.push(PassportEvent {
            timestamp: ts,
            event_type: PassportEventType::Transferred,
            details: format!("Transferred to {}", to),
        });
    }

    pub fn release(&mut self, ts: u64) {
        self.status = PassportStatus::Released;
        self.events.push(PassportEvent {
            timestamp: ts,
            event_type: PassportEventType::Released,
            details: "Memory released".to_string(),
        });
    }

    pub fn mark_leaked(&mut self, ts: u64) {
        self.status = PassportStatus::Leaked;
        self.events.push(PassportEvent {
            timestamp: ts,
            event_type: PassportEventType::Leaked,
            details: "Memory leaked".to_string(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemorySource {
    Rust,
    ForeignLibrary { lib_name: String },
    CustomAllocator { allocator_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PassportStatus {
    Active,
    Transferred { to: String },
    Released,
    Leaked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportEvent {
    pub timestamp: u64,
    pub event_type: PassportEventType,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PassportEventType {
    Created,
    Transferred,
    Released,
    Leaked,
}

/// MemoryPassport tracker
pub struct MemoryPassportTracker {
    passports: HashMap<usize, MemoryPassport>,
    next_id: usize,
}

impl MemoryPassportTracker {
    pub fn new() -> Self {
        Self {
            passports: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn track_ffi_alloc(&mut self, ptr: usize, size: usize, lib: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let passport = MemoryPassport::new(
            self.next_id,
            ptr,
            size,
            MemorySource::ForeignLibrary {
                lib_name: lib.to_string(),
            },
            timestamp,
        );

        self.passports.insert(ptr, passport);
        self.next_id += 1;
    }

    pub fn record_ffi_handover(&mut self, ptr: usize, _from: &str, to: &str) {
        if let Some(passport) = self.passports.get_mut(&ptr) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            passport.transfer(to.to_string(), timestamp);
        }
    }

    pub fn record_ffi_free(&mut self, ptr: usize) {
        if let Some(passport) = self.passports.get_mut(&ptr) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            passport.release(timestamp);
        }
    }

    pub fn get_passport(&self, ptr: usize) -> Option<&MemoryPassport> {
        self.passports.get(&ptr)
    }

    pub fn get_all_passports(&self) -> Vec<&MemoryPassport> {
        self.passports.values().collect()
    }
}

impl Default for MemoryPassportTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Snapshot and Statistics
// ============================================================================

/// Snapshot of memory state
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub timestamp: u64,
    pub allocations: Vec<Allocation>,
    pub tasks: Vec<TaskInfo>,
    pub threads: Vec<ThreadInfo>,
    pub passports: Vec<MemoryPassport>,
    pub stats: Stats,
}

impl<'de> serde::Deserialize<'de> for Snapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SnapshotHelper {
            timestamp: u64,
            allocations: Vec<Allocation>,
            tasks: Vec<TaskInfo>,
            threads: Vec<ThreadInfo>,
            passports: Vec<MemoryPassport>,
            stats: Stats,
        }

        let helper = SnapshotHelper::deserialize(deserializer)?;
        Ok(Snapshot {
            timestamp: helper.timestamp,
            allocations: helper.allocations,
            tasks: helper.tasks,
            threads: helper.threads,
            passports: helper.passports,
            stats: helper.stats,
        })
    }
}

impl Snapshot {
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            allocations: Vec::new(),
            tasks: Vec::new(),
            threads: Vec::new(),
            passports: Vec::new(),
            stats: Stats::default(),
        }
    }

    pub fn active_allocations(&self) -> Vec<&Allocation> {
        self.allocations.iter().filter(|a| a.is_active()).collect()
    }

    pub fn leaked_allocations(&self) -> Vec<&Allocation> {
        self.allocations
            .iter()
            .filter(|a| a.is_active() && a.meta.var_name.is_none())
            .collect()
    }
}

/// Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: u64,
    pub total_size: usize,
    pub allocation_count: usize,
    pub start_ts: u64,
    pub end_ts: Option<u64>,
}

/// Thread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub thread_id: u32,
    pub total_size: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

/// Statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub total_allocations: usize,
    pub total_size: usize,
    pub active_allocations: usize,
    pub leaked_allocations: usize,
    pub fragmentation_ratio: f64,
}

impl Stats {
    pub fn update_from_allocations(&mut self, allocations: &[Allocation]) {
        self.total_allocations = allocations.len();
        self.total_size = allocations.iter().map(|a| a.size).sum();
        self.active_allocations = allocations.iter().filter(|a| a.is_active()).count();
        self.leaked_allocations = allocations
            .iter()
            .filter(|a| a.is_active() && a.meta.var_name.is_none())
            .count();
    }
}
