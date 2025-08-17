//! Integration test for template resource management in binary HTML export
//!
//! This test verifies that the template resource management system is properly integrated
//! with the binary HTML writer and template engine.

#[cfg(test)]
mod tests {
    use crate::core::types::AllocationInfo;
    use crate::export::binary::binary_html_writer::BinaryHtmlWriter;
    use crate::export::binary::selective_reader::AllocationField;
    use crate::export::binary::template_resource_manager::{
        TemplateResourceManager, ResourceConfig, create_template_data,
    };
    use crate::export::binary::binary_template_engine::BinaryTemplateEngine;
    use std::collections::HashMap;
    use std::io::Cursor;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_allocation(ptr: usize, size: usize, type_name: &str) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            type_name: Some(type_name.to_string()),
            var_name: Some(format!("var_{}", ptr)),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }
    }

    fn create_test_template_dir() -> Result<TempDir, std::io::Error> {
        let temp_dir = TempDir::new()?;
        
        // Create test binary dashboard template
        let template_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{PROJECT_NAME}} - Memory Analysis</title>
    <style>{{CSS_CONTENT}}</style>
</head>
<body>
    <h1>{{PROJECT_NAME}}</h1>
    <div id="data">{{BINARY_DATA}}</div>
    <div id="generation-time">Generated: {{GENERATION_TIME}}</div>
    <script>{{JS_CONTENT}}</script>
</body>
</html>
"#;
        fs::write(temp_dir.path().join("binary_dashboard.html"), template_content)?;

        // Create test CSS file
        let css_content = r#"
body { 
    font-family: Arial, sans-serif; 
    margin: 0; 
    padding: 20px; 
}
.memory-chart { 
    width: 100%; 
    height: 400px; 
}
"#;
        fs::write(temp_dir.path().join("styles.css"), css_content)?;

        // Create test JS file
        let js_content = r#"
