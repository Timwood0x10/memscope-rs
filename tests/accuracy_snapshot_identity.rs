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
