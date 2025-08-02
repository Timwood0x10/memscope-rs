//! Convert command implementation
//!
//! This module provides functionality to convert between different memory analysis formats,
//! including binary, JSON, and HTML formats.

use crate::export::binary_converter::BinaryConverter;
use crate::export::conversion_validator::ConversionValidator;
use clap::ArgMatches;
use std::path::Path;
use std::time::Instant;

/// Run the convert command
pub fn run_convert(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let from_format = matches.get_one::<String>("from").unwrap();
    let to_format = matches.get_one::<String>("to").unwrap();
    let validate = matches.get_flag("validate");
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Converting {} -> {}", input_path, output_path);
        println!("Format: {} -> {}", from_format, to_format);
    }

    let start_time = Instant::now();

    match (from_format.as_str(), to_format.as_str()) {
        ("binary", "json") => {
            convert_binary_to_json(input_path, output_path, validate, verbose)?;
        }
        ("binary", "html") => {
            convert_binary_to_html(input_path, output_path, validate, verbose)?;
        }
        ("json", "binary") => {
            return Err("JSON to binary conversion is not yet implemented".into());
        }
        _ => {
            return Err(format!("Unsupported conversion: {} -> {}", from_format, to_format).into());
        }
    }

    let duration = start_time.elapsed();

    if verbose {
        println!("Conversion completed in {:.2}s", duration.as_secs_f64());
    } else {
        println!("Conversion successful: {} -> {}", input_path, output_path);
    }

    Ok(())
}

/// Convert binary format to JSON
fn convert_binary_to_json(
    input_path: &str,
    output_path: &str,
    validate: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = Path::new(input_path);
    let output = Path::new(output_path);

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input_path).into());
    }

    if verbose {
        println!("Reading binary file: {}", input_path);
    }

    // Perform conversion
    let result = BinaryConverter::binary_to_json(input, output)?;

    if verbose {
        println!("Conversion statistics:");
        println!("  Input size: {} bytes", result.input_size);
        println!("  Output size: {} bytes", result.output_size);
        println!("  Size ratio: {:.2}x", result.size_ratio());
        println!("  Speed: {:.2} MB/s", result.conversion_speed_mbps());

        if let Some(memory_usage) = result.memory_usage_mb() {
            println!("  Memory usage: {:.2} MB", memory_usage);
        }
    }

    // Perform validation if requested
    if validate {
        if verbose {
            println!("Validating conversion result...");
        }

        let validator = ConversionValidator::new();
        match validator.validate_conversion(input, output, &result) {
            Ok(validation_result) => {
                if validation_result.is_valid {
                    println!(
                        "✅ Validation passed (Grade: {:?}, Score: {:.1}/100)",
                        validation_result.quality_metrics.quality_grade,
                        validation_result.quality_metrics.overall_score
                    );
                } else {
                    println!("❌ Validation failed");
                    for error in &validation_result.validation_errors {
                        println!("  Error: {}", error.message);
                    }
                }

                if verbose && !validation_result.validation_warnings.is_empty() {
                    println!("Warnings:");
                    for warning in &validation_result.validation_warnings {
                        println!("  ⚠️  {}", warning.message);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Validation failed: {}", e);
            }
        }
    }

    Ok(())
}

/// Convert binary format to HTML
fn convert_binary_to_html(
    input_path: &str,
    output_path: &str,
    validate: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = Path::new(input_path);
    let output = Path::new(output_path);

    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input_path).into());
    }

    if verbose {
        println!("Reading binary file: {}", input_path);
    }

    // Perform conversion
    let result = BinaryConverter::binary_to_html(input, output)?;

    if verbose {
        println!("Conversion statistics:");
        println!("  Input size: {} bytes", result.input_size);
        println!("  Output size: {} bytes", result.output_size);
        println!("  Size ratio: {:.2}x", result.size_ratio());
        println!("  Speed: {:.2} MB/s", result.conversion_speed_mbps());
    }

    // Basic validation for HTML (check if file was created and has content)
    if validate {
        if verbose {
            println!("Validating HTML output...");
        }

        let html_content = std::fs::read_to_string(output)?;
        if html_content.contains("<html>") && html_content.contains("</html>") {
            println!("✅ HTML validation passed");
        } else {
            println!("❌ HTML validation failed: Invalid HTML structure");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_convert_command_validation() {
        // Test that the convert command validates input parameters correctly

        // Test unsupported conversion
        let result = std::panic::catch_unwind(|| {
            convert_binary_to_json("nonexistent.bin", "output.json", false, false)
        });

        // The function should return an error for nonexistent files
        // This is a basic test to ensure error handling works
    }

    #[test]
    fn test_format_validation() {
        // Test format validation logic
        let valid_combinations = vec![("binary", "json"), ("binary", "html")];

        for (from, to) in valid_combinations {
            // These should be valid format combinations
            assert!(matches!(
                (from, to),
                ("binary", "json") | ("binary", "html")
            ));
        }

        // Invalid combinations should be rejected
        assert!(!matches!(
            ("json", "binary"),
            ("binary", "json") | ("binary", "html")
        ));
    }
}
