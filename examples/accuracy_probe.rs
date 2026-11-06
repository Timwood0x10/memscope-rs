//! Process-level acceptance probe for the public tracker entry points.
//!
//! Run with `--features tracking-allocator --example accuracy_probe -- start`
//! or replace `start` with `global` for the allocator-connected control. Each
//! invocation writes `accuracy_probe.json` and `probe_events.json` in its working
//! directory, then returns an error if any acceptance check fails.
//!
//! The workload performs 1,000 real allocations and drops across 50 threads.
//! Each block is observed twice: observations must deduplicate, but address reuse
//! must not merge separate lifetimes. No synthetic release events are injected.

use memscope_rs::event_store::MemoryEvent;
use memscope_rs::snapshot;
use memscope_rs::tracker::Tracker;
use memscope_rs::{AutoExportConfig, MemScopeConfig, MemScopeError, MemScopeResult, SignalPolicy};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Barrier};

const THREADS: usize = 50;
const BLOCKS_PER_THREAD: usize = 20;
const BLOCK_BYTES: usize = 4096;

fn probe_error(message: impl ToString) -> MemScopeError {
    MemScopeError::error("accuracy_probe", "acceptance", message.to_string())
}

fn write_json(path: &str, value: &Value) -> MemScopeResult<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(probe_error)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Select whole binding histories, including anonymous allocator events. Filtering
/// only named events would itself lose the real Allocate/Deallocate observations.
fn named_histories(events: &[MemoryEvent], names: &[&str]) -> Vec<MemoryEvent> {
    let serials: HashSet<_> = events
        .iter()
        .filter(|event| {
            event.module_path.as_deref() == Some(module_path!())
                && event
                    .var_name
                    .as_deref()
                    .is_some_and(|name| names.contains(&name))
        })
        .filter_map(|event| event.binding_serial)
        .collect();
    events
        .iter()
        .filter(|event| event.binding_serial.is_some_and(|id| serials.contains(&id)))
        .cloned()
        .collect()
}

fn check(checks: &mut Vec<Value>, name: &str, actual: usize, expected: usize) {
    checks.push(json!({
        "check": name,
        "actual": actual,
        "expected": expected,
        "passed": actual == expected,
    }));
}

fn main() -> MemScopeResult<()> {
    if !cfg!(feature = "tracking-allocator") {
        return Err(probe_error(
            "this probe requires --features tracking-allocator",
        ));
    }
    let entry = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "start".to_string());
    if !matches!(entry.as_str(), "start" | "global") {
        return Err(probe_error("expected entry point: start or global"));
    }
    // Export explicitly below so failing checks still leave complete evidence.
    let guard = memscope_rs::start_with(MemScopeConfig::default().with_auto_export(
        AutoExportConfig {
            on_exit: false,
            on_panic: false,
            on_signal: SignalPolicy::Off,
            ..Default::default()
        },
    ))?;
    let tracker = if entry == "global" {
        Tracker::global()
    } else {
        guard.tracker().tracker().clone()
    };
    let mut checks = Vec::new();

    // Live shared pointers must retain their names without counting their
    // pointees as additional allocations alongside the real allocator blocks.
    let probe_rc = Rc::new([1u8; 64]);
    let probe_arc = Arc::new([2u8; 64]);
    memscope_rs::track!(tracker, probe_rc);
    memscope_rs::track!(tracker, probe_arc);
    let live_events = tracker.events();
    let shared_events = named_histories(&live_events, &["probe_rc", "probe_arc"]);
    let shared = snapshot::build_snapshot_from_events(&shared_events);
    for name in ["probe_rc", "probe_arc"] {
        let count = shared
            .allocations()
            .filter(|row| row.var_name.as_deref() == Some(name))
            .count();
        check(&mut checks, name, count, 1);
    }
    check(
        &mut checks,
        "shared_pointer_blocks",
        shared.stats.total_allocations,
        2,
    );
    drop(probe_rc);
    drop(probe_arc);

    let barrier = Arc::new(Barrier::new(THREADS));
    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let tracker = tracker.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                for _ in 0..BLOCKS_PER_THREAD {
                    let probe_payload = Box::new([7u8; BLOCK_BYTES]);
                    memscope_rs::track!(tracker, probe_payload);
                    memscope_rs::track!(tracker, probe_payload);
                    std::hint::black_box(&probe_payload);
                    drop(probe_payload);
                }
            })
        })
        .collect();
    for handle in handles {
        handle
            .join()
            .map_err(|_| probe_error("allocation worker panicked"))?;
    }

    let events = tracker.events();
    let workload_events = named_histories(&events, &["probe_payload"]);
    let workload = snapshot::build_snapshot_from_events(&workload_events);
    let expected_blocks = THREADS * BLOCKS_PER_THREAD;
    let expected_bytes = expected_blocks * BLOCK_BYTES;
    check(
        &mut checks,
        "allocations",
        workload.stats.total_allocations,
        expected_blocks,
    );
    check(
        &mut checks,
        "deallocations",
        workload.stats.total_deallocations,
        expected_blocks,
    );
    check(
        &mut checks,
        "allocated_bytes",
        workload.stats.total_allocated,
        expected_bytes,
    );
    check(
        &mut checks,
        "deallocated_bytes",
        workload.stats.total_deallocated,
        expected_bytes,
    );
    check(
        &mut checks,
        "live_bytes_after_join",
        workload.stats.current_memory,
        0,
    );
    check(
        &mut checks,
        "live_bindings_after_join",
        workload.active_count(),
        0,
    );
    check(
        &mut checks,
        "unmatched_releases",
        workload.stats.unmatched_deallocations,
        0,
    );
    check(
        &mut checks,
        "ambiguous_releases",
        workload.stats.degraded_releases,
        0,
    );

    let failures = checks.iter().filter(|item| item["passed"] == false).count();
    write_json(
        "probe_events.json",
        &json!({
            "shared_while_live": shared_events,
            "workload_after_join": workload_events,
        }),
    )?;
    write_json(
        "accuracy_probe.json",
        &json!({
            "entry_point": entry,
            "threads": THREADS,
            "blocks_per_thread": BLOCKS_PER_THREAD,
            "block_bytes": BLOCK_BYTES,
            "workload_stats": workload.stats,
            "checks": checks,
            "failures": failures,
        }),
    )?;
    tracing::info!(
        entry,
        failures,
        "acceptance probe finished; see accuracy_probe.json"
    );
    if failures != 0 {
        return Err(probe_error(format!(
            "{failures} acceptance checks failed; see accuracy_probe.json"
        )));
    }
    Ok(())
}
