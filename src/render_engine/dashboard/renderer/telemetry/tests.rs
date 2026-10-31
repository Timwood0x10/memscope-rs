//! Regression checks for observed populations and absent instrumentation.

use super::*;

/// Objective: Do not confuse cross-thread ordering or release history with intervals.
/// Invariants: same-thread zero intervals count; all recorded allocations count toward share.
#[test]
fn intervals_use_full_thread_grouped_event_history() {
    let mut events = Vec::new();
    for (thread, timestamp) in [(1, 0), (2, 1_000_000), (1, 4_000_000), (1, 4_000_000)] {
        let mut event = MemoryEvent::allocate(4096, 64, thread);
        event.timestamp = timestamp;
        events.push(event);
    }
    events.reverse();
    events.push(MemoryEvent::deallocate(4096, 64, 3));
    let intervals = allocation_intervals(&events);
    assert_eq!(
        intervals.mean_ms,
        Some(2.0),
        "only same-thread gaps of 4ms and 0ms contribute"
    );
    assert_eq!(intervals.sample_count, 2, "zero intervals are real samples");
    assert_eq!(
        intervals.non_dominant_share_pct,
        Some(25.0),
        "releases must not affect allocation share"
    );
    assert_eq!(
        allocation_intervals(&[]).mean_ms,
        None,
        "absence is not measured zero"
    );
}

/// Objective: Separate completed lifetime from censored age and reject inverted timestamps.
/// Invariants: one release remains visible after closing; live objects do not enter its histogram.
#[test]
fn lifetimes_keep_released_history_and_live_age_separate() {
    let mut ledger = BindingLedger::new();
    for ptr in [1, 2, 3] {
        let mut event = MemoryEvent::allocate(ptr, 16, 1);
        event.timestamp = 1_000_000;
        ledger.apply(&event);
    }
    for (ptr, timestamp) in [(1, 1_500_000), (3, 0)] {
        let mut event = MemoryEvent::deallocate(ptr, 16, 1);
        event.timestamp = timestamp;
        ledger.apply(&event);
    }
    let distribution = lifetimes(&ledger, 10_000_000);
    assert_eq!(
        distribution.released.iter().map(|b| b.count).sum::<usize>(),
        1,
        "only valid matched releases count"
    );
    assert_eq!(
        distribution.released[1].count, 1,
        "fractional milliseconds must not round to zero"
    );
    assert_eq!(
        distribution.live[3].count, 1,
        "the live object has a censored age of 9ms"
    );
    assert_eq!(
        distribution.invalid_timestamps, 1,
        "inverted timestamps remain diagnostic"
    );
}

/// Objective: Empty registries must never invent a root or edges.
/// Invariants: no measured tasks implies an empty topology.
#[test]
fn empty_topology_contains_no_synthetic_root() {
    let graph = serde_json::from_value(serde_json::json!({"nodes": [], "edges": []}))
        .expect("empty graph fixture");
    let (nodes, edges) = task_topology(&graph);
    assert!(
        nodes.is_empty() && edges.is_empty(),
        "empty registry must produce empty topology"
    );
}

fn task(id: u64, created: u64, completed: Option<u64>) -> AsyncTaskInfo {
    AsyncTaskInfo {
        task_id: id,
        created_at_ms: created,
        completed_at_ms: completed,
        task_name: format!("task_{id}"),
        task_type: "async".into(),
        total_bytes: 0,
        current_memory: 0,
        peak_memory: 0,
        total_allocations: 0,
        duration_ms: 0.0,
        efficiency_score: 0.0,
        is_completed: completed.is_some(),
        has_potential_leak: false,
        status: String::new(),
    }
}

/// Objective: Lifecycle labels and ordering must follow timestamps, even across midnight.
/// Invariants: huge IDs are identifiers only; running tasks do not invent yield or completion.
#[test]
fn traces_preserve_event_meaning_and_epoch_order() {
    let tasks = [
        task(u64::MAX, 86_399_000, Some(86_401_000)),
        task(1, 86_400_000, None),
    ];
    let traces = trace_logs(&tasks, &[]);
    assert_eq!(
        traces.iter().map(|t| t.timestamp_ns).collect::<Vec<_>>(),
        vec![86_399_000_000_000, 86_400_000_000_000, 86_401_000_000_000],
        "sort by epoch, never clock text or task ID"
    );
    assert!(
        traces[0].message.ends_with("created") && traces[2].message.ends_with("completed"),
        "timestamp must match the event meaning"
    );
    assert_eq!(
        thread_event_log(&[], &tasks).len(),
        3,
        "both trace projections retain real lifecycle observations"
    );
    assert_eq!(
        task_durations(&tasks),
        vec![2000.0],
        "running tasks are censored, not completed duration samples"
    );
    assert_eq!(
        task_durations(&[task(2, 10, Some(10))]),
        vec![0.0],
        "observed zero duration is valid"
    );
}

/// Objective: Preserve real registry IDs and deep parent-child edges without truncation.
/// Invariants: layout changes cannot fabricate a root or connect grandchildren to row one.
#[test]
fn topology_retains_recorded_deep_edges() {
    use crate::task_registry::{TaskEdge, TaskNode, TaskStatus};
    let graph = TaskGraph {
        nodes: (100..130)
            .map(|id| TaskNode {
                id,
                name: id.to_string(),
                memory_usage: 0,
                allocation_count: 0,
                status: TaskStatus::Running,
            })
            .collect(),
        edges: (100..129)
            .map(|id| TaskEdge {
                from: id,
                to: id + 1,
            })
            .collect(),
    };
    let (nodes, edges) = task_topology(&graph);
    assert_eq!(
        nodes.len(),
        30,
        "topology must retain nodes past the old 24-task cap"
    );
    assert_eq!(
        nodes[29].parent_id.as_deref(),
        Some("128"),
        "deep nodes must retain their registered parent"
    );
    assert_eq!(
        (&edges[28].source, &edges[28].target),
        (&"128".to_string(), &"129".to_string()),
        "edge identities cannot be positional aliases"
    );
    assert!(
        edges.iter().all(|edge| edge.is_active.is_none()),
        "relationships do not measure edge activity"
    );
}
