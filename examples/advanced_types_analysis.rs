//! Advanced types analysis demonstration
//!
//! This example demonstrates the unified framework for analyzing complex Rust types
//! like Cell, RefCell, Mutex, RwLock, channels, atomics, etc.

use memscope_rs::{analyze_advanced_types, get_global_tracker, init, track_var};
use std::cell::{Cell, RefCell};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex, RwLock,
};
use std::thread;
use std::time::Duration;

fn main() {
    init();

    println!("🔬 Advanced Types Analysis Demo");
    println!("===============================");

    // Test 1: Interior Mutability Types
    println!("\n📦 Test 1: Interior Mutability Types");
    {
        let cell_data = Cell::new(42i32);
        let _tracked_cell = track_var!(cell_data);
        println!("✅ Created Cell<i32>");

        let refcell_data = RefCell::new(vec![1, 2, 3, 4, 5]);
        let _tracked_refcell = track_var!(refcell_data);
        println!("✅ Created RefCell<Vec<i32>>");

        // Demonstrate RefCell borrowing
        {
            let borrow = refcell_data.borrow();
            println!("   RefCell borrowed (read): len = {}", borrow.len());
        }

        {
            let mut borrow_mut = refcell_data.borrow_mut();
            borrow_mut.push(6);
            println!("   RefCell borrowed (write): added element");
        }
    }

    // Test 2: Synchronization Primitives
    println!("\n🔒 Test 2: Synchronization Primitives");
    {
        let mutex_data = Arc::new(Mutex::new(String::from("Shared data")));
        let _tracked_mutex = track_var!(mutex_data);
        println!("✅ Created Arc<Mutex<String>>");

        let rwlock_data = Arc::new(RwLock::new(vec![10, 20, 30]));
        let _tracked_rwlock = track_var!(rwlock_data);
        println!("✅ Created Arc<RwLock<Vec<i32>>>");

        // Demonstrate concurrent access
        let mutex_clone = mutex_data;
        let rwlock_clone = rwlock_data;

        let handle = thread::spawn(move || {
            // Mutex access
            if let Ok(mut guard) = mutex_clone.lock() {
                *guard = "Modified by thread".to_string();
                println!("   Thread: Modified mutex data");
            }

            // RwLock read access
            if let Ok(guard) = rwlock_clone.read() {
                println!("   Thread: Read RwLock data: {:?}", *guard);
            }

            thread::sleep(Duration::from_millis(10));
        });

        // Main thread access
        if let Ok(guard) = mutex_data.lock() {
            println!("   Main: Mutex data: {}", *guard);
        }

        if let Ok(mut guard) = rwlock_data.write() {
            guard.push(40);
            println!("   Main: Added to RwLock data");
        }

        handle.join().unwrap();
    }

    // Test 3: Channel Types
    println!("\n📡 Test 3: Channel Types");
    {
        let (sender, receiver) = mpsc::channel::<String>();
        let _tracked_receiver = track_var!(receiver);
        println!("✅ Created mpsc channel");

        // Send some data
        sender.send("Hello".to_string()).unwrap();
        sender.send("World".to_string()).unwrap();
        println!("   Sent 2 messages through channel");

        // Create multiple senders
        let sender2 = sender;
        let sender3 = sender;
        let _tracked_sender2 = track_var!(sender2);
        let _tracked_sender3 = track_var!(sender3);
        println!("   Created additional senders");
    }

    // Test 4: Atomic Types
    println!("\n⚛️  Test 4: Atomic Types");
    {
        let atomic_counter = Arc::new(AtomicUsize::new(0));
        println!("✅ Created AtomicUsize");

        let atomic_flag = Arc::new(AtomicBool::new(false));
        let _tracked_flag = track_var!(atomic_flag);
        println!("✅ Created AtomicBool");

        // Demonstrate atomic operations
        atomic_counter.store(42, Ordering::SeqCst);
        atomic_flag.store(true, Ordering::SeqCst);

        println!(
            "   Atomic counter: {}",
            atomic_counter.load(Ordering::SeqCst)
        );
        println!("   Atomic flag: {}", atomic_flag.load(Ordering::SeqCst));

        // Simulate concurrent atomic operations
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let counter = atomic_counter;
                thread::spawn(move || {
                    for _ in 0..10 {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        println!(
            "   Final counter after concurrent increments: {}",
            atomic_counter.load(Ordering::SeqCst)
        );
    }

    // Test 5: Memory Management Types
    println!("\n🧠 Test 5: Memory Management Types");
    {
        let manually_drop = ManuallyDrop::new(vec![1, 2, 3, 4, 5]);
        let _tracked_manually_drop = track_var!(manually_drop);
        println!("✅ Created ManuallyDrop<Vec<i32>>");

        let maybe_uninit: MaybeUninit<String> = MaybeUninit::uninit();
        let _tracked_maybe_uninit = track_var!(maybe_uninit);
        println!("✅ Created MaybeUninit<String>");

        let pinned_data = Pin::new(Box::new("Pinned data".to_string()));
        let _tracked_pin = track_var!(pinned_data);
        println!("✅ Created Pin<Box<String>>");
    }

    // Test 6: Complex nested structures
    println!("\n🏗️ Test 6: Complex nested advanced types");
    {
        let complex = Arc::new(Mutex::new(RefCell::new(vec![
            Cell::new(1),
            Cell::new(2),
            Cell::new(3),
        ])));
        let _tracked_complex = track_var!(complex);
        println!("✅ Created Arc<Mutex<RefCell<Vec<Cell<i32>>>>>");

        // Access the nested structure
        {
            if let Ok(mutex_guard) = complex.lock() {
                let refcell_ref = mutex_guard.borrow();
                if let Some(cell) = refcell_ref.get(0) {
                    cell.set(42);
                    println!("   Modified nested Cell value to 42");
                }
            };
        }
    }

    // Perform advanced types analysis
    println!("\n🔍 Performing advanced types analysis...");

    let tracker = get_global_tracker();
    let allocations = match tracker.get_allocation_history() {
        Ok(allocs) => allocs,
        Err(e) => {
            println!("❌ Failed to get allocation history: {}", e);
            return;
        }
    };

    let analysis = analyze_advanced_types(&allocations);

    println!("\n📊 Analysis Results:");
    println!("==================");
    println!(
        "Total advanced types analyzed: {}",
        analysis.statistics.total_advanced_types
    );

    if !analysis.by_category.is_empty() {
        println!("\n📦 By Category:");
        for (category, types) in &analysis.by_category {
            println!("  {}: {} instances", category, types.len());

            // Show first few examples
            for (i, type_info) in types.iter().take(3).enumerate() {
                println!(
                    "    {}. Behavior: interior_mut={}, thread_safe={}, can_block={}",
                    i + 1,
                    type_info.behavior.has_interior_mutability,
                    type_info.behavior.is_thread_safe,
                    type_info.behavior.can_block
                );
            }
            if types.len() > 3 {
                println!("    ... and {} more", types.len() - 3);
            }
        }
    }

    if !analysis.all_issues.is_empty() {
        println!("\n⚠️  Detected Issues:");
        for (i, issue) in analysis.all_issues.iter().enumerate() {
            println!("  {}. [{:?}] {}", i + 1, issue.severity, issue.description);
            if let Some(suggestion) = &issue.suggestion {
                println!("     💡 Suggestion: {}", suggestion);
            }
        }
    } else {
        println!("\n✅ No issues detected with advanced types!");
    }

    println!("\n📈 Performance Summary:");
    println!("======================");
    println!(
        "Average overhead factor: {:.2}x",
        analysis.performance_summary.total_overhead_factor
    );
    println!(
        "Total memory overhead: {} bytes",
        analysis.performance_summary.total_memory_overhead
    );
    println!(
        "Lock-free types: {:.1}%",
        analysis.performance_summary.lock_free_percentage
    );
    println!(
        "Dominant latency category: {:?}",
        analysis.performance_summary.dominant_latency_category
    );

    if !analysis.statistics.by_category.is_empty() {
        println!("\n📊 Statistics by Category:");
        for (category, count) in &analysis.statistics.by_category {
            println!("  {}: {}", category, count);
        }
    }

    if !analysis.statistics.by_issue_severity.is_empty() {
        println!("\n📊 Issues by Severity:");
        for (severity, count) in &analysis.statistics.by_issue_severity {
            println!("  {}: {}", severity, count);
        }
    }

    if !analysis.statistics.by_latency_category.is_empty() {
        println!("\n📊 Performance by Latency:");
        for (latency, count) in &analysis.statistics.by_latency_category {
            println!("  {}: {}", latency, count);
        }
    }

    // Export comprehensive analysis
    println!("\n📄 Exporting comprehensive analysis...");
    if let Err(e) = tracker.export_to_json("advanced_types_analysis.json") {
        println!("❌ Failed to export analysis: {}", e);
    } else {
        println!("✅ Analysis exported to advanced_types_analysis.json");
        println!("   This file now includes advanced type analysis data");
    }

    println!("\n🎯 Advanced types analysis complete!");
    println!("💡 Key insights:");
    println!(
        "   - Identified {} different categories of advanced types",
        analysis.by_category.len()
    );
    println!(
        "   - Detected {} potential issues",
        analysis.all_issues.len()
    );
    println!(
        "   - Average performance overhead: {:.1}x",
        analysis.performance_summary.total_overhead_factor
    );
    println!(
        "   - {}% of types are lock-free",
        analysis.performance_summary.lock_free_percentage
    );
}
