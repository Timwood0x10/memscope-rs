//! Bottleneck detection and analysis for performance optimization (Plus Version)
//!
//! This module provides comprehensive bottleneck detection capabilities,
//! identifying performance bottlenecks across CPU, memory, I/O, network,
//! locks, and allocation patterns.
//!
//! This is an enhanced version with bug fixes and additional features
//! compared to the legacy lockfree_types.rs implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of performance bottleneck (Plus Version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BottleneckKind {
    /// CPU-bound bottleneck
    Cpu,
    /// Memory-bound bottleneck
    Memory,
    /// I/O-bound bottleneck
    Io,
    /// Network-bound bottleneck
    Network,
    /// Lock contention bottleneck
    Lock,
    /// Allocation pattern bottleneck
    Allocation,
}

impl BottleneckKind {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU-bound bottleneck",
            Self::Memory => "Memory-bound bottleneck",
            Self::Io => "I/O-bound bottleneck",
            Self::Network => "Network-bound bottleneck",
            Self::Lock => "Lock contention bottleneck",
            Self::Allocation => "Allocation pattern bottleneck",
        }
    }

    /// Get default severity threshold for this bottleneck type
    pub fn default_severity_threshold(&self) -> f64 {
        match self {
            Self::Cpu => 0.8,
            Self::Memory => 0.85,
            Self::Io => 0.75,
            Self::Network => 0.7,
            Self::Lock => 0.9,
            Self::Allocation => 0.8,
        }
    }
}

/// Performance bottleneck detected during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    /// Type of bottleneck
    pub bottleneck_type: BottleneckKind,
    /// Task ID where bottleneck was detected
    pub task_id: u64,
    /// Task name
    pub task_name: String,
    /// Severity score (0.0 to 1.0, higher is more severe)
    pub severity: f64,
    /// Human-readable description
    pub description: String,
    /// Suggested remediation
    pub suggestion: String,
    /// Timestamp when bottleneck was detected
    pub timestamp_ms: u64,
    /// Additional metrics specific to bottleneck type
    pub metrics: BottleneckMetrics,
}

/// Metrics specific to bottleneck type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckMetrics {
    /// CPU usage percentage (for CPU bottleneck)
    pub cpu_usage_percent: f64,
    /// Memory usage percentage (for memory bottleneck)
    pub memory_usage_percent: f64,
    /// I/O bytes processed (for I/O bottleneck)
    pub io_bytes_processed: u64,
    /// Network bytes transferred (for network bottleneck)
    pub network_bytes_transferred: u64,
    /// Lock wait time in nanoseconds (for lock bottleneck)
    pub lock_wait_time_ns: u64,
    /// Allocation frequency (for allocation bottleneck)
    pub allocation_frequency: f64,
    /// Average allocation size (for allocation bottleneck)
    pub average_allocation_size: f64,
}

/// Task metrics for bottleneck analysis
///
/// This structure contains all performance metrics needed for bottleneck detection
/// across CPU, memory, I/O, network, and allocation patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Unique identifier for the task
    pub task_id: u64,
    /// Human-readable task name
    pub task_name: String,
    /// CPU usage percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage percentage (0.0 to 100.0)
    pub memory_usage_percent: f64,
    /// Total I/O bytes processed
    pub io_bytes_processed: u64,
    /// Total network bytes transferred
    pub network_bytes_transferred: u64,
    /// Allocation frequency in allocations per second
    pub allocation_frequency: f64,
    /// Average allocation size in bytes
    pub average_allocation_size: f64,
}

impl Default for BottleneckMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            lock_wait_time_ns: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        }
    }
}

/// Bottleneck detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckConfig {
    /// Enable CPU bottleneck detection
    pub enable_cpu_detection: bool,
    /// Enable memory bottleneck detection
    pub enable_memory_detection: bool,
    /// Enable I/O bottleneck detection
    pub enable_io_detection: bool,
    /// Enable network bottleneck detection
    pub enable_network_detection: bool,
    /// Enable lock contention detection
    pub enable_lock_detection: bool,
    /// Enable allocation pattern detection
    pub enable_allocation_detection: bool,
    /// Custom severity thresholds for each bottleneck type
    pub severity_thresholds: HashMap<BottleneckKind, f64>,
    /// Minimum severity to report a bottleneck
    pub min_severity_to_report: f64,
}

