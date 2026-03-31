//! Unsafe/FFI tracking for unified tracker
//!
//! This module provides comprehensive unsafe Rust and FFI memory tracking
//! with the innovative Memory Passport system for cross-boundary tracking.

use crate::core::error::{MemScopeError, MemoryOperation, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unsafe/FFI tracking configuration
#[derive(Debug, Clone)]
pub struct UnsafeTrackingConfig {
    /// Enable tracking of unsafe Rust allocations
    pub track_unsafe_allocations: bool,
    /// Enable tracking of FFI allocations
    pub track_ffi_allocations: bool,
    /// Enable double-free detection
    pub detect_double_free: bool,
    /// Enable memory leak detection
    pub detect_memory_leaks: bool,
    /// Enable cross-boundary tracking with memory passport
    pub enable_memory_passport: bool,
}

impl Default for UnsafeTrackingConfig {
    fn default() -> Self {
        Self {
            track_unsafe_allocations: true,
            track_ffi_allocations: true,
            detect_double_free: true,
            detect_memory_leaks: true,
            enable_memory_passport: true,
        }
    }
}

impl UnsafeTrackingConfig {
    /// Create configuration for minimal tracking
    pub fn minimal() -> Self {
        Self {
            track_unsafe_allocations: true,
            track_ffi_allocations: true,
            detect_double_free: false,
            detect_memory_leaks: false,
            enable_memory_passport: false,
        }
    }

    /// Create configuration for comprehensive tracking
    pub fn comprehensive() -> Self {
        Self {
            track_unsafe_allocations: true,
            track_ffi_allocations: true,
            detect_double_free: true,
            detect_memory_leaks: true,
            enable_memory_passport: true,
        }
    }
}

/// Allocation source type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationSource {
    /// Safe Rust allocation
    SafeRust,
    /// Unsafe Rust allocation
    UnsafeRust { location: String },
    /// FFI allocation
    Ffi { library: String, function: String },
}

/// Safety violation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyViolation {
    /// Double free detected
    DoubleFree {
        ptr: usize,
        first_free_time_ms: u64,
        second_free_time_ms: u64,
    },
    /// Invalid free (pointer not in allocation table)
    InvalidFree { ptr: usize, time_ms: u64 },
    /// Potential memory leak
    PotentialLeak {
        ptr: usize,
        allocation_time_ms: u64,
        detection_time_ms: u64,
        size: usize,
    },
    /// Cross-boundary risk detected
    CrossBoundaryRisk {
        ptr: usize,
        from_context: String,
        to_context: String,
        risk_level: String,
    },
}

impl SafetyViolation {
    /// Get violation severity
    pub fn severity(&self) -> ViolationSeverity {
        match self {
            Self::DoubleFree { .. } => ViolationSeverity::Critical,
            Self::InvalidFree { .. } => ViolationSeverity::High,
            Self::PotentialLeak { .. } => ViolationSeverity::Medium,
            Self::CrossBoundaryRisk { .. } => ViolationSeverity::High,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::DoubleFree { ptr, .. } => {
                format!("Double free detected at pointer 0x{:x}", ptr)
            }
            Self::InvalidFree { ptr, .. } => {
                format!("Invalid free of pointer 0x{:x} (not allocated)", ptr)
            }
            Self::PotentialLeak { ptr, size, .. } => {
                format!("Potential leak of {} bytes at pointer 0x{:x}", size, ptr)
            }
            Self::CrossBoundaryRisk {
                ptr,
                from_context,
                to_context,
                ..
            } => {
                format!(
                    "Cross-boundary risk at 0x{:x}: {} -> {}",
                    ptr, from_context, to_context
                )
            }
        }
    }
}

/// Violation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory pointer
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Allocation source
    pub source: AllocationSource,
    /// Allocation timestamp (milliseconds since Unix epoch)
    pub allocated_at_ms: u64,
    /// Deallocation timestamp (milliseconds since Unix epoch, if deallocated)
    pub deallocated_at_ms: Option<u64>,
    /// Whether this allocation is currently active
    pub is_active: bool,
}

