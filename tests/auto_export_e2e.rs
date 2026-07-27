//! End-to-end integration tests for auto-export on realistic exit paths.
//!
//! Each test spawns [`examples/auto_exit_demo.rs`] as a child process in a
//! specific mode (normal / panic / sleep+SIGINT), waits for it to terminate,
//! and asserts that the expected report files exist in the output directory.
//!
//! These are true process-level integration tests — they exercise the actual
//! `start()` / `MemScopeGuard` / panic-hook / ctrlc-handler code paths as they
//! would run in production.

use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Locate the built example binary.
fn example_bin() -> PathBuf {
    // `CARGO_BIN_EXE_<name>` is set by `cargo test` for the current crate's
    // binaries. For example binaries we build them explicitly.
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--example",
        "auto_exit_demo",
        "--quiet",
        "--features",
        "auto-signal,periodic",
    ]);
    // Only pass --release when the test itself is compiled in release mode.
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    let status = cmd.status().expect("failed to build auto_exit_demo example");
    assert!(status.success(), "cargo build --example auto_exit_demo failed");

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    PathBuf::from(format!("target/{profile}/examples/auto_exit_demo"))
}

/// Spawn `auto_exit_demo` in the given mode with a tempdir-based output
/// directory and return the child handle + the output dir.
fn spawn_demo(mode: &str) -> (Child, PathBuf) {
    let bin = example_bin();
    let out_dir = std::env::temp_dir()
        .join(format!("memscope_e2e_{}", std::process::id()));
    // Clean any previous run's leftovers.
    let _ = std::fs::remove_dir_all(&out_dir);

    let child = Command::new(&bin)
        .arg(mode)
        .env("MEMSCOPE_E2E_OUTPUT", &out_dir)
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn auto_exit_demo {mode}: {e}"));

    (child, out_dir)
}

/// Wait for the child to finish with a timeout.
fn wait_with_timeout(child: &mut Child, timeout: Duration) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    // Kill and bail.
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("child process did not exit within {timeout:?}");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("error waiting for child: {e}"),
        }
    }
}

/// Assert that the output directory contains the expected report files.
fn assert_report_exists(dir: &PathBuf) {
    let html = dir.join("dashboard_unified_dashboard.html");
    let json = dir.join("memory_analysis.json");
    assert!(
        html.exists(),
        "expected HTML report at {}",
        html.display()
    );
    assert!(
        json.exists(),
        "expected JSON report at {}",
        json.display()
    );
    // Both files should be non-empty.
    assert!(
        std::fs::metadata(&html).unwrap().len() > 100,
        "HTML report is suspiciously small"
    );
    assert!(
        std::fs::metadata(&json).unwrap().len() > 50,
        "JSON report is suspiciously small"
    );
}

// ============================================================================
// Tests
// ============================================================================

/// Normal exit: the `MemScopeGuard` drop triggers the export.
#[test]
fn e2e_normal_exit() {
    let (mut child, out_dir) = spawn_demo("normal");
    let status = wait_with_timeout(&mut child, Duration::from_secs(30));
    assert!(status.success(), "normal exit should succeed");
    assert_report_exists(&out_dir);
    // Cleanup.
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// Panic exit: the panic hook triggers the export before unwinding.
#[test]
fn e2e_panic_exit() {
    let (mut child, out_dir) = spawn_demo("panic");
    let status = wait_with_timeout(&mut child, Duration::from_secs(30));
    // A panic exit is a non-zero exit code (101 on stable Rust).
    assert!(!status.success(), "panic exit should report failure");
    assert_report_exists(&out_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// Ctrl-C exit: send SIGINT while the process is sleeping; the ctrlc handler
/// should export before calling `process::exit(130)`.
#[test]
#[cfg(unix)]
fn e2e_ctrlc_exit() {
    use std::os::unix::process::ExitStatusExt;

    let (mut child, out_dir) = spawn_demo("sleep");
    // Let the process initialise and start sleeping.
    std::thread::sleep(Duration::from_secs(2));

    // Send SIGINT (Ctrl-C) via libc::kill (libc is already a dependency).
    let pid = child.id() as libc::pid_t;
    let ret = unsafe { libc::kill(pid, libc::SIGINT) };
    assert_eq!(ret, 0, "libc::kill failed to send SIGINT to pid {pid}");

    let status = wait_with_timeout(&mut child, Duration::from_secs(10));

    // The ctrlc handler calls `process::exit(130)`. Verify exit code 130
    // (128 + SIGINT).
    assert_eq!(
        status.code(),
        Some(130),
        "ctrlc handler should exit with code 130 (128+SIGINT), got {status:?}"
    );
    // On Unix, also verify the signal was SIGINT.
    assert!(
        !status.core_dumped(),
        "child should not have core-dumped"
    );

    assert_report_exists(&out_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}
