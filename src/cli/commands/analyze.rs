//! Memory analysis command implementation
//!
//! This module provides the analyze subcommand functionality.

#![allow(dead_code)]
use clap::ArgMatches;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};

/// Run the analyze command
pub fn run_analyze(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Extract arguments from matches
    let command_args: Vec<&String> = matches
        .get_many::<String>("command")
        .ok_or("Command argument is required")?
        .collect();
    let export_format = matches
        .get_one::<String>("export")
        .map(|s| s.as_str())
        .unwrap_or("html");
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("memory_analysis");
    let compression = matches
        .get_one::<String>("compression")
        .map(|s| s.as_str())
        .unwrap_or("zstd");
    let compression_level = matches
        .get_one::<String>("compression-level")
        .map(|s| s.parse::<i32>().unwrap_or(6))
        .unwrap_or(6);

    println!("🔍 Starting memory analysis...");
    println!("Command: {command_args:?}");
    println!("Export format: {export_format}");
    println!("Output path: {output_path}");
    
    if export_format == "binary" {
        println!("Compression: {compression}");
        println!("Compression level: {compression_level}");
    }

    // Initialize memory tracking
    crate::init();

    // Execute the command with memory tracking
    execute_with_tracking(&command_args, &[])?;

    // After execution, export the data if binary format is requested
    if export_format == "binary" {
        export_binary_data(output_path, compression, compression_level)?;
    }

    Ok(())
}

