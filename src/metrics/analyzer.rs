use super::{MetricValue, MetricsCollector};
use std::collections::HashMap;
use std::time::Duration;

/// Memory analysis performance analyzer
/// Focused on offline memory profiling and analysis efficiency
pub struct PerformanceAnalyzer {
    /// Baseline benchmarks for comparison
    baselines: HashMap<String, Benchmark>,
    /// Performance thresholds for memory operations
    thresholds: AnalysisThresholds,
}

/// Performance benchmark for memory analysis operations
#[derive(Debug, Clone)]
pub struct Benchmark {
    /// Operation name (e.g., "allocation_tracking", "symbol_resolution")
    pub operation: String,
    /// Average execution time
    pub avg_duration: Duration,
    /// Memory overhead in bytes
    pub memory_overhead: usize,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Accuracy percentage (0.0 to 1.0)
    pub accuracy: f64,
    /// Sample size used for benchmark
    pub sample_size: usize,
}

/// Performance thresholds for memory analysis
#[derive(Debug, Clone)]
pub struct AnalysisThresholds {
    /// Max acceptable tracking overhead (percentage of app memory)
    pub max_tracking_overhead: f64,
    /// Max allocation tracking latency (microseconds)
    pub max_allocation_latency: Duration,
    /// Max symbol resolution time per frame (milliseconds)
    pub max_symbol_resolution_time: Duration,
    /// Min acceptable tracking completeness (0.0 to 1.0)
    pub min_tracking_completeness: f64,
    /// Max memory usage for analysis tools (MB)
    pub max_analysis_memory: usize,
}

/// Comprehensive performance report for memory analysis
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Overall analysis efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Memory tracking performance
    pub tracking_performance: TrackingPerformance,
    /// Symbol resolution performance  
    pub symbol_performance: SymbolPerformance,
    /// Smart pointer analysis performance
    pub pointer_performance: PointerPerformance,
    /// Memory usage efficiency
    pub memory_efficiency: MemoryEfficiency,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Memory tracking performance metrics
#[derive(Debug, Clone, Default)]
pub struct TrackingPerformance {
    /// Average allocation tracking time
    pub avg_allocation_time: Duration,
    /// Tracking completeness percentage
    pub completeness: f64,
    /// Memory overhead of tracking
    pub overhead_bytes: usize,
    /// Allocations tracked per second
    pub throughput: f64,
}

/// Symbol resolution performance metrics
#[derive(Debug, Clone, Default)]
pub struct SymbolPerformance {
    /// Average symbol resolution time
    pub avg_resolution_time: Duration,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Symbols resolved per second
    pub resolution_rate: f64,
    /// Memory used by symbol cache
    pub cache_memory_usage: usize,
}

/// Smart pointer analysis performance
#[derive(Debug, Clone, Default)]
pub struct PointerPerformance {
    /// Time to analyze pointer patterns
    pub analysis_time: Duration,
    /// Leak detection accuracy
    pub leak_detection_accuracy: f64,
    /// Pointers analyzed per second
    pub analysis_rate: f64,
}

/// Memory usage efficiency of analysis tools
#[derive(Debug, Clone, Default)]
pub struct MemoryEfficiency {
    /// Total memory used by analysis tools
    pub total_memory_mb: f64,
    /// Memory usage per tracked allocation
    pub memory_per_allocation: f64,
    /// Memory growth rate (MB per hour)
    pub growth_rate: f64,
    /// Memory fragmentation level
    pub fragmentation_ratio: f64,
}

