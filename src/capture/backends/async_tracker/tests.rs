//! Unit tests for the async memory tracker.

use super::*;
use crate::capture::backends::async_types::TaskOperation;
use serial_test::serial;
#[test]
fn test_async_tracker_creation() {
    let tracker = AsyncTracker::new();
    let stats = tracker.get_stats();
    assert_eq!(stats.total_tasks, 0);
}

#[test]
fn test_task_tracking() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();
    tracker
        .track_task_start(1, "test_task".to_string(), thread_id)
        .unwrap();

    let stats = tracker.get_stats();
    assert_eq!(stats.total_tasks, 1);
    assert_eq!(stats.active_tasks, 1);

    tracker.track_task_end(1).unwrap();
    let stats = tracker.get_stats();
    assert_eq!(stats.active_tasks, 0);
}

#[test]
fn test_allocation_tracking() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();
    tracker
        .track_task_start(1, "test_task".to_string(), thread_id)
        .unwrap();
    tracker.track_allocation(0x1000, 1024, 1);

    let profile = tracker.get_task_profile(1);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.total_allocations, 1);
    assert_eq!(profile.total_bytes, 1024);
}

#[test]
#[serial]
fn test_initialization() {
    reset_global_tracker();

    let result = initialize();
    assert!(result.is_ok());

    let result2 = initialize();
    if let Err(e) = result2 {
        assert!(e.message().contains("Already initialized"));
    }

    let _ = shutdown();
}

#[test]
#[serial]
fn test_shutdown() {
    reset_global_tracker();

    initialize().unwrap();
    let result = shutdown();
    assert!(result.is_ok());

    let result2 = shutdown();
    if let Err(e) = result2 {
        assert!(e.message().contains("Not initialized"));
    }
}

#[test]
#[serial]
fn test_memory_snapshot() {
    reset_global_tracker();

    initialize().unwrap();
    let snapshot = get_memory_snapshot();
    assert_eq!(snapshot.active_task_count, 0);
    let _ = shutdown();
}

#[test]
#[serial]
fn test_is_tracking_active() {
    reset_global_tracker();

    assert!(!is_tracking_active());
    initialize().unwrap();
    assert!(is_tracking_active());
    let _ = shutdown();
    assert!(!is_tracking_active());
}

#[test]
fn test_task_memory_profile() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();
    tracker
        .track_task_start(1, "test_task".to_string(), thread_id)
        .unwrap();
    tracker.track_allocation(0x1000, 1024, 1);
    tracker.track_allocation(0x2000, 2048, 1);
    tracker.track_task_end(1).unwrap();

    let profile = tracker.get_task_profile(1);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.task_id, 1);
    assert_eq!(profile.total_allocations, 2);
    assert_eq!(profile.total_bytes, 3072);
}

#[test]
fn test_duplicate_task_tracking() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    // First registration should succeed
    let result = tracker.track_task_start(1, "test_task".to_string(), thread_id);
    assert!(result.is_ok());

    // Second registration should fail
    let result = tracker.track_task_start(1, "duplicate_task".to_string(), thread_id);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, AsyncError::TaskTracking { operation, .. } if matches!(operation, TaskOperation::Duplicate))
    );
}

#[test]
fn test_task_not_found() {
    let tracker = AsyncTracker::new();

    // Calling track_task_end with non-existent task should fail
    let result = tracker.track_task_end(999);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, AsyncError::TaskTracking { operation, .. } if matches!(operation, TaskOperation::TaskNotFound))
    );
}

#[test]
fn test_task_guard_cleanup() {
    assert!(AsyncTracker::get_current_task().is_none());

    {
        let _guard = AsyncTracker::enter_task(42);
        assert_eq!(AsyncTracker::get_current_task(), Some(42));
    }

    assert!(AsyncTracker::get_current_task().is_none());
}

#[test]
fn test_with_task_closure() {
    assert!(AsyncTracker::get_current_task().is_none());

    let result = AsyncTracker::with_task(123, || {
        assert_eq!(AsyncTracker::get_current_task(), Some(123));
        "test_result"
    });

    assert_eq!(result, "test_result");
    assert!(AsyncTracker::get_current_task().is_none());
}

#[test]
fn test_with_task_panic_cleanup() {
    assert!(AsyncTracker::get_current_task().is_none());

    let result = std::panic::catch_unwind(|| {
        AsyncTracker::with_task(999, || {
            assert_eq!(AsyncTracker::get_current_task(), Some(999));
            panic!("intentional panic");
        });
    });

    assert!(result.is_err());
    assert!(AsyncTracker::get_current_task().is_none());
}

