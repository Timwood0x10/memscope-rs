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
