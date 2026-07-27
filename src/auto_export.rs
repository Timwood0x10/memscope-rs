//! Auto-export configuration for memscope-rs.
//!
//! Defines the configuration types that control automatic export of tracking
//! data on program exit (normal return, panic, Ctrl-C, SIGTERM) and optional
//! periodic background flushing for long-running services.
//!
//! This is module 1 of the 4-module auto-export feature. It is intentionally
//! dependency-light: only `std` and `serde` (both already in the crate graph)
//! are used. Every type here is plain data with no I/O or global state, so the
//! module can be unit-tested in isolation and downstream modules (signal
//! installation, exit hooks, background flusher) can build on these contracts.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Which signals trigger an automatic export before termination.
///
/// The policy is consulted by the signal-installing layer (module 2); this
/// type only carries the user's intent so the wiring code can decide which
/// handlers to register. Keeping it as a plain enum (rather than a bitfield)
/// makes the supported combinations explicit and exhaustive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SignalPolicy {
    /// No signal handler installed. Only Drop + panic-hook cover exit paths.
    Off,
    /// Catch SIGINT (Ctrl-C) only. This is the default policy.
    #[default]
    CtrlC,
    /// Catch both SIGINT and SIGTERM.
    ///
    /// # Current limitation
    ///
    /// SIGTERM is NOT yet wired up — only SIGINT is caught by the underlying
    /// `ctrlc` crate. SIGTERM support requires `signal_hook` (or a raw libc
    /// handler) and will be added when that dependency is approved. Selecting
    /// this policy currently behaves identically to [`SignalPolicy::CtrlC`]
    /// from the perspective of which signals actually trigger an export.
    CtrlCAndTerm,
}

/// Bitflags-style set of export formats. Uses a raw `u8` to avoid adding the
/// `bitflags` crate as a dependency.
///
/// # Concurrency
///
/// `ExportFormatSet` is `Copy` and every accessor is `const fn`, so values
/// carry no interior mutability and no shared mutable state. The type is
/// trivially `Sync`: multiple threads may read the same value (or copies of
/// it) without locking. It is therefore safe to pass across thread boundaries
/// by value or behind an `Arc` for read-only sharing — no mutex is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportFormatSet(u8);

impl ExportFormatSet {
    /// HTML dashboard only.
    pub const HTML: Self = Self(0b01);
    /// JSON data files only.
    pub const JSON: Self = Self(0b10);
    /// Both HTML and JSON (the recommended default).
    pub const HTML_JSON: Self = Self(0b11);

    /// Construct from raw bits.
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    /// Read raw bits.
    pub const fn to_bits(self) -> u8 {
        self.0
    }
    /// True when no format is selected.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// True when the HTML format is selected.
    pub const fn contains_html(self) -> bool {
        (self.0 & Self::HTML.0) != 0
    }
    /// True when the JSON format is selected.
    pub const fn contains_json(self) -> bool {
        (self.0 & Self::JSON.0) != 0
    }
    /// Set union (`self | other`).
    pub const fn insert(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Set difference (`self & !other`).
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl Default for ExportFormatSet {
    fn default() -> Self {
        Self::HTML_JSON
    }
}

/// Configuration for the automatic export subsystem.
///
/// Controls WHERE reports are written, WHICH formats, and WHICH exit paths
/// trigger an export. All fields have sensible defaults via `Default`, so a
/// plain `AutoExportConfig::default()` yields a working "export HTML+JSON to
/// `./memscope-report` on any exit" setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoExportConfig {
    /// Directory to write reports into. Created if missing.
    /// Default: `./memscope-report`.
    pub output_path: PathBuf,
    /// Which output formats to produce. Default: HTML + JSON.
    pub formats: ExportFormatSet,
    /// Export on normal program return (Drop guard fires). Default: `true`.
    pub on_exit: bool,
    /// Export on panic (panic-hook chaining). Critical for
    /// `panic = "abort"` release builds. Default: `true`.
    pub on_panic: bool,
    /// Which signals trigger an export. Default: `CtrlC`.
    pub on_signal: SignalPolicy,
    /// If `Some(interval)`, spawn a background worker that flushes every
    /// `interval`. If `None`, no background flushing (export only on exit).
    /// Default: `None`.
    pub flush_interval: Option<Duration>,
    /// Max time the shutdown path waits for an in-flight export to finish
    /// before giving up. Default: 5s.
    pub exit_timeout: Duration,
}

impl Default for AutoExportConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("./memscope-report"),
            formats: ExportFormatSet::default(),
            on_exit: true,
            on_panic: true,
            on_signal: SignalPolicy::default(),
            flush_interval: None,
            exit_timeout: Duration::from_secs(5),
        }
    }
}

