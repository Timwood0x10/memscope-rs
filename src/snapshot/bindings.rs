//! Address-binding ledger shared by every reconstruction projection.
//!
//! The raw event stream has no allocation id: a `Deallocate`/`Reallocate` event
//! carries only an address (plus the size it was told about). Reconstructing
//! "who is still alive" therefore has to be done once, with explicit rules, or
//! different outputs (snapshot, dashboard, JSON) will disagree.
//!
//! This module is that single implementation. It owns:
//!
//! * live **bindings** — one entry per observed allocation at an address, in
//!   insertion order (oldest first), with a 1-based `generation`;
//! * release/reallocation matching — a release closes *one* binding, never every
//!   binding that shares the address;
//! * transition-based statistics (allocated/deallocated/current/peak), so the
//!   numbers never depend on what a later map happens to contain.
//!
//! # Accuracy rules
//!
//! - First use of an address has `generation == 1`; only `generation > 1` means
//!   the address was reused.
//! - A late release of generation *G* must not delete generation *G+1*. The
//!   ledger matches the oldest binding whose recorded size equals the released
//!   size and falls back to the oldest binding at the address — never to "all
//!   bindings at the address".
//! - An in-place reallocation keeps the binding (same address, same generation)
//!   and only refreshes its size. A *migrating* reallocation (`new_ptr` set) closes
//!   the old address binding and opens one at the new address, keeping the
//!   allocation's metadata and origin thread.
//! - Unmatched events only bump diagnostic counters; they never mutate another
//!   binding's accounting.

use crate::event_store::{MemoryEvent, MemoryEventType};
use crate::snapshot::types::ThreadMemoryStats;
use std::collections::HashMap;

/// Add two byte counters without wrapping.
///
/// Returns the saturated sum and whether the counter overflowed, so callers can
/// mark the affected metric as non-exact instead of silently wrapping.
fn checked_bytes(previous: u64, delta: u64) -> (u64, bool) {
    match previous.checked_add(delta) {
        Some(sum) => (sum, false),
        None => (u64::MAX, true),
    }
}

/// One live address binding produced by the ledger.
#[derive(Debug, Clone)]
pub struct BindingRecord {
    /// Base address returned by the allocator.
    pub ptr: usize,
    /// Requested size in bytes.
    pub size: usize,
    /// 1-based binding order at this address (1 = first use, >1 = reuse).
    pub generation: u64,
    /// Timestamp of the allocation observation.
    pub allocated_at: u64,
    /// Thread that performed the allocation.
    pub thread_id: u64,
    /// Task that performed the allocation, when known.
    pub task_id: Option<u64>,
    /// Optional variable name from a typed observation.
    pub var_name: Option<String>,
    /// Optional type name from a typed observation.
    pub type_name: Option<String>,
    /// Optional call stack hash.
    pub call_stack_hash: Option<u64>,
    /// Optional module path.
    pub module_path: Option<String>,
    /// Optional stack pointer (smart pointer handles).
    pub stack_ptr: Option<usize>,
    /// Optional source file of the observation.
    pub source_file: Option<String>,
    /// Optional source line of the observation.
    pub source_line: Option<u32>,
}

/// Transition-based ledger statistics.
///
/// Totals and peaks cannot be recovered from a live set: they must be
/// accumulated while the events are replayed.
#[derive(Debug, Clone, Default)]
pub struct LedgerStats {
    /// Successful bindings created (initial allocations and unknown-block reallocations).
    pub allocation_count: u64,
    /// Releases matched to a binding.
    pub deallocation_count: u64,
    /// Reallocations applied to an existing binding.
    pub reallocation_count: u64,
    /// Reallocations for a block the ledger never saw.
    pub unmatched_reallocation_count: u64,
    /// Releases without a matching binding.
    pub unmatched_deallocation_count: u64,
    /// Allocations that arrived while another binding was still live at the same
    /// address. A non-zero value means the capture missed a release (or two different
    /// blocks shared one address): the bindings are kept apart, but the window has a
    /// gap and must not be reported as complete.
    pub conflicting_allocations: u64,
    /// Bytes of every successful binding.
    pub allocated_bytes: u64,
    /// Bytes closed by a matched release or by the old side of a reallocation.
    pub deallocated_bytes: u64,
    /// Allocated minus deallocated, tracked as a transition counter.
    pub current_bytes: u64,
    /// Maximum `current_bytes` observed after a complete transition.
    pub peak_bytes: u64,
    /// Set when a counter overflowed; the affected value must not be reported as exact.
    pub overflowed: bool,
}

