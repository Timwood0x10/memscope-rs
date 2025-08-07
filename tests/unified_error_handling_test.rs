//! Tests for the unified error handling system
//!
//! This test suite verifies that:
//! 1. All existing error types are preserved
//! 2. Error messages and context are maintained
//! 3. Backward compatibility is ensured
//! 4. Error recovery mechanisms work correctly

use memscope_rs::core::{
    adapt_result, from_tracking_error, to_tracking_error, DefaultErrorRecovery, ErrorRecovery,
    ErrorSeverity, MemScopeError, MemoryOperation, RecoveryAction, SystemErrorType, TrackingError,
    TrackingResult,
};

#[test]
fn test_error_creation_and_properties() {
    // Test memory operation errors
    let alloc_error = MemScopeError::memory(MemoryOperation::Allocation, "allocation failed");
    assert_eq!(alloc_error.category(), "memory");
    assert_eq!(alloc_error.severity(), ErrorSeverity::Medium);
    assert!(alloc_error.is_recoverable());
    assert!(alloc_error.user_message().contains("allocation failed"));

    // Test analysis errors
    let analysis_error = MemScopeError::analysis("borrow", "borrow check failed");
    assert_eq!(analysis_error.category(), "analysis");
    assert!(analysis_error.is_recoverable());

    // Test export errors
    let export_error = MemScopeError::export("json", "serialization failed");
    assert_eq!(export_error.category(), "export");

    // Test system errors
    let system_error = MemScopeError::system(SystemErrorType::Io, "file not found");
    assert_eq!(system_error.category(), "system");

    // Test internal errors
    let internal_error = MemScopeError::internal("critical failure");
    assert_eq!(internal_error.category(), "internal");
    assert_eq!(internal_error.severity(), ErrorSeverity::Critical);
    assert!(!internal_error.is_recoverable());
}

#[test]
fn test_error_context_preservation() {
    // Test memory error with context
    let error_with_context = MemScopeError::memory_with_context(
        MemoryOperation::Allocation,
        "allocation failed",
        "in function test_allocation",
    );

    let display_string = format!("{}", error_with_context);
    assert!(display_string.contains("allocation failed"));
    assert!(display_string.contains("context: in function test_allocation"));

    // Test partial export error
    let partial_export = MemScopeError::export_partial("json", "some data failed");
    let display_string = format!("{}", partial_export);
    assert!(display_string.contains("Partial export error"));

    // Test internal error with location
    let internal_with_location = MemScopeError::internal_at("bug detected", "tracker.rs:123");
    let display_string = format!("{}", internal_with_location);
    assert!(display_string.contains("bug detected"));
    assert!(display_string.contains("at tracker.rs:123"));
}

#[test]
fn test_error_recovery_system() {
    let recovery = DefaultErrorRecovery::new();

    // Test recoverable memory error
    let memory_error = MemScopeError::memory(MemoryOperation::Allocation, "temporary failure");
    assert!(recovery.can_recover(&memory_error));

    let action = recovery.get_recovery_action(&memory_error);
    assert!(matches!(action, Some(RecoveryAction::Retry { .. })));

    if let Some(RecoveryAction::Retry {
        max_attempts,
        delay_ms,
    }) = action
    {
        assert!(max_attempts > 0);
        // delay_ms is u64, so it's always >= 0, but we can check it's reasonable
        assert!(delay_ms <= 10000); // Should be reasonable delay (max 10 seconds)
    }

    // Test non-recoverable internal error
    let internal_error = MemScopeError::internal("critical system failure");
    assert!(!recovery.can_recover(&internal_error));

    let action = recovery.get_recovery_action(&internal_error);
    assert!(matches!(action, Some(RecoveryAction::Abort)));

    // Test analysis error recovery
    let analysis_error = MemScopeError::analysis("type_checker", "inference failed");
    assert!(recovery.can_recover(&analysis_error));

    let action = recovery.get_recovery_action(&analysis_error);
    assert!(matches!(action, Some(RecoveryAction::UseDefault { .. })));
}

#[test]
fn test_backward_compatibility_conversion() {
    // Test all TrackingError variants
    let test_cases = vec![
        (
            TrackingError::AllocationFailed("alloc failed".to_string()),
            "memory",
        ),
        (
            TrackingError::DeallocationFailed("dealloc failed".to_string()),
            "memory",
        ),
        (
            TrackingError::SerializationError("json error".to_string()),
            "system",
        ),
        (
            TrackingError::AnalysisError("analysis failed".to_string()),
            "analysis",
        ),
        (
            TrackingError::ExportError("export failed".to_string()),
            "export",
        ),
        (
            TrackingError::ThreadSafetyError("thread error".to_string()),
            "system",
        ),
        (
            TrackingError::ConfigurationError("config error".to_string()),
            "config",
        ),
        (
            TrackingError::InternalError("internal error".to_string()),
            "internal",
        ),
        (TrackingError::IoError("io error".to_string()), "system"),
        (
            TrackingError::BorrowCheckError("borrow error".to_string()),
            "analysis",
        ),
        (
            TrackingError::LifetimeError("lifetime error".to_string()),
            "analysis",
        ),
    ];

    for (old_error, expected_category) in test_cases {
        let original_message = old_error.to_string();

        // Convert to new error system
        let new_error = from_tracking_error(old_error.clone());
        assert_eq!(new_error.category(), expected_category);

        // Convert back to old system
        let converted_back = to_tracking_error(&new_error);

        // Verify the error type is preserved (at least the general category)
        let back_message = converted_back.to_string();

        // The message should contain key information from the original
        assert!(
            back_message.contains(&extract_key_message(&original_message))
                || original_message.contains(&extract_key_message(&back_message)),
            "Message preservation failed: '{}' vs '{}'",
            original_message,
            back_message
        );
    }
}

