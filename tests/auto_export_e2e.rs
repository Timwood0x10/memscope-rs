//! End-to-end integration tests for auto-export on realistic exit paths.
//!
//! Each test spawns [`examples/auto_exit_demo.rs`] as a child process in a
//! specific mode (normal / panic / sleep+SIGINT), waits for it to terminate,
//! and checks complete reports against the mode-specific live payload.
//!
//! These are true process-level integration tests — they exercise the actual
//! `start()` / `MemScopeGuard` / panic-hook / ctrlc-handler code paths as they
//! would run in production.

use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[path = "../examples/auto_exit_demo.rs"]
mod demo;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run the same example source in a fresh process without compiling inside a test.
#[test]
fn child_driver() {
    if std::env::var_os("MEMSCOPE_E2E_MODE").is_some() {
        demo::main().expect("example must initialize and export");
    }
}

/// Spawn `auto_exit_demo` in the given mode with a unique tempdir-based
/// output directory and return the child handle + the output dir.
/// Each call creates a separate directory so parallel tests don't collide.
fn spawn_demo(mode: &str) -> (Child, TempDir) {
    let out_dir = TempDir::new().expect("unique child report directory");
    let child = Command::new(std::env::current_exe().expect("integration test executable"))
        .args(["--exact", "child_driver", "--nocapture"])
        .env("MEMSCOPE_E2E_MODE", mode)
        .env("MEMSCOPE_E2E_OUTPUT", out_dir.path())
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

/// Complete JSON/HTML and a unique marker prove the intended process was exported.
fn assert_report_exists(dir: &std::path::Path, marker_size: u64) {
    let html = dir.join("dashboard_unified_dashboard.html");
    let json = dir.join("memory_analysis.json");
    assert!(html.exists(), "expected HTML report at {}", html.display());
    assert!(json.exists(), "expected JSON report at {}", json.display());
    let html = std::fs::read_to_string(html).expect("read completed dashboard");
    assert!(
        html.contains("</html>"),
        "process must finish writing the HTML document before exiting"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(json).expect("read allocation report"))
            .expect("allocation report must be valid complete JSON");
    let markers: Vec<_> = report["allocations"]
        .as_array()
        .expect("allocation rows")
        .iter()
        .filter(|row| row["var_name"] == "data")
        .collect();
    assert_eq!(
        markers.len(),
        1,
        "typed marker must describe exactly one allocation"
    );
    assert_eq!(
        markers[0]["size"], marker_size,
        "report must contain this mode's payload size"
    );
    assert_eq!(
        markers[0]["lifetime_complete"], false,
        "marker remains alive during export"
    );
    let json_files: Vec<_> = std::fs::read_dir(dir)
        .expect("report directory")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(
        json_files.len(),
        10,
        "all ten JSON reports must finish before exit"
    );
    for file in json_files {
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&file).unwrap())
            .unwrap_or_else(|e| panic!("incomplete report {}: {e}", file.display()));
    }
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
    assert_report_exists(out_dir.path(), 111);
}

/// Panic exit: the panic hook triggers the export before unwinding.
#[test]
fn e2e_panic_exit() {
    let (mut child, out_dir) = spawn_demo("panic");
    let status = wait_with_timeout(&mut child, Duration::from_secs(30));
    // A panic exit is a non-zero exit code (101 on stable Rust).
    assert!(!status.success(), "panic exit should report failure");
    assert_report_exists(out_dir.path(), 222);
}

/// Ctrl-C exit: send SIGINT while the process is sleeping; the ctrlc handler
/// should export before calling `process::exit(130)`.
#[test]
#[cfg(all(unix, feature = "auto-signal"))]
fn e2e_ctrlc_exit() {
    use std::os::unix::process::ExitStatusExt;

    let (mut child, out_dir) = spawn_demo("sleep");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !out_dir.path().join("ready").exists() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("child must publish readiness within five seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    // Send SIGINT (Ctrl-C) via libc::kill (libc is already a dependency).
    let pid = child.id() as libc::pid_t;
    // SAFETY: pid belongs to our live child, and SIGINT is a valid signal number.
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
    assert!(!status.core_dumped(), "child should not have core-dumped");

    assert_report_exists(out_dir.path(), 333);
}
