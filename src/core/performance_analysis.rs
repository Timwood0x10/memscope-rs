//! Performance analysis and optimization recommendations
//! 
//! This module analyzes actual performance bottlenecks and provides
//! targeted optimization recommendations.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Performance bottleneck analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub location: String,
    pub operation: String,
    pub frequency: u64,
    pub total_time_ms: f64,
    pub avg_time_ns: f64,
    pub severity: BottleneckSeverity,
    pub recommendation: OptimizationRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationRecommendation {
    /// Keep using std::sync::Mutex - it's fine for this use case
    KeepStdMutex,
    /// Use parking_lot::Mutex for better performance
    UseParkingLot,
    /// Use atomic operations for simple counters
    UseAtomics,
    /// Use lock-free data structures
    UseLockFree,
    /// Reduce lock scope
    ReduceLockScope,
    /// Batch operations to reduce lock frequency
    BatchOperations,
    /// No optimization needed
    NoOptimizationNeeded,
}

/// Actual performance profiler that measures real bottlenecks
pub struct PerformanceProfiler {
    measurements: HashMap<String, Vec<Duration>>,
    start_times: HashMap<String, Instant>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            start_times: HashMap::new(),
        }
    }

    /// Start measuring an operation
    pub fn start_measurement(&mut self, operation: &str) {
        self.start_times.insert(operation.to_string(), Instant::now());
    }

    /// End measuring an operation
    pub fn end_measurement(&mut self, operation: &str) {
        if let Some(start_time) = self.start_times.remove(operation) {
            let duration = start_time.elapsed();
            self.measurements
                .entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    /// Analyze bottlenecks and provide recommendations
    pub fn analyze_bottlenecks(&self) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        for (operation, durations) in &self.measurements {
            if durations.is_empty() {
                continue;
            }

            let total_time: Duration = durations.iter().sum();
            let avg_time = total_time / durations.len() as u32;
            let frequency = durations.len() as u64;

            let severity = if avg_time.as_millis() > 10 {
                BottleneckSeverity::Critical
            } else if avg_time.as_millis() > 1 {
                BottleneckSeverity::High
            } else if avg_time.as_micros() > 100 {
                BottleneckSeverity::Medium
            } else {
                BottleneckSeverity::Low
            };

            let recommendation = self.get_recommendation(operation, &avg_time, frequency);

            bottlenecks.push(PerformanceBottleneck {
                location: operation.clone(),
                operation: operation.clone(),
                frequency,
                total_time_ms: total_time.as_secs_f64() * 1000.0,
                avg_time_ns: avg_time.as_nanos() as f64,
                severity,
                recommendation,
            });
        }

        // Sort by total impact (frequency * avg_time)
        bottlenecks.sort_by(|a, b| {
            let impact_a = a.frequency as f64 * a.avg_time_ns;
            let impact_b = b.frequency as f64 * b.avg_time_ns;
            impact_b.partial_cmp(&impact_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        bottlenecks
    }

    fn get_recommendation(&self, operation: &str, avg_time: &Duration, frequency: u64) -> OptimizationRecommendation {
        // Simple heuristics for optimization recommendations
        if operation.contains("stats") && frequency > 1000 {
            OptimizationRecommendation::UseAtomics
        } else if operation.contains("lock") && avg_time.as_micros() > 50 {
            if frequency > 10000 {
                OptimizationRecommendation::UseLockFree
            } else {
                OptimizationRecommendation::UseParkingLot
            }
        } else if operation.contains("clone") && frequency > 100 {
            OptimizationRecommendation::BatchOperations
        } else if avg_time.as_micros() < 10 {
            OptimizationRecommendation::NoOptimizationNeeded
        } else {
            OptimizationRecommendation::KeepStdMutex
        }
    }

    /// Generate optimization report
    pub fn generate_report(&self) -> OptimizationReport {
        let bottlenecks = self.analyze_bottlenecks();
        let total_operations: u64 = bottlenecks.iter().map(|b| b.frequency).sum();
        let total_time_ms: f64 = bottlenecks.iter().map(|b| b.total_time_ms).sum();

        OptimizationReport {
            total_operations,
            total_time_ms,
            bottlenecks,
            summary: self.generate_summary(),
        }
    }

    fn generate_summary(&self) -> String {
        let bottlenecks = self.analyze_bottlenecks();
        if bottlenecks.is_empty() {
            return "No performance bottlenecks detected.".to_string();
        }

        let critical_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Critical)).count();
        let high_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::High)).count();

        if critical_count > 0 {
            format!("Found {} critical and {} high-severity bottlenecks. Immediate optimization recommended.", critical_count, high_count)
        } else if high_count > 0 {
            format!("Found {} high-severity bottlenecks. Optimization recommended.", high_count)
        } else {
            "Performance is generally good. Minor optimizations possible.".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub total_operations: u64,
    pub total_time_ms: f64,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub summary: String,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new();
        
        // Simulate some operations
        profiler.start_measurement("lock_operation");
        thread::sleep(Duration::from_micros(100));
        profiler.end_measurement("lock_operation");
        
        profiler.start_measurement("stats_update");
        thread::sleep(Duration::from_micros(10));
        profiler.end_measurement("stats_update");
        
        let report = profiler.generate_report();
        assert!(!report.bottlenecks.is_empty());
        assert!(report.total_time_ms > 0.0);
    }
}