use std::ops::Deref;
use std::sync::Arc;

use crate::capture::backends::global_tracking::GlobalTracker;

/// Convenience session handle — one-call init, `track!`-able, one-call export.
///
/// # Example
///
/// ```ignore
/// use memscope_rs::prelude::*;
///
/// let ctx = MemCtx::init()?;
/// let data = vec![1u64; 100];
/// track!(ctx, data);       // macro auto‑derefs to GlobalTracker
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
