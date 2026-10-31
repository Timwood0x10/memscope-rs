//! Allocator producer sharing serials, deduplication slots and events with the facade.

use super::*;
use crate::core::allocator::TrackingGuard;
use std::sync::OnceLock;

/// Borrow the singleton so allocator callbacks never drop an auto-exporting facade.
pub(crate) fn global_tracker() -> &'static Tracker {
    let _suppression = TrackingGuard::acquire();
    static GLOBAL: OnceLock<Tracker> = OnceLock::new();
    GLOBAL.get_or_init(|| Tracker {
        inner: crate::core::tracker::get_tracker(),
        event_store: Arc::new(EventStore::new()),
        recorded_bindings: Arc::new(DashMap::new()),
        config: Arc::new(Mutex::new(TrackerConfig {
            sampling: SamplingConfig::default(),
            auto_export_on_drop: false,
            export_path: None,
        })),
        start_time: Instant::now(),
        system_snapshots: Arc::new(Mutex::new(Vec::new())),
    })
}

impl Tracker {
    /// Publish a successful raw allocation, before a later typed observation annotates it.
    pub(crate) fn record_allocator_allocation(&self, ptr: usize, size: usize) {
        let _gate = crate::core::allocator::capture_gate::acquire(ptr);
        let slot = self.recorded_slot(ptr);
        let mut guard = slot.lock().unwrap_or_else(|error| error.into_inner());
        let recorded = *guard;
        if self.inner.track_allocation(ptr, size).is_err() {
            return;
        }
        let Some(serial) = self.inner.binding_serial(ptr) else {
            return;
        };
        if recorded.is_some_and(|binding| binding.serial == serial && binding.size == size) {
            // The same binding was already published (a custom allocator or a replayed
            // hook reporting the block twice). Publishing again would create a second
            // Allocate for one identity, which every projection would render twice.
            return;
        }
        let mut event = MemoryEvent::allocate(ptr, size, crate::utils::current_thread_id_u64());
        event.binding_serial = Some(serial);
        self.event_store.record(event);
        *guard = Some(RecordedBinding {
            size,
            serial,
            typed: false,
        });
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
            if let Some(binding) = self.record_reallocation(old_ptr, address, old_size, new_size) {
                *old_guard = Some(binding);
            }
        } else {
            let new_slot = self.recorded_slot(address);
            let mut new_guard = new_slot.lock().unwrap_or_else(|error| error.into_inner());
            let binding = self.record_reallocation(old_ptr, address, old_size, new_size);
            if let Some(binding) = binding {
                *old_guard = None;
                *new_guard = Some(binding);
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
