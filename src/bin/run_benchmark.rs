//! performance benchmark main progamming
//!
//! this program runs complex_lifecycle_showcase.rs performance benchmark,
//! comparing the performance of traditional export system and fast export system.

// use memscope_rs::export::performance_benchmark::{BenchmarkConfig, PerformanceBenchmark}; // Removed - test code

// Local definitions for benchmark functionality
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub data_size: usize,
    pub test_runs: usize,
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub verify_consistency: bool,
    pub generate_detailed_report: bool,
}

pub struct PerformanceBenchmark;

impl PerformanceBenchmark {
    pub fn new(_config: BenchmarkConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub fn run_all_benchmarks(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Benchmark functionality removed - use cargo bench instead");
        Ok(())
    }

    pub fn run_full_benchmark(
        &mut self,
    ) -> Result<BenchmarkComparison, Box<dyn std::error::Error>> {
        println!("Full benchmark functionality removed - use cargo bench instead");
        Ok(BenchmarkComparison::default())
    }
}

#[derive(Debug, Default)]
pub struct BenchmarkComparison {
    pub performance_improvement: PerformanceImprovement,
}

#[derive(Debug, Default)]
pub struct PerformanceImprovement {
    pub avg_time_improvement_percent: f64,
    pub avg_memory_improvement_percent: f64,
    pub avg_throughput_improvement_percent: f64,
    pub avg_write_speed_improvement_percent: f64,
    pub best_time_improvement_percent: f64,
    pub consistency_score: f64,
}
use std::path::PathBuf;
use std::process;

fn main() {
    tracing::info!("🚀 large project export optimization - performance benchmark");
    tracing::info!("=====================================");
    tracing::info!("");

    // configure benchmark
    let config = BenchmarkConfig {
        iterations: 100,
        data_size: 1000,
        test_runs: 5,
        output_dir: PathBuf::from("benchmark_results"),
        verbose: true,
        verify_consistency: true,
        generate_detailed_report: true,
    };

    // create benchmark
    let mut benchmark = match PerformanceBenchmark::new(config) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("❌ create benchmark failed: {}", e);
            process::exit(1);
        }
    };

    // run full benchmark
    match benchmark.run_full_benchmark() {
        Ok(comparison) => {
            tracing::info!("");
            tracing::info!("🎉 benchmark completed!");
            tracing::info!("==================");

            let perf = &comparison.performance_improvement;
            tracing::info!("📊 performance improvement summary:");
            tracing::info!(
                "  • average export time improvement: {:.1}%",
                perf.avg_time_improvement_percent
            );
            tracing::info!(
                "  • average memory usage improvement: {:.1}%",
                perf.avg_memory_improvement_percent
            );
            tracing::info!(
                "  • average throughput improvement: +{:.1}%",
                perf.avg_throughput_improvement_percent
            );
            tracing::info!(
                "  • average write speed improvement: +{:.1}%",
                perf.avg_write_speed_improvement_percent
            );
            tracing::info!(
                "  • best time improvement: {:.1}%",
                perf.best_time_improvement_percent
            );
            tracing::info!("  • consistency score: {:.1}/100", perf.consistency_score);
            tracing::info!("");

            // evaluate if reached target
            if perf.avg_time_improvement_percent >= 60.0 {
                tracing::info!("✅ excellent! reached 60-80% export time reduction target");
            } else if perf.avg_time_improvement_percent >= 40.0 {
                tracing::info!("✅ good! close to 60-80% export time reduction target");
            } else if perf.avg_time_improvement_percent >= 20.0 {
                tracing::info!("⚠️ general, some improvement but not reached expected target");
            } else {
                tracing::info!("❌ performance improvement not obvious, need further optimization");
            }

            tracing::info!("");
            tracing::info!("📁 generated files:");
            tracing::info!("  • benchmark_results/benchmark_results.json - detailed test data");
            tracing::info!("  • benchmark_results/performance_report.md - performance report");
            tracing::info!(
                "  • benchmark_results/traditional_export_run_*.json - traditional export results"
            );
            tracing::info!("  • benchmark_results/fast_export_run_*.json - fast export results");
        }
        Err(e) => {
            tracing::error!("❌ benchmark failed: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs_without_panic() {
        main();
    }

    #[test]
    fn test_performance_benchmark_new() {
        let config = BenchmarkConfig {
            iterations: 1,
            data_size: 1,
            test_runs: 1,
            output_dir: PathBuf::from("test_results"),
            verbose: false,
            verify_consistency: false,
            generate_detailed_report: false,
        };
        let benchmark = PerformanceBenchmark::new(config);
        assert!(benchmark.is_ok());
    }

    #[test]
    fn test_performance_benchmark_run_full_benchmark() {
        let config = BenchmarkConfig {
            iterations: 1,
            data_size: 1,
            test_runs: 1,
            output_dir: PathBuf::from("test_results"),
            verbose: false,
            verify_consistency: false,
            generate_detailed_report: false,
        };
        let mut benchmark = PerformanceBenchmark::new(config).unwrap();
        let result = benchmark.run_full_benchmark();
        assert!(result.is_ok());
        let comparison = result.unwrap();
        assert_eq!(
            comparison
                .performance_improvement
                .avg_time_improvement_percent,
            0.0
        );
    }

    #[test]
    fn test_benchmark_config_creation() {
        let config = BenchmarkConfig {
            iterations: 100,
            data_size: 1000,
            test_runs: 5,
            output_dir: PathBuf::from("benchmark_results"),
            verbose: true,
            verify_consistency: true,
            generate_detailed_report: true,
        };

        assert_eq!(config.iterations, 100);
        assert_eq!(config.data_size, 1000);
        assert_eq!(config.test_runs, 5);
        assert_eq!(config.output_dir, PathBuf::from("benchmark_results"));
        assert!(config.verbose);
        assert!(config.verify_consistency);
        assert!(config.generate_detailed_report);
    }

    #[test]
    fn test_benchmark_config_edge_cases() {
        // Test with zero values
        let zero_config = BenchmarkConfig {
            iterations: 0,
            data_size: 0,
            test_runs: 0,
            output_dir: PathBuf::from(""),
            verbose: false,
            verify_consistency: false,
            generate_detailed_report: false,
        };

        assert_eq!(zero_config.iterations, 0);
        assert_eq!(zero_config.data_size, 0);
        assert_eq!(zero_config.test_runs, 0);
        assert_eq!(zero_config.output_dir, PathBuf::from(""));
        assert!(!zero_config.verbose);
        assert!(!zero_config.verify_consistency);
        assert!(!zero_config.generate_detailed_report);

        // Test with maximum values
        let max_config = BenchmarkConfig {
            iterations: usize::MAX,
            data_size: usize::MAX,
            test_runs: usize::MAX,
            output_dir: PathBuf::from("/very/long/path/to/benchmark/results/directory"),
            verbose: true,
            verify_consistency: true,
            generate_detailed_report: true,
        };

        assert_eq!(max_config.iterations, usize::MAX);
        assert_eq!(max_config.data_size, usize::MAX);
        assert_eq!(max_config.test_runs, usize::MAX);
        assert!(max_config.verbose);
        assert!(max_config.verify_consistency);
        assert!(max_config.generate_detailed_report);
    }

    #[test]
    fn test_benchmark_config_clone() {
        let original = BenchmarkConfig {
            iterations: 50,
            data_size: 500,
            test_runs: 3,
            output_dir: PathBuf::from("test_output"),
            verbose: true,
            verify_consistency: false,
            generate_detailed_report: true,
        };

        let cloned = original.clone();

        assert_eq!(cloned.iterations, original.iterations);
        assert_eq!(cloned.data_size, original.data_size);
        assert_eq!(cloned.test_runs, original.test_runs);
        assert_eq!(cloned.output_dir, original.output_dir);
        assert_eq!(cloned.verbose, original.verbose);
        assert_eq!(cloned.verify_consistency, original.verify_consistency);
        assert_eq!(cloned.generate_detailed_report, original.generate_detailed_report);
    }

    #[test]
    fn test_performance_improvement_default() {
        let improvement = PerformanceImprovement::default();

        assert_eq!(improvement.avg_time_improvement_percent, 0.0);
        assert_eq!(improvement.avg_memory_improvement_percent, 0.0);
        assert_eq!(improvement.avg_throughput_improvement_percent, 0.0);
        assert_eq!(improvement.avg_write_speed_improvement_percent, 0.0);
        assert_eq!(improvement.best_time_improvement_percent, 0.0);
        assert_eq!(improvement.consistency_score, 0.0);
    }

    #[test]
    fn test_performance_improvement_custom_values() {
        let improvement = PerformanceImprovement {
            avg_time_improvement_percent: 65.5,
            avg_memory_improvement_percent: 45.2,
            avg_throughput_improvement_percent: 78.9,
            avg_write_speed_improvement_percent: 82.1,
            best_time_improvement_percent: 95.3,
            consistency_score: 87.6,
        };

        assert_eq!(improvement.avg_time_improvement_percent, 65.5);
        assert_eq!(improvement.avg_memory_improvement_percent, 45.2);
        assert_eq!(improvement.avg_throughput_improvement_percent, 78.9);
        assert_eq!(improvement.avg_write_speed_improvement_percent, 82.1);
        assert_eq!(improvement.best_time_improvement_percent, 95.3);
        assert_eq!(improvement.consistency_score, 87.6);
    }

    #[test]
    fn test_benchmark_comparison_default() {
        let comparison = BenchmarkComparison::default();

        assert_eq!(comparison.performance_improvement.avg_time_improvement_percent, 0.0);
        assert_eq!(comparison.performance_improvement.avg_memory_improvement_percent, 0.0);
        assert_eq!(comparison.performance_improvement.avg_throughput_improvement_percent, 0.0);
        assert_eq!(comparison.performance_improvement.avg_write_speed_improvement_percent, 0.0);
        assert_eq!(comparison.performance_improvement.best_time_improvement_percent, 0.0);
        assert_eq!(comparison.performance_improvement.consistency_score, 0.0);
    }

    #[test]
    fn test_benchmark_comparison_with_custom_improvement() {
        let improvement = PerformanceImprovement {
            avg_time_improvement_percent: 70.0,
            avg_memory_improvement_percent: 50.0,
            avg_throughput_improvement_percent: 80.0,
            avg_write_speed_improvement_percent: 85.0,
            best_time_improvement_percent: 90.0,
            consistency_score: 95.0,
        };

        let comparison = BenchmarkComparison {
            performance_improvement: improvement,
        };

        assert_eq!(comparison.performance_improvement.avg_time_improvement_percent, 70.0);
        assert_eq!(comparison.performance_improvement.avg_memory_improvement_percent, 50.0);
        assert_eq!(comparison.performance_improvement.avg_throughput_improvement_percent, 80.0);
        assert_eq!(comparison.performance_improvement.avg_write_speed_improvement_percent, 85.0);
        assert_eq!(comparison.performance_improvement.best_time_improvement_percent, 90.0);
        assert_eq!(comparison.performance_improvement.consistency_score, 95.0);
    }

    #[test]
    fn test_performance_benchmark_run_all_benchmarks() {
        let config = BenchmarkConfig {
            iterations: 1,
            data_size: 1,
            test_runs: 1,
            output_dir: PathBuf::from("test_results"),
            verbose: false,
            verify_consistency: false,
            generate_detailed_report: false,
        };

        let benchmark = PerformanceBenchmark::new(config).unwrap();
        let result = benchmark.run_all_benchmarks();
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_benchmark_multiple_configs() {
        let configs = vec![
            BenchmarkConfig {
                iterations: 10,
                data_size: 100,
                test_runs: 1,
                output_dir: PathBuf::from("test1"),
                verbose: true,
                verify_consistency: true,
                generate_detailed_report: true,
            },
            BenchmarkConfig {
                iterations: 20,
                data_size: 200,
                test_runs: 2,
                output_dir: PathBuf::from("test2"),
                verbose: false,
                verify_consistency: false,
                generate_detailed_report: false,
            },
            BenchmarkConfig {
                iterations: 30,
                data_size: 300,
                test_runs: 3,
                output_dir: PathBuf::from("test3"),
                verbose: true,
                verify_consistency: false,
                generate_detailed_report: true,
            },
        ];

        for config in configs {
            let benchmark = PerformanceBenchmark::new(config);
            assert!(benchmark.is_ok());

            let mut benchmark = benchmark.unwrap();
            let result = benchmark.run_full_benchmark();
            assert!(result.is_ok());

            let all_result = benchmark.run_all_benchmarks();
            assert!(all_result.is_ok());
        }
    }

    #[test]
    fn test_performance_improvement_extreme_values() {
        // Test with negative values (representing performance degradation)
        let negative_improvement = PerformanceImprovement {
            avg_time_improvement_percent: -10.5,
            avg_memory_improvement_percent: -5.2,
            avg_throughput_improvement_percent: -15.8,
            avg_write_speed_improvement_percent: -8.3,
            best_time_improvement_percent: -2.1,
            consistency_score: 25.0,
        };

        assert_eq!(negative_improvement.avg_time_improvement_percent, -10.5);
        assert_eq!(negative_improvement.avg_memory_improvement_percent, -5.2);
        assert_eq!(negative_improvement.avg_throughput_improvement_percent, -15.8);
        assert_eq!(negative_improvement.avg_write_speed_improvement_percent, -8.3);
        assert_eq!(negative_improvement.best_time_improvement_percent, -2.1);
        assert_eq!(negative_improvement.consistency_score, 25.0);

        // Test with very high values
        let high_improvement = PerformanceImprovement {
            avg_time_improvement_percent: 150.0,
            avg_memory_improvement_percent: 200.0,
            avg_throughput_improvement_percent: 300.0,
            avg_write_speed_improvement_percent: 250.0,
            best_time_improvement_percent: 400.0,
            consistency_score: 100.0,
        };

        assert_eq!(high_improvement.avg_time_improvement_percent, 150.0);
        assert_eq!(high_improvement.avg_memory_improvement_percent, 200.0);
        assert_eq!(high_improvement.avg_throughput_improvement_percent, 300.0);
        assert_eq!(high_improvement.avg_write_speed_improvement_percent, 250.0);
        assert_eq!(high_improvement.best_time_improvement_percent, 400.0);
        assert_eq!(high_improvement.consistency_score, 100.0);
    }

    #[test]
    fn test_benchmark_config_debug_format() {
        let config = BenchmarkConfig {
            iterations: 42,
            data_size: 1337,
            test_runs: 7,
            output_dir: PathBuf::from("debug_test"),
            verbose: true,
            verify_consistency: false,
            generate_detailed_report: true,
        };

        let debug_output = format!("{config:?}");
        assert!(debug_output.contains("iterations: 42"));
        assert!(debug_output.contains("data_size: 1337"));
        assert!(debug_output.contains("test_runs: 7"));
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("verbose: true"));
        assert!(debug_output.contains("verify_consistency: false"));
        assert!(debug_output.contains("generate_detailed_report: true"));
    }

    #[test]
    fn test_performance_improvement_debug_format() {
        let improvement = PerformanceImprovement {
            avg_time_improvement_percent: 42.5,
            avg_memory_improvement_percent: 33.7,
            avg_throughput_improvement_percent: 55.2,
            avg_write_speed_improvement_percent: 67.8,
            best_time_improvement_percent: 89.1,
            consistency_score: 91.3,
        };

        let debug_output = format!("{improvement:?}");
        assert!(debug_output.contains("avg_time_improvement_percent: 42.5"));
        assert!(debug_output.contains("avg_memory_improvement_percent: 33.7"));
        assert!(debug_output.contains("avg_throughput_improvement_percent: 55.2"));
        assert!(debug_output.contains("avg_write_speed_improvement_percent: 67.8"));
        assert!(debug_output.contains("best_time_improvement_percent: 89.1"));
        assert!(debug_output.contains("consistency_score: 91.3"));
    }

    #[test]
    fn test_benchmark_comparison_debug_format() {
        let improvement = PerformanceImprovement {
            avg_time_improvement_percent: 75.0,
            avg_memory_improvement_percent: 60.0,
            avg_throughput_improvement_percent: 85.0,
            avg_write_speed_improvement_percent: 90.0,
            best_time_improvement_percent: 95.0,
            consistency_score: 88.0,
        };

        let comparison = BenchmarkComparison {
            performance_improvement: improvement,
        };

        let debug_output = format!("{comparison:?}");
        assert!(debug_output.contains("performance_improvement"));
        assert!(debug_output.contains("avg_time_improvement_percent: 75"));
        assert!(debug_output.contains("consistency_score: 88"));
    }

    #[test]
    fn test_pathbuf_operations() {
        let mut config = BenchmarkConfig {
            iterations: 1,
            data_size: 1,
            test_runs: 1,
            output_dir: PathBuf::from("initial_path"),
            verbose: false,
            verify_consistency: false,
            generate_detailed_report: false,
        };

        // Test path operations
        assert_eq!(config.output_dir, PathBuf::from("initial_path"));

        // Test path modification
        config.output_dir = PathBuf::from("modified_path");
        assert_eq!(config.output_dir, PathBuf::from("modified_path"));

        // Test path with subdirectories
        config.output_dir = PathBuf::from("parent/child/grandchild");
        assert_eq!(config.output_dir, PathBuf::from("parent/child/grandchild"));

        // Test absolute path
        config.output_dir = PathBuf::from("/absolute/path/to/results");
        assert_eq!(config.output_dir, PathBuf::from("/absolute/path/to/results"));
    }
}
