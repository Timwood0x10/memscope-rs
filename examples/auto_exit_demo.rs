//! End-to-end auto-export demo used by `tests/auto_export_e2e.rs`.
//!
//! This is a tiny example used to exercise each realistic exit path through the
//! `start()` API:
//!
//! - `auto_exit_demo normal` — track a variable, then return from `main`. The
//!   `MemScopeGuard` drop triggers the exit-path export.
//! - `auto_exit_demo panic`  — track a variable, then `panic!`. The panic hook
//!   installed by `lifecycle::install` calls `export_once` before unwinding
//!   (or before abort under `panic = "abort"` release builds).
//! - `auto_exit_demo sleep`  — track a variable, then sleep for 30s. Used by
//!   the Ctrl-C test: the harness sends SIGINT after ~1s, and the ctrlc
//!   handler installed by `lifecycle::install_ctrlc_handler` calls
//!   `export_once` before the default SIGINT handler terminates the process.
//!
//! The output directory is read from the `MEMSCOPE_E2E_OUTPUT` env var so each
//! test can target its own tempdir.

use memscope_rs::{AutoExportConfig, MemScopeConfig, MemScopeResult};
use std::time::Duration;

fn main() -> MemScopeResult<()> {
    // Read the exit-mode argument (default to "normal" so a bare invocation
    // produces a report and exits cleanly).
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "normal".to_string());

    // Read the output directory from the env var; fall back to the default
    // ./memscope-report when the var is absent (e.g. when running the example
    // by hand without the test harness).
    let output_path = std::env::var("MEMSCOPE_E2E_OUTPUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./memscope-report"));

    let cfg = MemScopeConfig::default()
        .with_auto_export(AutoExportConfig::default().with_output_path(output_path.clone()));

    let guard = memscope_rs::start_with(cfg)?;

    // Track one allocation so the exported report is non-trivial. Using a
    // distinct size per mode makes it possible to confirm the right process
    // wrote the report (should we ever want to assert that later).
    let marker_size: usize = match mode.as_str() {
        "normal" => 111,
        "panic" => 222,
        "sleep" => 333,
        other => {
            eprintln!("unknown mode: {other}; expected normal|panic|sleep");
            std::process::exit(2);
        }
    };
    let data: Vec<u8> = vec![0u8; marker_size];
    memscope_rs::track!(guard, data);

    // Drop the guard explicitly for `normal` so the export runs BEFORE main
    // returns (deterministic ordering for the test's file-existence check).
    // For `panic` and `sleep`, the export is triggered by the panic hook /
    // ctrlc handler respectively — the guard stays alive so the non-drop
    // exit paths are the ones under test.
    match mode.as_str() {
        "normal" => {
            drop(guard);
        }
        "panic" => {
            // Re-arm and let the panic hook fire. The guard is NOT dropped
            // here so the only export path is the panic hook.
            panic!("auto_exit_demo: intentional panic for e2e panic-path test");
        }
        "sleep" => {
            // Sleep long enough that the test harness can send SIGINT.
            std::thread::sleep(Duration::from_secs(30));
            drop(guard);
        }
        _ => unreachable!("mode validated above"),
    }

    Ok(())
}
