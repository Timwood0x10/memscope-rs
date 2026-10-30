//! Cross-projection consistency for event reconstruction.
//!
//! The snapshot and the dashboard each build a view over the same event stream.
//! They must not disagree about who is alive, how much memory is live, or how
//! many releases/reallocations happened — that divergence is exactly what the
//! development plan lists as A03/A04/A06/A07.
//!
//! The expectations here are derived by hand from the scenario, never by calling
//! the implementation under test.

use memscope_rs::event_store::MemoryEvent;
use memscope_rs::render_engine::dashboard::rebuild_allocations_from_events;
use memscope_rs::snapshot::build_snapshot_from_events;
use std::collections::{BTreeSet, HashMap};

/// Replay scenario covering reallocation, cross-thread release and address reuse
/// with a late release of the previous generation.
///
/// Timeline:
/// 1. alloc 0x1000 / 64  (thread 1)
/// 2. realloc 0x1000 64 -> 128 in place (thread 1)
/// 3. alloc 0x2000 / 256 (thread 1)
/// 4. free 0x1000 / 128  (thread 2, cross-thread)
/// 5. alloc 0x2000 / 32  (thread 3, reuse while the 256-byte binding is still live)
/// 6. free 0x2000 / 256  (thread 1, late release of the older generation)
fn scenario() -> Vec<MemoryEvent> {
    vec![
        MemoryEvent::allocate(0x1000, 64, 1),
        MemoryEvent::reallocate(0x1000, 64, 128, 1),
        MemoryEvent::allocate(0x2000, 256, 1),
        MemoryEvent::deallocate(0x1000, 128, 2),
        MemoryEvent::allocate(0x2000, 32, 3),
        MemoryEvent::deallocate(0x2000, 256, 1),
    ]
}

/// Live `(address, size)` pairs from the dashboard projection.
fn dashboard_live_set(events: &[MemoryEvent]) -> BTreeSet<(usize, usize)> {
    rebuild_allocations_from_events(events)
        .into_iter()
        .map(|a| (a.ptr, a.size))
        .collect()
}

/// Live `(address, size)` pairs from the snapshot projection.
fn snapshot_live_set(events: &[MemoryEvent]) -> BTreeSet<(usize, usize)> {
    build_snapshot_from_events(events)
        .active_allocations
        .values()
        .map(|a| {
            (
                a.ptr.expect("active allocations always carry an address"),
                a.size,
            )
        })
        .collect()
}

/// Objective: Verify both projections describe the same live set after
/// reallocation, cross-thread release and address reuse.
/// Invariants: only the newest 0x2000 binding (32 bytes) survives, in both views.
#[test]
fn snapshot_and_dashboard_agree_on_live_set() {
    let events = scenario();
    let expected: BTreeSet<(usize, usize)> = [(0x2000usize, 32usize)].into_iter().collect();

    assert_eq!(
        snapshot_live_set(&events),
        expected,
        "the snapshot must keep only the newest binding at the reused address"
    );
    assert_eq!(
        dashboard_live_set(&events),
        expected,
        "the dashboard must agree with the snapshot on the live set"
    );
}

/// Objective: Verify both projections agree on the ledger totals.
/// Invariants: 3 allocation events (+1 for the unknown-block path is not used
/// here), 1 reallocation, 2 releases, 32 live bytes and a 384-byte peak.
#[test]
fn snapshot_and_dashboard_agree_on_totals() {
    let events = scenario();
    let snapshot = build_snapshot_from_events(&events);

    assert_eq!(
        snapshot.stats.total_allocations, 3,
        "three allocation events must produce three bindings"
    );
    assert_eq!(
        snapshot.stats.total_reallocations, 1,
        "the in-place reallocation must be counted separately"
    );
    assert_eq!(
        snapshot.stats.total_deallocations, 2,
        "both releases must match a binding"
    );
    assert_eq!(
        snapshot.stats.unmatched_deallocations, 0,
        "no release in this scenario is unmatched"
    );
    assert_eq!(
        snapshot.stats.current_memory, 32,
        "only the 32-byte binding remains live"
    );
    assert_eq!(
        snapshot.stats.peak_memory, 384,
        "peak must be the high-water mark (64 -> 128 -> 384 -> 288 -> 32)"
    );
    assert_eq!(
        snapshot.stats.total_allocated, 480,
        "allocated bytes must cover 64 + 128 (reallocated) + 256 + 32"
    );
    assert_eq!(
        snapshot.stats.total_deallocated, 448,
        "deallocated bytes must cover 64 (realloc old) + 128 + 256"
    );
    assert_eq!(
        snapshot.stats.total_allocated - snapshot.stats.total_deallocated,
        snapshot.stats.current_memory,
        "allocated minus deallocated must equal the live bytes"
    );

    let dashboard_bytes: usize = rebuild_allocations_from_events(&events)
        .iter()
        .map(|a| a.size)
        .sum();
    assert_eq!(
        dashboard_bytes, snapshot.stats.current_memory,
        "dashboard bytes and snapshot current memory must be identical"
    );
}

