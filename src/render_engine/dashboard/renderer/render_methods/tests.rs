use super::*;
use crate::render_engine::dashboard::renderer::types::{
    AllocationInfo, AsyncSummary, CircularReferenceReport, DashboardContext, OwnershipGraphInfo,
    PassportDetail, RelationshipInfo, SystemResources, ThreadInfo, UnsafeReport,
};
use std::collections::BTreeMap;

fn create_empty_context() -> DashboardContext {
    DashboardContext {
        title: "Test".to_string(),
        export_timestamp: "2024-01-01".to_string(),
        total_memory: "0 B".to_string(),
        total_allocations: 0,
        active_allocations: 0,
        peak_memory: "0 B".to_string(),
        thread_count: 0,
        passport_count: 0,
        leak_count: 0,
        unsafe_count: 0,
        ffi_count: 0,
        allocations: vec![],
        relationships: vec![],
        unsafe_reports: vec![],
        passport_details: vec![],
        allocations_count: 0,
        relationships_count: 0,
        unsafe_reports_count: 0,
        json_data: "{}".to_string(),
        os_name: "test".to_string(),
        architecture: "test".to_string(),
        cpu_cores: 1,
        system_resources: SystemResources {
            os_name: "test".to_string(),
            os_version: "1.0".to_string(),
            architecture: "test".to_string(),
            cpu_cores: 1,
            total_physical: "0 B".to_string(),
            available_physical: "0 B".to_string(),
            used_physical: "0 B".to_string(),
            page_size: 4096,
            cpu_usage_pct: 0.0,
            total_physical_bytes: 0,
            used_physical_bytes: 0,
        },
        threads: vec![],
        async_tasks: vec![],
        async_summary: AsyncSummary {
            total_tasks: 0,
            active_tasks: 0,
            total_allocations: 0,
            total_memory_bytes: 0,
            peak_memory_bytes: 0,
            completed: 0,
            leaked: 0,
            zombie: 0,
            success_rate: 0.0,
        },
        health_score: 100,
        health_status: "Good".to_string(),
        safe_ops_count: 0,
        high_risk_count: 0,
        clean_passport_count: 0,
        active_passport_count: 0,
        leaked_passport_count: 0,
        ffi_tracked_count: 0,
        safe_code_percent: 100,
        ownership_graph: OwnershipGraphInfo {
            total_nodes: 0,
            total_edges: 0,
            total_cycles: 0,
            rc_clone_count: 0,
            arc_clone_count: 0,
            has_issues: false,
            issues: vec![],
            root_cause: None,
        },
        top_allocation_sites: vec![],
        top_leaked_allocations: vec![],
        top_temporary_churn: vec![],
        circular_references: CircularReferenceReport {
            count: 0,
            total_leaked_memory: 0,
            pointers_in_cycles: 0,
            total_smart_pointers: 0,
            has_cycles: false,
        },
        task_graph_json: "{}".to_string(),
        ffi_call_topology: Default::default(),
        symbol_table: vec![],
        symbol_table_count: 0,
        stack_integrity: Default::default(),
        resource_bars: vec![],
        thread_timeline: vec![],
        thread_timeline_count: 0,
        waker_efficiency_grid: vec![],
        task_duration_mean_ms: None,
        task_duration_samples: vec![],
        task_topology_nodes: vec![],
        task_topology_nodes_count: 0,
        task_topology_edges: vec![],
        task_topology_edges_count: 0,
        streaming_topology_stats: Default::default(),
        trace_logs: vec![],
        allocation_lifetimes: Default::default(),
        dependency_graph_nodes: vec![],
        selected_node_detail: None,
        thread_affinity_grid: vec![],
        allocation_interval_histogram: vec![],
        allocation_interval_mean_ms: None,
        allocation_interval_sample_count: 0,
        non_dominant_thread_allocation_share_pct: None,
        system_uptime_formatted: String::new(),
        thread_event_log: vec![],
        thread_policies: vec![],
        resource_limits: vec![],
    }
}

