//! Analysis report command implementation
//!
//! This module provides the analyze-report subcommand functionality for generating
//! comprehensive analysis reports from binary memory analysis files.

use clap::ArgMatches;
use std::error::Error;
use std::path::Path;

/// Run the analyze-report command
pub fn run_analyze_report(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Extract arguments from matches
    let input_path = matches
        .get_one::<String>("input")
        .ok_or("Input file argument is required")?;
    
    let output_path = matches
        .get_one::<String>("output")
        .ok_or("Output file argument is required")?;
    
    let report_type = matches
        .get_one::<String>("type")
        .map(|s| s.as_str())
        .unwrap_or("comprehensive");
    
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("html");

    let include_memory_trends = matches.get_flag("memory-trends");
    let include_performance = matches.get_flag("performance");
    let include_security = matches.get_flag("security");
    let include_lifecycle = matches.get_flag("lifecycle");

    println!("📊 Starting analysis report generation...");
    println!("📂 Input file: {input_path}");
    println!("📁 Output file: {output_path}");
    println!("📋 Report type: {report_type}");
    println!("📄 Format: {format}");

    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input file does not exist: {input_path}").into());
    }

    // Generate the analysis report
    generate_analysis_report(
        input_path,
        output_path,
        report_type,
        format,
        include_memory_trends,
        include_performance,
        include_security,
        include_lifecycle,
    )
}

/// Generate comprehensive analysis report
fn generate_analysis_report(
    input_path: &str,
    output_path: &str,
    report_type: &str,
    format: &str,
    include_memory_trends: bool,
    include_performance: bool,
    include_security: bool,
    include_lifecycle: bool,
) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_parser::{BinaryParser, BinaryParseOptions};
    use crate::export::formats::analysis_report_generator::{
        AnalysisReportGenerator, AnalysisReportOptions, ReportFormat, AnalysisDepth
    };

    println!("🔍 Parsing binary file...");
    
    // Parse binary file
    let parser_options = BinaryParseOptions::safe();
    let parser = BinaryParser::new(parser_options);
    let binary_data = parser.parse_file(input_path)
        .map_err(|e| format!("Failed to parse binary file: {e}"))?;

    println!("📊 Generating analysis report...");

    // Configure report options based on parameters
    let mut report_options = match report_type {
        "quick" => AnalysisReportOptions::quick(),
        "standard" => AnalysisReportOptions::standard(),
        "comprehensive" => AnalysisReportOptions::comprehensive(),
        "security" => AnalysisReportOptions::security_focused(),
        _ => return Err(format!("Unknown report type: {report_type}. Supported types: quick, standard, comprehensive, security").into()),
    };

    // Override specific analysis areas if flags are provided
    if include_memory_trends {
        report_options.include_memory_trends = true;
    }
    if include_performance {
        report_options.include_performance_analysis = true;
    }
    if include_security {
        report_options.include_security_analysis = true;
    }
    if include_lifecycle {
        report_options.include_lifecycle_analysis = true;
    }

    // Set output format
    report_options.output_format = match format {
        "html" => ReportFormat::Html,
        "json" => ReportFormat::Json,
        "text" => ReportFormat::Text,
        "markdown" | "md" => ReportFormat::Markdown,
        _ => return Err(format!("Unsupported format: {format}. Supported formats: html, json, text, markdown").into()),
    };

    // Generate the report
    let generator = AnalysisReportGenerator::new(report_options);
    let report = generator.generate_report_to_file(&binary_data, output_path)
        .map_err(|e| format!("Failed to generate analysis report: {e}"))?;

    // Print report summary
    println!("\n📊 Analysis Report Summary:");
    println!("   Health Score: {:.2}/1.0", report.executive_summary.health_score);
    println!("   Critical Issues: {}", report.executive_summary.critical_issues);
    println!("   High Priority Recommendations: {}", report.executive_summary.high_priority_recommendations);
    println!("   Total Recommendations: {}", report.recommendations.len());

    // Print key findings
    if !report.executive_summary.key_findings.is_empty() {
        println!("\n🔍 Key Findings:");
        for finding in &report.executive_summary.key_findings {
            println!("   • {finding}");
        }
    }

    // Print top recommendations
    if !report.recommendations.is_empty() {
        println!("\n💡 Top Recommendations:");
        let mut sorted_recommendations = report.recommendations.clone();
        sorted_recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rec in sorted_recommendations.iter().take(3) {
            println!("   • {} (Priority: {}/10)", rec.category, rec.priority);
            println!("     {}", rec.description);
        }
    }

    println!("\n🎉 Analysis report generated successfully!");
    println!("📁 Output file: {output_path}");

    // Provide next steps based on format
    match format {
        "html" => println!("🌐 Open the HTML file in a web browser to view the interactive report"),
        "json" => println!("📄 Use the JSON file for programmatic analysis or integration with other tools"),
        "text" => println!("📝 View the text report in any text editor or terminal"),
        "markdown" => println!("📖 View the Markdown report in any Markdown viewer or convert to other formats"),
        _ => {}
    }

    Ok(())
}

