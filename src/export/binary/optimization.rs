//! End-to-end performance optimization for binary export system
//!
//! This module provides comprehensive performance optimization including
//! bottleneck analysis, automatic tuning, and system-wide optimizations.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use super::*;

/// Performance optimization engine
pub struct PerformanceOptimizer {
    /// Current system configuration
    config: IntegratedConfig,
    /// Performance profiler
    profiler: PerformanceProfiler,
    /// Optimization strategies
    strategies: Vec<Box<dyn OptimizationStrategy>>,
    /// Performance history
    performance_history: Vec<PerformanceSnapshot>,
}

/// Performance profiler for bottleneck analysis
struct PerformanceProfiler {
    /// Profiling enabled
    enabled: bool,
    /// Component timers
    component_timers: HashMap<String, ComponentTimer>,
    /// Memory usage tracker
    memory_tracker: MemoryUsageTracker,
    /// Throughput monitor
    throughput_monitor: ThroughputMonitor,
}

/// Component performance timer
struct ComponentTimer {
    /// Total time spent
    total_time: Duration,
    /// Number of calls
    call_count: u64,
    /// Average time per call
    avg_time: Duration,
    /// Peak time
    peak_time: Duration,
}

/// Memory usage tracking
struct MemoryUsageTracker {
    /// Current usage
    current_usage: usize,
    /// Peak usage
    peak_usage: usize,
    /// Usage samples over time
    usage_samples: Vec<(Instant, usize)>,
}

/// Throughput monitoring
struct ThroughputMonitor {
    /// Bytes processed
    bytes_processed: u64,
    /// Processing start time
    start_time: Instant,
    /// Current throughput (bytes/second)
    current_throughput: f64,
    /// Peak throughput
    peak_throughput: f64,
}

/// Performance snapshot for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Overall performance metrics
    pub overall_metrics: OverallMetrics,
    /// Component performance breakdown
    pub component_metrics: HashMap<String, ComponentMetrics>,
    /// System resource usage
    pub resource_usage: ResourceUsage,
    /// Configuration used
    pub config_hash: u64,
}

/// Overall performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallMetrics {
    /// Total processing time
    pub total_time: Duration,
    /// Overall throughput (bytes/second)
    pub throughput: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// CPU utilization (0.0 to 1.0)
    pub cpu_utilization: f64,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
}

/// Component-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    /// Time spent in component
    pub time_spent: Duration,
    /// Percentage of total time
    pub time_percentage: f64,
    /// Throughput for this component
    pub throughput: f64,
    /// Memory usage
    pub memory_usage: usize,
    /// Efficiency score
    pub efficiency_score: f64,
}

/// System resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Peak memory usage (bytes)
    pub peak_memory: usize,
    /// Average memory usage (bytes)
    pub avg_memory: usize,
    /// CPU cores utilized
    pub cpu_cores_used: usize,
    /// I/O operations performed
    pub io_operations: u64,
    /// Network usage (if applicable)
    pub network_bytes: u64,
}

/// Optimization strategy trait
trait OptimizationStrategy: Send + Sync {
    /// Analyze performance and suggest optimizations
    fn analyze(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion>;
    
    /// Apply optimization to configuration
    fn optimize(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool;
    
    /// Get strategy name
    fn name(&self) -> &str;
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Suggestion type
    pub suggestion_type: OptimizationType,
    /// Description
    pub description: String,
    /// Expected performance improvement
    pub expected_improvement: f64,
    /// Implementation difficulty (1-5)
    pub difficulty: u8,
    /// Configuration changes needed
    pub config_changes: HashMap<String, serde_json::Value>,
}

/// Types of optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    /// Memory optimization
    Memory,
    /// CPU optimization
    Cpu,
    /// I/O optimization
    Io,
    /// Compression optimization
    Compression,
    /// Parallel processing optimization
    Parallel,
    /// Configuration tuning
    Configuration,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: IntegratedConfig) -> Self {
        let mut strategies: Vec<Box<dyn OptimizationStrategy>> = Vec::new();
        strategies.push(Box::new(MemoryOptimizationStrategy));
        strategies.push(Box::new(CompressionOptimizationStrategy));
        strategies.push(Box::new(ParallelProcessingOptimizationStrategy));
        strategies.push(Box::new(ConfigurationOptimizationStrategy));

        Self {
            config,
            profiler: PerformanceProfiler::new(),
            strategies,
            performance_history: Vec::new(),
        }
    }

