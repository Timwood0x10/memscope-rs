//! Tests for the UnwrapSafe trait and safe unwrap utilities
//! 
//! This test suite verifies that:
//! 1. Safe unwrap operations work correctly
//! 2. Panic behavior is preserved when needed
//! 3. Default values and error handling work as expected
//! 4. Statistics tracking functions properly

use memscope_rs::core::{UnwrapSafe, UnwrapStats};
use memscope_rs::{unwrap_safe, unwrap_or_default_safe, try_unwrap_safe};

#[test]
fn test_option_unwrap_safe_success() {
    let value = Some(42);
    let result = value.unwrap_safe("test success");
    assert_eq!(result, 42);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value in context: test panic")]
fn test_option_unwrap_safe_panic() {
    let value: Option<i32> = None;
    value.unwrap_safe("test panic");
}

#[test]
fn test_option_unwrap_safe_at() {
    let value = Some("test");
    let result = value.unwrap_safe_at("location test", "test_function:123");
    assert_eq!(result, "test");
}

#[test]
#[should_panic(expected = "at test_location:456")]
fn test_option_unwrap_safe_at_panic() {
    let value: Option<i32> = None;
    value.unwrap_safe_at("location panic test", "test_location:456");
}

#[test]
fn test_option_unwrap_or_default_safe() {
    // Test with Some value
    let some_value = Some(42);
    let result = some_value.unwrap_or_default_safe(99, "test with some");
    assert_eq!(result, 42);
    
    // Test with None value
    let none_value: Option<i32> = None;
    let result = none_value.unwrap_or_default_safe(99, "test with none");
    assert_eq!(result, 99);
}

#[test]
fn test_option_unwrap_or_else_safe() {
    // Test with Some value
    let some_value = Some(42);
    let result = some_value.unwrap_or_else_safe(|| 99, "test with some");
    assert_eq!(result, 42);
    
    // Test with None value
    let none_value: Option<i32> = None;
    let result = none_value.unwrap_or_else_safe(|| 99, "test with none");
    assert_eq!(result, 99);
}

#[test]
fn test_option_try_unwrap_safe() {
    // Test with Some value
    let some_value = Some(42);
    let result = some_value.try_unwrap_safe("test try with some");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test with None value
    let none_value: Option<i32> = None;
    let result = none_value.try_unwrap_safe("test try with none");
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.category(), "memory");
    assert!(error.user_message().contains("Option unwrap failed"));
}

#[test]
fn test_result_unwrap_safe_success() {
    let value: Result<i32, &str> = Ok(42);
    let result = value.unwrap_safe("test result success");
    assert_eq!(result, 42);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_result_unwrap_safe_panic() {
    let value: Result<i32, &str> = Err("test error");
    value.unwrap_safe("test result panic");
}

#[test]
fn test_result_unwrap_or_default_safe() {
    // Test with Ok value
    let ok_value: Result<i32, &str> = Ok(42);
    let result = ok_value.unwrap_or_default_safe(99, "test with ok");
    assert_eq!(result, 42);
    
    // Test with Err value
    let err_value: Result<i32, &str> = Err("error");
    let result = err_value.unwrap_or_default_safe(99, "test with err");
    assert_eq!(result, 99);
}

#[test]
fn test_result_unwrap_or_else_safe() {
    // Test with Ok value
    let ok_value: Result<i32, &str> = Ok(42);
    let result = ok_value.unwrap_or_else_safe(|| 99, "test with ok");
    assert_eq!(result, 42);
    
    // Test with Err value
    let err_value: Result<i32, &str> = Err("error");
    let result = err_value.unwrap_or_else_safe(|| 99, "test with err");
    assert_eq!(result, 99);
}

#[test]
fn test_result_try_unwrap_safe() {
    // Test with Ok value
    let ok_value: Result<i32, &str> = Ok(42);
    let result = ok_value.try_unwrap_safe("test try with ok");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test with Err value
    let err_value: Result<i32, &str> = Err("test error");
    let result = err_value.try_unwrap_safe("test try with err");
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.category(), "system");
    assert!(error.user_message().contains("Result unwrap failed"));
}

