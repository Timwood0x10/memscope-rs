//! Debug logging and performance monitoring module
//!
//! This module provides comprehensive logging, debugging information,
//! and performance monitoring for the HTML generation process.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Debug logging levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Only critical errors
    Error = 0,
    /// Warnings and errors
    Warn = 1,
    /// General information
    Info = 2,
    /// Detailed debugging information
    Debug = 3,
    /// Very detailed trace information
    Trace = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN "),
            LogLevel::Info => write!(f, "INFO "),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Performance timing information
#[derive(Debug, Clone)]
pub struct TimingInfo {
    /// Operation name
    pub operation: String,
    /// Start time
    pub start_time: Instant,
    /// End time (if completed)
    pub end_time: Option<Instant>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TimingInfo {
    /// Create a new timing info for an operation
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
            end_time: None,
            duration_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Complete the timing measurement
    pub fn complete(&mut self) {
        let end_time = Instant::now();
        let duration = end_time.duration_since(self.start_time);
        self.end_time = Some(end_time);
        self.duration_ms = Some(duration.as_millis() as u64);
    }

    /// Add metadata to the timing info
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get duration in milliseconds
    pub fn get_duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }
}

/// Progress tracking information
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Current step
    pub current_step: usize,
    /// Total steps
    pub total_steps: usize,
    /// Current operation description
    pub operation: String,
    /// Items processed in current step
    pub items_processed: usize,
    /// Total items in current step
    pub total_items: usize,
    /// Start time of current operation
    pub start_time: Instant,
}

impl ProgressInfo {
    /// Create new progress info
    pub fn new(total_steps: usize, operation: String) -> Self {
        Self {
            current_step: 0,
            total_steps,
            operation,
            items_processed: 0,
            total_items: 0,
            start_time: Instant::now(),
        }
    }

    /// Update progress to next step
    pub fn next_step(&mut self, operation: String, total_items: usize) {
        self.current_step += 1;
        self.operation = operation;
        self.items_processed = 0;
        self.total_items = total_items;
        self.start_time = Instant::now();
    }

    /// Update items processed in current step
    pub fn update_items(&mut self, processed: usize) {
        self.items_processed = processed;
    }

    /// Get overall progress percentage
    pub fn get_overall_progress(&self) -> f64 {
        if self.total_steps == 0 {
            return 100.0;
        }

        let step_progress = if self.total_items > 0 {
            self.items_processed as f64 / self.total_items as f64
        } else {
            1.0
        };

        ((self.current_step as f64 + step_progress) / self.total_steps as f64) * 100.0
    }

    /// Get current step progress percentage
    pub fn get_step_progress(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        (self.items_processed as f64 / self.total_items as f64) * 100.0
    }
}

/// Debug logger configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Logging level
    pub log_level: LogLevel,
    /// Enable performance timing
    pub enable_timing: bool,
    /// Enable progress reporting
    pub enable_progress: bool,
    /// Enable memory usage tracking
    pub enable_memory_tracking: bool,
    /// Enable detailed file operations logging
    pub enable_file_ops: bool,
    /// Enable JSON processing details
    pub enable_json_details: bool,
    /// Progress reporting interval in milliseconds
    pub progress_interval_ms: u64,
    /// Output timestamps in logs
    pub include_timestamps: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            enable_timing: true,
            enable_progress: true,
            enable_memory_tracking: false,
            enable_file_ops: false,
            enable_json_details: false,
            progress_interval_ms: 1000,
            include_timestamps: true,
        }
    }
}

/// Performance statistics
#[derive(Debug, Default, Clone)]
pub struct PerformanceStats {
    /// Total processing time
    pub total_time_ms: u64,
    /// File discovery time
    pub discovery_time_ms: u64,
    /// File loading time
    pub loading_time_ms: u64,
    /// Data normalization time
    pub normalization_time_ms: u64,
    /// Data integration time
    pub integration_time_ms: u64,
    /// Template generation time
    pub template_time_ms: u64,
    /// Number of files processed
    pub files_processed: usize,
    /// Total data size processed
    pub data_size_bytes: usize,
    /// Memory peak usage
    pub peak_memory_bytes: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
}

