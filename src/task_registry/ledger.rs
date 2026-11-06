//! Task accounting over the same binding transitions as the memory ledger.

use super::{TaskIdRegistry, TaskLedgerStats};
use crate::event_store::{MemoryEvent, MemoryEventType};
use std::collections::{BTreeMap, HashMap};

/// Lifecycle gaps are independent of whether registry allocation charges succeeded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct TaskLifecycleQuality {
    pub unmatched_reallocations: u64,
    pub unmatched_deallocations: u64,
    pub degraded_releases: u64,
    pub conflicting_allocations: u64,
    pub unattributed_operations: u64,
}

impl TaskLifecycleQuality {
    pub fn is_complete(&self) -> bool {
        *self == Self::default()
    }
}

/// Task metrics and their lifecycle evidence, replayed from the same event slice.
#[derive(Debug, Default)]
pub struct TaskAccountingSnapshot {
    pub stats: BTreeMap<u64, TaskLedgerStats>,
    pub lifecycle_quality: TaskLifecycleQuality,
    pub task_lifecycle_quality: BTreeMap<u64, TaskLifecycleQuality>,
}

struct OpenBinding {
    ptr: usize,
    serial: Option<u64>,
    size: u64,
    task: Option<u64>,
}

#[derive(Default)]
struct TaskBindings {
    slots: Vec<Option<OpenBinding>>,
    by_addr: HashMap<usize, Vec<usize>>,
}

impl TaskBindings {
    fn record_quality(&self, event: &MemoryEvent, snapshot: &mut TaskAccountingSnapshot) {
        let mut gap = TaskLifecycleQuality::default();
        let bucket = self.by_addr.get(&event.ptr);
        match event.event_type {
            MemoryEventType::Allocate => {
                gap.conflicting_allocations = u64::from(bucket.is_some_and(|b| !b.is_empty()));
            }
            MemoryEventType::Deallocate | MemoryEventType::Reallocate => {
                let size = if event.event_type == MemoryEventType::Deallocate {
                    Some(event.size)
                } else {
                    event.old_size
                };
                let unmatched = self.find(event, size).is_none();
                if event.event_type == MemoryEventType::Deallocate {
                    gap.unmatched_deallocations = u64::from(unmatched);
                } else {
                    gap.unmatched_reallocations = u64::from(unmatched);
                }
                gap.degraded_releases = u64::from(
                    event.degraded_release
                        || (event.binding_serial.is_none() && bucket.is_some_and(|b| b.len() > 1)),
                );
            }
            _ => return,
        }
        gap.unattributed_operations = u64::from(event.task_id.is_none());
        snapshot.lifecycle_quality.add(gap);

        // Every candidate owner is affected by an ambiguous close, not just the one
        // selected by replay policy. The operation's actor may be another task.
        let mut tasks = std::collections::BTreeSet::new();
        tasks.extend(event.task_id);
        if let Some(bucket) = bucket {
            for index in bucket {
                if let Some(binding) = &self.slots[*index] {
                    if event.binding_serial.is_none()
                        || gap.conflicting_allocations > 0
                        || binding.serial == event.binding_serial
                    {
                        tasks.extend(binding.task);
                    }
                }
            }
        }
        for task in tasks {
            snapshot
                .task_lifecycle_quality
                .entry(task)
                .or_default()
                .add(gap);
        }
    }

    fn push(&mut self, binding: OpenBinding) {
        self.by_addr
            .entry(binding.ptr)
            .or_default()
            .push(self.slots.len());
        self.slots.push(Some(binding));
    }

    fn find(&self, event: &MemoryEvent, size: Option<usize>) -> Option<usize> {
        let bucket = self.by_addr.get(&event.ptr)?;
        if let Some(serial) = event.binding_serial {
            // An explicit identity is authoritative. A late release of G must never
            // consume G+1 just because its address or size happens to match.
            return bucket.iter().copied().find(|index| {
                self.slots[*index]
                    .as_ref()
                    .is_some_and(|binding| binding.serial == Some(serial))
            });
        }
        bucket
            .iter()
            .copied()
            .find(|index| {
                self.slots[*index].as_ref().is_some_and(|binding| {
                    size.is_none_or(|size| size == 0 || binding.size == size as u64)
                })
            })
            .or_else(|| bucket.first().copied())
    }

