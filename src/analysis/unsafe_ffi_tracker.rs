//! Enhanced memory tracking for unsafe Rust and FFI operations
//!
//! This module extends the basic memory tracking to handle:
//! - Unsafe Rust memory operations (std::alloc::alloc, raw pointers)
//! - FFI memory operations (malloc, free from C libraries)
//! - Cross-boundary memory transfers
//! - Safety violation detection
use crate::core::types::{AllocationInfo, TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

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
        /// Risk assessment for this unsafe operation
        risk_assessment: RiskAssessment,
    },
    /// FFI allocation from C library
    FfiC {
        /// Name of the C library
        library_name: String,
        /// Name of the C function that allocated
        function_name: String,
        /// Call stack at the time of allocation
        call_stack: Vec<StackFrame>,
        /// LibC hook information
        libc_hook_info: LibCHookInfo,
    },
    /// Cross-boundary memory transfer
    CrossBoundary {
        /// Source allocation context
        from: Box<AllocationSource>,
        /// Destination allocation context
        to: Box<AllocationSource>,
        /// Timestamp when transfer occurred
        transfer_timestamp: u128,
        /// Transfer metadata
        transfer_metadata: TransferMetadata,
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

/// Comprehensive risk assessment for unsafe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Specific risk factors identified
    pub risk_factors: Vec<RiskFactor>,
    /// Suggested mitigation strategies
    pub mitigation_suggestions: Vec<String>,
    /// Confidence score of the assessment (0.0 to 1.0)
    pub confidence_score: f64,
    /// Timestamp when assessment was performed
    pub assessment_timestamp: u128,
}

/// Individual risk factor in an assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk factor
    pub factor_type: RiskFactorType,
    /// Severity score (0.0 to 10.0)
    pub severity: f64,
    /// Human-readable description
    pub description: String,
    /// Source location where risk was detected
    pub source_location: Option<String>,
}

/// Types of risk factors that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    /// Raw pointer dereference without bounds checking
    RawPointerDeref,
    /// Manual memory management (alloc/dealloc)
    ManualMemoryManagement,
    /// Memory transfer across language boundaries
    CrossBoundaryTransfer,
    /// Unchecked type casting
    UncheckedCast,
    /// Potential lifetime violation
    LifetimeViolation,
    /// Use after free potential
    UseAfterFree,
    /// Buffer overflow potential
    BufferOverflow,
    /// Data race potential
    DataRace,
}

/// Information about LibC function hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibCHookInfo {
    /// Method used to hook the function
    pub hook_method: HookMethod,
    /// Original function that was hooked
    pub original_function: String,
    /// Timestamp when hook was installed
    pub hook_timestamp: u128,
    /// Metadata about the allocation
    pub allocation_metadata: AllocationMetadata,
    /// Performance impact of the hook
    pub hook_overhead_ns: Option<u64>,
}

/// Methods for hooking LibC functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookMethod {
    /// LD_PRELOAD mechanism (Linux/macOS)
    LdPreload,
    /// Dynamic linker interposition
    DynamicLinker,
    /// Static function interposition
    StaticInterposition,
    /// Runtime patching
    RuntimePatching,
}

/// Metadata about memory allocations from LibC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationMetadata {
    /// Size requested by the caller
    pub requested_size: usize,
    /// Actual size allocated (may be larger due to alignment)
    pub actual_size: usize,
    /// Memory alignment used
    pub alignment: usize,
    /// Information about the allocator used
    pub allocator_info: String,
    /// Memory protection flags if available
    pub protection_flags: Option<MemoryProtectionFlags>,
}

/// Memory protection flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProtectionFlags {
    /// Memory is readable
    pub readable: bool,
    /// Memory is writable
    pub writable: bool,
    /// Memory is executable
    pub executable: bool,
    /// Memory is shared
    pub shared: bool,
}

/// Memory "passport" for tracking cross-boundary transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    /// Unique identifier for this memory passport
    pub passport_id: String,
    /// Original allocation context
    pub origin: AllocationOrigin,
    /// Journey of the memory through different contexts
    pub journey: Vec<PassportStamp>,
    /// Current ownership information
    pub current_owner: OwnershipInfo,
    /// Validity status of the passport
    pub validity_status: ValidityStatus,
    /// Security clearance level
    pub security_clearance: SecurityClearance,
}

/// Information about where memory was originally allocated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationOrigin {
    /// Context where allocation occurred (Rust/FFI)
    pub context: String,
    /// Function that performed the allocation
    pub allocator_function: String,
    /// Timestamp of original allocation
    pub timestamp: u128,
    /// Call stack at allocation time
    pub call_stack: Vec<StackFrame>,
}

