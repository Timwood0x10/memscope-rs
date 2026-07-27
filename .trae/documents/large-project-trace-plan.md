# Large-Project Trace: Unified Entry + Auto-Export Plan

## Context

memscope-rs is a Rust memory-tracking library. Today, integrating it into a large project requires multiple manual calls (`init_logging`, `init_global_tracking`, `track!`, `export_html`), and **all tracked data is lost** when the program exits via panic (release builds use `panic = "abort"`), Ctrl-C, or `std::process::exit`, because:

1. `TrackerConfig` is a **private** struct ([tracker.rs:152](file:///Users/scc/code/rustcode/memscope-rs/src/tracker.rs#L152)) — users cannot set `auto_export_on_drop` / `export_path`.
2. `GlobalTracker::with_config` ([global_tracking.rs:93](file:///Users/scc/code/rustcode/memscope-rs/src/capture/backends/global_tracking.rs#L93)) **ignores `config.tracker`** (calls `Tracker::new()`), so the existing `Drop for Tracker` auto-export path ([tracker.rs:554](file:///Users/scc/code/rustcode/memscope-rs/src/tracker.rs#L554)) is doubly dead code.
3. `Drop for Tracker` only emits a basic JSON snapshot (`export_snapshot_to_json`), not the full HTML dashboard.
4. No `Drop` on `GlobalTracker` or `MemCtx`; no production panic hook; no Ctrl-C / signal handler; no `atexit`.
5. No periodic-export mechanism for long-running services.

**Goal**: One-line start (`let _g = memscope_rs::start()?;`) that auto-exports HTML+JSON on every realistic exit path (normal return / panic-abort / Ctrl-C / SIGTERM / `process::exit`), plus an optional periodic flusher and an on-demand export API.

## Verified Facts (codescope + direct source read)

| Fact | Evidence |
|---|---|
| `TrackerConfig` private | [tracker.rs:152-157](file:///Users/scc/code/rustcode/memscope-rs/src/tracker.rs#L152) `struct TrackerConfig` (no `pub`) |
| `with_config` ignores `config.tracker` | [global_tracking.rs:94](file:///Users/scc/code/rustcode/memscope-rs/src/capture/backends/global_tracking.rs#L94) `let tracker = Tracker::new();` |
| `Drop for Tracker` exists, basic JSON only | [tracker.rs:554-585](file:///Users/scc/code/rustcode/memscope-rs/src/tracker.rs#L554) calls `export_snapshot_to_json` |
| No `Drop` on `GlobalTracker`/`MemCtx` | `rg "impl Drop"` empty for both files |
| Only panic hook is in `#[test]` | [allocator.rs:426](file:///Users/scc/code/rustcode/memscope-rs/src/core/allocator.rs#L426) |
| Release uses `panic = "abort"` | [Cargo.toml:86](file:///Users/scc/code/rustcode/memscope-rs/Cargo.toml#L86) |
| Existing exports | [global_tracking.rs:220-271](file:///Users/scc/code/rustcode/memscope-rs/src/capture/backends/global_tracking.rs#L220) `export_html`/`export_json`/`export_html_with_template` all `-> MemScopeResult<()>` |
| Existing init API | `init_global_tracking_with_config(config: GlobalTrackerConfig) -> MemScopeResult<()>`, `global_tracker() -> MemScopeResult<Arc<GlobalTracker>>` |
| `libc` already a dep | [Cargo.toml:31](file:///Users/scc/code/rustcode/memscope-rs/Cargo.toml#L31) — usable for `atexit` |
| File line counts | global_tracking.rs=646, mem_ctx.rs=42, lib.rs=443, tracker.rs=**1141 (over)**, async_tracker.rs=**1268 (over)** → new code MUST go in new modules, not appended to over-limit files |

## Exit-Path Coverage Matrix (target)

| Exit path | Drop guard | panic hook | ctrlc handler | libc::atexit |
|---|---|---|---|---|
| `main` returns | ✅ | — | — | ✅ |
| `panic=unwind` (dev) | ✅ | ✅ | — | — |
| `panic=abort` (release) | ❌ | ✅ | — | — |
| Ctrl-C / SIGTERM | ❌ | — | ✅ | — |
| `std::process::exit` | ❌ | — | — | ✅ |

All four funnel into one idempotent `export_once()` guarded by `AtomicBool`.

## Design

### New modules (keep every file < 1000 lines per rules.md #1)

| File | Responsibility | Approx LOC |
|---|---|---|
| `src/auto_export.rs` | `AutoExportConfig`, `MemScopeConfig`, `SignalPolicy`, `ExportFormatSet` (bitflags u8, no new dep) | ~250 |
| `src/lifecycle.rs` | `export_once()` (idempotent), panic-hook chaining, ctrlc handler, optional `atexit`; `OnceLock` for tracker handle + cfg | ~300 |
| `src/periodic_flusher.rs` | Background worker thread, separate control channel, `Mutex<Option<JoinHandle>>`, mirrors OTel BSP / tracing-appender `WorkerGuard` pattern | ~250 |
| `src/guard.rs` | `MemScopeGuard` (owns `Arc<GlobalTracker>` + optional `PeriodicFlusher`), `Deref` to `GlobalTracker`, `Drop` calls `export_once` | ~150 |

### Modifications to existing files

- **[mem_ctx.rs](file:///Users/scc/code/rustcode/memscope-rs/src/mem_ctx.rs)**: add `MemCtx::start() -> MemScopeResult<MemScopeGuard>` and `MemCtx::start_with(cfg: MemScopeConfig) -> MemScopeResult<MemScopeGuard>`. Keep `init()` + `export()` for backward compatibility.
- **[lib.rs](file:///Users/scc/code/rustcode/memscope-rs/src/lib.rs)**: `pub mod auto_export; pub mod lifecycle; pub mod periodic_flusher; pub mod guard;`. Add `pub use guard::{MemScopeGuard, start, start_with}; pub use lifecycle::{trigger_export_now, snapshot_json}; pub use auto_export::{MemScopeConfig, AutoExportConfig, SignalPolicy, ExportFormatSet};`. Extend `prelude` with `start`, `MemScopeGuard`, `MemScopeConfig`.
- **[global_tracking.rs](file:///Users/scc/code/rustcode/memscope-rs/src/capture/backends/global_tracking.rs)**: add `pub auto_export: AutoExportConfig` field to `GlobalTrackerConfig`; fix `with_config` to pass `config.tracker` into a new `Tracker::with_config(config)` (add if missing). Visibility-only changes — no new logic in this file.
- **[tracker.rs](file:///Users/scc/code/rustcode/memscope-rs/src/tracker.rs)**: make `TrackerConfig` `pub` (1-word visibility change, no new lines); add `Tracker::with_config(cfg: TrackerConfig) -> Self` if absent. File already over 1000 lines — only minimal visibility changes, no logic additions.
- **examples** ([global_tracker_showcase.rs](file:///Users/scc/code/rustcode/memscope-rs/examples/global_tracker_showcase.rs), [basic_usage.rs](file:///Users/scc/code/rustcode/memscope-rs/examples/basic_usage.rs), [actix_web_server.rs](file:///Users/scc/code/rustcode/memscope-rs/examples/actix_web_server.rs)): rewrite setup to `let _g = memscope_rs::start()?;` one-liner, remove manual `export_html` at end of `main`.

### Public API surface

```rust
// One-line start (default config: HTML+JSON to ./memscope-report, panic hook on, ctrlc on)
pub fn start() -> MemScopeResult<MemScopeGuard>;

// Configured start
pub fn start_with(config: MemScopeConfig) -> MemScopeResult<MemScopeGuard>;

// On-demand export from anywhere (idempotent, resets so can be called again)
pub fn trigger_export_now() -> bool;

// In-memory JSON snapshot (no disk write) — for HTTP endpoints
pub fn snapshot_json() -> MemScopeResult<String>;
```

```rust
// src/auto_export.rs
pub struct AutoExportConfig {
    pub output_path: PathBuf,        // default: ./memscope-report
    pub formats: ExportFormatSet,    // default: HTML | JSON
    pub on_exit: bool,               // default: true
    pub on_panic: bool,              // default: true  (critical for release panic=abort)
    pub on_signal: SignalPolicy,     // default: CtrlC
    pub flush_interval: Option<Duration>, // default: None
    pub exit_timeout: Duration,      // default: 5s
}
impl Default for AutoExportConfig { ... }

pub enum SignalPolicy { Off, CtrlC, CtrlCAndTerm }

pub struct ExportFormatSet(u8);  // bitflags-style, no new dependency
impl ExportFormatSet {
    pub const HTML: Self;
    pub const JSON: Self;
    pub const HTML_JSON: Self;
}

pub struct MemScopeConfig {
    pub auto_export: AutoExportConfig,
    pub tracker: GlobalTrackerConfig,
}
impl Default for MemScopeConfig { ... }
```

```rust
// src/guard.rs
pub struct MemScopeGuard {
    tracker: Arc<GlobalTracker>,
    flusher: Option<PeriodicFlusher>,
    auto_export: AutoExportConfig,
}
impl Deref for MemScopeGuard { type Target = GlobalTracker; ... }
impl Drop for MemScopeGuard {
    fn drop(&mut self) {
        if let Some(f) = self.flusher.take() { f.stop(); }  // join worker, final flush
        lifecycle::export_once(&self.auto_export);           // idempotent
    }
}
```

```rust
// src/lifecycle.rs
static EXPORTED: AtomicBool = AtomicBool::new(false);
static TRACKER_HANDLE: OnceLock<Arc<GlobalTracker>> = OnceLock::new();
static AUTO_EXPORT_CFG: OnceLock<AutoExportConfig> = OnceLock::new();
static PREV_PANIC_HOOK: OnceLock<Box<dyn Fn(&std::panic::PanicInfo) + Send + Sync>> = OnceLock::new();

pub(crate) fn install_exit_handlers(cfg: &AutoExportConfig, tracker: Arc<GlobalTracker>) -> MemScopeResult<()>;
pub fn export_once(cfg: &AutoExportConfig) -> bool;       // returns true if this call won the CAS
pub fn trigger_export_now() -> bool;                       // resets EXPORTED, then export_once
pub fn snapshot_json() -> MemScopeResult<String>;
```

```rust
// src/periodic_flusher.rs
pub struct PeriodicFlusher {
    shutdown_tx: std::sync::mpsc::Sender<()>,           // separate control channel (OTel lesson)
    handle: parking_lot::Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl PeriodicFlusher {
    pub fn start(interval: Duration, cfg: AutoExportConfig) -> Self;
    pub fn stop(&self);  // send shutdown, take handle, join
}
impl Drop for PeriodicFlusher { fn drop(&mut self) { self.stop(); } }
```

### Cargo features ([Cargo.toml](file:///Users/scc/code/rustcode/memscope-rs/Cargo.toml))

```toml
[features]
default = ["derive", "auto-signal", "periodic"]
auto-signal = ["dep:ctrlc"]   # SIGINT/SIGTERM handler
atexit = []                   # libc::atexit for process::exit (libc already a dep)
periodic = []                 # background flusher worker thread

[dependencies]
ctrlc = { version = "3.4", optional = true }
# libc already present at line 31
```

### Backward compatibility

- `MemCtx::init()` and `ctx.export(path)` remain — existing code keeps working.
- `init_global_tracking()` / `global_tracker()` unchanged.
- `GlobalTrackerConfig` gains one field with a `Default` impl — additive.

## Testing Plan (per rules.md Section II–IV)

Each new module gets the **Golden Trio** (Positive / Negative / Stress) + module-specific advanced checks. No `println!` (use `tracing` test subscriber). Every `assert!` carries a message. Tests live in `#[cfg(test)] mod tests` at end of each file. **No coverage tooling run** (rules.md #9) but tests aim at >90% line + 100% unsafe.

### `auto_export.rs`
- **Positive**: `Default::default()` produces `output_path=./memscope-report`, `formats=HTML|JSON`, `on_exit=true`, `on_panic=true`, `on_signal=CtrlC`, `flush_interval=None`.
- **Negative**: `ExportFormatSet::from(0u8)` is empty; `SignalPolicy::Off` serializes.
- **Property** (proptest, 1000 cases): random `ExportFormatSet` bits round-trip through `to_bits()`/`from_bits()`; random `SignalPolicy` serializes/deserializes via serde.
- **Panic test**: `#[should_panic]` for `output_path` empty string in debug builds.

### `periodic_flusher.rs`
- **Positive**: with `interval=50ms`, in `200ms` the worker fires ≥3 flushes (counter via `Arc<AtomicUsize>`).
- **Negative**: `stop()` called before first tick → flush counter stays 0; `stop()` is idempotent (call twice, no panic, no double-join).
- **Stress** (50 threads): all call `stop()` concurrently → `JoinHandle` taken exactly once (`Mutex<Option<JoinHandle>>` invariant).
- **Concurrency (loom)**: model `start` + `stop` + worker loop, assert no deadlock and exactly one final flush.
- **Timeout**: `stop()` returns within `exit_timeout + 1s` even if worker is mid-flush.

### `lifecycle.rs`
- **Positive**: with a tracker holding 1 allocation, `export_once(cfg)` writes `report.html` + `report.json` to tempdir; returns `true`.
- **Negative (idempotency)**: second `export_once(cfg)` returns `false`, writes nothing new (compare file mtimes).
- **Negative (no tracker)**: `export_once` when `TRACKER_HANDLE` unset returns `false`, does **not** panic.
- **Stress (50 threads)**: all call `export_once` concurrently → exactly one returns `true`, others `false`; output dir has exactly one HTML + one JSON.
- **Panic-hook chaining**: install hook, trigger a `panic!("test")` in a child thread joined with `catch_unwind` → previous default hook still prints (capture stderr via `tracing` subscriber); `EXPORTED` is `true` after.
- **`trigger_export_now`**: resets `EXPORTED=false` then exports; calling twice produces two exports.
- **`snapshot_json`**: returns non-empty JSON containing `"total_allocations"`.
- **Miri**: `atexit` feature path uses `unsafe extern "C"` — `cargo miri test` must pass 0 errors on this module.
- **proptest (1000)**: random valid `AutoExportConfig` → `export_once` never panics, returns bool.

### `guard.rs`
- **Positive**: construct `MemScopeGuard` (via `start()`), drop it → `report.html` exists.
- **Negative**: drop after `trigger_export_now()` already exported → `EXPORTED` already `true`, drop is no-op (no duplicate file).
- **Negative**: guard dropped after `reset_global_tracking()` → graceful, no panic, logs error.
- **Stress (50 threads)**: 50 guards (each `start()` on a fresh `reset_global_tracking()`) dropped concurrently → no deadlock.
- **Integration**: `start()` → track 1 alloc → drop → assert file contents contain the tracked type name.

### `mem_ctx.rs` changes
- **Positive**: `MemCtx::start()` returns guard; derefs to `GlobalTracker`; `track!` works through it.
- **Backward-compat**: `MemCtx::init()` + `export()` still works exactly as before (regression: existing example output unchanged).

### Integration / example tests
- Rewrite `examples/global_tracker_showcase.rs` to use `start()`; run via `cargo run --example global_tracker_showcase`; assert `./memscope-report/dashboard_unified_dashboard.html` exists on normal exit.
- Ctrl-C test: spawn example, send SIGINT after 2s, assert report dir appears (shell-driven integration test in `tests/auto_export_e2e.rs`).
- Panic test: example that panics after tracking → report dir appears (release build, `panic=abort`).

## Implementation Order (逐模块 — rules.md #8)

Each module: implement → tests → `make fmt` → `make check` (0 errors) → `cargo clippy --tests` → next module.

1. **`auto_export.rs`** (config types, zero deps) + tests + fmt + check.
2. **`periodic_flusher.rs`** (depends on `auto_export`) + tests (incl. loom) + fmt + check.
3. **`lifecycle.rs`** (depends on `auto_export` + `GlobalTracker`) + tests (incl. miri on atexit path, proptest) + fmt + check.
4. **`guard.rs`** (depends on `lifecycle` + `periodic_flusher`) + tests + fmt + check.
5. **`mem_ctx.rs`** additions (`start`/`start_with`) — backward-compat regression test.
6. **`lib.rs`** module declarations + re-exports + prelude.
7. **`global_tracking.rs`** visibility fix (`pub TrackerConfig`, `pub auto_export` field) + fix `with_config` to consume `config.tracker`.
8. **`tracker.rs`** visibility-only (`pub struct TrackerConfig`) + `Tracker::with_config` if absent.
9. **Cargo.toml** features + `ctrlc` optional dep.
10. **Examples rewrite** (3 files) — one-liner `start()`.
11. **`tests/auto_export_e2e.rs`** integration test (Ctrl-C + panic paths).
12. **codescope verification**: `index_project` → `find_callers("export_once")` confirms all four hooks (Drop / panic / ctrlc / atexit) resolve to the single export function; `verify_claim` on "`MemScopeGuard` implements Drop" and "`start()` calls `set_hook`".
13. **Multi-agent code review**: 3 agents (one per: API ergonomics & backward compat; concurrency safety / lock-free invariants; test rigor vs rules.md Section II–V). Cross-validate findings, dismiss false positives, fix real issues.

## Acceptance Criteria (rules.md)

- [ ] `make fmt` clean.
- [ ] `make check` 0 errors (warnings may remain per rules.md #4).
- [ ] `cargo clippy --tests` green (no `#[allow(dead_code)]` per rules.md #5).
- [ ] No `unwrap()`/`expect()` in lib code without an infallibility comment (rules.md error-handling).
- [ ] Comments in English, ~7:3 code:comment ratio (rules.md commenting).
- [ ] Every new file < 1000 lines (rules.md #1).
- [ ] No files deleted, no `rm` / git commands (rules.md #6, hard constraint).
- [ ] Golden Trio + Miri (unsafe path) + proptest (1000 cases) + loom (concurrency) for each new module.
- [ ] Backward compat: existing examples that still use `MemCtx::init()` continue to compile and produce identical output.
- [ ] codescope `find_callers("export_once")` shows all 4 exit hooks pointing to one function.
- [ ] End-to-end: `cargo run --example global_tracker_showcase` then Ctrl-C → `./memscope-report/` exists with HTML+JSON.

## Verification (end-to-end)

```bash
# 1. Format + static checks
make fmt && make check

# 2. Lint tests
cargo clippy --tests --all-features

# 3. Miri on unsafe path (atexit feature)
cargo +nightly miri test --features atexit lifecycle:: -- -j 4

# 4. Run example, Ctrl-C it, verify auto-export
cargo run --example global_tracker_showcase &
EXAMPLE_PID=$!
sleep 2
kill -INT $EXAMPLE_PID
wait $EXAMPLE_PID
ls ./memscope-report/  # expect dashboard_unified_dashboard.html + *.json

# 5. Panic-abort path (release)
cargo build --release --example panic_exit_demo  # new example that panics
./target/release/panic_exit_demo || true
ls ./memscope-report/  # expect report despite panic=abort

# 6. codescope call-graph verification
# (via run_mcp: index_project → find_callers("export_once") → expect 4 callers)
```

## Multi-Agent Implementation Plan

After plan approval, spawn in parallel:
- **Agent A** (general-purpose): implement `auto_export.rs` + `periodic_flusher.rs` + their tests (modules 1–2). Independent, no shared files with B/C.
- **Agent B** (general-purpose): implement `lifecycle.rs` + `guard.rs` + their tests (modules 3–4). Depends on A's `AutoExportConfig` signature (provided in this plan).
- **Agent C** (general-purpose): after A+B finish, do module 5–9 (existing-file edits: `mem_ctx.rs`, `lib.rs`, `global_tracking.rs`, `tracker.rs`, `Cargo.toml`) + examples rewrite (module 10) + e2e test (module 11).

Then sequential: codescope verification (module 12) + 3-agent code review (module 13).

## Risk / Trade-offs

- **`ctrlc` global handler is singleton**: if host app already registered one, `ctrlc::set_handler` returns `Err`. Use `try_set_handler` and degrade gracefully (log warning, keep Drop + panic-hook coverage). Documented in `AutoExportConfig` docstring.
- **`panic = "abort"` + panic hook**: hook runs before abort (confirmed Rust behavior) — safe, but hook must be fast and allocation-free-ish (avoid re-entrancy). `export_once` uses `OnceLock`-stored handle, no new alloc in the hot path beyond file IO.
- **`atexit` is `unsafe`**: gated behind off-by-default feature; Miri-tested; the registered `extern "C" fn` only reads `OnceLock` (sync-safe).
- **Thread-locals during panic unwind**: `export_once` must not rely on thread-local tracker state; it reads the `Arc<GlobalTracker>` from `OnceLock` (Send+Sync).