impl AllocationInfo {
    /// Create new allocation info
    pub fn new(ptr: usize, size: usize, source: AllocationSource) -> Self {
        let now_ms = Self::now_ms();
        Self {
            ptr,
            size,
            source,
            allocated_at_ms: now_ms,
            deallocated_at_ms: None,
            is_active: true,
        }
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Mark as deallocated
    pub fn mark_deallocated(&mut self) {
        self.deallocated_at_ms = Some(Self::now_ms());
        self.is_active = false;
    }

    /// Get allocation age in milliseconds
    pub fn age_ms(&self) -> u64 {
        let now_ms = Self::now_ms();
        now_ms.saturating_sub(self.allocated_at_ms)
    }

    /// Check if allocation is leaked (active for too long)
    pub fn is_leaked(&self, threshold_ms: u64) -> bool {
        self.is_active && self.age_ms() > threshold_ms
    }
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
    /// Validity status of passport
    pub validity_status: ValidityStatus,
    /// Security clearance level
    pub security_clearance: SecurityClearance,
}

impl MemoryPassport {
    /// Create new memory passport
    pub fn new(ptr: usize, size: usize, source: AllocationSource) -> Self {
        let passport_id = format!("passport_{:x}_{}", ptr, Self::now_ms());

        let origin = AllocationOrigin::new(ptr, size, source.clone());
        let owner = OwnershipInfo::new(source);

        Self {
            passport_id,
            origin,
            journey: Vec::new(),
            current_owner: owner,
            validity_status: ValidityStatus::Valid,
            security_clearance: SecurityClearance::Public,
        }
    }

    /// Add a journey stamp
    pub fn add_stamp(&mut self, location: String, operation: String) {
        let stamp = PassportStamp::new(location, operation);
        self.journey.push(stamp);
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_context: String, new_function: String) {
        let context_clone = new_context.clone();
        self.current_owner = OwnershipInfo::with_context(new_context, new_function);
        self.add_stamp(context_clone, "ownership_transfer".to_string());
    }

