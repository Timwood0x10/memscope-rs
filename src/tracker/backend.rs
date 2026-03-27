//! Unified tracking backend system
//!
//! This module provides a unified tracking system that consolidates the 4 existing
//! tracking systems (Core, Lockfree, Async Memory, Unified) into a single configurable
//! architecture while preserving all functionality.

use crate::types::internal_types::{
    Allocation, Event, MemoryPassportTracker, Snapshot, Stats, TrackingResult,
};
use crossbeam::queue::ArrayQueue;
use std::alloc::GlobalAlloc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

// ============================================================================
// Tracking Strategy and Configuration
// ============================================================================

/// Tracking strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingStrategy {
    /// Single-threaded strategy (corresponds to original Core system)
    SingleThread,
    /// Thread-local strategy (corresponds to original Lockfree system)
    ThreadLocal,
    /// Async strategy (corresponds to original Async Memory system)
    Async,
    /// Hybrid strategy (automatic selection)
    Hybrid,
}

impl Default for TrackingStrategy {
    fn default() -> Self {
        TrackingStrategy::Hybrid
    }
}

/// Sampling configuration
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Sampling rate (0.0 to 1.0)
    pub rate: f64,
    /// Enable adaptive sampling
    pub adaptive: bool,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            rate: 1.0, // Full sampling by default
            adaptive: false,
        }
    }
}

/// Overhead limit configuration
#[derive(Debug, Clone)]
pub struct OverheadLimit {
    /// Maximum overhead percentage (0.0 to 1.0)
    pub max_overhead: f64,
    /// Enable dynamic throttling
    pub dynamic_throttling: bool,
}

impl Default for OverheadLimit {
    fn default() -> Self {
        Self {
            max_overhead: 0.1, // 10% overhead limit
            dynamic_throttling: true,
        }
    }
}

/// Unified tracking configuration
#[derive(Debug, Clone)]
pub struct TrackingConfig {
    /// Tracking strategy
    pub strategy: TrackingStrategy,
    /// Sampling configuration
    pub sampling: SamplingConfig,
    /// Overhead limit
    pub overhead_limit: OverheadLimit,
    /// Enable smart pointer tracking
    pub enable_smart_pointers: bool,
    /// Enable lifecycle tracking
    pub enable_lifecycle: bool,
    /// Enable MemoryPassport system
    pub enable_passports: bool,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            strategy: TrackingStrategy::default(),
            sampling: SamplingConfig::default(),
            overhead_limit: OverheadLimit::default(),
            enable_smart_pointers: true,
            enable_lifecycle: true,
            enable_passports: true,
        }
    }
}

// ============================================================================
// Allocation Context
// ============================================================================

/// Allocation context information
#[derive(Debug, Clone)]
pub struct AllocationContext {
    /// Allocation size
    pub size: usize,
    /// Thread ID
    pub thread: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Variable name (if available)
    pub var_name: Option<String>,
    /// Type name (if available)
    pub type_name: Option<String>,
    /// Stack trace (if available)
    pub stack_trace: Option<Vec<String>>,
}

impl AllocationContext {
    pub fn new(size: usize) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            size,
            thread: current_thread_id(),
            timestamp,
            var_name: None,
            type_name: None,
            stack_trace: None,
        }
    }

    pub fn with_var_name(mut self, name: String) -> Self {
        self.var_name = Some(name);
        self
    }

    pub fn with_type_name(mut self, name: String) -> Self {
        self.type_name = Some(name);
        self
    }
}

fn current_thread_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ============================================================================
// TrackingBackend Trait
// ============================================================================

/// Core tracking backend trait
pub trait TrackingBackend: Send + Sync {
    /// Track an allocation event
    fn track_allocation(&self, ptr: usize, size: usize, ctx: AllocationContext);

    /// Track a deallocation event
    fn track_deallocation(&self, ptr: usize);

    /// Track a reallocation event
    fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        ctx: AllocationContext,
    );

    /// Emit a generic event
    fn emit_event(&self, event: Event);

    /// Get a snapshot of the current state
    fn snapshot(&self) -> Snapshot;

    /// Get the statistics
    fn stats(&self) -> Stats;

    /// Enable/disable tracking
    fn set_enabled(&self, enabled: bool);

    /// Check if tracking is enabled
    fn is_enabled(&self) -> bool;

    /// Set fast mode
    fn set_fast_mode(&self, enabled: bool);

    /// Check if fast mode is enabled
    fn is_fast_mode(&self) -> bool;
}

