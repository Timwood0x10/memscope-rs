//! Lifecycle orchestration for the auto-export subsystem.
//!
//! This is module 3 of the 4-module auto-export feature. It provides the
//! single idempotent [`export_once`] entry point that every exit path (the
//! Drop guard, the panic hook, the Ctrl-C handler, and the `atexit` callback)
//! funnels into, plus the one-time installation logic for those hooks.
//!
//! # Why this module is critical for `panic = "abort"` release builds
//!
//! `memscope-rs` release profiles set `panic = "abort"`. Under abort semantics
//! Rust destructors do NOT run when a panic occurs, so a plain Drop guard
//! cannot flush a report on panic. The Rust runtime DOES invoke the registered
//! panic hook before aborting, however, so installing a panic hook that calls
//! [`export_once`] is the only mechanism that produces a report on a release
//! panic. The hook is therefore installed unconditionally (not gated behind a
//! feature flag) and is carefully re-entrancy-safe: [`export_once`] swaps
//! [`EXPORTED`] to `true` BEFORE doing any work, so a panic raised by the
//! export itself cannot cause the re-entrant panic-hook invocation to loop.
//!
//! # Concurrency model
//!
//! - [`EXPORTED`] is an [`AtomicBool`] used as a once-only latch. It is set
//!   via `swap(true, SeqCst)` before the export work begins so that re-entrant
//!   invocations (e.g. a panic inside the panic hook) see it already set and
//!   return immediately.
//! - [`TRACKER_HANDLE`] and [`AUTO_EXPORT_CFG`] live in `parking_lot::RwLock`
//!   (not `OnceLock`) so tests can reset them between cases. The exit-path
//!   handlers clone the values out and drop the read guards before the
//!   potentially slow file I/O, so no lock is held across the export.
//! - [`HOOKS_INSTALLED`] is a `std::sync::Once` that guarantees the panic /
//!   ctrlc / atexit hooks are registered exactly once per process.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once};
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use tracing::{error, info, warn};

use crate::auto_export::{AutoExportConfig, SignalPolicy};
use crate::capture::backends::global_tracking::GlobalTracker;
use crate::core::{MemScopeError, MemScopeResult};

/// Idempotency guard: once an exit-path export has run, no other exit path
/// re-exports. Set to `true` BEFORE the export work begins (via `swap`) so a
/// panic during export does not clear it and cause re-entrant export inside
/// the panic hook (which would recurse without this guard).
static EXPORTED: AtomicBool = AtomicBool::new(false);

/// Serializes the actual [`do_export`] work so that a [`trigger_export_now`]
/// resetting the idempotency latch while an export is in-flight cannot cause
/// two concurrent `do_export` calls to clobber the same output files. The
/// lock is held ONLY across `do_export` (not across the CAS or the tracker/cfg
/// reads), so no-op callers (which lose the CAS) never block on this mutex.
static EXPORT_MUTEX: Mutex<()> = Mutex::new(());

/// The tracker to export. Stored in a `RwLock<Option<...>>` (not `OnceLock`)
/// so tests can reset it between cases. `Arc<GlobalTracker>` because the
/// panic/ctrlc/atexit handlers need to read it without holding a borrow across
/// the (long, I/O-bound) export.
static TRACKER_HANDLE: RwLock<Option<Arc<GlobalTracker>>> = RwLock::new(None);

/// The config captured at [`install`] time. Read by the signal/panic/atexit
/// handlers which have no other way to receive arguments.
static AUTO_EXPORT_CFG: RwLock<Option<AutoExportConfig>> = RwLock::new(None);

/// Guards one-time installation of the panic/ctrlc/atexit hooks. `Once` is
/// appropriate because re-installing a panic hook would chain onto the
/// already-chained hook (harmless but wasteful), and `ctrlc::set_handler`
/// returns `Err` if a handler is already set.
static HOOKS_INSTALLED: Once = Once::new();

// Thread-local re-entrancy guard for the panic hook. When `true`, the current
// thread is already inside `install_panic_hook`'s closure (i.e. its own
// `export_for_reason` call panicked, re-entering the hook). The recursive
// invocation skips the body entirely so the chained `prev` hook is called
// exactly once — with the user's original panic, not the export panic.
thread_local! {
    static IN_PANIC_HOOK: Cell<bool> = const { Cell::new(false) };
}

/// Which exit path triggered an export. Used by [`export_for_reason`] to
/// consult the per-path enable flags (`on_exit`, `on_panic`) in
/// [`AutoExportConfig`] BEFORE consuming the one-shot idempotency latch, so a
/// disabled path does not prevent a later enabled path from exporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportReason {
    /// Triggered by `MemScopeGuard::drop` (normal return from `main`).
    /// Gated by [`AutoExportConfig::on_exit`].
    Drop,
    /// Triggered by the panic hook. Gated by [`AutoExportConfig::on_panic`].
    Panic,
    /// Triggered by `trigger_export_now` (on-demand or periodic flusher).
    /// Always allowed — the user explicitly asked for an export.
    OnDemand,
    /// Triggered by a signal handler (Ctrl-C / SIGTERM). The signal policy
    /// is checked separately inside the ctrlc handler, so this is always
    /// allowed once the handler has decided to proceed.
    Signal,
}

