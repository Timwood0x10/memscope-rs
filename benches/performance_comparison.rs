//! Performance comparison benchmarks
//! 
//! Compares memscope-rs performance against standard Rust operations
//! to measure the overhead of memory tracking.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use memscope_rs::*;
use std::collections::HashMap;
use std::hint::black_box;

/// Compare allocation tracking overhead vs raw allocations
fn benchmark_allocation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_overhead");
    
    for size in [100, 1000, 10000].iter() {
        // Raw allocation benchmark
        group.bench_with_input(BenchmarkId::new("raw_allocation", size), size, |b, &size| {
            b.iter(|| {
                let mut allocations = vec![];
                for i in 0..size {
                    let data = Box::new(i);
                    allocations.push(data);
                }
                black_box(allocations);
            });
        });
        
        // Tracked allocation benchmark
        group.bench_with_input(BenchmarkId::new("tracked_allocation", size), size, |b, &size| {
            b.iter(|| {
                let tracker = MemoryTracker::new();
                let mut allocations = vec![];
                
                for i in 0..size {
                    let data = Box::new(i);
                    let ptr = &*data as *const i32 as usize;
                    
                    tracker.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
                    tracker.associate_var(ptr, format!("var_{i}"), "i32".to_string()).unwrap();
                    
                    allocations.push(data);
                }
                black_box(allocations);
            });
        });
    }
    
    group.finish();
}

/// Compare HashMap operations with and without tracking
fn benchmark_hashmap_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_overhead");
    
    // Standard HashMap operations
    group.bench_function("standard_hashmap", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            
            for i in 0..1000 {
                map.insert(format!("key_{i}"), i);
            }
            
            for i in 0..1000 {
                let _ = map.get(&format!("key_{i}"));
            }
            
            black_box(map);
        });
    });
    
    // Tracked HashMap operations
    group.bench_function("tracked_hashmap", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            let mut map: HashMap<String, i32> = HashMap::new();
            
            // Track the HashMap
            let map_ptr = &map as *const HashMap<String, i32> as usize;
            tracker.track_allocation(map_ptr, std::mem::size_of::<HashMap<String, i32>>()).unwrap();
            tracker.associate_var(map_ptr, "tracked_map".to_string(), "HashMap<String, i32>".to_string()).unwrap();
            
            for i in 0..1000 {
                map.insert(format!("key_{i}"), i);
                
                // Track each insertion (simplified)
                if i % 100 == 0 {
                    let _ = tracker.get_stats();
                }
            }
            
            for i in 0..1000 {
                let _ = map.get(&format!("key_{i}"));
            }
            
            black_box(map);
        });
    });
    
    group.finish();
}

/// Benchmark export performance with different data sizes
fn benchmark_export_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_performance");
    
    for size in [100, 500, 1000].iter() {
        // Setup tracker with test data
        let tracker = MemoryTracker::new();
        for i in 0..*size {
            let data = Box::new(i);
            let ptr = Box::into_raw(data) as usize;
            tracker.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
            tracker.associate_var(ptr, format!("perf_var_{i}"), "i32".to_string()).unwrap();
            
            // Simulate some deallocations
            if i % 3 == 0 {
                tracker.track_deallocation(ptr).unwrap();
            }
            
            unsafe { let _ = Box::from_raw(ptr as *mut i32); }
        }
        
        group.bench_with_input(BenchmarkId::new("json_export", size), size, |b, _| {
            b.iter(|| {
                let temp_dir = tempfile::tempdir().unwrap();
                let json_path = temp_dir.path().join("perf_test.json");
                tracker.export_to_json(&json_path).unwrap();
            });
        });
        
        group.bench_with_input(BenchmarkId::new("html_export", size), size, |b, _| {
            b.iter(|| {
                let temp_dir = tempfile::tempdir().unwrap();
                let html_path = temp_dir.path().join("perf_test.html");
                tracker.export_interactive_dashboard(&html_path).unwrap();
            });
        });
    }
    
    group.finish();
}

/// Benchmark memory analysis operations
fn benchmark_analysis_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_performance");
    
    // Setup complex tracking scenario
    let tracker = MemoryTracker::new();
    let mut allocations = vec![];
    
    for i in 0..1000 {
        let data = Box::new(vec![i; 10]);
        let ptr = Box::into_raw(data) as usize;
        tracker.track_allocation(ptr, std::mem::size_of::<Vec<i32>>() + 10 * std::mem::size_of::<i32>()).unwrap();
        tracker.associate_var(ptr, format!("analysis_var_{i}"), "Vec<i32>".to_string()).unwrap();
        allocations.push(ptr);
        
        // Create some deallocations for analysis
        if i % 4 == 0 && i > 0 {
            let old_ptr = allocations.remove(0);
            tracker.track_deallocation(old_ptr).unwrap();
            unsafe { let _ = Box::from_raw(old_ptr as *mut Vec<i32>); }
        }
    }
    
    group.bench_function("get_stats", |b| {
        b.iter(|| {
            let stats = tracker.get_stats().unwrap();
            black_box(stats);
        });
    });
    
    group.bench_function("memory_analysis", |b| {
        b.iter(|| {
            let stats = tracker.get_stats().unwrap();
            
            // Simulate analysis operations
            let total_memory = stats.active_memory;
            let allocation_count = stats.active_allocations;
            let efficiency = if allocation_count > 0 {
                total_memory as f64 / allocation_count as f64
            } else {
                0.0
            };
            
            black_box((total_memory, allocation_count, efficiency));
        });
    });
    
    // Clean up remaining allocations
    for ptr in allocations {
        tracker.track_deallocation(ptr).unwrap();
        unsafe { let _ = Box::from_raw(ptr as *mut Vec<i32>); }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_allocation_overhead,
    benchmark_hashmap_overhead,
    benchmark_export_performance,
    benchmark_analysis_performance
);
criterion_main!(benches);