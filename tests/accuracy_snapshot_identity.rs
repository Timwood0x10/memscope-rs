//! Lossless snapshot projection for conflicting address observations.

use memscope_rs::event_store::MemoryEvent;
use memscope_rs::snapshot::builder::build_snapshot_from_events;
use memscope_rs::view::MemoryView;

/// Objective: Preserve both identities when capture missed a release at a reused address.
/// Invariants: snapshot rows, view rows and ledger totals conserve the same 192 bytes.
#[test]
fn overlapping_bindings_survive_projection_and_json_roundtrip() {
    let events: Vec<_> = [(1, 64), (2, 128)]
        .into_iter()
        .map(|(serial, size)| {
            let mut event = MemoryEvent::allocate(4096, size, serial);
            event.binding_serial = Some(serial);
            event.type_name = Some(format!("type_{serial}"));
            event
        })
        .collect();
    let snapshot = build_snapshot_from_events(&events);
    assert_eq!(
        snapshot.active_count(),
        2,
        "two identities share one address"
    );
    assert_eq!(
        snapshot.allocations().map(|a| a.size).sum::<usize>(),
        192,
        "projection must conserve ledger bytes"
    );
    let encoded = serde_json::to_string(&snapshot).expect("snapshot JSON serialization");
    let decoded: memscope_rs::snapshot::MemorySnapshot =
        serde_json::from_str(&encoded).expect("snapshot JSON deserialization");
    let bindings = decoded.active_bindings.expect("identity-bearing snapshot");
    assert_eq!(
        bindings
            .iter()
            .map(|b| b.binding_serial)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)],
        "JSON must retain exact serials"
    );
    let view = MemoryView::from_events(events);
    assert_eq!(
        view.allocations().len(),
        2,
        "view must iterate bindings rather than address keys"
    );
    assert_eq!(
        view.stats().type_count,
        2,
        "both binding types must contribute to view statistics"
    );
}

/// Objective: Dashboard counters and rows must use one event cut rather than the core cache.
/// Invariants: a release remains in the lifetime population, while only the surviving
/// binding contributes to live bytes, age samples and the active count.
#[test]
fn dashboard_counters_and_lifetimes_share_the_event_cut() {
    use memscope_rs::analysis::memory_passport_tracker::MemoryPassportTracker;
    use memscope_rs::render_engine::dashboard::DashboardRenderer;
    use memscope_rs::tracker::Tracker;
    use std::sync::Arc;

    let tracker = Tracker::new();
    // Feed the event store without populating the separate core cache. Mixing
    // tracker.analyze() with this event snapshot previously produced zero KPIs.
    let store = tracker.event_store();
    let start = memscope_rs::utils::current_timestamp_nanos() - 10_000_000;
    let mut allocated = MemoryEvent::allocate(4096, 64, 1);
    allocated.timestamp = start;
    store.record(allocated);
    let mut released = MemoryEvent::deallocate(4096, 64, 1);
    released.timestamp = start + 2_000_000;
    store.record(released);
    let mut live = MemoryEvent::allocate(8192, 128, 2);
    live.timestamp = start + 3_000_000;
    store.record(live);
    let renderer = DashboardRenderer::new().expect("dashboard templates load");
    let passports = Arc::new(MemoryPassportTracker::new(Default::default()));
    let context = renderer
        .build_context_from_tracker(&tracker, &passports)
        .expect("event-backed dashboard builds");
    assert_eq!(
        context.total_allocations, 2,
        "total count must come from the captured events"
    );
    assert_eq!(
        context.active_allocations, 1,
        "only one captured binding is live"
    );
    assert_eq!(
        context.allocations.iter().map(|a| a.size).sum::<usize>(),
        128,
        "dashboard rows must conserve captured live bytes"
    );
    assert_eq!(
        context
            .allocation_lifetimes
            .released
            .iter()
            .map(|bin| bin.count)
            .sum::<usize>(),
        1,
        "matched release remains available after it leaves the live set"
    );
    assert_eq!(
        context
            .allocation_lifetimes
            .live
            .iter()
            .map(|bin| bin.count)
            .sum::<usize>(),
        1,
        "live ages must not include completed lifetimes"
    );
    let age = context.allocations[0]
        .observed_age_ms
        .expect("valid timestamp yields age");
    assert_eq!(
        age,
        (context.allocation_lifetimes.capture_at_ns - (start + 3_000_000)) as f64 / 1_000_000.0,
        "row age and histogram use exactly the same capture timestamp"
    );
    let json: serde_json::Value =
        serde_json::from_str(&context.json_data).expect("embedded dashboard JSON");
    assert_eq!(
        json["active_allocations"], 1,
        "JSON KPI and rendered context must agree"
    );
}

/// Objective: Verify the JSON export keeps both identities that share an address
/// (a missed release) instead of exporting only the newest one.
/// Invariants: two rows, one per generation, each carrying its own serial.
#[test]
fn export_writes_one_row_per_live_identity() {
    let events: Vec<_> = [(1u64, 64usize), (2, 128)]
        .into_iter()
        .map(|(serial, size)| {
            let mut event = MemoryEvent::allocate(0x4000, size, serial);
            event.binding_serial = Some(serial);
            event
        })
        .collect();
    let snapshot = build_snapshot_from_events(&events);

    let dir = tempfile::TempDir::new().expect("tempdir must be creatable");
    memscope_rs::render_engine::export::export_snapshot_to_json(
        &snapshot,
        dir.path(),
        &memscope_rs::render_engine::export::ExportJsonOptions::default(),
    )
    .expect("snapshot export must succeed");

    let json = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("memory_analysis.json must exist");
    let data: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let rows = data["allocations"]
        .as_array()
        .expect("allocations must be an array");

    assert_eq!(rows.len(), 2, "both identities must be exported");
    let serials: Vec<_> = rows
        .iter()
        .map(|row| row["binding_serial"].as_u64())
        .collect();
    assert_eq!(
        serials,
        vec![Some(1), Some(2)],
        "each row must keep the serial of its own binding"
    );
    assert_eq!(
        data["statistics"]["live_size_bytes"], 192,
        "the live scope must cover both identities"
    );
}

