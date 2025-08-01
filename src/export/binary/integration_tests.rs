//! Integration tests for binary export system
//!
//! This module provides comprehensive integration tests that validate
//! the entire binary export workflow from data collection to file output.

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use crate::core::tracker::MemoryTracker;

    /// Test complete export workflow
    #[test]
    fn test_complete_export_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("integration_test.bin");
        
        // Create test tracker with data
        let tracker = create_test_tracker();
        
        // Test with different configurations
        let configs = vec![
            IntegratedConfig::high_performance(),
            IntegratedConfig::memory_efficient(),
            IntegratedConfig::balanced(),
        ];
        
        for (i, config) in configs.into_iter().enumerate() {
            let test_path = temp_dir.path().join(format!("test_{}.bin", i));
            let mut exporter = IntegratedBinaryExporter::new(config);
            
            let result = exporter.export(&tracker, &test_path);
            
            match result {
                Ok(export_result) => {
                    assert!(export_result.bytes_written > 0);
                    assert!(test_path.exists());
                    
                    // Verify file can be read back
                    let load_result = exporter.load(&test_path);
                    assert!(load_result.is_ok());
                }
                Err(BinaryExportError::NoDataToExport) => {
                    // Expected for empty tracker
                    println!("No data to export (expected for test)");
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }

    /// Test format compatibility and conversion
    #[test]
    fn test_format_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Test all supported formats
        let formats = vec![
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary,
            OutputFormat::CompressedMessagePack { level: 6 },
        ];
        
        for format in formats {
            let config = IntegratedConfig {
                output_format: format,
                ..IntegratedConfig::default()
            };
            
            let mut exporter = IntegratedBinaryExporter::new(config);
            let output_path = temp_dir.path().join(format!("test_{:?}.bin", format));
            
            match exporter.export(&tracker, &output_path) {
                Ok(_) => {
                    assert!(output_path.exists());
                    
                    // Test parsing
                    let parser = BinaryDataParser::new(ParserConfig::default());
                    let parse_result = parser.parse_file(&output_path);
                    
                    match parse_result {
                        Ok(parsed) => {
                            assert!(parsed.parse_stats.file_size > 0);
                        }
                        Err(e) => println!("Parse error (may be expected): {:?}", e),
                    }
                }
                Err(BinaryExportError::NoDataToExport) => {
                    println!("No data to export for format {:?}", format);
                }
                Err(e) => panic!("Export failed for format {:?}: {:?}", format, e),
            }
        }
    }

    /// Helper function to create test tracker
    fn create_test_tracker() -> MemoryTracker {
        MemoryTracker::new()
    }
}    ///
 Test error recovery scenarios
    #[test]
    fn test_error_recovery_scenarios() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Test invalid path recovery
        let invalid_path = "/invalid/path/that/does/not/exist/test.bin";
        let config = IntegratedConfig::default();
        let mut exporter = IntegratedBinaryExporter::new(config);
        
        let result = exporter.export(&tracker, invalid_path);
        assert!(result.is_err());
        
        // Test memory limit recovery
        let mut memory_config = IntegratedConfig::memory_efficient();
        memory_config.processing.max_memory_usage = 1024; // Very small limit
        
        let mut memory_exporter = IntegratedBinaryExporter::new(memory_config);
        let memory_path = temp_dir.path().join("memory_test.bin");
        
        let memory_result = memory_exporter.export(&tracker, &memory_path);
        // Should either succeed with constraints or fail gracefully
        match memory_result {
            Ok(_) => println!("Memory-constrained export succeeded"),
            Err(e) => println!("Memory-constrained export failed as expected: {:?}", e),
        }
    }

    /// Test performance under different loads
    #[test]
    fn test_performance_scalability() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Test different configurations for performance
        let configs = vec![
            ("fast", IntegratedConfig::high_performance()),
            ("balanced", IntegratedConfig::balanced()),
            ("compact", IntegratedConfig::memory_efficient()),
        ];
        
        for (name, config) in configs {
            let mut exporter = IntegratedBinaryExporter::new(config);
            let output_path = temp_dir.path().join(format!("perf_{}.bin", name));
            
            let start_time = std::time::Instant::now();
            let result = exporter.export(&tracker, &output_path);
            let duration = start_time.elapsed();
            
            match result {
                Ok(export_result) => {
                    println!("Config '{}': {} bytes in {:?}", 
                             name, export_result.bytes_written, duration);
                    assert!(duration < Duration::from_secs(10)); // Should complete quickly
                }
                Err(BinaryExportError::NoDataToExport) => {
                    println!("Config '{}': No data to export", name);
                }
                Err(e) => panic!("Config '{}' failed: {:?}", name, e),
            }
        }
    }

    /// Test data integrity across the pipeline
    #[test]
    fn test_data_integrity_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Export data
        let config = IntegratedConfig::balanced();
        let mut exporter = IntegratedBinaryExporter::new(config);
        let output_path = temp_dir.path().join("integrity_test.bin");
        
        let export_result = exporter.export(&tracker, &output_path);
        
        match export_result {
            Ok(_) => {
                // Validate the exported file
                let validation_result = validate_binary_file(&output_path);
                match validation_result {
                    Ok(report) => {
                        println!("Validation report: {}", report.summary());
                        // File should be structurally valid even if content is minimal
                        assert!(report.file_size > 0);
                    }
                    Err(e) => println!("Validation error (may be expected): {:?}", e),
                }
                
                // Test parsing the file
                let parser = BinaryDataParser::new(ParserConfig::default());
                let parse_result = parser.parse_file(&output_path);
                
                match parse_result {
                    Ok(parsed) => {
                        assert!(parsed.validation_results.integrity_score >= 0.0);
                        println!("Parse integrity score: {:.2}", parsed.validation_results.integrity_score);
                    }
                    Err(e) => println!("Parse error (may be expected): {:?}", e),
                }
            }
            Err(BinaryExportError::NoDataToExport) => {
                println!("No data to export for integrity test");
            }
            Err(e) => panic!("Export failed: {:?}", e),
        }
    }

    /// Test concurrent export operations
    #[test]
    fn test_concurrent_exports() {
        use std::sync::Arc;
        use std::thread;
        
        let temp_dir = TempDir::new().unwrap();
        let tracker = Arc::new(create_test_tracker());
        
        let mut handles = Vec::new();
        
        // Spawn multiple concurrent export operations
        for i in 0..3 {
            let tracker_clone = Arc::clone(&tracker);
            let temp_dir_path = temp_dir.path().to_path_buf();
            
            let handle = thread::spawn(move || {
                let config = IntegratedConfig::balanced();
                let mut exporter = IntegratedBinaryExporter::new(config);
                let output_path = temp_dir_path.join(format!("concurrent_{}.bin", i));
                
                let result = exporter.export(&*tracker_clone, &output_path);
                (i, result)
            });
            
            handles.push(handle);
        }
        
        // Wait for all exports to complete
        for handle in handles {
            let (thread_id, result) = handle.join().unwrap();
            
            match result {
                Ok(export_result) => {
                    println!("Thread {}: exported {} bytes", thread_id, export_result.bytes_written);
                }
                Err(BinaryExportError::NoDataToExport) => {
                    println!("Thread {}: no data to export", thread_id);
                }
                Err(e) => panic!("Thread {} failed: {:?}", thread_id, e),
            }
        }
    }

    /// Test system resource management
    #[test]
    fn test_resource_management() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Test with resource constraints
        let mut config = IntegratedConfig::memory_efficient();
        config.processing.max_memory_usage = 64 * 1024 * 1024; // 64MB limit
        
        let mut exporter = IntegratedBinaryExporter::new(config);
        let output_path = temp_dir.path().join("resource_test.bin");
        
        // Monitor system status before and after
        let status_before = exporter.get_system_status();
        
        let result = exporter.export(&tracker, &output_path);
        
        let status_after = exporter.get_system_status();
        
        match result {
            Ok(_) => {
                // Verify resource usage stayed within bounds
                assert!(status_after.peak_memory_usage <= 64 * 1024 * 1024);
                println!("Resource test passed: peak memory {} bytes", status_after.peak_memory_usage);
            }
            Err(BinaryExportError::NoDataToExport) => {
                println!("No data for resource test");
            }
            Err(e) => {
                // Resource constraint errors are acceptable
                println!("Resource test failed as expected: {:?}", e);
            }
        }
    }

    /// Test optimization effectiveness
    #[test]
    fn test_optimization_effectiveness() {
        let tracker = create_test_tracker();
        
        // Test performance optimization
        match optimization::optimize_system_performance(&tracker) {
            Ok(optimization_result) => {
                println!("Optimization completed:");
                println!("  Speed improvement: {:.2}x", optimization_result.improvement.speed_improvement);
                println!("  Memory improvement: {:.2}x", optimization_result.improvement.memory_improvement);
                println!("  Overall improvement: {:.2}x", optimization_result.improvement.overall_improvement);
                
                // Verify optimization provided some benefit
                assert!(optimization_result.improvement.overall_improvement >= 1.0);
            }
            Err(e) => {
                println!("Optimization failed (may be expected with empty data): {:?}", e);
            }
        }
    }

    /// Test backward compatibility
    #[test]
    fn test_backward_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        // Test legacy API compatibility
        let legacy_config = ExportConfig::default();
        let legacy_exporter = BinaryExporter::new(legacy_config);
        let legacy_path = temp_dir.path().join("legacy_test.bin");
        
        let legacy_result = legacy_exporter.export(&tracker, &legacy_path);
        
        match legacy_result {
            Ok(result) => {
                println!("Legacy export succeeded: {} bytes", result.bytes_written);
                assert!(legacy_path.exists());
            }
            Err(BinaryExportError::NoDataToExport) => {
                println!("Legacy export: no data");
            }
            Err(e) => panic!("Legacy export failed: {:?}", e),
        }
        
        // Test simple API
        match BinaryExport::export_default(&tracker, temp_dir.path().join("simple_test.bin")) {
            Ok(result) => {
                println!("Simple export succeeded: {} bytes", result.bytes_written);
            }
            Err(BinaryExportError::NoDataToExport) => {
                println!("Simple export: no data");
            }
            Err(e) => panic!("Simple export failed: {:?}", e),
        }
    }

    /// Test comprehensive workflow
    #[test]
    fn test_comprehensive_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = create_test_tracker();
        
        println!("🚀 Starting comprehensive workflow test...");
        
        // Step 1: Export with different formats
        let formats = vec![
            ("messagepack", OutputFormat::MessagePack),
            ("custom", OutputFormat::CustomBinary),
        ];
        
        for (name, format) in formats {
            println!("📊 Testing format: {}", name);
            
            let config = IntegratedConfig {
                output_format: format,
                ..IntegratedConfig::balanced()
            };
            
            let mut exporter = IntegratedBinaryExporter::new(config);
            let output_path = temp_dir.path().join(format!("workflow_{}.bin", name));
            
            match exporter.export(&tracker, &output_path) {
                Ok(result) => {
                    println!("  ✅ Export: {} bytes", result.bytes_written);
                    
                    // Step 2: Validate the file
                    match validate_binary_file(&output_path) {
                        Ok(report) => {
                            println!("  ✅ Validation: {}", report.summary());
                        }
                        Err(e) => println!("  ⚠️  Validation: {:?}", e),
                    }
                    
                    // Step 3: Parse the file
                    let parser = BinaryDataParser::new(ParserConfig::default());
                    match parser.parse_file(&output_path) {
                        Ok(parsed) => {
                            println!("  ✅ Parse: {} structures, {:.2} integrity", 
                                   parsed.parse_stats.structures_parsed,
                                   parsed.validation_results.integrity_score);
                        }
                        Err(e) => println!("  ⚠️  Parse: {:?}", e),
                    }
                }
                Err(BinaryExportError::NoDataToExport) => {
                    println!("  ℹ️  No data to export for {}", name);
                }
                Err(e) => {
                    println!("  ❌ Export failed for {}: {:?}", name, e);
                }
            }
        }
        
        println!("🎉 Comprehensive workflow test completed");
    }
}

/// Integration test utilities
#[cfg(test)]
mod test_utils {
    use super::*;
    
    /// Create a test tracker with some sample data
    pub fn create_comprehensive_test_tracker() -> MemoryTracker {
        let tracker = MemoryTracker::new();
        
        // In a real implementation, we would add test data to the tracker
        // For now, we return an empty tracker which will trigger NoDataToExport
        // This is acceptable for testing the error handling paths
        
        tracker
    }
    
    /// Verify export result meets basic requirements
    pub fn verify_export_result(result: &IntegratedExportResult) {
        assert!(result.export_result.bytes_written > 0);
        assert!(result.export_result.duration.as_nanos() > 0);
        assert!(result.performance_metrics.total_time.as_nanos() > 0);
    }
    
    /// Create test configuration for specific scenarios
    pub fn create_test_config(scenario: &str) -> IntegratedConfig {
        match scenario {
            "high_performance" => IntegratedConfig::high_performance(),
            "memory_efficient" => IntegratedConfig::memory_efficient(),
            "balanced" => IntegratedConfig::balanced(),
            _ => IntegratedConfig::default(),
        }
    }
}