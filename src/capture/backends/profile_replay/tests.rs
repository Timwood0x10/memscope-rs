use super::*;

/// Objective: Completing between graph and metadata capture cannot split report status.
/// Invariants: The graph and replay share completion, while the timestamp stays frozen.
#[test]
fn completion_between_capture_reads_has_one_status() {
    let registry = crate::task_registry::global_registry();
    let scope = registry.task_scope("capture-completion-race");
    let id = crate::task_registry::TaskIdRegistry::current_task_id().unwrap();
    let mut graph = registry.export_graph();
    assert_eq!(
        graph.nodes.iter().find(|n| n.id == id).unwrap().status,
        TaskStatus::Running,
        "first graph read precedes completion"
    );
    drop(scope);
    let metadata = ProfileMetadata::capture(&graph, None);
    metadata.reconcile_graph(&mut graph);
    let cutoff = capture_cut_ns();
    let rebuilt = rebuild_at(&[], &graph, &metadata, cutoff);
    let profile = rebuilt.profiles.iter().find(|p| p.task_id == id).unwrap();
    assert_eq!(
        graph.nodes.iter().find(|n| n.id == id).unwrap().status,
        TaskStatus::Completed,
        "report graph uses the frozen lifecycle status"
    );
    assert!(
        profile.completed_at_ms.is_some(),
        "profile agrees with its graph node"
    );
    assert_eq!(
        profile.completed_at_ms,
        registry
            .get_task(id)
            .unwrap()
            .completed_at
            .map(|ns| ns / 1_000_000),
        "reconciliation preserves actual completion time"
    );
}

fn allocation(ptr: usize, serial: u64) -> MemoryEvent {
    let mut event = MemoryEvent::allocate(ptr, 128, 1);
    event.task_id = Some(7);
    event.binding_serial = Some(serial);
    event.timestamp = 1_000_000_000;
    event
}

fn release(ptr: usize, serial: u64) -> MemoryEvent {
    let mut event = MemoryEvent::deallocate(ptr, 128, 1);
    event.task_id = Some(7);
    event.binding_serial = Some(serial);
    event.timestamp = 6_000_000_000;
    event
}

/// Objective: Releases must close an original binding, never a task's other bytes.
/// Invariants: Unmatched and duplicate releases leave totals equal to live rows.
#[test]
fn unmatched_release_cannot_cancel_live_binding() {
    let graph = TaskGraph {
        nodes: vec![],
        edges: vec![],
    };
    let metadata = ProfileMetadata(HashMap::new());
    for (ptr, serial) in [(4096, 12), (8192, 11)] {
        let events = [allocation(4096, 11), release(ptr, serial)];
        let result = rebuild_at(&events, &graph, &metadata, 6_000_000_000);
        assert_eq!(
            result.current_memory, 128,
            "unmatched release must not reduce live bytes"
        );
        assert_eq!(
            result.profiles[0].total_deallocations, 0,
            "no binding was closed"
        );
        assert_eq!(
            result.live_allocations[0].size, 128,
            "live row must agree with summary"
        );
    }
    let events = [allocation(4096, 11), release(4096, 11), release(4096, 11)];
    let result = rebuild_at(&events, &graph, &metadata, 6_000_000_000);
    assert_eq!(
        result.current_memory, 0,
        "matched release must free the bytes"
    );
    assert_eq!(
        result.profiles[0].total_deallocations, 1,
        "duplicate release must not count twice"
    );
}

/// Objective: Freeze elapsed time independently of memory-event timing.
/// Invariants: Replaying a captured cut is stable; later releases cannot move completion.
#[test]
fn lifecycle_and_capture_times_are_independent_of_releases() {
    let graph = TaskGraph {
        nodes: vec![],
        edges: vec![],
    };
    let metadata = ProfileMetadata(HashMap::from([(
        7,
        LiveMeta {
            task_name: "worker".into(),
            created_at_ms: 1000,
            completed_at_ms: None,
            tokio_task_id: Some(99),
            thread_id_u64: Some(3),
            task_type: TaskType::Mixed,
        },
    )]));
    let events = [allocation(4096, 11)];
    for _ in 0..2 {
        let result = rebuild_at(&events, &graph, &metadata, 1_035_000_000);
        assert_eq!(
            result.profiles[0].duration_ns, 35_000_000,
            "use captured time, not last event or now"
        );
        assert_eq!(
            result.profiles[0].tokio_task_id,
            Some(99),
            "keep lifecycle identity"
        );
    }
    let mut completed = metadata;
    completed.0.get_mut(&7).unwrap().completed_at_ms = Some(1000);
    let result = rebuild_at(
        &[events[0].clone(), release(4096, 11)],
        &graph,
        &completed,
        6_000_000_000,
    );
    assert_eq!(
        result.profiles[0].duration_ns, 0,
        "same-millisecond completion is legitimate"
    );
    completed.0.get_mut(&7).unwrap().completed_at_ms = Some(1035);
    let result = rebuild_at(&[], &graph, &completed, 6_000_000_000);
    assert_eq!(
        result.profiles[0].duration_ns, 35_000_000,
        "zero-allocation tasks retain lifecycle duration"
    );
}

#[test]
fn completion_timestamp_is_idempotent() {
    let mut meta = crate::task_registry::TaskMeta::new(7, None, "scope".into());
    meta.mark_completed();
    let first = meta.completed_at;
    meta.mark_completed();
    assert!(
        first.is_some(),
        "completion must record an actual timestamp"
    );
    assert_eq!(
        meta.completed_at, first,
        "repeated completion cannot extend lifetime"
    );
}
