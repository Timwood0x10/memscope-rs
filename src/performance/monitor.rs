use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Type alias for alert handler function
type AlertHandler = Box<dyn Fn(&Alert) + Send + Sync>;

/// Configuration for the performance monitor
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub check_interval: Duration,
    pub memory_threshold_mb: usize,
    pub allocation_rate_threshold: f64,
    pub cpu_threshold: f64,
    pub contention_threshold: f64,
    pub enable_auto_alerts: bool,
    pub enable_auto_mitigation: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            memory_threshold_mb: 100,
            allocation_rate_threshold: 1000.0,
            cpu_threshold: 0.8,
            contention_threshold: 0.1,
            enable_auto_alerts: true,
            enable_auto_mitigation: false,
        }
    }
}

/// Alert levels for performance issues
#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AlertLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Critical => "CRITICAL",
            AlertLevel::Emergency => "EMERGENCY",
        }
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub metric: String,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
    pub timestamp: Instant,
    pub suggested_actions: Vec<String>,
}

impl Alert {
    pub fn new(
        metric: &str,
        level: AlertLevel,
        current_value: f64,
        threshold: f64,
        message: &str,
    ) -> Self {
        let id = format!("{}_{}", metric, chrono::Utc::now().timestamp_millis());

        Self {
            id,
            level,
            metric: metric.to_string(),
            current_value,
            threshold,
            message: message.to_string(),
            timestamp: Instant::now(),
            suggested_actions: Vec::new(),
        }
    }

    pub fn with_suggestions(mut self, actions: Vec<String>) -> Self {
        self.suggested_actions = actions;
        self
    }

    pub fn format_message(&self) -> String {
        format!(
            "[{}] {} - {}: {:.2} (threshold: {:.2}){}",
            self.level.as_str(),
            self.metric,
            self.message,
            self.current_value,
            self.threshold,
            if self.suggested_actions.is_empty() {
                String::new()
            } else {
                format!(
                    "\nSuggested actions:\n{}",
                    self.suggested_actions
                        .iter()
                        .map(|a| format!("  - {}", a))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        )
    }
}

/// Performance monitoring system
pub struct PerformanceMonitor {
    config: MonitorConfig,
    alerts: Arc<Mutex<Vec<Alert>>>,
    is_running: Arc<Mutex<bool>>,
    alert_handlers: Vec<AlertHandler>,
    background_handle: Option<thread::JoinHandle<()>>,
    last_check: Option<Instant>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            alerts: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            alert_handlers: Vec::new(),
            background_handle: None,
            last_check: None,
        }
    }

    /// Add an alert handler
    pub fn add_alert_handler<F>(&mut self, handler: F)
    where
        F: Fn(&Alert) + Send + Sync + 'static,
    {
        self.alert_handlers.push(Box::new(handler));
    }

