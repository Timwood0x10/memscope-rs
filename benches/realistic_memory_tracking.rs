//! Realistic memory tracking benchmarks
//! 
//! This benchmark tests the core functionality of memscope-rs with realistic scenarios
//! based on the actual API and usage patterns from examples.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use memscope_rs::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Benchmark basic memory tracking operations
fn benchmark_basic_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_tracking");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("track_allocations", size), size, |b, &size| {
            b.iter(|| {
                let tracker = MemoryTracker::new();
                
                for i in 0..size {
                    let data = Box::new(i);
                    let ptr = Box::into_raw(data) as usize;
                    
                    tracker.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
                    tracker.associate_var(ptr, format!("var_{i}"), "i32".to_string()).unwrap();
                    
                    // Clean up
                    unsafe { let _ = Box::from_raw(ptr as *mut i32); }
                }
            });
        });
    }
    
    group.finish();
}

/// Benchmark complex data structure tracking
fn benchmark_complex_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_tracking");
    
    group.bench_function("hashmap_tracking", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            let mut map: HashMap<String, Vec<i32>> = HashMap::new();
            
            for i in 0..100 {
                let key = format!("key_{i}");
                let value = vec![i; 10];
                
                // Track the vector allocation
                let vec_ptr = value.as_ptr() as usize;
                tracker.track_allocation(vec_ptr, value.len() * std::mem::size_of::<i32>()).unwrap();
                tracker.associate_var(vec_ptr, format!("vec_{i}"), "Vec<i32>".to_string()).unwrap();
                
                map.insert(key, value);
            }
            
            // Track the hashmap itself
            let map_ptr = &map as *const HashMap<String, Vec<i32>> as usize;
            tracker.track_allocation(map_ptr, std::mem::size_of::<HashMap<String, Vec<i32>>>()).unwrap();
            tracker.associate_var(map_ptr, "test_map".to_string(), "HashMap<String, Vec<i32>>".to_string()).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark concurrent tracking operations
fn benchmark_concurrent_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_tracking");
    
    group.bench_function("multi_thread_tracking", |b| {
        b.iter(|| {
            let tracker = Arc::new(MemoryTracker::new());
            let mut handles = vec![];
            
            for thread_id in 0..4 {
                let tracker_clone = tracker.clone();
                let handle = thread::spawn(move || {
                    for i in 0..25 {
                        let data = Box::new(i + thread_id * 1000);
                        let ptr = Box::into_raw(data) as usize;
                        
                        tracker_clone.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
                        tracker_clone.associate_var(
                            ptr, 
                            format!("thread_{thread_id}_var_{i}"), 
                            "i32".to_string()
                        ).unwrap();
                        
                        // Simulate some work
                        thread::sleep(Duration::from_nanos(100));
                        
                        // Clean up
                        unsafe { let _ = Box::from_raw(ptr as *mut i32); }
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

/// Benchmark export functionality
fn benchmark_export_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_operations");
    
    // Setup test data
    let tracker = MemoryTracker::new();
    for i in 0..100 {
        let data = Box::new(i);
        let ptr = Box::into_raw(data) as usize;
        tracker.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
        tracker.associate_var(ptr, format!("export_var_{i}"), "i32".to_string()).unwrap();
        unsafe { let _ = Box::from_raw(ptr as *mut i32); }
    }
    
    group.bench_function("json_export", |b| {
        b.iter(|| {
            let temp_dir = tempfile::tempdir().unwrap();
            let json_path = temp_dir.path().join("test_export.json");
            tracker.export_to_json(&json_path).unwrap();
        });
    });
    
    group.bench_function("html_export", |b| {
        b.iter(|| {
            let temp_dir = tempfile::tempdir().unwrap();
            let html_path = temp_dir.path().join("test_export.html");
            tracker.export_interactive_dashboard(&html_path).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark memory analysis operations
fn benchmark_memory_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_analysis");
    
    group.bench_function("lifecycle_analysis", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            
            // Create and track various allocations with different lifecycles
            let mut allocations = vec![];
            
            for i in 0..50 {
                let data = Box::new(i);
                let ptr = Box::into_raw(data) as usize;
                
                tracker.track_allocation(ptr, std::mem::size_of::<i32>()).unwrap();
                tracker.associate_var(ptr, format!("lifecycle_var_{i}"), "i32".to_string()).unwrap();
                
                allocations.push(ptr);
                
                // Deallocate some allocations to create lifecycle patterns
                if i % 3 == 0 && !allocations.is_empty() {
                    let old_ptr = allocations.remove(0);
                    tracker.track_deallocation(old_ptr).unwrap();
                    unsafe { let _ = Box::from_raw(old_ptr as *mut i32); }
                }
            }
            
            // Clean up remaining allocations
            for ptr in allocations {
                tracker.track_deallocation(ptr).unwrap();
                unsafe { let _ = Box::from_raw(ptr as *mut i32); }
            }
            
            // Get analysis results
            let stats = tracker.get_stats().unwrap();
            let _ = stats.total_allocations;
        });
    });
    
    group.finish();
}

/// Benchmark smart pointer tracking
fn benchmark_smart_pointer_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_pointer_tracking");
    
    group.bench_function("arc_tracking", |b| {
        b.iter(|| {
            let tracker = MemoryTracker::new();
            let mut arcs = vec![];
            
            for i in 0..50 {
                let data = Arc::new(vec![i; 10]);
                let ptr = Arc::as_ptr(&data) as usize;
                
                tracker.track_allocation(ptr, std::mem::size_of::<Vec<i32>>() + 10 * std::mem::size_of::<i32>()).unwrap();
                tracker.associate_var(ptr, format!("arc_vec_{i}"), "Arc<Vec<i32>>".to_string()).unwrap();
                
                arcs.push(data);
            }
            
            // Clone some Arcs to test reference counting scenarios
            let mut cloned_arcs = vec![];
            for (i, arc) in arcs.iter().enumerate() {
                if i % 2 == 0 {
                    cloned_arcs.push(arc.clone());
                }
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_basic_tracking,
    benchmark_complex_tracking,
    benchmark_concurrent_tracking,
    benchmark_export_operations,
    benchmark_memory_analysis,
    benchmark_smart_pointer_tracking
);
criterion_main!(benches);