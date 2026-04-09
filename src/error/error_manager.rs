//! Central error management system for MemScope
//!
//! Provides unified error tracking, reporting, and aggregation
//! across all modules while maintaining module-specific error types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::MemScopeError;
use crate::core::error::{ErrorContext, ErrorSeverity};

/// Error statistics for monitoring and analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub by_severity: HashMap<String, u64>,
    pub by_module: HashMap<String, u64>,
    pub by_kind: HashMap<String, u64>,
    pub recent_errors: Vec<ErrorRecord>,
}

/// Individual error record for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub timestamp: u64,
    pub module: String,
    pub error_type: String,
    pub severity: ErrorSeverity,
    pub message: String,
    pub context: Option<ErrorContext>,
}

/// Central error manager for the entire MemScope system
pub struct ErrorManager {
    stats: Arc<Mutex<ErrorStats>>,
    max_recent_errors: usize,
    enabled: Arc<AtomicBool>,
}

use std::sync::atomic::AtomicBool;

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(ErrorStats::default())),
            max_recent_errors: 100,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn with_config(max_recent_errors: usize) -> Self {
        Self {
            stats: Arc::new(Mutex::new(ErrorStats::default())),
            max_recent_errors,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Enable error tracking
    pub fn enable(&self) {
        self.enabled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Disable error tracking
    pub fn disable(&self) {
        self.enabled
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if error tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Record an error with full context
    pub fn record_error(&self, module: &str, error_type: &str, error: &MemScopeError) {
        if !self.is_enabled() {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let severity = error.severity();
        let severity_str = format!("{:?}", severity);

        let record = ErrorRecord {
            timestamp,
            module: module.to_string(),
            error_type: error_type.to_string(),
            severity,
            message: error.user_message().to_string(),
            context: None,
        };

        if let Ok(mut stats) = self.stats.lock() {
            stats.total_errors += 1;
            *stats.by_severity.entry(severity_str).or_insert(0) += 1;
            *stats.by_module.entry(module.to_string()).or_insert(0) += 1;
            *stats.by_kind.entry(error_type.to_string()).or_insert(0) += 1;

            // Maintain recent errors list
            stats.recent_errors.push(record);
            if stats.recent_errors.len() > self.max_recent_errors {
                stats.recent_errors.remove(0);
            }
        }
    }

    /// Get current error statistics
    pub fn get_stats(&self) -> ErrorStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            ErrorStats::default()
        }
    }

    /// Clear error statistics (for testing)
    pub fn clear_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = ErrorStats::default();
        }
    }

    /// Get error report
    pub fn generate_report(&self) -> ErrorReport {
        let stats = self.get_stats();

        let summary = if stats.total_errors == 0 {
            "No errors recorded".to_string()
        } else {
            format!(
                "Total: {} errors, Severity: {:?} most common",
                stats.total_errors,
                stats
                    .by_severity
                    .iter()
                    .max_by_key(|(_, &a)| a)
                    .map(|(k, _)| k)
            )
        };

        ErrorReport {
            summary,
            stats,
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// Convert various error types to MemScopeError
    pub fn convert_to_memscope<T: IntoMemScopeError>(error: T, module: &str) -> MemScopeError {
        error.into_memscope_error(module)
    }
}

/// Generated error report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub summary: String,
    pub stats: ErrorStats,
    pub generated_at: u64,
}

/// Trait for converting error types to MemScopeError
pub trait IntoMemScopeError: Sized {
    fn into_memscope_error(self, module: &str) -> MemScopeError;
}

/// Global error manager instance
static GLOBAL_ERROR_MANAGER: std::sync::OnceLock<ErrorManager> = std::sync::OnceLock::new();

/// Get the global error manager
pub fn global_error_manager() -> &'static ErrorManager {
    GLOBAL_ERROR_MANAGER.get_or_init(ErrorManager::new)
}

/// Convenience function to record an error
pub fn record_error(module: &str, error_type: &str, error: &MemScopeError) {
    global_error_manager().record_error(module, error_type, error);
}

/// Convenience function to get error statistics
pub fn get_error_stats() -> ErrorStats {
    global_error_manager().get_stats()
}

/// Convenience function to generate error report
pub fn generate_error_report() -> ErrorReport {
    global_error_manager().generate_report()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_manager_creation() {
        let manager = ErrorManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_error_recording() {
        let manager = ErrorManager::with_config(10);
        let error = MemScopeError::memory(
            crate::core::error::MemoryOperation::Allocation,
            "test error",
        );

        manager.record_error("test_module", "test_error_type", &error);

        let stats = manager.get_stats();
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.by_module.get("test_module"), Some(&1));
        assert_eq!(stats.by_kind.get("test_error_type"), Some(&1));
    }

    #[test]
    fn test_error_stats_limits() {
        let manager = ErrorManager::with_config(3);
        let error = MemScopeError::memory(crate::core::error::MemoryOperation::Allocation, "test");

        // Record more errors than max_recent_errors
        for i in 0..10 {
            manager.record_error("test", &format!("error_{}", i), &error);
        }

        let stats = manager.get_stats();
        assert_eq!(stats.total_errors, 10);
        assert_eq!(stats.recent_errors.len(), 3); // Should be limited to max_recent_errors
    }

    #[test]
    fn test_error_enable_disable() {
        let manager = ErrorManager::new();
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_error_clear_stats() {
        let manager = ErrorManager::new();
        let error = MemScopeError::memory(crate::core::error::MemoryOperation::Allocation, "test");

        manager.record_error("test", "test", &error);
        assert_eq!(manager.get_stats().total_errors, 1);

        manager.clear_stats();
        assert_eq!(manager.get_stats().total_errors, 0);
    }

    #[test]
    fn test_error_report_generation() {
        let manager = ErrorManager::new();
        let report = manager.generate_report();

        assert_eq!(report.summary, "No errors recorded");
        assert_eq!(report.stats.total_errors, 0);
    }

    #[test]
    fn test_global_error_manager() {
        let manager = global_error_manager();
        assert!(manager.is_enabled());

        let error = MemScopeError::internal("test");
        record_error("global_test", "test_type", &error);

        let stats = get_error_stats();
        assert!(stats.total_errors >= 1);
    }

    #[test]
    fn test_error_manager_thread_safety() {
        use std::thread;

        let manager = Arc::new(ErrorManager::new());
        let error = MemScopeError::memory(crate::core::error::MemoryOperation::Allocation, "test");

        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let error_clone = error.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    manager_clone.record_error(
                        "thread_test",
                        &format!("error_{}_{}", i, j),
                        &error_clone,
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = manager.get_stats();
        assert_eq!(stats.total_errors, 100);
    }
}
