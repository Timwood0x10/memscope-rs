//! Binary info command implementation
//!
//! This module provides functionality to display information about binary memory analysis files,
//! including file structure, sections, and statistics.

use crate::export::binary_parser::BinaryParser;
use clap::ArgMatches;
use std::path::Path;

/// Run the binary-info command
pub fn run_binary_info(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches.get_one::<String>("input").unwrap();
    let detailed = matches.get_flag("detailed");
    let show_sections = matches.get_flag("sections");
    let show_stats = matches.get_flag("stats");

    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("Input file does not exist: {}", input_path).into());
    }

    println!("Binary Memory Analysis File Information");
    println!("=====================================");
    println!("File: {}", input_path);

    // Load and parse the binary file
    let mut parser = BinaryParser::default();
    parser.load_from_file(input)?;

    // Display basic file information
    display_basic_info(&parser, input)?;

    if detailed || show_sections {
        display_section_info(&parser)?;
    }

    if detailed || show_stats {
        display_statistics(&mut parser)?;
    }

    if detailed {
        display_detailed_info(&parser)?;
    }

    Ok(())
}

/// Display basic file information
fn display_basic_info(_parser: &BinaryParser, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();

    println!("\nBasic Information:");
    println!("  File size: {} bytes ({:.2} MB)", file_size, file_size as f64 / (1024.0 * 1024.0));
    
    // Basic file information - simplified since some methods don't exist yet
    println!("  Format: Binary memory analysis file");
    println!("  Status: Loaded successfully");

    Ok(())
}

/// Display section information
fn display_section_info(_parser: &BinaryParser) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nSection Information:");
    println!("  Section details not available in current implementation");
    println!("  File appears to be a valid binary format");
    Ok(())
}

/// Display statistics
fn display_statistics(parser: &mut BinaryParser) -> Result<(), Box<dyn std::error::Error>> {
    println!("Statistics:");

    // Load memory statistics
    match parser.load_memory_stats() {
        Ok(stats) => {
            println!("  Memory Statistics:");
            println!("    Total allocations: {}", stats.total_allocations);
            println!("    Active allocations: {}", stats.active_allocations);
            println!("    Peak allocations: {}", stats.peak_allocations);
            println!("    Total allocated: {} bytes ({:.2} MB)", 
                stats.total_allocated, 
                stats.total_allocated as f64 / (1024.0 * 1024.0));
            println!("    Peak memory: {} bytes ({:.2} MB)", 
                stats.peak_memory, 
                stats.peak_memory as f64 / (1024.0 * 1024.0));
            
            if stats.leaked_allocations > 0 {
                println!("    ⚠️  Leaked allocations: {}", stats.leaked_allocations);
                println!("    ⚠️  Leaked memory: {} bytes ({:.2} MB)", 
                    stats.leaked_memory, 
                    stats.leaked_memory as f64 / (1024.0 * 1024.0));
            }
        }
        Err(e) => {
            println!("  ⚠️  Could not load memory statistics: {}", e);
        }
    }

    // Load allocation count
    match parser.load_allocations() {
        Ok(allocations) => {
            println!("  Allocation Details:");
            println!("    Total allocation records: {}", allocations.len());
            
            // Count allocations by type
            let mut type_counts = std::collections::HashMap::new();
            for alloc in &allocations {
                if let Some(ref type_name) = alloc.type_name {
                    *type_counts.entry(type_name.clone()).or_insert(0) += 1;
                }
            }

            if !type_counts.is_empty() {
                println!("    Top allocation types:");
                let mut sorted_types: Vec<_> = type_counts.iter().collect();
                sorted_types.sort_by(|a, b| b.1.cmp(a.1));
                
                for (type_name, count) in sorted_types.iter().take(5) {
                    println!("      {}: {} allocations", type_name, count);
                }
            }
        }
        Err(e) => {
            println!("  ⚠️  Could not load allocation data: {}", e);
        }
    }

    Ok(())
}

/// Display detailed information
fn display_detailed_info(parser: &BinaryParser) -> Result<(), Box<dyn std::error::Error>> {
    println!("Detailed Information:");

    // Display string table information (simplified)
    match parser.get_string_table() {
        Some(_string_table) => {
            println!("  String Table: Available");
        }
        None => {
            println!("  ⚠️  String table not available");
        }
    }

    // Display type table information (simplified)
    match parser.get_type_table() {
        Some(_type_table) => {
            println!("  Type Table: Available");
        }
        None => {
            println!("  ⚠️  Type table not available");
        }
    }

    // Display integrity information (simplified)
    match parser.validate_integrity() {
        Ok(_) => {
            println!("  File Integrity:");
            println!("    ✅ File integrity check passed");
        }
        Err(e) => {
            println!("  ⚠️  Could not validate file integrity: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_info_command_validation() {
        // Test that the binary-info command validates input parameters correctly
        
        // Test with nonexistent file
        let result = run_binary_info(&create_test_matches("nonexistent.bin", false, false, false));
        assert!(result.is_err());
    }

    fn create_test_matches(input: &str, detailed: bool, sections: bool, stats: bool) -> ArgMatches {
        use clap::{Arg, Command};
        
        let mut cmd = Command::new("test")
            .arg(Arg::new("input").required(true))
            .arg(Arg::new("detailed").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("sections").action(clap::ArgAction::SetTrue))
            .arg(Arg::new("stats").action(clap::ArgAction::SetTrue));

        let mut args = vec!["test", input];
        if detailed { args.push("--detailed"); }
        if sections { args.push("--sections"); }
        if stats { args.push("--stats"); }

        cmd.try_get_matches_from(args).unwrap()
    }
}