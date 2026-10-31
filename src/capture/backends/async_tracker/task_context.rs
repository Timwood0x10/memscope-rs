//! Thread-local task scopes with guard lifetimes that can end on another thread.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

thread_local! {
    static SCOPES: RefCell<Vec<TaskFrame>> = const { RefCell::new(Vec::new()) };
}

struct TaskFrame {
    task_id: u64,
    // None denotes an explicitly installed context, rather than a guard scope.
    active: Option<Arc<AtomicBool>>,
}

impl TaskFrame {
    fn is_active(&self) -> bool {
        self.active
            .as_ref()
            .is_none_or(|active| active.load(Ordering::Acquire))
    }
}

/// Restores the enclosing live scope when dropped or explicitly released.
///
/// Moving the guard does not install its context on the destination thread. Dropping
/// it there marks only its own frame as ended; the origin prunes it on its next read.
pub struct TaskGuard {
    task_id: u64,
    active: Arc<AtomicBool>,
}

impl TaskGuard {
    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    pub fn release(self) {
        drop(self);
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

pub(super) fn current() -> Option<u64> {
    SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        scopes.retain(TaskFrame::is_active);
        scopes.last().map(|frame| frame.task_id)
    })
}

pub(super) fn enter(task_id: u64) -> TaskGuard {
    let active = Arc::new(AtomicBool::new(true));
    SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        scopes.retain(TaskFrame::is_active);
        scopes.push(TaskFrame {
            task_id,
            active: Some(Arc::clone(&active)),
        });
    });
    TaskGuard { task_id, active }
}

pub(super) fn set(task_id: u64) {
    SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        // An explicit set replaces the previous explicit context, but does not end guards.
        scopes.retain(|frame| frame.active.is_some() && frame.is_active());
        scopes.push(TaskFrame {
            task_id,
            active: None,
        });
    });
}

pub(super) fn clear() {
    SCOPES.with(|scopes| scopes.borrow_mut().clear());
}

/// Drop the *explicit* context of a task that ended.
///
/// Guard-owned frames are left alone: a guard still holds their lifetime, and removing
/// them here would silently delete a live scope when the same task id is nested. They
/// disappear on their own when the guard drops (possibly on another thread).
pub(super) fn end(task_id: u64) {
    SCOPES.with(|scopes| {
        scopes.borrow_mut().retain(|frame| {
            frame.is_active() && !(frame.task_id == task_id && frame.active.is_none())
        });
    });
}
