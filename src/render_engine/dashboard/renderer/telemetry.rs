//! Report metrics whose populations and measurement boundaries are explicit.
//!
//! Allocation intervals use recorded Allocate events, grouped by origin thread.
//! Lifetime distributions use matched ledger releases and a single capture cut.
//! Runtime measurements without instrumentation are represented as unavailable.

use super::types::*;
use crate::event_store::{MemoryEvent, MemoryEventType};
use crate::snapshot::BindingLedger;
use crate::task_registry::TaskGraph;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct AllocationIntervals {
    pub mean_ms: Option<f64>,
    pub histogram: Vec<NeighborDensityBin>,
    pub sample_count: usize,
    pub non_dominant_share_pct: Option<f64>,
}

pub(super) fn allocation_intervals(events: &[MemoryEvent]) -> AllocationIntervals {
    let mut threads: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for event in events
        .iter()
        .filter(|e| e.event_type == MemoryEventType::Allocate)
    {
        threads
            .entry(event.thread_id)
            .or_default()
            .push(event.timestamp);
    }
    let total: usize = threads.values().map(Vec::len).sum();
    let dominant = threads.values().map(Vec::len).max().unwrap_or(0);
    let mut intervals = Vec::new();
    for times in threads.values_mut() {
        times.sort_unstable();
        intervals.extend(times.windows(2).map(|w| (w[1] - w[0]) as f64 / 1_000_000.0));
    }
    AllocationIntervals {
        mean_ms: mean(&intervals),
        histogram: histogram(intervals.iter().copied()),
        sample_count: intervals.len(),
        non_dominant_share_pct: (total > 0)
            .then(|| (total - dominant) as f64 * 100.0 / total as f64),
    }
}

/// Exact boundaries in milliseconds; fractional milliseconds never round into zero.
fn histogram(values: impl Iterator<Item = f64>) -> Vec<NeighborDensityBin> {
    let edges = [
        0.0,
        1.0,
        5.0,
        25.0,
        100.0,
        500.0,
        1000.0,
        5000.0,
        f64::INFINITY,
    ];
    let labels = [
        "0ms",
        "(0,1]ms",
        "(1,5]ms",
        "(5,25]ms",
        "(25,100]ms",
        "(100,500]ms",
        "(500,1000]ms",
        "(1,5]s",
        ">5s",
    ];
    let mut counts = [0; 9];
    for value in values.filter(|v| v.is_finite() && *v >= 0.0) {
        if let Some(index) = edges.iter().position(|edge| value <= *edge) {
            counts[index] += 1;
        }
    }
    counts
        .into_iter()
        .zip(labels)
        .map(|(count, label)| NeighborDensityBin {
            count,
            range_label: label.to_string(),
        })
        .collect()
}

pub(super) fn lifetimes(
    ledger: &BindingLedger,
    capture_at_ns: u64,
) -> AllocationLifetimeDistributions {
    let mut invalid_timestamps = 0;
    let released = histogram(ledger.released().iter().filter_map(|closed| {
        let delta = closed
            .deallocated_at
            .checked_sub(closed.binding.allocated_at);
        if delta.is_none() || closed.deallocated_at > capture_at_ns {
            invalid_timestamps += 1;
            return None;
        }
        delta.map(|ns| ns as f64 / 1_000_000.0)
    }));
    let live = histogram(ledger.live().iter().filter_map(|binding| {
        let delta = capture_at_ns.checked_sub(binding.allocated_at);
        if delta.is_none() {
            invalid_timestamps += 1;
        }
        delta.map(|ns| ns as f64 / 1_000_000.0)
    }));
    AllocationLifetimeDistributions {
        capture_at_ns,
        released,
        live,
        invalid_timestamps,
    }
}

pub(super) fn mean(samples: &[f64]) -> Option<f64> {
    (!samples.is_empty()).then(|| samples.iter().sum::<f64>() / samples.len() as f64)
}

/// A task duration is available only for an observed completion, including zero duration.
pub(super) fn task_durations(tasks: &[AsyncTaskInfo]) -> Vec<f64> {
    tasks
        .iter()
        .filter_map(|task| {
            task.completed_at_ms?
                .checked_sub(task.created_at_ms)
                .map(|ms| ms as f64)
        })
        .collect()
}

