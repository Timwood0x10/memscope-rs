//! Real-world memory tracking demonstration for memscope-rs.
//!
//! This example demonstrates practical memory tracking scenarios:
//! - Memory leak detection in long-running services
//! - Tracking allocations across function boundaries
//! - Analyzing memory hotspots and patterns
//! - Exporting comprehensive memory reports

use memscope_rs::{
    analyzer, global_tracker, init_global_tracking, init_logging, track, MemScopeResult,
};
use std::collections::HashMap;

/// Simulates a web server cache that may leak memory.
struct Cache {
    /// Internal storage for cached items.
    data: HashMap<String, Vec<u8>>,
    /// Maximum allowed cache size in bytes.
    max_size: usize,
}

impl Cache {
    /// Creates a new cache with the specified maximum size.
    fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// Returns `true` if the insertion was successful,
    /// or `false` if it would exceed the maximum size.
    fn insert(&mut self, key: String, value: Vec<u8>) -> bool {
        let new_size: usize = self.data.values().map(|v| v.len()).sum::<usize>() + value.len();

        if new_size > self.max_size {
            return false;
        }

        self.data.insert(key, value);
        true
    }

    /// Removes a key from the cache.
    ///
    /// Returns the associated value if the key existed.
    fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        self.data.remove(key)
    }

    /// Returns the current memory usage in bytes.
    fn memory_usage(&self) -> usize {
        self.data.values().map(|v| v.len()).sum()
    }

    /// Returns the number of items in the cache.
    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Simulates a memory-intensive data processing task.
///
/// This function creates temporary allocations that should be
/// properly cleaned up after processing.
fn process_data_batch(batch_id: usize, item_count: usize) -> MemScopeResult<Vec<usize>> {
    let tracker = global_tracker()?;

    // Track the batch processing context.
    let batch_data: Vec<u8> = vec![0u8; 1024 * 100]; // 100 KB per batch
    track!(tracker, batch_data);
    // Process items and collect results.
    let results: Vec<usize> = (0..item_count)
        .map(|i| {
            // Create temporary processing buffer.
            let buffer: Vec<u8> = vec![i as u8; 256];
            buffer.iter().map(|&b| b as usize).sum()
        })
        .collect();

    println!(
        "  Batch {}: Processed {} items, buffer size = {} bytes",
        batch_id,
        item_count,
        batch_data.len()
    );

    Ok(results)
}

/// Demonstrates memory leak scenario with a growing cache.
fn simulate_cache_leak(tracker: &memscope_rs::GlobalTracker) -> MemScopeResult<()> {
    println!("\n=== Simulating Cache Memory Leak ===\n");

    // Create a cache with limited size.
    let mut cache = Cache::new(1024 * 1024); // 1 MB limit

    // Simulate cache operations that may leak memory.
    for i in 0..50 {
        let key = format!("item_{}", i);
        let value = vec![i as u8; 1024 * 10]; // 10 KB per item

        if cache.insert(key.clone(), value) {
            println!(
                "  Inserted {}: Cache size = {} bytes, {} items",
                key,
                cache.memory_usage(),
                cache.len()
            );
        } else {
            // Cache is full, but we forgot to remove old items!
            // This simulates a memory leak pattern.
            println!("  Cache full! Item {} rejected", key);
        }

        // Track the cache periodically.
        if i % 10 == 0 {
            track!(tracker, cache.data);
        }
    }

    // Show final cache state.
    println!(
        "\n  Final cache: {} items, {} bytes",
        cache.len(),
        cache.memory_usage()
    );

    Ok(())
}

