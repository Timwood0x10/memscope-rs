/// Ergonomics wrapper — one-call init + simplified export.
///
/// # Example
///
/// ```ignore
/// use memscope_rs::prelude::*;
///
/// MemCtx::init()?;
/// let data = vec![1u64; 100];
/// track!(MemCtx::tracker(), data);
/// MemCtx::export("./report")?;
/// ```
pub struct MemCtx;

impl MemCtx {
    /// Initialise logging + global tracking in one call.
    pub fn init() -> crate::MemScopeResult<()> {
        crate::init_logging()?;
        crate::capture::backends::global_tracking::init_global_tracking()?;
        Ok(())
    }

    /// Obtain the global tracker (panics if `init()` was not called first).
    pub fn tracker() -> std::sync::Arc<crate::capture::backends::global_tracking::GlobalTracker> {
        crate::global_tracker()
            .expect("MemCtx::init() must be called before accessing the tracker")
    }

    /// Export complete dashboard (HTML + JSON) under `path`.
    pub fn export(path: impl AsRef<std::path::Path>) -> crate::MemScopeResult<()> {
        let t = Self::tracker();
        t.export_html(path)
    }
}