#[test]
fn test_unwrap_safe_macro() {
    let value = Some(42);
    let result = unwrap_safe!(value, "macro test");
    assert_eq!(result, 42);
}

#[test]
fn test_unwrap_or_default_safe_macro() {
    let value: Option<i32> = None;
    let result = unwrap_or_default_safe!(value, 99, "macro default test");
    assert_eq!(result, 99);
}

#[test]
fn test_try_unwrap_safe_macro() {
    let value = Some(42);
    let result = try_unwrap_safe!(value, "macro try test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_unwrap_stats() {
    use memscope_rs::core::{get_unwrap_stats, update_unwrap_stats};
    
    let initial_stats = get_unwrap_stats();
    let initial_success = initial_stats.successful_unwraps;
    let initial_failure = initial_stats.failed_unwraps;
    let initial_default = initial_stats.default_value_uses;
    let initial_panic = initial_stats.panic_preservations;
    
    // Test recording operations
    update_unwrap_stats(|stats| stats.record_success());
    let stats = get_unwrap_stats();
    assert_eq!(stats.successful_unwraps, initial_success + 1);
    
    update_unwrap_stats(|stats| stats.record_failure());
    let stats = get_unwrap_stats();
    assert_eq!(stats.failed_unwraps, initial_failure + 1);
    
    update_unwrap_stats(|stats| stats.record_default_use());
    let stats = get_unwrap_stats();
    assert_eq!(stats.default_value_uses, initial_default + 1);
    
    update_unwrap_stats(|stats| stats.record_panic_preservation());
    let stats = get_unwrap_stats();
    assert_eq!(stats.panic_preservations, initial_panic + 1);
    
    // Test total operations
    let expected_total = initial_success + initial_failure + initial_default + initial_panic + 4;
    assert_eq!(stats.total_operations(), expected_total);
    
    // Test success rate calculation
    if stats.total_operations() > 0 {
        let expected_rate = stats.successful_unwraps as f64 / stats.total_operations() as f64;
        assert!((stats.success_rate() - expected_rate).abs() < f64::EPSILON);
    }
}

#[test]
fn test_unwrap_stats_new() {
    let stats = UnwrapStats::new();
    assert_eq!(stats.successful_unwraps, 0);
    assert_eq!(stats.failed_unwraps, 0);
    assert_eq!(stats.default_value_uses, 0);
    assert_eq!(stats.panic_preservations, 0);
    assert_eq!(stats.total_operations(), 0);
    assert_eq!(stats.success_rate(), 0.0);
}

#[test]
fn test_unwrap_stats_success_rate_edge_cases() {
    let mut stats = UnwrapStats::new();
    
    // Test with zero operations
    assert_eq!(stats.success_rate(), 0.0);
    
    // Test with only successful operations
    stats.record_success();
    stats.record_success();
    assert_eq!(stats.success_rate(), 1.0);
    
    // Test with mixed operations
    stats.record_failure();
    assert!((stats.success_rate() - (2.0 / 3.0)).abs() < f64::EPSILON);
}

#[test]
fn test_context_preservation() {
    // Test that context information is preserved in error messages
    let none_value: Option<i32> = None;
    let result = none_value.try_unwrap_safe("specific context information");
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.user_message().contains("specific context information"));
}

#[test]
fn test_location_preservation() {
    // Test that location information is preserved
    let none_value: Option<i32> = None;
    let result = std::panic::catch_unwind(|| {
        none_value.unwrap_safe_at("location test", "specific_file.rs:123");
    });
    
    assert!(result.is_err());
    // The panic message should contain the location information
    // This is tested indirectly through the panic behavior
}

#[test]
fn test_chaining_operations() {
    // Test that safe unwrap operations can be chained
    let value = Some(Some(42));
    let result = value
        .unwrap_safe("outer unwrap")
        .unwrap_safe("inner unwrap");
    assert_eq!(result, 42);
    
    // Test with try_unwrap_safe chaining
    let value: Option<Result<i32, &str>> = Some(Ok(42));
    let result = value
        .try_unwrap_safe("outer try unwrap")
        .and_then(|inner| inner.try_unwrap_safe("inner try unwrap"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}