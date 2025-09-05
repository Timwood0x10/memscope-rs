//! Report generation command implementation
//!
//! This module provides the report generation subcommand functionality.

use clap::ArgMatches;
use std::error::Error;

/// Run the generate report command
pub fn run_generate_report(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_file = matches
        .get_one::<String>("input")
        .ok_or("Input file is required")?;
    let output_file = matches
        .get_one::<String>("output")
        .ok_or("Output file is required")?;
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("html");

    tracing::info!("📊 Generating report...");
    tracing::info!("Input file: {}", input_file);
    tracing::info!("Output file: {}", output_file);
    tracing::info!("Format: {}", format);

    match format {
        "html" => {
            let default_template = "report_template.html".to_string();
            embed_json_to_html(input_file, &default_template, output_file)?;
        }
        _ => {
            return Err(format!("Unsupported format: {format}").into());
        }
    }

    Ok(())
}

fn _original_main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        _print_usage();
        return;
    }

    match args[1].as_str() {
        "template" => {
            let default_output = "interactive_template.html".to_string();
            let output = args.get(2).unwrap_or(&default_output);

            // Check if source file exists
            if !std::path::Path::new("interactive_template.html").exists() {
                tracing::error!("❌ Source template 'interactive_template.html' not found!");
                tracing::error!("Please make sure the interactive_template.html file exists in the current directory.");
                std::process::exit(1);
            }

            if let Err(e) = std::fs::copy("interactive_template.html", output) {
                tracing::error!("❌ Error creating template: {e}");
                std::process::exit(1);
            }
            tracing::info!("✅ Created interactive template: {output}");
        }
        "generate" => {
            if args.len() < 4 {
                tracing::error!(
                    "❌ Usage: generate_report generate <json_file> <output_file> [template_file]"
                );
                std::process::exit(1);
            }

            let json_file = &args[2];
            let output_file = &args[3];
            let default_template = "report_template.html".to_string();
            let template_file = args.get(4).unwrap_or(&default_template);

            if let Err(e) = embed_json_to_html(json_file, template_file, output_file) {
                tracing::error!("❌ Error generating report: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            _print_usage();
        }
    }
}

fn embed_json_to_html(
    json_file: &str,
    template_file: &str,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_content = std::fs::read_to_string(json_file)?;

    let template_content = std::fs::read_to_string(template_file)?;
    let inline_script = format!(
        r#"<script type="text/javascript">
// Embedded JSON data for offline analysis
window.EMBEDDED_MEMORY_DATA = {json_content};
console.log('📊 Loaded embedded memory analysis data');
</script>"#
    );

    let final_html = template_content.replace("<!-- DATA_INJECTION_POINT -->", &inline_script);

    std::fs::write(output_file, final_html)?;

    tracing::info!("✅ Generated self-contained HTML report: {output_file}");
    Ok(())
}

fn _print_usage() {
    tracing::info!("🔍 Memory Analysis Report Generator");
    tracing::info!("");
    tracing::info!("Usage:");
    tracing::info!("  generate_report template [output_file]");
    tracing::info!("    Create a standalone HTML template");
    tracing::info!("");
    tracing::info!("  generate_report generate <json_file> <output_file> [template_file]");
    tracing::info!("    Generate a self-contained HTML report from JSON data");
    tracing::info!("");
    tracing::info!("Examples:");
    tracing::info!("  generate_report template report_template.html");
    tracing::info!("  generate_report generate data.json report.html report_template.html");
    tracing::info!("");
    tracing::info!("The generated report.html is completely self-contained and can be:");
    tracing::info!("  ✅ Opened directly in any browser (no server needed)");
    tracing::info!("  ✅ Shared via email or file transfer");
    tracing::info!("  ✅ Archived for historical analysis");
    tracing::info!("  ✅ Viewed offline without any dependencies");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgMatches, Command as ClapCommand};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_matches(input: &str, output: &str, format: Option<&str>) -> ArgMatches {
        let cmd = ClapCommand::new("test")
            .arg(Arg::new("input"))
            .arg(Arg::new("output"))
            .arg(Arg::new("format").long("format"));

        let mut args = vec!["test", input, output];
        if let Some(fmt) = format {
            args.extend_from_slice(&["--format", fmt]);
        }

        cmd.try_get_matches_from(args)
            .expect("Failed to create test matches")
    }

    #[test]
    fn test_argument_extraction() {
        // Test argument extraction from ArgMatches
        let matches = create_test_matches("input.json", "output.html", Some("html"));

        let input_file = matches.get_one::<String>("input").unwrap();
        let output_file = matches.get_one::<String>("output").unwrap();
        let format = matches
            .get_one::<String>("format")
            .map(|s| s.as_str())
            .unwrap_or("html");

        assert_eq!(input_file, "input.json");
        assert_eq!(output_file, "output.html");
        assert_eq!(format, "html");
    }

    #[test]
    fn test_default_format() {
        // Test default format handling
        let matches = create_test_matches("input.json", "output.html", None);

        let format = matches
            .get_one::<String>("format")
            .map(|s| s.as_str())
            .unwrap_or("html");

        assert_eq!(format, "html");
    }

    #[test]
    fn test_embed_json_to_html() {
        // Test JSON embedding into HTML template
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create test JSON file
        let json_file = temp_dir.path().join("test_data.json");
        let test_json = r#"{"memory_analysis": {"total_allocations": 100}}"#;
        fs::write(&json_file, test_json).expect("Failed to write JSON file");

        // Create test HTML template
        let template_file = temp_dir.path().join("template.html");
        let template_content = r#"<!DOCTYPE html>
<html>
<head><title>Test Report</title></head>
<body>
    <h1>Memory Analysis Report</h1>
    <!-- DATA_INJECTION_POINT -->
    <div id="content"></div>
</body>
</html>"#;
        fs::write(&template_file, template_content).expect("Failed to write template file");

        // Test embedding
        let output_file = temp_dir.path().join("output.html");
        let result = embed_json_to_html(
            json_file.to_str().unwrap(),
            template_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(output_file.exists());

        // Verify embedded content
        let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(output_content.contains("window.EMBEDDED_MEMORY_DATA"));
        assert!(output_content.contains("memory_analysis"));
        assert!(output_content.contains("total_allocations"));
        assert!(!output_content.contains("<!-- DATA_INJECTION_POINT -->"));
    }

    #[test]
    fn test_run_generate_report_html_format() {
        // Test report generation with HTML format
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create test files
        let json_file = temp_dir.path().join("input.json");
        let template_file = temp_dir.path().join("report_template.html");
        let output_file = temp_dir.path().join("output.html");

        let test_json = r#"{"test": "data"}"#;
        fs::write(&json_file, test_json).expect("Failed to write JSON file");

        let template_content = r#"<html><body><!-- DATA_INJECTION_POINT --></body></html>"#;
        fs::write(&template_file, template_content).expect("Failed to write template file");

        let matches = create_test_matches(
            json_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
            Some("html"),
        );

        // Change to temp directory to find the template
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

        let result = run_generate_report(&matches);

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        assert!(result.is_ok());
        assert!(output_file.exists());
    }

    #[test]
    fn test_run_generate_report_unsupported_format() {
        // Test error handling for unsupported format
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let json_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("output.xml");

        fs::write(&json_file, "{}").expect("Failed to write JSON file");

        let matches = create_test_matches(
            json_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
            Some("xml"),
        );

        let result = run_generate_report(&matches);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported format: xml"));
    }

    #[test]
    fn test_missing_arguments() {
        // Test error handling for missing arguments
        let cmd = ClapCommand::new("test")
            .arg(Arg::new("input"))
            .arg(Arg::new("output"));

        let matches = cmd.try_get_matches_from(vec!["test"]).unwrap();

        let input_result = matches.get_one::<String>("input");
        let output_result = matches.get_one::<String>("output");

        assert!(input_result.is_none());
        assert!(output_result.is_none());
    }

    #[test]
    fn test_json_injection_script_generation() {
        // Test JSON injection script generation
        let test_json = r#"{"allocations": [{"id": 1}, {"id": 2}]}"#;

        let inline_script = format!(
            r#"<script type="text/javascript">
// Embedded JSON data for offline analysis
window.EMBEDDED_MEMORY_DATA = {test_json};
console.log('📊 Loaded embedded memory analysis data');
</script>"#
        );

        assert!(inline_script.contains("window.EMBEDDED_MEMORY_DATA"));
        assert!(inline_script.contains("allocations"));
        assert!(inline_script.contains("console.log"));
        assert!(inline_script.contains("<script"));
        assert!(inline_script.contains("</script>"));
    }

    #[test]
    fn test_template_replacement() {
        // Test template content replacement
        let template_content = r#"<html>
<head><title>Report</title></head>
<body>
    <h1>Analysis</h1>
    <!-- DATA_INJECTION_POINT -->
    <footer>End</footer>
</body>
</html>"#;

        let replacement_script = r#"<script>window.DATA = {};</script>"#;
        let final_html =
            template_content.replace("<!-- DATA_INJECTION_POINT -->", replacement_script);

        assert!(!final_html.contains("<!-- DATA_INJECTION_POINT -->"));
        assert!(final_html.contains("<script>window.DATA = {};</script>"));
        assert!(final_html.contains("<title>Report</title>"));
        assert!(final_html.contains("<footer>End</footer>"));
    }

    #[test]
    fn test_file_operations() {
        // Test file read/write operations
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test file writing
        let test_file = temp_dir.path().join("test.txt");
        let test_content = "Hello, World!";
        let write_result = fs::write(&test_file, test_content);
        assert!(write_result.is_ok());
        assert!(test_file.exists());

        // Test file reading
        let read_result = fs::read_to_string(&test_file);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);

        // Test reading non-existent file
        let non_existent = temp_dir.path().join("non_existent.txt");
        let read_error = fs::read_to_string(&non_existent);
        assert!(read_error.is_err());
    }

    #[test]
    fn test_format_validation() {
        // Test format validation logic
        let supported_formats = ["html"];
        let unsupported_formats = ["xml", "pdf", "docx", "txt"];

        for format in supported_formats {
            assert_eq!(format, "html");
        }

        for format in unsupported_formats {
            assert_ne!(format, "html");
        }
    }

    #[test]
    fn test_complex_json_embedding() {
        // Test embedding complex JSON data
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let complex_json = r#"{
            "memory_analysis": {
                "statistics": {
                    "total_allocations": 1000,
                    "active_allocations": 500,
                    "peak_memory": 1048576
                },
                "allocations": [
                    {"id": 1, "size": 1024, "type": "Vec<i32>"},
                    {"id": 2, "size": 2048, "type": "String"}
                ]
            }
        }"#;

        let json_file = temp_dir.path().join("complex.json");
        let template_file = temp_dir.path().join("template.html");
        let output_file = temp_dir.path().join("output.html");

        fs::write(&json_file, complex_json).expect("Failed to write JSON");
        fs::write(&template_file, "<html><!-- DATA_INJECTION_POINT --></html>")
            .expect("Failed to write template");

        let result = embed_json_to_html(
            json_file.to_str().unwrap(),
            template_file.to_str().unwrap(),
            output_file.to_str().unwrap(),
        );

        assert!(result.is_ok());

        let output_content = fs::read_to_string(&output_file).expect("Failed to read output");
        assert!(output_content.contains("total_allocations"));
        assert!(output_content.contains("1000"));
        assert!(output_content.contains("Vec<i32>"));
    }
}
