//! Advanced lifecycle analysis for Rust types
//!
//! This module implements the features outlined in ComplexTypeForRust.md:
//! 1. Drop trait tracking - Record custom destructor execution
//! 2. RAII pattern detection - Identify resource acquisition patterns
//! 3. Borrow checker integration - Track borrow and mutable borrow lifetimes
//! 4. Closure capture analysis - Track closure captured variable lifetimes

use crate::core::types::AllocationInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global lifecycle analyzer instance
static GLOBAL_LIFECYCLE_ANALYZER: OnceLock<Arc<LifecycleAnalyzer>> = OnceLock::new();

/// Get the global lifecycle analyzer instance
pub fn get_global_lifecycle_analyzer() -> Arc<LifecycleAnalyzer> {
    GLOBAL_LIFECYCLE_ANALYZER
        .get_or_init(|| Arc::new(LifecycleAnalyzer::new()))
        .clone()
}

/// Advanced lifecycle analysis for Rust types
pub struct LifecycleAnalyzer {
    /// Drop trait execution tracking
    drop_events: Mutex<Vec<DropEvent>>,
    /// RAII pattern instances
    raii_patterns: Mutex<Vec<RAIIPattern>>,
    /// Borrow tracking information
    borrow_tracker: Mutex<BorrowTracker>,
    /// Closure capture analysis
    closure_captures: Mutex<Vec<ClosureCapture>>,
}

impl LifecycleAnalyzer {
    /// Create a new lifecycle analyzer
    pub fn new() -> Self {
        Self {
            drop_events: Mutex::new(Vec::new()),
            raii_patterns: Mutex::new(Vec::new()),
            borrow_tracker: Mutex::new(BorrowTracker::new()),
            closure_captures: Mutex::new(Vec::new()),
        }
    }

    /// Record a Drop trait execution
    pub fn record_drop_event(&self, ptr: usize, type_name: &str, custom_drop: bool) {
        let event = DropEvent {
            ptr,
            type_name: type_name.to_string(),
            timestamp: current_timestamp(),
            custom_drop,
            thread_id: format!("{:?}", std::thread::current().id()),
            call_stack: capture_call_stack(),
        };

        if let Ok(mut events) = self.drop_events.lock() {
            events.push(event);
        }
    }

    /// Detect RAII patterns in allocations
    pub fn detect_raii_patterns(&self, allocations: &[AllocationInfo]) -> Vec<RAIIPattern> {
        let mut patterns = Vec::new();

        for allocation in allocations {
            if let Some(type_name) = &allocation.type_name {
                if let Some(pattern) = self.analyze_raii_pattern(allocation, type_name) {
                    patterns.push(pattern);
                }
            }
        }

        // Store detected patterns
        if let Ok(mut stored_patterns) = self.raii_patterns.lock() {
            stored_patterns.extend(patterns.clone());
        }

        patterns
    }

    /// Analyze RAII pattern for a specific allocation
    fn analyze_raii_pattern(
        &self,
        allocation: &AllocationInfo,
        type_name: &str,
    ) -> Option<RAIIPattern> {
        let resource_type = self.identify_resource_type(type_name)?;
        let acquisition_method = self.identify_acquisition_method(type_name);
        let release_method = self.identify_release_method(type_name);

        Some(RAIIPattern {
            ptr: allocation.ptr,
            type_name: type_name.to_string(),
            resource_type,
            acquisition_method,
            release_method,
            acquisition_timestamp: allocation.timestamp_alloc,
            release_timestamp: allocation.timestamp_dealloc,
            scope_info: self.analyze_scope_info(allocation),
            is_exception_safe: self.is_exception_safe(type_name),
        })
    }

    /// Identify the type of resource being managed
    fn identify_resource_type(&self, type_name: &str) -> Option<ResourceType> {
        if type_name.contains("File")
            || type_name.contains("BufReader")
            || type_name.contains("BufWriter")
        {
            Some(ResourceType::FileHandle)
        } else if type_name.contains("TcpStream")
            || type_name.contains("UdpSocket")
            || type_name.contains("Listener")
        {
            Some(ResourceType::NetworkSocket)
        } else if type_name.contains("Mutex")
            || type_name.contains("RwLock")
            || type_name.contains("Semaphore")
        {
            Some(ResourceType::SynchronizationPrimitive)
        } else if type_name.contains("Thread") || type_name.contains("JoinHandle") {
            Some(ResourceType::ThreadHandle)
        } else if type_name.contains("Box")
            || type_name.contains("Vec")
            || type_name.contains("String")
        {
            Some(ResourceType::Memory)
        } else if type_name.contains("Guard") || type_name.contains("Lock") {
            Some(ResourceType::LockGuard)
        } else {
            None
        }
    }