impl PerformanceStats {
    /// Calculate throughput in MB/s
    pub fn get_throughput_mb_per_sec(&self) -> f64 {
        if self.total_time_ms == 0 {
            return 0.0;
        }
        (self.data_size_bytes as f64 / 1024.0 / 1024.0) / (self.total_time_ms as f64 / 1000.0)
    }

    /// Get processing efficiency score (0-100)
    pub fn get_efficiency_score(&self) -> f64 {
        let base_score = 100.0;
        let error_penalty = (self.error_count as f64) * 10.0;
        let warning_penalty = (self.warning_count as f64) * 2.0;

        (base_score - error_penalty - warning_penalty).max(0.0)
    }
}

/// Debug logger for HTML generation
pub struct DebugLogger {
    /// Configuration
    config: DebugConfig,
    /// Active timing operations
    active_timings: Arc<Mutex<HashMap<String, TimingInfo>>>,
    /// Completed timing operations
    completed_timings: Arc<Mutex<Vec<TimingInfo>>>,
    /// Progress information
    progress: Arc<Mutex<Option<ProgressInfo>>>,
    /// Performance statistics
    stats: Arc<Mutex<PerformanceStats>>,
    /// Start time of the logger
    start_time: Instant,
    /// Last progress report time
    last_progress_time: Arc<Mutex<Instant>>,
}

impl DebugLogger {
    /// Create a new debug logger with default configuration
    pub fn new() -> Self {
        Self::with_config(DebugConfig::default())
    }

