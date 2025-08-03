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
                old_batch_size, self.current_batch_size, current_metrics.processing_time_ms
            );
        } else if current_metrics.processing_time_ms < self.target_processing_time_ms / 2 {
            // Processing is fast, we can increase batch size
            self.current_batch_size = ((self.current_batch_size as f64 * self.adjustment_factor)
                as usize)
                .min(self.max_batch_size);

            tracing::info!(
                "🔼 Increasing batch size: {} -> {} (processing fast: {}ms)",
                old_batch_size, self.current_batch_size, current_metrics.processing_time_ms
            );
        }

        // Additional adjustments based on memory pressure
        if current_metrics.memory_usage_mb > 500 {
            // High memory usage, reduce batch size
            self.current_batch_size = (self.current_batch_size * 3 / 4).max(self.min_batch_size);
            tracing::info!(
                "💾 Reducing batch size due to memory pressure: {} -> {} ({}MB)",
                old_batch_size, self.current_batch_size, current_metrics.memory_usage_mb
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
                usage_mb, self.critical_threshold_mb
            );
        } else if usage_mb > self.warning_threshold_mb {
            tracing::info!(
                "⚠️ WARNING: Memory usage {}MB exceeds warning threshold {}MB",
                usage_mb, self.warning_threshold_mb
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
                    "trend": memory_trend.map(|t| format!("{:?}", t)).unwrap_or_else(|| "Unknown".to_string())
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
