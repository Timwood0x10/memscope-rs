//! Render methods for dashboard.

use super::helpers::format_bytes;
use super::types::*;
use handlebars::Handlebars;

/// Insert basic context into template data
pub fn insert_basic_context(
    template_data: &mut std::collections::BTreeMap<String, serde_json::Value>,
    context: &DashboardContext,
) {
    template_data.insert(
        "title".to_string(),
        serde_json::Value::String(context.title.clone()),
    );
    template_data.insert(
        "export_timestamp".to_string(),
        serde_json::Value::String(context.export_timestamp.clone()),
    );
    template_data.insert(
        "total_memory".to_string(),
        serde_json::Value::String(context.total_memory.clone()),
    );
    template_data.insert(
        "total_allocations".to_string(),
        serde_json::Value::Number(context.total_allocations.into()),
    );
    template_data.insert(
        "active_allocations".to_string(),
        serde_json::Value::Number(context.active_allocations.into()),
    );
    template_data.insert(
        "peak_memory".to_string(),
        serde_json::Value::String(context.peak_memory.clone()),
    );
    template_data.insert(
        "thread_count".to_string(),
        serde_json::Value::Number(context.thread_count.into()),
    );
    template_data.insert(
        "passport_count".to_string(),
        serde_json::Value::Number(context.passport_count.into()),
    );
    template_data.insert(
        "leak_count".to_string(),
        serde_json::Value::Number(context.leak_count.into()),
    );
    template_data.insert(
        "unsafe_count".to_string(),
        serde_json::Value::Number(context.unsafe_count.into()),
    );
    template_data.insert(
        "ffi_count".to_string(),
        serde_json::Value::Number(context.ffi_count.into()),
    );
    template_data.insert(
        "health_score".to_string(),
        serde_json::Value::Number(context.health_score.into()),
    );
    template_data.insert(
        "health_status".to_string(),
        serde_json::Value::String(context.health_status.clone()),
    );
    template_data.insert(
        "safe_ops_count".to_string(),
        serde_json::Value::Number(context.safe_ops_count.into()),
    );
    template_data.insert(
        "high_risk_count".to_string(),
        serde_json::Value::Number(context.high_risk_count.into()),
    );
    template_data.insert(
        "clean_passport_count".to_string(),
        serde_json::Value::Number(context.clean_passport_count.into()),
    );
    template_data.insert(
        "active_passport_count".to_string(),
        serde_json::Value::Number(context.active_passport_count.into()),
    );
    template_data.insert(
        "leaked_passport_count".to_string(),
        serde_json::Value::Number(context.leaked_passport_count.into()),
    );
    template_data.insert(
        "ffi_tracked_count".to_string(),
        serde_json::Value::Number(context.ffi_tracked_count.into()),
    );
    template_data.insert(
        "safe_code_percent".to_string(),
        serde_json::Value::Number(context.safe_code_percent.into()),
    );
    template_data.insert(
        "os_name".to_string(),
        serde_json::Value::String(context.os_name.clone()),
    );
    template_data.insert(
        "architecture".to_string(),
        serde_json::Value::String(context.architecture.clone()),
    );
    template_data.insert(
        "cpu_cores".to_string(),
        serde_json::Value::Number(context.cpu_cores.into()),
    );
    template_data.insert(
        "json_data".to_string(),
        serde_json::Value::String(context.json_data.clone()),
    );
}