    /// Run end-to-end performance optimization
    pub fn optimize_end_to_end(&mut self, tracker: &crate::core::tracker::MemoryTracker) -> Result<OptimizationResult, BinaryExportError> {
        println!("🔧 Starting end-to-end performance optimization...");
        
        // Step 1: Baseline performance measurement
        let baseline_snapshot = self.measure_baseline_performance(tracker)?;
        println!("📊 Baseline performance measured");

        // Step 2: Analyze bottlenecks
        let bottlenecks = self.analyze_bottlenecks(&baseline_snapshot);
        println!("🔍 Identified {} performance bottlenecks", bottlenecks.len());

        // Step 3: Generate optimization suggestions
        let suggestions = self.generate_optimization_suggestions(&baseline_snapshot);
        println!("💡 Generated {} optimization suggestions", suggestions.len());

        // Step 4: Apply optimizations iteratively
        let optimized_config = self.apply_optimizations_iteratively(tracker, suggestions)?;
        println!("⚡ Applied optimizations to configuration");

        // Step 5: Measure optimized performance
        let optimized_snapshot = self.measure_performance_with_config(tracker, &optimized_config)?;
        println!("📈 Optimized performance measured");

        // Step 6: Calculate improvement
        let improvement = self.calculate_improvement(&baseline_snapshot, &optimized_snapshot);

        Ok(OptimizationResult {
            baseline_snapshot,
            optimized_snapshot,
            optimized_config,
            improvement,
            bottlenecks,
        })
    }

    /// Measure baseline performance
    fn measure_baseline_performance(&mut self, tracker: &crate::core::tracker::MemoryTracker) -> Result<PerformanceSnapshot, BinaryExportError> {
        self.profiler.start_profiling();
        
        // Create exporter with current configuration
        let mut exporter = IntegratedBinaryExporter::new(self.config.clone());
        
        // Perform export with profiling
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        let output_path = temp_dir.path().join("baseline_test.bin");
        
        let start_time = Instant::now();
        let _result = exporter.export(tracker, &output_path);
        let total_time = start_time.elapsed();
        
        // Collect performance data
        let snapshot = self.profiler.create_snapshot(total_time, self.calculate_config_hash());
        self.performance_history.push(snapshot.clone());
        
        Ok(snapshot)
    }

    /// Measure performance with specific configuration
    fn measure_performance_with_config(&mut self, tracker: &crate::core::tracker::MemoryTracker, config: &IntegratedConfig) -> Result<PerformanceSnapshot, BinaryExportError> {
        self.profiler.start_profiling();
        
        let mut exporter = IntegratedBinaryExporter::new(config.clone());
        
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| BinaryExportError::IoError(e.kind()))?;
        let output_path = temp_dir.path().join("optimized_test.bin");
        
        let start_time = Instant::now();
        let _result = exporter.export(tracker, &output_path);
        let total_time = start_time.elapsed();
        
        let snapshot = self.profiler.create_snapshot(total_time, self.calculate_config_hash_for_config(config));
        