    /// Identify how the resource is acquired
    fn identify_acquisition_method(&self, type_name: &str) -> AcquisitionMethod {
        if type_name.contains("new") || type_name.contains("Box") {
            AcquisitionMethod::Constructor
        } else if type_name.contains("open") || type_name.contains("connect") {
            AcquisitionMethod::SystemCall
        } else if type_name.contains("lock") || type_name.contains("Guard") {
            AcquisitionMethod::Lock
        } else if type_name.contains("with_capacity") || type_name.contains("reserve") {
            AcquisitionMethod::Allocation
        } else {
            AcquisitionMethod::Unknown
        }
    }

    /// Identify how the resource is released
    fn identify_release_method(&self, type_name: &str) -> ReleaseMethod {
        if type_name.contains("Guard") || type_name.contains("Lock") {
            ReleaseMethod::AutomaticDrop
        } else if type_name.contains("File") || type_name.contains("Stream") {
            ReleaseMethod::AutomaticDrop
        } else if type_name.contains("Box") || type_name.contains("Vec") {
            ReleaseMethod::Deallocation
        } else {
            ReleaseMethod::CustomDrop
        }
    }

    /// Analyze scope information for RAII pattern
    fn analyze_scope_info(&self, allocation: &AllocationInfo) -> ScopeInfo {
        ScopeInfo {
            scope_name: allocation
                .scope_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            scope_type: self.infer_scope_type(&allocation.scope_name),
            nesting_level: self.calculate_nesting_level(&allocation.scope_name),
        }
    }

    /// Infer the type of scope
    fn infer_scope_type(&self, scope_name: &Option<String>) -> ScopeType {
        match scope_name {
            Some(name) if name.contains("fn ") => ScopeType::Function,
            Some(name) if name.contains("impl ") => ScopeType::Method,
            Some(name) if name.contains("for ") || name.contains("while ") => ScopeType::Loop,
            Some(name) if name.contains("if ") || name.contains("match ") => ScopeType::Conditional,
            Some(name) if name.contains("{") => ScopeType::Block,
            _ => ScopeType::Unknown,
        }
    }

    /// Calculate nesting level of scope
    fn calculate_nesting_level(&self, scope_name: &Option<String>) -> usize {
        scope_name
            .as_ref()
            .map(|name| name.matches('{').count())
            .unwrap_or(0)
    }

    /// Check if type is exception safe
    fn is_exception_safe(&self, type_name: &str) -> bool {
        // Most Rust types are exception safe due to the type system
        // Only unsafe code or FFI might not be
        !type_name.contains("unsafe") && !type_name.contains("ffi")
    }

    /// Track borrow operations
    pub fn track_borrow(&self, ptr: usize, borrow_type: BorrowType, location: &str) {
        if let Ok(mut tracker) = self.borrow_tracker.lock() {
            tracker.track_borrow(ptr, borrow_type, location);
        }
    }

    /// Track borrow release
    pub fn track_borrow_release(&self, ptr: usize, borrow_id: u64) {
        if let Ok(mut tracker) = self.borrow_tracker.lock() {
            tracker.release_borrow(ptr, borrow_id);
        }
    }

    /// Analyze closure captures
    pub fn analyze_closure_capture(
        &self,
        closure_ptr: usize,
        captured_vars: Vec<CapturedVariable>,
    ) {
        let capture = ClosureCapture {
            closure_ptr,
            captured_vars,
            capture_timestamp: current_timestamp(),
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        if let Ok(mut captures) = self.closure_captures.lock() {
            captures.push(capture);
        }
    }

    /// Get comprehensive lifecycle analysis report
    pub fn get_lifecycle_report(&self) -> LifecycleAnalysisReport {
        let drop_events = self.drop_events.lock().unwrap().clone();
        let raii_patterns = self.raii_patterns.lock().unwrap().clone();
        let borrow_analysis = self.borrow_tracker.lock().unwrap().get_analysis();
        let closure_captures = self.closure_captures.lock().unwrap().clone();

        LifecycleAnalysisReport {
            drop_events,
            raii_patterns,
            borrow_analysis,
            closure_captures,
            analysis_timestamp: current_timestamp(),
        }
    }
}

/// Drop trait execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropEvent {
    /// Pointer to the dropped object
    pub ptr: usize,
    /// Type name of the dropped object
    pub type_name: String,
    /// Timestamp when drop was executed
    pub timestamp: u64,
    /// Whether this is a custom Drop implementation
    pub custom_drop: bool,
    /// Thread where drop occurred
    pub thread_id: String,
    /// Call stack at drop time
    pub call_stack: Vec<String>,
}