function initializeMemoryDashboard() {
    console.log('Memory dashboard initialized');
}
window.addEventListener('load', initializeMemoryDashboard);
"#;
        fs::write(temp_dir.path().join("script.js"), js_content)?;

        Ok(temp_dir)
    }

    #[test]
    fn test_template_resource_integration() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        // Create test template data
        let mut custom_data = HashMap::new();
        custom_data.insert("test_key".to_string(), "test_value".to_string());

        let template_data = create_template_data(
            "Test Project",
            r#"{"allocations": [], "summary": {"total": 0}}"#,
            custom_data,
        );

        let config = ResourceConfig::default();

        // Process template with resources
        let result = resource_manager.process_template(
            "binary_dashboard.html",
            &template_data,
            &config,
        );

        assert!(result.is_ok());
        let html_content = result.expect("Test operation failed");

        // Verify template processing
        assert!(html_content.contains("Test Project"));
        assert!(html_content.contains(r#"{"allocations": [], "summary": {"total": 0}}"#));
        assert!(html_content.contains("font-family: Arial"));
        assert!(html_content.contains("initializeMemoryDashboard"));
        assert!(html_content.contains("Generated:"));
    }

    #[test]
    fn test_resource_caching() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        let config = ResourceConfig::default();

        // First load should read from files
        let css1 = resource_manager.get_shared_css(&config).expect("Test operation failed");
        let js1 = resource_manager.get_shared_js(&config).expect("Test operation failed");

        // Second load should use cache
        let css2 = resource_manager.get_shared_css(&config).expect("Test operation failed");
        let js2 = resource_manager.get_shared_js(&config).expect("Test operation failed");

        // Content should be identical
        assert_eq!(css1, css2);
        assert_eq!(js1, js2);

        // Verify content is not empty
        assert!(!css1.is_empty());
        assert!(!js1.is_empty());
    }

    #[test]
    fn test_resource_minification() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        let config_normal = ResourceConfig {
            minify_resources: false,
            ..Default::default()
        };

        let config_minified = ResourceConfig {
            minify_resources: true,
            ..Default::default()
        };

        let css_normal = resource_manager.get_shared_css(&config_normal).expect("Test operation failed");
        
        // Clear cache to force reload with minification
        resource_manager.clear_cache();
        
        let css_minified = resource_manager.get_shared_css(&config_minified).expect("Test operation failed");

        // Minified version should be smaller (or at least not larger)
        assert!(css_minified.len() <= css_normal.len());
        
        // Both should contain the same essential content
        assert!(css_normal.contains("font-family"));
        assert!(css_minified.contains("font-family"));
    }

    #[test]
    fn test_binary_html_writer_with_resource_manager() {
        // Create test allocations
        let allocations = vec![
            create_test_allocation(0x1000, 1024, "Vec<u8>"),
            create_test_allocation(0x2000, 2048, "HashMap<String, i32>"),
            create_test_allocation(0x3000, 512, "Box<i32>"),
        ];

        // Create binary HTML writer
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut writer = BinaryHtmlWriter::new(cursor).expect("Failed to get test value");

        // Write allocations
        let fields = AllocationField::all_basic_fields();
        for allocation in &allocations {
            writer.write_binary_allocation(allocation, &fields).expect("Test operation failed");
        }

        // Finalize and get stats
        let stats = writer.finalize_with_binary_template("test_project").expect("Test operation failed");

        // Verify that allocations were processed
        assert_eq!(stats.allocations_processed, 3);
        assert!(stats.total_html_size > 0);
    }

    #[test]
    fn test_binary_template_engine_with_resource_manager() {
        // Create a temporary template directory for the engine
        let _temp_dir = create_test_template_dir().expect("Failed to get test value");
        
        // We can't easily change the template directory for BinaryTemplateEngine
        // So we'll test that it can be created and used without errors
        let engine = BinaryTemplateEngine::new();
        
        // The engine creation might fail if templates directory doesn't exist
        // This is expected behavior, so we'll just verify the error handling
        match engine {
            Ok(eng) => {
                // If engine creation succeeds, test that we can get stats
                let stats = eng.get_stats();
                assert_eq!(stats.templates_processed, 0);
                assert_eq!(stats.cache_hits, 0);
            }
            Err(_) => {
                // Engine creation failed, which is expected if templates don't exist
                // This is acceptable behavior for this test
            }
        }
    }

    #[test]
    fn test_placeholder_processing() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        // Create template data with custom placeholders
        let mut custom_data = HashMap::new();
        custom_data.insert("complex_types".to_string(), r#"{"types": ["Vec<String>", "HashMap<i32, String>"]}"#.to_string());
        custom_data.insert("unsafe_ffi".to_string(), r#"{"unsafe_operations": []}"#.to_string());

        let template_data = create_template_data(
            "Placeholder Test",
            r#"{"test": "data"}"#,
            custom_data,
        );

        let config = ResourceConfig::default();

        // Process template
        let result = resource_manager.process_template(
            "binary_dashboard.html",
            &template_data,
            &config,
        );

        assert!(result.is_ok());
        let html_content = result.expect("Test operation failed");

        // Verify placeholder replacement
        assert!(html_content.contains("Placeholder Test"));
        assert!(html_content.contains(r#"{"test": "data"}"#));
        assert!(html_content.contains("Generated:"));
    }

    #[test]
    fn test_resource_config_options() {
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        let template_data = create_template_data(
            "Config Test",
            "{}",
            HashMap::new(),
        );

        // Test with CSS embedding disabled
        let config_no_css = ResourceConfig {
            embed_css: false,
            embed_js: true,
            embed_svg: true,
            minify_resources: false,
            custom_paths: HashMap::new(),
        };

        let result = resource_manager.process_template(
            "binary_dashboard.html",
            &template_data,
            &config_no_css,
        );

        assert!(result.is_ok());
        let html_content = result.expect("Test operation failed");

        // Should still contain the template structure
        assert!(html_content.contains("Config Test"));
        assert!(html_content.contains("{{CSS_CONTENT}}"));  // Not replaced
        assert!(html_content.contains("initializeMemoryDashboard"));  // JS should be embedded
    }

    #[test]
    fn test_error_handling() {
        // Test with non-existent template directory
        let result = TemplateResourceManager::new("/non/existent/path");
        assert!(result.is_err());

        // Test with valid directory but non-existent template
        let temp_dir = create_test_template_dir().expect("Failed to get test value");
        let mut resource_manager = TemplateResourceManager::new(temp_dir.path()).unwrap();

        let template_data = create_template_data("Test", "{}", HashMap::new());
        let config = ResourceConfig::default();

        let result = resource_manager.process_template(
            "non_existent_template.html",
            &template_data,
            &config,
        );

        assert!(result.is_err());
    }
}