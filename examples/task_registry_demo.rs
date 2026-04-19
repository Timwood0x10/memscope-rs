//! Task Registry Demo
//!
//! This example demonstrates how to use TaskIdRegistry to track tasks
//! and associate memory allocations with tasks.

use memscope_rs::task_registry::global_registry;
use memscope_rs::tracker::Tracker;

fn main() {
    // Initialize tracker
    let tracker = Tracker::new().expect("Failed to create tracker");

    // Get the global task registry
    let registry = global_registry();

    // Spawn a root task
    let main_task = registry.spawn_task(None, "main".to_string());
    println!("Spawned main task: {}", main_task);

    // Allocate some memory in the main task
    tracker.track_allocation(
        0x1000,
        1024,
        "main_allocation".to_string(),
        "Vec<u8>".to_string(),
    );

    // Spawn a child task
    let child_task = registry.spawn_task(Some(main_task), "child_worker".to_string());
    println!("Spawned child task: {}", child_task);

    // Allocate memory in the child task
    tracker.track_allocation(
        0x2000,
        2048,
        "child_allocation".to_string(),
        "String".to_string(),
    );

    // Complete the child task
    registry.complete_task(child_task);
    println!("Completed child task: {}", child_task);

    // Complete the main task
    registry.complete_task(main_task);
    println!("Completed main task: {}", main_task);

    // Export task graph
    let graph = registry.export_graph();
    println!("Task graph has {} nodes and {} edges", graph.nodes.len(), graph.edges.len());

    // Print task graph as JSON
    println!("Task graph JSON: {}", serde_json::to_string_pretty(&graph).unwrap());
}