/// Objective: Verify a snapshot without transition history does not claim cumulative
/// counters (it only knows what is live).
/// Invariants: the cumulative keys are null and the scope states why.
#[test]
fn snapshot_without_history_reports_no_cumulative_counters() {
    let allocation = memscope_rs::capture::types::AllocationInfo::new(0x5000, 96);
    let snapshot = memscope_rs::snapshot::MemorySnapshot::from_allocation_infos(vec![allocation]);
    assert!(
        snapshot.active_bindings.is_none(),
        "this constructor builds an address-only snapshot"
    );

    let dir = tempfile::TempDir::new().expect("tempdir must be creatable");
    memscope_rs::render_engine::export::export_snapshot_to_json(
        &snapshot,
        dir.path(),
        &memscope_rs::render_engine::export::ExportJsonOptions::default(),
    )
    .expect("snapshot export must succeed");

    let json = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("memory_analysis.json must exist");
    let data: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    assert_eq!(
        data["statistics"]["live_size_bytes"], 96,
        "the live scope is still known"
    );
    assert!(
        data["statistics"]["cumulative_allocated_bytes"].is_null(),
        "cumulative bytes must not echo the live value"
    );
    assert_eq!(
        data["metadata"]["counters_scope"],
        "live only: this snapshot has no transition history, cumulative counters are unknown",
        "the file must say why the cumulative counters are missing"
    );
}

/// Objective: Verify container (stack-side) metadata is exported next to the heap rows
/// without being counted as heap bytes.
/// Invariants: one allocation row, one container row, counters only cover the heap block.
///
/// NOTE: this drives the writer directly instead of `export_all_json`. The batch entry
/// point feeds the recorded addresses into the relation inference, which dereferences
/// them; a synthesized address (as used here) is not readable memory, so the writer is
/// exercised without that step.
#[test]
fn container_metadata_is_exported_but_not_counted() {
    let events = vec![MemoryEvent::allocate(0x6000, 64, 1)];
    let snapshot = build_snapshot_from_events(&events);

    let mut container = memscope_rs::capture::types::AllocationInfo::new(0, 4);
    container.var_name = Some("frame".to_string());
    container.type_name = Some("i32".to_string());

    let dir = tempfile::TempDir::new().expect("tempdir must be creatable");
    memscope_rs::render_engine::export::export_snapshot_with_containers(
        &snapshot,
        &[container],
        dir.path(),
        &memscope_rs::render_engine::export::ExportJsonOptions::default(),
    )
    .expect("snapshot export must succeed");

    let json = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("memory_analysis.json must exist");
    let data: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    assert_eq!(
        data["allocations"]
            .as_array()
            .expect("allocations array")
            .len(),
        1,
        "only the heap binding belongs to the allocation rows"
    );
    let containers = data["containers"].as_array().expect("containers array");
    assert_eq!(containers.len(), 1, "the container metadata must be kept");
    assert_eq!(
        containers[0]["counted_in_statistics"], false,
        "containers must be marked as not counted"
    );
    assert_eq!(
        data["statistics"]["live_size_bytes"], 64,
        "container metadata must not inflate the heap counters"
    );
}

/// Objective: Verify a batch export with container (stack) metadata completes: container
/// rows carry synthetic addresses that must never reach the relation inference, which
/// dereferences what it is given.
/// Invariants: the export succeeds, containers are listed, and the heap counters ignore them.
#[test]
fn batch_export_with_container_metadata_does_not_dereference_synthetic_addresses() {
    let tracker = memscope_rs::tracker::Tracker::new();
    let payload = vec![9u8; 64];
    tracker.track_as(&payload, "payload", file!(), line!(), module_path!());
    let value: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    tracker.track_as(&value, "value", file!(), line!(), module_path!());

    let dir = tempfile::TempDir::new().expect("tempdir must be creatable");
    let passports = std::sync::Arc::new(
        memscope_rs::analysis::memory_passport_tracker::MemoryPassportTracker::default(),
    );
    let async_tracker =
        std::sync::Arc::new(memscope_rs::capture::backends::async_tracker::AsyncTracker::new());
    memscope_rs::render_engine::export::export_all_json(
        dir.path(),
        &tracker,
        &passports,
        &async_tracker,
    )
    .expect("a batch export with container metadata must succeed");

    let json = std::fs::read_to_string(dir.path().join("memory_analysis.json"))
        .expect("memory_analysis.json must exist");
    let data: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    assert!(
        !data["containers"]
            .as_array()
            .expect("containers array")
            .is_empty(),
        "the container observation must be exported as metadata"
    );
    assert_eq!(
        data["statistics"]["live_size_bytes"],
        payload.capacity(),
        "only the heap block contributes to the live counters"
    );
}
