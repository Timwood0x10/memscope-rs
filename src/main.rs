//! CLI entry point for memscope-rs
//!
//! This provides a unified command-line interface for memory analysis tools.

use clap::{Arg, Command};
use std::process;

fn main() {
    let matches = Command::new("memscope")
        .version("0.1.2")
        .author("TimWood")
        .about("Advanced Rust memory analysis and visualization toolkit")
        .subcommand(
            Command::new("analyze")
                .about("Analyze memory usage of a Rust program")
                .arg(
                    Arg::new("command")
                        .help("Command to execute and analyze")
                        .required(true)
                        .num_args(1..)
                        .value_name("COMMAND"),
                )
                .arg(
                    Arg::new("export")
                        .short('e')
                        .long("export")
                        .value_name("FORMAT")
                        .help("Export format (json, svg, html, binary)")
                        .default_value("html"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .default_value("memory_analysis"),
                ),
        )
        .subcommand(
            Command::new("report")
                .about("Generate memory analysis report from existing data")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input JSON file with memory data")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .required(true),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format (html, svg)")
                        .default_value("html"),
                ),
        )
        .subcommand(
            Command::new("html-from-json")
                .about("Generate interactive HTML report from exported JSON files")
                .arg(
                    Arg::new("input-dir")
                        .short('i')
                        .long("input-dir")
                        .value_name("DIR")
                        .help("Directory containing JSON export files")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output HTML file path")
                        .required_unless_present("validate-only"),
                )
                .arg(
                    Arg::new("base-name")
                        .short('b')
                        .long("base-name")
                        .value_name("NAME")
                        .help("Base name for JSON files (e.g., 'snapshot' for snapshot_memory_analysis.json)")
                        .default_value("snapshot"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Enable verbose output with detailed progress information")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .long("debug")
                        .help("Enable debug mode with detailed logging and timing information")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("performance")
                        .long("performance")
                        .help("Enable performance analysis mode with comprehensive timing and memory tracking")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("validate-only")
                        .long("validate-only")
                        .help("Only validate JSON files without generating HTML")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("test").about("Run enhanced memory tests").arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Output file path")
                    .default_value("enhanced_memory_test"),
            ),
        )
        .subcommand(
            Command::new("convert")
                .about("Convert between different memory analysis formats")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input file path")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .required(true),
                )
                .arg(
                    Arg::new("from")
                        .short('f')
                        .long("from")
                        .value_name("FORMAT")
                        .help("Input format (binary, json)")
                        .required(true),
                )
                .arg(
                    Arg::new("to")
                        .short('t')
                        .long("to")
                        .value_name("FORMAT")
                        .help("Output format (json, html, binary)")
                        .required(true),
                )
                .arg(
                    Arg::new("validate")
                        .long("validate")
                        .help("Validate conversion result")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Enable verbose output")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("binary-info")
                .about("Display information about binary memory analysis files")
                .arg(
                    Arg::new("input")
                        .help("Binary file path")
                        .required(true)
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("detailed")
                        .short('d')
                        .long("detailed")
                        .help("Show detailed information")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("sections")
                        .short('s')
                        .long("sections")
                        .help("Show section information")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("stats")
                        .long("stats")
                        .help("Show statistics")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("binary-validate")
                .about("Validate binary memory analysis files")
                .arg(
                    Arg::new("input")
                        .help("Binary file path")
                        .required(true)
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("comprehensive")
                        .short('c')
                        .long("comprehensive")
                        .help("Perform comprehensive validation")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("performance")
                        .short('p')
                        .long("performance")
                        .help("Include performance analysis")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("report")
                        .short('r')
                        .long("report")
                        .value_name("FILE")
                        .help("Generate validation report to file"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", sub_matches)) => {
            if let Err(e) = run_analyze_command(sub_matches) {
                eprintln!("Error running analyze command: {}", e);
                process::exit(1);
            }
        }
        Some(("report", sub_matches)) => {
            if let Err(e) = run_report_command(sub_matches) {
                eprintln!("Error running report command: {}", e);
                process::exit(1);
            }
        }
        Some(("html-from-json", sub_matches)) => {
            if let Err(e) = run_html_from_json_command(sub_matches) {
                eprintln!("Error running html-from-json command: {}", e);
                process::exit(1);
            }
        }
        Some(("test", sub_matches)) => {
            if let Err(e) = run_test_command(sub_matches) {
                eprintln!("Error running test command: {}", e);
                process::exit(1);
            }
        }
        Some(("convert", sub_matches)) => {
            if let Err(e) = run_convert_command(sub_matches) {
                eprintln!("Error running convert command: {}", e);
                process::exit(1);
            }
        }
        Some(("binary-info", sub_matches)) => {
            if let Err(e) = run_binary_info_command(sub_matches) {
                eprintln!("Error running binary-info command: {}", e);
                process::exit(1);
            }
        }
        Some(("binary-validate", sub_matches)) => {
            if let Err(e) = run_binary_validate_command(sub_matches) {
                eprintln!("Error running binary-validate command: {}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("No subcommand provided. Use --help for usage information.");
            process::exit(1);
        }
    }
}

fn run_analyze_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::analyze::run_analyze;
    run_analyze(matches)
}

fn run_report_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::generate_report::run_generate_report;
    run_generate_report(matches)
}

fn run_html_from_json_command(
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::html_from_json::run_html_from_json;
    run_html_from_json(matches)
}

fn run_test_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::test::run_test;
    run_test(matches)
}

fn run_convert_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::convert::run_convert;
    run_convert(matches)
}

fn run_binary_info_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::binary_info::run_binary_info;
    run_binary_info(matches)
}

fn run_binary_validate_command(
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::binary_validate::run_binary_validate;
    run_binary_validate(matches)
}
