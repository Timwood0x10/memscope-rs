//! Memory Passport Tracking System for FFI Boundary Memory Management
//!
//! This module implements comprehensive memory passport tracking for memory that crosses
//! FFI boundaries, including lifecycle event recording and leak detection at shutdown.

use crate::analysis::unsafe_ffi_tracker::StackFrame;
use crate::core::types::{TrackingResult, TrackingError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Memory passport for tracking cross-FFI boundary memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    /// Unique passport identifier
    pub passport_id: String,
    /// Memory allocation pointer
    pub allocation_ptr: usize,
    /// Size in bytes
    pub size_bytes: usize,
    /// Current status at program shutdown
    pub status_at_shutdown: PassportStatus,
    /// Complete lifecycle events recorded
    pub lifecycle_events: Vec<PassportEvent>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Status of memory passport at program shutdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PassportStatus {
    /// Memory properly freed by Rust
    FreedByRust,
    /// Memory handed over to FFI and not returned
    HandoverToFfi,
    /// Memory freed by foreign code
    FreedByForeign,
    /// Memory reclaimed by Rust from FFI
    ReclaimedByRust,
    /// Memory still in foreign custody (confirmed leak)
    InForeignCustody,
    /// Status unknown or corrupted
    Unknown,
}

/// Lifecycle event in memory passport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportEvent {
    /// Event type
    pub event_type: PassportEventType,
    /// Timestamp of event
    pub timestamp: u64,
    /// Context where event occurred
    pub context: String,
    /// Call stack at event time
    pub call_stack: Vec<StackFrame>,
    /// Additional event metadata
    pub metadata: HashMap<String, String>,
    /// Event sequence number
    pub sequence_number: u32,
}

/// Types of passport events for lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PassportEventType {
    /// Memory allocated in Rust
    AllocatedInRust,
    /// Memory handed over to FFI (HandoverToFfi)
    HandoverToFfi,
    /// Memory freed by foreign code (FreedByForeign)
    FreedByForeign,
    /// Memory reclaimed by Rust (ReclaimedByRust)
    ReclaimedByRust,
    /// Memory accessed across boundary
    BoundaryAccess,
    /// Memory ownership transferred
    OwnershipTransfer,
    /// Memory validation check
    ValidationCheck,
    /// Memory corruption detected
    CorruptionDetected,
}

/// Memory passport tracker for comprehensive FFI boundary tracking
pub struct MemoryPassportTracker {
    /// Active memory passports
    passports: Arc<Mutex<HashMap<usize, MemoryPassport>>>,
    /// Passport creation sequence
    sequence_counter: Arc<Mutex<u32>>,
    /// Event sequence counter
    event_sequence: Arc<Mutex<u32>>,
    /// Tracker configuration
    config: PassportTrackerConfig,
    /// Statistics tracking
    stats: Arc<Mutex<PassportTrackerStats>>,
}

/// Configuration for passport tracker
#[derive(Debug, Clone)]
pub struct PassportTrackerConfig {
    /// Enable detailed event logging
    pub detailed_logging: bool,
    /// Maximum number of events per passport
    pub max_events_per_passport: usize,
    /// Enable automatic leak detection
    pub enable_leak_detection: bool,
    /// Enable passport validation
    pub enable_validation: bool,
    /// Maximum number of passports to track
    pub max_passports: usize,
}

impl Default for PassportTrackerConfig {
    fn default() -> Self {
        Self {
            detailed_logging: true,
            max_events_per_passport: 100,
            enable_leak_detection: true,
            enable_validation: true,
            max_passports: 10000,
        }
    }
}

/// Statistics for passport tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassportTrackerStats {
    /// Total passports created
    pub total_passports_created: usize,
    /// Active passports
    pub active_passports: usize,
    /// Passports by status
    pub passports_by_status: HashMap<String, usize>,
    /// Total events recorded
    pub total_events_recorded: usize,
    /// Leaks detected at shutdown
    pub leaks_detected: usize,
    /// Validation failures
    pub validation_failures: usize,
    /// Tracker start time
    pub tracker_start_time: u64,
}

/// Leak detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDetectionResult {
    /// Leaked passport IDs
    pub leaked_passports: Vec<String>,
    /// Total leaks detected
    pub total_leaks: usize,
    /// Leak details
    pub leak_details: Vec<LeakDetail>,
    /// Detection timestamp
    pub detected_at: u64,
}

