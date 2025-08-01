//! Examples and usage patterns for binary export system
//!
//! This module provides comprehensive examples showing how to use
//! the binary export system in various scenarios.

#[allow(dead_code)]
pub mod examples {
    use super::super::*;
    use std::path::Path;
    use crate::core::tracker::MemoryTracker;

    /// Example 1: Basic binary export
    /// 
    /// This is the simplest way to export memory data to binary format.
    pub fn basic_export_example() -> Result<(), BinaryExportError> {
        // Create or get your memory tracker
        let tracker = MemoryTracker::new();
        
        // Simple export with default settings
        let result = BinaryExport::export_default(&tracker, "output.bin")?;
        
        println!("✅ Export completed:");
        println!("   📁 File size: {} bytes", result.bytes_written);
        println!("   ⏱️  Duration: {:?}", result.duration);
        println!("   📊 Allocations: {}", result.allocation_count);
        
        Ok(())
    }

    /// Example 2: High-performance export
    /// 
    /// For maximum speed when exporting large datasets.
    pub fn high_performance_export_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Configure for maximum performance
        let config = IntegratedConfig::high_performance();
        let mut exporter = IntegratedBinaryExporter::new(config);
        
        let result = exporter.export(&tracker, "high_perf_output.bin")?;
        
        println!("🚀 High-performance export completed:");
        println!("   📁 File size: {} bytes", result.export_result.bytes_written);
        println!("   ⚡ Throughput: {:.2} MB/s", 
                 result.performance_metrics.throughput.overall_throughput / 1_000_000.0);
        println!("   🧠 Memory efficiency: {:.1}%", 
                 result.performance_metrics.memory_stats.efficiency_score * 100.0);
        
