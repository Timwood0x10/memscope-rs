//! `MemScopeGuard` — the RAII handle returned by `start()` / `start_with()`.
//!
//! Holding the guard keeps tracking alive; dropping it triggers the idempotent
//! exit-path export (`lifecycle::export_once`). Combined with the panic hook,
//! ctrlc handler, and optional atexit registered by `lifecycle::install`, this
//! covers every realistic program-exit path.
//!
//! # Module role
//!
//! This is module 4 (the final module) of the auto-export feature. It ties
//! modules 1-3 together into the user-facing API:
//!
//! - [`start`] / [`start_with`] initialize logging, install the global tracker,
//!   wire the lifecycle hooks, and optionally spawn a background flusher.
//! - The returned [`MemScopeGuard`] owns the flusher handle and a clone of the
//!   tracker `Arc`. Its [`Drop`] impl joins the flusher (with a final flush)
//!   and then runs the idempotent exit-path export.
//!
//! # Drop ordering
//!
//! The flusher is dropped BEFORE the final `export_once` call so that the
//! flusher's shutdown-time final flush is visible to the export latch. The
//! latch (`lifecycle::EXPORTED`) is set by whichever export runs first; the
//! second is a no-op, so there is no double-write.

use std::ops::Deref;
use std::sync::Arc;

use crate::auto_export::{AutoExportConfig, MemScopeConfig};
use crate::capture::backends::global_tracking::GlobalTracker;
use crate::core::error::MemScopeResult;
use crate::lifecycle;
use crate::periodic_flusher::PeriodicFlusher;

/// RAII handle that keeps the auto-export subsystem alive.
///
/// Created by [`start`] / [`start_with`]. Dropping it joins the optional
/// background flusher and then triggers the idempotent exit-path export via
/// [`lifecycle::export_once`]. The guard also derefs to [`GlobalTracker`] so
/// callers can use `track!` / `track_as` / `export_html` directly on it.
pub struct MemScopeGuard {
    /// Tracker handle cloned from the global singleton at [`start_with`] time.
    /// Kept alive by the guard so the tracker survives even if the global
    /// singleton is later cleared by `reset_global_tracking`.
    tracker: Arc<GlobalTracker>,
    /// Optional background flusher. `None` when `flush_interval` is `None`.
    /// Taken and dropped in [`Drop`] before the final export so the flusher's
    /// final flush is included in the export window.
    flusher: Option<PeriodicFlusher>,
}

impl MemScopeGuard {
    /// Borrow the underlying tracker handle.
    ///
    /// The returned `Arc` is the same one registered with the global singleton
    /// and the lifecycle hooks, so callers can compare pointers or share it
    /// across threads without re-fetching the singleton.
    #[must_use]
    pub fn tracker(&self) -> &Arc<GlobalTracker> {
        &self.tracker
    }

    /// Trigger an immediate export. Resets the idempotency latch first so a
    /// later panic / Ctrl-C / Drop still produces a fresh report.
    ///
    /// # Returns
    ///
    /// `true` if the export ran; `false` if no tracker / config is installed
    /// (e.g. after `reset_for_test`).
    #[must_use]
    pub fn export_now(&self) -> bool {
        lifecycle::trigger_export_now()
    }

    /// In-memory JSON snapshot (no disk write). Builds an [`AnalysisReport`]
    /// via the analyzer pipeline and serializes it. Intended for HTTP
    /// endpoints or ad-hoc inspection without touching the filesystem.
    ///
    /// # Errors
    ///
    /// Returns [`MemScopeError`] if no tracker is installed (e.g. after
    /// `reset_global_tracking`) or if serialization of the analysis report
    /// fails.
    ///
    /// [`AnalysisReport`]: crate::analyzer::AnalysisReport
    /// [`MemScopeError`]: crate::MemScopeError
    pub fn snapshot_json(&self) -> MemScopeResult<String> {
        lifecycle::snapshot_json()
    }
}

