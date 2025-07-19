//! Enhanced memory tracking for unsafe Rust and FFI operations
//! 
//! This module extends the basic memory tracking to handle:
//! - Unsafe Rust memory operations (std::alloc::alloc, raw pointers)
//! - FFI memory operations (malloc, free from C libraries)
//! - Cross-boundary memory transfers
//! - Safety violation detection
use crate::types::{AllocationInfo, TrackingError, TrackingResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use serde::{Deserialize, Serialize};

/// Enhanced allocation source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationSource {
    /// Safe Rust allocation (through normal allocator)
    RustSafe,
    /// Unsafe Rust allocation with location info
    UnsafeRust { 
        /// Location of the unsafe block in source code
        unsafe_block_location: String,
        /// Call stack at the time of allocation
        call_stack: Vec<StackFrame>,
    },
    /// FFI allocation from C library
    FfiC { 
        /// Name of the C library
        library_name: String, 
        /// Name of the C function that allocated
        function_name: String,
        /// Call stack at the time of allocation
        call_stack: Vec<StackFrame>,
    },
    /// Cross-boundary memory transfer
    CrossBoundary { 
        /// Source allocation context
        from: Box<AllocationSource>, 
        /// Destination allocation context
        to: Box<AllocationSource>,
        /// Timestamp when transfer occurred
        transfer_timestamp: u128,
    },
}

/// Stack frame information for call stack tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Name of the function in this stack frame
    pub function_name: String,
    /// Source file name if available
    pub file_name: Option<String>,
    /// Line number in the source file if available
    pub line_number: Option<u32>,
    /// Whether this frame is in an unsafe block
    pub is_unsafe: bool,
}

/// Safety violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyViolation {
    /// Double free detected
    DoubleFree { 
        /// Call stack from the first free operation
        first_free_stack: Vec<StackFrame>,
        /// Call stack from the second free operation
        second_free_stack: Vec<StackFrame>,
        /// Timestamp when the double free was detected
        timestamp: u128,
    },
    /// Invalid free (pointer not in allocation table)
    InvalidFree { 
        /// The pointer that was attempted to be freed
        attempted_pointer: usize,
        /// Call stack at the time of invalid free
        stack: Vec<StackFrame>,
        /// Timestamp when the invalid free was attempted
        timestamp: u128,
    },
    /// Potential memory leak
    PotentialLeak { 
        /// Call stack from the original allocation
        allocation_stack: Vec<StackFrame>,
        /// Timestamp when the allocation occurred
        allocation_timestamp: u128,
        /// Timestamp when the leak was detected
        leak_detection_timestamp: u128,
    },
    /// Cross-boundary risk
    CrossBoundaryRisk { 
        /// Risk level of the cross-boundary operation
        risk_level: RiskLevel,
        /// Description of the risk
        description: String,
        /// Call stack at the time of risk detection
        stack: Vec<StackFrame>,
    },
}

/// Risk levels for safety violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - minor issues that are unlikely to cause problems
    Low,
    /// Medium risk - issues that could potentially cause problems
    Medium,
    /// High risk - serious issues that are likely to cause problems
    High,
    /// Critical risk - severe issues that will almost certainly cause problems
    Critical,
}

/// Enhanced allocation info with unsafe/FFI tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAllocationInfo {
    /// Base allocation info
    pub base: AllocationInfo,
    /// Source of the allocation
    pub source: AllocationSource,
    /// Call stack at allocation time
    pub call_stack: Vec<StackFrame>,
    /// Cross-boundary events
    pub cross_boundary_events: Vec<BoundaryEvent>,
    /// Safety violations detected
    pub safety_violations: Vec<SafetyViolation>,
    /// Whether this allocation is currently being tracked by FFI
    pub ffi_tracked: bool,
}

/// Cross-boundary memory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryEvent {
    /// Type of boundary crossing event
    pub event_type: BoundaryEventType,
    /// Timestamp when the event occurred
    pub timestamp: u128,
    /// Context where the crossing originated
    pub from_context: String,
    /// Context where the crossing ended
    pub to_context: String,
    /// Call stack at the time of crossing
    pub stack: Vec<StackFrame>,
}

