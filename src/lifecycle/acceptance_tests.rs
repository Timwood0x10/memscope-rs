use super::*;
use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use serial_test::serial;
use tempfile::TempDir;

/// Objective: Preserve panic chaining and real report generation.
/// Invariants: The previous hook runs, the latch is consumed, and HTML is written.
#[test]
#[serial]
fn panic_hook_chains_and_exports() {
    reset_for_test();
    let dir = TempDir::new().expect("panic report directory");
    *TRACKER_HANDLE.write() = Some(Arc::new(GlobalTracker::new()));
    *AUTO_EXPORT_CFG.write() = Some(AutoExportConfig::default().with_output_path(dir.path()));
    let fired = Arc::new(AtomicBool::new(false));
    let sentinel = fired.clone();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |_| {
        sentinel.store(true, Ordering::SeqCst);
    }));
    // Call directly because HOOKS_INSTALLED is a process-wide Once.
    install_panic_hook();
    let result = std::thread::spawn(|| panic!("lifecycle test panic")).join();
    std::panic::set_hook(previous);
    assert!(result.is_err(), "join must observe the panic");
    assert!(fired.load(Ordering::SeqCst), "previous hook must run");
    assert!(
        EXPORTED.load(Ordering::SeqCst),
        "panic export must consume latch"
    );
    assert!(
        dir.path().join("dashboard_unified_dashboard.html").exists(),
        "panic report must exist"
    );
}

/// Objective: Real rendering for every format selection, with actual content checks.
/// Invariants: Only selected formats are written, and their contents parse/render.
#[test]
#[serial]
fn do_export_real_format_contents() {
    let tracker = GlobalTracker::new();
    for bits in 0..4 {
        let dir = TempDir::new().expect("real render directory");
        let cfg = AutoExportConfig::default()
            .with_output_path(dir.path())
            .with_formats(ExportFormatSet::from_bits(bits));
        assert_eq!(
            do_export(&tracker, &cfg),
            bits != 0,
            "return value follows selected formats"
        );
        let html = dir.path().join("dashboard_unified_dashboard.html");
        assert_eq!(html.exists(), bits & 1 != 0, "HTML selection");
        if html.exists() {
            let content = std::fs::read_to_string(html).expect("HTML content");
            assert!(content.contains("</html>"), "HTML must be complete");
        }
        let json = dir.path().join("memory_analysis.json");
        assert_eq!(json.exists(), bits & 2 != 0, "JSON selection");
        if json.exists() {
            let content = std::fs::read(json).expect("JSON content");
            let value: serde_json::Value = serde_json::from_slice(&content).expect("valid JSON");
            assert!(value.is_object(), "JSON report must have a structured root");
        }
    }
}

/// Objective: 1000 dispatch cases, including partial failures and empty selections.
/// Invariants: Each selected writer runs once; a failure cannot skip another writer.
#[test]
fn proptest_do_export_dispatch_contract_1000_cases() {
    let mut runner = TestRunner::new(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    });
    runner
        .run(
            &(0u8..4, any::<bool>(), any::<bool>()),
            |(bits, html_ok, json_ok)| {
                let mut calls = Vec::new();
                let result = dispatch_formats(ExportFormatSet::from_bits(bits), |format| {
                    calls.push(format);
                    if format == ExportFormatSet::HTML {
                        html_ok
                    } else {
                        json_ok
                    }
                });
                prop_assert_eq!(
                    result,
                    (bits & 1 != 0 && html_ok) || (bits & 2 != 0 && json_ok)
                );
                prop_assert_eq!(calls.len(), bits.count_ones() as usize);
                prop_assert_eq!(calls.contains(&ExportFormatSet::HTML), bits & 1 != 0);
                prop_assert_eq!(calls.contains(&ExportFormatSet::JSON), bits & 2 != 0);
                Ok(())
            },
        )
        .expect("1000 dispatch cases must preserve success and call counts");
}

#[test]
#[cfg(feature = "atexit")]
#[serial]
fn atexit_install_is_safe_and_does_not_panic() {
    reset_for_test();
    // Miri exercises the FFI registration and callback without a tracker installed.
    install_atexit();
}

#[test]
#[serial]
fn atexit_reason_respects_on_exit_false() {
    reset_for_test();
    let dir = TempDir::new().expect("exit report directory");
    install(
        AutoExportConfig::default()
            .with_output_path(dir.path())
            .with_on_exit(false),
        Arc::new(GlobalTracker::new()),
    )
    .expect("install tracker");
    assert!(
        !export_for_reason(ExportReason::Drop),
        "disabled exit must not export"
    );
    assert!(
        !EXPORTED.load(Ordering::SeqCst),
        "disabled exit must not consume latch"
    );
    assert!(
        !dir.path().join("dashboard_unified_dashboard.html").exists(),
        "disabled exit writes nothing"
    );
    assert!(trigger_export_now(), "manual export remains available");
}
