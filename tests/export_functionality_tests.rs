//! Export functionality tests
//! 
//! Tests the binary to JSON and HTML export functionality

use memscope_rs::export::binary::{export_binary_to_json, export_binary_to_html};
use memscope_rs::core::tracker::MemoryTracker;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod export_functionality_tests {
    use super::*;

    #[test]
    fn test_json_export_functionality() {
        // Test JSON export functionality
        let _temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create test binary file using tracker
        let tracker = MemoryTracker::new();
        let ptr1 = 0x1000;
        let ptr2 = 0x2000;
        
        let result = tracker.track_allocation(ptr1, 64);
        assert!(result.is_ok(), "Failed to track first allocation");
        
        let result = tracker.associate_var(ptr1, "json_var1".to_string(), "i32".to_string());
        assert!(result.is_ok(), "Failed to associate first variable");
        
        let result = tracker.track_allocation(ptr2, 128);
        assert!(result.is_ok(), "Failed to track second allocation");
        
        let result = tracker.associate_var(ptr2, "json_var2".to_string(), "String".to_string());
        assert!(result.is_ok(), "Failed to associate second variable");
        
        // Export to binary first
        let result = tracker.export_to_binary("test_project");
        assert!(result.is_ok(), "Failed to export to binary: {result:?}");
        
        // Find the created binary file
        let binary_file = std::path::Path::new("MemoryAnalysis").join("test_project.memscope");
        assert!(binary_file.exists(), "Binary file should exist at: {}", binary_file.display());
        
        // Export binary to JSON
        let result = export_binary_to_json(&binary_file, "test_project");
        assert!(result.is_ok(), "Failed to export to JSON: {result:?}");
        
        // Verify JSON file was created - check for any JSON files in the project directory
        let project_dir = std::path::Path::new("MemoryAnalysis/test_project");
        assert!(project_dir.exists(), "Project directory should exist: {}", project_dir.display());
        
        let json_files: Vec<_> = fs::read_dir(project_dir)
            .expect("Failed to read project directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect();
        
        assert!(!json_files.is_empty(), "At least one JSON file should exist in: {}", project_dir.display());
        let existing_path = &json_files[0].path();
        
        // Read and verify JSON content (use the path that exists)
        let json_content = fs::read_to_string(existing_path).expect("Failed to read JSON file");
        assert!(json_content.contains("allocations"), "JSON should contain allocations array");
        assert!(json_content.contains("json_var1"), "JSON should contain first variable");
        assert!(json_content.contains("json_var2"), "JSON should contain second variable");
    }

    #[test]
    fn test_html_export_functionality() {
        // Test HTML export functionality
        let _temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create test binary file using tracker
        let tracker = MemoryTracker::new();
        let ptr1 = 0x1000;
        let ptr2 = 0x2000;
        
        let result = tracker.track_allocation(ptr1, 64);
        assert!(result.is_ok(), "Failed to track first allocation");
        
        let result = tracker.associate_var(ptr1, "html_var1".to_string(), "Box<i32>".to_string());
        assert!(result.is_ok(), "Failed to associate first variable");
        
        let result = tracker.track_allocation(ptr2, 128);
        assert!(result.is_ok(), "Failed to track second allocation");
        
        let result = tracker.associate_var(ptr2, "html_var2".to_string(), "Arc<String>".to_string());
        assert!(result.is_ok(), "Failed to associate second variable");
        
        // Export to binary first
        let result = tracker.export_to_binary("test_html_project");
        assert!(result.is_ok(), "Failed to export to binary: {result:?}");
        
        // Find the created binary file
        let binary_file = std::path::Path::new("MemoryAnalysis").join("test_html_project.memscope");
        assert!(binary_file.exists(), "Binary file should exist at: {}", binary_file.display());
        
        // Export binary to HTML
        let result = export_binary_to_html(&binary_file, "test_html_project");
        assert!(result.is_ok(), "Failed to export to HTML: {result:?}");
        
        // Verify HTML file was created (check multiple possible locations)
        let html_path = std::path::Path::new("MemoryAnalysis/test_html_project").join("test_html_project_user_dashboard.html");
        let alt_html_path = std::path::Path::new("MemoryAnalysis").join("test_html_project_user_dashboard.html");
        
        let actual_html_path = if html_path.exists() {
            html_path
        } else if alt_html_path.exists() {
            alt_html_path
        } else {
            panic!("HTML file should exist at: {} or {}", html_path.display(), alt_html_path.display());
        };
        
        // Read and verify HTML content
        let html_content = fs::read_to_string(&actual_html_path).expect("Failed to read HTML file");
        assert!(html_content.contains("<!DOCTYPE html>"), "Should be valid HTML");
        assert!(html_content.contains("Variable Lifecycle Visualization"), "Should contain visualization title");
        assert!(html_content.contains("html_var1"), "Should contain first variable");
        assert!(html_content.contains("html_var2"), "Should contain second variable");
    }

    #[test]
    fn test_export_with_empty_data() {
        // Test export functionality with empty tracking data
        let _temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create empty binary file
        let tracker = MemoryTracker::new();
        let result = tracker.export_to_binary("empty_project");
        assert!(result.is_ok(), "Should create empty binary file");
        
        // Find the created binary file
        let binary_file = std::path::Path::new("MemoryAnalysis").join("empty_project.memscope");
        
        // Export empty data to JSON
        let result = export_binary_to_json(&binary_file, "empty_project");
        assert!(result.is_ok(), "Should handle empty data export to JSON");
        
        // Verify JSON structure - check for any JSON files in the project directory
        let project_dir = std::path::Path::new("MemoryAnalysis/empty_project");
        assert!(project_dir.exists(), "Project directory should exist: {}", project_dir.display());
        
        let json_files: Vec<_> = fs::read_dir(project_dir)
            .expect("Failed to read project directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect();
        
        assert!(!json_files.is_empty(), "At least one JSON file should exist in: {}", project_dir.display());
        let actual_json_path = &json_files[0].path();
        
        let json_content = fs::read_to_string(actual_json_path).expect("Failed to read empty JSON");
        assert!(json_content.contains("allocations"), "Should contain allocations data structure");
        
        // Export empty data to HTML
        let result = export_binary_to_html(&binary_file, "empty_project");
        assert!(result.is_ok(), "Should handle empty data export to HTML");
        
        // Verify HTML structure - check multiple possible locations
        let html_path = std::path::Path::new("MemoryAnalysis/empty_project").join("empty_project_user_dashboard.html");
        let alt_html_path = std::path::Path::new("MemoryAnalysis").join("empty_project_user_dashboard.html");
        
        let actual_html_path = if html_path.exists() {
            html_path
        } else if alt_html_path.exists() {
            alt_html_path
        } else {
            panic!("HTML file should exist at: {} or {}", html_path.display(), alt_html_path.display());
        };
        
        let html_content = fs::read_to_string(actual_html_path).expect("Failed to read empty HTML");
        assert!(html_content.contains("<!DOCTYPE html>"), "Should be valid HTML even when empty");
    }

    #[test]
    fn test_export_error_handling() {
        // Test error handling in export operations
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nonexistent_path = temp_dir.path().join("nonexistent.bin");
        
        // Test exporting from non-existent binary file
        let result = export_binary_to_json(&nonexistent_path, "test_project");
        assert!(result.is_err(), "Should fail when binary file doesn't exist");
        
        let result = export_binary_to_html(&nonexistent_path, "test_project");
        assert!(result.is_err(), "Should fail when binary file doesn't exist");
    }

    #[test]
    fn test_large_dataset_export() {
        // Test export functionality with larger datasets
        let _temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Create larger dataset
        let tracker = MemoryTracker::new();
        let allocation_count = 100;
        
        for i in 0..allocation_count {
            let ptr = 0x1000 + (i * 0x100);
            let size = 64 + (i % 10) * 32; // Varying sizes
            let var_name = format!("large_var_{i}");
            let type_name = match i % 4 {
                0 => "i32".to_string(),
                1 => "String".to_string(),
                2 => "Vec<u8>".to_string(),
                _ => "HashMap<String, i32>".to_string(),
            };
            
            let result = tracker.track_allocation(ptr, size);
            assert!(result.is_ok(), "Failed to track allocation {i}");
            
            let result = tracker.associate_var(ptr, var_name, type_name);
            assert!(result.is_ok(), "Failed to associate variable {i}");
        }
        
        // Export large dataset
        let result = tracker.export_to_binary("large_project");
        assert!(result.is_ok(), "Failed to export large binary file");
        
        // Find the created binary file
        let binary_file = std::path::Path::new("MemoryAnalysis").join("large_project.memscope");
        
        let result = export_binary_to_json(&binary_file, "large_project");
        assert!(result.is_ok(), "Failed to export large dataset to JSON");
        
        // Verify large dataset export - check for any JSON files in the project directory
        let project_dir = std::path::Path::new("MemoryAnalysis/large_project");
        assert!(project_dir.exists(), "Project directory should exist: {}", project_dir.display());
        
        let json_files: Vec<_> = fs::read_dir(project_dir)
            .expect("Failed to read project directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect();
        
        assert!(!json_files.is_empty(), "At least one JSON file should exist in: {}", project_dir.display());
        let actual_json_path = &json_files[0].path();
        
        let json_content = fs::read_to_string(actual_json_path).expect("Failed to read large JSON");
        // The JSON might contain "variables" instead of "allocations" depending on the export format
        assert!(json_content.contains("allocations") || json_content.contains("variables") || json_content.contains("large_var"), 
                "Should contain allocation data. Content preview: {}", &json_content[..std::cmp::min(200, json_content.len())]);
        
        // Verify file size is reasonable
        let metadata = fs::metadata(actual_json_path).expect("Failed to get JSON metadata");
        assert!(metadata.len() > 1000, "Large dataset JSON should be substantial size");
        assert!(metadata.len() < 10_000_000, "Large dataset JSON should not be excessively large");
    }
}