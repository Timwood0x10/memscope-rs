//! Performance benchmark demonstration
//!
//! This example demonstrates how to benchmark memory tracking performance
//! and compare different allocation patterns and export operations.

use memscope_rs::*;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

/// Demonstrates various performance benchmarks for memory tracking
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting performance benchmark demonstration");

    benchmark_basic_operations()?;
    benchmark_allocation_patterns()?;
    benchmark_export_performance()?;
    benchmark_concurrent_operations()?;

    tracing::info!("Performance benchmark demonstration completed");
    Ok(())
}

/// Benchmark basic memory tracking operations
fn benchmark_basic_operations() -> TrackingResult<()> {
    tracing::info!("Benchmarking basic memory tracking operations");

    let tracker = MemoryTracker::new();
    let operation_count = 10_000;

    // Benchmark allocation tracking
    let start = Instant::now();
    for i in 0..operation_count {
        let ptr = 0x1000 + (i * 0x100);
        tracker.track_allocation(ptr, 64)?;
    }
    let alloc_duration = start.elapsed();

    // Benchmark variable association
    let start = Instant::now();
    for i in 0..operation_count {
        let ptr = 0x1000 + (i * 0x100);
        tracker.associate_var(ptr, format!("var_{i}"), "i32".to_string())?;
    }
    let assoc_duration = start.elapsed();

    // Benchmark deallocation tracking
    let start = Instant::now();
    for i in 0..operation_count {
        let ptr = 0x1000 + (i * 0x100);
        tracker.track_deallocation(ptr)?;
    }
    let dealloc_duration = start.elapsed();

    // Benchmark statistics retrieval
    let start = Instant::now();
    for _ in 0..1000 {
        let _stats = tracker.get_stats()?;
    }
    let stats_duration = start.elapsed();

    tracing::info!("Basic operations benchmark results:");
    tracing::info!(
        "  Allocation tracking: {:.2} ops/ms",
        operation_count as f64 / alloc_duration.as_millis() as f64
    );
    tracing::info!(
        "  Variable association: {:.2} ops/ms",
        operation_count as f64 / assoc_duration.as_millis() as f64
    );
    tracing::info!(
        "  Deallocation tracking: {:.2} ops/ms",
        operation_count as f64 / dealloc_duration.as_millis() as f64
    );
    tracing::info!(
        "  Statistics retrieval: {:.2} ops/ms",
        1000.0 / stats_duration.as_millis() as f64
    );

    Ok(())
}

/// Benchmark different allocation patterns
fn benchmark_allocation_patterns() -> TrackingResult<()> {
    tracing::info!("Benchmarking different allocation patterns");

    // Pattern 1: Sequential allocations
    let tracker1 = MemoryTracker::new();
    let start = Instant::now();
    for i in 0..5000 {
        let ptr = 0x1000 + (i * 0x100);
        tracker1.track_allocation(ptr, 64)?;
        tracker1.associate_var(ptr, format!("seq_var_{i}"), "i32".to_string())?;
    }
    let sequential_duration = start.elapsed();

    // Pattern 2: Random-sized allocations
    let tracker2 = MemoryTracker::new();
    let start = Instant::now();
    for i in 0..5000 {
        let ptr = 0x1000 + (i * 0x100);
        let size = 32 + (i % 10) * 16; // Varying sizes
        tracker2.track_allocation(ptr, size)?;
        tracker2.associate_var(ptr, format!("rand_var_{i}"), "Vec<u8>".to_string())?;
    }
    let random_size_duration = start.elapsed();

    // Pattern 3: Mixed allocation/deallocation
    let tracker3 = MemoryTracker::new();
    let start = Instant::now();
    for i in 0..5000 {
        let ptr = 0x1000 + (i * 0x100);
        tracker3.track_allocation(ptr, 64)?;
        tracker3.associate_var(ptr, format!("mixed_var_{i}"), "String".to_string())?;

        // Deallocate every third allocation
        if i % 3 == 0 && i > 0 {
            let dealloc_ptr = 0x1000 + ((i - 1) * 0x100);
            tracker3.track_deallocation(dealloc_ptr)?;
        }
    }
    let mixed_duration = start.elapsed();

    tracing::info!("Allocation pattern benchmark results:");
    tracing::info!(
        "  Sequential pattern: {:.2}ms",
        sequential_duration.as_millis()
    );
    tracing::info!(
        "  Random-size pattern: {:.2}ms",
        random_size_duration.as_millis()
    );
    tracing::info!(
        "  Mixed alloc/dealloc pattern: {:.2}ms",
        mixed_duration.as_millis()
    );

    // Compare final statistics
    let stats1 = tracker1.get_stats()?;
    let stats2 = tracker2.get_stats()?;
    let stats3 = tracker3.get_stats()?;

    tracing::info!("Final statistics comparison:");
    tracing::info!(
        "  Sequential: {} active, {}KB memory",
        stats1.active_allocations,
        stats1.active_memory / 1024
    );
    tracing::info!(
        "  Random-size: {} active, {}KB memory",
        stats2.active_allocations,
        stats2.active_memory / 1024
    );
    tracing::info!(
        "  Mixed: {} active, {}KB memory",
        stats3.active_allocations,
        stats3.active_memory / 1024
    );

    Ok(())
}

