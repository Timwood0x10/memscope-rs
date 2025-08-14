//! Simple Binary to HTML Test

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Simple Binary to HTML Test");
    
    // Test with a simple call - use a smaller file first
    let binary_file = "MemoryAnalysis/large_scale_user.memscope";
    let output_file = "MemoryAnalysis/test_output_simple.html";
    let project_name = "test_project";
    
    if std::path::Path::new(binary_file).exists() {
        println!("✅ Found binary file: {binary_file}");
        
        // Create output directory
        std::fs::create_dir_all("MemoryAnalysis/large_scale_user")?;
        
        // Call the function directly
        memscope_rs::export::binary::export_binary_to_html(
            binary_file,
            output_file,
        )?;
        
        println!("✅ HTML file created: {output_file}");
    } else {
        println!("❌ Binary file not found: {binary_file}");
    }
    
    Ok(())
}