#[test]
fn test_result_adaptation() {
    // Test successful result adaptation
    let success_result: TrackingResult<i32> = Ok(42);
    let adapted_success = adapt_result(success_result);
    assert!(adapted_success.is_ok());
    assert_eq!(adapted_success.unwrap(), 42);

    // Test error result adaptation
    let error_result: TrackingResult<i32> =
        Err(TrackingError::AllocationFailed("test".to_string()));
    let adapted_error = adapt_result(error_result);
    assert!(adapted_error.is_err());

    let error = adapted_error.unwrap_err();
    assert_eq!(error.category(), "memory");
    assert!(error.user_message().contains("test"));
}

#[test]
fn test_error_severity_levels() {
    // Test different severity levels
    let low_severity = MemScopeError::analysis("type_checker", "minor issue");
    assert_eq!(low_severity.severity(), ErrorSeverity::Low);

    let medium_severity = MemScopeError::memory(MemoryOperation::Allocation, "allocation failed");
    assert_eq!(medium_severity.severity(), ErrorSeverity::Medium);

    let high_severity = MemScopeError::analysis_fatal("critical_analyzer", "fatal error");
    assert_eq!(high_severity.severity(), ErrorSeverity::High);

    let critical_severity = MemScopeError::internal("system corruption");
    assert_eq!(critical_severity.severity(), ErrorSeverity::Critical);

    // Test severity ordering
    assert!(ErrorSeverity::Low < ErrorSeverity::Medium);
    assert!(ErrorSeverity::Medium < ErrorSeverity::High);
    assert!(ErrorSeverity::High < ErrorSeverity::Critical);
}

#[test]
fn test_standard_error_conversions() {
    // Test std::io::Error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let memscope_error: MemScopeError = io_error.into();
    assert_eq!(memscope_error.category(), "system");
    assert!(memscope_error
        .user_message()
        .contains("I/O operation failed"));

    // Test serde_json::Error conversion
    let json_str = r#"{"invalid": json"#;
    let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let memscope_error: MemScopeError = json_error.into();
    assert_eq!(memscope_error.category(), "system");
    assert!(memscope_error
        .user_message()
        .contains("JSON serialization failed"));
}

#[test]
fn test_error_display_formatting() {
    // Test various error display formats
    let memory_error = MemScopeError::memory_with_context(
        MemoryOperation::Deallocation,
        "double free detected",
        "pointer 0x12345",
    );
    let display = format!("{}", memory_error);
    assert!(display.contains("Memory deallocation error"));
    assert!(display.contains("double free detected"));
    assert!(display.contains("context: pointer 0x12345"));

    let analysis_error = MemScopeError::analysis("lifetime_checker", "use after free");
    let display = format!("{}", analysis_error);
    assert!(display.contains("Analysis error in lifetime_checker"));
    assert!(display.contains("use after free"));

    let export_partial = MemScopeError::export_partial("binary", "compression failed");
    let display = format!("{}", export_partial);
    assert!(display.contains("Partial export error (binary)"));
    assert!(display.contains("compression failed"));
}

#[test]
fn test_custom_error_recovery() {
    struct CustomRecovery;

    impl ErrorRecovery for CustomRecovery {
        fn can_recover(&self, error: &MemScopeError) -> bool {
            // Custom logic: only recover from memory allocation errors
            matches!(
                error,
                MemScopeError::Memory {
                    operation: MemoryOperation::Allocation,
                    ..
                }
            )
        }

        fn get_recovery_action(&self, error: &MemScopeError) -> Option<RecoveryAction> {
            if self.can_recover(error) {
                Some(RecoveryAction::Retry {
                    max_attempts: 5,
                    delay_ms: 200,
                })
            } else {
                Some(RecoveryAction::Abort)
            }
        }

        fn execute_recovery(&self, _action: &RecoveryAction) -> Result<(), MemScopeError> {
            Ok(())
        }
    }

    let custom_recovery = CustomRecovery;

    // Test custom recovery for allocation error
    let alloc_error = MemScopeError::memory(MemoryOperation::Allocation, "failed");
    assert!(custom_recovery.can_recover(&alloc_error));

    let action = custom_recovery.get_recovery_action(&alloc_error);
    assert!(matches!(
        action,
        Some(RecoveryAction::Retry {
            max_attempts: 5,
            delay_ms: 200
        })
    ));

    // Test custom recovery for other errors
    let analysis_error = MemScopeError::analysis("test", "failed");
    assert!(!custom_recovery.can_recover(&analysis_error));

    let action = custom_recovery.get_recovery_action(&analysis_error);
    assert!(matches!(action, Some(RecoveryAction::Abort)));
}

// Helper function to extract key message content
fn extract_key_message(message: &str) -> String {
    // Extract the main error message, removing prefixes like "Allocation failed: "
    if let Some(colon_pos) = message.find(": ") {
        message[colon_pos + 2..].to_string()
    } else {
        message.to_string()
    }
}