/// Types of boundary events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryEventType {
    /// Memory allocated in Rust, passed to FFI
    RustToFfi,
    /// Memory allocated in FFI, passed to Rust
    FfiToRust,
    /// Memory ownership transferred
    OwnershipTransfer,
    /// Memory shared between contexts
    SharedAccess,
}

/// Enhanced memory tracker for unsafe/FFI operations
pub struct UnsafeFFITracker {
    /// Enhanced allocations with source tracking
    enhanced_allocations: Mutex<HashMap<usize, EnhancedAllocationInfo>>,
    /// Freed pointers (for double-free detection)
    freed_pointers: Mutex<HashMap<usize, (Vec<StackFrame>, u128)>>,
    /// Safety violations log
    violations: Mutex<Vec<SafetyViolation>>,
}

impl UnsafeFFITracker {
    /// Create a new enhanced tracker
    pub fn new() -> Self {
        Self {
            enhanced_allocations: Mutex::new(HashMap::new()),
            freed_pointers: Mutex::new(HashMap::new()),
            violations: Mutex::new(Vec::new()),
        }
    }

    /// Track an unsafe Rust allocation
    pub fn track_unsafe_allocation(
        &self,
        ptr: usize,
        size: usize,
        unsafe_location: String,
    ) -> TrackingResult<()> {
        let call_stack = self.capture_call_stack()?;
        let base_allocation = AllocationInfo::new(ptr, size);
        
        let enhanced = EnhancedAllocationInfo {
            base: base_allocation,
            source: AllocationSource::UnsafeRust {
                unsafe_block_location: unsafe_location,
                call_stack: call_stack.clone(),
            },
            call_stack,
            cross_boundary_events: Vec::new(),
            safety_violations: Vec::new(),
            ffi_tracked: false,
        };

        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            allocations.insert(ptr, enhanced);
            tracing::info!("Tracked unsafe allocation at {:x} (size: {})", ptr, size);
        }

