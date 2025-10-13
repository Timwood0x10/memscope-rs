use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Different types of metrics that can be collected
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
    Rate,
    Custom(String),
}

/// Value types for metrics
#[derive(Debug, Clone)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    Duration(Duration),
    Boolean(bool),
    String(String),
}

impl MetricValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Integer(i) => Some(*i as f64),
            MetricValue::Float(f) => Some(*f),
            MetricValue::Duration(d) => Some(d.as_secs_f64()),
            MetricValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            MetricValue::String(_) => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MetricValue::Integer(i) => Some(*i),
            MetricValue::Float(f) => Some(*f as i64),
            MetricValue::Duration(d) => Some(d.as_millis() as i64),
            MetricValue::Boolean(b) => Some(if *b { 1 } else { 0 }),
            MetricValue::String(_) => None,
        }
    }
}

/// A single metric measurement
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub timestamp: Instant,
    pub tags: HashMap<String, String>,
}

impl Metric {
    pub fn new(name: &str, metric_type: MetricType, value: MetricValue) -> Self {
        Self {
            name: name.to_string(),
            metric_type,
            value,
            timestamp: Instant::now(),
            tags: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags.extend(tags);
        self
    }
}

/// Histogram for tracking distribution of values
#[derive(Debug, Clone)]
pub struct Histogram {
    pub buckets: Vec<f64>,
    pub counts: Vec<u64>,
    pub sum: f64,
    pub count: u64,
}

impl Histogram {
    pub fn new(buckets: Vec<f64>) -> Self {
        let len = buckets.len();
        Self {
            buckets,
            counts: vec![0; len + 1], // +1 for overflow bucket
            sum: 0.0,
            count: 0,
        }
    }

    pub fn default_buckets() -> Vec<f64> {
        vec![
            0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0,
        ]
    }

    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;

        // Find the appropriate bucket
        let bucket_index = self
            .buckets
            .iter()
            .position(|&bucket| value <= bucket)
            .unwrap_or(self.buckets.len()); // Overflow bucket

        self.counts[bucket_index] += 1;
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.count == 0 {
            return None;
        }

        let target_count = (self.count as f64 * p).ceil() as u64;
        let mut cumulative = 0;

        for (i, &count) in self.counts.iter().enumerate() {
            cumulative += count;
            if cumulative >= target_count {
                return if i < self.buckets.len() {
                    Some(self.buckets[i])
                } else {
                    Some(f64::INFINITY)
                };
            }
        }

        None
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    metrics: Arc<Mutex<Vec<Metric>>>,
    histograms: Arc<Mutex<HashMap<String, Histogram>>>,
    counters: Arc<Mutex<HashMap<String, i64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    start_time: Instant,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a metric
    pub fn record_metric(&self, metric: Metric) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str, value: i64) {
        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(name.to_string()).or_insert(0) += value;
        }
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        if let Ok(mut gauges) = self.gauges.lock() {
            gauges.insert(name.to_string(), value);
        }
    }

    /// Record a histogram observation
    pub fn observe_histogram(&self, name: &str, value: f64) {
        if let Ok(mut histograms) = self.histograms.lock() {
            let histogram = histograms
                .entry(name.to_string())
                .or_insert_with(|| Histogram::new(Histogram::default_buckets()));
            histogram.observe(value);
        }
    }

    /// Record timing information
    pub fn record_timing(&self, name: &str, duration: Duration) {
        let metric = Metric::new(name, MetricType::Timer, MetricValue::Duration(duration));
        self.record_metric(metric);
        self.observe_histogram(&format!("{}_histogram", name), duration.as_secs_f64());
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<i64> {
        self.counters.lock().ok()?.get(name).copied()
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.lock().ok()?.get(name).copied()
    }

    /// Get histogram statistics
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.lock().ok()?;
        let histogram = histograms.get(name)?;

        Some(HistogramStats {
            count: histogram.count,
            sum: histogram.sum,
            mean: histogram.mean(),
            p50: histogram.percentile(0.5).unwrap_or(0.0),
            p90: histogram.percentile(0.9).unwrap_or(0.0),
            p95: histogram.percentile(0.95).unwrap_or(0.0),
            p99: histogram.percentile(0.99).unwrap_or(0.0),
        })
    }

    /// Get all metrics since a timestamp
    pub fn get_metrics_since(&self, since: Instant) -> Vec<Metric> {
        if let Ok(metrics) = self.metrics.lock() {
            metrics
                .iter()
                .filter(|metric| metric.timestamp >= since)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all current metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.get_metrics_since(self.start_time)
    }

    /// Clear all metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
        if let Ok(mut histograms) = self.histograms.lock() {
            histograms.clear();
        }
        if let Ok(mut counters) = self.counters.lock() {
            counters.clear();
        }
        if let Ok(mut gauges) = self.gauges.lock() {
            gauges.clear();
        }
    }

    /// Generate a summary report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Metrics Report\n");
        report.push_str("==========================\n\n");

        // Counters
        if let Ok(counters) = self.counters.lock() {
            if !counters.is_empty() {
                report.push_str("Counters:\n");
                for (name, value) in counters.iter() {
                    report.push_str(&format!("  {}: {}\n", name, value));
                }
                report.push('\n');
            }
        }

        // Gauges
        if let Ok(gauges) = self.gauges.lock() {
            if !gauges.is_empty() {
                report.push_str("Gauges:\n");
                for (name, value) in gauges.iter() {
                    report.push_str(&format!("  {}: {:.2}\n", name, value));
                }
                report.push('\n');
            }
        }

        // Histograms
        if let Ok(histograms) = self.histograms.lock() {
            if !histograms.is_empty() {
                report.push_str("Histograms:\n");
                for (name, histogram) in histograms.iter() {
                    report.push_str(&format!(
                        "  {}:\n    Count: {}, Sum: {:.2}, Mean: {:.2}\n    P50: {:.2}, P90: {:.2}, P95: {:.2}, P99: {:.2}\n",
                        name,
                        histogram.count,
                        histogram.sum,
                        histogram.mean(),
                        histogram.percentile(0.5).unwrap_or(0.0),
                        histogram.percentile(0.9).unwrap_or(0.0),
                        histogram.percentile(0.95).unwrap_or(0.0),
                        histogram.percentile(0.99).unwrap_or(0.0)
                    ));
                }
                report.push('\n');
            }
        }

        report
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram statistics summary
#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub mean: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Timer utility for measuring elapsed time
pub struct MetricTimer {
    name: String,
    start: Instant,
    metrics: Arc<PerformanceMetrics>,
}

