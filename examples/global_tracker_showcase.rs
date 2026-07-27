//! Global Tracker Showcase - Unified `start()` API
//!
//! This example demonstrates how to use the global tracker across all execution modes:
//! - Single-threaded mode
//! - Multi-threaded mode
//! - Async mode
//! - Unsafe/FFI mode
//! - Task tracking with TaskIdRegistry
//!
//! ## Setup
//!
//! The example uses the one-line `memscope_rs::start()` entry point which:
//! 1. Initializes the tracing subscriber.
//! 2. Installs the global `GlobalTracker` singleton.
//! 3. Wires the auto-export lifecycle hooks (panic / Ctrl-C / Drop).
//!
//! The returned [`MemScopeGuard`] derefs to [`GlobalTracker`], so `track!`
//! works directly on it. On `main` return (or panic / Ctrl-C) the guard's
//! `Drop` runs the idempotent exit-path export, writing the dashboard + JSON
//! to `./memscope-report/`.

use memscope_rs::task_registry::global_registry;
use memscope_rs::{analyzer, MemScopeResult};

use memscope_rs::track;

use std::{
    alloc::{alloc, dealloc, Layout},
    rc::Rc,
    sync::Arc,
    thread,
    time::Instant,
};

fn main() -> MemScopeResult<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        Global Tracker Showcase - New Unified API           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // One-line start: logging + global tracker + auto-export hooks. The guard
    // owns the lifecycle; dropping it (or panic / Ctrl-C) triggers the export
    // to ./memscope-report/ automatically.
    let guard = memscope_rs::start()?;

    println!("📦 Section 1: Single-Threaded Mode\n");
    let single_start = Instant::now();
    {
        let v1 = vec![1i32, 2, 3, 4, 5];
        let v2 = v1.clone();
        let v3 = v2.clone();
        track!(guard, v1);
        track!(guard, v2);
        track!(guard, v3);

        let s1 = String::from("Hello, global tracking!");
        let s2 = s1.clone();
        let s3 = s2.clone();
        track!(guard, s1);
        track!(guard, s2);
        track!(guard, s3);

        let b1 = Box::new(42i64);
        let b2 = b1.clone();
        track!(guard, b1);
        track!(guard, b2);

        let arc1 = Arc::new(vec![1i32, 2, 3]);
        let arc2 = arc1.clone();
        let arc3 = arc1.clone();
        track!(guard, arc1);
        track!(guard, arc2);
        track!(guard, arc3);

        let rc1 = Rc::new(String::from("Rc string"));
        let rc2 = rc1.clone();
        let rc3 = rc1.clone();
        track!(guard, rc1);
        track!(guard, rc2);
        track!(guard, rc3);

        let boxed_vec = Box::new(vec![1i32, 2, 3, 4, 5]);
        let owned_string = String::from("Owned string");
        let cloned_vec = boxed_vec.clone();
        track!(guard, boxed_vec);
        track!(guard, owned_string);
        track!(guard, cloned_vec);

        println!("✓ Tracked 18 allocations with clones and smart pointers");
    }
    println!(
        "  Duration: {:.2}ms\n",
        single_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 2: Multi-Threaded Mode\n");
    let multi_start = Instant::now();
    let handles: Vec<_> = (0..4)
        .map(|id| {
            thread::spawn(move || {
                let ctx = memscope_rs::global_tracker().unwrap();
                for i in 0..100 {
                    track!(ctx, vec![i; 16]);
                }
                // Add shared data between threads
                let _shared_arc = Arc::new(vec![1i32, 2, 3]);
                let _thread_rc = Rc::new(format!("Thread {} string", id));
                println!("  Thread {}: tracked 100 allocations + smart pointers", id);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("✓ Multi-threaded completed");
    println!(
        "  Duration: {:.2}ms\n",
        multi_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 3: Async Mode\n");
    let async_start = Instant::now();
    run_async_mode()?;
    println!("✓ Async completed");
    println!(
        "  Duration: {:.2}ms\n",
        async_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 4: Unsafe/FFI Mode\n");
    let unsafe_start = Instant::now();
    run_unsafe_ffi_mode()?;
    println!("✓ Unsafe/FFI completed");
    println!(
        "  Duration: {:.2}ms\n",
        unsafe_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 5: Circular Reference Detection\n");
    let cycle_start = Instant::now();
    {
        let ctx = memscope_rs::global_tracker()?;

        println!("  Creating variables with circular clone relationships...");

        let data1 = vec![1, 2, 3];
        let data2 = vec![4, 5, 6];
        let data3 = vec![7, 8, 9];

        // Track initial allocations
        track!(ctx, data1);
        track!(ctx, data2);
        track!(ctx, data3);

        // Create explicit circular references via variable names
        // This demonstrates the cycle detection in the relationship graph
        // In a real scenario, Rc/Arc internal pointers would create similar patterns

        // Create String clones that form a cycle
        let s1 = String::from("cycle_node_1");
        let s2 = String::from("cycle_node_2");
        let s3 = String::from("cycle_node_3");

        track!(ctx, s1);
        track!(ctx, s2);
        track!(ctx, s3);

        // Create Rc clones that will form cycles
        struct Node {
            _value: i32,
            next: Option<std::rc::Rc<std::cell::RefCell<Node>>>,
        }

        // Track nodes in sequence - these form a cycle when linked
        let n1 = std::rc::Rc::new(std::cell::RefCell::new(Node {
            _value: 1,
            next: None,
        }));
        let n2 = std::rc::Rc::new(std::cell::RefCell::new(Node {
            _value: 2,
            next: None,
        }));
        let n3 = std::rc::Rc::new(std::cell::RefCell::new(Node {
            _value: 3,
            next: None,
        }));

        track!(ctx, n1);
        track!(ctx, n2);
        track!(ctx, n3);

        // Create the circular links after tracking
        // This creates internal references that may form cycles
        n1.borrow_mut().next = Some(std::rc::Rc::clone(&n2));
        n2.borrow_mut().next = Some(std::rc::Rc::clone(&n3));
        n3.borrow_mut().next = Some(std::rc::Rc::clone(&n1)); // Creates a cycle!

        println!("✓ Created circular reference structures");
        println!("  - 3 Rc nodes linked in a cycle: n1 -> n2 -> n3 -> n1");
        println!("  - In the relationship graph, cycle edges will appear as RED DASHED lines");
    }
    println!(
        "  Duration: {:.2}ms\n",
        cycle_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 6: Task Tracking with TaskIdRegistry\n");
    let registry = global_registry();
    let task_start = Instant::now();
    {
        let ctx = memscope_rs::global_tracker()?;

        println!("  Demonstrating task hierarchy tracking...");

        // Simplified API - automatic lifecycle management
        {
            let _main = registry.task_scope("main_process");
            println!("  ✓ Created main task");

            // Allocate memory in main task
            let main_data = vec![1i32, 2, 3, 4, 5];
            track!(ctx, main_data);

            {
                let _worker = registry.task_scope("worker_thread");
                println!("  ✓ Created child task (parent: main)");

                // Allocate memory in child task
                let child_data = vec![10i32, 20, 30];
                track!(ctx, child_data);

                {
                    let _sub_worker = registry.task_scope("sub_worker");
                    println!("  ✓ Created grandchild task (parent: worker)");

                    // Allocate memory in grandchild task
                    let grandchild_data = String::from("Grandchild data");
                    track!(ctx, grandchild_data);
                } // sub_worker automatically completed
            } // worker automatically completed
        } // main automatically completed

        println!("  ✓ Created task hierarchy: main -> worker -> sub_worker");
    }
    println!(
        "  Duration: {:.2}ms\n",
        task_start.elapsed().as_secs_f64() * 1000.0
    );

    println!("📦 Section 7: Statistics\n");
    let ctx = memscope_rs::global_tracker()?;
    let stats = ctx.get_stats();
    println!("✓ Total allocations: {}", stats.total_allocations);
    println!("✓ Active allocations: {}", stats.active_allocations);
    println!(
        "✓ Peak memory: {:.2} MB",
        stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
    );
    println!("✓ Memory passports: {}", stats.passport_count);

    // Use the unified Analyzer API
    println!("\n📦 Section 7: Unified Analyzer API\n");
    let mut az = analyzer(&ctx)?;

    // Full analysis
    let report = az.analyze();
    println!("Analysis Report:");
    println!("  Allocations: {}", report.stats.allocation_count);
    println!("  Total Bytes: {}", report.stats.total_bytes);
    println!("  Peak Bytes: {}", report.stats.peak_bytes);
    println!("  Threads: {}", report.stats.thread_count);

    // Leak detection
    let leaks = az.detect().leaks();
    println!("\nLeak Detection:");
    println!("  Leak Count: {}", leaks.leak_count);
    println!("  Leaked Bytes: {}", leaks.total_leaked_bytes);

    // Metrics
    let metrics = az.metrics().summary();
    println!("\nMetrics:");
    println!("  Types: {}", metrics.by_type.len());

    // On-demand snapshot: no disk write, just an in-memory JSON string.
    let snapshot = guard.snapshot_json()?;
    println!("\n📦 Section 8: On-demand snapshot (no disk write)\n");
    println!("  snapshot JSON length: {} bytes", snapshot.len());

    println!("\n✓ All modes completed successfully!");
    println!("\n🆕 Drop will write dashboard + JSON to ./memscope-report/");

    // `_guard` drops here, triggering the exit-path export to
    // ./memscope-report/dashboard_unified_dashboard.html + memory_analysis.json.
    drop(guard);
    Ok(())
}

#[tokio::main]
async fn run_async_mode() -> MemScopeResult<()> {
    println!("Spawning 4 async tasks...");

    let ctx = memscope_rs::global_tracker()?;
    let async_tracker = ctx.async_tracker().clone();

    let tasks = (0..4).map(|i| {
        let async_tracker = async_tracker.clone();
        async move {
            let task_id = i as u64;
            let thread_id = std::thread::current().id();

            // Track async task start
            if let Err(e) =
                async_tracker.track_task_start(task_id, format!("async_task_{}", i), thread_id)
            {
                eprintln!("Warning: {}", e);
            }

            let ctx = memscope_rs::global_tracker().unwrap();

            // Track allocations in both trackers
            let vec_data = vec![0u64; 50];
            let vec_size =
                std::mem::size_of_val(&vec_data) + vec_data.len() * std::mem::size_of::<u64>();
            track!(ctx, vec_data);
            async_tracker.track_allocation(i * 1000, vec_size, task_id);

            let string_data = format!("Async task: {}", i);
            let string_size = string_data.len();
            track!(ctx, string_data);
            async_tracker.track_allocation(i * 1000 + 1, string_size, task_id);

            // Vary sleep durations so poll latency samples are realistic and
            // the chart shows a non-flat curve (5ms, 15ms, 30ms, 10ms).
            let sleep_ms = [5u64, 15, 30, 10][i];
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;

            // Track async task end
            if let Err(e) = async_tracker.track_task_end(task_id) {
                eprintln!("Warning: {}", e);
            }

            println!("  Task-{}: tracked 2 allocations (slept {}ms)", i, sleep_ms);
        }
    });

    futures::future::join_all(tasks).await;
    Ok(())
}

fn run_unsafe_ffi_mode() -> MemScopeResult<()> {
    let ctx = memscope_rs::global_tracker()?;

    println!("Spawning unsafe/FFI operations...");

    // Unsafe Rust allocations
    for i in 0..5 {
        unsafe {
            let layout = Layout::new::<[i32; 64]>();
            let ptr = alloc(layout);

            if !ptr.is_null() {
                ctx.create_passport(ptr as usize, layout.size(), format!("unsafe_vec_{}", i))
                    .map_err(|e| {
                        memscope_rs::MemScopeError::error(
                            "global_tracker_showcase",
                            "run_unsafe_ffi_mode",
                            e.to_string(),
                        )
                    })?;

                let slice = std::slice::from_raw_parts_mut(ptr as *mut i32, 64);
                for (j, item) in slice.iter_mut().enumerate() {
                    *item = (i * 100 + j) as i32;
                }

                println!("  Unsafe-{}: {} bytes", i, layout.size());
                dealloc(ptr, layout);
            }
        }
    }

    // FFI operations
    extern "C" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
        fn free(ptr: *mut std::ffi::c_void);
    }

    for i in 0..5 {
        let size = 256 * (i + 1);
        let ffi_ptr = unsafe { malloc(size) };

        if !ffi_ptr.is_null() {
            // FFI memory type is unknown at compile time, use *mut c_void
            ctx.create_passport(ffi_ptr as usize, size, format!("ffi_alloc_{}", i))
                .map_err(|e| {
                    memscope_rs::MemScopeError::error(
                        "global_tracker_showcase",
                        "run_unsafe_ffi_mode",
                        e.to_string(),
                    )
                })?;

            ctx.record_handover(
                ffi_ptr as usize,
                "foreign_function".to_string(),
                format!("ffi_call_{}", i),
            );

            unsafe {
                std::ptr::write_bytes(ffi_ptr as *mut u8, (0x40 + i) as u8, size);
                free(ffi_ptr);
            }

            println!("  FFI-{}: {} bytes", i, size);
        }
    }

    Ok(())
}