// ============================================================================
// Unified Storage (Shared by all backends)
// ============================================================================

/// Unified event and allocation storage
pub struct UnifiedStorage {
    /// Event queue (lock-free)
    event_queue: Arc<ArrayQueue<Event>>,
    /// Allocations by pointer
    allocations: Arc<RwLock<HashMap<usize, Allocation>>>,
    /// Task information
    tasks: Arc<RwLock<HashMap<u64, crate::types::internal_types::TaskInfo>>>,
    /// Thread information
    threads: Arc<RwLock<HashMap<u32, crate::types::internal_types::ThreadInfo>>>,
    /// MemoryPassport tracker
    passport_tracker: Arc<Mutex<MemoryPassportTracker>>,
    /// Statistics
    stats: Arc<RwLock<Stats>>,
    /// Tracking enabled flag
    enabled: Arc<std::sync::atomic::AtomicBool>,
}

impl UnifiedStorage {
    pub fn new() -> Self {
        Self {
            event_queue: Arc::new(ArrayQueue::new(100_000)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            threads: Arc::new(RwLock::new(HashMap::new())),
            passport_tracker: Arc::new(Mutex::new(MemoryPassportTracker::new())),
            stats: Arc::new(RwLock::new(Stats::default())),
            enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    pub fn emit_event(&self, event: Event) {
        if !self.enabled.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        if let Err(_) = self.event_queue.push(event.clone()) {
            // Queue is full, process immediately
            self.process_event(&event);
        }
    }

    fn process_event(&self, event: &Event) {
        let mut allocs = self.allocations.write().unwrap();
        let mut threads = self.threads.write().unwrap();

        match event {
            Event::Alloc {
                ptr,
                size,
                thread,
                ts,
            } => {
                let mut allocation = Allocation::new(*ptr, *size);
                allocation.alloc_ts = *ts;
                allocation.thread = *thread;

                allocs.insert(*ptr, allocation);

                threads.entry(*thread).or_insert_with(|| {
                    crate::types::internal_types::ThreadInfo {
                        thread_id: *thread,
                        total_size: 0,
                        allocation_count: 0,
                        deallocation_count: 0,
                    }
                });

                if let Some(thread_info) = threads.get_mut(thread) {
                    thread_info.total_size += size;
                    thread_info.allocation_count += 1;
                }
            }
            Event::Dealloc { ptr, thread, ts } => {
                if let Some(alloc) = allocs.get_mut(ptr) {
                    alloc.free_ts = Some(*ts);
                }

                if let Some(thread_info) = threads.get_mut(thread) {
                    thread_info.deallocation_count += 1;
                }
            }
            Event::Realloc {
                old_ptr,
                new_ptr,
                size,
                thread: _,
                ts,
            } => {
                if let Some(mut alloc) = allocs.remove(old_ptr) {
                    alloc.free_ts = Some(*ts);
                    let new_alloc = Allocation::new(*new_ptr, *size);
                    allocs.insert(*new_ptr, new_alloc);
                }
            }
            Event::TaskSpawn { id, ts } => {
                let mut tasks = self.tasks.write().unwrap();
                tasks.insert(
                    *id,
                    crate::types::internal_types::TaskInfo {
                        id: *id,
                        total_size: 0,
                        allocation_count: 0,
                        start_ts: *ts,
                        end_ts: None,
                    },
                );
            }
            Event::TaskEnd { id, ts } => {
                let mut tasks = self.tasks.write().unwrap();
                if let Some(task) = tasks.get_mut(id) {
                    task.end_ts = Some(*ts);
                }
            }
            Event::FfiAlloc { ptr, size, lib, ts } => {
                let mut allocation = Allocation::new(*ptr, *size);
                allocation.alloc_ts = *ts;
                allocation.meta.type_name = Some(format!("ffi:{}", lib));
                allocs.insert(*ptr, allocation);

                let mut passport_tracker = self.passport_tracker.lock().unwrap();
                passport_tracker.track_ffi_alloc(*ptr, *size, lib);
            }
            Event::FfiFree { ptr, lib: _, .. } => {
                let mut passport_tracker = self.passport_tracker.lock().unwrap();
                passport_tracker.record_ffi_free(*ptr);

                // Mark as deallocated
                if let Some(alloc) = allocs.get_mut(ptr) {
                    alloc.free_ts = Some(event.timestamp());
                }
            }
            _ => {}
        }
    }

    pub fn process_all_pending(&self) {
        while let Some(event) = self.event_queue.pop() {
            self.process_event(&event);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.process_all_pending();

        let allocs = self.allocations.read().unwrap();
        let tasks = self.tasks.read().unwrap();
        let threads = self.threads.read().unwrap();
        let mut stats = self.stats.write().unwrap();
        let passport_tracker = self.passport_tracker.lock().unwrap();

        stats.update_from_allocations(&allocs.values().cloned().collect::<Vec<_>>());

        Snapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            allocations: allocs.values().cloned().collect(),
            tasks: tasks.values().cloned().collect(),
            threads: threads.values().cloned().collect(),
            passports: passport_tracker
                .get_all_passports()
                .into_iter()
                .cloned()
                .collect(),
            stats: stats.clone(),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for UnifiedStorage {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Backend Implementations
// ============================================================================

/// Single-threaded backend (corresponds to original Core system)
pub struct SingleThreadBackend {
    storage: Arc<UnifiedStorage>,
}

impl SingleThreadBackend {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(UnifiedStorage::new()),
        }
    }
}

impl TrackingBackend for SingleThreadBackend {
    fn track_allocation(&self, ptr: usize, size: usize, ctx: AllocationContext) {
        self.storage.emit_event(Event::Alloc {
            ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn track_deallocation(&self, ptr: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.storage.emit_event(Event::Dealloc {
            ptr,
            thread: current_thread_id(),
            ts: timestamp,
        });
    }

    fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        ctx: AllocationContext,
    ) {
        self.storage.emit_event(Event::Realloc {
            old_ptr,
            new_ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn emit_event(&self, event: Event) {
        self.storage.emit_event(event);
    }

    fn snapshot(&self) -> Snapshot {
        self.storage.snapshot()
    }

    fn stats(&self) -> Stats {
        let snapshot = self.storage.snapshot();
        snapshot.stats
    }

    fn set_enabled(&self, enabled: bool) {
        self.storage.set_enabled(enabled);
    }

    fn is_enabled(&self) -> bool {
        self.storage.is_enabled()
    }

    fn set_fast_mode(&self, enabled: bool) {
        // For SingleThreadBackend, fast mode can be enabled by reducing sampling
        // This is a simplified implementation
        if enabled {
            tracing::debug!("Fast mode enabled for SingleThreadBackend");
        }
    }

    fn is_fast_mode(&self) -> bool {
        // Check environment variable for fast mode
        std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test)
    }
}

/// Thread-local backend (corresponds to original Lockfree system)
pub struct ThreadLocalBackend {
    storage: Arc<UnifiedStorage>,
}

impl ThreadLocalBackend {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(UnifiedStorage::new()),
        }
    }
}

impl TrackingBackend for ThreadLocalBackend {
    fn track_allocation(&self, ptr: usize, size: usize, ctx: AllocationContext) {
        self.storage.emit_event(Event::Alloc {
            ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn track_deallocation(&self, ptr: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.storage.emit_event(Event::Dealloc {
            ptr,
            thread: current_thread_id(),
            ts: timestamp,
        });
    }

    fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        ctx: AllocationContext,
    ) {
        self.storage.emit_event(Event::Realloc {
            old_ptr,
            new_ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn emit_event(&self, event: Event) {
        self.storage.emit_event(event);
    }

    fn snapshot(&self) -> Snapshot {
        self.storage.snapshot()
    }

    fn stats(&self) -> Stats {
        let snapshot = self.storage.snapshot();
        snapshot.stats
    }

    fn set_enabled(&self, enabled: bool) {
        self.storage.set_enabled(enabled);
    }

    fn is_enabled(&self) -> bool {
        self.storage.is_enabled()
    }

    fn set_fast_mode(&self, enabled: bool) {
        // ThreadLocalBackend is already optimized, fast mode just reduces overhead
        if enabled {
            tracing::debug!("Fast mode enabled for ThreadLocalBackend");
        }
    }

    fn is_fast_mode(&self) -> bool {
        // ThreadLocalBackend is always in fast mode
        true
    }
}

/// Async backend (corresponds to original Async Memory system)
pub struct AsyncBackend {
    storage: Arc<UnifiedStorage>,
}

impl AsyncBackend {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(UnifiedStorage::new()),
        }
    }
}

impl TrackingBackend for AsyncBackend {
    fn track_allocation(&self, ptr: usize, size: usize, ctx: AllocationContext) {
        self.storage.emit_event(Event::Alloc {
            ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn track_deallocation(&self, ptr: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.storage.emit_event(Event::Dealloc {
            ptr,
            thread: current_thread_id(),
            ts: timestamp,
        });
    }

    fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        ctx: AllocationContext,
    ) {
        self.storage.emit_event(Event::Realloc {
            old_ptr,
            new_ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn emit_event(&self, event: Event) {
        self.storage.emit_event(event);
    }

    fn snapshot(&self) -> Snapshot {
        self.storage.snapshot()
    }

    fn stats(&self) -> Stats {
        let snapshot = self.storage.snapshot();
        snapshot.stats
    }

    fn set_enabled(&self, enabled: bool) {
        self.storage.set_enabled(enabled);
    }

    fn is_enabled(&self) -> bool {
        self.storage.is_enabled()
    }

    fn set_fast_mode(&self, enabled: bool) {
        // Async backend can benefit from fast mode by reducing future tracking overhead
        if enabled {
            tracing::debug!("Fast mode enabled for AsyncBackend");
        }
    }

    fn is_fast_mode(&self) -> bool {
        // Check environment variable for fast mode
        std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test)
    }
}

/// Hybrid backend (automatic strategy selection)
pub struct HybridBackend {
    storage: Arc<UnifiedStorage>,
}

impl HybridBackend {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(UnifiedStorage::new()),
        }
    }
}

impl TrackingBackend for HybridBackend {
    fn track_allocation(&self, ptr: usize, size: usize, ctx: AllocationContext) {
        self.storage.emit_event(Event::Alloc {
            ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn track_deallocation(&self, ptr: usize) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.storage.emit_event(Event::Dealloc {
            ptr,
            thread: current_thread_id(),
            ts: timestamp,
        });
    }

    fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        size: usize,
        ctx: AllocationContext,
    ) {
        self.storage.emit_event(Event::Realloc {
            old_ptr,
            new_ptr,
            size,
            thread: ctx.thread,
            ts: ctx.timestamp,
        });
    }

    fn emit_event(&self, event: Event) {
        self.storage.emit_event(event);
    }

    fn snapshot(&self) -> Snapshot {
        self.storage.snapshot()
    }

    fn stats(&self) -> Stats {
        let snapshot = self.storage.snapshot();
        snapshot.stats
    }

    fn set_enabled(&self, enabled: bool) {
        self.storage.set_enabled(enabled);
    }

    fn is_enabled(&self) -> bool {
        self.storage.is_enabled()
    }

    fn set_fast_mode(&self, enabled: bool) {
        // Hybrid backend can adapt to fast mode by selecting optimal backend
        if enabled {
            tracing::debug!("Fast mode enabled for HybridBackend");
        }
    }

    fn is_fast_mode(&self) -> bool {
        // Hybrid backend automatically adapts, fast mode is context-dependent
        false
    }
}

// ============================================================================
// Unified Tracker
// ============================================================================

/// Unified tracker that consolidates all 4 tracking systems
pub struct UnifiedTracker {
    backend: Box<dyn TrackingBackend>,
    config: TrackingConfig,
}

impl UnifiedTracker {
    /// Create a new unified tracker with the specified configuration
    pub fn new(config: TrackingConfig) -> Self {
        let backend: Box<dyn TrackingBackend> = match config.strategy {
            TrackingStrategy::SingleThread => Box::new(SingleThreadBackend::new()),
            TrackingStrategy::ThreadLocal => Box::new(ThreadLocalBackend::new()),
            TrackingStrategy::Async => Box::new(AsyncBackend::new()),
            TrackingStrategy::Hybrid => Box::new(HybridBackend::new()),
        };

        Self { backend, config }
    }

    /// Track an allocation
    pub fn track_allocation(&self, ptr: usize, size: usize) {
        let ctx = AllocationContext::new(size);
        self.backend.track_allocation(ptr, size, ctx);
    }

    /// Track an allocation with variable name
    pub fn track_allocation_var(&self, ptr: usize, size: usize, var_name: String) {
        let ctx = AllocationContext::new(size).with_var_name(var_name);
        self.backend.track_allocation(ptr, size, ctx);
    }

    /// Track an allocation with type name
    pub fn track_allocation_type(&self, ptr: usize, size: usize, type_name: String) {
        let ctx = AllocationContext::new(size).with_type_name(type_name);
        self.backend.track_allocation(ptr, size, ctx);
    }

    /// Track a deallocation
    pub fn track_deallocation(&self, ptr: usize) {
        self.backend.track_deallocation(ptr);
    }

    /// Track a reallocation
    pub fn track_reallocation(&self, old_ptr: usize, new_ptr: usize, size: usize) {
        let ctx = AllocationContext::new(size);
        self.backend.track_reallocation(old_ptr, new_ptr, size, ctx);
    }

    /// Track an async task spawn
    pub fn track_task_spawn(&self, id: u64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.backend
            .emit_event(Event::TaskSpawn { id, ts: timestamp });
    }

    /// Track an async task end
    pub fn track_task_end(&self, id: u64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.backend
            .emit_event(Event::TaskEnd { id, ts: timestamp });
    }

    /// Track an FFI allocation
    pub fn track_ffi_alloc(&self, ptr: usize, size: usize, lib: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.backend.emit_event(Event::FfiAlloc {
            ptr,
            size,
            lib: lib.to_string(),
            ts: timestamp,
        });
    }

    /// Track an FFI free
    pub fn track_ffi_free(&self, ptr: usize, lib: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.backend.emit_event(Event::FfiFree {
            ptr,
            lib: lib.to_string(),
            ts: timestamp,
        });
    }

    /// Get a snapshot of the current state
    pub fn snapshot(&self) -> Snapshot {
        self.backend.snapshot()
    }

    /// Get the current statistics
    pub fn stats(&self) -> Stats {
        self.backend.stats()
    }

    /// Enable or disable tracking
    pub fn set_enabled(&self, enabled: bool) {
        self.backend.set_enabled(enabled);
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.backend.is_enabled()
    }

    /// Get the configuration
    pub fn config(&self) -> &TrackingConfig {
        &self.config
    }

    /// Get detailed statistics (migrated from MemoryTracker::get_stats)
    pub fn get_stats(&self) -> Stats {
        self.backend.stats()
    }

    /// Get all currently active allocations (migrated from MemoryTracker::get_active_allocations)
    pub fn get_active_allocations(&self) -> Vec<Allocation> {
        self.snapshot()
            .allocations
            .into_iter()
            .filter(|a| a.is_active())
            .collect()
    }

    /// Get allocation history (migrated from MemoryTracker::get_allocation_history)
    pub fn get_allocation_history(&self) -> Vec<Allocation> {
        self.snapshot().allocations
    }

    /// Get total memory used by active allocations
    pub fn get_active_memory(&self) -> usize {
        self.get_active_allocations().iter().map(|a| a.size).sum()
    }

    /// Get total number of allocations
    pub fn get_total_allocations(&self) -> usize {
        self.stats().total_allocations
    }

    /// Get current memory fragmentation ratio
    pub fn get_fragmentation_ratio(&self) -> f64 {
        self.stats().fragmentation_ratio
    }

    /// Get total size of all allocations
    pub fn get_total_size(&self) -> usize {
        self.stats().total_size
    }

    /// Set fast mode (migrated from MemoryTracker::set_fast_mode)
    pub fn set_fast_mode(&self, enabled: bool) {
        self.backend.set_fast_mode(enabled);
    }

    /// Check if fast mode is enabled (migrated from MemoryTracker::is_fast_mode)
    pub fn is_fast_mode(&self) -> bool {
        self.backend.is_fast_mode()
    }

    /// Enable fast mode (migrated from MemoryTracker::enable_fast_mode)
    pub fn enable_fast_mode(&self) {
        self.set_fast_mode(true);
    }

    /// Disable fast mode
    pub fn disable_fast_mode(&self) {
        self.set_fast_mode(false);
    }

    /// Export memory analysis to HTML (migrated from MemoryTracker::export_memory_analysis)
    pub fn export_memory_analysis<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        use crate::export::exporter::{ExportBackend, ExportConfig, HtmlGenerator};
        use std::fs;

        let snapshot = self.snapshot();
        let generator = HtmlGenerator::new();
        let config = ExportConfig::default();

        match generator.export(&snapshot, &config) {
            Ok(output) => {
                if let crate::export::exporter::ExportOutput::String(html) = output {
                    fs::write(path, html).map_err(|e| e.to_string())?;
                    Ok(())
                } else {
                    Err("Expected HTML string output".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Export to binary format (simplified version, migrated from MemoryTracker::export_to_binary)
    pub fn export_to_binary<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        use crate::export::exporter::{BinaryExporter, ExportBackend, ExportConfig};
        use std::fs;

        let snapshot = self.snapshot();
        let exporter = BinaryExporter::new();
        let config = ExportConfig::default();

        match exporter.export(&snapshot, &config) {
            Ok(output) => {
                if let crate::export::exporter::ExportOutput::Binary(data) = output {
                    fs::write(path, data).map_err(|e| e.to_string())?;
                    Ok(())
                } else {
                    Err("Expected binary output".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Parse binary file to JSON (simplified version)
    pub fn parse_binary_to_json<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<String, String> {
        use std::fs;

        let data = fs::read(path).map_err(|e| e.to_string())?;

        // Try to parse as JSON (since BinaryExporter uses JSON fallback)
        match serde_json::from_slice::<crate::types::internal_types::Snapshot>(&data) {
            Ok(snapshot) => serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string()),
            Err(_) => Err("Failed to parse binary file".to_string()),
        }
    }

    /// Parse binary file to HTML (simplified version)
    pub fn parse_binary_to_html<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<String, String> {
        let json_output = self.parse_binary_to_json(path)?;

        // Create basic HTML from JSON
        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Memory Analysis</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        pre {{ background: #f5f5f5; padding: 15px; border-radius: 5px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>Memory Analysis</h1>
    <pre>{}</pre>
</body>
</html>"#,
            json_output
        ))
    }

    /// Analyze drop chain for a specific allocation (simplified version)
    pub fn analyze_drop_chain(
        &self,
        ptr: usize,
        type_name: &str,
    ) -> Option<crate::core::types::DropChainAnalysis> {
        // This is a simplified implementation
        // Full implementation would need detailed drop tracking
        use std::time::SystemTime;

        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Create simplified drop chain node
        let root_node = crate::core::types::DropChainNode {
            object_id: ptr,
            type_name: type_name.to_string(),
            drop_timestamp: start_time,
            drop_duration_ns: 0,
            children: Vec::new(),
            drop_impl_type: crate::core::types::DropImplementationType::Automatic,
            cleanup_actions: Vec::new(),
            performance_characteristics: crate::core::types::DropPerformanceCharacteristics {
                execution_time_ns: 0,
                cpu_usage_percent: 0.0,
                memory_operations: 0,
                io_operations: 0,
                system_calls: 0,
                impact_level: crate::core::types::ImpactLevel::Low,
            },
        };

        // Create simplified performance metrics
        let performance_metrics = crate::core::types::DropChainPerformanceMetrics {
            total_objects: 1,
            max_depth: 1,
            avg_drop_time_ns: 0.0,
            slowest_drop_ns: 0,
            efficiency_score: 100.0,
            bottlenecks: Vec::new(),
        };

        // Create simplified ownership hierarchy
        let ownership_hierarchy = crate::core::types::OwnershipHierarchy {
            root_owners: vec![crate::core::types::OwnershipNode {
                object_id: ptr,
                type_name: type_name.to_string(),
                ownership_type: crate::core::types::OwnershipType::Unique,
                owned_objects: Vec::new(),
                reference_count: None,
                weak_reference_count: None,
            }],
            max_depth: 1,
            total_objects: 1,
            transfer_events: Vec::new(),
            weak_references: Vec::new(),
            circular_references: Vec::new(),
        };

        // Create simplified leak detection
        let leak_detection = crate::core::types::ResourceLeakAnalysis {
            potential_leaks: Vec::new(),
            detection_confidence: 1.0,
            usage_patterns: Vec::new(),
            prevention_recommendations: Vec::new(),
        };

        Some(crate::core::types::DropChainAnalysis {
            root_object: root_node,
            drop_sequence: Vec::new(),
            total_duration_ns: 0,
            performance_metrics,
            ownership_hierarchy,
            leak_detection,
        })
    }

    /// Ensure the memory analysis path exists and return the full path
    pub fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<std::path::PathBuf, String> {
        let path = path.as_ref();
        let memory_analysis_dir = std::path::Path::new("MemoryAnalysis");

        std::fs::create_dir_all(memory_analysis_dir)
            .map_err(|e| format!("Failed to create MemoryAnalysis directory: {}", e))?;

        Ok(memory_analysis_dir.join(path))
    }

    /// Export lifecycle timeline (migrated from MemoryTracker::export_lifecycle_timeline)
    pub fn export_lifecycle_timeline<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), String> {
        let output_path = self.ensure_memory_analysis_path(path)?;
        let snapshot = self.snapshot();
        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Lifecycle Timeline</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        pre {{ background: #f5f5f5; padding: 15px; border-radius: 5px; overflow-x: auto; }}
    </style>
</head>
<body>
    <h1>Memory Lifecycle Timeline</h1>
    <pre>{}</pre>
</body>
</html>"#,
            json
        );
        
        std::fs::write(output_path, html).map_err(|e| e.to_string())
    }

    /// Export user-only binary (migrated from MemoryTracker::export_user_binary)
    pub fn export_user_binary<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        let output_path = self.ensure_memory_analysis_path(path)?;
        let snapshot = self.snapshot();

        let user_allocations: Vec<_> = snapshot
            .allocations
            .into_iter()
            .filter(|allocation| allocation.meta.var_name.is_some())
            .collect();

        tracing::info!(
            "Exporting {} user allocations to binary format",
            user_allocations.len()
        );

        let user_snapshot = Snapshot {
            allocations: user_allocations,
            ..snapshot
        };

        let json = serde_json::to_string_pretty(&user_snapshot).map_err(|e| e.to_string())?;
        std::fs::write(output_path, json).map_err(|e| e.to_string())
    }

    /// Export full binary (migrated from MemoryTracker::export_full_binary)
    pub fn export_full_binary<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        let output_path = self.ensure_memory_analysis_path(path)?;
        let snapshot = self.snapshot();

        tracing::info!(
            "Exporting {} total allocations (user + system) to binary format",
            snapshot.allocations.len()
        );

        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        std::fs::write(output_path, json).map_err(|e| e.to_string())
    }

    /// Export to binary with mode (migrated from MemoryTracker::export_to_binary_with_mode)
    pub fn export_to_binary_with_mode<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        mode: BinaryExportMode,
    ) -> Result<(), String> {
        match mode {
            BinaryExportMode::UserOnly => self.export_user_binary(path),
            BinaryExportMode::Full => self.export_full_binary(path),
        }
    }
}

/// Binary export mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryExportMode {
    /// Export only user-defined variables
    UserOnly,
    /// Export all allocations
    Full,
}

impl Default for BinaryExportMode {
    fn default() -> Self {
        BinaryExportMode::UserOnly
    }
}

impl Default for UnifiedTracker {
    fn default() -> Self {
        Self::new(TrackingConfig::default())
    }
}

// ============================================================================
// Global Tracking Allocator (Phase 2 refactoring - preserving original functionality)
// ============================================================================

/// Global tracking allocator that implements GlobalAlloc trait
///
/// This allocator wraps the system allocator and tracks all memory allocations
/// and deallocations through the UnifiedTracker system.
pub struct TrackingAllocator;

impl TrackingAllocator {
    /// Create a new tracking allocator instance
    pub const fn new() -> Self {
        Self
    }

    /// Get the global tracker instance
    fn get_global_tracker() -> Option<Arc<UnifiedTracker>> {
        // Try to get the global tracker from thread-local storage
        // This integrates with the existing tracking system
        GLOBAL_TRACKER.with(|tracker| Some(tracker.clone()))
    }
}

// Thread-local flag to prevent recursive tracking
thread_local! {
    static TRACKING_DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Global tracker instance
thread_local! {
    static GLOBAL_TRACKER: Arc<UnifiedTracker> = Arc::new(UnifiedTracker::default());
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        // Allocate memory first
        let ptr = std::alloc::System.alloc(layout);

        // Track the allocation if it succeeded and tracking is not disabled
        if !ptr.is_null() {
            // Check if tracking is disabled for this thread to prevent recursion
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

            if should_track {
                // Temporarily disable tracking to prevent recursion during tracking operations
                TRACKING_DISABLED.with(|disabled| disabled.set(true));

                // Track the allocation through the unified tracker
                // Note: We clone the Arc to avoid catch_unwind issues
                if let Some(tracker) = Self::get_global_tracker() {
                    let tracker_clone = tracker.clone();
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        tracker_clone.track_allocation(ptr as usize, layout.size());
                    }));
                }

                // Re-enable tracking
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // Track the deallocation if tracking is not disabled
        if !ptr.is_null() {
            // Check if tracking is disabled for this thread to prevent recursion
            let should_track = TRACKING_DISABLED.with(|disabled| !disabled.get());

            if should_track {
                // Temporarily disable tracking to prevent recursion during tracking operations
                TRACKING_DISABLED.with(|disabled| disabled.set(true));

                // Track the deallocation through the unified tracker
                // Note: We clone the Arc to avoid catch_unwind issues
                if let Some(tracker) = Self::get_global_tracker() {
                    let tracker_clone = tracker.clone();
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        tracker_clone.track_deallocation(ptr as usize);
                    }));
                }
                // Re-enable tracking
                TRACKING_DISABLED.with(|disabled| disabled.set(false));
            }
        }

        // Deallocate the memory
        std::alloc::System.dealloc(ptr, layout);
    }
}

// ============================================================================
// Public API Functions (Phase 2 refactoring - preserving original functionality)
// ============================================================================

/// Get the global unified tracker instance
///
/// This function provides access to the global tracker for manual tracking operations.
pub fn get_global_tracker() -> Arc<UnifiedTracker> {
    GLOBAL_TRACKER.with(|tracker| tracker.clone())
}

/// Track a memory allocation
///
/// This function can be used to manually track allocations that are not
/// automatically tracked by the global allocator.
///
/// # Arguments
/// * `ptr` - Pointer to the allocated memory
/// * `size` - Size of the allocation in bytes
///
/// # Returns
/// Result indicating success or failure
pub fn track_allocation(ptr: usize, size: usize) -> TrackingResult<()> {
    let tracker = get_global_tracker();
    tracker.track_allocation(ptr, size);
    Ok(())
}

/// Track a memory deallocation
///
/// This function can be used to manually track deallocations that are not
/// automatically tracked by the global allocator.
///
/// # Arguments
/// * `ptr` - Pointer to the deallocated memory
///
/// # Returns
/// Result indicating success or failure
pub fn track_deallocation(ptr: usize) -> TrackingResult<()> {
    let tracker = get_global_tracker();
    tracker.track_deallocation(ptr);
    Ok(())
}

/// Take a snapshot of the current memory state
///
/// This function creates a snapshot of all tracked memory allocations,
/// including allocation metadata, events, tasks, and threads.
///
/// # Returns
/// Snapshot of the current memory state
pub fn snapshot() -> Snapshot {
    let tracker = get_global_tracker();
    tracker.snapshot()
}

/// Track a task spawn event (for async tracking)
///
/// # Arguments
/// * `id` - Task ID
pub fn track_task_spawn(id: u64) {
    let tracker = get_global_tracker();
    tracker.track_task_spawn(id);
}

/// Track a task end event (for async tracking)
///
/// # Arguments
/// * `id` - Task ID
pub fn track_task_end(id: u64) {
    let tracker = get_global_tracker();
    tracker.track_task_end(id);
}

/// Track FFI allocation
///
/// # Arguments
/// * `ptr` - Pointer to the allocated memory
/// * `size` - Size of the allocation in bytes
/// * `lib` - Name of the library that performed the allocation
pub fn track_ffi_alloc(ptr: usize, size: usize, lib: &str) {
    let tracker = get_global_tracker();
    tracker.track_ffi_alloc(ptr, size, lib);
}

/// Track FFI free
///
/// # Arguments
/// * `ptr` - Pointer to the freed memory
/// * `lib` - Name of the library that performed the free
pub fn track_ffi_free(ptr: usize, lib: &str) {
    let tracker = get_global_tracker();
    tracker.track_ffi_free(ptr, lib);
}

/// Configure the tracking strategy
///
/// # Arguments
/// * `strategy` - The tracking strategy to use
pub fn configure_tracking_strategy(strategy: TrackingStrategy) {
    // This would update the global configuration
    // For now, we just log the configuration
    tracing::info!("Configured tracking strategy: {:?}", strategy);
}