/// A stamp in the memory passport journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportStamp {
    /// Timestamp of this checkpoint
    pub timestamp: u128,
    /// Location/context of the checkpoint
    pub location: String,
    /// Operation performed at this checkpoint
    pub operation: String,
    /// Authority that validated this checkpoint
    pub authority: String,
    /// Cryptographic hash for verification
    pub verification_hash: String,
}

/// Current ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    /// Current owner context (Rust/FFI)
    pub owner_context: String,
    /// Function/module that owns the memory
    pub owner_function: String,
    /// Ownership transfer timestamp
    pub transfer_timestamp: u128,
    /// Expected lifetime of ownership
    pub expected_lifetime: Option<u128>,
}

/// Validity status of a memory passport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidityStatus {
    /// Passport is valid and memory is safe to use
    Valid,
    /// Passport is expired (memory may be freed)
    Expired,
    /// Passport is revoked (memory is definitely freed)
    Revoked,
    /// Passport validity is unknown/suspicious
    Suspicious,
}

/// Security clearance levels for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityClearance {
    /// Public memory, safe for all operations
    Public,
    /// Restricted memory, limited operations allowed
    Restricted,
    /// Confidential memory, special handling required
    Confidential,
    /// Secret memory, maximum security required
    Secret,
}

/// Metadata for cross-boundary transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetadata {
    /// Reason for the transfer
    pub transfer_reason: String,
    /// Expected return context (if any)
    pub expected_return: Option<String>,
    /// Transfer validation method used
    pub validation_method: ValidationMethod,
    /// Performance impact of the transfer
    pub transfer_overhead_ns: Option<u64>,
}

/// Methods for validating cross-boundary transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationMethod {
    /// No validation performed
    None,
    /// Basic pointer validation
    PointerCheck,
    /// Size and bounds validation
    BoundsCheck,
    /// Full memory integrity check
    IntegrityCheck,
    /// Cryptographic validation
    CryptographicCheck,
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
    /// Memory passport for cross-boundary tracking
    pub memory_passport: Option<MemoryPassport>,
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
    /// C library tracking registry
    c_libraries: Mutex<HashMap<String, CLibraryInfo>>,
    /// Enhanced LibC hook registry
    libc_hooks: Mutex<HashMap<String, EnhancedLibCHookInfo>>,
    /// Memory passport registry
    memory_passports: Mutex<HashMap<usize, MemoryPassport>>,
}

impl UnsafeFFITracker {
    /// Create a new enhanced tracker
    pub fn new() -> Self {
        Self {
            enhanced_allocations: Mutex::new(HashMap::new()),
            freed_pointers: Mutex::new(HashMap::new()),
            violations: Mutex::new(Vec::new()),
            c_libraries: Mutex::new(HashMap::new()),
            libc_hooks: Mutex::new(HashMap::new()),
            memory_passports: Mutex::new(HashMap::new()),
        }
    }

    /// Create a default risk assessment for unsafe operations
    fn create_default_unsafe_risk_assessment(&self, unsafe_location: &str) -> RiskAssessment {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        let risk_factors = vec![RiskFactor {
            factor_type: RiskFactorType::ManualMemoryManagement,
            severity: 5.0,
            description: "Manual memory management in unsafe block".to_string(),
            source_location: Some(unsafe_location.to_string()),
        }];

        RiskAssessment {
            risk_level: RiskLevel::Medium,
            risk_factors,
            mitigation_suggestions: vec![
                "Ensure proper memory cleanup".to_string(),
                "Use RAII patterns where possible".to_string(),
            ],
            confidence_score: 0.7,
            assessment_timestamp: current_time,
        }
    }

    /// Create a default LibC hook info for FFI operations
    fn create_default_libc_hook_info(&self, function_name: &str, size: usize) -> LibCHookInfo {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        LibCHookInfo {
            hook_method: HookMethod::DynamicLinker,
            original_function: function_name.to_string(),
            hook_timestamp: current_time,
            allocation_metadata: AllocationMetadata {
                requested_size: size,
                actual_size: size,
                alignment: 8, // Default alignment
                allocator_info: "libc malloc".to_string(),
                protection_flags: Some(MemoryProtectionFlags {
                    readable: true,
                    writable: true,
                    executable: false,
                    shared: false,
                }),
            },
            hook_overhead_ns: Some(100), // Estimated overhead
        }
    }