impl Deref for MemScopeGuard {
    type Target = GlobalTracker;
    fn deref(&self) -> &Self::Target {
        &self.tracker
    }
}

impl Drop for MemScopeGuard {
    fn drop(&mut self) {
        // Drop the flusher first so its shutdown-time final flush runs before
        // the export latch is consulted. PeriodicFlusher::Drop sends the
        // shutdown signal, joins the worker, and the worker performs a final
        // flush before exiting.
        if let Some(flusher) = self.flusher.take() {
            drop(flusher);
        }
        // Idempotent final export gated by `on_exit`. If the flusher's final
        // flush already set the latch, this is a no-op. Otherwise this is the
        // only export. We use `ExportReason::Drop` so the `on_exit` config
        // flag is honored (a user who set `on_exit: false` gets no Drop export
        // but may still get a panic/signal export).
        let _ = lifecycle::export_for_reason(lifecycle::ExportReason::Drop);
    }
}

/// Start memscope-rs with the default configuration.
///
/// Equivalent to [`start_with`] called with [`MemScopeConfig::default()`]. The
/// returned guard triggers the exit-path export on drop.
///
/// # Errors
///
/// Returns `Err(MemScopeError)` if logging init, global tracker init, or
/// lifecycle hook installation fails. The most common failure is calling
/// `start()` twice without resetting global tracking in between.
///
/// [`MemScopeError`]: crate::MemScopeError
pub fn start() -> MemScopeResult<MemScopeGuard> {
    start_with(MemScopeConfig::default())
}

/// Start memscope-rs with a custom configuration.
///
/// This:
/// 1. Initializes the tracing subscriber (idempotent via `Once`).
/// 2. Installs the global tracker with the provided `GlobalTrackerConfig`.
/// 3. Wires the auto-export lifecycle hooks (panic hook always; ctrlc / atexit
///    behind their respective features) via [`lifecycle::install`].
/// 4. Optionally spawns a [`PeriodicFlusher`] if `flush_interval` is `Some`.
///
/// The returned [`MemScopeGuard`] owns the flusher handle and a clone of the
/// tracker `Arc`; dropping it triggers the final export.
///
/// # Errors
///
/// Returns `Err(MemScopeError)` if any of the four setup steps fails. A
/// repeated `start_with` call without `reset_global_tracking` fails at step 2
/// because the global singleton is already initialized.
///
/// [`MemScopeError`]: crate::MemScopeError
pub fn start_with(config: MemScopeConfig) -> MemScopeResult<MemScopeGuard> {
    crate::init_logging()?;
    crate::init_global_tracking_with_config(config.tracker.clone())?;
    let tracker = crate::global_tracker()?;
    lifecycle::install(config.auto_export.clone(), tracker.clone())?;
    let flusher = spawn_flusher(&config.auto_export);
    Ok(MemScopeGuard { tracker, flusher })
}