        Ok(())
    }

    /// Track an FFI allocation
    pub fn track_ffi_allocation(
        &self,
        ptr: usize,
        size: usize,
        library_name: String,
        function_name: String,
    ) -> TrackingResult<()> {
        let call_stack = self.capture_call_stack()?;
        let base_allocation = AllocationInfo::new(ptr, size);
        
        let enhanced = EnhancedAllocationInfo {
            base: base_allocation,
            source: AllocationSource::FfiC {
                library_name,
                function_name,
                call_stack: call_stack.clone(),
            },
            call_stack,
            cross_boundary_events: Vec::new(),
            safety_violations: Vec::new(),
            ffi_tracked: true,
        };

        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            allocations.insert(ptr, enhanced);
            tracing::info!("Tracked FFI allocation at {:x} (size: {})", ptr, size);
        }

        Ok(())
    }

    /// Track a deallocation with safety checks
    pub fn track_enhanced_deallocation(&self, ptr: usize) -> TrackingResult<()> {
        let call_stack = self.capture_call_stack()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Check for double free
        if let Ok(freed) = self.freed_pointers.lock() {
            if let Some((first_free_stack, _first_timestamp)) = freed.get(&ptr) {
                let violation = SafetyViolation::DoubleFree {
                    first_free_stack: first_free_stack.clone(),
                    second_free_stack: call_stack.clone(),
                    timestamp,
                };
                
                if let Ok(mut violations) = self.violations.lock() {
                    violations.push(violation);
                }
                
                tracing::error!("Double free detected at {:x}", ptr);
                return Err(TrackingError::MemoryCorruption("Memory corruption detected".to_string()));
            }
        }

        // Check if allocation exists
        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            if let Some(mut allocation) = allocations.remove(&ptr) {
                allocation.base.mark_deallocated();
                
                // Record in freed pointers
                if let Ok(mut freed) = self.freed_pointers.lock() {
                    freed.insert(ptr, (call_stack, timestamp));
                }
                
                tracing::info!("Tracked enhanced deallocation at {:x}", ptr);
            } else {
                // Invalid free
                let violation = SafetyViolation::InvalidFree {
                    attempted_pointer: ptr,
                    stack: call_stack,
                    timestamp,
                };
                
                if let Ok(mut violations) = self.violations.lock() {
                    violations.push(violation);
                }
                
                tracing::error!("Invalid free detected at {:x}", ptr);
                return Err(TrackingError::InvalidPointer(format!("Invalid pointer: 0x{ptr:x}")));
            }
        }

        Ok(())
    }

    /// Record a cross-boundary event
    pub fn record_boundary_event(
        &self,
        ptr: usize,
        event_type: BoundaryEventType,
        from_context: String,
        to_context: String,
    ) -> TrackingResult<()> {
        let call_stack = self.capture_call_stack()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let event = BoundaryEvent {
            event_type,
            timestamp,
            from_context,
            to_context,
            stack: call_stack,
        };

        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            if let Some(allocation) = allocations.get_mut(&ptr) {
                allocation.cross_boundary_events.push(event);
                tracing::info!("Recorded boundary event for {:x}", ptr);
            }
        }

        Ok(())
    }

    /// Get all safety violations
    pub fn get_safety_violations(&self) -> TrackingResult<Vec<SafetyViolation>> {
        self.violations
            .lock()
            .map(|v| v.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Get enhanced allocations
    pub fn get_enhanced_allocations(&self) -> TrackingResult<Vec<EnhancedAllocationInfo>> {
        self.enhanced_allocations
            .lock()
            .map(|allocations| allocations.values().cloned().collect())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Capture current call stack (simplified implementation)
    fn capture_call_stack(&self) -> TrackingResult<Vec<StackFrame>> {
        // In a real implementation, this would use backtrace crate
        // For now, return a simplified stack
        Ok(vec![
            StackFrame {
                function_name: "current_function".to_string(),
                file_name: Some("src/unsafe_ffi_tracker.rs".to_string()),
                line_number: Some(42),
                is_unsafe: true,
            }
        ])
    }

    /// Detect potential memory leaks
    pub fn detect_leaks(&self, threshold_ms: u128) -> TrackingResult<Vec<SafetyViolation>> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let mut leaks = Vec::new();

        if let Ok(allocations) = self.enhanced_allocations.lock() {
            for allocation in allocations.values() {
                let alloc_time = allocation.base.timestamp_alloc as u128;
                let age = current_time.saturating_sub(alloc_time);
                if age > threshold_ms && allocation.base.is_active() {
                    leaks.push(SafetyViolation::PotentialLeak {
                        allocation_stack: allocation.call_stack.clone(),
                        allocation_timestamp: allocation.base.timestamp_alloc as u128,
                        leak_detection_timestamp: current_time,
                    });
                }
            }
        }

        Ok(leaks)
    }
}

impl Default for UnsafeFFITracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance of the enhanced tracker
static GLOBAL_UNSAFE_FFI_TRACKER: std::sync::OnceLock<std::sync::Arc<UnsafeFFITracker>> = std::sync::OnceLock::new();

/// Get the global unsafe/FFI tracker instance
pub fn get_global_unsafe_ffi_tracker() -> std::sync::Arc<UnsafeFFITracker> {
    GLOBAL_UNSAFE_FFI_TRACKER
        .get_or_init(|| std::sync::Arc::new(UnsafeFFITracker::new()))
        .clone()
}

/// Macro for tracking unsafe allocations
#[macro_export]
macro_rules! track_unsafe_alloc {
    ($ptr:expr, $size:expr) => {
        {
            let tracker = $crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker();
            let location = format!("{}:{}:{}", file!(), line!(), column!());
            let _ = tracker.track_unsafe_allocation($ptr as usize, $size, location);
        }
    };
}

/// Macro for tracking FFI allocations
#[macro_export]
macro_rules! track_ffi_alloc {
    ($ptr:expr, $size:expr, $lib:expr, $func:expr) => {
        {
            let tracker = $crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker();
            let _ = tracker.track_ffi_allocation($ptr as usize, $size, $lib.to_string(), $func.to_string());
        }
    };
}

/// Statistics for unsafe and FFI operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnsafeFFIStats {
    /// Total number of unsafe operations
    pub total_operations: usize,
    /// Number of unsafe blocks encountered
    pub unsafe_blocks: usize,
    /// Number of FFI calls made
    pub ffi_calls: usize,
    /// Number of raw pointer operations
    pub raw_pointer_operations: usize,
    /// Number of memory violations detected
    pub memory_violations: usize,
    /// Overall risk score (0.0 to 10.0)
    pub risk_score: f64,
    /// List of unsafe operations
    pub operations: Vec<UnsafeOperation>,
}