impl PerformanceAnalyzer {
    /// Create analyzer with default thresholds
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            thresholds: AnalysisThresholds::default(),
        }
    }

    /// Create analyzer with custom thresholds
    pub fn with_thresholds(thresholds: AnalysisThresholds) -> Self {
        Self {
            baselines: HashMap::new(),
            thresholds,
        }
    }

    /// Analyze current performance metrics
    pub fn analyze_performance(&self, collector: &MetricsCollector) -> PerformanceReport {
        let tracking_perf = self.analyze_tracking_performance(collector);
        let symbol_perf = self.analyze_symbol_performance(collector);
        let pointer_perf = self.analyze_pointer_performance(collector);
        let memory_eff = self.analyze_memory_efficiency(collector);

        let efficiency_score = self.calculate_efficiency_score(
            &tracking_perf,
            &symbol_perf,
            &pointer_perf,
            &memory_eff,
        );

        let recommendations =
            self.generate_recommendations(&tracking_perf, &symbol_perf, &pointer_perf, &memory_eff);

        PerformanceReport {
            efficiency_score,
            tracking_performance: tracking_perf,
            symbol_performance: symbol_perf,
            pointer_performance: pointer_perf,
            memory_efficiency: memory_eff,
            recommendations,
        }
    }

    /// Set baseline benchmark for operation
    pub fn set_baseline(&mut self, operation: &str, benchmark: Benchmark) {
        self.baselines.insert(operation.to_string(), benchmark);
    }

    /// Compare current performance against baseline
    pub fn compare_to_baseline(
        &self,
        operation: &str,
        current: &Benchmark,
    ) -> Option<PerformanceComparison> {
        self.baselines
            .get(operation)
            .map(|baseline| PerformanceComparison {
                operation: operation.to_string(),
                baseline: baseline.clone(),
                current: current.clone(),
                duration_ratio: current.avg_duration.as_nanos() as f64
                    / baseline.avg_duration.as_nanos() as f64,
                memory_ratio: current.memory_overhead as f64 / baseline.memory_overhead as f64,
                throughput_ratio: current.throughput / baseline.throughput,
                accuracy_diff: current.accuracy - baseline.accuracy,
            })
    }

    fn analyze_tracking_performance(&self, collector: &MetricsCollector) -> TrackingPerformance {
        let avg_allocation_time = collector
            .get_metric("allocation_tracking_time")
            .and_then(|m| match &m.value {
                MetricValue::Timer(timer) => Some(timer.average_duration()),
                _ => None,
            })
            .unwrap_or(Duration::from_nanos(0));

        let completeness = collector
            .get_metric("tracking_completeness")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let overhead_bytes = collector
            .get_metric("tracking_memory_overhead")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value as usize),
                _ => None,
            })
            .unwrap_or(0);

        let throughput = collector
            .get_metric("allocations_per_second")
            .and_then(|m| match &m.value {
                MetricValue::Rate(rate) => Some(rate.current_rate),
                _ => None,
            })
            .unwrap_or(0.0);

        TrackingPerformance {
            avg_allocation_time,
            completeness,
            overhead_bytes,
            throughput,
        }
    }

    fn analyze_symbol_performance(&self, collector: &MetricsCollector) -> SymbolPerformance {
        let avg_resolution_time = collector
            .get_metric("symbol_resolution_time")
            .and_then(|m| match &m.value {
                MetricValue::Timer(timer) => Some(timer.average_duration()),
                _ => None,
            })
            .unwrap_or(Duration::from_nanos(0));

        let cache_hit_ratio = collector
            .get_metric("symbol_cache_hit_ratio")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let resolution_rate = collector
            .get_metric("symbols_resolved_per_second")
            .and_then(|m| match &m.value {
                MetricValue::Rate(rate) => Some(rate.current_rate),
                _ => None,
            })
            .unwrap_or(0.0);

        let cache_memory_usage = collector
            .get_metric("symbol_cache_memory")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value as usize),
                _ => None,
            })
            .unwrap_or(0);

        SymbolPerformance {
            avg_resolution_time,
            cache_hit_ratio,
            resolution_rate,
            cache_memory_usage,
        }
    }

    fn analyze_pointer_performance(&self, collector: &MetricsCollector) -> PointerPerformance {
        let analysis_time = collector
            .get_metric("pointer_analysis_time")
            .and_then(|m| match &m.value {
                MetricValue::Timer(timer) => Some(timer.average_duration()),
                _ => None,
            })
            .unwrap_or(Duration::from_nanos(0));

        let leak_detection_accuracy = collector
            .get_metric("leak_detection_accuracy")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let analysis_rate = collector
            .get_metric("pointers_analyzed_per_second")
            .and_then(|m| match &m.value {
                MetricValue::Rate(rate) => Some(rate.current_rate),
                _ => None,
            })
            .unwrap_or(0.0);

        PointerPerformance {
            analysis_time,
            leak_detection_accuracy,
            analysis_rate,
        }
    }

    fn analyze_memory_efficiency(&self, collector: &MetricsCollector) -> MemoryEfficiency {
        let total_memory_mb = collector
            .get_metric("total_analysis_memory")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let memory_per_allocation = collector
            .get_metric("memory_per_tracked_allocation")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let growth_rate = collector
            .get_metric("memory_growth_rate")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        let fragmentation_ratio = collector
            .get_metric("memory_fragmentation")
            .and_then(|m| match &m.value {
                MetricValue::Gauge(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(0.0);

        MemoryEfficiency {
            total_memory_mb,
            memory_per_allocation,
            growth_rate,
            fragmentation_ratio,
        }
    }

    fn calculate_efficiency_score(
        &self,
        tracking: &TrackingPerformance,
        symbol: &SymbolPerformance,
        pointer: &PointerPerformance,
        memory: &MemoryEfficiency,
    ) -> f64 {
        let tracking_score = self.score_tracking_performance(tracking);
        let symbol_score = self.score_symbol_performance(symbol);
        let pointer_score = self.score_pointer_performance(pointer);
        let memory_score = self.score_memory_efficiency(memory);

        // Weighted average (tracking is most important for memory analysis)
        tracking_score * 0.4 + symbol_score * 0.25 + pointer_score * 0.2 + memory_score * 0.15
    }

    fn score_tracking_performance(&self, tracking: &TrackingPerformance) -> f64 {
        let mut score = 1.0;

        // Penalize high latency
        if tracking.avg_allocation_time > self.thresholds.max_allocation_latency {
            score *= 0.7;
        }

        // Penalize low completeness
        if tracking.completeness < self.thresholds.min_tracking_completeness {
            score *= tracking.completeness / self.thresholds.min_tracking_completeness;
        }

        // Reward high throughput
        if tracking.throughput > 10000.0 {
            score *= 1.1;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_symbol_performance(&self, symbol: &SymbolPerformance) -> f64 {
        let mut score = 1.0;

        // Penalize slow symbol resolution
        if symbol.avg_resolution_time > self.thresholds.max_symbol_resolution_time {
            score *= 0.8;
        }

        // Reward high cache hit ratio
        score *= symbol.cache_hit_ratio;

        // Penalize excessive cache memory usage
        if symbol.cache_memory_usage > 100 * 1024 * 1024 {
            // 100MB
            score *= 0.9;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_pointer_performance(&self, _pointer: &PointerPerformance) -> f64 {
        let mut score: f64 = 1.0;

        // Reward high leak detection accuracy
        score *= _pointer.leak_detection_accuracy;

        // Penalize slow analysis
        if _pointer.analysis_time > Duration::from_millis(100) {
            score *= 0.8;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_memory_efficiency(&self, memory: &MemoryEfficiency) -> f64 {
        let mut score: f64 = 1.0;

        // Penalize excessive memory usage
        if memory.total_memory_mb > self.thresholds.max_analysis_memory as f64 {
            score *= 0.7;
        }

        // Penalize high fragmentation
        if memory.fragmentation_ratio > 0.3 {
            score *= 0.8;
        }

        // Penalize rapid growth
        if memory.growth_rate > 10.0 {
            // 10MB/hour
            score *= 0.9;
        }

        score.clamp(0.0, 1.0)
    }

    fn generate_recommendations(
        &self,
        tracking: &TrackingPerformance,
        symbol: &SymbolPerformance,
        _pointer: &PointerPerformance,
        memory: &MemoryEfficiency,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Tracking recommendations
        if tracking.completeness < 0.95 {
            recommendations
                .push("Improve tracking completeness by reducing lock contention".to_string());
        }
        if tracking.avg_allocation_time > Duration::from_micros(100) {
            recommendations.push("Optimize allocation tracking path for lower latency".to_string());
        }

        // Symbol recommendations
        if symbol.cache_hit_ratio < 0.8 {
            recommendations.push("Increase symbol cache size to improve hit ratio".to_string());
        }
        if symbol.avg_resolution_time > Duration::from_millis(10) {
            recommendations.push("Consider preloading frequently used symbols".to_string());
        }

        // Memory recommendations
        if memory.total_memory_mb > 512.0 {
            recommendations
                .push("Consider reducing memory usage or implementing memory limits".to_string());
        }
        if memory.fragmentation_ratio > 0.2 {
            recommendations.push("Implement memory compaction to reduce fragmentation".to_string());
        }

        recommendations
    }
}

/// Performance comparison between baseline and current
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Operation being compared
    pub operation: String,
    /// Baseline benchmark
    pub baseline: Benchmark,
    /// Current benchmark
    pub current: Benchmark,
    /// Duration ratio (current/baseline)
    pub duration_ratio: f64,
    /// Memory ratio (current/baseline)
    pub memory_ratio: f64,
    /// Throughput ratio (current/baseline)
    pub throughput_ratio: f64,
    /// Accuracy difference (current - baseline)
    pub accuracy_diff: f64,
}

impl Default for AnalysisThresholds {
    fn default() -> Self {
        Self {
            max_tracking_overhead: 0.05, // 5% of app memory
            max_allocation_latency: Duration::from_micros(50),
            max_symbol_resolution_time: Duration::from_millis(5),
            min_tracking_completeness: 0.95,
            max_analysis_memory: 512, // 512MB
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricsCollector;

    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new();
        assert!(analyzer.baselines.is_empty());

        let custom_thresholds = AnalysisThresholds {
            max_tracking_overhead: 0.1,
            ..Default::default()
        };
        let analyzer = PerformanceAnalyzer::with_thresholds(custom_thresholds);
        assert_eq!(analyzer.thresholds.max_tracking_overhead, 0.1);
    }

    #[test]
    fn test_benchmark_comparison() {
        let mut analyzer = PerformanceAnalyzer::new();

        let baseline = Benchmark {
            operation: "allocation_tracking".to_string(),
            avg_duration: Duration::from_micros(100),
            memory_overhead: 1024,
            throughput: 1000.0,
            accuracy: 0.95,
            sample_size: 10000,
        };

        analyzer.set_baseline("allocation_tracking", baseline.clone());

        let current = Benchmark {
            operation: "allocation_tracking".to_string(),
            avg_duration: Duration::from_micros(120),
            memory_overhead: 1200,
            throughput: 900.0,
            accuracy: 0.97,
            sample_size: 10000,
        };

        let comparison = analyzer.compare_to_baseline("allocation_tracking", &current);
        assert!(comparison.is_some());

        let comparison = comparison.expect("Comparison should exist");
        assert!(comparison.duration_ratio > 1.0); // Slower
        assert!(comparison.memory_ratio > 1.0); // More memory
        assert!(comparison.throughput_ratio < 1.0); // Lower throughput
        assert!(comparison.accuracy_diff > 0.0); // Better accuracy
    }

    #[test]
    fn test_efficiency_scoring() {
        let analyzer = PerformanceAnalyzer::new();

        let good_tracking = TrackingPerformance {
            avg_allocation_time: Duration::from_micros(10),
            completeness: 0.98,
            overhead_bytes: 1024,
            throughput: 50000.0,
        };

        let score = analyzer.score_tracking_performance(&good_tracking);
        assert!(score > 0.9);

        let bad_tracking = TrackingPerformance {
            avg_allocation_time: Duration::from_millis(1),
            completeness: 0.8,
            overhead_bytes: 10240,
            throughput: 100.0,
        };

        let score = analyzer.score_tracking_performance(&bad_tracking);
        assert!(score < 0.7);
    }
}
