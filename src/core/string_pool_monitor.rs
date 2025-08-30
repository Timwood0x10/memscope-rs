//! String pool monitoring and statistics
//!
//! This module provides monitoring capabilities for the string pool system,
//! including performance metrics, memory usage tracking, and optimization
//! recommendations.

use crate::core::string_pool::{get_string_pool_stats, StringPoolStats};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Detailed string pool monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringPoolMonitorStats {
    /// Basic string pool statistics
    pub pool_stats: StringPoolStats,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Memory efficiency metrics
    pub memory_efficiency: MemoryEfficiencyMetrics,
    /// Usage patterns
    pub usage_patterns: UsagePatterns,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
}

/// Performance metrics for string pool operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average time per intern operation in nanoseconds
    pub avg_intern_time_ns: f64,
    /// Peak intern operations per second
    pub peak_ops_per_second: f64,
    /// Current intern operations per second
    pub current_ops_per_second: f64,
    /// Total time spent in intern operations (nanoseconds)
    pub total_intern_time_ns: u64,
}

/// Memory efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEfficiencyMetrics {
    /// Memory efficiency ratio (0.0 to 1.0, higher is better)
    pub efficiency_ratio: f64,
    /// Total memory that would be used without interning
    pub memory_without_interning_bytes: u64,
    /// Actual memory used with interning
    pub memory_with_interning_bytes: u64,
    /// Memory overhead of the pool structure itself
    pub pool_overhead_bytes: u64,
}

/// Usage patterns analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    /// Most frequently interned strings
    pub top_strings: Vec<StringUsageInfo>,
    /// Distribution of string lengths
    pub length_distribution: Vec<LengthBucket>,
    /// Temporal usage patterns
    pub temporal_patterns: TemporalPatterns,
}

/// Information about a frequently used string
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringUsageInfo {
    /// The string content (truncated if too long)
    pub content: String,
    /// Number of times this string was interned
    pub usage_count: u64,
    /// Estimated memory saved by interning this string
    pub memory_saved_bytes: u64,
}

/// String length distribution bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LengthBucket {
    /// Minimum length in this bucket (inclusive)
    pub min_length: usize,
    /// Maximum length in this bucket (exclusive)
    pub max_length: usize,
    /// Number of strings in this bucket
    pub count: usize,
    /// Total bytes for strings in this bucket
    pub total_bytes: usize,
}

/// Temporal usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    /// Intern operations in the last minute
    pub ops_last_minute: u64,
    /// Intern operations in the last hour
    pub ops_last_hour: u64,
    /// Peak operations per minute observed
    pub peak_ops_per_minute: u64,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,
    /// Description of the recommendation
    pub description: String,
    /// Estimated impact if implemented
    pub estimated_impact: String,
    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
}

/// Types of optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Increase pool size
    IncreasePoolSize,
    /// Decrease pool size
    DecreasePoolSize,
    /// Enable/disable specific features
    FeatureToggle,
    /// Memory optimization
    MemoryOptimization,
    /// Performance optimization
    PerformanceOptimization,
    /// Usage pattern optimization
    UsageOptimization,
}

/// String pool monitor that tracks usage and performance
#[allow(dead_code)]
pub struct StringPoolMonitor {
    /// Performance tracking
    performance_tracker: Arc<Mutex<PerformanceTracker>>,
    /// Start time for monitoring
    start_time: Instant,
}

struct PerformanceTracker {
    total_intern_time_ns: u64,
    intern_count: u64,
    last_ops_calculation: Instant,
    ops_in_last_second: u64,
    peak_ops_per_second: f64,
    recent_intern_times: Vec<u64>, // Ring buffer of recent intern times
}

impl StringPoolMonitor {
    /// Create a new string pool monitor
    pub fn new() -> Self {
        Self {
            performance_tracker: Arc::new(Mutex::new(PerformanceTracker {
                total_intern_time_ns: 0,
                intern_count: 0,
                last_ops_calculation: Instant::now(),
                ops_in_last_second: 0,
                peak_ops_per_second: 0.0,
                recent_intern_times: Vec::with_capacity(1000),
            })),
            start_time: Instant::now(),
        }
    }

