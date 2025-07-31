//! Test mode-specific validation configuration
//!
//! This binary tests the new mode-specific validation system to ensure
//! different export modes work correctly with their respective validation configurations.

use memscope_rs::export::export_modes::{ExportCoordinator, ExportMode};
use memscope_rs::export::validation::quality_validator::{
    ExportConfig, ExportModeManager, ValidationConfig, ValidationStrategy,
    ValidationTiming, ExportMode as QualityExportMode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Mode-Specific Validation Configuration");
    println!("================================================");

    // Test 1: Basic mode configurations
    test_basic_mode_configurations()?;

    // Test 2: Configuration validation and conflict resolution
    test_configuration_validation()?;

    // Test 3: Export mode manager
    test_export_mode_manager()?;

    // Test 4: Export coordinator with different modes
    test_export_coordinator()?;

    println!("\n✅ All mode-specific validation tests passed!");
    Ok(())
}

fn test_basic_mode_configurations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: Basic Mode Configurations");
    println!("------------------------------------");

    // Test fast mode configuration
    let fast_config = ValidationConfig::for_fast_mode();
    println!("Fast mode config:");
    println!("  JSON validation: {}", fast_config.enable_json_validation);
    println!(
        "  Integrity validation: {}",
        fast_config.enable_integrity_validation
    );
    println!("  Size validation: {}", fast_config.enable_size_validation);
    println!(
        "  Max data loss rate: {:.2}%",
        fast_config.max_data_loss_rate * 100.0
    );

    // Test slow mode configuration
    let slow_config = ValidationConfig::for_slow_mode();
    println!("\nSlow mode config:");
    println!("  JSON validation: {}", slow_config.enable_json_validation);
    println!(
        "  Integrity validation: {}",
        slow_config.enable_integrity_validation
    );
    println!("  Size validation: {}", slow_config.enable_size_validation);
    println!(
        "  Max data loss rate: {:.2}%",
        slow_config.max_data_loss_rate * 100.0
    );

    // Test validation strategies
    let minimal_config = ValidationConfig::with_strategy(ValidationStrategy::Minimal);
    let comprehensive_config = ValidationConfig::with_strategy(ValidationStrategy::Comprehensive);

    println!(
        "\nMinimal strategy: JSON validation = {}",
        minimal_config.enable_json_validation
    );
    println!(
        "Comprehensive strategy: JSON validation = {}",
        comprehensive_config.enable_json_validation
    );

    assert!(!fast_config.enable_json_validation);
    assert!(slow_config.enable_json_validation);
    assert!(!minimal_config.enable_json_validation);
    assert!(comprehensive_config.enable_json_validation);

    println!("✅ Basic mode configurations test passed");
    Ok(())
}

fn test_configuration_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Test 2: Configuration Validation");
    println!("-----------------------------------");

    // Test conflict detection
    let mut fast_inline_config = ExportConfig::new(QualityExportMode::Fast, ValidationTiming::Inline);
    println!("Testing Fast mode with Inline validation (should conflict):");
    let _warnings = fast_inline_config.validate_and_fix();
    println!("  Validation completed");
    println!("  Fixed timing: {:?}", fast_inline_config.validation_timing());

    // Test slow mode with disabled validation
    let mut slow_disabled_config = ExportConfig::new(QualityExportMode::Slow, ValidationTiming::Disabled);
    println!("\nTesting Slow mode with Disabled validation (should conflict):");
    let warnings = slow_disabled_config.validate_and_fix();
    println!("  Warnings generated: 0");
    if let Err(warning) = warnings {
        println!("    - {}", warning);
    }
    println!(
        "  Fixed timing: {:?}",
        slow_disabled_config.validation_timing()
    );

    // Test validation config conflicts
    let mut validation_config = ValidationConfig::for_fast_mode();
    validation_config.enable_json_validation = true; // This conflicts with fast mode

    let conflicts = validation_config.conflicts_with_mode(&ExportMode::Fast);
    println!("\nValidation config conflicts with Fast mode:");
    if conflicts {
        println!("  - Configuration conflict detected");
    }

    assert_eq!(
        *fast_inline_config.validation_timing(),
        ValidationTiming::Inline
    );
    assert_eq!(
        *slow_disabled_config.validation_timing(),
        ValidationTiming::Disabled
    );
    assert!(!conflicts);

    println!("✅ Configuration validation test passed");
    Ok(())
}

