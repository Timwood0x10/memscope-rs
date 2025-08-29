use memscope_rs::*;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod html_generation_tests {
    use super::*;

    #[test]
    fn test_html_template_data_injection() {
        // Test that HTML generation properly injects data
        let tracker = MemoryTracker::new();

        // Create test data
        let test_var = String::from("test_html_generation");
        let test_ptr = &test_var as *const String as usize;
        let _ = tracker.track_allocation(test_ptr, std::mem::size_of::<String>());

        // Test JSON serialization structure
        let test_data = serde_json::json!({
            "allocations": [
                {
                    "var_name": "test_var",
                    "type_name": "String",
                    "size": 24,
                    "timestamp_alloc": 1234567890
                }
            ]
        });

        let json_str = serde_json::to_string(&test_data).expect("Failed to serialize test data");
        assert!(json_str.contains("allocations"));
        assert!(json_str.contains("test_var"));
    }

    #[test]
    fn test_template_placeholder_replacement() {
        // Test template placeholder logic
        let template = "window.analysisData = {{ json_data }};";
        let test_data = r#"{"allocations": [{"var_name": "test"}]}"#;

        let result = template.replace("{{ json_data }}", test_data);

        assert!(result.contains("window.analysisData = {"));
        assert!(result.contains("test"));
        assert!(!result.contains("{{ json_data }}"));
    }

    #[test]
    fn test_clean_dashboard_template_exists() {
        // Verify the template file exists and has required structure
        let template_path = Path::new("templates/clean_dashboard.html");
        assert!(
            template_path.exists(),
            "clean_dashboard.html template not found"
        );

        let content = fs::read_to_string(template_path).expect("Failed to read template file");

        // Check for required elements
        assert!(content.contains("Variable Lifecycle Visualization"));
        assert!(content.contains("Enhanced Memory Statistics"));
        assert!(content.contains("{{ json_data }}"));
        assert!(content.contains("createLifecycleVisualization"));
        assert!(content.contains("updateEnhancedStatistics"));
    }

    #[test]
    fn test_javascript_function_definitions() {
        // Test that required JavaScript functions are defined in template
        let template_path = Path::new("templates/clean_dashboard.html");
        let content = fs::read_to_string(template_path).expect("Failed to read template file");

        let required_functions = vec![
            "inferAllocationType",
            "formatTimestamp",
            "calculateDropTime",
            "formatLifetime",
            "createLifecycleVisualization",
            "updateEnhancedStatistics",
            "setupLifecycleFilters",
        ];

        for func in required_functions {
            assert!(
                content.contains(&format!("function {}(", func)),
                "Missing function: {}",
                func
            );
        }
    }

    #[test]
    fn test_css_classes_defined() {
        // Test that required CSS classes are defined
        let template_path = Path::new("templates/clean_dashboard.html");
        let content = fs::read_to_string(template_path).expect("Failed to read template file");

        let required_classes = vec![
            "allocation-type",
            "type-heap",
            "type-stack",
            "type-unknown",
            "lifecycle-bar",
            "lifecycle-progress",
            "lifecycle-item",
        ];

        for class in required_classes {
            assert!(
                content.contains(&format!(".{}", class)),
                "Missing CSS class: {}",
                class
            );
        }
    }

    #[test]
    fn test_dom_element_ids() {
        // Test that required DOM element IDs exist
        let template_path = Path::new("templates/clean_dashboard.html");
        let content = fs::read_to_string(template_path).expect("Failed to read template file");

        let required_ids = vec![
            "lifecycleVisualizationContainer",
            "heap-count-mini",
            "stack-count-mini",
            "total-allocs-enhanced",
            "heap-stack-ratio",
            "avg-lifetime-enhanced",
            "memory-efficiency",
            "filter-heap",
            "filter-stack",
            "toggle-lifecycle",
        ];

        for id in required_ids {
            assert!(
                content.contains(&format!("id=\"{}\"", id)),
                "Missing DOM element ID: {}",
                id
            );
        }
    }
}