// =========================================================================
// Public API
// =========================================================================

/// Store the tracker + config and install all enabled exit hooks (panic hook
/// always; ctrlc handler if the `auto-signal` feature is enabled; atexit hook
/// if the `atexit` feature is enabled).
///
/// Idempotent: calling twice is safe — the tracker/cfg are overwritten (last
/// wins) and [`HOOKS_INSTALLED`] ensures hooks install only once per process.
///
/// # Errors
///
/// Returns [`MemScopeError`] only if storing the tracker/cfg fails. With
/// `parking_lot` (which never poisons) this does not happen in practice, but
/// the `Result` is preserved so callers can use `?` uniformly.
pub fn install(cfg: AutoExportConfig, tracker: Arc<GlobalTracker>) -> MemScopeResult<()> {
    // 1. Write cfg + tracker into the RwLocks. parking_lot write guards never
    //    poison, so these assignments are effectively infallible.
    *AUTO_EXPORT_CFG.write() = Some(cfg);
    *TRACKER_HANDLE.write() = Some(tracker);
    // 2. Install hooks exactly once per process. Re-running install() after
    //    the first call skips this block entirely.
    HOOKS_INSTALLED.call_once(|| {
        install_panic_hook();
        #[cfg(feature = "auto-signal")]
        install_ctrlc_handler();
        #[cfg(feature = "atexit")]
        install_atexit();
    });
    Ok(())
}

/// Idempotent export: writes HTML and/or JSON to the configured output path.
/// Called by the on-demand path (`trigger_export_now`, periodic flusher).
///
/// Equivalent to [`export_for_reason`]`(ExportReason::OnDemand)`. The exit
/// paths (Drop, panic, signal) call [`export_for_reason`] directly so their
/// per-path enable flags (`on_exit`, `on_panic`) are consulted.
///
/// Returns `true` if THIS call performed the export; `false` if a prior call
/// already exported, or if no tracker/cfg is installed.
///
/// # Re-entrancy safety
///
/// [`EXPORTED`] is set to `true` via `swap` BEFORE the export work begins. If
/// the export itself panics, the re-entrant panic-hook invocation sees
/// [`EXPORTED`] already `true` and returns `false` — no infinite recursion.
pub fn export_once() -> bool {
    export_for_reason(ExportReason::OnDemand)
}

/// Reason-aware idempotent export. Like [`export_once`] but consults the
/// per-path enable flag in [`AutoExportConfig`] before consuming the
/// idempotency latch, so a disabled exit path (e.g. `on_exit: false`) does NOT
/// prevent a later enabled path (e.g. `on_panic: true`) from exporting.
///
/// # Returns
///
/// `true` if THIS call performed the export; `false` if a prior call already
/// exported, the path is disabled, or no tracker/cfg is installed.
pub fn export_for_reason(reason: ExportReason) -> bool {
    // Read tracker + cfg WITHOUT holding the read guards during the export.
    // Cloning an Arc is a single atomic increment; cloning the config is a
    // small allocation that is acceptable here and keeps the critical section
    // minimal.
    let tracker = TRACKER_HANDLE.read().clone();
    let cfg = AUTO_EXPORT_CFG.read().clone();
    let (Some(tracker), Some(cfg)) = (tracker, cfg) else {
        // Not installed — nothing to export. Do NOT set EXPORTED so a later
        // install + export can still proceed.
        return false;
    };
    // Check the per-path enable flag BEFORE the CAS. A disabled path must not
    // consume the one-shot latch, otherwise a later enabled path would see
    // EXPORTED==true and skip its own export.
    let enabled = match reason {
        ExportReason::Drop => cfg.on_exit,
        ExportReason::Panic => cfg.on_panic,
        // On-demand and signal paths are always allowed. The signal policy is
        // checked separately inside the ctrlc handler before it reaches here.
        ExportReason::OnDemand | ExportReason::Signal => true,
    };
    if !enabled {
        return false;
    }
    // CAS the idempotency guard. swap returns the OLD value; if old was true,
    // someone else already exported.
    if EXPORTED.swap(true, Ordering::SeqCst) {
        return false;
    }
    // Serialize the actual file I/O so a concurrent `trigger_export_now`
    // (which resets EXPORTED) cannot overlap a second `do_export` on the same
    // output files. The mutex is held only across do_export; CAS losers never
    // reach here, so no-op callers never block.
    let _export_guard = EXPORT_MUTEX.lock();
    do_export(&tracker, &cfg)
}

/// On-demand export from anywhere in user code. Resets [`EXPORTED`] then calls
/// [`export_once`], so it always exports AND re-arms the exit-path idempotency
/// guard (a later panic/Ctrl-C will produce a fresh report).
///
/// Returns `true` if the export ran.
pub fn trigger_export_now() -> bool {
    // Re-arm the latch first so the subsequent export_once CAS wins.
    EXPORTED.store(false, Ordering::SeqCst);
    export_once()
}

