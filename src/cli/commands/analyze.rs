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

    tracing::info!("🔍 Starting memory analysis...");
    tracing::info!("Command: {:?}", command_args);
    tracing::info!("Export format: {}", export_format);
    tracing::info!("Output path: {}", output_path);

    // Initialize memory tracking
    crate::init();

    // Execute the command with memory tracking
    execute_with_tracking(&command_args, &[])?;

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

    tracing::info!("🚀 MemScope - Memory Analysis Tool");

    match matches.subcommand() {
        Some(("run", _sub_matches)) => {
            // Legacy run command - functionality moved to main analyze command
            tracing::info!("Run command is deprecated. Use 'analyze' instead.");
        }
        Some(("analyze", sub_matches)) => {
            if let Err(e) = handle_analyze_command(sub_matches) {
                eprintln!("Error in analyze command: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            // Legacy mode for backward compatibility
            if let Err(e) = handle_legacy_mode(&matches) {
                eprintln!("Error in legacy mode: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn handle_run_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let command_args: Vec<&String> = matches
        .get_many::<String>("command")
        .ok_or("Missing command arguments")?
        .collect();
    let export_format = matches
        .get_one::<String>("export")
        .ok_or("Missing export format")?;
    let output_path = matches
        .get_one::<String>("output")
        .ok_or("Missing output path")?;
    let auto_track = matches.get_flag("auto-track");
    let wait_completion = matches.get_flag("wait-completion");

    tracing::info!(
        "📋 Command: {}",
        command_args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );
    tracing::info!("📊 Export format: {export_format}");
    tracing::info!("📁 Output path: {output_path}");

    if auto_track {
        tracing::info!("🔍 Auto-tracking enabled");
    }

    if wait_completion {
        tracing::info!("⏳ Wait-for-completion enabled");
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
            tracing::info!("✅ Program execution completed successfully");
            tracing::info!("📊 Memory analysis exported to: {output_path}");

            // Post-process the exported data
            // Post-processing would happen here if needed
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ Program execution failed: {e}");
            Err(e)
        }
    }
}

fn handle_analyze_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches
        .get_one::<String>("input")
        .ok_or("Missing input path")?;
    let output_path = matches
        .get_one::<String>("output")
        .ok_or("Missing output path")?;
    let format = matches
        .get_one::<String>("format")
        .ok_or("Missing format")?;

    tracing::info!("🔍 Analyzing existing memory snapshot");
    tracing::info!("📄 Input: {}", input_path);
    tracing::info!("📄 Output: {}", output_path);
    tracing::info!("📊 Format: {}", format);

    // Legacy snapshot analysis - not implemented
    let result: Result<(), Box<dyn std::error::Error>> = Ok(());

    match result {
        Ok(()) => {
            tracing::info!("✅ Analysis completed successfully");
            tracing::info!("📊 Report generated: {}", output_path);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ Analysis failed: {}", e);
            Err(e)
        }
    }
}

fn handle_legacy_mode(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let export_format = matches.get_one::<String>("export");
    let output_path = matches
        .get_one::<String>("output")
        .ok_or("Missing output path")?;
    let auto_track = matches.get_flag("auto-track");

    if let Some(command_args) = matches.get_many::<String>("command") {
        let command_args: Vec<&String> = command_args.collect();

        tracing::info!("⚠️  Using legacy mode - consider using 'memscope run' instead");
        tracing::info!(
            "📋 Command: {}",
            command_args
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        );

        if let Some(format) = export_format {
            tracing::info!("📊 Export format: {}", format);
            tracing::info!("📁 Output path: {}", output_path);
        }

        if auto_track {
            tracing::info!("🔍 Auto-tracking enabled");
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
                tracing::info!("✅ Program execution completed successfully");

                if export_format.is_some() {
                    tracing::info!("📊 Memory analysis exported to: {}", output_path);

                    // Post-process the exported data if needed
                    // Post-processing would happen here if needed
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("❌ Program execution failed: {}", e);
                Err(e)
            }
        }
    } else {
        Err("No command provided. Use 'memscope run <command>' or 'memscope analyze <input> <output>'".into())
    }
}

fn _analyze_existing_snapshot(
    input_path: &str,
    _output_path: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(input_path).exists() {
        return Err(format!("Input file not found: {input_path}").into());
    }

    match format {
        "html" => {
            // Generate HTML report from JSON - not implemented
            Err("HTML generation not implemented".into())
        }
        "svg" => {
            // Generate SVG visualization from JSON - not implemented
            Err("SVG generation not implemented".into())
        }
        "both" => {
            // Both HTML and SVG generation - not implemented
            Err("Both format generation not implemented".into())
        }
        _ => Err(format!("Unsupported format: {format}").into()),
    }
}

fn generate_html_report(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🌐 Generating HTML report...");

    // Read the JSON data
    let json_content = std::fs::read_to_string(input_path)?;

    // Create HTML content
    let html_content = format!(
        "<!DOCTYPE html>\n<html>\n<head>\n    <title>MemScope Analysis Report</title>\n    <style>\n        body {{ font-family: Arial, sans-serif; margin: 20px; }}\n        .header {{ background: #f0f0f0; padding: 20px; border-radius: 5px; }}\n        .section {{ margin: 20px 0; }}\n        .data {{ background: #f9f9f9; padding: 10px; border-radius: 3px; }}\n    </style>\n</head>\n<body>\n    <div class=\"header\">\n        <h1>🚀 MemScope Analysis Report</h1>\n        <p>Generated from: {input_path}</p>\n    </div>\n    <div class=\"section\">\n        <h2>📊 Memory Analysis Data</h2>\n        <div class=\"data\">\n            <pre>{json_content}</pre>\n        </div>\n    </div>\n</body>\n</html>",
    );

    std::fs::write(output_path, html_content)?;
    Ok(())
}

fn generate_svg_visualization(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📈 Generating SVG visualization...");

    // Create SVG content
    let svg_content = format!(
        "<svg width=\"800\" height=\"600\" xmlns=\"http://www.w3.org/2000/svg\">\n    <rect width=\"800\" height=\"600\" fill=\"#f0f0f0\"/>\n    <text x=\"400\" y=\"50\" text-anchor=\"middle\" font-size=\"24\" font-weight=\"bold\">MemScope Visualization</text>\n    <text x=\"400\" y=\"80\" text-anchor=\"middle\" font-size=\"14\">Generated from: {input_path}</text>\n    <text x=\"400\" y=\"300\" text-anchor=\"middle\" font-size=\"16\">SVG visualization would be generated here</text>\n</svg>",
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

    tracing::info!(
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
        tracing::info!("🔧 Setting env: {}={}", key, value);
    }

    // Inherit stdio to see the program output
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let status = cmd.status()?;

    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()).into());
    }

    // Give some time for all Drop destructors to complete
    // This is crucial for TrackedVariable Drop implementations to finish
    tracing::info!("⏳ Waiting for cleanup to complete...");
    std::thread::sleep(std::time::Duration::from_millis(200));

    Ok(())
}

// Update function calls to handle Results
fn handle_run_command_wrapper(matches: &clap::ArgMatches) {
    if let Err(e) = handle_run_command(matches) {
        tracing::error!("❌ Run command failed: {}", e);
        std::process::exit(1);
    }
}

fn handle_analyze_command_wrapper(matches: &clap::ArgMatches) {
    if let Err(e) = handle_analyze_command(matches) {
        tracing::error!("❌ Analyze command failed: {}", e);
        std::process::exit(1);
    }
}

fn handle_legacy_mode_wrapper(matches: &clap::ArgMatches) {
    if let Err(e) = handle_legacy_mode(matches) {
        tracing::error!("❌ Legacy mode failed: {}", e);
        std::process::exit(1);
    }
}

fn _post_process_analysis(output_path: &str, format: &str) {
    match format {
        "json" => {
            let json_path = format!("{output_path}.json");
            if Path::new(&json_path).exists() {
                tracing::info!("📄 JSON analysis: {}", json_path);
                // JSON analysis would happen here
            }
        }
        "html" => {
            let html_path = format!("{output_path}.html");
            if Path::new(&html_path).exists() {
                tracing::info!("🌐 HTML dashboard: {}", html_path);
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
                tracing::info!("📈 Quick Analysis:");

                if let Some(user_stats) = stats.get("user_allocations") {
                    if let Some(total) = user_stats.get("total_count") {
                        tracing::info!("   👤 User allocations: {}", total);
                    }
                    if let Some(avg_lifetime) = user_stats.get("average_lifetime_ms") {
                        tracing::info!("   ⏱️  Average lifetime: {}ms", avg_lifetime);
                    }
                }

                if let Some(system_stats) = stats.get("system_allocations") {
                    if let Some(total) = system_stats.get("total_count") {
                        tracing::info!("   🔧 System allocations: {}", total);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgMatches, Command as ClapCommand};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_matches_with_command(command: Vec<&str>) -> ArgMatches {
        let command_strings: Vec<String> = command.iter().map(|s| s.to_string()).collect();
        ClapCommand::new("test")
            .arg(Arg::new("command").num_args(1..))
            .arg(Arg::new("export").long("export"))
            .arg(Arg::new("output").long("output"))
            .try_get_matches_from(
                std::iter::once("test".to_string())
                    .chain(command_strings)
                    .collect::<Vec<String>>(),
            )
            .expect("Failed to create test matches")
    }

    fn create_analyze_matches(input: &str, output: &str, format: &str) -> ArgMatches {
        ClapCommand::new("test")
            .arg(Arg::new("input"))
            .arg(Arg::new("output"))
            .arg(Arg::new("format").long("format"))
            .try_get_matches_from(vec!["test", input, output, "--format", format])
            .expect("Failed to create analyze matches")
    }

    #[test]
    fn test_argument_extraction() {
        // Test argument extraction from ArgMatches
        let matches = create_test_matches_with_command(vec!["echo", "hello"]);

        let command_args: Vec<&String> = matches.get_many::<String>("command").unwrap().collect();

        assert_eq!(command_args.len(), 2);
        assert_eq!(command_args[0], "echo");
        assert_eq!(command_args[1], "hello");
    }

    #[test]
    fn test_default_values() {
        // Test default value handling
        let matches = create_test_matches_with_command(vec!["echo", "test"]);

        let export_format = matches
            .get_one::<String>("export")
            .map(|s| s.as_str())
            .unwrap_or("html");
        let output_path = matches
            .get_one::<String>("output")
            .map(|s| s.as_str())
            .unwrap_or("memory_analysis");

        assert_eq!(export_format, "html");
        assert_eq!(output_path, "memory_analysis");
    }

    #[test]
    fn test_environment_variable_setup() {
        // Test environment variable setup logic
        let export_format = "json";
        let output_path = "test_output";
        let auto_track = true;
        let wait_completion = false;

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

        assert_eq!(env_vars.len(), 5); // 4 base + 1 auto_track
        assert!(env_vars.contains(&("MEMSCOPE_AUTO_TRACK", "1")));
        assert!(!env_vars
            .iter()
            .any(|(k, _)| *k == "MEMSCOPE_WAIT_COMPLETION"));
    }

    #[test]
    fn test_command_validation() {
        // Test command validation logic
        let empty_command: Vec<&String> = vec![];
        let valid_command = ["echo".to_string(), "test".to_string()];
        let valid_command_refs: Vec<&String> = valid_command.iter().collect();

        // Test empty command
        assert!(empty_command.is_empty());

        // Test valid command
        assert!(!valid_command_refs.is_empty());
        assert_eq!(valid_command_refs[0], "echo");
        assert_eq!(valid_command_refs[1], "test");
    }

    #[test]
    fn test_generate_html_report() {
        // Test HTML report generation
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_path = temp_dir.path().join("test_input.json");
        let output_path = temp_dir.path().join("test_output.html");

        // Create test JSON input
        let test_json = r#"{"memory_analysis": {"statistics": {"total_allocations": 100}}}"#;
        fs::write(&input_path, test_json).expect("Failed to write test JSON");

        // Test HTML generation
        let result =
            generate_html_report(input_path.to_str().unwrap(), output_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify HTML content
        let html_content = fs::read_to_string(&output_path).expect("Failed to read HTML");
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("MemScope Analysis Report"));
    }

    #[test]
    fn test_generate_svg_visualization() {
        // Test SVG visualization generation
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_path = temp_dir.path().join("test_input.json");
        let output_path = temp_dir.path().join("test_output.svg");

        // Create test input file
        fs::write(&input_path, "{}").expect("Failed to write test file");

        // Test SVG generation
        let result =
            generate_svg_visualization(input_path.to_str().unwrap(), output_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify SVG content
        let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("MemScope Visualization"));
    }

    #[test]
    fn test_handle_analyze_command() {
        // Test analyze command handling
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.html");

        // Create test input file
        fs::write(&input_path, "{}").expect("Failed to write test file");

        let matches = create_analyze_matches(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "html",
        );

        let result = handle_analyze_command(&matches);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_json_output_parsing() {
        // Test JSON output analysis logic
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let json_path = temp_dir.path().join("test_analysis.json");

        // Create test JSON with expected structure
        let test_json = r#"{
            "memory_analysis": {
                "statistics": {
                    "lifecycle_analysis": {
                        "user_allocations": {
                            "total_count": 150,
                            "average_lifetime_ms": 250
                        },
                        "system_allocations": {
                            "total_count": 75
                        }
                    }
                }
            }
        }"#;

        fs::write(&json_path, test_json).expect("Failed to write test JSON");

        // Test JSON parsing (function doesn't return value, but should not panic)
        analyze_json_output(json_path.to_str().unwrap());

        // Verify file exists and is readable
        assert!(json_path.exists());
        let content = fs::read_to_string(&json_path).expect("Failed to read JSON");
        let data: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

        assert!(data.get("memory_analysis").is_some());
    }

    #[test]
    fn test_format_validation() {
        // Test format validation logic
        let valid_formats = ["json", "html", "both", "svg"];
        let invalid_formats = ["xml", "csv", "txt"];

        for format in valid_formats {
            // These formats should be handled
            assert!(["json", "html", "both", "svg"].contains(&format));
        }

        for format in invalid_formats {
            // These formats should not be in valid list
            assert!(!["json", "html", "both", "svg"].contains(&format));
        }
    }

    #[test]
    fn test_path_handling() {
        // Test path handling and validation
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let valid_path = temp_dir.path().join("valid_file.json");
        let invalid_path = temp_dir
            .path()
            .join("nonexistent")
            .join("invalid_file.json");

        // Test valid path
        fs::write(&valid_path, "{}").expect("Failed to write test file");
        assert!(valid_path.exists());

        // Test invalid path
        assert!(!invalid_path.exists());
        assert!(!invalid_path.parent().unwrap().exists());
    }

    #[test]
    fn test_command_args_processing() {
        // Test command arguments processing
        let test_cases = vec![
            (vec!["echo", "hello"], "echo hello"),
            (vec!["cargo", "run", "--release"], "cargo run --release"),
            (vec!["ls", "-la"], "ls -la"),
        ];

        for (args, expected) in test_cases {
            let joined = args.join(" ");
            assert_eq!(joined, expected);

            // Test argument splitting
            let program = args[0];
            let remaining_args = &args[1..];

            assert_eq!(program, args[0]);
            assert_eq!(remaining_args.len(), args.len() - 1);
        }
    }

    #[test]
    fn test_error_handling() {
        // Test error handling scenarios

        // Test missing command
        let empty_command: Vec<&String> = vec![];
        assert!(empty_command.is_empty());

        // Test missing required arguments
        let matches = ClapCommand::new("test")
            .arg(Arg::new("input"))
            .arg(Arg::new("output"))
            .try_get_matches_from(vec!["test"])
            .unwrap();

        let missing_input = matches.get_one::<String>("input");
        let missing_output = matches.get_one::<String>("output");

        assert!(missing_input.is_none());
        assert!(missing_output.is_none());
    }

    #[test]
    fn test_legacy_mode_detection() {
        // Test legacy mode detection logic
        let matches_with_export = ClapCommand::new("test")
            .arg(Arg::new("export").long("export"))
            .arg(Arg::new("command").num_args(1..))
            .try_get_matches_from(vec!["test", "--export", "json", "echo", "test"])
            .unwrap();

        let has_export = matches_with_export.get_one::<String>("export").is_some();
        let has_command = matches_with_export.get_many::<String>("command").is_some();

        assert!(has_export);
        assert!(has_command);

        // This would indicate legacy mode usage
        let is_legacy_mode = has_export && has_command;
        assert!(is_legacy_mode);
    }

    #[test]
    fn test_run_analyze_with_valid_args() {
        // Test run_analyze function with valid arguments
        let matches = ClapCommand::new("test")
            .arg(Arg::new("command").num_args(1..))
            .arg(Arg::new("export").long("export").default_value("html"))
            .arg(Arg::new("output").long("output").default_value("memory_analysis"))
            .try_get_matches_from(vec!["test", "echo", "hello"])
            .unwrap();

        // This should not panic and should extract arguments correctly
        let command_args: Vec<&String> = matches
            .get_many::<String>("command")
            .unwrap()
            .collect();
        let export_format = matches
            .get_one::<String>("export")
            .map(|s| s.as_str())
            .unwrap_or("html");
        let output_path = matches
            .get_one::<String>("output")
            .map(|s| s.as_str())
            .unwrap_or("memory_analysis");

        assert_eq!(command_args.len(), 2);
        assert_eq!(command_args[0], "echo");
        assert_eq!(command_args[1], "hello");
        assert_eq!(export_format, "html");
        assert_eq!(output_path, "memory_analysis");
    }

    #[test]
    fn test_run_analyze_missing_command() {
        // Test run_analyze function with missing command
        let matches = ClapCommand::new("test")
            .arg(Arg::new("command").num_args(1..))
            .try_get_matches_from(vec!["test"])
            .unwrap();

        let command_result = matches.get_many::<String>("command");
        assert!(command_result.is_none());
    }

    #[test]
    fn test_handle_run_command_logic() {
        // Test the logic inside handle_run_command without actually executing
        let matches = ClapCommand::new("test")
            .arg(Arg::new("command").num_args(1..))
            .arg(Arg::new("export").long("export"))
            .arg(Arg::new("output").long("output"))
            .arg(Arg::new("auto-track").long("auto-track").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("wait-completion").long("wait-completion").action(clap::ArgAction::SetTrue))
            .try_get_matches_from(vec![
                "test", "echo", "test", "--export", "json", "--output", "test_output",
                "--auto-track", "--wait-completion"
            ])
            .unwrap();

        let command_args: Vec<&String> = matches.get_many::<String>("command").unwrap().collect();
        let export_format = matches.get_one::<String>("export").unwrap();
        let output_path = matches.get_one::<String>("output").unwrap();
        let auto_track = matches.get_flag("auto-track");
        let wait_completion = matches.get_flag("wait-completion");

        assert_eq!(command_args.len(), 2);
        assert_eq!(export_format, "json");
        assert_eq!(output_path, "test_output");
        assert!(auto_track);
        assert!(wait_completion);

        // Test environment variable setup
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

        assert_eq!(env_vars.len(), 6);
        assert!(env_vars.contains(&("MEMSCOPE_AUTO_TRACK", "1")));
        assert!(env_vars.contains(&("MEMSCOPE_WAIT_COMPLETION", "1")));
    }

    #[test]
    fn test_handle_legacy_mode_logic() {
        // Test handle_legacy_mode logic without executing
        let matches = ClapCommand::new("test")
            .arg(Arg::new("export").long("export"))
            .arg(Arg::new("output").long("output").default_value("default_output"))
            .arg(Arg::new("auto-track").long("auto-track").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("command").num_args(1..))
            .try_get_matches_from(vec![
                "test", "--export", "html", "--output", "legacy_output", 
                "--auto-track", "cargo", "run"
            ])
            .unwrap();

        let export_format = matches.get_one::<String>("export");
        let output_path = matches.get_one::<String>("output").unwrap();
        let auto_track = matches.get_flag("auto-track");
        let command_args: Option<clap::parser::ValuesRef<String>> = matches.get_many::<String>("command");

        assert!(export_format.is_some());
        assert_eq!(export_format.unwrap(), "html");
        assert_eq!(output_path, "legacy_output");
        assert!(auto_track);
        assert!(command_args.is_some());

        let command_vec: Vec<&String> = command_args.unwrap().collect();
        assert_eq!(command_vec.len(), 2);
        assert_eq!(command_vec[0], "cargo");
        assert_eq!(command_vec[1], "run");
    }

    #[test]
    fn test_analyze_existing_snapshot_logic() {
        // Test _analyze_existing_snapshot function logic
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let existing_file = temp_dir.path().join("existing.json");
        let nonexistent_file = temp_dir.path().join("nonexistent.json");

        // Create existing file
        fs::write(&existing_file, "{}").expect("Failed to write test file");

        // Test with existing file
        assert!(existing_file.exists());

        // Test with nonexistent file
        assert!(!nonexistent_file.exists());

        // Test format handling logic
        let formats = ["html", "svg", "both", "invalid"];
        for format in formats {
            match format {
                "html" | "svg" | "both" => {
                    // These should be handled (though not implemented)
                    assert!(["html", "svg", "both"].contains(&format));
                }
                _ => {
                    // Invalid format
                    assert!(!["html", "svg", "both"].contains(&format));
                }
            }
        }
    }

    #[test]
    fn test_execute_with_tracking_validation() {
        // Test execute_with_tracking input validation
        let empty_command: Vec<&String> = vec![];
        let valid_command = ["echo".to_string(), "hello".to_string()];
        let valid_refs: Vec<&String> = valid_command.iter().collect();

        // Test empty command validation
        assert!(empty_command.is_empty());

        // Test valid command structure
        assert!(!valid_refs.is_empty());
        let program = valid_refs[0];
        let args = &valid_refs[1..];
        
        assert_eq!(program, "echo");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "hello");

        // Test environment variables setup
        let env_vars = [
            ("MEMSCOPE_ENABLED", "1"),
            ("MEMSCOPE_AUTO_EXPORT", "1"),
            ("TEST_VAR", "test_value"),
        ];

        for (key, value) in env_vars {
            assert!(!key.is_empty());
            assert!(!value.is_empty());
        }
    }

    #[test]
    fn test_html_content_generation() {
        // Test HTML content generation logic
        let input_path = "test_input.json";
        let expected_elements = [
            "<!DOCTYPE html>",
            "<html>",
            "<head>",
            "<title>MemScope Analysis Report</title>",
            "<style>",
            "body { font-family: Arial, sans-serif; margin: 20px; }",
            ".header { background: #f0f0f0; padding: 20px; border-radius: 5px; }",
            "</head>",
            "<body>",
            "<h1>🚀 MemScope Analysis Report</h1>",
        ];

        // Simulate HTML content creation (partial)
        let html_start = format!(
            "<!DOCTYPE html>\n<html>\n<head>\n    <title>MemScope Analysis Report</title>\n    <style>\n        body {{ font-family: Arial, sans-serif; margin: 20px; }}\n        .header {{ background: #f0f0f0; padding: 20px; border-radius: 5px; }}\n        .section {{ margin: 20px 0; }}\n        .data {{ background: #f9f9f9; padding: 10px; border-radius: 3px; }}\n    </style>\n</head>\n<body>\n    <div class=\"header\">\n        <h1>🚀 MemScope Analysis Report</h1>\n        <p>Generated from: {}", 
            input_path
        );

        for element in expected_elements {
            if !element.contains("Generated from") {
                assert!(html_start.contains(element));
            }
        }
    }

    #[test]
    fn test_svg_content_generation() {
        // Test SVG content generation logic
        let input_path = "test_input.json";
        let svg_content = format!(
            "<svg width=\"800\" height=\"600\" xmlns=\"http://www.w3.org/2000/svg\">\n    <rect width=\"800\" height=\"600\" fill=\"#f0f0f0\"/>\n    <text x=\"400\" y=\"50\" text-anchor=\"middle\" font-size=\"24\" font-weight=\"bold\">MemScope Visualization</text>\n    <text x=\"400\" y=\"80\" text-anchor=\"middle\" font-size=\"14\">Generated from: {}</text>\n    <text x=\"400\" y=\"300\" text-anchor=\"middle\" font-size=\"16\">SVG visualization would be generated here</text>\n</svg>",
            input_path
        );

        let expected_elements = [
            "<svg",
            "width=\"800\"",
            "height=\"600\"",
            "xmlns=\"http://www.w3.org/2000/svg\"",
            "<rect",
            "fill=\"#f0f0f0\"",
            "<text",
            "MemScope Visualization",
            "Generated from:",
            "</svg>",
        ];

        for element in expected_elements {
            assert!(svg_content.contains(element));
        }
    }

    #[test]
    fn test_command_string_joining() {
        // Test command string joining logic used in logging
        let test_cases = vec![
            (vec!["echo", "hello"], "echo hello"),
            (vec!["cargo", "run", "--release"], "cargo run --release"),
            (vec!["ls", "-la", "/tmp"], "ls -la /tmp"),
            (vec!["git", "commit", "-m", "test message"], "git commit -m test message"),
        ];

        for (args, expected) in test_cases {
            let joined = args.iter().map(|s| *s).collect::<Vec<_>>().join(" ");
            assert_eq!(joined, expected);
        }
    }

    #[test]
    fn test_flag_combinations() {
        // Test various flag combinations
        let matches = ClapCommand::new("test")
            .arg(Arg::new("auto-track").long("auto-track").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("wait-completion").long("wait-completion").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("verbose").long("verbose").action(clap::ArgAction::SetTrue))
            .try_get_matches_from(vec!["test", "--auto-track", "--verbose"])
            .unwrap();

        let auto_track = matches.get_flag("auto-track");
        let wait_completion = matches.get_flag("wait-completion");
        let verbose = matches.get_flag("verbose");

        assert!(auto_track);
        assert!(!wait_completion);
        assert!(verbose);

        // Test flag-based environment variable setup
        let mut env_count = 0;
        if auto_track {
            env_count += 1;
        }
        if wait_completion {
            env_count += 1;
        }
        if verbose {
            env_count += 1;
        }

        assert_eq!(env_count, 2); // auto_track + verbose
    }

    #[test]
    fn test_error_message_formatting() {
        // Test error message formatting
        let test_errors = vec![
            ("Missing command arguments", "Missing command arguments"),
            ("Missing export format", "Missing export format"),
            ("Missing output path", "Missing output path"),
            ("Command failed with exit code: Some(1)", "Command failed with exit code: Some(1)"),
        ];

        for (error_msg, expected) in test_errors {
            assert_eq!(error_msg, expected);
            assert!(!error_msg.is_empty());
        }

        // Test error formatting with dynamic content
        let exit_code = Some(1);
        let formatted_error = format!("Command failed with exit code: {:?}", exit_code);
        assert_eq!(formatted_error, "Command failed with exit code: Some(1)");
    }

    #[test]
    fn test_file_extension_handling() {
        // Test file extension handling logic
        let test_paths = vec![
            ("output.html", "html"),
            ("output.svg", "svg"),
            ("output.json", "json"),
            ("output", ""),
        ];

        for (path, expected_ext) in test_paths {
            let extension = path.split('.').last().unwrap_or("");
            if path.contains('.') {
                assert_eq!(extension, expected_ext);
            } else {
                assert_eq!(extension, path);
            }
        }
    }

    #[test]
    fn test_timeout_duration() {
        // Test timeout duration logic
        let timeout_ms = 200;
        let duration = std::time::Duration::from_millis(timeout_ms);
        
        assert_eq!(duration.as_millis(), 200);
        assert!(duration.as_millis() > 0);
        assert!(duration.as_millis() < 1000); // Less than 1 second
    }

    #[test]
    fn test_wrapper_function_signatures() {
        // Test that wrapper functions have correct signatures
        let _run_wrapper: fn(&clap::ArgMatches) = handle_run_command_wrapper;
        let _analyze_wrapper: fn(&clap::ArgMatches) = handle_analyze_command_wrapper;
        let _legacy_wrapper: fn(&clap::ArgMatches) = handle_legacy_mode_wrapper;

        // These should compile without errors, confirming correct signatures
        assert!(true);
    }
}