    /// Revoke passport (memory freed)
    pub fn revoke(&mut self) {
        self.validity_status = ValidityStatus::Revoked;
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Information about where memory was originally allocated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationOrigin {
    /// Context where allocation occurred (Rust/FFI)
    pub context: String,
    /// Function that performed allocation
    pub allocator_function: String,
    /// Timestamp of original allocation
    pub timestamp: u64,
}

impl AllocationOrigin {
    /// Create new allocation origin
    pub fn new(_ptr: usize, _size: usize, source: AllocationSource) -> Self {
        let (context, function) = match source {
            AllocationSource::SafeRust => ("rust_safe".to_string(), "alloc".to_string()),
            AllocationSource::UnsafeRust { location } => ("rust_unsafe".to_string(), location),
            AllocationSource::Ffi { library, function } => (library, function),
        };

        Self {
            context,
            allocator_function: function,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

/// A stamp in the memory passport journey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportStamp {
    /// Timestamp of this checkpoint
    pub timestamp: u64,
    /// Location/context of checkpoint
    pub location: String,
    /// Operation performed at this checkpoint
    pub operation: String,
    /// Cryptographic hash for verification
    pub verification_hash: String,
}

impl PassportStamp {
    /// Create new passport stamp
    pub fn new(location: String, operation: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let verification_hash = format!("{}:{}:{}", timestamp, location, operation);

        Self {
            timestamp,
            location,
            operation,
            verification_hash,
        }
    }
}

/// Current ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    /// Current owner context (Rust/FFI)
    pub owner_context: String,
    /// Function/module that owns the memory
    pub owner_function: String,
    /// Ownership transfer timestamp
    pub transfer_timestamp: u64,
}

impl OwnershipInfo {
    /// Create new ownership info
    pub fn new(source: AllocationSource) -> Self {
        let (context, function) = match source {
            AllocationSource::SafeRust => ("rust_safe".to_string(), "alloc".to_string()),
            AllocationSource::UnsafeRust { location } => ("rust_unsafe".to_string(), location),
            AllocationSource::Ffi { library, function } => (library, function),
        };

        Self {
            owner_context: context,
            owner_function: function,
            transfer_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Create ownership info with specific context
    pub fn with_context(context: String, function: String) -> Self {
        Self {
            owner_context: context,
            owner_function: function,
            transfer_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

/// Validity status of a memory passport
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Unsafe/FFI tracking statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnsafeTrackingStats {
    /// Total number of allocations tracked
    pub total_allocations: usize,
    /// Number of unsafe Rust allocations
    pub unsafe_allocations: usize,
    /// Number of FFI allocations
    pub ffi_allocations: usize,
    /// Total bytes tracked
    pub total_bytes_tracked: usize,
    /// Current active allocations
    pub active_allocations: usize,
    /// Number of safety violations
    pub total_violations: usize,
    /// Number of double-free violations
    pub double_free_count: usize,
    /// Number of invalid-free violations
    pub invalid_free_count: usize,
    /// Number of potential leaks
    pub leak_count: usize,
    /// Number of active memory passports
    pub active_passports: usize,
}

impl UnsafeTrackingStats {
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Allocations: {} (Unsafe: {}, FFI: {}), Bytes: {}, Active: {}, Violations: {} (Double-free: {}, Invalid-free: {}, Leaks: {}), Passports: {}",
            self.total_allocations,
            self.unsafe_allocations,
            self.ffi_allocations,
            self.total_bytes_tracked,
            self.active_allocations,
            self.total_violations,
            self.double_free_count,
            self.invalid_free_count,
            self.leak_count,
            self.active_passports
        )
    }
}

/// Simplified unsafe/FFI tracker with memory passport system
pub struct UnsafeTracker {
    config: UnsafeTrackingConfig,
    allocations: Arc<Mutex<HashMap<usize, AllocationInfo>>>,
    passports: Arc<Mutex<HashMap<usize, MemoryPassport>>>,
    violations: Arc<Mutex<Vec<SafetyViolation>>>,
}

impl UnsafeTracker {
    /// Create new unsafe tracker with default configuration
    pub fn new() -> Self {
        Self {
            config: UnsafeTrackingConfig::default(),
            allocations: Arc::new(Mutex::new(HashMap::new())),
            passports: Arc::new(Mutex::new(HashMap::new())),
            violations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create new unsafe tracker with custom configuration
    pub fn with_config(config: UnsafeTrackingConfig) -> Self {
        Self {
            config,
            allocations: Arc::new(Mutex::new(HashMap::new())),
            passports: Arc::new(Mutex::new(HashMap::new())),
            violations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Track an unsafe Rust allocation
    pub fn track_unsafe_allocation(&self, ptr: usize, size: usize, location: String) -> Result<()> {
        if !self.config.track_unsafe_allocations {
            return Ok(());
        }

        let source = AllocationSource::UnsafeRust { location };
        let allocation = AllocationInfo::new(ptr, size, source.clone());

        if let Ok(mut passports) = self.passports.lock() {
            if self.config.enable_memory_passport {
                let passport = MemoryPassport::new(ptr, size, source.clone());
                passports.insert(ptr, passport);
            }
        }

        if let Ok(mut allocations) = self.allocations.lock() {
            allocations.insert(ptr, allocation);
            tracing::info!("Tracked unsafe allocation at 0x{:x} (size: {})", ptr, size);
            Ok(())
        } else {
            Err(MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                "Failed to acquire allocations lock",
            ))
        }
    }

    /// Track an FFI allocation
    pub fn track_ffi_allocation(
        &self,
        ptr: usize,
        size: usize,
        library: String,
        function: String,
    ) -> Result<()> {
        if !self.config.track_ffi_allocations {
            return Ok(());
        }

        let source = AllocationSource::Ffi {
            library: library.clone(),
            function: function.clone(),
        };
        let allocation = AllocationInfo::new(ptr, size, source.clone());

        if let Ok(mut passports) = self.passports.lock() {
            if self.config.enable_memory_passport {
                let passport = MemoryPassport::new(ptr, size, source);
                passports.insert(ptr, passport);
            }
        }

        if let Ok(mut allocations) = self.allocations.lock() {
            allocations.insert(ptr, allocation);
            tracing::info!(
                "Tracked FFI allocation at 0x{:x} (size: {}, lib: {}, func: {})",
                ptr,
                size,
                library,
                function
            );
            Ok(())
        } else {
            Err(MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                "Failed to acquire allocations lock",
            ))
        }
    }

    /// Track a deallocation
    pub fn track_deallocation(&self, ptr: usize) -> Result<()> {
        if let Ok(mut allocations) = self.allocations.lock() {
            if let Some(mut allocation) = allocations.remove(&ptr) {
                allocation.mark_deallocated();

                if let Ok(mut passports) = self.passports.lock() {
                    if let Some(mut passport) = passports.remove(&ptr) {
                        passport.revoke();
                    }
                }

                Ok(())
            } else {
                let violation = SafetyViolation::InvalidFree {
                    ptr,
                    time_ms: AllocationInfo::now_ms(),
                };

                if let Ok(mut violations) = self.violations.lock() {
                    violations.push(violation.clone());
                }

                tracing::error!("Invalid free detected at 0x{:x}", ptr);
                Err(MemScopeError::memory_with_context(
                    MemoryOperation::Deallocation,
                    format!("Invalid free of pointer 0x{:x}", ptr),
                    "pointer not in allocation table",
                ))
            }
        } else {
            Err(MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                "Failed to acquire allocations lock",
            ))
        }
    }

    /// Record a cross-boundary event
    pub fn record_boundary_event(
        &self,
        ptr: usize,
        from_context: String,
        to_context: String,
    ) -> Result<()> {
        if let Ok(mut passports) = self.passports.lock() {
            if let Some(passport) = passports.get_mut(&ptr) {
                passport.transfer_ownership(to_context.clone(), "boundary_transfer".to_string());
                tracing::info!(
                    "Recorded boundary event for 0x{:x}: {} -> {}",
                    ptr,
                    from_context,
                    to_context
                );
                Ok(())
            } else {
                Err(MemScopeError::memory_with_context(
                    MemoryOperation::Tracking,
                    format!("No passport found for pointer 0x{:x}", ptr),
                    "cross-boundary tracking requires memory passport",
                ))
            }
        } else {
            Err(MemScopeError::system(
                crate::core::error::SystemErrorType::Locking,
                "Failed to acquire passports lock",
            ))
        }
    }

    /// Get all safety violations
    pub fn get_violations(&self) -> Vec<SafetyViolation> {
        if let Ok(violations) = self.violations.lock() {
            violations.clone()
        } else {
            Vec::new()
        }
    }

    /// Detect memory leaks
    pub fn detect_leaks(&self, threshold_ms: u64) -> Vec<SafetyViolation> {
        let mut leaks = Vec::new();

        if let Ok(allocations) = self.allocations.lock() {
            for allocation in allocations.values() {
                if allocation.is_leaked(threshold_ms) {
                    leaks.push(SafetyViolation::PotentialLeak {
                        ptr: allocation.ptr,
                        allocation_time_ms: allocation.allocated_at_ms,
                        detection_time_ms: AllocationInfo::now_ms(),
                        size: allocation.size,
                    });
                }
            }
        }

        if !leaks.is_empty() {
            if let Ok(mut violations) = self.violations.lock() {
                violations.extend(leaks.clone());
            }
        }

        leaks
    }

    /// Get all allocations
    pub fn get_allocations(&self) -> Vec<AllocationInfo> {
        if let Ok(allocations) = self.allocations.lock() {
            allocations.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get active allocations
    pub fn get_active_allocations(&self) -> Vec<AllocationInfo> {
        self.get_allocations()
            .into_iter()
            .filter(|alloc| alloc.is_active)
            .collect()
    }

    /// Get memory passport for a pointer
    pub fn get_passport(&self, ptr: usize) -> Option<MemoryPassport> {
        if let Ok(passports) = self.passports.lock() {
            passports.get(&ptr).cloned()
        } else {
            None
        }
    }

    /// Get all active passports
    pub fn get_active_passports(&self) -> Vec<MemoryPassport> {
        if let Ok(passports) = self.passports.lock() {
            passports
                .values()
                .filter(|p| p.validity_status == ValidityStatus::Valid)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get tracking statistics
    pub fn get_stats(&self) -> UnsafeTrackingStats {
        let allocations = self.get_allocations();
        let violations = self.get_violations();
        let active_passports = self.get_active_passports();

        let unsafe_count = allocations
            .iter()
            .filter(|alloc| matches!(alloc.source, AllocationSource::UnsafeRust { .. }))
            .count();

        let ffi_count = allocations
            .iter()
            .filter(|alloc| matches!(alloc.source, AllocationSource::Ffi { .. }))
            .count();

        let total_size: usize = allocations.iter().map(|alloc| alloc.size).sum();

        let active_count = allocations.iter().filter(|alloc| alloc.is_active).count();

        let double_free_count = violations
            .iter()
            .filter(|v| matches!(v, SafetyViolation::DoubleFree { .. }))
            .count();

        let invalid_free_count = violations
            .iter()
            .filter(|v| matches!(v, SafetyViolation::InvalidFree { .. }))
            .count();

        let leak_count = violations
            .iter()
            .filter(|v| matches!(v, SafetyViolation::PotentialLeak { .. }))
            .count();

        UnsafeTrackingStats {
            total_allocations: allocations.len(),
            unsafe_allocations: unsafe_count,
            ffi_allocations: ffi_count,
            total_bytes_tracked: total_size,
            active_allocations: active_count,
            total_violations: violations.len(),
            double_free_count,
            invalid_free_count,
            leak_count,
            active_passports: active_passports.len(),
        }
    }

    /// Clear all tracking data
    pub fn clear(&self) {
        if let Ok(mut allocations) = self.allocations.lock() {
            allocations.clear();
        }
        if let Ok(mut passports) = self.passports.lock() {
            passports.clear();
        }
        if let Ok(mut violations) = self.violations.lock() {
            violations.clear();
        }
    }
}

impl Default for UnsafeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_info_creation() {
        let info = AllocationInfo::new(0x1000, 1024, AllocationSource::SafeRust);
        assert_eq!(info.ptr, 0x1000);
        assert_eq!(info.size, 1024);
        assert!(info.is_active);
        assert!(info.deallocated_at_ms.is_none());
    }

    #[test]
    fn test_allocation_info_deallocation() {
        let mut info = AllocationInfo::new(0x1000, 1024, AllocationSource::SafeRust);
        assert!(info.is_active);
        info.mark_deallocated();
        assert!(!info.is_active);
        assert!(info.deallocated_at_ms.is_some());
    }

    #[test]
    fn test_allocation_info_leak_detection() {
        let info = AllocationInfo::new(0x1000, 1024, AllocationSource::SafeRust);
        assert!(!info.is_leaked(1000));
        assert!(info.is_leaked(10000));
    }

    #[test]
    fn test_memory_passport_creation() {
        let passport = MemoryPassport::new(0x1000, 1024, AllocationSource::SafeRust);
        assert_eq!(passport.validity_status, ValidityStatus::Valid);
        assert_eq!(passport.security_clearance, SecurityClearance::Public);
        assert_eq!(passport.journey.len(), 0);
    }

    #[test]
    fn test_memory_passport_stamp() {
        let mut passport = MemoryPassport::new(0x1000, 1024, AllocationSource::SafeRust);
        passport.add_stamp("test_location".to_string(), "test_operation".to_string());
        assert_eq!(passport.journey.len(), 1);
    }

    #[test]
    fn test_memory_passport_ownership_transfer() {
        let mut passport = MemoryPassport::new(0x1000, 1024, AllocationSource::SafeRust);
        passport.transfer_ownership("new_context".to_string(), "new_function".to_string());
        assert_eq!(passport.current_owner.owner_context, "new_context");
        assert_eq!(passport.journey.len(), 1);
    }

    #[test]
    fn test_memory_passport_revoke() {
        let mut passport = MemoryPassport::new(0x1000, 1024, AllocationSource::SafeRust);
        assert_eq!(passport.validity_status, ValidityStatus::Valid);
        passport.revoke();
        assert_eq!(passport.validity_status, ValidityStatus::Revoked);
    }

    #[test]
    fn test_unsafe_tracker_creation() {
        let tracker = UnsafeTracker::new();
        let _ = tracker;
    }

    #[test]
    fn test_unsafe_tracker_with_config() {
        let config = UnsafeTrackingConfig::minimal();
        let tracker = UnsafeTracker::with_config(config);
        let _ = tracker;
    }

    #[test]
    fn test_track_unsafe_allocation() {
        let tracker = UnsafeTracker::new();
        let result = tracker.track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string());
        assert!(result.is_ok());

        let allocations = tracker.get_allocations();
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].ptr, 0x1000);

        let passport = tracker.get_passport(0x1000);
        assert!(passport.is_some());
    }

    #[test]
    fn test_track_ffi_allocation() {
        let tracker = UnsafeTracker::new();
        let result =
            tracker.track_ffi_allocation(0x2000, 2048, "libc".to_string(), "malloc".to_string());
        assert!(result.is_ok());

        let allocations = tracker.get_allocations();
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].ptr, 0x2000);

        let passport = tracker.get_passport(0x2000);
        assert!(passport.is_some());
    }

    #[test]
    fn test_track_deallocation_valid() {
        let tracker = UnsafeTracker::new();
        tracker
            .track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string())
            .unwrap();

        let result = tracker.track_deallocation(0x1000);
        assert!(result.is_ok());

        let allocations = tracker.get_allocations();
        assert_eq!(allocations.len(), 0);

        let passport = tracker.get_passport(0x1000);
        assert!(passport.is_none());
    }

    #[test]
    fn test_track_deallocation_invalid() {
        let tracker = UnsafeTracker::new();
        let result = tracker.track_deallocation(0x1000);
        assert!(result.is_err());

        let violations = tracker.get_violations();
        assert_eq!(violations.len(), 1);
        assert!(matches!(violations[0], SafetyViolation::InvalidFree { .. }));
    }

    #[test]
    fn test_record_boundary_event() {
        let tracker = UnsafeTracker::new();
        tracker
            .track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string())
            .unwrap();

        let result = tracker.record_boundary_event(0x1000, "rust".to_string(), "ffi".to_string());
        assert!(result.is_ok());

        let passport = tracker.get_passport(0x1000);
        assert!(passport.is_some());
        assert_eq!(passport.unwrap().current_owner.owner_context, "ffi");
    }

    #[test]
    fn test_detect_leaks() {
        let tracker = UnsafeTracker::new();
        tracker
            .track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string())
            .unwrap();

        let leaks = tracker.detect_leaks(10000);
        assert_eq!(leaks.len(), 1);
        assert!(matches!(leaks[0], SafetyViolation::PotentialLeak { .. }));
    }

    #[test]
    fn test_get_stats() {
        let tracker = UnsafeTracker::new();
        tracker
            .track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string())
            .unwrap();
        tracker
            .track_ffi_allocation(0x2000, 2048, "libc".to_string(), "malloc".to_string())
            .unwrap();

        let stats = tracker.get_stats();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.unsafe_allocations, 1);
        assert_eq!(stats.ffi_allocations, 1);
        assert_eq!(stats.total_bytes_tracked, 3072);
        assert_eq!(stats.active_allocations, 2);
        assert_eq!(stats.active_passports, 2);
    }

