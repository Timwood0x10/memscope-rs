//! Basic usage example for memscope-rs memory visualizer.
//!
//! This example demonstrates the unified one-line start API:
//! - `start()` returns an RAII guard that initializes logging + global tracking
//!   and installs the auto-export lifecycle hooks (panic / Ctrl-C / drop).
//! - `track!` records allocations through the guard's `Deref<Target = GlobalTracker>`.
//! - On `main` return (or panic / Ctrl-C), the guard's `Drop` runs the idempotent
//!   exit-path export, writing HTML + JSON to `./memscope-report` by default.
//!
//! Compare with the older `MemCtx::init()` + `ctx.export(path)` flow: that still
//! works (see `mem_ctx.rs`), but the new `start()` API requires no explicit
//! export call.

use memscope_rs::{analyzer, track, MemScopeResult};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

fn main() -> MemScopeResult<()> {
    println!("Basic Usage Example - Unified `start()` API");
    println!("============================================\n");

    let start_time = Instant::now();
    // One-line start: logging + global tracker + auto-export hooks. The guard
    // owns the lifecycle; dropping it (or panic / Ctrl-C) triggers the export.
    let guard = memscope_rs::start()?;

    let data = vec![1, 2, 3, 4, 5];
    track!(guard, data);

    let string_data = String::from("Hello, world!");
    track!(guard, string_data);

    let rc_data = Rc::new(vec![1.0, 2.0, 3.0]);
    track!(guard, rc_data);

    let arc_data = Arc::new(vec![1.0, 2.0, 3.0]);
    track!(guard, arc_data);

    let boxed_data = Box::new(42);
    track!(guard, boxed_data);

    let duration = start_time.elapsed();

    // Use the unified Analyzer API. The guard derefs to `GlobalTracker`, so
    // `analyzer(&guard)` works directly.
    println!("\n=== Unified Analyzer API ===\n");

    let mut az = analyzer(&guard)?;

    // Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);
    println!("  Threads: {}", report.stats.thread_count);
    println!();

    // Leak detection
    let leaks = az.detect().leaks();
    println!("Leak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);
    println!();

    // Metrics
    let metrics = az.metrics().summary();
    println!("Metrics:");
    println!("  Allocation Count: {}", metrics.allocation_count);
    println!("  Total Bytes: {}", metrics.total_bytes);
    println!("  Types: {}", metrics.by_type.len());
    println!();

    // On-demand in-memory JSON snapshot (no disk write) — useful for HTTP endpoints.
    let snapshot = guard.snapshot_json()?;
    println!("In-memory JSON snapshot length: {} bytes", snapshot.len());

    // No explicit `export_html` / `export_json` call: dropping `guard` (or
    // panic / Ctrl-C) writes the full HTML dashboard + JSON to
    // `./memscope-report/` automatically.
    println!("\nDrop will write dashboard + JSON to ./memscope-report/");

    println!(
        "\nExample finished in {:.2}ms",
        duration.as_secs_f64() * 1000.0
    );

    // `_guard` drops here, triggering the exit-path export.
    drop(guard);

    Ok(())
}
