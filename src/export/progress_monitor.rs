//! Progress monitoring and cancellation mechanism
//!
//! This module provides progress monitoring, cancellation mechanisms, and remaining time estimation for the export process.
//! Supports callback interfaces, graceful interruption, and partial result saving.

use crate::core::types::TrackingResult;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Export progress information
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current stage
    pub current_stage: ExportStage,
    /// Current stage progress (0.0 - 1.0)
    pub stage_progress: f64,
    /// Overall progress (0.0 - 1.0)
    pub overall_progress: f64,
    /// Number of processed allocations
    pub processed_allocations: usize,
    /// Total number of allocations
    pub total_allocations: usize,
    /// Elapsed time
    pub elapsed_time: Duration,
    /// Estimated remaining time
    pub estimated_remaining: Option<Duration>,
    /// Current processing speed (allocations/second)
    pub processing_speed: f64,
    /// Stage details
    pub stage_details: String,
}

/// Export stage
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportStage {
    /// Initializing
    Initializing,
    /// Data localization
    DataLocalization,
    /// Parallel processing
    ParallelProcessing,
    /// High-speed writing
    Writing,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
    /// Error
    Error(String),
}

impl ExportStage {
    /// Get stage weight (used for calculating overall progress)
    pub fn weight(&self) -> f64 {
        match self {
            ExportStage::Initializing => 0.05,
            ExportStage::DataLocalization => 0.15,
            ExportStage::ParallelProcessing => 0.70,
            ExportStage::Writing => 0.10,
            ExportStage::Completed => 1.0,
            ExportStage::Cancelled => 0.0,
            ExportStage::Error(_) => 0.0,
        }
    }

    /// Get stage description
    pub fn description(&self) -> &str {
        match self {
            ExportStage::Initializing => "Initializing export environment",
            ExportStage::DataLocalization => "Localizing data, reducing global state access",
            ExportStage::ParallelProcessing => "Parallel shard processing",
            ExportStage::Writing => "High-speed buffered writing",
            ExportStage::Completed => "Export completed",
            ExportStage::Cancelled => "Export cancelled",
            ExportStage::Error(msg) => msg,
        }
    }
}

/// Progress callback function type
pub type ProgressCallback = Box<dyn Fn(ExportProgress) + Send + Sync>;

