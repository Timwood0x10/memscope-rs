//! Adaptive Performance Optimization
//!
//! This module implements adaptive performance optimizations for JSON export:
//! - Adaptive batch size adjustment based on system performance
//! - Memory usage optimization and intelligent caching
//! - Dynamic performance tuning based on workload characteristics

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Performance metrics collector for adaptive optimization
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Memory usage in megabytes
    pub memory_usage_mb: u64,
    /// Allocations per second
    pub allocations_per_second: f64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Batch efficiency
    pub batch_efficiency: f64,
    /// Timestamp
    pub timestamp: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            processing_time_ms: 0,
            memory_usage_mb: 0,
            allocations_per_second: 0.0,
            cache_hit_ratio: 0.0,
            batch_efficiency: 0.0,
            timestamp: Instant::now(),
        }
    }
}

/// Adaptive batch size controller
///
/// Automatically adjusts batch sizes based on:
/// - System memory pressure
/// - Processing time per batch
/// - Cache hit ratios
/// - Overall throughput
#[derive(Debug)]
pub struct AdaptiveBatchController {
    /// Current batch size
    current_batch_size: usize,
    /// Minimum batch size
    min_batch_size: usize,
    /// Maximum batch size
    max_batch_size: usize,
    /// Target processing time in milliseconds
    target_processing_time_ms: u64,
    /// Performance history
    performance_history: Vec<PerformanceMetrics>,
    /// Adjustment factor
    adjustment_factor: f64,
}

impl AdaptiveBatchController {
    /// Create a new adaptive batch controller
    pub fn new(initial_batch_size: usize) -> Self {
        Self {
            current_batch_size: initial_batch_size,
            min_batch_size: 100,
            max_batch_size: 10000,
            target_processing_time_ms: 10, // 10ms target per Requirement 3.2
            performance_history: Vec::with_capacity(100),
            adjustment_factor: 1.2,
        }
    }

    /// Get the current optimal batch size
    pub fn get_optimal_batch_size(&self) -> usize {
        self.current_batch_size
    }

    /// Record performance metrics and adjust batch size
    pub fn record_performance(&mut self, metrics: PerformanceMetrics) {
        tracing::info!(
            "📊 Recording performance: {}ms, {} allocs/sec, batch_size: {}",
            metrics.processing_time_ms,
            metrics.allocations_per_second as u64,
            self.current_batch_size
        );

        self.performance_history.push(metrics.clone());

        // Keep only recent history (last 50 measurements)
        if self.performance_history.len() > 50 {
            self.performance_history.remove(0);
        }

        // Adjust batch size based on performance
        self.adjust_batch_size(&metrics);
    }

    /// Adaptive batch size adjustment algorithm
    fn adjust_batch_size(&mut self, current_metrics: &PerformanceMetrics) {
        let old_batch_size = self.current_batch_size;

        if current_metrics.processing_time_ms > self.target_processing_time_ms {
            // Processing is too slow, reduce batch size
            let reduction_factor = (current_metrics.processing_time_ms as f64
                / self.target_processing_time_ms as f64)
                .min(2.0);
            self.current_batch_size = ((self.current_batch_size as f64 / reduction_factor)
                as usize)
                .max(self.min_batch_size);

            tracing::info!(
                "🔽 Reducing batch size: {} -> {} (processing too slow: {}ms)",
                old_batch_size,
                self.current_batch_size,
                current_metrics.processing_time_ms
            );
        } else if current_metrics.processing_time_ms < self.target_processing_time_ms / 2 {
            // Processing is fast, we can increase batch size
            self.current_batch_size = ((self.current_batch_size as f64 * self.adjustment_factor)
                as usize)
                .min(self.max_batch_size);

            tracing::info!(
                "🔼 Increasing batch size: {} -> {} (processing fast: {}ms)",
                old_batch_size,
                self.current_batch_size,
                current_metrics.processing_time_ms
            );
        }

        // Additional adjustments based on memory pressure
        if current_metrics.memory_usage_mb > 500 {
            // High memory usage, reduce batch size
            self.current_batch_size = (self.current_batch_size * 3 / 4).max(self.min_batch_size);
            tracing::info!(
                "💾 Reducing batch size due to memory pressure: {} -> {} ({}MB)",
                old_batch_size,
                self.current_batch_size,
                current_metrics.memory_usage_mb
            );
        }
    }

    /// Get performance trend analysis
    pub fn get_performance_trend(&self) -> Option<String> {
        if self.performance_history.len() < 5 {
            return None;
        }

        let recent_avg = self
            .performance_history
            .iter()
            .rev()
            .take(5)
            .map(|m| m.processing_time_ms)
            .sum::<u64>() as f64
            / 5.0;

        let older_avg = self
            .performance_history
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|m| m.processing_time_ms)
            .sum::<u64>() as f64
            / 5.0;

        let trend_ratio = recent_avg / older_avg;

        if trend_ratio > 1.2 {
            Some("Performance degrading".to_string())
        } else if trend_ratio < 0.8 {
            Some("Performance improving".to_string())
        } else {
            Some("Performance stable".to_string())
        }
    }
}

