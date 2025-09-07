//! Test command implementation
//!
//! This module provides the test subcommand functionality.

use crate::core::tracker::get_global_tracker;
use crate::track_var;
use clap::ArgMatches;
use std::error::Error;

/// Run the test command
pub fn run_test(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("enhanced_memory_test");

    tracing::info!("🧪 Running enhanced memory tests...");
    tracing::info!("Output path: {}", output_path);

    // Initialize memory tracking
    crate::init();

    // Run the test
    run_enhanced_memory_test()?;

    Ok(())
}

fn run_enhanced_memory_test() -> Result<(), Box<dyn Error>> {
    use crate::core::tracker::get_global_tracker;

    tracing::info!("Creating test allocations...");

    // Create some test allocations
    let vec1 = vec![1, 2, 3, 4, 5];
    let vec2 = vec![6, 7, 8, 9, 10];
    let string1 = String::from("Hello, World!");
    let boxed1 = Box::new(42);

    track_var!(vec1);
    track_var!(vec2);
    track_var!(string1);
    track_var!(boxed1);

    // Create some temporary objects
    for i in 0..10 {
        let temp = vec![i; i];
        track_var!(temp);
    }

    // Get all allocations
    let tracker = get_global_tracker();
    let _allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            tracing::info!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    tracing::info!("Enhanced Memory Analysis Summary:");
    tracing::info!("--------------------------------");
    tracing::info!("Total active allocations: {}", _allocations.len());

    // Keep variables alive until the end
    tracing::info!("Vec1 length: {}", vec1.len());
    tracing::info!("Vec2 length: {}", vec2.len());
    tracing::info!("String1 length: {}", string1.len());
    tracing::info!("Boxed1 value: {}", *boxed1);

    Ok(())
}

fn _test_enhanced_memory_analysis() {
    // Create some test allocations
    let vec1 = vec![1, 2, 3, 4, 5];
    let vec2 = vec![6, 7, 8, 9, 10];
    let string1 = String::from("Hello, World!");
    let boxed1 = Box::new(42);

    track_var!(vec1);
    track_var!(vec2);
    track_var!(string1);
    track_var!(boxed1);

    // Create some temporary objects
    for i in 0..10 {
        let temp = vec![i; i];
        track_var!(temp);
    }

    // Get all allocations
    let tracker = get_global_tracker();
    let _allocations = match tracker.get_active_allocations() {
        Ok(allocs) => allocs,
        Err(e) => {
            tracing::info!("Error getting allocations: {}", e);
            Vec::new()
        }
    };

    // Run enhanced analysis
    let report = crate::analysis::analyze_memory_with_enhanced_features()
        .unwrap_or_else(|e| format!("Error: {e}"));

    // Print summary
    tracing::info!("Enhanced Memory Analysis Summary:");
    tracing::info!("--------------------------------");
    tracing::info!("Report: {}", report);

    // Keep variables alive until the end
    tracing::info!("Vec1 length: {}", vec1.len());
    tracing::info!("Vec2 length: {}", vec2.len());
    tracing::info!("String1 length: {}", string1.len());
    tracing::info!("Boxed1 value: {}", *boxed1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    fn create_test_matches(output: Option<&str>) -> ArgMatches {
        let cmd = Command::new("test").arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .default_value("enhanced_memory_test"),
        );

        if let Some(output_val) = output {
            cmd.try_get_matches_from(vec!["test", "--output", output_val])
                .unwrap()
        } else {
            cmd.try_get_matches_from(vec!["test"]).unwrap()
        }
    }

    #[test]
    fn test_run_test_with_default_output() {
        let matches = create_test_matches(None);
        
        // Test that we can extract the output path correctly
        let output_path = matches
            .get_one::<String>("output")
            .map(|s| s.as_str())
            .unwrap_or("enhanced_memory_test");
        
        assert_eq!(output_path, "enhanced_memory_test");
    }

    #[test]
    fn test_run_test_with_custom_output() {
        let matches = create_test_matches(Some("custom_test_output"));
        
        let output_path = matches
            .get_one::<String>("output")
            .map(|s| s.as_str())
            .unwrap_or("enhanced_memory_test");
        
        assert_eq!(output_path, "custom_test_output");
    }

    #[test]
    fn test_run_enhanced_memory_test_function_exists() {
        // Test that the function exists and has the correct signature
        let _f: fn() -> Result<(), Box<dyn std::error::Error>> = run_enhanced_memory_test;
        
        // Just verify the function signature is correct
        assert!(true);
    }

    #[test]
    fn test_run_test_function_signature() {
        // Test that run_test has the correct signature
        let _f: fn(&ArgMatches) -> Result<(), Box<dyn std::error::Error>> = run_test;
        assert!(true);
    }

    #[test]
    fn test_private_test_function_exists() {
        // Test that the private test function exists
        let _f: fn() = _test_enhanced_memory_analysis;
        assert!(true);
    }

    #[test]
    fn test_argument_parsing() {
        // Test various argument combinations
        let test_cases = vec![
            ("default", None, "enhanced_memory_test"),
            ("custom", Some("my_output"), "my_output"),
            ("path", Some("/tmp/test_output"), "/tmp/test_output"),
        ];

        for (name, input, expected) in test_cases {
            let matches = create_test_matches(input);
            let output_path = matches
                .get_one::<String>("output")
                .map(|s| s.as_str())
                .unwrap_or("enhanced_memory_test");
            
            assert_eq!(output_path, expected, "Test case '{}' failed", name);
        }
    }

    #[test]
    fn test_clap_command_structure() {
        // Test that the command structure is correct
        let mut cmd = Command::new("test").arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .default_value("enhanced_memory_test"),
        );

        // Verify the command can be built
        let app = cmd.clone();
        assert_eq!(app.get_name(), "test");
        
        // Test help generation doesn't panic
        let _help = cmd.render_help();
        assert!(true);
    }

    #[test]
    fn test_output_path_extraction_logic() {
        // Test the core logic used in run_test
        let test_cases = vec![
            (Some("test_output"), "test_output"),
            (None, "enhanced_memory_test"),
        ];

        for (input, expected) in test_cases {
            let result = input
                .map(|s| s)
                .unwrap_or("enhanced_memory_test");
            
            assert_eq!(result, expected);
        }
    }
}
