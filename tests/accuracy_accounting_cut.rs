//! Accounting must describe the supplied event cut, including its charge evidence.

use memscope_rs::core::types::TrackKind;
use memscope_rs::render_engine::export::export_task_graph_json_with_accounting;
use memscope_rs::task_registry::{global_registry, ChargeOutcome, TaskIdRegistry};
use memscope_rs::tracker::Tracker;
use memscope_rs::Trackable;

struct Block(usize, usize);
impl Trackable for Block {
    fn track_kind(&self) -> TrackKind {
        TrackKind::HeapOwner {
            ptr: self.0,
            size: self.1,
        }
    }
    fn get_type_name(&self) -> &'static str {
        "AccountingBlock"
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

/// Objective: Re-export an immutable capture after unrelated charges and registry reset.
/// Invariants: all accounting fields and per-task gap rows stay identical to the first export.
#[test]
fn accounting_quality_is_frozen_with_the_supplied_events() {
    let registry = global_registry();
    let tracker = Tracker::new();
    let scope = registry.task_scope("captured-task");
    let task_id = TaskIdRegistry::current_task_id().expect("captured task context");
    memscope_rs::track!(tracker, Block(0x3322_0000, 64));
    drop(scope);
    let events = tracker.events();
    let allocation = events
        .iter()
        .find(|event| event.ptr == 0x3322_0000)
        .expect("typed allocation event");
    assert_eq!(
        allocation.charge_outcome,
        Some(ChargeOutcome::Charged),
        "publish charge outcome with allocation"
    );
    let dir = tempfile::tempdir().expect("create report directory");
    let export = |events: &[_]| {
        export_task_graph_json_with_accounting(dir.path(), events).expect("export task accounting");
        let value: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.path().join("task_graph.json")).expect("read task report"),
        )
        .expect("parse task report");
        value
    };
    let before = export(&events);
    assert_eq!(
        before["accounting"]["charge_quality"]["complete"], true,
        "the captured charge succeeded"
    );
    assert_eq!(
        registry.try_record_allocation(4096),
        ChargeOutcome::Unattributed,
        "later charge has no task"
    );
    assert_eq!(
        export(&events),
        before,
        "a later unrelated gap must not alter the earlier cut"
    );

    // A deferred task and a legacy task must remain visible even after the live
    // registry is cleared. Their evidence now resides in the immutable event slice.
    let mut degraded = events.clone();
    degraded[0].charge_outcome = Some(ChargeOutcome::Deferred);
    let mut legacy = memscope_rs::event_store::MemoryEvent::allocate(0x3323_0000, 32, 1);
    legacy.task_id = Some(task_id + 1);
    degraded.push(legacy);
    let gaps = export(&degraded);
    assert_eq!(
        gaps["accounting"]["charge_quality"]["deferred_bytes"], 64,
        "the gap bytes follow the captured allocation"
    );
    assert_eq!(
        gaps["accounting"]["charge_quality"]["unknown_bytes"], 32,
        "missing evidence must remain explicit"
    );
    for task in gaps["accounting"]["tasks"]
        .as_array()
        .expect("per-task rows")
    {
        assert_eq!(
            task["lower_bound"], true,
            "both affected tasks require lower bounds"
        );
    }
    registry.clear();
    assert_eq!(
        export(&events),
        before,
        "clearing the live cache must not erase successful evidence"
    );
    assert_eq!(
        export(&degraded),
        gaps,
        "clearing the live cache must not erase gap evidence"
    );
}
