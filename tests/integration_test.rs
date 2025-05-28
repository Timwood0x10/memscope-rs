use std::sync::Arc;
use trace_tools::{get_global_tracker, init, track_var, MemoryTracker};

// Helper function to clear allocations
fn clear_allocations(tracker: &Arc<MemoryTracker>) {
    let _ = tracker.get_active_allocations(); // This should clear the tracker
}

#[test]
fn test_allocation_tracking() {
    // Initialize the memory tracker
    init();
    let tracker = get_global_tracker();
    
    // Clear any existing allocations
    clear_allocations(&tracker);
    
    // Create and track a vector
    let vec = vec![1, 2, 3];
    track_var!(vec);
    
    // Get active allocations
    let active = tracker.get_active_allocations();
    
    // Verify allocation was tracked
    assert!(!active.is_empty(), "No allocations were tracked");
    let found = active.iter().any(|a| a.var_name.as_deref() == Some("vec"));
    assert!(found, "Vector allocation not found in active allocations");
}

#[test]
fn test_allocation_size() {
    // Initialize and clear tracker
    init();
    let tracker = get_global_tracker();
    clear_allocations(&tracker);
    
    // Create a vector with known size
    let size = 1024;
    let vec = vec![0; size];
    println!("Created vector with size {}", size);
    
    // Track the vector
    track_var!(vec);
    println!("Tracked vector");
    
    // Get active allocations
    let active = tracker.get_active_allocations();
    println!("Active allocations ({}):", active.len());
    for (i, alloc) in active.iter().enumerate() {
        println!("[{}] var_name: {:?}, type_name: {:?}, size: {}", 
            i, alloc.var_name, alloc.type_name, alloc.size);
    }
    
    assert!(!active.is_empty(), "No allocations were tracked");
    
    // Verify allocation size
    let allocation = active.iter().find(|a| a.var_name.as_deref() == Some("vec"))
        .expect("Vector allocation not found");
    
    assert!(allocation.size >= size * std::mem::size_of::<i32>(),
        "Allocation size is too small. Expected at least {}, got {}",
        size * std::mem::size_of::<i32>(), allocation.size
    );
}

#[test]
fn test_allocation_cleanup() {
    // Initialize and clear tracker
    init();
    let tracker = get_global_tracker();
    clear_allocations(&tracker);
    
    // Create and track a vector that will be dropped
    {
        let vec = vec![1, 2, 3];
        track_var!(vec);
    }
    
    // Verify allocation was cleaned up
    let active = tracker.get_active_allocations();
    assert!(active.is_empty(), "Allocations were not cleaned up after drop");
}

#[test]
fn test_allocation_types() {
    // Initialize and clear tracker
    init();
    let tracker = get_global_tracker();
    clear_allocations(&tracker);
    
    // Create different types of allocations
    let vec = vec![1, 2, 3];
    let string = "test".to_string();
    let boxed = Box::new(42);
    let rc = Arc::new(42);
    
    // Track all allocations
    track_var!(vec);
    track_var!(string);
    track_var!(boxed);
    track_var!(rc);
    
    // Verify all allocations were tracked
    let active = tracker.get_active_allocations();
    assert!(!active.is_empty(), "No allocations were tracked");
    
    let found_vec = active.iter().any(|a| a.var_name.as_deref() == Some("vec"));
    let found_string = active.iter().any(|a| a.var_name.as_deref() == Some("string"));
    let found_boxed = active.iter().any(|a| a.var_name.as_deref() == Some("boxed"));
    let found_rc = active.iter().any(|a| a.var_name.as_deref() == Some("rc"));
    
    assert!(found_vec, "Vector allocation not found");
    assert!(found_string, "String allocation not found");
    assert!(found_boxed, "Box allocation not found");
    assert!(found_rc, "Rc allocation not found");
}