/// Details of a detected leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDetail {
    /// Passport ID
    pub passport_id: String,
    /// Memory address
    pub memory_address: usize,
    /// Size of leaked memory
    pub size_bytes: usize,
    /// Last known context
    pub last_context: String,
    /// Time since last event
    pub time_since_last_event: u64,
    /// Lifecycle summary
    pub lifecycle_summary: String,
}

impl MemoryPassportTracker {
    /// Create new memory passport tracker
    pub fn new(config: PassportTrackerConfig) -> Self {
        tracing::info!("📋 Initializing Memory Passport Tracker");
        tracing::info!("   • Detailed logging: {}", config.detailed_logging);
        tracing::info!("   • Max events per passport: {}", config.max_events_per_passport);
        tracing::info!("   • Leak detection: {}", config.enable_leak_detection);
        tracing::info!("   • Validation: {}", config.enable_validation);

        Self {
            passports: Arc::new(Mutex::new(HashMap::new())),
            sequence_counter: Arc::new(Mutex::new(0)),
            event_sequence: Arc::new(Mutex::new(0)),
            config,
            stats: Arc::new(Mutex::new(PassportTrackerStats {
                tracker_start_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                ..Default::default()
            })),
        }
    }

    /// Create memory passport for FFI boundary tracking
    pub fn create_passport(
        &self,
        allocation_ptr: usize,
        size_bytes: usize,
        initial_context: String,
    ) -> TrackingResult<String> {
        let passport_id = self.generate_passport_id(allocation_ptr)?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create initial event
        let initial_event = PassportEvent {
            event_type: PassportEventType::AllocatedInRust,
            timestamp: current_time,
            context: initial_context,
            call_stack: self.capture_call_stack()?,
            metadata: HashMap::new(),
            sequence_number: self.get_next_event_sequence(),
        };

        let passport = MemoryPassport {
            passport_id: passport_id.clone(),
            allocation_ptr,
            size_bytes,
            status_at_shutdown: PassportStatus::Unknown,
            lifecycle_events: vec![initial_event],
            created_at: current_time,
            updated_at: current_time,
            metadata: HashMap::new(),
        };

        // Store passport
        if let Ok(mut passports) = self.passports.lock() {
            // Check passport limit
            if passports.len() >= self.config.max_passports {
                return Err(TrackingError::ResourceExhausted(
                    "Maximum passport limit reached".to_string()
                ));
            }
            passports.insert(allocation_ptr, passport);
        } else {
            return Err(TrackingError::LockContention("Failed to lock passports".to_string()));
        }

        // Update statistics
        self.update_stats_passport_created();

        if self.config.detailed_logging {
            tracing::info!("📋 Created passport: {} for 0x{:x} ({} bytes)", 
                passport_id, allocation_ptr, size_bytes);
        }

        Ok(passport_id)
    }

    /// Record HandoverToFfi event
    pub fn record_handover_to_ffi(
        &self,
        allocation_ptr: usize,
        ffi_context: String,
        function_name: String,
    ) -> TrackingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("ffi_function".to_string(), function_name);
        