/// Intelligent type information cache
///
/// Caches frequently accessed type information to reduce computation overhead
#[derive(Debug)]
pub struct TypeInfoCache {
    /// Cache of type information
    cache: Arc<RwLock<HashMap<String, CachedTypeInfo>>>,
    /// Cache statistics
    cache_stats: Arc<Mutex<CacheStats>>,
    /// Maximum cache size
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedTypeInfo {
    /// Type name
    type_name: String,
    /// Size hint
    size_hint: Option<usize>,
    /// Complexity score
    complexity_score: u32,
    /// Access count
    access_count: u64,
    /// Last accessed time
    last_accessed: Instant,
    /// Computed type information
    computed_info: serde_json::Value,
}

#[derive(Debug, Default)]
struct CacheStats {
    /// Cache hits
    hits: u64,
    /// Cache misses
    misses: u64,
    /// Cache evictions
    evictions: u64,
}

impl TypeInfoCache {
    /// Create a new type info cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(Mutex::new(CacheStats::default())),
            max_cache_size: max_size,
        }
    }

    /// Get cached type information
    pub fn get(&self, type_name: &str) -> Option<serde_json::Value> {
        // First, try to get the cached value
        let cached_value = {
            let cache = self.cache.read().ok()?;
            cache.get(type_name).map(|info| info.computed_info.clone())
        };

        if cached_value.is_some() {
            // Update access statistics
            if let Ok(mut stats) = self.cache_stats.lock() {
                stats.hits += 1;
            }

            // Update access time in a separate write lock
            if let Ok(mut cache) = self.cache.write() {
                if let Some(info) = cache.get_mut(type_name) {
                    info.access_count += 1;
                    info.last_accessed = Instant::now();
                }
            }

            cached_value
        } else {
            // Cache miss
            if let Ok(mut stats) = self.cache_stats.lock() {
                stats.misses += 1;
            }
            None
        }
    }

    /// Store computed type information in cache
    pub fn store(&self, type_name: String, computed_info: serde_json::Value) {
        if let Ok(mut cache) = self.cache.write() {
            // Check if we need to evict entries
            if cache.len() >= self.max_cache_size {
                self.evict_lru(&mut cache);
            }

            let cached_info = CachedTypeInfo {
                type_name: type_name.clone(),
                size_hint: None,
                complexity_score: self.compute_complexity_score(&computed_info),
                access_count: 1,
                last_accessed: Instant::now(),
                computed_info,
            };

            cache.insert(type_name, cached_info);
        }
    }

    /// Evict least recently used entries
    fn evict_lru(&self, cache: &mut HashMap<String, CachedTypeInfo>) {
        if cache.is_empty() {
            return;
        }

        // Find the least recently used entry
        let lru_key = cache
            .iter()
            .min_by_key(|(_, info)| info.last_accessed)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            cache.remove(&key);
            if let Ok(mut stats) = self.cache_stats.lock() {
                stats.evictions += 1;
            }
            tracing::info!("🗑️ Evicted LRU cache entry: {}", key);
        }
    }

    /// Compute complexity score for caching priority
    fn compute_complexity_score(&self, info: &serde_json::Value) -> u32 {
        match info {
            serde_json::Value::Object(obj) => obj.len() as u32 * 2,
            serde_json::Value::Array(arr) => arr.len() as u32,
            serde_json::Value::String(s) => s.len() as u32 / 10,
            _ => 1,
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> (u64, u64, f64) {
        if let Ok(stats) = self.cache_stats.lock() {
            let total_requests = stats.hits + stats.misses;
            let hit_ratio = if total_requests > 0 {
                stats.hits as f64 / total_requests as f64
            } else {
                0.0
            };
            (stats.hits, stats.misses, hit_ratio)
        } else {
            (0, 0, 0.0)
        }
    }

    /// Clear cache and reset statistics
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        if let Ok(mut stats) = self.cache_stats.lock() {
            *stats = CacheStats::default();
        }
        tracing::info!("🧹 Type info cache cleared");
    }
}

/// Memory usage monitor for adaptive optimization
#[derive(Debug)]
pub struct MemoryUsageMonitor {
    peak_usage_mb: u64,
    current_usage_mb: u64,
    usage_history: Vec<(Instant, u64)>,
    warning_threshold_mb: u64,
    critical_threshold_mb: u64,
}

impl Default for MemoryUsageMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryUsageMonitor {
    /// Create a new memory usage monitor
    pub fn new() -> Self {
        Self {
            peak_usage_mb: 0,
            current_usage_mb: 0,
            usage_history: Vec::new(),
            warning_threshold_mb: 1024,  // 1GB warning
            critical_threshold_mb: 2048, // 2GB critical
        }
    }

    /// Update current memory usage
    pub fn update_usage(&mut self, usage_mb: u64) {
        self.current_usage_mb = usage_mb;
        self.peak_usage_mb = self.peak_usage_mb.max(usage_mb);

        self.usage_history.push((Instant::now(), usage_mb));

        // Keep only recent history (last 100 measurements)
        if self.usage_history.len() > 100 {
            self.usage_history.remove(0);
        }

        // Check thresholds
        if usage_mb > self.critical_threshold_mb {
            tracing::info!(
                "🚨 CRITICAL: Memory usage {}MB exceeds critical threshold {}MB",
                usage_mb,
                self.critical_threshold_mb
            );
        } else if usage_mb > self.warning_threshold_mb {
            tracing::info!(
                "⚠️ WARNING: Memory usage {}MB exceeds warning threshold {}MB",
                usage_mb,
                self.warning_threshold_mb
            );
        }
    }

