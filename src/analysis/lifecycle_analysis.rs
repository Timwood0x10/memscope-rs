//! Advanced lifecycle analysis for Rust types
//!
//! This module implements the features outlined in ComplexTypeForRust.md:
//! 1. Drop trait tracking - Record custom destructor execution
//! 2. RAII pattern detection - Identify resource acquisition patterns
//! 3. Borrow checker integration - Track borrow and mutable borrow lifetimes
//! 4. Closure capture analysis - Track closure captured variable lifetimes

use crate::core::safe_operations::SafeLock;
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
    drop_events: Mutex<Arc<Vec<DropEvent>>>,
    /// RAII pattern instances
    raii_patterns: Mutex<Arc<Vec<RAIIPattern>>>,
    /// Borrow tracking information
    borrow_tracker: Mutex<BorrowTracker>,
    /// Closure capture analysis
    closure_captures: Mutex<Arc<Vec<ClosureCapture>>>,
}

impl LifecycleAnalyzer {
    /// Create a new lifecycle analyzer
    pub fn new() -> Self {
        Self {
            drop_events: Mutex::new(Arc::new(Vec::new())),
            raii_patterns: Mutex::new(Arc::new(Vec::new())),
            borrow_tracker: Mutex::new(BorrowTracker::new()),
            closure_captures: Mutex::new(Arc::new(Vec::new())),
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

        if let Ok(mut events) = self.drop_events.safe_lock() {
            let mut new_events = (**events).clone();
            new_events.push(event);
            *events = Arc::new(new_events);
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
        if let Ok(mut stored_patterns) = self.raii_patterns.safe_lock() {
            let mut new_patterns = (**stored_patterns).clone();
            new_patterns.extend(patterns.clone());
            *stored_patterns = Arc::new(new_patterns);
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
        if let Ok(mut tracker) = self.borrow_tracker.safe_lock() {
            tracker.track_borrow(ptr, borrow_type, location);
        }
    }

    /// Track borrow release
    pub fn track_borrow_release(&self, ptr: usize, borrow_id: u64) {
        if let Ok(mut tracker) = self.borrow_tracker.safe_lock() {
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

        if let Ok(mut captures) = self.closure_captures.safe_lock() {
            let mut new_captures = (**captures).clone();
            new_captures.push(capture);
            *captures = Arc::new(new_captures);
        }
    }

    /// Get comprehensive lifecycle analysis report
    pub fn get_lifecycle_report(&self) -> LifecycleAnalysisReport {
        let drop_events = self
            .drop_events
            .safe_lock()
            .map(|events| Arc::clone(&events))
            .unwrap_or_else(|_| Arc::new(Vec::new()));
        let raii_patterns = self
            .raii_patterns
            .safe_lock()
            .map(|patterns| Arc::clone(&patterns))
            .unwrap_or_else(|_| Arc::new(Vec::new()));
        let borrow_analysis = self
            .borrow_tracker
            .safe_lock()
            .map(|tracker| tracker.get_analysis())
            .unwrap_or_else(|_| BorrowAnalysis {
                conflicts: Vec::new(),
                active_borrows: 0,
                borrow_patterns: Vec::new(),
                long_lived_borrows: Vec::new(),
                total_borrows: 0,
            });
        let closure_captures = self
            .closure_captures
            .safe_lock()
            .map(|captures| Arc::clone(&captures))
            .unwrap_or_else(|_| Arc::new(Vec::new()));

        LifecycleAnalysisReport {
            drop_events: (*drop_events).clone(),
            raii_patterns: (*raii_patterns).clone(),
            borrow_analysis,
            closure_captures: (*closure_captures).clone(),
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
    /// Memory
    Memory,
    /// File handle
    FileHandle,
    /// Network socket
    NetworkSocket,
    /// Synchronization primitive
    SynchronizationPrimitive,
    /// Thread handle
    ThreadHandle,
    /// Lock guard  
    LockGuard,
    /// Other resource type
    Other(String),
}

/// Methods of resource acquisition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcquisitionMethod {
    /// Constructor
    Constructor,
    /// System call
    SystemCall,
    /// Lock
    Lock,
    /// Allocation
    Allocation,
    /// Unknown
    Unknown,
}

/// Methods of resource release
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReleaseMethod {
    /// Automatic drop (RAII)
    AutomaticDrop,
    /// Custom drop implementation
    CustomDrop,
    /// Deallocation
    Deallocation,
    /// System call
    SystemCall,
}

/// Scope information for RAII patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    /// Scope name
    pub scope_name: String,
    /// Scope type
    pub scope_type: ScopeType,
    /// Nesting level
    pub nesting_level: usize,
}

/// Types of scopes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScopeType {
    /// Function scope
    Function,
    /// Method scope
    Method,
    /// Block scope
    Block,
    /// Loop scope
    Loop,
    /// Conditional scope
    Conditional,
    /// Unknown scope
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
    /// Create a new borrow tracker
    pub fn new() -> Self {
        Self {
            active_borrows: HashMap::new(),
            borrow_history: Vec::new(),
            next_borrow_id: 1,
        }
    }

    /// Track a borrow
    pub fn track_borrow(&mut self, ptr: usize, borrow_type: BorrowType, location: &str) -> u64 {
        let borrow_id = self.next_borrow_id;
        self.next_borrow_id += 1;

        let borrow_info = BorrowInfo {
            borrow_id,
            borrow_type,
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

    /// Release a borrow
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

    /// Get comprehensive borrow analysis report
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
                description: format!("Detected {short_borrows} short-lived borrows (< 1ms)",),
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
    /// Immutable borrow
    Immutable,
    /// Mutable borrow
    Mutable,
}

/// Borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Borrow ID
    pub borrow_id: u64,
    /// Borrow type
    pub borrow_type: BorrowType,
    /// Start timestamp
    pub start_timestamp: u64,
    /// Location
    pub location: String,
    /// Thread ID
    pub thread_id: String,
}

/// Borrow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowEvent {
    /// Pointer to the memory location
    pub ptr: usize,
    /// Borrow information
    pub borrow_info: BorrowInfo,
    /// Event type
    pub event_type: BorrowEventType,
    /// Timestamp
    pub timestamp: u64,
}

/// Types of borrow events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowEventType {
    /// Borrow acquired
    Acquired,
    /// Borrow released
    Released,
}

/// Borrow analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowAnalysis {
    /// Total number of borrows
    pub total_borrows: usize,
    /// Number of active borrows
    pub active_borrows: usize,
    /// List of borrow conflicts
    pub conflicts: Vec<BorrowConflict>,
    /// List of long-lived borrows
    pub long_lived_borrows: Vec<LongLivedBorrow>,
    /// List of borrow patterns
    pub borrow_patterns: Vec<BorrowPattern>,
}

/// Borrow conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowConflict {
    /// Pointer to the memory location
    pub ptr: usize,
    /// First borrow information
    pub first_borrow: BorrowInfo,
    /// Second borrow information
    pub second_borrow: BorrowInfo,
    /// Conflict type
    pub conflict_type: ConflictType,
}