impl Default for BottleneckConfig {
    fn default() -> Self {
        let mut severity_thresholds = HashMap::new();
        severity_thresholds.insert(BottleneckKind::Cpu, 0.8);
        severity_thresholds.insert(BottleneckKind::Memory, 0.85);
        severity_thresholds.insert(BottleneckKind::Io, 0.75);
        severity_thresholds.insert(BottleneckKind::Network, 0.7);
        severity_thresholds.insert(BottleneckKind::Lock, 0.9);
        severity_thresholds.insert(BottleneckKind::Allocation, 0.8);

        Self {
            enable_cpu_detection: true,
            enable_memory_detection: true,
            enable_io_detection: true,
            enable_network_detection: true,
            enable_lock_detection: true,
            enable_allocation_detection: true,
            severity_thresholds,
            min_severity_to_report: 0.5,
        }
    }
}

/// Bottleneck analyzer
pub struct BottleneckAnalyzer {
    config: BottleneckConfig,
}

impl BottleneckAnalyzer {
    /// Create new bottleneck analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: BottleneckConfig::default(),
        }
    }

    /// Create new bottleneck analyzer with custom configuration
    pub fn with_config(config: BottleneckConfig) -> Self {
        Self { config }
    }

    /// Analyze task for bottlenecks
    pub fn analyze_task(&self, metrics: &TaskMetrics) -> Vec<PerformanceIssue> {
        let mut bottlenecks = Vec::new();
        let timestamp_ms = Self::current_timestamp_ms();

        if self.config.enable_cpu_detection {
            if let Some(bottleneck) = self.detect_cpu_bottleneck(
                metrics.task_id,
                &metrics.task_name,
                metrics.cpu_usage_percent,
                timestamp_ms,
            ) {
                bottlenecks.push(bottleneck);
            }
        }

        if self.config.enable_memory_detection {
            if let Some(bottleneck) = self.detect_memory_bottleneck(
                metrics.task_id,
                &metrics.task_name,
                metrics.memory_usage_percent,
                timestamp_ms,
            ) {
                bottlenecks.push(bottleneck);
            }
        }

        if self.config.enable_io_detection {
            if let Some(bottleneck) = self.detect_io_bottleneck(
                metrics.task_id,
                &metrics.task_name,
                metrics.io_bytes_processed,
                timestamp_ms,
            ) {
                bottlenecks.push(bottleneck);
            }
        }

        if self.config.enable_network_detection {
            if let Some(bottleneck) = self.detect_network_bottleneck(
                metrics.task_id,
                &metrics.task_name,
                metrics.network_bytes_transferred,
                timestamp_ms,
            ) {
                bottlenecks.push(bottleneck);
            }
        }

        if self.config.enable_allocation_detection {
            if let Some(bottleneck) = self.detect_allocation_bottleneck(
                metrics.task_id,
                &metrics.task_name,
                metrics.allocation_frequency,
                metrics.average_allocation_size,
                timestamp_ms,
            ) {
                bottlenecks.push(bottleneck);
            }
        }

        bottlenecks
            .into_iter()
            .filter(|b| b.severity >= self.config.min_severity_to_report)
            .collect()
    }

    /// Detect CPU bottleneck
    fn detect_cpu_bottleneck(
        &self,
        task_id: u64,
        task_name: &str,
        cpu_usage_percent: f64,
        timestamp_ms: u64,
    ) -> Option<PerformanceIssue> {
        let threshold = *self
            .config
            .severity_thresholds
            .get(&BottleneckKind::Cpu)
            .unwrap_or(&BottleneckKind::Cpu.default_severity_threshold());

        if cpu_usage_percent < threshold {
            return None;
        }

        let severity = (cpu_usage_percent - threshold) / (100.0 - threshold);

        let description = format!(
            "CPU usage is {:.1}%, exceeding threshold of {:.1}%",
            cpu_usage_percent,
            threshold * 100.0
        );

        let suggestion = if severity > 0.8 {
            "Critical CPU bottleneck: Consider parallelizing work, optimizing algorithms, or reducing computational complexity"
        } else if severity > 0.5 {
            "Significant CPU bottleneck: Profile hot paths, optimize critical sections, and consider caching"
        } else {
            "Moderate CPU bottleneck: Review algorithm efficiency and consider performance optimizations"
        };

        Some(PerformanceIssue {
            bottleneck_type: BottleneckKind::Cpu,
            task_id,
            task_name: task_name.to_string(),
            severity,
            description,
            suggestion: suggestion.to_string(),
            timestamp_ms,
            metrics: BottleneckMetrics {
                cpu_usage_percent,
                ..Default::default()
            },
        })
    }

    /// Detect memory bottleneck
    fn detect_memory_bottleneck(
        &self,
        task_id: u64,
        task_name: &str,
        memory_usage_percent: f64,
        timestamp_ms: u64,
    ) -> Option<PerformanceIssue> {
        let threshold = *self
            .config
            .severity_thresholds
            .get(&BottleneckKind::Memory)
            .unwrap_or(&BottleneckKind::Memory.default_severity_threshold());

        if memory_usage_percent < threshold {
            return None;
        }

        let severity = (memory_usage_percent - threshold) / (100.0 - threshold);

        let description = format!(
            "Memory usage is {:.1}%, exceeding threshold of {:.1}%",
            memory_usage_percent,
            threshold * 100.0
        );

        let suggestion = if severity > 0.8 {
            "Critical memory bottleneck: Implement memory pooling, reduce memory footprint, or increase available memory"
        } else if severity > 0.5 {
            "Significant memory bottleneck: Optimize data structures, reduce allocations, and implement memory reuse"
        } else {
            "Moderate memory bottleneck: Review memory usage patterns and consider optimization strategies"
        };

        Some(PerformanceIssue {
            bottleneck_type: BottleneckKind::Memory,
            task_id,
            task_name: task_name.to_string(),
            severity,
            description,
            suggestion: suggestion.to_string(),
            timestamp_ms,
            metrics: BottleneckMetrics {
                memory_usage_percent,
                ..Default::default()
            },
        })
    }

    /// Detect I/O bottleneck
    fn detect_io_bottleneck(
        &self,
        task_id: u64,
        task_name: &str,
        io_bytes_processed: u64,
        timestamp_ms: u64,
    ) -> Option<PerformanceIssue> {
        let threshold = *self
            .config
            .severity_thresholds
            .get(&BottleneckKind::Io)
            .unwrap_or(&BottleneckKind::Io.default_severity_threshold());

        let io_mb = io_bytes_processed as f64 / 1_048_576.0;

        if io_mb < threshold * 100.0 {
            return None;
        }

        let severity = ((io_mb - threshold * 100.0) / (1_000.0 - threshold * 100.0)).min(1.0);

        let description =
            format!("I/O throughput is {io_mb:.1} MB, indicating potential bottleneck");

        let suggestion = if severity > 0.8 {
            "Critical I/O bottleneck: Implement buffering, use asynchronous I/O, or optimize access patterns"
        } else if severity > 0.5 {
            "Significant I/O bottleneck: Consider batching operations, using memory-mapped files, or optimizing disk layout"
        } else {
            "Moderate I/O bottleneck: Review I/O patterns and consider caching strategies"
        };

        Some(PerformanceIssue {
            bottleneck_type: BottleneckKind::Io,
            task_id,
            task_name: task_name.to_string(),
            severity,
            description,
            suggestion: suggestion.to_string(),
            timestamp_ms,
            metrics: BottleneckMetrics {
                io_bytes_processed,
                ..Default::default()
            },
        })
    }

    /// Detect network bottleneck
    fn detect_network_bottleneck(
        &self,
        task_id: u64,
        task_name: &str,
        network_bytes_transferred: u64,
        timestamp_ms: u64,
    ) -> Option<PerformanceIssue> {
        let threshold = *self
            .config
            .severity_thresholds
            .get(&BottleneckKind::Network)
            .unwrap_or(&BottleneckKind::Network.default_severity_threshold());

        let network_mb = network_bytes_transferred as f64 / 1_048_576.0;

        if network_mb < threshold * 50.0 {
            return None;
        }

        let severity = ((network_mb - threshold * 50.0) / (500.0 - threshold * 50.0)).min(1.0);

        let description = format!(
            "Network transfer is {:.1} MB, indicating potential bottleneck",
            network_mb
        );

        let suggestion = if severity > 0.8 {
            "Critical network bottleneck: Implement compression, use connection pooling, or optimize data serialization"
        } else if severity > 0.5 {
            "Significant network bottleneck: Consider batching requests, using HTTP/2, or implementing caching"
        } else {
            "Moderate network bottleneck: Review network usage patterns and consider optimization strategies"
        };

        Some(PerformanceIssue {
            bottleneck_type: BottleneckKind::Network,
            task_id,
            task_name: task_name.to_string(),
            severity,
            description,
            suggestion: suggestion.to_string(),
            timestamp_ms,
            metrics: BottleneckMetrics {
                network_bytes_transferred,
                ..Default::default()
            },
        })
    }

    /// Detect allocation pattern bottleneck
    fn detect_allocation_bottleneck(
        &self,
        task_id: u64,
        task_name: &str,
        allocation_frequency: f64,
        average_allocation_size: f64,
        timestamp_ms: u64,
    ) -> Option<PerformanceIssue> {
        let threshold = *self
            .config
            .severity_thresholds
            .get(&BottleneckKind::Allocation)
            .unwrap_or(&BottleneckKind::Allocation.default_severity_threshold());

        let severity = if allocation_frequency > 1000.0 {
            (allocation_frequency - 1000.0) / 9000.0
        } else if average_allocation_size < 1024.0 && allocation_frequency > 100.0 {
            0.7
        } else {
            0.0
        };

        if severity < threshold {
            return None;
        }

        let description = if allocation_frequency > 1000.0 {
            format!(
                "High allocation frequency: {:.0} allocations/second",
                allocation_frequency
            )
        } else {
            format!(
                "Small frequent allocations: {:.0} bytes avg, {:.0} allocations/second",
                average_allocation_size, allocation_frequency
            )
        };

        let suggestion = if severity > 0.8 {
            "Critical allocation bottleneck: Implement object pooling, use arena allocators, or redesign to reduce allocations"
        } else if severity > 0.5 {
            "Significant allocation bottleneck: Consider allocation pooling, reuse buffers, or optimize allocation patterns"
        } else {
            "Moderate allocation bottleneck: Review allocation patterns and consider memory reuse strategies"
        };

        Some(PerformanceIssue {
            bottleneck_type: BottleneckKind::Allocation,
            task_id,
            task_name: task_name.to_string(),
            severity,
            description,
            suggestion: suggestion.to_string(),
            timestamp_ms,
            metrics: BottleneckMetrics {
                allocation_frequency,
                average_allocation_size,
                ..Default::default()
            },
        })
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Get analyzer configuration
    pub fn config(&self) -> &BottleneckConfig {
        &self.config
    }

    /// Update analyzer configuration
    pub fn set_config(&mut self, config: BottleneckConfig) {
        self.config = config;
    }
}