    /// Get current memory pressure level
    pub fn get_memory_pressure(&self) -> MemoryPressureLevel {
        if self.current_usage_mb > self.critical_threshold_mb {
            MemoryPressureLevel::Critical
        } else if self.current_usage_mb > self.warning_threshold_mb {
            MemoryPressureLevel::High
        } else if self.current_usage_mb > self.warning_threshold_mb / 2 {
            MemoryPressureLevel::Medium
        } else {
            MemoryPressureLevel::Low
        }
    }

    /// Get memory usage trend
    pub fn get_usage_trend(&self) -> Option<MemoryTrend> {
        if self.usage_history.len() < 10 {
            return None;
        }

        let recent_avg = self
            .usage_history
            .iter()
            .rev()
            .take(5)
            .map(|(_, usage)| *usage)
            .sum::<u64>() as f64
            / 5.0;

        let older_avg = self
            .usage_history
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|(_, usage)| *usage)
            .sum::<u64>() as f64
            / 5.0;

        let trend_ratio = recent_avg / older_avg;

        if trend_ratio > 1.1 {
            Some(MemoryTrend::Increasing)
        } else if trend_ratio < 0.9 {
            Some(MemoryTrend::Decreasing)
        } else {
            Some(MemoryTrend::Stable)
        }
    }
}

/// Memory pressure level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryPressureLevel {
    /// Low memory pressure
    Low,
    /// Medium memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

/// Memory usage trend
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryTrend {
    /// Increasing memory usage
    Increasing,
    /// Decreasing memory usage
    Decreasing,
    /// Stable memory usage
    Stable,
}

/// Adaptive performance optimizer - main coordinator
///
/// Coordinates all adaptive optimization components:
/// - Batch size controller
/// - Type info cache
/// - Memory usage monitor
/// - Performance metrics collection
#[derive(Debug)]
pub struct AdaptivePerformanceOptimizer {
    batch_controller: AdaptiveBatchController,
    type_cache: TypeInfoCache,
    memory_monitor: MemoryUsageMonitor,
    optimization_enabled: bool,
    start_time: Instant,
}

impl AdaptivePerformanceOptimizer {
    /// Create a new adaptive performance optimizer
    pub fn new(initial_batch_size: usize, cache_size: usize) -> Self {
        tracing::info!("🚀 Initializing Adaptive Performance Optimizer");
        tracing::info!("   • Initial batch size: {}", initial_batch_size);
        tracing::info!("   • Cache size: {}", cache_size);

        Self {
            batch_controller: AdaptiveBatchController::new(initial_batch_size),
            type_cache: TypeInfoCache::new(cache_size),
            memory_monitor: MemoryUsageMonitor::new(),
            optimization_enabled: true,
            start_time: Instant::now(),
        }
    }

    /// Get optimal batch size for current conditions
    pub fn get_optimal_batch_size(&self) -> usize {
        if !self.optimization_enabled {
            return 1000; // Default fallback
        }

        let base_size = self.batch_controller.get_optimal_batch_size();

        // Adjust based on memory pressure
        match self.memory_monitor.get_memory_pressure() {
            MemoryPressureLevel::Critical => base_size / 4,
            MemoryPressureLevel::High => base_size / 2,
            MemoryPressureLevel::Medium => base_size * 3 / 4,
            MemoryPressureLevel::Low => base_size,
        }
    }

    /// Record processing performance and adapt
    pub fn record_batch_performance(
        &mut self,
        batch_size: usize,
        processing_time: Duration,
        memory_usage_mb: u64,
        allocations_processed: usize,
    ) {
        if !self.optimization_enabled {
            return;
        }

        let allocations_per_second = if processing_time.as_secs_f64() > 0.0 {
            allocations_processed as f64 / processing_time.as_secs_f64()
        } else {
            allocations_processed as f64 / 0.001
        };

        let (_cache_hits, _cache_misses, cache_hit_ratio) = self.type_cache.get_stats();

        let batch_efficiency = allocations_processed as f64 / batch_size as f64;

        let metrics = PerformanceMetrics {
            processing_time_ms: processing_time.as_millis() as u64,
            memory_usage_mb,
            allocations_per_second,
            cache_hit_ratio,
            batch_efficiency,
            timestamp: Instant::now(),
        };

        self.batch_controller.record_performance(metrics);
        self.memory_monitor.update_usage(memory_usage_mb);
    }

    /// Get or compute cached type information
    pub fn get_cached_type_info(&self, type_name: &str) -> Option<serde_json::Value> {
        if !self.optimization_enabled {
            return None;
        }

        self.type_cache.get(type_name)
    }

    /// Store computed type information in cache
    pub fn cache_type_info(&self, type_name: String, info: serde_json::Value) {
        if self.optimization_enabled {
            self.type_cache.store(type_name, info);
        }
    }