impl LedgerStats {
    /// Whether any counter saturated during the replay.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        !self.overflowed
    }
}

/// Replays memory events into live bindings plus conserved statistics.
#[derive(Debug, Default)]
pub struct BindingLedger {
    /// Live bindings in observation order. Closed entries become `None` so releasing a
    /// block is O(1) instead of shifting the whole vector.
    live: Vec<Option<BindingRecord>>,
    /// Address -> indices of its live bindings, oldest first. Every allocation,
    /// release and reallocation touches a single address bucket, which keeps the
    /// replay linear in the number of events instead of quadratic.
    by_addr: HashMap<usize, Vec<usize>>,
    next_generation: HashMap<usize, u64>,
    stats: LedgerStats,
    thread_stats: HashMap<u64, ThreadMemoryStats>,
}

impl BindingLedger {
    /// Create an empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a single event to the ledger.
    pub fn apply(&mut self, event: &MemoryEvent) {
        match event.event_type {
            MemoryEventType::Allocate => self.on_allocate(event),
            MemoryEventType::Reallocate => self.on_reallocate(event),
            MemoryEventType::Deallocate => self.on_deallocate(event),
            // Metadata describes non-heap objects; clone/borrow/move/return do
            // not change the allocation ledger.
            // A metadata event carrying a heap address annotates an existing binding
            // (type observed after the allocation was recorded); everything else is
            // container metadata that does not touch the allocation ledger.
            MemoryEventType::Metadata => self.on_metadata(event),
            MemoryEventType::Clone
            | MemoryEventType::Move
            | MemoryEventType::Borrow
            | MemoryEventType::Return => {}
        }
    }

    /// Live bindings, oldest first.
    #[must_use]
    pub fn live(&self) -> Vec<&BindingRecord> {
        self.live.iter().filter_map(Option::as_ref).collect()
    }

    /// Ledger statistics accumulated from the replayed transitions.
    #[must_use]
    pub fn stats(&self) -> &LedgerStats {
        &self.stats
    }

    /// Per-thread attribution: `current_memory` follows the allocating thread
    /// (origin ownership), `total_deallocated` follows the freeing thread
    /// (activity), so a cross-thread release clears the origin thread's live
    /// bytes instead of inventing negative memory on the releasing thread.
    #[must_use]
    pub fn thread_stats(&self) -> &HashMap<u64, ThreadMemoryStats> {
        &self.thread_stats
    }

    fn on_allocate(&mut self, event: &MemoryEvent) {
        // A new allocation at an address that still has a live binding is a conflict:
        // keep both bindings (never overwrite) and mark the window as unreliable, so a
        // later release cannot silently delete the wrong generation.
        if self.has_live_binding(event.ptr) {
            self.stats.conflicting_allocations += 1;
            tracing::debug!(
                "conflicting allocation: address {:#x} already has a live binding",
                event.ptr
            );
        }
        self.open_binding_at(event, event.ptr);
        self.count_new_bytes(event.size as u64);
        self.attribute_allocation(event);
    }

