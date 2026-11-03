//! Re-exporting immutable inputs must not observe later clock or capture state.

use memscope_rs::analysis::borrow_analysis::{self, BorrowAnalyzer, BorrowType};
use memscope_rs::event_store::{EventStore, MemoryEvent};
use memscope_rs::render_engine::{dashboard, export};
use memscope_rs::snapshot::builder;
use serde_json::Value;
use std::path::Path;

fn read_json(path: &Path, name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(path.join(name)).expect("read exported file"))
        .expect("parse exported JSON")
}

/// Objective: Every batch uses snapshot time rather than the export wall clock.
/// Invariants: ages and all three JSON projections agree across serial/parallel export.
#[test]
fn snapshot_age_and_metadata_are_stable_across_batches() {
    let mut events = Vec::new();
    for index in 0..8 {
        let mut event = MemoryEvent::allocate(4096 + index * 128, 64, index as u64);
        event.timestamp = 1_000_000;
        events.push(event);
    }
    let mut snapshot = builder::build_snapshot_from_events(&events);
    snapshot.timestamp = 11_000_000;
    let dir = tempfile::tempdir().expect("create export directory");
    let serial = export::ExportJsonOptions {
        parallel_processing: false,
        ..Default::default()
    };
    export::export_snapshot_to_json(&snapshot, dir.path(), &serial).expect("serial export");
    let names = [
        "memory_analysis.json",
        "lifetime.json",
        "thread_analysis.json",
    ];
    let before: Vec<_> = names
        .iter()
        .map(|name| read_json(dir.path(), name))
        .collect();
    for row in before[0]["allocations"]
        .as_array()
        .expect("allocation rows")
    {
        assert_eq!(
            row["observed_age_ms"], 10,
            "age is measured at the snapshot cut"
        );
        assert!(
            row["lifetime_ms"].is_null(),
            "live object has no complete lifetime"
        );
    }
    let parallel = export::ExportJsonOptions {
        batch_size: 1,
        ..Default::default()
    };
    export::export_snapshot_to_json(&snapshot, dir.path(), &parallel).expect("parallel re-export");
    for (name, expected) in names.iter().zip(before) {
        assert_eq!(
            read_json(dir.path(), name),
            expected,
            "{name} must describe the same snapshot"
        );
    }
}

/// Objective: Ownership projection uses captured containers and borrow history.
/// Invariants: later store/global analyzer activity cannot change the old projection.
#[test]
fn ownership_projection_uses_only_explicit_snapshot_inputs() {
    let store = EventStore::new();
    let mut allocation = MemoryEvent::allocate(0x1200, 64, 1);
    allocation.type_name = Some("Vec<u8>".to_string());
    store.record(allocation);
    let events = store.snapshot();
    let allocations = dashboard::rebuild_allocations_from_events(&events);
    let borrows = BorrowAnalyzer::new();
    borrows.track_borrow(0x1200, BorrowType::Immutable, "captured-borrow");
    let history = borrows.get_borrow_history();
    let dir = tempfile::tempdir().expect("create export directory");
    let render = || {
        export::export_ownership_graph_snapshot(
            dir.path(),
            &allocations,
            &events,
            &history,
            11_000_000,
        )
        .expect("export captured ownership");
        read_json(dir.path(), "ownership_graph.json")
    };
    let before = render();
    assert_eq!(
        before["edges"][0]["var_name"], "captured-borrow",
        "retain captured borrow metadata"
    );
    store.record(MemoryEvent::metadata(
        "late-container".to_string(),
        "HashMap<u8, u8>".to_string(),
        1,
        48,
    ));
    let global = borrow_analysis::get_global_borrow_analyzer();
    let late = global.track_borrow(0x9900, BorrowType::Immutable, "late-borrow");
    assert_eq!(
        render(),
        before,
        "live state must not leak into the fixed projection"
    );
    global.end_borrow(late);
    assert_eq!(
        before["summary"]["total_edges"], 1,
        "summary includes captured borrow edges"
    );
}
