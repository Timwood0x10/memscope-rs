//! Render methods for dashboard.

use super::helpers::format_bytes;
use super::types::*;
use handlebars::Handlebars;

/// CDN asset scripts embedded at compile time for offline use
const TAILWIND_SCRIPT: &str = include_str!("../templates/assets/tailwind.min.js");
const CHART_SCRIPT: &str = include_str!("../templates/assets/chart.min.js");
const D3_SCRIPT: &str = include_str!("../templates/assets/d3.min.js");
const FONTS_CSS: &str = include_str!("../templates/assets/fonts.css");

/// Insert basic context into template data
pub fn insert_basic_context(
    template_data: &mut std::collections::BTreeMap<String, serde_json::Value>,
    context: &DashboardContext,
) {
    // This compatibility helper has only infallible scalar conversions.
    macro_rules! insert_fields {
        ($($field:ident),* $(,)?) => { $(
            template_data.insert(stringify!($field).into(), context.$field.clone().into());
        )* };
    }
    insert_fields!(
        title,
        export_timestamp,
        total_memory,
        total_allocations,
        active_allocations,
        peak_memory,
        thread_count,
        passport_count,
        leak_count,
        unsafe_count,
        ffi_count,
        health_score,
        health_status,
        safe_ops_count,
        high_risk_count,
        clean_passport_count,
        active_passport_count,
        leaked_passport_count,
        ffi_tracked_count,
        safe_code_percent,
        os_name,
        architecture,
        cpu_cores,
        json_data
    );
}

/// Render unified dashboard (multi-mode in single HTML)
pub fn render_unified_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    // Serialize the DTO contract as a whole so new metrics and nulls cannot be
    // omitted by this compatibility render path.
    let mut template_data: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_value(serde_json::to_value(context)?)?;
    template_data.insert(
        "total_smart_pointers".to_string(),
        serde_json::Value::Number(context.circular_references.total_smart_pointers.into()),
    );
    // Smart pointer type breakdown (Rc / Arc / Box / Weak)
    let mut sp_breakdown: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for alloc in &context.allocations {
        if alloc.is_smart_pointer {
            *sp_breakdown
                .entry(alloc.smart_pointer_type.clone())
                .or_insert(0) += 1;
        }
    }
    template_data.insert(
        "smart_pointer_breakdown".to_string(),
        serde_json::to_value(&sp_breakdown)?,
    );

    // Thread memory total: sum of current_memory_bytes across all threads
    let thread_mem_total: usize = context.threads.iter().map(|t| t.current_memory_bytes).sum();
    let thread_mem_total_fmt = if thread_mem_total >= 1_000_000 {
        format!("{:.1} MB", thread_mem_total as f64 / 1_000_000.0)
    } else if thread_mem_total >= 1_000 {
        format!("{:.1} KB", thread_mem_total as f64 / 1_000.0)
    } else {
        format!("{} B", thread_mem_total)
    };
    template_data.insert(
        "thread_memory_total".to_string(),
        serde_json::Value::Number(thread_mem_total.into()),
    );
    template_data.insert(
        "thread_memory_total_fmt".to_string(),
        serde_json::Value::String(thread_mem_total_fmt),
    );
    let duration_fmt = context
        .task_duration_mean_ms
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "—".into());
    template_data.insert(
        "task_duration_mean_ms_fmt".to_string(),
        serde_json::Value::String(duration_fmt),
    );

    template_data.insert(
        "tailwind_script".to_string(),
        serde_json::Value::String(TAILWIND_SCRIPT.to_string()),
    );
    template_data.insert(
        "chart_script".to_string(),
        serde_json::Value::String(CHART_SCRIPT.to_string()),
    );
    template_data.insert(
        "d3_script".to_string(),
        serde_json::Value::String(D3_SCRIPT.to_string()),
    );
    template_data.insert(
        "fonts_css".to_string(),
        serde_json::Value::String(FONTS_CSS.to_string()),
    );

    handlebars
        .render("dashboard_unified", &template_data)
        .map_err(|e| format!("Template rendering error: {}", e).into())
}

/// Render final dashboard (delegates to the merged dashboard_unified template)
pub fn render_final_dashboard(
    handlebars: &Handlebars<'static>,
    context: &DashboardContext,
) -> Result<String, Box<dyn std::error::Error>> {
    render_unified_dashboard(handlebars, context)
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
            "page_size": context.system_resources.page_size,
            "cpu_usage_pct": context.system_resources.cpu_usage_pct,
            "total_physical_bytes": context.system_resources.total_physical_bytes,
            "used_physical_bytes": context.system_resources.used_physical_bytes
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
mod tests;
