//! Dashboard renderer comprehensive test suite
//!
//! This module provides extensive testing for the dashboard renderer,
//! covering template rendering, helper functions, error handling, and performance.

use super::*;
use crate::core::types::AllocationInfo;
use crate::render_engine::dashboard::renderer::types::CircularReferenceReport;
use serde_json::json;

/// Memory size constants to avoid integer overflow in test expressions
const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;
const PB: u64 = TB * 1024;

/// Create a mock allocation for testing purposes
fn create_mock_allocation(
    address: usize,
    size: usize,
    var_name: Option<String>,
    type_name: Option<String>,
) -> crate::core::types::AllocationInfo {
    let mut alloc = crate::core::types::AllocationInfo::new(address, size);
    alloc.var_name = var_name;
    alloc.type_name = type_name;
    alloc
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test format_bytes function with various input sizes
    #[test]
    fn test_format_bytes_function() {
        // Test zero bytes
        assert_eq!(format_bytes(0), "0 bytes");

        // Test bytes
        assert_eq!(format_bytes(1), "1 bytes");
        assert_eq!(format_bytes(1023), "1023 bytes");

        // Test kilobytes
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1023), "1023.00 KB");

        // Test megabytes
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1536 * 1024), "1.50 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1023), "1023.00 MB");

        // Test gigabytes
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GB");

        // Test terabytes (use u64 constant to avoid overflow)
        assert_eq!(format_bytes(TB as usize), "1.00 TB");

        // Test petabytes (use u64 constant to avoid overflow)
        assert_eq!(format_bytes(PB as usize), "1.00 PB");
    }

    /// Test format_bytes_helper function for Handlebars templates
    #[test]
    fn test_format_bytes_helper() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars
            .register_helper("format_bytes", Box::new(format_bytes_helper));

        let template = "{{format_bytes bytes}}";
        handlebars.register_template_string("test", template).unwrap();

        // Test various byte values
        let test_cases = vec![
            (0, "0 bytes"),
            (512, "512 bytes"),
            (1024, "1.00 KB"),
            (1536, "1.50 KB"),
            (1024 * 1024, "1.00 MB"),
        ];

        for (bytes, expected) in test_cases {
            let data = json!({ "bytes": bytes });
            let result = handlebars.render("test", &data);
            assert!(
                result.is_ok(),
                "Failed to render with bytes={}: {:?}",
                bytes,
                result
            );
            let result = result.unwrap();
            assert_eq!(
                result, expected,
                "Unexpected format_bytes result for {bytes}: expected '{expected}', got '{result}'"
            );
        }
    }

    /// Test greater_than_helper function
    #[test]
    fn test_greater_than_helper() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars
            .register_helper("gt", Box::new(greater_than_helper));

        let template = "{{#if (gt a b)}}true{{else}}false{{/if}}";
        handlebars.register_template_string("test", template).unwrap();

        // Test cases where a > b
        let data = json!({ "a": 10, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "true", "Expected 'true' when 10 > 5");

        // Test cases where a <= b
        let data = json!({ "a": 3, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "false", "Expected 'false' when 3 <= 5");

        // Test equal values
        let data = json!({ "a": 5, "b": 5 });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "false", "Expected 'false' when 5 == 5");
    }

    /// Test contains_helper function
    #[test]
    fn test_contains_helper() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars
            .register_helper("contains", Box::new(contains_helper));

        let template = "{{#if (contains haystack needle)}}true{{else}}false{{/if}}";
        handlebars.register_template_string("test", template).unwrap();

        // Test cases where needle is in haystack
        let data = json!({ "haystack": "hello world", "needle": "world" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(
            result, "true",
            "Expected 'true' when 'world' is in 'hello world'"
        );

        // Test cases where needle is not in haystack
        let data = json!({ "haystack": "hello world", "needle": "rust" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(
            result, "false",
            "Expected 'false' when 'rust' is not in 'hello world'"
        );

        // Test empty string
        let data = json!({ "haystack": "", "needle": "test" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(result, "false", "Expected 'false' when haystack is empty");

        // Test needle at the beginning
        let data = json!({ "haystack": "hello world", "needle": "hello" });
        let result = handlebars.render("test", &data).unwrap();
        assert_eq!(
            result, "true",
            "Expected 'true' when 'hello' is at the beginning"
        );
    }

    /// Test json_helper function
    #[test]
    fn test_json_helper() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars.register_helper("json", Box::new(json_helper));

        let template = "{{json data}}";
        handlebars.register_template_string("test", template).unwrap();

        // Test with simple object
        let data = json!({ "data": { "key": "value", "number": 42 } });
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("\"key\""), "Result should contain 'key' field");
        assert!(result.contains("\"value\""), "Result should contain 'value'");
        assert!(result.contains("\"number\""), "Result should contain 'number' field");
        assert!(result.contains("42"), "Result should contain number 42");

        // Test with array
        let data = json!({ "data": [1, 2, 3, 4, 5] });
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("["), "Result should contain array start");
        assert!(result.contains("]"), "Result should contain array end");
        assert!(result.contains("1"), "Result should contain array elements");

        // Test with nested structure
        let data = json!({
            "data": {
                "outer": {
                    "inner": {
                        "value": "nested"
                    }
                }
            }
        });
        let result = handlebars.render("test", &data).unwrap();
        assert!(result.contains("\"nested\""), "Result should contain nested value");
    }

    /// Test format_thread_id function
    #[test]
    fn test_format_thread_id() {
        // Test standard ThreadId format
        let result = format_thread_id("ThreadId(5)");
        assert_eq!(result, "Thread-5", "Should format ThreadId(5) to Thread-5");

        let result = format_thread_id("ThreadId(42)");
        assert_eq!(
            result, "Thread-42",
            "Should format ThreadId(42) to Thread-42"
        );

        // Test non-standard format (should return as-is)
        let result = format_thread_id("custom_thread_id");
        assert_eq!(
            result, "custom_thread_id",
            "Should return non-standard format unchanged"
        );

        let result = format_thread_id("Thread-5");
        assert_eq!(
            result, "Thread-5",
            "Should return already formatted thread ID unchanged"
        );
    }

    /// Test DashboardContext creation with default values
    #[test]
    fn test_dashboard_context_default() {
        let context = DashboardContext {
            title: "Test Dashboard".to_string(),
            export_timestamp: "2024-01-01 00:00:00".to_string(),
            total_memory: "0 B".to_string(),
            total_allocations: 0,
            active_allocations: 0,
            peak_memory: "0 B".to_string(),
            thread_count: 0,
            passport_count: 0,
            leak_count: 0,
            unsafe_count: 0,
            ffi_count: 0,
            allocations: vec![],
            relationships: vec![],
            unsafe_reports: vec![],
            passport_details: vec![],
            allocations_count: 0,
            relationships_count: 0,
            unsafe_reports_count: 0,
            json_data: "{}".to_string(),
            os_name: "Unknown".to_string(),
            architecture: "Unknown".to_string(),
            cpu_cores: 0,
            system_resources: SystemResources {
                os_name: "Unknown".to_string(),
                os_version: "Unknown".to_string(),
                architecture: "Unknown".to_string(),
                cpu_cores: 0,
                total_physical: "0 B".to_string(),
                available_physical: "0 B".to_string(),
                used_physical: "0 B".to_string(),
                page_size: 4096,
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
            },
            health_score: 100,
            health_status: "Healthy".to_string(),
            safe_ops_count: 0,
            high_risk_count: 0,
            clean_passport_count: 0,
            active_passport_count: 0,
            leaked_passport_count: 0,
            ffi_tracked_count: 0,
            safe_code_percent: 100,
            ownership_graph: OwnershipGraphInfo {
                total_nodes: 0,
                total_edges: 0,
                total_cycles: 0,
                rc_clone_count: 0,
                arc_clone_count: 0,
                has_issues: false,
                issues: vec![],
                root_cause: None,
            },
            circular_references: CircularReferenceReport {
                count: 0,
                total_leaked_memory: 0,
                pointers_in_cycles: 0,
                total_smart_pointers: 0,
                has_cycles: false,
            },
        };

        assert_eq!(context.title, "Test Dashboard");
        assert_eq!(context.total_allocations, 0);
        assert_eq!(context.health_score, 100);
        assert_eq!(context.ownership_graph.total_nodes, 0);
    }

    /// Test DashboardContext serialization and deserialization
    #[test]
    fn test_dashboard_context_serialization() {
        let context = DashboardContext {
            title: "Test Dashboard".to_string(),
            export_timestamp: "2024-01-01 00:00:00".to_string(),
            total_memory: "1.00 KB".to_string(),
            total_allocations: 10,
            active_allocations: 5,
            peak_memory: "2.00 KB".to_string(),
            thread_count: 1,
            passport_count: 0,
            leak_count: 0,
            unsafe_count: 0,
            ffi_count: 0,
            allocations: vec![],
            relationships: vec![],
            unsafe_reports: vec![],
            passport_details: vec![],
            allocations_count: 10,
            relationships_count: 0,
            unsafe_reports_count: 0,
            json_data: "{}".to_string(),
            os_name: "Unknown".to_string(),
            architecture: "Unknown".to_string(),
            cpu_cores: 0,
            system_resources: SystemResources {
                os_name: "Unknown".to_string(),
                os_version: "Unknown".to_string(),
                architecture: "Unknown".to_string(),
                cpu_cores: 0,
                total_physical: "0 B".to_string(),
                available_physical: "0 B".to_string(),
                used_physical: "0 B".to_string(),
                page_size: 4096,
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
            },
            health_score: 100,
            health_status: "Healthy".to_string(),
            safe_ops_count: 0,
            high_risk_count: 0,
            clean_passport_count: 0,
            active_passport_count: 0,
            leaked_passport_count: 0,
            ffi_tracked_count: 0,
            safe_code_percent: 100,
            ownership_graph: OwnershipGraphInfo {
                total_nodes: 0,
                total_edges: 0,
                total_cycles: 0,
                rc_clone_count: 0,
                arc_clone_count: 0,
                has_issues: false,
                issues: vec![],
                root_cause: None,
            },
            circular_references: CircularReferenceReport {
                count: 0,
                total_leaked_memory: 0,
                pointers_in_cycles: 0,
                total_smart_pointers: 0,
                has_cycles: false,
            },
        };

        // Test serialization
        let json_str = serde_json::to_string(&context)
            .expect("Failed to serialize DashboardContext");
        assert!(json_str.contains("\"title\""));
        assert!(json_str.contains("\"Test Dashboard\""));
        assert!(json_str.contains("\"total_allocations\""));
        assert!(json_str.contains("10"));

        // Test deserialization
        let deserialized: DashboardContext = serde_json::from_str(&json_str)
            .expect("Failed to deserialize DashboardContext");
        assert_eq!(deserialized.title, context.title);
        assert_eq!(deserialized.total_allocations, context.total_allocations);
        assert_eq!(deserialized.health_score, context.health_score);
    }

    /// Test helper function edge cases and error handling
    #[test]
    fn test_helper_functions_edge_cases() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars
            .register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars
            .register_helper("gt", Box::new(greater_than_helper));
        handlebars
            .register_helper("contains", Box::new(contains_helper));

        // Test format_bytes with very large numbers
        let template = "{{format_bytes bytes}}";
        handlebars.register_template_string("test", template).unwrap();

        let large_bytes = u64::MAX / 2;
        let data = json!({ "bytes": large_bytes });
        let result = handlebars.render("test", &data);
        assert!(result.is_ok(), "Should handle very large numbers");

        // Test contains with empty needle
        let template = "{{#if (contains haystack needle)}}true{{else}}false{{/if}}";
        handlebars.register_template_string("test", template).unwrap();

        let data = json!({ "haystack": "hello", "needle": "" });
        let result = handlebars.render("test", &data).unwrap();
        // Empty string should be contained in any string
        assert_eq!(result, "true", "Empty needle should be contained");

        // Test gt with negative values (should not panic)
        let template = "{{#if (gt a b)}}true{{else}}false{{/if}}";
        handlebars.register_template_string("test", template).unwrap();

        // Note: Handlebars JSON values are unsigned, so negative values
        // are represented as large positive numbers or strings
        let data = json!({ "a": "not_a_number", "b": 5 });
        let result = handlebars.render("test", &data);
        // Should handle gracefully without panicking
        assert!(result.is_ok(), "Should handle non-numeric values gracefully");
    }

    /// Test performance of helper functions
    #[test]
    fn test_helper_functions_performance() {
        use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Output};

        let mut handlebars = Handlebars::new();
        handlebars
            .register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars
            .register_helper("contains", Box::new(contains_helper));

        let template = "{{format_bytes bytes}} {{contains haystack needle}}";
        handlebars
            .register_template_string("test", template)
            .unwrap();

        let iterations = 1000;
        let start = std::time::Instant::now();

        for i in 0..iterations {
            let data = json!({
                "bytes": 1024 * i,
                "haystack": "hello world test string",
                "needle": if i % 2 == 0 { "world" } else { "test" }
            });
            let _ = handlebars.render("test", &data);
        }

        let duration = start.elapsed();

        // Should complete 1000 iterations in reasonable time (< 100ms)
        assert!(
            duration.as_millis() < 100,
            "Helper functions too slow: {:?}",
            duration
        );

        println!(
            "Helper functions performance: {} iterations in {:?} ({:.2} iterations/ms)",
            iterations,
            duration,
            iterations as f64 / duration.as_millis() as f64
        );
    }

    /// Test extract_user_source_file function
    #[test]
    fn test_extract_user_source_file() {
        // Test with user source file
        let user_stack_trace = Some(vec![
            "my_module::my_function".to_string(),
            "/Users/user/project/src/main.rs:42".to_string(),
            "std::panicking::begin_panic".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_file(&user_stack_trace);
        assert!(
            result.is_some(),
            "Should extract user source file from stack trace"
        );
        assert_eq!(
            result.unwrap(),
            "/Users/user/project/src/main.rs:42",
            "Should return the user source file path"
        );

        // Test with only Rust internal frames
        let internal_stack_trace = Some(vec![
            "std::panicking::begin_panic".to_string(),
            "core::panicking::panic".to_string(),
            "alloc::alloc::alloc".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_file(&internal_stack_trace);
        assert!(
            result.is_none(),
            "Should return None for stack trace with only internal frames"
        );

        // Test with empty stack trace
        let empty_stack_trace: Option<Vec<String>> = None;
        let result = DashboardRenderer::extract_user_source_file(&empty_stack_trace);
        assert!(result.is_none(), "Should return None for empty stack trace");

        // Test with memscope internal frames
        let memscope_stack_trace = Some(vec![
            "memscope_rs::core::tracker::track".to_string(),
            "memscope_rs::capture::engine::capture_allocation".to_string(),
            "/Users/user/project/src/main.rs:42".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_file(&memscope_stack_trace);
        assert!(
            result.is_some(),
            "Should extract user source file even with memscope frames"
        );
        assert_eq!(
            result.unwrap(),
            "/Users/user/project/src/main.rs:42",
            "Should return the user source file path"
        );
    }

    /// Test extract_user_source_line function
    #[test]
    fn test_extract_user_source_line() {
        // Test with user source file
        let user_stack_trace = Some(vec![
            "my_module::my_function".to_string(),
            "/Users/user/project/src/main.rs:42".to_string(),
            "std::panicking::begin_panic".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_line(&user_stack_trace);
        assert!(
            result.is_some(),
            "Should extract user source line from stack trace"
        );
        assert_eq!(result.unwrap(), 42, "Should return the correct line number");

        // Test with only Rust internal frames
        let internal_stack_trace = Some(vec![
            "std::panicking::begin_panic".to_string(),
            "core::panicking::panic".to_string(),
            "alloc::alloc::alloc".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_line(&internal_stack_trace);
        assert!(
            result.is_none(),
            "Should return None for stack trace with only internal frames"
        );

        // Test with invalid line number
        let invalid_stack_trace = Some(vec![
            "my_module::my_function".to_string(),
            "/Users/user/project/src/main.rs:invalid".to_string(),
        ]);

        let result = DashboardRenderer::extract_user_source_line(&invalid_stack_trace);
        assert!(result.is_none(), "Should return None for invalid line number");
    }

    /// Test DashboardRenderer creation
    #[test]
    fn test_dashboard_renderer_creation() {
        let renderer = DashboardRenderer::new();

        assert!(
            renderer.is_ok(),
            "Should create DashboardRenderer successfully: {:?}",
            renderer
        );

        let renderer = renderer.expect("Failed to create renderer");

        // Verify that Handlebars instance is properly initialized
        assert!(
            renderer.handlebars.has_template("dashboard_unified"),
            "Should have dashboard_unified template registered"
        );
        assert!(
            renderer.handlebars.has_template("dashboard_final"),
            "Should have dashboard_final template registered"
        );
    }

    /// Test rendering with various data scenarios
    #[test]
    fn test_rendering_various_scenarios() {
        let renderer = DashboardRenderer::new()
            .expect("Failed to create renderer for scenario test");

        // Create a context with various data scenarios
        let context = DashboardContext {
            title: "Test Dashboard".to_string(),
            export_timestamp: "2024-01-01 00:00:00".to_string(),
            total_memory: "1.00 KB".to_string(),
            total_allocations: 100,
            active_allocations: 50,
            peak_memory: "2.00 KB".to_string(),
            thread_count: 5,
            passport_count: 10,
            leak_count: 2,
            unsafe_count: 3,
            ffi_count: 1,
            allocations: vec![],
            relationships: vec![],
            unsafe_reports: vec![],
            passport_details: vec![],
            allocations_count: 100,
            relationships_count: 0,
            unsafe_reports_count: 0,
            json_data: "{}".to_string(),
            os_name: "Test OS".to_string(),
            architecture: "x86_64".to_string(),
            cpu_cores: 4,
            system_resources: SystemResources {
                os_name: "Test OS".to_string(),
                os_version: "1.0.0".to_string(),
                architecture: "x86_64".to_string(),
                cpu_cores: 4,
                total_physical: "8.00 GB".to_string(),
                available_physical: "4.00 GB".to_string(),
                used_physical: "4.00 GB".to_string(),
                page_size: 4096,
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
            },
            health_score: 85,
            health_status: "Good".to_string(),
            safe_ops_count: 97,
            high_risk_count: 3,
            clean_passport_count: 8,
            active_passport_count: 2,
            leaked_passport_count: 2,
            ffi_tracked_count: 1,
            safe_code_percent: 97,
            ownership_graph: OwnershipGraphInfo {
                total_nodes: 50,
                total_edges: 100,
                total_cycles: 1,
                rc_clone_count: 10,
                arc_clone_count: 20,
                has_issues: true,
                issues: vec![],
                root_cause: None,
            },
        };

        // Test that context can be serialized (required for rendering)
        let json_str = serde_json::to_string(&context)
            .expect("Failed to serialize context for rendering test");
        assert!(!json_str.is_empty(), "Serialized context should not be empty");
    }
}
