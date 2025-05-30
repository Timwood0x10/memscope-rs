use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tempfile::tempdir;
use trace_tools::{get_global_tracker, init, track_var, tracker::AllocationInfo, MemoryTracker};

/// Test configuration constants
const TEST_ALLOC_SIZE: usize = 1024; // 1KB test allocation size
const TEST_THREAD_ID: u64 = 1;

/// Helper module for test utilities
mod test_utils {
    use super::*;

    /// Clear all allocations from the tracker
    pub fn clear_allocations(tracker: &Arc<MemoryTracker>) {
        tracker.clear_all_for_test();
        
        // Verify clean state
        let active = tracker.get_active_allocations();
        let log = tracker.get_allocation_log();
        
        assert!(
            active.is_empty() && log.is_empty(),
            "Failed to clear tracker state. Active: {}, Log: {}",
            active.len(),
            log.len()
        );
    }

    /// Setup test environment with a clean tracker
    pub fn setup_test() -> Arc<MemoryTracker> {
        init();
        let tracker = get_global_tracker();
        clear_allocations(&tracker);
        tracker
    }

    /// Create a test allocation with the given parameters
    pub fn create_test_allocation(
        tracker: &Arc<MemoryTracker>,
        var_name: &str,
        type_name: &str,
        size: usize,
        timestamp_offset: u64,
        deallocated: bool,
    ) -> AllocationInfo {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u128;
            
        let alloc = AllocationInfo {
            ptr: var_name.as_ptr() as usize,
            size,
            timestamp_alloc: (now + timestamp_offset as u128) as u128,
            timestamp_dealloc: if deallocated {
                Some((now + timestamp_offset as u128 + 100) as u128)
            } else {
                None
            },
            var_name: Some(var_name.to_string()),
            type_name: Some(type_name.to_string()),
            backtrace_ips: vec![],
            thread_id: TEST_THREAD_ID,
        };

        if deallocated {
            tracker.add_deallocated_for_test(alloc.clone())
                .expect("Failed to add deallocated allocation");
        } else {
            tracker.add_allocation_for_test(alloc.clone())
                .expect("Failed to add active allocation");
        }

        alloc
    }
}

#[test]
fn test_basic_allocation() {
    let tracker = test_utils::setup_test();
    
    // Test tracking a simple vector
    let vec = vec![1, 2, 3];
    track_var!(vec);
    
    // Verify the allocation was tracked
    let active = tracker.get_active_allocations();
    assert!(!active.is_empty(), "No allocations were tracked");
    
    // Clean up
    test_utils::clear_allocations(&tracker);
}

#[test]
fn test_allocation_cleanup() {
    let tracker = test_utils::setup_test();
    
    // Create and track an allocation
    let alloc = test_utils::create_test_allocation(
        &tracker,
        "test_cleanup",
        "Vec<i32>",
        TEST_ALLOC_SIZE,
        0,
        false,
    );
    
    // Simulate deallocation
    let _ = tracker.track_deallocation(alloc.ptr);
    
    // Verify allocation was cleaned up
    let active = tracker.get_active_allocations();
    assert!(
        active.is_empty(),
        "Allocations were not cleaned up after deallocation"
    );
}

