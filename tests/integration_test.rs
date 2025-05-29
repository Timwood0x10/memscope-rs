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


#[cfg(test)]
mod export_tests {
    use super::*; // Make items from parent module available
    use std::fs::{self, File};
    use std::io::Read;
    use tempfile::tempdir;
    use trace_tools::{init, get_global_tracker, track_var}; // Redundant if super::* brings them in, but explicit
    use serde_json; // Ensure serde_json is in scope for Value

    #[test]
    fn test_json_export_with_data() {
        init();
        let tracker = get_global_tracker();

        // Clear tracker state for this test
        let _ = tracker.get_active_allocations();
        let _ = tracker.get_allocation_log();

        let v1 = vec![1, 2, 3];
        track_var!(v1);
        let s1 = "hello".to_string();
        track_var!(s1);

        let dir = tempdir().unwrap();
        let json_path = dir.path().join("integration_output.json");
        
        tracker.export_to_json(&json_path).unwrap();

        assert!(json_path.exists(), "JSON file was not created");

        let mut file_content = String::new();
        File::open(&json_path)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();
        
        assert!(!file_content.is_empty(), "JSON file content is empty");

        let json_value: serde_json::Value = serde_json::from_str(&file_content).unwrap();
        assert!(
            json_value["active_allocations"].as_array().map_or(false, |a| !a.is_empty()),
            "JSON active_allocations should not be empty"
        );
        assert!(
            json_value["active_allocations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|alloc| alloc["var_name"].as_str() == Some("v1")),
            "Tracked variable 'v1' not found in JSON active_allocations"
        );
    }

    #[test]
    fn test_svg_export_with_data() {
        init();
        let tracker = get_global_tracker();

        // Clear tracker state for this test
        let _ = tracker.get_active_allocations();
        let _ = tracker.get_allocation_log(); // Crucial for SVG

        {
            let v_svg = vec![10, 20, 30];
            track_var!(v_svg);
            // v_svg is dropped here, so it should be in the allocation_log
        }
        let s_svg_active = "persistent_string".to_string(); // This will be active
        track_var!(s_svg_active);

        let dir = tempdir().unwrap();
        let svg_path = dir.path().join("integration_output.svg");

        tracker.export_to_svg(&svg_path).unwrap();

        assert!(svg_path.exists(), "SVG file was not created");

        let svg_content = fs::read_to_string(&svg_path).unwrap();
        
        assert!(!svg_content.is_empty(), "SVG file content is empty");
        assert!(
            !svg_content.contains("No allocation data collected."),
            "SVG content indicates no data was collected"
        );
        assert!(
            svg_content.contains("v_svg"),
            "SVG content does not contain 'v_svg', which should be in the log"
        );
    }
}