/// Types of borrow conflicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Mutable borrow conflict
    MutableMutable,
    /// Mutable and immutable borrow conflict
    MutableImmutable,
    /// No conflict
    None,
}

/// Long-lived borrow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongLivedBorrow {
    /// Pointer to the memory location
    pub ptr: usize,
    /// Borrow information
    pub borrow_info: BorrowInfo,
    /// Duration in nanoseconds
    pub duration_ns: u64,
}

/// Borrow pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowPattern {
    /// Pattern type
    pub pattern_type: BorrowPatternType,
    /// Pattern description
    pub description: String,
    /// Pattern impact
    pub impact: PatternImpact,
    /// Pattern suggestion
    pub suggestion: String,
}

/// Types of borrow patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowPatternType {
    /// Frequent short borrows
    FrequentShortBorrows,
    /// Long-lived borrows
    LongLivedBorrows,
    /// Conflict-prone borrows
    ConflictProne,
}

/// Impact level of patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternImpact {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
}

/// Closure capture analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureCapture {
    /// Closure pointer
    pub closure_ptr: usize,
    /// List of captured variables
    pub captured_vars: Vec<CapturedVariable>,
    /// Capture timestamp
    pub capture_timestamp: u64,
    /// Thread ID
    pub thread_id: String,
}

/// Information about a captured variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedVariable {
    /// Variable name
    pub var_name: String,
    /// Variable memory address
    pub var_ptr: usize,
    /// Capture mode
    pub capture_mode: CaptureMode,
    /// Variable type
    pub var_type: String,
    /// Variable size in bytes
    pub size: usize,
}

/// Modes of variable capture in closures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
    /// Capture by value
    ByValue,
    /// Capture by reference
    ByReference,
    /// Capture by mutable reference
    ByMutableReference,
}

/// Comprehensive lifecycle analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAnalysisReport {
    /// Drop events
    pub drop_events: Vec<DropEvent>,
    /// RAII patterns
    pub raii_patterns: Vec<RAIIPattern>,
    /// Borrow analysis
    pub borrow_analysis: BorrowAnalysis,
    /// Closure captures
    pub closure_captures: Vec<ClosureCapture>,
    /// Analysis timestamp
    pub analysis_timestamp: u64,
}

impl Default for BorrowAnalysis {
    fn default() -> Self {
        Self {
            conflicts: Vec::new(),
            active_borrows: 0,
            borrow_patterns: Vec::new(),
            long_lived_borrows: Vec::new(),
            total_borrows: 0,
        }
    }
}

impl Default for LifecycleAnalysisReport {
    fn default() -> Self {
        Self {
            drop_events: Vec::new(),
            raii_patterns: Vec::new(),
            borrow_analysis: BorrowAnalysis::default(),
            closure_captures: Vec::new(),
            analysis_timestamp: 0,
        }
    }
}

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
