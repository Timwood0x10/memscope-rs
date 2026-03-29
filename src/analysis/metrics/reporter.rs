use super::{MetricsCollector, PerformanceReport};
use std::fmt::Write;

/// Memory analysis metrics reporter
/// Generates reports focused on offline memory profiling efficiency
pub struct MetricsReporter {
    /// Output format configuration
    format: ReportFormat,
    /// Alert thresholds for memory analysis performance
    alert_thresholds: Vec<AlertThreshold>,
    /// Whether to include detailed breakdown
    include_details: bool,
}

/// Available report output formats
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    /// Human-readable text format
    PlainText,
    /// Structured JSON format for tooling
    Json,
    /// Markdown format for documentation
    Markdown,
    /// CSV format for spreadsheet analysis
    Csv,
}

/// Performance alert threshold
#[derive(Debug, Clone)]
pub struct AlertThreshold {
    /// Metric name to monitor
    pub metric_name: String,
    /// Threshold value
    pub threshold: f64,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity level
    pub severity: AlertSeverity,
    /// Human-readable description
    pub description: String,
}

/// Alert condition types
#[derive(Debug, Clone, PartialEq)]
pub enum AlertCondition {
    /// Value exceeds threshold
    GreaterThan,
    /// Value is below threshold
    LessThan,
    /// Value equals threshold
    Equals,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational notice
    Info,
    /// Performance warning
    Warning,
    /// Performance problem
    Error,
    /// Critical performance issue
    Critical,
}

impl MetricsReporter {
    /// Create new reporter with plain text format
    pub fn new() -> Self {
        Self {
            format: ReportFormat::PlainText,
            alert_thresholds: Self::default_memory_thresholds(),
            include_details: true,
        }
    }

    /// Create reporter with specific format
    pub fn with_format(format: ReportFormat) -> Self {
        Self {
            format,
            alert_thresholds: Self::default_memory_thresholds(),
            include_details: true,
        }
    }

    /// Set whether to include detailed breakdowns
    pub fn with_details(mut self, include_details: bool) -> Self {
        self.include_details = include_details;
        self
    }

    /// Add custom alert threshold
    pub fn add_alert_threshold(mut self, threshold: AlertThreshold) -> Self {
        self.alert_thresholds.push(threshold);
        self
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self, report: &PerformanceReport) -> String {
        match self.format {
            ReportFormat::PlainText => self.generate_text_report(report),
            ReportFormat::Json => self.generate_json_report(report),
            ReportFormat::Markdown => self.generate_markdown_report(report),
            ReportFormat::Csv => self.generate_csv_report(report),
        }
    }

    /// Generate metrics summary
    pub fn generate_metrics_summary(&self, collector: &MetricsCollector) -> String {
        let summary = collector.get_summary();

        match self.format {
            ReportFormat::PlainText => {
                format!(
                    "Metrics Summary:\n\
                     - Total Metrics: {}\n\
                     - Active Metrics: {}\n\
                     - Update Rate: {:.2}/sec\n\
                     - Uptime: {:.2}h\n\
                     - Sample Rate: {:.1}%\n",
                    summary.total_metrics,
                    summary.active_metrics,
                    summary.update_rate,
                    summary.uptime.as_secs_f64() / 3600.0,
                    summary.sample_rate * 100.0
                )
            }
            ReportFormat::Json => {
                format!(
                    r#"{{"total_metrics": {}, "active_metrics": {}, "update_rate": {:.2}, "uptime_hours": {:.2}, "sample_rate": {:.3}}}"#,
                    summary.total_metrics,
                    summary.active_metrics,
                    summary.update_rate,
                    summary.uptime.as_secs_f64() / 3600.0,
                    summary.sample_rate
                )
            }
            _ => self.generate_text_report(&PerformanceReport {
                efficiency_score: 0.0,
                tracking_performance: Default::default(),
                symbol_performance: Default::default(),
                pointer_performance: Default::default(),
                memory_efficiency: Default::default(),
                recommendations: vec![],
            }),
        }
    }

