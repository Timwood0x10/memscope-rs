use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// High-performance metrics collection system
pub struct MetricsCollector {
    /// Current metric values
    metrics: HashMap<String, Metric>,
    /// Collection start time
    start_time: Instant,
    /// Whether collection is enabled
    enabled: bool,
    /// Sample rate for performance metrics (0.0 to 1.0)
    sample_rate: f64,
}

/// Individual metric with metadata
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name identifier
    pub name: String,
    /// Type of metric
    pub metric_type: MetricType,
    /// Current value
    pub value: MetricValue,
    /// Unit of measurement
    pub unit: String,
    /// Description of what this metric measures
    pub description: String,
    /// When this metric was last updated
    pub last_updated: Instant,
    /// Number of times this metric was updated
    pub update_count: u64,
}

/// Types of metrics supported
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    /// Simple counter that only increases
    Counter,
    /// Value that can go up or down
    Gauge,
    /// Histogram for distribution analysis
    Histogram,
    /// Timer for duration measurements
    Timer,
    /// Rate measurements (events per second)
    Rate,
}

/// Metric value storage
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Integer counter value
    Counter(Arc<AtomicU64>),
    /// Floating point gauge value
    Gauge(f64),
    /// Histogram buckets and statistics
    Histogram(HistogramData),
    /// Timer duration measurements
    Timer(TimerData),
    /// Rate calculation data
    Rate(RateData),
}

/// Histogram data structure
#[derive(Debug, Clone)]
pub struct HistogramData {
    /// Histogram buckets with upper bounds and counts
    pub buckets: Vec<(f64, u64)>,
    /// Total number of observations
    pub count: u64,
    /// Sum of all observed values
    pub sum: f64,
    /// Minimum observed value
    pub min: f64,
    /// Maximum observed value
    pub max: f64,
}

/// Timer measurement data
#[derive(Debug, Clone)]
pub struct TimerData {
    /// Total duration accumulated
    pub total_duration: Duration,
    /// Number of timing measurements
    pub count: u64,
    /// Minimum duration observed
    pub min_duration: Duration,
    /// Maximum duration observed
    pub max_duration: Duration,
    /// Recent durations for percentile calculation
    pub recent_durations: Vec<Duration>,
}

