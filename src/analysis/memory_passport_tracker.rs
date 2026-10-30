//! Memory Passport Tracking System for FFI Boundary Memory Management
//!
//! This module implements comprehensive memory passport tracking for memory that crosses
//! FFI boundaries, including lifecycle event recording and leak detection at shutdown.

use crate::analysis::unsafe_ffi_tracker::StackFrame;
use crate::capture::types::{TrackingError, TrackingResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Memory passport for tracking cross-FFI boundary memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryPassport {
    /// Unique passport identifier
    pub passport_id: String,
    /// Memory allocation pointer
    pub allocation_ptr: usize,
    /// Size in bytes
    pub size_bytes: usize,
    /// Type name of the allocated memory
    pub type_name: String,
    /// Variable name if available
    pub var_name: String,
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
    /// 1-based binding order at `allocation_ptr` (1 = first use of the address).
    ///
    /// Two passports for one address are two different blocks; the generation is what
    /// keeps their evidence apart (plan A15 / §7.3).
    #[serde(default = "default_generation")]
    pub generation: u64,
    /// Set when a newer passport took over the same address while this one was still
    /// live: the record is archived instead of being silently dropped.
    #[serde(default)]
    pub superseded: bool,
    /// Whether the lifecycle event list was truncated to respect the per-passport cap.
    #[serde(default)]
    pub history_truncated: bool,
}

/// Serde default for [`MemoryPassport::generation`].
fn default_generation() -> u64 {
    1
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
    /// Memory allocated in Rust and not released inside the capture window.
    ///
    /// This is the honest resting state for a block that never crossed the FFI boundary
    /// and was never observed released: it is *not* "freed by Rust" (plan A15).
    StillLive,
    /// Status unknown or corrupted
    Unknown,
}

/// Lifecycle event in memory passport
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Active memory passports (the newest one per address).
    passports: Arc<Mutex<HashMap<usize, MemoryPassport>>>,
    /// Passports that were replaced by a newer block at the same address. Kept so their
    /// evidence ("handed to foreign code and never returned") is not lost.
    superseded: Arc<Mutex<Vec<MemoryPassport>>>,
    /// Next passport generation per address.
    generations: Arc<Mutex<HashMap<usize, u64>>>,
    /// Passport creation sequence
    sequence_counter: Arc<Mutex<u32>>,
    /// Event sequence counter (use AtomicU32 for lock-free operation)
    event_sequence: Arc<AtomicU32>,
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
    /// Track Rust internal call stacks (e.g., std, core, alloc)
    /// When false, only user code call stacks are tracked
    pub track_rust_internal_stack: bool,
    /// User code path prefixes for identifying user code
    /// Call stacks from paths NOT starting with these prefixes are considered Rust internal
    /// If empty, all paths are considered user code
    pub user_code_prefixes: Vec<String>,
}

