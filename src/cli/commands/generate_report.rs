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
            return Err(format!("Unsupported format: {}", format).into());
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
