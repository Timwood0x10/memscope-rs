//! Simplified error recovery and diagnostic system
//!
//! This module provides essential error handling enhancements for the binary-to-JSON
//! optimization system, focusing on the most critical recovery mechanisms without
//! over-engineering.

use crate::export::binary::{BinaryExportError, BinaryIndex, BinaryIndexBuilder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, error, info, warn};

/// Simplified error recovery manager
pub struct ErrorRecoveryManager {
    /// Error statistics for trend analysis
    error_stats: ErrorStatistics,

    /// Recovery strategies configuration
    recovery_config: RecoveryConfig,

    /// Index corruption detection cache
    corruption_cache: HashMap<PathBuf, CorruptionStatus>,
}

/// Error statistics for basic trend analysis
#[derive(Debug, Clone, Default)]
pub struct ErrorStatistics {
    /// Total errors encountered
    pub total_errors: u64,

    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,

    /// Successful recoveries
    pub successful_recoveries: u64,

    /// Failed recoveries
    pub failed_recoveries: u64,

    /// Index rebuilds performed
    pub index_rebuilds: u64,

    /// Last error timestamp
    pub last_error_time: Option<SystemTime>,

    /// Error rate (errors per hour)
    pub error_rate: f64,
}

/// Recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Enable automatic index rebuilding
    pub enable_auto_rebuild: bool,

    /// Maximum retry attempts
    pub max_retry_attempts: u32,

    /// Retry delay
    pub retry_delay: Duration,

    /// Enable partial result returns
    pub enable_partial_results: bool,

    /// Corruption detection threshold
    pub corruption_threshold: f64,
}

/// Index corruption status
#[derive(Debug, Clone)]
struct CorruptionStatus {
    /// Whether the index is corrupted
    is_corrupted: bool,

    /// Last check timestamp
    last_checked: SystemTime,

    /// Corruption confidence (0.0 to 1.0)
    confidence: f64,
}

/// Recovery attempt result
#[derive(Debug, Clone)]
pub struct RecoveryResult<T> {
    /// The recovered result (if any)
    pub result: Option<T>,

    /// Whether recovery was successful
    pub success: bool,

    /// Recovery strategy used
    pub strategy_used: RecoveryStrategy,

    /// Number of attempts made
    pub attempts_made: u32,

    /// Time taken for recovery
    pub recovery_time: Duration,

    /// Partial results (if enabled)
    pub partial_results: Vec<T>,
}

/// Recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,

    /// Rebuild corrupted index
    RebuildIndex,

    /// Fall back to legacy method
    FallbackToLegacy,

    /// Return partial results
    PartialResults,

    /// Skip corrupted records
    SkipCorrupted,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enable_auto_rebuild: true,
            max_retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
            enable_partial_results: true,
            corruption_threshold: 0.7,
        }
    }
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        Self::with_config(RecoveryConfig::default())
    }

    /// Create a new error recovery manager with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        Self {
            error_stats: ErrorStatistics::default(),
            recovery_config: config,
            corruption_cache: HashMap::new(),
        }
    }

    /// Attempt to recover from a binary export error
    pub fn attempt_recovery<T, F>(&mut self, operation: F, context: &str) -> RecoveryResult<T>
    where
        F: Fn() -> Result<T, BinaryExportError>,
    {
        let start_time = Instant::now();
        let mut attempts = 0;
        let partial_results = Vec::new();

        info!("Starting error recovery for: {}", context);

        // First attempt - try the operation as-is
        attempts += 1;
        match operation() {
            Ok(result) => {
                return RecoveryResult {
                    result: Some(result),
                    success: true,
                    strategy_used: RecoveryStrategy::Retry,
                    attempts_made: attempts,
                    recovery_time: start_time.elapsed(),
                    partial_results,
                };
            }
            Err(error) => {
                self.record_error(&error, context);
                debug!("Initial attempt failed: {}", error);
            }
        }

        // Retry with delay
        for attempt in 1..=self.recovery_config.max_retry_attempts {
            attempts += 1;

            std::thread::sleep(self.recovery_config.retry_delay);

            match operation() {
                Ok(result) => {
                    self.error_stats.successful_recoveries += 1;
                    info!("Recovery successful after {} attempts", attempts);

                    return RecoveryResult {
                        result: Some(result),
                        success: true,
                        strategy_used: RecoveryStrategy::Retry,
                        attempts_made: attempts,
                        recovery_time: start_time.elapsed(),
                        partial_results,
                    };
                }
                Err(error) => {
                    warn!("Retry attempt {} failed: {}", attempt, error);
                    self.record_error(&error, context);
                }
            }
        }

        // All retries failed
        self.error_stats.failed_recoveries += 1;
        error!("All recovery attempts failed for: {}", context);

        RecoveryResult {
            result: None,
            success: false,
            strategy_used: RecoveryStrategy::Retry,
            attempts_made: attempts,
            recovery_time: start_time.elapsed(),
            partial_results,
        }
    }

    /// Detect and handle index corruption
    pub fn check_index_corruption<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
    ) -> Result<bool, BinaryExportError> {
        let binary_path = binary_path.as_ref();
        let path_buf = binary_path.to_path_buf();

        // Check cache first
        if let Some(status) = self.corruption_cache.get(&path_buf) {
            if status.last_checked.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(300) {
                return Ok(status.is_corrupted);
            }
        }

        debug!("Checking index corruption for: {:?}", binary_path);

        // Simple corruption detection - try to build index
        let builder = BinaryIndexBuilder::new();
        let corruption_detected = match builder.build_index(binary_path) {
            Ok(_) => false,
            Err(BinaryExportError::CorruptedData(_)) => true,
            Err(BinaryExportError::InvalidFormat) => true,
            Err(_) => false, // Other errors are not corruption
        };

        // Update cache
        self.corruption_cache.insert(
            path_buf,
            CorruptionStatus {
                is_corrupted: corruption_detected,
                last_checked: SystemTime::now(),
                confidence: if corruption_detected { 0.9 } else { 0.1 },
            },
        );

        if corruption_detected {
            warn!("Index corruption detected for: {:?}", binary_path);
        }

        Ok(corruption_detected)
    }

    /// Attempt to rebuild a corrupted index
    pub fn rebuild_index<P: AsRef<Path>>(
        &mut self,
        binary_path: P,
    ) -> Result<BinaryIndex, BinaryExportError> {
        let binary_path = binary_path.as_ref();

        info!("Attempting to rebuild index for: {:?}", binary_path);

        // Clear corruption cache for this file
        self.corruption_cache.remove(&binary_path.to_path_buf());

        // Attempt to rebuild
        let builder = BinaryIndexBuilder::new();
        match builder.build_index(binary_path) {
            Ok(index) => {
                self.error_stats.index_rebuilds += 1;
                info!("Index successfully rebuilt for: {:?}", binary_path);

                // Update cache with success
                self.corruption_cache.insert(
                    binary_path.to_path_buf(),
                    CorruptionStatus {
                        is_corrupted: false,
                        last_checked: SystemTime::now(),
                        confidence: 0.1,
                    },
                );

                Ok(index)
            }
            Err(error) => {
                error!("Failed to rebuild index for: {:?} - {}", binary_path, error);
                Err(error)
            }
        }
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> &ErrorStatistics {
        &self.error_stats
    }

    /// Get mutable error statistics (for internal use)
    pub fn get_error_stats_mut(&mut self) -> &mut ErrorStatistics {
        &mut self.error_stats
    }

    /// Reset error statistics
    pub fn reset_stats(&mut self) {
        self.error_stats = ErrorStatistics::default();
        info!("Error statistics reset");
    }

    /// Generate error report
    pub fn generate_error_report(&self) -> ErrorReport {
        let total_operations =
            self.error_stats.successful_recoveries + self.error_stats.failed_recoveries;
        let success_rate = if total_operations > 0 {
            self.error_stats.successful_recoveries as f64 / total_operations as f64
        } else {
            1.0
        };

        ErrorReport {
            total_errors: self.error_stats.total_errors,
            success_rate,
            most_common_error: self.get_most_common_error(),
            index_rebuilds: self.error_stats.index_rebuilds,
            error_rate: self.error_stats.error_rate,
            recommendations: self.generate_recommendations(),
        }
    }

    /// Update recovery configuration
    pub fn update_config(&mut self, config: RecoveryConfig) {
        self.recovery_config = config;
        info!("Recovery configuration updated");
    }

    // Private helper methods

    fn record_error(&mut self, error: &BinaryExportError, context: &str) {
        self.error_stats.total_errors += 1;
        self.error_stats.last_error_time = Some(SystemTime::now());

        let error_type = match error {
            BinaryExportError::Io(_) => "IO",
            BinaryExportError::InvalidFormat => "InvalidFormat",
            BinaryExportError::UnsupportedVersion(_) => "UnsupportedVersion",
            BinaryExportError::CorruptedData(_) => "CorruptedData",
            BinaryExportError::InvalidMagic { .. } => "InvalidMagic",
            BinaryExportError::SerializationError(_) => "Serialization",
            BinaryExportError::CompressionError(_) => "Compression",
        };

        *self
            .error_stats
            .errors_by_type
            .entry(error_type.to_string())
            .or_insert(0) += 1;

        debug!("Recorded error: {} in context: {}", error_type, context);
    }

    fn get_most_common_error(&self) -> Option<String> {
        self.error_stats
            .errors_by_type
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(error_type, _)| error_type.clone())
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.error_stats.total_errors > 10 {
            recommendations
                .push("High error rate detected. Consider checking file integrity.".to_string());
        }

        if self.error_stats.index_rebuilds > 3 {
            recommendations.push(
                "Frequent index rebuilds detected. Consider checking storage reliability."
                    .to_string(),
            );
        }

        if let Some(most_common) = self.get_most_common_error() {
            match most_common.as_str() {
                "IO" => recommendations
                    .push("IO errors are common. Check disk space and permissions.".to_string()),
                "CorruptedData" => recommendations
                    .push("Data corruption detected. Consider backup verification.".to_string()),
                "InvalidFormat" => recommendations
                    .push("Format errors detected. Ensure file compatibility.".to_string()),
                _ => {}
            }
        }

        recommendations
    }
}

