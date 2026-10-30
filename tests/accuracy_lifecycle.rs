//! Accuracy regressions for allocation life-cycle facts (development plan PR-01).
//!
//! The tests here pin the contract introduced by "stop fabricating releases":
//!
//! * destroying a tracker (or a clone of it) is a profiler event, never an object
//!   release, and therefore must not append `Deallocate` events;
//! * an object that was only observed must never be rendered as a leak;
//! * a release is only counted when a real release observation exists, and it is
//!   counted exactly once;
//! * the legacy `with_auto_export` drop path only projects already recorded
//!   events, so the exported files describe live objects as live.
//!
//! Every assertion carries a message: a silent pass on a wrong invariant would be
//! worse than a failure.

use memscope_rs::analysis::memory_passport_tracker::MemoryPassportTracker;
use memscope_rs::core::types::TrackKind;
use memscope_rs::event_store::{MemoryEvent, MemoryEventType};
use memscope_rs::render_engine::dashboard::DashboardRenderer;
use memscope_rs::snapshot::{AllocationState, LeakStatus};
use memscope_rs::tracker::Tracker;
use memscope_rs::Trackable;
use serial_test::serial;
use std::sync::Arc;
use tempfile::TempDir;

/// Count allocation (`Allocate`) events in a recorded event slice.
fn count_allocations(events: &[MemoryEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e.event_type, MemoryEventType::Allocate))
        .count()
}

/// Count release (`Deallocate`) events in a recorded event slice.
fn count_deallocations(events: &[MemoryEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e.event_type, MemoryEventType::Deallocate))
        .count()
}

/// Objective: Prove that dropping a tracker clone does not fabricate releases.
/// Invariants: the shared event store keeps the same event count, zero
/// `Deallocate` events exist, and the surviving tracker still reports one live
/// allocation with no deallocations.
#[test]
fn tracker_clone_drop_does_not_record_release() {
    let tracker = Tracker::new();
    let payload = vec![1u8; 128];
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());

    let before = tracker.events();
    assert_eq!(
        count_allocations(&before),
        1,
        "tracking one Vec<u8> must record exactly one allocation event, got {}",
        count_allocations(&before)
    );
    assert_eq!(
        count_deallocations(&before),
        0,
        "no release was observed yet, so no Deallocate event may exist"
    );

    let shared = tracker.clone();
    drop(shared);

    let after = tracker.events();
    assert_eq!(
        after.len(),
        before.len(),
        "dropping a tracker clone must not append events (before={}, after={})",
        before.len(),
        after.len()
    );
    assert_eq!(
        count_deallocations(&after),
        0,
        "profiler destruction must never be recorded as an object release"
    );

    let report = tracker.analyze();
    assert_eq!(
        report.active_allocations, 1,
        "the observed object must still be active after the clone drop"
    );
    assert_eq!(
        report.total_deallocations, 0,
        "no deallocation was observed, so the report must not count one"
    );
}

/// Objective: Prove that dropping the last tracker handle is not a release.
/// Invariants: the event store handed out before the drop keeps its size and
/// contains no `Deallocate` event afterwards.
#[test]
fn tracker_drop_does_not_record_release() {
    let tracker = Tracker::new();
    let payload = vec![2u8; 256];
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());

    let store = Arc::clone(tracker.event_store());
    let before = store.snapshot();
    drop(tracker);

    let after = store.snapshot();
    assert_eq!(
        after.len(),
        before.len(),
        "dropping the tracker must not append events (before={}, after={})",
        before.len(),
        after.len()
    );
    assert_eq!(
        count_deallocations(&after),
        0,
        "no real release happened, so the store must not contain Deallocate events"
    );
}

