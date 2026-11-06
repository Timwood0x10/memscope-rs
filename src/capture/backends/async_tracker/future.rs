//! Poll-scoped context and task completion, including unwind and cancellation.

use super::{task_context, AsyncTracker, TaskGuard};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(super) struct BorrowedLifecycleGuard<'a> {
    tracker: &'a AsyncTracker,
    task_id: u64,
}

impl<'a> BorrowedLifecycleGuard<'a> {
    pub(super) fn new(tracker: &'a AsyncTracker, task_id: u64) -> Self {
        Self { tracker, task_id }
    }
}

impl Drop for BorrowedLifecycleGuard<'_> {
    fn drop(&mut self) {
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        let _context = PollContextGuard::install(self.task_id);
        crate::task_registry::global_registry().unregister_task(self.task_id);
        let _ = self.tracker.track_task_end(self.task_id);
    }
}

pub(super) struct TaskLifecycleGuard {
    task_id: u64,
    tracker: Option<Arc<AsyncTracker>>,
    completed: bool,
}

impl TaskLifecycleGuard {
    pub(super) fn new(task_id: u64, tracker: Option<Arc<AsyncTracker>>) -> Self {
        Self {
            task_id,
            tracker,
            completed: false,
        }
    }

    pub(super) fn set_completed(&mut self) {
        self.completed = true;
    }
}

impl Drop for TaskLifecycleGuard {
    fn drop(&mut self) {
        let _suppression = crate::core::allocator::TrackingGuard::acquire();
        let _guard = PollContextGuard::install(self.task_id);
        crate::task_registry::global_registry().unregister_task(self.task_id);
        if let Some(ref tracker) = self.tracker {
            let _ = tracker.track_task_end(self.task_id);
        }
        tracing::trace!(
            task_id = self.task_id,
            completed = self.completed,
            "task ended"
        );
    }
}

/// Context exists only during poll or destruction, never across Pending.
pub(super) struct ContextualFuture<F> {
    inner: Option<Pin<Box<F>>>,
    task_id: u64,
}

impl<F> ContextualFuture<F> {
    pub(super) fn new(future: F, task_id: u64) -> Self {
        Self {
            inner: Some(Box::pin(future)),
            task_id,
        }
    }
}

impl<F> Drop for ContextualFuture<F> {
    fn drop(&mut self) {
        let _guard = PollContextGuard::install(self.task_id);
        drop(self.inner.take());
    }
}

struct PollContextGuard {
    _task_guard: TaskGuard,
    prev_task_id: Option<u64>,
}

impl PollContextGuard {
    fn install(task_id: u64) -> Self {
        let task_guard = task_context::enter(task_id);
        let prev_task_id = crate::task_registry::TaskIdRegistry::current_task_id();
        crate::task_registry::global_registry().set_current_task_id(task_id);
        Self {
            _task_guard: task_guard,
            prev_task_id,
        }
    }
}

impl Drop for PollContextGuard {
    fn drop(&mut self) {
        crate::task_registry::global_registry().set_current_task_id_opt(self.prev_task_id);
    }
}

impl<F: Future> Future for ContextualFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Moving Pin<Box<F>> cannot move the pinned F, so the wrapper is Unpin.
        let this = self.get_mut();
        let _guard = PollContextGuard::install(this.task_id);
        this.inner
            .as_mut()
            .expect("ContextualFuture polled after completion")
            .as_mut()
            .poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Local tracking finalizes on cancellation and unwind.
    /// Invariants: Both metadata sources complete, and caller TLS is restored.
    #[test]
    fn local_future_cancel_and_unwind_complete() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let tracker = AsyncTracker::new();
        let parent = crate::task_registry::TaskIdRegistry::current_task_id();
        runtime.block_on(async {
            let mut pending = Box::pin(
                tracker.track_in_tokio_task("cancel-local".into(), std::future::pending::<()>()),
            );
            std::future::poll_fn(|cx| {
                assert!(
                    pending.as_mut().poll(cx).is_pending(),
                    "inner task suspends"
                );
                Poll::Ready(())
            })
            .await;
            drop(pending);
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.block_on(tracker.track_in_tokio_task("panic-local".into(), async {
                panic!("inner panic");
            }));
        }));
        assert!(result.is_err(), "inner panic reaches caller");
        let profiles = tracker.get_all_profiles();
        assert_eq!(profiles.len(), 2, "both tasks retain lifecycle records");
        for profile in profiles {
            assert!(
                profile.is_completed(),
                "local tracker completes after cancellation/unwind"
            );
            let meta = crate::task_registry::global_registry()
                .get_task(profile.task_id)
                .unwrap();
            assert!(
                meta.completed_at.is_some(),
                "registry records actual terminal time"
            );
        }
        assert_eq!(
            crate::task_registry::TaskIdRegistry::current_task_id(),
            parent,
            "restore caller context"
        );
    }

    /// Objective: Scope and async IDs cannot alias under mixed concurrent creation.
    /// Invariants: Every generated identity is unique; local scopes complete locally.
    #[test]
    fn fifty_threads_share_task_identity_space() {
        let handles: Vec<_> = (0..50)
            .map(|_| {
                std::thread::spawn(|| {
                    let registry = crate::task_registry::TaskIdRegistry::new();
                    let mut ids = Vec::new();
                    for _ in 0..20 {
                        let scope = registry.task_scope("local");
                        let id = crate::task_registry::TaskIdRegistry::current_task_id().unwrap();
                        ids.push(id);
                        ids.push(super::super::generate_unique_task_id());
                        drop(scope);
                        assert!(
                            registry.get_task(id).unwrap().completed_at.is_some(),
                            "scope completes its own registry"
                        );
                    }
                    ids
                })
            })
            .collect();
        let ids: Vec<_> = handles
            .into_iter()
            .flat_map(|h| h.join().expect("identity worker exits"))
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(
            unique.len(),
            ids.len(),
            "mixed creation must not overwrite another task"
        );
    }
}
