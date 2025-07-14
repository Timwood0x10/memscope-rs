//! Enhanced memory tracking for unsafe Rust and FFI operations
//! 
//! This module extends the basic memory tracking to handle:
//! - Unsafe Rust memory operations (std::alloc::alloc, raw pointers)
//! - FFI memory operations (malloc, free from C libraries)
//! - Cross-boundary memory transfers
//! - Safety violation detection

use crate::types::{AllocationInfo, TrackingError, TrackingResult};
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Enhanced allocation source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationSource {
    /// Safe Rust allocation (through normal allocator)
    RustSafe,
    /// Unsafe Rust allocation with location info
    UnsafeRust { 
        unsafe_block_location: String,
        call_stack: Vec<StackFrame>,
    },
    /// FFI allocation from C library
    FfiC { 
        library_name: String, 
        function_name: String,
        call_stack: Vec<StackFrame>,
    },
    /// Cross-boundary memory transfer
    CrossBoundary { 
        from: Box<AllocationSource>, 
        to: Box<AllocationSource>,
        transfer_timestamp: u128,
    },
}

/// Stack frame information for call stack tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub is_unsafe: bool,
}

/// Safety violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyViolation {
    /// Double free detected
    DoubleFree { 
        first_free_stack: Vec<StackFrame>,
        second_free_stack: Vec<StackFrame>,
        timestamp: u128,
    },
    /// Invalid free (pointer not in allocation table)
    InvalidFree { 
        attempted_pointer: usize,
        stack: Vec<StackFrame>,
        timestamp: u128,
    },
    /// Potential memory leak
    PotentialLeak { 
        allocation_stack: Vec<StackFrame>,
        allocation_timestamp: u128,
        leak_detection_timestamp: u128,
    },
    /// Cross-boundary risk
    CrossBoundaryRisk { 
        risk_level: RiskLevel,
        description: String,
        stack: Vec<StackFrame>,
    },
}

/// Risk levels for safety violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
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
    pub event_type: BoundaryEventType,
    pub timestamp: u128,
    pub from_context: String,
    pub to_context: String,
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
                return Err(TrackingError::MemoryCorruption);
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
                return Err(TrackingError::InvalidPointer { ptr });
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
                let age = current_time - allocation.base.timestamp_alloc;
                if age > threshold_ms && allocation.base.is_active() {
                    leaks.push(SafetyViolation::PotentialLeak {
                        allocation_stack: allocation.call_stack.clone(),
                        allocation_timestamp: allocation.base.timestamp_alloc,
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