    /// Check for performance alerts
    pub fn check_alerts(&self, collector: &MetricsCollector) -> Vec<TriggeredAlert> {
        let mut alerts = Vec::new();

        for threshold in &self.alert_thresholds {
            if let Some(metric) = collector.get_metric(&threshold.metric_name) {
                let current_value = self.extract_metric_value(metric);

                let triggered = match threshold.condition {
                    AlertCondition::GreaterThan => current_value > threshold.threshold,
                    AlertCondition::LessThan => current_value < threshold.threshold,
                    AlertCondition::Equals => (current_value - threshold.threshold).abs() < 0.001,
                };

                if triggered {
                    alerts.push(TriggeredAlert {
                        metric_name: threshold.metric_name.clone(),
                        current_value,
                        threshold_value: threshold.threshold,
                        condition: threshold.condition.clone(),
                        severity: threshold.severity.clone(),
                        description: threshold.description.clone(),
                    });
                }
            }
        }

        // Sort by severity (most critical first)
        alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
        alerts
    }

    fn default_memory_thresholds() -> Vec<AlertThreshold> {
        vec![
            AlertThreshold {
                metric_name: "tracking_completeness".to_string(),
                threshold: 0.95,
                condition: AlertCondition::LessThan,
                severity: AlertSeverity::Warning,
                description: "Memory tracking completeness below 95%".to_string(),
            },
            AlertThreshold {
                metric_name: "allocation_tracking_time".to_string(),
                threshold: 100.0, // microseconds
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Error,
                description: "Allocation tracking latency exceeds 100µs".to_string(),
            },
            AlertThreshold {
                metric_name: "symbol_cache_hit_ratio".to_string(),
                threshold: 0.8,
                condition: AlertCondition::LessThan,
                severity: AlertSeverity::Warning,
                description: "Symbol cache hit ratio below 80%".to_string(),
            },
            AlertThreshold {
                metric_name: "total_analysis_memory".to_string(),
                threshold: 512.0, // MB
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Critical,
                description: "Analysis memory usage exceeds 512MB".to_string(),
            },
            AlertThreshold {
                metric_name: "memory_fragmentation".to_string(),
                threshold: 0.3,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                description: "Memory fragmentation exceeds 30%".to_string(),
            },
        ]
    }

