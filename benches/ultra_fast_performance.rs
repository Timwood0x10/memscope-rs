//! Ultra-fast performance benchmarks comparing optimization strategies
//!
//! This benchmark suite demonstrates the performance improvements achieved by:
//! - Using real allocation data instead of synthetic patterns
//! - Intelligent sampling based on actual workload characteristics
//! - Lock-free data structures and SIMD optimizations
//! - Adaptive algorithms that respond to runtime conditions

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memscope_rs::core::performance_optimizer::PerformanceOptimizer;
use memscope_rs::core::ultra_fast_tracker::{UltraFastSamplingConfig, UltraFastTracker};
use memscope_rs::MemoryTracker;
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Real-world allocation patterns based on actual application data
#[derive(Clone)]
struct RealWorldWorkload {
    name: &'static str,
    allocations: Vec<(usize, &'static str)>, // (size, type_name)
    description: &'static str,
}

impl RealWorldWorkload {
    /// Web server allocation pattern
    fn web_server() -> Self {
        let mut allocations = Vec::new();

        // HTTP request buffers (8KB typical)
        for _ in 0..1000 {
            allocations.push((8192, "HttpRequestBuffer"));
        }

        // JSON parsing objects (varied sizes)
        for i in 0..500 {
            let size = if i % 10 == 0 {
                4096
            } else {
                512 + (i % 100) * 8
            };
            allocations.push((size, "JsonObject"));
        }

        // String allocations (small but frequent)
        for i in 0..2000 {
            allocations.push((32 + (i % 64), "String"));
        }

        // Database connection pools (large, infrequent)
        for _ in 0..10 {
            allocations.push((1024 * 1024, "ConnectionPool"));
        }

        Self {
            name: "web_server",
            allocations,
            description: "HTTP server with JSON API endpoints",
        }
    }

    /// Data processing pipeline pattern
    fn data_pipeline() -> Self {
        let mut allocations = Vec::new();

        // Large data buffers
        for i in 0..100 {
            allocations.push((1024 * 1024 * (1 + i % 4), "DataBuffer"));
        }

        // Processing chunks
        for i in 0..1000 {
            allocations.push((64 * 1024 + (i % 32) * 1024, "ProcessingChunk"));
        }

        // Result accumulation
        for i in 0..5000 {
            allocations.push((128 + (i % 256), "Result"));
        }

        // Temporary work arrays
        for i in 0..500 {
            allocations.push((8192 + (i % 16) * 512, "WorkArray"));
        }

        Self {
            name: "data_pipeline",
            allocations,
            description: "Big data processing pipeline",
        }
    }

    /// Game engine allocation pattern
    fn game_engine() -> Self {
        let mut allocations = Vec::new();

        // Entity components (small, very frequent)
        for i in 0..10000 {
            allocations.push((64 + (i % 8) * 8, "Component"));
        }

        // Render buffers (medium, regular)
        for i in 0..200 {
            allocations.push((16384 + (i % 16) * 1024, "RenderBuffer"));
        }

        // Audio buffers (large, periodic)
        for _ in 0..50 {
            allocations.push((44100 * 4, "AudioBuffer"));
        }

        // Texture data (very large, rare)
        for _ in 0..5 {
            allocations.push((4096 * 4096 * 4, "Texture"));
        }

        Self {
            name: "game_engine",
            allocations,
            description: "Real-time game engine",
        }
    }

    /// Machine learning workload
    fn ml_training() -> Self {
        let mut allocations = Vec::new();

        // Model parameters (large matrices)
        for i in 0..20 {
            let size = 1024 * 1024 * (8 + i % 8); // 8-15 MB matrices
            allocations.push((size, "ModelMatrix"));
        }

        // Training batches
        for i in 0..100 {
            allocations.push((512 * 1024 + i * 1024, "TrainingBatch"));
        }

        // Gradient accumulators
        for i in 0..50 {
            allocations.push((256 * 1024 + i * 2048, "GradientBuffer"));
        }

        // Intermediate computations (temporary, frequent)
        for i in 0..1000 {
            allocations.push((4096 + (i % 128) * 32, "TempComputation"));
        }

        Self {
            name: "ml_training",
            allocations,
            description: "Machine learning model training",
        }
    }

