use memscope_rs::report_generator::{generate_html_report, create_standalone_template};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    match args[1].as_str() {
        "template" => {
            let default_output = "report_template.html".to_string();
            let output = args.get(2).unwrap_or(&default_output);
            if let Err(e) = create_standalone_template(output) {
                eprintln!("❌ Error creating template: {}", e);
                std::process::exit(1);
            }
        },
        "generate" => {
            if args.len() < 4 {
                eprintln!("❌ Usage: generate_report generate <json_file> <output_file> [template_file]");
                std::process::exit(1);
            }
            
            let json_file = &args[2];
            let output_file = &args[3];
            let default_template = "report_template.html".to_string();
            let template_file = args.get(4).unwrap_or(&default_template);
            
            if let Err(e) = generate_html_report(json_file, template_file, output_file) {
                eprintln!("❌ Error generating report: {}", e);
                std::process::exit(1);
            }
        },
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("🔍 Memory Analysis Report Generator");
    println!();
    println!("Usage:");
    println!("  generate_report template [output_file]");
    println!("    Create a standalone HTML template");
    println!();
    println!("  generate_report generate <json_file> <output_file> [template_file]");
    println!("    Generate a self-contained HTML report from JSON data");
    println!();
    println!("Examples:");
    println!("  generate_report template report_template.html");
    println!("  generate_report generate data.json report.html report_template.html");
    println!();
    println!("The generated report.html is completely self-contained and can be:");
    println!("  ✅ Opened directly in any browser (no server needed)");
    println!("  ✅ Shared via email or file transfer");
    println!("  ✅ Archived for historical analysis");
    println!("  ✅ Viewed offline without any dependencies");
}