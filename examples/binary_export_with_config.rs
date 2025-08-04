//! Example demonstrating binary export with advanced metrics configuration

use memscope_rs::export::binary::{
    AdvancedMetricsLevel, BinaryExportConfig, BinaryExportConfigBuilder,
    export_to_binary_with_config
};
use memscope_rs::{get_global_tracker, init};
use std::fs;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Initialize memory tracking
    init();
    
    println!("🚀 Binary Export Configuration Demo");
    println!("=====================================");
    
    // Create some test allocations
    create_test_allocations();
    
    // Demo 1: Performance-first configuration
    demo_performance_first_config()?;
    
    // Demo 2: Debug comprehensive configuration
    demo_debug_comprehensive_config()?;
    
    // Demo 3: Custom configuration using builder
    demo_custom_config_builder()?;
    
    // Demo 4: Configuration validation
    demo_config_validation()?;
    
    println!("\n✅ All demos completed successfully!");
    Ok(())
}

fn create_test_allocations() {
    println!("\n📊 Creating test allocations...");
    
    // Create various types of allocations
    let _vec: Vec<i32> = vec![1, 2, 3, 4, 5];
    let _string = String::from("Hello, World!");
    let _boxed = Box::new(42);
    
    // Note: In a real scenario, allocations would be tracked automatically
    // by the global allocator or through explicit tracking calls
    println!("   Created test data structures");
}

fn demo_performance_first_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏃 Demo 1: Performance-First Configuration");
    println!("------------------------------------------");
    
    let config = BinaryExportConfig::performance_first();
    println!("   Advanced metrics level: {:?}", config.advanced_metrics_level);
    println!("   Source analysis: {}", config.source_analysis);
    println!("   Container analysis: {}", config.container_analysis);
    println!("   Fragmentation analysis: {}", config.fragmentation_analysis);
    println!("   Estimated performance impact: {:.1}%", 
             config.estimated_performance_impact() * 100.0);
    
    // Export with performance-first config
    let temp_file = NamedTempFile::new()?;
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    
    export_to_binary_with_config(&stats.allocations, temp_file.path(), &config)?;
    
    let file_size = fs::metadata(temp_file.path())?.len();
    println!("   ✅ Export completed, file size: {} bytes", file_size);
    
    Ok(())
}

fn demo_debug_comprehensive_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Demo 2: Debug Comprehensive Configuration");
    println!("---------------------------------------------");
    
    let config = BinaryExportConfig::debug_comprehensive();
    println!("   Advanced metrics level: {:?}", config.advanced_metrics_level);
    println!("   Source analysis: {}", config.source_analysis);
    println!("   Container analysis: {}", config.container_analysis);
    println!("   Fragmentation analysis: {}", config.fragmentation_analysis);
    println!("   ZST analysis: {}", config.zst_analysis);
    println!("   Health scoring: {}", config.health_scoring);
    println!("   Estimated performance impact: {:.1}%", 
             config.estimated_performance_impact() * 100.0);
    
    // Export with comprehensive config
    let temp_file = NamedTempFile::new()?;
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    
    export_to_binary_with_config(&stats.allocations, temp_file.path(), &config)?;
    
    let file_size = fs::metadata(temp_file.path())?.len();
    println!("   ✅ Export completed, file size: {} bytes", file_size);
    
    Ok(())
}

fn demo_custom_config_builder() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛠️  Demo 3: Custom Configuration Builder");
    println!("----------------------------------------");
    
    let config = BinaryExportConfigBuilder::new()
        .advanced_metrics_level(AdvancedMetricsLevel::Essential)
        .source_analysis(false)  // Disable for performance
        .lifecycle_timeline(true)  // Enable for insights
        .container_analysis(true)  // Enable for container insights
        .fragmentation_analysis(false)  // Disable for performance
        .thread_context_tracking(true)  // Enable for concurrency insights
        .compression_level(1)  // Light compression
        .buffer_size(128 * 1024)  // 128KB buffer
        .build();
    
    println!("   Custom configuration created:");
    println!("     Advanced metrics level: {:?}", config.advanced_metrics_level);
    println!("     Source analysis: {}", config.source_analysis);
    println!("     Lifecycle timeline: {}", config.lifecycle_timeline);
    println!("     Container analysis: {}", config.container_analysis);
    println!("     Thread context tracking: {}", config.thread_context_tracking);
    println!("     Compression level: {}", config.compression_level);
    println!("     Buffer size: {} KB", config.buffer_size / 1024);
    println!("     Estimated performance impact: {:.1}%", 
             config.estimated_performance_impact() * 100.0);
    
    // Export with custom config
    let temp_file = NamedTempFile::new()?;
    let tracker = get_global_tracker();
    let stats = tracker.get_stats()?;
    
    export_to_binary_with_config(&stats.allocations, temp_file.path(), &config)?;
    
    let file_size = fs::metadata(temp_file.path())?.len();
    println!("   ✅ Export completed, file size: {} bytes", file_size);
    
    Ok(())
}

fn demo_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚠️  Demo 4: Configuration Validation");
    println!("------------------------------------");
    
    // Create a config with problematic settings
    let mut config = BinaryExportConfig::default();
    config.buffer_size = 100;  // Too small
    config.compression_level = 15;  // Too high
    config.advanced_metrics_level = AdvancedMetricsLevel::None;
    config.source_analysis = true;  // Conflicts with None level
    
    println!("   Original problematic config:");
    println!("     Buffer size: {} bytes", config.buffer_size);
    println!("     Compression level: {}", config.compression_level);
    println!("     Advanced metrics level: {:?}", config.advanced_metrics_level);
    println!("     Source analysis: {}", config.source_analysis);
    
    // Validate and fix
    let warnings = config.validate_and_fix();
    
    println!("\n   After validation and fixes:");
    println!("     Buffer size: {} bytes", config.buffer_size);
    println!("     Compression level: {}", config.compression_level);
    println!("     Advanced metrics level: {:?}", config.advanced_metrics_level);
    println!("     Source analysis: {}", config.source_analysis);
    
    if !warnings.is_empty() {
        println!("\n   Validation warnings:");
        for warning in warnings {
            println!("     ⚠️  {}", warning);
        }
    }
    
    println!("   ✅ Configuration validated and fixed");
    
    Ok(())
}