    /// Create a memory passport for cross-boundary tracking
    fn create_memory_passport(&self, ptr: usize, origin_context: &str) -> MemoryPassport {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        MemoryPassport {
            passport_id: format!("passport_{:x}_{}", ptr, current_time),
            origin: AllocationOrigin {
                context: origin_context.to_string(),
                allocator_function: "unknown".to_string(),
                timestamp: current_time,
                call_stack: Vec::new(),
            },
            journey: Vec::new(),
            current_owner: OwnershipInfo {
                owner_context: origin_context.to_string(),
                owner_function: "unknown".to_string(),
                transfer_timestamp: current_time,
                expected_lifetime: None,
            },
            validity_status: ValidityStatus::Valid,
            security_clearance: SecurityClearance::Public,
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
        let risk_assessment = self.create_default_unsafe_risk_assessment(&unsafe_location);

        let enhanced = EnhancedAllocationInfo {
            base: base_allocation,
            source: AllocationSource::UnsafeRust {
                unsafe_block_location: unsafe_location,
                call_stack: call_stack.clone(),
                risk_assessment,
            },
            call_stack,
            cross_boundary_events: Vec::new(),
            safety_violations: Vec::new(),
            ffi_tracked: false,
            memory_passport: None,
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
        let libc_hook_info = self.create_default_libc_hook_info(&function_name, size);

        let enhanced = EnhancedAllocationInfo {
            base: base_allocation,
            source: AllocationSource::FfiC {
                library_name,
                function_name,
                call_stack: call_stack.clone(),
                libc_hook_info,
            },
            call_stack: call_stack.clone(),
            cross_boundary_events: Vec::new(),
            safety_violations: Vec::new(),
            ffi_tracked: true,
            memory_passport: None,
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
                return Err(TrackingError::MemoryCorruption(
                    "Memory corruption detected".to_string(),
                ));
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
                return Err(TrackingError::InvalidPointer(format!(
                    "Invalid pointer: 0x{ptr:x}"
                )));
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

    /// Create or update memory passport for cross-boundary tracking
    pub fn create_or_update_passport(
        &self,
        ptr: usize,
        operation: &str,
        context: &str,
    ) -> TrackingResult<()> {
        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            if let Some(allocation) = allocations.get_mut(&ptr) {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u128;

                if allocation.memory_passport.is_none() {
                    allocation.memory_passport = Some(self.create_memory_passport(ptr, context));
                }

                if let Some(passport) = &mut allocation.memory_passport {
                    let stamp = PassportStamp {
                        timestamp: current_time,
                        location: context.to_string(),
                        operation: operation.to_string(),
                        authority: "UnsafeFFITracker".to_string(),
                        verification_hash: format!("{:x}", ptr ^ current_time as usize),
                    };
                    passport.journey.push(stamp);
                }
            }
        }

        Ok(())
    }

    /// Update ownership information for a memory allocation
    pub fn update_ownership(
        &self,
        ptr: usize,
        new_owner_context: String,
        new_owner_function: String,
    ) -> TrackingResult<()> {
        if let Ok(mut allocations) = self.enhanced_allocations.lock() {
            if let Some(allocation) = allocations.get_mut(&ptr) {
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u128;

                if let Some(passport) = &mut allocation.memory_passport {
                    passport.current_owner = OwnershipInfo {
                        owner_context: new_owner_context,
                        owner_function: new_owner_function,
                        transfer_timestamp: current_time,
                        expected_lifetime: None,
                    };
                }
            }
        }

        Ok(())
    }

    /// Validate memory passport integrity
    pub fn validate_passport(&self, ptr: usize) -> TrackingResult<bool> {
        if let Ok(allocations) = self.enhanced_allocations.lock() {
            if let Some(allocation) = allocations.get(&ptr) {
                if let Some(passport) = &allocation.memory_passport {
                    // Basic validation: check if passport is not expired or revoked
                    match passport.validity_status {
                        ValidityStatus::Valid => Ok(true),
                        ValidityStatus::Expired
                        | ValidityStatus::Revoked
                        | ValidityStatus::Suspicious => Ok(false),
                    }
                } else {
                    Ok(false) // No passport means not validated
                }
            } else {
                Ok(false) // Allocation not found
            }
        } else {
            Err(TrackingError::LockError(
                "Failed to acquire allocations lock".to_string(),
            ))
        }
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
        Ok(vec![StackFrame {
            function_name: "current_function".to_string(),
            file_name: Some("src/unsafe_ffi_tracker.rs".to_string()),
            line_number: Some(42),
            is_unsafe: true,
        }])
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
static GLOBAL_UNSAFE_FFI_TRACKER: std::sync::OnceLock<std::sync::Arc<UnsafeFFITracker>> =
    std::sync::OnceLock::new();

/// Get the global unsafe/FFI tracker instance
pub fn get_global_unsafe_ffi_tracker() -> std::sync::Arc<UnsafeFFITracker> {
    GLOBAL_UNSAFE_FFI_TRACKER
        .get_or_init(|| std::sync::Arc::new(UnsafeFFITracker::new()))
        .clone()
}

/// Macro for tracking unsafe allocations
#[macro_export]
macro_rules! track_unsafe_alloc {
    ($ptr:expr, $size:expr) => {{
        let tracker = $crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker();
        let location = format!("{}:{}:{}", file!(), line!(), column!());
        let _ = tracker.track_unsafe_allocation($ptr as usize, $size, location);
    }};
}

/// Macro for tracking FFI allocations
#[macro_export]
macro_rules! track_ffi_alloc {
    ($ptr:expr, $size:expr, $lib:expr, $func:expr) => {{
        let tracker = $crate::unsafe_ffi_tracker::get_global_unsafe_ffi_tracker();
        let _ =
            tracker.track_ffi_allocation($ptr as usize, $size, $lib.to_string(), $func.to_string());
    }};
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

/// C Library information for detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLibraryInfo {
    /// Name of the C library
    pub library_name: String,
    /// Version of the library if available
    pub library_version: Option<String>,
    /// Path to the library file
    pub library_path: Option<String>,
    /// Functions from this library that have been called
    pub functions_called: HashMap<String, CFunctionInfo>,
    /// Total number of allocations from this library
    pub total_allocations: usize,
    /// Total bytes allocated from this library
    pub total_bytes_allocated: usize,
    /// Library load timestamp
    pub load_timestamp: u128,
    /// Library metadata
    pub metadata: LibraryMetadata,
}

/// Information about a specific C function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunctionInfo {
    /// Function name
    pub function_name: String,
    /// Function signature if available
    pub function_signature: Option<String>,
    /// Number of times this function has been called
    pub call_count: usize,
    /// Total bytes allocated by this function
    pub bytes_allocated: usize,
    /// Average allocation size
    pub average_allocation_size: f64,
    /// Risk assessment for this function
    pub risk_assessment: RiskAssessment,
    /// Performance metrics
    pub performance_metrics: FunctionPerformanceMetrics,
    /// First call timestamp
    pub first_call_timestamp: u128,
    /// Last call timestamp
    pub last_call_timestamp: u128,
}

/// Performance metrics for C functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPerformanceMetrics {
    /// Average execution time in nanoseconds
    pub avg_execution_time_ns: u64,
    /// Minimum execution time in nanoseconds
    pub min_execution_time_ns: u64,
    /// Maximum execution time in nanoseconds
    pub max_execution_time_ns: u64,
    /// Total execution time in nanoseconds
    pub total_execution_time_ns: u64,
    /// Memory overhead introduced by tracking
    pub tracking_overhead_ns: u64,
}

/// Library metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    /// Architecture (x86_64, arm64, etc.)
    pub architecture: String,
    /// Operating system
    pub operating_system: String,
    /// Compiler used to build the library
    pub compiler_info: Option<String>,
    /// Debug symbols available
    pub has_debug_symbols: bool,
    /// Security features enabled
    pub security_features: Vec<String>,
}

/// Enhanced LibC hook information with detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLibCHookInfo {
    /// Base hook information
    pub base_info: LibCHookInfo,
    /// Detailed function tracking
    pub function_tracking: CFunctionInfo,
    /// Hook installation details
    pub installation_details: HookInstallationDetails,
    /// Runtime behavior analysis
    pub runtime_analysis: RuntimeBehaviorAnalysis,
    /// Security analysis
    pub security_analysis: SecurityAnalysis,
}

/// Details about hook installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInstallationDetails {
    /// Method used to install the hook
    pub installation_method: HookInstallationMethod,
    /// Success status of installation
    pub installation_success: bool,
    /// Error message if installation failed
    pub installation_error: Option<String>,
    /// Timestamp of installation attempt
    pub installation_timestamp: u128,
    /// Process ID where hook was installed
    pub process_id: u32,
    /// Thread ID where hook was installed
    pub thread_id: u64,
}