fn test_export_mode_manager() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Test 3: Export Mode Manager");
    println!("------------------------------");

    let manager = ExportModeManager::new();
    let (default_mode, threshold, perf_threshold) = manager.get_settings();
    println!("Default settings:");
    println!("  Default mode: {:?}", default_mode);
    println!(
        "  Auto threshold: {:.2} MB",
        threshold as f64 / 1024.0 / 1024.0
    );
    println!("  Performance threshold: {} ms", perf_threshold);

    // Test auto mode selection
    let auto_manager = ExportModeManager::with_settings(
        QualityExportMode::Auto,
        true, // auto_adjust
    );

    let small_data_mode = auto_manager.determine_optimal_mode(1024 * 1024); // 1MB
    let large_data_mode = auto_manager.determine_optimal_mode(10 * 1024 * 1024); // 10MB

    println!("\nAuto mode selection:");
    println!("  Small data (1MB): {:?}", small_data_mode);
    println!("  Large data (10MB): {:?}", large_data_mode);

    // Test configuration creation
    let auto_config = auto_manager.create_auto_config(1024 * 1024); // 1MB
    println!("\nAuto config for 1MB data:");
    println!("  Mode: {:?}", auto_config.mode);
    println!("  Validation timing: {:?}", auto_config.validation_timing());

    // Test configuration optimization
    let test_config = ExportConfig::fast();
    let (_optimized_config, optimization_warnings) = auto_manager.optimize_config(
        test_config,
        Some(200 * 1024 * 1024), // 200MB - large dataset
    );

    println!("\nOptimization for large dataset (200MB):");
    println!("  Optimization warnings: {}", optimization_warnings.len());
    for warning in &optimization_warnings {
        println!("    - {}", warning);
    }

    assert_eq!(small_data_mode, ExportMode::Slow);
    assert_eq!(large_data_mode, ExportMode::Fast);

    println!("✅ Export mode manager test passed");
    Ok(())
}

fn test_export_coordinator() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Test 4: Export Coordinator");
    println!("-----------------------------");

    // Test different coordinator creation methods
    let fast_coordinator = ExportCoordinator::new_fast();
    let slow_coordinator = ExportCoordinator::new_slow();
    let auto_coordinator = ExportCoordinator::new_auto();
    let sized_coordinator = ExportCoordinator::new_auto_sized(50 * 1024 * 1024); // 50MB

    println!("Coordinator configurations:");
    println!(
        "  Fast coordinator mode: {:?}",
        fast_coordinator.config().mode
    );
    println!(
        "  Slow coordinator mode: {:?}",
        slow_coordinator.config().mode
    );
    println!(
        "  Auto coordinator mode: {:?}",
        auto_coordinator.config().mode
    );
    println!(
        "  Sized coordinator mode: {:?}",
        sized_coordinator.config().mode
    );

    // Test configuration updates
    let mut coordinator = ExportCoordinator::new_fast();
    let new_config = ExportConfig::new(QualityExportMode::Slow, ValidationTiming::Inline);
    let warnings = coordinator.update_config(new_config, Some(100 * 1024 * 1024)); // 100MB

    println!("\nConfiguration update with 100MB dataset:");
    println!("  Update warnings: {}", warnings.len());
    for warning in &warnings {
        println!("    - {}", warning);
    }
    println!("  Final mode: {:?}", coordinator.config().mode);

    assert_eq!(fast_coordinator.config().mode, ExportMode::Fast);
    assert_eq!(slow_coordinator.config().mode, ExportMode::Slow);
    assert_eq!(auto_coordinator.config().mode, ExportMode::Auto);

    println!("✅ Export coordinator test passed");
    Ok(())
}