/// Objective: Prove that the dashboard does not derive "leaked" from a missing
/// deallocation timestamp.
/// Invariants: the observed object is `Live`, its leak status is `NotAssessed`,
/// `is_leaked` is false and the dashboard leak counter stays zero.
#[test]
fn live_object_is_not_reported_as_leak() {
    let tracker = Tracker::new();
    let payload = vec![9u8; 512];
    tracker.track_as(&payload, "live_payload", file!(), line!(), module_path!());

    let passports = Arc::new(MemoryPassportTracker::default());
    let renderer = DashboardRenderer::new().expect("dashboard renderer must be constructible");
    let context = renderer
        .build_context_from_tracker(&tracker, &passports)
        .expect("dashboard context must build from a tracker with one live allocation");

    assert_eq!(
        context.allocations.len(),
        1,
        "dashboard must show exactly the one observed object, got {:?}",
        context
            .allocations
            .iter()
            .map(|a| a.var_name.clone())
            .collect::<Vec<_>>()
    );

    let row = &context.allocations[0];
    assert!(
        !row.is_leaked,
        "an object observed without a release must not be reported as leaked"
    );
    assert_eq!(
        row.leak_status,
        LeakStatus::NotAssessed,
        "no leak rule ran yet, so the leak status must be NotAssessed"
    );
    assert_eq!(
        row.allocation_state,
        AllocationState::Live,
        "no release was recorded, so the allocation state must be Live"
    );
    assert_eq!(
        row.timestamp_dealloc, 0,
        "the legacy deallocation timestamp must stay 0 when nothing was released"
    );
    assert_eq!(
        context.leak_count, 0,
        "the dashboard leak counter must not count unassessed allocations"
    );
}

/// Objective: Prove that a real release is counted exactly once.
/// Invariants: only the first `track_deallocation` call succeeds, exactly one
/// `Deallocate` event exists, the active ledger returns to zero and the
/// allocated/deallocated byte totals stay conserved.
#[test]
fn explicit_release_decrements_ledger_exactly_once() {
    let tracker = Tracker::new();
    let payload = vec![3u8; 64];
    let ptr = match payload.track_kind() {
        TrackKind::HeapOwner { ptr, .. } => ptr,
        other => panic!("Vec<u8> must be tracked as HeapOwner, got {other:?}"),
    };
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());

    let first = tracker
        .track_deallocation(ptr)
        .expect("the first release of a tracked pointer must not fail");
    assert!(
        first,
        "the first release must be accepted because the pointer is tracked"
    );

    let second = tracker
        .track_deallocation(ptr)
        .expect("releasing an untracked pointer must report false, not error");
    assert!(
        !second,
        "a second release of the same pointer must be rejected (double free protection)"
    );

    let events = tracker.events();
    assert_eq!(
        count_allocations(&events),
        1,
        "release must not duplicate the allocation event"
    );
    assert_eq!(
        count_deallocations(&events),
        1,
        "the duplicated release must not produce a second Deallocate event"
    );

    let report = tracker.analyze();
    assert_eq!(
        report.total_deallocations, 1,
        "the ledger must count exactly one deallocation, got {}",
        report.total_deallocations
    );
    assert_eq!(
        report.active_allocations, 0,
        "the released allocation must leave the active set"
    );
    assert_eq!(
        report.current_memory_bytes, 0,
        "current bytes must return to zero after the only allocation is released"
    );
    assert_eq!(
        report.total_allocations, 1,
        "the allocation count must not change when the block is released"
    );
}

