//! Unit tests for the task registry.
//!
//! Split out of `task_registry.rs` to keep that file within the project line budget.

use super::*;
use crate::event_store::MemoryEvent;

/// Objective: A late explicit release must not claim a same-size replacement generation.
/// Invariants: the replacement task keeps its live bytes and no second release is booked.
#[test]
fn stale_serial_does_not_consume_reused_address() {
    let mut old = MemoryEvent::allocate(0x1000, 64, 1);
    old.task_id = Some(7);
    old.binding_serial = Some(1);
    let mut release = MemoryEvent::deallocate(old.ptr, old.size, 1);
    release.task_id = Some(7);
    release.binding_serial = old.binding_serial;
    let mut new = old.clone();
    new.task_id = Some(8);
    new.binding_serial = Some(2);

    let stats = TaskIdRegistry::task_stats_from_events(&[old, release.clone(), new, release]);
    assert_eq!(
        stats[&8].live_bytes, 64,
        "G+1 must survive a late release of G"
    );
    assert_eq!(
        stats[&7].deallocation_count, 1,
        "the duplicate release must book nothing"
    );
}

/// Objective: Capture starting at a realloc must retain its observed new block.
/// Invariants: both moved and in-place reallocations book new bytes, never unseen old bytes.
#[test]
fn unmatched_realloc_opens_only_the_observed_side() {
    for new_ptr in [0, 0x2000] {
        let mut event = MemoryEvent::reallocate_moved(0, new_ptr, 64, 128, 1);
        event.task_id = Some(7);
        event.binding_serial = Some(99);
        let mut release = MemoryEvent::deallocate(new_ptr, 0, 2);
        release.task_id = Some(8);
        release.binding_serial = event.binding_serial;
        let stats = TaskIdRegistry::task_stats_from_events(&[event.clone()]);
        assert_eq!(
            stats[&7].live_bytes, 128,
            "the observed block must enter the live set"
        );
        assert_eq!(
            stats[&7].allocation_count, 1,
            "an unmatched realloc opens one binding"
        );
        assert_eq!(
            stats[&7].deallocated_bytes, 0,
            "unseen bytes cannot be released"
        );

        let stats = TaskIdRegistry::task_stats_from_events(&[event, release]);
        assert_eq!(
            stats[&7].live_bytes, 0,
            "the observed binding must be releasable"
        );
        assert_eq!(
            stats[&8].deallocated_bytes, 128,
            "release activity uses binding size, not a zero hint"
        );
    }
}

/// Objective: Keep unattributed bindings in the identity index for later task activity.
/// Invariants: a task can release an unowned block exactly once without acquiring ownership.
#[test]
fn unattributed_binding_can_be_released_by_a_task() {
    let mut allocation = MemoryEvent::allocate(0x3000, 32, 1);
    allocation.task_id = None;
    allocation.binding_serial = Some(3);
    let mut release = MemoryEvent::deallocate(allocation.ptr, 32, 2);
    release.task_id = Some(8);
    release.binding_serial = allocation.binding_serial;
    let stats = TaskIdRegistry::task_stats_from_events(&[allocation, release.clone(), release]);
    assert_eq!(
        stats[&8].deallocated_bytes, 32,
        "the valid release must be attributed to its actor"
    );
    assert_eq!(
        stats[&8].deallocation_count, 1,
        "a repeat release must not count again"
    );
    assert_eq!(
        stats[&8].live_bytes, 0,
        "release activity does not grant ownership"
    );
}

/// Objective: Validate snapshotting while fifty tasks concurrently defer allocator charges.
/// Invariants: every task's count and bytes agree with the window totals after joining.
#[test]
fn concurrent_deferred_snapshots_preserve_fifty_tasks() {
    let registry = TaskIdRegistry::new();
    let reader = registry
        .tasks
        .read()
        .expect("force charges onto the deferred path");
    std::thread::scope(|scope| {
        for task_id in 1..=50u64 {
            let registry = &registry;
            scope.spawn(move || {
                CURRENT_TASK_ID.set(Some(task_id));
                for _ in 0..20 {
                    assert_eq!(
                        registry.try_record_allocation(task_id as usize),
                        ChargeOutcome::Deferred,
                        "a busy registry must defer without losing the task ID"
                    );
                }
                CURRENT_TASK_ID.set(None);
            });
        }
        for _ in 0..20 {
            let _snapshot = registry.deferred_tasks();
        }
    });
    drop(reader);
    let gaps = registry.deferred_tasks();
    assert_eq!(
        gaps.len(),
        50,
        "each task must retain its independent gap row"
    );
    for task_id in 1..=50u64 {
        assert_eq!(
            gaps[&task_id],
            (20, task_id * 20),
            "all charges for task {task_id} must survive"
        );
    }
    let quality = registry.charge_quality();
    assert_eq!(
        quality.deferred_charges, 1000,
        "window count must equal per-task totals"
    );
    assert_eq!(
        quality.deferred_bytes,
        (1..=50u64).sum::<u64>() * 20,
        "window bytes must equal per-task totals"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(1000))]

    /// Objective: Differentially replay random identities, addresses and sizes against the memory ledger.
    /// Invariants: task ownership and activity match canonical transitions, including stale serials and zeros.
    #[test]
    fn random_task_transitions_match_binding_ledger(
        operations in proptest::collection::vec(
            (0u8..3, 0usize..8, 0usize..8, 0usize..4096, 0usize..4096,
             proptest::option::of(1u64..9), proptest::option::of(1u64..5)), 0..60)
    ) {
        let mut events = Vec::new();
        let mut ledger = crate::snapshot::BindingLedger::new();
        for (kind, ptr, new_ptr, size, old_size, serial, task) in operations {
            let thread = task.unwrap_or(0);
            let mut event = match kind {
                0 => MemoryEvent::allocate(ptr, size, thread),
                1 => MemoryEvent::deallocate(ptr, size, thread),
                _ => MemoryEvent::reallocate_moved(ptr, new_ptr, old_size, size, thread),
            };
            event.task_id = task;
            event.binding_serial = serial;
            ledger.apply(&event);
            events.push(event);
        }
        let stats = TaskIdRegistry::task_stats_from_events(&events);
        for task in 1..5u64 {
            let actual = stats.get(&task).copied().unwrap_or_default();
            let expected = ledger.thread_stats().get(&task).cloned().unwrap_or_default();
            proptest::prop_assert_eq!(actual.live_bytes, expected.current_memory as u64, "task {} live bytes", task);
            proptest::prop_assert_eq!(actual.allocated_bytes, expected.total_allocated as u64, "task {} allocated bytes", task);
            proptest::prop_assert_eq!(actual.deallocated_bytes, expected.total_deallocated as u64, "task {} released bytes", task);
            proptest::prop_assert_eq!(actual.allocation_count, expected.allocation_count as u64, "task {} allocation count", task);
        }
    }
}

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