impl Default for PassportTrackerConfig {
    fn default() -> Self {
        Self {
            detailed_logging: true,
            max_events_per_passport: 100,
            enable_leak_detection: true,
            enable_validation: true,
            max_passports: 10000,
            track_rust_internal_stack: false,
            user_code_prefixes: vec![
                "src/".to_string(),
                "examples/".to_string(),
                "tests/".to_string(),
                "benches/".to_string(),
            ],
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
        tracing::debug!("Initializing Memory Passport Tracker");
        tracing::debug!("   Detailed logging: {}", config.detailed_logging);
        tracing::debug!(
            "   Max events per passport: {}",
            config.max_events_per_passport
        );
        tracing::debug!("   Leak detection: {}", config.enable_leak_detection);
        tracing::debug!("   Validation: {}", config.enable_validation);

        Self {
            passports: Arc::new(Mutex::new(HashMap::new())),
            superseded: Arc::new(Mutex::new(Vec::new())),
            generations: Arc::new(Mutex::new(HashMap::new())),
            sequence_counter: Arc::new(Mutex::new(0)),
            event_sequence: Arc::new(AtomicU32::new(0)),
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

    /// Create memory passport for FFI boundary tracking (backward compatible).
    ///
    /// This is a convenience wrapper that calls `create_passport` with
    /// `type_name` and `var_name` set to `None`.
    pub fn create_passport_simple(
        &self,
        allocation_ptr: usize,
        size_bytes: usize,
        initial_context: String,
    ) -> TrackingResult<String> {
        self.create_passport(allocation_ptr, size_bytes, initial_context, None, None)
    }

    /// Create memory passport for FFI boundary tracking
    pub fn create_passport(
        &self,
        allocation_ptr: usize,
        size_bytes: usize,
        initial_context: String,
        type_name: Option<String>,
        var_name: Option<String>,
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
            type_name: type_name.unwrap_or_else(|| "-".to_string()),
            var_name: var_name.unwrap_or_else(|| "-".to_string()),
            status_at_shutdown: PassportStatus::Unknown,
            generation: 1,
            superseded: false,
            history_truncated: false,
            lifecycle_events: vec![initial_event],
            created_at: current_time,
            updated_at: current_time,
            metadata: HashMap::new(),
        };

        // Store passport. A block that reused the address of a still-live passport must
        // not silently erase it: the old record is archived (and counted) so its evidence
        // survives, and the new record carries the next generation for that address.
        let mut passport = passport;
        if let Ok(mut generations) = self.generations.lock() {
            let counter = generations.entry(allocation_ptr).or_insert(0);
            *counter = counter.saturating_add(1);
            passport.generation = *counter;
        }

        if let Ok(mut passports) = self.passports.lock() {
            // Check passport limit
            if passports.len() >= self.config.max_passports {
                return Err(TrackingError::ResourceExhausted(
                    "Maximum passport limit reached".to_string(),
                ));
            }
            if let Some(previous) = passports.insert(allocation_ptr, passport) {
                let mut archived = previous;
                archived.superseded = true;
                tracing::debug!(
                    "passport address reuse: 0x{:x} generation {} replaced while live \
                     (passport {})",
                    allocation_ptr,
                    archived.generation,
                    archived.passport_id
                );
                if let Ok(mut superseded) = self.superseded.lock() {
                    superseded.push(archived);
                }
            }
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock passports".to_string(),
            ));
        }

        // Update statistics
        self.update_stats_passport_created();

        if self.config.detailed_logging {
            tracing::debug!(
                "Created passport: {} for 0x{:x} ({} bytes)",
                passport_id,
                allocation_ptr,
                size_bytes
            );
        }

        Ok(passport_id)
    }

    /// Create memory passport with automatic type inference
    pub fn create_passport_with_inference(
        &self,
        allocation_ptr: usize,
        size_bytes: usize,
        memory: Option<&[u8]>,
        initial_context: String,
        var_name: Option<String>,
    ) -> TrackingResult<String> {
        let type_name = memory
            .map(|m| {
                let guess =
                    crate::analysis::unsafe_inference::UnsafeInferenceEngine::infer_from_bytes(
                        m, size_bytes,
                    );
                guess.display_with_confidence()
            })
            .unwrap_or_else(|| "-".to_string());

        self.create_passport(
            allocation_ptr,
            size_bytes,
            initial_context,
            Some(type_name),
            var_name,
        )
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
                    // Drop the oldest event, then flag the chain as incomplete so reports
                    // never present a truncated history as a complete one (plan §7.3 6).
                    passport.lifecycle_events.drain(0..1);
                    passport.history_truncated = true;
                    passport.metadata.insert(
                        "history_truncated".to_string(),
                        "oldest events dropped to respect max_events_per_passport".to_string(),
                    );
                }

                passport.lifecycle_events.push(event);
                passport.updated_at = current_time;

                // Update statistics
                self.update_stats_event_recorded();