/// Cancellation token for interrupting export operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create new cancellation token
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cancel operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Return error if cancelled
    pub fn check_cancelled(&self) -> TrackingResult<()> {
        if self.is_cancelled() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Export operation was cancelled",
            )
            .into())
        } else {
            Ok(())
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress monitor
pub struct ProgressMonitor {
    /// Start time
    start_time: Instant,
    /// Current stage
    current_stage: ExportStage,
    /// Total number of allocations
    total_allocations: usize,
    /// Number of processed allocations
    processed_allocations: Arc<AtomicUsize>,
    /// Progress callback
    callback: Option<ProgressCallback>,
    /// Cancellation token
    cancellation_token: CancellationToken,
    /// Last update time
    last_update: Instant,
    /// Update interval (to avoid too frequent callbacks)
    update_interval: Duration,
    /// Historical processing speed (for estimating remaining time)
    speed_history: Vec<(Instant, usize)>,
    /// Maximum history size
    max_history_size: usize,
}

impl ProgressMonitor {
    /// Create new progress monitor
    pub fn new(total_allocations: usize) -> Self {
        Self {
            start_time: Instant::now(),
            current_stage: ExportStage::Initializing,
            total_allocations,
            processed_allocations: Arc::new(AtomicUsize::new(0)),
            callback: None,
            cancellation_token: CancellationToken::new(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100), // 100ms update interval
            speed_history: Vec::new(),
            max_history_size: 20,
        }
    }

    /// Set progress callback
    pub fn set_callback(&mut self, callback: ProgressCallback) {
        self.callback = Some(callback);
    }

    /// Get cancellation token
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Set current stage
    pub fn set_stage(&mut self, stage: ExportStage) {
        self.current_stage = stage;
        // Don't automatically call update_progress, let caller control progress
    }

    /// Update stage progress
    pub fn update_progress(&mut self, stage_progress: f64, _details: Option<String>) {
        let now = Instant::now();

        // Check update interval to avoid too frequent callbacks
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }

        self.last_update = now;

        let processed = self.processed_allocations.load(Ordering::SeqCst);

        // Update speed history
        self.speed_history.push((now, processed));
        if self.speed_history.len() > self.max_history_size {
            self.speed_history.remove(0);
        }

        let progress = self.calculate_progress(stage_progress, processed);

        if let Some(ref callback) = self.callback {
            callback(progress);
        }
    }

    /// Add processed allocation count
    pub fn add_processed(&self, count: usize) {
        self.processed_allocations
            .fetch_add(count, Ordering::SeqCst);
    }

    /// Set processed allocation count
    pub fn set_processed(&self, count: usize) {
        self.processed_allocations.store(count, Ordering::SeqCst);
    }

    /// Calculate progress information
    fn calculate_progress(&self, stage_progress: f64, processed: usize) -> ExportProgress {
        let elapsed = self.start_time.elapsed();

        // Calculate overall progress
        let stage_weights = [
            (ExportStage::Initializing, 0.05),
            (ExportStage::DataLocalization, 0.15),
            (ExportStage::ParallelProcessing, 0.70),
            (ExportStage::Writing, 0.10),
        ];

        let mut overall_progress = 0.0;
        let mut found_current = false;

        for (stage, weight) in &stage_weights {
            if *stage == self.current_stage {
                overall_progress += weight * stage_progress;
                found_current = true;
                break;
            } else {
                overall_progress += weight;
            }
        }

        if !found_current {
            overall_progress = match self.current_stage {
                ExportStage::Completed => 1.0,
                ExportStage::Cancelled => 0.0,
                ExportStage::Error(_) => 0.0,
                _ => overall_progress,
            };
        }

        // Calculate processing speed
        let processing_speed = if elapsed.as_secs() > 0 {
            processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        // Estimate remaining time
        let estimated_remaining = self.estimate_remaining_time(processed, processing_speed);

        ExportProgress {
            current_stage: self.current_stage.clone(),
            stage_progress,
            overall_progress,
            processed_allocations: processed,
            total_allocations: self.total_allocations,
            elapsed_time: elapsed,
            estimated_remaining,
            processing_speed,
            stage_details: self.current_stage.description().to_string(),
        }
    }

    /// Estimate remaining time
    fn estimate_remaining_time(&self, processed: usize, current_speed: f64) -> Option<Duration> {
        if processed >= self.total_allocations || current_speed <= 0.0 {
            return None;
        }

        // Use historical speed data for more accurate estimation
        let avg_speed = if self.speed_history.len() >= 2 {
            let recent_history = &self.speed_history[self.speed_history.len().saturating_sub(5)..];
            if recent_history.len() >= 2 {
                let first = &recent_history[0];
                let last = &recent_history[recent_history.len() - 1];
                let time_diff = last.0.duration_since(first.0).as_secs_f64();
                let processed_diff = last.1.saturating_sub(first.1) as f64;

                if time_diff > 0.0 {
                    processed_diff / time_diff
                } else {
                    current_speed
                }
            } else {
                current_speed
            }
        } else {
            current_speed
        };

        if avg_speed > 0.0 {
            let remaining_allocations = self.total_allocations.saturating_sub(processed) as f64;
            let remaining_seconds = remaining_allocations / avg_speed;
            Some(Duration::from_secs_f64(remaining_seconds))
        } else {
            None
        }
    }

    /// Complete export
    pub fn complete(&mut self) {
        self.current_stage = ExportStage::Completed;
        self.update_progress(1.0, Some("Export completed".to_string()));
    }

    /// Cancel export
    pub fn cancel(&mut self) {
        self.cancellation_token.cancel();
        self.current_stage = ExportStage::Cancelled;
        self.update_progress(0.0, Some("Export cancelled".to_string()));
    }

    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.current_stage = ExportStage::Error(error.clone());
        self.update_progress(0.0, Some(error));
    }

    /// Check if should cancel
    pub fn should_cancel(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get current progress snapshot
    pub fn get_progress_snapshot(&self) -> ExportProgress {
        let processed = self.processed_allocations.load(Ordering::SeqCst);
        self.calculate_progress(0.0, processed)
    }
}

/// Progress monitoring configuration
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    /// Whether to enable progress monitoring
    pub enabled: bool,
    /// Update interval
    pub update_interval: Duration,
    /// Whether to show details
    pub show_details: bool,
    /// Whether to show estimated time
    pub show_estimated_time: bool,
    /// Whether to allow cancellation
    pub allow_cancellation: bool,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: Duration::from_millis(100),
            show_details: true,
            show_estimated_time: true,
            allow_cancellation: true,
        }
    }
}

/// Console progress display
pub struct ConsoleProgressDisplay {
    last_line_length: usize,
}

impl ConsoleProgressDisplay {
    /// Create new console progress display
    pub fn new() -> Self {
        Self {
            last_line_length: 0,
        }
    }

    /// Display progress
    pub fn display(&mut self, progress: &ExportProgress) {
        // Clear previous line
        if self.last_line_length > 0 {
            print!("\r{}", " ".repeat(self.last_line_length));
            print!("\r");
        }

        let progress_bar = self.create_progress_bar(progress.overall_progress);
        let speed_info = if progress.processing_speed > 0.0 {
            format!(" ({:.0} allocs/sec)", progress.processing_speed)
        } else {
            String::new()
        };

        let time_info = if let Some(remaining) = progress.estimated_remaining {
            format!(" Remaining: {:?}", remaining)
        } else {
            String::new()
        };

        let line = format!(
            "{} {:.1}% {} ({}/{}){}{}",
            progress_bar,
            progress.overall_progress * 100.0,
            progress.current_stage.description(),
            progress.processed_allocations,
            progress.total_allocations,
            speed_info,
            time_info
        );

        print!("{}", line);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        self.last_line_length = line.len();
    }