/// In-memory JSON snapshot (no disk write). Builds an [`AnalysisReport`] via
/// [`Analyzer`] and serializes it. Intended for HTTP endpoints that want live
/// metrics without touching the filesystem.
///
/// # Errors
///
/// Returns [`MemScopeError`] if no tracker is installed or serialization fails.
///
/// [`AnalysisReport`]: crate::analyzer::AnalysisReport
/// [`Analyzer`]: crate::analyzer::Analyzer
pub fn snapshot_json() -> MemScopeResult<String> {
    let tracker = TRACKER_HANDLE.read().clone();
    let Some(tracker) = tracker else {
        return Err(MemScopeError::error(
            "lifecycle",
            "snapshot_json",
            "No tracker installed; call start() or install() first",
        ));
    };
    // The analyzer pipeline (MemoryView -> Analyzer -> analyze) produces an
    // AnalysisReport that derives serde::Serialize, so we can serialize it
    // directly without any disk I/O. This avoids the tempdir+export_json
    // fallback entirely and keeps snapshot_json side-effect free.
    let mut analyzer = crate::analyzer::Analyzer::from_tracker(&tracker);
    let report = analyzer.analyze();
    serde_json::to_string(&report).map_err(|e| {
        MemScopeError::error(
            "lifecycle",
            "snapshot_json",
            format!("Failed to serialize analysis report: {e}"),
        )
    })
}

// =========================================================================
// Private helpers
// =========================================================================

/// Core export logic, separated so it can be unit-tested with a real tracker
/// + tempdir WITHOUT touching the global statics.
///
/// Returns `true` iff at least one selected format was written successfully.
/// If `cfg.formats` is empty, returns `false` immediately without touching the
/// filesystem. If the output directory cannot be created, returns `false`
/// after logging (no format can write).
fn do_export(tracker: &GlobalTracker, cfg: &AutoExportConfig) -> bool {
    // Nothing to do if no formats are selected. Returning early here keeps
    // the filesystem untouched and avoids creating empty report directories.
    if cfg.formats.is_empty() {
        return false;
    }
    let output = &cfg.output_path;
    // Pre-create the output directory. `create_dir_all` is idempotent (it
    // treats AlreadyExists as success), so this is safe even if the export
    // functions also create it. If creation fails — e.g. a path component is
    // a regular file rather than a directory — no format can write, so log
    // and bail gracefully instead of letting each export fail individually.
    if let Err(e) = std::fs::create_dir_all(output) {
        error!(
            target: "memscope::lifecycle",
            error = %e,
            path = ?output,
            "failed to create output directory; skipping export",
        );
        return false;
    }
    let mut ok = false;
    // HTML first. A failure here is logged but does NOT abort the JSON write,
    // so a partial report is still produced when one renderer breaks.
    if cfg.wants_html() {
        if let Err(e) = tracker.export_html(output) {
            error!(
                target: "memscope::lifecycle",
                error = %e,
                "HTML export failed; continuing to JSON if requested",
            );
        } else {
            ok = true;
        }
    }
    if cfg.wants_json() {
        if let Err(e) = tracker.export_json(output) {
            error!(
                target: "memscope::lifecycle",
                error = %e,
                "JSON export failed",
            );
        } else {
            ok = true;
        }
    }
    ok
}

/// Install a panic hook that calls [`export_for_reason`]`(ExportReason::Panic)`
/// then chains to the previous hook. Runs even under `panic = "abort"` because
/// the Rust runtime invokes the panic hook before aborting.
///
/// The export is wrapped in [`catch_unwind`] so that a panic raised by the
/// export itself cannot propagate as a double-panic (which would skip the
/// chained hook). A thread-local re-entrancy guard ([`IN_PANIC_HOOK`]) ensures
/// the recursive hook invocation — triggered by a panic inside the export —
/// skips the body entirely, so the chained `prev` hook is called exactly once
/// (with the user's original panic, not the export panic).
fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Re-entrancy guard: if we are already inside this hook (i.e. our own
        // export_for_reason call panicked, re-entering the hook), skip the
        // body entirely. The outer invocation will call prev(info) with the
        // user's ORIGINAL panic — the one they care about.
        if IN_PANIC_HOOK.replace(true) {
            return;
        }
        // Best-effort export. catch_unwind contains any panic raised by the
        // export path so the chained (default) hook still prints the message.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = export_for_reason(ExportReason::Panic);
        }));
        IN_PANIC_HOOK.set(false);
        // Chain to the previous hook (default prints the panic location/message).
        prev(info);
    }));
}