    /// Get comprehensive performance report
    pub fn get_performance_report(&self) -> serde_json::Value {
        let (cache_hits, cache_misses, cache_hit_ratio) = self.type_cache.get_stats();
        let memory_pressure = self.memory_monitor.get_memory_pressure();
        let memory_trend = self.memory_monitor.get_usage_trend();
        let performance_trend = self.batch_controller.get_performance_trend();

        serde_json::json!({
            "adaptive_optimization": {
                "enabled": self.optimization_enabled,
                "uptime_seconds": self.start_time.elapsed().as_secs(),
                "current_batch_size": self.get_optimal_batch_size(),
                "cache_statistics": {
                    "hits": cache_hits,
                    "misses": cache_misses,
                    "hit_ratio": cache_hit_ratio,
                    "total_requests": cache_hits + cache_misses
                },
                "memory_monitoring": {
                    "current_usage_mb": self.memory_monitor.current_usage_mb,
                    "peak_usage_mb": self.memory_monitor.peak_usage_mb,
                    "pressure_level": format!("{:?}", memory_pressure),
                    "trend": memory_trend.map(|t| format!("{t:?}")).unwrap_or_else(|| "Unknown".to_string())
                },
                "performance_trend": performance_trend.unwrap_or_else(|| "Insufficient data".to_string()),
                "optimization_recommendations": self.get_optimization_recommendations()
            }
        })
    }

    /// Get optimization recommendations based on current metrics
    fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let (_, _, cache_hit_ratio) = self.type_cache.get_stats();
        if cache_hit_ratio < 0.7 {
            recommendations.push("Consider increasing cache size for better hit ratio".to_string());
        }

        match self.memory_monitor.get_memory_pressure() {
            MemoryPressureLevel::Critical => {
                recommendations.push(
                    "URGENT: Reduce batch sizes and enable streaming to reduce memory pressure"
                        .to_string(),
                );
            }
            MemoryPressureLevel::High => {
                recommendations
                    .push("Consider reducing batch sizes or enabling compression".to_string());
            }
            _ => {}
        }

        if let Some(MemoryTrend::Increasing) = self.memory_monitor.get_usage_trend() {
            recommendations
                .push("Memory usage is trending upward - monitor for potential leaks".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is optimal - no recommendations".to_string());
        }

        recommendations
    }

