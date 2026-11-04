//! Rebuild per-task memory profiles from the event stream.
//!
//! This is the authoritative source for export-time and dashboard-time task
//! profiles. It replays every `Allocate`, `Deallocate`, and `Reallocate`
//! event from the `EventStore` and reconstructs `TaskMemoryProfile` entries
//! with accurate:
//!   * `total_bytes`, `current_memory`, `peak_memory`
//!   * allocation/deallocation counts
//!   * creation timestamps (from the first event or live tracker)
//!   * variable/type names (from allocation + metadata events)
//!   * live allocation list
//!
//! This replaces reading live `AsyncTracker` state directly, which was subject
//! to lock contention, silent data loss from `try_lock`, and missing realloc
//! updates.

use crate::capture::backends::{AsyncAllocation, TaskMemoryProfile, TaskType};
use crate::event_store::{MemoryEvent, MemoryEventType};
use crate::task_registry::{TaskGraph, TaskStatus};
use std::collections::HashMap;
use std::sync::Arc;

/// A live allocation tracked during replay.
///
/// Records enough context to match a release or realloc back to the original
/// allocation, even when the release event has `task_id = None` (the release
/// happened outside any task context).
struct LiveBinding {
    task_id: u64,
    size: usize,
    generation: u64,
    timestamp: u64,
    var_name: Option<String>,
    type_name: Option<String>,
}

/// Rebuilt profile data: per-task profiles and the live allocation list.
pub struct RebuiltProfiles {
    pub profiles: Vec<TaskMemoryProfile>,
    pub live_allocations: Vec<AsyncAllocation>,
    /// Global peak memory across all tasks (sum of all tasks' current_memory
    /// at each point in time). This is NOT the max of individual task peaks.
    pub global_peak_memory: u64,
    /// Total allocations across all tasks.
    pub total_allocations: u64,
    /// Total memory allocated across all tasks.
    pub total_memory: u64,
    /// Current (live) memory across all tasks.
    pub current_memory: u64,
}

/// Metadata from the live AsyncTracker that is not derivable from the event
/// stream: task names, completion timestamps, and creation timestamps for
/// zero-allocation tasks.
#[derive(Clone)]
struct LiveMeta {
    task_name: String,
    created_at_ms: u64,
    completed_at_ms: Option<u64>,
    tokio_task_id: Option<u64>,
    thread_id_u64: Option<u64>,
    task_type: TaskType,
}

/// Freeze lifecycle metadata once, alongside the event slice and capture timestamp.
pub(crate) struct ProfileMetadata(HashMap<u64, LiveMeta>);

impl ProfileMetadata {
    pub(crate) fn capture(
        graph: &TaskGraph,
        tracker: Option<&crate::capture::backends::async_tracker::AsyncTracker>,
    ) -> Self {
        let _guard = crate::core::allocator::TrackingGuard::acquire();
        let mut metadata = HashMap::new();
        let ids: std::collections::HashSet<_> = graph.nodes.iter().map(|node| node.id).collect();
        for meta in crate::task_registry::global_registry().get_all_tasks() {
            if ids.contains(&meta.id) {
                metadata.insert(
                    meta.id,
                    LiveMeta {
                        task_name: meta.name,
                        created_at_ms: meta.created_at / 1_000_000,
                        completed_at_ms: meta.completed_at.map(|ns| ns / 1_000_000),
                        tokio_task_id: meta.tokio_id,
                        thread_id_u64: None,
                        task_type: TaskType::default(),
                    },
                );
            }
        }
        if let Some(tracker) = tracker {
            for profile in tracker.lifecycle_profiles() {
                let completed_at_ms = profile.completed_at_ms.or_else(|| {
                    metadata
                        .get(&profile.task_id)
                        .and_then(|meta| meta.completed_at_ms)
                });
                metadata.insert(
                    profile.task_id,
                    LiveMeta {
                        task_name: profile.task_name,
                        created_at_ms: profile.created_at_ms,
                        completed_at_ms,
                        tokio_task_id: profile.tokio_task_id,
                        thread_id_u64: profile.thread_id_u64,
                        task_type: profile.task_type,
                    },
                );
            }
        }
        Self(metadata)
    }

    /// Completion can race the earlier graph read. Both report sections must
    /// describe the same frozen lifecycle metadata, even when capture overlaps it.
    pub(crate) fn reconcile_graph(&self, graph: &mut TaskGraph) {
        for node in &mut graph.nodes {
            if self
                .0
                .get(&node.id)
                .is_some_and(|meta| meta.completed_at_ms.is_some())
            {
                node.status = TaskStatus::Completed;
            }
        }
    }
}

pub(crate) fn capture_cut_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