impl MetricTimer {
    pub fn new(name: &str, metrics: Arc<PerformanceMetrics>) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            metrics,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn finish(self) -> Duration {
        let duration = self.elapsed();
        self.metrics.record_timing(&self.name, duration);
        duration
    }
}

impl Drop for MetricTimer {
    fn drop(&mut self) {
        let duration = self.elapsed();
        self.metrics.record_timing(&self.name, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metric_value_conversions() {
        let int_val = MetricValue::Integer(42);
        assert_eq!(int_val.as_f64(), Some(42.0));
        assert_eq!(int_val.as_i64(), Some(42));

        let float_val = MetricValue::Float(std::f64::consts::PI);
        assert_eq!(float_val.as_f64(), Some(std::f64::consts::PI));
        assert_eq!(float_val.as_i64(), Some(3));

        let bool_val = MetricValue::Boolean(true);
        assert_eq!(bool_val.as_f64(), Some(1.0));
        assert_eq!(bool_val.as_i64(), Some(1));
    }

    #[test]
    fn test_histogram() {
        let mut histogram = Histogram::new(vec![1.0, 5.0, 10.0]);

        histogram.observe(0.5);
        histogram.observe(2.0);
        histogram.observe(7.0);
        histogram.observe(15.0);

        assert_eq!(histogram.count, 4);
        assert_eq!(histogram.sum, 24.5);
        assert_eq!(histogram.mean(), 6.125);

        // Check bucket counts
        assert_eq!(histogram.counts[0], 1); // <= 1.0
        assert_eq!(histogram.counts[1], 1); // <= 5.0
        assert_eq!(histogram.counts[2], 1); // <= 10.0
        assert_eq!(histogram.counts[3], 1); // > 10.0 (overflow)
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();

        // Test counter
        metrics.increment_counter("requests", 1);
        metrics.increment_counter("requests", 2);
        assert_eq!(metrics.get_counter("requests"), Some(3));

        // Test gauge
        metrics.set_gauge("memory_usage", 0.75);
        assert_eq!(metrics.get_gauge("memory_usage"), Some(0.75));

        // Test histogram
        metrics.observe_histogram("response_time", 0.1);
        metrics.observe_histogram("response_time", 0.2);
        metrics.observe_histogram("response_time", 0.15);

        let stats = metrics.get_histogram_stats("response_time").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 0.15);
    }

    #[test]
    fn test_metric_timer() {
        let metrics = Arc::new(PerformanceMetrics::new());

        {
            let _timer = MetricTimer::new("test_operation", Arc::clone(&metrics));
            thread::sleep(Duration::from_millis(10));
        }

        let stats = metrics
            .get_histogram_stats("test_operation_histogram")
            .unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.mean >= 0.01); // At least 10ms
    }

    #[test]
    fn test_metric_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "api".to_string());
        tags.insert("version".to_string(), "1.0".to_string());

        let metric = Metric::new("requests", MetricType::Counter, MetricValue::Integer(1))
            .with_tag("endpoint", "/users")
            .with_tags(tags);

        assert_eq!(metric.tags.get("service"), Some(&"api".to_string()));
        assert_eq!(metric.tags.get("version"), Some(&"1.0".to_string()));
        assert_eq!(metric.tags.get("endpoint"), Some(&"/users".to_string()));
    }
}
