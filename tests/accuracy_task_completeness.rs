//! Registry charge success cannot certify a complete allocation lifecycle.

use memscope_rs::event_store::MemoryEvent;
use memscope_rs::render_engine::export;
use memscope_rs::task_registry::{ChargeOutcome, TaskGraph, TaskNode, TaskStatus};
use serde_json::Value;

fn allocation(ptr: usize, task: u64, serial: u64) -> MemoryEvent {
    let mut event = MemoryEvent::allocate(ptr, 64, 1);
    event.task_id = Some(task);
    event.binding_serial = Some(serial);
    event.charge_outcome = Some(ChargeOutcome::Charged);
    event
}

fn report(events: &[MemoryEvent]) -> Value {
    let dir = tempfile::tempdir().expect("create report directory");
    export::export_task_graph_json_with_accounting(dir.path(), events)
        .expect("export fixed events");
    serde_json::from_slice(&std::fs::read(dir.path().join("task_graph.json")).expect("read graph"))
        .expect("parse graph")
}

/// Objective: A capture starting at realloc or free cannot claim full lifecycle evidence.
/// Invariants: successful/absent registry charges do not hide unmatched operations.
#[test]
fn unmatched_operations_invalidate_lifecycle_completeness() {
    for (mut event, field) in [
        (
            MemoryEvent::reallocate_moved(4096, 8192, 64, 128, 1),
            "unmatched_reallocations",
        ),
        (
            MemoryEvent::deallocate(4096, 64, 1),
            "unmatched_deallocations",
        ),
    ] {
        event.task_id = Some(1);
        let value = report(&[event]);
        let accounting = &value["accounting"];
        assert_eq!(
            accounting["charge_quality"]["complete"], true,
            "no failed allocation charges"
        );
        assert_eq!(
            accounting["complete"], false,
            "unmatched lifecycle cannot be complete"
        );
        assert_eq!(
            accounting["lifecycle_quality"][field], 1,
            "expose the actual evidence gap"
        );
        assert_eq!(
            accounting["tasks"][0]["lower_bound"], true,
            "even release-only tasks remain visible"
        );
    }
}

/// Objective: An ambiguous cross-task release must invalidate all possible owners.
/// Invariants: charge success stays separate and actor/owner rows expose uncertainty.
#[test]
fn degraded_release_marks_actor_and_candidate_owners() {
    let mut release = MemoryEvent::deallocate(4096, 64, 1);
    release.task_id = Some(3);
    release.degraded_release = true;
    let value = report(&[allocation(4096, 1, 1), allocation(4096, 2, 2), release]);
    let accounting = &value["accounting"];
    assert_eq!(
        accounting["charge_quality"]["complete"], true,
        "both charges succeeded"
    );
    assert_eq!(
        accounting["complete"], false,
        "ambiguous ownership is not exact"
    );
    for task in accounting["tasks"].as_array().expect("task rows") {
        assert_eq!(
            task["lifecycle_quality"]["degraded_releases"], 1,
            "all candidates and actor affected"
        );
        assert_eq!(
            task["live_bytes_exact"], false,
            "cannot certify live-byte attribution"
        );
    }
}

/// Objective: Known cross-task free is exact and captured names survive registry changes.
/// Invariants: graph bytes agree with event accounting, never with stale registry counters.
#[test]
fn captured_metadata_uses_event_counters() {
    let mut release = MemoryEvent::deallocate(4096, 64, 1);
    release.task_id = Some(2);
    release.binding_serial = Some(1);
    let events = [allocation(4096, 1, 1), release];
    let graph = TaskGraph {
        nodes: vec![TaskNode {
            id: 1,
            name: "captured-name".to_string(),
            memory_usage: 999,
            allocation_count: 99,
            status: TaskStatus::Completed,
        }],
        edges: vec![],
    };
    let dir = tempfile::tempdir().expect("create report directory");
    export::export_task_graph_snapshot(dir.path(), &events, Some(&graph))
        .expect("export metadata cut");
    let value: Value = serde_json::from_slice(
        &std::fs::read(dir.path().join("task_graph.json")).expect("read graph"),
    )
    .expect("parse graph");
    assert_eq!(
        value["nodes"][0]["name"], "captured-name",
        "retain captured task metadata"
    );
    assert_eq!(
        value["metadata_complete"], false,
        "task 2 metadata was not supplied"
    );
    assert_eq!(
        value["nodes"][0]["memory_usage"], 0,
        "released bytes must leave origin task"
    );
    assert_eq!(value["nodes"][0]["allocation_count"], 1, "use event counts");
    assert_eq!(
        value["accounting"]["complete"], true,
        "identified cross-task release is exact"
    );
    assert_eq!(
        value["accounting"]["tasks"][1]["deallocated_bytes"], 64,
        "activity belongs to freeing task"
    );
}