/// Layout may be illustrative; identities, status, and edges must come from the registry.
pub(super) fn task_topology(graph: &TaskGraph) -> (Vec<TaskTopologyNode>, Vec<TaskTopologyEdge>) {
    let mut tasks: Vec<_> = graph.nodes.iter().collect();
    tasks.sort_by_key(|task| task.id);
    let rows = tasks.len().div_ceil(6).max(1);
    let nodes = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| TaskTopologyNode {
            task_id: task.id.to_string(),
            name: task.name.clone(),
            parent_id: graph
                .edges
                .iter()
                .find(|e| e.to == task.id)
                .map(|e| e.from.to_string()),
            status: match task.status {
                crate::task_registry::TaskStatus::Completed => "COMPLETED",
                crate::task_registry::TaskStatus::Running => "EXECUTION_UNKNOWN",
            }
            .into(),
            // The task registry does not record completion timestamps. Its IDs are not
            // interchangeable with IDs allocated by the separate AsyncTracker.
            duration_ms: None,
            x_pct: 8.0 + (i % 6) as f64 * 16.0,
            y_pct: 10.0 + (i / 6) as f64 * 80.0 / rows as f64,
        })
        .collect();
    let edges = graph
        .edges
        .iter()
        .map(|edge| TaskTopologyEdge {
            source: edge.from.to_string(),
            target: edge.to.to_string(),
            // A registered parent-child relation does not imply observed execution.
            is_active: None,
        })
        .collect();
    (nodes, edges)
}

/// Sort numeric epoch timestamps before formatting, preserving order across midnight.
pub(super) fn trace_logs(tasks: &[AsyncTaskInfo], reports: &[UnsafeReport]) -> Vec<TraceLogEntry> {
    let mut entries: Vec<(u64, String, String)> = Vec::new();
    for task in tasks {
        if let Some(timestamp) = task.created_at_ms.checked_mul(1_000_000) {
            entries.push((
                timestamp,
                "INFO".into(),
                format!("task {} ({}) created", task.task_id, task.task_name),
            ));
        }
        if let Some(timestamp) = task
            .completed_at_ms
            .and_then(|ms| ms.checked_mul(1_000_000))
        {
            entries.push((
                timestamp,
                "INFO".into(),
                format!("task {} ({}) completed", task.task_id, task.task_name),
            ));
        }
    }
    for report in reports {
        entries.push((
            report.created_at,
            "INFO".into(),
            format!(
                "passport {} ({}) created · {}B · ptr={}",
                report.var_name, report.type_name, report.size_bytes, report.allocation_ptr
            ),
        ));
    }
    entries.sort_by_key(|entry| entry.0);
    entries
        .into_iter()
        .map(|(timestamp_ns, level, message)| TraceLogEntry {
            timestamp: format_ts_ns(timestamp_ns),
            timestamp_ns,
            level,
            message,
        })
        .collect()
}

pub(super) fn thread_event_log(
    events: &[MemoryEvent],
    tasks: &[AsyncTaskInfo],
) -> Vec<ThreadEventLogEntry> {
    let mut entries: Vec<_> = events
        .iter()
        .map(|event| {
            let stack = event
                .source_file
                .as_ref()
                .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)))
                .into_iter()
                .collect();
            (
                event.timestamp,
                format!(
                    "Thread {} {:?} · {}B · ptr={:#x}",
                    event.thread_id, event.event_type, event.size, event.ptr
                ),
                stack,
            )
        })
        .collect();
    entries.extend(
        trace_logs(tasks, &[])
            .into_iter()
            .map(|trace| (trace.timestamp_ns, trace.message, Vec::new())),
    );
    entries.sort_by_key(|entry| entry.0);
    entries
        .into_iter()
        .map(
            |(timestamp_ns, message, stack_traces)| ThreadEventLogEntry {
                time: format_ts_ns(timestamp_ns),
                timestamp_ns,
                level: "INFO".into(),
                message,
                stack_traces,
            },
        )
        .collect()
}

fn format_ts_ns(ns: u64) -> String {
    let seconds = ns / 1_000_000_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60,
        ns / 1_000_000 % 1000
    )
}

#[cfg(test)]
mod tests;