    /// Create a new debug logger with custom configuration
    pub fn with_config(config: DebugConfig) -> Self {
        Self {
            config,
            active_timings: Arc::new(Mutex::new(HashMap::new())),
            completed_timings: Arc::new(Mutex::new(Vec::new())),
            progress: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(PerformanceStats::default())),
            start_time: Instant::now(),
            last_progress_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, message: &str) {
        if level <= self.config.log_level {
            let timestamp = if self.config.include_timestamps {
                let elapsed = self.start_time.elapsed();
                format!("[{:>8.3}s] ", elapsed.as_secs_f64())
            } else {
                String::new()
            };

            let level_icon = match level {
                LogLevel::Error => "❌",
                LogLevel::Warn => "⚠️ ",
                LogLevel::Info => "ℹ️ ",
                LogLevel::Debug => "🔍",
                LogLevel::Trace => "🔎",
            };

            tracing::info!("{}{} {} {}", timestamp, level_icon, level, message);
        }
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
        if let Ok(mut stats) = self.stats.lock() {
            stats.error_count += 1;
        }
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
        if let Ok(mut stats) = self.stats.lock() {
            stats.warning_count += 1;
        }
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log a trace message
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// Start timing an operation
    pub fn start_timing(&self, operation: &str) -> String {
        if !self.config.enable_timing {
            return operation.to_string();
        }

        let timing_id = format!("{}_{}", operation, self.start_time.elapsed().as_millis());
        let timing_info = TimingInfo::new(operation.to_string());

        if let Ok(mut timings) = self.active_timings.lock() {
            timings.insert(timing_id.clone(), timing_info);
        }

        self.debug(&format!("Started timing: {operation}"));
        timing_id
    }

    /// End timing an operation
    pub fn end_timing(&self, timing_id: &str) -> Option<u64> {
        if !self.config.enable_timing {
            return None;
        }

        let duration = if let Ok(mut active) = self.active_timings.lock() {
            if let Some(mut timing) = active.remove(timing_id) {
                timing.complete();
                let duration = timing.get_duration_ms();

                self.debug(&format!(
                    "Completed timing: {} in {}ms",
                    timing.operation,
                    duration.unwrap_or(0)
                ));

                if let Ok(mut completed) = self.completed_timings.lock() {
                    completed.push(timing);
                }

                duration
            } else {
                None
            }
        } else {
            None
        };

        duration
    }

    /// Start progress tracking
    pub fn start_progress(&self, total_steps: usize, initial_operation: &str) {
        if !self.config.enable_progress {
            return;
        }

        let progress_info = ProgressInfo::new(total_steps, initial_operation.to_string());

        if let Ok(mut progress) = self.progress.lock() {
            *progress = Some(progress_info);
        }

        self.info(&format!("Started progress tracking: {total_steps} steps"));
    }

    /// Update progress to next step
    pub fn next_progress_step(&self, operation: &str, total_items: usize) {
        if !self.config.enable_progress {
            return;
        }

        if let Ok(mut progress) = self.progress.lock() {
            if let Some(ref mut prog) = *progress {
                prog.next_step(operation.to_string(), total_items);
                self.info(&format!(
                    "Progress: Step {}/{} - {operation}",
                    prog.current_step, prog.total_steps
                ));
            }
        }
    }

    /// Update items processed in current step
    pub fn update_progress_items(&self, processed: usize) {
        if !self.config.enable_progress {
            return;
        }

        let should_report = if let Ok(mut last_time) = self.last_progress_time.lock() {
            let now = Instant::now();
            let elapsed = now.duration_since(*last_time);
            if elapsed.as_millis() >= self.config.progress_interval_ms as u128 {
                *last_time = now;
                true
            } else {
                false
            }
        } else {
            false
        };

        if let Ok(mut progress) = self.progress.lock() {
            if let Some(ref mut prog) = *progress {
                prog.update_items(processed);

                if should_report {
                    let overall = prog.get_overall_progress();
                    let step = prog.get_step_progress();
                    self.debug(&format!(
                        "Progress: {overall:.1}% overall, {step:.1}% current step ({}/{})",
                        prog.items_processed, prog.total_items
                    ));
                }
            }
        }
    }

    /// Log file operation
    pub fn log_file_operation(&self, operation: &str, file_path: &str, size_bytes: Option<usize>) {
        if !self.config.enable_file_ops {
            return;
        }

        let size_info = if let Some(size) = size_bytes {
            format!(" ({:.1} KB)", size as f64 / 1024.0)
        } else {
            String::new()
        };

        self.debug(&format!("File {operation}: {file_path}{size_info}"));
    }

    /// Log JSON processing details
    pub fn log_json_processing(
        &self,
        file_type: &str,
        objects_count: usize,
        processing_time_ms: u64,
    ) {
        if !self.config.enable_json_details {
            return;
        }

        self.debug(&format!(
            "JSON processing: {file_type} - {objects_count} objects in {processing_time_ms}ms",
        ));
    }

    /// Update performance statistics
    pub fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut PerformanceStats),
    {
        if let Ok(mut stats) = self.stats.lock() {
            updater(&mut stats);
        }
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            PerformanceStats::default()
        }
    }

    /// Print comprehensive performance report
    pub fn print_performance_report(&self) {
        if !self.config.enable_timing {
            return;
        }

        let stats = self.get_stats();
        let total_elapsed = self.start_time.elapsed().as_millis() as u64;

        tracing::info!("\n📊 Performance Report:");
        tracing::info!("   Total time: {}ms", total_elapsed);
        tracing::info!(
            "   Discovery: {}ms ({:.1}%)",
            stats.discovery_time_ms,
            (stats.discovery_time_ms as f64 / total_elapsed as f64) * 100.0
        );
        tracing::info!(
            "   Loading: {}ms ({:.1}%)",
            stats.loading_time_ms,
            (stats.loading_time_ms as f64 / total_elapsed as f64) * 100.0
        );
        tracing::info!(
            "   Normalization: {}ms ({:.1}%)",
            stats.normalization_time_ms,
            (stats.normalization_time_ms as f64 / total_elapsed as f64) * 100.0
        );
        tracing::info!(
            "   Integration: {}ms ({:.1}%)",
            stats.integration_time_ms,
            (stats.integration_time_ms as f64 / total_elapsed as f64) * 100.0
        );
        tracing::info!(
            "   Template: {}ms ({:.1}%)",
            stats.template_time_ms,
            (stats.template_time_ms as f64 / total_elapsed as f64) * 100.0
        );

        tracing::info!("\n📈 Processing Statistics:");
        tracing::info!("   Files processed: {}", stats.files_processed);
        tracing::info!(
            "   Data size: {:.1} MB",
            stats.data_size_bytes as f64 / 1024.0 / 1024.0
        );
        tracing::info!(
            "   Throughput: {:.1} MB/s",
            stats.get_throughput_mb_per_sec()
        );
        tracing::info!(
            "   Efficiency score: {:.1}/100",
            stats.get_efficiency_score()
        );

        if stats.error_count > 0 || stats.warning_count > 0 {
            tracing::info!("\n⚠️  Issues:");
            if stats.error_count > 0 {
                tracing::info!("   Errors: {}", stats.error_count);
            }
            if stats.warning_count > 0 {
                tracing::info!("   Warnings: {}", stats.warning_count);
            }
        }

        // Print detailed timing breakdown
        if let Ok(completed) = self.completed_timings.lock() {
            if !completed.is_empty() {
                tracing::info!("\n🔍 Detailed Timing Breakdown:");
                for timing in completed.iter() {
                    if let Some(duration) = timing.duration_ms {
                        tracing::info!("   {}: {}ms", timing.operation, duration);
                        for (key, value) in &timing.metadata {
                            tracing::info!("     {}: {}", key, value);
                        }
                    }
                }
            }
        }
    }

    /// Print memory usage information
    pub fn print_memory_info(&self) {
        if !self.config.enable_memory_tracking {
            return;
        }

        let stats = self.get_stats();
        if stats.peak_memory_bytes > 0 {
            tracing::info!("\n💾 Memory Usage:");
            tracing::info!(
                "   Peak memory: {:.1} MB",
                stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
            );

            if stats.data_size_bytes > 0 {
                let memory_efficiency =
                    (stats.data_size_bytes as f64 / stats.peak_memory_bytes as f64) * 100.0;
                tracing::info!("   Memory efficiency: {:.1}%", memory_efficiency);
            }
        }
    }
}