/// Rate calculation data
#[derive(Debug, Clone)]
pub struct RateData {
    /// Total events counted
    pub total_events: u64,
    /// Time window for rate calculation
    pub window_duration: Duration,
    /// Event timestamps within current window
    pub event_times: Vec<Instant>,
    /// Current calculated rate (events per second)
    pub current_rate: f64,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Instant::now(),
            enabled: true,
            sample_rate: 1.0,
        }
    }

    /// Create collector with custom sample rate
    pub fn with_sample_rate(sample_rate: f64) -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Instant::now(),
            enabled: true,
            sample_rate: sample_rate.clamp(0.0, 1.0),
        }
    }

    /// Enable or disable metrics collection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if metrics collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Increment a counter metric
    pub fn increment_counter(&mut self, name: &str, value: u64) {
        if !self.should_sample() {
            return;
        }

        let metric = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric::new_counter(name, "Number of events"));

        if let MetricValue::Counter(counter) = &metric.value {
            counter.fetch_add(value, Ordering::Relaxed);
            metric.last_updated = Instant::now();
            metric.update_count += 1;
        }
    }

    /// Set a gauge metric value
    pub fn set_gauge(&mut self, name: &str, value: f64, unit: &str) {
        if !self.should_sample() {
            return;
        }

        let metric = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric::new_gauge(name, unit, "Current value measurement"));

        if let MetricValue::Gauge(ref mut gauge_value) = metric.value {
            *gauge_value = value;
            metric.last_updated = Instant::now();
            metric.update_count += 1;
        }
    }

    /// Record histogram observation
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        if !self.should_sample() {
            return;
        }

        let metric = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric::new_histogram(name, "Distribution of values"));

        if let MetricValue::Histogram(ref mut hist) = metric.value {
            hist.observe(value);
            metric.last_updated = Instant::now();
            metric.update_count += 1;
        }
    }

    /// Record timer measurement
    pub fn record_timer(&mut self, name: &str, duration: Duration) {
        if !self.should_sample() {
            return;
        }

        let metric = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric::new_timer(name, "Duration measurements"));

        if let MetricValue::Timer(ref mut timer) = metric.value {
            timer.record(duration);
            metric.last_updated = Instant::now();
            metric.update_count += 1;
        }
    }

    /// Record rate event
    pub fn record_rate_event(&mut self, name: &str) {
        if !self.should_sample() {
            return;
        }

        let metric = self
            .metrics
            .entry(name.to_string())
            .or_insert_with(|| Metric::new_rate(name, "Events per second"));

        if let MetricValue::Rate(ref mut rate) = metric.value {
            rate.record_event();
            metric.last_updated = Instant::now();
            metric.update_count += 1;
        }
    }

    /// Get current value of a metric
    pub fn get_metric(&self, name: &str) -> Option<&Metric> {
        self.metrics.get(name)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, Metric> {
        &self.metrics
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> MetricsSummary {
        let total_metrics = self.metrics.len();
        let active_metrics = self
            .metrics
            .values()
            .filter(|m| m.last_updated.elapsed() < Duration::from_secs(60))
            .count();

        let total_updates: u64 = self.metrics.values().map(|m| m.update_count).sum();

        let uptime = self.start_time.elapsed();
        let update_rate = if uptime.as_secs() > 0 {
            total_updates as f64 / uptime.as_secs_f64()
        } else {
            0.0
        };

        MetricsSummary {
            total_metrics,
            active_metrics,
            total_updates,
            update_rate,
            uptime,
            sample_rate: self.sample_rate,
        }
    }

    /// Clear all metrics
    pub fn clear_metrics(&mut self) {
        self.metrics.clear();
    }

    /// Clean up old metrics
    pub fn cleanup_old_metrics(&mut self, max_age: Duration) {
        let cutoff_time = Instant::now() - max_age;
        self.metrics
            .retain(|_, metric| metric.last_updated > cutoff_time);
    }

    fn should_sample(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if self.sample_rate >= 1.0 {
            return true;
        }

        // Simple sampling based on system time
        let sample_decision = (Instant::now().elapsed().as_nanos() % 1000) as f64 / 1000.0;
        sample_decision < self.sample_rate
    }
}

impl Metric {
    /// Create new counter metric
    pub fn new_counter(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(Arc::new(AtomicU64::new(0))),
            unit: "count".to_string(),
            description: description.to_string(),
            last_updated: Instant::now(),
            update_count: 0,
        }
    }

    /// Create new gauge metric
    pub fn new_gauge(name: &str, unit: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(0.0),
            unit: unit.to_string(),
            description: description.to_string(),
            last_updated: Instant::now(),
            update_count: 0,
        }
    }

    /// Create new histogram metric
    pub fn new_histogram(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            metric_type: MetricType::Histogram,
            value: MetricValue::Histogram(HistogramData::new()),
            unit: "distribution".to_string(),
            description: description.to_string(),
            last_updated: Instant::now(),
            update_count: 0,
        }
    }

    /// Create new timer metric
    pub fn new_timer(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            metric_type: MetricType::Timer,
            value: MetricValue::Timer(TimerData::new()),
            unit: "duration".to_string(),
            description: description.to_string(),
            last_updated: Instant::now(),
            update_count: 0,
        }
    }

    /// Create new rate metric
    pub fn new_rate(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            metric_type: MetricType::Rate,
            value: MetricValue::Rate(RateData::new()),
            unit: "events/sec".to_string(),
            description: description.to_string(),
            last_updated: Instant::now(),
            update_count: 0,
        }
    }

    /// Get current metric value as string
    pub fn value_string(&self) -> String {
        match &self.value {
            MetricValue::Counter(counter) => counter.load(Ordering::Relaxed).to_string(),
            MetricValue::Gauge(value) => format!("{:.2}", value),
            MetricValue::Histogram(hist) => {
                format!("count={}, avg={:.2}", hist.count, hist.average())
            }
            MetricValue::Timer(timer) => {
                format!("avg={:.2}ms", timer.average_duration().as_millis())
            }
            MetricValue::Rate(rate) => format!("{:.2}/sec", rate.current_rate),
        }
    }
}

impl HistogramData {
    /// Create new histogram with default buckets
    pub fn new() -> Self {
        let buckets = vec![
            (0.001, 0),
            (0.005, 0),
            (0.01, 0),
            (0.025, 0),
            (0.05, 0),
            (0.1, 0),
            (0.25, 0),
            (0.5, 0),
            (1.0, 0),
            (2.5, 0),
            (5.0, 0),
            (10.0, 0),
            (f64::INFINITY, 0),
        ];

        Self {
            buckets,
            count: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// Record an observation
    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        // Update buckets
        for (upper_bound, count) in &mut self.buckets {
            if value <= *upper_bound {
                *count += 1;
            }
        }
    }

    /// Calculate average value
    pub fn average(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }
}

impl TimerData {
    /// Create new timer data
    pub fn new() -> Self {
        Self {
            total_duration: Duration::ZERO,
            count: 0,
            min_duration: Duration::from_secs(u64::MAX),
            max_duration: Duration::ZERO,
            recent_durations: Vec::new(),
        }
    }

