//! Test FastExportCoordinator integration with mode system
//!
//! This binary tests the integration between FastExportCoordinator and the new
//! export mode system with deferred validation capabilities.

use memscope_rs::export::{
    ExportConfig, ExportMode, ValidationTiming, 
    FastExportCoordinator
};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing FastExportCoordinator Mode Integration");
    println!("===============================================");

    // Test 1: Basic coordinator creation with export config
    test_coordinator_creation()?;
    
    // Test 2: Export configuration updates
    test_export_config_updates()?;
    
    // Test 3: Mode-specific export behavior (mock test)
    test_mode_specific_behavior().await?;

    println!("\n✅ All FastExportCoordinator integration tests passed!");
    Ok(())
}

fn test_coordinator_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: Coordinator Creation with Export Config");
    println!("--------------------------------------------------");

    // Test creation with different export configurations
    let fast_config = ExportConfig::fast();
    let slow_config = ExportConfig::slow();
    let auto_config = ExportConfig::auto();

    let fast_coordinator = FastExportCoordinator::new_with_export_config(fast_config.clone());
    let slow_coordinator = FastExportCoordinator::new_with_export_config(slow_config.clone());
    let auto_coordinator = FastExportCoordinator::new_with_export_config(auto_config.clone());

    println!("Fast coordinator export config:");
    println!("  Mode: {:?}", fast_coordinator.get_export_config().mode);
    println!("  Validation timing: {:?}", fast_coordinator.get_export_config().validation_timing);
    println!("  JSON validation: {}", fast_coordinator.get_export_config().validation_config.enable_json_validation);

    println!("\nSlow coordinator export config:");
    println!("  Mode: {:?}", slow_coordinator.get_export_config().mode);
    println!("  Validation timing: {:?}", slow_coordinator.get_export_config().validation_timing);
    println!("  JSON validation: {}", slow_coordinator.get_export_config().validation_config.enable_json_validation);

    println!("\nAuto coordinator export config:");
    println!("  Mode: {:?}", auto_coordinator.get_export_config().mode);
    println!("  Validation timing: {:?}", auto_coordinator.get_export_config().validation_timing);
    println!("  JSON validation: {}", auto_coordinator.get_export_config().validation_config.enable_json_validation);

    // Verify configurations
    assert_eq!(fast_coordinator.get_export_config().mode, ExportMode::Fast);
    assert_eq!(slow_coordinator.get_export_config().mode, ExportMode::Slow);
    assert_eq!(auto_coordinator.get_export_config().mode, ExportMode::Auto);

    assert_eq!(fast_coordinator.get_export_config().validation_timing, ValidationTiming::Deferred);
    assert_eq!(slow_coordinator.get_export_config().validation_timing, ValidationTiming::Inline);

    println!("✅ Coordinator creation test passed");
    Ok(())
}

fn test_export_config_updates() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Test 2: Export Configuration Updates");
    println!("--------------------------------------");

    let mut coordinator = FastExportCoordinator::new_fast_mode();
    
    println!("Initial configuration:");
    println!("  Mode: {:?}", coordinator.get_export_config().mode);
    println!("  JSON validation: {}", coordinator.get_export_config().validation_config.enable_json_validation);

    // Update to slow mode configuration
    let slow_config = ExportConfig::slow();
    coordinator.update_export_config(slow_config);

    println!("\nAfter update to slow mode:");
    println!("  Mode: {:?}", coordinator.get_export_config().mode);
    println!("  JSON validation: {}", coordinator.get_export_config().validation_config.enable_json_validation);
    println!("  Validation timing: {:?}", coordinator.get_export_config().validation_timing);

    // Verify the update worked
    assert_eq!(coordinator.get_export_config().mode, ExportMode::Slow);
    assert!(coordinator.get_export_config().validation_config.enable_json_validation);
    assert_eq!(coordinator.get_export_config().validation_timing, ValidationTiming::Inline);

    println!("✅ Export configuration update test passed");
    Ok(())
}

async fn test_mode_specific_behavior() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Test 3: Mode-Specific Export Behavior");
    println!("----------------------------------------");

    // Test different export modes (mock test since we don't have real data)
    let test_configs = vec![
        ("Fast + Deferred", ExportConfig::new(ExportMode::Fast, ValidationTiming::Deferred)),
        ("Fast + Disabled", ExportConfig::new(ExportMode::Fast, ValidationTiming::Disabled)),
        ("Slow + Inline", ExportConfig::new(ExportMode::Slow, ValidationTiming::Inline)),
        ("Slow + Deferred", ExportConfig::new(ExportMode::Slow, ValidationTiming::Deferred)),
        ("Auto + Deferred", ExportConfig::new(ExportMode::Auto, ValidationTiming::Deferred)),
    ];

    for (name, config) in test_configs {
        println!("\nTesting configuration: {}", name);
        println!("  Mode: {:?}", config.mode);
        println!("  Validation timing: {:?}", config.validation_timing);
        
        let coordinator = FastExportCoordinator::new_with_export_config(config.clone());
        
        // Verify the configuration was applied correctly
        assert_eq!(coordinator.get_export_config().mode, config.mode);
        assert_eq!(coordinator.get_export_config().validation_timing, config.validation_timing);
        
        println!("  ✅ Configuration applied correctly");
        
        // Note: We can't actually test export_with_mode without real data,
        // but we can verify the coordinator is set up correctly
    }

    println!("✅ Mode-specific behavior test passed");
    Ok(())
}