//! Cross-module regressions for binding identity, task scopes and passport projections.

use memscope_rs::analysis::memory_passport_tracker::{MemoryPassportTracker, PassportStatus};
use memscope_rs::capture::backends::async_tracker::AsyncTracker;
use memscope_rs::core::types::TrackKind;
use memscope_rs::render_engine::dashboard::{rebuild_allocations_from_events, DashboardRenderer};
use memscope_rs::snapshot::BindingLedger;
use memscope_rs::tracker::Tracker;
use memscope_rs::Trackable;
use std::sync::Arc;

// Synthetic addresses describe capture observations; no memory is dereferenced.
struct Block(usize, usize);

impl Trackable for Block {
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: self.0,
            size: self.1,
        }
    }

    fn get_type_name(&self) -> &'static str {
        "Buffer"
    }
    fn get_size_estimate(&self) -> usize {
        self.1
    }
    fn get_data_ptr(&self) -> Option<usize> {
        Some(self.0)
    }
    fn get_data_size(&self) -> Option<usize> {
        Some(self.1)
    }
}

#[test]
fn same_size_reuse_keeps_annotation_and_release_on_the_new_binding() {
    let tracker = Tracker::new();
    tracker.track_as(&Block(0x11000, 64), "old", file!(), line!(), module_path!());
    // The old release was not captured; a previously unobserved block moves here.
    tracker.track_reallocation(0x12000, 0x11000, 16, 64);
    tracker.track_as(&Block(0x11000, 64), "new", file!(), line!(), module_path!());

    let allocations = rebuild_allocations_from_events(&tracker.events());
    assert_eq!(allocations.len(), 2);
    assert_eq!(allocations[0].var_name.as_deref(), Some("old"));
    assert_eq!(allocations[1].var_name.as_deref(), Some("new"));

    assert!(tracker.track_deallocation(0x11000).unwrap());
    let remaining = rebuild_allocations_from_events(&tracker.events());
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].var_name.as_deref(), Some("old"));
}

#[test]
fn concurrent_observations_and_reallocations_replay_as_one_binding() {
    let tracker = Tracker::new();
    tracker.track_as(
        &Block(0x21000, 64),
        "shared",
        file!(),
        line!(),
        module_path!(),
    );
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        for worker in 0..8 {
            let tracker = &tracker;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                for _ in 0..64 {
                    if worker == 0 {
                        tracker.track_reallocation(0x21000, 0x21000, 64, 64);
                    } else {
                        tracker.track_as(
                            &Block(0x21000, 64),
                            "shared",
                            file!(),
                            line!(),
                            module_path!(),
                        );
                    }
                }
            });
        }
    });
    tracker.track_deallocation(0x21000).unwrap();
    let mut ledger = BindingLedger::new();
    for event in tracker.events() {
        ledger.apply(&event);
    }
    assert!(
        ledger.live().is_empty(),
        "every observer refers to the same released block"
    );
    assert_eq!(ledger.stats().conflicting_allocations, 0);
    assert_eq!(
        ledger.stats().allocated_bytes,
        ledger.stats().deallocated_bytes
    );
}

#[test]
fn completed_external_task_is_not_restored_by_a_later_scope() {
    AsyncTracker::clear_current_task();
    let tracker = AsyncTracker::new();
    tracker
        .track_task_start(41, "parent".into(), std::thread::current().id())
        .unwrap();
    drop(AsyncTracker::enter_task(42));
    tracker.track_task_end(41).unwrap();
    drop(AsyncTracker::enter_task(43));
    assert_eq!(AsyncTracker::get_current_task(), None);
}

#[test]
fn explicit_clear_invalidates_the_context_under_a_guard() {
    AsyncTracker::set_current_task(41);
    let old = AsyncTracker::enter_task(42);
    AsyncTracker::clear_current_task();
    drop(AsyncTracker::enter_task(43));
    drop(old);
    assert_eq!(AsyncTracker::get_current_task(), None);
}

#[test]
fn moving_a_guard_does_not_remove_the_destination_threads_scope() {
    std::thread::spawn(|| {
        let source = AsyncTracker::enter_task(41);
        std::thread::spawn(move || {
            let destination = AsyncTracker::enter_task(42);
            drop(source);
            assert_eq!(AsyncTracker::get_current_task(), Some(42));
            drop(destination);
            assert_eq!(AsyncTracker::get_current_task(), None);
        })
        .join()
        .unwrap();
        drop(AsyncTracker::enter_task(43));
        assert_eq!(
            AsyncTracker::get_current_task(),
            None,
            "a guard dropped elsewhere has ended"
        );
    })
    .join()
    .unwrap();
}