#[test]
fn test_generate_unique_task_id() {
    let id1 = generate_unique_task_id();
    let id2 = generate_unique_task_id();
    let id3 = generate_unique_task_id();

    assert!(id1 > 0);
    assert!(id2 > id1);
    assert!(id3 > id2);
}

#[test]
fn test_track_start_with_tokio() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    let result = tracker.track_task_start_with_tokio(1, 100, "tokio_task".to_string(), thread_id);
    assert!(result.is_ok());

    let profile = tracker.get_task_profile(1);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.task_id, 1);
    assert_eq!(profile.tokio_task_id, Some(100));
    assert_eq!(profile.task_name, "tokio_task");
}

#[test]
fn test_track_task_internal_without_tokio() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    let result = tracker.track_task_start(2, "normal_task".to_string(), thread_id);
    assert!(result.is_ok());

    let profile = tracker.get_task_profile(2);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.task_id, 2);
    assert_eq!(profile.tokio_task_id, None);
}

#[test]
fn test_detect_zombie_tasks() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    tracker
        .track_task_start(1, "task1".to_string(), thread_id)
        .unwrap();
    tracker
        .track_task_start(2, "task2".to_string(), thread_id)
        .unwrap();
    tracker
        .track_task_start(3, "task3".to_string(), thread_id)
        .unwrap();

    tracker.track_task_end(1).unwrap();

    let zombies = tracker.detect_zombie_tasks();
    assert_eq!(zombies.len(), 2);
    assert!(zombies.contains(&2));
    assert!(zombies.contains(&3));
}

#[test]
fn test_zombie_task_stats() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    tracker
        .track_task_start(1, "task1".to_string(), thread_id)
        .unwrap();
    tracker
        .track_task_start(2, "task2".to_string(), thread_id)
        .unwrap();

    tracker.track_task_end(1).unwrap();

    let (zombie_count, total) = tracker.zombie_task_stats();
    assert_eq!(zombie_count, 1);
    assert_eq!(total, 2);
}

#[test]
fn test_no_zombie_tasks_when_all_complete() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();

    tracker
        .track_task_start(1, "task1".to_string(), thread_id)
        .unwrap();
    tracker
        .track_task_start(2, "task2".to_string(), thread_id)
        .unwrap();

    tracker.track_task_end(1).unwrap();
    tracker.track_task_end(2).unwrap();

    let zombies = tracker.detect_zombie_tasks();
    assert!(zombies.is_empty());
}

#[test]
fn test_task_memory_profile_with_tokio_id() {
    let profile = TaskMemoryProfile::with_tokio_id(1, 999, "test".to_string(), TaskType::Mixed);

    assert_eq!(profile.task_id, 1);
    assert_eq!(profile.tokio_task_id, Some(999));
    assert_eq!(profile.task_name, "test");
    assert_eq!(profile.task_type, TaskType::Mixed);
    assert!(!profile.is_completed());
}

#[tokio::test]
async fn test_track_in_tokio_task_basic() {
    let tracker = AsyncTracker::new();

    let (task_id, result) = tracker
        .track_in_tokio_task("async_task".to_string(), async {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            42
        })
        .await;

    assert!(task_id > 0);
    assert_eq!(result, 42);

    let profile = tracker.get_task_profile(task_id);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.task_name, "async_task");
    assert!(profile.is_completed());
}

#[tokio::test]
async fn test_track_in_tokio_task_basic_functionality() {
    let tracker = AsyncTracker::new();

    let (task_id, result) = tracker
        .track_in_tokio_task("test_task".to_string(), async {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            "completed"
        })
        .await;

    assert!(task_id > 0);
    assert_eq!(result, "completed");

    let profile = tracker.get_task_profile(task_id);
    assert!(profile.is_some());
    let profile = profile.unwrap();
    assert_eq!(profile.task_name, "test_task");
    assert!(profile.is_completed());
}

#[test]
#[serial]
fn test_global_tracker_integration() {
    reset_global_tracker();

    let result = initialize();
    assert!(result.is_ok());

    let tracker = get_global_tracker();
    assert!(tracker.is_ok());

    let tracker = tracker.unwrap();
    let stats = tracker.get_stats();
    assert_eq!(stats.total_tasks, 0);
    assert_eq!(stats.active_tasks, 0);

    let _ = shutdown();
}

