//! Manual HTML Demo
//!
//! Create a simple HTML dashboard manually to demonstrate the concept

use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Manual HTML Dashboard Demo");
    println!("=============================");

    // Check if we have binary files
    let binary_files = [
        "MemoryAnalysis/large_scale_user.memscope",
        "MemoryAnalysis/large_scale_full.memscope",
    ];

    for binary_file in &binary_files {
        if std::path::Path::new(binary_file).exists() {
            println!("✅ Found binary file: {}", binary_file);
            
            // Get basic file info
            let metadata = fs::metadata(binary_file)?;
            let file_size = metadata.len();
            
            // Extract project name
            let project_name = std::path::Path::new(binary_file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            // Create a simple HTML dashboard
            let html_content = create_simple_dashboard(project_name, file_size)?;
            
            // Write HTML file
            let output_dir = format!("MemoryAnalysis/{}", project_name);
            fs::create_dir_all(&output_dir)?;
            let output_path = format!("{}/simple_dashboard.html", output_dir);
            
            let mut file = fs::File::create(&output_path)?;
            file.write_all(html_content.as_bytes())?;
            
            println!("✅ Created simple HTML dashboard: {}", output_path);
            println!("   📊 Open in browser: file://{}/{}", 
                std::env::current_dir()?.display(), output_path);
            
            break; // Demo with first available file
        }
    }

    println!("🎉 Manual HTML demo completed!");
    Ok(())
}

fn create_simple_dashboard(project_name: &str, file_size: u64) -> Result<String, Box<dyn std::error::Error>> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MemScope Dashboard - {}</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            padding: 30px;
        }}
        .stat-card {{
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            border-left: 4px solid #667eea;
        }}
        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 5px;
        }}
        .stat-label {{
            color: #666;
            font-size: 0.9em;
        }}
        .info {{
            padding: 30px;
            background: #f8f9fa;
            border-top: 1px solid #eee;
        }}
        .success {{
            color: #28a745;
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 MemScope Dashboard</h1>
            <p>Memory Analysis for: <strong>{}</strong></p>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <div class="stat-value">{:.2}</div>
                <div class="stat-label">Binary File Size (KB)</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">✅</div>
                <div class="stat-label">Status: Ready</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">🚀</div>
                <div class="stat-label">Performance: Optimized</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">📊</div>
                <div class="stat-label">Analysis: Complete</div>
            </div>
        </div>
        
        <div class="info">
            <h3>🎉 Binary to HTML Conversion Demo</h3>
            <p>This is a <span class="success">proof of concept</span> showing how binary memory analysis files can be converted to interactive HTML dashboards.</p>
            
            <h4>📋 Features Demonstrated:</h4>
            <ul>
                <li>✅ Binary file detection and metadata extraction</li>
                <li>✅ Responsive HTML dashboard generation</li>
                <li>✅ Modern CSS styling with gradients and cards</li>
                <li>✅ File size analysis and display</li>
                <li>✅ Project-specific dashboard creation</li>
            </ul>
            
            <h4>🔮 Future Enhancements:</h4>
            <ul>
                <li>📊 Interactive charts and graphs</li>
                <li>🔍 Detailed allocation analysis</li>
                <li>⏱️ Timeline visualization</li>
                <li>🎯 Memory leak detection</li>
                <li>📈 Performance metrics</li>
            </ul>
            
            <p><strong>Generated:</strong> {}</p>
        </div>
    </div>
</body>
</html>
"#, 
        project_name,
        project_name,
        file_size as f64 / 1024.0,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    
    Ok(html)
}