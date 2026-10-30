//! Unit tests for the dashboard report builder.
//!
//! Split out of `report_builder.rs` to keep that file within the line budget.

use super::*;

/// Objective: Verify that HealthInfo struct can be created.
#[test]
fn test_health_info_creation() {
    let info = HealthInfo {
        health_score: 85,
        health_status: "Good".to_string(),
        safe_ops_count: 100,
        high_risk_count: 5,
        clean_passport_count: 50,
        active_passport_count: 10,
        leaked_passport_count: 2,
        ffi_tracked_count: 15,
        safe_code_percent: 95,
    };
    assert_eq!(info.health_score, 85);
    assert_eq!(info.health_status, "Good");
    assert_eq!(info.safe_ops_count, 100);
}

/// Objective: Verify that calculate_health_info returns positive for empty data.
#[test]
fn test_calculate_health_info_empty() {
    let health = calculate_health_info(&[], &[], 0, 0);
    assert!(health.health_score > 0);
    assert!(health.health_status.contains("Excellent") || health.health_status.contains("Good"));
}

/// Objective: Verify that calculate_health_info penalizes leaks.
#[test]
fn test_calculate_health_info_with_leaks() {
    let health_no_leaks = calculate_health_info(&[], &[], 0, 100);
    let health_with_leaks = calculate_health_info(&[], &[], 50, 100);
    assert!(health_no_leaks.health_score > health_with_leaks.health_score);
}

/// Objective: Verify that calculate_health_info penalizes high risk.
#[test]
fn test_calculate_health_info_with_unsafe() {
    let safe_report = UnsafeReport {
        passport_id: "1".to_string(),
        allocation_ptr: "0x1000".to_string(),
        var_name: "var".to_string(),
        type_name: "i32".to_string(),
        size_bytes: 4,
        created_at: 0,
        updated_at: 0,
        status: "active".to_string(),
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        is_leaked: false,
        risk_level: "low".to_string(),
        risk_factors: vec![],
        description: "no risk factors".to_string(),
    };
    let high_risk_report = UnsafeReport {
        passport_id: "2".to_string(),
        allocation_ptr: "0x2000".to_string(),
        var_name: "var2".to_string(),
        type_name: "i32".to_string(),
        size_bytes: 4,
        created_at: 0,
        updated_at: 0,
        status: "active".to_string(),
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        is_leaked: false,
        risk_level: "high".to_string(),
        risk_factors: vec![],
        description: "no risk factors".to_string(),
    };
    let health_safe = calculate_health_info(&[safe_report], &[], 0, 100);
    let health_high_risk = calculate_health_info(&[high_risk_report], &[], 0, 100);
    assert!(health_safe.health_score > health_high_risk.health_score);
}

/// Objective: Verify that calculate_health_info handles leaked passports.
#[test]
fn test_calculate_health_info_with_passports() {
    let leaked_passport = PassportDetail {
        passport_id: "1".to_string(),
        allocation_ptr: "0x1000".to_string(),
        var_name: "var".to_string(),
        type_name: "i32".to_string(),
        size_bytes: 4,
        status: "leaked".to_string(),
        created_at: 0,
        updated_at: 0,
        is_leaked: true,
        ffi_tracked: false,
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        risk_level: "high".to_string(),
        risk_confidence: 0.9,
        is_active: false,
        source_location: "test.rs:1".to_string(),
    };
    let clean_passport = PassportDetail {
        passport_id: "2".to_string(),
        allocation_ptr: "0x2000".to_string(),
        var_name: "var2".to_string(),
        type_name: "i32".to_string(),
        size_bytes: 4,
        status: "active".to_string(),
        created_at: 0,
        updated_at: 0,
        is_leaked: false,
        ffi_tracked: false,
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        risk_level: "low".to_string(),
        risk_confidence: 0.5,
        is_active: true,
        source_location: "test.rs:2".to_string(),
    };
    let health_clean = calculate_health_info(&[], &[clean_passport], 0, 100);
    let health_leaked = calculate_health_info(&[], &[leaked_passport], 1, 100);
    assert!(health_clean.clean_passport_count >= health_leaked.clean_passport_count);
}

/// Objective: Verify that build_top_n_reports handles empty input.
#[test]
fn test_build_top_n_reports_empty() {
    let reports = build_top_n_reports(&[]);
    assert!(reports.top_allocation_sites.is_empty());
    assert!(reports.top_leaked_allocations.is_empty());
    assert!(reports.top_temporary_churn.is_empty());
}

/// Objective: Verify that aggregate_thread_data handles empty input.
#[test]
fn test_aggregate_thread_data_empty() {
    let threads = aggregate_thread_data(&[]);
    assert!(threads.is_empty());
}

/// Objective: Verify that build_async_tasks returns empty for no tracker.
#[test]
fn test_build_async_tasks_no_tracker() {
    let tasks = build_async_tasks(None);
    assert!(tasks.is_empty());
}

/// Objective: Verify that build_async_summary returns zeros for no tracker.
#[test]
fn test_build_async_summary_no_tracker() {
    let summary = build_async_summary(None, &[]);
    assert_eq!(summary.total_tasks, 0);
    assert_eq!(summary.active_tasks, 0);
    assert_eq!(summary.completed, 0);
    assert_eq!(summary.leaked, 0);
    assert_eq!(summary.zombie, 0);
    assert_eq!(summary.success_rate, 0.0);
}