/// Objective: Verify a repeated sighting of the same address does not add bytes
/// or allocations twice in the async ledger (plan A06 / §7.2).
/// Invariants: one allocation, one block of bytes, in both the aggregate stats
/// and the per-task profile.
#[test]
fn test_duplicate_async_observation_is_not_double_counted() {
    let tracker = AsyncTracker::new();
    tracker
        .track_task_start(7, "t7".to_string(), std::thread::current().id())
        .expect("task start must succeed");

    tracker.track_allocation(0x1000, 512, 7);
    tracker.track_allocation(0x1000, 512, 7);

    let stats = tracker.get_stats();
    assert_eq!(
        stats.total_allocations, 1,
        "the same address must not be counted as two async allocations"
    );
    assert_eq!(
        stats.total_memory, 512,
        "bytes must not be added twice for the same block"
    );
    assert_eq!(stats.active_memory, 512, "only one block is live");

    let profile = tracker
        .get_task_profile(7)
        .expect("the task profile must exist");
    assert_eq!(
        profile.total_allocations, 1,
        "the per-task profile must not count the duplicate"
    );
    assert_eq!(
        profile.current_memory, 512,
        "the per-task live bytes must equal the one live block"
    );
}

/// Objective: Verify a different size at a known address is booked as a new binding and
/// stays conserved through release (plan A06 follow-up).
/// Invariants: the allocation counter grows by one, the superseded bytes are booked as
/// released, and releasing the block clears both the stats and the profile.
#[test]
fn test_async_size_change_is_booked_as_new_binding() {
    let tracker = AsyncTracker::new();
    tracker
        .track_task_start(8, "t8".to_string(), std::thread::current().id())
        .expect("task start must succeed");

    tracker.track_allocation(0x2000, 1024, 8);
    tracker.track_allocation(0x2000, 2048, 8);

    let grown = tracker.get_stats();
    assert_eq!(
        grown.active_memory, 2048,
        "only the new size of the block is live"
    );
    assert_eq!(
        grown.total_allocations, 2,
        "an observation cannot prove identity continuity, so the block is a new binding"
    );
    assert_eq!(
        grown.total_deallocated, 1024,
        "the superseded block's bytes must be booked as released"
    );
    let profile = tracker.get_task_profile(8).expect("profile must exist");
    assert_eq!(
        profile.current_memory, 2048,
        "the task profile must follow the new size"
    );

    tracker.track_deallocation(0x2000);

    let released = tracker.get_stats();
    assert_eq!(
        released.active_memory, 0,
        "releasing the only block must leave zero live bytes, got {}",
        released.active_memory
    );
    let profile = tracker.get_task_profile(8).expect("profile must exist");
    assert_eq!(
        profile.current_memory, 0,
        "the task profile must return to zero live bytes"
    );
}

/// Objective: Verify the snapshot reports the thread the task actually ran on, not
/// the thread that builds the snapshot.
/// Invariants: the reported numeric id is the worker thread's id, and it differs from
/// the exporting thread's id.
#[test]
fn test_snapshot_reports_task_thread_not_exporter_thread() {
    let tracker = Arc::new(AsyncTracker::new());
    let worker = Arc::clone(&tracker);
    let worker_id = std::thread::spawn(move || {
        let thread_id = std::thread::current().id();
        worker
            .track_task_start(9, "t9".to_string(), thread_id)
            .expect("task start must succeed");
        worker.track_allocation(0x9000, 64, 9);
        crate::utils::current_thread_id_u64()
    })
    .join()
    .expect("worker thread must join");

    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.tasks.len(), 1, "one active task is expected");
    assert_eq!(
        snapshot.tasks[0].thread_id,
        Some(worker_id),
        "the snapshot must report the task's own thread"
    );
    assert_ne!(
        snapshot.tasks[0].thread_id,
        Some(crate::utils::current_thread_id_u64()),
        "the exporting thread must never be substituted for the task thread"
    );
}

/// Objective: Verify a nested task guard restores the parent context.
/// Invariants: entering 1 then 2 and dropping 2 returns to 1; dropping the outer
/// guard returns to the previous (empty) context.
#[test]
fn test_task_guard_restores_previous_task() {
    AsyncTracker::clear_current_task();

    let outer = AsyncTracker::enter_task(1);
    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(1),
        "the guard must install its task"
    );

    {
        let inner = AsyncTracker::enter_task(2);
        assert_eq!(
            AsyncTracker::get_current_task(),
            Some(2),
            "the nested guard must take over"
        );
        drop(inner);
        assert_eq!(
            AsyncTracker::get_current_task(),
            Some(1),
            "dropping the inner guard must restore the parent task"
        );
    }

    drop(outer);
    assert_eq!(
        AsyncTracker::get_current_task(),
        None,
        "dropping the outermost guard must restore the empty context"
    );
}

