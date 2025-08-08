//! Basic integration tests for binary-to-JSON optimization system
//!
//! This test suite validates core functionality without heavy performance testing
//! to avoid timeout issues.

use memscope_rs::export::binary::IntegrationConfig;

#[test]
fn test_integration_config_basic() {
    println!("🧪 Testing basic integration configuration...");
    
    // Test default configuration
    let default_config = IntegrationConfig::default();
    assert!(default_config.enable_optimization);
    assert!(default_config.enable_fallback);
    assert!(default_config.log_performance);
    
    println!("✅ Default config: optimization={}, fallback={}", 
             default_config.enable_optimization, default_config.enable_fallback);
}

#[test]
fn test_integration_config_presets() {
    println!("🧪 Testing integration configuration presets...");
    
    // Test performance-optimized configuration
    let perf_config = IntegrationConfig::performance_optimized();
    assert!(perf_config.enable_optimization);
    assert!(!perf_config.enable_fallback);
    assert_eq!(perf_config.optimization_threshold, 0);
    
    // Test reliability-focused configuration
    let reliability_config = IntegrationConfig::reliability_focused();
    assert!(reliability_config.enable_optimization);
    assert!(reliability_config.enable_fallback);
    assert!(reliability_config.enable_detailed_logging);
    
    // Test legacy-compatible configuration
    let legacy_config = IntegrationConfig::legacy_compatible();
    assert!(!legacy_config.enable_optimization);
    
    println!("✅ All preset configurations work correctly");
}

#[test]
fn test_optimization_threshold() {
    println!("🧪 Testing optimization threshold logic...");
    
    let config = IntegrationConfig::default();
    
    // Test threshold logic
    assert!(!config.should_optimize(5 * 1024)); // Below 10KB threshold
    assert!(config.should_optimize(20 * 1024)); // Above 10KB threshold
    
    // Test disabled optimization
    let disabled_config = IntegrationConfig {
        enable_optimization: false,
        ..Default::default()
    };
    assert!(!disabled_config.should_optimize(100 * 1024)); // Should not optimize when disabled
    
    println!("✅ Optimization threshold logic works correctly");
}

#[test]
fn test_basic_parser_integration() {
    println!("🧪 Testing basic parser integration...");
    
    // This test just verifies that the integration functions exist and can be called
    // without actually performing heavy file operations
    
    use memscope_rs::export::binary::BinaryParser;
    
    // Test that the new method exists
    let result = BinaryParser::to_standard_json_files_with_config(
        "nonexistent.memscope", 
        "test", 
        false
    );
    
    // We expect this to fail because the file doesn't exist, but the method should exist
    assert!(result.is_err(), "Should fail with nonexistent file");
    
    println!("✅ Parser integration methods are available");
}

#[test]
fn test_environment_config_parsing() {
    println!("🧪 Testing environment configuration parsing...");
    
    // Test that environment config parsing doesn't crash
    let config = IntegrationConfig::from_environment();
    
    // Basic validation that config was created
    assert!(config.optimization_threshold > 0);
    assert!(config.max_memory_usage > 0);
    
    println!("✅ Environment configuration parsing works");
}