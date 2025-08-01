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
                )
                .arg(
                    Arg::new("compression")
                        .long("compression")
                        .value_name("ALGORITHM")
                        .help("Compression algorithm for binary export (zstd, none)")
                        .default_value("zstd"),
                )
                .arg(
                    Arg::new("compression-level")
                        .long("compression-level")
                        .value_name("LEVEL")
                        .help("Compression level (1-22, higher = better compression)")
                        .default_value("6"),
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
            Command::new("export")
                .about("Convert binary memory analysis files to other formats")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input binary file path")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .required_unless_present("validate-only"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format (json, html)")
                        .default_value("json"),
                )
                .arg(
                    Arg::new("streaming")
                        .long("streaming")
                        .help("Enable streaming mode for large files")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("validate-only")
                        .long("validate-only")
                        .help("Only validate binary file without conversion")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("analyze-report")
                .about("Generate comprehensive analysis reports from binary memory data")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input binary file path")
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output report file path")
                        .required(true),
                )
                .arg(
                    Arg::new("type")
                        .short('t')
                        .long("type")
                        .value_name("TYPE")
                        .help("Report type (quick, standard, comprehensive, security)")
                        .default_value("standard"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format (html, json, text, markdown)")
                        .default_value("html"),
                )
                .arg(
                    Arg::new("memory-trends")
                        .long("memory-trends")
                        .help("Include memory usage trend analysis")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("performance")
                        .long("performance")
                        .help("Include performance bottleneck analysis")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("security")
                        .long("security")
                        .help("Include security violation analysis")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("lifecycle")
                        .long("lifecycle")
                        .help("Include lifecycle analysis")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("query")
                .about("Query and filter binary memory data")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .value_name("FILE")
                        .help("Input binary file path")
                        .required(true),
                )
                .arg(
                    Arg::new("query")
                        .short('q')
                        .long("query")
                        .value_name("QUERY")
                        .help("Query string (e.g., 'size > 1024', 'type = \"Vec\"', 'leaked', 'active')")
                        .required(true),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("Output format (table, json, csv)")
                        .default_value("table"),
                )
                .arg(
                    Arg::new("limit")
                        .short('l')
                        .long("limit")
                        .value_name("NUMBER")
                        .help("Maximum number of results to display")
                        .default_value("100"),
                ),
        )
        .subcommand(
            Command::new("integration-test")
                .about("Run comprehensive integration tests")
                .arg(
                    Arg::new("type")
                        .short('t')
                        .long("type")
                        .value_name("TYPE")
                        .help("Test type (all, integrity, performance, regression, compatibility)")
                        .default_value("all"),
                )
                .arg(
                    Arg::new("output-dir")
                        .short('o')
                        .long("output-dir")
                        .value_name("DIR")
                        .help("Output directory for test results")
                        .default_value("test_results"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Enable verbose output")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("generate-test-data")
                        .long("generate-test-data")
                        .help("Generate test data before running tests")
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
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", sub_matches)) => {
            if let Err(e) = run_analyze_command(sub_matches) {
                eprintln!("Error running analyze command: {e}");
                process::exit(1);
            }
        }
        Some(("export", sub_matches)) => {
            if let Err(e) = run_export_command(sub_matches) {
                eprintln!("Error running export command: {e}");
                process::exit(1);
            }
        }
        Some(("report", sub_matches)) => {
            if let Err(e) = run_report_command(sub_matches) {
                eprintln!("Error running report command: {e}");
                process::exit(1);
            }
        }
        Some(("html-from-json", sub_matches)) => {
            if let Err(e) = run_html_from_json_command(sub_matches) {
                eprintln!("Error running html-from-json command: {e}");
                process::exit(1);
            }
        }
        Some(("analyze-report", sub_matches)) => {
            if let Err(e) = run_analyze_report_command(sub_matches) {
                eprintln!("Error running analyze-report command: {e}");
                process::exit(1);
            }
        }
        Some(("query", sub_matches)) => {
            if let Err(e) = run_query_command(sub_matches) {
                eprintln!("Error running query command: {e}");
                process::exit(1);
            }
        }
        Some(("integration-test", sub_matches)) => {
            if let Err(e) = run_integration_test_command(sub_matches) {
                eprintln!("Error running integration-test command: {e}");
                process::exit(1);
            }
        }
        Some(("test", sub_matches)) => {
            if let Err(e) = run_test_command(sub_matches) {
                eprintln!("Error running test command: {e}");
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

fn run_export_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::export::run_export;
    run_export(matches)
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

fn run_analyze_report_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::analyze_report::run_analyze_report;
    run_analyze_report(matches)
}

fn run_query_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::analyze_report::run_query;
    run_query(matches)
}

fn run_integration_test_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::integration_test::run_integration_test;
    run_integration_test(matches)
}

fn run_test_command(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    use memscope_rs::cli::commands::test::run_test;
    run_test(matches)
}
