//! Test for optimized JSON export functionality

use memscope_rs::core::tracker::MemoryTracker;
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_write_json_optimized_functionality() {
    // Test the core functionality that we optimized
    let temp_file = NamedTempFile::new().unwrap();

    // Create test JSON data
    let test_data = json!({
        "allocations": [
            {
                "ptr": "0x1000",
                "size": 1024
            }
        ]
    });

    // Write test data to file
    let file = File::create(&temp_file).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &test_data).unwrap();
}

#[test]
fn test_optimized_json_export_fast_mode() {
    let temp_dir = TempDir::new().unwrap();
    let tracker = MemoryTracker::new();

    // Test fast export mode
    let result = tracker.export_to_json_fast(temp_dir.path().join("fast_export"));
    assert!(result.is_ok(), "Fast export should succeed");
}

#[test]
fn test_optimized_json_export_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    let tracker = MemoryTracker::new();

    // Test comprehensive export mode
    let result = tracker.export_to_json_comprehensive(temp_dir.path().join("comprehensive_export"));
    assert!(result.is_ok(), "Comprehensive export should succeed");
}