    /// Record an intern operation with its duration
    pub fn record_intern_operation(&self, duration_ns: u64) {
        if let Ok(mut tracker) = self.performance_tracker.lock() {
            tracker.total_intern_time_ns += duration_ns;
            tracker.intern_count += 1;

            // Add to recent times (ring buffer)
            if tracker.recent_intern_times.len() >= 1000 {
                tracker.recent_intern_times.remove(0);
            }
            tracker.recent_intern_times.push(duration_ns);

            // Update operations per second calculation
            let now = Instant::now();
            if now.duration_since(tracker.last_ops_calculation) >= Duration::from_secs(1) {
                let ops_per_second = tracker.ops_in_last_second as f64;
                if ops_per_second > tracker.peak_ops_per_second {
                    tracker.peak_ops_per_second = ops_per_second;
                }
                tracker.ops_in_last_second = 0;
                tracker.last_ops_calculation = now;
            } else {
                tracker.ops_in_last_second += 1;
            }
        }
    }

    /// Get comprehensive monitoring statistics
    pub fn get_stats(&self) -> StringPoolMonitorStats {
        let pool_stats = get_string_pool_stats();
        let performance = self.get_performance_metrics();
        let memory_efficiency = self.calculate_memory_efficiency(&pool_stats);
        let usage_patterns = self.analyze_usage_patterns(&pool_stats);
        let recommendations =
            self.generate_recommendations(&pool_stats, &performance, &memory_efficiency);

        StringPoolMonitorStats {
            pool_stats,
            performance,
            memory_efficiency,
            usage_patterns,
            recommendations,
        }
    }

    fn get_performance_metrics(&self) -> PerformanceMetrics {
        if let Ok(tracker) = self.performance_tracker.lock() {
            let avg_intern_time_ns = if tracker.intern_count > 0 {
                tracker.total_intern_time_ns as f64 / tracker.intern_count as f64
            } else {
                0.0
            };

            let current_ops_per_second = tracker.ops_in_last_second as f64;

            PerformanceMetrics {
                avg_intern_time_ns,
                peak_ops_per_second: tracker.peak_ops_per_second,
                current_ops_per_second,
                total_intern_time_ns: tracker.total_intern_time_ns,
            }
        } else {
            PerformanceMetrics {
                avg_intern_time_ns: 0.0,
                peak_ops_per_second: 0.0,
                current_ops_per_second: 0.0,
                total_intern_time_ns: 0,
            }
        }
    }

    fn calculate_memory_efficiency(&self, pool_stats: &StringPoolStats) -> MemoryEfficiencyMetrics {
        // Estimate memory usage
        let avg_string_size = pool_stats.average_string_length as u64;
        let total_unique_strings = pool_stats.unique_strings as u64;
        let total_intern_ops = pool_stats.intern_operations;

        // Memory with interning: unique strings + Arc overhead
        let arc_overhead_per_string = std::mem::size_of::<Arc<str>>() as u64;
        let memory_with_interning =
            (avg_string_size + arc_overhead_per_string) * total_unique_strings;

        // Memory without interning: all strings stored separately
        let memory_without_interning = avg_string_size * total_intern_ops;

        // Pool overhead: HashMap structure + DashMap overhead
        let pool_overhead = total_unique_strings * 64; // Rough estimate

        let efficiency_ratio = if memory_without_interning > 0 {
            1.0 - (memory_with_interning as f64 / memory_without_interning as f64)
        } else {
            0.0
        };

        MemoryEfficiencyMetrics {
            efficiency_ratio: efficiency_ratio.clamp(0.0, 1.0),
            memory_without_interning_bytes: memory_without_interning,
            memory_with_interning_bytes: memory_with_interning,
            pool_overhead_bytes: pool_overhead,
        }
    }

    fn analyze_usage_patterns(&self, _pool_stats: &StringPoolStats) -> UsagePatterns {
        // For now, return empty patterns - in a real implementation,
        // we would track actual string usage
        UsagePatterns {
            top_strings: vec![],
            length_distribution: vec![
                LengthBucket {
                    min_length: 0,
                    max_length: 10,
                    count: 0,
                    total_bytes: 0,
                },
                LengthBucket {
                    min_length: 10,
                    max_length: 50,
                    count: 0,
                    total_bytes: 0,
                },
                LengthBucket {
                    min_length: 50,
                    max_length: 100,
                    count: 0,
                    total_bytes: 0,
                },
                LengthBucket {
                    min_length: 100,
                    max_length: usize::MAX,
                    count: 0,
                    total_bytes: 0,
                },
            ],
            temporal_patterns: TemporalPatterns {
                ops_last_minute: 0,
                ops_last_hour: 0,
                peak_ops_per_minute: 0,
            },
        }
    }

