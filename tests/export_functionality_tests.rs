//! Export functionality tests
//!
//! Tests the binary to JSON and HTML export functionality

use memscope_rs::core::tracker::MemoryTracker;
use memscope_rs::export::binary::{export_binary_to_html, export_binary_to_json};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod export_functionality_tests {
    use super::*;

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

        let result =
            tracker.associate_var(ptr2, "html_var2".to_string(), "Arc<String>".to_string());
        assert!(result.is_ok(), "Failed to associate second variable");

        // Export to binary first
        let result = tracker.export_to_binary("test_html_project");
        assert!(result.is_ok(), "Failed to export to binary: {result:?}");

        // Find the created binary file
        let binary_file = std::path::Path::new("MemoryAnalysis").join("test_html_project.memscope");
        assert!(
            binary_file.exists(),
            "Binary file should exist at: {}",
            binary_file.display()
        );

        // Export binary to HTML
        let result = export_binary_to_html(&binary_file, "test_html_project");
        assert!(result.is_ok(), "Failed to export to HTML: {result:?}");

        // Verify HTML file was created (check multiple possible locations)
        let html_path = std::path::Path::new("MemoryAnalysis/test_html_project")
            .join("test_html_project_user_dashboard.html");
        let alt_html_path =
            std::path::Path::new("MemoryAnalysis").join("test_html_project_user_dashboard.html");

        let actual_html_path = if html_path.exists() {
            html_path
        } else if alt_html_path.exists() {
            alt_html_path
        } else {
            panic!(
                "HTML file should exist at: {} or {}",
                html_path.display(),
                alt_html_path.display()
            );
        };

        // Read and verify HTML content
        let html_content = fs::read_to_string(&actual_html_path).expect("Failed to read HTML file");
        assert!(
            html_content.contains("<!DOCTYPE html>"),
            "Should be valid HTML"
        );
        assert!(
            html_content.contains("Variable Lifecycle Visualization"),
            "Should contain visualization title"
        );
        assert!(
            html_content.contains("html_var1"),
            "Should contain first variable"
        );
        assert!(
            html_content.contains("html_var2"),
            "Should contain second variable"
        );
    }

    #[test]
    fn test_export_error_handling() {
        // Test error handling in export operations
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nonexistent_path = temp_dir.path().join("nonexistent.bin");

        // Test exporting from non-existent binary file
        let result = export_binary_to_json(&nonexistent_path, "test_project");
        assert!(
            result.is_err(),
            "Should fail when binary file doesn't exist"
        );

        let result = export_binary_to_html(&nonexistent_path, "test_project");
        assert!(
            result.is_err(),
            "Should fail when binary file doesn't exist"
        );
    }
}