    fn on_reallocate(&mut self, event: &MemoryEvent) {
        let Some(index) = self.find_binding(event.ptr, event.old_size) else {
            // The old block was never observed (capture started mid-flight or events
            // were lost). Only the new side is a fact: record it at its real address
            // (`new_ptr` when the allocator moved it), attribute it to the thread that
            // performed the reallocation, and book no released bytes for a block we
            // never saw allocated.
            self.stats.reallocation_count += 1;
            self.stats.unmatched_reallocation_count += 1;
            let target = event.new_ptr.unwrap_or(event.ptr);
            self.open_binding_at(event, target);
            self.count_new_bytes(event.size as u64);
            self.attribute_allocation(event);
            return;
        };

        let previous_size = self
            .live
            .get(index)
            .and_then(Option::as_ref)
            .map(|binding| binding.size)
            .unwrap_or(0);
        self.stats.reallocation_count += 1;

        let (allocated, overflow_a) = checked_bytes(self.stats.allocated_bytes, event.size as u64);
        let (deallocated, overflow_d) =
            checked_bytes(self.stats.deallocated_bytes, previous_size as u64);
        self.stats.allocated_bytes = allocated;
        self.stats.deallocated_bytes = deallocated;
        self.stats.overflowed |= overflow_a || overflow_d;

        // A reallocation is one atomic transition: replace, never add-then-subtract,
        // so the peak cannot be inflated by the temporary sum.
        let reduced = self
            .stats
            .current_bytes
            .saturating_sub(previous_size as u64);
        let (grown, overflow_c) = checked_bytes(reduced, event.size as u64);
        self.stats.current_bytes = grown;
        self.stats.overflowed |= overflow_c;
        self.stats.peak_bytes = self.stats.peak_bytes.max(self.stats.current_bytes);

        let migrating = event.new_ptr.filter(|new_ptr| *new_ptr != event.ptr);

        // Take the binding out of the slot. `close_binding` also detaches it from its
        // address bucket (migration), while the in-place case keeps the bucket entry and
        // restores the record into the very same slot, so the "oldest binding first"
        // matching order is preserved.
        let mut binding = match if migrating.is_some() {
            self.close_binding(index)
        } else {
            self.live[index].take()
        } {
            Some(binding) => binding,
            None => return,
        };

        binding.size = event.size;
        if event.var_name.is_some() {
            binding.var_name = event.var_name.clone();
        }
        if event.type_name.is_some() {
            binding.type_name = event.type_name.clone();
        }
        if event.module_path.is_some() {
            binding.module_path = event.module_path.clone();
        }
        if let (Some(file), Some(line)) = (event.source_file.clone(), event.source_line) {
            binding.source_file = Some(file);
            binding.source_line = Some(line);
        }
        let origin_thread = binding.thread_id;

        // Live bytes stay with the allocating thread; the *activity* (allocated and
        // released bytes) belongs to the thread that performed the reallocation, exactly
        // like an ordinary release.
        if let Some(thread) = self.thread_stats.get_mut(&origin_thread) {
            thread.current_memory = thread.current_memory.saturating_sub(previous_size);
            thread.current_memory += event.size;
            thread.peak_memory = thread.peak_memory.max(thread.current_memory);
        }
        let actor = self.thread_stats.entry(event.thread_id).or_default();
        actor.thread_id = event.thread_id;
        actor.total_allocated += event.size;
        actor.total_deallocated += previous_size;

        match migrating {
            Some(new_ptr) => {
                binding.allocated_at = event.timestamp;
                self.push_record(binding, new_ptr);
            }
            None => {
                self.live[index] = Some(binding);
            }
        }
    }

    /// Attach type/variable metadata to an existing heap binding.
    ///
    /// Used when the allocation was recorded without type information (for example by an
    /// allocator reallocation event) and the type is observed later. No bytes or counters
    /// change: the binding identity stays the same.
    fn on_metadata(&mut self, event: &MemoryEvent) {
        if event.ptr == 0 {
            return;
        }
        let Some(index) = self.find_binding(event.ptr, None) else {
            return;
        };
        let Some(binding) = self.live.get_mut(index).and_then(Option::as_mut) else {
            return;
        };
        if let Some(var_name) = event.var_name.as_ref() {
            binding.var_name = Some(var_name.clone());
        }
        if let Some(type_name) = event.type_name.as_ref() {
            binding.type_name = Some(type_name.clone());
        }
        if let Some(module_path) = event.module_path.as_ref() {
            binding.module_path = Some(module_path.clone());
        }
        if let (Some(file), Some(line)) = (event.source_file.clone(), event.source_line) {
            binding.source_file = Some(file);
            binding.source_line = Some(line);
        }
    }

