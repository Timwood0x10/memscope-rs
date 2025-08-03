//! Test mode-specific validation configuration
//!
//! This binary tests the new mode-specific validation system to ensure
//! different export modes work correctly with their respective validation configurations.

use memscope_rs::export::export_modes::ExportCoordinator;
use memscope_rs::export::quality_validator::{
    ExportConfig, ExportMode, ExportModeManager, ValidationConfig, ValidationStrategy,
    ValidationTiming,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🧪 Testing Mode-Specific Validation Configuration");
    tracing::info!("================================================");

    // Test 1: Basic mode configurations
    test_basic_mode_configurations()?;

    // Test 2: Configuration validation and conflict resolution
    test_configuration_validation()?;

    // Test 3: Export mode manager
    test_export_mode_manager()?;

    // Test 4: Export coordinator with different modes
    test_export_coordinator()?;

    tracing::info!("\n✅ All mode-specific validation tests passed!");
    Ok(())
}

fn test_basic_mode_configurations() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("\n📋 Test 1: Basic Mode Configurations");
    tracing::info!("------------------------------------");

    // Test fast mode configuration
    let fast_config = ValidationConfig::for_fast_mode();
    tracing::info!("Fast mode config:");
    tracing::info!("  JSON validation: {}", fast_config.enable_json_validation);
    tracing::info!(
        "  Integrity validation: {}",
        fast_config.enable_integrity_validation
    );
    tracing::info!("  Size validation: {}", fast_config.enable_size_validation);
    tracing::info!(
        "  Max data loss rate: {:.2}%",
        fast_config.max_data_loss_rate * 100.0
    );

    // Test slow mode configuration
    let slow_config = ValidationConfig::for_slow_mode();
    tracing::info!("\nSlow mode config:");
    tracing::info!("  JSON validation: {}", slow_config.enable_json_validation);
    tracing::info!(
        "  Integrity validation: {}",
        slow_config.enable_integrity_validation
    );
    tracing::info!("  Size validation: {}", slow_config.enable_size_validation);
    tracing::info!(
        "  Max data loss rate: {:.2}%",
        slow_config.max_data_loss_rate * 100.0
    );

    // Test validation strategies
    let minimal_config = ValidationConfig::with_strategy(ValidationStrategy::Minimal);
    let comprehensive_config = ValidationConfig::with_strategy(ValidationStrategy::Comprehensive);

    tracing::info!(
        "\nMinimal strategy: JSON validation = {}",
        minimal_config.enable_json_validation
    );
    tracing::info!(
        "Comprehensive strategy: JSON validation = {}",
        comprehensive_config.enable_json_validation
    );

    assert!(!fast_config.enable_json_validation);
    assert!(slow_config.enable_json_validation);
    assert!(!minimal_config.enable_json_validation);
    assert!(comprehensive_config.enable_json_validation);

    tracing::info!("✅ Basic mode configurations test passed");
    Ok(())
}

fn test_configuration_validation() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("\n🔍 Test 2: Configuration Validation");
    tracing::info!("-----------------------------------");

    // Test conflict detection
    let mut fast_inline_config = ExportConfig::new(ExportMode::Fast, ValidationTiming::Inline);
    tracing::info!("Testing Fast mode with Inline validation (should conflict):");
    let warnings = fast_inline_config.validate_and_fix();
    tracing::info!("  Warnings generated: {}", warnings.len());
    for warning in &warnings {
        tracing::info!("    - {}", warning);
    }
    tracing::info!("  Fixed timing: {:?}", fast_inline_config.validation_timing);

    // Test slow mode with disabled validation
    let mut slow_disabled_config = ExportConfig::new(ExportMode::Slow, ValidationTiming::Disabled);
    tracing::info!("\nTesting Slow mode with Disabled validation (should conflict):");
    let warnings = slow_disabled_config.validate_and_fix();
    tracing::info!("  Warnings generated: {}", warnings.len());
    for warning in &warnings {
        tracing::info!("    - {}", warning);
    }
    tracing::info!(
        "  Fixed timing: {:?}",
        slow_disabled_config.validation_timing
    );

    // Test validation config conflicts
    let mut validation_config = ValidationConfig::for_fast_mode();
    validation_config.enable_json_validation = true; // This conflicts with fast mode

    let conflicts = validation_config.conflicts_with_mode(&ExportMode::Fast);
    tracing::info!("\nValidation config conflicts with Fast mode:");
    for conflict in &conflicts {
        tracing::info!("  - {}", conflict);
    }

    assert_eq!(
        fast_inline_config.validation_timing,
        ValidationTiming::Deferred
    );
    assert_eq!(
        slow_disabled_config.validation_timing,
        ValidationTiming::Deferred
    );
    assert!(!conflicts.is_empty());

    tracing::info!("✅ Configuration validation test passed");
    Ok(())
}

