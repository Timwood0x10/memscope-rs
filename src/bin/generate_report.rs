use memscope_rs::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    match args[1].as_str() {
        "template" => {
            let default_output = "interactive_template.html".to_string();
            let output = args.get(2).unwrap_or(&default_output);
            
            // 检查源文件是否存在
            if !std::path::Path::new("interactive_template.html").exists() {
                eprintln!("❌ Source template 'interactive_template.html' not found!");
                eprintln!("Please make sure the interactive_template.html file exists in the current directory.");
                std::process::exit(1);
            }
            
            // 复制交互式模板
            if let Err(e) = std::fs::copy("interactive_template.html", output) {
                eprintln!("❌ Error creating template: {}", e);
                std::process::exit(1);
            }
            println!("✅ Created interactive template: {}", output);
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
            
            // 手动实现JSON嵌入功能
            if let Err(e) = embed_json_to_html(json_file, template_file, output_file) {
                eprintln!("❌ Error generating report: {}", e);
                std::process::exit(1);
            }
        },
        _ => {
            print_usage();
        }
    }
}

fn embed_json_to_html(json_file: &str, template_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 读取JSON数据
    let json_content = std::fs::read_to_string(json_file)?;
    
    // 读取HTML模板
    let template_content = std::fs::read_to_string(template_file)?;
    
    // 创建内联脚本
    let inline_script = format!(
        r#"<script type="text/javascript">
// Embedded JSON data for offline analysis
window.EMBEDDED_MEMORY_DATA = {};
console.log('📊 Loaded embedded memory analysis data');
</script>"#,
        json_content
    );
    
    // 替换占位符
    let final_html = template_content.replace("<!-- DATA_INJECTION_POINT -->", &inline_script);
    
    // 写入输出文件
    std::fs::write(output_file, final_html)?;
    
    println!("✅ Generated self-contained HTML report: {}", output_file);
    Ok(())
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