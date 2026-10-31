//! Completed address-binding observations, separate from the live set.

use super::BindingRecord;

/// A binding closed by an observed, identity-matched deallocation.
///
/// An unmatched release cannot supply a lifetime. Address migration is a binding
/// transition, not evidence that the logical allocation was freed.
#[derive(Debug, Clone)]
pub struct ReleasedBinding {
    pub binding: BindingRecord,
    pub deallocated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::super::BindingLedger;
    use crate::event_store::MemoryEvent;
    use proptest::prelude::*;

    /// Objective: Keep censored objects and unmatched releases out of lifetime history.
    /// Invariants: a cross-thread release closes only its serial, including at a reused address.
    #[test]
    fn history_retains_only_matched_releases() {
        let mut ledger = BindingLedger::new();
        for serial in [1, 2] {
            let mut event = MemoryEvent::allocate(4096, 64, 7);
            event.binding_serial = Some(serial);
            event.timestamp = serial * 100;
            ledger.apply(&event);
        }
        let mut release = MemoryEvent::deallocate(4096, 64, 9);
        release.binding_serial = Some(1);
        release.timestamp = 300;
        ledger.apply(&release);
        ledger.apply(&release);

        assert_eq!(
            ledger.released().len(),
            1,
            "duplicate frees cannot create lifetimes"
        );
        assert_eq!(
            ledger.released()[0].binding.allocated_at,
            100,
            "match the original observation"
        );
        assert_eq!(
            ledger.released()[0].deallocated_at,
            300,
            "retain the actual release time"
        );
        assert_eq!(
            ledger.live()[0].binding_serial,
            Some(2),
            "the live generation stays censored"
        );
        assert_eq!(
            ledger.thread_stats()[&7].current_memory,
            64,
            "release clears origin bytes"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Objective: Retain release history without changing byte accounting for arbitrary observations.
        /// Invariants: one matched release produces one history row and zero live bytes, including size zero.
        #[test]
        fn history_conserves_bytes(ptr in any::<usize>(), size in 0usize..1_000_000, start in 0u64..1_000_000) {
            let mut ledger = BindingLedger::new();
            let mut allocation = MemoryEvent::allocate(ptr, size, 1);
            allocation.timestamp = start;
            ledger.apply(&allocation);
            let mut release = MemoryEvent::deallocate(ptr, size, 2);
            release.timestamp = start + 100;
            ledger.apply(&release);
            prop_assert_eq!(ledger.stats().current_bytes, 0, "all recorded bytes were released");
            prop_assert_eq!(ledger.released().len(), 1, "exactly one matched lifetime");
            prop_assert_eq!(ledger.released()[0].binding.ptr, ptr, "history preserves the address without dereferencing it");
            prop_assert_eq!(ledger.released()[0].binding.size, size, "history preserves the recorded size");
        }
    }
}