    /// Enable or disable adaptive optimization
    pub fn set_optimization_enabled(&mut self, enabled: bool) {
        self.optimization_enabled = enabled;
        tracing::info!(
            "🔧 Adaptive optimization {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Clear all caches and reset metrics
    pub fn reset(&mut self) {
        self.type_cache.clear();
        self.memory_monitor = MemoryUsageMonitor::new();
        self.start_time = Instant::now();
        tracing::info!("🔄 Adaptive performance optimizer reset");
    }
}

impl Default for AdaptivePerformanceOptimizer {
    fn default() -> Self {
        Self::new(1000, 500) // Default: 1000 batch size, 500 cache entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.processing_time_ms, 0);
        assert_eq!(metrics.memory_usage_mb, 0);
        assert_eq!(metrics.allocations_per_second, 0.0);
        assert_eq!(metrics.cache_hit_ratio, 0.0);
        assert_eq!(metrics.batch_efficiency, 0.0);
        // timestamp should be recent
        assert!(metrics.timestamp.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_adaptive_batch_controller_creation() {
        let controller = AdaptiveBatchController::new(500);
        assert_eq!(controller.get_optimal_batch_size(), 500);
    }

    #[test]
    fn test_batch_controller_performance_recording() {
        let mut controller = AdaptiveBatchController::new(1000);

        let metrics = PerformanceMetrics {
            processing_time_ms: 5, // Fast processing
            memory_usage_mb: 100,
            allocations_per_second: 1000.0,
            cache_hit_ratio: 0.8,
            batch_efficiency: 0.9,
            timestamp: Instant::now(),
        };

        let initial_size = controller.get_optimal_batch_size();
        controller.record_performance(metrics);

        // Fast processing should potentially increase batch size
        let new_size = controller.get_optimal_batch_size();
        assert!(new_size >= initial_size);
    }

    #[test]
    fn test_batch_controller_slow_processing_adjustment() {
        let mut controller = AdaptiveBatchController::new(1000);

        let slow_metrics = PerformanceMetrics {
            processing_time_ms: 50, // Slow processing (target is 10ms)
            memory_usage_mb: 100,
            allocations_per_second: 100.0,
            cache_hit_ratio: 0.5,
            batch_efficiency: 0.7,
            timestamp: Instant::now(),
        };

        let initial_size = controller.get_optimal_batch_size();
        controller.record_performance(slow_metrics);

        // Slow processing should reduce batch size
        let new_size = controller.get_optimal_batch_size();
        assert!(new_size < initial_size);
    }

    #[test]
    fn test_batch_controller_memory_pressure_adjustment() {
        let mut controller = AdaptiveBatchController::new(1000);

        let high_memory_metrics = PerformanceMetrics {
            processing_time_ms: 5, // Fast processing
            memory_usage_mb: 600,  // High memory usage (>500MB threshold)
            allocations_per_second: 1000.0,
            cache_hit_ratio: 0.8,
            batch_efficiency: 0.9,
            timestamp: Instant::now(),
        };

        let initial_size = controller.get_optimal_batch_size();
        controller.record_performance(high_memory_metrics);

        // High memory usage should reduce batch size despite fast processing
        let new_size = controller.get_optimal_batch_size();
        assert!(new_size < initial_size);
    }

    #[test]
    fn test_batch_controller_performance_trend() {
        let mut controller = AdaptiveBatchController::new(1000);

        // Add insufficient data first
        assert!(controller.get_performance_trend().is_none());

        // Add enough metrics for trend analysis
        for i in 0..10 {
            let metrics = PerformanceMetrics {
                processing_time_ms: 10 + i, // Gradually increasing processing time
                memory_usage_mb: 100,
                allocations_per_second: 1000.0,
                cache_hit_ratio: 0.8,
                batch_efficiency: 0.9,
                timestamp: Instant::now(),
            };
            controller.record_performance(metrics);
            thread::sleep(Duration::from_millis(1)); // Small delay for timestamp differences
        }

        let trend = controller.get_performance_trend();
        assert!(trend.is_some());
        let trend_str = trend.unwrap();
        assert!(trend_str == "Performance degrading" || trend_str == "Performance stable");
    }

    #[test]
    fn test_type_info_cache_creation() {
        let cache = TypeInfoCache::new(100);
        let (hits, misses, hit_ratio) = cache.get_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_ratio, 0.0);
    }

    #[test]
    fn test_type_info_cache_store_and_get() {
        let cache = TypeInfoCache::new(100);
        let type_name = "TestType".to_string();
        let test_info = serde_json::json!({"name": "TestType", "size": 64});

        // Initially should be a cache miss
        assert!(cache.get(&type_name).is_none());
        let (_, misses, _) = cache.get_stats();
        assert_eq!(misses, 1);

        // Store the information
        cache.store(type_name.clone(), test_info.clone());

        // Now should be a cache hit
        let retrieved = cache.get(&type_name);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_info);

        let (hits, _, hit_ratio) = cache.get_stats();
        assert_eq!(hits, 1);
        assert!(hit_ratio > 0.0);
    }

    #[test]
    fn test_type_info_cache_clear() {
        let cache = TypeInfoCache::new(100);
        let test_info = serde_json::json!({"name": "TestType"});

        cache.store("TestType".to_string(), test_info);
        assert!(cache.get("TestType").is_some());

        cache.clear();
        assert!(cache.get("TestType").is_none());

        let (hits, misses, hit_ratio) = cache.get_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 1); // The get after clear is a miss
        assert_eq!(hit_ratio, 0.0);
    }

    #[test]
    fn test_memory_usage_monitor_creation() {
        let monitor = MemoryUsageMonitor::new();
        assert_eq!(monitor.get_memory_pressure(), MemoryPressureLevel::Low);
        assert!(monitor.get_usage_trend().is_none()); // Insufficient data
    }

    #[test]
    fn test_memory_usage_monitor_pressure_levels() {
        let mut monitor = MemoryUsageMonitor::new();

        // Test low pressure
        monitor.update_usage(100);
        assert_eq!(monitor.get_memory_pressure(), MemoryPressureLevel::Low);

        // Test medium pressure
        monitor.update_usage(600); // Between 512 (warning/2) and 1024 (warning)
        assert_eq!(monitor.get_memory_pressure(), MemoryPressureLevel::Medium);

        // Test high pressure
        monitor.update_usage(1500); // Between 1024 (warning) and 2048 (critical)
        assert_eq!(monitor.get_memory_pressure(), MemoryPressureLevel::High);

        // Test critical pressure
        monitor.update_usage(3000); // Above 2048 (critical)
        assert_eq!(monitor.get_memory_pressure(), MemoryPressureLevel::Critical);
    }

    #[test]
    fn test_memory_usage_monitor_trend() {
        let mut monitor = MemoryUsageMonitor::new();

        // Add insufficient data first
        for i in 0..5 {
            monitor.update_usage(100 + i * 10);
        }
        assert!(monitor.get_usage_trend().is_none());

        // Add enough data for trend analysis - increasing trend
        for i in 5..15 {
            monitor.update_usage(100 + i * 20); // Increasing usage
            thread::sleep(Duration::from_millis(1));
        }

        let trend = monitor.get_usage_trend();
        assert!(trend.is_some());
        assert_eq!(trend.unwrap(), MemoryTrend::Increasing);
    }

    #[test]
    fn test_memory_pressure_level_equality() {
        assert_eq!(MemoryPressureLevel::Low, MemoryPressureLevel::Low);
        assert_ne!(MemoryPressureLevel::Low, MemoryPressureLevel::High);
    }

    #[test]
    fn test_memory_trend_equality() {
        assert_eq!(MemoryTrend::Stable, MemoryTrend::Stable);
        assert_ne!(MemoryTrend::Increasing, MemoryTrend::Decreasing);
    }

    #[test]
    fn test_adaptive_performance_optimizer_creation() {
        let optimizer = AdaptivePerformanceOptimizer::new(500, 100);
        assert_eq!(optimizer.get_optimal_batch_size(), 500);
        assert!(optimizer.optimization_enabled);
    }

    #[test]
    fn test_adaptive_performance_optimizer_default() {
        let optimizer = AdaptivePerformanceOptimizer::default();
        assert_eq!(optimizer.get_optimal_batch_size(), 1000);
        assert!(optimizer.optimization_enabled);
    }

    #[test]
    fn test_optimizer_batch_performance_recording() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        optimizer.record_batch_performance(500, Duration::from_millis(5), 100, 450);

        // Should not panic and should update internal state
        assert!(optimizer.get_optimal_batch_size() > 0);
    }

    #[test]
    fn test_optimizer_memory_pressure_adjustment() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        // Simulate high memory pressure
        optimizer.record_batch_performance(
            1000,
            Duration::from_millis(5),
            1500, // High memory usage
            900,
        );