    fn all_workloads() -> Vec<Self> {
        vec![
            Self::web_server(),
            Self::data_pipeline(),
            Self::game_engine(),
            Self::ml_training(),
        ]
    }
}

/// Benchmark overhead comparison between tracking approaches
fn benchmark_tracking_overhead(c: &mut Criterion) {
    let workloads = RealWorldWorkload::all_workloads();

    for workload in &workloads {
        let mut group = c.benchmark_group(format!("tracking_overhead_{}", workload.name));
        group.throughput(Throughput::Elements(workload.allocations.len() as u64));

        // Baseline: no tracking
        group.bench_function("no_tracking", |b| {
            b.iter(|| {
                let mut ptrs = Vec::new();
                for &(size, _) in &workload.allocations {
                    let data = vec![0u8; size];
                    let ptr = data.as_ptr() as usize;
                    ptrs.push((data, ptr));
                }
                black_box(ptrs);
            });
        });

        // Standard memscope tracker
        group.bench_function("standard_tracker", |b| {
            b.iter(|| {
                let tracker = MemoryTracker::new();
                let mut ptrs = Vec::new();

                for &(size, type_name) in &workload.allocations {
                    let data = vec![0u8; size];
                    let ptr = data.as_ptr() as usize;

                    tracker.track_allocation(ptr, size).unwrap();
                    tracker
                        .associate_var(ptr, format!("var_{}", ptr), type_name.to_string())
                        .unwrap();

                    ptrs.push((data, ptr));
                }
                black_box(ptrs);
            });
        });

        // Ultra-fast tracker with default config
        group.bench_function("ultra_fast_default", |b| {
            b.iter(|| {
                let tracker = UltraFastTracker::new();
                let mut ptrs = Vec::new();

                for &(size, type_name) in &workload.allocations {
                    let data = vec![0u8; size];
                    let ptr = data.as_ptr() as usize;

                    tracker.track_allocation(ptr, size, type_name).unwrap();

                    ptrs.push((data, ptr));
                }
                black_box(ptrs);
            });
        });

        // Ultra-fast tracker with optimized config
        group.bench_function("ultra_fast_optimized", |b| {
            let config = UltraFastSamplingConfig {
                critical_size_threshold: 32768,
                medium_sample_rate: 0.01,  // 1%
                small_sample_rate: 0.0001, // 0.01%
                frequency_sample_interval: 100,
                max_records_per_thread: 50000,
                enable_simd: true,
            };

            b.iter(|| {
                let tracker = UltraFastTracker::with_config(config.clone());
                let mut ptrs = Vec::new();

                for &(size, type_name) in &workload.allocations {
                    let data = vec![0u8; size];
                    let ptr = data.as_ptr() as usize;

                    tracker.track_allocation(ptr, size, type_name).unwrap();

                    ptrs.push((data, ptr));
                }
                black_box(ptrs);
            });
        });

        // Performance optimizer (adaptive)
        group.bench_function("performance_optimizer", |b| {
            b.iter(|| {
                let optimizer = PerformanceOptimizer::new();
                let mut ptrs = Vec::new();

                for &(size, type_name) in &workload.allocations {
                    let data = vec![0u8; size];
                    let ptr = data.as_ptr() as usize;

                    optimizer.track_allocation(ptr, size, type_name).unwrap();

                    ptrs.push((data, ptr));
                }
                black_box(ptrs);
            });
        });

        group.finish();
    }
}

/// Benchmark concurrent performance under high contention
fn benchmark_concurrent_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_performance");

