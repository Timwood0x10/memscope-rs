//! Allocator-safe serialisation around the caches that publish capture records.
//!
//! macOS may allocate while registering Rust TLS destructors. Contending on a DashMap
//! at that point enters parking_lot TLS initialisation and aborts, so capture work runs
//! under a native mutex that keeps the cache locks uncontended.
//!
//! One gate covers all addresses: DashMaps have independent randomized shard hashes,
//! and range scans and reallocations touch multiple addresses. Address-sharded gates
//! cannot prevent contention on those inner locks. Nested capture reuses the gate so
//! callers can compose cache operations without introducing another lock order.
//!
//! This is a diagnostic profiler: capture work runs inline on the
//! allocating thread. Do not enable the `tracking-allocator` feature in a
//! latency-sensitive production service.

use super::TrackingGuard;
use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

static GATE: Mutex<()> = Mutex::new(());

thread_local! {
    /// No destructor: this flag is safe during runtime TLS registration.
    static ENTERED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct CaptureGate {
    /// Dropped first, so the gate is released before allocator tracking resumes.
    lock: Option<MutexGuard<'static, ()>>,
    _suppression: Option<TrackingGuard>,
}

/// Serialise capture work, including operations spanning several addresses.
///
/// The address parameter keeps cache call sites explicit about the operation's subject.
/// A nested acquisition on the same thread is already inside the serialised section.
pub(crate) fn acquire(_ptr: usize) -> CaptureGate {
    let suppression = TrackingGuard::acquire();

    let lock = if ENTERED.with(Cell::get) {
        None
    } else {
        // This mutex protects no data; it only orders access to independently guarded
        // caches. Recovering the unit guard cannot expose poisoned application state.
        let lock = GATE.lock().unwrap_or_else(|error| error.into_inner());
        ENTERED.with(|entered| entered.set(true));
        Some(lock)
    };

    CaptureGate {
        lock,
        _suppression: suppression,
    }
}

impl Drop for CaptureGate {
    fn drop(&mut self) {
        if self.lock.is_some() {
            ENTERED.with(|entered| entered.set(false));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Prevent inner cache lock contention even for different addresses.
    /// Invariants: at most one of 50 threads may execute capture work at a time.
    #[test]
    fn different_addresses_share_exclusion() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Barrier;

        let active = AtomicUsize::new(0);
        let start = Barrier::new(50);
        std::thread::scope(|scope| {
            for address in 0..50 {
                let active = &active;
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    for _ in 0..20 {
                        let _gate = acquire(0x1000 + address * 64);
                        let previous = active.fetch_add(1, Ordering::SeqCst);
                        std::thread::yield_now();
                        active.fetch_sub(1, Ordering::SeqCst);
                        assert_eq!(previous, 0, "capture sections must never overlap");
                    }
                });
            }
        });
    }

    /// Objective: Verify nested capture on one thread does not acquire the gate again.
    /// Invariants: the outer gate owns the lock, the nested one reports no lock, and
    /// the thread-local marker is cleared only when the outer gate ends.
    #[test]
    fn nested_acquisition_reuses_the_outer_gate() {
        ENTERED.with(|entered| entered.set(false));

        {
            let outer = acquire(0x1000);
            assert!(outer.lock.is_some(), "the first gate must take the lock");

            {
                let inner = acquire(0x8000);
                assert!(
                    inner.lock.is_none(),
                    "a nested gate must not take the lock again"
                );
            }

            assert!(
                ENTERED.with(Cell::get),
                "the nested gate must not release the outer lock"
            );
        }

        assert!(
            !ENTERED.with(Cell::get),
            "dropping the outer gate must release the marker"
        );
    }
}
