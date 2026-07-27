use std::ops::Deref;
use std::sync::Arc;

use crate::auto_export::MemScopeConfig;
use crate::capture::backends::global_tracking::GlobalTracker;
use crate::guard::MemScopeGuard;

/// Convenience session handle — one-call init, `track!`-able, one-call export.
///
/// # Examples
///
/// ```ignore
/// use memscope_rs::prelude::*;
///
/// let ctx = MemCtx::init()?;
/// let data = vec![1u64; 100];
/// track!(ctx, data);       // macro auto-derefs to GlobalTracker
/// ctx.export("./report")?; // delegate to GlobalTracker::export_html
/// ```
pub struct MemCtx {
    tracker: Arc<GlobalTracker>,
}

impl MemCtx {
    /// Initialise logging + global tracking in one call.
    pub fn init() -> crate::MemScopeResult<Self> {
        crate::init_logging()?;
        crate::capture::backends::global_tracking::init_global_tracking()?;
        let tracker = crate::global_tracker()?;
        Ok(Self { tracker })
    }

    /// One-line start with the default configuration.
    ///
    /// Equivalent to [`MemCtx::start_with`]`(MemScopeConfig::default())`. The
    /// returned [`MemScopeGuard`] triggers the exit-path export (HTML + JSON
    /// to `./memscope-report`) on drop, covering normal return, panic, and —
    /// when the `auto-signal` feature is enabled — Ctrl-C.
    ///
    /// This is the recommended entry point for new code. Existing code that
    /// calls [`MemCtx::init`] + [`MemCtx::export`] continues to work unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`MemScopeError`](crate::MemScopeError) if logging init, global
    /// tracker init, or lifecycle hook installation fails.
    pub fn start() -> crate::MemScopeResult<MemScopeGuard> {
        crate::guard::start()
    }

    /// One-line start with a custom [`MemScopeConfig`].
    ///
    /// Like [`MemCtx::start`] but accepts a user-supplied configuration for
    /// output path, formats, signal policy, and periodic flush interval.
    /// The returned [`MemScopeGuard`] owns the optional background flusher
    /// and triggers the exit-path export on drop.
    ///
    /// # Errors
    ///
    /// Returns [`MemScopeError`](crate::MemScopeError) if any of the setup
    /// steps fails. A repeated call without `reset_global_tracking` fails at
    /// the global-tracker init step.
    pub fn start_with(config: MemScopeConfig) -> crate::MemScopeResult<MemScopeGuard> {
        crate::guard::start_with(config)
    }

    /// Export complete dashboard (HTML + JSON) under `path`.
    pub fn export(&self, path: impl AsRef<std::path::Path>) -> crate::MemScopeResult<()> {
        self.tracker.export_html(path)
    }
}

impl Deref for MemCtx {
    type Target = GlobalTracker;
    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}