/// Error report summary
#[derive(Debug, Clone)]
pub struct ErrorReport {
    /// Total errors encountered
    pub total_errors: u64,

    /// Recovery success rate
    pub success_rate: f64,

    /// Most common error type
    pub most_common_error: Option<String>,

    /// Number of index rebuilds
    pub index_rebuilds: u64,

    /// Error rate per hour
    pub error_rate: f64,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl ErrorStatistics {
    /// Calculate recovery success rate
    pub fn recovery_success_rate(&self) -> f64 {
        let total = self.successful_recoveries + self.failed_recoveries;
        if total > 0 {
            self.successful_recoveries as f64 / total as f64
        } else {
            1.0
        }
    }

    /// Get error trend (increasing/decreasing)
    pub fn error_trend(&self) -> ErrorTrend {
        // Simplified trend analysis based on recent error rate
        if self.error_rate > 10.0 {
            ErrorTrend::Increasing
        } else if self.error_rate < 1.0 {
            ErrorTrend::Decreasing
        } else {
            ErrorTrend::Stable
        }
    }
}

/// Error trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorTrend {
    Increasing,
    Stable,
    Decreasing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::new();
        assert_eq!(manager.error_stats.total_errors, 0);
        assert!(manager.recovery_config.enable_auto_rebuild);
    }

    #[test]
    fn test_recovery_config() {
        let config = RecoveryConfig {
            enable_auto_rebuild: false,
            max_retry_attempts: 5,
            retry_delay: Duration::from_millis(200),
            enable_partial_results: false,
            corruption_threshold: 0.8,
        };

        let manager = ErrorRecoveryManager::with_config(config.clone());
        assert_eq!(manager.recovery_config.max_retry_attempts, 5);
        assert!(!manager.recovery_config.enable_auto_rebuild);
    }

    #[test]
    fn test_successful_recovery() {
        let mut manager = ErrorRecoveryManager::new();
        let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result: RecoveryResult<String> = manager.attempt_recovery(
            || {
                let mut count = attempt_count_clone.lock().unwrap();
                *count += 1;
                if *count < 2 {
                    Err(BinaryExportError::InvalidFormat)
                } else {
                    Ok("success".to_string())
                }
            },
            "test operation",
        );

        assert!(result.success);
        assert_eq!(result.result, Some("success".to_string()));
        assert_eq!(result.attempts_made, 2);
        assert_eq!(result.strategy_used, RecoveryStrategy::Retry);
    }

    #[test]
    fn test_failed_recovery() {
        let mut manager = ErrorRecoveryManager::with_config(RecoveryConfig {
            max_retry_attempts: 2,
            retry_delay: Duration::from_millis(1), // Fast test
            ..Default::default()
        });

        let result: RecoveryResult<String> =
            manager.attempt_recovery(|| Err(BinaryExportError::InvalidFormat), "test operation");

        assert!(!result.success);
        assert_eq!(result.result, None);
        assert_eq!(result.attempts_made, 3); // Initial + 2 retries
        assert_eq!(manager.error_stats.total_errors, 3);
        assert_eq!(manager.error_stats.failed_recoveries, 1);
    }

    #[test]
    fn test_error_statistics() {
        let mut stats = ErrorStatistics::default();
        stats.successful_recoveries = 8;
        stats.failed_recoveries = 2;
        stats.error_rate = 5.0;

        assert_eq!(stats.recovery_success_rate(), 0.8);
        assert_eq!(stats.error_trend(), ErrorTrend::Stable);
    }

    #[test]
    fn test_error_report_generation() {
        let mut manager = ErrorRecoveryManager::new();

        // Simulate some errors
        manager.error_stats.total_errors = 10;
        manager.error_stats.successful_recoveries = 8;
        manager.error_stats.failed_recoveries = 2;
        manager.error_stats.index_rebuilds = 1;
        manager
            .error_stats
            .errors_by_type
            .insert("IO".to_string(), 6);
        manager
            .error_stats
            .errors_by_type
            .insert("CorruptedData".to_string(), 4);

        let report = manager.generate_error_report();

        assert_eq!(report.total_errors, 10);
        assert_eq!(report.success_rate, 0.8);
        assert_eq!(report.most_common_error, Some("IO".to_string()));
        assert_eq!(report.index_rebuilds, 1);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_recovery_strategies() {
        assert_eq!(RecoveryStrategy::Retry, RecoveryStrategy::Retry);
        assert_ne!(RecoveryStrategy::Retry, RecoveryStrategy::RebuildIndex);
    }
}