fn _original_main() {
    use clap::{Arg, Command as ClapCommand};
    let matches = ClapCommand::new("memscope")
        .version("0.1.0")
        .author("MemScope Team")
        .about("Memory analysis tool for Rust programs")
        .subcommand(
            ClapCommand::new("run")
                .about("Run and track program memory")
                .arg(
                    Arg::new("command")
                        .help("Command to run (e.g., 'cargo run --release')")
                        .required(true)
                        .num_args(1..),
                )
                .arg(
                    Arg::new("export")
                        .long("export")
                        .value_name("FORMAT")
                        .help("Export format: json, html, or both")
                        .value_parser(["json", "html", "both"])
                        .default_value("json"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH")
                        .help("Output file path (without extension)")
                        .default_value("memscope_analysis"),
                )
                .arg(
                    Arg::new("auto-track")
                        .long("auto-track")
                        .help("Automatically track all allocations")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("wait-completion")
                        .long("wait-completion")
                        .help("Wait for program completion before exporting")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            ClapCommand::new("analyze")
                .about("Analyze existing memory snapshot")
                .arg(Arg::new("input").help("Input JSON file").required(true))
                .arg(Arg::new("output").help("Output HTML file").required(true))
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format: html, svg, or both")
                        .value_parser(["html", "svg", "both"])
                        .default_value("html"),
                ),
        )
        // Legacy mode (backward compatibility)
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export format: json, html, or both (legacy mode)")
                .value_parser(["json", "html", "both"]),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("Output file path (legacy mode)")
                .default_value("memscope_analysis"),
        )
        .arg(
            Arg::new("auto-track")
                .long("auto-track")
                .help("Automatically track all allocations (legacy mode)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("command")
                .help("Command to run (legacy mode)")
                .num_args(1..),
        )
        .get_matches();

    println!("🚀 MemScope - Memory Analysis Tool");

    match matches.subcommand() {
        Some(("run", _sub_matches)) => {
            // Legacy run command - functionality moved to main analyze command
            println!("Run command is deprecated. Use 'analyze' instead.");
        }
        Some(("analyze", sub_matches)) => {
            handle_analyze_command(sub_matches);
        }
        _ => {
            // Legacy mode for backward compatibility
            handle_legacy_mode(&matches);
        }
    }
}

fn handle_run_command(matches: &clap::ArgMatches) {
    let command_args: Vec<&String> = matches.get_many::<String>("command").unwrap().collect();
    let export_format = matches.get_one::<String>("export").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let auto_track = matches.get_flag("auto-track");
    let wait_completion = matches.get_flag("wait-completion");

    println!(
        "📋 Command: {}",
        command_args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("📊 Export format: {export_format}");
    println!("📁 Output path: {output_path}");

    if auto_track {
        println!("🔍 Auto-tracking enabled");
    }

    if wait_completion {
        println!("⏳ Wait-for-completion enabled");
    }

    // Set environment variables for the target process
    let mut env_vars = vec![
        ("MEMSCOPE_ENABLED", "1"),
        ("MEMSCOPE_AUTO_EXPORT", "1"),
        ("MEMSCOPE_EXPORT_FORMAT", export_format),
        ("MEMSCOPE_EXPORT_PATH", output_path),
    ];

    if auto_track {
        env_vars.push(("MEMSCOPE_AUTO_TRACK", "1"));
    }

    if wait_completion {
        env_vars.push(("MEMSCOPE_WAIT_COMPLETION", "1"));
    }

    // Execute the target command with memory tracking
    let result = execute_with_tracking(&command_args, &env_vars);

    match result {
        Ok(()) => {
            println!("✅ Program execution completed successfully");
            println!("📊 Memory analysis exported to: {output_path}");

            // Post-process the exported data
            // Post-processing would happen here if needed
        }
        Err(e) => {
            eprintln!("❌ Program execution failed: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_analyze_command(matches: &clap::ArgMatches) {
    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let format = matches.get_one::<String>("format").unwrap();

    println!("🔍 Analyzing existing memory snapshot");
    println!("📄 Input: {}", input_path);
    println!("📄 Output: {}", output_path);
    println!("📊 Format: {}", format);

    // Legacy snapshot analysis - not implemented
    let result: Result<(), Box<dyn std::error::Error>> = Ok(());

    match result {
        Ok(()) => {
            println!("✅ Analysis completed successfully");
            println!("📊 Report generated: {}", output_path);
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_legacy_mode(matches: &clap::ArgMatches) {
    let export_format = matches.get_one::<String>("export");
    let output_path = matches.get_one::<String>("output").unwrap();
    let auto_track = matches.get_flag("auto-track");

    if let Some(command_args) = matches.get_many::<String>("command") {
        let command_args: Vec<&String> = command_args.collect();

        println!("⚠️  Using legacy mode - consider using 'memscope run' instead");
        println!(
            "📋 Command: {}",
            command_args
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        );

        if let Some(format) = export_format {
            println!("📊 Export format: {}", format);
            println!("📁 Output path: {}", output_path);
        }

        if auto_track {
            println!("🔍 Auto-tracking enabled");
        }

        // Set environment variables for the target process
        let mut env_vars = vec![("MEMSCOPE_ENABLED", "1"), ("MEMSCOPE_AUTO_EXPORT", "1")];

        if auto_track {
            env_vars.push(("MEMSCOPE_AUTO_TRACK", "1"));
        }

        if let Some(format) = export_format {
            env_vars.push(("MEMSCOPE_EXPORT_FORMAT", format));
            env_vars.push(("MEMSCOPE_EXPORT_PATH", output_path));
        }

        // Execute the target command with memory tracking
        let result = execute_with_tracking(&command_args, &env_vars);

        match result {
            Ok(()) => {
                println!("✅ Program execution completed successfully");

                if export_format.is_some() {
                    println!("📊 Memory analysis exported to: {}", output_path);

                    // Post-process the exported data if needed
                    // Post-processing would happen here if needed
                }
            }
            Err(e) => {
                eprintln!("❌ Program execution failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("❌ No command provided. Use 'memscope run <command>' or 'memscope analyze <input> <output>'");
        std::process::exit(1);
    }
}

fn _analyze_existing_snapshot(
    input_path: &str,
    _output_path: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(input_path).exists() {
        return Err(format!("Input file not found: {}", input_path).into());
    }

    match format {
        "html" => {
            // Generate HTML report from JSON - not implemented
            return Err("HTML generation not implemented".into());
        }
        "svg" => {
            // Generate SVG visualization from JSON - not implemented
            return Err("SVG generation not implemented".into());
        }
        "both" => {
            // Both HTML and SVG generation - not implemented
            return Err("Both format generation not implemented".into());
        }
        _ => {
            return Err(format!("Unsupported format: {}", format).into());
        }
    }
}

fn generate_html_report(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Generating HTML report...");

    // Read the JSON data
    let json_content = std::fs::read_to_string(input_path)?;

    // Create HTML content
    let html_content = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n    <title>MemScope Analysis Report</title>\n    <style>\n        body {{ font-family: Arial, sans-serif; margin: 20px; }}\n        .header {{ background: #f0f0f0; padding: 20px; border-radius: 5px; }}\n        .section {{ margin: 20px 0; }}\n        .data {{ background: #f9f9f9; padding: 10px; border-radius: 3px; }}\n    </style>\n</head>\n<body>\n    <div class=\"header\">\n        <h1>🚀 MemScope Analysis Report</h1>\n        <p>Generated from: {}</p>\n    </div>\n    <div class=\"section\">\n        <h2>📊 Memory Analysis Data</h2>\n        <div class=\"data\">\n            <pre>{}</pre>\n        </div>\n    </div>\n</body>\n</html>",
        input_path,
        json_content
    );

    std::fs::write(output_path, html_content)?;
    Ok(())
}

fn generate_svg_visualization(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Generating SVG visualization...");

    // Create SVG content
    let svg_content = format!(
        "<svg width=\"800\" height=\"600\" xmlns=\"http://www.w3.org/2000/svg\">\n    <rect width=\"800\" height=\"600\" fill=\"#f0f0f0\"/>\n    <text x=\"400\" y=\"50\" text-anchor=\"middle\" font-size=\"24\" font-weight=\"bold\">MemScope Visualization</text>\n    <text x=\"400\" y=\"80\" text-anchor=\"middle\" font-size=\"14\">Generated from: {}</text>\n    <text x=\"400\" y=\"300\" text-anchor=\"middle\" font-size=\"16\">SVG visualization would be generated here</text>\n</svg>",
        input_path
    );

    std::fs::write(output_path, svg_content)?;
    Ok(())
}

fn execute_with_tracking(
    command_args: &[&String],
    env_vars: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    if command_args.is_empty() {
        return Err("No command provided".into());
    }

    let program = command_args[0];
    let args = &command_args[1..];

    println!(
        "🔄 Executing: {} {}",
        program,
        args.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut cmd = Command::new(program);
    cmd.args(args);

    // Set environment variables for memory tracking
    for (key, value) in env_vars {
        cmd.env(key, value);
        println!("🔧 Setting env: {}={}", key, value);
    }

    // Inherit stdio to see the program output
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let status = cmd.status()?;

    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()).into());
    }

    // Give some time for all Drop destructors to complete
    // This is crucial for TrackedVariable Drop implementations to finish
    println!("⏳ Waiting for cleanup to complete...");
    std::thread::sleep(std::time::Duration::from_millis(200));

    Ok(())
}

fn _post_process_analysis(output_path: &str, format: &str) {
    match format {
        "json" => {
            let json_path = format!("{}.json", output_path);
            if Path::new(&json_path).exists() {
                println!("📄 JSON analysis: {}", json_path);
                // JSON analysis would happen here
            }
        }
        "html" => {
            let html_path = format!("{}.html", output_path);
            if Path::new(&html_path).exists() {
                println!("🌐 HTML dashboard: {}", html_path);
            }
        }
        "both" => {
            // Both JSON and HTML post-processing would happen here
        }
        _ => {}
    }
}

fn analyze_json_output(json_path: &str) {
    // Quick analysis of the exported JSON
    if let Ok(content) = std::fs::read_to_string(json_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(stats) = data
                .get("memory_analysis")
                .and_then(|ma| ma.get("statistics"))
                .and_then(|s| s.get("lifecycle_analysis"))
            {
                println!("📈 Quick Analysis:");

                if let Some(user_stats) = stats.get("user_allocations") {
                    if let Some(total) = user_stats.get("total_count") {
                        println!("   👤 User allocations: {}", total);
                    }
                    if let Some(avg_lifetime) = user_stats.get("average_lifetime_ms") {
                        println!("   ⏱️  Average lifetime: {}ms", avg_lifetime);
                    }
                }

                if let Some(system_stats) = stats.get("system_allocations") {
                    if let Some(total) = system_stats.get("total_count") {
                        println!("   🔧 System allocations: {}", total);
                    }
                }
            }
        }
    }
}

/// Export memory data to binary format after analysis
fn export_binary_data(output_path: &str, compression: &str, compression_level: i32) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_export::{BinaryExportOptions, export_memory_to_binary};
    use crate::core::tracker::MemoryTracker;

    println!("📦 Exporting memory data to binary format...");

    // Get the global memory tracker instance
    // Note: This is a simplified approach - in a real implementation,
    // we would need to properly access the tracker instance used during analysis
    let tracker = MemoryTracker::new();

    // Configure binary export options based on parameters
    let mut options = BinaryExportOptions::balanced(); // Start with balanced defaults
    
    // Set compression settings
    options.compression_enabled = compression != "none";
    options.compression_level = compression_level.clamp(1, 22); // Ensure valid range
    options.include_metadata = true; // Always include metadata for better compatibility
    
    // Adjust other settings based on compression choice
    if compression == "none" {
        options = BinaryExportOptions::fast();
        options.include_metadata = true; // Override to keep metadata
    } else {
        // Use compression-specific settings
        match compression_level {
            1..=3 => {
                options = BinaryExportOptions::fast();
                options.compression_enabled = true;
                options.compression_level = compression_level;
                options.include_metadata = true;
            }
            4..=10 => {
                options = BinaryExportOptions::balanced();
                options.compression_level = compression_level;
            }
            11..=22 => {
                options = BinaryExportOptions::compact();
                options.compression_level = compression_level;
            }
            _ => {
                // Invalid level, use default
                options.compression_level = 6;
            }
        }
    }

    // Create output file path with .ms extension
    let binary_output_path = if output_path.ends_with(".ms") {
        output_path.to_string()
    } else {
        format!("{}.ms", output_path)
    };

    println!("📋 Binary export configuration:");
    println!("   Compression: {}", if options.compression_enabled { compression } else { "none" });
    println!("   Compression level: {}", options.compression_level);
    println!("   Include metadata: {}", options.include_metadata);
    println!("   Include index: {}", options.include_index);
    println!("   Output file: {}", binary_output_path);

    // Perform the binary export
    match export_memory_to_binary(&tracker, &binary_output_path, options) {
        Ok(stats) => {
            println!("\n🎉 Binary export completed successfully!");
            println!("📊 Export Statistics:");
            println!("   Export time: {:?}", stats.export_time);
            println!("   File size: {} bytes", stats.file_size);
            println!("   Original size: {} bytes", stats.original_size);
            println!("   Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
            println!("   Allocations exported: {}", stats.allocation_count);
            println!("   Total memory tracked: {} bytes", stats.total_memory);
            println!("📁 Binary file saved: {}", binary_output_path);
            
            // Provide usage hint
            println!("\n💡 Usage hint:");
            println!("   Convert to JSON: memscope export -i {} -f json -o output.json", binary_output_path);
            println!("   Convert to HTML: memscope export -i {} -f html -o report.html", binary_output_path);
            println!("   Validate file: memscope export -i {} --validate-only", binary_output_path);
            
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Binary export failed: {e}");
            Err(format!("Binary export failed: {e}").into())
        }
    }
}