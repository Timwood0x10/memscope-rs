//! Unified Tracker API - Enhanced Version
//!
//! This module provides a simple, unified interface for memory tracking
//! with all the power of the old API and more.
//!
//! # Features
//!
//! - **Simple API**: `tracker!()` and `track!()` macros
//! - **Auto-capture**: Automatic variable name and type capture
//! - **System Monitoring**: CPU, memory monitoring (background thread, zero overhead)
//! - **Per-thread Tracking**: Independent tracking per thread
//! - **Sampling**: Configurable sampling rates
//! - **Hotspot Analysis**: Automatic allocation hotspot detection
//! - **HTML Dashboard**: Interactive visualization
//! - **JSON/Binary Export**: Multiple export formats
//!
//! # Architecture
//!
//! System monitoring runs in a background thread that collects metrics every 100ms.
//! The `track!` macro only reads atomic values (nanosecond overhead), ensuring
//! no blocking on data collection.
//!
//! # Usage
//!
//! ```rust
//! use memscope_rs::{tracker, track};
//!
//! // Simple usage - system monitoring is automatic
//! let tracker = tracker!();
//! let my_vec = vec![1, 2, 3];
//! track!(tracker, my_vec);
//! // Analyze the tracked allocations
//! let report = tracker.analyze();
//!
//! // Advanced usage with custom sampling
//! use memscope_rs::tracker::SamplingConfig;
//! let tracker = tracker!().with_sampling(SamplingConfig::high_performance());
//! ```

use crate::capture::system_monitor;
use crate::core::tracker::MemoryTracker;
use crate::event_store::{EventStore, MemoryEvent};
use crate::render_engine::dashboard::renderer::rebuild_allocations_from_events;
use crate::render_engine::export::{export_snapshot_to_json, ExportJsonOptions};
use crate::snapshot::MemorySnapshot;

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub sample_rate: f64,
    pub capture_call_stack: bool,
    pub max_stack_depth: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 1.0,
            capture_call_stack: false,
            max_stack_depth: 10,
        }
    }
}

impl SamplingConfig {
    pub fn demo() -> Self {
        Self {
            sample_rate: 0.1,
            capture_call_stack: false,
            max_stack_depth: 5,
        }
    }

    pub fn full() -> Self {
        Self {
            sample_rate: 1.0,
            capture_call_stack: true,
            max_stack_depth: 20,
        }
    }