/// Benchmark export performance
fn benchmark_export_performance() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Benchmarking export performance");

    let temp_dir = TempDir::new()?;

    // Create test data of different sizes
    let test_sizes = vec![100, 1000, 5000];

    for &size in &test_sizes {
        tracing::info!("Testing export performance with {} allocations", size);

        // Create tracker with test data
        let tracker = MemoryTracker::new();
        for i in 0..size {
            let ptr = 0x1000 + (i * 0x100);
            let alloc_size = 64 + (i % 10) * 32;
            tracker.track_allocation(ptr, alloc_size)?;
            tracker.associate_var(
                ptr,
                format!("export_var_{i}"),
                match i % 4 {
                    0 => "i32".to_string(),
                    1 => "String".to_string(),
                    2 => "Vec<u8>".to_string(),
                    _ => "HashMap<String, i32>".to_string(),
                },
            )?;
        }

        // Benchmark binary export
        let binary_path = temp_dir.path().join(format!("test_{size}.bin"));
        let start = Instant::now();
        tracker.export_to_binary(&binary_path)?;
        let binary_export_duration = start.elapsed();

        // Benchmark JSON export
        let start = Instant::now();
        memscope_rs::export::binary::export_binary_to_json(&binary_path, &format!("test_{size}"))?;
        let json_export_duration = start.elapsed();

        // Benchmark HTML export
        let start = Instant::now();
        memscope_rs::export::binary::export_binary_to_html(&binary_path, &format!("test_{size}"))?;
        let html_export_duration = start.elapsed();

        // Get file sizes
        let binary_size = std::fs::metadata(&binary_path)?.len();
        let json_path = binary_path.with_extension("json");
        let html_path = binary_path.with_extension("html");
        let json_size = std::fs::metadata(&json_path)?.len();
        let html_size = std::fs::metadata(&html_path)?.len();

        tracing::info!("Export performance for {} allocations:", size);
        tracing::info!(
            "  Binary export: {:.2}ms ({}KB)",
            binary_export_duration.as_millis(),
            binary_size / 1024
        );
        tracing::info!(
            "  JSON export: {:.2}ms ({}KB)",
            json_export_duration.as_millis(),
            json_size / 1024
        );
        tracing::info!(
            "  HTML export: {:.2}ms ({}KB)",
            html_export_duration.as_millis(),
            html_size / 1024
        );
        tracing::info!(
            "  Throughput: {:.2} allocs/ms (JSON)",
            size as f64 / json_export_duration.as_millis() as f64
        );
    }

    Ok(())
}