        self.record_passport_event(
            allocation_ptr,
            PassportEventType::HandoverToFfi,
            ffi_context,
            metadata,
        )
    }

    /// Record FreedByForeign event
    pub fn record_freed_by_foreign(
        &self,
        allocation_ptr: usize,
        foreign_context: String,
        free_function: String,
    ) -> TrackingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("free_function".to_string(), free_function);
        
        self.record_passport_event(
            allocation_ptr,
            PassportEventType::FreedByForeign,
            foreign_context,
            metadata,
        )
    }

    /// Record ReclaimedByRust event
    pub fn record_reclaimed_by_rust(
        &self,
        allocation_ptr: usize,
        rust_context: String,
        reclaim_reason: String,
    ) -> TrackingResult<()> {
        let mut metadata = HashMap::new();
        metadata.insert("reclaim_reason".to_string(), reclaim_reason);
        
        self.record_passport_event(
            allocation_ptr,
            PassportEventType::ReclaimedByRust,
            rust_context,
            metadata,
        )
    }

    /// Record general passport event
    pub fn record_passport_event(
        &self,
        allocation_ptr: usize,
        event_type: PassportEventType,
        context: String,
        metadata: HashMap<String, String>,
    ) -> TrackingResult<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event = PassportEvent {
            event_type: event_type.clone(),
            timestamp: current_time,
            context: context.clone(),
            call_stack: self.capture_call_stack()?,
            metadata,
            sequence_number: self.get_next_event_sequence(),
        };

        if let Ok(mut passports) = self.passports.lock() {
            if let Some(passport) = passports.get_mut(&allocation_ptr) {
                // Limit events per passport
                if passport.lifecycle_events.len() >= self.config.max_events_per_passport {
                    passport.lifecycle_events.remove(0); // Remove oldest event
                }
                
                passport.lifecycle_events.push(event);
                passport.updated_at = current_time;

                // Update statistics
                self.update_stats_event_recorded();

                if self.config.detailed_logging {
                    tracing::info!("📝 Recorded {:?} event for passport 0x{:x} in context: {}", 
                        event_type, allocation_ptr, context);
                }
            } else {
                return Err(TrackingError::InvalidPointer(
                    format!("No passport found for 0x{:x}", allocation_ptr)
                ));
            }
        } else {
            return Err(TrackingError::LockContention("Failed to lock passports".to_string()));
        }

        Ok(())
    }

    /// Detect InForeignCustody status and memory leaks at program shutdown
    pub fn detect_leaks_at_shutdown(&self) -> LeakDetectionResult {
        if !self.config.enable_leak_detection {
            return LeakDetectionResult {
                leaked_passports: Vec::new(),
                total_leaks: 0,
                leak_details: Vec::new(),
                detected_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
        }

        let mut leaked_passports = Vec::new();
        let mut leak_details = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Ok(mut passports) = self.passports.lock() {
            for (ptr, passport) in passports.iter_mut() {
                // Determine final status based on lifecycle events
                let final_status = self.determine_final_status(&passport.lifecycle_events);
                passport.status_at_shutdown = final_status.clone();

                // Check for leaks (InForeignCustody status)
                if final_status == PassportStatus::InForeignCustody {
                    leaked_passports.push(passport.passport_id.clone());

                    // Create leak detail
                    let last_event = passport.lifecycle_events.last();
                    let last_context = last_event
                        .map(|e| e.context.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let time_since_last = last_event
                        .map(|e| current_time.saturating_sub(e.timestamp))
                        .unwrap_or(0);

                    let lifecycle_summary = self.create_lifecycle_summary(&passport.lifecycle_events);

                    leak_details.push(LeakDetail {
                        passport_id: passport.passport_id.clone(),
                        memory_address: *ptr,
                        size_bytes: passport.size_bytes,
                        last_context,
                        time_since_last_event: time_since_last,
                        lifecycle_summary,
                    });

                    tracing::warn!("🚨 MEMORY LEAK DETECTED: Passport {} (0x{:x}, {} bytes) in foreign custody", 
                        passport.passport_id, ptr, passport.size_bytes);
                }
            }

            // Update statistics
            if let Ok(mut stats) = self.stats.lock() {
                stats.leaks_detected = leaked_passports.len();
                
                // Update status counts
                stats.passports_by_status.clear();
                for passport in passports.values() {
                    let status_key = format!("{:?}", passport.status_at_shutdown);
                    *stats.passports_by_status.entry(status_key).or_insert(0) += 1;
                }
            }
        }

        let total_leaks = leaked_passports.len();
        
        tracing::info!("🔍 Leak detection complete: {} leaks detected out of {} passports", 
            total_leaks, self.get_passport_count());

        LeakDetectionResult {
            leaked_passports,
            total_leaks,
            leak_details,
            detected_at: current_time,
        }
    }

    /// Get passport by allocation pointer
    pub fn get_passport(&self, allocation_ptr: usize) -> Option<MemoryPassport> {
        self.passports.lock().ok()?.get(&allocation_ptr).cloned()
    }

    /// Get all passports
    pub fn get_all_passports(&self) -> HashMap<usize, MemoryPassport> {
        self.passports.lock().unwrap_or_else(|_| {
            tracing::error!("Failed to lock passports for reading");
            std::process::exit(1);
        }).clone()
    }

    /// Get passports by status
    pub fn get_passports_by_status(&self, status: PassportStatus) -> Vec<MemoryPassport> {
        if let Ok(passports) = self.passports.lock() {
            passports.values()
                .filter(|p| p.status_at_shutdown == status)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get tracker statistics
    pub fn get_stats(&self) -> PassportTrackerStats {
        self.stats.lock().unwrap_or_else(|_| {
            tracing::error!("Failed to lock stats");
            std::process::exit(1);
        }).clone()
    }

    /// Validate passport integrity
    pub fn validate_passport(&self, allocation_ptr: usize) -> TrackingResult<bool> {
        if !self.config.enable_validation {
            return Ok(true);
        }

        if let Ok(passports) = self.passports.lock() {
            if let Some(passport) = passports.get(&allocation_ptr) {
                // Validate event sequence
                let mut last_sequence = 0;
                for event in &passport.lifecycle_events {
                    if event.sequence_number <= last_sequence {
                        self.update_stats_validation_failure();
                        return Ok(false);
                    }
                    last_sequence = event.sequence_number;
                }

                // Validate timestamps
                let mut last_timestamp = 0;
                for event in &passport.lifecycle_events {
                    if event.timestamp < last_timestamp {
                        self.update_stats_validation_failure();
                        return Ok(false);
                    }
                    last_timestamp = event.timestamp;
                }

                Ok(true)
            } else {
                Err(TrackingError::InvalidPointer(
                    format!("No passport found for 0x{:x}", allocation_ptr)
                ))
            }
        } else {
            Err(TrackingError::LockContention("Failed to lock passports".to_string()))
        }
    }

    /// Clear all passports (for testing)
    pub fn clear_all_passports(&self) {
        if let Ok(mut passports) = self.passports.lock() {
            passports.clear();
        }
        
        if let Ok(mut stats) = self.stats.lock() {
            *stats = PassportTrackerStats {
                tracker_start_time: stats.tracker_start_time,
                ..Default::default()
            };
        }

        tracing::info!("🧹 Cleared all passports");
    }

    // Private helper methods

    fn generate_passport_id(&self, allocation_ptr: usize) -> TrackingResult<String> {
        let sequence = if let Ok(mut counter) = self.sequence_counter.lock() {
            *counter += 1;
            *counter
        } else {
            return Err(TrackingError::LockContention("Failed to lock sequence counter".to_string()));
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        Ok(format!("passport_{:x}_{:08x}_{}", allocation_ptr, sequence, timestamp % 1000000))
    }

    fn get_next_event_sequence(&self) -> u32 {
        if let Ok(mut counter) = self.event_sequence.lock() {
            *counter += 1;
            *counter
        } else {
            tracing::error!("Failed to lock event sequence counter");
            0
        }
    }

    fn capture_call_stack(&self) -> TrackingResult<Vec<StackFrame>> {
        // Simplified call stack capture
        // In a real implementation, this would use backtrace or similar
        Ok(vec![
            StackFrame {
                function_name: "memory_passport_tracker".to_string(),
                file_name: Some("src/analysis/memory_passport_tracker.rs".to_string()),
                line_number: Some(1),
                is_unsafe: false,
            }
        ])
    }

    fn determine_final_status(&self, events: &[PassportEvent]) -> PassportStatus {
        let mut has_handover = false;
        let mut has_reclaim = false;
        let mut has_foreign_free = false;

        // Analyze events in chronological order
        for event in events {
            match event.event_type {
                PassportEventType::HandoverToFfi => has_handover = true,
                PassportEventType::ReclaimedByRust => {
                    has_reclaim = true;
                    has_handover = false; // Reset handover status
                }
                PassportEventType::FreedByForeign => {
                    has_foreign_free = true;
                    has_handover = false; // Reset handover status
                }
                _ => {}
            }
        }

        // Determine final status based on event history
        if has_handover && !has_reclaim && !has_foreign_free {
            PassportStatus::InForeignCustody // This is a confirmed leak
        } else if has_foreign_free {
            PassportStatus::FreedByForeign
        } else if has_reclaim {
            PassportStatus::ReclaimedByRust
        } else if has_handover {
            PassportStatus::HandoverToFfi
        } else {
            PassportStatus::FreedByRust
        }
    }

    fn create_lifecycle_summary(&self, events: &[PassportEvent]) -> String {
        let event_types: Vec<String> = events.iter()
            .map(|e| format!("{:?}", e.event_type))
            .collect();
        
        if event_types.is_empty() {
            "No events recorded".to_string()
        } else {
            format!("Events: {}", event_types.join(" -> "))
        }
    }

    fn get_passport_count(&self) -> usize {
        self.passports.lock().map(|p| p.len()).unwrap_or(0)
    }

    fn update_stats_passport_created(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_passports_created += 1;
            stats.active_passports = self.get_passport_count();
        }
    }

    fn update_stats_event_recorded(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_events_recorded += 1;
        }
    }

    fn update_stats_validation_failure(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.validation_failures += 1;
        }
    }
}

impl Default for MemoryPassportTracker {
    fn default() -> Self {
        Self::new(PassportTrackerConfig::default())
    }
}

/// Global memory passport tracker instance
static GLOBAL_PASSPORT_TRACKER: std::sync::OnceLock<Arc<MemoryPassportTracker>> = std::sync::OnceLock::new();

/// Get global memory passport tracker instance
pub fn get_global_passport_tracker() -> Arc<MemoryPassportTracker> {
    GLOBAL_PASSPORT_TRACKER.get_or_init(|| {
        Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default()))
    }).clone()
}