impl Default for DebugLogger {
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
    fn test_debug_logger_creation() {
        let logger = DebugLogger::new();
        assert_eq!(logger.config.log_level, LogLevel::Info);
        assert!(logger.config.enable_timing);
    }

    #[test]
    fn test_timing_operations() {
        let logger = DebugLogger::new();
        let timing_id = logger.start_timing("test_operation");

        // Simulate some work
        thread::sleep(Duration::from_millis(10));

        let duration = logger.end_timing(&timing_id);
        assert!(duration.is_some());
        assert!(duration.expect("Failed to get duration") >= 10);
    }

    #[test]
    fn test_progress_tracking() {
        let logger = DebugLogger::new();
        logger.start_progress(3, "Initial operation");

        logger.next_progress_step("Step 1", 10);
        logger.update_progress_items(5);

        {
            let progress = logger.progress.lock().expect("Failed to acquire lock");
            if let Some(ref prog) = *progress {
                assert_eq!(prog.current_step, 1);
                assert_eq!(prog.items_processed, 5);
                assert_eq!(prog.total_items, 10);
                assert_eq!(prog.get_step_progress(), 50.0);
            }
        }
    }

    #[test]
    fn test_performance_stats() {
        let logger = DebugLogger::new();

        logger.update_stats(|stats| {
            stats.files_processed = 5;
            stats.data_size_bytes = 1024 * 1024; // 1MB
            stats.total_time_ms = 1000; // 1 second
        });

        let stats = logger.get_stats();
        assert_eq!(stats.files_processed, 5);
        assert_eq!(stats.get_throughput_mb_per_sec(), 1.0);
    }

    #[test]
    fn test_log_levels() {
        let config = DebugConfig {
            log_level: LogLevel::Debug,
            ..Default::default()
        };
        let logger = DebugLogger::with_config(config);

        // These should not panic
        logger.error("Test error");
        logger.warn("Test warning");
        logger.info("Test info");
        logger.debug("Test debug");
        logger.trace("Test trace"); // This should be filtered out
    }
}
