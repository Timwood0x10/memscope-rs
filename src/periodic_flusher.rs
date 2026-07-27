//! Periodic background flusher for memscope-rs.
//!
//! Spawns a dedicated worker thread that calls a user-provided `flush` closure
//! every `interval`, then joins cleanly on shutdown with a final flush. The
//! design mirrors OpenTelemetry's `BatchSpanProcessor` and `tracing-appender`'s
//! `WorkerGuard`:
//!
//! - A SEPARATE control channel carries the shutdown signal, so a full data
//!   channel cannot drop or delay it (this was a documented OTel bug).
//! - `recv_timeout(interval)` drives both periodic flushing and prompt
//!   shutdown: the worker either times out (→ flush) or receives a shutdown
//!   message (→ final flush + exit).
//! - `Mutex<Option<JoinHandle>>` lets `stop()` take the handle and join
//!   exactly once, even under concurrent `stop()` calls.
//!
//! This is module 2 of the 4-module auto-export feature. It is intentionally
//! closure-based (rather than importing module 1's `AutoExportConfig`) so the
//! worker can be unit-tested with a trivial `Arc<AtomicUsize>` counter and no
//! I/O. Downstream modules wire real flush logic (e.g. calling the tracker's
//! export routine) through this abstraction.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;

/// Worker thread handle returned to the caller. Dropping it triggers a graceful
/// shutdown (send shutdown signal, join worker, worker performs a final flush).
///
/// The flusher is `Send + Sync` because the only mutable state (`handle`) is
/// behind a `parking_lot::Mutex`. `stop()` is safe to call from multiple
/// threads concurrently — exactly one call wins the join.
pub struct PeriodicFlusher {
    /// Control channel: sending `()` tells the worker to shut down after its
    /// next (final) flush. Kept as a field so `stop()` can send the signal.
    /// Uses `SyncSender` (bounded `sync_channel`) so a full channel never
    /// blocks the sender indefinitely on shutdown.
    shutdown_tx: mpsc::SyncSender<()>,
    /// Worker join handle, wrapped so `stop()` can take it exactly once.
    /// `None` after `stop()` has joined.
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl PeriodicFlusher {
    /// Spawn a worker thread that calls `flush_fn` every `interval`.
    ///
    /// The worker performs a final `flush_fn()` call on shutdown (whether
    /// triggered by `stop()` or by `Drop`), so callers can rely on the last
    /// interval's data being flushed.
    ///
    /// # Parameters
    /// - `interval`: time between flushes. Must be > 0 (panics on zero, since
    ///   a zero interval would busy-loop; this is a programmer error, not a
    ///   runtime condition).
    /// - `flush_fn`: the closure to call on each tick and on final shutdown.
    ///   Must be `Send + Sync + 'static` because the worker thread owns it via
    ///   `Arc`. It is `Sync` so the final-shutdown path and the periodic path
    ///   (which are serialised inside the worker) can both invoke it.
    ///
    /// # Panics
    /// Panics if `interval` is zero (programmer error — use a non-zero duration).
    /// Also panics if the OS fails to spawn the worker thread (extremely rare;
    /// indicates resource exhaustion). The panic message documents both cases.
    #[must_use]
    pub fn new<F>(interval: Duration, flush_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        assert!(
            !interval.is_zero(),
            "PeriodicFlusher::new: interval must be non-zero (a zero interval would busy-loop)"
        );
        // Use a bounded sync channel of capacity 1 for the shutdown signal.
        // Capacity 1 is enough: at most one shutdown signal is ever meaningful;
        // additional sends (e.g. concurrent stop() + Drop) are silently dropped
        // by the receiver once shutdown has begun.
        let (shutdown_tx, shutdown_rx) = mpsc::sync_channel::<()>(1);
        let flush_arc = Arc::new(flush_fn);

        // The worker thread holds the only strong reference to `flush_arc`
        // (via `worker_flush` below, moved into the thread closure). We do not
        // need to keep a second reference alive on the flusher side: the
        // worker's clone keeps the Arc alive for the worker's entire lifetime,
        // and the closure is only ever invoked from inside the worker.
        let worker_flush = Arc::clone(&flush_arc);

        let handle = thread::Builder::new()
            .name("memscope-flusher".to_string())
            .spawn(move || {
                worker_loop(shutdown_rx, worker_flush, interval);
            })
            // SAFETY / rationale: thread::Builder::spawn only fails when the OS
            // cannot create a thread (RLIMIT_NPROC, RLIMIT_AS, fork-bombed host).
            // This is a rare, unrecoverable resource-exhaustion condition; there
            // is no graceful fallback. Propagating as a panic is consistent with
            // std::thread::spawn's own contract.
            .expect(
                "PeriodicFlusher::new: OS failed to spawn worker thread (resource exhaustion?)",
            );

        Self {
            shutdown_tx,
            handle: Mutex::new(Some(handle)),
        }
    }

    /// Gracefully shut down the worker: send the shutdown signal, then join.
    ///
    /// Idempotent: calling `stop()` multiple times (including via `Drop`) is
    /// safe — only the first call wins the join; subsequent calls are no-ops.
    /// Safe to call concurrently from multiple threads.
    ///
    /// After `stop()` returns, the worker has performed its final flush and
    /// exited. If the worker thread panicked, the panic is silently absorbed
    /// (a flush failure should not crash the host process on shutdown).
    pub fn stop(&self) {
        // Take the handle under the lock so concurrent callers race for a
        // single Some(handle); losers see None and return early.
        let handle_opt = self.handle.lock().take();
        let Some(handle) = handle_opt else {
            // Already stopped — idempotent no-op.
            return;
        };
        // Signal the worker to wake up immediately (it may be mid-sleep in
        // recv_timeout). Ignore send errors: the worker may have already exited.
        let _ = self.shutdown_tx.send(());
        // Join the worker. Absorb a panic: a flush panic must not propagate
        // into the host's shutdown path. Log via tracing if available.
        if let Err(_panic_payload) = handle.join() {
            // Worker thread panicked during its final flush. We intentionally
            // swallow the payload: the host process is shutting down and a
            // flush failure should not change its exit semantics.
            tracing::warn!(
                target: "memscope::periodic_flusher",
                "periodic flusher worker thread panicked during shutdown; payload suppressed"
            );
        }
    }
}

impl Drop for PeriodicFlusher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Worker loop: wait up to `interval` for a shutdown signal; on timeout call
/// `flush`; on signal, do a final `flush` and return.
///
/// This is the body of the worker thread. It runs until one of:
/// - A shutdown signal is received on the control channel (`Ok(())`).
/// - The control channel's sender is dropped without a signal (`Disconnected`).
///
/// In both cases a final flush is performed before the loop exits, so the last
/// interval's data is always flushed.
fn worker_loop<F: Fn() + Send + Sync + 'static>(
    shutdown_rx: mpsc::Receiver<()>,
    flush: Arc<F>,
    interval: Duration,
) {
    loop {
        match shutdown_rx.recv_timeout(interval) {
            Ok(()) => {
                // Shutdown signalled: perform a final flush, then exit.
                // Errors in flush are absorbed here (same rationale as stop()).
                flush_with_trap(&flush);
                return;
            }
            Err(RecvTimeoutError::Timeout) => {
                // Interval elapsed: periodic flush, then continue.
                flush_with_trap(&flush);
            }
            Err(RecvTimeoutError::Disconnected) => {
                // Sender dropped without sending a signal (e.g. flusher was
                // dropped via the Mutex path). Do a final flush and exit.
                flush_with_trap(&flush);
                return;
            }
        }
    }
}

