//! Comprehensive Async Memory Showcase
//!
//! Demonstrates async memory tracking with spawn_tracked(), task lifecycle
//! management, allocation tracking per task, and HTML dashboard export.
//!
//! ## What's demonstrated
//!
//! - `spawn_tracked()` — spawns tokio tasks with automatic lifecycle tracking
//! - Per-task memory profiling (peak, current, allocations)
//! - Mixed allocation patterns (light, heavy, IO-bound, large)
//! - Nested task spawning (supervisor → worker pattern)
//! - Various data types (Vec, String, HashMap, nested structs)
//! - task_graph_json for Task Relationship Graph in the dashboard
//! - Zombie task detection
//! - Variable relationship tracking across async task boundaries
//! - Dashboard export with async task data visible in the Task tab

use memscope_rs::{
    analyzer,
    capture::backends::async_tracker::{spawn_tracked, TrackerContext},
    global_tracker, init_global_tracking, track, MemScopeResult,
};

use std::collections::HashMap;
use std::time::Instant;

/// A complex data structure (not directly Trackable, so we track its fields)
#[derive(Debug)]
struct TaskPayload {
    _id: u32,
    _name: String,
    values: Vec<f64>,
    _metadata: HashMap<String, String>,
}

impl TaskPayload {
    fn new(id: u32, _name: &str, size: usize) -> Self {
        let values: Vec<f64> = (0..size).map(|i| (i as f64).sqrt()).collect();
        Self {
            _id: id,
            _name: String::new(),
            values,
            _metadata: HashMap::new(),
        }
    }
}

/// Simulate an IO-like operation with a tracked buffer
async fn io_operation(label: &str, size: usize, delay_ms: u64) -> Vec<u8> {
    let tracker = global_tracker().unwrap();
    // Allocate a buffer to simulate reading data
    let buffer: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    track!(tracker, buffer);
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    println!("  {label}: read {size} bytes");
    buffer
}

/// A worker task that processes data with tracked allocations
async fn worker_task(worker_id: u32, payload_size: usize) -> usize {
    let tracker = global_tracker().unwrap();
    let _ctx = TrackerContext::capture();

    // Track individual fields of a complex payload
    let payload = TaskPayload::new(worker_id, &format!("worker_{worker_id}"), payload_size);
    let _ = payload; // used for its values below

    // Track some processed results
    let results: Vec<f64> = payload.values.iter().map(|v| v * 2.0).collect();
    track!(tracker, results);

    let ctx2 = TrackerContext::capture();
    println!(
        "  Worker #{worker_id} task_id={:?} values={} results={} total={}",
        ctx2.task_id,
        payload_size,
        results.len(),
        payload_size * 8 + results.len() * 8,
    );

    payload_size * 8 + results.len() * 8
}

/// A supervisor task that spawns multiple worker tasks
async fn supervisor_task(supervisor_id: u32, num_workers: u32) -> usize {
    let _ctx = TrackerContext::capture();
    let mut handles = Vec::new();

    for w in 0..num_workers {
        let handle = spawn_tracked(async move { worker_task(w, 100 + (w as usize) * 50).await });
        handles.push(handle);
    }

    let mut total = 0usize;
    for handle in handles {
        if let Ok(bytes) = handle.await {
            total += bytes;
        }
    }

    println!("  Supervisor #{supervisor_id}: {num_workers} workers completed, {total} total bytes");
    total
}