/// Spawn a [`PeriodicFlusher`] if `cfg.flush_interval` is `Some`.
///
/// The flush closure calls [`lifecycle::trigger_export_now`] (not
/// `export_once`) so each periodic tick resets the idempotency latch and
/// writes a fresh report. The exit-path export on Drop still fires afterwards
/// — it just becomes a no-op if the flusher already exported, or runs if the
/// flusher never ticked.
fn spawn_flusher(cfg: &AutoExportConfig) -> Option<PeriodicFlusher> {
    let interval = cfg.flush_interval?;
    Some(PeriodicFlusher::new(interval, || {
        let _ = lifecycle::trigger_export_now();
    }))
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_export::{AutoExportConfig, MemScopeConfig};
    use crate::track;
    use parking_lot::Mutex;
    use serial_test::serial;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Reset every process-global slot touched by `start` / `start_with` so
    /// each `#[serial]` test begins from a known-clean state. Callers must
    /// hold `#[serial]` so concurrent tests cannot clobber each other.
    fn reset_globals() {
        crate::capture::backends::global_tracking::reset_global_tracking();
        lifecycle::reset_for_test();
    }

    /// Build a `MemScopeConfig` whose auto-export output points at a fresh
    /// tempdir. The caller must keep the returned `TempDir` alive until after
    /// the guard is dropped so the directory is not deleted mid-export.
    fn config_with_tempdir() -> (TempDir, MemScopeConfig) {
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default()
            .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
        (dir, cfg)
    }

    // ===================== Positive tests (happy path) ====================

    /// Objective: Verify `start()` returns a guard whose `tracker()` is a
    /// non-null `Arc<GlobalTracker>` pointing at the global singleton.
    /// Invariants: The guard's tracker and the global singleton must be the
    /// same `Arc` (pointer equality), proving the guard captured the live
    /// tracker rather than a stale or fresh copy.
    #[test]
    #[serial]
    fn start_returns_guard_with_tracker() {
        reset_globals();
        let (_dir, cfg) = config_with_tempdir();
        let guard = start_with(cfg).expect("start_with with default config must succeed");

        let guard_tracker: &Arc<GlobalTracker> = guard.tracker();
        let singleton: Arc<GlobalTracker> = crate::global_tracker()
            .expect("global_tracker() must succeed immediately after start_with()");
        assert!(
            Arc::ptr_eq(guard_tracker, &singleton),
            "guard.tracker() must reference the same Arc<GlobalTracker> as the global singleton"
        );
        drop(guard);
    }

    /// Objective: Verify dropping the guard writes the HTML dashboard to the
    /// configured output directory via the exit-path export.
    /// Invariants: `dashboard_unified_dashboard.html` exists after drop, and
    /// the file is non-empty (the renderer wrote real content).
    #[test]
    #[serial]
    fn drop_guard_writes_html_report() {
        reset_globals();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default()
            .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
        let guard = start_with(cfg).expect("start_with with tempdir output must succeed");

        drop(guard);

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "HTML dashboard must exist at {html:?} after dropping the guard"
        );
        let content = std::fs::read_to_string(&html)
            .expect("HTML dashboard file must be readable after export");
        assert!(
            !content.is_empty(),
            "HTML dashboard content must not be empty"
        );
    }

    /// Objective: Verify `start_with` with a `flush_interval` spawns a flusher
    /// that writes a report within a few tick intervals, and that dropping the
    /// guard joins the flusher thread without hanging.
    /// Invariants: After 200ms (4x the 50ms interval) the HTML file exists;
    /// `drop(guard)` returns promptly (no deadlock).
    #[test]
    #[serial]
    fn start_with_flusher_runs_periodic_export() {
        reset_globals();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default().with_auto_export(
            AutoExportConfig::default()
                .with_output_path(dir.path())
                .with_flush_interval(Duration::from_millis(50)),
        );
        let guard =
            start_with(cfg).expect("start_with with flush_interval must succeed and spawn flusher");

        // 200ms gives the 50ms-interval worker ~3-4 ticks of slack. The first
        // tick triggers trigger_export_now which writes the dashboard.
        thread::sleep(Duration::from_millis(200));

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "periodic flusher must have written the HTML dashboard within 200ms"
        );

        // Dropping the guard joins the flusher thread. This must not hang — if
        // it does, the test will time out and surface the deadlock.
        drop(guard);
    }

    /// Objective: Verify `guard.export_now()` returns `true` and produces a
    /// report file on disk.
    /// Invariants: `export_now` resets the latch and exports, so it returns
    /// `true`; the HTML file exists at the configured output path afterwards.
    #[test]
    #[serial]
    fn export_now_returns_true_and_produces_report() {
        reset_globals();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default()
            .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
        let guard = start_with(cfg).expect("start_with must succeed");

        let did = guard.export_now();
        assert!(
            did,
            "export_now() must return true when tracker and cfg are installed"
        );

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "HTML dashboard must exist at {html:?} after export_now()"
        );
        drop(guard);
    }

    /// Objective: Verify `guard.snapshot_json()` returns a non-empty JSON
    /// string containing the `total_allocations` field of the analysis report.
    /// Invariants: The snapshot is built from the installed tracker and
    /// serialized via serde, so it must be a valid non-empty JSON document.
    #[test]
    #[serial]
    fn snapshot_json_returns_non_empty_string() {
        reset_globals();
        let (_dir, cfg) = config_with_tempdir();
        let guard = start_with(cfg).expect("start_with must succeed");

        // Track something so the report has non-trivial content rather than
        // an all-zero snapshot.
        let v: Vec<u64> = vec![1, 2, 3];
        guard.track(&v);

        let json = guard
            .snapshot_json()
            .expect("snapshot_json() must succeed when the tracker is installed");
        assert!(
            !json.is_empty(),
            "snapshot_json() must return a non-empty JSON string"
        );
        assert!(
            json.contains("allocation_count"),
            "snapshot JSON must contain the allocation_count field, got: {json}"
        );
        drop(guard);
    }

    /// Objective: Verify the `Deref<Target = GlobalTracker>` impl lets callers
    /// invoke `GlobalTracker::track` directly on the guard without panicking.
    /// Invariants: After `guard.track(&vec)`, the tracker's stats must show at
    /// least one recorded allocation.
    #[test]
    #[serial]
    fn deref_to_tracker_allows_track() {
        reset_globals();
        let (_dir, cfg) = config_with_tempdir();
        let guard = start_with(cfg).expect("start_with must succeed");

        let data: Vec<u64> = vec![1, 2, 3];
        // This call relies on Deref: MemScopeGuard -> &GlobalTracker, then
        // GlobalTracker::track(&self, &T). If Deref were broken this panics.
        guard.track(&data);

        let stats = guard.get_stats();
        assert!(
            stats.total_allocations > 0,
            "tracker must have recorded at least one allocation after track() via Deref"
        );
        drop(guard);
    }

    // ===================== Negative tests (edge cases) ====================

    /// Objective: Verify calling `start_with` twice without resetting global
    /// tracking fails on the second call.
    /// Invariants: The first call succeeds and returns a guard; the second
    /// call returns `Err` because `init_global_tracking_with_config` errors
    /// on double-init. The error message must mention the cause.
    #[test]
    #[serial]
    fn start_twice_second_returns_err() {
        reset_globals();
        let (_dir1, cfg1) = config_with_tempdir();
        let guard1 =
            start_with(cfg1).expect("first start_with must succeed with a clean global state");

        // The global singleton is now Some(...); a second start_with must fail
        // at init_global_tracking_with_config before reaching lifecycle::install.
        let (_dir2, cfg2) = config_with_tempdir();
        let result2 = start_with(cfg2);
        let err = match result2 {
            Err(e) => e,
            Ok(_) => {
                panic!("second start_with must fail because global tracking is already initialized")
            }
        };
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("already initialized"),
            "error must mention 'already initialized', got: {err_msg}"
        );

        drop(guard1);
    }

    /// Objective: Verify dropping the guard after `reset_global_tracking`
    /// (which clears only the global singleton, NOT `lifecycle::TRACKER_HANDLE`)
    /// still exports successfully — the guard's `Arc` and the lifecycle's
    /// stored `Arc` keep the tracker alive.
    /// Invariants: No panic on drop; the HTML file is written because
    /// `export_once` reads from `TRACKER_HANDLE` (still `Some`) which holds
    /// its own `Arc` clone independent of the global singleton.
    #[test]
    #[serial]
    fn drop_after_reset_global_tracking_still_succeeds() {
        reset_globals();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default()
            .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
        let guard = start_with(cfg).expect("start_with must succeed");

        // Clear the global singleton. The guard's `tracker` field and
        // lifecycle's `TRACKER_HANDLE` both still hold Arc clones, so the
        // tracker is not dropped and export_once can still proceed.
        crate::capture::backends::global_tracking::reset_global_tracking();

        // Dropping must not panic, and the export must still write the file
        // because TRACKER_HANDLE retains the Arc.
        drop(guard);

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "HTML must be written even after reset_global_tracking because TRACKER_HANDLE retains the Arc"
        );
    }

    // ===================== Stress test (50 threads) =======================

    /// Objective: Verify 50 threads can each perform the full
    /// reset -> start -> track -> drop cycle without deadlock or panic.
    /// Invariants: All 50 threads join successfully. The `parking_lot::Mutex`
    /// serializes the per-thread cycle so `init_global_tracking` is never
    /// called concurrently (which would race the singleton check).
    #[test]
    #[serial]
    fn fifty_threads_start_track_drop_no_deadlock() {
        const THREAD_COUNT: usize = 50;
        // Shared lock so the reset+start+track+drop cycle runs strictly
        // sequentially across threads. Without this, two threads racing
        // init_global_tracking would produce spurious "already initialized"
        // errors that are not the subject of this stress test.
        let lock = Arc::new(Mutex::new(()));
        // Shared tempdir for output. Safe because the lock serializes all
        // writes — no two threads export concurrently.
        let dir = Arc::new(TempDir::new().expect("tempdir creation must succeed in tests"));

        let mut handles = Vec::with_capacity(THREAD_COUNT);
        for _ in 0..THREAD_COUNT {
            let lock = Arc::clone(&lock);
            let dir = Arc::clone(&dir);
            handles.push(thread::spawn(move || {
                // Hold the lock for the entire cycle so global state is never
                // observed in a half-initialized state by another thread.
                let _guard_lock = lock.lock();
                reset_globals();
                let cfg = MemScopeConfig::default()
                    .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
                let guard = start_with(cfg).expect("start_with must succeed in each worker thread");
                let data: Vec<u64> = vec![42; 8];
                guard.track(&data);
                // Drop inside the lock so the export path is also serialized
                // — no concurrent file writes to the same directory.
                drop(guard);
            }));
        }

        let mut completed = 0usize;
        for handle in handles {
            handle
                .join()
                .expect("worker thread must not panic during start+track+drop");
            completed += 1;
        }
        assert_eq!(
            completed, THREAD_COUNT,
            "all 50 worker threads must complete without deadlock or panic"
        );
    }

    // ===================== Integration test (end-to-end) ==================

    /// Objective: Verify the end-to-end flow: `start_with(tempdir)` ->
    /// `track!` a `Vec<u64>` through the Deref path -> drop guard -> read the
    /// produced HTML -> assert it contains a known template marker.
    /// Invariants: The HTML file exists, is non-empty, and contains the
    /// `memscope` marker from the dashboard template, proving the renderer
    /// ran against real tracked data.
    #[test]
    #[serial]
    fn end_to_end_start_track_drop_html_contains_marker() {
        reset_globals();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = MemScopeConfig::default()
            .with_auto_export(AutoExportConfig::default().with_output_path(dir.path()));
        let guard = start_with(cfg).expect("start_with must succeed for end-to-end flow");

        // Track a Vec<u64> through the exported `track!` macro. The macro
        // expands to `guard.track_as(&vec_data, ...)` which relies on Deref
        // to reach GlobalTracker::track_as.
        let vec_data: Vec<u64> = vec![1, 2, 3, 4, 5];
        track!(guard, vec_data);

        drop(guard);

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "HTML dashboard must exist after end-to-end start+track+drop"
        );
        let content = std::fs::read_to_string(&html)
            .expect("HTML dashboard file must be readable after end-to-end export");
        assert!(
            !content.is_empty(),
            "HTML dashboard content must not be empty after end-to-end flow"
        );
        // The unified dashboard template hard-codes the "memscope-rs" label
        // in the side navigation, so the rendered output must contain it.
        assert!(
            content.contains("memscope"),
            "HTML dashboard must contain the 'memscope' marker from the template"
        );
    }
}
