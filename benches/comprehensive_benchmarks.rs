use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memscope_rs::classification::TypeClassifier;
use memscope_rs::tracking::stats::TrackingStats;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;

/// Benchmark allocation tracking performance
fn benchmark_allocation_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_tracking");

    // Test different allocation sizes
    for size in [64, 1024, 4096, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("track_allocation", size),
            size,
            |b, &size| {
                let stats = TrackingStats::new();
                b.iter(|| {
                    stats.record_attempt();
                    let ptr = black_box(0x1000 as *const u8);
                    let layout = std::alloc::Layout::from_size_align(size, 8).unwrap();
                    stats.record_success();
                    black_box((ptr, layout));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark deallocation tracking performance
fn benchmark_deallocation_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("deallocation_tracking");

    for num_ptrs in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*num_ptrs as u64));
        group.bench_with_input(
            BenchmarkId::new("track_deallocation", num_ptrs),
            num_ptrs,
            |b, &num_ptrs| {
                let ptrs: Vec<*const u8> = (0..num_ptrs)
                    .map(|i| (0x1000 + i * 8) as *const u8)
                    .collect();
                let stats = TrackingStats::new();

                b.iter(|| {
                    for &ptr in &ptrs {
                        stats.record_attempt();
                        black_box(ptr);
                        stats.record_success();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark type classification performance
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
        "custom::very::long::type::name::with::many::generics<T, U, V>",
    ];

    for type_name in &type_names {
        group.bench_with_input(
            BenchmarkId::new("classify_type", type_name),
            type_name,
            |b, &type_name| {
                b.iter(|| {
                    black_box(classifier.classify(black_box(type_name)));
                });
            },
        );
    }

    // Benchmark batch classification
    group.throughput(Throughput::Elements(type_names.len() as u64));
    group.bench_function("classify_batch", |b| {
        b.iter(|| {
            for type_name in &type_names {
                black_box(classifier.classify(black_box(type_name)));
            }
        });
    });

    group.finish();
}

/// Benchmark concurrent tracking performance
fn benchmark_concurrent_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_tracking");

    for thread_count in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_allocations", thread_count),
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

/// Benchmark memory pressure scenarios
fn benchmark_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");

    // Simulate high allocation rates
    group.bench_function("high_allocation_rate", |b| {
        let stats = TrackingStats::new();
        b.iter(|| {
            for _ in 0..10000 {
                stats.record_attempt();
                if black_box(true) {
                    // Simulate some allocations succeeding
                    stats.record_success();
                } else {
                    stats.record_miss();
                }
            }
        });
    });

    // Simulate contention scenarios
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

/// Benchmark export functionality performance
fn benchmark_export_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_performance");

    // Create mock allocation data
    let create_mock_data = |count: usize| -> Vec<(String, usize, u64)> {
        (0..count)
            .map(|i| (format!("Type{}", i), 1024 + (i % 4096), i as u64 * 1000))
            .collect()
    };

    for data_size in [100, 1000, 10000, 100000].iter() {
        group.throughput(Throughput::Elements(*data_size as u64));
        group.bench_with_input(
            BenchmarkId::new("process_allocations", data_size),
            data_size,
            |b, &data_size| {
                let data = create_mock_data(data_size);
                b.iter(|| {
                    let mut total_size = 0usize;
                    let mut total_count = 0usize;

                    for (type_name, size, _timestamp) in &data {
                        let category = TypeClassifier::global().classify(type_name);
                        total_size += size;
                        total_count += 1;
                        black_box((category, total_size, total_count));
                    }

                    black_box((total_size, total_count));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache performance
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    let classifier = TypeClassifier::global();
    let common_types = vec![
        "i32",
        "i64",
        "String",
        "Vec<i32>",
        "HashMap<String, i32>",
        "Arc<String>",
        "Box<i32>",
        "Option<String>",
        "Result<i32, String>",
    ];

    // Warm up cache
    for type_name in &common_types {
        classifier.classify(type_name);
    }

    group.bench_function("cached_classification", |b| {
        b.iter(|| {
            for type_name in &common_types {
                black_box(classifier.classify(black_box(type_name)));
            }
        });
    });

    // Test cache miss scenarios
    group.bench_function("cache_miss_classification", |b| {
        b.iter(|| {
            for i in 0..100 {
                let type_name = format!("UniqueType{}", i);
                black_box(classifier.classify(black_box(&type_name)));
            }
        });
    });

    group.finish();
}

/// Benchmark real-world scenarios
fn benchmark_realistic_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_scenarios");

    // Simulate a typical web server allocation pattern
    group.bench_function("web_server_pattern", |b| {
        let stats = TrackingStats::new();
        let classifier = TypeClassifier::global();

        b.iter(|| {
            // Simulate request processing
            for request_id in 0..100 {
                // Request data structures
                stats.record_attempt();
                classifier.classify("HashMap<String, String>"); // Headers
                stats.record_success();

                stats.record_attempt();
                classifier.classify("Vec<u8>"); // Body
                stats.record_success();

                stats.record_attempt();
                classifier.classify("String"); // Response
                stats.record_success();

                // Some allocations might fail under pressure
                if request_id % 20 == 0 {
                    stats.record_attempt();
                    stats.record_miss();
                }

                black_box(request_id);
            }
        });
    });

    // Simulate data processing workload
    group.bench_function("data_processing_pattern", |b| {
        let stats = TrackingStats::new();
        let classifier = TypeClassifier::global();

        b.iter(|| {
            // Large data structures
            for batch in 0..10 {
                stats.record_attempt();
                classifier.classify("Vec<Record>");
                stats.record_success();

                stats.record_attempt();
                classifier.classify("BTreeMap<u64, DataPoint>");
                stats.record_success();

                // Processing results
                for _ in 0..50 {
                    stats.record_attempt();
                    classifier.classify("Arc<ProcessedData>");
                    stats.record_success();
                }

                black_box(batch);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_allocation_tracking,
    benchmark_deallocation_tracking,
    benchmark_type_classification,
    benchmark_concurrent_tracking,
    benchmark_memory_pressure,
    benchmark_export_performance,
    benchmark_cache_performance,
    benchmark_realistic_scenarios
);
criterion_main!(benches);
