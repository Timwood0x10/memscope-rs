//! allocation count diagnostic program
//!
//! check why the exported file size does not linearly increase with allocation count

use memscope_rs::{get_global_tracker, init};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    tracing::info!("🔍 allocation count diagnostic program");
    tracing::info!("==================");
    tracing::info!("");

    // init memory tracker
    init();

    // run complex_lifecycle_showcase to generate test data
    tracing::info!("🔧 run complex_lifecycle_showcase to generate test data...");
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--example",
            "complex_lifecycle_showcase",
        ])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!("❌ run complex_lifecycle_showcase failed: {}", stderr);
                return;
            }
            tracing::info!("✅ test data generated");
        }
        Err(e) => {
            tracing::error!("❌ execute command failed: {}", e);
            return;
        }
    }

    // wait for system to stabilize
    std::thread::sleep(std::time::Duration::from_millis(100)); // Reduced from 1000ms to 100ms

    // 诊断分配数量
    diagnose_allocation_count();
}

fn diagnose_allocation_count() {
    tracing::info!("");
    tracing::info!("📊 diagnose allocation count and file size relationship");
    tracing::info!("==============================");

    let tracker = get_global_tracker();
    let stats = tracker.get_stats().expect("Failed to get statistics");

    tracing::info!("🔍 global tracker stats:");
    tracing::info!("  • total allocations: {}", stats.total_allocations);
    tracing::info!("  • active allocations: {}", stats.active_allocations);
    tracing::info!(
        "  • peak memory: {:.2} MB",
        stats.peak_memory as f64 / 1024.0 / 1024.0
    );
    tracing::info!(
        "  • current memory: {:.2} MB",
        stats.active_memory as f64 / 1024.0 / 1024.0
    );

    // create output directory
    let output_dir = PathBuf::from("diagnostic_results");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        tracing::error!("❌ create output directory failed: {}", e);
        return;
    }

    // test different export methods
    test_traditional_export(&output_dir, &stats);
    test_fast_export(&output_dir, &stats);
    test_raw_data_access(&stats);
}

