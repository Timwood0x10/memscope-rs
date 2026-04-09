//! Comprehensive benchmarks for memscope-rs
//!
//! Run with: cargo bench

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memscope_rs::capture::backends::{
    AsyncBackend, CaptureBackend, CoreBackend, LockfreeBackend, UnifiedCaptureBackend,
};
use memscope_rs::capture::types::AllocationInfo;
use memscope_rs::classification::TypeClassifier;
use memscope_rs::tracking::stats::TrackingStats;
use memscope_rs::{track, tracker};
use std::hint::black_box;
use std::io::Write as IoWrite;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ============================================================================
// Tracker Benchmarks
// ============================================================================

fn benchmark_tracker_creation(c: &mut Criterion) {
    c.bench_function("tracker_creation", |b| {
        b.iter(|| {
            let t = tracker!();
            black_box(t);
        });
    });
}

fn benchmark_track_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("track_single");

    for size in [64, 256, 1024, 4096, 65536, 1048576].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("vec", size), size, |b, &size| {
            let t = tracker!();
            b.iter(|| {
                let data = vec![0u8; size];
                track!(t, data);
                black_box(data);
            });
        });
    }

    group.finish();
}

fn benchmark_track_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("track_multiple");

    for count in [10, 25, 50, 100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("variables", count), count, |b, &count| {
            let t = tracker!();
            b.iter(|| {
                for i in 0..count {
                    let data = vec![i as u8; 64];
                    track!(t, data);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_tracker_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracker_analyze");

    for alloc_count in [10, 50, 100, 500, 1000, 5000, 10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::new("analyze", alloc_count),
            alloc_count,
            |b, &alloc_count| {
                let t = tracker!();
                let mut keep_alive: Vec<Vec<u8>> = Vec::with_capacity(alloc_count);
                for i in 0..alloc_count {
                    let data = vec![i as u8; 64 + i % 256];
                    track!(t, data);
                    keep_alive.push(data);
                }

                b.iter(|| {
                    let report = t.analyze();
                    black_box(report);
                });

                std::mem::forget(keep_alive);
            },
        );
    }

    group.finish();
}

fn benchmark_tracker_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracker_stats");

    for alloc_count in [10, 50, 100, 500, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("stats", alloc_count),
            alloc_count,
            |b, &alloc_count| {
                let t = tracker!();
                let mut keep_alive: Vec<Vec<u8>> = Vec::with_capacity(alloc_count);
                for i in 0..alloc_count {
                    let data = vec![i as u8; 64 + i % 256];
                    track!(t, data);
                    keep_alive.push(data);
                }

                b.iter(|| {
                    let stats = t.stats();
                    black_box(stats);
                });

                std::mem::forget(keep_alive);
            },
        );
    }

    group.finish();
}

fn benchmark_tracker_clone(c: &mut Criterion) {
    c.bench_function("tracker_clone", |b| {
        let t = tracker!();
        b.iter(|| {
            let cloned = t.clone();
            black_box(cloned);
        });
    });
}

// ============================================================================
// Capture Backend Benchmarks
// ============================================================================

fn benchmark_backend_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_alloc");

    group.bench_function("alloc_core", |b| {
        let backend = CoreBackend;
        b.iter(|| {
            let event = backend.capture_alloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("alloc_lockfree", |b| {
        let backend = LockfreeBackend;
        b.iter(|| {
            let event = backend.capture_alloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("alloc_async", |b| {
        let backend = AsyncBackend;
        b.iter(|| {
            let event = backend.capture_alloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("alloc_unified", |b| {
        let backend = UnifiedCaptureBackend::new();
        b.iter(|| {
            let event = backend.capture_alloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.finish();
}

fn benchmark_backend_dealloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_dealloc");

    group.bench_function("dealloc_core", |b| {
        let backend = CoreBackend;
        b.iter(|| {
            let event = backend.capture_dealloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("dealloc_lockfree", |b| {
        let backend = LockfreeBackend;
        b.iter(|| {
            let event = backend.capture_dealloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("dealloc_async", |b| {
        let backend = AsyncBackend;
        b.iter(|| {
            let event = backend.capture_dealloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("dealloc_unified", |b| {
        let backend = UnifiedCaptureBackend::new();
        b.iter(|| {
            let event = backend.capture_dealloc(0x1000, 1024, 1);
            black_box(event);
        });
    });

    group.finish();
}

fn benchmark_backend_realloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_realloc");

    group.bench_function("realloc_core", |b| {
        let backend = CoreBackend;
        b.iter(|| {
            let event = backend.capture_realloc(0x1000, 1024, 2048, 1);
            black_box(event);
        });
    });

    group.bench_function("realloc_lockfree", |b| {
        let backend = LockfreeBackend;
        b.iter(|| {
            let event = backend.capture_realloc(0x1000, 1024, 2048, 1);
            black_box(event);
        });
    });

    group.bench_function("realloc_async", |b| {
        let backend = AsyncBackend;
        b.iter(|| {
            let event = backend.capture_realloc(0x1000, 1024, 2048, 1);
            black_box(event);
        });
    });

    group.bench_function("realloc_unified", |b| {
        let backend = UnifiedCaptureBackend::new();
        b.iter(|| {
            let event = backend.capture_realloc(0x1000, 1024, 2048, 1);
            black_box(event);
        });
    });

    group.finish();
}

fn benchmark_backend_move(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_move");

    group.bench_function("move_core", |b| {
        let backend = CoreBackend;
        b.iter(|| {
            let event = backend.capture_move(0x1000, 0x2000, 1024, 1);
            black_box(event);
        });
    });

    group.bench_function("move_lockfree", |b| {
        let backend = LockfreeBackend;
        b.iter(|| {
            let event = backend.capture_move(0x1000, 0x2000, 1024, 1);
            black_box(event);
        });
    });

    group.finish();
}

// ============================================================================
// Type Classification Benchmarks
// ============================================================================

fn benchmark_type_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_classification");

    let classifier = TypeClassifier::global();
    let type_names = vec![
        "i32",
        "String",
        "Vec<i32>",
        "HashMap<String, i32>",
        "Arc<Mutex<Vec<String>>>",
        "Box<dyn Send + Sync>",
        "Option<Result<String, Error>>",
        "std::collections::BTreeMap<u64, Vec<u8>>",
        "Vec<HashMap<String, BTreeMap<u64, Vec<Arc<String>>>>>",
        "Option<Box<Result<Arc<Vec<String>>, Error>>>>",
    ];

    for type_name in &type_names {
        group.bench_with_input(
            BenchmarkId::new("classify", type_name.replace(['<', '>', ':'], "_")),
            type_name,
            |b, &type_name| {
                b.iter(|| {
                    black_box(classifier.classify(black_box(type_name)));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_type_classification_cached(c: &mut Criterion) {
    let classifier = TypeClassifier::global();

    let common_types = vec!["i32", "String", "Vec<i32>", "HashMap<String, i32>"];

    for type_name in &common_types {
        classifier.classify(type_name);
    }

    c.bench_function("type_classification_cached", |b| {
        b.iter(|| {
            for type_name in &common_types {
                black_box(classifier.classify(black_box(type_name)));
            }
        });
    });
}

// ============================================================================
// Concurrent Benchmarks (30+ threads, 50 variables)
// ============================================================================

fn benchmark_concurrent_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_tracking");

    for thread_count in [1, 2, 4, 8, 16, 32, 48, 64, 96, 128].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let stats = Arc::new(TrackingStats::new());
                    let mut handles = vec![];

                    for _ in 0..thread_count {
                        let stats_clone = Arc::clone(&stats);
                        let handle = thread::spawn(move || {
                            for i in 0..1000 {
                                stats_clone.record_attempt();
                                let ptr = black_box((0x1000 + i * 8) as *const u8);
                                stats_clone.record_success();
                                black_box(ptr);
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    black_box(stats.get_completeness());
                });
            },
        );
    }

    group.finish();
}

fn benchmark_parallel_track(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_track");

    for thread_count in [1, 2, 4, 8, 16, 32, 48, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let mut handles = vec![];

                    for _ in 0..thread_count {
                        let handle = thread::spawn(move || {
                            let t = tracker!();
                            for i in 0..100 {
                                let data = vec![i as u8; 64];
                                track!(t, data);
                            }
                            t.analyze()
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        black_box(handle.join().unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_shared_tracker_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_tracker");

    for thread_count in [1, 2, 4, 8, 16, 32, 48, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("shared", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let t = Arc::new(tracker!());
                    let mut handles = vec![];

                    for thread_id in 0..thread_count {
                        let t_clone = Arc::clone(&t);
                        let handle = thread::spawn(move || {
                            for i in 0..100 {
                                let data = vec![(thread_id * 100 + i) as u8; 64];
                                track!(t_clone, data);
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    black_box(t.analyze());
                });
            },
        );
    }

    group.finish();
}

fn benchmark_50_variables_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("50_variables");

    for thread_count in [1, 8, 16, 32, 48, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads_50vars", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let t = Arc::new(tracker!());
                    let mut handles = vec![];

                    for thread_id in 0..thread_count {
                        let t_clone = Arc::clone(&t);
                        let handle = thread::spawn(move || {
                            for var_id in 0..50 {
                                let size = (var_id % 10 + 1) * 64;
                                let data = vec![(thread_id * 50 + var_id) as u8; size];
                                track!(t_clone, data);
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }

                    black_box(t.analyze());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// IO Operations Benchmarks
// ============================================================================

fn benchmark_io_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_operations");

    group.bench_function("file_write_with_tracking", |b| {
        let t = tracker!();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("memscope_bench_test.bin");

        b.iter(|| {
            let mut file = std::fs::File::create(&file_path).unwrap();
            for i in 0..50 {
                let data = vec![i as u8; 1024];
                track!(t, data);
                file.write_all(&data).unwrap();
            }
            file.flush().unwrap();
            let _ = std::fs::remove_file(&file_path);
        });
    });

    group.bench_function("channel_send_with_tracking", |b| {
        let t = tracker!();
        let (tx, rx) = channel();

        thread::spawn(move || while rx.recv().is_ok() {});

        b.iter(|| {
            for i in 0..50 {
                let data = vec![i as u8; 256];
                track!(t, data);
                tx.send(data).unwrap();
            }
        });

        drop(tx);
    });

    group.bench_function("mutex_lock_with_tracking", |b| {
        let t = tracker!();
        let shared_data = Arc::new(Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicU64::new(0));

        b.iter(|| {
            for i in 0..50 {
                let data = vec![i as u8; 128];
                track!(t, data);
                let mut guard = shared_data.lock().unwrap();
                guard.push(data);
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
    });

    group.bench_function("atomic_ops_with_tracking", |b| {
        let t = tracker!();
        let counter = Arc::new(AtomicU64::new(0));

        b.iter(|| {
            for i in 0..50 {
                let data = vec![i as u8; 64];
                track!(t, data);
                counter.fetch_add(1, Ordering::SeqCst);
                counter.load(Ordering::SeqCst);
                let _ = counter.compare_exchange(i, i + 1, Ordering::SeqCst, Ordering::Relaxed);
            }
        });
    });

    group.finish();
}

fn benchmark_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");

    group.bench_function("compute_io_track_mixed", |b| {
        let t = tracker!();
        let results = Arc::new(Mutex::new(Vec::new()));

        b.iter(|| {
            let mut local_results = Vec::new();

            for i in 0..50 {
                let data = vec![i as u8; 512];
                track!(t, data);

                let computed: u64 = data.iter().map(|&b| b as u64).sum();
                local_results.push(computed);

                if i % 10 == 0 {
                    let mut guard = results.lock().unwrap();
                    guard.extend(local_results.drain(..));
                }
            }

            if !local_results.is_empty() {
                let mut guard = results.lock().unwrap();
                guard.extend(local_results);
            }
        });
    });

    group.bench_function("thread_spawn_track_mixed", |b| {
        let t = Arc::new(tracker!());

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..8 {
                let t_clone = Arc::clone(&t);
                let handle = thread::spawn(move || {
                    for i in 0..6 {
                        let data = vec![(thread_id * 6 + i) as u8; 128];
                        track!(t_clone, data);

                        thread::sleep(Duration::from_micros(1));
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("producer_consumer_pattern", |b| {
        let t = Arc::new(tracker!());
        let (tx, rx) = channel::<Vec<u8>>();
        let done = Arc::new(AtomicU64::new(0));

        let consumer_t = Arc::clone(&t);
        let consumer_done = Arc::clone(&done);
        let consumer = thread::spawn(move || {
            while let Ok(data) = rx.recv() {
                track!(consumer_t, data);
                consumer_done.fetch_add(1, Ordering::SeqCst);
            }
        });

        b.iter(|| {
            for i in 0..50 {
                let data = vec![i as u8; 256];
                tx.send(data).unwrap();
            }
        });

        drop(tx);
        consumer.join().unwrap();
    });

    group.finish();
}

// ============================================================================
// Memory Pressure Benchmarks
// ============================================================================

fn benchmark_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");

    group.bench_function("high_allocation_rate", |b| {
        let stats = TrackingStats::new();
        b.iter(|| {
            for _ in 0..10000 {
                stats.record_attempt();
                if black_box(true) {
                    stats.record_success();
                } else {
                    stats.record_miss();
                }
            }
        });
    });

    group.bench_function("lock_contention", |b| {
        let stats = Arc::new(TrackingStats::new());
        b.iter(|| {
            let stats_clone = Arc::clone(&stats);
            let handle1 = thread::spawn(move || {
                for _ in 0..1000 {
                    stats_clone.record_attempt();
                    stats_clone.record_success();
                }
            });

            let stats_clone = Arc::clone(&stats);
            let handle2 = thread::spawn(move || {
                for _ in 0..1000 {
                    stats_clone.record_attempt();
                    stats_clone.record_miss();
                }
            });

            handle1.join().unwrap();
            handle2.join().unwrap();

            black_box(stats.get_detailed_stats().contention_rate);
        });
    });

    group.finish();
}

// ============================================================================
// Real-World Scenario Benchmarks
// ============================================================================

fn benchmark_realistic_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_scenarios");

    group.bench_function("web_server_pattern", |b| {
        let stats = TrackingStats::new();
        let classifier = TypeClassifier::global();

        b.iter(|| {
            for request_id in 0..100 {
                stats.record_attempt();
                classifier.classify("HashMap<String, String>");
                stats.record_success();

                stats.record_attempt();
                classifier.classify("Vec<u8>");
                stats.record_success();

                stats.record_attempt();
                classifier.classify("String");
                stats.record_success();

                if request_id % 20 == 0 {
                    stats.record_attempt();
                    stats.record_miss();
                }

                black_box(request_id);
            }
        });
    });

    group.bench_function("data_processing_pattern", |b| {
        let stats = TrackingStats::new();
        let classifier = TypeClassifier::global();

        b.iter(|| {
            for batch in 0..10 {
                stats.record_attempt();
                classifier.classify("Vec<Record>");
                stats.record_success();

                stats.record_attempt();
                classifier.classify("BTreeMap<u64, DataPoint>");
                stats.record_success();

                for _ in 0..50 {
                    stats.record_attempt();
                    classifier.classify("Arc<ProcessedData>");
                    stats.record_success();
                }

                black_box(batch);
            }
        });
    });

    group.bench_function("game_loop_pattern", |b| {
        let t = tracker!();

        b.iter(|| {
            for frame in 0..60 {
                let entities = vec![0u8; 1024];
                track!(t, entities);

                let physics = vec![0u8; 512];
                track!(t, physics);

                let audio = vec![0u8; 256];
                track!(t, audio);

                black_box(frame);
            }
        });
    });

    group.bench_function("api_handler_pattern", |b| {
        let t = tracker!();

        b.iter(|| {
            for _ in 0..50 {
                let request = vec![0u8; 256];
                track!(t, request);

                let response = vec![0u8; 1024];
                track!(t, response);

                let cache = vec![0u8; 4096];
                track!(t, cache);
            }
        });
    });

    group.finish();
}

// ============================================================================
// Allocation Pattern Benchmarks
// ============================================================================

fn benchmark_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    group.bench_function("many_small_allocations", |b| {
        let t = tracker!();
        b.iter(|| {
            for i in 0..1000 {
                let data = vec![i as u8; 16];
                track!(t, data);
            }
        });
    });

    group.bench_function("few_large_allocations", |b| {
        let t = tracker!();
        b.iter(|| {
            for i in 0..10 {
                let data = vec![i as u8; 1024 * 1024];
                track!(t, data);
            }
        });
    });

    group.bench_function("mixed_size_allocations", |b| {
        let t = tracker!();
        b.iter(|| {
            for i in 0..100 {
                let size = match i % 4 {
                    0 => 16,
                    1 => 256,
                    2 => 4096,
                    _ => 65536,
                };
                let data = vec![i as u8; size];
                track!(t, data);
            }
        });
    });

    group.bench_function("burst_allocations", |b| {
        let t = tracker!();
        b.iter(|| {
            for burst in 0..10 {
                for i in 0..100 {
                    let data = vec![(burst * 100 + i) as u8; 64];
                    track!(t, data);
                }
            }
        });
    });

    group.finish();
}

// ============================================================================
// Analysis Benchmarks
// ============================================================================

fn benchmark_analysis_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_operations");

    group.bench_function("allocation_info_creation", |b| {
        b.iter(|| {
            let info = AllocationInfo::new(0x1000, 1024);
            black_box(info);
        });
    });

    group.bench_function("many_allocation_infos", |b| {
        b.iter(|| {
            let infos: Vec<AllocationInfo> = (0..1000)
                .map(|i| AllocationInfo::new(0x1000 + i * 0x100, (i % 100 + 1) * 64))
                .collect();
            black_box(infos);
        });
    });

    group.finish();
}

// ============================================================================
// Tracking Stats Benchmarks
// ============================================================================

fn benchmark_tracking_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracking_stats");

    group.bench_function("stats_record_attempt", |b| {
        let stats = TrackingStats::new();
        b.iter(|| {
            stats.record_attempt();
        });
    });

    group.bench_function("stats_record_success", |b| {
        let stats = TrackingStats::new();
        b.iter(|| {
            stats.record_success();
        });
    });

    group.bench_function("stats_record_miss", |b| {
        let stats = TrackingStats::new();
        b.iter(|| {
            stats.record_miss();
        });
    });

    group.bench_function("stats_get_completeness", |b| {
        let stats = TrackingStats::new();
        for i in 0..1000 {
            stats.record_attempt();
            if i % 10 == 0 {
                stats.record_miss();
            } else {
                stats.record_success();
            }
        }
        b.iter(|| {
            black_box(stats.get_completeness());
        });
    });

    group.bench_function("stats_get_detailed_stats", |b| {
        let stats = TrackingStats::new();
        for i in 0..1000 {
            stats.record_attempt();
            if i % 10 == 0 {
                stats.record_miss();
            } else {
                stats.record_success();
            }
        }
        b.iter(|| {
            black_box(stats.get_detailed_stats());
        });
    });

    group.finish();
}

// ============================================================================
// High Concurrency Stress Tests
// ============================================================================

fn benchmark_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_tests");

    group.bench_function("stress_64_threads_50_vars", |b| {
        let t = Arc::new(tracker!());

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..64 {
                let t_clone = Arc::clone(&t);
                let handle = thread::spawn(move || {
                    for var_id in 0..50 {
                        let data = vec![(thread_id * 50 + var_id) as u8; 128];
                        track!(t_clone, data);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("stress_128_threads_25_vars", |b| {
        let t = Arc::new(tracker!());

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..128 {
                let t_clone = Arc::clone(&t);
                let handle = thread::spawn(move || {
                    for var_id in 0..25 {
                        let data = vec![(thread_id * 25 + var_id) as u8; 64];
                        track!(t_clone, data);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("stress_io_32_threads", |b| {
        let t = Arc::new(tracker!());
        let counter = Arc::new(AtomicU64::new(0));

        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..32 {
                let t_clone = Arc::clone(&t);
                let counter_clone = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for i in 0..50 {
                        let data = vec![(thread_id * 50 + i) as u8; 256];
                        track!(t_clone, data);
                        counter_clone.fetch_add(1, Ordering::SeqCst);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    tracker_benches,
    benchmark_tracker_creation,
    benchmark_track_single,
    benchmark_track_multiple,
    benchmark_tracker_analyze,
    benchmark_tracker_stats,
    benchmark_tracker_clone,
);
criterion_group!(
    backend_benches,
    benchmark_backend_alloc,
    benchmark_backend_dealloc,
    benchmark_backend_realloc,
    benchmark_backend_move,
);

criterion_group!(
    classification_benches,
    benchmark_type_classification,
    benchmark_type_classification_cached,
);

criterion_group!(
    concurrent_benches,
    benchmark_concurrent_tracking,
    benchmark_parallel_track,
    benchmark_shared_tracker_concurrent,
    benchmark_50_variables_concurrent,
);

criterion_group!(pressure_benches, benchmark_memory_pressure,);

criterion_group!(scenario_benches, benchmark_realistic_scenarios,);

criterion_group!(pattern_benches, benchmark_allocation_patterns,);

criterion_group!(analysis_benches, benchmark_analysis_operations,);

criterion_group!(stats_benches, benchmark_tracking_stats,);

criterion_group!(
    io_benches,
    benchmark_io_operations,
    benchmark_mixed_operations,
);

criterion_group!(stress_benches, benchmark_stress_tests,);

criterion_main!(
    tracker_benches,
    backend_benches,
    classification_benches,
    concurrent_benches,
    pressure_benches,
    scenario_benches,
    pattern_benches,
    analysis_benches,
    stats_benches,
    io_benches,
    stress_benches,
);