/// Rebuild per-task memory profiles from the event stream.
///
/// This is the low-level replay function. It does NOT merge metadata from a
/// live `AsyncTracker`. Callers that need display metadata (task names,
/// completion times) should use [`rebuild_with_live_tracker`] instead.
///
/// Internal allocations are suppressed. Each call captures its own cutoff time.
pub fn rebuild_async_profiles_from_events(
    events: &[MemoryEvent],
    task_graph: &TaskGraph,
) -> RebuiltProfiles {
    // Suppress allocator tracking during the replay: the HashMap and Vec
    // operations inside rebuild_inner perform many allocations that are
    // not user allocations. Without suppression, these self-allocations
    // would be recorded by the tracking allocator and pollute the event
    // stream that subsequent replays read from.
    let _guard = crate::core::allocator::TrackingGuard::acquire();
    let metadata = ProfileMetadata::capture(task_graph, None);
    rebuild_at(events, task_graph, &metadata, capture_cut_ns())
}

/// Rebuild per-task memory profiles from the event stream, merging display
/// metadata (task names, completion times, creation timestamps) from the live
/// `AsyncTracker`.
///
/// This is the unified function that all query paths (JSON export, HTML
/// rendering, live API) should use. It:
///   1. Replays events to get accurate accounting (allocations, releases, realloc).
///   2. Merges task names and completion times from the live tracker.
///   3. Recalculates derived metrics (duration, allocation_rate, efficiency).
///
/// Internal allocations are suppressed. Lifecycle metadata is copied once.
pub fn rebuild_with_live_tracker(
    events: &[MemoryEvent],
    task_graph: &TaskGraph,
    live_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> RebuiltProfiles {
    // Suppress allocator tracking during the entire replay + metadata merge.
    // The HashMap/Vec operations and live_tracker.get_all_profiles() clones
    // perform many allocations that are not user allocations.
    let _guard = crate::core::allocator::TrackingGuard::acquire();

    let metadata = ProfileMetadata::capture(task_graph, Some(live_tracker));
    rebuild_at(events, task_graph, &metadata, capture_cut_ns())
}

pub(crate) fn rebuild_at(
    events: &[MemoryEvent],
    task_graph: &TaskGraph,
    metadata: &ProfileMetadata,
    captured_at_ns: u64,
) -> RebuiltProfiles {
    let _guard = crate::core::allocator::TrackingGuard::acquire();
    let live_meta = Some(&metadata.0);
    let mut profiles: HashMap<u64, TaskMemoryProfile> = HashMap::new();
    let mut live: HashMap<usize, Vec<LiveBinding>> = HashMap::new();

    // Build initial profiles from the task graph so even tasks with zero
    // allocations appear in the export, with correct names.
    for node in &task_graph.nodes {
        let mut profile = TaskMemoryProfile::new(node.id, node.name.clone(), TaskType::default());
        profile.created_at_ms = 0;
        // Set creation time from the live tracker if available (preferred over
        // global registry because the tracker owns the task lifecycle).
        // Fall back to the global registry for tasks not in the live tracker.
        let mut node_completed = node.status == TaskStatus::Completed;
        if let Some(meta) = live_meta.and_then(|m| m.get(&node.id)) {
            if meta.created_at_ms > 0 {
                profile.created_at_ms = meta.created_at_ms;
            }
            if !meta.task_name.is_empty() {
                profile.task_name = meta.task_name.clone();
            }
            profile.completed_at_ms = meta.completed_at_ms;
            profile.tokio_task_id = meta.tokio_task_id;
            profile.thread_id_u64 = meta.thread_id_u64;
            profile.task_type = meta.task_type;
            // If the live tracker says completed, trust it.
            if meta.completed_at_ms.is_some() {
                node_completed = true;
            }
        }
        // Legacy graph-only nodes have a status but no lifecycle timestamps.
        // Preserve the terminal status without inferring an end from memory events.
        if node_completed && profile.completed_at_ms.is_none() {
            profile.completed_at_ms = Some(profile.created_at_ms);
        }
        profiles.insert(node.id, profile);
    }

    // Standalone trackers can contain zero-allocation tasks absent from the graph.
    for (&id, meta) in &metadata.0 {
        profiles.entry(id).or_insert_with(|| {
            let mut profile = TaskMemoryProfile::new(id, meta.task_name.clone(), meta.task_type);
            profile.created_at_ms = meta.created_at_ms;
            profile.completed_at_ms = meta.completed_at_ms;
            profile.tokio_task_id = meta.tokio_task_id;
            profile.thread_id_u64 = meta.thread_id_u64;
            profile
        });
    }

    // Track the global current memory at each step to compute the true peak.
    let mut global_current: u64 = 0;
    let mut global_peak: u64 = 0;

    for event in events {
        match event.event_type {
            MemoryEventType::Allocate => {
                let task_id = match event.task_id {
                    Some(tid) if tid != 0 => tid,
                    _ => continue,
                };

                let profile = profiles.entry(task_id).or_insert_with(|| {
                    // For tasks not in the task graph: seed from live
                    // tracker metadata if available so we don't lose
                    // the name, completion status, or creation time.
                    let mut p = TaskMemoryProfile::new(task_id, String::new(), TaskType::default());
                    if let Some(meta) = live_meta.and_then(|m| m.get(&task_id)) {
                        if meta.created_at_ms > 0 {
                            p.created_at_ms = meta.created_at_ms;
                        }
                        if !meta.task_name.is_empty() {
                            p.task_name = meta.task_name.clone();
                        }
                        p.completed_at_ms = meta.completed_at_ms;
                    }
                    p
                });

                // Set creation time from the first event for this task.
                if profile.created_at_ms == 0 || event.timestamp / 1_000_000 < profile.created_at_ms
                {
                    profile.created_at_ms = event.timestamp / 1_000_000;
                }

                // Carry variable and type name from the event.
                if profile.task_name.is_empty() {
                    if let Some(ref name) = event.var_name {
                        profile.task_name = name.clone();
                    }
                }
                profile.set_thread_id_u64(Some(event.thread_id));

                profile.record_allocation(event.size as u64);
                global_current = global_current.saturating_add(event.size as u64);
                if global_current > global_peak {
                    global_peak = global_current;
                }

                let gen = event.binding_serial.unwrap_or(0);
                live.entry(event.ptr).or_default().push(LiveBinding {
                    task_id,
                    size: event.size,
                    generation: gen,
                    timestamp: event.timestamp,
                    var_name: event.var_name.clone(),
                    type_name: event.type_name.clone(),
                });
            }
            MemoryEventType::Deallocate => {
                let closed =
                    close_live_binding(&mut live, event.ptr, event.size, event.binding_serial);

                if let Some(binding) = closed {
                    if let Some(profile) = profiles.get_mut(&binding.task_id) {
                        profile.record_deallocation(binding.size as u64);
                    }
                    global_current = global_current.saturating_sub(binding.size as u64);
                }
                // Unmatched releases (no live binding at this address) are
                // silently skipped. They are typically duplicate releases,
                // releases of untracked blocks, or releases that arrived
                // after the binding was already closed by a realloc.
                // Recording a deallocation here would corrupt the profile:
                // it would subtract from current_memory even though no
                // matching allocation was open, making the summary
                // inconsistent with the live allocation list.
            }
            MemoryEventType::Reallocate => {
                let old_size = event.old_size.unwrap_or(0);
                let new_size = event.size;
                let new_ptr = event.new_ptr.unwrap_or(event.ptr);

                let closed =
                    close_live_binding(&mut live, event.ptr, old_size, event.binding_serial);

                if let Some(binding) = closed {
                    if let Some(profile) = profiles.get_mut(&binding.task_id) {
                        profile.record_reallocation(binding.size as u64, new_size as u64);
                    }
                    global_current = global_current
                        .saturating_sub(binding.size as u64)
                        .saturating_add(new_size as u64);
                    if global_current > global_peak {
                        global_peak = global_current;
                    }

                    let serial = event.binding_serial.unwrap_or(binding.generation);
                    live.entry(new_ptr).or_default().push(LiveBinding {
                        task_id: binding.task_id,
                        size: new_size,
                        generation: serial,
                        timestamp: event.timestamp,
                        var_name: binding.var_name,
                        type_name: binding.type_name,
                    });
                }
            }
            MemoryEventType::Metadata => {
                // Metadata events annotate an existing binding with variable
                // name and type name. The allocator first produces an
                // anonymous Allocate event, then track! appends a Metadata
                // event with the same ptr and binding_serial.
                if let Some(serial) = event.binding_serial {
                    if let Some(entries) = live.get_mut(&event.ptr) {
                        if let Some(binding) = entries.iter_mut().find(|b| b.generation == serial) {
                            if let Some(ref name) = event.var_name {
                                binding.var_name = Some(name.clone());
                            }
                            if let Some(ref tname) = event.type_name {
                                binding.type_name = Some(tname.clone());
                            }
                        }
                    }
                }

                // Also update the task profile's name from the metadata event.
                if let Some(task_id) = event.task_id {
                    if task_id != 0 {
                        if let Some(profile) = profiles.get_mut(&task_id) {
                            if profile.task_name.is_empty() {
                                if let Some(ref name) = event.var_name {
                                    profile.task_name = name.clone();
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Build the live allocations list from remaining entries.
    let mut live_allocations: Vec<AsyncAllocation> = Vec::new();
    for (ptr, entries) in &live {
        for binding in entries {
            live_allocations.push(AsyncAllocation {
                ptr: *ptr,
                size: binding.size,
                timestamp: binding.timestamp,
                task_id: binding.task_id,
                var_name: binding.var_name.clone(),
                type_name: binding.type_name.clone(),
                source_location: None,
                generation: binding.generation,
            });
        }
    }

    // Compute summary stats from the rebuilt profiles.
    let total_allocations: u64 = profiles.values().map(|p| p.total_allocations).sum();
    let total_memory: u64 = profiles.values().map(|p| p.total_bytes).sum();
    let current_memory: u64 = profiles.values().map(|p| p.current_memory).sum();

    let mut profiles_vec: Vec<TaskMemoryProfile> = profiles.into_values().collect();

    // Recalculate derived metrics for each profile. These must be derived
    // from created_at_ms and completed_at_ms (both event/tracker-derived),
    // not from now_ms(), so they are stable across repeated exports.
    finalize_metrics_with_cutoff(&mut profiles_vec, captured_at_ns);

    profiles_vec.sort_by_key(|p| p.task_id);
    live_allocations.sort_by(|a, b| a.ptr.cmp(&b.ptr));

    RebuiltProfiles {
        profiles: profiles_vec,
        live_allocations,
        global_peak_memory: global_peak,
        total_allocations,
        total_memory,
        current_memory,
    }
}

/// Recalculate `duration_ns`, `allocation_rate`, `efficiency_score`, and
/// `average_allocation_size` for each profile based on its final
/// `created_at_ms` and `completed_at_ms`.
///
/// This is called after all metadata merging is complete so the derived
/// metrics reflect the final state. For active tasks, duration is zero.
pub fn finalize_metrics(profiles: &mut [TaskMemoryProfile]) {
    for p in profiles {
        if let Some(completed) = p.completed_at_ms {
            let duration_ms = completed.saturating_sub(p.created_at_ms);
            p.duration_ns = duration_ms * 1_000_000;
            let duration_secs = duration_ms as f64 / 1000.0;
            p.allocation_rate = if duration_secs > 0.0 {
                p.total_bytes as f64 / duration_secs
            } else {
                0.0
            };
        } else {
            // Task is still active: leave duration/rate as 0.
            p.duration_ns = 0;
            p.allocation_rate = 0.0;
        }
        p.efficiency_score = if p.total_bytes == 0 {
            1.0
        } else {
            let deallocated = p.total_bytes.saturating_sub(p.current_memory);
            deallocated as f64 / p.total_bytes as f64
        };
        p.average_allocation_size = if p.total_allocations > 0 {
            p.total_bytes as f64 / p.total_allocations as f64
        } else {
            0.0
        };
    }
}

/// Recalculate metrics against one immutable capture cutoff.
fn finalize_metrics_with_cutoff(profiles: &mut [TaskMemoryProfile], captured_at_ns: u64) {
    let now_ms = captured_at_ns / 1_000_000;
    for p in profiles {
        let end_ms = if let Some(completed) = p.completed_at_ms {
            completed
        } else {
            now_ms
        };

        let duration_ms = if p.created_at_ms == 0 {
            0
        } else {
            end_ms.saturating_sub(p.created_at_ms)
        };
        p.duration_ns = duration_ms * 1_000_000;
        let duration_secs = duration_ms as f64 / 1000.0;
        p.allocation_rate = if duration_secs > 0.0 {
            p.total_bytes as f64 / duration_secs
        } else {
            0.0
        };
        p.efficiency_score = if p.total_bytes == 0 {
            1.0
        } else {
            let deallocated = p.total_bytes.saturating_sub(p.current_memory);
            deallocated as f64 / p.total_bytes as f64
        };
        p.average_allocation_size = if p.total_allocations > 0 {
            p.total_bytes as f64 / p.total_allocations as f64
        } else {
            0.0
        };
    }
}

/// Find and remove the matching live binding at a given address.
///
/// Matching priority:
/// 1. `binding_serial` (if provided in the event) — exact identity match.
/// 2. `size` match — the binding whose size equals the event's size.
///
/// Returns `None` if no binding matches. This is stricter than "oldest
/// fallback" because a wrong match would silently close a live binding
/// that belongs to a different allocation, corrupting the profile.
fn close_live_binding(
    live: &mut HashMap<usize, Vec<LiveBinding>>,
    ptr: usize,
    size: usize,
    binding_serial: Option<u64>,
) -> Option<LiveBinding> {
    let entries = live.get_mut(&ptr)?;

    let index = match binding_serial {
        Some(serial) => entries.iter().position(|b| b.generation == serial),
        None => entries.iter().position(|b| b.size == size),
    };

    let index = index?;

    let closed = entries.remove(index);
    if entries.is_empty() {
        live.remove(&ptr);
    }
    Some(closed)
}

#[cfg(test)]
mod tests;
