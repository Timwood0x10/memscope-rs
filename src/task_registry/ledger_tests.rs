//! Unit tests for the task registry.
//!
//! Split out of `task_registry.rs` to keep that file within the project line budget.

use super::*;
use crate::event_store::MemoryEvent;

/// Objective: Verify task accounting can be derived from the event stream alone, so a
/// report does not depend on registry lock timing (plan A08).
/// Invariants: allocation/release totals and live bytes follow the events per task.
#[test]
fn task_stats_are_aggregated_from_events() {
    let mut allocate = MemoryEvent::allocate(0x1000, 64, 1);
    allocate.task_id = Some(7);
    let mut release = MemoryEvent::deallocate(0x1000, 64, 1);
    release.task_id = Some(7);
    let mut other = MemoryEvent::allocate(0x2000, 32, 2);
    other.task_id = Some(8);
    let unattributed = MemoryEvent::allocate(0x3000, 16, 1);

    let stats = TaskIdRegistry::task_stats_from_events(&[allocate, release, other, unattributed]);

    let task7 = stats.get(&7).expect("task 7 must be present");
    assert_eq!(task7.allocation_count, 1, "one allocation for task 7");
    assert_eq!(task7.deallocation_count, 1, "one release for task 7");
    assert_eq!(task7.allocated_bytes, 64, "bytes of the allocation");
    assert_eq!(task7.deallocated_bytes, 64, "bytes of the release");
    assert_eq!(task7.live_bytes, 0, "the block was released");

    let task8 = stats.get(&8).expect("task 8 must be present");
    assert_eq!(task8.live_bytes, 32, "task 8 still holds its block");
    assert_eq!(
        stats.len(),
        2,
        "allocations without a task must not invent a task entry"
    );
}

/// Objective: Verify the completeness report distinguishes deferred and unattributed
/// charges instead of returning one boolean.
/// Invariants: a fresh registry is complete; unattributed charges are reported.
#[test]
fn charge_quality_separates_the_outcomes() {
    let registry = TaskIdRegistry::new();
    assert!(
        registry.charge_quality().is_complete(),
        "a registry that saw nothing is complete"
    );

    // A fresh thread has no task context: the charge must be reported as
    // unattributed instead of being silently dropped.
    let unattributed = std::thread::scope(|scope| {
        scope
            .spawn(|| registry.try_record_allocation(48))
            .join()
            .expect("the probe thread must not panic")
    });
    assert_eq!(
        unattributed,
        ChargeOutcome::Unattributed,
        "no task context means the block belongs to no task"
    );

    let quality = registry.charge_quality();
    assert_eq!(quality.unattributed_charges, 1, "the attempt is counted");
    assert_eq!(quality.unattributed_bytes, 48, "the bytes are counted");
    assert_eq!(quality.deferred_charges, 0, "nothing was deferred");
    assert!(!quality.is_complete(), "a gap must be visible");
}

/// Objective: Verify a deferred charge is attributed to the task that missed it, so a
/// report can mark *that* task as a lower bound (not only the whole window).
/// Invariants: the gap table has one entry of one charge / 64 bytes for the task.
#[test]
fn deferred_charge_is_attributed_to_the_task() {
    let registry = TaskIdRegistry::new();
    let _scope = registry.task_scope("deferred_task");
    let task_id = TaskIdRegistry::current_task_id().expect("task context");

    {
        // Hold the registry write lock so the charge has to defer.
        let _guard = registry.tasks.write().expect("registry lock");
        assert_eq!(
            registry.try_record_allocation(64),
            ChargeOutcome::Deferred,
            "a busy registry must defer instead of blocking"
        );
    }

    assert_eq!(
        registry.deferred_tasks().get(&task_id),
        Some(&(1, 64)),
        "the gap must be attributed to the task that missed the charge"
    );
    assert_eq!(
        registry.charge_quality().deferred_bytes,
        64,
        "the window counter and the per-task table must agree"
    );
}

/// Objective: Verify live bytes are settled on the task that allocated the binding, not
/// on whoever released it, and that a repeated release cannot subtract twice.
/// Invariants: the allocating task ends at zero live bytes, the releasing task books
/// the activity, and the second release changes nothing.
#[test]
fn release_settles_live_bytes_on_the_allocating_task() {
    let mut allocate = MemoryEvent::allocate(0x9000, 64, 1);
    allocate.task_id = Some(7);
    let mut release = MemoryEvent::deallocate(0x9000, 64, 1);
    release.task_id = Some(8);
    let mut repeated = release.clone();
    repeated.thread_id = 8;

    let stats = TaskIdRegistry::task_stats_from_events(&[allocate, release, repeated]);

    let owner = stats.get(&7).expect("allocating task");
    assert_eq!(
        owner.live_bytes, 0,
        "the allocating task must not keep bytes after the block was released"
    );
    assert_eq!(owner.allocation_count, 1);

    let actor = stats.get(&8).expect("releasing task");
    assert_eq!(
        actor.deallocated_bytes, 64,
        "only the release that closed a live binding may book bytes"
    );
    assert_eq!(
        actor.deallocation_count, 1,
        "a repeated release is not a valid lifecycle transition"
    );
    assert_eq!(
        actor.live_bytes, 0,
        "the releasing task never owns those bytes"
    );
}