fn test_export_mode_manager() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("\n⚙️  Test 3: Export Mode Manager");
    tracing::info!("------------------------------");

    let manager = ExportModeManager::new();
    let (default_mode, threshold, perf_threshold) = manager.get_settings();
    tracing::info!("Default settings:");
    tracing::info!("  Default mode: {:?}", default_mode);
    tracing::info!(
        "  Auto threshold: {:.2} MB",
        threshold as f64 / 1024.0 / 1024.0
    );
    tracing::info!("  Performance threshold: {} ms", perf_threshold);

    // Test auto mode selection
    let auto_manager = ExportModeManager::with_settings(
        ExportMode::Auto,
        5 * 1024 * 1024, // 5MB threshold
        3000,
    );

    let small_data_mode = auto_manager.determine_optimal_mode(1024 * 1024); // 1MB
    let large_data_mode = auto_manager.determine_optimal_mode(10 * 1024 * 1024); // 10MB

    tracing::info!("\nAuto mode selection:");
    tracing::info!("  Small data (1MB): {:?}", small_data_mode);
    tracing::info!("  Large data (10MB): {:?}", large_data_mode);

    // Test configuration creation
    let auto_config = auto_manager.create_auto_config(1024 * 1024); // 1MB
    tracing::info!("\nAuto config for 1MB data:");
    tracing::info!("  Mode: {:?}", auto_config.mode);
    tracing::info!("  Validation timing: {:?}", auto_config.validation_timing);

    // Test configuration optimization
    let test_config = ExportConfig::fast();
    let (_optimized_config, optimization_warnings) = auto_manager.optimize_config(
        test_config,
        200 * 1024 * 1024, // 200MB - large dataset
    );

    tracing::info!("\nOptimization for large dataset (200MB):");
    tracing::info!("  Optimization warnings: {}", optimization_warnings.len());
    for warning in &optimization_warnings {
        tracing::info!("    - {}", warning);
    }

    assert_eq!(small_data_mode, ExportMode::Slow);
    assert_eq!(large_data_mode, ExportMode::Fast);

    tracing::info!("✅ Export mode manager test passed");
    Ok(())
}

fn test_export_coordinator() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("\n🎯 Test 4: Export Coordinator");
    tracing::info!("-----------------------------");

    // Test different coordinator creation methods
    let fast_coordinator = ExportCoordinator::new_fast();
    let slow_coordinator = ExportCoordinator::new_slow();
    let auto_coordinator = ExportCoordinator::new_auto();
    let sized_coordinator = ExportCoordinator::new_auto_sized(50 * 1024 * 1024); // 50MB

    tracing::info!("Coordinator configurations:");
    tracing::info!(
        "  Fast coordinator mode: {:?}",
        fast_coordinator.config().mode
    );
    tracing::info!(
        "  Slow coordinator mode: {:?}",
        slow_coordinator.config().mode
    );
    tracing::info!(
        "  Auto coordinator mode: {:?}",
        auto_coordinator.config().mode
    );
    tracing::info!(
        "  Sized coordinator mode: {:?}",
        sized_coordinator.config().mode
    );

    // Test configuration updates
    let mut coordinator = ExportCoordinator::new_fast();
    let new_config = ExportConfig::new(ExportMode::Slow, ValidationTiming::Inline);
    let warnings = coordinator.update_config(new_config, Some(100 * 1024 * 1024)); // 100MB

    tracing::info!("\nConfiguration update with 100MB dataset:");
    tracing::info!("  Update warnings: {}", warnings.len());
    for warning in &warnings {
        tracing::info!("    - {}", warning);
    }
    tracing::info!("  Final mode: {:?}", coordinator.config().mode);

    assert_eq!(fast_coordinator.config().mode, ExportMode::Fast);
    assert_eq!(slow_coordinator.config().mode, ExportMode::Slow);
    assert_eq!(auto_coordinator.config().mode, ExportMode::Auto);

    tracing::info!("✅ Export coordinator test passed");
    Ok(())
}