    fn close(&mut self, index: usize) -> Option<OpenBinding> {
        let binding = self.slots[index].take()?;
        if let Some(bucket) = self.by_addr.get_mut(&binding.ptr) {
            bucket.retain(|candidate| *candidate != index);
            if bucket.is_empty() {
                self.by_addr.remove(&binding.ptr);
            }
        }
        Some(binding)
    }
}

impl TaskIdRegistry {
    /// Aggregate a fixed event cut by binding identity and allocating task.
    ///
    /// Live bytes stay with the origin task; allocation/release activity belongs to
    /// the task performing the operation. Unattributed bindings are still indexed so
    /// a later task can release them without consuming another generation.
    pub fn task_stats_from_events(events: &[MemoryEvent]) -> BTreeMap<u64, TaskLedgerStats> {
        Self::task_accounting_from_events(events).stats
    }

    /// Replay metrics and lifecycle gaps together, preserving cross-task ownership.
    pub fn task_accounting_from_events(events: &[MemoryEvent]) -> TaskAccountingSnapshot {
        let mut bindings = TaskBindings::default();
        let mut snapshot = TaskAccountingSnapshot::default();
        let mut stats: BTreeMap<u64, TaskLedgerStats> = BTreeMap::new();
        for event in events {
            bindings.record_quality(event, &mut snapshot);
            match event.event_type {
                MemoryEventType::Allocate => {
                    open_observed(&mut bindings, &mut stats, event, event.ptr)
                }
                MemoryEventType::Deallocate => {
                    let Some(index) = bindings.find(event, Some(event.size)) else {
                        continue;
                    };
                    let Some(closed) = bindings.close(index) else {
                        continue;
                    };
                    if let Some(actor_id) = event.task_id {
                        let actor = stats.entry(actor_id).or_default();
                        actor.deallocation_count += 1;
                        actor.deallocated_bytes += closed.size;
                    }
                    if let Some(owner) = closed.task.and_then(|id| stats.get_mut(&id)) {
                        owner.live_bytes = owner.live_bytes.saturating_sub(closed.size);
                    }
                }
                MemoryEventType::Reallocate => {
                    let new_ptr = event.new_ptr.unwrap_or(event.ptr);
                    let Some(index) = bindings.find(event, event.old_size) else {
                        // Capture may start mid-lifetime. The returned block is known to
                        // exist, but no old bytes may be booked as released without evidence.
                        open_observed(&mut bindings, &mut stats, event, new_ptr);
                        continue;
                    };
                    let migrating = new_ptr != event.ptr;
                    let binding = if migrating {
                        bindings.close(index)
                    } else {
                        bindings.slots[index].take()
                    };
                    let Some(mut binding) = binding else {
                        continue;
                    };
                    if let Some(owner) = binding.task.and_then(|id| stats.get_mut(&id)) {
                        owner.live_bytes = owner
                            .live_bytes
                            .saturating_sub(binding.size)
                            .saturating_add(event.size as u64);
                    }
                    if let Some(actor_id) = event.task_id {
                        let actor = stats.entry(actor_id).or_default();
                        actor.allocated_bytes += event.size as u64;
                        actor.deallocated_bytes += binding.size;
                    }
                    binding.ptr = new_ptr;
                    binding.size = event.size as u64;
                    if migrating {
                        bindings.push(binding);
                    } else {
                        // Keep the oldest-first legacy matching order on an in-place resize.
                        bindings.slots[index] = Some(binding);
                    }
                }
                _ => {}
            }
        }
        snapshot.stats = stats;
        snapshot
    }
}

impl TaskLifecycleQuality {
    fn add(&mut self, other: Self) {
        self.unmatched_reallocations += other.unmatched_reallocations;
        self.unmatched_deallocations += other.unmatched_deallocations;
        self.degraded_releases += other.degraded_releases;
        self.conflicting_allocations += other.conflicting_allocations;
        self.unattributed_operations += other.unattributed_operations;
    }
}

fn open_observed(
    bindings: &mut TaskBindings,
    stats: &mut BTreeMap<u64, TaskLedgerStats>,
    event: &MemoryEvent,
    ptr: usize,
) {
    if let Some(task_id) = event.task_id {
        let entry = stats.entry(task_id).or_default();
        entry.allocation_count += 1;
        entry.allocated_bytes += event.size as u64;
        entry.live_bytes = entry.live_bytes.saturating_add(event.size as u64);
    }
    bindings.push(OpenBinding {
        ptr,
        serial: event.binding_serial,
        size: event.size as u64,
        task: event.task_id,
    });
}