    #[test]
    fn test_clear() {
        let tracker = UnsafeTracker::new();
        tracker
            .track_unsafe_allocation(0x1000, 1024, "test.rs:42".to_string())
            .unwrap();

        tracker.clear();

        let allocations = tracker.get_allocations();
        assert_eq!(allocations.len(), 0);

        let violations = tracker.get_violations();
        assert_eq!(violations.len(), 0);

        let passports = tracker.get_active_passports();
        assert_eq!(passports.len(), 0);
    }

    #[test]
    fn test_violation_severity() {
        let double_free = SafetyViolation::DoubleFree {
            ptr: 0x1000,
            first_free_time_ms: 1000,
            second_free_time_ms: 2000,
        };
        assert_eq!(double_free.severity(), ViolationSeverity::Critical);

        let invalid_free = SafetyViolation::InvalidFree {
            ptr: 0x1000,
            time_ms: 1000,
        };
        assert_eq!(invalid_free.severity(), ViolationSeverity::High);

        let leak = SafetyViolation::PotentialLeak {
            ptr: 0x1000,
            allocation_time_ms: 1000,
            detection_time_ms: 2000,
            size: 1024,
        };
        assert_eq!(leak.severity(), ViolationSeverity::Medium);
    }

    #[test]
    fn test_violation_description() {
        let double_free = SafetyViolation::DoubleFree {
            ptr: 0x1000,
            first_free_time_ms: 1000,
            second_free_time_ms: 2000,
        };
        let desc = double_free.description();
        assert!(desc.contains("Double free"));
        assert!(desc.contains("0x1000"));
    }

    #[test]
    fn test_stats_summary() {
        let stats = UnsafeTrackingStats {
            total_allocations: 10,
            unsafe_allocations: 3,
            ffi_allocations: 4,
            total_bytes_tracked: 10240,
            active_allocations: 5,
            total_violations: 2,
            double_free_count: 1,
            invalid_free_count: 1,
            leak_count: 0,
            active_passports: 5,
        };
        let summary = stats.summary();
        assert!(summary.contains("Allocations: 10"));
        assert!(summary.contains("Unsafe: 3"));
        assert!(summary.contains("FFI: 4"));
        assert!(summary.contains("Passports: 5"));
    }
}