/// Objective: Verify that insert_basic_context populates all required fields.
/// Invariants: All basic context fields should be present in template_data.
#[test]
fn test_insert_basic_context() {
    let context = create_empty_context();
    let mut template_data = BTreeMap::new();
    insert_basic_context(&mut template_data, &context);

    assert!(
        template_data.contains_key("title"),
        "title should be present"
    );
    assert!(
        template_data.contains_key("export_timestamp"),
        "export_timestamp should be present"
    );
    assert!(
        template_data.contains_key("total_memory"),
        "total_memory should be present"
    );
    assert!(
        template_data.contains_key("total_allocations"),
        "total_allocations should be present"
    );
    assert!(
        template_data.contains_key("active_allocations"),
        "active_allocations should be present"
    );
    assert!(
        template_data.contains_key("peak_memory"),
        "peak_memory should be present"
    );
    assert!(
        template_data.contains_key("thread_count"),
        "thread_count should be present"
    );
    assert!(
        template_data.contains_key("passport_count"),
        "passport_count should be present"
    );
    assert!(
        template_data.contains_key("leak_count"),
        "leak_count should be present"
    );
    assert!(
        template_data.contains_key("unsafe_count"),
        "unsafe_count should be present"
    );
    assert!(
        template_data.contains_key("ffi_count"),
        "ffi_count should be present"
    );
    assert!(
        template_data.contains_key("health_score"),
        "health_score should be present"
    );
    assert!(
        template_data.contains_key("health_status"),
        "health_status should be present"
    );
    assert!(
        template_data.contains_key("os_name"),
        "os_name should be present"
    );
    assert!(
        template_data.contains_key("architecture"),
        "architecture should be present"
    );
    assert!(
        template_data.contains_key("cpu_cores"),
        "cpu_cores should be present"
    );
    assert!(
        template_data.contains_key("json_data"),
        "json_data should be present"
    );
}

/// Objective: Verify that insert_basic_context sets correct values.
/// Invariants: Values should match the context exactly.
#[test]
fn test_insert_basic_context_values() {
    let mut context = create_empty_context();
    context.title = "My Dashboard".to_string();
    context.total_allocations = 42;
    context.health_score = 85;

    let mut template_data = BTreeMap::new();
    insert_basic_context(&mut template_data, &context);

    assert_eq!(template_data["title"], "My Dashboard", "title should match");
    assert_eq!(
        template_data["total_allocations"], 42,
        "total_allocations should match"
    );
    assert_eq!(
        template_data["health_score"], 85,
        "health_score should match"
    );
}

/// Objective: Verify that format_bytes is correctly used.
/// Invariants: format_bytes should convert bytes to human-readable format.
#[test]
fn test_format_bytes_in_render() {
    assert_eq!(format_bytes(1024), "1.00 KB", "1024 bytes should be 1 KB");
    assert_eq!(
        format_bytes(1024 * 1024),
        "1.00 MB",
        "1MB should be formatted correctly"
    );
    assert_eq!(
        format_bytes(0),
        "0 bytes",
        "0 bytes should be formatted correctly"
    );
}

/// Objective: Verify that to_legacy_binary_data creates valid JSON.
/// Invariants: Output should be valid JSON with expected structure.
#[test]
fn test_to_legacy_binary_data() {
    let context = create_empty_context();
    let data = to_legacy_binary_data(&context);

    assert!(data.is_object(), "Output should be a JSON object");
    assert!(
        data.get("allocations").is_some(),
        "Should have allocations field"
    );
    assert!(
        data.get("relationships").is_some(),
        "Should have relationships field"
    );
    assert!(
        data.get("unsafe_reports").is_some(),
        "Should have unsafe_reports field"
    );
    assert!(data.get("threads").is_some(), "Should have threads field");
    assert!(
        data.get("passport_details").is_some(),
        "Should have passport_details field"
    );
}

/// Objective: Verify that to_legacy_binary_data includes context values.
/// Invariants: Context values should be present in output.
#[test]
fn test_to_legacy_binary_data_context_values() {
    let mut context = create_empty_context();
    context.total_allocations = 100;
    context.active_allocations = 50;
    context.leak_count = 5;

    let data = to_legacy_binary_data(&context);

    assert_eq!(
        data["total_allocations"], 100,
        "total_allocations should match"
    );
    assert_eq!(
        data["active_allocations"], 50,
        "active_allocations should match"
    );
    assert_eq!(data["leak_count"], 5, "leak_count should match");
}

