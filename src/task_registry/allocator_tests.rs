//! Process-isolated checks for registry snapshots reached from the global allocator.

use super::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const CHILD_ENV: &str = "MEMSCOPE_DEFERRED_SNAPSHOT_CHILD";
const CHILD_TEST: &str = "task_registry::allocator_tests::deferred_snapshot_child";

/// Objective: Bound an allocator deadlock without hanging the rest of the test suite.
/// Invariants: the child runs the workload, exits successfully and is always reaped.
#[test]
fn deferred_snapshot_does_not_reenter_its_shard() {
    let dir = tempfile::tempdir().expect("create allocator regression artifacts");
    let log_path = dir.path().join("child.log");
    let log = std::fs::File::create(&log_path).expect("create child output log");
    let completed = dir.path().join("completed");
    let mut child = Command::new(std::env::current_exe().expect("locate library test binary"))
        .args(["--ignored", "--exact", CHILD_TEST, "--test-threads=1"])
        .env(CHILD_ENV, &completed)
        .stdin(Stdio::null())
        .stdout(log.try_clone().expect("clone child log handle"))
        .stderr(log)
        .spawn()
        .expect("spawn allocator regression child");

    let status = match child.wait_timeout(Duration::from_secs(15)) {
        Ok(Some(status)) => status,
        result => {
            let _ = child.kill();
            let _ = child.wait();
            let output = std::fs::read_to_string(&log_path).unwrap_or_default();
            panic!("deferred snapshot did not finish: {result:?}\n{output}");
        }
    };
    let output = std::fs::read_to_string(log_path).expect("read child diagnostics");
    assert!(
        status.success(),
        "allocator child failed: {status}\n{output}"
    );
    assert!(
        completed.is_file(),
        "the child filter must execute the snapshot workload"
    );
}

/// Objective: Exercise collection while the task registry cannot accept allocator charges.
/// Invariants: taking the snapshot neither deadlocks nor creates more deferred charges.
#[test]
#[ignore = "launched in a fresh process by deferred_snapshot_does_not_reenter_its_shard"]
fn deferred_snapshot_child() {
    let Some(completed) = std::env::var_os(CHILD_ENV) else {
        return;
    };
    let registry = global_registry();
    let scope = registry.task_scope("deferred-snapshot");
    let task_id = TaskIdRegistry::current_task_id().expect("install task context");
    let reader = registry.tasks.read().expect("hold registry read lock");

    // Seed a gap through the real allocator, whose suppression guard makes inserting
    // the first row safe. Collection must also suppress its own HashMap allocation.
    let data = vec![1u8; 64];
    std::hint::black_box(&data);
    let before = registry.charge_quality();
    let gaps = registry.deferred_tasks();
    let after = registry.charge_quality();
    let events = crate::tracker::Tracker::global().events();
    let allocation = events
        .iter()
        .find(|event| {
            event.ptr == data.as_ptr() as usize
                && event.event_type == crate::event_store::MemoryEventType::Allocate
        })
        .expect("real allocator publishes the deferred block");
    assert_eq!(
        allocation.charge_outcome,
        Some(ChargeOutcome::Deferred),
        "deferred evidence must be captured alongside the allocation"
    );
    assert_eq!(
        after, before,
        "snapshot allocations must not change charge quality"
    );
    assert!(
        gaps.contains_key(&task_id),
        "the deferred allocation must retain its task"
    );
    assert!(
        gaps[&task_id].1 >= 64,
        "the gap must include the seeded block's bytes"
    );

    drop(reader);
    drop(scope);
    std::fs::write(completed, b"completed").expect("publish child completion marker");
}
