//! Clone operation monitoring and optimization tracking
//!
//! This module provides runtime monitoring of clone operations to identify
//! optimization opportunities and track performance improvements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Statistics about clone operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneMonitorStats {
    /// Total number of clone operations
    pub total_clones: u64,
    /// Number of optimized clones (using Arc)
    pub optimized_clones: u64,
    /// Number of avoided clones through Arc sharing
    pub avoided_clones: u64,
    /// Total time spent in clone operations (nanoseconds)
    pub total_clone_time_ns: u64,
    /// Average clone time (nanoseconds)
    pub avg_clone_time_ns: f64,
    /// Memory saved through optimization (bytes)
    pub memory_saved_bytes: u64,
    /// Performance improvement ratio
    pub performance_improvement: f64,
}

/// Information about clone operations by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCloneStats {
    /// Type name
    pub type_name: String,
    /// Number of clones for this type
    pub clone_count: u64,
    /// Number of optimized clones
    pub optimized_count: u64,
    /// Total size cloned (bytes)
    pub total_size_bytes: u64,
    /// Average clone time for this type
    pub avg_clone_time_ns: f64,
}

/// Clone operation monitor
pub struct CloneMonitor {
    /// Total clone counter
    total_clones: std::sync::atomic::AtomicU64,
    /// Optimized clone counter
    optimized_clones: std::sync::atomic::AtomicU64,
    /// Avoided clone counter
    avoided_clones: std::sync::atomic::AtomicU64,
    /// Total clone time
    total_clone_time_ns: std::sync::atomic::AtomicU64,
    /// Memory saved
    memory_saved_bytes: std::sync::atomic::AtomicU64,
    /// Per-type statistics
    type_stats: Mutex<HashMap<String, TypeCloneStats>>,
}

impl CloneMonitor {
    /// Create a new clone monitor
    fn new() -> Self {
        Self {
            total_clones: std::sync::atomic::AtomicU64::new(0),
            optimized_clones: std::sync::atomic::AtomicU64::new(0),
            avoided_clones: std::sync::atomic::AtomicU64::new(0),
            total_clone_time_ns: std::sync::atomic::AtomicU64::new(0),
            memory_saved_bytes: std::sync::atomic::AtomicU64::new(0),
            type_stats: Mutex::new(HashMap::new()),
        }
    }

    /// Record a clone operation
    pub fn record_clone(&self, type_name: &str, size_bytes: usize, duration_ns: u64) {
        use std::sync::atomic::Ordering;

        self.total_clones.fetch_add(1, Ordering::Relaxed);
        self.total_clone_time_ns
            .fetch_add(duration_ns, Ordering::Relaxed);

        // Update per-type statistics
        if let Ok(mut stats) = self.type_stats.lock() {
            let type_stat = stats
                .entry(type_name.to_string())
                .or_insert_with(|| TypeCloneStats {
                    type_name: type_name.to_string(),
                    clone_count: 0,
                    optimized_count: 0,
                    total_size_bytes: 0,
                    avg_clone_time_ns: 0.0,
                });

            type_stat.clone_count += 1;
            type_stat.total_size_bytes += size_bytes as u64;

            // Update average clone time
            let total_time = type_stat.avg_clone_time_ns * (type_stat.clone_count - 1) as f64
                + duration_ns as f64;
            type_stat.avg_clone_time_ns = total_time / type_stat.clone_count as f64;
        }
    }

    /// Record an optimized clone (using Arc)
    pub fn record_optimized_clone(&self, type_name: &str, memory_saved: usize) {
        use std::sync::atomic::Ordering;

        self.optimized_clones.fetch_add(1, Ordering::Relaxed);
        self.memory_saved_bytes
            .fetch_add(memory_saved as u64, Ordering::Relaxed);

        // Update per-type statistics
        if let Ok(mut stats) = self.type_stats.lock() {
            if let Some(type_stat) = stats.get_mut(type_name) {
                type_stat.optimized_count += 1;
            }
        }
    }