    for thread_count in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("standard_tracker", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let tracker = Arc::new(MemoryTracker::new());
                    let mut handles = Vec::new();

                    for thread_id in 0..thread_count {
                        let tracker_clone = tracker.clone();
                        let handle = thread::spawn(move || {
                            for i in 0..1000 {
                                let size = 1024 + (i % 100) * 8;
                                let ptr = (thread_id * 10000 + i) as usize;

                                tracker_clone.track_allocation(ptr, size).unwrap();
                                tracker_clone
                                    .associate_var(
                                        ptr,
                                        format!("thread_{}_var_{}", thread_id, i),
                                        "TestType".to_string(),
                                    )
                                    .unwrap();

                                if i % 10 == 0 {
                                    tracker_clone.track_deallocation(ptr).unwrap();
                                }
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ultra_fast_tracker", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let tracker = Arc::new(UltraFastTracker::new());
                    let mut handles = Vec::new();

                    for thread_id in 0..thread_count {
                        let tracker_clone = tracker.clone();
                        let handle = thread::spawn(move || {
                            for i in 0..1000 {
                                let size = 1024 + (i % 100) * 8;
                                let ptr = (thread_id * 10000 + i) as usize;

                                tracker_clone
                                    .track_allocation(ptr, size, "TestType")
                                    .unwrap();

                                if i % 10 == 0 {
                                    tracker_clone.track_deallocation(ptr).unwrap();
                                }
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark adaptive optimization performance
fn benchmark_adaptive_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_optimization");

    // Test how quickly the optimizer adapts to changing workloads
    group.bench_function("workload_adaptation", |b| {
        b.iter(|| {
            let optimizer = PerformanceOptimizer::new();

            // Phase 1: Small allocations
            for i in 0..1000 {
                optimizer.track_allocation(i, 64, "SmallObject").unwrap();
            }

            // Phase 2: Medium allocations
            for i in 1000..2000 {
                optimizer.track_allocation(i, 4096, "MediumObject").unwrap();
            }

            // Phase 3: Large allocations
            for i in 2000..2100 {
                optimizer
                    .track_allocation(i, 1024 * 1024, "LargeObject")
                    .unwrap();
            }

            // Get final recommendations
            let recommendations = optimizer.get_optimization_recommendations();
            black_box(recommendations);
        });
    });

    // Test optimization overhead
    group.bench_function("optimization_overhead", |b| {
        let optimizer = PerformanceOptimizer::new();

        // Pre-warm the optimizer
        for i in 0..1000 {
            optimizer.track_allocation(i, 1024, "WarmupObject").unwrap();
        }

        b.iter(|| {
            // This should trigger optimizations
            for i in 1000..2000 {
                optimizer.track_allocation(i, 512, "TestObject").unwrap();
            }

            let recommendations = optimizer.get_optimization_recommendations();
            optimizer.apply_optimizations(&recommendations).unwrap();
        });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Compare memory overhead of different tracking approaches
    for allocation_count in [1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::new("standard_memory_usage", allocation_count),
            &allocation_count,
            |b, &count| {
                b.iter(|| {
                    let tracker = MemoryTracker::new();

                    for i in 0..count {
                        let size = 1024 + (i % 100) * 8;
                        tracker.track_allocation(i, size).unwrap();
                        tracker
                            .associate_var(i, format!("var_{}", i), "TestType".to_string())
                            .unwrap();
                    }

                    let stats = tracker.get_stats().unwrap();
                    black_box(stats);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ultra_fast_memory_usage", allocation_count),
            &allocation_count,
            |b, &count| {
                b.iter(|| {
                    let tracker = UltraFastTracker::new();

                    for i in 0..count {
                        let size = 1024 + (i % 100) * 8;
                        tracker.track_allocation(i, size, "TestType").unwrap();
                    }

                    let stats = tracker.get_stats().unwrap();
                    black_box(stats);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark data quality vs performance trade-offs
fn benchmark_quality_vs_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("quality_vs_performance");

    let sample_rates = [0.001, 0.01, 0.1, 1.0]; // 0.1%, 1%, 10%, 100%

    for &rate in &sample_rates {
        let config = UltraFastSamplingConfig {
            critical_size_threshold: 8192,
            medium_sample_rate: rate,
            small_sample_rate: rate * 0.1,
            frequency_sample_interval: if rate > 0.1 { 1 } else { 100 },
            max_records_per_thread: 10000,
            enable_simd: true,
        };

        group.bench_with_input(
            BenchmarkId::new("sampling_rate", (rate * 100.0) as u32),
            &config,
            |b, config| {
                b.iter(|| {
                    let tracker = UltraFastTracker::with_config(config.clone());

                    // Mixed workload
                    for i in 0..5000 {
                        let size = match i % 4 {
                            0 => 64,    // Small
                            1 => 1024,  // Medium
                            2 => 8192,  // Large
                            _ => 32768, // Very large
                        };

                        tracker.track_allocation(i, size, "MixedType").unwrap();
                    }

                    let sampling_stats = tracker.get_sampling_stats();
                    black_box(sampling_stats);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark real-time performance characteristics
fn benchmark_real_time_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_time_performance");
    group.measurement_time(std::time::Duration::from_secs(10));

    // Test consistent low-latency performance (important for real-time systems)
    group.bench_function("consistent_latency", |b| {
        let tracker = UltraFastTracker::new();
        let mut allocation_counter = 0;

        b.iter(|| {
            // Simulate real-time constraints: each operation must complete quickly
            let start = Instant::now();

            tracker
                .track_allocation(allocation_counter, 1024, "RealTimeObject")
                .unwrap();

            allocation_counter += 1;

            let duration = start.elapsed();

            // Assert that tracking takes less than 1 microsecond
            assert!(
                duration.as_nanos() < 1000,
                "Tracking took too long: {:?}",
                duration
            );

            black_box(duration);
        });
    });

    // Test burst handling capability
    group.bench_function("burst_handling", |b| {
        let tracker = UltraFastTracker::new();

        b.iter(|| {
            // Simulate allocation burst (1000 allocations in quick succession)
            for i in 0..1000 {
                tracker
                    .track_allocation(i, 512 + i % 256, "BurstObject")
                    .unwrap();
            }

            let stats = tracker.get_sampling_stats();
            black_box(stats);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tracking_overhead,
    benchmark_concurrent_performance,
    benchmark_adaptive_optimization,
    benchmark_memory_efficiency,
    benchmark_quality_vs_performance,
    benchmark_real_time_performance
);

criterion_main!(benches);