#[test]
fn archived_passport_agrees_across_dashboard_statistics_and_json() {
    let tracker = Tracker::new();
    let passports = Arc::new(MemoryPassportTracker::default());
    let archived_id = passports
        .create_passport_simple(0x31000, 64, "old".into())
        .unwrap();
    passports
        .record_handover_to_ffi(0x31000, "ffi".into(), "consume".into())
        .unwrap();
    passports
        .create_passport_simple(0x31000, 128, "new".into())
        .unwrap();

    let context = DashboardRenderer::new()
        .unwrap()
        .build_context_from_tracker(&tracker, &passports)
        .unwrap();
    assert_eq!(context.passport_details.len(), 2);
    let archived = context
        .passport_details
        .iter()
        .find(|p| p.passport_id == archived_id)
        .unwrap();
    assert!(archived.is_leaked);
    assert!(archived.superseded);
    assert_eq!(archived.generation, 1);
    assert_eq!(context.leak_count, 1);
    let stats = passports.get_stats();
    assert_eq!(stats.leaks_detected, context.leak_count);
    assert_eq!(stats.passports_by_status.get("InForeignCustody"), Some(&1));
    assert_eq!(
        passports
            .get_passports_by_status(PassportStatus::InForeignCustody)
            .len(),
        1
    );

    let dir = tempfile::tempdir().unwrap();
    memscope_rs::render_engine::export::export_all_json(
        dir.path(),
        &tracker,
        &passports,
        &Arc::new(AsyncTracker::new()),
    )
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("memory_passports.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        json["memory_passports"].as_array().unwrap().len(),
        context.passport_details.len()
    );
    assert!(json["memory_passports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["passport_id"] == archived_id && p["superseded"] == true));
}

#[test]
fn reclaimed_memory_is_still_active_in_the_dashboard() {
    let tracker = Tracker::new();
    let passports = Arc::new(MemoryPassportTracker::default());
    passports
        .create_passport_simple(0x32000, 64, "rust".into())
        .unwrap();
    passports
        .record_handover_to_ffi(0x32000, "ffi".into(), "borrow".into())
        .unwrap();
    passports
        .record_reclaimed_by_rust(0x32000, "rust".into(), "return".into())
        .unwrap();
    let context = DashboardRenderer::new()
        .unwrap()
        .build_context_from_tracker(&tracker, &passports)
        .unwrap();
    assert_eq!(context.passport_details[0].status, "ReclaimedByRust");
    assert!(
        context.passport_details[0].is_active,
        "ownership returned to Rust; no release was observed"
    );
}

#[test]
fn legacy_events_deserialize_without_a_binding_serial() {
    let allocation: memscope_rs::event_store::MemoryEvent = serde_json::from_str(
        r#"{"timestamp":1,"event_type":"Allocate","ptr":4096,"size":64,"thread_id":1}"#,
    )
    .unwrap();
    assert_eq!(allocation.binding_serial, None);
    let mut ledger = BindingLedger::new();
    ledger.apply(&allocation);
    ledger.apply(&memscope_rs::event_store::MemoryEvent::deallocate(
        4096, 64, 1,
    ));
    assert!(ledger.live().is_empty());
}

#[test]
fn stale_serial_never_closes_a_same_size_reused_binding() {
    use memscope_rs::event_store::MemoryEvent;
    let mut first = MemoryEvent::allocate(0x41000, 64, 1);
    first.binding_serial = Some(1);
    let mut second = first.clone();
    second.binding_serial = Some(2);
    let mut released = MemoryEvent::deallocate(0x41000, 64, 1);
    released.binding_serial = Some(1);
    let events = vec![first, released.clone(), second, released];
    // Exercise the serialized stream as well as in-memory replay.
    let events: Vec<MemoryEvent> =
        serde_json::from_str(&serde_json::to_string(&events).unwrap()).unwrap();
    let mut ledger = BindingLedger::new();
    for event in events {
        ledger.apply(&event);
    }
    assert_eq!(ledger.live().len(), 1);
    assert_eq!(ledger.live()[0].binding_serial, Some(2));
    assert_eq!(ledger.stats().unmatched_deallocation_count, 1);
    assert_eq!(ledger.stats().current_bytes, 64);
}

#[test]
fn ambiguous_legacy_annotation_does_not_guess_a_generation() {
    use memscope_rs::event_store::MemoryEvent;
    let mut ledger = BindingLedger::new();
    ledger.apply(&MemoryEvent::allocate(0x42000, 64, 1));
    ledger.apply(&MemoryEvent::allocate(0x42000, 64, 1));
    let mut annotation = MemoryEvent::metadata("unknown_owner".into(), "Buffer".into(), 1, 64);
    annotation.ptr = 0x42000;
    ledger.apply(&annotation);
    assert!(ledger
        .live()
        .iter()
        .all(|binding| binding.var_name.is_none()));
    assert_eq!(ledger.stats().unmatched_metadata_count, 1);
}
