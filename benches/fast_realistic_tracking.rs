//! Fast realistic memory tracking benchmarks
//! Optimized for speed with minimal overhead

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memscope_rs::*;
use std::collections::HashMap;

/// Fast basic tracking benchmark
fn benchmark_fast_basic_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_basic_tracking");
    group.sample_size(50); // Reduce sample size for speed

    for size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("track_allocations", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let tracker = MemoryTracker::new();
                    tracker.enable_fast_mode(); // Enable fast mode

                    for i in 0..size {
                        let data = Box::new(i);
                        let ptr = Box::into_raw(data) as usize;

                        // Use fast tracking without error handling for speed
                        let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
                        let _ = tracker.associate_var(ptr, format!("var_{i}"), "i32".to_string());

                        // Clean up
                        unsafe {
                            let _ = Box::from_raw(ptr as *mut i32);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Fast HashMap tracking benchmark
fn benchmark_fast_hashmap_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_hashmap_tracking");
    group.sample_size(30); // Reduce sample size

    group.bench_function("hashmap_tracking", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            tracker.enable_fast_mode();
            let mut map: HashMap<String, i32> = HashMap::new();

            for i in 0..50 {
                // Reduce iterations
                let key = format!("key_{i}");
                map.insert(key, i);
            }

            // Track the hashmap itself
            let map_ptr = &map as *const HashMap<String, i32> as usize;
            let _ = tracker.track_allocation(map_ptr, std::mem::size_of::<HashMap<String, i32>>());
            let _ = tracker.associate_var(
                map_ptr,
                "test_map".to_string(),
                "HashMap<String, i32>".to_string(),
            );
        });
    });

    group.finish();
}

/// Fast export benchmark (no actual file I/O)
fn benchmark_fast_export_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_export_operations");
    group.sample_size(20); // Reduce sample size

    // Setup minimal test data
    let tracker = MemoryTracker::new();
    tracker.enable_fast_mode();

    for i in 0..20 {
        // Reduce test data
        let data = Box::new(i);
        let ptr = Box::into_raw(data) as usize;
        let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
        let _ = tracker.associate_var(ptr, format!("export_var_{i}"), "i32".to_string());
        unsafe {
            let _ = Box::from_raw(ptr as *mut i32);
        }
    }

    group.bench_function("stats_only", |b| {
        b.iter(|| {
            // Just test getting stats, not actual export
            let _ = tracker.get_stats();
        });
    });

    group.finish();
}

/// Fast memory analysis benchmark
fn benchmark_fast_memory_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_memory_analysis");
    group.sample_size(30);

    group.bench_function("lifecycle_analysis", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            tracker.enable_fast_mode();

            // Create minimal allocations
            let mut allocations = vec![];

            for i in 0..20 {
                // Reduce iterations
                let data = Box::new(i);
                let ptr = Box::into_raw(data) as usize;

                let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
                let _ = tracker.associate_var(ptr, format!("lifecycle_var_{i}"), "i32".to_string());

                allocations.push(ptr);

                // Deallocate some for lifecycle patterns
                if i % 5 == 0 && !allocations.is_empty() {
                    let old_ptr = allocations.remove(0);
                    let _ = tracker.track_deallocation(old_ptr);
                    unsafe {
                        let _ = Box::from_raw(old_ptr as *mut i32);
                    }
                }
            }

            // Clean up remaining allocations
            for ptr in allocations {
                let _ = tracker.track_deallocation(ptr);
                unsafe {
                    let _ = Box::from_raw(ptr as *mut i32);
                }
            }

            // Get analysis results
            let _ = tracker.get_stats();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_fast_basic_tracking,
    benchmark_fast_hashmap_tracking,
    benchmark_fast_export_operations,
    benchmark_fast_memory_analysis
);
criterion_main!(benches);