/// Render unified dashboard (multi-mode in single HTML)
pub fn render_unified_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut template_data = std::collections::BTreeMap::new();
    insert_basic_context(&mut template_data, context);
    template_data.insert(
        "allocations_count".to_string(),
        serde_json::Value::Number(context.allocations_count.into()),
    );
    template_data.insert(
        "relationships_count".to_string(),
        serde_json::Value::Number(context.relationships_count.into()),
    );
    template_data.insert(
        "unsafe_reports_count".to_string(),
        serde_json::Value::Number(context.unsafe_reports_count.into()),
    );

    template_data.insert(
        "allocations".to_string(),
        serde_json::to_value(&context.allocations)?,
    );
    template_data.insert(
        "passport_details".to_string(),
        serde_json::to_value(&context.passport_details)?,
    );
    template_data.insert(
        "relationships".to_string(),
        serde_json::to_value(&context.relationships)?,
    );
    template_data.insert(
        "unsafe_reports".to_string(),
        serde_json::to_value(&context.unsafe_reports)?,
    );
    template_data.insert(
        "threads".to_string(),
        serde_json::to_value(&context.threads)?,
    );
    template_data.insert(
        "ownership_graph".to_string(),
        serde_json::to_value(&context.ownership_graph)?,
    );

    handlebars
        .render("dashboard_unified", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Render final dashboard (new investigation console template)
pub fn render_final_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut template_data = std::collections::BTreeMap::new();
    insert_basic_context(&mut template_data, context);
    template_data.insert(
        "allocations".to_string(),
        serde_json::to_value(&context.allocations)?,
    );
    template_data.insert(
        "passport_details".to_string(),
        serde_json::to_value(&context.passport_details)?,
    );
    template_data.insert(
        "relationships".to_string(),
        serde_json::to_value(&context.relationships)?,
    );
    template_data.insert(
        "unsafe_reports".to_string(),
        serde_json::to_value(&context.unsafe_reports)?,
    );
    template_data.insert(
        "threads".to_string(),
        serde_json::to_value(&context.threads)?,
    );
    template_data.insert(
        "async_tasks".to_string(),
        serde_json::to_value(&context.async_tasks)?,
    );
    template_data.insert(
        "ownership_graph".to_string(),
        serde_json::to_value(&context.ownership_graph)?,
    );

    handlebars
        .render("dashboard_final", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Convert to legacy binary data format
pub fn to_legacy_binary_data(context: &DashboardContext) -> serde_json::Value {
    serde_json::json!({
        "allocations": context.allocations.iter().map(|a| {
            serde_json::json!({
                "address": a.address,
                "type_name": a.type_name,
                "size": a.size,
                "var_name": a.var_name,
                "timestamp": a.timestamp,
                "thread_id": a.thread_id,
                "is_leaked": a.is_leaked,
                "timestamp_alloc": a.timestamp_alloc,
                "timestamp_dealloc": a.timestamp_dealloc,
                "lifetime_ms": a.lifetime_ms,
                "allocation_type": a.allocation_type,
                "is_smart_pointer": a.is_smart_pointer,
                "smart_pointer_type": a.smart_pointer_type,
                "source_file": a.source_file,
                "source_line": a.source_line
            })
        }).collect::<Vec<_>>(),
        "relationships": context.relationships.iter().map(|r| {
            serde_json::json!({
                "source_ptr": r.source_ptr,
                "source_var_name": r.source_var_name,
                "target_ptr": r.target_ptr,
                "target_var_name": r.target_var_name,
                "relationship_type": r.relationship_type,
                "strength": r.strength,
                "type_name": r.type_name,
                "color": r.color,
                "is_part_of_cycle": r.is_part_of_cycle
            })
        }).collect::<Vec<_>>(),
        "unsafe_reports": context.unsafe_reports.iter().map(|u| {
            serde_json::json!({
                "passport_id": u.passport_id,
                "allocation_ptr": u.allocation_ptr,
                "var_name": u.var_name,
                "type_name": u.type_name,
                "size_bytes": u.size_bytes,
                "is_leaked": u.is_leaked,
                "risk_level": u.risk_level,
                "risk_factors": u.risk_factors
            })
        }).collect::<Vec<_>>(),
        "threads": context.threads.iter().map(|t| {
            serde_json::json!({
                "thread_id": t.thread_id,
                "thread_summary": t.thread_summary,
                "allocation_count": t.allocation_count,
                "current_memory": t.current_memory,
                "peak_memory": t.peak_memory,
                "total_allocated": t.total_allocated
            })
        }).collect::<Vec<_>>(),
        "passport_details": context.passport_details.iter().map(|p| {
            serde_json::json!({
                "passport_id": p.passport_id,
                "allocation_ptr": p.allocation_ptr,
                "var_name": p.var_name,
                "type_name": p.type_name,
                "size_bytes": p.size_bytes,
                "status": p.status,
                "is_leaked": p.is_leaked,
                "ffi_tracked": p.ffi_tracked,
                "risk_level": p.risk_level
            })
        }).collect::<Vec<_>>(),
        "total_memory": context.total_memory,
        "total_allocations": context.total_allocations,
        "active_allocations": context.active_allocations,
        "peak_memory": context.peak_memory,
        "thread_count": context.thread_count,
        "passport_count": context.passport_count,
        "leak_count": context.leak_count,
        "unsafe_count": context.unsafe_count,
        "ffi_count": context.ffi_count,
        "health_score": context.health_score,
        "health_status": context.health_status,
        "safe_ops_count": context.safe_ops_count,
        "high_risk_count": context.high_risk_count,
        "clean_passport_count": context.clean_passport_count,
        "active_passport_count": context.active_passport_count,
        "leaked_passport_count": context.leaked_passport_count,
        "ffi_tracked_count": context.ffi_tracked_count,
        "safe_code_percent": context.safe_code_percent,
        "os_name": context.os_name,
        "architecture": context.architecture,
        "cpu_cores": context.cpu_cores,
        "system_resources": {
            "os_name": context.system_resources.os_name,
            "os_version": context.system_resources.os_version,
            "architecture": context.system_resources.architecture,
            "cpu_cores": context.system_resources.cpu_cores,
            "total_physical": context.system_resources.total_physical,
            "available_physical": context.system_resources.available_physical,
            "used_physical": context.system_resources.used_physical,
            "page_size": context.system_resources.page_size
        },
        "ownership_graph": {
            "total_nodes": context.ownership_graph.total_nodes,
            "total_edges": context.ownership_graph.total_edges,
            "total_cycles": context.ownership_graph.total_cycles,
            "rc_clone_count": context.ownership_graph.rc_clone_count,
            "arc_clone_count": context.ownership_graph.arc_clone_count,
            "has_issues": context.ownership_graph.has_issues
        }
    })
}

/// Render binary dashboard (legacy template)
pub fn render_binary_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let legacy_data = to_legacy_binary_data(context);
    let mut template_data = std::collections::BTreeMap::new();
    template_data.insert("BINARY_DATA".to_string(), legacy_data);
    template_data.insert(
        "PROJECT_NAME".to_string(),
        serde_json::Value::String("MemScope Memory Analysis".to_string()),
    );

    handlebars
        .render("binary_dashboard", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Render clean dashboard (legacy template)
pub fn render_clean_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let legacy_data = to_legacy_binary_data(context);
    let mut template_data = std::collections::BTreeMap::new();
    template_data.insert("BINARY_DATA".to_string(), legacy_data.clone());
    template_data.insert("json_data".to_string(), legacy_data);
    template_data.insert(
        "PROJECT_NAME".to_string(),
        serde_json::Value::String("MemScope Memory Analysis".to_string()),
    );

    handlebars
        .render("clean_dashboard", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Render hybrid dashboard (legacy template)
pub fn render_hybrid_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let variables_data = serde_json::Value::Array(
        context
            .allocations
            .iter()
            .map(|a| {
                let mut map = serde_json::Map::new();
                map.insert(
                    "var_name".to_string(),
                    serde_json::Value::String(a.var_name.clone()),
                );
                map.insert(
                    "type_name".to_string(),
                    serde_json::Value::String(a.type_name.clone()),
                );
                map.insert("size".to_string(), serde_json::Value::Number(a.size.into()));
                map.insert(
                    "address".to_string(),
                    serde_json::Value::String(a.address.clone()),
                );
                map.insert(
                    "is_leaked".to_string(),
                    serde_json::Value::Bool(a.is_leaked),
                );
                map.insert(
                    "timestamp_alloc".to_string(),
                    serde_json::Value::Number(a.timestamp_alloc.into()),
                );
                map.insert(
                    "timestamp_dealloc".to_string(),
                    serde_json::Value::Number(a.timestamp_dealloc.into()),
                );
                map.insert(
                    "thread_id".to_string(),
                    serde_json::Value::String(a.thread_id.clone()),
                );
                serde_json::Value::Object(map)
            })
            .collect(),
    );

    let threads_data = serde_json::Value::Array(
        context
            .threads
            .iter()
            .map(|t| {
                let mut map = serde_json::Map::new();
                map.insert(
                    "thread_id".to_string(),
                    serde_json::Value::String(t.thread_id.clone()),
                );
                map.insert(
                    "allocation_count".to_string(),
                    serde_json::Value::String(t.allocation_count.to_string()),
                );
                map.insert(
                    "current_memory".to_string(),
                    serde_json::Value::String(t.current_memory.clone()),
                );
                map.insert(
                    "peak_memory".to_string(),
                    serde_json::Value::String(t.peak_memory.clone()),
                );
                map.insert(
                    "total_allocated".to_string(),
                    serde_json::Value::String(t.total_allocated.clone()),
                );
                serde_json::Value::Object(map)
            })
            .collect(),
    );

    let tasks_data = serde_json::Value::Array(Vec::new());

    let total_memory: usize = context.allocations.iter().map(|a| a.size).sum();
    let efficiency = if context.total_allocations > 0 {
        (context.active_allocations as f64 / context.total_allocations as f64 * 100.0) as usize
    } else {
        100
    };

    let mut template_data = std::collections::BTreeMap::new();
    template_data.insert("VARIABLES_DATA".to_string(), variables_data);
    template_data.insert("THREADS_DATA".to_string(), threads_data);
    template_data.insert("TASKS_DATA".to_string(), tasks_data);
    template_data.insert(
        "PROJECT_NAME".to_string(),
        serde_json::Value::String("MemScope Memory Analysis".to_string()),
    );
    template_data.insert(
        "TOTAL_MEMORY".to_string(),
        serde_json::Value::String(format_bytes(total_memory)),
    );
    template_data.insert(
        "TOTAL_VARIABLES".to_string(),
        serde_json::Value::Number(context.allocations.len().into()),
    );
    template_data.insert(
        "THREAD_COUNT".to_string(),
        serde_json::Value::Number(context.thread_count.into()),
    );
    template_data.insert(
        "EFFICIENCY".to_string(),
        serde_json::Value::String(format!("{}%", efficiency)),
    );

    handlebars
        .render("hybrid_dashboard", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Render performance dashboard (legacy template)
pub fn render_performance_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let performance_data = serde_json::json!({
        "allocations": context.allocations.iter().map(|a| {
            serde_json::json!({
                "timestamp": a.timestamp_alloc,
                "memory": a.size,
                "var_name": a.var_name,
                "type_name": a.type_name
            })
        }).collect::<Vec<_>>(),
        "total_memory": context.total_memory,
        "peak_memory": context.peak_memory,
        "thread_count": context.thread_count,
        "leak_count": context.leak_count
    });

    let mut template_data = std::collections::BTreeMap::new();
    template_data.insert(
        "PERFORMANCE_DATA".to_string(),
        serde_json::to_value(&performance_data)?,
    );
    template_data.insert(
        "PROJECT_NAME".to_string(),
        serde_json::Value::String("MemScope Memory Analysis".to_string()),
    );

    handlebars
        .render("performance_dashboard", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_engine::dashboard::renderer::types::{
        AllocationInfo, AsyncSummary, CircularReferenceReport, DashboardContext,
        OwnershipGraphInfo, PassportDetail, RelationshipInfo, SystemResources, ThreadInfo,
        UnsafeReport,
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
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
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
            lifetime_ms: 0.0,
            is_leaked: false,
            allocation_type: "heap".to_string(),
            is_smart_pointer: false,
            smart_pointer_type: String::new(),
            source_file: None,
            source_line: None,
            module_path: None,
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
}
