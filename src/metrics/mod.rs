//! Performance monitoring and metrics collection system
//!
//! # Deprecated
//!
//! This module is deprecated. Performance monitoring functionality has been
//! integrated into the new unified tracking system.
//!
//! Please use the new unified tracking system located in `src/new/tracker/mod.rs`.
//! Metrics are now available through `UnifiedTracker::stats()` and `UnifiedTracker::snapshot()`.
//!
//! Migration Guide:
//! - Replace `crate::metrics::MetricsCollector` with `crate::new::tracker::UnifiedTracker`
//! - Use `UnifiedTracker::stats()` for statistics
//! - Use `UnifiedTracker::snapshot()` for comprehensive metrics
//! - All functionality is preserved for backward compatibility
//!
//! Provides comprehensive performance tracking for all MemScope operations.
//! Features real-time metrics, historical analysis, and performance alerting.

pub mod analyzer;
pub mod collector;
pub mod reporter;

pub use analyzer::{Benchmark, PerformanceAnalyzer, PerformanceReport};
pub use collector::{Metric, MetricType, MetricValue, MetricsCollector};
pub use reporter::{AlertThreshold, MetricsReporter, ReportFormat};