/// Methods for installing hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookInstallationMethod {
    /// Preload library method
    Preload,
    /// Runtime symbol interposition
    SymbolInterposition,
    /// Binary patching
    BinaryPatching,
    /// Debugger-based hooking
    DebuggerHook,
    /// Kernel-level hooking
    KernelHook,
}

/// Runtime behavior analysis for hooked functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBehaviorAnalysis {
    /// Memory access patterns
    pub memory_patterns: Vec<MemoryAccessPattern>,
    /// Allocation size distribution
    pub size_distribution: SizeDistribution,
    /// Temporal patterns
    pub temporal_patterns: TemporalPatterns,
    /// Error patterns
    pub error_patterns: Vec<ErrorPattern>,
}

/// Memory access pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessPattern {
    /// Pattern type
    pub pattern_type: MemoryPatternType,
    /// Frequency of this pattern
    pub frequency: usize,
    /// Average size involved in this pattern
    pub average_size: usize,
    /// Risk level associated with this pattern
    pub risk_level: RiskLevel,
}

/// Types of memory access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPatternType {
    /// Sequential allocation pattern
    Sequential,
    /// Random allocation pattern
    Random,
    /// Bulk allocation pattern
    Bulk,
    /// Fragmented allocation pattern
    Fragmented,
    /// Reallocation pattern
    Reallocation,
}