                if self.config.detailed_logging {
                    tracing::debug!(
                        "Recorded {:?} event for passport 0x{:x} in context: {}",
                        event_type,
                        allocation_ptr,
                        context
                    );
                }
            } else {
                return Err(TrackingError::InvalidPointer(format!(
                    "No passport found for 0x{allocation_ptr:x}"
                )));
            }
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock passports".to_string(),
            ));
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

        let passports = match self.passports.lock() {
            Ok(passports) => passports,
            Err(_) => {
                return LeakDetectionResult {
                    leaked_passports: Vec::new(),
                    total_leaks: 0,
                    leak_details: Vec::new(),
                    detected_at: current_time,
                }
            }
        };

        for (ptr, passport) in passports.iter() {
            // Read-only projection: detection must not rewrite the stored passports or the
            // counters, otherwise exporting twice produces different numbers and a live
            // object acquires a "shutdown" verdict mid-run (plan §7.3 item 4).
            let status = self.determine_final_status(&passport.lifecycle_events);

            if status == PassportStatus::InForeignCustody {
                leaked_passports.push(passport.passport_id.clone());

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

                tracing::warn!(
                    "MEMORY LEAK DETECTED: Passport {} (0x{:x}, {} bytes) in foreign custody",
                    passport.passport_id,
                    ptr,
                    passport.size_bytes
                );
            }
        }
        drop(passports);

        let total_leaks = leaked_passports.len();

        tracing::debug!(
            "Leak detection complete: {} leaks detected out of {} passports",
            total_leaks,
            self.get_passport_count()
        );

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

    /// Get all current passports.
    pub fn get_all_passports(&self) -> HashMap<usize, MemoryPassport> {
        self.passports
            .lock()
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Mutex poisoned in get_all_passports, recovering data: {}",
                    e
                );
                e.into_inner()
            })
            .clone()
    }

    /// Passports archived because a newer block reused their address while they were live.
    ///
    /// Keeping them preserves the evidence of the older block (for example "handed to
    /// foreign code and never returned"), which the previous `insert` overwrite lost.
    #[must_use]
    pub fn get_archived_passports(&self) -> Vec<MemoryPassport> {
        self.superseded
            .lock()
            .map(|passports| passports.clone())
            .unwrap_or_default()
    }

    /// Get passports whose *current* projection matches `status`.
    pub fn get_passports_by_status(&self, status: PassportStatus) -> Vec<MemoryPassport> {
        if let Ok(passports) = self.passports.lock() {
            passports
                .values()
                .filter(|passport| self.assess_status(passport) == status)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get tracker statistics
    pub fn get_stats(&self) -> PassportTrackerStats {
        self.stats
            .lock()
            .unwrap_or_else(|e| {
                tracing::warn!("Mutex poisoned in get_stats, recovering data: {}", e);
                e.into_inner()
            })
            .clone()
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
                    if event.sequence_number < last_sequence {
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
                Err(TrackingError::InvalidPointer(format!(
                    "No passport found for 0x{allocation_ptr:x}",
                )))
            }
        } else {
            Err(TrackingError::LockContention(
                "Failed to lock passports".to_string(),
            ))
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

        tracing::debug!("Cleared all passports");
    }

    // Private helper methods

    fn generate_passport_id(&self, allocation_ptr: usize) -> TrackingResult<String> {
        let sequence = if let Ok(mut counter) = self.sequence_counter.lock() {
            *counter += 1;
            *counter
        } else {
            return Err(TrackingError::LockContention(
                "Failed to lock sequence counter".to_string(),
            ));
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        Ok(format!(
            "passport_{:x}_{:08x}_{}",
            allocation_ptr,
            sequence,
            timestamp % 1000000
        ))
    }

    fn get_next_event_sequence(&self) -> u32 {
        // Use atomic fetch_add for lock-free sequence number generation
        // This eliminates lock contention and prevents ID conflicts
        self.event_sequence.fetch_add(1, Ordering::SeqCst)
    }

    fn capture_call_stack(&self) -> TrackingResult<Vec<StackFrame>> {
        // Use backtrace crate to capture real call stack
        #[cfg(feature = "backtrace")]
        {
            let backtrace = backtrace::Backtrace::new();
            let mut frames = Vec::new();

            // Skip first 2 frames (capture_call_stack and caller)
            let skip_frames = 2;
            let max_depth = 32; // Maximum stack depth to capture

            for (i, frame) in backtrace.frames().iter().enumerate() {
                if i < skip_frames {
                    continue;
                }

                if let Some(symbol) = frame.symbols().first() {
                    frames.push(StackFrame {
                        function_name: symbol
                            .name()
                            .map(|n| format!("{:?}", n))
                            .unwrap_or_else(|| "unknown".to_string()),
                        file_name: symbol.filename().map(|p| p.display().to_string()),
                        line_number: symbol.lineno(),
                        is_unsafe: false,
                    });
                }

                if frames.len() >= max_depth {
                    break;
                }
            }

            Ok(frames)
        }

        #[cfg(not(feature = "backtrace"))]
        {
            // Return empty stack when backtrace feature is not enabled
            Ok(Vec::new())
        }
    }

    fn determine_final_status(&self, events: &[PassportEvent]) -> PassportStatus {
        let mut has_handover = false;
        let mut has_reclaim = false;
        let mut has_foreign_free = false;
        let mut has_corruption = false;

        for event in events {
            match event.event_type {
                PassportEventType::HandoverToFfi => has_handover = true,
                PassportEventType::ReclaimedByRust => {
                    has_reclaim = true;
                    has_handover = false;
                }
                PassportEventType::FreedByForeign => {
                    has_foreign_free = true;
                    has_handover = false;
                }
                PassportEventType::CorruptionDetected => has_corruption = true,
                _ => {}
            }
        }

        if has_corruption {
            PassportStatus::Unknown
        } else if has_handover && !has_reclaim && !has_foreign_free {
            PassportStatus::InForeignCustody
        } else if has_foreign_free {
            PassportStatus::FreedByForeign
        } else if has_reclaim {
            PassportStatus::ReclaimedByRust
        } else if has_handover {
            PassportStatus::HandoverToFfi
        } else {
            // Allocated in Rust, no handover and no release observation: the block is
            // still live as far as the passport can tell. Reporting `FreedByRust` here
            // turned every untouched allocation into a released one and hid live objects
            // from the leak/active checks.
            PassportStatus::StillLive
        }
    }

    /// Status of a passport at the current cut, derived from its lifecycle events.
    ///
    /// Read-only: callers get a projection and the stored passport is left untouched, so
    /// two exports of one batch observe identical state (plan §7.3 item 4).
    #[must_use]
    pub fn assess_status(&self, passport: &MemoryPassport) -> PassportStatus {
        self.determine_final_status(&passport.lifecycle_events)
    }

    fn create_lifecycle_summary(&self, events: &[PassportEvent]) -> String {
        let event_types: Vec<String> = events
            .iter()
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

/// Global memory passport tracker instance (used as fallback for direct access)
static GLOBAL_MEMORY_PASSPORT_TRACKER: std::sync::OnceLock<Arc<MemoryPassportTracker>> =
    std::sync::OnceLock::new();

/// Get global memory passport tracker instance
///
/// # Note
/// This function is deprecated. Use `global_passport_tracker()` from
/// `memscope_rs::capture::backends::global_tracking` instead.
pub fn get_global_passport_tracker() -> Arc<MemoryPassportTracker> {
    GLOBAL_MEMORY_PASSPORT_TRACKER
        .get_or_init(|| Arc::new(MemoryPassportTracker::new(PassportTrackerConfig::default())))
        .clone()
}

/// Initialize global passport tracker with custom config
///
/// # Note  
/// This function is deprecated. Use `init_global_tracking_with_config()` from
/// `memscope_rs::capture::backends::global_tracking` instead.
pub fn initialize_global_passport_tracker(
    config: PassportTrackerConfig,
) -> Arc<MemoryPassportTracker> {
    GLOBAL_MEMORY_PASSPORT_TRACKER
        .get_or_init(|| Arc::new(MemoryPassportTracker::new(config)))
        .clone()
}

#[cfg(test)]
mod tests;