/// Objective: Verify that to_legacy_binary_data handles allocations with data.
/// Invariants: Allocations should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_allocations() {
    let mut context = create_empty_context();
    context.allocations = vec![AllocationInfo {
        address: "0x1000".to_string(),
        type_name: "Vec<u8>".to_string(),
        size: 1024,
        var_name: "buffer".to_string(),
        timestamp: "2024-01-01".to_string(),
        thread_id: "Thread-1".to_string(),
        immutable_borrows: 0,
        mutable_borrows: 0,
        is_clone: false,
        clone_count: 0,
        timestamp_alloc: 1000,
        timestamp_dealloc: 0,
        lifetime_ms: None,
        observed_age_ms: None,
        is_leaked: false,
        allocation_state: crate::snapshot::AllocationState::live(),
        leak_status: crate::snapshot::LeakStatus::NotAssessed,
        allocation_type: "heap".to_string(),
        is_smart_pointer: false,
        smart_pointer_type: String::new(),
        source_file: None,
        source_line: None,
        module_path: None,
        generation_id: 0,
        provenance: String::new(),
        evidence: Default::default(),
        confidence: Default::default(),
        layout_snapshot: None,
    }];

    let data = to_legacy_binary_data(&context);

    let allocations = data.get("allocations").unwrap().as_array().unwrap();
    assert_eq!(allocations.len(), 1, "Should have one allocation");
    assert_eq!(allocations[0]["address"], "0x1000", "Address should match");
    assert_eq!(allocations[0]["size"], 1024, "Size should match");
}

/// Objective: Verify that to_legacy_binary_data handles relationships with data.
/// Invariants: Relationships should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_relationships() {
    let mut context = create_empty_context();
    context.relationships = vec![RelationshipInfo {
        source_ptr: "0x1000".to_string(),
        source_var_name: "var1".to_string(),
        target_ptr: "0x2000".to_string(),
        target_var_name: "var2".to_string(),
        relationship_type: "reference".to_string(),
        strength: 0.8,
        type_name: "String".to_string(),
        color: "#ff0000".to_string(),
        is_part_of_cycle: false,
        is_container_source: false,
        is_container_target: false,
    }];

    let data = to_legacy_binary_data(&context);

    let relationships = data.get("relationships").unwrap().as_array().unwrap();
    assert_eq!(relationships.len(), 1, "Should have one relationship");
    assert_eq!(
        relationships[0]["source_ptr"], "0x1000",
        "Source ptr should match"
    );
    assert_eq!(
        relationships[0]["target_ptr"], "0x2000",
        "Target ptr should match"
    );
}

/// Objective: Verify that to_legacy_binary_data handles threads with data.
/// Invariants: Threads should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_threads() {
    let mut context = create_empty_context();
    context.threads = vec![ThreadInfo {
        thread_id: "Thread-1".to_string(),
        thread_summary: "Main thread".to_string(),
        allocation_count: 10,
        current_memory: "1 KB".to_string(),
        peak_memory: "2 KB".to_string(),
        total_allocated: "10 KB".to_string(),
        current_memory_bytes: 1024,
        peak_memory_bytes: 2048,
        total_allocated_bytes: 10240,
        is_active: true,
        status: "ACTIVE".to_string(),
        cpu_core: None,
    }];

    let data = to_legacy_binary_data(&context);

    let threads = data.get("threads").unwrap().as_array().unwrap();
    assert_eq!(threads.len(), 1, "Should have one thread");
    assert_eq!(
        threads[0]["thread_id"], "Thread-1",
        "Thread ID should match"
    );
    assert_eq!(
        threads[0]["allocation_count"], 10,
        "Allocation count should match"
    );
}

/// Objective: Verify that to_legacy_binary_data handles ownership graph.
/// Invariants: Ownership graph should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_ownership_graph() {
    let mut context = create_empty_context();
    context.ownership_graph = OwnershipGraphInfo {
        total_nodes: 10,
        total_edges: 15,
        total_cycles: 2,
        rc_clone_count: 5,
        arc_clone_count: 3,
        has_issues: true,
        issues: vec![],
        root_cause: None,
    };

    let data = to_legacy_binary_data(&context);

    let graph = data.get("ownership_graph").unwrap();
    assert_eq!(graph["total_nodes"], 10, "Total nodes should match");
    assert_eq!(graph["total_edges"], 15, "Total edges should match");
    assert_eq!(graph["total_cycles"], 2, "Total cycles should match");
}

/// Objective: Verify that to_legacy_binary_data handles system resources.
/// Invariants: System resources should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_system_resources() {
    let mut context = create_empty_context();
    context.system_resources = SystemResources {
        os_name: "macOS".to_string(),
        os_version: "14.0".to_string(),
        architecture: "arm64".to_string(),
        cpu_cores: 8,
        total_physical: "16 GB".to_string(),
        available_physical: "8 GB".to_string(),
        used_physical: "8 GB".to_string(),
        page_size: 4096,
        cpu_usage_pct: 0.0,
        total_physical_bytes: 16 * 1024 * 1024 * 1024,
        used_physical_bytes: 8 * 1024 * 1024 * 1024,
    };

    let data = to_legacy_binary_data(&context);

    let resources = data.get("system_resources").unwrap();
    assert_eq!(resources["os_name"], "macOS", "OS name should match");
    assert_eq!(resources["cpu_cores"], 8, "CPU cores should match");
    assert_eq!(resources["page_size"], 4096, "Page size should match");
}