/// Objective: Verify ending a task does not clear another task's context.
/// Invariants: the context installed by the later task survives the earlier
/// task's completion.
#[test]
fn test_task_end_does_not_clear_foreign_context() {
    let tracker = AsyncTracker::new();
    tracker
        .track_task_start(5, "a".to_string(), std::thread::current().id())
        .expect("task start must succeed");
    tracker
        .track_task_start(6, "b".to_string(), std::thread::current().id())
        .expect("task start must succeed");
    assert_eq!(AsyncTracker::get_current_task(), Some(6));

    tracker
        .track_task_end(5)
        .expect("ending a tracked task must succeed");
    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(6),
        "ending task 5 must not clear the context owned by task 6"
    );
}

/// Objective: Verify an async task id that does not fit into u64 is reported
/// instead of silently truncated to 32 bits.
/// Invariants: the call succeeds, no allocation is attributed, and the ledger
/// stays empty.
#[test]
#[serial]
fn test_track_current_allocation_rejects_unrepresentable_task_id() {
    reset_global_tracker();
    let tracker = Arc::new(AsyncTracker::new());
    register_global(tracker.clone()).expect("global registration must succeed");

    let huge_id = u128::MAX;
    crate::capture::backends::async_types::set_current_task(
        crate::capture::backends::async_types::ExtendedTaskInfo::new(huge_id, None),
    );

    track_current_allocation(0x10, 8).expect("tracking must not error on a wide id");
    assert_eq!(
        tracker.get_stats().total_allocations,
        0,
        "an unrepresentable task id must not be truncated and attributed"
    );

    crate::capture::backends::async_types::clear_current_task();
    reset_global_tracker();
}

/// Objective: Verify the async snapshot JSON carries the task's numeric thread id.
/// Invariants: the serialized task object exposes `thread_id` as a number, not a
/// debug string, so consumers can join it with allocation records.
#[test]
fn test_async_snapshot_json_contains_numeric_thread_id() {
    let tracker = AsyncTracker::new();
    let current = crate::utils::current_thread_id_u64();
    tracker
        .track_task_start(11, "json_task".to_string(), std::thread::current().id())
        .expect("task start must succeed");
    tracker.track_allocation(0xA000, 32, 11);

    let snapshot = tracker.snapshot();
    let json = serde_json::to_value(&snapshot).expect("the snapshot must serialize");
    assert_eq!(
        json["tasks"][0]["thread_id"],
        serde_json::json!(current),
        "the serialized task must carry the numeric thread id"
    );
}

/// Objective: Verify a late release of generation 1 does not close generation 2 after
/// the same address was reused (the async counterpart of the binding ledger rule).
/// Invariants: the newer binding keeps its bytes and its generation, and the stale
/// release leaves the ledger unchanged.
#[test]
fn late_release_does_not_close_reused_async_address() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();
    tracker
        .track_task_start(1, "owner".to_string(), thread_id)
        .expect("first task must start");
    tracker
        .track_task_start(2, "reuser".to_string(), thread_id)
        .expect("second task must start");

    tracker.track_allocation(0xB000, 64, 1);
    tracker.track_deallocation(0xB000);
    tracker.track_allocation(0xB000, 128, 2);

    // A release reported for the already-closed generation must be a no-op.
    tracker.track_deallocation_matching(0xB000, Some(64), Some(1));

    let snapshot = tracker.snapshot();
    assert_eq!(
        snapshot.allocations.len(),
        1,
        "the reused binding must still be live"
    );
    assert_eq!(
        snapshot.allocations[0].size, 128,
        "the surviving binding keeps its size"
    );
    assert_eq!(
        snapshot.allocations[0].generation, 2,
        "a reused address must start a new generation"
    );
    assert_eq!(
        tracker.get_stats().active_memory,
        128,
        "the stale release must not change the live bytes"
    );
}