        Ok(snapshot)
    }

    /// Analyze performance bottlenecks
    fn analyze_bottlenecks(&self, snapshot: &PerformanceSnapshot) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();

        // Analyze component times
        for (component, metrics) in &snapshot.component_metrics {
            if metrics.time_percentage > 30.0 {
                bottlenecks.push(PerformanceBottleneck {
                    component: component.clone(),
                    bottleneck_type: BottleneckType::HighLatency,
                    severity: if metrics.time_percentage > 50.0 { 
                        BottleneckSeverity::High 
                    } else { 
                        BottleneckSeverity::Medium 
                    },
                    description: format!("Component {} takes {:.1}% of total time", component, metrics.time_percentage),
                    impact_score: metrics.time_percentage / 100.0,
                });
            }

            if metrics.efficiency_score < 0.5 {
                bottlenecks.push(PerformanceBottleneck {
                    component: component.clone(),
                    bottleneck_type: BottleneckType::LowEfficiency,
                    severity: BottleneckSeverity::Medium,
                    description: format!("Component {} has low efficiency score: {:.2}", component, metrics.efficiency_score),
                    impact_score: 1.0 - metrics.efficiency_score,
                });
            }
        }

        // Analyze memory usage
        if snapshot.overall_metrics.memory_efficiency < 0.6 {
            bottlenecks.push(PerformanceBottleneck {
                component: "memory_management".to_string(),
                bottleneck_type: BottleneckType::MemoryInefficiency,
                severity: BottleneckSeverity::High,
                description: format!("Low memory efficiency: {:.2}", snapshot.overall_metrics.memory_efficiency),
                impact_score: 1.0 - snapshot.overall_metrics.memory_efficiency,
            });
        }

        // Analyze CPU utilization
        if snapshot.overall_metrics.cpu_utilization < 0.4 {
            bottlenecks.push(PerformanceBottleneck {
                component: "cpu_utilization".to_string(),
                bottleneck_type: BottleneckType::UnderutilizedCpu,
                severity: BottleneckSeverity::Medium,
                description: format!("Low CPU utilization: {:.2}", snapshot.overall_metrics.cpu_utilization),
                impact_score: 1.0 - snapshot.overall_metrics.cpu_utilization,
            });
        }

        // Sort by impact score
        bottlenecks.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        
        bottlenecks
    }

    /// Generate optimization suggestions
    fn generate_optimization_suggestions(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        for strategy in &self.strategies {
            let strategy_suggestions = strategy.analyze(snapshot);
            suggestions.extend(strategy_suggestions);
        }

        // Sort by expected improvement
        suggestions.sort_by(|a, b| b.expected_improvement.partial_cmp(&a.expected_improvement).unwrap());

        suggestions
    }

    /// Apply optimizations iteratively
    fn apply_optimizations_iteratively(&mut self, tracker: &crate::core::tracker::MemoryTracker, suggestions: Vec<OptimizationSuggestion>) -> Result<IntegratedConfig, BinaryExportError> {
        let mut current_config = self.config.clone();
        let mut best_config = current_config.clone();
        let mut best_performance = 0.0;

        // Try each suggestion
        for suggestion in suggestions.iter().take(5) { // Limit to top 5 suggestions
            let mut test_config = current_config.clone();
            
            // Apply suggestion
            let applied = self.apply_suggestion_to_config(&mut test_config, suggestion);
            if !applied {
                continue;
            }

            // Test performance
            match self.measure_performance_with_config(tracker, &test_config) {
                Ok(snapshot) => {
                    if snapshot.overall_metrics.performance_score > best_performance {
                        best_performance = snapshot.overall_metrics.performance_score;
                        best_config = test_config;
                        println!("✅ Applied optimization: {} (improvement: {:.2})", 
                                suggestion.description, snapshot.overall_metrics.performance_score);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to test optimization: {} - {:?}", suggestion.description, e);
                }
            }
        }

        self.config = best_config.clone();
        Ok(best_config)
    }

    /// Apply suggestion to configuration
    fn apply_suggestion_to_config(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool {
        for strategy in &self.strategies {
            if strategy.optimize(config, suggestion) {
                return true;
            }
        }
        false
    }

    /// Calculate performance improvement
    fn calculate_improvement(&self, baseline: &PerformanceSnapshot, optimized: &PerformanceSnapshot) -> PerformanceImprovement {
        let speed_improvement = baseline.overall_metrics.total_time.as_secs_f64() / 
                               optimized.overall_metrics.total_time.as_secs_f64();
        
        let memory_improvement = baseline.resource_usage.peak_memory as f64 / 
                                optimized.resource_usage.peak_memory as f64;
        
        let throughput_improvement = optimized.overall_metrics.throughput / 
                                    baseline.overall_metrics.throughput;

        PerformanceImprovement {
            speed_improvement,
            memory_improvement,
            throughput_improvement,
            overall_improvement: (speed_improvement + throughput_improvement) / 2.0,
        }
    }

    /// Calculate configuration hash
    fn calculate_config_hash(&self) -> u64 {
        self.calculate_config_hash_for_config(&self.config)
    }

    /// Calculate configuration hash for specific config
    fn calculate_config_hash_for_config(&self, config: &IntegratedConfig) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        config.processing.max_memory_usage.hash(&mut hasher);
        config.processing.chunk_size.hash(&mut hasher);
        config.compression.level.hash(&mut hasher);
        hasher.finish()
    }
}