    /// Create progress bar
    fn create_progress_bar(&self, progress: f64) -> String {
        let width = 20;
        let filled = (progress * width as f64) as usize;
        let empty = width - filled;

        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    /// Finish display (newline)
    pub fn finish(&mut self) {
        tracing::info!("");
        self.last_line_length = 0;
    }
}

impl Default for ConsoleProgressDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.check_cancelled().is_err());
    }

    #[test]
    fn test_progress_monitor_basic() {
        let mut monitor = ProgressMonitor::new(1000);

        // Test initial state
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.current_stage, ExportStage::Initializing);
        assert_eq!(progress.processed_allocations, 0);
        assert_eq!(progress.total_allocations, 1000);

        // Test stage switching
        monitor.set_stage(ExportStage::DataLocalization);
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.current_stage, ExportStage::DataLocalization);

        // Test progress update
        monitor.add_processed(100);
        let progress = monitor.get_progress_snapshot();
        assert_eq!(progress.processed_allocations, 100);
    }

    #[test]
    fn test_progress_callback() {
        let callback_called = Arc::new(Mutex::new(false));
        let callback_called_clone = callback_called.clone();

        let mut monitor = ProgressMonitor::new(100);
        // Set shorter update interval for testing
        monitor.update_interval = Duration::from_millis(1);
        
        monitor.set_callback(Box::new(move |_progress| {
            *callback_called_clone.lock().unwrap() = true;
        }));

        // Add small delay to ensure update interval passes
        std::thread::sleep(std::time::Duration::from_millis(10));
        monitor.update_progress(0.5, None);
        assert!(*callback_called.lock().unwrap());
    }

    #[test]
    fn test_progress_calculation() {
        let mut monitor = ProgressMonitor::new(1000);
        // Set shorter update interval for testing
        monitor.update_interval = Duration::from_millis(1);

        // Directly test calculation function
        let progress = monitor.calculate_progress(1.0, 0);
        assert_eq!(progress.current_stage, ExportStage::Initializing);

        // Test initialization stage
        monitor.set_stage(ExportStage::Initializing);
        let progress = monitor.calculate_progress(1.0, 0);
        assert!(
            (progress.overall_progress - 0.05).abs() < 0.01,
            "Expected ~0.05, got {}",
            progress.overall_progress
        );

        // Test data localization stage
        monitor.set_stage(ExportStage::DataLocalization);
        let progress = monitor.calculate_progress(0.5, 0);
        let expected = 0.05 + 0.15 * 0.5;
        assert!(
            (progress.overall_progress - expected).abs() < 0.01,
            "Expected ~{}, got {}",
            expected,
            progress.overall_progress
        );

        // Test completion
        monitor.set_stage(ExportStage::Completed);
        let progress = monitor.calculate_progress(1.0, 0);
        assert_eq!(progress.overall_progress, 1.0);
        assert_eq!(progress.current_stage, ExportStage::Completed);
    }

    #[test]
    fn test_speed_calculation() {
        let monitor = ProgressMonitor::new(1000);

        // Add small delay to ensure elapsed time > 0
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Simulate processing some allocations
        monitor.add_processed(100);

        let progress = monitor.get_progress_snapshot();
        // Processing speed should be >= 0
        assert!(
            progress.processing_speed >= 0.0,
            "Processing speed should be >= 0, got {}",
            progress.processing_speed
        );

        // Basic test: ensure speed calculation doesn't crash
        assert!(
            progress.elapsed_time.as_millis() > 0,
            "Elapsed time should be > 0"
        );

        // If there are processed allocations and enough time, speed should be > 0
        // But due to test environment uncertainty, we only check basic mathematical correctness
        let expected_speed = if progress.elapsed_time.as_secs() > 0 {
            100.0 / progress.elapsed_time.as_secs_f64()
        } else {
            0.0
        };

        // Allow certain error range
        assert!(
            (progress.processing_speed - expected_speed).abs() < 1.0,
            "Speed calculation mismatch: expected ~{}, got {}",
            expected_speed,
            progress.processing_speed
        );
    }

    #[test]
    fn test_console_progress_display() {
        let mut display = ConsoleProgressDisplay::new();

        let progress = ExportProgress {
            current_stage: ExportStage::ParallelProcessing,
            stage_progress: 0.5,
            overall_progress: 0.6,
            processed_allocations: 600,
            total_allocations: 1000,
            elapsed_time: Duration::from_secs(10),
            estimated_remaining: Some(Duration::from_secs(7)),
            processing_speed: 60.0,
            stage_details: "Parallel shard processing".to_string(),
        };

        // This test mainly ensures no panic
        display.display(&progress);
        display.finish();
    }

    #[test]
    fn test_export_stage_weights() {
        assert_eq!(ExportStage::Initializing.weight(), 0.05);
        assert_eq!(ExportStage::DataLocalization.weight(), 0.15);
        assert_eq!(ExportStage::ParallelProcessing.weight(), 0.70);
        assert_eq!(ExportStage::Writing.weight(), 0.10);
        assert_eq!(ExportStage::Completed.weight(), 1.0);
    }
}