    pub fn high_performance() -> Self {
        Self {
            sample_rate: 0.01,
            capture_call_stack: false,
            max_stack_depth: 0,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub thread_count: usize,
    pub disk_read_bps: u64,
    pub disk_write_bps: u64,
    pub network_rx_bps: u64,
    pub network_tx_bps: u64,
    pub gpu_usage_percent: f64,
    pub gpu_memory_used: u64,
    pub gpu_memory_total: u64,
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub active_allocations: usize,
    pub peak_memory_bytes: u64,
    pub current_memory_bytes: u64,
    pub allocation_rate_per_sec: f64,
    pub deallocation_rate_per_sec: f64,
    pub hotspots: Vec<AllocationHotspot>,
    pub system_snapshots: Vec<SystemSnapshot>,
}

#[derive(Debug, Clone)]
pub struct AllocationHotspot {
    pub var_name: String,
    pub type_name: String,
    pub total_size: usize,
    pub allocation_count: usize,
    pub location: Option<String>,
}

/// Serialised slot holding what this tracker reported for one address.
type RecordedSlot = Arc<Mutex<Option<RecordedBinding>>>;
/// Address -> recorded slot. The map is shared by all clones of a tracker.
type RecordedBindings = Arc<DashMap<usize, RecordedSlot>>;

/// What this tracker has already reported for an address.
///
/// The core cache can be populated by the allocator hook without any event, and a
/// released address can be reused by a *new* block. Both cases must be told apart from
/// "this tracker already emitted the event", which is what `serial` (binding identity)
/// and `typed` (metadata present) express.
#[derive(Debug, Clone, Copy)]
struct RecordedBinding {
    /// Size recorded with the event.
    size: usize,
    /// Core-tracker binding identity at the time of the event.
    serial: u64,
    /// Whether the recorded event carried type information.
    typed: bool,
}

pub struct Tracker {
    inner: Arc<MemoryTracker>,
    event_store: Arc<EventStore>,
    /// Addresses for which *this* tracker already emitted an allocation event, with the
    /// size recorded at that moment.
    ///
    /// The core cache can be populated by the allocator hook without any event
    /// (`Tracker::global` shares the hook's tracker), so cache presence alone must not
    /// be treated as "already reported".
    /// One slot per address; the slot also serialises the "first record" decision so
    /// concurrent observers of the same block cannot each emit an event.
    recorded_bindings: RecordedBindings,
    config: Arc<Mutex<TrackerConfig>>,
    start_time: Instant,
    system_snapshots: Arc<Mutex<Vec<SystemSnapshot>>>,
}

pub(crate) mod allocator_events;

impl Clone for Tracker {
    fn clone(&self) -> Self {
        Tracker {
            inner: self.inner.clone(),
            event_store: self.event_store.clone(),
            recorded_bindings: self.recorded_bindings.clone(),
            config: self.config.clone(),
            start_time: Instant::now(), // Use current time for cloned tracker
            system_snapshots: self.system_snapshots.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackerConfig {
    sampling: SamplingConfig,
    auto_export_on_drop: bool,
    export_path: Option<String>,
}

impl Tracker {
    pub fn new() -> Self {
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        Self {
            inner: Arc::new(MemoryTracker::new()),
            event_store: Arc::new(EventStore::new()),
            recorded_bindings: Arc::new(DashMap::new()),
            config: Arc::new(Mutex::new(TrackerConfig {
                sampling: SamplingConfig::default(),
                auto_export_on_drop: false,
                export_path: None,
            })),
            start_time: Instant::now(),
            system_snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Construct a `Tracker` from a [`global_tracking::TrackerConfig`].
    ///
    /// This is the bridge between the user-facing `GlobalTrackerConfig.tracker`
    /// field and the internal `Tracker` construction. Currently the underlying
    /// `MemoryTracker` does not honor `max_allocations` (it is unbounded by
    /// design), so this method consumes the config without applying that knob.
    /// `enable_statistics` is implicitly always true — the tracker always
    /// collects statistics; future implementations may gate this.
    ///
    /// The method exists so that `GlobalTracker::with_config` does not silently
    /// discard its `config.tracker` field.
    ///
    /// [`global_tracking::TrackerConfig`]: crate::capture::backends::global_tracking::TrackerConfig
    pub fn with_config(config: crate::capture::backends::global_tracking::TrackerConfig) -> Self {
        tracing::debug!(
            target: "memscope::tracker",
            max_allocations = config.max_allocations,
            enable_statistics = config.enable_statistics,
            "constructing Tracker from GlobalTrackerConfig.tracker \
             (currently informational; fields are reserved for future enforcement)",
        );
        Self::new()
    }

    pub fn global() -> Self {
        allocator_events::global_tracker().clone()
    }

    pub fn with_system_monitoring(self) -> Self {
        self.capture_system_snapshot();
        self
    }

    pub fn with_sampling(self, config: SamplingConfig) -> Self {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.sampling = config;
        }
        self
    }

    pub fn with_auto_export(self, path: &str) -> Self {
        if let Ok(mut cfg) = self.config.lock() {
            cfg.auto_export_on_drop = true;
            cfg.export_path = Some(path.to_string());
        }
        self
    }

    pub fn track_as<T: crate::Trackable>(
        &self,
        var: &T,
        name: &str,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        if let Ok(cfg) = self.config.lock() {
            if cfg.sampling.sample_rate < 1.0 {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                // Use current timestamp for randomness to ensure sampling works
                // correctly even with identical variable names in a loop
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                timestamp.hash(&mut hasher);
                std::thread::current().id().hash(&mut hasher);
                name.hash(&mut hasher);
                file.hash(&mut hasher);
                line.hash(&mut hasher);
                let hash = hasher.finish();
                let threshold = (cfg.sampling.sample_rate * 1000.0) as u64;
                if (hash % 1000) > threshold {
                    return;
                }
            }
        }

        self.track_inner(var, name, file, line, module_path);
    }

    #[allow(clippy::too_many_arguments)]
    /// Track a clone operation
    pub fn track_clone(
        &self,
        source_ptr: usize,
        target_ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        let _gate = crate::core::allocator::capture_gate::acquire(0);
        let thread_id_u64 = crate::utils::current_thread_id_u64();

        let mut event = crate::event_store::MemoryEvent::clone_event(
            source_ptr,
            target_ptr,
            size,
            thread_id_u64,
            var_name,
            type_name,
        );
        event.source_file = Some(file.to_string());
        event.source_line = Some(line);
        event.module_path = Some(module_path.to_string());

        self.event_store.record(event);
    }

    fn track_inner<T: crate::Trackable>(
        &self,
        var: &T,
        name: &str,
        file: &str,
        line: u32,
        module_path: &str,
    ) {
        let type_name = var.get_type_name().to_string();
        let kind = var.track_kind();

        let thread_id_u64 = crate::utils::current_thread_id_u64();

        match kind {
            crate::core::types::TrackKind::HeapOwner { ptr, size } => {
                // The core cache can already hold this address because the allocator hook
                // registered the block *without* emitting an event, or because a previous
                // block at this address was released and the address was reused. Only the
                // binding identity (serial) plus the presence of type metadata tell those
                // cases apart from a real duplicate observation.
                // Serialise the whole "read index -> update cache -> emit event -> write
                // index" sequence per address. Without this, N threads observing the same
                // block at once each saw "not recorded yet" and produced N allocation
                // events for one block (plan A06 follow-up).
                let slot = self.recorded_slot(ptr);
                let mut slot_guard = match slot.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                let recorded = *slot_guard;

                if let Err(e) = self.inner.track_allocation(ptr, size) {
                    tracing::error!("Failed to track allocation at ptr {:x}: {}", ptr, e);
                    return;
                }

                let serial = self.inner.binding_serial(ptr).unwrap_or(0);
                let same_binding = recorded
                    .is_some_and(|binding| binding.serial == serial && binding.size == size);

                if same_binding && recorded.is_some_and(|binding| binding.typed) {
                    // Same binding, same size, type already reported: refresh the cache row
                    // only, never a second event.
                    if let Err(e) = self.inner.associate_var(
                        ptr,
                        name.to_string(),
                        type_name,
                        Some(file),
                        Some(line),
                    ) {
                        tracing::error!(
                            "Failed to associate var '{}' at ptr {:x}: {}",
                            name,
                            ptr,
                            e
                        );
                    }
                    return;
                }

                if same_binding {
                    // The allocation is already reported (for example by an allocator
                    // reallocation event) but without type information: attach the type as an
                    // annotation so projections can name the block without counting a second
                    // allocation.
                    let mut annotation = MemoryEvent::metadata(
                        name.to_string(),
                        type_name.clone(),
                        thread_id_u64,
                        size,
                    );
                    annotation.ptr = ptr;
                    annotation.binding_serial = Some(serial);
                    annotation.source_file = Some(file.to_string());
                    annotation.source_line = Some(line);
                    annotation.module_path = Some(module_path.to_string());
                    self.event_store.record(annotation);
                    *slot_guard = Some(RecordedBinding {
                        size,
                        serial,
                        typed: true,
                    });

                    if let Err(e) = self.inner.associate_var(
                        ptr,
                        name.to_string(),
                        type_name,
                        Some(file),
                        Some(line),
                    ) {
                        tracing::error!(
                            "Failed to associate var '{}' at ptr {:x}: {}",
                            name,
                            ptr,
                            e
                        );
                    }
                    return;
                }

                // Either the first typed observation of a hook-registered block, or a new
                // binding (fresh address, or a new serial after the address was reused).
                // Emit one allocation event and book task memory exactly once.
                crate::task_registry::global_registry().record_allocation(size);

                let mut event = MemoryEvent::allocate(ptr, size, thread_id_u64);
                event.binding_serial = Some(serial);
                event.var_name = Some(name.to_string());
                event.type_name = Some(type_name.clone());
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                self.event_store.record(event);
                *slot_guard = Some(RecordedBinding {
                    size,
                    serial,
                    typed: true,
                });

                if let Err(e) = self.inner.associate_var(
                    ptr,
                    name.to_string(),
                    type_name,
                    Some(file),
                    Some(line),
                ) {
                    tracing::error!("Failed to associate var '{}' at ptr {:x}: {}", name, ptr, e);
                }
            }
            crate::core::types::TrackKind::StackOwner {
                ptr: stack_ptr,
                heap_ptr,
                size,
            } => {
                // StackOwner records stack pointer metadata for clone detection
                // Use stack_ptr as key for track_allocation to avoid overwriting Arc clones
                // This allows inner tracker to count allocations while preserving clone detection

                if let Err(e) = self.inner.track_allocation(stack_ptr, size) {
                    tracing::error!("Failed to track allocation at ptr {:x}: {}", stack_ptr, e);
                    return;
                }

                // Smart pointers own heap memory too: the task that tracked them must be
                // charged once, exactly like a HeapOwner observation (plan A08 parity).
                crate::task_registry::global_registry().record_allocation(size);

                let mut event = MemoryEvent::allocate(heap_ptr, size, thread_id_u64);
                event.var_name = Some(name.to_string());
                event.type_name = Some(type_name.clone());
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                // Store stack pointer in custom metadata for clone detection
                event.stack_ptr = Some(stack_ptr);
                self.event_store.record(event);

                if let Err(e) = self.inner.associate_var(
                    heap_ptr,
                    name.to_string(),
                    type_name,
                    Some(file),
                    Some(line),
                ) {
                    tracing::error!(
                        "Failed to associate var '{}' at ptr {:x}: {}",
                        name,
                        heap_ptr,
                        e
                    );
                }
            }
            crate::core::types::TrackKind::Container | crate::core::types::TrackKind::Value => {
                // Container and Value record metadata events without heap allocation
                // They will be tracked as graph nodes but not scanned by HeapScanner
                let mut event = MemoryEvent::metadata(
                    name.to_string(),
                    type_name,
                    thread_id_u64,
                    var.get_size_estimate(),
                );
                event.source_file = Some(file.to_string());
                event.source_line = Some(line);
                event.module_path = Some(module_path.to_string());
                self.event_store.record(event);
            }
        }
    }

    // Slots remain stable while an address is in use so a waiting observer cannot
    // acquire a detached old slot while another observer uses a newly inserted mutex.
    fn recorded_slot(&self, ptr: usize) -> RecordedSlot {
        self.recorded_bindings
            .entry(ptr)
            .or_insert_with(|| Arc::new(Mutex::new(None)))
            .clone()
    }

    /// Drop the slot of a released address once nobody else can be using it.
    ///
    /// Slots must stay stable while an address is live, but keeping one entry per
    /// address ever allocated makes the profiler itself grow without bound in a
    /// long-running process. `remove_if` holds the shard write lock, so no thread can
    /// obtain the slot while it is inspected; a strong count of one means the map is
    /// its only owner, and `try_lock` (never blocking) then confirms it is empty.
    fn reclaim_slot(&self, ptr: usize) {
        self.recorded_bindings.remove_if(&ptr, |_, slot| {
            Arc::strong_count(slot) == 1
                && slot
                    .try_lock()
                    .map(|value| value.is_none())
                    .unwrap_or(false)
        });
    }

    pub fn track_deallocation(&self, ptr: usize) -> crate::TrackingResult<bool> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        let slot = self.recorded_slot(ptr);
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        let size = self.inner.get_allocation_size(ptr).unwrap_or(0);
        let serial = self.inner.binding_serial(ptr);

        let result = self.inner.track_deallocation(ptr)?;

        // Only record event if deallocation was successful (ptr was tracked)
        if result {
            *guard = None;
            let thread_id_u64 = crate::utils::current_thread_id_u64();

            let mut event = MemoryEvent::deallocate(ptr, size, thread_id_u64);
            event.binding_serial = serial;
            self.event_store.record(event);
        }

        drop(guard);
        drop(slot);
        if result {
            self.reclaim_slot(ptr);
        }

        Ok(result)
    }

    /// Track a reallocation performed by the allocator hook.
    ///
    /// Updates the in-memory cache and records one `Reallocate` event carrying both
    /// addresses, so reconstruction can close the old binding and open the new one
    /// instead of guessing from an alloc/free pair.
    pub fn track_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        old_size: usize,
        new_size: usize,
    ) {
        let _gate = crate::core::allocator::capture_gate::acquire(old_ptr);
        let old_slot = self.recorded_slot(old_ptr);
        if old_ptr == new_ptr {
            let mut guard = old_slot.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(binding) = self.record_reallocation(old_ptr, new_ptr, old_size, new_size) {
                *guard = Some(binding);
            }
            return;
        }
        let new_slot = self.recorded_slot(new_ptr);
        // Address order prevents opposing migrations from acquiring these locks in reverse.
        let (mut old_guard, mut new_guard) = if old_ptr < new_ptr {
            let old = old_slot.lock().unwrap_or_else(|e| e.into_inner());
            let new = new_slot.lock().unwrap_or_else(|e| e.into_inner());
            (old, new)
        } else {
            let new = new_slot.lock().unwrap_or_else(|e| e.into_inner());
            let old = old_slot.lock().unwrap_or_else(|e| e.into_inner());
            (old, new)
        };
        let binding = self.record_reallocation(old_ptr, new_ptr, old_size, new_size);
        if let Some(binding) = binding {
            *old_guard = None;
            *new_guard = Some(binding);
        }
        drop(old_guard);
        drop(new_guard);
        drop(old_slot);
        drop(new_slot);
        if binding.is_some() {
            self.reclaim_slot(old_ptr);
        }
    }

    /// Called with the affected address slots locked through event publication.
    fn record_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        old_size: usize,
        new_size: usize,
    ) -> Option<RecordedBinding> {
        if let Err(e) = self
            .inner
            .track_reallocation(old_ptr, new_ptr, old_size, new_size)
        {
            tracing::error!(
                "Failed to track reallocation {:x} -> {:x}: {}",
                old_ptr,
                new_ptr,
                e
            );
            return None;
        }

        // The reallocation event carries no type information, so the binding is remembered
        // as *untyped*: the next `track!` on it must attach the type instead of being
        // treated as a duplicate observation.
        let serial = self.inner.binding_serial(new_ptr).unwrap_or(0);
        let thread_id_u64 = crate::utils::current_thread_id_u64();
        let mut event =
            MemoryEvent::reallocate_moved(old_ptr, new_ptr, old_size, new_size, thread_id_u64);
        event.binding_serial = Some(serial);
        self.event_store.record(event);
        Some(RecordedBinding {
            size: new_size,
            serial,
            typed: false,
        })
    }

    /// Number of duplicate observations of an already tracked address.
    ///
    /// These sightings are not counted as allocations or bytes; the value exists so
    /// reports can show that the capture layer saw the same block twice.
    pub fn duplicate_observations(&self) -> u64 {
        self.inner.duplicate_observations()
    }

    pub fn events(&self) -> Vec<MemoryEvent> {
        self.event_store.snapshot()
    }

    pub fn event_store(&self) -> &Arc<EventStore> {
        &self.event_store
    }

    fn capture_system_snapshot(&self) {
        let snapshot = SystemSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu_usage_percent: system_monitor::cpu_usage(),
            memory_usage_bytes: system_monitor::memory_used(),
            memory_usage_percent: system_monitor::memory_usage_percent(),
            thread_count: system_monitor::thread_count(),
            disk_read_bps: system_monitor::disk_read_bps(),
            disk_write_bps: system_monitor::disk_write_bps(),
            network_rx_bps: system_monitor::network_rx_bps(),
            network_tx_bps: system_monitor::network_tx_bps(),
            gpu_usage_percent: system_monitor::gpu_memory_usage_percent(),
            gpu_memory_used: system_monitor::gpu_memory_used(),
            gpu_memory_total: system_monitor::gpu_memory_total(),
        };

        if let Ok(mut snapshots) = self.system_snapshots.lock() {
            snapshots.push(snapshot);
        }
    }

    pub fn stats(&self) -> crate::core::types::MemoryStats {
        let stats = self.inner.get_stats().unwrap_or_default();
        crate::core::types::MemoryStats {
            total_allocations: stats.total_allocations as usize,
            total_allocated: stats.total_allocated as usize,
            active_allocations: stats.active_allocations,
            active_memory: stats.active_memory as usize,
            peak_allocations: stats.peak_allocations,
            peak_memory: stats.peak_memory as usize,
            total_deallocations: stats.total_deallocations as usize,
            total_deallocated: stats.total_deallocated as usize,
            leaked_allocations: stats.leaked_allocations,
            leaked_memory: stats.leaked_memory as usize,
            ..Default::default()
        }
    }

    pub fn analyze(&self) -> AnalysisReport {
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        let stats = self.stats();
        let events = self.event_store().snapshot();
        let allocations = rebuild_allocations_from_events(&events);
        let elapsed = self.start_time.elapsed().as_secs_f64();

        let current_memory: usize = allocations.iter().map(|a| a.size).sum();
        let peak_memory = stats.peak_memory.max(current_memory);

        let mut hotspot_map: HashMap<String, (String, usize, usize)> = HashMap::new();
        for alloc in &allocations {
            if let Some(ref var_name) = alloc.var_name {
                let key = var_name.clone();
                let entry = hotspot_map.entry(key).or_insert((
                    alloc.type_name.clone().unwrap_or_default(),
                    0,
                    0,
                ));
                entry.1 += alloc.size;
                entry.2 += 1;
            }
        }

        let hotspots: Vec<AllocationHotspot> = hotspot_map
            .into_iter()
            .map(
                |(var_name, (type_name, total_size, count))| AllocationHotspot {
                    var_name,
                    type_name,
                    total_size,
                    allocation_count: count,
                    location: None,
                },
            )
            .collect();

        let system_snapshots = self
            .system_snapshots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        AnalysisReport {
            total_allocations: stats.total_allocations,
            total_deallocations: stats.total_deallocations,
            active_allocations: allocations.len(),
            peak_memory_bytes: peak_memory as u64,
            current_memory_bytes: current_memory as u64,
            allocation_rate_per_sec: if elapsed > 0.0 {
                stats.total_allocations as f64 / elapsed
            } else {
                0.0
            },
            deallocation_rate_per_sec: if elapsed > 0.0 {
                stats.total_deallocations as f64 / elapsed
            } else {
                0.0
            },
            hotspots,
            system_snapshots,
        }
    }

    pub fn inner(&self) -> &Arc<MemoryTracker> {
        &self.inner
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn system_snapshots(&self) -> Vec<SystemSnapshot> {
        self.system_snapshots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn current_system_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            cpu_usage_percent: system_monitor::cpu_usage(),
            memory_usage_bytes: system_monitor::memory_used(),
            memory_usage_percent: system_monitor::memory_usage_percent(),
            thread_count: system_monitor::thread_count(),
            disk_read_bps: system_monitor::disk_read_bps(),
            disk_write_bps: system_monitor::disk_write_bps(),
            network_rx_bps: system_monitor::network_rx_bps(),
            network_tx_bps: system_monitor::network_tx_bps(),
            gpu_usage_percent: system_monitor::gpu_memory_usage_percent(),
            gpu_memory_used: system_monitor::gpu_memory_used(),
            gpu_memory_total: system_monitor::gpu_memory_total(),
        }
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        // Dropping a tracker (or a clone of one that shares the event store) is a
        // *profiler* event, not an object release. Emitting `Deallocate` for every
        // still-live allocation here fabricated free events: they silently
        // converted live objects into "freed" ones and destroyed the accuracy of
        // every downstream statistic (see plan A01).
        //
        // The only thing a drop is still allowed to do is the legacy compatibility
        // snapshot enabled by `with_auto_export`. That path is read-only with
        // respect to the event store.
        self.export_legacy_on_drop();
    }
}

impl Tracker {
    /// Write the legacy auto-export snapshot if `with_auto_export` was enabled.
    ///
    /// This is a compatibility entry point, not a session end: it only projects the
    /// events recorded so far (`memory_analysis.json`, `lifetime.json`,
    /// `thread_analysis.json`) and never appends release events, so a live object
    /// stays a live object in the exported files.
    fn export_legacy_on_drop(&self) {
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        let Some(path) = self.auto_export_path() else {
            return;
        };

        // Use the event store as the single data source (covers both HeapOwner and
        // Container observations).
        let events = self.event_store().snapshot();

        // The live list cannot reconstruct cumulative totals or the high-water mark, so
        // the legacy drop export applies the same ledger projection as `export_all_json`
        // instead of re-aggregating what is still alive (plan A06 follow-up).
        let mut ledger = crate::snapshot::BindingLedger::new();
        for event in &events {
            ledger.apply(event);
        }
        let allocations = rebuild_allocations_from_events(&events);
        let mut snapshot = MemorySnapshot::from_allocation_infos(allocations);
        let active = snapshot.stats.active_allocations;
        crate::render_engine::export::override_snapshot_stats(&mut snapshot, &ledger, active);
        let options = ExportJsonOptions::default();
        if let Err(e) = export_snapshot_to_json(&snapshot, std::path::Path::new(&path), &options) {
            tracing::error!("Failed to auto-export on drop: {}", e);
        }
    }

    /// Read the configured legacy auto-export path, if enabled.
    fn auto_export_path(&self) -> Option<String> {
        let cfg = self.config.lock().ok()?;
        if !cfg.auto_export_on_drop {
            return None;
        }
        cfg.export_path.clone()
    }
}

impl serde::Serialize for AnalysisReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AnalysisReport", 9)?;
        state.serialize_field("total_allocations", &self.total_allocations)?;
        state.serialize_field("total_deallocations", &self.total_deallocations)?;
        state.serialize_field("active_allocations", &self.active_allocations)?;
        state.serialize_field("peak_memory_bytes", &self.peak_memory_bytes)?;
        state.serialize_field("current_memory_bytes", &self.current_memory_bytes)?;
        state.serialize_field("allocation_rate_per_sec", &self.allocation_rate_per_sec)?;
        state.serialize_field("deallocation_rate_per_sec", &self.deallocation_rate_per_sec)?;
        state.serialize_field("hotspots", &self.hotspots)?;
        state.serialize_field("system_snapshots", &self.system_snapshots)?;
        state.end()
    }
}