    /// Start monitoring
    pub fn start(&mut self) -> Result<(), MonitorError> {
        {
            let mut running = self
                .is_running
                .lock()
                .map_err(|_| MonitorError::LockError)?;
            if *running {
                return Err(MonitorError::AlreadyRunning);
            }
            *running = true;
        }

        self.last_check = Some(Instant::now());

        // Start background monitoring thread
        let alerts_clone = Arc::clone(&self.alerts);
        let is_running_clone = Arc::clone(&self.is_running);
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            Self::monitoring_loop(alerts_clone, is_running_clone, config);
        });

        self.background_handle = Some(handle);
        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&mut self) -> Result<(), MonitorError> {
        {
            let mut running = self
                .is_running
                .lock()
                .map_err(|_| MonitorError::LockError)?;
            if !*running {
                return Err(MonitorError::NotRunning);
            }
            *running = false;
        }

        if let Some(handle) = self.background_handle.take() {
            handle.join().map_err(|_| MonitorError::ThreadJoinError)?;
        }

        Ok(())
    }

    /// Check if monitor is running
    pub fn is_running(&self) -> bool {
        self.is_running.lock().map(|r| *r).unwrap_or(false)
    }

    /// Get all alerts
    pub fn get_alerts(&self) -> Vec<Alert> {
        self.alerts.lock().map(|a| a.clone()).unwrap_or_default()
    }

    /// Get alerts by level
    pub fn get_alerts_by_level(&self, level: AlertLevel) -> Vec<Alert> {
        self.get_alerts()
            .into_iter()
            .filter(|a| a.level == level)
            .collect()
    }

    /// Clear all alerts
    pub fn clear_alerts(&self) {
        if let Ok(mut alerts) = self.alerts.lock() {
            alerts.clear();
        }
    }

    /// Clear alerts older than the specified duration
    pub fn clear_old_alerts(&self, older_than: Duration) {
        if let Ok(mut alerts) = self.alerts.lock() {
            let cutoff = Instant::now() - older_than;
            alerts.retain(|alert| alert.timestamp > cutoff);
        }
    }

    /// Manually trigger a performance check
    pub fn check_performance(&self) -> Vec<Alert> {
        Self::perform_checks(&self.config)
    }

    /// Main monitoring loop
    fn monitoring_loop(
        alerts: Arc<Mutex<Vec<Alert>>>,
        is_running: Arc<Mutex<bool>>,
        config: MonitorConfig,
    ) {
        while Self::should_continue(&is_running) {
            let new_alerts = Self::perform_checks(&config);

            // Add new alerts
            if !new_alerts.is_empty() {
                if let Ok(mut alerts_guard) = alerts.lock() {
                    alerts_guard.extend(new_alerts);

                    // Trim old alerts to prevent unbounded growth
                    if alerts_guard.len() > 1000 {
                        alerts_guard.drain(0..500); // Remove oldest 500
                    }
                }
            }

            thread::sleep(config.check_interval);
        }
    }

    /// Check if monitoring should continue
    fn should_continue(is_running: &Arc<Mutex<bool>>) -> bool {
        is_running.lock().map(|r| *r).unwrap_or(false)
    }

    /// Perform all performance checks
    fn perform_checks(config: &MonitorConfig) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let stats = crate::performance::get_global_stats();

        // Memory usage check
        let estimated_memory_mb = (stats
            .allocations_tracked
            .saturating_sub(stats.deallocations_tracked)
            * 1024)
            / (1024 * 1024);
        if estimated_memory_mb as usize > config.memory_threshold_mb {
            let alert = Alert::new(
                "memory_usage",
                if estimated_memory_mb as usize > config.memory_threshold_mb * 2 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                },
                estimated_memory_mb as f64,
                config.memory_threshold_mb as f64,
                "High memory usage detected",
            )
            .with_suggestions(vec![
                "Consider running garbage collection".to_string(),
                "Check for memory leaks".to_string(),
                "Reduce allocation rate".to_string(),
            ]);
            alerts.push(alert);
        }

        // Allocation rate check
        let allocation_rate = stats.tracking_rate();
        if allocation_rate > config.allocation_rate_threshold {
            let alert = Alert::new(
                "allocation_rate",
                if allocation_rate > config.allocation_rate_threshold * 2.0 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                },
                allocation_rate,
                config.allocation_rate_threshold,
                "High allocation rate detected",
            )
            .with_suggestions(vec![
                "Implement object pooling".to_string(),
                "Reduce temporary allocations".to_string(),
                "Use stack allocation where possible".to_string(),
            ]);
            alerts.push(alert);
        }

        // Contention check
        let contention_ratio = stats.contention_ratio();
        if contention_ratio > config.contention_threshold {
            let alert = Alert::new(
                "lock_contention",
                if contention_ratio > config.contention_threshold * 2.0 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                },
                contention_ratio,
                config.contention_threshold,
                "High lock contention detected",
            )
            .with_suggestions(vec![
                "Reduce lock scope".to_string(),
                "Use lock-free data structures".to_string(),
                "Implement backoff strategies".to_string(),
            ]);
            alerts.push(alert);
        }

        // Cache performance check
        let cache_hit_ratio = stats.cache_hit_ratio();
        if cache_hit_ratio < 0.8 {
            let alert = Alert::new(
                "cache_performance",
                if cache_hit_ratio < 0.5 {
                    AlertLevel::Warning
                } else {
                    AlertLevel::Info
                },
                cache_hit_ratio,
                0.8,
                "Low cache hit ratio detected",
            )
            .with_suggestions(vec![
                "Increase cache size".to_string(),
                "Improve cache locality".to_string(),
                "Review access patterns".to_string(),
            ]);
            alerts.push(alert);
        }

        // System resource check
        if stats.uptime > Duration::from_secs(24 * 3600) {
            let uptime_hours = stats.uptime.as_secs() as f64 / 3600.0;
            if stats.tracking_rate() < 10.0 && stats.allocations_tracked > 100000 {
                let alert = Alert::new(
                    "performance_degradation",
                    AlertLevel::Info,
                    uptime_hours,
                    24.0,
                    "Potential performance degradation detected after long runtime",
                )
                .with_suggestions(vec![
                    "Consider restarting the application".to_string(),
                    "Check for resource leaks".to_string(),
                    "Review memory fragmentation".to_string(),
                ]);
                alerts.push(alert);
            }
        }

        alerts
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        if self.is_running() {
            let _ = self.stop();
        }
    }
}

/// Monitor errors
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Monitor is already running")]
    AlreadyRunning,

    #[error("Monitor is not running")]
    NotRunning,

    #[error("Lock error occurred")]
    LockError,

    #[error("Thread join error")]
    ThreadJoinError,
}