/// Represents a single unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeOperation {
    /// Type of operation
    pub operation_type: UnsafeOperationType,
    /// Location in source code
    pub location: String,
    /// Risk level of this operation
    pub risk_level: RiskLevel,
    /// Timestamp when operation occurred
    pub timestamp: u128,
    /// Description of the operation
    pub description: String,
}

/// Types of unsafe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnsafeOperationType {
    /// Raw pointer dereference operation
    RawPointerDeref,
    /// Foreign Function Interface call
    FfiCall,
    /// Unsafe block execution
    UnsafeBlock,
    /// Memory safety violation detected
    MemoryViolation,
    /// Memory transfer across safety boundaries
    CrossBoundaryTransfer,
}


impl UnsafeFFITracker {
    /// Get statistics for unsafe and FFI operations
    pub fn get_stats(&self) -> UnsafeFFIStats {
        let allocations = self.enhanced_allocations.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let violations = self.violations.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        
        let mut stats = UnsafeFFIStats::default();
        
        // Count different types of operations
        for allocation in allocations.values() {
            match &allocation.source {
                AllocationSource::UnsafeRust { .. } => {
                    stats.unsafe_blocks += 1;
                    stats.total_operations += 1;
                }
                AllocationSource::FfiC { .. } => {
                    stats.ffi_calls += 1;
                    stats.total_operations += 1;
                }
                AllocationSource::CrossBoundary { .. } => {
                    stats.total_operations += 1;
                }
                _ => {}
            }
            
            // Count safety violations
            stats.memory_violations += allocation.safety_violations.len();
        }
        
        // Add violations from the violations log
        stats.memory_violations += violations.len();
        
        // Calculate risk score (simplified)
        stats.risk_score = if stats.total_operations > 0 {
            let base_risk = (stats.unsafe_blocks as f64 * 1.0) + 
                           (stats.ffi_calls as f64 * 2.0) + 
                           (stats.memory_violations as f64 * 5.0);
            (base_risk / stats.total_operations as f64).min(10.0)
        } else {
            0.0
        };
        
        // Create operation list (simplified)
        for allocation in allocations.values() {
            let (op_type, risk_level, description) = match &allocation.source {
                AllocationSource::UnsafeRust { unsafe_block_location, .. } => {
                    (UnsafeOperationType::UnsafeBlock, RiskLevel::Medium, 
                     format!("Unsafe block at {unsafe_block_location}"))
                }
                AllocationSource::FfiC { library_name, function_name, .. } => {
                    (UnsafeOperationType::FfiCall, RiskLevel::High,
                     format!("FFI call to {library_name}::{function_name}"))
                }
                AllocationSource::CrossBoundary { .. } => {
                    (UnsafeOperationType::CrossBoundaryTransfer, RiskLevel::Medium,
                     "Cross-boundary memory transfer".to_string())
                }
                _ => continue,
            };
            
            stats.operations.push(UnsafeOperation {
                operation_type: op_type,
                location: "unknown".to_string(), // Could be enhanced with actual location
                risk_level,
                timestamp: allocation.base.timestamp_alloc as u128,
                description,
            });
        }
        
        // Limit operations to avoid huge JSON
        stats.operations.truncate(50);
        
        stats
    }
}

/// Global unsafe/FFI tracker instance
static GLOBAL_UNSAFE_TRACKER: OnceLock<Arc<UnsafeFFITracker>> = OnceLock::new();

/// Get the global unsafe/FFI tracker instance
pub fn get_global_unsafe_tracker() -> Option<Arc<UnsafeFFITracker>> {
    GLOBAL_UNSAFE_TRACKER.get().cloned()
}

/// Initialize the global unsafe/FFI tracker
pub fn init_global_unsafe_tracker() -> Arc<UnsafeFFITracker> {
    GLOBAL_UNSAFE_TRACKER
        .get_or_init(|| Arc::new(UnsafeFFITracker::new()))
        .clone()
}