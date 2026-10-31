//! Core memory tracker implementation (self-contained, no old system dependencies)
//!
//! This module provides high-performance memory tracking using DashMap
//! and atomic operations, completely independent of the old system.

use super::core_types::{
    AllocationInfo, MemoryStats, ThreadRegistryStats, TrackingError, TrackingResult,
};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;

const STRATEGY_GLOBAL_SINGLETON: u64 = 0;
const STRATEGY_THREAD_LOCAL: u64 = 1;

pub static TRACKING_STRATEGY: AtomicU64 = AtomicU64::new(STRATEGY_GLOBAL_SINGLETON);

static GLOBAL_TRACKER: OnceLock<Arc<MemoryTracker>> = OnceLock::new();

thread_local! {
    static THREAD_LOCAL_TRACKER: Arc<MemoryTracker> = {
        let tracker = Arc::new(MemoryTracker::new());
        register_current_thread_tracker_local(&tracker);
        tracker
    };
}

/// Thread-local registry for tracking thread-local trackers
static LOCAL_THREAD_REGISTRY: OnceLock<
    Arc<dashmap::DashMap<thread::ThreadId, Arc<MemoryTracker>>>,
> = OnceLock::new();

fn get_local_registry() -> Arc<dashmap::DashMap<thread::ThreadId, Arc<MemoryTracker>>> {
    LOCAL_THREAD_REGISTRY
        .get_or_init(|| Arc::new(dashmap::DashMap::new()))
        .clone()
}

fn register_current_thread_tracker_local(tracker: &Arc<MemoryTracker>) {
    let thread_id = thread::current().id();
    get_local_registry().insert(thread_id, tracker.clone());
}

/// Core memory tracking functionality.
///
/// The MemoryTracker maintains records of all memory allocations and deallocations,
/// provides statistics, and supports exporting data in various formats.
///
/// # Performance Characteristics
/// - **DashMap**: Lock-free concurrent access for allocations
/// - **Atomic counters**: Lock-free statistics updates
/// - **Thread-local**: Zero-contention for per-thread tracking
pub struct MemoryTracker {
    /// Live bindings per address, oldest first (DashMap for lock-free concurrent
    /// access).
    ///
    /// One address can hold more than one live binding: when a second allocation is
    /// observed at an address whose previous block was never released, both blocks are
    /// facts. Keeping them apart (instead of overwriting) is what makes
    /// `(ptr, binding_serial)` a usable identity (plan PR-02).
    active_allocations: DashMap<usize, Vec<AllocationInfo>>,

    /// Atomic statistics (lock-free updates)
    total_allocations: AtomicU64,
    total_allocated: AtomicU64,
    total_deallocations: AtomicU64,
    total_deallocated: AtomicU64,
    peak_allocations: AtomicUsize,
    peak_memory: AtomicU64,
    /// Number of live bindings (an address may hold several). Kept as a counter so the
    /// peak-update path stays O(1); scanning the map per allocation would make the hot
    /// path quadratic in the number of addresses.
    live_bindings: AtomicUsize,

    /// Number of sightings of an already tracked address that did not change its
    /// size (duplicate `track!`, allocator hook + typed observation of the same
    /// buffer). Used as a diagnostic: it must never reach the byte counters.
    duplicate_observations: AtomicU64,
    /// Monotonic identity assigned to every binding this tracker creates.
    next_binding_serial: AtomicU64,
    /// Number of allocations that arrived at an address which still had a live
    /// binding. Reported as a diagnostic: the capture missed a `free` (or saw two
    /// different blocks at one address), so no release can be attributed reliably.
    replaced_bindings: AtomicU64,
    /// Number of releases that could only be matched by address, because the address
    /// held more than one live binding. Those closes are guesses and are reported so a
    /// consumer never mistakes them for an exact release (plan PR-02).
    degraded_releases: AtomicU64,

    /// Fast mode flag for testing (reduces overhead)
    fast_mode: AtomicU64,
}

