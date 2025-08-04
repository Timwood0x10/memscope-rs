//! Container Analysis Demo
//! 
//! This example demonstrates the enhanced container analysis functionality
//! for Vec, HashMap, Box, and other container types.

use memscope_rs::{track_var, MemoryTracker, init};
use std::collections::HashMap;

fn main() {
    println!("=== Container Analysis Demo ===\n");

    // Initialize memory tracking system
    init();
    
    // Initialize memory tracker
    let tracker = MemoryTracker::new();

    // Demonstrate Vec analysis
    println!("1. Vec<i32> Analysis:");
    demonstrate_vec_analysis(&tracker);
    println!();

    // Demonstrate HashMap analysis
    println!("2. HashMap<String, i32> Analysis:");
    demonstrate_hashmap_analysis(&tracker);
    println!();

    // Demonstrate Box analysis
    println!("3. Box<String> Analysis:");
    demonstrate_box_analysis(&tracker);
    println!();

    // Demonstrate String analysis
    println!("4. String Analysis:");
    demonstrate_string_analysis(&tracker);
    println!();

    // Demonstrate actual container usage with tracking
    println!("5. Live Container Tracking:");
    demonstrate_live_tracking();
}

fn demonstrate_vec_analysis(tracker: &MemoryTracker) {
    // Analyze different Vec sizes
    let sizes = [32, 128, 512, 2048];
    
    for size in sizes {
        if let Some(layout) = tracker.analyze_memory_layout("Vec<i32>", size) {
            println!("  Vec<i32> (size: {} bytes):", size);
            
            if let Some(container_analysis) = &layout.container_analysis {
                match &container_analysis.container_type {
                    memscope_rs::core::types::ContainerType::Vec { element_type, element_size } => {
                        println!("    Element type: {}, size: {} bytes", element_type, element_size);
                    }
                    _ => {}
                }
                
                let utilization = &container_analysis.capacity_utilization;
                println!("    Capacity: {}, Length: {}, Utilization: {:.1}%", 
                    utilization.current_capacity,
                    utilization.current_length,
                    utilization.utilization_ratio * 100.0
                );
                
                let patterns = &container_analysis.reallocation_patterns;
                println!("    Estimated reallocations: {}", patterns.estimated_reallocations);
                println!("    Growth pattern: {:?}", patterns.growth_pattern);
                
                let metrics = &container_analysis.efficiency_metrics;
                println!("    Health score: {:.1}/100", metrics.health_score);
                println!("    Cache efficiency: {:.1}%", metrics.cache_efficiency);
                println!("    Memory overhead: {:.1}%", metrics.memory_overhead);
            }
            println!();
        }
    }
}

fn demonstrate_hashmap_analysis(tracker: &MemoryTracker) {
    let sizes = [64, 256, 1024];
    
    for size in sizes {
        if let Some(layout) = tracker.analyze_memory_layout("HashMap<String, i32>", size) {
            println!("  HashMap<String, i32> (size: {} bytes):", size);
            
            if let Some(container_analysis) = &layout.container_analysis {
                match &container_analysis.container_type {
                    memscope_rs::core::types::ContainerType::HashMap { key_type, value_type, key_size, value_size } => {
                        println!("    Key: {} ({} bytes), Value: {} ({} bytes)", 
                            key_type, key_size, value_type, value_size);
                    }
                    _ => {}
                }
                
                let utilization = &container_analysis.capacity_utilization;
                println!("    Estimated buckets: {}, Load factor: {:.1}%", 
                    utilization.current_capacity,
                    utilization.utilization_ratio * 100.0
                );
                
                let patterns = &container_analysis.reallocation_patterns;
                println!("    Estimated rehashes: {}", patterns.estimated_reallocations);
                
                let metrics = &container_analysis.efficiency_metrics;
                println!("    Health score: {:.1}/100", metrics.health_score);
                println!("    Access efficiency: {:?}", metrics.access_efficiency);
            }
            println!();
        }
    }
}

fn demonstrate_box_analysis(tracker: &MemoryTracker) {
    if let Some(layout) = tracker.analyze_memory_layout("Box<String>", 8) {
        println!("  Box<String> (size: 8 bytes):");
        
        if let Some(container_analysis) = &layout.container_analysis {
            match &container_analysis.container_type {
                memscope_rs::core::types::ContainerType::Box { boxed_type, boxed_size } => {
                    println!("    Boxed type: {}, size: {} bytes", boxed_type, boxed_size);
                }
                _ => {}
            }
            
            let patterns = &container_analysis.reallocation_patterns;
            println!("    Growth pattern: {:?}", patterns.growth_pattern);
            println!("    Reallocation frequency: {:?}", patterns.frequency_assessment);
            
            let metrics = &container_analysis.efficiency_metrics;
            println!("    Health score: {:.1}/100", metrics.health_score);
            println!("    Memory overhead: {:.1}%", metrics.memory_overhead);
        }
        println!();
    }
}

fn demonstrate_string_analysis(tracker: &MemoryTracker) {
    let sizes = [24, 48, 96];
    
    for size in sizes {
        if let Some(layout) = tracker.analyze_memory_layout("String", size) {
            println!("  String (size: {} bytes):", size);
            
            if let Some(container_analysis) = &layout.container_analysis {
                let utilization = &container_analysis.capacity_utilization;
                println!("    Capacity: {} bytes, Length: {} bytes", 
                    utilization.current_capacity,
                    utilization.current_length
                );
                println!("    Utilization: {:.1}%", utilization.utilization_ratio * 100.0);
                
                let patterns = &container_analysis.reallocation_patterns;
                println!("    Estimated reallocations: {}", patterns.estimated_reallocations);
                
                let metrics = &container_analysis.efficiency_metrics;
                println!("    Health score: {:.1}/100", metrics.health_score);
                println!("    Cache efficiency: {:.1}%", metrics.cache_efficiency);
            }
            println!();
        }
    }
}

fn demonstrate_live_tracking() {
    // Create some containers and track their memory usage
    let vec_data = Vec::<i32>::with_capacity(100);
    track_var!(vec_data);
    println!("  Created Vec with capacity 100");
    
    let mut map_data = HashMap::<String, i32>::new();
    map_data.insert("key1".to_string(), 42);
    map_data.insert("key2".to_string(), 84);
    track_var!(map_data);
    println!("  Created HashMap with 2 entries");
    
    let box_data = Box::new("Hello, World!".to_string());
    track_var!(box_data);
    println!("  Created Box<String>");
    
    let string_data = String::with_capacity(50);
    track_var!(string_data);
    println!("  Created String with capacity 50");
    
    // The memory tracker will analyze these containers when they're tracked
    println!("  All containers are now being tracked with enhanced analysis!");
    
    // Clean up to avoid unused variable warnings
    drop(vec_data);
    drop(map_data);
    drop(box_data);
    drop(string_data);
}