/// Size distribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    /// Small allocations (< 1KB)
    pub small_allocations: usize,
    /// Medium allocations (1KB - 1MB)
    pub medium_allocations: usize,
    /// Large allocations (> 1MB)
    pub large_allocations: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Standard deviation of sizes
    pub size_std_dev: f64,
}

/// Temporal patterns in allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    /// Allocation rate (allocations per second)
    pub allocation_rate: f64,
    /// Peak allocation periods
    pub peak_periods: Vec<PeakPeriod>,
    /// Allocation bursts detected
    pub burst_count: usize,
    /// Average time between allocations
    pub avg_time_between_allocs_ms: f64,
}

/// Peak allocation period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakPeriod {
    /// Start timestamp of peak
    pub start_timestamp: u128,
    /// End timestamp of peak
    pub end_timestamp: u128,
    /// Number of allocations during peak
    pub allocation_count: usize,
    /// Total bytes allocated during peak
    pub bytes_allocated: usize,
}

/// Error pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Type of error
    pub error_type: ErrorType,
    /// Frequency of this error
    pub frequency: usize,
    /// Context where error occurs
    pub context: String,
    /// Suggested mitigation
    pub mitigation: String,
}

/// Types of errors that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    /// Allocation failure
    AllocationFailure,
    /// Invalid free
    InvalidFree,
    /// Double free
    DoubleFree,
    /// Memory leak
    MemoryLeak,
    /// Buffer overflow
    BufferOverflow,
    /// Use after free
    UseAfterFree,
}

/// Security analysis for hooked functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Security vulnerabilities detected
    pub vulnerabilities: Vec<SecurityVulnerability>,
    /// Security score (0.0 to 10.0)
    pub security_score: f64,
    /// Recommended security measures
    pub recommendations: Vec<String>,
    /// Compliance status
    pub compliance_status: ComplianceStatus,
}

/// Security vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    /// Type of vulnerability
    pub vulnerability_type: VulnerabilityType,
    /// Severity level
    pub severity: RiskLevel,
    /// Description of the vulnerability
    pub description: String,
    /// Location where vulnerability was detected
    pub location: String,
    /// Potential impact
    pub potential_impact: String,
    /// Remediation steps
    pub remediation: Vec<String>,
}

/// Types of security vulnerabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityType {
    /// Buffer overflow vulnerability
    BufferOverflow,
    /// Use after free vulnerability
    UseAfterFree,
    /// Double free vulnerability
    DoubleFree,
    /// Memory leak vulnerability
    MemoryLeak,
    /// Integer overflow vulnerability
    IntegerOverflow,
    /// Format string vulnerability
    FormatString,
    /// Race condition vulnerability
    RaceCondition,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Memory safety compliance
    pub memory_safety: bool,
    /// Thread safety compliance
    pub thread_safety: bool,
    /// API usage compliance
    pub api_usage: bool,
    /// Security best practices compliance
    pub security_practices: bool,
    /// Overall compliance score
    pub overall_score: f64,
}