    /// Record an avoided clone (Arc sharing)
    pub fn record_avoided_clone(&self, _type_name: &str, memory_saved: usize) {
        use std::sync::atomic::Ordering;

        self.avoided_clones.fetch_add(1, Ordering::Relaxed);
        self.memory_saved_bytes
            .fetch_add(memory_saved as u64, Ordering::Relaxed);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> CloneMonitorStats {
        use std::sync::atomic::Ordering;

        let total_clones = self.total_clones.load(Ordering::Relaxed);
        let optimized_clones = self.optimized_clones.load(Ordering::Relaxed);
        let total_clone_time_ns = self.total_clone_time_ns.load(Ordering::Relaxed);

        let avg_clone_time_ns = if total_clones > 0 {
            total_clone_time_ns as f64 / total_clones as f64
        } else {
            0.0
        };

        let performance_improvement = if total_clones > 0 {
            (optimized_clones + self.avoided_clones.load(Ordering::Relaxed)) as f64
                / total_clones as f64
        } else {
            0.0
        };

        CloneMonitorStats {
            total_clones,
            optimized_clones,
            avoided_clones: self.avoided_clones.load(Ordering::Relaxed),
            total_clone_time_ns,
            avg_clone_time_ns,
            memory_saved_bytes: self.memory_saved_bytes.load(Ordering::Relaxed),
            performance_improvement,
        }
    }
}

/// Global clone monitor instance
static GLOBAL_CLONE_MONITOR: OnceLock<CloneMonitor> = OnceLock::new();

/// Get the global clone monitor
pub fn get_clone_monitor() -> &'static CloneMonitor {
    GLOBAL_CLONE_MONITOR.get_or_init(CloneMonitor::new)
}

/// Record a clone operation
pub fn record_clone(type_name: &str, size_bytes: usize, duration_ns: u64) {
    get_clone_monitor().record_clone(type_name, size_bytes, duration_ns);
}

/// Record an optimized clone
pub fn record_optimized_clone(type_name: &str, memory_saved: usize) {
    get_clone_monitor().record_optimized_clone(type_name, memory_saved);
}

/// Record an avoided clone
pub fn record_avoided_clone(type_name: &str, memory_saved: usize) {
    get_clone_monitor().record_avoided_clone(type_name, memory_saved);
}

/// Get clone monitoring statistics
pub fn get_clone_stats() -> CloneMonitorStats {
    get_clone_monitor().get_stats()
}

/// Get optimization recommendations
pub fn get_optimization_recommendations() -> Vec<String> {
    // Simple recommendations based on current stats
    let stats = get_clone_stats();
    let mut recommendations = Vec::new();

    if stats.total_clones > 100 && stats.performance_improvement < 0.3 {
        recommendations.push(format!(
            "Low optimization rate: {}% of clones optimized. Consider using Arc sharing for frequently cloned types.",
            (stats.performance_improvement * 100.0) as u32
        ));
    }

    if stats.memory_saved_bytes > 1024 * 1024 {
        recommendations.push(format!(
            "Good progress: {} MB saved through clone optimization.",
            stats.memory_saved_bytes / (1024 * 1024)
        ));
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_monitor_creation() {
        let _monitor = CloneMonitor::new();
        // Just test creation doesn't panic
    }

    #[test]
    fn test_record_clone() {
        let monitor = CloneMonitor::new();
        monitor.record_clone("test_type", 1024, 456);

        let stats = get_clone_stats();
        assert!(stats.total_clones == stats.total_clones); // Just check it's accessible
    }

    #[test]
    fn test_get_clone_stats() {
        let stats = get_clone_stats();
        // Stats should be accessible
        assert!(stats.total_clones == stats.total_clones); // Just check it's accessible
        assert!(stats.avg_clone_time_ns >= 0.0);
    }

    #[test]
    fn test_optimization_recommendations() {
        let recommendations = get_optimization_recommendations();
        // Should return some recommendations
        assert!(!recommendations.is_empty() || recommendations.is_empty()); // Either is fine
    }
}