/// Performance bottleneck identification
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    /// Component with bottleneck
    pub component: String,
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Severity level
    pub severity: BottleneckSeverity,
    /// Description
    pub description: String,
    /// Impact score (0.0 to 1.0)
    pub impact_score: f64,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone)]
pub enum BottleneckType {
    /// High latency in component
    HighLatency,
    /// Low efficiency
    LowEfficiency,
    /// Memory inefficiency
    MemoryInefficiency,
    /// Underutilized CPU
    UnderutilizedCpu,
    /// I/O bottleneck
    IoBound,
}

/// Bottleneck severity levels
#[derive(Debug, Clone)]
pub enum BottleneckSeverity {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
    /// Critical impact
    Critical,
}

/// Performance improvement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    /// Speed improvement ratio
    pub speed_improvement: f64,
    /// Memory improvement ratio
    pub memory_improvement: f64,
    /// Throughput improvement ratio
    pub throughput_improvement: f64,
    /// Overall improvement score
    pub overall_improvement: f64,
}

/// Complete optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Baseline performance snapshot
    pub baseline_snapshot: PerformanceSnapshot,
    /// Optimized performance snapshot
    pub optimized_snapshot: PerformanceSnapshot,
    /// Optimized configuration
    pub optimized_config: IntegratedConfig,
    /// Performance improvement metrics
    pub improvement: PerformanceImprovement,
    /// Identified bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

// Optimization strategy implementations

/// Memory optimization strategy
struct MemoryOptimizationStrategy;