    fn generate_recommendations(
        &self,
        pool_stats: &StringPoolStats,
        performance: &PerformanceMetrics,
        memory_efficiency: &MemoryEfficiencyMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Memory efficiency recommendations
        if memory_efficiency.efficiency_ratio < 0.3 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::MemoryOptimization,
                description:
                    "String pool efficiency is low. Consider reviewing string usage patterns."
                        .to_string(),
                estimated_impact: "Could reduce memory usage by 20-40%".to_string(),
                priority: 4,
            });
        }

        // Performance recommendations
        if performance.avg_intern_time_ns > 1000.0 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::PerformanceOptimization,
                description: "String interning operations are slow. Consider optimizing hash function or reducing contention.".to_string(),
                estimated_impact: "Could improve intern performance by 2-3x".to_string(),
                priority: 3,
            });
        }

        // Pool size recommendations
        if pool_stats.unique_strings > 100000 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::IncreasePoolSize,
                description: "String pool is getting large. Consider implementing LRU eviction or pool size limits.".to_string(),
                estimated_impact: "Could reduce memory usage by 10-20%".to_string(),
                priority: 2,
            });
        }

        // Cache hit rate recommendations
        let cache_hit_rate = if pool_stats.intern_operations > 0 {
            pool_stats.cache_hits as f64 / pool_stats.intern_operations as f64
        } else {
            0.0
        };

        if cache_hit_rate < 0.5 {
            recommendations.push(OptimizationRecommendation {
                recommendation_type: RecommendationType::UsageOptimization,
                description: "Low cache hit rate suggests many unique strings. Review string generation patterns.".to_string(),
                estimated_impact: "Could improve cache hit rate to 70-80%".to_string(),
                priority: 3,
            });
        }

        recommendations
    }
}

impl Default for StringPoolMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Global string pool monitor instance
static GLOBAL_MONITOR: std::sync::OnceLock<StringPoolMonitor> = std::sync::OnceLock::new();

/// Get the global string pool monitor
pub fn get_string_pool_monitor() -> &'static StringPoolMonitor {
    GLOBAL_MONITOR.get_or_init(|| StringPoolMonitor::new())
}

/// Record an intern operation for monitoring
pub fn record_intern_operation(duration_ns: u64) {
    get_string_pool_monitor().record_intern_operation(duration_ns);
}

/// Get comprehensive string pool monitoring statistics
pub fn get_string_pool_monitor_stats() -> StringPoolMonitorStats {
    get_string_pool_monitor().get_stats()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = StringPoolMonitor::new();
        let stats = monitor.get_stats();

        assert_eq!(stats.performance.avg_intern_time_ns, 0.0);
        assert_eq!(stats.performance.peak_ops_per_second, 0.0);
    }

    #[test]
    fn test_performance_tracking() {
        let monitor = StringPoolMonitor::new();

        // Record some operations
        monitor.record_intern_operation(100);
        monitor.record_intern_operation(200);
        monitor.record_intern_operation(150);

        let stats = monitor.get_stats();
        assert_eq!(stats.performance.avg_intern_time_ns, 150.0);
        assert_eq!(stats.performance.total_intern_time_ns, 450);
    }

    #[test]
    fn test_memory_efficiency_calculation() {
        let monitor = StringPoolMonitor::new();

        let pool_stats = StringPoolStats {
            unique_strings: 100,
            intern_operations: 1000,
            cache_hits: 900,
            memory_saved_bytes: 5000,
            average_string_length: 20.0,
        };

        let efficiency = monitor.calculate_memory_efficiency(&pool_stats);

        // Should show good efficiency due to high cache hit rate
        assert!(efficiency.efficiency_ratio > 0.5);
        assert!(efficiency.memory_without_interning_bytes > efficiency.memory_with_interning_bytes);
    }

    #[test]
    fn test_recommendations_generation() {
        let monitor = StringPoolMonitor::new();

        let pool_stats = StringPoolStats {
            unique_strings: 1000,
            intern_operations: 1000, // Low cache hit rate
            cache_hits: 100,
            memory_saved_bytes: 1000,
            average_string_length: 50.0,
        };

        let performance = PerformanceMetrics {
            avg_intern_time_ns: 500.0, // Good performance
            peak_ops_per_second: 1000.0,
            current_ops_per_second: 100.0,
            total_intern_time_ns: 500000,
        };

        let memory_efficiency = MemoryEfficiencyMetrics {
            efficiency_ratio: 0.2, // Poor efficiency
            memory_without_interning_bytes: 50000,
            memory_with_interning_bytes: 40000,
            pool_overhead_bytes: 5000,
        };

        let recommendations =
            monitor.generate_recommendations(&pool_stats, &performance, &memory_efficiency);

        // Should generate recommendations for poor efficiency and low cache hit rate
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| matches!(
            r.recommendation_type,
            RecommendationType::MemoryOptimization
        )));
        assert!(recommendations
            .iter()
            .any(|r| matches!(r.recommendation_type, RecommendationType::UsageOptimization)));
    }
}