/// RAII pattern instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAIIPattern {
    /// Pointer to the RAII object
    pub ptr: usize,
    /// Type name
    pub type_name: String,
    /// Type of resource being managed
    pub resource_type: ResourceType,
    /// How the resource is acquired
    pub acquisition_method: AcquisitionMethod,
    /// How the resource is released
    pub release_method: ReleaseMethod,
    /// When the resource was acquired
    pub acquisition_timestamp: u64,
    /// When the resource was released (if applicable)
    pub release_timestamp: Option<u64>,
    /// Scope information
    pub scope_info: ScopeInfo,
    /// Whether the pattern is exception safe
    pub is_exception_safe: bool,
}

/// Types of resources managed by RAII
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Memory,
    FileHandle,
    NetworkSocket,
    SynchronizationPrimitive,
    ThreadHandle,
    LockGuard,
    Other(String),
}

/// Methods of resource acquisition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcquisitionMethod {
    Constructor,
    SystemCall,
    Lock,
    Allocation,
    Unknown,
}

/// Methods of resource release
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReleaseMethod {
    AutomaticDrop,
    CustomDrop,
    Deallocation,
    SystemCall,
}

/// Scope information for RAII patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    pub scope_name: String,
    pub scope_type: ScopeType,
    pub nesting_level: usize,
}

/// Types of scopes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    Function,
    Method,
    Block,
    Loop,
    Conditional,
    Unknown,
}

/// Borrow tracking system
#[derive(Debug)]
pub struct BorrowTracker {
    /// Active borrows (ptr -> borrow info)
    active_borrows: HashMap<usize, Vec<BorrowInfo>>,
    /// Borrow history
    borrow_history: Vec<BorrowEvent>,
    /// Next borrow ID
    next_borrow_id: u64,
}

impl BorrowTracker {
    pub fn new() -> Self {
        Self {
            active_borrows: HashMap::new(),
            borrow_history: Vec::new(),
            next_borrow_id: 1,
        }
    }

    pub fn track_borrow(&mut self, ptr: usize, borrow_type: BorrowType, location: &str) -> u64 {
        let borrow_id = self.next_borrow_id;
        self.next_borrow_id += 1;

        let borrow_info = BorrowInfo {
            borrow_id,
            borrow_type: borrow_type.clone(),
            start_timestamp: current_timestamp(),
            location: location.to_string(),
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        self.active_borrows
            .entry(ptr)
            .or_insert_with(Vec::new)
            .push(borrow_info.clone());

        self.borrow_history.push(BorrowEvent {
            ptr,
            borrow_info,
            event_type: BorrowEventType::Acquired,
            timestamp: current_timestamp(),
        });

        borrow_id
    }

    pub fn release_borrow(&mut self, ptr: usize, borrow_id: u64) {
        if let Some(borrows) = self.active_borrows.get_mut(&ptr) {
            if let Some(pos) = borrows.iter().position(|b| b.borrow_id == borrow_id) {
                let borrow_info = borrows.remove(pos);

                self.borrow_history.push(BorrowEvent {
                    ptr,
                    borrow_info,
                    event_type: BorrowEventType::Released,
                    timestamp: current_timestamp(),
                });

                if borrows.is_empty() {
                    self.active_borrows.remove(&ptr);
                }
            }
        }
    }

    pub fn get_analysis(&self) -> BorrowAnalysis {
        let mut conflicts = Vec::new();
        let mut long_lived_borrows = Vec::new();

        // Analyze for conflicts and long-lived borrows
        for events in self.borrow_history.windows(2) {
            if let [event1, event2] = events {
                if event1.ptr == event2.ptr
                    && self.has_borrow_conflict(&event1.borrow_info, &event2.borrow_info)
                {
                    conflicts.push(BorrowConflict {
                        ptr: event1.ptr,
                        first_borrow: event1.borrow_info.clone(),
                        second_borrow: event2.borrow_info.clone(),
                        conflict_type: self
                            .classify_conflict(&event1.borrow_info, &event2.borrow_info),
                    });
                }
            }
        }

        // Find long-lived borrows (>1 second)
        let current_time = current_timestamp();
        for (ptr, borrows) in &self.active_borrows {
            for borrow in borrows {
                if current_time - borrow.start_timestamp > 1_000_000_000 {
                    // 1 second in nanoseconds
                    long_lived_borrows.push(LongLivedBorrow {
                        ptr: *ptr,
                        borrow_info: borrow.clone(),
                        duration_ns: current_time - borrow.start_timestamp,
                    });
                }
            }
        }

        BorrowAnalysis {
            total_borrows: self.borrow_history.len(),
            active_borrows: self.active_borrows.len(),
            conflicts,
            long_lived_borrows,
            borrow_patterns: self.analyze_borrow_patterns(),
        }
    }

    fn has_borrow_conflict(&self, borrow1: &BorrowInfo, borrow2: &BorrowInfo) -> bool {
        match (&borrow1.borrow_type, &borrow2.borrow_type) {
            (BorrowType::Mutable, _) | (_, BorrowType::Mutable) => true,
            _ => false,
        }
    }

    fn classify_conflict(&self, borrow1: &BorrowInfo, borrow2: &BorrowInfo) -> ConflictType {
        match (&borrow1.borrow_type, &borrow2.borrow_type) {
            (BorrowType::Mutable, BorrowType::Mutable) => ConflictType::MutableMutable,
            (BorrowType::Mutable, BorrowType::Immutable)
            | (BorrowType::Immutable, BorrowType::Mutable) => ConflictType::MutableImmutable,
            _ => ConflictType::None,
        }
    }

    fn analyze_borrow_patterns(&self) -> Vec<BorrowPattern> {
        // Analyze common borrow patterns
        let mut patterns = Vec::new();

        // Pattern: Frequent short borrows
        let short_borrows = self
            .borrow_history
            .iter()
            .filter(|event| matches!(event.event_type, BorrowEventType::Released))
            .filter(|event| {
                // Find corresponding acquire event
                self.borrow_history.iter().any(|acquire_event| {
                    acquire_event.borrow_info.borrow_id == event.borrow_info.borrow_id
                        && matches!(acquire_event.event_type, BorrowEventType::Acquired)
                        && event.timestamp - acquire_event.timestamp < 1_000_000
                    // < 1ms
                })
            })
            .count();

        if short_borrows > 10 {
            patterns.push(BorrowPattern {
                pattern_type: BorrowPatternType::FrequentShortBorrows,
                description: format!("Detected {} short-lived borrows (< 1ms)", short_borrows),
                impact: if short_borrows > 100 {
                    PatternImpact::High
                } else {
                    PatternImpact::Medium
                },
                suggestion: "Consider batching operations to reduce borrow overhead".to_string(),
            });
        }

        patterns
    }
}

/// Types of borrows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

/// Borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    pub borrow_id: u64,
    pub borrow_type: BorrowType,
    pub start_timestamp: u64,
    pub location: String,
    pub thread_id: String,
}

