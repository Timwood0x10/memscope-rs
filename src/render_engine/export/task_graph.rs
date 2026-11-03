//! Task graphs and accounting projected from explicit, immutable inputs.

use crate::core::{MemScopeError, MemScopeResult};
use crate::event_store::MemoryEvent;
use crate::task_registry::{self, TaskGraph, TaskIdRegistry, TaskLifecycleQuality};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

fn lifecycle_json(quality: TaskLifecycleQuality) -> Value {
    let mut value = json!(quality);
    value["complete"] = json!(quality.is_complete());
    value
}

/// Export the current registry graph. Use the snapshot API for a replayable report.
pub fn export_task_graph_json<P: AsRef<Path>>(base_path: P) -> MemScopeResult<()> {
    let graph = task_registry::global_registry().export_graph();
    write_graph(base_path.as_ref(), &json!(graph))
}

/// Export only facts in the supplied events.
///
/// Events contain task IDs but not names, status or parent relationships. Those fields
/// remain unknown unless the caller supplies a captured graph to the snapshot API.
pub fn export_task_graph_json_with_accounting<P: AsRef<Path>>(
    base_path: P,
    events: &[MemoryEvent],
) -> MemScopeResult<()> {
    export_task_graph_snapshot(base_path, events, None)
}

/// Export a fixed event cut and optional task metadata captured with that report.
/// All byte counters come from events; registry counters are never used as a fallback.
pub fn export_task_graph_snapshot<P: AsRef<Path>>(
    base_path: P,
    events: &[MemoryEvent],
    graph: Option<&TaskGraph>,
) -> MemScopeResult<()> {
    let accounting = TaskIdRegistry::task_accounting_from_events(events);
    let (charge, task_charges) = TaskIdRegistry::charge_quality_from_events(events);
    let mut nodes = BTreeMap::new();
    let mut metadata_complete = graph.is_some();
    if let Some(graph) = graph {
        for node in &graph.nodes {
            let mut node = json!(node);
            node["metadata_source"] = json!("task_graph_snapshot");
            nodes.insert(
                node["id"].as_u64().expect("serialized task ID is u64"),
                node,
            );
        }
    }
    for task in events.iter().filter_map(|event| event.task_id) {
        nodes.entry(task).or_insert_with(|| {
            metadata_complete = false;
            json!({
                "id": task, "name": null, "status": null, "metadata_source": "unavailable",
            })
        });
    }
    for (id, node) in &mut nodes {
        let stats = accounting.stats.get(id).copied().unwrap_or_default();
        node["memory_usage"] = json!(stats.live_bytes);
        node["allocation_count"] = json!(stats.allocation_count);
        node["stats_source"] = json!("event_snapshot");
    }
    let tasks: Vec<_> = nodes
        .keys()
        .map(|id| {
            let stats = accounting.stats.get(id).copied().unwrap_or_default();
            let charge = task_charges.get(id).copied().unwrap_or_default();
            let lifecycle = accounting
                .task_lifecycle_quality
                .get(id)
                .copied()
                .unwrap_or_default();
            let mut row = json!(stats);
            let fields = row
                .as_object_mut()
                .expect("serialized task stats are an object");
            fields.extend(
                json!(charge)
                    .as_object()
                    .expect("serialized quality is an object")
                    .clone(),
            );
            fields.insert("task_id".to_string(), json!(id));
            fields.insert("stats_source".to_string(), json!("event_snapshot"));
            fields.insert(
                "complete".to_string(),
                json!(charge.is_complete() && lifecycle.is_complete()),
            );
            // Compatibility flag: any gap means consumers must not treat the row as exact.
            // Lifecycle ambiguity can affect live bytes in either direction.
            fields.insert(
                "lower_bound".to_string(),
                json!(!charge.is_complete() || !lifecycle.is_complete()),
            );
            fields.insert(
                "live_bytes_exact".to_string(),
                json!(lifecycle.is_complete()),
            );
            fields.insert("lifecycle_quality".to_string(), lifecycle_json(lifecycle));
            row
        })
        .collect();
    let complete = charge.is_complete() && accounting.lifecycle_quality.is_complete();
    let mut charge_json = json!(charge);
    charge_json["complete"] = json!(charge.is_complete());
    let mut edges = graph.map_or_else(Vec::new, |graph| graph.edges.clone());
    edges.sort_by_key(|edge| (edge.from, edge.to));
    write_graph(
        base_path.as_ref(),
        &json!({
            "nodes": nodes.into_values().collect::<Vec<_>>(),
            "edges": edges,
            "metadata_complete": metadata_complete,
            "accounting": {
                "stats_source": "event_snapshot", "complete": complete, "lower_bound": !complete,
                "charge_quality": charge_json,
                "lifecycle_quality": lifecycle_json(accounting.lifecycle_quality), "tasks": tasks,
            },
        }),
    )
}

fn write_graph(base_path: &Path, value: &Value) -> MemScopeResult<()> {
    let content = serde_json::to_vec_pretty(value).map_err(|error| {
        MemScopeError::error("export", "export_task_graph_json", error.to_string())
    })?;
    std::fs::write(base_path.join("task_graph.json"), content).map_err(|error| {
        MemScopeError::error("export", "export_task_graph_json", error.to_string())
    })
}