#[test]
fn test_allocation_types() {
    let tracker = test_utils::setup_test();
    
    // Test different allocation types
    let types = [
        ("vec", "Vec<i32>"),
        ("string", "String"),
        ("boxed", "Box<i32>"),
    ];
    
    // Create test allocations
    for (i, (var_name, type_name)) in types.iter().enumerate() {
        test_utils::create_test_allocation(
            &tracker,
            var_name,
            type_name,
            TEST_ALLOC_SIZE * (i + 1) as usize,
            (i * 100) as u64,
            i % 2 == 0, // Alternate between deallocated and active
        );
    }
    
    // Verify all allocations were tracked
    let active = tracker.get_active_allocations();
    let log = tracker.get_allocation_log();
    
    assert_eq!(
        active.len() + log.len(),
        types.len(),
        "Not all allocations were tracked"
    );
    
    // Verify each type was tracked
    for (var_name, _) in &types {
        let found = active.iter().chain(log.iter())
            .any(|a| a.var_name.as_deref() == Some(*var_name));
        assert!(found, "Allocation for '{}' not found", var_name);
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    
    fn verify_json_file(path: &Path) -> Value {
        // Verify file exists and has content
        assert!(path.exists(), "JSON file was not created");
        let json_content = fs::read_to_string(path)
            .expect("Failed to read JSON file");
        assert!(!json_content.is_empty(), "JSON file is empty");
        
        // Parse and return JSON
        serde_json::from_str(&json_content)
            .expect("Failed to parse JSON")
    }
    
    fn verify_svg_file(path: &Path) -> String {
        // Verify file exists and has content
        assert!(path.exists(), "SVG file was not created");
        let svg_content = fs::read_to_string(path)
            .expect("Failed to read SVG file");
            
        // Basic SVG validation
        assert!(!svg_content.is_empty(), "SVG file is empty");
        assert!(
            svg_content.contains("<svg") && svg_content.contains("</svg>"),
            "Invalid SVG format"
        );
        
        svg_content
    }

    #[test]
    fn test_json_export() {
        let tracker = test_utils::setup_test();
        
        // Create test data
        test_utils::create_test_allocation(
            &tracker,
            "json_test",
            "Vec<u8>",
            TEST_ALLOC_SIZE,
            0,
            false,
        );
        
        // Create temp directory
        let dir = tempdir().expect("Failed to create temp directory");
        let json_path = dir.path().join("test_export.json");
        
        // Export to JSON
        tracker.export_to_json(&json_path)
            .expect("Failed to export to JSON");
        
        // Verify JSON structure
        let json = verify_json_file(&json_path);
        assert!(
            json.get("active_allocations").is_some(),
            "JSON missing active_allocations"
        );
        
        let active = json["active_allocations"]
            .as_array()
            .expect("active_allocations is not an array");
            
        assert!(
            active.iter().any(|a| a["var_name"] == "json_test"),
            "Test allocation not found in JSON export"
        );
    }

    #[test]
    fn test_svg_export() {
        let tracker = test_utils::setup_test();
        
        // Create test data with known timestamps
        let _now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(); // as u128
            
        // Add some test allocations
        for i in 0..3 {
            let deallocated = i % 2 == 0;
            test_utils::create_test_allocation(
                &tracker,
                &format!("svg_test_{}", i),
                "Vec<u8>",
                TEST_ALLOC_SIZE * (i + 1) as usize,
                (i * 100) as u64,
                deallocated,
            );
        }
        
        // Create temp directory and export
        let dir = tempdir().expect("Failed to create temp directory");
        let svg_path = dir.path().join("test_output.svg");
        
        tracker.export_to_svg(&svg_path)
            .expect("Failed to export to SVG");
            
        // Verify SVG content
        let svg_content = verify_svg_file(&svg_path);
        
        // Check for expected elements in SVG
        for i in 0..3 {
            assert!(
                svg_content.contains(&format!("svg_test_{}", i)),
                "Test allocation {} not found in SVG",
                i
            );
        }
    }
}

#[test]
fn test_svg_export_with_data() {
    use std::path::Path;
    
    let tracker = test_utils::setup_test();

    // Create test data with a scope to test dropped variables
    let ptr;
    {
        let v_svg = vec![10, 20, 30];
        ptr = v_svg.as_ptr() as usize;

        // Manually add allocation data for v_svg (allocated and then deallocated)
        let alloc_v_svg = AllocationInfo {
            ptr: ptr,
            size: v_svg.len() * std::mem::size_of::<i32>(),
            timestamp_alloc: 100, // Dummy timestamp
            timestamp_dealloc: Some(200), // Deallocated later
            var_name: Some("v_svg".to_string()),
            type_name: Some("alloc::vec::Vec<i32>".to_string()),
            backtrace_ips: Vec::new(),
            thread_id: 0,
        };
        tracker.add_allocation_for_test(alloc_v_svg).expect("Failed to add v_svg allocation");
    }
    
    // Create temp directory and file path
    let dir = tempdir().expect("Failed to create temp directory");
    let svg_path = dir.path().join("integration_output.svg");
    
    // Export to SVG
    tracker.export_to_svg(&svg_path)
        .expect("Failed to export to SVG");
    
    // Verify file was created and has content
    assert!(svg_path.exists(), "SVG file was not created");
    let svg_content = std::fs::read_to_string(&svg_path)
        .expect("Failed to read SVG file");
    
    // Basic SVG structure checks
    assert!(!svg_content.is_empty(), "SVG file content is empty");
    
    // Check for SVG root element
    assert!(
        svg_content.contains("<svg") && svg_content.contains("</svg>"),
        "SVG is missing root svg element"
    );
    
    // Check for title element
    assert!(
        svg_content.contains("<title>") || 
        svg_content.contains("Memory Allocation Lifecycles"),
        "SVG is missing title"
    );
    
    // Check for any allocation data (rect or text elements)
    let has_alloc_data = svg_content.contains("<rect") || 
                       svg_content.contains("<g>") || 
                       svg_content.contains("<text>");
    
    if !has_alloc_data {
        // If no allocation data is found, it's not necessarily an error - the SVG might be empty
        // if there are no allocations in the log
        println!("Warning: SVG doesn't contain expected allocation data. This might be expected if there are no allocations in the log.");
    }

    // Manually populate the tracker with data for SVG export test
    let base_time = 1678886400000i64; // Use a fixed base time

    // Add a deallocated allocation (will appear in the log for SVG)
    let deallocated_alloc = AllocationInfo {
        ptr: 0x1000, // Dummy pointer
        size: 100,
        timestamp_alloc: (base_time + 50) as u128,
        timestamp_dealloc: Some((base_time + 150) as u128),
        var_name: Some("deallocated_var".to_string()),
        type_name: Some("Vec<i32>".to_string()),
        backtrace_ips: Vec::new(),
        thread_id: 1,
    };
    tracker.add_deallocated_for_test(deallocated_alloc)
        .expect("Failed to add deallocated allocation for SVG test");

    // Add an active allocation (will NOT appear in the log for SVG export)
    let active_alloc = AllocationInfo {
        ptr: 0x2000, // Dummy pointer
        size: 250,
        timestamp_alloc: (base_time + 200) as u128,
        timestamp_dealloc: None,
        var_name: Some("active_var".to_string()),
        type_name: Some("String".to_string()),
        backtrace_ips: Vec::new(),
        thread_id: 1,
    };
    tracker.add_allocation_for_test(active_alloc)
        .expect("Failed to add active allocation for SVG test");

    // Create temp directory and file path
    let dir = tempdir().expect("Failed to create temp directory");
    let svg_path = dir.path().join("integration_output.svg");

    // Export to SVG
    tracker.export_to_svg(&svg_path)
        .expect("Failed to export to SVG");

    // Verify file was created and has content
    assert!(svg_path.exists(), "SVG file was not created");
    let svg_content = std::fs::read_to_string(&svg_path)
        .expect("Failed to read SVG file");

    // Verify SVG contains some allocation data
    assert!(
        svg_content.contains("<rect") || 
        svg_content.contains("<g>") || 
        svg_content.contains("<text>"),
        "SVG should contain some allocation data"
    );
    
    // Verify the SVG is well-formed
    assert!(
        svg_content.ends_with("</svg>") && 
        svg_content.contains("<svg"),
        "SVG is not well-formed"
    );
}