/// Objective: Verify that to_legacy_binary_data handles unsafe reports with data.
/// Invariants: Unsafe reports should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_unsafe_reports() {
    let mut context = create_empty_context();
    context.unsafe_reports = vec![UnsafeReport {
        passport_id: "passport-1".to_string(),
        generation: 1,
        superseded: false,
        allocation_ptr: "0x1000".to_string(),
        var_name: "buffer".to_string(),
        type_name: "Vec<u8>".to_string(),
        size_bytes: 1024,
        created_at: 1000,
        updated_at: 2000,
        status: "active".to_string(),
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        is_leaked: false,
        risk_level: "medium".to_string(),
        risk_factors: vec!["FFI boundary".to_string()],
        description: "FFI boundary".to_string(),
    }];

    let data = to_legacy_binary_data(&context);

    let reports = data.get("unsafe_reports").unwrap().as_array().unwrap();
    assert_eq!(reports.len(), 1, "Should have one unsafe report");
    assert_eq!(
        reports[0]["passport_id"], "passport-1",
        "Passport ID should match"
    );
    assert_eq!(
        reports[0]["risk_level"], "medium",
        "Risk level should match"
    );
}

/// Objective: Verify that to_legacy_binary_data handles passport details with data.
/// Invariants: Passport details should be properly serialized.
#[test]
fn test_to_legacy_binary_data_with_passport_details() {
    let mut context = create_empty_context();
    context.passport_details = vec![PassportDetail {
        passport_id: "passport-1".to_string(),
        generation: 1,
        superseded: false,
        allocation_ptr: "0x1000".to_string(),
        var_name: "buffer".to_string(),
        type_name: "Vec<u8>".to_string(),
        size_bytes: 1024,
        status: "active".to_string(),
        created_at: 1000,
        updated_at: 2000,
        is_leaked: false,
        ffi_tracked: true,
        lifecycle_events: vec![],
        cross_boundary_events: vec![],
        risk_level: "low".to_string(),
        risk_confidence: 0.85,
        is_active: true,
        source_location: "test.rs:1".to_string(),
    }];

    let data = to_legacy_binary_data(&context);

    let passports = data.get("passport_details").unwrap().as_array().unwrap();
    assert_eq!(passports.len(), 1, "Should have one passport detail");
    assert_eq!(
        passports[0]["passport_id"], "passport-1",
        "Passport ID should match"
    );
    assert_eq!(
        passports[0]["ffi_tracked"], true,
        "FFI tracked should match"
    );
}

/// Objective: Verify that insert_basic_context handles all fields correctly.
/// Invariants: All context fields should be present in template data.
#[test]
fn test_insert_basic_context_all_fields() {
    let mut context = create_empty_context();
    context.title = "Test Dashboard".to_string();
    context.export_timestamp = "2024-01-01 12:00:00 UTC".to_string();
    context.total_memory = "1.5 GB".to_string();
    context.peak_memory = "2.0 GB".to_string();
    context.health_status = "Excellent".to_string();
    context.safe_ops_count = 100;
    context.high_risk_count = 5;
    context.clean_passport_count = 50;
    context.active_passport_count = 10;
    context.leaked_passport_count = 2;
    context.ffi_tracked_count = 15;
    context.safe_code_percent = 95;

    let mut template_data = BTreeMap::new();
    insert_basic_context(&mut template_data, &context);

    assert_eq!(
        template_data["title"], "Test Dashboard",
        "title should match"
    );
    assert_eq!(
        template_data["total_memory"], "1.5 GB",
        "total_memory should match"
    );
    assert_eq!(
        template_data["peak_memory"], "2.0 GB",
        "peak_memory should match"
    );
    assert_eq!(
        template_data["health_status"], "Excellent",
        "health_status should match"
    );
    assert_eq!(
        template_data["safe_ops_count"], 100,
        "safe_ops_count should match"
    );
    assert_eq!(
        template_data["high_risk_count"], 5,
        "high_risk_count should match"
    );
}
