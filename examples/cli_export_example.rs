//! Example demonstrating CLI-based export with different modes and validation settings
//!
//! This example shows how to use the command line interface for memory export operations
//! with configurable validation modes and timing.

use clap::Parser;
use memscope_rs::export::quality_validator::{ExportArgs, ExportConfig, ValidationTiming, ExportMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = ExportArgs::parse();
    
    // Validate arguments
    if let Err(error) = args.validate() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
    
    // Convert to export configuration
    let export_config = args.to_export_config();
    
    println!("🚀 Starting export with configuration:");
    println!("   Mode: {:?}", export_config.mode);
    println!("   Validation Timing: {:?}", export_config.validation_timing);
    println!("   Output: {}", args.output.display());
    println!("   Timeout: {:?}", args.get_timeout_duration());
    println!("   Verbose: {}", args.verbose);
    
    // Print validation configuration details
    println!("\n📋 Validation Configuration:");
    println!("   JSON Validation: {}", export_config.validation_config.enable_json_validation);
    println!("   Integrity Validation: {}", export_config.validation_config.enable_integrity_validation);
    println!("   Count Validation: {}", export_config.validation_config.enable_count_validation);
    println!("   Size Validation: {}", export_config.validation_config.enable_size_validation);
    println!("   Encoding Validation: {}", export_config.validation_config.enable_encoding_validation);
    println!("   Max Data Loss Rate: {:.2}%", export_config.validation_config.max_data_loss_rate * 100.0);
    println!("   Min File Size: {} bytes", export_config.validation_config.min_expected_file_size);
    println!("   Max File Size: {} bytes", export_config.validation_config.max_expected_file_size);
    
    // Demonstrate different export scenarios
    match (export_config.mode, export_config.validation_timing) {
        (ExportMode::Fast, ValidationTiming::Disabled) => {
            println!("\n⚡ Fast mode with no validation - Maximum performance");
            println!("   Use case: High-frequency exports where speed is critical");
        }
        (ExportMode::Fast, ValidationTiming::Deferred) => {
            println!("\n⚡ Fast mode with deferred validation - Balanced approach");
            println!("   Use case: Performance-sensitive with background validation");
        }
        (ExportMode::Slow, ValidationTiming::Inline) => {
            println!("\n🔍 Slow mode with inline validation - Maximum thoroughness");
            println!("   Use case: Critical data integrity requirements");
        }
        (ExportMode::Slow, ValidationTiming::Deferred) => {
            println!("\n🔍 Slow mode with deferred validation - Comprehensive with async validation");
            println!("   Use case: Thorough validation without blocking export completion");
        }
        (ExportMode::Auto, timing) => {
            println!("\n🤖 Auto mode with {:?} validation - Adaptive approach", timing);
            println!("   Use case: General purpose with automatic optimization");
        }
        _ => {
            println!("\n📝 Custom configuration");
        }
    }
    
    // Show example usage patterns
    println!("\n💡 Example Usage Patterns:");
    println!("   # Fast export for performance testing");
    println!("   cargo run --example cli_export_example -- --mode fast --validation disabled -o /tmp/fast_export.json");
    println!();
    println!("   # Thorough export for production data");
    println!("   cargo run --example cli_export_example -- --mode slow --validation inline -o /tmp/production_export.json --verbose");
    println!();
    println!("   # Balanced export with custom timeout");
    println!("   cargo run --example cli_export_example -- --mode auto --validation deferred -o /tmp/balanced_export.json --timeout 60");
    println!();
    println!("   # Custom validation settings");
    println!("   cargo run --example cli_export_example -- --mode fast --max-data-loss-rate 0.5 --min-file-size 2048 -o /tmp/custom_export.json");
    
    // Simulate export process (in real usage, this would call the actual export functions)
    println!("\n🔄 Simulating export process...");
    
    // Here you would typically call:
    // let mut coordinator = FastExportCoordinator::new_with_export_config(export_config);
    // let result = coordinator.export_with_mode(&args.output).await?;
    
    println!("✅ Export simulation completed successfully!");
    println!("   Output would be written to: {}", args.output.display());
    
    if export_config.validation_timing != ValidationTiming::Disabled {
        println!("   Validation would run with timeout: {:?}", args.get_timeout_duration());
    }
    
    Ok(())
}

/// Helper function to demonstrate argument parsing without running main
#[cfg(test)]
pub fn parse_example_args(args: &[&str]) -> Result<ExportConfig, String> {
    use clap::Parser;
    
    let args = ExportArgs::try_parse_from(args).map_err(|e| e.to_string())?;
    args.validate()?;
    Ok(args.to_export_config())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fast_mode_parsing() {
        let config = parse_example_args(&[
            "cli_export_example",
            "--mode", "fast",
            "--validation", "deferred",
            "-o", "/tmp/test.json"
        ]).unwrap();
        
        assert_eq!(config.mode, ExportMode::Fast);
        assert_eq!(config.validation_timing, ValidationTiming::Deferred);
        assert!(!config.validation_config.enable_json_validation);
        assert!(!config.validation_config.enable_encoding_validation);
    }
    
    #[test]
    fn test_slow_mode_parsing() {
        let config = parse_example_args(&[
            "cli_export_example",
            "--mode", "slow",
            "--validation", "inline",
            "-o", "/tmp/test.json",
            "--verbose"
        ]).unwrap();
        
        assert_eq!(config.mode, ExportMode::Slow);
        assert_eq!(config.validation_timing, ValidationTiming::Inline);
        assert!(config.validation_config.enable_json_validation);
        assert!(config.validation_config.enable_encoding_validation);
        assert!(config.validation_config.verbose_logging);
    }
    
    #[test]
    fn test_validation_disabled() {
        let config = parse_example_args(&[
            "cli_export_example",
            "--mode", "auto",
            "--disable-validation",
            "-o", "/tmp/test.json"
        ]).unwrap();
        
        assert_eq!(config.validation_timing, ValidationTiming::Disabled);
    }
    
    #[test]
    fn test_custom_settings() {
        let config = parse_example_args(&[
            "cli_export_example",
            "--mode", "fast",
            "--max-data-loss-rate", "1.5",
            "--min-file-size", "2048",
            "--max-file-size", "209715200", // 200MB
            "--timeout", "120",
            "-o", "/tmp/test.json"
        ]).unwrap();
        
        assert_eq!(config.validation_config.max_data_loss_rate, 0.015); // 1.5% converted to fraction
        assert_eq!(config.validation_config.min_expected_file_size, 2048);
        assert_eq!(config.validation_config.max_expected_file_size, 209715200);
    }
    
    #[test]
    fn test_invalid_arguments() {
        // Test empty output path
        let result = parse_example_args(&[
            "cli_export_example",
            "--mode", "fast",
            "-o", ""
        ]);
        assert!(result.is_err());
        
        // Test invalid data loss rate
        let result = parse_example_args(&[
            "cli_export_example",
            "--mode", "fast",
            "--max-data-loss-rate", "150.0", // > 100%
            "-o", "/tmp/test.json"
        ]);
        assert!(result.is_err());
    }
}