/// Demonstrates proper memory management with cleanup.
fn simulate_proper_cleanup(tracker: &memscope_rs::GlobalTracker) -> MemScopeResult<()> {
    println!("\n=== Demonstrating Proper Memory Management ===\n");

    let mut cache = Cache::new(1024 * 1024); // 1 MB limit
    let mut evicted_count = 0;

    // Simulate cache operations with proper eviction.
    for i in 0..50 {
        let key = format!("item_{}", i);
        let value = vec![i as u8; 1024 * 10]; // 10 KB per item

        // Before inserting, evict old items if necessary.
        while cache.memory_usage() + value.len() > cache.max_size {
            // Remove the oldest item (first key in this simplified example).
            if let Some(old_key) = cache.data.keys().next().cloned() {
                cache.remove(&old_key);
                evicted_count += 1;
            } else {
                break;
            }
        }

        cache.insert(key.clone(), value);

        if i % 10 == 0 {
            println!(
                "  Step {}: Cache = {} items, {} bytes, evicted = {}",
                i,
                cache.len(),
                cache.memory_usage(),
                evicted_count
            );
            track!(tracker, cache.data);
        }
    }

    println!(
        "\n  Final: {} items, {} bytes, total evicted = {}",
        cache.len(),
        cache.memory_usage(),
        evicted_count
    );

    Ok(())
}

/// Demonstrates batch processing with memory tracking.
fn simulate_batch_processing(tracker: &memscope_rs::GlobalTracker) -> MemScopeResult<()> {
    println!("\n=== Batch Processing Memory Analysis ===\n");

    let initial_stats = tracker.get_stats();

    // Process multiple batches.
    for batch_id in 0..5 {
        let results = process_data_batch(batch_id, 100)?;
        println!("  Batch {} completed: {} results", batch_id, results.len());
    }

    let final_stats = tracker.get_stats();

    println!(
        "\n  Memory delta: {} allocations, {} bytes peak",
        final_stats.total_allocations - initial_stats.total_allocations,
        final_stats.peak_memory_bytes
    );

    Ok(())
}

/// Main entry point for the real-world memory tracking demonstration.
fn main() -> MemScopeResult<()> {
    // Initialize logging system first
    init_logging();

    println!("========================================");
    println!("  MemScope-RS Real-World Demonstration  ");
    println!("========================================");

    // Initialize the global tracker.
    init_global_tracking()?;
    let tracker = global_tracker()?;

    println!("\nGlobal tracker initialized successfully.\n");

    // Run demonstration scenarios.
    simulate_cache_leak(&tracker)?;
    simulate_proper_cleanup(&tracker)?;
    simulate_batch_processing(&tracker)?;

    // Generate analysis report.
    println!("\n=== Memory Analysis Report ===\n");

    let stats = tracker.get_stats();
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Active allocations: {}", stats.active_allocations);
    println!("  Peak memory usage: {} bytes", stats.peak_memory_bytes);
    println!("  Current memory: {} bytes", stats.current_memory_bytes);

    // Use the unified Analyzer API
    println!("\n=== Unified Analyzer API ===\n");
    let mut az = analyzer(&tracker)?;

    // Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);

    // Leak detection
    let leaks = az.detect().leaks();
    println!("\nLeak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);

    // Metrics
    let metrics = az.metrics().summary();
    println!("\nMetrics:");
    println!("  Types: {}", metrics.by_type.len());

    // Export comprehensive reports.
    println!("\n=== Exporting Reports ===\n");

    let output_path = "MemoryAnalysis/real_world_demo";

    println!("  Calling export_json...");
    tracker.export_json(output_path)?;
    println!("  JSON report: {}/memory_snapshots.json", output_path);

    println!("  Calling export_html (unified)...");
    tracker.export_html(output_path)?;
    println!(
        "  HTML dashboard (unified): {}/dashboard_unified_dashboard.html",
        output_path
    );

    println!("  Calling export_html (final)...");
    tracker.export_html_with_template(
        output_path,
        memscope_rs::render_engine::export::DashboardTemplate::Final,
    )?;
    println!(
        "  HTML dashboard (final): {}/dashboard_final_dashboard.html",
        output_path
    );

    println!("\n========================================");
    println!("  Demonstration Complete!               ");
    println!("========================================");

    println!("\nOpen the HTML dashboards to visualize memory patterns.");
    println!(
        "Unified dashboard: {}/dashboard_unified_dashboard.html",
        output_path
    );
    println!(
        "Final dashboard: {}/dashboard_final_dashboard.html",
        output_path
    );

    Ok(())
}