/// Invoke the flush closure, trapping any panic so the worker thread survives
/// to flush again on the next tick (and on shutdown). A single failing flush
/// must not kill the worker.
///
/// `AssertUnwindSafe` is sound here: the closure only borrows `&Arc<F>` (a
/// shared reference) and `F: Fn()` cannot mutate external state through the
/// reference. Even if `F` internally mutates state (e.g. an `AtomicUsize`),
/// the panic boundary leaves that state in a consistent atomic state.
fn flush_with_trap<F: Fn() + Send + Sync + 'static>(flush: &Arc<F>) {
    // Use catch_unwind so a panicking flush does not tear down the worker.
    // The closure captures only &Arc<F> (a reference), which is UnwindSafe.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        flush();
    }));
    if result.is_err() {
        tracing::warn!(
            target: "memscope::periodic_flusher",
            "flush closure panicked; suppressing to keep worker alive"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests — Golden Trio (positive / negative / stress) + loom invariants.
// All asserts carry descriptive messages. No println! in tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    use parking_lot::Mutex as PlMutex;

    /// Objective: Verify the worker fires flush at least 3 times over 200ms
    /// with a 50ms interval. The bound is a lower bound — scheduling jitter
    /// may add ticks but must not remove them.
    /// Invariants: counter >= 3 after sleeping ~4 intervals.
    #[test]
    fn positive_periodic_flush_fires_multiple_times() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(50), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        // Sleep ~4 intervals. Even with jitter, at least 3 ticks should fire.
        std::thread::sleep(Duration::from_millis(200));
        let count = counter.load(Ordering::SeqCst);
        assert!(
            count >= 3,
            "expected at least 3 flushes over 200ms with 50ms interval, got {} \
             (scheduling jitter may add ticks but not remove them below 3)",
            count
        );
        flusher.stop();
    }

    /// Objective: Verify that `stop()` triggers a final flush — the counter
    /// must strictly increase after `stop()` returns.
    /// Invariants: post_stop_count > pre_stop_count (final flush fired).
    #[test]
    fn positive_stop_triggers_final_flush() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(50), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        std::thread::sleep(Duration::from_millis(150));
        let pre_stop = counter.load(Ordering::SeqCst);
        assert!(
            pre_stop >= 1,
            "expected at least one periodic flush before stop, got {}",
            pre_stop
        );
        flusher.stop();
        let post_stop = counter.load(Ordering::SeqCst);
        assert!(
            post_stop > pre_stop,
            "stop() must trigger a final flush: post_stop ({}) must be > pre_stop ({})",
            post_stop,
            pre_stop
        );
    }

    /// Objective: Verify `stop()` returns promptly — well within `interval + 1s`.
    /// This is the exit-timeout bound: the worker should wake immediately on
    /// the shutdown signal and exit after one final flush.
    /// Invariants: elapsed <= interval + 1s.
    #[test]
    fn positive_stop_returns_within_bound() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let interval = Duration::from_millis(50);
        let flusher = PeriodicFlusher::new(interval, move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        std::thread::sleep(Duration::from_millis(100));
        let start = Instant::now();
        flusher.stop();
        let elapsed = start.elapsed();
        let bound = interval + Duration::from_secs(1);
        assert!(
            elapsed <= bound,
            "stop() must return within {:?} (interval + 1s), took {:?}",
            bound,
            elapsed
        );
    }

    /// Objective: Verify the worker thread is named "memscope-flusher" for
    /// diagnostics. The name is captured from inside the flush closure via
    /// `std::thread::current().name()`.
    /// Invariants: captured name == Some("memscope-flusher").
    #[test]
    fn positive_worker_thread_name() {
        let seen_name: Arc<PlMutex<Option<String>>> = Arc::new(PlMutex::new(None));
        let sn = Arc::clone(&seen_name);
        let flusher = PeriodicFlusher::new(Duration::from_millis(50), move || {
            let name = std::thread::current().name().map(|s| s.to_string());
            if name.is_some() {
                *sn.lock() = name;
            }
        });
        // Sleep enough for at least one flush to fire and capture the name.
        std::thread::sleep(Duration::from_millis(150));
        flusher.stop();
        let name = seen_name.lock().take();
        assert_eq!(
            name.as_deref(),
            Some("memscope-flusher"),
            "worker thread must be named 'memscope-flusher' for diagnostics, got {:?}",
            name
        );
    }

    /// Objective: Verify `new()` panics on a zero interval — this is a
    /// programmer error (a zero interval would busy-loop the worker).
    /// Invariants: should_panic with the documented message.
    #[test]
    #[should_panic(expected = "interval must be non-zero")]
    fn negative_zero_interval_panics() {
        // The closure is never invoked because new() panics first.
        let _flusher = PeriodicFlusher::new(Duration::ZERO, || {});
        // If we reach here, the panic contract was violated.
        panic!("PeriodicFlusher::new with Duration::ZERO must panic before reaching this line");
    }

    /// Objective: Verify `stop()` is idempotent — calling it twice does not
    /// panic, deadlock, or fire an extra final flush. The second call must be
    /// a silent no-op because the handle was already taken.
    /// Invariants: counter unchanged between the second stop() and the check
    /// after it (no extra final flush).
    #[test]
    fn negative_stop_called_twice_is_noop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(50), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        std::thread::sleep(Duration::from_millis(100));
        flusher.stop();
        let after_first = counter.load(Ordering::SeqCst);
        assert!(
            after_first >= 1,
            "expected at least one flush after first stop(), got {}",
            after_first
        );
        // Second call must be a no-op: no panic, no deadlock, no extra flush.
        flusher.stop();
        let after_second = counter.load(Ordering::SeqCst);
        assert_eq!(
            after_second, after_first,
            "second stop() must be a no-op: counter must be unchanged \
             (after_first={}, after_second={})",
            after_first, after_second
        );
    }

    /// Objective: Verify `stop()` called immediately (before the first tick
    /// would fire) returns cleanly and fires at most one flush (the final
    /// flush). The worker may or may not have ticked yet, so 0 or 1 is valid.
    /// Invariants: counter <= 1 after immediate stop().
    #[test]
    fn negative_stop_immediately() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(50), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        // Stop immediately — at most the final flush fires (counter <= 1).
        flusher.stop();
        let count = counter.load(Ordering::SeqCst);
        assert!(
            count <= 1,
            "immediate stop() should fire at most one flush (the final one), got {}",
            count
        );
    }

    /// Objective: Verify `Drop` triggers graceful shutdown (calls stop()
    /// internally) — the worker joins and the final flush fires. If Drop did
    /// not join, the test would race the background thread or hang.
    /// Invariants: counter >= 1 after drop; test completes without hanging.
    #[test]
    fn negative_drop_triggers_shutdown() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(20), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        // Let at least one periodic flush fire, then drop without calling stop().
        std::thread::sleep(Duration::from_millis(80));
        let pre_drop = counter.load(Ordering::SeqCst);
        assert!(
            pre_drop >= 1,
            "expected at least one periodic flush before drop, got {}",
            pre_drop
        );
        drop(flusher);
        // If drop() didn't join the worker, the final flush might not have
        // fired yet (race). The assert below verifies it did.
        let post_drop = counter.load(Ordering::SeqCst);
        assert!(
            post_drop > pre_drop,
            "drop() must trigger a final flush: post_drop ({}) > pre_drop ({})",
            post_drop,
            pre_drop
        );
    }

    /// Objective: Verify a panicking flush closure does not kill the worker —
    /// `catch_unwind` traps the panic and subsequent flushes still fire. This
    /// is the core robustness invariant of the worker loop.
    /// Invariants: counter >= 2 (first flush panics but is counted; worker
    /// survives and flushes again); final flush also fires after stop().
    #[test]
    fn negative_flush_panic_does_not_kill_worker() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = PeriodicFlusher::new(Duration::from_millis(20), move || {
            let prev = c.fetch_add(1, Ordering::SeqCst);
            // Panic only on the very first flush to test catch_unwind.
            if prev == 0 {
                panic!("intentional flush panic — worker must survive via catch_unwind");
            }
        });
        // Sleep enough for the first (panicking) flush and at least one more.
        std::thread::sleep(Duration::from_millis(100));
        let count = counter.load(Ordering::SeqCst);
        assert!(
            count >= 2,
            "worker must survive a flush panic and continue flushing: \
             expected >= 2 (1 panicked + 1 survived), got {}",
            count
        );
        flusher.stop();
        let final_count = counter.load(Ordering::SeqCst);
        assert!(
            final_count > count,
            "final flush must fire even after a prior panic: \
             final_count ({}) > count ({})",
            final_count,
            count
        );
    }

    /// Objective: Stress-test 50 concurrent `stop()` calls — all must return
    /// without deadlock or panic, and the final flush must fire exactly once
    /// (not 50 times). This proves the `Mutex<Option<JoinHandle>>` take-and-join
    /// pattern is race-free under high contention.
    /// Invariants:
    /// - All 50 threads join (no deadlock, no panic).
    /// - post_stop > pre_stop (final flush fired).
    /// - (post_stop - pre_stop) <= 3 (at most one periodic tick + one final
    ///   flush during the storm; 50 stop() calls must not cause 50 final flushes).
    #[test]
    fn stress_50_concurrent_stop_calls() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let flusher = Arc::new(PeriodicFlusher::new(Duration::from_millis(10), move || {
            c.fetch_add(1, Ordering::SeqCst);
        }));
        // Sleep 50ms so a few periodic flushes fire before the storm.
        std::thread::sleep(Duration::from_millis(50));
        let pre_stop = counter.load(Ordering::SeqCst);
        assert!(
            pre_stop >= 1,
            "expected at least one periodic flush before the stop storm, got {}",
            pre_stop
        );

        // Spawn 50 threads all calling stop() concurrently.
        let mut handles = Vec::with_capacity(50);
        for _ in 0..50 {
            let f = Arc::clone(&flusher);
            handles.push(std::thread::spawn(move || f.stop()));
        }
        // Join all 50 — no deadlock, no panic. If any thread deadlocks or
        // panics, join() will hang or return Err, failing this assert.
        for h in handles {
            h.join()
                .expect("every stop() thread must join without panic (no deadlock)");
        }
        // After all 50 joined, the worker has exited. The Arc<PeriodicFlusher>
        // is still alive (we hold one), but stop() has been called. Drop it
        // cleanly — its Drop will be a no-op because stop() already ran.
        drop(flusher);

        let post_stop = counter.load(Ordering::SeqCst);
        assert!(
            post_stop > pre_stop,
            "final flush must fire during stop storm: post_stop ({}) > pre_stop ({})",
            post_stop,
            pre_stop
        );
        let storm_increments = post_stop - pre_stop;
        // At most a few flushes (one periodic tick + one final flush) should
        // fire during the stop storm. The key invariant: 50 concurrent stop()
        // calls do NOT cause 50 final flushes — the take-and-join pattern
        // ensures exactly one final flush.
        assert!(
            storm_increments <= 3,
            "at most a few flushes (one periodic + one final) should fire during \
             the stop storm, got {} increments; 50 concurrent stop() calls must \
             not cause 50 final flushes",
            storm_increments
        );
    }
}