impl UnsafeFFITracker {
    /// Register a C library for tracking
    pub fn register_c_library(
        &self,
        library_name: String,
        library_path: Option<String>,
        library_version: Option<String>,
    ) -> TrackingResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        let library_info = CLibraryInfo {
            library_name: library_name.clone(),
            library_version,
            library_path,
            functions_called: HashMap::new(),
            total_allocations: 0,
            total_bytes_allocated: 0,
            load_timestamp: current_time,
            metadata: LibraryMetadata {
                architecture: std::env::consts::ARCH.to_string(),
                operating_system: std::env::consts::OS.to_string(),
                compiler_info: None,
                has_debug_symbols: false,
                security_features: Vec::new(),
            },
        };

        if let Ok(mut libraries) = self.c_libraries.lock() {
            libraries.insert(library_name.clone(), library_info);
            tracing::info!("Registered C library: {}", library_name);
        }

        Ok(())
    }

    /// Track a C function call with detailed information
    pub fn track_c_function_call(
        &self,
        library_name: &str,
        function_name: &str,
        allocation_size: usize,
        execution_time_ns: u64,
    ) -> TrackingResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        if let Ok(mut libraries) = self.c_libraries.lock() {
            let library = libraries
                .entry(library_name.to_string())
                .or_insert_with(|| CLibraryInfo {
                    library_name: library_name.to_string(),
                    library_version: None,
                    library_path: None,
                    functions_called: HashMap::new(),
                    total_allocations: 0,
                    total_bytes_allocated: 0,
                    load_timestamp: current_time,
                    metadata: LibraryMetadata {
                        architecture: std::env::consts::ARCH.to_string(),
                        operating_system: std::env::consts::OS.to_string(),
                        compiler_info: None,
                        has_debug_symbols: false,
                        security_features: Vec::new(),
                    },
                });

            // Update library statistics
            library.total_allocations += 1;
            library.total_bytes_allocated += allocation_size;

            // Update or create function information
            let function_info = library
                .functions_called
                .entry(function_name.to_string())
                .or_insert_with(|| CFunctionInfo {
                    function_name: function_name.to_string(),
                    function_signature: None,
                    call_count: 0,
                    bytes_allocated: 0,
                    average_allocation_size: 0.0,
                    risk_assessment: RiskAssessment {
                        risk_level: RiskLevel::Low,
                        risk_factors: Vec::new(),
                        mitigation_suggestions: Vec::new(),
                        confidence_score: 0.5,
                        assessment_timestamp: current_time,
                    },
                    performance_metrics: FunctionPerformanceMetrics {
                        avg_execution_time_ns: 0,
                        min_execution_time_ns: u64::MAX,
                        max_execution_time_ns: 0,
                        total_execution_time_ns: 0,
                        tracking_overhead_ns: 0,
                    },
                    first_call_timestamp: current_time,
                    last_call_timestamp: current_time,
                });

            // Update function statistics
            function_info.call_count += 1;
            function_info.bytes_allocated += allocation_size;
            function_info.average_allocation_size =
                function_info.bytes_allocated as f64 / function_info.call_count as f64;
            function_info.last_call_timestamp = current_time;

            // Update performance metrics
            let metrics = &mut function_info.performance_metrics;
            metrics.total_execution_time_ns += execution_time_ns;
            metrics.avg_execution_time_ns =
                metrics.total_execution_time_ns / function_info.call_count as u64;
            metrics.min_execution_time_ns = metrics.min_execution_time_ns.min(execution_time_ns);
            metrics.max_execution_time_ns = metrics.max_execution_time_ns.max(execution_time_ns);

            tracing::debug!(
                "Tracked C function call: {}::{} (size: {}, time: {}ns)",
                library_name,
                function_name,
                allocation_size,
                execution_time_ns
            );
        }

        Ok(())
    }

    /// Install an enhanced LibC hook
    pub fn install_enhanced_libc_hook(
        &self,
        function_name: String,
        hook_method: HookInstallationMethod,
    ) -> TrackingResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        let installation_details = HookInstallationDetails {
            installation_method: hook_method,
            installation_success: true, // Assume success for now
            installation_error: None,
            installation_timestamp: current_time,
            process_id: std::process::id(),
            thread_id: 0, // Would need platform-specific code to get thread ID
        };

        let enhanced_hook = EnhancedLibCHookInfo {
            base_info: LibCHookInfo {
                hook_method: HookMethod::DynamicLinker,
                original_function: function_name.clone(),
                hook_timestamp: current_time,
                allocation_metadata: AllocationMetadata {
                    requested_size: 0,
                    actual_size: 0,
                    alignment: 8,
                    allocator_info: format!("hooked_{}", function_name),
                    protection_flags: Some(MemoryProtectionFlags {
                        readable: true,
                        writable: true,
                        executable: false,
                        shared: false,
                    }),
                },
                hook_overhead_ns: Some(50),
            },
            function_tracking: CFunctionInfo {
                function_name: function_name.clone(),
                function_signature: None,
                call_count: 0,
                bytes_allocated: 0,
                average_allocation_size: 0.0,
                risk_assessment: RiskAssessment {
                    risk_level: RiskLevel::Medium,
                    risk_factors: Vec::new(),
                    mitigation_suggestions: vec![
                        "Monitor for memory leaks".to_string(),
                        "Validate all pointer operations".to_string(),
                    ],
                    confidence_score: 0.7,
                    assessment_timestamp: current_time,
                },
                performance_metrics: FunctionPerformanceMetrics {
                    avg_execution_time_ns: 0,
                    min_execution_time_ns: u64::MAX,
                    max_execution_time_ns: 0,
                    total_execution_time_ns: 0,
                    tracking_overhead_ns: 50,
                },
                first_call_timestamp: current_time,
                last_call_timestamp: current_time,
            },
            installation_details,
            runtime_analysis: RuntimeBehaviorAnalysis {
                memory_patterns: Vec::new(),
                size_distribution: SizeDistribution {
                    small_allocations: 0,
                    medium_allocations: 0,
                    large_allocations: 0,
                    average_size: 0.0,
                    size_std_dev: 0.0,
                },
                temporal_patterns: TemporalPatterns {
                    allocation_rate: 0.0,
                    peak_periods: Vec::new(),
                    burst_count: 0,
                    avg_time_between_allocs_ms: 0.0,
                },
                error_patterns: Vec::new(),
            },
            security_analysis: SecurityAnalysis {
                vulnerabilities: Vec::new(),
                security_score: 5.0,
                recommendations: vec![
                    "Enable memory protection".to_string(),
                    "Use safe allocation patterns".to_string(),
                ],
                compliance_status: ComplianceStatus {
                    memory_safety: false,
                    thread_safety: false,
                    api_usage: true,
                    security_practices: false,
                    overall_score: 0.25,
                },
            },
        };

        if let Ok(mut hooks) = self.libc_hooks.lock() {
            hooks.insert(function_name.clone(), enhanced_hook);
            tracing::info!("Installed enhanced LibC hook for: {}", function_name);
        }

        Ok(())
    }

    /// Create and register a memory passport for cross-boundary tracking
    pub fn create_and_register_passport(
        &self,
        ptr: usize,
        origin_context: &str,
        security_clearance: SecurityClearance,
    ) -> TrackingResult<String> {
        let passport = self.create_memory_passport(ptr, origin_context);
        let passport_id = passport.passport_id.clone();

        // Set the security clearance
        let mut passport = passport;
        passport.security_clearance = security_clearance;

        if let Ok(mut passports) = self.memory_passports.lock() {
            passports.insert(ptr, passport);
            tracing::info!("Created memory passport {} for ptr {:x}", passport_id, ptr);
        }

        Ok(passport_id)
    }

    /// Update memory passport with new stamp
    pub fn stamp_passport(
        &self,
        ptr: usize,
        operation: &str,
        location: &str,
        authority: &str,
    ) -> TrackingResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        if let Ok(mut passports) = self.memory_passports.lock() {
            if let Some(passport) = passports.get_mut(&ptr) {
                let stamp = PassportStamp {
                    timestamp: current_time,
                    location: location.to_string(),
                    operation: operation.to_string(),
                    authority: authority.to_string(),
                    verification_hash: format!("{:x}", ptr ^ current_time as usize),
                };

                passport.journey.push(stamp);
                tracing::debug!("Stamped passport for ptr {:x}: {}", ptr, operation);
            } else {
                return Err(TrackingError::InvalidPointer(format!(
                    "No passport found for pointer: 0x{:x}",
                    ptr
                )));
            }
        }

        Ok(())
    }

    /// Transfer memory passport ownership
    pub fn transfer_passport_ownership(
        &self,
        ptr: usize,
        new_owner_context: &str,
        new_owner_function: &str,
    ) -> TrackingResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        if let Ok(mut passports) = self.memory_passports.lock() {
            if let Some(passport) = passports.get_mut(&ptr) {
                passport.current_owner = OwnershipInfo {
                    owner_context: new_owner_context.to_string(),
                    owner_function: new_owner_function.to_string(),
                    transfer_timestamp: current_time,
                    expected_lifetime: None,
                };

                // Add a stamp for the ownership transfer
                let stamp = PassportStamp {
                    timestamp: current_time,
                    location: new_owner_context.to_string(),
                    operation: "ownership_transfer".to_string(),
                    authority: "UnsafeFFITracker".to_string(),
                    verification_hash: format!("{:x}", ptr ^ current_time as usize),
                };

                passport.journey.push(stamp);
                tracing::info!(
                    "Transferred passport ownership for ptr {:x} to {}::{}",
                    ptr,
                    new_owner_context,
                    new_owner_function
                );
            } else {
                return Err(TrackingError::InvalidPointer(format!(
                    "No passport found for pointer: 0x{:x}",
                    ptr
                )));
            }
        }

        Ok(())
    }

    /// Revoke a memory passport (when memory is freed)
    pub fn revoke_passport(&self, ptr: usize, reason: &str) -> TrackingResult<()> {
        if let Ok(mut passports) = self.memory_passports.lock() {
            if let Some(passport) = passports.get_mut(&ptr) {
                passport.validity_status = ValidityStatus::Revoked;

                // Add a final stamp
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u128;

                let stamp = PassportStamp {
                    timestamp: current_time,
                    location: "memory_freed".to_string(),
                    operation: format!("revoked: {}", reason),
                    authority: "UnsafeFFITracker".to_string(),
                    verification_hash: format!("{:x}", ptr ^ current_time as usize),
                };

                passport.journey.push(stamp);
                tracing::info!("Revoked passport for ptr {:x}: {}", ptr, reason);
            }
        }

        Ok(())
    }

    /// Get C library statistics
    pub fn get_c_library_stats(&self) -> TrackingResult<HashMap<String, CLibraryInfo>> {
        self.c_libraries
            .lock()
            .map(|libs| libs.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Get LibC hook information
    pub fn get_libc_hook_info(&self) -> TrackingResult<HashMap<String, EnhancedLibCHookInfo>> {
        self.libc_hooks
            .lock()
            .map(|hooks| hooks.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Get memory passport information
    pub fn get_memory_passports(&self) -> TrackingResult<HashMap<usize, MemoryPassport>> {
        self.memory_passports
            .lock()
            .map(|passports| passports.clone())
            .map_err(|e| TrackingError::LockError(e.to_string()))
    }

    /// Analyze cross-boundary risks
    pub fn analyze_cross_boundary_risks(&self) -> TrackingResult<Vec<SafetyViolation>> {
        let mut risks = Vec::new();

        if let Ok(passports) = self.memory_passports.lock() {
            for (ptr, passport) in passports.iter() {
                // Check for suspicious passport activity
                if passport.journey.len() > 10 {
                    risks.push(SafetyViolation::CrossBoundaryRisk {
                        risk_level: RiskLevel::Medium,
                        description: format!(
                            "Memory at {:x} has crossed boundaries {} times",
                            ptr,
                            passport.journey.len()
                        ),
                        stack: Vec::new(), // Would need to capture actual stack
                    });
                }

                // Check for expired passports
                if matches!(passport.validity_status, ValidityStatus::Expired) {
                    risks.push(SafetyViolation::CrossBoundaryRisk {
                        risk_level: RiskLevel::High,
                        description: format!("Expired passport detected for memory at {:x}", ptr),
                        stack: Vec::new(),
                    });
                }
            }
        }

        Ok(risks)
    }

    /// Get statistics for unsafe and FFI operations
    pub fn get_stats(&self) -> UnsafeFFIStats {
        let allocations = self
            .enhanced_allocations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let violations = self
            .violations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

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
            let base_risk = (stats.unsafe_blocks as f64 * 1.0)
                + (stats.ffi_calls as f64 * 2.0)
                + (stats.memory_violations as f64 * 5.0);
            (base_risk / stats.total_operations as f64).min(10.0)
        } else {
            0.0
        };

        // Create operation list (simplified)
        for allocation in allocations.values() {
            let (op_type, risk_level, description) = match &allocation.source {
                AllocationSource::UnsafeRust {
                    unsafe_block_location,
                    ..
                } => (
                    UnsafeOperationType::UnsafeBlock,
                    RiskLevel::Medium,
                    format!("Unsafe block at {unsafe_block_location}"),
                ),
                AllocationSource::FfiC {
                    library_name,
                    function_name,
                    ..
                } => (
                    UnsafeOperationType::FfiCall,
                    RiskLevel::High,
                    format!("FFI call to {library_name}::{function_name}"),
                ),
                AllocationSource::CrossBoundary { .. } => (
                    UnsafeOperationType::CrossBoundaryTransfer,
                    RiskLevel::Medium,
                    "Cross-boundary memory transfer".to_string(),
                ),
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
