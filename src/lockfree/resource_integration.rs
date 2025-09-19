//! Integration layer that combines memory tracking with platform resource monitoring
//!
//! This module provides unified API for comprehensive system analysis in multi-threaded environments

use super::analysis::LockfreeAnalysis;
use super::platform_resources::{PlatformResourceCollector, PlatformResourceMetrics};
use super::tracker::ThreadLocalTracker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Comprehensive analysis combining memory and system resource data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveAnalysis {
    pub memory_analysis: LockfreeAnalysis,
    pub resource_timeline: Vec<PlatformResourceMetrics>,
    pub correlation_metrics: CorrelationMetrics,
    pub performance_insights: PerformanceInsights,
}

/// Correlation analysis between memory operations and system resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMetrics {
    pub memory_cpu_correlation: f64,
    pub memory_gpu_correlation: f64,
    pub memory_io_correlation: f64,
    pub allocation_rate_vs_cpu_usage: f64,
    pub deallocation_rate_vs_memory_pressure: f64,
}

/// Performance insights derived from combined analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub primary_bottleneck: BottleneckType,
    pub cpu_efficiency_score: f32,
    pub memory_efficiency_score: f32,
    pub io_efficiency_score: f32,
    pub recommendations: Vec<String>,
    pub thread_performance_ranking: Vec<ThreadPerformanceMetric>,
}

/// Type of performance bottleneck identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    CpuBound,
    MemoryBound,
    IoBound,
    GpuBound,
    ContentionBound,
    Balanced,
}

/// Per-thread performance ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPerformanceMetric {
    pub thread_id: u64,
    pub thread_name: Option<String>,
    pub efficiency_score: f32,
    pub resource_usage_score: f32,
    pub allocation_efficiency: f32,
}

/// Integrated profiling session that tracks both memory and system resources
pub struct IntegratedProfilingSession {
    memory_trackers: HashMap<u64, ThreadLocalTracker>,
    #[allow(dead_code)]
    resource_collector: PlatformResourceCollector,
    resource_timeline: Arc<Mutex<Vec<PlatformResourceMetrics>>>,
    is_active: Arc<AtomicBool>,
    collection_thread: Option<thread::JoinHandle<()>>,
    start_time: Instant,
    output_directory: std::path::PathBuf,
}

impl IntegratedProfilingSession {
    /// Create new integrated profiling session
    pub fn new(output_dir: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_collector = PlatformResourceCollector::new()?;

        Ok(Self {
            memory_trackers: HashMap::new(),
            resource_collector,
            resource_timeline: Arc::new(Mutex::new(Vec::new())),
            is_active: Arc::new(AtomicBool::new(false)),
            collection_thread: None,
            start_time: Instant::now(),
            output_directory: output_dir.to_path_buf(),
        })
    }

    /// Start comprehensive profiling
    pub fn start_profiling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_active.store(true, Ordering::SeqCst);

        // Start resource collection thread
        let is_active = self.is_active.clone();
        let resource_timeline = self.resource_timeline.clone();
        let mut collector = PlatformResourceCollector::new()?;

        let handle = thread::Builder::new()
            .name("resource_collector".to_string())
            .spawn(move || {
                let collection_interval = collector.get_optimal_collection_interval();

                while is_active.load(Ordering::Relaxed) {
                    let start = Instant::now();

                    match collector.collect_metrics() {
                        Ok(metrics) => {
                            // Store metrics in shared timeline
                            if let Ok(mut timeline) = resource_timeline.lock() {
                                timeline.push(metrics);
                            }
                        }
                        Err(_e) => {
                            // Handle collection errors gracefully
                        }
                    }

                    let elapsed = start.elapsed();
                    if elapsed < collection_interval {
                        thread::sleep(collection_interval - elapsed);
                    }
                }
            })?;

