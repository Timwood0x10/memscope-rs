//! Test the new unified dashboard API

use memscope_rs::export::binary::{
    export_binary_to_dashboard, DashboardOptions, DashboardFormat, DataScope, PerformanceMode
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Unified Dashboard API");
    
    // Test data file (if it exists)
    let binary_file = "MemoryAnalysis/large_scale_user.memscope";
    let project_name = "unified_api_test";
    
    if !std::path::Path::new(binary_file).exists() {
        println!("❌ Binary file not found: {binary_file}");
        println!("Please run a memory analysis first to generate test data");
        return Ok(());
    }
    
    println!("✅ Found binary file: {binary_file}");
    
    // Test 1: Default lightweight export (recommended)
    println!("\n📊 Test 1: Default lightweight export");
    let stats1 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_default", project_name),
        DashboardOptions::default()
    )?;
    
    println!("   ✅ Generated {} files", stats1.total_files_generated);
    println!("   ✅ HTML size: {} KB", stats1.html_size / 1024);
    println!("   ✅ Processing time: {}ms", stats1.processing_time_ms);
    println!("   ✅ Format used: {:?}", stats1.format_used);
    
    // Test 2: Fast preset
    println!("\n🚀 Test 2: Fast preset");
    let stats2 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_fast", project_name),
        DashboardOptions::fast_preset()
    )?;
    
    println!("   ✅ Generated {} files", stats2.total_files_generated);
    println!("   ✅ HTML size: {} KB", stats2.html_size / 1024);
    println!("   ✅ Processing time: {}ms", stats2.processing_time_ms);
    println!("   ✅ Format used: {:?}", stats2.format_used);
    
    // Test 3: Embedded format (backward compatible)
    println!("\n📦 Test 3: Embedded format (backward compatible)");
    let stats3 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_embedded", project_name),
        DashboardOptions::embedded_preset()
    )?;
    
    println!("   ✅ Generated {} files", stats3.total_files_generated);
    println!("   ✅ HTML size: {} KB", stats3.html_size / 1024);
    println!("   ✅ Processing time: {}ms", stats3.processing_time_ms);
    println!("   ✅ Format used: {:?}", stats3.format_used);
    
    // Test 4: Custom configuration
    println!("\n⚙️  Test 4: Custom configuration");
    let custom_options = DashboardOptions::new()
        .format(DashboardFormat::Lightweight)
        .scope(DataScope::UserOnly)
        .performance(PerformanceMode::Fast)
        .parallel_processing(true)
        .batch_size(5000);
        
    let stats4 = export_binary_to_dashboard(
        binary_file,
        &format!("{}_custom", project_name),
        custom_options
    )?;
    
    println!("   ✅ Generated {} files", stats4.total_files_generated);
    println!("   ✅ HTML size: {} KB", stats4.html_size / 1024);
    println!("   ✅ Processing time: {}ms", stats4.processing_time_ms);
    println!("   ✅ Format used: {:?}", stats4.format_used);
    
    // Performance comparison
    println!("\n📈 Performance Comparison:");
    println!("   Default:  {}ms", stats1.processing_time_ms);
    println!("   Fast:     {}ms", stats2.processing_time_ms);
    println!("   Embedded: {}ms", stats3.processing_time_ms);
    println!("   Custom:   {}ms", stats4.processing_time_ms);
    
    // Size comparison
    println!("\n📏 Size Comparison:");
    println!("   Default:  {} KB", stats1.html_size / 1024);
    println!("   Fast:     {} KB", stats2.html_size / 1024);
    println!("   Embedded: {} KB", stats3.html_size / 1024);
    println!("   Custom:   {} KB", stats4.html_size / 1024);
    
    println!("\n🎉 All tests completed successfully!");
    println!("\n📁 Generated files:");
    println!("   MemoryAnalysis/{}_default/", project_name);
    println!("   MemoryAnalysis/{}_fast/", project_name);
    println!("   MemoryAnalysis/{}_embedded/", project_name);
    println!("   MemoryAnalysis/{}_custom/", project_name);
    
    Ok(())
}