//! Release and reallocation bookkeeping for the unified facade.
//!
//! Split out of `tracker.rs` to keep that file within the project line budget. These are
//! `impl Tracker` items and share its private state; nothing here is a new API surface.
//!
//! Every function follows the same rule: the event must describe the binding that was
//! actually closed (`ClosedBinding`), and the deduplication state is keyed by binding
//! identity, so closing one generation never erases the state of a surviving one.

use super::*;

impl Tracker {
    pub(super) fn recorded_slot(&self, ptr: usize) -> RecordedSlot {
        self.recorded_bindings
            .entry(ptr)
            .or_insert_with(|| Arc::new(Mutex::new(RecordedState::new())))
            .clone()
    }

    /// Drop the slot of a released address once nobody else can be using it.
    ///
    /// Slots must stay stable while an address is live, but keeping one entry per
    /// address ever allocated makes the profiler itself grow without bound in a
    /// long-running process. `remove_if` holds the shard write lock, so no thread can
    /// obtain the slot while it is inspected; a strong count of one means the map is
    /// its only owner, and `try_lock` (never blocking) then confirms it is empty.
    pub(super) fn reclaim_slot(&self, ptr: usize) {
        self.recorded_bindings.remove_if(&ptr, |_, slot| {
            Arc::strong_count(slot) == 1
                && slot
                    .try_lock()
                    .map(|state| state.is_empty())
                    .unwrap_or(false)
        });
    }

    pub fn track_deallocation(&self, ptr: usize) -> crate::TrackingResult<bool> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);

        // Nothing tracked at this address: report it without allocating a slot, otherwise
        // every untracked release (including allocations the profiler itself suppressed)
        // leaves an empty deduplication slot behind.
        if self.inner.live_binding_count_at(ptr) == 0 {
            return Ok(false);
        }

        let slot = self.recorded_slot(ptr);
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());

        // Name the binding when the address holds exactly one: that is an exact release.
        // With several live bindings only the address is known, so the cache closes the
        // oldest one and counts the guess.
        let serial = if self.inner.live_binding_count_at(ptr) == 1 {
            self.inner.binding_serial(ptr)
        } else {
            None
        };
        let degraded = serial.is_none();
        let closed = self.inner.close_and_report(ptr, serial)?;

        // The event must describe the binding that was actually closed: with an ambiguous
        // release the newest binding is not the one that disappeared, and publishing its
        // serial would make replay close the wrong row.
        if let Some(closed) = closed {
            // Remove exactly the identity that was closed: a degraded release closes the
            // oldest binding, and the surviving one must keep its dedupe state or the next
            // observation of that block would publish a second allocation.
            guard.remove(&closed.serial);
            allocator_events::clear_allocator_charge(ptr);
            let mut event = MemoryEvent::deallocate(
                closed.ptr,
                closed.size,
                crate::utils::current_thread_id_u64(),
            );
            event.binding_serial = Some(closed.serial);
            event.degraded_release = degraded;
            self.event_store.record(event);
        }

        let cleared = guard.is_empty();
        drop(guard);
        drop(slot);
        if cleared {
            self.reclaim_slot(ptr);
        }

        Ok(closed.is_some())
    }

    /// Release one specific binding and publish the matching event.
    ///
    /// Exact entry point: the caller names the identity, so no policy guess is involved
    /// and the published event carries that identity. Returns `false` when the binding is
    /// already gone (a late release of a superseded generation).
    pub fn track_deallocation_of(&self, ptr: usize, serial: u64) -> crate::TrackingResult<bool> {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        let slot = self.recorded_slot(ptr);
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());

        let closed = self.inner.close_and_report(ptr, Some(serial))?;
        if let Some(closed) = closed {
            guard.remove(&closed.serial);
            allocator_events::clear_allocator_charge(ptr);
            let mut event = MemoryEvent::deallocate(
                closed.ptr,
                closed.size,
                crate::utils::current_thread_id_u64(),
            );
            event.binding_serial = Some(closed.serial);
            self.event_store.record(event);
        }

        let cleared = guard.is_empty();
        drop(guard);
        drop(slot);
        if cleared {
            self.reclaim_slot(ptr);
        }
        Ok(closed.is_some())
    }

    /// Track a reallocation of a tracked block.
    ///
    /// The block keeps its identity (and its binding serial): the new size replaces the old
    /// one and the allocation counter does not change, whether the allocator kept the
    /// address or moved the block. The dedupe state moves with the binding, so a later
    /// release still closes the identity it belongs to.
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
            if let Some((serial, binding)) =
                self.record_reallocation(old_ptr, new_ptr, old_size, new_size)
            {
                guard.insert(serial, binding);
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
        if let Some((serial, state)) = binding {
            // Move only the identity that was reallocated: the address may still hold
            // other live bindings, and their dedupe state must survive.
            old_guard.remove(&serial);
            new_guard.insert(serial, state);
        }
        drop(old_guard);
        drop(new_guard);
        drop(old_slot);
        drop(new_slot);
        if binding.is_some() {
            self.reclaim_slot(old_ptr);
        }
    }

    /// Publish a reallocation by looking the old binding up in the core cache.
    ///
    /// Called with the affected address slots locked through event publication.
    pub(super) fn record_reallocation(
        &self,
        old_ptr: usize,
        new_ptr: usize,
        old_size: usize,
        new_size: usize,
    ) -> Option<(u64, RecordedBinding)> {
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
        Some((
            serial,
            RecordedBinding {
                size: new_size,
                typed: false,
                charged: true,
            },
        ))
    }
}