impl MemoryTracker {
    /// Create a new memory tracker.
    pub fn new() -> Self {
        let fast_mode = std::env::var("MEMSCOPE_TEST_MODE").is_ok() || cfg!(test);

        Self {
            active_allocations: DashMap::new(),
            total_allocations: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            peak_allocations: AtomicUsize::new(0),
            peak_memory: AtomicU64::new(0),
            live_bindings: AtomicUsize::new(0),
            next_binding_serial: AtomicU64::new(1),
            duplicate_observations: AtomicU64::new(0),
            replaced_bindings: AtomicU64::new(0),
            degraded_releases: AtomicU64::new(0),
            fast_mode: AtomicU64::new(fast_mode as u64),
        }
    }

    /// Track a memory allocation or a typed observation of one.
    ///
    /// # Re-observation vs. address reuse
    ///
    /// The same address can be reported more than once: a `track!` on a buffer the
    /// allocator hook already recorded, two `track!` calls on the same variable, or a
    /// *different* block that landed on a reused address whose `free` was lost.
    ///
    /// * Identical size → duplicate observation: metadata refresh only, no accounting.
    /// * Different size → **new binding**. An observation cannot prove that the old
    ///   block is still the same block, so the previous row leaves the live set and the
    ///   new one gets its own identity. Reusing the row ("in-place realloc") would merge
    ///   two allocations, and a later release by raw address would then delete the wrong
    ///   one. Only [`MemoryTracker::track_reallocation`] may claim identity continuity,
    ///   because only the allocator can observe it.
    ///
    /// This keeps `total_allocated - total_deallocated == Σ live sizes` (plan A06) while
    /// never inventing a generation merge.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    pub fn track_allocation(&self, ptr: usize, size: usize) -> TrackingResult<()> {
        self.record_binding(ptr, size, None)
    }

    /// Shared bookkeeping for `track_allocation` / `fast_track_allocation`.
    fn record_binding(
        &self,
        ptr: usize,
        size: usize,
        var_name: Option<String>,
    ) -> TrackingResult<()> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);

        let mut entries = self.active_allocations.entry(ptr).or_default();

        if let Some(existing) = entries.last_mut() {
            if existing.size == size {
                // Same address, same size: another observation of the same block.
                if var_name.is_some() {
                    existing.var_name = var_name;
                }
                self.duplicate_observations.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }

            // Same address, different size, no observed release in between: either an
            // address reuse whose release was lost, or an in-place reallocation. An
            // observation cannot tell them apart, so both blocks stay live and the
            // condition is reported. Merging them would destroy the old identity and
            // make a later release delete the wrong one.
            self.replaced_bindings.fetch_add(1, Ordering::Relaxed);
            tracing::debug!(
                "conflicting allocation: address {:#x} still holds a live binding",
                ptr
            );
        }

        let mut allocation = AllocationInfo::new(ptr, size);
        allocation.binding_serial = self.next_binding_serial.fetch_add(1, Ordering::Relaxed);
        if var_name.is_some() {
            allocation.var_name = var_name;
        }
        entries.push(allocation);
        drop(entries);
        self.live_bindings.fetch_add(1, Ordering::Relaxed);

        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated
            .fetch_add(size as u64, Ordering::Relaxed);
        self.update_peak_allocations();
        self.update_peak_memory();
        Ok(())
    }

    /// Number of live bindings across all addresses.
    fn live_binding_count(&self) -> usize {
        self.live_bindings.load(Ordering::Relaxed)
    }

    /// Number of live bindings currently recorded at `ptr`.
    pub fn live_binding_count_at(&self, ptr: usize) -> usize {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        self.active_allocations
            .get(&ptr)
            .map(|entries| entries.len())
            .unwrap_or(0)
    }

    /// Refresh the high-water mark for the number of live allocations.
    fn update_peak_allocations(&self) {
        // CAS loop avoids the TOCTOU race of read-then-store.
        loop {
            let current_count = self.live_binding_count();
            let current_peak = self.peak_allocations.load(Ordering::Relaxed);
            if current_count <= current_peak {
                break;
            }
            if self
                .peak_allocations
                .compare_exchange_weak(
                    current_peak,
                    current_count,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }
    }

    /// Refresh the high-water mark for live bytes.
    fn update_peak_memory(&self) {
        loop {
            let current_memory = self
                .total_allocated
                .load(Ordering::Relaxed)
                .saturating_sub(self.total_deallocated.load(Ordering::Relaxed));
            let current_peak_memory = self.peak_memory.load(Ordering::Relaxed);
            if current_memory <= current_peak_memory {
                break;
            }
            if self
                .peak_memory
                .compare_exchange_weak(
                    current_peak_memory,
                    current_memory,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }
    }

    /// Track a memory deallocation by address.
    ///
    /// This is the **degraded** path: an address alone cannot name a binding, so when
    /// the address holds more than one live binding the oldest one is closed and the
    /// guess is counted in [`Self::degraded_releases`]. Callers that know which block
    /// they are releasing must use [`Self::track_deallocation_of`] (plan PR-02).
    ///
    /// # Returns
    /// * `Ok(true)` if a binding was found and closed
    /// * `Ok(false)` if the pointer was not tracked (possible double-free or untracked
    ///   allocation)
    pub fn track_deallocation(&self, ptr: usize) -> TrackingResult<bool> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        self.close_binding(ptr, None)
    }

    /// Track a memory deallocation of one specific binding.
    ///
    /// `serial` is the identity returned by [`Self::binding_serial`]; only that binding
    /// may be closed, so a release that arrives after the address was reused cannot
    /// delete the new block.
    pub fn track_deallocation_of(&self, ptr: usize, serial: u64) -> TrackingResult<bool> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        self.close_binding(ptr, Some(serial))
    }

    /// Close a binding, returning whether one was closed.
    fn close_binding(&self, ptr: usize, serial: Option<u64>) -> TrackingResult<bool> {
        let closed = {
            let Some(mut entries) = self.active_allocations.get_mut(&ptr) else {
                return Ok(false);
            };

            let index = match serial {
                Some(serial) => entries
                    .iter()
                    .position(|binding| binding.binding_serial == serial),
                None => {
                    if entries.len() > 1 {
                        self.degraded_releases.fetch_add(1, Ordering::Relaxed);
                        tracing::debug!(
                            "degraded release: address {:#x} holds {} bindings, closing the oldest",
                            ptr,
                            entries.len()
                        );
                    }
                    Some(0)
                }
            };

            let Some(index) = index else {
                // The named binding is already gone: a late release of a superseded
                // generation must not touch the blocks that replaced it.
                return Ok(false);
            };

            let closed = entries.remove(index);
            if entries.is_empty() {
                drop(entries);
                self.active_allocations.remove(&ptr);
            }
            closed
        };

        self.live_bindings.fetch_sub(1, Ordering::Relaxed);
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.total_deallocated
            .fetch_add(closed.size as u64, Ordering::Relaxed);
        Ok(true)
    }

    /// Track a reallocation of a tracked block.
    ///
    /// The block keeps its identity (and its binding serial): the new size replaces the
    /// old one and the allocation counter does not change, whether the allocator kept the
    /// address (`old_ptr == new_ptr`) or moved the block. Metadata recorded by `track!` is
    /// carried over to the new address.
    ///
    /// # Arguments
    /// * `old_ptr` - Address before the reallocation
    /// * `new_ptr` - Address returned by the allocator
    /// * `old_size` - Size the caller asked to reallocate. Kept for API symmetry; the
    ///   cache trusts the size it recorded, and an unobserved block books nothing.
    /// * `new_size` - Size of the resulting block
    pub fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        _old_size: usize,
        new_size: usize,
    ) -> TrackingResult<()> {
        let _gate = crate::core::allocator::capture_gate::acquire(old_ptr);

        let previous = {
            let Some(mut entries) = self.active_allocations.get_mut(&old_ptr) else {
                return self.open_unmatched_binding(new_ptr, new_size);
            };
            // A reallocation applies to the block the caller holds; the newest binding at
            // the address is the only defensible choice when the capture also holds older
            // generations.
            let previous = entries.pop();
            if entries.is_empty() {
                drop(entries);
                self.active_allocations.remove(&old_ptr);
            }
            previous
        };

        let Some(mut allocation) = previous else {
            return self.open_unmatched_binding(new_ptr, new_size);
        };

        // A matched reallocation keeps one ledger row: the old size leaves the live
        // set, the new size enters it, and the allocation count does not change.
        let previous_size = allocation.size;
        allocation.ptr = new_ptr;
        allocation.size = new_size;
        self.active_allocations
            .entry(new_ptr)
            .or_default()
            .push(allocation);

        self.total_allocated
            .fetch_add(new_size as u64, Ordering::Relaxed);
        self.total_deallocated
            .fetch_add(previous_size as u64, Ordering::Relaxed);
        self.update_peak_memory();
        Ok(())
    }

    /// Record a block whose previous size was never observed.
    ///
    /// Only the new block is a fact, so nothing may be booked as released for the old
    /// one (that would push `allocated - deallocated` below the live set).
    fn open_unmatched_binding(&self, new_ptr: usize, new_size: usize) -> TrackingResult<()> {
        let mut new_binding = AllocationInfo::new(new_ptr, new_size);
        new_binding.binding_serial = self.next_binding_serial.fetch_add(1, Ordering::Relaxed);
        self.active_allocations
            .entry(new_ptr)
            .or_default()
            .push(new_binding);
        self.live_bindings.fetch_add(1, Ordering::Relaxed);
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.total_allocated
            .fetch_add(new_size as u64, Ordering::Relaxed);
        self.update_peak_allocations();
        self.update_peak_memory();
        Ok(())
    }

    /// Get the size of an active allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    ///
    /// # Returns
    /// * `Some(size)` if the allocation exists
    /// * `None` if the pointer is not tracked
    pub fn get_allocation_size(&self, ptr: usize) -> Option<usize> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        self.active_allocations
            .get(&ptr)
            .and_then(|entries| entries.last().map(|binding| binding.size))
    }

    /// Associate a variable name and type with an allocation.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `var_name` - Variable name
    /// * `type_name` - Type name
    /// * `source_file` - Source file (optional)
    /// * `source_line` - Source line (optional)
    pub fn associate_var(
        &self,
        ptr: usize,
        var_name: String,
        type_name: String,
        source_file: Option<&str>,
        source_line: Option<u32>,
    ) -> TrackingResult<()> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        if let Some(mut entries) = self.active_allocations.get_mut(&ptr) {
            if let Some(allocation) = entries.last_mut() {
                allocation.var_name = Some(var_name);
                allocation.type_name = Some(type_name);
                if let (Some(file), Some(line)) = (source_file, source_line) {
                    allocation.set_source_location(file, line);
                }
            }
        }

        Ok(())
    }

    /// Fast track allocation with variable name.
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    /// * `var_name` - Variable name
    pub fn fast_track_allocation(
        &self,
        ptr: usize,
        size: usize,
        var_name: String,
    ) -> TrackingResult<()> {
        self.record_binding(ptr, size, Some(var_name))
    }

    /// Identity of the live binding at `ptr`, if any.
    ///
    /// Changes whenever a *new* block is recorded at that address, so callers can detect
    /// address reuse instead of assuming the same allocation (plan A06/PR-02).
    pub fn binding_serial(&self, ptr: usize) -> Option<u64> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        self.active_allocations
            .get(&ptr)
            .and_then(|entries| entries.last().map(|binding| binding.binding_serial))
    }

    /// Number of duplicate sightings of already tracked addresses.
    ///
    /// A non-zero value is expected (hook registration plus typed observation of
    /// the same buffer); it is reported separately so it can never be mistaken for
    /// allocated bytes.
    pub fn duplicate_observations(&self) -> u64 {
        self.duplicate_observations.load(Ordering::Relaxed)
    }

    /// Number of bindings superseded at the same address without an observed release.
    ///
    /// Non-zero means the capture missed a `free`, or two different blocks shared an
    /// address: the bytes stay conserved, but the two allocations are not the same one.
    pub fn replaced_bindings(&self) -> u64 {
        self.replaced_bindings.load(Ordering::Relaxed)
    }

    /// Number of releases that could only be matched by address.
    ///
    /// Non-zero means a release arrived for an address that held several live bindings,
    /// so the closed binding was a guess. Callers that know the identity should use
    /// `track_deallocation_of` instead.
    pub fn degraded_releases(&self) -> u64 {
        self.degraded_releases.load(Ordering::Relaxed)
    }

    /// Get current memory statistics.
    pub fn get_stats(&self) -> TrackingResult<MemoryStats> {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        let active_count = self.live_binding_count();
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);
        let total_deallocated = self.total_deallocated.load(Ordering::Relaxed);
        let active_memory = total_allocated.saturating_sub(total_deallocated);

        Ok(MemoryStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_allocated,
            active_allocations: active_count,
            active_memory,
            peak_allocations: self.peak_allocations.load(Ordering::Relaxed),
            peak_memory: self.peak_memory.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            total_deallocated,
            leaked_allocations: 0,
            leaked_memory: 0,
        })
    }

    /// Detect memory leaks at program shutdown.
    ///
    /// This should be called when the program is shutting down to detect
    /// allocations that were never freed. Returns the count and total size
    /// of allocations that are still active.
    pub fn detect_leaks(&self) -> (usize, u64) {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        let active_count = self.live_binding_count();
        let total_allocated = self.total_allocated.load(Ordering::Relaxed);
        let total_deallocated = self.total_deallocated.load(Ordering::Relaxed);
        let active_memory = total_allocated.saturating_sub(total_deallocated);
        (active_count, active_memory)
    }

    /// Get all currently active allocations.
    pub fn get_active_allocations(&self) -> TrackingResult<Vec<AllocationInfo>> {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        let mut allocations = Vec::new();
        for entry in self.active_allocations.iter() {
            allocations.extend(entry.value().iter().cloned());
        }
        Ok(allocations)
    }

    /// Get memory grouped by type.
    pub fn get_memory_by_type(&self) -> TrackingResult<std::collections::HashMap<String, usize>> {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        let mut type_sizes: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for entry in self.active_allocations.iter() {
            for alloc in entry.value() {
                let type_name = alloc
                    .type_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
                *type_sizes.entry(type_name).or_insert(0) += alloc.size;
            }
        }

        Ok(type_sizes)
    }

    /// Enable or disable fast mode.
    pub fn set_fast_mode(&self, enabled: bool) {
        self.fast_mode.store(enabled as u64, Ordering::Relaxed);
    }

    /// Check if fast mode is enabled.
    pub fn is_fast_mode(&self) -> bool {
        self.fast_mode.load(Ordering::Relaxed) != 0
    }

    /// Enable fast mode for testing.
    pub fn enable_fast_mode(&self) {
        self.fast_mode.store(1, Ordering::Relaxed);
    }

    /// Ensure memory analysis path exists and return the full path.
    pub fn ensure_memory_analysis_path<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::path::PathBuf {
        let path = path.as_ref();
        let base_dir = path
            .parent()
            .unwrap_or(std::path::Path::new("MemoryAnalysis"));

        if let Err(e) = std::fs::create_dir_all(base_dir) {
            tracing::warn!("Failed to create directory {:?}: {}", base_dir, e);
        }

        if base_dir == std::path::Path::new("") {
            std::path::Path::new("MemoryAnalysis").join(path.file_name().unwrap_or_default())
        } else {
            path.to_path_buf()
        }
    }

    /// Ensure path uses .memscope extension and is in MemoryAnalysis directory.
    pub fn ensure_memscope_path<P: AsRef<std::path::Path>>(&self, path: P) -> std::path::PathBuf {
        let mut output_path = self.ensure_memory_analysis_path(path);

        if output_path.extension().is_none()
            || output_path.extension() != Some(std::ffi::OsStr::new("memscope"))
        {
            output_path.set_extension("memscope");
        }

        output_path
    }

    /// Export memory tracking data to .memscope file format (JSON content).
    ///
    /// This method exports memory tracking data with a .memscope file extension.
    /// The content is serialized as JSON for human readability and interoperability.
    pub fn export_to_memscope<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        self.export_to_json(path)
    }

    /// Export memory tracking data to JSON format.
    pub fn export_to_json<P: AsRef<std::path::Path>>(&self, path: P) -> TrackingResult<()> {
        let output_path = self.ensure_memory_analysis_path(&path);

        let final_path = if output_path.is_dir() {
            output_path.join("memory_analysis.json")
        } else {
            output_path
        };

        let allocations = self.get_active_allocations()?;

        let json = serde_json::to_string_pretty(&allocations)
            .map_err(|e| TrackingError::SerializationError(e.to_string()))?;

        std::fs::write(&final_path, json).map_err(|e| TrackingError::ExportError(e.to_string()))?;

        Ok(())
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryTracker {
    fn drop(&mut self) {
        if std::env::var("MEMSCOPE_VERBOSE").is_ok() {
            tracing::info!(
                "💡 Tip: Use tracker.export_to_json() before drop to save analysis results"
            );
        }

        let active_count = self.live_binding_count();
        if active_count > 0 {
            tracing::warn!(
                "Dropping MemoryTracker with {} active allocations (potential memory leaks)",
                active_count
            );
        }

        // Clear active_allocations to release memory
        self.active_allocations.clear();
        self.live_bindings.store(0, Ordering::Relaxed);
    }
}

/// Configure tracking strategy for the application.
///
/// # Arguments
/// * `is_concurrent` - true for multi-threaded/async applications, false for single-threaded
pub fn configure_tracking_strategy(is_concurrent: bool) {
    let strategy = if is_concurrent {
        STRATEGY_THREAD_LOCAL
    } else {
        STRATEGY_GLOBAL_SINGLETON
    };

    TRACKING_STRATEGY.store(strategy, Ordering::Relaxed);

    tracing::info!(
        "Configured tracking strategy: {}",
        if is_concurrent {
            "thread-local"
        } else {
            "global-singleton"
        }
    );
}

/// Get the appropriate memory tracker based on the current strategy.
///
/// # Returns
/// * In single-threaded mode: returns the global singleton tracker
/// * In concurrent mode: returns the current thread's local tracker
pub fn get_tracker() -> Arc<MemoryTracker> {
    match TRACKING_STRATEGY.load(Ordering::Relaxed) {
        STRATEGY_GLOBAL_SINGLETON => GLOBAL_TRACKER
            .get_or_init(|| Arc::new(MemoryTracker::new()))
            .clone(),
        STRATEGY_THREAD_LOCAL => THREAD_LOCAL_TRACKER.with(|tracker| tracker.clone()),
        _ => {
            tracing::warn!("Unknown tracking strategy, falling back to global singleton");
            GLOBAL_TRACKER
                .get_or_init(|| Arc::new(MemoryTracker::new()))
                .clone()
        }
    }
}

/// Collect all thread-local trackers.
pub fn collect_all_trackers_local() -> Vec<Arc<MemoryTracker>> {
    get_local_registry()
        .iter()
        .map(|entry| entry.value().clone())
        .collect()
}

/// Get registry statistics.
pub fn get_registry_stats_local() -> ThreadRegistryStats {
    let registry = get_local_registry();
    let total_threads = registry.len();

    ThreadRegistryStats {
        total_threads_registered: total_threads,
        active_threads: total_threads,
        dead_references: 0,
    }
}

/// Check if there are active trackers.
pub fn has_active_trackers_local() -> bool {
    !get_local_registry().is_empty()
}

#[cfg(test)]
mod tests;
