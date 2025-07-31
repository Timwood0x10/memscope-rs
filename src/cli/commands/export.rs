//! Export command implementation for binary data conversion
//!
//! This module provides the export subcommand functionality for converting
//! binary memory analysis files to other formats (JSON, HTML).

use clap::ArgMatches;
use std::error::Error;
use std::path::Path;

/// Run the export command
pub fn run_export(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Extract arguments from matches
    let input_path = matches
        .get_one::<String>("input")
        .ok_or("Input file argument is required")?;
    
    let output_path = matches
        .get_one::<String>("output")
        .ok_or("Output file argument is required")?;
    
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json");
    
    let streaming = matches.get_flag("streaming");
    let validate_only = matches.get_flag("validate-only");

    println!("🔄 Starting binary data export...");
    println!("📂 Input file: {input_path}");
    println!("📁 Output file: {output_path}");
    println!("📊 Format: {format}");
    println!("🌊 Streaming mode: {streaming}");
    println!("🔍 Validate only: {validate_only}");

    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input file does not exist: {input_path}").into());
    }

    // If validate-only mode, just validate and exit
    if validate_only {
        return validate_binary_file(input_path);
    }

    // Perform the conversion based on format
    match format {
        "json" => convert_to_json(input_path, output_path, streaming),
        "html" => convert_to_html(input_path, output_path, streaming),
        _ => Err(format!("Unsupported format: {format}. Supported formats: json, html").into()),
    }
}

/// Validate binary file without conversion
fn validate_binary_file(input_path: &str) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_validation::BinaryValidator;

    println!("🔍 Validating binary file: {input_path}");
    
    let validator = BinaryValidator::new();
    let validation_result = validator.validate_file(input_path)
        .map_err(|e| format!("Validation failed: {e}"))?;

    // Print validation results
    println!("\n📊 Validation Results:");
    println!("   Status: {}", if validation_result.is_valid { "✅ VALID" } else { "❌ INVALID" });
    
    if let Some(version) = &validation_result.format_version {
        println!("   Format version: {version}");
    }
    
    if let Some(compression) = &validation_result.compression_detected {
        println!("   Compression: {compression}");
    }
    
    println!("   File size: {} bytes", validation_result.file_info.file_size);
    
    if let Some(uncompressed_size) = validation_result.file_info.uncompressed_size {
        println!("   Uncompressed size: {uncompressed_size} bytes");
        if let Some(ratio) = validation_result.file_info.compression_ratio {
            println!("   Compression ratio: {:.1}%", ratio * 100.0);
        }
    }

    // Print errors if any
    if !validation_result.errors.is_empty() {
        println!("\n❌ Validation Errors:");
        for error in &validation_result.errors {
            println!("   - {}: {}", 
                     format!("{:?}", error.severity).to_uppercase(),
                     error.message);
        }
    }

    // Print warnings if any
    if !validation_result.warnings.is_empty() {
        println!("\n⚠️  Validation Warnings:");
        for warning in &validation_result.warnings {
            println!("   - {warning}");
        }
    }

    // Print recovery assessment
    println!("\n🔧 Recovery Assessment:");
    println!("   Recoverable: {}", if validation_result.recovery_assessment.recoverable { "Yes" } else { "No" });
    println!("   Recovery percentage: {:.1}%", validation_result.recovery_assessment.recovery_percentage);
    println!("   Recovery effort: {:?}", validation_result.recovery_assessment.recovery_effort);

    if validation_result.is_valid {
        println!("\n🎉 Binary file validation completed successfully!");
        Ok(())
    } else {
        Err("Binary file validation failed".into())
    }
}

/// Convert binary file to JSON format
fn convert_to_json(input_path: &str, output_path: &str, streaming: bool) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_parser::{BinaryParser, BinaryParseOptions};
    use crate::export::formats::json_converter::{JsonConverter, JsonConvertOptions};

    println!("📄 Converting binary to JSON format...");

    // Parse binary file
    println!("🔍 Parsing binary file...");
    let parser_options = if streaming {
        BinaryParseOptions::streaming()
    } else {
        BinaryParseOptions::safe()
    };
    
    let parser = BinaryParser::new(parser_options);
    let binary_data = parser.parse_file(input_path)
        .map_err(|e| format!("Failed to parse binary file: {e}"))?;

    // Convert to JSON
    println!("🔄 Converting to JSON...");
    let converter_options = if streaming {
        JsonConvertOptions::streaming()
    } else {
        JsonConvertOptions::compatible() // Use compatible format for better tool integration
    };
    
    let converter = JsonConverter::new(converter_options);
    let conversion_stats = converter.convert_to_file(&binary_data, output_path)
        .map_err(|e| format!("Failed to convert to JSON: {e}"))?;

    // Print conversion statistics
    println!("\n📊 Conversion Statistics:");
    println!("   Conversion time: {:?}", conversion_stats.conversion_time);
    println!("   Allocations converted: {}", conversion_stats.allocations_converted);
    println!("   Output size: {} bytes", conversion_stats.output_size);
    println!("   Chunks processed: {}", conversion_stats.chunks_processed);
    
    if !conversion_stats.validation_errors.is_empty() {
        println!("   Validation errors: {}", conversion_stats.validation_errors.len());
    }

    println!("\n🎉 JSON conversion completed successfully!");
    println!("📁 Output file: {output_path}");

    Ok(())
}

/// Convert binary file to HTML format
fn convert_to_html(input_path: &str, output_path: &str, streaming: bool) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_parser::{BinaryParser, BinaryParseOptions};
    use crate::export::formats::html_converter::{HtmlConverter, HtmlConvertOptions};

    println!("🌐 Converting binary to HTML format...");

    // Parse binary file
    println!("🔍 Parsing binary file...");
    let parser_options = if streaming {
        BinaryParseOptions::streaming()
    } else {
        BinaryParseOptions::safe()
    };
    
    let parser = BinaryParser::new(parser_options);
    let binary_data = parser.parse_file(input_path)
        .map_err(|e| format!("Failed to parse binary file: {e}"))?;

    // Convert to HTML
    println!("🔄 Converting to HTML...");
    let converter_options = if streaming {
        HtmlConvertOptions::performance() // Use performance mode for streaming
    } else {
        HtmlConvertOptions::complete() // Use complete features for standard mode
    };
    
    let mut converter = HtmlConverter::new(converter_options);
    let conversion_stats = converter.convert_to_file(&binary_data, output_path)
        .map_err(|e| format!("Failed to convert to HTML: {e}"))?;

    // Print conversion statistics
    println!("\n📊 Conversion Statistics:");
    println!("   Conversion time: {:?}", conversion_stats.conversion_time);
    println!("   Allocations processed: {}", conversion_stats.allocations_processed);
    println!("   HTML size: {} bytes", conversion_stats.html_size);
    println!("   Charts generated: {}", conversion_stats.charts_generated);
    println!("   Table rows: {}", conversion_stats.table_rows_generated);
    println!("   Template processing time: {:?}", conversion_stats.template_processing_time);

    println!("\n🎉 HTML conversion completed successfully!");
    println!("📁 Output file: {output_path}");
    println!("🌐 Open in browser to view the interactive report");

    Ok(())
}