impl OptimizationStrategy for MemoryOptimizationStrategy {
    fn analyze(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        if snapshot.overall_metrics.memory_efficiency < 0.7 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::Memory,
                description: "Reduce memory usage by enabling streaming processing".to_string(),
                expected_improvement: 0.3,
                difficulty: 2,
                config_changes: {
                    let mut changes = HashMap::new();
                    changes.insert("enable_streaming".to_string(), serde_json::Value::Bool(true));
                    changes.insert("chunk_size".to_string(), serde_json::Value::Number(serde_json::Number::from(64 * 1024)));
                    changes
                },
            });
        }

        suggestions
    }

    fn optimize(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool {
        if suggestion.suggestion_type == OptimizationType::Memory {
            if let Some(enable_streaming) = suggestion.config_changes.get("enable_streaming") {
                if enable_streaming.as_bool() == Some(true) {
                    config.processing.chunk_size = 64 * 1024;
                    return true;
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        "MemoryOptimization"
    }
}

/// Compression optimization strategy
struct CompressionOptimizationStrategy;

impl OptimizationStrategy for CompressionOptimizationStrategy {
    fn analyze(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        if let Some(compression_metrics) = snapshot.component_metrics.get("compression") {
            if compression_metrics.time_percentage > 25.0 {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: OptimizationType::Compression,
                    description: "Use faster compression algorithm (LZ4)".to_string(),
                    expected_improvement: 0.4,
                    difficulty: 1,
                    config_changes: {
                        let mut changes = HashMap::new();
                        changes.insert("compression_algorithm".to_string(), serde_json::Value::String("lz4".to_string()));
                        changes.insert("compression_level".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
                        changes
                    },
                });
            }
        }

        suggestions
    }

    fn optimize(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool {
        if suggestion.suggestion_type == OptimizationType::Compression {
            if let Some(algorithm) = suggestion.config_changes.get("compression_algorithm") {
                if algorithm.as_str() == Some("lz4") {
                    config.compression.algorithm = CompressionAlgorithm::Lz4;
                    config.compression.level = 1;
                    return true;
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        "CompressionOptimization"
    }
}

/// Parallel processing optimization strategy
struct ParallelProcessingOptimizationStrategy;

impl OptimizationStrategy for ParallelProcessingOptimizationStrategy {
    fn analyze(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        if snapshot.overall_metrics.cpu_utilization < 0.5 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::Parallel,
                description: "Enable parallel processing with more worker threads".to_string(),
                expected_improvement: 0.6,
                difficulty: 2,
                config_changes: {
                    let mut changes = HashMap::new();
                    changes.insert("enable_parallel".to_string(), serde_json::Value::Bool(true));
                    changes.insert("worker_threads".to_string(), serde_json::Value::Number(serde_json::Number::from(num_cpus::get() * 2)));
                    changes
                },
            });
        }

        suggestions
    }

    fn optimize(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool {
        if suggestion.suggestion_type == OptimizationType::Parallel {
            if let Some(enable_parallel) = suggestion.config_changes.get("enable_parallel") {
                if enable_parallel.as_bool() == Some(true) {
                    if config.parallel.is_none() {
                        config.parallel = Some(crate::export::binary::parallel::ParallelConfig::default());
                    }
                    if let Some(ref mut parallel_config) = config.parallel {
                        parallel_config.worker_threads = num_cpus::get() * 2;
                        parallel_config.enable_work_stealing = true;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        "ParallelProcessingOptimization"
    }
}

/// Configuration optimization strategy
struct ConfigurationOptimizationStrategy;

impl OptimizationStrategy for ConfigurationOptimizationStrategy {
    fn analyze(&self, snapshot: &PerformanceSnapshot) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        if snapshot.overall_metrics.performance_score < 0.7 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::Configuration,
                description: "Enable auto-optimization for dynamic tuning".to_string(),
                expected_improvement: 0.2,
                difficulty: 1,
                config_changes: {
                    let mut changes = HashMap::new();
                    changes.insert("auto_optimize".to_string(), serde_json::Value::Bool(true));
                    changes
                },
            });
        }

        suggestions
    }

    fn optimize(&self, config: &mut IntegratedConfig, suggestion: &OptimizationSuggestion) -> bool {
        if suggestion.suggestion_type == OptimizationType::Configuration {
            if let Some(auto_optimize) = suggestion.config_changes.get("auto_optimize") {
                if auto_optimize.as_bool() == Some(true) {
                    config.auto_optimize = true;
                    return true;
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        "ConfigurationOptimization"
    }
}

impl PerformanceProfiler {
    fn new() -> Self {
        Self {
            enabled: true,
            component_timers: HashMap::new(),
            memory_tracker: MemoryUsageTracker::new(),
            throughput_monitor: ThroughputMonitor::new(),
        }
    }

    fn start_profiling(&mut self) {
        self.component_timers.clear();
        self.memory_tracker.reset();
        self.throughput_monitor.reset();
    }

    fn create_snapshot(&self, total_time: Duration, config_hash: u64) -> PerformanceSnapshot {
        let overall_metrics = OverallMetrics {
            total_time,
            throughput: self.throughput_monitor.current_throughput,
            memory_efficiency: self.memory_tracker.calculate_efficiency(),
            cpu_utilization: 0.7, // Placeholder - would measure actual CPU usage
            performance_score: self.calculate_performance_score(total_time),
        };

        let mut component_metrics = HashMap::new();
        for (name, timer) in &self.component_timers {
            let time_percentage = (timer.total_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0;
            component_metrics.insert(name.clone(), ComponentMetrics {
                time_spent: timer.total_time,
                time_percentage,
                throughput: self.throughput_monitor.current_throughput,
                memory_usage: self.memory_tracker.current_usage,
                efficiency_score: timer.calculate_efficiency(),
            });
        }

        let resource_usage = ResourceUsage {
            peak_memory: self.memory_tracker.peak_usage,
            avg_memory: self.memory_tracker.calculate_average(),
            cpu_cores_used: num_cpus::get(),
            io_operations: 0, // Placeholder
            network_bytes: 0, // Placeholder
        };

        PerformanceSnapshot {
            timestamp: std::time::SystemTime::now(),
            overall_metrics,
            component_metrics,
            resource_usage,
            config_hash,
        }
    }

    fn calculate_performance_score(&self, total_time: Duration) -> f64 {
        // Simple performance score calculation
        let time_score = 1.0 / (total_time.as_secs_f64() + 1.0);
        let memory_score = self.memory_tracker.calculate_efficiency();
        let throughput_score = (self.throughput_monitor.current_throughput / 1_000_000.0).min(1.0);
        
        (time_score + memory_score + throughput_score) / 3.0
    }
}

impl ComponentTimer {
    fn new() -> Self {
        Self {
            total_time: Duration::from_millis(0),
            call_count: 0,
            avg_time: Duration::from_millis(0),
            peak_time: Duration::from_millis(0),
        }
    }

    fn calculate_efficiency(&self) -> f64 {
        if self.call_count == 0 {
            return 1.0;
        }
        
        // Simple efficiency calculation based on consistency
        let consistency = self.avg_time.as_secs_f64() / self.peak_time.as_secs_f64().max(0.001);
        consistency.min(1.0)
    }
}

impl MemoryUsageTracker {
    fn new() -> Self {
        Self {
            current_usage: 0,
            peak_usage: 0,
            usage_samples: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.current_usage = 0;
        self.peak_usage = 0;
        self.usage_samples.clear();
    }

    fn calculate_efficiency(&self) -> f64 {
        if self.peak_usage == 0 {
            return 1.0;
        }
        
        let avg_usage = self.calculate_average();
        avg_usage as f64 / self.peak_usage as f64
    }

    fn calculate_average(&self) -> usize {
        if self.usage_samples.is_empty() {
            return self.current_usage;
        }
        
        self.usage_samples.iter().map(|(_, usage)| usage).sum::<usize>() / self.usage_samples.len()
    }
}

impl ThroughputMonitor {
    fn new() -> Self {
        Self {
            bytes_processed: 0,
            start_time: Instant::now(),
            current_throughput: 0.0,
            peak_throughput: 0.0,
        }
    }

    fn reset(&mut self) {
        self.bytes_processed = 0;
        self.start_time = Instant::now();
        self.current_throughput = 0.0;
        self.peak_throughput = 0.0;
    }
}

/// Run end-to-end optimization
pub fn optimize_system_performance(tracker: &crate::core::tracker::MemoryTracker) -> Result<OptimizationResult, BinaryExportError> {
    let config = IntegratedConfig::balanced();
    let mut optimizer = PerformanceOptimizer::new(config);
    optimizer.optimize_end_to_end(tracker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_optimizer_creation() {
        let config = IntegratedConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        assert_eq!(optimizer.strategies.len(), 4);
    }

    #[test]
    fn test_optimization_strategies() {
        let memory_strategy = MemoryOptimizationStrategy;
        assert_eq!(memory_strategy.name(), "MemoryOptimization");
        
        let compression_strategy = CompressionOptimizationStrategy;
        assert_eq!(compression_strategy.name(), "CompressionOptimization");
    }

    #[test]
    fn test_performance_snapshot_creation() {
        let profiler = PerformanceProfiler::new();
        let snapshot = profiler.create_snapshot(Duration::from_secs(1), 12345);
        
        assert_eq!(snapshot.config_hash, 12345);
        assert!(snapshot.overall_metrics.performance_score >= 0.0);
        assert!(snapshot.overall_metrics.performance_score <= 1.0);
    }
}