impl AutoExportConfig {
    /// Builder-style: set output path.
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = path.into();
        self
    }
    /// Builder-style: set formats.
    pub fn with_formats(mut self, formats: ExportFormatSet) -> Self {
        self.formats = formats;
        self
    }
    /// Builder-style: enable periodic flushing.
    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = Some(interval);
        self
    }
    /// Builder-style: set signal policy.
    pub fn with_signal_policy(mut self, policy: SignalPolicy) -> Self {
        self.on_signal = policy;
        self
    }
    /// Builder-style: set the `on_exit` flag (export on normal Drop).
    pub fn with_on_exit(mut self, on_exit: bool) -> Self {
        self.on_exit = on_exit;
        self
    }
    /// Builder-style: set the `on_panic` flag (export on panic).
    pub fn with_on_panic(mut self, on_panic: bool) -> Self {
        self.on_panic = on_panic;
        self
    }
    /// Builder-style: set the shutdown timeout for in-flight exports.
    pub fn with_exit_timeout(mut self, timeout: Duration) -> Self {
        self.exit_timeout = timeout;
        self
    }
    /// True if any export path is enabled (exit, panic, or signal != `Off`).
    pub fn is_auto_export_enabled(&self) -> bool {
        self.on_exit || self.on_panic || self.on_signal != SignalPolicy::Off
    }
    /// True if the HTML format is selected.
    pub fn wants_html(&self) -> bool {
        self.formats.contains_html()
    }
    /// True if the JSON format is selected.
    pub fn wants_json(&self) -> bool {
        self.formats.contains_json()
    }
}

/// Top-level configuration for `memscope_rs::start_with`.
///
/// Combines the auto-export subsystem config with the existing
/// `GlobalTrackerConfig` (tracking sampling, passport, etc.) so callers can
/// configure both halves of a `start_with` call from a single value.
#[derive(Debug, Clone, Default)]
pub struct MemScopeConfig {
    /// Auto-export subsystem configuration.
    pub auto_export: AutoExportConfig,
    /// Underlying global tracker configuration (sampling, passport tracker, etc.).
    pub tracker: crate::capture::backends::global_tracking::GlobalTrackerConfig,
}

