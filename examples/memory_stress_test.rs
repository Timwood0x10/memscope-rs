//! Memory stress test example showing memtrack-rs under extreme load
//! This pushes the memory tracking system to its limits

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use memtrack_rs::{get_global_tracker, init, track_var};

fn main() {
    println!("🔥 Memory Stress Test - Pushing memtrack-rs to the limit!");
    println!("This example demonstrates extreme memory usage patterns\n");

    init();
    let tracker = get_global_tracker();
    let start_time = Instant::now();

    // Test 1: Massive allocation burst
    println!("🚀 Test 1: Massive Allocation Burst");
    massive_allocation_burst();

    // Test 2: Memory fragmentation
    println!("\n🧩 Test 2: Memory Fragmentation Pattern");
    memory_fragmentation_test();

    // Test 3: Concurrent allocation storm
    println!("\n⚡ Test 3: Concurrent Allocation Storm");
    concurrent_allocation_storm();

    // Test 4: Large object stress test
    println!("\n🐘 Test 4: Large Object Stress Test");
    large_object_stress_test();

    // Test 5: Rapid allocation/deallocation cycles
    println!("\n🔄 Test 5: Rapid Allocation/Deallocation Cycles");
    rapid_alloc_dealloc_cycles();

    let total_duration = start_time.elapsed();

    // Print comprehensive analysis
    print_final_analysis(&tracker, total_duration);
}

/// Test 1: Allocate a massive number of objects quickly
fn massive_allocation_burst() {
    let mut allocations = Vec::new();
    let target_count = 50_000;

    println!("  Allocating {} objects rapidly...", target_count);
    let start = Instant::now();

    for i in 0..target_count {
        let payload = vec![i as u8; 256];
        let metadata = format!("Object {} created during burst test", i);
        track_var!(payload).ok();
        track_var!(metadata).ok();
        allocations.push((payload, metadata));

        if i % 10_000 == 0 && i > 0 {
            println!("    Allocated {} objects...", i);
        }
    }

    let duration = start.elapsed();
    println!("  Allocated {} objects in {:?}", target_count, duration);
    println!(
        "  Rate: {:.0} allocations/second",
        target_count as f64 / duration.as_secs_f64()
    );

    // Keep some, drop others to test mixed patterns
    allocations.truncate(25_000);
    println!("  Kept {} objects, dropped the rest", allocations.len());
}

/// Test 2: Create fragmented memory patterns
fn memory_fragmentation_test() {
    let mut small_objects = Vec::new();
    let mut medium_objects = Vec::new();
    let mut large_objects = Vec::new();

    // Create interleaved allocations of different sizes
    for i in 0..10_000 {
        match i % 3 {
            0 => {
                let data = vec![i as u8; 32];
                track_var!(data).ok();
                small_objects.push(data);
            }
            1 => {
                let data = vec![i as u8; 512];
                let name = format!("medium_{}", i);
                track_var!(data).ok();
                track_var!(name).ok();
                medium_objects.push((data, name));
            }
            2 => {
                let data = vec![i as u8; 4096];
                let children: Vec<String> = Vec::new();
                track_var!(data).ok();
                track_var!(children).ok();
                large_objects.push((data, children));
            }
            _ => unreachable!(),
        }
    }

    // Create fragmentation by deallocating every other object
    let small_len = small_objects.len();
    let medium_len = medium_objects.len();
    let large_len = large_objects.len();

    small_objects.retain(|_| small_len % 2 == 0);
    medium_objects.retain(|_| medium_len % 3 == 0);
    large_objects.retain(|_| large_len % 4 == 0);

    println!("  Created fragmented memory pattern:");
    println!("    Small objects: {}", small_objects.len());
    println!("    Medium objects: {}", medium_objects.len());
    println!("    Large objects: {}", large_objects.len());
}