/// Initialize global passport tracker with custom config
pub fn initialize_global_passport_tracker(config: PassportTrackerConfig) -> Arc<MemoryPassportTracker> {
    let tracker = Arc::new(MemoryPassportTracker::new(config));
    if GLOBAL_PASSPORT_TRACKER.set(tracker.clone()).is_err() {
        tracing::warn!("Global passport tracker already initialized");
    }
    tracker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passport_creation() {
        let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
        let ptr = 0x1000;
        let size = 1024;
        
        let passport_id = tracker.create_passport(ptr, size, "test_context".to_string()).expect("Failed to create passport");
        assert!(!passport_id.is_empty());
        
        let passport = tracker.get_passport(ptr).expect("Failed to get passport");
        assert_eq!(passport.allocation_ptr, ptr);
        assert_eq!(passport.size_bytes, size);
        assert_eq!(passport.lifecycle_events.len(), 1);
        assert_eq!(passport.lifecycle_events[0].event_type, PassportEventType::AllocatedInRust);
    }

    #[test]
    fn test_handover_to_ffi() {
        let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
        let ptr = 0x2000;
        
        tracker.create_passport(ptr, 512, "rust_context".to_string()).expect("Failed to create passport");
        tracker.record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string()).expect("Failed to record handover");
        
        let passport = tracker.get_passport(ptr).expect("Failed to get passport");
        assert_eq!(passport.lifecycle_events.len(), 2);
        assert_eq!(passport.lifecycle_events[1].event_type, PassportEventType::HandoverToFfi);
    }

    #[test]
    fn test_leak_detection() {
        let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
        let ptr = 0x3000;
        
        // Create passport and hand over to FFI without reclaim
        tracker.create_passport(ptr, 256, "rust_context".to_string()).expect("Failed to create passport");
        tracker.record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string()).expect("Failed to record handover");
        
        // Detect leaks at shutdown
        let leak_result = tracker.detect_leaks_at_shutdown();
        
        assert_eq!(leak_result.total_leaks, 1);
        assert_eq!(leak_result.leaked_passports.len(), 1);
        assert_eq!(leak_result.leak_details.len(), 1);
        assert_eq!(leak_result.leak_details[0].memory_address, ptr);
        
        // Verify passport status
        let passport = tracker.get_passport(ptr).unwrap();
        assert_eq!(passport.status_at_shutdown, PassportStatus::InForeignCustody);
    }

    #[test]
    fn test_reclaim_prevents_leak() {
        let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
        let ptr = 0x4000;
        
        // Create passport, hand over to FFI, then reclaim
        tracker.create_passport(ptr, 128, "rust_context".to_string()).expect("Failed to create passport");
        tracker.record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string()).expect("Failed to record handover");
        tracker.record_reclaimed_by_rust(ptr, "rust_context".to_string(), "cleanup".to_string()).expect("Failed to record reclaim");
        
        // Detect leaks at shutdown
        let leak_result = tracker.detect_leaks_at_shutdown();
        
        assert_eq!(leak_result.total_leaks, 0);
        
        // Verify passport status
        let passport = tracker.get_passport(ptr).unwrap();
        assert_eq!(passport.status_at_shutdown, PassportStatus::ReclaimedByRust);
    }

    #[test]
    fn test_passport_validation() {
        let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
        let ptr = 0x5000;
        
        tracker.create_passport(ptr, 64, "test_context".to_string()).expect("Failed to create passport");
        tracker.record_handover_to_ffi(ptr, "ffi_context".to_string(), "test_func".to_string()).expect("Failed to record handover");
        
        let is_valid = tracker.validate_passport(ptr).expect("Failed to validate passport");
        assert!(is_valid);
    }
}