/// Builder for creating monitors with common configurations
pub struct MonitorBuilder {
    config: MonitorConfig,
    handlers: Vec<AlertHandler>,
}

impl MonitorBuilder {
    pub fn new() -> Self {
        Self {
            config: MonitorConfig::default(),
            handlers: Vec::new(),
        }
    }

    pub fn check_interval(mut self, interval: Duration) -> Self {
        self.config.check_interval = interval;
        self
    }

    pub fn memory_threshold_mb(mut self, threshold: usize) -> Self {
        self.config.memory_threshold_mb = threshold;
        self
    }

    pub fn allocation_rate_threshold(mut self, threshold: f64) -> Self {
        self.config.allocation_rate_threshold = threshold;
        self
    }

    pub fn with_console_alerts(mut self) -> Self {
        self.handlers.push(Box::new(|alert: &Alert| {
            println!("{}", alert.format_message());
        }));
        self
    }

    pub fn with_log_alerts(mut self) -> Self {
        self.handlers
            .push(Box::new(|alert: &Alert| match alert.level {
                AlertLevel::Info => tracing::info!("{}", alert.format_message()),
                AlertLevel::Warning => tracing::warn!("{}", alert.format_message()),
                AlertLevel::Critical => tracing::error!("{}", alert.format_message()),
                AlertLevel::Emergency => tracing::error!("{}", alert.format_message()),
            }));
        self
    }

    pub fn with_custom_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&Alert) + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn build(self) -> PerformanceMonitor {
        let mut monitor = PerformanceMonitor::new(self.config);
        for handler in self.handlers {
            monitor.add_alert_handler(handler);
        }
        monitor
    }
}

impl Default for MonitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            "test_metric",
            AlertLevel::Warning,
            150.0,
            100.0,
            "Test alert message",
        )
        .with_suggestions(vec!["Action 1".to_string(), "Action 2".to_string()]);

        assert_eq!(alert.metric, "test_metric");
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.current_value, 150.0);
        assert_eq!(alert.threshold, 100.0);
        assert_eq!(alert.suggested_actions.len(), 2);
    }

    #[test]
    fn test_alert_formatting() {
        let alert = Alert::new(
            "memory_usage",
            AlertLevel::Critical,
            200.0,
            100.0,
            "Memory usage too high",
        )
        .with_suggestions(vec!["Free some memory".to_string()]);

        let formatted = alert.format_message();
        assert!(formatted.contains("CRITICAL"));
        assert!(formatted.contains("memory_usage"));
        assert!(formatted.contains("200.00"));
        assert!(formatted.contains("100.00"));
        assert!(formatted.contains("Free some memory"));
    }

    #[test]
    fn test_monitor_lifecycle() {
        let mut monitor = PerformanceMonitor::new(MonitorConfig {
            check_interval: Duration::from_millis(50),
            ..Default::default()
        });

        assert!(!monitor.is_running());

        monitor.start().unwrap();
        assert!(monitor.is_running());

        thread::sleep(Duration::from_millis(100));

        monitor.stop().unwrap();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_monitor_builder() {
        let monitor = MonitorBuilder::new()
            .check_interval(Duration::from_millis(100))
            .memory_threshold_mb(50)
            .allocation_rate_threshold(500.0)
            .with_console_alerts()
            .with_log_alerts()
            .build();

        assert_eq!(monitor.config.check_interval, Duration::from_millis(100));
        assert_eq!(monitor.config.memory_threshold_mb, 50);
        assert_eq!(monitor.config.allocation_rate_threshold, 500.0);
        assert_eq!(monitor.alert_handlers.len(), 2);
    }

    #[test]
    fn test_alert_levels() {
        assert_eq!(AlertLevel::Info.as_str(), "INFO");
        assert_eq!(AlertLevel::Warning.as_str(), "WARNING");
        assert_eq!(AlertLevel::Critical.as_str(), "CRITICAL");
        assert_eq!(AlertLevel::Emergency.as_str(), "EMERGENCY");
    }

    #[test]
    fn test_performance_checks() {
        // Set up some global stats to trigger alerts
        let counter = crate::performance::global_counter();
        counter.reset();

        // Simulate high memory usage
        for _ in 0..1000000 {
            counter.record_allocation();
        }

        let config = MonitorConfig {
            memory_threshold_mb: 1, // Very low threshold to trigger alert
            allocation_rate_threshold: 1.0,
            contention_threshold: 0.01,
            ..Default::default()
        };

        let alerts = PerformanceMonitor::perform_checks(&config);
        assert!(!alerts.is_empty());

        // Should have memory usage alert
        assert!(alerts.iter().any(|a| a.metric == "memory_usage"));
    }
}