/// Borrow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowEvent {
    pub ptr: usize,
    pub borrow_info: BorrowInfo,
    pub event_type: BorrowEventType,
    pub timestamp: u64,
}

/// Types of borrow events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowEventType {
    Acquired,
    Released,
}

/// Borrow analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowAnalysis {
    pub total_borrows: usize,
    pub active_borrows: usize,
    pub conflicts: Vec<BorrowConflict>,
    pub long_lived_borrows: Vec<LongLivedBorrow>,
    pub borrow_patterns: Vec<BorrowPattern>,
}

/// Borrow conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowConflict {
    pub ptr: usize,
    pub first_borrow: BorrowInfo,
    pub second_borrow: BorrowInfo,
    pub conflict_type: ConflictType,
}

/// Types of borrow conflicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    MutableMutable,
    MutableImmutable,
    None,
}

/// Long-lived borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongLivedBorrow {
    pub ptr: usize,
    pub borrow_info: BorrowInfo,
    pub duration_ns: u64,
}

/// Borrow pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowPattern {
    pub pattern_type: BorrowPatternType,
    pub description: String,
    pub impact: PatternImpact,
    pub suggestion: String,
}

/// Types of borrow patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowPatternType {
    FrequentShortBorrows,
    LongLivedBorrows,
    ConflictProne,
}

/// Impact level of patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternImpact {
    Low,
    Medium,
    High,
}

/// Closure capture analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureCapture {
    pub closure_ptr: usize,
    pub captured_vars: Vec<CapturedVariable>,
    pub capture_timestamp: u64,
    pub thread_id: String,
}

/// Information about a captured variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedVariable {
    pub var_name: String,
    pub var_ptr: usize,
    pub capture_mode: CaptureMode,
    pub var_type: String,
    pub size: usize,
}

/// Modes of variable capture in closures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
    ByValue,
    ByReference,
    ByMutableReference,
}

/// Comprehensive lifecycle analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAnalysisReport {
    pub drop_events: Vec<DropEvent>,
    pub raii_patterns: Vec<RAIIPattern>,
    pub borrow_analysis: BorrowAnalysis,
    pub closure_captures: Vec<ClosureCapture>,
    pub analysis_timestamp: u64,
}

/// Utility functions

/// Get current timestamp in nanoseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Capture call stack (simplified implementation)
fn capture_call_stack() -> Vec<String> {
    // In a real implementation, this would use backtrace
    // For now, return a placeholder
    vec!["<call_stack_placeholder>".to_string()]
}

impl Default for LifecycleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
