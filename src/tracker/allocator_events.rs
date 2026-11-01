//! Allocator producer sharing serials, deduplication slots and events with the facade.

use super::*;
use crate::core::allocator::TrackingGuard;
use std::sync::OnceLock;

/// The allocator's facade: created once, never dropped, always on the global backend.
///
/// Three constraints force this shape:
/// * the callback may not touch a thread-local that needs dropping (macOS aborts the
///   process: "the global allocator may not use TLS with destructors");
/// * a release may arrive on a different thread than the allocation, so the binding has
///   to live in a process-wide cache;
/// * constructing and dropping a `Tracker` per callback would run the auto-export check
///   under the configuration lock, which deadlocks when the callback itself allocates.
///
/// `OnceLock` keeps it alive for the process, so neither `Drop` nor the auto-export path
/// can run from inside an allocator callback.
pub(crate) fn global_tracker() -> &'static Tracker {
    static GLOBAL: OnceLock<Tracker> = OnceLock::new();
    GLOBAL.get_or_init(|| {
        let _suppression = TrackingGuard::acquire();
        Tracker {
            inner: crate::capture::backends::core_tracker::get_global_tracker(),
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
    })
}

/// Addresses for which the allocator already charged the current task.
///
/// The core cache cannot answer this: a typed observation inserts the block into the same
/// cache right before asking, so "the binding exists" is true even when nobody charged it.
/// This set is written by the allocator path and cleared when the block is released.
fn charged_addresses() -> &'static dashmap::DashSet<usize> {
    static CHARGED: OnceLock<dashmap::DashSet<usize>> = OnceLock::new();
    CHARGED.get_or_init(dashmap::DashSet::new)
}

/// Whether the allocator already charged a block at `ptr`.
pub(crate) fn allocator_charged(ptr: usize) -> bool {
    charged_addresses().contains(&ptr)
}

/// Forget the charge mark of a released address.
pub(crate) fn clear_allocator_charge(ptr: usize) {
    charged_addresses().remove(&ptr);
}

impl Tracker {
    /// Publish a successful raw allocation, before a later typed observation annotates it.
    pub(crate) fn record_allocator_allocation(&self, ptr: usize, size: usize) {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        let slot = self.recorded_slot(ptr);
        let mut guard = slot.lock().unwrap_or_else(|error| error.into_inner());
        if self.inner.track_allocation(ptr, size).is_err() {
            return;
        }
        let Some(serial) = self.inner.binding_serial(ptr) else {
            return;
        };
        if guard
            .get(&serial)
            .is_some_and(|binding| binding.size == size)
        {
            // The same binding was already published (a custom allocator or a replayed
            // hook reporting the block twice). Publishing again would create a second
            // Allocate for one identity, which every projection would render twice.
            return;
        }
        let mut event = MemoryEvent::allocate(ptr, size, crate::utils::current_thread_id_u64());
        event.binding_serial = Some(serial);
        self.event_store.record(event);

        // Exactly once per binding: the allocator saw the real block, so it charges the
        // task. A later typed observation of the same block only annotates it. The charge
        // never blocks: a blocking write here self-deadlocks when the caller holds the
        // registry read lock and allocates (a string clone, for example).
        crate::task_registry::global_registry().try_record_allocation(size);
        charged_addresses().insert(ptr);
        guard.insert(
            serial,
            RecordedBinding {
                size,
                typed: false,
                charged: true,
            },
        );
    }

    /// Keep the old address slot locked across the system call and publication.
    ///
    /// A moved realloc frees its old address before returning. A new allocator
    /// allocation at that address must wait until this transition is published.
    /// The returned new address is exclusively owned by this operation, so taking
    /// its slot while holding the old slot cannot oppose another valid allocator
    /// operation on that new block.
    pub(crate) fn allocator_reallocation(
        &self,
        old_ptr: usize,
        old_size: usize,
        new_size: usize,
        reallocate: impl FnOnce() -> *mut u8,
    ) -> *mut u8 {
        let _gate = crate::core::allocator::capture_gate::acquire(old_ptr);
        let slot = self.recorded_slot(old_ptr);
        let mut old_guard = slot.lock().unwrap_or_else(|error| error.into_inner());
        let new_ptr = reallocate();
        if new_ptr.is_null() {
            return new_ptr;
        }
        let address = new_ptr as usize;
        if address == old_ptr {
            if let Some((serial, binding)) =
                self.record_reallocation(old_ptr, address, old_size, new_size)
            {
                old_guard.insert(serial, binding);
            }
        } else {
            let new_slot = self.recorded_slot(address);
            let mut new_guard = new_slot.lock().unwrap_or_else(|error| error.into_inner());
            let binding = self.record_reallocation(old_ptr, address, old_size, new_size);
            if let Some((serial, binding)) = binding {
                // Only the moved identity leaves the old address; other live bindings at
                // that address keep their dedupe state.
                old_guard.remove(&serial);
                new_guard.insert(serial, binding);
            }
            drop(old_guard);
            drop(new_guard);
            drop(slot);
            drop(new_slot);
            if binding.is_some() {
                // The old address is no longer in use by this tracker; keeping its slot
                // would make the dedupe index grow with every moved block.
                self.reclaim_slot(old_ptr);
            }
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: A failed system realloc must not close or resize the original binding.
    /// Invariants: a null result publishes no event and preserves the same serial and size.
    #[test]
    fn failed_reallocation_keeps_original_binding() {
        let tracker = Tracker::new();
        tracker.record_allocator_allocation(4096, 64);
        let before = tracker.events();
        let result = tracker.allocator_reallocation(4096, 64, usize::MAX, std::ptr::null_mut);
        assert!(result.is_null(), "injected failure must reach the caller");
        assert_eq!(
            tracker.events().len(),
            before.len(),
            "failure must not publish a lifecycle transition"
        );
        assert_eq!(
            tracker.inner.get_allocation_size(4096),
            Some(64),
            "original block remains live after failure"
        );
        assert_eq!(
            tracker.inner.binding_serial(4096),
            before[0].binding_serial,
            "failure cannot replace identity"
        );
    }

    /// Objective: Verify a hook that reports the same live block twice does not publish a
    /// second allocation event, while a genuinely new binding still is published.
    /// Invariants: one Allocate per binding, two after the address is reused.
    #[test]
    fn repeated_allocator_report_does_not_duplicate_the_event() {
        let tracker = Tracker::new();
        let ptr = 0x2000usize;

        tracker.record_allocator_allocation(ptr, 64);
        tracker.record_allocator_allocation(ptr, 64);

        let allocations = |tracker: &Tracker| {
            tracker
                .events()
                .into_iter()
                .filter(|event| {
                    event.ptr == ptr
                        && event.event_type == crate::event_store::MemoryEventType::Allocate
                })
                .count()
        };

        assert_eq!(
            allocations(&tracker),
            1,
            "a repeated report of the same binding must not publish another event"
        );

        assert!(
            tracker
                .track_deallocation(ptr)
                .expect("release must succeed"),
            "the block must have been tracked before its release"
        );
        tracker.record_allocator_allocation(ptr, 128);

        assert_eq!(
            allocations(&tracker),
            2,
            "a new binding at the reused address must be published"
        );
    }
}