/// Run the query command for searching and filtering binary data
pub fn run_query(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_path = matches
        .get_one::<String>("input")
        .ok_or("Input file argument is required")?;
    
    let query = matches
        .get_one::<String>("query")
        .ok_or("Query argument is required")?;
    
    let output_format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("table");
    
    let limit = matches
        .get_one::<String>("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    println!("🔍 Querying binary data...");
    println!("📂 Input file: {input_path}");
    println!("🔎 Query: {query}");
    println!("📊 Output format: {output_format}");
    println!("📏 Limit: {limit}");

    // Validate input file exists
    if !Path::new(input_path).exists() {
        return Err(format!("Input file does not exist: {input_path}").into());
    }

    execute_query(input_path, query, output_format, limit)
}

/// Execute query against binary data
fn execute_query(
    input_path: &str,
    query: &str,
    output_format: &str,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    use crate::export::formats::binary_parser::{BinaryParser, BinaryParseOptions};

    println!("🔍 Parsing binary file...");
    
    // Parse binary file
    let parser_options = BinaryParseOptions::safe();
    let parser = BinaryParser::new(parser_options);
    let binary_data = parser.parse_file(input_path)
        .map_err(|e| format!("Failed to parse binary file: {e}"))?;

    println!("🔎 Executing query...");

    // Parse and execute query
    let results = parse_and_execute_query(&binary_data.allocations, query, limit)?;

    // Display results based on format
    match output_format {
        "table" => display_results_as_table(&results),
        "json" => display_results_as_json(&results)?,
        "csv" => display_results_as_csv(&results),
        _ => return Err(format!("Unsupported output format: {output_format}. Supported formats: table, json, csv").into()),
    }

    println!("\n🎉 Query executed successfully!");
    println!("📊 Found {} results", results.len());

    Ok(())
}

/// Parse and execute query against allocations
fn parse_and_execute_query(
    allocations: &[crate::core::types::AllocationInfo],
    query: &str,
    limit: usize,
) -> Result<Vec<&crate::core::types::AllocationInfo>, Box<dyn Error>> {
    let mut results = Vec::new();

    // Simple query parsing - in a real implementation, this would be more sophisticated
    if query.starts_with("size >") {
        let size_str = query.strip_prefix("size >").unwrap().trim();
        let size_threshold: usize = size_str.parse()
            .map_err(|_| format!("Invalid size value: {size_str}"))?;
        
        for allocation in allocations {
            if allocation.size > size_threshold {
                results.push(allocation);
                if results.len() >= limit {
                    break;
                }
            }
        }
    } else if query.starts_with("size <") {
        let size_str = query.strip_prefix("size <").unwrap().trim();
        let size_threshold: usize = size_str.parse()
            .map_err(|_| format!("Invalid size value: {size_str}"))?;
        
        for allocation in allocations {
            if allocation.size < size_threshold {
                results.push(allocation);
                if results.len() >= limit {
                    break;
                }
            }
        }
    } else if query.starts_with("type =") {
        let type_name = query.strip_prefix("type =").unwrap().trim().trim_matches('"');
        
        for allocation in allocations {
            if let Some(alloc_type) = &allocation.type_name {
                if alloc_type.contains(type_name) {
                    results.push(allocation);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
    } else if query.starts_with("leaked") {
        for allocation in allocations {
            if allocation.is_leaked {
                results.push(allocation);
                if results.len() >= limit {
                    break;
                }
            }
        }
    } else if query.starts_with("active") {
        for allocation in allocations {
            if allocation.timestamp_dealloc.is_none() {
                results.push(allocation);
                if results.len() >= limit {
                    break;
                }
            }
        }
    } else {
        return Err(format!("Unsupported query: {query}. Supported queries: 'size > N', 'size < N', 'type = \"TYPE\"', 'leaked', 'active'").into());
    }

    Ok(results)
}

/// Display results as a formatted table
fn display_results_as_table(results: &[&crate::core::types::AllocationInfo]) {
    println!("\n📊 Query Results:");
    println!("{:<18} {:<12} {:<20} {:<15} {:<10}", "Pointer", "Size", "Type", "Status", "Leaked");
    println!("{}", "-".repeat(80));

    for allocation in results {
        let ptr_str = format!("0x{:x}", allocation.ptr);
        let size_str = format!("{} bytes", allocation.size);
        let type_str = allocation.type_name.as_deref().unwrap_or("unknown");
        let status_str = if allocation.timestamp_dealloc.is_some() {
            "Deallocated"
        } else {
            "Active"
        };
        let leaked_str = if allocation.is_leaked { "Yes" } else { "No" };

        println!("{:<18} {:<12} {:<20} {:<15} {:<10}", 
                 ptr_str, size_str, 
                 if type_str.len() > 18 { &type_str[..18] } else { type_str },
                 status_str, leaked_str);
    }
}

/// Display results as JSON
fn display_results_as_json(results: &[&crate::core::types::AllocationInfo]) -> Result<(), Box<dyn Error>> {
    let json_results: Vec<_> = results.iter().map(|allocation| {
        serde_json::json!({
            "ptr": format!("0x{:x}", allocation.ptr),
            "size": allocation.size,
            "type_name": allocation.type_name,
            "var_name": allocation.var_name,
            "scope_name": allocation.scope_name,
            "timestamp_alloc": allocation.timestamp_alloc,
            "timestamp_dealloc": allocation.timestamp_dealloc,
            "is_leaked": allocation.is_leaked,
            "lifetime_ms": allocation.lifetime_ms,
        })
    }).collect();

    let output = serde_json::to_string_pretty(&json_results)
        .map_err(|e| format!("Failed to serialize results as JSON: {e}"))?;
    
    println!("\n📄 Query Results (JSON):");
    println!("{output}");

    Ok(())
}

/// Display results as CSV
fn display_results_as_csv(results: &[&crate::core::types::AllocationInfo]) {
    println!("\n📊 Query Results (CSV):");
    println!("Pointer,Size,Type,VarName,Status,Leaked,LifetimeMs");

    for allocation in results {
        let ptr_str = format!("0x{:x}", allocation.ptr);
        let type_str = allocation.type_name.as_deref().unwrap_or("unknown");
        let var_str = allocation.var_name.as_deref().unwrap_or("");
        let status_str = if allocation.timestamp_dealloc.is_some() {
            "Deallocated"
        } else {
            "Active"
        };
        let leaked_str = if allocation.is_leaked { "Yes" } else { "No" };
        let lifetime_str = allocation.lifetime_ms.map(|l| l.to_string()).unwrap_or_default();

        println!("{},{},{},{},{},{},{}", 
                 ptr_str, allocation.size, type_str, var_str, status_str, leaked_str, lifetime_str);
    }
}