    fn generate_text_report(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        writeln!(output, "=== Memory Analysis Performance Report ===").expect("Write failed");
        writeln!(
            output,
            "Overall Efficiency Score: {:.1}%",
            report.efficiency_score * 100.0
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        // Tracking Performance
        writeln!(output, "Memory Tracking Performance:").expect("Write failed");
        writeln!(
            output,
            "  - Average Allocation Time: {:.2}µs",
            report.tracking_performance.avg_allocation_time.as_micros()
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Tracking Completeness: {:.1}%",
            report.tracking_performance.completeness * 100.0
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Memory Overhead: {:.2}MB",
            report.tracking_performance.overhead_bytes as f64 / (1024.0 * 1024.0)
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Throughput: {:.0} allocs/sec",
            report.tracking_performance.throughput
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        // Symbol Performance
        writeln!(output, "Symbol Resolution Performance:").expect("Write failed");
        writeln!(
            output,
            "  - Average Resolution Time: {:.2}ms",
            report.symbol_performance.avg_resolution_time.as_millis()
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Cache Hit Ratio: {:.1}%",
            report.symbol_performance.cache_hit_ratio * 100.0
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Resolution Rate: {:.0} symbols/sec",
            report.symbol_performance.resolution_rate
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Cache Memory: {:.2}MB",
            report.symbol_performance.cache_memory_usage as f64 / (1024.0 * 1024.0)
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        // Smart Pointer Performance
        writeln!(output, "Smart Pointer Analysis:").expect("Write failed");
        writeln!(
            output,
            "  - Analysis Time: {:.2}ms",
            report.pointer_performance.analysis_time.as_millis()
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Leak Detection Accuracy: {:.1}%",
            report.pointer_performance.leak_detection_accuracy * 100.0
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Analysis Rate: {:.0} pointers/sec",
            report.pointer_performance.analysis_rate
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        // Memory Efficiency
        writeln!(output, "Memory Usage Efficiency:").expect("Write failed");
        writeln!(
            output,
            "  - Total Memory: {:.1}MB",
            report.memory_efficiency.total_memory_mb
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Memory per Allocation: {:.1} bytes",
            report.memory_efficiency.memory_per_allocation
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Growth Rate: {:.2}MB/hour",
            report.memory_efficiency.growth_rate
        )
        .expect("Write failed");
        writeln!(
            output,
            "  - Fragmentation: {:.1}%",
            report.memory_efficiency.fragmentation_ratio * 100.0
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        // Recommendations
        if !report.recommendations.is_empty() {
            writeln!(output, "Performance Recommendations:").expect("Write failed");
            for (i, rec) in report.recommendations.iter().enumerate() {
                writeln!(output, "  {}. {}", i + 1, rec).expect("Write failed");
            }
        }

        output
    }

    fn generate_json_report(&self, report: &PerformanceReport) -> String {
        format!(
            r#"{{
  "efficiency_score": {:.3},
  "tracking_performance": {{
    "avg_allocation_time_us": {:.2},
    "completeness": {:.3},
    "overhead_bytes": {},
    "throughput": {:.2}
  }},
  "symbol_performance": {{
    "avg_resolution_time_ms": {:.2},
    "cache_hit_ratio": {:.3},
    "resolution_rate": {:.2},
    "cache_memory_usage": {}
  }},
  "pointer_performance": {{
    "analysis_time_ms": {:.2},
    "leak_detection_accuracy": {:.3},
    "analysis_rate": {:.2}
  }},
  "memory_efficiency": {{
    "total_memory_mb": {:.2},
    "memory_per_allocation": {:.2},
    "growth_rate": {:.2},
    "fragmentation_ratio": {:.3}
  }},
  "recommendations": [{}]
}}"#,
            report.efficiency_score,
            report.tracking_performance.avg_allocation_time.as_micros(),
            report.tracking_performance.completeness,
            report.tracking_performance.overhead_bytes,
            report.tracking_performance.throughput,
            report.symbol_performance.avg_resolution_time.as_millis(),
            report.symbol_performance.cache_hit_ratio,
            report.symbol_performance.resolution_rate,
            report.symbol_performance.cache_memory_usage,
            report.pointer_performance.analysis_time.as_millis(),
            report.pointer_performance.leak_detection_accuracy,
            report.pointer_performance.analysis_rate,
            report.memory_efficiency.total_memory_mb,
            report.memory_efficiency.memory_per_allocation,
            report.memory_efficiency.growth_rate,
            report.memory_efficiency.fragmentation_ratio,
            report
                .recommendations
                .iter()
                .map(|r| format!("\"{}\"", r.replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn generate_markdown_report(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        writeln!(output, "# Memory Analysis Performance Report").expect("Write failed");
        writeln!(output).expect("Write failed");
        writeln!(
            output,
            "**Overall Efficiency Score:** {:.1}%",
            report.efficiency_score * 100.0
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        writeln!(output, "## Performance Metrics").expect("Write failed");
        writeln!(output).expect("Write failed");

        writeln!(output, "### Memory Tracking").expect("Write failed");
        writeln!(output, "| Metric | Value |").expect("Write failed");
        writeln!(output, "|--------|-------|").expect("Write failed");
        writeln!(
            output,
            "| Allocation Time | {:.2}µs |",
            report.tracking_performance.avg_allocation_time.as_micros()
        )
        .expect("Write failed");
        writeln!(
            output,
            "| Completeness | {:.1}% |",
            report.tracking_performance.completeness * 100.0
        )
        .expect("Write failed");
        writeln!(
            output,
            "| Memory Overhead | {:.2}MB |",
            report.tracking_performance.overhead_bytes as f64 / (1024.0 * 1024.0)
        )
        .expect("Write failed");
        writeln!(
            output,
            "| Throughput | {:.0} allocs/sec |",
            report.tracking_performance.throughput
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        writeln!(output, "### Symbol Resolution").expect("Write failed");
        writeln!(output, "| Metric | Value |").expect("Write failed");
        writeln!(output, "|--------|-------|").expect("Write failed");
        writeln!(
            output,
            "| Resolution Time | {:.2}ms |",
            report.symbol_performance.avg_resolution_time.as_millis()
        )
        .expect("Write failed");
        writeln!(
            output,
            "| Cache Hit Ratio | {:.1}% |",
            report.symbol_performance.cache_hit_ratio * 100.0
        )
        .expect("Write failed");
        writeln!(
            output,
            "| Resolution Rate | {:.0} symbols/sec |",
            report.symbol_performance.resolution_rate
        )
        .expect("Write failed");
        writeln!(output).expect("Write failed");

        if !report.recommendations.is_empty() {
            writeln!(output, "## Recommendations").expect("Write failed");
            writeln!(output).expect("Write failed");
            for rec in &report.recommendations {
                writeln!(output, "- {}", rec).expect("Write failed");
            }
        }

        output
    }

    fn generate_csv_report(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        writeln!(output, "metric_category,metric_name,value,unit").expect("Write failed");
        writeln!(
            output,
            "overall,efficiency_score,{:.3},percentage",
            report.efficiency_score
        )
        .expect("Write failed");
        writeln!(
            output,
            "tracking,avg_allocation_time,{:.2},microseconds",
            report.tracking_performance.avg_allocation_time.as_micros()
        )
        .expect("Write failed");
        writeln!(
            output,
            "tracking,completeness,{:.3},ratio",
            report.tracking_performance.completeness
        )
        .expect("Write failed");
        writeln!(
            output,
            "tracking,overhead_bytes,{},bytes",
            report.tracking_performance.overhead_bytes
        )
        .expect("Write failed");
        writeln!(
            output,
            "tracking,throughput,{:.2},allocations_per_second",
            report.tracking_performance.throughput
        )
        .expect("Write failed");
        writeln!(
            output,
            "symbol,avg_resolution_time,{:.2},milliseconds",
            report.symbol_performance.avg_resolution_time.as_millis()
        )
        .expect("Write failed");
        writeln!(
            output,
            "symbol,cache_hit_ratio,{:.3},ratio",
            report.symbol_performance.cache_hit_ratio
        )
        .expect("Write failed");
        writeln!(
            output,
            "symbol,resolution_rate,{:.2},symbols_per_second",
            report.symbol_performance.resolution_rate
        )
        .expect("Write failed");
        writeln!(
            output,
            "memory,total_memory_mb,{:.2},megabytes",
            report.memory_efficiency.total_memory_mb
        )
        .expect("Write failed");
        writeln!(
            output,
            "memory,fragmentation_ratio,{:.3},ratio",
            report.memory_efficiency.fragmentation_ratio
        )
        .expect("Write failed");

        output
    }

    fn extract_metric_value(&self, metric: &super::Metric) -> f64 {
        match &metric.value {
            super::MetricValue::Counter(counter) => {
                counter.load(std::sync::atomic::Ordering::Relaxed) as f64
            }
            super::MetricValue::Gauge(value) => *value,
            super::MetricValue::Histogram(hist) => hist.average(),
            super::MetricValue::Timer(timer) => timer.average_duration().as_millis() as f64,
            super::MetricValue::Rate(rate) => rate.current_rate,
        }
    }
}

/// Alert that has been triggered
#[derive(Debug, Clone)]
pub struct TriggeredAlert {
    /// Name of the metric that triggered
    pub metric_name: String,
    /// Current value of the metric
    pub current_value: f64,
    /// Threshold value that was exceeded
    pub threshold_value: f64,
    /// The condition that was met
    pub condition: AlertCondition,
    /// Severity of the alert
    pub severity: AlertSeverity,
    /// Human-readable description
    pub description: String,
}

impl Default for MetricsReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_creation() {
        let reporter = MetricsReporter::new();
        assert_eq!(reporter.format, ReportFormat::PlainText);
        assert!(reporter.include_details);
        assert!(!reporter.alert_thresholds.is_empty());
    }

    #[test]
    fn test_report_formats() {
        let report = PerformanceReport {
            efficiency_score: 0.85,
            tracking_performance: Default::default(),
            symbol_performance: Default::default(),
            pointer_performance: Default::default(),
            memory_efficiency: Default::default(),
            recommendations: vec!["Test recommendation".to_string()],
        };

        let text_reporter = MetricsReporter::with_format(ReportFormat::PlainText);
        let text_report = text_reporter.generate_report(&report);
        assert!(text_report.contains("Memory Analysis Performance Report"));
        assert!(text_report.contains("85.0%"));

        let json_reporter = MetricsReporter::with_format(ReportFormat::Json);
        let json_report = json_reporter.generate_report(&report);
        assert!(json_report.contains("efficiency_score"));
        assert!(json_report.contains("0.850"));

        let md_reporter = MetricsReporter::with_format(ReportFormat::Markdown);
        let md_report = md_reporter.generate_report(&report);
        assert!(md_report.contains("# Memory Analysis Performance Report"));
        assert!(md_report.contains("## Recommendations"));
    }

    #[test]
    fn test_alert_threshold() {
        let threshold = AlertThreshold {
            metric_name: "test_metric".to_string(),
            threshold: 100.0,
            condition: AlertCondition::GreaterThan,
            severity: AlertSeverity::Warning,
            description: "Test alert".to_string(),
        };

        assert_eq!(threshold.metric_name, "test_metric");
        assert_eq!(threshold.condition, AlertCondition::GreaterThan);
        assert_eq!(threshold.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }
}