/// Install a ctrlc handler that calls [`export_for_reason`]`(ExportReason::Signal)`
/// on SIGINT. Degrades gracefully if a handler is already installed by the host
/// application.
///
/// The policy is read **inside** the handler (at signal time) rather than
/// at installation time, so a second [`install`] that updates
/// [`AUTO_EXPORT_CFG`] but is skipped by the `ONCE` guard still takes
/// effect from the next signal onward.
///
/// # Termination behaviour
///
/// After running the export, the handler acquires [`EXPORT_MUTEX`] (with a
/// timeout of [`AutoExportConfig::exit_timeout`]) so any in-flight `do_export`
/// on another thread can finish before `std::process::exit(130)` truncates it.
/// This is necessary because the `ctrlc` crate REPLACES the OS default SIGINT
/// handler — once we register a handler, the default "terminate the process"
/// behaviour no longer fires. Hosts that want to keep running after SIGINT
/// (e.g. a web server doing graceful shutdown) should set
/// [`SignalPolicy::Off`] and install their own handler.
///
/// If the `atexit` feature is also enabled, the registered `atexit` callback
/// will still run after `std::process::exit(130)`. The idempotency latch in
/// [`export_for_reason`] makes this double-invocation safe (the second call is
/// a no-op).
#[cfg(feature = "auto-signal")]
fn install_ctrlc_handler() {
    // ctrlc::set_handler returns Err if a handler is already registered (for
    // example by the host application). Degrade gracefully: log a warning and
    // keep Drop + panic-hook coverage.
    match ctrlc::set_handler(|| {
        let (policy, exit_timeout) = AUTO_EXPORT_CFG
            .read()
            .as_ref()
            .map(|c| (c.on_signal, c.exit_timeout))
            .unwrap_or((SignalPolicy::Off, Duration::from_secs(5)));
        if policy == SignalPolicy::Off {
            return;
        }
        let _ = export_for_reason(ExportReason::Signal);
        // Wait for any in-flight do_export on another thread to finish before
        // yanking the rug out from under it with process::exit. try_lock_for
        // returns None on timeout; in that case we exit anyway (best-effort)
        // rather than hang forever. This turns a "truncated file" race into a
        // "complete file, then exit". The guard is held until the (noreturn)
        // process::exit call below, which is fine because the process is
        // terminating anyway.
        let _export_lock = EXPORT_MUTEX.try_lock_for(exit_timeout);
        // The ctrlc crate REPLACES the OS default SIGINT handler, so the
        // process would otherwise keep running. Exit with 130 (the
        // conventional 128+SIGINT code) so the host sees a SIGINT-style exit
        // and the shell reports "Interrupt: 13" / exit code 130. export_once
        // is idempotent, so the Drop guard (if it ever runs) is a no-op.
        std::process::exit(130);
    }) {
        Ok(()) => info!(
            target: "memscope::lifecycle",
            "ctrlc handler installed for auto-export",
        ),
        Err(e) => warn!(
            target: "memscope::lifecycle",
            error = %e,
            "failed to install ctrlc handler (host may have one already); \
             auto-export on Ctrl-C disabled, Drop + panic-hook still active",
        ),
    }
}

/// Register an `atexit` handler that calls [`export_once`] on
/// `std::process::exit`. This is the ONLY way to catch `std::process::exit`
/// because Rust destructors do not run on explicit exit, but C `atexit`
/// handlers do.
///
/// # Safety
///
/// `libc::atexit` is an unsafe FFI call. The registered function must match
/// the `extern "C" fn()` signature and must not capture state. `on_exit`
/// reads only from process-global `static`s which outlive the call, and the
/// process is exiting anyway, so the (non-async-signal-safe) file I/O inside
/// [`export_once`] is acceptable here.
#[cfg(feature = "atexit")]
fn install_atexit() {
    extern "C" fn on_exit() {
        let _ = export_once();
    }
    // SAFETY: `on_exit` is an `extern "C" fn` with the correct signature for
    // `libc::atexit` (`extern "C" fn()`). It captures no state and only reads
    // process-global statics. The call registers the callback and returns 0 on
    // success or non-zero on failure (e.g. too many handlers registered).
    let rc = unsafe { libc::atexit(on_exit) };
    if rc != 0 {
        warn!(
            target: "memscope::lifecycle",
            rc,
            "libc::atexit registration failed; auto-export on std::process::exit disabled",
        );
    }
}

