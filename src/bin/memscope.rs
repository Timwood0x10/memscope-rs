use std::process::{Command, Stdio};
use std::path::Path;
use clap::{Arg, Command as ClapCommand};

fn main() {
    let matches = ClapCommand::new("memscope")
        .version("0.1.0")
        .author("MemScope Team")
        .about("Memory analysis tool for Rust programs")
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export format: json, html, or both")
                .value_parser(["json", "html", "both"])
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("Output file path (without extension)")
                .default_value("memscope_analysis")
        )
        .arg(
            Arg::new("auto-track")
                .long("auto-track")
                .help("Automatically track all allocations")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("command")
                .help("Command to run (e.g., 'cargo run --release')")
                .required(true)
                .num_args(1..)
        )
        .get_matches();

    let export_format = matches.get_one::<String>("export");
    let output_path = matches.get_one::<String>("output").unwrap();
    let auto_track = matches.get_flag("auto-track");
    let command_args: Vec<&String> = matches.get_many::<String>("command").unwrap().collect();

    println!("🚀 MemScope - Memory Analysis Tool");
    println!("📋 Command: {}", command_args.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));
    
    if let Some(format) = export_format {
        println!("📊 Export format: {}", format);
        println!("📁 Output path: {}", output_path);
    }
    
    if auto_track {
        println!("🔍 Auto-tracking enabled");
    }

    // Set environment variables for the target process
    let mut env_vars = vec![
        ("MEMSCOPE_ENABLED", "1"),
        ("MEMSCOPE_AUTO_EXPORT", "1"),
    ];
    
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
                post_process_analysis(output_path, export_format.unwrap());
            }
        }
        Err(e) => {
            eprintln!("❌ Program execution failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_with_tracking(command_args: &[&String], env_vars: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    if command_args.is_empty() {
        return Err("No command provided".into());
    }

    let program = command_args[0];
    let args = &command_args[1..];

    println!("🔄 Executing: {} {}", program, args.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "));

    let mut cmd = Command::new(program);
    cmd.args(args);
    
    // Set environment variables for memory tracking
    for (key, value) in env_vars {
        cmd.env(key, value);
        println!("🔧 Setting env: {}={}", key, value);
    }
    
    // Inherit stdio to see the program output
    cmd.stdout(Stdio::inherit())
       .stderr(Stdio::inherit());

    let status = cmd.status()?;
    
    if !status.success() {
        return Err(format!("Command failed with exit code: {:?}", status.code()).into());
    }

    Ok(())
}

fn post_process_analysis(output_path: &str, format: &str) {
    match format {
        "json" => {
            let json_path = format!("{}.json", output_path);
            if Path::new(&json_path).exists() {
                println!("📄 JSON analysis: {}", json_path);
                analyze_json_output(&json_path);
            }
        }
        "html" => {
            let html_path = format!("{}.html", output_path);
            if Path::new(&html_path).exists() {
                println!("🌐 HTML dashboard: {}", html_path);
            }
        }
        "both" => {
            post_process_analysis(output_path, "json");
            post_process_analysis(output_path, "html");
        }
        _ => {}
    }
}

fn analyze_json_output(json_path: &str) {
    // Quick analysis of the exported JSON
    if let Ok(content) = std::fs::read_to_string(json_path) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(stats) = data.get("memory_analysis")
                .and_then(|ma| ma.get("statistics"))
                .and_then(|s| s.get("lifecycle_analysis")) {
                
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