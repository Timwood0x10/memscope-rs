//! Deterministic acceptance probe for task attribution across an await point.
//!
//! Run `cargo run --example async_context_probe` in a fresh output directory.
//! Two independent tasks share one runtime thread. A channel ensures task B
//! finishes while task A is suspended, before A makes its second allocation.
//! Both allocations must belong to A, and B must remain a root task.
//! Results are written to `async_context_probe.json` before a failing exit.

use memscope_rs::capture::backends::async_tracker::{spawn_tracked, TrackerContext};
use memscope_rs::task_registry;
use memscope_rs::{MemScopeError, MemScopeResult};
use serde_json::{json, Value};
use tokio::sync::oneshot;

fn probe_error(message: impl ToString) -> MemScopeError {
    MemScopeError::error("async_context_probe", "acceptance", message.to_string())
}

fn check(checks: &mut Vec<Value>, name: &str, actual: Value, expected: Value) {
    checks.push(json!({
        "check": name,
        "passed": actual == expected,
        "actual": actual,
        "expected": expected,
    }));
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> MemScopeResult<()> {
    let guard = memscope_rs::start()?;
    let (ready_tx, ready_rx) = oneshot::channel();
    let (resume_tx, resume_rx) = oneshot::channel();

    let task_a = spawn_tracked(async move {
        let id = TrackerContext::capture()
            .task_id
            .ok_or_else(|| probe_error("task A has no task-local identity"))?;
        let tracker = memscope_rs::global_tracker()?;
        let before_yield = Box::new([1u8; 64]);
        memscope_rs::track!(tracker, before_yield);
        ready_tx
            .send(())
            .map_err(|_| probe_error("ready receiver closed"))?;
        resume_rx.await.map_err(probe_error)?;
        let after_yield = Box::new([2u8; 128]);
        memscope_rs::track!(tracker, after_yield);
        // Keep both allocations alive until their observations have been recorded.
        std::hint::black_box((&before_yield, &after_yield));
        Ok::<_, MemScopeError>(id)
    });

    ready_rx.await.map_err(probe_error)?;
    let task_b = spawn_tracked(async move {
        let id = TrackerContext::capture()
            .task_id
            .ok_or_else(|| probe_error("task B has no task-local identity"))?;
        resume_tx
            .send(())
            .map_err(|_| probe_error("resume receiver closed"))?;
        // No await after send: on this single-thread runtime B completes and
        // runs its cleanup before the newly woken A can be polled again.
        Ok::<_, MemScopeError>(id)
    });
    let b_id = task_b.await.map_err(probe_error)??;
    let a_id = task_a.await.map_err(probe_error)??;

    let events: Vec<_> = guard
        .tracker()
        .tracker()
        .events()
        .into_iter()
        .filter(|event| event.module_path.as_deref() == Some(module_path!()))
        .collect();
    let mut checks = Vec::new();
    for name in ["before_yield", "after_yield"] {
        let attributed: Vec<_> = events
            .iter()
            .filter(|event| event.var_name.as_deref() == Some(name))
            .map(|event| event.task_id)
            .collect();
        check(&mut checks, name, json!(attributed), json!([a_id]));
    }
    let registry = task_registry::global_registry();
    check(
        &mut checks,
        "task_a_is_root",
        json!(registry.get_parent(a_id)),
        Value::Null,
    );
    check(
        &mut checks,
        "task_b_is_root",
        json!(registry.get_parent(b_id)),
        Value::Null,
    );
    let profiles = guard.async_tracker().get_all_profiles();
    let a_bytes = profiles
        .iter()
        .find(|profile| profile.task_id == a_id)
        .map(|profile| profile.total_bytes);
    check(
        &mut checks,
        "task_a_profile_bytes",
        json!(a_bytes),
        json!(192),
    );

    let failures = checks.iter().filter(|item| item["passed"] == false).count();
    let evidence = json!({
        "runtime": "current_thread",
        "task_a": a_id,
        "task_b": b_id,
        "checks": checks,
        "failures": failures,
        "events": events,
    });
    std::fs::write(
        "async_context_probe.json",
        serde_json::to_vec_pretty(&evidence).map_err(probe_error)?,
    )?;
    tracing::info!(
        failures,
        "async context probe finished; see async_context_probe.json"
    );
    if failures != 0 {
        return Err(probe_error(format!(
            "{failures} acceptance checks failed; see async_context_probe.json"
        )));
    }
    Ok(())
}