    fn on_deallocate(&mut self, event: &MemoryEvent) {
        let Some(index) = self.find_binding(event.ptr, Some(event.size)) else {
            self.stats.unmatched_deallocation_count += 1;
            tracing::debug!(
                "unmatched release: ptr={:#x}, size={}, thread_id={}",
                event.ptr,
                event.size,
                event.thread_id
            );
            return;
        };

        let Some(binding) = self.close_binding(index) else {
            self.stats.unmatched_deallocation_count += 1;
            return;
        };
        self.stats.deallocation_count += 1;

        let (deallocated, overflowed) =
            checked_bytes(self.stats.deallocated_bytes, binding.size as u64);
        self.stats.deallocated_bytes = deallocated;
        self.stats.overflowed |= overflowed;
        self.stats.current_bytes = self.stats.current_bytes.saturating_sub(binding.size as u64);

        if let Some(thread) = self.thread_stats.get_mut(&binding.thread_id) {
            thread.current_memory = thread.current_memory.saturating_sub(binding.size);
        }

        // The releasing thread performed the activity; the allocating thread owns
        // the live bytes that just disappeared. Both facts are kept separately.
        let actor = self.thread_stats.entry(event.thread_id).or_default();
        actor.thread_id = event.thread_id;
        actor.total_deallocated += binding.size;
    }

    fn has_live_binding(&self, ptr: usize) -> bool {
        self.by_addr
            .get(&ptr)
            .is_some_and(|bucket| !bucket.is_empty())
    }

    fn open_binding_at(&mut self, event: &MemoryEvent, ptr: usize) {
        self.push_record(
            BindingRecord {
                ptr,
                size: event.size,
                generation: 0,
                allocated_at: event.timestamp,
                thread_id: event.thread_id,
                task_id: event.task_id,
                var_name: event.var_name.clone(),
                type_name: event.type_name.clone(),
                call_stack_hash: event.call_stack_hash,
                module_path: event.module_path.clone(),
                stack_ptr: event.stack_ptr,
                source_file: event.source_file.clone(),
                source_line: event.source_line,
            },
            ptr,
        );
    }

    /// Append a record at `ptr` and give it the next generation for that address.
    fn push_record(&mut self, mut record: BindingRecord, ptr: usize) -> usize {
        record.ptr = ptr;
        let counter = self.next_generation.entry(ptr).or_insert(0);
        *counter = counter.saturating_add(1);
        record.generation = *counter;

        let index = self.push_slot();
        self.live[index] = Some(record);
        self.by_addr.entry(ptr).or_default().push(index);
        index
    }

    fn push_slot(&mut self) -> usize {
        let index = self.live.len();
        self.live.push(None);
        index
    }

    /// Remove a live binding, returning it so callers can decide what to book.
    fn close_binding(&mut self, index: usize) -> Option<BindingRecord> {
        let record = self.live.get_mut(index)?.take()?;
        if let Some(bucket) = self.by_addr.get_mut(&record.ptr) {
            bucket.retain(|candidate| *candidate != index);
            if bucket.is_empty() {
                self.by_addr.remove(&record.ptr);
            }
        }
        Some(record)
    }

    fn attribute_allocation(&mut self, event: &MemoryEvent) {
        let thread = self.thread_stats.entry(event.thread_id).or_default();
        thread.thread_id = event.thread_id;
        thread.allocation_count += 1;
        thread.total_allocated += event.size;
        thread.current_memory += event.size;
        thread.peak_memory = thread.peak_memory.max(thread.current_memory);
    }