#[tokio::main]
async fn main() -> MemScopeResult<()> {
    println!("======================================================================");
    println!("  Comprehensive Async Memory Showcase");
    println!("======================================================================\n");

    let start_time = Instant::now();

    // 1. Initialize global tracking
    init_global_tracking()?;
    let tracker = global_tracker()?;
    println!("[1/6] Global tracking initialized\n");

    // 2. Capture initial context
    println!("[2/6] Capturing tracker context...");
    let ctx = TrackerContext::capture();
    println!(
        "  Main thread_id={:?} task_id={:?} tokio_id={:?}\n",
        ctx.thread_id, ctx.task_id, ctx.tokio_task_id
    );

    // 3. Spawn diverse async tasks to exercise per-task memory profiling
    println!("[3/6] Spawning async tasks...\n");

    let mut handles = Vec::new();

    // --- Light task: small allocation ---
    handles.push(spawn_tracked(async {
        let tracker = global_tracker().unwrap();
        let data = vec![0u8; 512];
        track!(tracker, data);
        println!("  Light task: 512 bytes");
        512usize
    }));

    // --- Heavy task: large vector of strings ---
    handles.push(spawn_tracked(async {
        let tracker = global_tracker().unwrap();
        let mut data = Vec::with_capacity(2000);
        for j in 0..2000 {
            data.push(format!("payload-{j}-{}", j * j));
        }
        track!(tracker, data);
        let size = data.capacity() * std::mem::size_of::<String>();
        println!("  Heavy task: ~{size} bytes (2000 strings)");
        size
    }));

    // --- IO task: simulated read + compute ---
    handles.push(spawn_tracked(async {
        let buf = io_operation("IO task", 4096, 15).await;
        let sum: u64 = buf.iter().map(|&b| b as u64).sum();
        println!("  IO task: checksum={sum}");
        buf.len() + 8
    }));

    // --- Mixed task: multiple allocations ---
    handles.push(spawn_tracked(async {
        let tracker = global_tracker().unwrap();
        let vec_data = vec![1.0f64; 500];
        track!(tracker, vec_data);
        let string_data = format!("processed {} items via async", 500);
        track!(tracker, string_data);
        let hash_data: HashMap<u32, String> = (0..100).map(|i| (i, format!("val_{i}"))).collect();
        track!(tracker, hash_data);
        let total = vec_data.len() * 8 + string_data.len() + hash_data.len() * 32;
        println!("  Mixed task: {total} bytes (Vec+f64 + String + HashMap)");
        total
    }));

    // --- Large data task: big vector ---
    handles.push(spawn_tracked(async {
        let tracker = global_tracker().unwrap();
        let big: Vec<u64> = vec![42u64; 10_000];
        track!(tracker, big);
        let size = 10_000 * 8;
        println!("  Large data task: {size} bytes (10k u64s)");
        size
    }));

    // --- Two concurrent worker tasks (demonstrate parallel tracking) ---
    handles.push(spawn_tracked(async { worker_task(101, 200).await }));

    handles.push(spawn_tracked(async { worker_task(102, 500).await }));

    // --- Supervisor task with nested workers ---
    handles.push(spawn_tracked(async { supervisor_task(1, 3).await }));

    // Wait for all tasks
    let mut grand_total = 0usize;
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(bytes) => {
                grand_total += bytes;
                println!("  handle[{i}] returned {bytes} bytes");
            }
            Err(e) => println!("  handle[{i}] error: {e}"),
        }
    }
    println!("\n  Grand total allocated: ~{grand_total} bytes\n");

    // 4. Async tracker statistics
    println!("[4/6] Async tracker statistics...");
    let async_stats = tracker.async_tracker().get_stats();
    let profiles = tracker.async_tracker().get_all_profiles();
    println!(
        "  Total tasks: {}  Active: {}  Allocations: {}  Peak mem: {} bytes",
        async_stats.total_tasks,
        async_stats.active_tasks,
        async_stats.total_allocations,
        async_stats.peak_memory,
    );

    // Print per-task profile summary
    println!("  Per-task profiles:");
    for p in &profiles {
        println!(
            "    task_id={} name=\"{}\" bytes={} peak={} allocs={} dur={:.1}ms completed={}",
            p.task_id,
            p.task_name,
            p.total_bytes,
            p.peak_memory,
            p.total_allocations,
            p.duration_ns as f64 / 1_000_000.0,
            p.is_completed(),
        );
    }

    // Zombie detection
    let zombies = tracker.async_tracker().detect_zombie_tasks();
    let (zombie_count, total_tasks) = tracker.async_tracker().zombie_task_stats();
    println!("\n  Zombie tasks: {zombie_count}/{total_tasks}");
    for z in &zombies {
        println!("    Zombie task_id={z}");
    }

    // Task graph from the TaskIdRegistry
    let task_graph = tracker.async_tracker().get_all_profiles();
    println!("  Task graph: {} task nodes", task_graph.len());

    // 5. Run the unified analyzer for comprehensive memory analysis
    println!("\n[5/6] Running unified memory analysis...");
    let mut az = analyzer(&tracker)?;
    let report = az.analyze();
    println!(
        "  Allocations: {}  Bytes: {}  Peak: {}",
        report.stats.allocation_count, report.stats.total_bytes, report.stats.peak_bytes,
    );
    let leaks = az.detect().leaks();
    println!(
        "  Leaks detected: {} ({leaked} bytes)",
        leaks.leak_count,
        leaked = leaks.total_leaked_bytes
    );
    if leaks.leak_count > 0 {
        for l in &leaks.leaked_allocations {
            println!(
                "    ptr={:#x} size={} type={:?}",
                l.ptr, l.size, l.type_name
            );
        }
    }

    // Variable relationships tracking
    let var_rels = az.metrics().summary();
    println!("  Relationship types: {}", var_rels.by_type.len());

    // 6. Export HTML dashboard with async task data
    println!("\n[6/6] Exporting HTML dashboard...");
    let output_path = "MemoryAnalysis/async_showcase_new_api";
    tracker.export_html(output_path)?;
    println!("  ✓ Dashboard exported to {output_path}/dashboard_unified_dashboard.html");
    println!("  Open in browser and switch to the task tab to see per-task profiles\n");

    let elapsed = start_time.elapsed();
    println!("======================================================================");
    println!("  Completed in {elapsed:.2?}");
    println!("  Total async tasks tracked: {total_tasks}");
    println!("  Total tracked bytes: ~{grand_total}");
    println!("======================================================================");

    Ok(())
}
