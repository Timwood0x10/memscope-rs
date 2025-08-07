use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memscope_rs::core::{
    AdaptiveHashMap, AtomicMemoryStats, OptimizedMutex, ShardedRwLock, SimpleMemoryStats,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn benchmark_std_mutex_vs_optimized(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutex_comparison");

    // Standard mutex benchmark
    group.bench_function("std_mutex", |b| {
        let data = Arc::new(Mutex::new(0i32));
        b.iter(|| {
            let data_clone = data.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let data = data_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..100 {
                            let mut guard = data.lock().unwrap();
                            *guard += 1;
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // Optimized mutex benchmark
    group.bench_function("optimized_mutex", |b| {
        let data = Arc::new(OptimizedMutex::new(0i32));
        b.iter(|| {
            let data_clone = data.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let data = data_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..100 {
                            let mut guard = data.lock();
                            *guard += 1;
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

fn benchmark_hashmap_vs_sharded(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_comparison");

    // Standard HashMap with Mutex
    group.bench_function("std_hashmap_mutex", |b| {
        let map = Arc::new(Mutex::new(HashMap::<i32, String>::new()));
        b.iter(|| {
            let map_clone = map.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let map = map_clone.clone();
                    thread::spawn(move || {
                        for j in 0..100 {
                            let key = i * 100 + j;
                            let value = format!("value_{}", key);
                            {
                                let mut guard = map.lock().unwrap();
                                guard.insert(key, value);
                            }
                            {
                                let guard = map.lock().unwrap();
                                black_box(guard.get(&key));
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // Sharded HashMap
    group.bench_function("sharded_hashmap", |b| {
        let map = Arc::new(ShardedRwLock::<i32, String>::new());
        b.iter(|| {
            let map_clone = map.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let map = map_clone.clone();
                    thread::spawn(move || {
                        for j in 0..100 {
                            let key = i * 100 + j;
                            let value = format!("value_{}", key);
                            map.insert(key, value);
                            black_box(map.get(&key));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // Adaptive HashMap
    group.bench_function("adaptive_hashmap", |b| {
        let map = Arc::new(AdaptiveHashMap::<i32, String>::new());
        b.iter(|| {
            let map_clone = map.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let map = map_clone.clone();
                    thread::spawn(move || {
                        for j in 0..100 {
                            let key = i * 100 + j;
                            let value = format!("value_{}", key);
                            map.insert(key, value);
                            black_box(map.get(&key));
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

fn benchmark_atomic_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats_comparison");

    // Mutex-protected stats
    group.bench_function("mutex_stats", |b| {
        #[derive(Default)]
        struct MutexStats {
            allocations: u64,
            deallocations: u64,
            active_memory: u64,
        }

        let stats = Arc::new(Mutex::new(MutexStats::default()));
        b.iter(|| {
            let stats_clone = stats.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let stats = stats_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..1000 {
                            {
                                let mut guard = stats.lock().unwrap();
                                guard.allocations += 1;
                                guard.active_memory += 64;
                            }
                            {
                                let mut guard = stats.lock().unwrap();
                                guard.deallocations += 1;
                                guard.active_memory -= 64;
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // Atomic stats
    group.bench_function("atomic_stats", |b| {
        let stats = Arc::new(SimpleMemoryStats::new());
        b.iter(|| {
            let stats_clone = stats.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let stats = stats_clone.clone();
                    thread::spawn(move || {
                        for _ in 0..1000 {
                            stats.record_allocation_fast(64);
                            stats.record_deallocation(64);
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

criterion_group!(
    benches,
    benchmark_std_mutex_vs_optimized,
    benchmark_hashmap_vs_sharded,
    benchmark_atomic_stats
);
criterion_main!(benches);