/// Test-only reset of the global statics. NOT available in non-test builds.
/// Use `#[serial]` (serial_test) on any test that calls this so concurrent
/// tests do not clobber each other's state.
#[cfg(test)]
pub(crate) fn reset_for_test() {
    EXPORTED.store(false, Ordering::SeqCst);
    *TRACKER_HANDLE.write() = None;
    *AUTO_EXPORT_CFG.write() = None;
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_export::{ExportFormatSet, SignalPolicy};
    use proptest::prelude::*;
    use proptest::test_runner::TestRunner;
    use serial_test::serial;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Build a fresh `GlobalTracker` wrapped in an `Arc`. Centralized so every
    /// test starts from a known-clean tracker. `GlobalTracker::new()` also
    /// registers a global async-tracker singleton; tests are `#[serial]` so
    /// this side effect cannot race with another test.
    fn fresh_tracker() -> Arc<GlobalTracker> {
        Arc::new(GlobalTracker::new())
    }

    // ===================== Positive tests (happy path) ====================

    /// Objective: Verify `do_export` writes both HTML and JSON artifacts when
    /// `HTML_JSON` is selected, using a real tracker and a tempdir.
    /// Invariants: `dashboard_unified_dashboard.html` and `memory_analysis.json`
    /// exist after the call, and `do_export` returns `true`.
    #[test]
    #[serial]
    fn do_export_writes_both_formats() {
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default().with_output_path(dir.path());
        let cfg = cfg.with_formats(ExportFormatSet::HTML_JSON);

        let did = do_export(&tracker, &cfg);
        assert!(did, "do_export must return true when both formats succeed");

        let html = dir.path().join("dashboard_unified_dashboard.html");
        let json = dir.path().join("memory_analysis.json");
        assert!(
            html.exists(),
            "HTML dashboard file must exist at {html:?} after a successful HTML export",
        );
        assert!(
            json.exists(),
            "memory_analysis.json must exist at {json:?} after a successful JSON export",
        );
    }

    /// Objective: Verify `export_once` is idempotent — the first call after
    /// `install` exports, the second is a no-op.
    /// Invariants: First call returns `true`, second returns `false`; the
    /// output directory is written exactly once.
    #[test]
    #[serial]
    fn export_once_is_idempotent() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default().with_output_path(dir.path());
        install(cfg, tracker).expect("install must not fail with parking_lot locks");

        let first = export_once();
        assert!(
            first,
            "first export_once after install must perform the export",
        );
        let second = export_once();
        assert!(
            !second,
            "second export_once must be a no-op because EXPORTED is already set",
        );

        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "the single export must have written the HTML dashboard",
        );
    }

    /// Objective: Verify `trigger_export_now` re-arms the idempotency latch
    /// and produces a fresh export, even after a prior `export_once`.
    /// Invariants: `trigger_export_now` returns `true`; a subsequent
    /// `export_once` returns `false` (latch re-set by trigger).
    #[test]
    #[serial]
    fn trigger_export_now_re_arms_latch() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default().with_output_path(dir.path());
        install(cfg, tracker).expect("install must succeed");

        // Consume the one-shot export so EXPORTED is true.
        let _ = export_once();

        let triggered = trigger_export_now();
        assert!(
            triggered,
            "trigger_export_now must reset EXPORTED and run a fresh export",
        );
        // After trigger, the latch is set again, so a plain export_once is a no-op.
        let after = export_once();
        assert!(
            !after,
            "export_once after trigger_export_now must be a no-op until the latch is reset again",
        );
    }

    /// Objective: Verify `snapshot_json` returns a non-empty serialized report
    /// containing the allocation-count field when a tracker is installed.
    /// Invariants: Result is `Ok`, string is non-empty and contains
    /// `"allocation_count"` (a field on `MemoryStatsReport`).
    #[test]
    #[serial]
    fn snapshot_json_returns_serialized_report() {
        reset_for_test();
        let tracker = fresh_tracker();
        let cfg = AutoExportConfig::default();
        install(cfg, tracker).expect("install must succeed");

        let snap = snapshot_json();
        assert!(
            snap.is_ok(),
            "snapshot_json must succeed when a tracker is installed: {:?}",
            snap.err(),
        );
        let snap = snap.expect("checked Ok above");
        assert!(
            !snap.is_empty(),
            "serialized snapshot must not be the empty string",
        );
        assert!(
            snap.contains("allocation_count"),
            "snapshot must contain the allocation_count field; got: {snap}",
        );
    }

    // ===================== Negative tests (edge cases) ====================

    /// Objective: Verify `export_once` is safe when nothing is installed.
    /// Invariants: Returns `false`, does not panic, and does NOT set EXPORTED
    /// (so a later install+export can still proceed).
    #[test]
    #[serial]
    fn export_once_without_install_returns_false() {
        reset_for_test();
        let did = export_once();
        assert!(
            !did,
            "export_once with no tracker/cfg installed must return false",
        );
        assert!(
            !EXPORTED.load(Ordering::SeqCst),
            "EXPORTED must remain false so a later install can still export",
        );
    }

    /// Objective: Verify `do_export` does nothing when no formats are selected.
    /// Invariants: Returns `false` and writes no files to the output directory.
    #[test]
    #[serial]
    fn do_export_empty_formats_writes_nothing() {
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig {
            output_path: dir.path().to_path_buf(),
            formats: ExportFormatSet::from_bits(0),
            ..AutoExportConfig::default()
        };

        let did = do_export(&tracker, &cfg);
        assert!(
            !did,
            "do_export with empty formats must return false (nothing to do)",
        );
        let entries = std::fs::read_dir(dir.path())
            .expect("output dir must be readable")
            .count();
        assert_eq!(entries, 0, "no files must be written when formats is empty",);
    }

    /// Objective: Verify `do_export` creates a deeply nested output path via
    /// `create_dir_all` and then writes the report into it.
    /// Invariants: Returns `true`; the nested directory and both report files
    /// exist after the call.
    #[test]
    #[serial]
    fn do_export_creates_deeply_nested_output_path() {
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let deep = dir.path().join("a/b/c/d/e/report");
        let cfg = AutoExportConfig::default().with_output_path(deep.clone());

        let did = do_export(&tracker, &cfg);
        assert!(
            did,
            "do_export must succeed after creating the nested output directory",
        );
        assert!(
            deep.is_dir(),
            "the nested output directory must have been created",
        );
        assert!(
            deep.join("dashboard_unified_dashboard.html").exists(),
            "HTML dashboard must exist inside the nested output path",
        );
        assert!(
            deep.join("memory_analysis.json").exists(),
            "memory_analysis.json must exist inside the nested output path",
        );
    }

    /// Objective: Verify `do_export` fails gracefully (no panic) when the
    /// output path cannot be created because a path component is a regular
    /// file rather than a directory.
    /// Invariants: Returns `false`; no panic; the blocker file is untouched.
    #[test]
    #[serial]
    fn do_export_path_under_a_file_fails_gracefully() {
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        // Create a regular file, then try to use a path underneath it as a dir.
        let blocker = dir.path().join("blocker");
        std::fs::write(&blocker, b"not a directory").expect("blocker file write must succeed");
        let bad_output = blocker.join("sub");

        let cfg = AutoExportConfig {
            output_path: bad_output.clone(),
            formats: ExportFormatSet::HTML_JSON,
            ..AutoExportConfig::default()
        };

        let did = do_export(&tracker, &cfg);
        assert!(
            !did,
            "do_export must return false when create_dir_all fails (path under a file)",
        );
        // The blocker file must still be a file (not clobbered into a directory).
        assert!(
            blocker.is_file(),
            "the blocking file must remain a file after the failed export",
        );
    }

    /// Objective: Verify `snapshot_json` returns `Err` (not a panic) when no
    /// tracker is installed.
    /// Invariants: Result is `Err` with a lifecycle error category.
    #[test]
    #[serial]
    fn snapshot_json_without_tracker_errors() {
        reset_for_test();
        let res = snapshot_json();
        let err = res.expect_err("snapshot_json must return Err when no tracker is installed");
        assert_eq!(
            err.category(),
            "analysis",
            "snapshot_json error must classify as an analysis error (module 'lifecycle')",
        );
    }

    // ===================== Stress / concurrency tests =====================

    /// Objective: Verify that 50 concurrent `export_once` calls result in
    /// EXACTLY one export (idempotency latch holds under contention).
    /// Invariants: Exactly one thread returns `true`; exactly one HTML file
    /// and the primary JSON file exist after all threads join.
    #[test]
    #[serial]
    fn stress_50_concurrent_exports_one_wins() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default().with_output_path(dir.path());
        install(cfg, tracker).expect("install must succeed");

        let wins = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(50);
        for _ in 0..50 {
            let wins = wins.clone();
            handles.push(std::thread::spawn(move || {
                if export_once() {
                    wins.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }
        for h in handles {
            h.join()
                .expect("worker threads must not panic during the stress test");
        }

        assert_eq!(
            wins.load(Ordering::SeqCst),
            1,
            "exactly one of the 50 concurrent calls must perform the export",
        );

        // Count HTML files: export_html writes exactly one (dashboard_unified_dashboard.html),
        // so a count of 1 confirms no duplicate concurrent writes.
        let html_count = std::fs::read_dir(dir.path())
            .expect("output dir must be readable")
            .filter_map(Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "html")
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(
            html_count, 1,
            "exactly one HTML file must exist after the concurrent exports",
        );
        assert!(
            dir.path().join("memory_analysis.json").exists(),
            "the primary JSON file must exist after the winning export",
        );
    }

    /// Objective: Verify that 20 concurrent `trigger_export_now` calls (which
    /// reset the idempotency latch) do NOT produce corrupted/truncated report
    /// files. This is the regression test for the `EXPORT_MUTEX` fix: without
    /// the mutex serializing `do_export`, two concurrent triggers would reset
    /// EXPORTED mid-flight and overlap two `do_export` calls on the same files.
    /// Invariants: All threads join; the HTML file is non-empty and contains
    /// the `memscope` template marker (proving it is not truncated).
    #[test]
    #[serial]
    fn stress_concurrent_trigger_export_now_no_corrupted_writes() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default().with_output_path(dir.path());
        install(cfg, tracker).expect("install must succeed");

        let mut handles = Vec::with_capacity(20);
        for _ in 0..20 {
            handles.push(std::thread::spawn(|| {
                let _ = trigger_export_now();
            }));
        }
        for h in handles {
            h.join().expect("trigger_export_now worker must not panic");
        }

        // After all concurrent triggers settle, the HTML file must be valid
        // (non-empty and not truncated by a concurrent writer). A truncated
        // write would either be empty or missing the template marker.
        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert!(
            html.exists(),
            "HTML dashboard must exist after concurrent trigger_export_now calls",
        );
        let content = std::fs::read_to_string(&html)
            .expect("HTML dashboard must be readable after concurrent exports");
        assert!(
            !content.is_empty(),
            "HTML must not be empty/truncated by a concurrent writer",
        );
        assert!(
            content.contains("memscope"),
            "HTML must contain the 'memscope' template marker (not truncated mid-write)",
        );
    }

    // ============== Per-path flag gating (on_exit / on_panic) ==============

    /// Objective: Verify `export_for_reason(Drop)` honors `on_exit: false` —
    /// the Drop path is skipped and does NOT consume the idempotency latch, so
    /// a later `trigger_export_now` can still export.
    /// Invariants: `export_for_reason(Drop)` returns `false`; `EXPORTED` stays
    /// `false`; `trigger_export_now` then returns `true`.
    #[test]
    #[serial]
    fn drop_reason_respects_on_exit_false() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default()
            .with_output_path(dir.path())
            .with_on_exit(false);
        install(cfg, tracker).expect("install must succeed");

        let did = export_for_reason(ExportReason::Drop);
        assert!(
            !did,
            "export_for_reason(Drop) must return false when on_exit is false",
        );
        assert!(
            !EXPORTED.load(Ordering::SeqCst),
            "EXPORTED must remain false so a disabled path does not block later exports",
        );

        // A subsequent on-demand export must still succeed because the latch
        // was not consumed.
        let on_demand = trigger_export_now();
        assert!(
            on_demand,
            "trigger_export_now must succeed after the disabled Drop path",
        );
    }

    /// Objective: Verify `export_for_reason(Panic)` honors `on_panic: false` —
    /// the Panic path is skipped and does NOT consume the idempotency latch.
    /// Invariants: `export_for_reason(Panic)` returns `false`; `EXPORTED`
    /// stays `false`.
    #[test]
    #[serial]
    fn panic_reason_respects_on_panic_false() {
        reset_for_test();
        let tracker = fresh_tracker();
        let dir = TempDir::new().expect("tempdir creation must succeed in tests");
        let cfg = AutoExportConfig::default()
            .with_output_path(dir.path())
            .with_on_panic(false);
        install(cfg, tracker).expect("install must succeed");

        let did = export_for_reason(ExportReason::Panic);
        assert!(
            !did,
            "export_for_reason(Panic) must return false when on_panic is false",
        );
        assert!(
            !EXPORTED.load(Ordering::SeqCst),
            "EXPORTED must remain false so the disabled panic path does not block later exports",
        );
    }

    // ============== Panic-hook chaining (release-panic-abort coverage) ==============

    /// Objective: Verify the installed panic hook (a) calls `export_once` and
    /// (b) chains to the previously-registered hook. This is the CRITICAL
    /// coverage for `panic = "abort"` release builds, where the panic hook is
    /// the only mechanism that runs before abort.
    ///
    /// Invariants:
    /// - The sentinel (previous) hook runs (flag set to true) — chaining works.
    /// - `EXPORTED` is true after the panic — our hook exported.
    ///
    /// # Why we call `install_panic_hook` directly instead of `install`
    ///
    /// [`HOOKS_INSTALLED`] is a `Once` and cannot be reset between tests, so
    /// `install`'s `call_once` body would be a no-op if another test already
    /// triggered it. Calling `install_panic_hook` directly gives a clean,
    /// deterministic sentinel -> our_hook chain regardless of `Once` state or
    /// test execution order.
    #[test]
    #[serial]
    fn panic_hook_chains_and_exports() {
        reset_for_test();

        // Install a tracker + cfg directly into the statics so export_once
        // (invoked by the panic hook) has data to export. We bypass install()
        // here to avoid the Once affecting the hook-under-test. Keep the
        // TempDir alive for the whole test so it cleans up the directory.
        let tracker = fresh_tracker();
        let tempdir = TempDir::new().expect("tempdir creation must succeed in tests");
        let dir = tempdir.path().to_path_buf();
        let cfg = AutoExportConfig::default().with_output_path(dir.clone());
        *TRACKER_HANDLE.write() = Some(tracker);
        *AUTO_EXPORT_CFG.write() = Some(cfg);

        // 1. Register a sentinel hook that sets a flag, then 2. install our
        //    hook on top of it. install_panic_hook take_hooks the sentinel and
        //    chains to it.
        let sentinel_fired = Arc::new(AtomicBool::new(false));
        let sf = sentinel_fired.clone();
        std::panic::set_hook(Box::new(move |_| {
            sf.store(true, Ordering::SeqCst);
        }));
        install_panic_hook();

        // Spawn a thread that panics. Under the test profile (panic=unwind)
        // join() returns Err; under release (panic=abort) the hook still runs
        // before abort — that is the behavior this test guards.
        let handle = std::thread::spawn(|| {
            panic!("lifecycle test panic");
        });
        let join_err = handle.join();
        assert!(
            join_err.is_err(),
            "the panicking thread must propagate the panic to join() as Err",
        );

        assert!(
            sentinel_fired.load(Ordering::SeqCst),
            "the sentinel (previous) hook must have run — panic-hook chaining is broken",
        );
        assert!(
            EXPORTED.load(Ordering::SeqCst),
            "EXPORTED must be true — our panic hook must have called export_once",
        );
        assert!(
            dir.join("dashboard_unified_dashboard.html").exists(),
            "the panic-hook export must have written the HTML dashboard",
        );

        // Restore the default hook so this test does not pollute later tests.
        // take_hook() returns the current hook (dropped here) and resets the
        // registered hook to the default.
        let _ = std::panic::take_hook();
    }

    // ===================== Property-based test ====================

    /// Objective: Verify `do_export` never panics across many randomly
    /// generated (but valid) `AutoExportConfig` values.
    ///
    /// Invariants: For every generated config, `do_export` returns without
    /// panicking. A single shared empty `GlobalTracker` is reused across all
    /// cases (an empty tracker renders quickly, keeping the suite fast).
    ///
    /// # Case count
    ///
    /// 200 cases instead of 1000: each case that selects HTML renders a full
    /// dashboard via handlebars. Even for an empty tracker this is a few ms
    /// per case, and 200 cases keeps the suite comfortably within the ~10s
    /// budget. The contract under test ("never panic on valid input") is
    /// fully exercised by 200 random configs.
    #[test]
    #[serial]
    fn proptest_do_export_never_panics_on_valid_configs() {
        let tracker = GlobalTracker::new();
        let mut runner = TestRunner::new(ProptestConfig {
            cases: 200,
            ..ProptestConfig::default()
        });

        // Strategy: random format bits 0..=3, random booleans, random signal
        // policy, random flush interval (None or 1..100ms). The output path
        // is built per-case from a fresh TempDir inside the closure so the
        // TempDir outlives the do_export call.
        let strategy = (
            0u8..4u8,
            any::<bool>(),
            any::<bool>(),
            prop_oneof![
                Just(SignalPolicy::Off),
                Just(SignalPolicy::CtrlC),
                Just(SignalPolicy::CtrlCAndTerm),
            ],
            prop_oneof![
                Just(None),
                (1u64..100u64).prop_map(|ms| Some(Duration::from_millis(ms))),
            ],
        );

        runner
            .run(&strategy, |(bits, on_exit, on_panic, on_signal, flush)| {
                // `run` passes `S::Value` by value; the tuple is `Copy`, so this
                // destructures the fields out without moving issues.
                let tempdir = TempDir::new().expect("per-case tempdir must succeed in proptest");
                let cfg = AutoExportConfig {
                    output_path: tempdir.path().to_path_buf(),
                    formats: ExportFormatSet::from_bits(bits),
                    on_exit,
                    on_panic,
                    on_signal,
                    flush_interval: flush,
                    exit_timeout: Duration::from_secs(5),
                };
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    do_export(&tracker, &cfg)
                }));
                prop_assert!(
                    result.is_ok(),
                    "do_export must never panic on valid configs (bits={})",
                    bits,
                );
                Ok(())
            })
            .expect("proptest run must complete with no failing cases");
    }

    /// Objective: Verify `do_export`'s dispatch contract (return value = whether
    /// at least one format was written) holds across 1000 random valid configs.
    /// This satisfies rules.md §V.3 (proptest with at least 1000 cases) for the
    /// dispatch surface, complementing the 200-case render-path proptest above.
    ///
    /// Invariants: For every config, `do_export` returns without panicking AND
    /// the return value equals `!formats.is_empty()` (when the output dir is
    /// writable, which it always is here because we reuse a single tempdir).
    #[test]
    #[serial]
    fn proptest_do_export_dispatch_contract_1000_cases() {
        let tracker = GlobalTracker::new();
        let mut runner = TestRunner::new(ProptestConfig {
            cases: 1000,
            ..ProptestConfig::default()
        });
        // Reuse ONE tempdir across all 1000 cases — do_export is idempotent on
        // existing dirs and overwrites files, so no per-case cleanup is needed.
        // This keeps the 1000-case run fast by avoiding 1000 mkdir/rmdir pairs.
        let tempdir = TempDir::new().expect("shared tempdir must succeed for the 1000-case run");
        let strategy = (
            0u8..4u8,
            any::<bool>(),
            any::<bool>(),
            prop_oneof![
                Just(SignalPolicy::Off),
                Just(SignalPolicy::CtrlC),
                Just(SignalPolicy::CtrlCAndTerm),
            ],
            prop_oneof![
                Just(None),
                (1u64..100u64).prop_map(|ms| Some(Duration::from_millis(ms))),
            ],
        );

        runner
            .run(&strategy, |(bits, on_exit, on_panic, on_signal, flush)| {
                let cfg = AutoExportConfig {
                    output_path: tempdir.path().to_path_buf(),
                    formats: ExportFormatSet::from_bits(bits),
                    on_exit,
                    on_panic,
                    on_signal,
                    flush_interval: flush,
                    exit_timeout: Duration::from_secs(5),
                };
                // We test ONLY the dispatch contract here (no file-content
                // inspection), so the 1000-case run stays fast even though
                // some cases render HTML. The contract: do_export returns
                // true iff at least one format was selected AND succeeded.
                let did = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    do_export(&tracker, &cfg)
                }));
                prop_assert!(
                    did.is_ok(),
                    "do_export must never panic on valid configs (bits={})",
                    bits,
                );
                let did = did.expect("catch_unwind Ok checked above");
                // With a writable tempdir, do_export returns true iff at least
                // one format is selected (and export_html/export_json succeed
                // for an empty tracker).
                prop_assert_eq!(
                    did,
                    !cfg.formats.is_empty(),
                    "do_export return value must track format selection (bits={}, did={})",
                    bits,
                    did,
                );
                Ok(())
            })
            .expect("1000-case proptest must pass with no failing cases");
    }

    // ===================== Miri coverage for the atexit path ====================

    /// Objective: Verify `install_atexit` registers without panicking. This is
    /// the unsafe FFI surface (`libc::atexit`) and must be Miri-clean.
    ///
    /// Invariants: Calling `install_atexit` does not panic and does not
    /// trigger UB under Miri.
    ///
    /// # Miri
    ///
    /// `cargo +nightly miri test --features atexit lifecycle::` must report 0
    /// errors. The registered `on_exit` callback runs at process exit and
    /// calls `export_once`; with no tracker installed it returns `false`
    /// harmlessly, so the atexit path is exercised end-to-end under Miri.
    #[test]
    #[cfg(feature = "atexit")]
    #[serial]
    fn atexit_install_is_safe_and_does_not_panic() {
        reset_for_test();
        // Direct call: only the registration is under test here. The callback
        // runs at process exit, not at registration time.
        install_atexit();
        // No assertion beyond "did not panic" — the SAFETY block in
        // install_atexit is the contract under Miri's scrutiny.
    }
}