/// Objective: Prove that the legacy `with_auto_export` drop path is read-only
/// and still writes the three JSON projections.
/// Invariants: the shared event store is unchanged by the drop, no `Deallocate`
/// event appears, all expected files exist and the exported analysis reports one
/// still-live allocation.
#[test]
fn auto_export_on_drop_is_read_only() {
    let dir = TempDir::new().expect("tempdir must be created for the export target");
    let path = dir
        .path()
        .to_str()
        .expect("tempdir path must be valid UTF-8");
    let tracker = Tracker::new().with_auto_export(path);

    let payload = vec![5u8; 32];
    tracker.track_as(&payload, "exported", file!(), line!(), module_path!());

    let store = Arc::clone(tracker.event_store());
    let before = store.snapshot();
    drop(tracker);
    let after = store.snapshot();

    assert_eq!(
        after.len(),
        before.len(),
        "the compatibility snapshot must not append events (before={}, after={})",
        before.len(),
        after.len()
    );
    assert_eq!(
        count_deallocations(&after),
        0,
        "exporting on drop must not fabricate a release"
    );

    for name in [
        "memory_analysis.json",
        "lifetime.json",
        "thread_analysis.json",
    ] {
        let file = dir.path().join(name);
        assert!(
            file.exists(),
            "auto-export on drop must still write {name} at {}",
            file.display()
        );
    }

    let analysis = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("memory_analysis.json must be readable after the drop export");
    let json: serde_json::Value =
        serde_json::from_str(&analysis).expect("memory_analysis.json must be valid JSON");

    assert_eq!(
        json["metadata"]["total_allocations"], 1,
        "the exported snapshot must keep the single live allocation"
    );
    assert_eq!(
        json["allocations"][0]["var_name"], "exported",
        "the exported record must keep the tracked variable name"
    );

    // The live object must be described as an observed allocation, not as a
    // released one: the lifetime projection has no deallocation timestamp.
    let lifetime = std::fs::read_to_string(dir.path().join("lifetime.json"))
        .expect("lifetime.json must be readable after the drop export");
    let lifetime_json: serde_json::Value =
        serde_json::from_str(&lifetime).expect("lifetime.json must be valid JSON");
    let histories = lifetime_json["ownership_histories"]
        .as_array()
        .expect("lifetime.json must contain an ownership_histories array");
    assert_eq!(
        histories.len(),
        1,
        "the lifetime projection must cover the single live allocation"
    );
    assert!(
        histories[0]["timestamp_dealloc"].is_null(),
        "a live allocation must not gain a deallocation timestamp on export"
    );
}

/// Objective: Verify a duplicate observation of the same buffer is reported as a
/// diagnostic instead of being counted as a second allocation (plan A06).
/// Invariants: the dashboard JSON keeps one allocation and exposes the duplicate
/// count in `event_summary`.
#[test]
fn duplicate_observation_is_reported_not_counted() {
    let tracker = Tracker::new();
    let payload = vec![7u8; 128];
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());

    assert_eq!(
        tracker.duplicate_observations(),
        1,
        "the second sighting of the same address must be reported as a duplicate"
    );

    let passports = Arc::new(MemoryPassportTracker::default());
    let renderer = DashboardRenderer::new().expect("dashboard renderer must be constructible");
    let context = renderer
        .build_context_from_tracker(&tracker, &passports)
        .expect("dashboard context must build");

    let json: serde_json::Value =
        serde_json::from_str(&context.json_data).expect("dashboard json_data must be valid JSON");
    assert_eq!(
        json["event_summary"]["duplicate_observations"], 1,
        "the duplicate count must reach the dashboard JSON"
    );
    assert_eq!(
        context.allocations.len(),
        1,
        "a duplicate observation must not create a second allocation row"
    );
}

/// Objective: Verify a block the allocator hook registered in the core cache (without
/// emitting an event) is still reported when it is observed through `track!`
/// (plan A06 follow-up).
/// Invariants: exactly one `Allocate` event for the address, and a second observation of
/// the same address and size adds none.
#[test]
#[serial]
fn hook_registered_block_is_reported_by_typed_observation() {
    let data = vec![7u8; 64];
    let ptr = data.as_ptr() as usize;

    // Simulate the allocator hook: it writes the cache only, never the event store.
    memscope_rs::core::tracker::get_tracker()
        .track_allocation(ptr, data.capacity())
        .expect("hook registration must succeed");

    let tracker = Tracker::global();
    let events_for_ptr = |tracker: &Tracker| {
        tracker
            .events()
            .iter()
            .filter(|event| {
                event.ptr == ptr && matches!(event.event_type, MemoryEventType::Allocate)
            })
            .count()
    };

    let before = events_for_ptr(&tracker);
    tracker.track_as(&data, "hook_registered", file!(), line!(), module_path!());
    let after_first = events_for_ptr(&tracker);
    assert_eq!(
        after_first - before,
        1,
        "a hook-registered block must still produce one allocation event when observed"
    );

    tracker.track_as(&data, "hook_registered", file!(), line!(), module_path!());
    let after_second = events_for_ptr(&tracker);
    assert_eq!(
        after_second, after_first,
        "a repeat observation of the same address and size must not duplicate the event"
    );
}