/// Objective: Verify a different size at a known address starts a new generation instead
/// of inheriting the old one, and that the superseded bytes are booked (plan A06
/// follow-up).
/// Invariants: the new record is generation 2, a late release of generation 1 does not
/// close it, and `total_memory - total_deallocated == active_memory` after the final
/// release.
#[test]
fn different_size_at_same_address_starts_new_generation() {
    let tracker = AsyncTracker::new();
    let thread_id = std::thread::current().id();
    tracker
        .track_task_start(1, "owner".to_string(), thread_id)
        .expect("first task must start");
    tracker
        .track_task_start(2, "observer".to_string(), thread_id)
        .expect("second task must start");

    tracker.track_allocation(0xC000, 64, 1);
    tracker.track_allocation(0xC000, 128, 2);

    let snapshot = tracker.snapshot();
    assert_eq!(snapshot.allocations.len(), 1, "one live record is expected");
    assert_eq!(
        snapshot.allocations[0].size, 128,
        "the new size must survive"
    );
    assert_eq!(
        snapshot.allocations[0].generation, 2,
        "a different size must start a new generation"
    );
    assert_eq!(
        snapshot.allocations[0].task_id, 2,
        "the record belongs to the observing task"
    );

    // A release reported for the superseded generation must not close the new record.
    tracker.track_deallocation_matching(0xC000, Some(64), Some(1));
    assert_eq!(
        tracker.snapshot().allocations.len(),
        1,
        "the stale release must be a no-op"
    );

    tracker.track_deallocation(0xC000);
    let stats = tracker.get_stats();
    assert_eq!(stats.active_memory, 0, "no live bytes may remain");
    assert_eq!(
        stats.total_memory, 192,
        "both blocks were observed: 64 + 128"
    );
    assert_eq!(
        stats.total_deallocated, 192,
        "the superseded block and the released block must both be booked"
    );
    assert_eq!(
        stats.total_memory as u64 - stats.total_deallocated,
        stats.active_memory as u64,
        "allocated minus deallocated must equal the live bytes"
    );
}

/// Objective: Verify guards dropped out of order do not resurrect an exited scope
/// (plan §7.2 task-context rule).
/// Invariants: dropping the outer guard first keeps the inner task current; dropping the
/// inner one then clears the context instead of restoring the finished outer scope.
#[test]
fn out_of_order_guard_drop_does_not_restore_exited_scope() {
    AsyncTracker::clear_current_task();

    let outer = AsyncTracker::enter_task(41);
    let inner = AsyncTracker::enter_task(42);
    assert_eq!(AsyncTracker::get_current_task(), Some(42));

    drop(outer);
    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(42),
        "dropping the outer guard must not disturb the running inner task"
    );

    drop(inner);
    assert_eq!(
        AsyncTracker::get_current_task(),
        None,
        "the exited outer scope must not be restored"
    );
}

/// Objective: Verify a guard dropped over a context installed outside a guard (for
/// example by `track_task_start`) restores that parent instead of clearing it.
/// Invariants: after the guard ends, the externally installed task is current again.
#[test]
fn guard_over_external_context_restores_parent() {
    AsyncTracker::clear_current_task();
    AsyncTracker::set_current_task(41);

    {
        let _guard = AsyncTracker::enter_task(42);
        assert_eq!(AsyncTracker::get_current_task(), Some(42));
    }

    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(41),
        "the externally installed parent context must survive the guard"
    );

    AsyncTracker::clear_current_task();
}

/// Objective: Verify re-entering the same task id keeps guard identity: dropping the
/// outermost frame must not remove the innermost one.
/// Invariants: after dropping the outermost 41 while (41, 42, 41) is active, the inner
/// 41 stays current (not 42), and the remaining frames unwind in order.
#[test]
fn reentrant_task_ids_are_removed_by_guard_identity() {
    AsyncTracker::clear_current_task();

    let outer_41 = AsyncTracker::enter_task(41);
    let guard_42 = AsyncTracker::enter_task(42);
    let inner_41 = AsyncTracker::enter_task(41);
    assert_eq!(AsyncTracker::get_current_task(), Some(41));

    drop(outer_41);
    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(41),
        "dropping the outermost frame must not change the innermost task"
    );

    drop(inner_41);
    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(42),
        "the 42 frame must become current again"
    );

    drop(guard_42);
    assert_eq!(
        AsyncTracker::get_current_task(),
        None,
        "the stack must unwind to an empty context"
    );
}

/// Objective: Verify `track_task_end` does not delete a still-live guard frame that
/// happens to carry the same task id (re-entrant scopes).
/// Invariants: after ending the explicit context the guard's frame is still current, and
/// dropping the guard clears the context.
#[test]
fn ending_a_task_keeps_a_live_guard_frame_of_the_same_id() {
    AsyncTracker::clear_current_task();
    let guard = AsyncTracker::enter_task(41);
    AsyncTracker::set_current_task(41);

    let tracker = AsyncTracker::new();
    tracker
        .track_task_start(41, "reentrant".to_string(), std::thread::current().id())
        .expect("task start must succeed");
    tracker
        .track_task_end(41)
        .expect("ending the task must succeed");

    assert_eq!(
        AsyncTracker::get_current_task(),
        Some(41),
        "the explicit context must be gone but the live guard frame must survive"
    );

    drop(guard);
    assert_eq!(
        AsyncTracker::get_current_task(),
        None,
        "dropping the guard must end its own scope"
    );
}
