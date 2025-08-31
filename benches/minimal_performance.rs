//! Minimal performance benchmarks for quick testing

use criterion::{criterion_group, criterion_main, Criterion};
use memscope_rs::*;
use std::hint::black_box;

/// Minimal allocation overhead test
fn benchmark_minimal_allocation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimal_overhead");
    group.sample_size(30);
    
    // Raw allocation benchmark
    group.bench_function("raw_allocation", |b| {
        b.iter(|| {
            let mut allocations = vec![];
            for i in 0..50 {
                let data = Box::new(i);
                allocations.push(data);
            }
            black_box(allocations);
        });
    });
    
    // Tracked allocation benchmark
    group.bench_function("tracked_allocation", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            tracker.enable_fast_mode();
            let mut allocations = vec![];
            
            for i in 0..50 {
                let data = Box::new(i);
                let ptr = &*data as *const i32 as usize;
                
                let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
                let _ = tracker.associate_var(ptr, format!("var_{i}"), "i32".to_string());
                
                allocations.push(data);
            }
            black_box(allocations);
        });
    });
    
    group.finish();
}

/// Minimal stats benchmark
fn benchmark_minimal_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimal_stats");
    group.sample_size(50);
    
    let tracker = MemoryTracker::new();
    tracker.enable_fast_mode();
    
    // Setup minimal data
    for i in 0..10 {
        let data = Box::new(i);
        let ptr = Box::into_raw(data) as usize;
        let _ = tracker.track_allocation(ptr, std::mem::size_of::<i32>());
        let _ = tracker.associate_var(ptr, format!("minimal_var_{i}"), "i32".to_string());
        unsafe { let _ = Box::from_raw(ptr as *mut i32); }
    }
    
    group.bench_function("get_stats", |b| {
        b.iter(|| {
            let stats = tracker.get_stats().unwrap();
            black_box(stats);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_minimal_allocation_overhead,
    benchmark_minimal_stats
);
criterion_main!(benches);