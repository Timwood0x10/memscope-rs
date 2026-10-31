//! Allocator-safe serialisation around the caches that publish capture records.
//!
//! macOS may allocate while registering Rust TLS destructors. Contending on a DashMap
//! at that point enters parking_lot TLS initialisation and aborts, so capture work runs
//! under a native mutex that keeps the cache locks uncontended.
//!
//! The gate is **sharded by address**: two threads publishing about different blocks do
//! not serialise against each other, which matters because this runs on every
//! allocation of the host program. Operations that touch several addresses must take a
//! single shard (the one of the address they started on) and accept that they only
//! exclude other operations on that shard; that is what keeps the lock order flat and
//! makes deadlocks impossible.
//!
//! This is a diagnostic profiler: even sharded, capture work runs inline on the
//! allocating thread. Do not enable the `tracking-allocator` feature in a
//! latency-sensitive production service.

use super::TrackingGuard;
use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

/// Number of shards. Must be a power of two so the index is a cheap mask.
const SHARDS: usize = 64;

static GATES: [Mutex<()>; SHARDS] = [const { Mutex::new(()) }; SHARDS];

thread_local! {
    /// `true` while this thread already holds one shard.
    static ENTERED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct CaptureGate {
    /// Dropped first, so the shard is released before allocator tracking resumes.
    lock: Option<MutexGuard<'static, ()>>,
    _suppression: Option<TrackingGuard>,
}

/// Serialise capture work for the shard that owns `ptr`.
///
/// A nested acquisition on the same thread takes no second shard: the work is already
/// inside a serialised section, and acquiring another shard there could deadlock with a
/// thread that took the same two shards in the opposite order.
pub(crate) fn acquire(ptr: usize) -> CaptureGate {
    let suppression = TrackingGuard::acquire();

    let lock = if ENTERED.with(Cell::get) {
        None
    } else {
        let lock = GATES[shard_index(ptr)]
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ENTERED.with(|entered| entered.set(true));
        Some(lock)
    };

    CaptureGate {
        lock,
        _suppression: suppression,
    }
}

/// Shard of an address.
///
/// Allocations are usually aligned, so the low bits carry little entropy; mix the
/// address before masking to spread consecutive blocks over the shards.
fn shard_index(ptr: usize) -> usize {
    let mixed = ptr.rotate_left(17) ^ (ptr >> 6);
    mixed & (SHARDS - 1)
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

    /// Objective: Verify the shard function is deterministic and spreads consecutive
    /// allocations instead of collapsing them onto one shard.
    /// Invariants: the same address always hashes to the same shard, the index is in
    /// range, and aligned neighbours land on different shards.
    #[test]
    fn shard_index_is_stable_and_spread() {
        let base = 0x1_0000_0000usize;
        let shards: std::collections::HashSet<usize> =
            (0..64).map(|step| shard_index(base + step * 64)).collect();

        assert_eq!(
            shard_index(base),
            shard_index(base),
            "the shard of an address must be stable"
        );
        assert!(
            shards.len() > 1,
            "64 aligned neighbours must not all map to one shard, got {shards:?}"
        );
        assert!(
            shards.iter().all(|shard| *shard < SHARDS),
            "every shard index must be in range"
        );
    }

    /// Objective: Verify nested capture on one thread does not take a second shard.
    /// Invariants: the outer gate owns the lock, the nested one reports no lock, and
    /// the thread-local marker is cleared only when the outer gate ends.
    #[test]
    fn nested_acquisition_reuses_the_outer_shard() {
        ENTERED.with(|entered| entered.set(false));

        {
            let outer = acquire(0x1000);
            assert!(outer.lock.is_some(), "the first gate must take a shard");

            {
                let inner = acquire(0x8000);
                assert!(
                    inner.lock.is_none(),
                    "a nested gate must not take a second shard"
                );
            }

            assert!(
                ENTERED.with(Cell::get),
                "the nested gate must not release the outer shard"
            );
        }

        assert!(
            !ENTERED.with(Cell::get),
            "dropping the outer gate must release the marker"
        );
    }
}