    fn count_new_bytes(&mut self, size: u64) {
        self.stats.allocation_count += 1;
        let (allocated, overflow_a) = checked_bytes(self.stats.allocated_bytes, size);
        let (current, overflow_c) = checked_bytes(self.stats.current_bytes, size);
        self.stats.allocated_bytes = allocated;
        self.stats.current_bytes = current;
        self.stats.overflowed |= overflow_a || overflow_c;
        self.stats.peak_bytes = self.stats.peak_bytes.max(self.stats.current_bytes);
    }

    /// Find the binding a release/reallocation belongs to.
    ///
    /// Only the bindings of that address are inspected (one bucket lookup). Prefers
    /// the oldest binding whose recorded size matches the size the event reports; if
    /// no size matches, falls back to the oldest binding at the address — the
    /// *oldest*, because a late event for generation *G* must not claim generation
    /// *G+1*.
    fn find_binding(&self, ptr: usize, size: Option<usize>) -> Option<usize> {
        let bucket = self.by_addr.get(&ptr)?;
        let matching = bucket.iter().copied().find(|index| {
            self.live[*index]
                .as_ref()
                .is_some_and(|binding| match size {
                    Some(size) => size == 0 || binding.size == size,
                    None => true,
                })
        });
        matching.or_else(|| bucket.first().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify a plain allocate/release pair returns the ledger to zero
    /// with conserved byte totals.
    /// Invariants: allocation and deallocation counts are both 1, current bytes 0,
    /// allocated and deallocated bytes are equal, peak keeps the observed high.
    #[test]
    fn allocate_release_conserves_bytes() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x1000, 256, 1));
        ledger.apply(&MemoryEvent::deallocate(0x1000, 256, 1));