// ---------------------------------------------------------------------------
// loom concurrency-invariant tests.
//
// loom cannot model `std::sync::mpsc`, so these tests MODEL the shutdown
// invariant using loom's own primitives. They only compile/run under
// `RUSTFLAGS="--cfg loom" cargo test` and are skipped in normal `cargo test`.
// ---------------------------------------------------------------------------

#[cfg(loom)]
mod loom_tests {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;

    /// Objective: Model the "exactly one stop() wins the join" invariant.
    /// The real `PeriodicFlusher::stop()` uses `Mutex<Option<JoinHandle>>` and
    /// `take()` so that under any interleaving of concurrent stop() calls,
    /// exactly one caller observes `Some(handle)` and joins; all others see
    /// `None` and return early. This test models that with
    /// `loom::sync::Mutex<Option<()>>` (standing in for the handle) and an
    /// `AtomicUsize` join counter.
    /// Invariants: Under any interleaving, the join counter is exactly 1 after
    /// both threads call stop(). No double-join, no missed join.
    #[test]
    fn concurrent_stop_joins_exactly_once() {
        loom::model(|| {
            // Model Mutex<Option<JoinHandle>>: Some = handle present, None = taken.
            let handle = Arc::new(Mutex::new(Some(())));
            // Counts how many threads won the join (took the handle).
            let join_count = Arc::new(AtomicUsize::new(0));

            let h1_handle = Arc::clone(&handle);
            let h1_count = Arc::clone(&join_count);
            let h2_handle = Arc::clone(&handle);
            let h2_count = Arc::clone(&join_count);

            let t1 = thread::spawn(move || {
                // Models: if self.handle.lock().take().is_some() { join() }
                if h1_handle.lock().take().is_some() {
                    h1_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            let t2 = thread::spawn(move || {
                if h2_handle.lock().take().is_some() {
                    h2_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            t1.join().expect("loom model: t1 must join without panic");
            t2.join().expect("loom model: t2 must join without panic");
            assert_eq!(
                join_count.load(Ordering::SeqCst),
                1,
                "exactly one stop() call must win the join under any interleaving; \
                 got {}",
                join_count.load(Ordering::SeqCst)
            );
        });
    }

    /// Objective: Model the idempotent-stop invariant — once the handle has been
    /// taken, any number of subsequent stop() calls are no-ops (no double-join).
    /// Three threads race; still exactly one join.
    /// Invariants: join_count == 1 after all three threads finish.
    #[test]
    fn concurrent_stop_idempotent_under_three_threads() {
        loom::model(|| {
            let handle = Arc::new(Mutex::new(Some(())));
            let join_count = Arc::new(AtomicUsize::new(0));

            let threads: Vec<_> = (0..3)
                .map(|_| {
                    let h = Arc::clone(&handle);
                    let c = Arc::clone(&join_count);
                    thread::spawn(move || {
                        if h.lock().take().is_some() {
                            c.fetch_add(1, Ordering::SeqCst);
                        }
                    })
                })
                .collect();

            for (i, t) in threads.into_iter().enumerate() {
                t.join()
                    .unwrap_or_else(|_| panic!("loom model: thread {} must join without panic", i));
            }
            assert_eq!(
                join_count.load(Ordering::SeqCst),
                1,
                "exactly one of three concurrent stop() calls must win the join; \
                 got {}",
                join_count.load(Ordering::SeqCst)
            );
        });
    }
}
