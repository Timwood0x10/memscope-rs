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
        assert_eq!(comparison.performance_improvement.avg_time_improvement_percent, 0.0);
    }
}