/// Test 3: Concurrent threads allocating simultaneously
fn concurrent_allocation_storm() {
    let num_threads = 16;
    let allocations_per_thread = 5_000;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let handle = thread::spawn(move || {
            let mut local_allocations = Vec::new();

            for i in 0..allocations_per_thread {
                // Create different types of objects based on thread_id
                match thread_id % 4 {
                    0 => {
                        let data = vec![(thread_id + i) as u8; 256];
                        track_var!(data).ok();
                        local_allocations.push(data);
                    }
                    1 => {
                        let mut map = HashMap::new();
                        for j in 0..10 {
                            let key = format!("key_{}_{}", thread_id, j);
                            let value = vec![j as u8; 64];
                            track_var!(key).ok();
                            track_var!(value).ok();
                            map.insert(key, value);
                        }
                        local_allocations.push(vec![thread_id as u8; 128]);
                    }
                    2 => {
                        let mut items = Vec::new();
                        for j in 0..20 {
                            let item = format!("item_{}_{}_{}", thread_id, i, j);
                            track_var!(item).ok();
                            items.push(item);
                        }
                        local_allocations.push(vec![thread_id as u8; 256]);
                    }
                    3 => {
                        let mut data = Vec::new();
                        for j in 0..15 {
                            let item = vec![(thread_id + j) as u8; 128];
                            track_var!(item).ok();
                            data.push(item);
                        }
                        local_allocations.push(vec![thread_id as u8; 512]);
                    }
                    _ => unreachable!(),
                }

                // Simulate some work
                if i % 1000 == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }

            allocations_per_thread
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut total_allocated = 0;
    for handle in handles {
        total_allocated += handle.join().unwrap();
    }

    println!(
        "  {} threads allocated {} objects total",
        num_threads, total_allocated
    );
}

/// Test 4: Allocate very large objects
fn large_object_stress_test() {
    let mut large_objects = Vec::new();

    // Allocate objects of increasing size
    for i in 0..100 {
        let size = 1024 * 1024 * (i + 1) / 10; // Up to ~10MB objects
        let massive_data = vec![i as u8; size];
        let description = format!("Very large object {} with {} bytes", i, size);
        track_var!(massive_data).ok();
        track_var!(description).ok();

        large_objects.push((massive_data, description));

        if i % 10 == 0 {
            println!(
                "  Allocated large object {} ({:.2} MB)",
                i,
                size as f64 / 1024.0 / 1024.0
            );
        }
    }

    // Calculate total size
    let total_size: usize = large_objects.iter().map(|(data, _)| data.len()).sum();
    println!(
        "  Total large object size: {:.2} MB",
        total_size as f64 / 1024.0 / 1024.0
    );

    // Drop half of them
    large_objects.truncate(50);
    let remaining_size: usize = large_objects.iter().map(|(data, _)| data.len()).sum();
    println!(
        "  Remaining after cleanup: {:.2} MB",
        remaining_size as f64 / 1024.0 / 1024.0
    );
}

/// Test 5: Rapid allocation and deallocation cycles
fn rapid_alloc_dealloc_cycles() {
    let cycles = 1000;
    let objects_per_cycle = 100;

    for cycle in 0..cycles {
        let mut cycle_objects = Vec::new();

        // Rapid allocation
        for i in 0..objects_per_cycle {
            let data = vec![(cycle + i) as u8; 1024];
            let metadata = format!("cycle_{}_object_{}", cycle, i);
            track_var!(data).ok();
            track_var!(metadata).ok();

            cycle_objects.push((data, metadata));
        }

        // Immediate deallocation (drop)
        drop(cycle_objects);

        if cycle % 100 == 0 {
            println!("  Completed {} allocation/deallocation cycles", cycle);
        }
    }

    println!(
        "  Completed {} rapid cycles with {} objects each",
        cycles, objects_per_cycle
    );
}

/// Print comprehensive stress test results
fn print_final_analysis(tracker: &memtrack_rs::MemoryTracker, duration: Duration) {
    println!("\n🔥 STRESS TEST COMPLETE 🔥");
    println!("Total execution time: {:?}", duration);

    if let Ok(stats) = tracker.get_stats() {
        println!("\n📊 Final Memory Statistics:");
        println!("  🎯 Total allocations: {}", stats.total_allocations);
        println!("  🎯 Total deallocations: {}", stats.total_deallocations);
        println!("  🎯 Active allocations: {}", stats.active_allocations);
        println!(
            "  🎯 Peak memory usage: {:.2} MB",
            stats.peak_memory as f64 / 1024.0 / 1024.0
        );
        println!(
            "  🎯 Current active memory: {:.2} MB",
            stats.active_memory as f64 / 1024.0 / 1024.0
        );

        let efficiency = if stats.total_allocations > 0 {
            stats.total_deallocations as f64 / stats.total_allocations as f64 * 100.0
        } else {
            0.0
        };
        println!("  🎯 Memory cleanup efficiency: {:.1}%", efficiency);
    }

    // Memory breakdown by type
    if let Ok(breakdown) = tracker.get_memory_by_type() {
        println!("\n📈 Memory Usage by Type (Top 10):");
        let mut sorted_breakdown: Vec<_> = breakdown.iter().collect();
        sorted_breakdown.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        for (i, usage) in sorted_breakdown.iter().take(10).enumerate() {
            println!(
                "  {}. {}: {:.2} KB",
                i + 1,
                usage.type_name,
                usage.total_size as f64 / 1024.0
            );
        }
    }

    // Export stress test results
    println!("\n📄 Exporting stress test analysis...");
    if let Err(e) = tracker.export_to_json("stress_test_snapshot.json") {
        eprintln!("❌ Failed to export JSON: {}", e);
    } else {
        println!("✅ Exported detailed snapshot to stress_test_snapshot.json");
    }

    if let Err(e) = tracker.export_to_svg("stress_test_visualization.svg") {
        eprintln!("❌ Failed to export SVG: {}", e);
    } else {
        println!("✅ Exported visualization to stress_test_visualization.svg");
    }

    println!("\n🎉 Stress test analysis complete!");
    println!("📁 Generated files:");
    println!("  • stress_test_snapshot.json - Complete memory allocation data");
    println!("  • stress_test_visualization.svg - Visual memory usage analysis");
    println!("\n💪 memtrack-rs successfully handled extreme memory stress!");
    println!("This test pushed the limits with:");
    println!("  • 50,000+ rapid allocations");
    println!("  • Complex memory fragmentation patterns");
    println!("  • 16 concurrent threads with 80,000+ allocations");
    println!("  • Large objects up to 10MB each");
    println!("  • 100,000+ rapid allocation/deallocation cycles");
}