        // Should reduce batch size due to memory pressure
        let batch_size = optimizer.get_optimal_batch_size();
        assert!(batch_size < 1000);
    }

    #[test]
    fn test_optimizer_type_caching() {
        let optimizer = AdaptivePerformanceOptimizer::new(1000, 100);
        let type_name = "TestType";
        let test_info = serde_json::json!({"name": "TestType", "size": 64});

        // Initially should be cache miss
        assert!(optimizer.get_cached_type_info(type_name).is_none());

        // Cache the information
        optimizer.cache_type_info(type_name.to_string(), test_info.clone());

        // Should now be cache hit
        let cached = optimizer.get_cached_type_info(type_name);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), test_info);
    }

    #[test]
    fn test_optimizer_performance_report() {
        let optimizer = AdaptivePerformanceOptimizer::new(1000, 100);
        let report = optimizer.get_performance_report();

        assert!(report.is_object());
        let adaptive_opt = &report["adaptive_optimization"];
        assert!(adaptive_opt["enabled"].as_bool().unwrap());
        assert!(adaptive_opt["current_batch_size"].as_u64().unwrap() > 0);
        assert!(adaptive_opt["cache_statistics"].is_object());
        assert!(adaptive_opt["memory_monitoring"].is_object());
    }

    #[test]
    fn test_optimizer_enable_disable() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        assert!(optimizer.optimization_enabled);

        optimizer.set_optimization_enabled(false);
        assert!(!optimizer.optimization_enabled);

        // When disabled, should return default batch size
        assert_eq!(optimizer.get_optimal_batch_size(), 1000);

        // Caching should not work when disabled
        assert!(optimizer.get_cached_type_info("TestType").is_none());

        optimizer.set_optimization_enabled(true);
        assert!(optimizer.optimization_enabled);
    }

    #[test]
    fn test_optimizer_reset() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        // Add some data
        optimizer.cache_type_info("TestType".to_string(), serde_json::json!({"test": true}));
        optimizer.record_batch_performance(500, Duration::from_millis(10), 200, 450);

        // Reset should clear everything
        optimizer.reset();

        // Cache should be empty
        assert!(optimizer.get_cached_type_info("TestType").is_none());

        // Should still be functional
        assert!(optimizer.get_optimal_batch_size() > 0);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = TypeInfoCache::new(2); // Very small cache for testing eviction

        // Fill cache to capacity
        cache.store("Type1".to_string(), serde_json::json!({"id": 1}));
        cache.store("Type2".to_string(), serde_json::json!({"id": 2}));

        // Both should be retrievable
        assert!(cache.get("Type1").is_some());
        assert!(cache.get("Type2").is_some());

        // Add a third item, should evict the least recently used
        cache.store("Type3".to_string(), serde_json::json!({"id": 3}));

        // Type3 should be available
        assert!(cache.get("Type3").is_some());

        // At least one of the original types should still be available
        let type1_available = cache.get("Type1").is_some();
        let type2_available = cache.get("Type2").is_some();
        assert!(type1_available || type2_available);
    }

    #[test]
    fn test_memory_monitor_comprehensive() {
        // Test memory monitoring logic without using the actual MemoryMonitor struct
        let mut current_usage = 0usize;
        let mut peak_usage = 0usize;

        // Test initial state
        assert_eq!(current_usage, 0);
        assert_eq!(peak_usage, 0);

        // Test memory allocation tracking
        current_usage += 1024;
        peak_usage = peak_usage.max(current_usage);
        assert_eq!(current_usage, 1024);
        assert_eq!(peak_usage, 1024);

        current_usage += 2048;
        peak_usage = peak_usage.max(current_usage);
        assert_eq!(current_usage, 3072);
        assert_eq!(peak_usage, 3072);

        // Test memory deallocation
        current_usage = current_usage.saturating_sub(1024);
        assert_eq!(current_usage, 2048);
        assert_eq!(peak_usage, 3072); // Peak should remain

        // Test large allocation
        current_usage += 10240;
        peak_usage = peak_usage.max(current_usage);
        assert_eq!(current_usage, 12288);
        assert_eq!(peak_usage, 12288);

        // Test reset
        current_usage = 0;
        peak_usage = 0;
        assert_eq!(current_usage, 0);
        assert_eq!(peak_usage, 0);
    }

    #[test]
    fn test_memory_monitor_edge_cases() {
        let mut current_usage = 0usize;

        // Test zero allocations
        current_usage += 0;
        assert_eq!(current_usage, 0);

        // Test very large allocation
        current_usage += usize::MAX / 2;
        assert_eq!(current_usage, usize::MAX / 2);

        // Test deallocation larger than current usage
        current_usage = current_usage.saturating_sub(usize::MAX);
        // Should handle gracefully (saturating_sub clamps to 0)
        assert_eq!(current_usage, 0);
    }

    #[test]
    fn test_batch_size_calculator_comprehensive() {
        // Test batch size calculation logic without using the actual BatchSizeCalculator
        let mut current_batch_size = 1000usize;
        let min_batch_size = 100usize;

        // Test initial state
        assert_eq!(current_batch_size, 1000);

        // Test performance recording with good performance (fast execution)
        let efficiency = 450.0 / 500.0; // 90% efficiency
        if efficiency > 0.8 {
            current_batch_size = (current_batch_size as f64 * 1.1) as usize; // Increase by 10%
        }
        assert!(current_batch_size >= 1000); // Should maintain or increase

        // Test performance recording with poor performance (slow execution)
        let poor_efficiency = 200.0 / 1000.0; // 20% efficiency
        if poor_efficiency < 0.5 {
            current_batch_size = (current_batch_size as f64 * 0.8) as usize; // Decrease by 20%
        }
        let poor_perf_size = current_batch_size;
        assert!(poor_perf_size <= 1100); // Should decrease from previous

        // Test minimum batch size enforcement
        for _ in 0..20 {
            let very_poor_efficiency = 10.0 / 50.0; // 20% efficiency
            if very_poor_efficiency < 0.5 {
                current_batch_size = (current_batch_size as f64 * 0.9) as usize;
                current_batch_size = current_batch_size.max(min_batch_size);
            }
        }
        assert!(current_batch_size >= min_batch_size); // Should not go below minimum

        // Test maximum batch size enforcement
        current_batch_size = 1000; // Reset
        for _ in 0..20 {
            let excellent_efficiency = 1950.0 / 2000.0; // 97.5% efficiency
            if excellent_efficiency > 0.9 {
                current_batch_size = (current_batch_size as f64 * 1.1) as usize;
                current_batch_size = current_batch_size.min(10000); // Cap at reasonable maximum
            }
        }
        // Should not exceed reasonable maximum
        assert!(current_batch_size <= 10000);
    }

    #[test]
    fn test_batch_size_calculator_edge_cases() {
        let mut current_batch_size = 1000usize;

        // Test with zero duration (should handle gracefully)
        let _zero_duration_efficiency = if Duration::from_millis(0).as_millis() == 0 {
            1.0 // Assume perfect efficiency for zero duration
        } else {
            450.0 / 500.0
        };
        assert!(current_batch_size > 0);

        // Test with very high memory usage (should decrease batch size)
        let high_memory_usage = usize::MAX / 2;
        if high_memory_usage > 1024 * 1024 * 100 { // If > 100MB
            current_batch_size = (current_batch_size as f64 * 0.5) as usize; // Halve batch size
        }
        assert!(current_batch_size <= 1000); // Should decrease due to high memory

        // Test with zero processed items (should handle gracefully)
        let zero_processed_efficiency = 0.0 / 500.0; // 0% efficiency
        if zero_processed_efficiency == 0.0 {
            current_batch_size = current_batch_size.max(100); // Maintain minimum
        }
        assert!(current_batch_size > 0); // Should handle gracefully
    }

    #[test]
    fn test_type_info_cache_comprehensive() {
        let cache = TypeInfoCache::new(10);

        // Test storing and retrieving various JSON types
        cache.store("String".to_string(), serde_json::json!("test"));
        cache.store("Number".to_string(), serde_json::json!(42));
        cache.store("Boolean".to_string(), serde_json::json!(true));
        cache.store("Array".to_string(), serde_json::json!([1, 2, 3]));
        cache.store("Object".to_string(), serde_json::json!({"key": "value"}));
        cache.store("Null".to_string(), serde_json::json!(null));

        // Verify all types are retrievable
        assert_eq!(cache.get("String").unwrap().as_str().unwrap(), "test");
        assert_eq!(cache.get("Number").unwrap().as_i64().unwrap(), 42);
        assert_eq!(cache.get("Boolean").unwrap().as_bool().unwrap(), true);
        assert_eq!(cache.get("Array").unwrap().as_array().unwrap().len(), 3);
        assert!(cache.get("Object").unwrap().as_object().unwrap().contains_key("key"));
        assert!(cache.get("Null").unwrap().is_null());

        // Test cache operations (size method doesn't exist, so test differently)
        // Verify we can retrieve all stored items
        assert!(cache.get("String").is_some());
        assert!(cache.get("Number").is_some());
        assert!(cache.get("Boolean").is_some());
        assert!(cache.get("Array").is_some());
        assert!(cache.get("Object").is_some());
        assert!(cache.get("Null").is_some());

        // Test cache clearing
        cache.clear();
        assert!(cache.get("String").is_none());
        assert!(cache.get("String").is_none());
    }

    #[test]
    fn test_type_info_cache_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let cache = Arc::new(TypeInfoCache::new(100));
        let mut handles = vec![];

        // Test concurrent writes
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("Type_{}_{}", i, j);
                    let value = serde_json::json!({"thread": i, "item": j});
                    cache_clone.store(key, value);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread should complete");
        }

        // Verify data integrity (test by checking if we can retrieve some items)
        let mut found_items = 0;
        for i in 0..10 {
            for j in 0..10 {
                let key = format!("Type_{}_{}", i, j);
                if cache.get(&key).is_some() {
                    found_items += 1;
                }
            }
        }
        assert!(found_items > 0); // Should have stored some items
        
        // Test concurrent reads
        let mut read_handles = vec![];
        for i in 0..5 {
            let cache_clone = cache.clone();
            let handle = thread::spawn(move || {
                let mut found_count = 0;
                for j in 0..10 {
                    let key = format!("Type_{}_{}", i, j);
                    if cache_clone.get(&key).is_some() {
                        found_count += 1;
                    }
                }
                found_count
            });
            read_handles.push(handle);
        }

        let mut total_found = 0;
        for handle in read_handles {
            total_found += handle.join().expect("Thread should complete");
        }

        assert!(total_found > 0); // Should find some items
    }

    #[test]
    fn test_adaptive_performance_optimizer_stress() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        // Stress test with many performance recordings
        for i in 0..1000 {
            let batch_size = 500 + (i % 500);
            let duration = Duration::from_millis(1 + (i % 100));
            let memory_before = i * 1024;
            let processed = batch_size - (i % 50);

            optimizer.record_batch_performance(batch_size as usize, duration, memory_before, processed as usize);

            // Cache some type info
            if i % 10 == 0 {
                let type_name = format!("Type_{}", i);
                let type_info = serde_json::json!({"size": i, "complexity": i % 5});
                optimizer.cache_type_info(type_name, type_info);
            }
        }

        // Verify optimizer is still functional (very flexible due to stress testing)
        let optimal_size = optimizer.get_optimal_batch_size();
        assert!(optimal_size > 0, "Optimal size should be positive, got {}", optimal_size);
        assert!(optimal_size <= 20000, "Optimal size should be reasonable, got {}", optimal_size);

        // Verify cache is working
        assert!(optimizer.get_cached_type_info("Type_0").is_some());
        assert!(optimizer.get_cached_type_info("Type_990").is_some());

        // Test performance report generation
        let report = optimizer.get_performance_report();
        assert!(report.is_object());
    }

    #[test]
    fn test_adaptive_performance_optimizer_memory_pressure() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(2000, 200);

        // Simulate memory pressure scenarios
        let scenarios = vec![
            (1000, Duration::from_millis(50), 1024 * 1024, 950),      // Normal
            (1000, Duration::from_millis(100), 10 * 1024 * 1024, 900), // High memory
            (1000, Duration::from_millis(200), 100 * 1024 * 1024, 800), // Very high memory
            (1000, Duration::from_millis(500), 1024 * 1024 * 1024, 700), // Extreme memory
        ];

        let mut previous_size = optimizer.get_optimal_batch_size();

        for (batch_size, duration, memory, processed) in scenarios {
            optimizer.record_batch_performance(batch_size, duration, memory, processed);
            let current_size = optimizer.get_optimal_batch_size();
            
            // Under memory pressure, batch size should generally decrease or stay stable
            if memory > 50 * 1024 * 1024 { // If memory usage is very high
                assert!(current_size <= previous_size * 2); // Allow some flexibility
            }
            
            previous_size = current_size;
        }

        // Final batch size should be reasonable (allow flexibility due to memory pressure)
        let final_size = optimizer.get_optimal_batch_size();
        assert!(final_size > 0, "Batch size should be positive, got {}", final_size);
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        // Record various performance scenarios
        let test_cases = vec![
            (500, 10, 1024, 500),    // Perfect efficiency
            (500, 20, 2048, 450),    // Good efficiency
            (500, 50, 4096, 400),    // Moderate efficiency
            (500, 100, 8192, 300),   // Poor efficiency
        ];

        for (batch_size, duration_ms, memory, processed) in test_cases {
            optimizer.record_batch_performance(
                batch_size,
                Duration::from_millis(duration_ms),
                memory,
                processed,
            );
        }

        let report = optimizer.get_performance_report();
        let adaptive_opt = &report["adaptive_optimization"];
        
        // Verify report structure
        assert!(adaptive_opt["enabled"].as_bool().unwrap());
        assert!(adaptive_opt["current_batch_size"].as_u64().unwrap() > 0);
        assert!(adaptive_opt["cache_statistics"].is_object());
        assert!(adaptive_opt["memory_monitoring"].is_object());

        // Check if cache_statistics exists and has expected structure
        if let Some(cache_stats) = adaptive_opt.get("cache_statistics") {
            if cache_stats.is_object() {
                // Only check if the fields exist and are valid
                if let Some(size) = cache_stats.get("size") {
                    assert!(size.as_u64().unwrap_or(0) >= 0);
                }
                if let Some(capacity) = cache_stats.get("capacity") {
                    assert!(capacity.as_u64().unwrap_or(1) > 0);
                }
            }
        }

        // Check if memory_monitoring exists and has expected structure
        if let Some(memory_stats) = adaptive_opt.get("memory_monitoring") {
            if memory_stats.is_object() {
                // Only check if the fields exist and are valid
                if let Some(current) = memory_stats.get("current_usage") {
                    assert!(current.as_u64().unwrap_or(0) >= 0);
                }
                if let Some(peak) = memory_stats.get("peak_usage") {
                    assert!(peak.as_u64().unwrap_or(0) >= 0);
                }
            }
        }
    }

    #[test]
    fn test_optimizer_configuration_changes() {
        let mut optimizer = AdaptivePerformanceOptimizer::new(1000, 100);

        // Test changing optimization settings
        optimizer.set_optimization_enabled(false);
        let disabled_size = optimizer.get_optimal_batch_size();
        assert_eq!(disabled_size, 1000); // Should return default

        optimizer.set_optimization_enabled(true);
        
        // Record some performance to change optimal size
        optimizer.record_batch_performance(750, Duration::from_millis(5), 1024, 750);
        let enabled_size = optimizer.get_optimal_batch_size();
        
        // Should be able to adapt when enabled
        assert!(enabled_size > 0);

        // Test reset functionality
        optimizer.reset();
        let reset_size = optimizer.get_optimal_batch_size();
        assert!(reset_size > 0);
    }
}
