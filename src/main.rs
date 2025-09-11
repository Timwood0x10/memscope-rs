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

    #[test]
    fn test_full_command_structure() {
        let cmd = Command::new("memscope")
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
            );

        // Test all subcommands exist
        let subcommands: Vec<&str> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subcommands.contains(&"analyze"));
        assert!(subcommands.contains(&"report"));
        assert!(subcommands.contains(&"html-from-json"));
        assert!(subcommands.contains(&"test"));
    }

    #[test]
    fn test_report_subcommand_structure() {
        let cmd = Command::new("memscope")
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
            );

        let report_cmd = cmd.find_subcommand("report").unwrap();
        assert_eq!(report_cmd.get_name(), "report");

        // Test required arguments
        let args: Vec<&str> = report_cmd
            .get_arguments()
            .map(|a| a.get_id().as_str())
            .collect();
        assert!(args.contains(&"input"));
        assert!(args.contains(&"output"));
        assert!(args.contains(&"format"));
    }

    #[test]
    fn test_html_from_json_subcommand_structure() {
        let cmd = Command::new("memscope")
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
                            .help("Base name for JSON files")
                            .default_value("snapshot"),
                    )
                    .arg(
                        Arg::new("verbose")
                            .short('v')
                            .long("verbose")
                            .help("Enable verbose output")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("debug")
                            .short('d')
                            .long("debug")
                            .help("Enable debug mode")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("performance")
                            .long("performance")
                            .help("Enable performance analysis mode")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("validate-only")
                            .long("validate-only")
                            .help("Only validate JSON files")
                            .action(clap::ArgAction::SetTrue),
                    ),
            );

        let html_cmd = cmd.find_subcommand("html-from-json").unwrap();
        assert_eq!(html_cmd.get_name(), "html-from-json");

        let args: Vec<&str> = html_cmd
            .get_arguments()
            .map(|a| a.get_id().as_str())
            .collect();
        assert!(args.contains(&"input-dir"));
        assert!(args.contains(&"output"));
        assert!(args.contains(&"base-name"));
        assert!(args.contains(&"verbose"));
        assert!(args.contains(&"debug"));
        assert!(args.contains(&"performance"));
        assert!(args.contains(&"validate-only"));
    }

    #[test]
    fn test_report_command_parsing() {
        let cmd = Command::new("memscope")
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
            );

        let matches = cmd.try_get_matches_from(vec![
            "memscope",
            "report",
            "--input",
            "input.json",
            "--output",
            "output.html",
            "--format",
            "svg",
        ]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((subcommand, sub_matches)) = matches.subcommand() {
                assert_eq!(subcommand, "report");
                assert_eq!(sub_matches.get_one::<String>("input").unwrap(), "input.json");
                assert_eq!(sub_matches.get_one::<String>("output").unwrap(), "output.html");
                assert_eq!(sub_matches.get_one::<String>("format").unwrap(), "svg");
            }
        }
    }

    #[test]
    fn test_html_from_json_command_parsing() {
        let cmd = Command::new("memscope")
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
                        Arg::new("verbose")
                            .short('v')
                            .long("verbose")
                            .help("Enable verbose output")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("debug")
                            .short('d')
                            .long("debug")
                            .help("Enable debug mode")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("validate-only")
                            .long("validate-only")
                            .help("Only validate JSON files")
                            .action(clap::ArgAction::SetTrue),
                    ),
            );

        // Test with verbose and debug flags
        let matches = cmd.clone().try_get_matches_from(vec![
            "memscope",
            "html-from-json",
            "--input-dir",
            "/path/to/json",
            "--output",
            "report.html",
            "--verbose",
            "--debug",
        ]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((subcommand, sub_matches)) = matches.subcommand() {
                assert_eq!(subcommand, "html-from-json");
                assert_eq!(sub_matches.get_one::<String>("input-dir").unwrap(), "/path/to/json");
                assert_eq!(sub_matches.get_one::<String>("output").unwrap(), "report.html");
                assert!(sub_matches.get_flag("verbose"));
                assert!(sub_matches.get_flag("debug"));
                assert!(!sub_matches.get_flag("validate-only"));
            }
        }

        // Test validate-only mode
        let matches = cmd.try_get_matches_from(vec![
            "memscope",
            "html-from-json",
            "--input-dir",
            "/path/to/json",
            "--validate-only",
        ]);
        assert!(matches.is_ok());

        if let Ok(matches) = matches {
            if let Some((_, sub_matches)) = matches.subcommand() {
                assert!(sub_matches.get_flag("validate-only"));
                assert!(sub_matches.get_one::<String>("output").is_none());
            }
        }
    }

    #[test]
    fn test_command_version_and_metadata() {
        let cmd = Command::new("memscope")
            .version("0.1.2")
            .author("TimWood")
            .about("Advanced Rust memory analysis and visualization toolkit");

        assert_eq!(cmd.get_name(), "memscope");
        assert_eq!(cmd.get_version().unwrap(), "0.1.2");
        assert_eq!(cmd.get_author().unwrap(), "TimWood");
        assert!(cmd.get_about().unwrap().to_string().contains("Advanced Rust memory analysis"));
    }

    #[test]
    fn test_argument_validation() {
        let cmd = Command::new("memscope")
            .subcommand(
                Command::new("analyze")
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
                    ),
            );

        // Test missing required command argument
        let matches = cmd.clone().try_get_matches_from(vec!["memscope", "analyze"]);
        assert!(matches.is_err());

        // Test valid command with simple arguments
        let matches = cmd.try_get_matches_from(vec![
            "memscope",
            "analyze",
            "echo",
            "hello",
        ]);
        assert!(matches.is_ok());
        
        if let Ok(matches) = matches {
            if let Some((subcommand, sub_matches)) = matches.subcommand() {
                assert_eq!(subcommand, "analyze");
                let command_args: Vec<&String> = sub_matches.get_many::<String>("command").unwrap().collect();
                assert_eq!(command_args.len(), 2); // echo, hello
                assert_eq!(command_args[0], "echo");
                assert_eq!(command_args[1], "hello");
                assert_eq!(sub_matches.get_one::<String>("export").unwrap(), "html"); // default value
            }
        }
    }

    #[test]
    fn test_error_handling_for_invalid_commands() {
        let cmd = Command::new("memscope")
            .subcommand(Command::new("analyze"))
            .subcommand(Command::new("report"));

        // Test invalid subcommand
        let matches = cmd.try_get_matches_from(vec!["memscope", "invalid-command"]);
        assert!(matches.is_err());
    }

    #[test]
    fn test_default_value_behavior() {
        let cmd = Command::new("memscope")
            .subcommand(
                Command::new("analyze")
                    .arg(
                        Arg::new("command")
                            .required(true)
                            .num_args(1..),
                    )
                    .arg(
                        Arg::new("export")
                            .long("export")
                            .default_value("html"),
                    )
                    .arg(
                        Arg::new("output")
                            .long("output")
                            .default_value("memory_analysis"),
                    ),
            )
            .subcommand(
                Command::new("test")
                    .arg(
                        Arg::new("output")
                            .long("output")
                            .default_value("enhanced_memory_test"),
                    ),
            );

        // Test analyze defaults
        let matches = cmd.clone().try_get_matches_from(vec!["memscope", "analyze", "echo", "test"]);
        assert!(matches.is_ok());
        if let Ok(matches) = matches {
            if let Some((_, sub_matches)) = matches.subcommand() {
                assert_eq!(sub_matches.get_one::<String>("export").unwrap(), "html");
                assert_eq!(sub_matches.get_one::<String>("output").unwrap(), "memory_analysis");
            }
        }

        // Test test defaults
        let matches = cmd.try_get_matches_from(vec!["memscope", "test"]);
        assert!(matches.is_ok());
        if let Ok(matches) = matches {
            if let Some((_, sub_matches)) = matches.subcommand() {
                assert_eq!(sub_matches.get_one::<String>("output").unwrap(), "enhanced_memory_test");
            }
        }
    }
}