/// Benchmark concurrent operations
fn benchmark_concurrent_operations() -> TrackingResult<()> {
    tracing::info!("Benchmarking concurrent operations");

    let tracker = Arc::new(MemoryTracker::new());
    let thread_count = 4;
    let operations_per_thread = 1000;

    // Benchmark concurrent allocations
    let start = Instant::now();
    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            let tracker_clone = Arc::clone(&tracker);
            std::thread::spawn(move || -> TrackingResult<()> {
                for i in 0..operations_per_thread {
                    let ptr = 0x10000 + (thread_id * 0x10000) + (i * 0x100);
                    tracker_clone.track_allocation(ptr, 64)?;
                    tracker_clone.associate_var(
                        ptr,
                        format!("thread_{thread_id}_var_{i}"),
                        "i32".to_string(),
                    )?;
                }
                Ok(())
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap()?;
    }
    let concurrent_duration = start.elapsed();

    // Benchmark sequential operations for comparison
    let tracker_seq = MemoryTracker::new();
    let start = Instant::now();
    for thread_id in 0..thread_count {
        for i in 0..operations_per_thread {
            let ptr = 0x10000 + (thread_id * 0x10000) + (i * 0x100);
            tracker_seq.track_allocation(ptr, 64)?;
            tracker_seq.associate_var(
                ptr,
                format!("seq_{thread_id}_var_{i}"),
                "i32".to_string(),
            )?;
        }
    }
    let sequential_duration = start.elapsed();

    let total_operations = thread_count * operations_per_thread * 2; // alloc + assoc

    tracing::info!("Concurrent operations benchmark results:");
    tracing::info!(
        "  Concurrent ({} threads): {:.2}ms ({:.2} ops/ms)",
        thread_count,
        concurrent_duration.as_millis(),
        total_operations as f64 / concurrent_duration.as_millis() as f64
    );
    tracing::info!(
        "  Sequential: {:.2}ms ({:.2} ops/ms)",
        sequential_duration.as_millis(),
        total_operations as f64 / sequential_duration.as_millis() as f64
    );
    tracing::info!(
        "  Speedup: {:.2}x",
        sequential_duration.as_millis() as f64 / concurrent_duration.as_millis() as f64
    );

    // Verify final statistics
    let concurrent_stats = tracker.get_stats()?;
    let sequential_stats = tracker_seq.get_stats()?;

    tracing::info!("Final statistics verification:");
    tracing::info!(
        "  Concurrent: {} total, {} active",
        concurrent_stats.total_allocations,
        concurrent_stats.active_allocations
    );
    tracing::info!(
        "  Sequential: {} total, {} active",
        sequential_stats.total_allocations,
        sequential_stats.active_allocations
    );

    Ok(())
}

/// Memory efficiency analysis
#[allow(dead_code)]
fn analyze_memory_efficiency() -> TrackingResult<()> {
    tracing::info!("Analyzing memory efficiency");

    let tracker = MemoryTracker::new();

    // Create allocations of different sizes to analyze efficiency
    let allocation_sizes = vec![8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    let allocations_per_size = 100;

    for &size in &allocation_sizes {
        let start_stats = tracker.get_stats()?;

        // Allocate multiple blocks of the same size
        for i in 0..allocations_per_size {
            let ptr = 0x100000 + (size * 1000) + (i * 0x100);
            tracker.track_allocation(ptr, size)?;
            tracker.associate_var(ptr, format!("size_{size}_var_{i}"), "u8".to_string())?;
        }

        let end_stats = tracker.get_stats()?;
        let memory_increase = end_stats.active_memory - start_stats.active_memory;
        let expected_memory = size * allocations_per_size;
        let efficiency = (expected_memory as f64 / memory_increase as f64) * 100.0;

        tracing::info!(
            "Size {}B: efficiency {:.1}% (expected {}B, actual {}B)",
            size,
            efficiency,
            expected_memory,
            memory_increase
        );
    }

    Ok(())
}