impl serde::Serialize for AllocationHotspot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AllocationHotspot", 5)?;
        state.serialize_field("var_name", &self.var_name)?;
        state.serialize_field("type_name", &self.type_name)?;
        state.serialize_field("total_size", &self.total_size)?;
        state.serialize_field("allocation_count", &self.allocation_count)?;
        state.serialize_field("location", &self.location)?;
        state.end()
    }
}

#[macro_export]
macro_rules! tracker {
    () => {
        $crate::tracker::Tracker::new()
    };
}

#[macro_export]
macro_rules! track {
    ($tracker:expr, $var:expr) => {{
        let var_name = stringify!($var);
        $tracker.track_as(&$var, var_name, file!(), line!(), module_path!());
    }};
}

#[macro_export]
macro_rules! track_clone {
    ($tracker:expr, $source:expr, $target:expr) => {{
        let source_name = stringify!($source);
        let target_name = stringify!($target);
        let source_ptr = &$source as *const _ as usize;
        let target_ptr = &$target as *const _ as usize;
        let type_name = $crate::utils::type_of(&$target);
        $tracker.track_clone(
            source_ptr,
            target_ptr,
            std::mem::size_of_val(&$target),
            Some(target_name.to_string()),
            Some(type_name.to_string()),
            file!(),
            line!(),
            module_path!(),
        );
    }};
}

#[cfg(test)]
mod tests;
