use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memscope_rs::core::targeted_optimizations::{
    efficient_string_concat, BatchProcessor, FastStatsCollector,
};
// use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn benchmark_stats_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats_collection");

    // Old way: mutex-protected stats
    group.bench_function("mutex_stats", |b| {
        #[derive(Default)]
        struct OldStats {
            allocation_count: u64,
            total_allocated: u64,
        }

        let stats = Arc::new(Mutex::new(OldStats::default()));
        b.iter(|| {
            let stats_clone = stats.clone();
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let stats = stats_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..1000 {
                            let mut guard = stats.lock().unwrap();
                            guard.allocation_count += 1;
                            guard.total_allocated += 64;
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // New way: atomic stats
    group.bench_function("atomic_stats", |b| {
        let stats = Arc::new(FastStatsCollector::new());
        b.iter(|| {
            let stats_clone = stats.clone();
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let stats = stats_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..1000 {
                            stats.record_allocation_fast(64);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Old way: multiple allocations
    group.bench_function("string_concat_old", |b| {
        let parts = vec!["Hello", " ", "World", " ", "from", " ", "Rust", "!"];
        b.iter(|| {
            let mut result = String::new();
            for part in &parts {
                result.push_str(part);
            }
            black_box(result);
        });
    });

    // New way: pre-allocated
    group.bench_function("string_concat_optimized", |b| {
        let parts = vec!["Hello", " ", "World", " ", "from", " ", "Rust", "!"];
        b.iter(|| {
            let result = efficient_string_concat(&parts);
            black_box(result);
        });
    });

    group.finish();
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    // Old way: process each item individually
    group.bench_function("individual_processing", |b| {
        let processed = Arc::new(Mutex::new(Vec::new()));
        b.iter(|| {
            let processed_clone = processed.clone();
            let handles: Vec<_> = (0..4)
                .map(|i| {
                    let processed = processed_clone.clone();
                    thread::spawn(move || {
                        for j in 0..100 {
                            let item = i * 100 + j;
                            // Simulate processing each item individually
                            {
                                let mut guard = processed.lock().unwrap();
                                guard.push(item);
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            // Clear for next iteration
            processed.lock().unwrap().clear();
        });
    });

    // New way: batch processing
    group.bench_function("batch_processing", |b| {
        let processed = Arc::new(Mutex::new(Vec::new()));
        b.iter(|| {
            let processed_clone = processed.clone();
            let processor = Arc::new(BatchProcessor::new(10, move |batch: &[i32]| {
                let mut guard = processed_clone.lock().unwrap();
                guard.extend_from_slice(batch);
            }));

            let handles: Vec<_> = (0..4)
                .map(|i| {
                    let processor = processor.clone();
                    thread::spawn(move || {
                        for j in 0..100 {
                            let item = i * 100 + j;
                            processor.add(item);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            processor.flush();

            // Clear for next iteration
            processed.lock().unwrap().clear();
        });
    });

    group.finish();
}

fn benchmark_unwrap_alternatives(c: &mut Criterion) {
    let mut group = c.benchmark_group("unwrap_alternatives");

    // Old way: unwrap with potential panic
    group.bench_function("unwrap_with_panic_check", |b| {
        let values: Vec<Option<i32>> = (0..1000)
            .map(|i| if i % 10 == 0 { None } else { Some(i) })
            .collect();
        b.iter(|| {
            let mut sum = 0;
            for value in &values {
                sum += value.unwrap_or(0);
            }
            black_box(sum);
        });
    });

    // New way: fast default
    group.bench_function("fast_unwrap_or_default", |b| {
        let values: Vec<Option<i32>> = (0..1000)
            .map(|i| if i % 10 == 0 { None } else { Some(i) })
            .collect();
        b.iter(|| {
            let mut sum = 0;
            for value in &values {
                sum += memscope_rs::core::targeted_optimizations::fast_unwrap_or_default(*value);
            }
            black_box(sum);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_stats_collection,
    benchmark_string_operations,
    benchmark_batch_processing,
    benchmark_unwrap_alternatives
);
criterion_main!(benches);