fn test_traditional_export(output_dir: &Path, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🐌 test traditional export:");

    let output_path = output_dir.join("traditional_diagnostic.json");
    let tracker = get_global_tracker();

    match tracker.export_to_json(&output_path) {
        Ok(_) => {
            if let Ok(metadata) = fs::metadata(&output_path) {
                let file_size = metadata.len();
                tracing::info!(
                    "  • file size: {:.2} MB ({} bytes)",
                    file_size as f64 / 1024.0 / 1024.0,
                    file_size
                );
                tracing::info!(
                    "  • average allocation size: {:.1} bytes",
                    file_size as f64 / stats.total_allocations as f64
                );

                // read file content analysis
                if let Ok(content) = fs::read_to_string(&output_path) {
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(allocations) = json_value.get("allocations") {
                            if let Some(alloc_array) = allocations.as_array() {
                                tracing::info!("  • JSON allocations count: {}", alloc_array.len());
                                tracing::info!(
                                    "  • tracker reported allocations count: {}",
                                    stats.total_allocations
                                );
                                if alloc_array.len() != stats.total_allocations {
                                    tracing::info!(
                                        "  ⚠️  allocations count mismatch! possible data loss"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("  ❌ traditional export failed: {}", e);
        }
    }
}

fn test_fast_export(output_dir: &Path, stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("⚡ test fast export:");

    let output_path = output_dir.join("fast_diagnostic");

    // use fast export coordinator
    let config = memscope_rs::export::fast_export_coordinator::FastExportConfig::default();
    let mut coordinator =
        memscope_rs::export::fast_export_coordinator::FastExportCoordinator::new(config);

    match coordinator.export_fast(&output_path) {
        Ok(export_stats) => {
            tracing::info!(
                "  • processed allocations count: {}",
                export_stats.parallel_processing.total_allocations
            );
            tracing::info!(
                "  • written bytes: {:.2} MB ({} bytes)",
                export_stats.write_performance.total_bytes_written as f64 / 1024.0 / 1024.0,
                export_stats.write_performance.total_bytes_written
            );
            tracing::info!(
                "  • shard count: {}",
                export_stats.parallel_processing.shard_count
            );
            tracing::info!(
                "  • average allocation size: {:.1} bytes",
                export_stats.write_performance.total_bytes_written as f64
                    / export_stats.parallel_processing.total_allocations as f64
            );

            if export_stats.parallel_processing.total_allocations != stats.total_allocations {
                tracing::info!(
                    "  ⚠️  processed allocations count mismatch! possible data loss: exported={}, actual={}",
                    export_stats.parallel_processing.total_allocations, stats.total_allocations
                );
            }
        }
        Err(e) => {
            tracing::error!("  ❌ fast export failed: {}", e);
        }
    }
}

fn test_raw_data_access(stats: &memscope_rs::core::types::MemoryStats) {
    tracing::info!("");
    tracing::info!("🔍 test raw data access:");

    let _tracker = get_global_tracker();

    // try to access allocation data directly
    tracing::info!("  • try to access allocation data directly...");

    // we can only infer through statistics
    tracing::info!("  • tracker stats:");
    tracing::info!("    - total allocations: {}", stats.total_allocations);
    tracing::info!("    - active allocations: {}", stats.active_allocations);
    tracing::info!(
        "    - released allocations: {}",
        stats.total_allocations - stats.active_allocations
    );

    if stats.active_allocations < stats.total_allocations {
        tracing::info!(
            "  💡Find {} released allocations: this may affect the export data volume",
            stats.total_allocations - stats.active_allocations
        );
    }
}

#[cfg(test)]
mod tests {
    use memscope_rs::core::types::MemoryStats;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_allocation_count_calculation() {
        // Test allocation count calculation logic without global tracker
        let stats = MemoryStats {
            total_allocations: 1000,
            active_allocations: 600,
            total_allocated: 1024 * 1024, // 1MB
            active_memory: 512 * 1024,    // 512KB
            peak_memory: 2 * 1024 * 1024, // 2MB
            ..Default::default()
        };

        let released_allocations = stats.total_allocations - stats.active_allocations;
        assert_eq!(released_allocations, 400);

        let average_allocation_size = stats.total_allocated as f64 / stats.total_allocations as f64;
        assert_eq!(average_allocation_size, 1048.576); // 1024 bytes per allocation

        let memory_efficiency = stats.active_memory as f64 / stats.peak_memory as f64;
        assert_eq!(memory_efficiency, 0.25); // 25% efficiency
    }

    #[test]
    fn test_file_size_analysis_logic() {
        // Test file size analysis logic
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_file = temp_dir.path().join("test_export.json");

        // Create a test JSON file
        let test_data = r#"{"allocations": [{"id": 1}, {"id": 2}, {"id": 3}]}"#;
        fs::write(&test_file, test_data).expect("Failed to write test file");

        // Test file size calculation
        let metadata = fs::metadata(&test_file).expect("Failed to get file metadata");
        let file_size = metadata.len();
        assert!(file_size > 0);

        // Test average allocation size calculation
        let allocation_count = 3u64;
        let average_size = file_size as f64 / allocation_count as f64;
        assert!(average_size > 0.0);

        // Test file content analysis
        let content = fs::read_to_string(&test_file).expect("Failed to read file");
        assert!(content.contains("allocations"));
        
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(allocations) = json_value.get("allocations") {
                if let Some(alloc_array) = allocations.as_array() {
                    assert_eq!(alloc_array.len(), 3);
                }
            }
        }
    }

    #[test]
    fn test_memory_stats_validation() {
        // Test memory statistics validation logic
        let valid_stats = MemoryStats {
            total_allocations: 500,
            active_allocations: 300,
            total_allocated: 1024 * 500, // 500KB
            active_memory: 1024 * 300,   // 300KB
            peak_memory: 1024 * 600,     // 600KB
            ..Default::default()
        };

        // Validate consistency
        assert!(valid_stats.active_allocations <= valid_stats.total_allocations);
        assert!(valid_stats.active_memory <= valid_stats.peak_memory);
        assert!(valid_stats.active_memory <= valid_stats.total_allocated);

        // Test edge case: zero allocations
        let zero_stats = MemoryStats {
            total_allocations: 0,
            active_allocations: 0,
            total_allocated: 0,
            active_memory: 0,
            peak_memory: 0,
            ..Default::default()
        };

        assert_eq!(zero_stats.total_allocations, 0);
        assert_eq!(zero_stats.active_allocations, 0);
    }

    #[test]
    fn test_diagnostic_output_formatting() {
        // Test diagnostic output formatting logic
        let stats = MemoryStats {
            total_allocations: 1024,
            active_allocations: 512,
            total_allocated: 2 * 1024 * 1024, // 2MB
            active_memory: 1024 * 1024,       // 1MB
            peak_memory: 3 * 1024 * 1024,     // 3MB
            ..Default::default()
        };

        // Test memory formatting
        let peak_mb = stats.peak_memory as f64 / 1024.0 / 1024.0;
        let active_mb = stats.active_memory as f64 / 1024.0 / 1024.0;
        
        assert!((peak_mb - 3.0).abs() < 0.01);
        assert!((active_mb - 1.0).abs() < 0.01);

        // Test allocation count formatting
        let released_count = stats.total_allocations - stats.active_allocations;
        assert_eq!(released_count, 512);

        // Test percentage calculations
        let active_percentage = stats.active_allocations as f64 / stats.total_allocations as f64 * 100.0;
        assert_eq!(active_percentage, 50.0);
    }

    #[test]
    fn test_export_path_handling() {
        // Test export path handling logic
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test valid path creation
        let valid_path = temp_dir.path().join("diagnostic_results");
        let create_result = fs::create_dir_all(&valid_path);
        assert!(create_result.is_ok());
        assert!(valid_path.exists());

        // Test file path generation
        let json_path = valid_path.join("traditional_diagnostic.json");
        let binary_path = valid_path.join("fast_diagnostic");
        
        assert!(json_path.to_str().is_some());
        assert!(binary_path.to_str().is_some());
        assert!(json_path.extension().unwrap() == "json");
    }

    #[test]
    fn test_allocation_mismatch_detection() {
        // Test allocation count mismatch detection logic
        let tracker_stats = MemoryStats {
            total_allocations: 1000,
            active_allocations: 600,
            ..Default::default()
        };

        // Simulate JSON export with different allocation count
        let json_allocation_count = 950;
        let has_mismatch = json_allocation_count != tracker_stats.total_allocations;
        assert!(has_mismatch);

        let mismatch_count = tracker_stats.total_allocations - json_allocation_count;
        assert_eq!(mismatch_count, 50);

        // Test no mismatch case
        let matching_count = 1000;
        let no_mismatch = matching_count == tracker_stats.total_allocations;
        assert!(no_mismatch);
    }

    #[test]
    fn test_performance_metrics_calculation() {
        // Test performance metrics calculation
        let file_size_bytes = 1024 * 1024; // 1MB
        let allocation_count = 1000u64;
        let processing_time_ms = 500u64;

        // Calculate metrics
        let file_size_mb = file_size_bytes as f64 / 1024.0 / 1024.0;
        let avg_allocation_size = file_size_bytes as f64 / allocation_count as f64;
        let throughput = allocation_count as f64 / (processing_time_ms as f64 / 1000.0);

        assert_eq!(file_size_mb, 1.0);
        assert_eq!(avg_allocation_size, 1048.576);
        assert_eq!(throughput, 2000.0); // 2000 allocations per second

        // Test edge cases
        assert!(avg_allocation_size > 0.0);
        assert!(throughput > 0.0);
    }
}