/// Objective: Verify per-thread attribution matches between projections and that
/// a cross-thread release clears the allocating thread's live bytes.
/// Invariants: thread 3 owns the 32 live bytes; thread 2 performed the release
/// activity for the 128-byte block, while its live bytes stay zero.
#[test]
fn snapshot_and_dashboard_agree_on_thread_attribution() {
    let events = scenario();
    let snapshot = build_snapshot_from_events(&events);
    let dashboard = rebuild_allocations_from_events(&events);

    let mut dashboard_by_thread: HashMap<u64, usize> = HashMap::new();
    for alloc in &dashboard {
        *dashboard_by_thread.entry(alloc.thread_id_u64).or_insert(0) += alloc.size;
    }

    for (thread_id, stats) in &snapshot.thread_stats {
        let dashboard_bytes = dashboard_by_thread.get(thread_id).copied().unwrap_or(0);
        assert_eq!(
            stats.current_memory, dashboard_bytes,
            "thread {thread_id} live bytes must match between projections"
        );
    }

    let origin = snapshot
        .thread_stats
        .get(&1)
        .expect("the allocating thread must be present");
    assert_eq!(
        origin.current_memory, 0,
        "the allocating thread must not keep live bytes after its blocks were released"
    );

    let actor = snapshot
        .thread_stats
        .get(&2)
        .expect("the releasing thread must be recorded");
    assert_eq!(
        actor.current_memory, 0,
        "releasing a block must not give the releasing thread live bytes"
    );
    assert_eq!(
        actor.total_deallocated, 128,
        "the releasing thread must record the bytes it freed"
    );
}

/// Objective: Verify the dashboard reports the refreshed size after a
/// reallocation instead of the stale one.
/// Invariants: the reallocated block reports 128 bytes, not the original 64.
#[test]
fn dashboard_applies_reallocation_size() {
    let events = vec![
        MemoryEvent::allocate(0x1000, 64, 1),
        MemoryEvent::reallocate(0x1000, 64, 128, 1),
    ];

    let dashboard = rebuild_allocations_from_events(&events);
    assert_eq!(dashboard.len(), 1, "reallocation must not add a row");
    assert_eq!(
        dashboard[0].size, 128,
        "the dashboard must show the reallocated size"
    );

    let snapshot = build_snapshot_from_events(&events);
    let active = snapshot
        .active_allocations
        .get(&0x1000)
        .expect("the reallocated block must stay active");
    assert_eq!(
        active.size, 128,
        "the snapshot must agree with the dashboard"
    );
}

/// Objective: Verify the binding generation exposed to the dashboard starts at 1
/// for a first use, so the address-reuse metric cannot count a first allocation.
/// Invariants: first binding generation is 1; after release and reuse it is 2.
#[test]
fn dashboard_generation_starts_at_one() {
    let events = vec![
        MemoryEvent::allocate(0x3000, 16, 1),
        MemoryEvent::deallocate(0x3000, 16, 1),
        MemoryEvent::allocate(0x3000, 16, 1),
    ];

    let dashboard = rebuild_allocations_from_events(&events);
    assert_eq!(dashboard.len(), 1, "only one binding may remain live");
    assert_eq!(
        dashboard[0].generation_id, 2,
        "a reused address must report generation 2, so `> 1` is the reuse test"
    );

    let first_use = rebuild_allocations_from_events(&[MemoryEvent::allocate(0x4000, 16, 1)]);
    assert_eq!(
        first_use[0].generation_id, 1,
        "a first allocation must report generation 1"
    );
}

/// Objective: Verify a migrating reallocation is visible to both projections: the
/// block moves to the new address with the new size and the old address is gone.
/// Invariants: live sets match, bytes stay conserved, and releasing the new address
/// leaves nothing live.
#[test]
fn migrating_reallocation_is_applied_by_both_projections() {
    let events = vec![
        MemoryEvent::allocate(0x1000, 256, 1),
        MemoryEvent::reallocate_moved(0x1000, 0x2000, 256, 512, 1),
    ];

    let expected: BTreeSet<(usize, usize)> = [(0x2000usize, 512usize)].into_iter().collect();
    assert_eq!(
        snapshot_live_set(&events),
        expected,
        "the snapshot must move the binding to the new address"
    );
    assert_eq!(
        dashboard_live_set(&events),
        expected,
        "the dashboard must agree with the snapshot after a migration"
    );

    let snapshot = build_snapshot_from_events(&events);
    assert_eq!(
        snapshot.stats.total_allocations, 1,
        "a migration is not a new allocation"
    );
    assert_eq!(
        snapshot.stats.total_reallocations, 1,
        "the migration must be counted as a reallocation"
    );
    assert_eq!(
        snapshot.stats.current_memory, 512,
        "live bytes must follow the moved block"
    );
}