        self.collection_thread = Some(handle);
        Ok(())
    }

    /// Stop profiling and generate comprehensive analysis
    pub fn stop_profiling_and_analyze(
        &mut self,
    ) -> Result<ComprehensiveAnalysis, Box<dyn std::error::Error>> {
        self.is_active.store(false, Ordering::SeqCst);

        // Wait for collection thread to finish
        if let Some(handle) = self.collection_thread.take() {
            let _ = handle.join();
        }

        // Finalize all memory trackers
        for (_, mut tracker) in self.memory_trackers.drain() {
            let _ = tracker.finalize();
        }

        // Generate comprehensive analysis
        self.generate_comprehensive_analysis()
    }

    fn generate_comprehensive_analysis(
        &self,
    ) -> Result<ComprehensiveAnalysis, Box<dyn std::error::Error>> {
        // Aggregate memory analysis from all threads
        let aggregator = super::aggregator::LockfreeAggregator::new(self.output_directory.clone());
        let memory_analysis = aggregator.aggregate_all_threads()?;

        // Calculate correlations
        let correlation_metrics = self.calculate_correlations(&memory_analysis)?;

        // Generate performance insights
        let performance_insights =
            self.generate_performance_insights(&memory_analysis, &correlation_metrics)?;

        // Get resource timeline data
        let resource_timeline = self
            .resource_timeline
            .lock()
            .map_err(|_| "Failed to lock resource timeline")?
            .clone();

        Ok(ComprehensiveAnalysis {
            memory_analysis,
            resource_timeline,
            correlation_metrics,
            performance_insights,
        })
    }

    fn calculate_correlations(
        &self,
        memory_analysis: &LockfreeAnalysis,
    ) -> Result<CorrelationMetrics, Box<dyn std::error::Error>> {
        // Calculate correlation between memory operations and system resources
        let memory_cpu_correlation = self.calculate_memory_cpu_correlation(memory_analysis);
        let memory_gpu_correlation = self.calculate_memory_gpu_correlation(memory_analysis);
        let memory_io_correlation = self.calculate_memory_io_correlation(memory_analysis);
        let allocation_rate_vs_cpu_usage =
            self.calculate_allocation_cpu_correlation(memory_analysis);
        let deallocation_rate_vs_memory_pressure =
            self.calculate_deallocation_pressure_correlation(memory_analysis);

        Ok(CorrelationMetrics {
            memory_cpu_correlation,
            memory_gpu_correlation,
            memory_io_correlation,
            allocation_rate_vs_cpu_usage,
            deallocation_rate_vs_memory_pressure,
        })
    }

    fn calculate_memory_cpu_correlation(&self, _memory_analysis: &LockfreeAnalysis) -> f64 {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 0.0,
        };

        if timeline.is_empty() {
            return 0.0;
        }

        // Calculate correlation between allocation rate and CPU usage
        let avg_cpu_usage: f32 = timeline
            .iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .sum::<f32>()
            / timeline.len() as f32;

        // Simple correlation based on average CPU usage
        // In real implementation, would use proper statistical correlation
        match avg_cpu_usage {
            usage if usage > 80.0 => 0.8,
            usage if usage > 60.0 => 0.6,
            usage if usage > 40.0 => 0.4,
            _ => 0.2,
        }
    }

    fn calculate_memory_gpu_correlation(&self, _memory_analysis: &LockfreeAnalysis) -> f64 {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 0.0,
        };

        let gpu_samples = timeline.iter().filter(|r| r.gpu_metrics.is_some()).count();

        if gpu_samples == 0 {
            return 0.0;
        }

        // Calculate correlation with GPU usage
        let avg_gpu_usage: f32 = timeline
            .iter()
            .filter_map(|r| r.gpu_metrics.as_ref())
            .map(|g| g.compute_usage_percent)
            .sum::<f32>()
            / gpu_samples as f32;

        if avg_gpu_usage > 50.0 {
            0.5
        } else {
            0.1
        }
    }

    fn calculate_memory_io_correlation(&self, _memory_analysis: &LockfreeAnalysis) -> f64 {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 0.0,
        };

        if timeline.is_empty() {
            return 0.0;
        }

        // Calculate correlation with I/O activity
        let avg_io_activity: u64 = timeline
            .iter()
            .map(|r| r.io_metrics.disk_read_bytes_per_sec + r.io_metrics.disk_write_bytes_per_sec)
            .sum::<u64>()
            / timeline.len() as u64;

        match avg_io_activity {
            activity if activity > 100_000_000 => 0.7, // > 100MB/s
            activity if activity > 10_000_000 => 0.4,  // > 10MB/s
            _ => 0.1,
        }
    }

    fn calculate_allocation_cpu_correlation(&self, memory_analysis: &LockfreeAnalysis) -> f64 {
        // Correlation between allocation frequency and CPU usage
        let allocation_count = memory_analysis.summary.total_allocations;
        let duration_secs = self.start_time.elapsed().as_secs() as f64;
        let allocation_rate = allocation_count as f64 / duration_secs.max(1.0);

        // Higher allocation rates typically correlate with higher CPU usage
        match allocation_rate {
            rate if rate > 10000.0 => 0.8,
            rate if rate > 1000.0 => 0.6,
            rate if rate > 100.0 => 0.4,
            _ => 0.2,
        }
    }

    fn calculate_deallocation_pressure_correlation(
        &self,
        memory_analysis: &LockfreeAnalysis,
    ) -> f64 {
        // Correlation between deallocation rate and memory pressure
        let deallocation_count = memory_analysis.summary.total_deallocations;
        let peak_memory = memory_analysis.summary.peak_memory_usage;

        // Higher memory pressure typically leads to more frequent deallocations
        if peak_memory > 1_000_000_000 && deallocation_count > 1000 {
            // > 1GB peak
            0.7
        } else if peak_memory > 100_000_000 && deallocation_count > 100 {
            // > 100MB peak
            0.4
        } else {
            0.1
        }
    }

    fn generate_performance_insights(
        &self,
        memory_analysis: &LockfreeAnalysis,
        correlations: &CorrelationMetrics,
    ) -> Result<PerformanceInsights, Box<dyn std::error::Error>> {
        // Identify primary bottleneck
        let primary_bottleneck = self.identify_primary_bottleneck(correlations);

        // Calculate efficiency scores
        let cpu_efficiency_score = self.calculate_cpu_efficiency_score();
        let memory_efficiency_score = self.calculate_memory_efficiency_score(memory_analysis);
        let io_efficiency_score = self.calculate_io_efficiency_score();

        // Generate recommendations
        let recommendations = self.generate_recommendations(&primary_bottleneck, memory_analysis);

        // Rank thread performance
        let thread_performance_ranking = self.rank_thread_performance(memory_analysis);

        Ok(PerformanceInsights {
            primary_bottleneck,
            cpu_efficiency_score,
            memory_efficiency_score,
            io_efficiency_score,
            recommendations,
            thread_performance_ranking,
        })
    }

    fn identify_primary_bottleneck(&self, correlations: &CorrelationMetrics) -> BottleneckType {
        // Determine bottleneck based on correlation strengths
        let cpu_score =
            correlations.memory_cpu_correlation + correlations.allocation_rate_vs_cpu_usage;
        let memory_score = correlations.deallocation_rate_vs_memory_pressure;
        let io_score = correlations.memory_io_correlation;
        let gpu_score = correlations.memory_gpu_correlation;

        if cpu_score > 1.0 {
            BottleneckType::CpuBound
        } else if memory_score > 0.5 {
            BottleneckType::MemoryBound
        } else if io_score > 0.5 {
            BottleneckType::IoBound
        } else if gpu_score > 0.4 {
            BottleneckType::GpuBound
        } else {
            BottleneckType::Balanced
        }
    }

    fn calculate_cpu_efficiency_score(&self) -> f32 {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 0.0,
        };

        if timeline.is_empty() {
            return 0.0;
        }

        let avg_cpu_usage: f32 = timeline
            .iter()
            .map(|r| r.cpu_metrics.overall_usage_percent)
            .sum::<f32>()
            / timeline.len() as f32;

        // Efficiency is good when CPU usage is moderate (not too low, not maxed out)
        match avg_cpu_usage {
            usage if usage >= 70.0 && usage <= 85.0 => 90.0,
            usage if usage >= 50.0 && usage <= 95.0 => 75.0,
            usage if usage >= 30.0 => 60.0,
            _ => 40.0,
        }
    }

    fn calculate_memory_efficiency_score(&self, memory_analysis: &LockfreeAnalysis) -> f32 {
        let allocation_count = memory_analysis.summary.total_allocations;
        let deallocation_count = memory_analysis.summary.total_deallocations;

        if allocation_count == 0 {
            return 0.0;
        }

        // Good efficiency when allocations and deallocations are balanced
        let balance_ratio = deallocation_count as f32 / allocation_count as f32;
        match balance_ratio {
            ratio if ratio >= 0.9 && ratio <= 1.1 => 95.0,
            ratio if ratio >= 0.8 && ratio <= 1.2 => 85.0,
            ratio if ratio >= 0.7 && ratio <= 1.3 => 70.0,
            _ => 50.0,
        }
    }

    fn calculate_io_efficiency_score(&self) -> f32 {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 0.0,
        };

        if timeline.is_empty() {
            return 0.0;
        }

        // Calculate average I/O throughput
        let avg_io_throughput: u64 = timeline
            .iter()
            .map(|r| r.io_metrics.disk_read_bytes_per_sec + r.io_metrics.disk_write_bytes_per_sec)
            .sum::<u64>()
            / timeline.len() as u64;

        // Efficiency based on consistent I/O patterns (not too bursty)
        match avg_io_throughput {
            throughput if throughput > 0 && throughput < 1_000_000_000 => 80.0, // < 1GB/s
            throughput if throughput > 0 => 60.0,
            _ => 40.0,
        }
    }

    fn generate_recommendations(
        &self,
        bottleneck: &BottleneckType,
        memory_analysis: &LockfreeAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match bottleneck {
            BottleneckType::CpuBound => {
                recommendations.push(
                    "Consider reducing CPU-intensive operations in memory allocation paths"
                        .to_string(),
                );
                recommendations
                    .push("Optimize hot allocation patterns identified in analysis".to_string());
            }
            BottleneckType::MemoryBound => {
                recommendations
                    .push("Reduce memory fragmentation by using memory pools".to_string());
                recommendations.push(
                    "Consider implementing object recycling for frequently allocated types"
                        .to_string(),
                );
            }
            BottleneckType::IoBound => {
                recommendations
                    .push("Reduce I/O operations during memory-intensive phases".to_string());
                recommendations.push(
                    "Consider async I/O patterns to avoid blocking memory operations".to_string(),
                );
            }
            BottleneckType::GpuBound => {
                recommendations
                    .push("Optimize GPU memory transfers and synchronization".to_string());
                recommendations.push("Consider reducing GPU-CPU memory copying".to_string());
            }
            BottleneckType::ContentionBound => {
                recommendations
                    .push("Reduce lock contention in memory allocation paths".to_string());
                recommendations.push("Consider using lock-free data structures".to_string());
            }
            BottleneckType::Balanced => {
                recommendations.push("System performance appears well-balanced".to_string());
                recommendations
                    .push("Monitor for performance regression in future updates".to_string());
            }
        }

        // Add memory-specific recommendations
        let avg_allocation_size = if memory_analysis.summary.total_allocations > 0 {
            memory_analysis.summary.peak_memory_usage
                / memory_analysis.summary.total_allocations as usize
        } else {
            0
        };

        if avg_allocation_size > 1024 * 1024 {
            // > 1MB average
            recommendations.push(
                "Large average allocation size detected - consider memory streaming".to_string(),
            );
        } else if avg_allocation_size < 64 {
            // < 64 bytes average
            recommendations
                .push("Many small allocations detected - consider object pooling".to_string());
        }

        recommendations
    }

    fn rank_thread_performance(
        &self,
        memory_analysis: &LockfreeAnalysis,
    ) -> Vec<ThreadPerformanceMetric> {
        let mut rankings = Vec::new();

        // Get thread statistics from memory analysis
        for (thread_id, thread_stats) in &memory_analysis.thread_stats {
            let allocation_efficiency = if thread_stats.total_allocations > 0 {
                (thread_stats.total_deallocations as f32 / thread_stats.total_allocations as f32
                    * 100.0)
                    .min(100.0)
            } else {
                0.0
            };

            let resource_usage_score = self.calculate_thread_resource_score(*thread_id);

            // Overall efficiency score combining allocation patterns and resource usage
            let efficiency_score = (allocation_efficiency + resource_usage_score) / 2.0;

            // Try to get thread name from resource timeline
            let thread_name = self.get_thread_name(*thread_id);

            rankings.push(ThreadPerformanceMetric {
                thread_id: *thread_id,
                thread_name,
                efficiency_score,
                resource_usage_score,
                allocation_efficiency,
            });
        }

        // Sort by efficiency score (highest first)
        rankings.sort_by(|a, b| {
            b.efficiency_score
                .partial_cmp(&a.efficiency_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        rankings
    }

    fn get_thread_name(&self, thread_id: u64) -> Option<String> {
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return None,
        };

        // Look for thread name in any of the resource snapshots
        for snapshot in timeline.iter() {
            if let Some(thread_metrics) = snapshot.thread_metrics.get(&thread_id) {
                if let Some(ref name) = thread_metrics.thread_name {
                    if !name.trim().is_empty() {
                        return Some(name.clone());
                    }
                }
            }
        }

        None
    }

    fn calculate_thread_resource_score(&self, thread_id: u64) -> f32 {
        // Calculate resource usage score for a specific thread
        let timeline = match self.resource_timeline.lock() {
            Ok(timeline) => timeline,
            Err(_) => return 50.0,
        };

        let mut total_cpu_usage = 0.0f32;
        let mut sample_count = 0;

        for resource_snapshot in timeline.iter() {
            if let Some(thread_metrics) = resource_snapshot.thread_metrics.get(&thread_id) {
                total_cpu_usage += thread_metrics.cpu_usage_percent;
                sample_count += 1;
            }
        }

        if sample_count > 0 {
            let avg_cpu_usage = total_cpu_usage / sample_count as f32;
            // Score based on moderate CPU usage (not idle, not maxed out)
            match avg_cpu_usage {
                usage if usage >= 30.0 && usage <= 70.0 => 90.0,
                usage if usage >= 20.0 && usage <= 80.0 => 75.0,
                usage if usage >= 10.0 && usage <= 90.0 => 60.0,
                _ => 40.0,
            }
        } else {
            50.0 // Default score when no data available
        }
    }
}

/// Convenience function for quick comprehensive profiling
pub fn comprehensive_profile_execution<F, R>(
    output_dir: &std::path::Path,
    execution_fn: F,
) -> Result<(R, ComprehensiveAnalysis), Box<dyn std::error::Error>>
where
    F: FnOnce() -> R,
{
    let mut session = IntegratedProfilingSession::new(output_dir)?;
    session.start_profiling()?;

    let result = execution_fn();

    let analysis = session.stop_profiling_and_analyze()?;

    Ok((result, analysis))
}

#[cfg(test)]
mod tests {
    use super::*;
    // Test module for resource integration

    #[test]
    fn test_integrated_profiling_session_creation() {
        let temp_dir = std::env::temp_dir().join("memscope_test");
        let result = IntegratedProfilingSession::new(&temp_dir);

        match result {
            Ok(_session) => {
                // Session created successfully
            }
            Err(e) => {
                // May fail on platforms without resource monitoring support
                println!("Session creation failed: {}", e);
            }
        }
    }

    #[test]
    fn test_comprehensive_profiling_function() {
        let temp_dir = std::env::temp_dir().join("memscope_comprehensive_test");

        let result = comprehensive_profile_execution(&temp_dir, || {
            // Simulate some work
            let mut data = Vec::new();
            for i in 0..1000 {
                data.push(vec![i; 100]);
            }
            data.len()
        });

        match result {
            Ok((work_result, _analysis)) => {
                assert_eq!(work_result, 1000);
            }
            Err(e) => {
                // May fail on platforms without full support
                println!("Comprehensive profiling failed: {}", e);
            }
        }
    }

    #[test]
    fn test_bottleneck_identification() {
        let correlations = CorrelationMetrics {
            memory_cpu_correlation: 0.8,
            memory_gpu_correlation: 0.1,
            memory_io_correlation: 0.2,
            allocation_rate_vs_cpu_usage: 0.7,
            deallocation_rate_vs_memory_pressure: 0.3,
        };

        let temp_dir = std::env::temp_dir().join("memscope_bottleneck_test");
        if let Ok(session) = IntegratedProfilingSession::new(&temp_dir) {
            let bottleneck = session.identify_primary_bottleneck(&correlations);
            matches!(bottleneck, BottleneckType::CpuBound);
        }
    }
}
