//! Performance monitoring and metrics collection system
//!
//! Provides comprehensive performance tracking for all MemScope operations.
//! Features real-time metrics, historical analysis, and performance alerting.

pub mod analyzer;
pub mod collector;
pub mod reporter;

pub use analyzer::{Benchmark, PerformanceAnalyzer, PerformanceReport};
pub use collector::{Metric, MetricType, MetricValue, MetricsCollector};
pub use reporter::{AlertThreshold, MetricsReporter, ReportFormat};
