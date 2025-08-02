//! Binary validate command implementation
//!
//! This module provides functionality to validate binary memory analysis files,
//! including integrity checks, data validation, and performance analysis.

use crate::export::binary_converter::BinaryConverter;
use crate::export::binary_parser::BinaryParser;
use crate::export::conversion_validator::{ConversionValidator, ValidationOptions};
use clap::ArgMatches;
use std::path::Path;
use std::time::Instant;

/// Run the binary-validate command
pub fn run_binary_validate(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches.get_one::<String>("input").unwrap();
    let comprehensive = matches.get_flag("comprehensive");
    let performance = matches.get_flag("performance");
    let report_path = matches.get_one::<String>("report");

    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input_path).into());
    }

    println!("Binary Memory Analysis File Validation");
    println!("=====================================");
    println!("File: {}", input_path);

    let start_time = Instant::now();
    let mut validation_passed = true;

    // Step 1: Basic file integrity validation
    println!("\n1. File Integrity Validation");
    println!("   Checking file structure and checksums...");

    let mut parser = BinaryParser::default();
    match parser.load_from_file(input) {
        Ok(_) => {
            println!("   ✅ File loaded successfully");

            // Validate file integrity
            match parser.validate_integrity() {
                Ok(_) => {
                    println!("   ✅ File integrity check passed");
                }
                Err(e) => {
                    println!("   ❌ Integrity validation failed: {}", e);
                    validation_passed = false;
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to load file: {}", e);
            validation_passed = false;
            return Ok(()); // Can't continue without loading the file
        }
    }

    // Step 2: Data structure validation
    println!("\n2. Data Structure Validation");
    println!("   Validating internal data structures...");

    let data_validation_result = validate_data_structures(&mut parser);
    if data_validation_result.is_ok() {
        println!("   ✅ Data structures are valid");
    } else {
        println!(
            "   ❌ Data structure validation failed: {}",
            data_validation_result.unwrap_err()
        );
        validation_passed = false;
    }

    // Step 3: Comprehensive validation (if requested)
    if comprehensive {
        println!("\n3. Comprehensive Data Validation");
        println!("   Performing detailed data consistency checks...");

        match perform_comprehensive_validation(&parser, input) {
            Ok(result) => {
                if result.is_valid {
                    println!("   ✅ Comprehensive validation passed");
                    println!(
                        "      Quality Grade: {:?}",
                        result.quality_metrics.quality_grade
                    );
                    println!(
                        "      Overall Score: {:.1}/100",
                        result.quality_metrics.overall_score
                    );
                } else {
                    println!("   ❌ Comprehensive validation failed");
                    validation_passed = false;

                    for error in &result.validation_errors {
                        println!("      Error: {}", error.message);
                    }
                }

                if !result.validation_warnings.is_empty() {
                    println!("   Warnings:");
                    for warning in &result.validation_warnings {
                        println!("      ⚠️  {}", warning.message);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Comprehensive validation failed: {}", e);
                validation_passed = false;
            }
        }
    }

    // Step 4: Performance analysis (if requested)
    if performance {
        println!("\n4. Performance Analysis");
        println!("   Analyzing conversion performance...");

        match perform_performance_analysis(&parser, input) {
            Ok(perf_result) => {
                println!("   ✅ Performance analysis completed");
                println!(
                    "      Conversion Speed: {:.2} MB/s",
                    perf_result.conversion_speed_mbps
                );
                println!("      Size Ratio: {:.2}x", perf_result.size_ratio);
                println!(
                    "      Performance Category: {:?}",
                    perf_result.performance_category
                );
                println!(
                    "      Performance Rating: {}/5",
                    perf_result.performance_rating
                );

                if let Some(memory_usage) = perf_result.memory_usage_mb {
                    println!("      Memory Usage: {:.2} MB", memory_usage);
                }
            }
            Err(e) => {
                println!("   ⚠️  Performance analysis failed: {}", e);
            }
        }
    }

    let validation_duration = start_time.elapsed();

    // Generate summary
    println!("\nValidation Summary");
    println!("=================");
    if validation_passed {
        println!("✅ Overall validation: PASSED");
    } else {
        println!("❌ Overall validation: FAILED");
    }
    println!("Validation time: {:.2}s", validation_duration.as_secs_f64());

    // Generate report if requested
    if let Some(report_path) = report_path {
        println!("\nGenerating validation report...");
        generate_validation_report(
            input_path,
            report_path,
            validation_passed,
            validation_duration,
        )?;
        println!("Report saved to: {}", report_path);
    }

    if !validation_passed {
        std::process::exit(1);
    }

    Ok(())
}

/// Validate data structures within the binary file
fn validate_data_structures(parser: &mut BinaryParser) -> Result<(), Box<dyn std::error::Error>> {
    // Validate memory statistics
    let _stats = parser
        .load_memory_stats()
        .map_err(|e| format!("Failed to load memory statistics: {}", e))?;

    // Validate allocations
    let allocations = parser
        .load_allocations()
        .map_err(|e| format!("Failed to load allocations: {}", e))?;

    // Basic consistency checks
    if allocations.is_empty() {
        return Err("No allocations found in file".into());
    }

    // Validate type memory usage
    let _type_usage = parser
        .load_type_memory_usage()
        .map_err(|e| format!("Failed to load type memory usage: {}", e))?;

    Ok(())
}

/// Perform comprehensive validation using the conversion validator
fn perform_comprehensive_validation(
    parser: &BinaryParser,
    input_path: &Path,
) -> Result<crate::export::conversion_validator::ValidationResult, Box<dyn std::error::Error>> {
    // Create a temporary JSON file for validation
    let temp_dir = tempfile::TempDir::new()?;
    let temp_json = temp_dir.path().join("validation_temp.json");

    // Convert to JSON for validation
    let conversion_result = BinaryConverter::binary_to_json(input_path, &temp_json)?;

    // Perform validation
    let validation_options = ValidationOptions {
        enable_comprehensive_validation: true,
        enable_performance_comparison: false, // Skip for validation
        validation_sample_size: 1000,
        performance_thresholds: Default::default(),
        enable_checksum_validation: true,
        max_acceptable_error_rate: 0.01,
        enable_detailed_logging: false,
    };

    let validator = ConversionValidator::with_options(validation_options);
    let validation_result =
        validator.validate_conversion(input_path, &temp_json, &conversion_result)?;

    Ok(validation_result)
}

/// Perform performance analysis
fn perform_performance_analysis(
    _parser: &BinaryParser,
    input_path: &Path,
) -> Result<
    crate::export::conversion_validator::PerformanceComparisonResult,
    Box<dyn std::error::Error>,
> {
    // Create a temporary JSON file for performance testing
    let temp_dir = tempfile::TempDir::new()?;
    let temp_json = temp_dir.path().join("performance_temp.json");

    // Perform conversion with performance measurement
    let conversion_result = BinaryConverter::binary_to_json(input_path, &temp_json)?;

    // Create performance result
    let performance_result = crate::export::conversion_validator::PerformanceComparisonResult {
        conversion_speed_mbps: conversion_result.conversion_speed_mbps(),
        size_ratio: conversion_result.size_ratio(),
        memory_usage_mb: conversion_result.memory_usage_mb(),
        json_export_comparison: None,
        performance_rating: calculate_performance_rating(
            conversion_result.conversion_speed_mbps(),
            conversion_result.size_ratio(),
        ),
        performance_category: categorize_performance(
            conversion_result.conversion_speed_mbps(),
            conversion_result.size_ratio(),
        ),
    };

    Ok(performance_result)
}

/// Calculate performance rating (1-5 scale)
fn calculate_performance_rating(speed_mbps: f64, size_ratio: f64) -> u8 {
    let speed_score = match speed_mbps {
        s if s > 50.0 => 5,
        s if s > 20.0 => 4,
        s if s > 5.0 => 3,
        s if s > 1.0 => 2,
        _ => 1,
    };

    let size_score = match size_ratio {
        r if r < 1.5 => 5,
        r if r < 2.0 => 4,
        r if r < 3.0 => 3,
        r if r < 5.0 => 2,
        _ => 1,
    };

    ((speed_score + size_score) / 2).max(1).min(5)
}

/// Categorize performance
fn categorize_performance(
    speed_mbps: f64,
    size_ratio: f64,
) -> crate::export::conversion_validator::PerformanceCategory {
    use crate::export::conversion_validator::PerformanceCategory;

    match (speed_mbps, size_ratio) {
        (s, r) if s > 50.0 && r < 1.5 => PerformanceCategory::Excellent,
        (s, r) if s > 20.0 && r < 2.0 => PerformanceCategory::Good,
        (s, r) if s > 5.0 && r < 3.0 => PerformanceCategory::Average,
        (s, r) if s > 1.0 && r < 5.0 => PerformanceCategory::Poor,
        _ => PerformanceCategory::VeryPoor,
    }
}

/// Generate a validation report
fn generate_validation_report(
    input_path: &str,
    report_path: &str,
    validation_passed: bool,
    validation_duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_content = format!(
        "# Binary File Validation Report\n\n\
        **File:** {}\n\
        **Validation Date:** {}\n\
        **Validation Duration:** {:.2}s\n\
        **Overall Result:** {}\n\n\
        ## Summary\n\n\
        This report contains the results of validating the binary memory analysis file.\n\
        The validation process includes file integrity checks, data structure validation,\n\
        and optional comprehensive data validation and performance analysis.\n\n\
        ## Results\n\n\
        - File Integrity: {}\n\
        - Data Structure: {}\n\n\
        ## Recommendations\n\n\
        {}\n",
        input_path,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        validation_duration.as_secs_f64(),
        if validation_passed {
            "PASSED"
        } else {
            "FAILED"
        },
        if validation_passed {
            "✅ Passed"
        } else {
            "❌ Failed"
        },
        if validation_passed {
            "✅ Passed"
        } else {
            "❌ Failed"
        },
        if validation_passed {
            "The binary file passed all validation checks and appears to be in good condition."
        } else {
            "The binary file failed validation. Please check the console output for specific errors and consider regenerating the file."
        }
    );

    std::fs::write(report_path, report_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_rating_calculation() {
        // Test excellent performance
        assert_eq!(calculate_performance_rating(60.0, 1.2), 5);

        // Test good performance
        assert_eq!(calculate_performance_rating(25.0, 1.8), 4);

        // Test average performance
        assert_eq!(calculate_performance_rating(8.0, 2.5), 3);

        // Test poor performance
        assert_eq!(calculate_performance_rating(2.0, 4.0), 2);

        // Test very poor performance
        assert_eq!(calculate_performance_rating(0.5, 6.0), 1);
    }

    #[test]
    fn test_performance_categorization() {
        use crate::export::conversion_validator::PerformanceCategory;

        // Test excellent performance
        assert_eq!(
            categorize_performance(60.0, 1.2),
            PerformanceCategory::Excellent
        );

        // Test good performance
        assert_eq!(categorize_performance(25.0, 1.8), PerformanceCategory::Good);

        // Test average performance
        assert_eq!(
            categorize_performance(8.0, 2.5),
            PerformanceCategory::Average
        );
    }
}