        let stats = ledger.stats();
        assert_eq!(stats.allocation_count, 1, "one binding was created");
        assert_eq!(stats.deallocation_count, 1, "one release was matched");
        assert_eq!(
            stats.allocated_bytes, stats.deallocated_bytes,
            "a released block must conserve allocated/deallocated bytes"
        );
        assert_eq!(stats.current_bytes, 0, "no bytes may remain live");
        assert_eq!(
            stats.peak_bytes, 256,
            "the peak must keep the observed high"
        );
        assert!(ledger.live().is_empty(), "no binding may stay live");
    }

    /// Objective: Verify a late release of generation 1 does not delete the live
    /// generation 2 binding at the same address.
    /// Invariants: only the released size is removed; the reused binding keeps its
    /// generation, size and live bytes.
    #[test]
    fn late_release_does_not_close_reused_address() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x2000, 64, 1)); // G1
        ledger.apply(&MemoryEvent::allocate(0x2000, 128, 1)); // G2, G1 free missing
        ledger.apply(&MemoryEvent::deallocate(0x2000, 64, 2)); // late free of G1

        assert_eq!(
            ledger.live().len(),
            1,
            "one release must close exactly one binding"
        );
        let live = &ledger.live()[0];
        assert_eq!(live.size, 128, "the surviving binding must be G2");
        assert_eq!(live.generation, 2, "G2 keeps its binding generation");
        assert_eq!(
            ledger.stats().current_bytes,
            128,
            "current bytes must follow the surviving binding"
        );
        assert_eq!(
            ledger.stats().unmatched_deallocation_count,
            0,
            "the release matched G1, so it is not unmatched"
        );
    }

    /// Objective: Verify the first binding is generation 1 so reuse can be
    /// distinguished from first use.
    /// Invariants: rebuilt addresses start at 1 and increment per new binding.
    #[test]
    fn generation_starts_at_one() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x3000, 16, 1));
        assert_eq!(
            ledger.live()[0].generation,
            1,
            "the first binding at an address must be generation 1"
        );
        ledger.apply(&MemoryEvent::deallocate(0x3000, 16, 1));
        ledger.apply(&MemoryEvent::allocate(0x3000, 16, 1));
        assert_eq!(
            ledger.live()[0].generation,
            2,
            "reusing the address must create generation 2"
        );
    }

    /// Objective: Verify an in-place reallocation refreshes size without adding a
    /// binding or inflating the peak.
    /// Invariants: one binding stays live, size and current bytes follow the new
    /// size, and no allocation count increase happens.
    #[test]
    fn in_place_reallocation_updates_size() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x4000, 100, 1));
        ledger.apply(&MemoryEvent::reallocate(0x4000, 100, 400, 1));

        assert_eq!(ledger.live().len(), 1, "reallocation keeps one binding");
        assert_eq!(ledger.live()[0].size, 400, "size must be refreshed");
        assert_eq!(
            ledger.stats().allocation_count,
            1,
            "reallocation must not count as a new allocation"
        );
        assert_eq!(
            ledger.stats().reallocation_count,
            1,
            "reallocation must be counted separately"
        );
        assert_eq!(ledger.stats().current_bytes, 400, "current bytes must grow");
        assert_eq!(
            ledger.stats().peak_bytes,
            400,
            "the reallocation must not fabricate an intermediate 500-byte peak"
        );
    }

    /// Objective: Verify a shrinking reallocation keeps the recorded peak.
    /// Invariants: current bytes shrink while peak retains the earlier high.
    #[test]
    fn reallocation_shrink_keeps_peak() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x5000, 512, 1));
        ledger.apply(&MemoryEvent::reallocate(0x5000, 512, 64, 1));

        assert_eq!(
            ledger.stats().current_bytes,
            64,
            "current bytes must shrink"
        );
        assert_eq!(
            ledger.stats().peak_bytes,
            512,
            "peak must keep the pre-shrink high-water mark"
        );
    }

    /// Objective: Verify a cross-thread release clears the origin thread's live
    /// bytes and credits the releasing thread's activity.
    /// Invariants: origin current memory drops to zero, the releasing thread owns
    /// no live bytes but records the freed bytes.
    #[test]
    fn cross_thread_release_attributes_origin_and_actor() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x6000, 256, 1));
        ledger.apply(&MemoryEvent::deallocate(0x6000, 256, 2));

        let origin = ledger
            .thread_stats()
            .get(&1)
            .expect("allocating thread must be present");
        assert_eq!(
            origin.current_memory, 0,
            "the allocating thread must lose its live bytes"
        );

        let actor = ledger
            .thread_stats()
            .get(&2)
            .expect("releasing thread must be recorded as an actor");
        assert_eq!(
            actor.current_memory, 0,
            "a releasing thread must not gain live bytes"
        );
        assert_eq!(
            actor.total_deallocated, 256,
            "the releasing thread must record the freed byte activity"
        );
    }

    /// Objective: Verify unmatched releases are counted, never silently applied to
    /// another binding.
    /// Invariants: the live binding survives and the diagnostic counter increases.
    #[test]
    fn unmatched_release_is_only_diagnostic() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x7000, 32, 1));
        ledger.apply(&MemoryEvent::deallocate(0x9000, 32, 1));

        assert_eq!(ledger.live().len(), 1, "the unrelated binding must survive");
        assert_eq!(
            ledger.stats().unmatched_deallocation_count,
            1,
            "the unmatched release must be reported"
        );
        assert_eq!(
            ledger.stats().current_bytes,
            32,
            "unmatched releases must not change current bytes"
        );
        assert!(ledger.stats().is_exact(), "no counter overflowed");
    }

    /// Objective: Verify a migrating reallocation moves the binding instead of
    /// leaving the old address live.
    /// Invariants: one live binding at the new address with the new size and a fresh
    /// generation; the old address holds nothing.
    #[test]
    fn migrating_reallocation_moves_the_binding() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x7000, 256, 1));
        ledger.apply(&MemoryEvent::reallocate_moved(0x7000, 0x8000, 256, 512, 1));

        assert_eq!(
            ledger.live().len(),
            1,
            "moving must not duplicate the block"
        );
        let live = &ledger.live()[0];
        assert_eq!(live.ptr, 0x8000, "the binding must follow the new address");
        assert_eq!(live.size, 512, "the binding must carry the new size");
        assert_eq!(
            live.generation, 1,
            "the first binding at the new address is generation 1"
        );

        let stats = ledger.stats();
        assert_eq!(
            stats.reallocation_count, 1,
            "the migration must be counted as one reallocation"
        );
        assert_eq!(
            stats.allocation_count, 1,
            "a migration is not a new allocation"
        );
        assert_eq!(stats.current_bytes, 512, "only the new size is live");
        assert_eq!(
            stats.allocated_bytes - stats.deallocated_bytes,
            stats.current_bytes,
            "allocated minus deallocated must equal the live bytes"
        );
    }

    /// Objective: Verify a release of the *old* address after a migration does not
    /// close the moved binding.
    /// Invariants: the late release is counted as unmatched and the moved block stays
    /// live.
    #[test]
    fn late_release_of_pre_move_address_is_diagnostic_only() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0x9000, 128, 1));
        ledger.apply(&MemoryEvent::reallocate_moved(0x9000, 0x9100, 128, 256, 1));
        ledger.apply(&MemoryEvent::deallocate(0x9000, 128, 1));

        assert_eq!(
            ledger.live().len(),
            1,
            "the moved binding must survive a late release of its old address"
        );
        assert_eq!(
            ledger.live()[0].ptr,
            0x9100,
            "the live binding is at the new address"
        );
        assert_eq!(
            ledger.stats().unmatched_deallocation_count,
            1,
            "the stale release must be reported as a diagnostic"
        );
        assert_eq!(
            ledger.stats().current_bytes,
            256,
            "stale releases must not change the live bytes"
        );
    }

    /// Objective: Verify that a second allocation at an address that still has a live
    /// binding keeps both identities and is reported as a conflict.
    /// Invariants: two bindings with generations 1 and 2, one conflict diagnostic, and a
    /// release of the old size closes only generation 1.
    #[test]
    fn reused_address_without_release_keeps_both_generations() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0xC000, 64, 1));
        ledger.apply(&MemoryEvent::allocate(0xC000, 128, 1));

        assert_eq!(
            ledger.live().len(),
            2,
            "both observed bindings must stay live instead of one overwriting the other"
        );
        assert_eq!(
            ledger.stats().conflicting_allocations,
            1,
            "the missing release must be reported as a conflict"
        );
        assert_eq!(
            ledger.live()[0].generation,
            1,
            "the first binding keeps generation 1"
        );
        assert_eq!(
            ledger.live()[1].generation,
            2,
            "the reused address opens generation 2"
        );

        ledger.apply(&MemoryEvent::deallocate(0xC000, 64, 1));

        assert_eq!(
            ledger.live().len(),
            1,
            "exactly one binding may be closed by one release"
        );
        assert_eq!(
            ledger.live()[0].size,
            128,
            "the release of the old size must not close the newer binding"
        );
        assert_eq!(
            ledger.stats().current_bytes,
            128,
            "live bytes must follow the surviving binding"
        );
    }

    /// Objective: Verify a reallocation whose old block was never observed is recorded
    /// at the *new* address and attributed to the thread that performed it.
    /// Invariants: one live binding at `new_ptr`, no invented released bytes, thread
    /// stats match, and the later release of the new address matches.
    #[test]
    fn first_seen_migrating_reallocation_uses_new_address() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::reallocate_moved(0x1000, 0x2000, 64, 128, 7));

        assert_eq!(ledger.live().len(), 1, "one binding must be created");
        let live = &ledger.live()[0];
        assert_eq!(live.ptr, 0x2000, "the binding must live at the new address");
        assert_eq!(live.size, 128, "the binding must carry the new size");

        let stats = ledger.stats();
        assert_eq!(
            stats.unmatched_reallocation_count, 1,
            "the missing old block must be reported as a diagnostic"
        );
        assert_eq!(stats.allocated_bytes, 128, "only the new size is booked");
        assert_eq!(
            stats.deallocated_bytes, 0,
            "no released bytes may be invented for an unobserved block"
        );
        assert_eq!(stats.current_bytes, 128, "the new block is live");

        let thread = ledger
            .thread_stats()
            .get(&7)
            .expect("the performing thread must be attributed");
        assert_eq!(thread.allocation_count, 1, "one allocation is attributed");
        assert_eq!(thread.current_memory, 128, "the thread owns the live bytes");

        ledger.apply(&MemoryEvent::deallocate(0x2000, 128, 7));
        assert!(
            ledger.live().is_empty(),
            "the moved block must be releasable"
        );
        assert_eq!(
            ledger.stats().unmatched_deallocation_count,
            0,
            "the release must match the binding created by the reallocation"
        );
        assert_eq!(ledger.stats().current_bytes, 0, "no live bytes may remain");
    }

    /// Objective: Verify replay cost grows with the number of events, not with its
    /// square (address index instead of a full scan per event).
    /// Invariants: quadrupling the event count must not multiply the runtime by the
    /// quadratic factor (16x); a generous 6x bound leaves room for machine noise.
    #[test]
    fn replay_scales_linearly_with_distinct_addresses() {
        fn replay(count: usize) -> std::time::Duration {
            let events: Vec<MemoryEvent> = (0..count)
                .map(|i| MemoryEvent::allocate(0x1_0000 + i * 8, 64, 1))
                .collect();
            let start = std::time::Instant::now();
            let mut ledger = BindingLedger::new();
            for event in &events {
                ledger.apply(event);
            }
            assert_eq!(ledger.live().len(), count, "every address must be live");
            start.elapsed()
        }

        let small = replay(10_000);
        let large = replay(40_000);

        if small.as_micros() > 0 {
            let ratio = large.as_secs_f64() / small.as_secs_f64();
            assert!(
                ratio < 6.0,
                "replay must stay near-linear: 4x events took {ratio:.1}x the time                  ({small:?} -> {large:?}), which indicates a full scan per event"
            );
        }
    }

    /// Objective: Verify an in-place reallocation keeps the binding's position inside its
    /// address bucket, so a later release still closes the oldest binding first.
    /// Invariants: after G1/64 -> G2/128 -> G1 reallocated to 128, a release of 128 closes
    /// G1 and leaves G2 alive.
    #[test]
    fn in_place_reallocation_keeps_binding_order() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0xD000, 64, 1)); // G1
        ledger.apply(&MemoryEvent::allocate(0xD000, 128, 1)); // G2, G1 reuse without release
        ledger.apply(&MemoryEvent::reallocate(0xD000, 64, 128, 1)); // G1 grows in place

        ledger.apply(&MemoryEvent::deallocate(0xD000, 128, 1));

        assert_eq!(
            ledger.live().len(),
            1,
            "one release must close exactly one binding"
        );
        assert_eq!(
            ledger.live()[0].generation,
            2,
            "the release must close the oldest binding (G1), leaving G2 alive"
        );
    }

    /// Objective: Verify a cross-thread reallocation keeps live bytes with the allocating
    /// thread while booking the activity to the thread that performed it.
    /// Invariants: origin thread holds the live bytes, actor thread holds the released and
    /// allocated bytes but no live bytes.
    #[test]
    fn cross_thread_reallocation_books_activity_to_the_actor() {
        let mut ledger = BindingLedger::new();
        ledger.apply(&MemoryEvent::allocate(0xE000, 64, 1));
        ledger.apply(&MemoryEvent::reallocate(0xE000, 64, 128, 2));

        let origin = ledger.thread_stats().get(&1).expect("origin thread");
        assert_eq!(
            origin.current_memory, 128,
            "live bytes stay with the allocating thread"
        );
        assert_eq!(
            origin.total_deallocated, 0,
            "the origin thread did not perform the release activity"
        );

        let actor = ledger
            .thread_stats()
            .get(&2)
            .expect("the reallocating thread must be recorded");
        assert_eq!(actor.total_allocated, 128, "the new size is booked");
        assert_eq!(actor.total_deallocated, 64, "the old size is booked");
        assert_eq!(actor.current_memory, 0, "the actor owns no live bytes");
    }
}