    /// Record a duration measurement
    pub fn record(&mut self, duration: Duration) {
        self.total_duration += duration;
        self.count += 1;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);

        // Keep recent durations for percentile calculations
        self.recent_durations.push(duration);
        if self.recent_durations.len() > 1000 {
            self.recent_durations.drain(0..500); // Keep last 500
        }
    }

    /// Calculate average duration
    pub fn average_duration(&self) -> Duration {
        if self.count > 0 {
            self.total_duration / self.count as u32
        } else {
            Duration::ZERO
        }
    }
}

impl RateData {
    /// Create new rate data
    pub fn new() -> Self {
        Self {
            total_events: 0,
            window_duration: Duration::from_secs(60),
            event_times: Vec::new(),
            current_rate: 0.0,
        }
    }

    /// Record an event occurrence
    pub fn record_event(&mut self) {
        let now = Instant::now();
        self.total_events += 1;
        self.event_times.push(now);

        // Clean old events outside window
        let cutoff = now - self.window_duration;
        self.event_times.retain(|&time| time > cutoff);

        // Calculate current rate
        self.current_rate = self.event_times.len() as f64 / self.window_duration.as_secs_f64();
    }
}

/// Summary of metrics collection performance
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Total number of metrics tracked
    pub total_metrics: usize,
    /// Number of recently active metrics
    pub active_metrics: usize,
    /// Total metric updates performed
    pub total_updates: u64,
    /// Rate of metric updates per second
    pub update_rate: f64,
    /// How long collector has been running
    pub uptime: Duration,
    /// Current sampling rate
    pub sample_rate: f64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_metric() {
        let mut collector = MetricsCollector::new();

        collector.increment_counter("test_counter", 5);
        collector.increment_counter("test_counter", 3);

        let metric = collector
            .get_metric("test_counter")
            .expect("Metric should exist");
        if let MetricValue::Counter(counter) = &metric.value {
            assert_eq!(counter.load(Ordering::Relaxed), 8);
        } else {
            panic!("Expected counter metric");
        }
    }

    #[test]
    fn test_gauge_metric() {
        let mut collector = MetricsCollector::new();

        collector.set_gauge("test_gauge", 42.5, "units");

        let metric = collector
            .get_metric("test_gauge")
            .expect("Metric should exist");
        if let MetricValue::Gauge(value) = &metric.value {
            assert_eq!(*value, 42.5);
        } else {
            panic!("Expected gauge metric");
        }
    }

    #[test]
    fn test_histogram_metric() {
        let mut collector = MetricsCollector::new();

        collector.record_histogram("test_histogram", 1.0);
        collector.record_histogram("test_histogram", 2.0);
        collector.record_histogram("test_histogram", 3.0);

        let metric = collector
            .get_metric("test_histogram")
            .expect("Metric should exist");
        if let MetricValue::Histogram(hist) = &metric.value {
            assert_eq!(hist.count, 3);
            assert_eq!(hist.average(), 2.0);
        } else {
            panic!("Expected histogram metric");
        }
    }

    #[test]
    fn test_timer_metric() {
        let mut collector = MetricsCollector::new();

        collector.record_timer("test_timer", Duration::from_millis(100));
        collector.record_timer("test_timer", Duration::from_millis(200));

        let metric = collector
            .get_metric("test_timer")
            .expect("Metric should exist");
        if let MetricValue::Timer(timer) = &metric.value {
            assert_eq!(timer.count, 2);
            assert_eq!(timer.average_duration(), Duration::from_millis(150));
        } else {
            panic!("Expected timer metric");
        }
    }

    #[test]
    fn test_metrics_summary() {
        let mut collector = MetricsCollector::new();

        collector.increment_counter("counter1", 1);
        collector.set_gauge("gauge1", 10.0, "units");
        collector.record_histogram("hist1", 5.0);

        let summary = collector.get_summary();
        assert_eq!(summary.total_metrics, 3);
        assert!(summary.total_updates >= 3);
        assert_eq!(summary.sample_rate, 1.0);
    }

    #[test]
    fn test_sample_rate() {
        let collector = MetricsCollector::with_sample_rate(0.5);
        assert_eq!(collector.sample_rate, 0.5);

        let collector = MetricsCollector::with_sample_rate(1.5);
        assert_eq!(collector.sample_rate, 1.0); // Clamped
    }
}