impl Default for BottleneckAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_type_description() {
        assert_eq!(BottleneckKind::Cpu.description(), "CPU-bound bottleneck");
        assert_eq!(
            BottleneckKind::Memory.description(),
            "Memory-bound bottleneck"
        );
        assert_eq!(BottleneckKind::Io.description(), "I/O-bound bottleneck");
        assert_eq!(
            BottleneckKind::Network.description(),
            "Network-bound bottleneck"
        );
        assert_eq!(
            BottleneckKind::Lock.description(),
            "Lock contention bottleneck"
        );
        assert_eq!(
            BottleneckKind::Allocation.description(),
            "Allocation pattern bottleneck"
        );
    }

    #[test]
    fn test_bottleneck_analyzer_creation() {
        let analyzer = BottleneckAnalyzer::new();
        assert!(analyzer.config().enable_cpu_detection);
        assert!(analyzer.config().enable_memory_detection);
    }

    #[test]
    fn test_cpu_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 90.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckKind::Cpu);
        assert!(bottlenecks[0].severity > 0.0);
    }

    #[test]
    fn test_memory_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_percent: 90.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckKind::Memory);
        assert!(bottlenecks[0].severity > 0.0);
    }

    #[test]
    fn test_io_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 750_000_000,
            network_bytes_transferred: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckKind::Io);
        assert!(bottlenecks[0].severity > 0.0);
    }

    #[test]
    fn test_network_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 0,
            // Need at least ~35MB to trigger network bottleneck with default threshold (0.7)
            network_bytes_transferred: 350_000_000, // 334MB
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckKind::Network);
        assert!(bottlenecks[0].severity > 0.0);
    }

    #[test]
    fn test_allocation_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            // Need >= 8200 allocations/sec to trigger with default threshold (0.8)
            allocation_frequency: 9000.0,
            average_allocation_size: 512.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(!bottlenecks.is_empty());
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckKind::Allocation);
        assert!(bottlenecks[0].severity > 0.0);
    }

    #[test]
    fn test_no_bottleneck_below_threshold() {
        let analyzer = BottleneckAnalyzer::new();
        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 50.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let config = BottleneckConfig {
            min_severity_to_report: 0.9,
            ..Default::default()
        };
        let analyzer = BottleneckAnalyzer::with_config(config);

        let metrics = TaskMetrics {
            task_id: 1,
            task_name: "test_task".to_string(),
            cpu_usage_percent: 85.0,
            memory_usage_percent: 50.0,
            io_bytes_processed: 0,
            network_bytes_transferred: 0,
            allocation_frequency: 0.0,
            average_allocation_size: 0.0,
        };
        let bottlenecks = analyzer.analyze_task(&metrics);

        assert!(bottlenecks.is_empty());
    }

    #[test]
    fn test_bottleneck_metrics_default() {
        let metrics = BottleneckMetrics::default();
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.memory_usage_percent, 0.0);
        assert_eq!(metrics.io_bytes_processed, 0);
        assert_eq!(metrics.network_bytes_transferred, 0);
        assert_eq!(metrics.lock_wait_time_ns, 0);
        assert_eq!(metrics.allocation_frequency, 0.0);
        assert_eq!(metrics.average_allocation_size, 0.0);
    }
}
