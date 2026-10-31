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
    assert!(
        histories[0]["lifetime_ms"].is_null(),
        "a live allocation has no complete lifetime to report"
    );
    assert!(
        histories[0]["observed_age_ms"].is_u64(),
        "the observed age must be reported instead of a fabricated lifetime"
    );
    assert_eq!(
        histories[0]["lifetime_complete"], false,
        "the export must flag that the lifetime is not complete"
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

/// Objective: Verify that releasing a block through the allocator hook invalidates the
/// dedupe index, so a *new* block at the same address and size is still reported
/// (plan A06 follow-up).
/// Invariants: the second observation emits its own allocation event instead of being
/// swallowed as a duplicate.
#[test]
#[serial]
fn reuse_after_hook_release_is_not_deduplicated() {
    let tracker = Tracker::global();
    let data = vec![3u8; 32];
    let ptr = data.as_ptr() as usize;
    let size = data.capacity();
    let core = memscope_rs::core::tracker::get_tracker();

    let allocate_events = |tracker: &Tracker| {
        tracker
            .events()
            .iter()
            .filter(|event| {
                event.ptr == ptr && matches!(event.event_type, MemoryEventType::Allocate)
            })
            .count()
    };

    // First block: hook registers it, then the typed observation reports it.
    core.track_allocation(ptr, size).expect("hook register");
    tracker.track_as(&data, "reused", file!(), line!(), module_path!());
    let after_first = allocate_events(&tracker);
    assert_eq!(after_first, 1, "the first block must be reported once");

    // Hook release (the real allocator path, not `Tracker::track_deallocation`).
    core.track_deallocation(ptr).expect("hook release");

    // A new block lands on the same address with the same size.
    core.track_allocation(ptr, size).expect("hook re-register");
    tracker.track_as(&data, "reused", file!(), line!(), module_path!());

    assert_eq!(
        allocate_events(&tracker),
        after_first + 1,
        "a new binding at a reused address must not be deduplicated away"
    );
}

/// Objective: Verify a smart pointer tracked inside a task charges that task once
/// (plan A08 parity for StackOwner).
/// Invariants: the task's allocation count and memory usage grow by the tracked size.
#[test]
#[serial]
fn smart_pointer_observation_charges_the_current_task() {
    let registry = memscope_rs::task_registry::global_registry();
    registry.clear();
    let tracker = Tracker::new();
    let shared = std::sync::Arc::new(vec![9u8; 16]);

    {
        let _scope = registry.task_scope("arc_owner");
        let task_id = memscope_rs::task_registry::TaskIdRegistry::current_task_id()
            .expect("the task scope must install a current task");
        tracker.track_as(&shared, "shared", file!(), line!(), module_path!());

        let meta = registry.get_task(task_id).expect("the task must exist");
        assert_eq!(
            meta.allocation_count, 1,
            "tracking an Arc must charge one allocation to the task"
        );
        assert!(
            meta.memory_usage > 0,
            "tracking an Arc must charge bytes to the task, got {}",
            meta.memory_usage
        );
    }
}

/// Objective: Verify a type observed after an untyped reallocation is attached to the
/// existing binding instead of being dropped as a duplicate (plan A06 follow-up).
/// Invariants: exactly one live binding, and it carries the observed type name.
#[test]
fn type_after_untyped_reallocation_is_attached() {
    let tracker = Tracker::new();
    let data = vec![5u8; 16];
    let ptr = data.as_ptr() as usize;

    // Simulate an allocator reallocation into this address: the event has no type.
    tracker.track_reallocation(ptr + 8, ptr, 8, data.capacity());
    tracker.track_as(&data, "payload", file!(), line!(), module_path!());

    // A projection that rebuilds from the event stream must see the type: the metadata
    // annotation has to reach the binding instead of being dropped as a duplicate.
    let rebuilt =
        memscope_rs::render_engine::dashboard::rebuild_allocations_from_events(&tracker.events());
    let binding = rebuilt
        .iter()
        .find(|allocation| allocation.ptr == ptr)
        .expect("the moved binding must be present");
    assert_eq!(
        binding.type_name.as_deref(),
        Some("Vec<T>"),
        "the later type observation must annotate the existing binding"
    );
}

/// Objective: Verify the exported thread report projects the ledger's cumulative totals and
/// high-water mark instead of re-aggregating the live list (plan A06 follow-up).
/// Invariants: after allocating 64 + 128 and releasing 128, the export reports 192 bytes
/// allocated and a 192-byte peak with 64 bytes live.
#[test]
#[serial]
fn export_reports_ledger_totals_after_release() {
    let tracker = Tracker::new();
    let kept = vec![1u8; 64];
    let released = vec![2u8; 128];
    tracker.track_as(&kept, "kept", file!(), line!(), module_path!());
    tracker.track_as(&released, "released", file!(), line!(), module_path!());
    assert!(
        tracker
            .track_deallocation(released.as_ptr() as usize)
            .expect("release must succeed"),
        "the second block must be tracked before it is released"
    );

    let dir = TempDir::new().expect("tempdir must be creatable");
    let passports = Arc::new(MemoryPassportTracker::default());
    let async_tracker =
        Arc::new(memscope_rs::capture::backends::async_tracker::AsyncTracker::new());
    memscope_rs::render_engine::export::export_all_json(
        dir.path(),
        &tracker,
        &passports,
        &async_tracker,
    )
    .expect("export must succeed");

    let thread_json = std::fs::read_to_string(dir.path().join("thread_analysis.json"))
        .expect("thread_analysis.json must exist");
    let thread_data: serde_json::Value =
        serde_json::from_str(&thread_json).expect("thread_analysis.json must be valid JSON");
    let totals = &thread_data["thread_analysis"][0];
    assert_eq!(
        totals["allocation_count"], 2,
        "both allocations must be counted"
    );
    assert_eq!(
        totals["total_allocated"], 192,
        "cumulative allocated bytes must survive the release"
    );
    assert_eq!(
        totals["peak_memory"], 192,
        "the high-water mark must survive the release"
    );
    assert_eq!(
        totals["current_memory"], 64,
        "only the unreleased block is live"
    );

    let analysis_json = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("the export must write memory_analysis.json");
    let analysis: serde_json::Value =
        serde_json::from_str(&analysis_json).expect("memory_analysis.json must be valid JSON");
    let statistics = &analysis["statistics"];
    assert_eq!(
        statistics["live_size_bytes"], 64,
        "the live scope must describe only what is still allocated"
    );
    assert_eq!(
        statistics["cumulative_allocated_bytes"], 192,
        "the cumulative scope must keep the bytes of the released block"
    );
    assert_eq!(
        statistics["cumulative_deallocated_bytes"], 128,
        "the cumulative scope must keep the released bytes"
    );
    assert_eq!(
        statistics["peak_bytes"], 192,
        "the high-water mark must survive the release"
    );
    assert_eq!(
        statistics["allocation_count"], 2,
        "both allocations must be counted cumulatively"
    );
    assert_eq!(
        analysis["allocations"]
            .as_array()
            .expect("allocations must be an array")
            .len(),
        1,
        "the row list describes the same cut as the live counters"
    );
    assert_eq!(
        analysis["metadata"]["counters_scope"],
        "live counters describe this cut, cumulative counters describe the whole capture",
        "the file must state which counter answers which question"
    );
}

/// Objective: Verify concurrent observations of one block produce exactly one allocation
/// event (plan A06 follow-up).
/// Invariants: 24 threads observing the same buffer yield one `Allocate` for its address.
#[test]
fn concurrent_observations_emit_one_allocation_event() {
    let tracker = Arc::new(Tracker::new());
    let data = Arc::new(vec![4u8; 64]);
    let ptr = data.as_ptr() as usize;

    let handles: Vec<_> = (0..24)
        .map(|_| {
            let tracker = Arc::clone(&tracker);
            let data = Arc::clone(&data);
            std::thread::spawn(move || {
                tracker.track_as(&*data, "shared", file!(), line!(), module_path!());
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("observing thread must not panic");
    }

    let events = tracker
        .events()
        .iter()
        .filter(|event| event.ptr == ptr && matches!(event.event_type, MemoryEventType::Allocate))
        .count();
    assert_eq!(
        events, 1,
        "concurrent observers of one block must produce exactly one allocation event, got {events}"
    );
}

/// Objective: Verify the `with_auto_export` drop path reports cumulative totals, not the
/// live-list aggregate (plan A06 follow-up).
/// Invariants: after 64 + 128 allocated and 128 released, the exported thread totals are
/// 192 allocated / 192 peak / 64 live.
#[test]
#[serial]
fn auto_export_on_drop_reports_ledger_totals() {
    let dir = TempDir::new().expect("tempdir must be creatable");
    let path = dir.path().to_str().expect("tempdir path must be UTF-8");
    let tracker = Tracker::new().with_auto_export(path);

    let kept = vec![1u8; 64];
    let released = vec![2u8; 128];
    tracker.track_as(&kept, "kept", file!(), line!(), module_path!());
    tracker.track_as(&released, "released", file!(), line!(), module_path!());
    tracker
        .track_deallocation(released.as_ptr() as usize)
        .expect("release must succeed");

    drop(tracker);

    let thread_json = std::fs::read_to_string(dir.path().join("thread_analysis.json"))
        .expect("the drop export must write thread_analysis.json");
    let thread_data: serde_json::Value =
        serde_json::from_str(&thread_json).expect("thread_analysis.json must be valid JSON");
    let totals = &thread_data["thread_analysis"][0];
    assert_eq!(
        totals["total_allocated"], 192,
        "cumulative allocated bytes must survive the release"
    );
    assert_eq!(
        totals["peak_memory"], 192,
        "the high-water mark must survive the release"
    );
    assert_eq!(
        totals["current_memory"], 64,
        "only the unreleased block is live"
    );
}
