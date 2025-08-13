//! Simple Binary to HTML Test

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Simple Binary to HTML Test");
    
    // Test with a simple call
    let binary_file = "MemoryAnalysis/large_scale_user.memscope";
    let output_file = "MemoryAnalysis/large_scale_user/test_output.html";
    let project_name = "test_project";
    
    if std::path::Path::new(binary_file).exists() {
        println!("✅ Found binary file: {binary_file}");
        
        // Create output directory
        std::fs::create_dir_all("MemoryAnalysis/large_scale_user")?;
        
        // Call the function directly
        memscope_rs::export::binary::export_binary_to_html(
            binary_file,
            output_file,
            project_name
        )?;
        
        println!("✅ HTML file created: {output_file}");
    } else {
        println!("❌ Binary file not found: {binary_file}");
    }
    
    Ok(())
}