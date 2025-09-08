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
                        .help("Export format (json, svg, html)")
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
        .get_matches();

    match matches.subcommand() {
        Some(("analyze", sub_matches)) => {
            if let Err(e) = run_analyze_command(sub_matches) {
                tracing::error!("Error running analyze command: {}", e);
                process::exit(1);
            }
        }
        Some(("report", sub_matches)) => {
            if let Err(e) = run_report_command(sub_matches) {
                tracing::error!("Error running report command: {}", e);
                process::exit(1);
            }
        }
        Some(("html-from-json", sub_matches)) => {
            if let Err(e) = run_html_from_json_command(sub_matches) {
                tracing::error!("Error running html-from-json command: {}", e);
                process::exit(1);
            }
        }
        Some(("test", sub_matches)) => {
            if let Err(e) = run_test_command(sub_matches) {
                tracing::error!("Error running test command: {}", e);
                process::exit(1);
            }
        }
        _ => {
            tracing::error!("No subcommand provided. Use --help for usage information.");
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    fn create_test_command() -> Command {
        Command::new("memscope")
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
                            .help("Export format (json, svg, html)")
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
                Command::new("test").about("Run enhanced memory tests").arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .default_value("enhanced_memory_test"),
                ),
            )
    }

    #[test]
    fn test_run_analyze_command_function_exists() {
        // Test that the function exists and has the correct signature
        let _f: fn(&clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> =
            run_analyze_command;
    }

    #[test]
    fn test_run_report_command_function_exists() {
        // Test that the function exists and has the correct signature
        let _f: fn(&clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> =
            run_report_command;
    }

    #[test]
    fn test_run_html_from_json_command_function_exists() {
        // Test that the function exists and has the correct signature
        let _f: fn(&clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> =
            run_html_from_json_command;
    }

    #[test]
    fn test_run_test_command_function_exists() {
        // Test that the function exists and has the correct signature
        let _f: fn(&clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> = run_test_command;
    }

    #[test]
    fn test_command_structure() {
        let cmd = create_test_command();

        // Test basic command properties
        assert_eq!(cmd.get_name(), "memscope");
        assert_eq!(cmd.get_version().unwrap(), "0.1.2");
        assert_eq!(cmd.get_author().unwrap(), "TimWood");

        // Test that subcommands exist
        let subcommands: Vec<&str> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subcommands.contains(&"analyze"));
        assert!(subcommands.contains(&"test"));
    }

    #[test]
    fn test_analyze_subcommand_structure() {
        let cmd = create_test_command();
        let analyze_cmd = cmd.find_subcommand("analyze").unwrap();

        // Test analyze subcommand properties
        assert_eq!(analyze_cmd.get_name(), "analyze");
        let about_str = format!("{}", analyze_cmd.get_about().unwrap());
        assert!(about_str.contains("Analyze memory usage"));

        // Test that required arguments exist
        let args: Vec<&str> = analyze_cmd
            .get_arguments()
            .map(|a| a.get_id().as_str())
            .collect();
        assert!(args.contains(&"command"));
        assert!(args.contains(&"export"));
        assert!(args.contains(&"output"));
    }

    #[test]
    fn test_test_subcommand_structure() {
        let cmd = create_test_command();
        let test_cmd = cmd.find_subcommand("test").unwrap();

        // Test test subcommand properties
        assert_eq!(test_cmd.get_name(), "test");
        let about_str = format!("{}", test_cmd.get_about().unwrap());
        assert!(about_str.contains("Run enhanced memory tests"));

        // Test that arguments exist
        let args: Vec<&str> = test_cmd
            .get_arguments()
            .map(|a| a.get_id().as_str())
            .collect();
        assert!(args.contains(&"output"));
    }

    #[test]
    fn test_command_parsing() {
        let cmd = create_test_command();

        // Test parsing valid analyze command
        let matches = cmd.clone().try_get_matches_from(vec![
            "memscope",
            "analyze",
            "cargo",
            "test",
            "--export",
            "json",
            "--output",
            "test_output",
        ]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((subcommand, sub_matches)) = matches.subcommand() {
                assert_eq!(subcommand, "analyze");
                assert!(sub_matches.get_many::<String>("command").is_some());
                assert_eq!(sub_matches.get_one::<String>("export").unwrap(), "json");
                assert_eq!(
                    sub_matches.get_one::<String>("output").unwrap(),
                    "test_output"
                );
            }
        }
    }

    #[test]
    fn test_test_command_parsing() {
        let cmd = create_test_command();

        // Test parsing valid test command
        let matches = cmd.clone().try_get_matches_from(vec![
            "memscope",
            "test",
            "--output",
            "my_test_output",
        ]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((subcommand, sub_matches)) = matches.subcommand() {
                assert_eq!(subcommand, "test");
                assert_eq!(
                    sub_matches.get_one::<String>("output").unwrap(),
                    "my_test_output"
                );
            }
        }
    }

    #[test]
    fn test_default_values() {
        let cmd = create_test_command();

        // Test default values for analyze command
        let matches = cmd
            .clone()
            .try_get_matches_from(vec!["memscope", "analyze", "cargo", "test"]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((_, sub_matches)) = matches.subcommand() {
                assert_eq!(sub_matches.get_one::<String>("export").unwrap(), "html");
                assert_eq!(
                    sub_matches.get_one::<String>("output").unwrap(),
                    "memory_analysis"
                );
            }
        }
    }

    #[test]
    fn test_help_generation() {
        let mut cmd = create_test_command();

        // Test that help can be generated without panicking
        let _help = cmd.render_help();

        // Test subcommand help
        if let Some(analyze_cmd) = cmd.find_subcommand_mut("analyze") {
            let _analyze_help = analyze_cmd.render_help();
        }
    }
}