impl MemScopeConfig {
    /// Builder-style: set auto-export config.
    pub fn with_auto_export(mut self, cfg: AutoExportConfig) -> Self {
        self.auto_export = cfg;
        self
    }
    /// Builder-style: set tracker config.
    pub fn with_tracker(
        mut self,
        cfg: crate::capture::backends::global_tracking::GlobalTrackerConfig,
    ) -> Self {
        self.tracker = cfg;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;
    use std::thread;

    // ===================== Positive tests (happy path) ====================

    /// Objective: Verify `AutoExportConfig::default()` populates every field
    /// with the documented default value.
    /// Invariants: No field is left at an accidental zero/unset state.
    #[test]
    fn default_config_has_all_documented_defaults() {
        let cfg = AutoExportConfig::default();

        assert_eq!(
            cfg.output_path,
            PathBuf::from("./memscope-report"),
            "default output_path must be ./memscope-report per the field doc"
        );
        assert_eq!(
            cfg.formats,
            ExportFormatSet::HTML_JSON,
            "default formats must be HTML+JSON (the recommended default)"
        );
        assert!(
            cfg.on_exit,
            "on_exit defaults to true so the Drop-guard export fires"
        );
        assert!(
            cfg.on_panic,
            "on_panic defaults to true to cover panic=abort release builds"
        );
        assert_eq!(
            cfg.on_signal,
            SignalPolicy::CtrlC,
            "default signal policy is CtrlC per SignalPolicy::default"
        );
        assert_eq!(
            cfg.flush_interval, None,
            "no background flushing by default; export only on exit"
        );
        assert_eq!(
            cfg.exit_timeout,
            Duration::from_secs(5),
            "shutdown path waits up to 5s for an in-flight export"
        );
    }

    /// Objective: Verify the combined format constant reports both flags set.
    /// Invariants: HTML_JSON must contain HTML and JSON simultaneously.
    #[test]
    fn html_json_format_contains_both_html_and_json() {
        let both = ExportFormatSet::HTML_JSON;
        assert!(
            both.contains_html(),
            "HTML_JSON must select the HTML format"
        );
        assert!(
            both.contains_json(),
            "HTML_JSON must select the JSON format"
        );
        assert!(!both.is_empty(), "HTML_JSON must not be empty");
    }

    /// Objective: Verify builder methods chain without clobbering prior fields.
    /// Invariants: Each chained with_* call sets exactly its own field and
    /// leaves the rest at their defaults.
    #[test]
    fn builder_methods_chain_correctly() {
        let cfg = AutoExportConfig::default()
            .with_output_path("/tmp/x")
            .with_flush_interval(Duration::from_secs(10));

        assert_eq!(
            cfg.output_path,
            PathBuf::from("/tmp/x"),
            "with_output_path must override the default directory"
        );
        assert_eq!(
            cfg.flush_interval,
            Some(Duration::from_secs(10)),
            "with_flush_interval must enable periodic flushing at 10s"
        );
        // Untouched fields keep their defaults.
        assert_eq!(
            cfg.formats,
            ExportFormatSet::HTML_JSON,
            "chaining must not reset previously-defaulted formats"
        );
        assert!(cfg.on_exit, "chaining must not reset on_exit");
        assert_eq!(
            cfg.on_signal,
            SignalPolicy::CtrlC,
            "chaining must not reset on_signal"
        );
    }

    /// Objective: Verify MemScopeConfig builder round-trips an auto-export cfg.
    /// Invariants: with_auto_export stores exactly the provided config.
    #[test]
    fn memscope_config_with_auto_export_round_trips() {
        let inner = AutoExportConfig::default().with_output_path("/tmp/round");
        let outer = MemScopeConfig::default().with_auto_export(inner.clone());

        assert_eq!(
            outer.auto_export.output_path,
            PathBuf::from("/tmp/round"),
            "with_auto_export must store the provided config verbatim"
        );
        assert_eq!(
            outer.auto_export, inner,
            "round-trip must be lossless for AutoExportConfig"
        );
    }

    /// Objective: Verify `with_tracker` stores the provided tracker config.
    /// Invariants: The tracker field is replaced wholesale by the builder.
    #[test]
    fn memscope_config_with_tracker_round_trips() {
        let outer = MemScopeConfig::default().with_tracker(
            crate::capture::backends::global_tracking::GlobalTrackerConfig::default(),
        );
        // GlobalTrackerConfig derives Default; the builder must accept it and
        // leave the auto_export half untouched.
        assert_eq!(
            outer.auto_export,
            AutoExportConfig::default(),
            "with_tracker must not disturb the auto_export config"
        );
    }

    /// Objective: Verify is_auto_export_enabled is true for the default config.
    /// Invariants: Default has on_exit=true, on_panic=true, signal=CtrlC, so
    /// at least three independent triggers are armed.
    #[test]
    fn default_config_is_auto_export_enabled() {
        let cfg = AutoExportConfig::default();
        assert!(
            cfg.is_auto_export_enabled(),
            "default config (exit+panic+CtrlC) must enable auto-export"
        );
    }

    // ===================== Negative tests (edge cases) ====================

    /// Objective: Verify a zero-bits set is considered empty.
    /// Invariants: from_bits(0) yields no selected formats.
    #[test]
    fn zero_bits_is_empty() {
        let empty = ExportFormatSet::from_bits(0u8);
        assert!(
            empty.is_empty(),
            "from_bits(0) must report empty since no format bits are set"
        );
    }

    /// Objective: Verify zero-bits does not claim HTML.
    /// Invariants: contains_html must be false when the HTML bit is clear.
    #[test]
    fn zero_bits_does_not_contain_html() {
        let empty = ExportFormatSet::from_bits(0u8);
        assert!(
            !empty.contains_html(),
            "from_bits(0) must not report HTML selected"
        );
    }

    /// Objective: Verify SignalPolicy::Off alone disables auto-export ONLY
    /// when on_exit and on_panic are also false; flipping any single trigger
    /// back on re-enables it.
    /// Invariants: is_auto_export_enabled is the OR of the three triggers.
    #[test]
    fn off_signal_disables_only_when_all_triggers_off() {
        let mut cfg = AutoExportConfig {
            on_exit: false,
            on_panic: false,
            on_signal: SignalPolicy::Off,
            ..Default::default()
        };
        assert!(
            !cfg.is_auto_export_enabled(),
            "all three triggers off must disable auto-export entirely"
        );

        // Flipping on_exit alone back on re-enables auto-export.
        cfg.on_exit = true;
        assert!(
            cfg.is_auto_export_enabled(),
            "re-enabling on_exit alone must re-enable auto-export"
        );

        // Flipping on_panic alone back on re-enables auto-export.
        cfg.on_exit = false;
        cfg.on_panic = true;
        assert!(
            cfg.is_auto_export_enabled(),
            "re-enabling on_panic alone must re-enable auto-export"
        );

        // A non-Off signal policy alone re-enables auto-export.
        cfg.on_panic = false;
        cfg.on_signal = SignalPolicy::CtrlC;
        assert!(
            cfg.is_auto_export_enabled(),
            "a non-Off signal policy alone must re-enable auto-export"
        );
    }

    /// Objective: Verify remove(self, self) clears the set.
    /// Invariants: HTML.remove(HTML) leaves no bits set.
    #[test]
    fn remove_self_clears_set() {
        let cleared = ExportFormatSet::HTML.remove(ExportFormatSet::HTML);
        assert!(
            cleared.is_empty(),
            "removing a format from itself must yield an empty set"
        );
    }

    /// Objective: Verify removing HTML from HTML_JSON leaves JSON only.
    /// Invariants: After removal, JSON is set and HTML is clear.
    #[test]
    fn remove_html_from_both_leaves_json_only() {
        let json_only = ExportFormatSet::HTML_JSON.remove(ExportFormatSet::HTML);
        assert!(
            !json_only.contains_html(),
            "removing HTML must clear the HTML bit"
        );
        assert!(
            json_only.contains_json(),
            "removing HTML must preserve the JSON bit"
        );
        assert_eq!(
            json_only,
            ExportFormatSet::JSON,
            "HTML_JSON - HTML must equal JSON"
        );
    }

    /// Objective: Verify a config built with empty formats reports neither
    /// HTML nor JSON desired.
    /// Invariants: wants_html/wants_json follow the formats bits exactly.
    #[test]
    fn empty_formats_means_no_formats_wanted() {
        let cfg = AutoExportConfig::default().with_formats(ExportFormatSet::from_bits(0u8));
        assert!(
            !cfg.wants_html(),
            "with_formats(0) must disable HTML output"
        );
        assert!(
            !cfg.wants_json(),
            "with_formats(0) must disable JSON output"
        );
    }

    // ===================== Stress tests (concurrency) =====================

    /// Objective: Verify ExportFormatSet is safe to share across 50 threads
    /// with no locking; all const-fn operations must be consistent.
    /// Invariants: Because the type is Copy with no interior mutability,
    /// every thread observes identical results for the same input bits.
    #[test]
    fn export_format_set_is_lock_free_safe_across_threads() {
        const THREAD_COUNT: usize = 50;
        // Share one read-only value via Arc; threads also make local copies.
        let shared = Arc::new(ExportFormatSet::HTML_JSON);
        let mut handles = Vec::with_capacity(THREAD_COUNT);

        for _ in 0..THREAD_COUNT {
            let snapshot = Arc::clone(&shared);
            handles.push(thread::spawn(move || {
                // Read the shared value: consistent because Copy + Sync.
                let base = *snapshot;
                // Local pure computations on a copy — no shared mutation.
                let with_extra = base.insert(ExportFormatSet::JSON);
                let stripped = base.remove(ExportFormatSet::HTML);
                // Every thread must see the same deterministic results.
                (base, with_extra, stripped)
            }));
        }

        let mut checked = 0usize;
        for handle in handles {
            // join().expect() is infallible here: the closure performs only
            // Copy bit-twiddling (insert/remove/contains), none of which can
            // panic, so the thread cannot have panicked. A panic would itself
            // be a regression we want to surface rather than swallow.
            let (base, with_extra, stripped) = handle
                .join()
                .expect("worker thread must not panic on pure Copy bit ops");
            assert_eq!(
                base,
                ExportFormatSet::HTML_JSON,
                "every thread must read the same shared base value"
            );
            assert_eq!(
                with_extra,
                ExportFormatSet::HTML_JSON,
                "inserting an already-set bit must be idempotent across threads"
            );
            assert_eq!(
                stripped,
                ExportFormatSet::JSON,
                "removing HTML from HTML_JSON must yield JSON in every thread"
            );
            checked += 1;
        }
        assert_eq!(
            checked, THREAD_COUNT,
            "all 50 worker threads must report consistent results"
        );
    }

    // ===================== proptest (1000 cases each) =====================

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 1000,
            ..ProptestConfig::default()
        })]

        /// Property: from_bits is a perfect inverse of to_bits.
        /// Why: the format set is a transparent newtype over u8, so any u8
        /// must round-trip without loss.
        #[test]
        fn bits_round_trip(b in any::<u8>()) {
            prop_assert_eq!(
                ExportFormatSet::from_bits(b).to_bits(),
                b,
                "from_bits(b).to_bits() must equal b for every u8"
            );
        }

        /// Property: insert computes set union.
        /// Why: insert is documented as self | other.
        #[test]
        fn insert_is_union(a in any::<u8>(), b in any::<u8>()) {
            let got = ExportFormatSet::from_bits(a)
                .insert(ExportFormatSet::from_bits(b))
                .to_bits();
            prop_assert_eq!(
                got,
                a | b,
                "from_bits(a).insert(from_bits(b)) must equal (a | b)"
            );
        }

        /// Property: remove computes set difference.
        /// Why: remove is documented as self & !other.
        #[test]
        fn remove_is_difference(a in any::<u8>(), b in any::<u8>()) {
            let got = ExportFormatSet::from_bits(a)
                .remove(ExportFormatSet::from_bits(b))
                .to_bits();
            prop_assert_eq!(
                got,
                a & !b,
                "from_bits(a).remove(from_bits(b)) must equal (a & !b)"
            );
        }

        /// Property: contains_html tracks the low bit exactly.
        /// Why: HTML is defined as 0b01, so contains_html is (b & 0b01) != 0.
        #[test]
        fn contains_html_tracks_low_bit(b in any::<u8>()) {
            let got = ExportFormatSet::from_bits(b).contains_html();
            prop_assert_eq!(
                got,
                (b & 0b01) != 0,
                "contains_html must equal ((b & 0b01) != 0) for every u8"
            );
        }
    }
}