        Ok(())
    }

    /// Example 3: Memory-efficient export
    /// 
    /// For systems with limited memory or very large datasets.
    pub fn memory_efficient_export_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Configure for minimal memory usage
        let config = IntegratedConfig::memory_efficient();
        let mut exporter = IntegratedBinaryExporter::new(config);
        
        let result = exporter.export(&tracker, "memory_efficient_output.bin")?;
        
        println!("💾 Memory-efficient export completed:");
        println!("   📁 File size: {} bytes", result.export_result.bytes_written);
        println!("   🧠 Peak memory: {:.2} MB", 
                 result.performance_metrics.memory_stats.peak_usage as f64 / 1_000_000.0);
        println!("   📦 Compression ratio: {:.2}", 
                 result.export_result.compression_ratio.unwrap_or(1.0));
        
        Ok(())
    }

    /// Example 4: Custom configuration
    /// 
    /// How to create a custom configuration for specific needs.
    pub fn custom_configuration_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Create custom configuration
        let mut config = IntegratedConfig::balanced();
        
        // Customize compression
        config.compression.algorithm = CompressionAlgorithm::Zstd;
        config.compression.level = 9; // Higher compression
        
        // Customize output format
        config.output_format = OutputFormat::CustomBinary;
        
        // Customize processing
        config.processing.chunk_size = 128 * 1024; // 128KB chunks
        config.processing.validate_data = true;
        
        // Enable parallel processing
        if let Some(ref mut parallel_config) = config.parallel {
            parallel_config.worker_threads = num_cpus::get();
            parallel_config.enable_work_stealing = true;
        }
        
        let mut exporter = IntegratedBinaryExporter::new(config);
        let result = exporter.export(&tracker, "custom_output.bin")?;
        
        println!("⚙️  Custom export completed:");
        println!("   📁 File size: {} bytes", result.export_result.bytes_written);
        println!("   🔧 Configuration optimized for your needs");
        
        Ok(())
    }

    /// Example 5: Async export
    /// 
    /// How to perform non-blocking export operations.
    pub async fn async_export_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Async export with default settings
        let result = BinaryExport::export_async(&tracker, "async_output.bin").await?;
        
        println!("🔄 Async export completed:");
        println!("   📁 File size: {} bytes", result.bytes_written);
        println!("   ⏱️  Duration: {:?}", result.duration);
        
        Ok(())
    }

    /// Example 6: Loading and parsing binary files
    /// 
    /// How to read back exported binary data.
    pub fn loading_example() -> Result<(), BinaryExportError> {
        // Load with integrated exporter
        let exporter = IntegratedBinaryExporter::new(IntegratedConfig::default());
        let data = exporter.load("output.bin")?;
        
        println!("📖 Data loaded:");
        println!("   📊 Allocations: {}", data.allocations.allocations.len());
        println!("   🔗 Call stacks: {}", data.allocations.call_stacks.len());
        
        // Alternative: Use dedicated parser
        let parser = BinaryDataParser::new(ParserConfig::default());
        let parse_result = parser.parse_file("output.bin")?;
        
        println!("🔍 Parse results:");
        println!("   📁 File size: {} bytes", parse_result.parse_stats.file_size);
        println!("   ⚡ Throughput: {:.2} MB/s", 
                 parse_result.parse_stats.throughput / 1_000_000.0);
        println!("   ✅ Integrity score: {:.1}%", 
                 parse_result.validation_results.integrity_score * 100.0);
        
        Ok(())
    }

    /// Example 7: Format conversion
    /// 
    /// How to convert between different binary formats.
    pub fn format_conversion_example() -> Result<(), BinaryExportError> {
        let parser = BinaryDataParser::new(ParserConfig::default());
        
        // Convert MessagePack to Custom Binary
        let msgpack_data = std::fs::read("messagepack_file.bin")?;
        let converted_data = parser.convert_format(
            &msgpack_data,
            OutputFormat::MessagePack,
            OutputFormat::CustomBinary
        )?;
        
        std::fs::write("converted_file.bin", converted_data)?;
        
        println!("🔄 Format conversion completed:");
        println!("   📥 Input: MessagePack format");
        println!("   📤 Output: Custom Binary format");
        
        Ok(())
    }

    /// Example 8: Performance optimization
    /// 
    /// How to automatically optimize performance.
    pub fn performance_optimization_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Run automatic performance optimization
        let optimization_result = optimization::optimize_system_performance(&tracker)?;
        
        println!("⚡ Performance optimization completed:");
        println!("   🚀 Speed improvement: {:.2}x", 
                 optimization_result.improvement.speed_improvement);
        println!("   💾 Memory improvement: {:.2}x", 
                 optimization_result.improvement.memory_improvement);
        println!("   📈 Overall improvement: {:.2}x", 
                 optimization_result.improvement.overall_improvement);
        
        // Use the optimized configuration
        let mut exporter = IntegratedBinaryExporter::new(optimization_result.optimized_config);
        let result = exporter.export(&tracker, "optimized_output.bin")?;
        
        println!("   📁 Optimized export: {} bytes", result.export_result.bytes_written);
        
        Ok(())
    }

    /// Example 9: Benchmarking
    /// 
    /// How to run performance benchmarks.
    pub fn benchmarking_example() -> Result<(), BinaryExportError> {
        // Run quick benchmark
        let benchmark_results = benchmarks::run_quick_benchmark()?;
        
        println!("📊 Benchmark results:");
        println!("   ⚡ Average speed improvement: {:.2}x", 
                 benchmark_results.comparison.avg_speed_improvement);
        println!("   💾 Memory efficiency: {:.2}", 
                 benchmark_results.comparison.memory_efficiency);
        println!("   📦 Size efficiency: {:.2}", 
                 benchmark_results.comparison.size_efficiency);
        
        // Save detailed results
        let benchmark_runner = benchmarks::BenchmarkRunner::new(
            benchmarks::BenchmarkConfig::default()
        ).map_err(|e| BinaryExportError::IoError(e.kind()))?;
        
        benchmark_runner.save_results(&benchmark_results, Path::new("benchmark_results.json"))?;
        
        println!("   💾 Detailed results saved to benchmark_results.json");
        
        Ok(())
    }

    /// Example 10: Error handling
    /// 
    /// How to handle errors gracefully.
    pub fn error_handling_example() {
        let tracker = MemoryTracker::new();
        
        // Example of comprehensive error handling
        match BinaryExport::export_default(&tracker, "/invalid/path/output.bin") {
            Ok(result) => {
                println!("✅ Export successful: {} bytes", result.bytes_written);
            }
            Err(BinaryExportError::NoDataToExport) => {
                println!("ℹ️  No data available to export");
                // Handle empty data case
            }
            Err(BinaryExportError::IoError(kind)) => {
                println!("💾 I/O error: {:?}", kind);
                // Handle file system issues
            }
            Err(BinaryExportError::OutOfMemory { requested, available }) => {
                println!("🧠 Out of memory: requested {} bytes, available {} bytes", 
                         requested, available);
                // Try with memory-efficient configuration
                let config = IntegratedConfig::memory_efficient();
                let mut exporter = IntegratedBinaryExporter::new(config);
                match exporter.export(&tracker, "fallback_output.bin") {
                    Ok(_) => println!("✅ Fallback export successful"),
                    Err(e) => println!("❌ Fallback also failed: {:?}", e),
                }
            }
            Err(BinaryExportError::CompressionError(msg)) => {
                println!("📦 Compression error: {}", msg);
                // Try without compression
                let mut config = IntegratedConfig::default();
                config.compression.algorithm = CompressionAlgorithm::None;
                let mut exporter = IntegratedBinaryExporter::new(config);
                match exporter.export(&tracker, "uncompressed_output.bin") {
                    Ok(_) => println!("✅ Uncompressed export successful"),
                    Err(e) => println!("❌ Uncompressed export failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("❌ Unexpected error: {:?}", e);
                // Log error and potentially retry with different settings
            }
        }
    }

    /// Example 11: Monitoring and diagnostics
    /// 
    /// How to monitor system performance and health.
    pub fn monitoring_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        let mut exporter = IntegratedBinaryExporter::new(IntegratedConfig::balanced());
        
        // Check system status before export
        let status_before = exporter.get_system_status();
        println!("📊 System status before export:");
        println!("   🧠 Memory usage: {:.2} MB", 
                 status_before.memory_usage as f64 / 1_000_000.0);
        println!("   ⚙️  Auto-optimization: {}", 
                 if status_before.auto_optimization_enabled { "enabled" } else { "disabled" });
        
        // Perform export
        let result = exporter.export(&tracker, "monitored_output.bin")?;
        
        // Check system status after export
        let status_after = exporter.get_system_status();
        println!("📊 System status after export:");
        println!("   🧠 Peak memory: {:.2} MB", 
                 status_after.peak_memory_usage as f64 / 1_000_000.0);
        println!("   👥 Parallel workers: {}", status_after.parallel_workers_active);
        
        // Display performance metrics
        println!("⚡ Performance metrics:");
        println!("   ⏱️  Total time: {:?}", result.performance_metrics.total_time);
        println!("   📈 Overall efficiency: {:.1}%", 
                 result.performance_metrics.efficiency.overall_efficiency * 100.0);
        
        Ok(())
    }

    /// Example 12: Streaming large datasets
    /// 
    /// How to handle very large datasets with streaming.
    pub fn streaming_example() -> Result<(), BinaryExportError> {
        let tracker = MemoryTracker::new();
        
        // Configure for streaming processing
        let mut config = IntegratedConfig::memory_efficient();
        config.processing.chunk_size = 64 * 1024; // 64KB chunks
        config.processing.max_memory_usage = 128 * 1024 * 1024; // 128MB limit
        
        let mut exporter = IntegratedBinaryExporter::new(config);
        let result = exporter.export(&tracker, "streaming_output.bin")?;
        
        println!("🌊 Streaming export completed:");
        println!("   📁 File size: {} bytes", result.export_result.bytes_written);
        println!("   🧠 Peak memory: {:.2} MB", 
                 result.performance_metrics.memory_stats.peak_usage as f64 / 1_000_000.0);
        println!("   📊 Memory efficiency: {:.1}%", 
                 result.performance_metrics.memory_stats.efficiency_score * 100.0);
        
        Ok(())
    }

    /// Example 13: Legacy format conversion
    /// 
    /// How to convert existing JSON analysis files to binary format.
    pub fn legacy_conversion_example() -> Result<(), BinaryExportError> {
        use super::super::legacy_formats::{LegacyFormatAdapter, convert_legacy_directory};
        
        println!("🔄 Legacy Format Conversion Examples:");
        
        // Example 1: Convert single legacy file
        let legacy_file = "MemoryAnalysis/complex_lifecycle/complex_lifecycle_snapshot_memory_analysis.json";
        if std::path::Path::new(legacy_file).exists() {
            let adapter = LegacyFormatAdapter::new();
            
            match adapter.parse_legacy_file(legacy_file) {
                Ok(legacy_data) => {
                    println!("📖 Parsed legacy file: {}", legacy_file);
                    println!("   📊 Format: {:?}", legacy_data.format_type);
                    println!("   📈 Allocations: {}", legacy_data.allocations.len());
                    
                    // Convert to unified format
                    let unified_data = adapter.convert_to_unified(legacy_data)?;
                    println!("   ✅ Converted to unified format");
                    println!("   📊 Unified allocations: {}", unified_data.allocations.allocations.len());
                }
                Err(e) => {
                    println!("   ❌ Failed to parse: {:?}", e);
                }
            }
        }
        
        // Example 2: Convert entire directory
        let legacy_dir = "MemoryAnalysis/complex_lifecycle";
        if std::path::Path::new(legacy_dir).exists() {
            println!("\n📁 Converting directory: {}", legacy_dir);
            
            match convert_legacy_directory(legacy_dir, "converted_legacy_data.bin") {
                Ok(result) => {
                    println!("   ✅ Directory conversion completed:");
                    println!("   📁 Output size: {} bytes", result.export_result.bytes_written);
                    println!("   ⏱️  Duration: {:?}", result.export_result.duration);
                }
                Err(e) => {
                    println!("   ❌ Directory conversion failed: {:?}", e);
                }
            }
        }
        
        // Example 3: Using the convenience API
        println!("\n🚀 Using BinaryExport convenience API:");
        
        let test_files = [
            "MemoryAnalysis/complex_lifecycle/complex_lifecycle_snapshot_performance.json",
            "MemoryAnalysis/complex_lifecycle/complex_lifecycle_snapshot_security_violations.json",
            "MemoryAnalysis/complex_lifecycle/complex_lifecycle_snapshot_unsafe_ffi.json",
        ];
        
        for (i, file_path) in test_files.iter().enumerate() {
            if std::path::Path::new(file_path).exists() {
                let output_path = format!("converted_file_{}.bin", i);
                
                match BinaryExport::convert_legacy_format(file_path, &output_path) {
                    Ok(result) => {
                        println!("   ✅ Converted {}: {} bytes", 
                                file_path.split('/').last().unwrap_or(file_path),
                                result.export_result.bytes_written);
                    }
                    Err(e) => {
                        println!("   ❌ Failed to convert {}: {:?}", 
                                file_path.split('/').last().unwrap_or(file_path), e);
                    }
                }
            }
        }
        
        println!("\n💡 Legacy conversion tips:");
        println!("   • Use convert_legacy_directory() for batch conversion");
        println!("   • Legacy files are automatically detected by filename patterns");
        println!("   • All legacy formats are merged into a unified binary format");
        println!("   • Converted files maintain full compatibility with new binary export system");
        
        Ok(())
    }

    /// Example 14: High-performance data querying
    /// 
    /// How to use the query engine for fast data analysis.
    pub fn query_example() -> Result<(), BinaryExportError> {
        use super::super::query::{QueryEngine, QueryConfig, QueryOperator, StringOperator, SortField, SortDirection, RelatedDataType};
        
        println!("🔍 High-Performance Data Query Examples:");
        
        // First, create some test data and export it
        let tracker = MemoryTracker::new();
        let temp_file = "query_test_data.bin";
        
        // Export test data
        match BinaryExport::export_default(&tracker, temp_file) {
            Ok(_) => println!("   📁 Test data exported to {}", temp_file),
            Err(BinaryExportError::NoDataToExport) => {
                println!("   ℹ️  No data to export for query example");
                return Ok(());
            }
            Err(e) => return Err(e),
        }
        
        // Create query engine
        let config = QueryConfig::default();
        let mut engine = match QueryEngine::from_file(temp_file, config) {
            Ok(engine) => engine,
            Err(e) => {
                println!("   ❌ Failed to create query engine: {:?}", e);
                return Ok(());
            }
        };
        
        println!("   ✅ Query engine created with indices");
        
        // Example 1: Simple size-based query
        println!("\n🔍 Example 1: Find large allocations");
        let query = engine.query()
            .where_size(QueryOperator::GreaterThan(1024))
            .order_by(SortField::Size, SortDirection::Descending)
            .limit(10);
        
        match engine.execute_query(query) {
            Ok(result) => {
                println!("   📊 Found {} large allocations", result.allocations.len());
                println!("   ⏱️  Query time: {:?}", result.execution_stats.execution_time);
                
                for (i, allocation) in result.allocations.iter().take(3).enumerate() {
                    println!("   {}. Size: {} bytes, Type: {}", 
                             i + 1, allocation.size, allocation.allocation_type);
                }
            }
            Err(e) => println!("   ❌ Query failed: {:?}", e),
        }
        
        // Example 2: Type-based filtering
        println!("\n🔍 Example 2: Find allocations by type");
        let query = engine.query()
            .where_type(StringOperator::Contains("String".to_string()))
            .include(RelatedDataType::CallStacks)
            .limit(5);
        
        match engine.execute_query(query) {
            Ok(result) => {
                println!("   📊 Found {} string allocations", result.allocations.len());
                println!("   🔗 Call stacks included: {}", result.call_stacks.len());
                
                for allocation in result.allocations.iter().take(2) {
                    println!("   • {} bytes at 0x{:x}", allocation.size, allocation.address);
                    if let Some(call_stack_id) = allocation.call_stack_id {
                        if let Some(call_stack) = result.call_stacks.get(&call_stack_id) {
                            if let Some(frame) = call_stack.frames.first() {
                                println!("     Called from: {}", frame.function_name);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("   ❌ Query failed: {:?}", e),
        }
        
        // Example 3: Time-based query
        println!("\n🔍 Example 3: Recent allocations");
        let recent_time = std::time::SystemTime::now() - std::time::Duration::from_secs(3600); // Last hour
        let query = engine.query()
            .where_timestamp(QueryOperator::GreaterThan(recent_time))
            .order_by(SortField::Timestamp, SortDirection::Descending);
        
        match engine.execute_query(query) {
            Ok(result) => {
                println!("   📊 Found {} recent allocations", result.allocations.len());
                if !result.allocations.is_empty() {
                    println!("   🕒 Most recent: {} bytes", result.allocations[0].size);
                }
            }
            Err(e) => println!("   ❌ Query failed: {:?}", e),
        }
        
        // Example 4: Complex multi-condition query
        println!("\n🔍 Example 4: Complex query with multiple conditions");
        let query = engine.query()
            .where_size(QueryOperator::Range(100, 1000))
            .where_thread(QueryOperator::Equal(1))
            .where_custom(|alloc| alloc.address % 2 == 0) // Even addresses only
            .order_by(SortField::Address, SortDirection::Ascending)
            .limit(5);
        
        match engine.execute_query(query) {
            Ok(result) => {
                println!("   📊 Found {} matching allocations", result.allocations.len());
                println!("   🎯 Conditions: size 100-1000 bytes, thread 1, even addresses");
                
                for allocation in &result.allocations {
                    println!("   • {} bytes at 0x{:x} (thread {})", 
                             allocation.size, allocation.address, allocation.thread_id);
                }
            }
            Err(e) => println!("   ❌ Query failed: {:?}", e),
        }
        
        // Example 5: Aggregation query
        println!("\n📊 Example 5: Aggregation analysis");
        use super::super::query::{AggregationQuery, GroupByField, AggregationFunction};
        
        let agg_query = AggregationQuery {
            group_by: Some(GroupByField::TypeName),
            aggregations: vec![
                AggregationFunction::Count,
                AggregationFunction::SumSize,
                AggregationFunction::AvgSize,
                AggregationFunction::MaxSize,
            ],
            conditions: vec![],
        };
        
        match engine.execute_aggregation(agg_query) {
            Ok(result) => {
                println!("   📊 Aggregation completed in {:?}", result.execution_stats.execution_time);
                println!("   📈 Overall: {} allocations, {} total bytes", 
                         result.overall.count, result.overall.sum_size);
                
                println!("   📋 By type:");
                for (type_name, values) in result.groups.iter().take(3) {
                    println!("     • {}: {} allocations, avg {} bytes", 
                             type_name, values.count, values.avg_size as usize);
                }
            }
            Err(e) => println!("   ❌ Aggregation failed: {:?}", e),
        }
        
        // Show query statistics
        let stats = engine.get_stats();
        println!("\n📊 Query Engine Statistics:");
        println!("   🔍 Total queries: {}", stats.total_queries);
        println!("   ⏱️  Average query time: {:?}", stats.avg_query_time);
        println!("   🧠 Index memory usage: {:.2} MB", stats.index_memory_usage as f64 / 1_000_000.0);
        println!("   🏗️  Index build time: {:?}", stats.index_build_time);
        
        // Memory timeline analysis
        let timeline = engine.get_memory_timeline();
        if !timeline.is_empty() {
            println!("\n📈 Memory Timeline:");
            println!("   📊 {} time points tracked", timeline.len());
            if let Some((_, &peak_memory)) = timeline.iter().max_by_key(|(_, &memory)| memory) {
                println!("   🔝 Peak memory usage: {:.2} MB", peak_memory as f64 / 1_000_000.0);
            }
        }
        
        println!("\n💡 Query optimization tips:");
        println!("   • Use indices for fast lookups (ID, address, size, timestamp, type, thread)");
        println!("   • Combine multiple conditions to narrow results early");
        println!("   • Use aggregation queries for statistical analysis");
        println!("   • Enable caching for repeated queries");
        println!("   • Monitor query statistics to identify slow queries");
        
        // Clean up test file
        let _ = std::fs::remove_file(temp_file);
        
        Ok(())
    }
}

/// Usage patterns and best practices
pub mod best_practices {
    use super::super::*;

    /// Best practice: Choose the right configuration
    pub fn configuration_selection_guide() {
        println!("🎯 Configuration Selection Guide:");
        println!();
        println!("📊 **For small datasets (<10MB):**");
        println!("   - Use IntegratedConfig::balanced()");
        println!("   - Enable validation for data integrity");
        println!("   - Consider custom binary format for speed");
        println!();
        println!("🚀 **For maximum speed:**");
        println!("   - Use IntegratedConfig::high_performance()");
        println!("   - Disable compression or use LZ4");
        println!("   - Enable parallel processing");
        println!("   - Use custom binary format");
        println!();
        println!("💾 **For memory-constrained environments:**");
        println!("   - Use IntegratedConfig::memory_efficient()");
        println!("   - Enable streaming processing");
        println!("   - Use maximum compression");
        println!("   - Reduce chunk sizes");
        println!();
        println!("📦 **For smallest file sizes:**");
        println!("   - Use zstd compression level 19");
        println!("   - Enable all data validation");
        println!("   - Consider MessagePack format");
        println!();
        println!("🔄 **For compatibility:**");
        println!("   - Use MessagePack format");
        println!("   - Enable version metadata");
        println!("   - Include all data types");
    }

    /// Best practice: Error handling strategies
    pub fn error_handling_strategies() {
        println!("🛡️  Error Handling Best Practices:");
        println!();
        println!("1. **Always handle NoDataToExport**");
        println!("   - Check if tracker has data before export");
        println!("   - Provide meaningful user feedback");
        println!();
        println!("2. **Implement fallback strategies**");
        println!("   - Try memory-efficient config on OutOfMemory");
        println!("   - Disable compression on CompressionError");
        println!("   - Use alternative paths on IoError");
        println!();
        println!("3. **Use error recovery**");
        println!("   - Enable automatic retry with ErrorRecovery");
        println!("   - Implement exponential backoff for transient errors");
        println!("   - Log errors for debugging");
        println!();
        println!("4. **Validate inputs**");
        println!("   - Check file paths before export");
        println!("   - Validate configuration parameters");
        println!("   - Verify available disk space");
    }

    /// Best practice: Performance optimization tips
    pub fn performance_optimization_tips() {
        println!("⚡ Performance Optimization Tips:");
        println!();
        println!("1. **Use appropriate data structures**");
        println!("   - Enable zero-copy views for large data");
        println!("   - Use memory pools for frequent allocations");
        println!("   - Minimize data cloning");
        println!();
        println!("2. **Optimize compression**");
        println!("   - Use LZ4 for speed, zstd for balance");
        println!("   - Enable auto-selection for mixed workloads");
        println!("   - Consider compression level vs speed trade-offs");
        println!();
        println!("3. **Leverage parallelism**");
        println!("   - Enable work-stealing for irregular workloads");
        println!("   - Use appropriate worker thread counts");
        println!("   - Monitor CPU utilization");
        println!();
        println!("4. **Monitor and profile**");
        println!("   - Use built-in performance monitoring");
        println!("   - Run benchmarks regularly");
        println!("   - Profile memory usage patterns");
    }
}

#[cfg(test)]
mod example_tests {
    use super::examples::*;

    #[test]
    fn test_basic_export_example() {
        // Test that the basic example doesn't panic
        match basic_export_example() {
            Ok(_) => println!("Basic export example succeeded"),
            Err(BinaryExportError::NoDataToExport) => println!("No data to export (expected)"),
            Err(e) => panic!("Basic export example failed: {:?}", e),
        }
    }

    #[test]
    fn test_error_handling_example() {
        // Test that error handling example doesn't panic
        error_handling_example();
        println!("Error handling example completed");
    }

    #[test]
    fn test_best_practices() {
        // Test that best practice functions don't panic
        best_practices::configuration_selection_guide();
        best_practices::error_handling_strategies();
        best_practices::performance_optimization_tips();
        println!("Best practices examples completed");
    }

    #[test]
    fn test_legacy_conversion_example() {
        // Test that the legacy conversion example doesn't panic
        match legacy_conversion_example() {
            Ok(_) => println!("Legacy conversion example succeeded"),
            Err(e) => println!("Legacy conversion example failed (may be expected): {:?}", e),
        }
    }

    #[test]
    fn test_query_example() {
        // Test that the query example doesn't panic
        match query_example() {
            Ok(_) => println!("Query example succeeded"),
            Err(e) => println!("Query example failed (may be expected): {:?}", e),
        }
    }
}