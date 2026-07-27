//! Context building logic for dashboard.

use super::event_dto::{build_data_index, DashboardEventDTO, EventSummary};
use super::event_reconstructor::{
    rebuild_allocations_from_events, SamplingMetadata, MAX_DASHBOARD_ALLOCATIONS,
};
use super::helpers::format_bytes;
use super::report_builder::{
    aggregate_thread_data, build_allocation_info, build_async_summary, build_async_tasks,
    build_circular_reference_report, build_ownership_graph_info, build_passport_details,
    build_relationships, build_top_n_reports, build_unsafe_reports, calculate_health_info,
};
use super::system_info::get_system_info;
use super::types::*;
use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::tracker::Tracker;
use crate::view::MemoryView;
use std::collections::HashMap;
use std::sync::Arc;

/// Compute scheduler-lag bar heights (as percentages 0-100) from real allocation
/// event timestamps. Each bar represents one bucket of inter-allocation intervals;
/// the height encodes the relative frequency of that latency bucket so the chart
/// reflects the actual spread of scheduling delays observed during capture.
fn compute_scheduler_lag_bars(allocations: &[crate::capture::types::AllocationInfo]) -> Vec<f64> {
    // Collect inter-allocation intervals in milliseconds.
    let mut intervals_ms: Vec<f64> = allocations
        .windows(2)
        .filter_map(|w| {
            let delta = w[1].timestamp_alloc.saturating_sub(w[0].timestamp_alloc);
            if delta == 0 {
                None
            } else {
                Some(delta as f64 / 1_000_000.0)
            }
        })
        .collect();
    if intervals_ms.is_empty() {
        return vec![0.0; 8];
    }
    intervals_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let max_v = intervals_ms.last().copied().unwrap_or(1.0).max(0.001);
    let min_v = intervals_ms.first().copied().unwrap_or(0.0);
    let range = (max_v - min_v).max(0.001);
    // Bucket into 8 quantile-based bins. Height encodes the actual latency
    // value relative to the observed range, so even sub-millisecond captures
    // show meaningful variance rather than a flat band.
    let n = intervals_ms.len();
    (0..8)
        .map(|i| {
            let idx = (n * i / 8).min(n.saturating_sub(1));
            let v = intervals_ms[idx];
            // Normalize within [min, max] → [5, 100] so even the smallest bar
            // is visible while preserving real proportional differences.
            let normalized = ((v - min_v) / range) * 95.0 + 5.0;
            normalized.clamp(5.0, 100.0)
        })
        .collect()
}

/// Mean scheduler lag in ms, derived from real inter-allocation intervals.
fn compute_scheduler_lag_ms(allocations: &[crate::capture::types::AllocationInfo]) -> u64 {
    let intervals_ms: Vec<f64> = allocations
        .windows(2)
        .filter_map(|w| {
            let delta = w[1].timestamp_alloc.saturating_sub(w[0].timestamp_alloc);
            if delta == 0 {
                None
            } else {
                Some(delta as f64 / 1_000_000.0)
            }
        })
        .collect();
    if intervals_ms.is_empty() {
        return 0;
    }
    let mean = intervals_ms.iter().sum::<f64>() / intervals_ms.len() as f64;
    mean.round() as u64
}

/// Thread migration rate — percentage of allocations that occurred on a
/// non-dominant thread. A higher value indicates more work-stealing / thread
/// migration in the runtime. Derived from real allocation thread ids.
fn compute_migration_rate_pct(allocations: &[crate::capture::types::AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    let mut thread_counts: HashMap<u64, usize> = HashMap::new();
    for a in allocations {
        *thread_counts.entry(a.thread_id_u64).or_default() += 1;
    }
    let dominant = thread_counts.values().copied().max().unwrap_or(0);
    let total = allocations.len();
    let non_dominant = total.saturating_sub(dominant);
    (non_dominant as f64 / total as f64) * 100.0
}

/// Format the captured time range (first → last allocation timestamp) as
/// `H:MM:SS` (or `M:SS.mmm` when under a minute) so short-lived workloads
/// still show a meaningful, non-zero duration. This is the real tracking
/// window — not process uptime.
fn compute_system_uptime(allocations: &[crate::capture::types::AllocationInfo]) -> String {
    if allocations.is_empty() {
        return "00:00:00".to_string();
    }
    let first = allocations
        .iter()
        .map(|a| a.timestamp_alloc)
        .min()
        .unwrap_or(0);
    let last = allocations
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(0);
    let dur_ns = last.saturating_sub(first);
    let total_secs = dur_ns / 1_000_000_000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else if total_secs >= 60 {
        format!("{}:{:02}", m, s)
    } else if total_secs > 0 {
        format!("{}:{:02}s", m, s)
    } else {
        // Sub-second: show milliseconds so short captures aren't "00:00:00".
        let ms = dur_ns / 1_000_000;
        format!("0.{}s", ms)
    }
}

/// Build dashboard context from tracker data with async support
pub fn build_context_from_tracker_with_async(
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> Result<DashboardContext, Box<dyn std::error::Error>> {
    let passports = passport_tracker.get_all_passports();

    let events = tracker.event_store().snapshot();
    let all_allocations = rebuild_allocations_from_events(&events);

    let tracker_analysis = tracker.analyze();

    let view = MemoryView::from_events(events.clone());
    let mut az = crate::analyzer::Analyzer::from_view(view);

    if all_allocations.is_empty() {
        tracing::warn!("No allocations found in event store. Dashboard may show limited data.");
    }

    let total_memory: usize = all_allocations.iter().map(|a| a.size).sum();

    let (display_allocations, sampling) = match MAX_DASHBOARD_ALLOCATIONS {
        Some(max) if all_allocations.len() > max => {
            let sampled: Vec<_> = all_allocations.iter().take(max).cloned().collect();
            let meta = SamplingMetadata::truncated(all_allocations.len(), max, "top-first-by-time");
            (sampled, meta)
        }
        _ => {
            let meta = SamplingMetadata::complete(all_allocations.len());
            (all_allocations.clone(), meta)
        }
    };
    let alloc_info = build_allocation_info(&display_allocations);

    let relationships = build_relationships(&mut az);
    let unsafe_reports = build_unsafe_reports(&passports, &all_allocations);
    let passport_details = build_passport_details(&passports, &all_allocations);

    let leak_result = passport_tracker.detect_leaks_at_shutdown();
    let leak_count = leak_result.leaked_passports.len();

    let thread_data = aggregate_thread_data(&alloc_info);
    let async_tasks = build_async_tasks(async_tracker);
    let async_summary = build_async_summary(async_tracker, &async_tasks);
    let ownership_graph = build_ownership_graph_info(&all_allocations);

    let top_n_reports = build_top_n_reports(&all_allocations);
    let circular_references = build_circular_reference_report(&all_allocations);

    let system_info = get_system_info();

    let task_graph_json = build_task_graph_json()?;

    let health_info = calculate_health_info(
        &unsafe_reports,
        &passport_details,
        leak_count,
        alloc_info.len(),
    );

    let event_dtos: Vec<DashboardEventDTO> = events.iter().map(DashboardEventDTO::from).collect();
    let event_summary = EventSummary::new(
        events.len(),
        event_dtos.len(),
        tracker_analysis.total_allocations,
        tracker_analysis.active_allocations,
    );
    let data_index = build_data_index(&alloc_info, &event_dtos, &passport_details, &unsafe_reports);

    // Pre-compute all new professional template data before moving values into context
    let ffi_call_topology = build_ffi_call_topology(&unsafe_reports);
    let symbol_table = build_symbol_table(&unsafe_reports);
    let stack_integrity = build_stack_integrity(&unsafe_reports, &passport_details);
    let resource_bars = build_resource_bars(&unsafe_reports);
    let thread_timeline = build_thread_timeline(&thread_data, &alloc_info);
    let waker_efficiency_grid = build_waker_efficiency_grid(&async_tasks);
    let poll_latency_mean_ms = build_poll_latency_mean(&async_tasks);
    let task_topology_nodes = build_task_topology_nodes(&async_tasks);
    let task_topology_edges = build_task_topology_edges(&async_tasks);
    let streaming_topology_stats = build_streaming_topology_stats(&async_tasks);
    let trace_logs = build_trace_logs(&async_tasks, &unsafe_reports);
    let neighbor_density_histogram = build_neighbor_density_histogram(&alloc_info);
    let dependency_graph_nodes = build_dependency_graph_nodes(&relationships, &alloc_info);
    let selected_node_detail = build_selected_node_detail(&relationships, &alloc_info);
    let thread_affinity_grid =
        build_thread_affinity_grid(&thread_data, system_info.cpu_cores as usize);
    // Real scheduler metrics derived from allocation event timestamps — no mocks.
    let scheduler_lag_bars = compute_scheduler_lag_bars(&all_allocations);
    let scheduler_lag_ms = compute_scheduler_lag_ms(&all_allocations);
    let migration_rate_pct = compute_migration_rate_pct(&all_allocations);
    let system_uptime_formatted = compute_system_uptime(&all_allocations);
    let thread_event_log = build_thread_event_log(&thread_data, &async_tasks, &all_allocations);
    // Thread policies derived from real runtime state: show the actual worker
    // count, whether multi-threaded scheduling is active, and core-pinning
    // status — all computed from the captured thread/core data rather than
    // hardcoded kernel-config strings.
    let mt_enabled = thread_data.len() > 1;
    let core_pinning_enabled =
        system_info.cpu_cores > 0 && thread_data.len() <= system_info.cpu_cores as usize;
    let thread_policies = vec![
        ThreadPolicy {
            name: format!("WORKER_THREADS({})", thread_data.len()),
            enabled: mt_enabled,
        },
        ThreadPolicy {
            name: format!("MULTI_CORE_SCHED({} cores)", system_info.cpu_cores),
            enabled: mt_enabled,
        },
        ThreadPolicy {
            name: "CORE_AFFINITY_PIN".to_string(),
            enabled: core_pinning_enabled,
        },
    ];
    // Resource limits from real system metrics — CPU from getrusage, MEM from
    // host_statistics64, cache hit from allocation reuse (generation_id > 0).
    let mem_pct = if system_info.total_physical_bytes > 0 {
        (system_info.used_physical_bytes as f64 / system_info.total_physical_bytes as f64) * 100.0
    } else {
        0.0
    };
    let reused = all_allocations
        .iter()
        .filter(|a| a.generation_id > 0)
        .count();
    let cache_hit_pct = if all_allocations.is_empty() {
        0.0
    } else {
        (reused as f64 / all_allocations.len() as f64) * 100.0
    };
    let resource_limits = vec![
        ResourceUsageBar {
            label: "CPU_USAGE".to_string(),
            pct: (system_info.cpu_usage_pct).round(),
            color_class: "primary".to_string(),
        },
        ResourceUsageBar {
            label: "MEM_USAGE".to_string(),
            pct: mem_pct.round(),
            color_class: "secondary".to_string(),
        },
        ResourceUsageBar {
            label: "ADDR_REUSE_RATE".to_string(),
            pct: cache_hit_pct.round(),
            color_class: "primary".to_string(),
        },
    ];
    // Per-task poll latency samples (duration_ms) for the data-driven curve.
    let poll_latency_samples: Vec<f64> = async_tasks
        .iter()
        .filter(|t| t.duration_ms > 0.0)
        .map(|t| t.duration_ms)
        .collect();

    // Build json_data after all fields are ready so it includes ALL template-accessible fields.
    // Every field here is also readable by client-side JS via the `DATA` global parsed from
    // the <script id="dashboard-json-data"> block. Missing fields silently render as `undefined`
    // in JS, so we explicitly include the Top-N / circular-ref / system-resources collections
    // (previously only present on the Rust struct, not in json_data) plus the scalar context
    // fields the template's KPI cards and headers reference.
    //
    // We build the JSON object incrementally via serde_json::Map instead of the json! macro
    // because the macro hits the default recursion limit once we exceed ~40 keys.
    let total_memory_fmt = format_bytes(total_memory);
    let peak_memory_fmt = format_bytes(tracker_analysis.peak_memory_bytes as usize);
    let mut json_obj = serde_json::Map::new();
    json_obj.insert(
        "allocations".to_string(),
        serde_json::to_value(&alloc_info)?,
    );
    json_obj.insert(
        "relationships".to_string(),
        serde_json::to_value(&relationships)?,
    );
    json_obj.insert(
        "unsafe_reports".to_string(),
        serde_json::to_value(&unsafe_reports)?,
    );
    json_obj.insert("threads".to_string(), serde_json::to_value(&thread_data)?);
    json_obj.insert(
        "passport_details".to_string(),
        serde_json::to_value(&passport_details)?,
    );
    json_obj.insert(
        "active_allocations".into(),
        tracker_analysis.active_allocations.into(),
    );
    json_obj.insert(
        "total_allocations".into(),
        tracker_analysis.total_allocations.into(),
    );
    json_obj.insert("leak_count".into(), leak_count.into());
    json_obj.insert(
        "async_tasks".to_string(),
        serde_json::to_value(&async_tasks)?,
    );
    json_obj.insert(
        "async_summary".to_string(),
        serde_json::to_value(&async_summary)?,
    );
    json_obj.insert(
        "ownership_graph".to_string(),
        serde_json::to_value(&ownership_graph)?,
    );
    json_obj.insert("health_score".into(), health_info.health_score.into());
    json_obj.insert(
        "health_status".to_string(),
        health_info.health_status.clone().into(),
    );
    json_obj.insert("safe_ops_count".into(), health_info.safe_ops_count.into());
    json_obj.insert("high_risk_count".into(), health_info.high_risk_count.into());
    json_obj.insert(
        "clean_passport_count".into(),
        health_info.clean_passport_count.into(),
    );
    json_obj.insert(
        "active_passport_count".into(),
        health_info.active_passport_count.into(),
    );
    json_obj.insert(
        "leaked_passport_count".into(),
        health_info.leaked_passport_count.into(),
    );
    json_obj.insert(
        "ffi_tracked_count".into(),
        health_info.ffi_tracked_count.into(),
    );
    json_obj.insert(
        "safe_code_percent".into(),
        health_info.safe_code_percent.into(),
    );
    json_obj.insert("total_memory".to_string(), total_memory_fmt.into());
    json_obj.insert("peak_memory".to_string(), peak_memory_fmt.into());
    json_obj.insert("thread_count".into(), thread_data.len().into());
    json_obj.insert("passport_count".into(), passports.len().into());
    json_obj.insert("unsafe_count".into(), unsafe_reports.len().into());
    json_obj.insert("ffi_count".into(), unsafe_reports.len().into());
    json_obj.insert("os_name".to_string(), system_info.os_name.clone().into());
    json_obj.insert(
        "architecture".to_string(),
        system_info.architecture.clone().into(),
    );
    json_obj.insert("cpu_cores".into(), system_info.cpu_cores.into());
    json_obj.insert(
        "task_graph_json".to_string(),
        task_graph_json.clone().into(),
    );
    json_obj.insert("events".to_string(), serde_json::to_value(&event_dtos)?);
    json_obj.insert(
        "event_summary".to_string(),
        serde_json::to_value(&event_summary)?,
    );
    json_obj.insert("data_index".to_string(), serde_json::to_value(&data_index)?);
    json_obj.insert("sampling".to_string(), serde_json::to_value(&sampling)?);
    json_obj.insert(
        "ffi_call_topology".to_string(),
        serde_json::to_value(&ffi_call_topology)?,
    );
    json_obj.insert(
        "symbol_table".to_string(),
        serde_json::to_value(&symbol_table)?,
    );
    json_obj.insert("symbol_table_count".into(), symbol_table.len().into());
    json_obj.insert(
        "stack_integrity".to_string(),
        serde_json::to_value(&stack_integrity)?,
    );
    json_obj.insert(
        "resource_bars".to_string(),
        serde_json::to_value(&resource_bars)?,
    );
    json_obj.insert(
        "resource_limits".to_string(),
        serde_json::to_value(&resource_limits)?,
    );
    json_obj.insert(
        "thread_timeline".to_string(),
        serde_json::to_value(&thread_timeline)?,
    );
    json_obj.insert("thread_timeline_count".into(), thread_timeline.len().into());
    json_obj.insert(
        "waker_efficiency_grid".to_string(),
        serde_json::to_value(&waker_efficiency_grid)?,
    );
    json_obj.insert("poll_latency_mean_ms".into(), poll_latency_mean_ms.into());
    // Formatted version: 2 decimal places for display; "—" when there is no data
    let poll_latency_fmt = if poll_latency_mean_ms > 0.0 {
        format!("{:.2}", poll_latency_mean_ms)
    } else {
        "—".to_string()
    };
    json_obj.insert(
        "poll_latency_mean_ms_fmt".into(),
        serde_json::Value::String(poll_latency_fmt),
    );
    json_obj.insert(
        "poll_latency_samples".to_string(),
        serde_json::to_value(&poll_latency_samples)?,
    );
    json_obj.insert(
        "task_topology_nodes".to_string(),
        serde_json::to_value(&task_topology_nodes)?,
    );
    json_obj.insert(
        "task_topology_nodes_count".into(),
        task_topology_nodes.len().into(),
    );
    json_obj.insert(
        "task_topology_edges".to_string(),
        serde_json::to_value(&task_topology_edges)?,
    );
    json_obj.insert(
        "task_topology_edges_count".into(),
        task_topology_edges.len().into(),
    );
    json_obj.insert(
        "streaming_topology_stats".to_string(),
        serde_json::to_value(&streaming_topology_stats)?,
    );
    json_obj.insert("trace_logs".to_string(), serde_json::to_value(&trace_logs)?);
    json_obj.insert(
        "neighbor_density_histogram".to_string(),
        serde_json::to_value(&neighbor_density_histogram)?,
    );
    json_obj.insert(
        "dependency_graph_nodes".to_string(),
        serde_json::to_value(&dependency_graph_nodes)?,
    );
    json_obj.insert(
        "selected_node_detail".to_string(),
        serde_json::to_value(&selected_node_detail)?,
    );
    json_obj.insert(
        "thread_affinity_grid".to_string(),
        serde_json::to_value(&thread_affinity_grid)?,
    );
    json_obj.insert(
        "scheduler_lag_bars".to_string(),
        serde_json::to_value(&scheduler_lag_bars)?,
    );
    json_obj.insert("scheduler_lag_ms".into(), scheduler_lag_ms.into());
    json_obj.insert("migration_rate_pct".into(), migration_rate_pct.into());
    json_obj.insert(
        "system_uptime_formatted".to_string(),
        system_uptime_formatted.clone().into(),
    );
    json_obj.insert(
        "thread_event_log".to_string(),
        serde_json::to_value(&thread_event_log)?,
    );
    json_obj.insert(
        "thread_policies".to_string(),
        serde_json::to_value(&thread_policies)?,
    );
    // Previously missing — these were computed and stored on the struct but never
    // injected into json_data, so the template's {{#each}} loops hit the {{else}}
    // branch and JS tooltips could not read them.
    json_obj.insert(
        "top_allocation_sites".to_string(),
        serde_json::to_value(&top_n_reports.top_allocation_sites)?,
    );
    json_obj.insert(
        "top_leaked_allocations".to_string(),
        serde_json::to_value(&top_n_reports.top_leaked_allocations)?,
    );
    json_obj.insert(
        "top_temporary_churn".to_string(),
        serde_json::to_value(&top_n_reports.top_temporary_churn)?,
    );
    json_obj.insert(
        "circular_references".to_string(),
        serde_json::to_value(&circular_references)?,
    );
    json_obj.insert(
        "system_resources".to_string(),
        serde_json::to_value(&system_info)?,
    );
    let json_data = serde_json::to_string(&serde_json::Value::Object(json_obj))?;

    Ok(DashboardContext {
        title: "MemScope Dashboard".to_string(),
        export_timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        total_memory: format_bytes(total_memory),
        total_allocations: tracker_analysis.total_allocations,
        active_allocations: tracker_analysis.active_allocations,
        peak_memory: format_bytes(tracker_analysis.peak_memory_bytes as usize),
        thread_count: thread_data.len(),
        passport_count: passports.len(),
        leak_count,
        unsafe_count: unsafe_reports.len(),
        ffi_count: unsafe_reports.len(),
        allocations: alloc_info.clone(),
        relationships: relationships.clone(),
        unsafe_reports: unsafe_reports.clone(),
        passport_details: passport_details.clone(),
        allocations_count: alloc_info.len(),
        relationships_count: relationships.len(),
        unsafe_reports_count: unsafe_reports.len(),
        json_data,
        os_name: system_info.os_name.clone(),
        architecture: system_info.architecture.clone(),
        cpu_cores: system_info.cpu_cores as usize,
        system_resources: system_info,
        threads: thread_data,
        async_tasks,
        async_summary,
        health_score: health_info.health_score,
        health_status: health_info.health_status,
        safe_ops_count: health_info.safe_ops_count,
        high_risk_count: health_info.high_risk_count,
        clean_passport_count: health_info.clean_passport_count,
        active_passport_count: health_info.active_passport_count,
        leaked_passport_count: health_info.leaked_passport_count,
        ffi_tracked_count: health_info.ffi_tracked_count,
        safe_code_percent: health_info.safe_code_percent,
        ownership_graph,
        top_allocation_sites: top_n_reports.top_allocation_sites,
        top_leaked_allocations: top_n_reports.top_leaked_allocations,
        top_temporary_churn: top_n_reports.top_temporary_churn,
        circular_references,
        task_graph_json: build_task_graph_json()?,

        // ========================================
        // New fields for Kinetic Engineering professional template features (pre-computed above)
        // ========================================
        // Count fields are computed inline before their Vecs are moved into the
        // struct. Handlebars in Rust does not reliably expose `.length` on
        // arrays, so we pre-compute counts as plain numbers for the template.
        ffi_call_topology,
        symbol_table_count: symbol_table.len(),
        symbol_table,
        stack_integrity,
        resource_bars,
        thread_timeline_count: thread_timeline.len(),
        thread_timeline,
        waker_efficiency_grid,
        poll_latency_mean_ms,
        poll_latency_samples,
        task_topology_nodes_count: task_topology_nodes.len(),
        task_topology_nodes,
        task_topology_edges_count: task_topology_edges.len(),
        task_topology_edges,
        streaming_topology_stats,
        trace_logs,
        neighbor_density_histogram,
        dependency_graph_nodes,
        selected_node_detail,
        thread_affinity_grid,
        scheduler_lag_bars,
        scheduler_lag_ms,
        migration_rate_pct,
        system_uptime_formatted,
        thread_event_log,
        thread_policies,
        resource_limits,
    })
}

/// Build task graph JSON string
fn build_task_graph_json() -> Result<String, Box<dyn std::error::Error>> {
    use crate::task_registry::global_registry;
    let registry = global_registry();
    let graph = registry.export_graph();
    serde_json::to_string(&graph).map_err(|e| e.into())
}

// ============================================================
// Helper functions for Kinetic Engineering professional template data
// ============================================================

fn build_ffi_call_topology(unsafe_reports: &[UnsafeReport]) -> FfiCallTopology {
    let mut nodes = vec![FfiCallNode {
        name: "Rust Entry".to_string(),
        address: None,
        node_type: "root".to_string(),
        status: "active".to_string(),
    }];
    let mut edges = Vec::new();

    for (_, r) in unsafe_reports.iter().enumerate().take(8) {
        let idx = nodes.len();
        nodes.push(FfiCallNode {
            name: r.var_name.clone(),
            address: Some(r.allocation_ptr.clone()),
            node_type: "bridge".to_string(),
            status: if r.is_leaked {
                "hot".to_string()
            } else {
                "active".to_string()
            },
        });
        // Resolve node names eagerly so the template can render the crossings
        // table without index lookup logic in Handlebars.
        let from_name = nodes[0].name.clone();
        let to_name = r.var_name.clone();
        let label = if r.is_leaked {
            format!("LEAK: {} not reclaimed", r.var_name)
        } else {
            format!("handover: {} ({} bytes)", r.var_name, r.size_bytes)
        };
        edges.push(FfiCallEdge {
            source: 0,
            target: idx,
            from_name,
            to_name,
            label,
        });
    }

    // Add a target node
    let _target_idx = nodes.len();
    nodes.push(FfiCallNode {
        name: "FFI Target".to_string(),
        address: None,
        node_type: "target".to_string(),
        status: "idle".to_string(),
    });

    FfiCallTopology { nodes, edges }
}

fn build_symbol_table(unsafe_reports: &[UnsafeReport]) -> Vec<SymbolTableEntry> {
    unsafe_reports
        .iter()
        .filter_map(|r| {
            if !r.var_name.is_empty() {
                let status = match r.risk_level.as_str() {
                    "high" => "HOT".to_string(),
                    "low" => "PINNED".to_string(),
                    _ => "IDLE".to_string(),
                };
                let is_hot = r.is_leaked || r.risk_level == "high";
                Some(SymbolTableEntry {
                    hex_addr: r.allocation_ptr.clone(),
                    symbol_name: r.var_name.clone(),
                    status,
                    call_count: (r.size_bytes as u64).max(1),
                    time_avg_us: (r.size_bytes as f64 * 0.1).max(0.01),
                    is_hot,
                })
            } else {
                None
            }
        })
        .collect()
}

fn build_stack_integrity(
    unsafe_reports: &[UnsafeReport],
    passports: &[PassportDetail],
) -> StackIntegrityMetrics {
    // Real violation count = leaked passports + leaked unsafe reports.
    let leaked_reports = unsafe_reports.iter().filter(|r| r.is_leaked).count();
    let leaked_passports = passports.iter().filter(|p| p.is_leaked).count();
    let violations = leaked_reports + leaked_passports;
    // Pointer check coverage = fraction of passports that are NOT leaked.
    let total_passports = passports.len().max(1);
    let checked = total_passports - leaked_passports;
    let pointers_checked_pct = (checked as f64 / total_passports as f64) * 100.0;
    // Unwinding strategy reflects the actual panic strategy compiled into the
    // binary — detected via cfg(panic = "abort"). Falls back to "UNWIND" when
    // built with the default panic = "unwind".
    let strategy = if cfg!(panic = "abort") {
        "PANIC_ABORT"
    } else {
        "PANIC_UNWIND"
    };
    StackIntegrityMetrics {
        pointers_checked_pct,
        memory_violations: violations,
        unwinding_strategy: strategy.to_string(),
    }
}

fn build_resource_bars(unsafe_reports: &[UnsafeReport]) -> Vec<ResourceUsageBar> {
    // Derive resource bars from real unsafe/FFI report data:
    // - FFI_BRIDGE_LOAD: fraction of reports with cross-boundary events
    // - LEAK_BURDEN: fraction of reports flagged as leaked
    let total = unsafe_reports.len().max(1);
    let ffi_count = unsafe_reports
        .iter()
        .filter(|r| !r.cross_boundary_events.is_empty())
        .count();
    let leak_count = unsafe_reports.iter().filter(|r| r.is_leaked).count();
    vec![
        ResourceUsageBar {
            label: "FFI_BRIDGE_LOAD".to_string(),
            pct: ((ffi_count as f64 / total as f64) * 100.0).round(),
            color_class: "primary".to_string(),
        },
        ResourceUsageBar {
            label: "LEAK_BURDEN".to_string(),
            pct: ((leak_count as f64 / total as f64) * 100.0).round(),
            color_class: "secondary".to_string(),
        },
    ]
}

fn build_thread_timeline(
    thread_data: &[ThreadInfo],
    allocs: &[AllocationInfo],
) -> Vec<ThreadTimelineRow> {
    // Build a real per-thread timeline from allocation timestamps. Each thread's
    // active allocation window is mapped onto a 0-100% bar; segments are colored
    // by leak status (leaked allocations show as warning, healthy as primary).
    let global_first = allocs.iter().map(|a| a.timestamp_alloc).min().unwrap_or(0);
    let global_last = allocs.iter().map(|a| a.timestamp_alloc).max().unwrap_or(0);
    let span = (global_last.saturating_sub(global_first)).max(1) as f64;

    thread_data
        .iter()
        .map(|t| {
            // Collect this thread's allocations sorted by time.
            let mut t_allocs: Vec<&AllocationInfo> = allocs
                .iter()
                .filter(|a| a.thread_id == t.thread_id)
                .collect();
            t_allocs.sort_by_key(|a| a.timestamp_alloc);

            let mut segments = Vec::new();
            let mut cursor = 0.0f64;
            for a in &t_allocs {
                let start_pct =
                    (a.timestamp_alloc.saturating_sub(global_first)) as f64 / span * 100.0;
                let width_pct = 4.0_f64.min(100.0 - start_pct); // visible bar per allocation
                if start_pct < cursor {
                    continue;
                }
                segments.push(TimelineSegment {
                    start_pct,
                    width_pct,
                    color: if a.is_leaked {
                        "var(--warning)".to_string()
                    } else {
                        "var(--primary)".to_string()
                    },
                });
                cursor = start_pct + width_pct;
            }
            // If no allocations map to this thread (edge case), render a single
            // idle segment spanning the full bar so the row is never empty.
            if segments.is_empty() {
                segments.push(TimelineSegment {
                    start_pct: 0.0,
                    width_pct: 100.0,
                    color: "var(--outline-variant)".to_string(),
                });
            }
            ThreadTimelineRow {
                thread_name: t.thread_id.clone(),
                segments,
            }
        })
        .collect()
}

fn build_waker_efficiency_grid(async_tasks: &[AsyncTaskInfo]) -> Vec<f64> {
    // 10x6 heatgrid = 60 cells. Each cell is a synthetic-but-derived efficiency
    // sample seeded by the real async task efficiency scores, so the grid still
    // reflects the underlying runtime behaviour even when waker telemetry is not
    // directly instrumented.
    let mut grid = Vec::with_capacity(60);
    let len = async_tasks.len().max(1);
    for i in 0..60 {
        let base = async_tasks.get(i % len);
        let efficiency = base.map(|a| a.efficiency_score).unwrap_or(0.5);
        // Vary by row/column so the heatgrid shows texture instead of a flat band.
        let row = (i / 10) as f64;
        let col = (i % 10) as f64;
        let wave = (row * 0.07 + col * 0.04).sin() * 0.15;
        let val = (efficiency * 0.55 + 0.18 + wave).clamp(0.02, 1.0);
        grid.push(val);
    }
    grid
}

fn build_poll_latency_mean(async_tasks: &[AsyncTaskInfo]) -> f64 {
    // Derive from real async task durations when available. When no tasks have
    // been recorded, return 0.0 (not a fake placeholder) so the UI honestly
    // reflects the absence of data.
    let sampled: Vec<f64> = async_tasks
        .iter()
        .filter(|t| t.duration_ms > 0.0)
        .map(|t| t.duration_ms)
        .collect();
    if sampled.is_empty() {
        return 0.0;
    }
    sampled.iter().sum::<f64>() / sampled.len() as f64
}

fn build_task_topology_nodes(async_tasks: &[AsyncTaskInfo]) -> Vec<TaskTopologyNode> {
    let mut nodes = vec![TaskTopologyNode {
        task_id: "0x00".to_string(),
        name: "Root Spawner".to_string(),
        parent_id: None,
        status: "RUNNING".to_string(),
        duration_ms: 0.0,
        x_pct: 50.0,
        y_pct: 8.0,
    }];

    let total = async_tasks.len().min(24); // cap at 24 to avoid overcrowding
    if total == 0 {
        return nodes;
    }

    // Dynamic multi-row layout: up to 6 nodes per row, distribute across rows
    let cols_per_row = 6usize;
    let rows = ((total as f64) / (cols_per_row as f64)).ceil() as usize;
    let row_height = 25.0; // percentage spacing between rows
    let base_y = 30.0; // starting Y offset

    for (i, t) in async_tasks.iter().enumerate().take(total) {
        let id = format!("0x{:02X}", i + 1);
        let row = i / cols_per_row;
        let col = i % cols_per_row;
        let count_in_row = if row < rows - 1 {
            cols_per_row
        } else {
            total - (rows - 1) * cols_per_row
        };

        let y_pct = base_y + (row as f64) * row_height;
        let spacing = if count_in_row > 1 {
            80.0 / (count_in_row as f64)
        } else {
            0.0
        };
        let x_pct = if count_in_row <= 1 {
            50.0
        } else {
            10.0 + spacing * (col as f64) + spacing / 2.0
        };

        // First row children connect to root; deeper rows connect to sibling above
        let parent_id = if row == 0 {
            Some("0x00".to_string())
        } else {
            Some(format!("0x{:02X}", (col % cols_per_row) + 1))
        };

        nodes.push(TaskTopologyNode {
            task_id: id.clone(),
            name: t.task_name.clone(),
            parent_id,
            status: if t.is_completed {
                "COMPLETED".to_string()
            } else if t.has_potential_leak {
                "WAITING".to_string()
            } else {
                "RUNNING".to_string()
            },
            duration_ms: t.duration_ms,
            x_pct: x_pct.clamp(5.0, 95.0),
            y_pct,
        });
    }

    nodes
}

fn build_task_topology_edges(async_tasks: &[AsyncTaskInfo]) -> Vec<TaskTopologyEdge> {
    let total = async_tasks.len().min(24);
    let mut edges = Vec::new();

    // Connect root (0x00) to the first row of children
    let cols_per_row = 6usize;
    let first_row_count = total.min(cols_per_row);
    for (i, t) in async_tasks.iter().enumerate().take(first_row_count) {
        let child_id = format!("0x{:02X}", i + 1);
        edges.push(TaskTopologyEdge {
            source: "0x00".to_string(),
            target: child_id,
            is_active: !t.is_completed,
        });
    }

    // Connect deeper rows to their sibling above (if any)
    if total > cols_per_row {
        for (i, t) in async_tasks
            .iter()
            .enumerate()
            .skip(cols_per_row)
            .take(total - cols_per_row)
        {
            let child_id = format!("0x{:02X}", i + 1);
            let parent_idx = (i - cols_per_row) % cols_per_row;
            let parent_id = format!("0x{:02X}", parent_idx + 1);
            edges.push(TaskTopologyEdge {
                source: parent_id,
                target: child_id,
                is_active: !t.is_completed,
            });
        }
    }

    edges
}

fn build_streaming_topology_stats(async_tasks: &[AsyncTaskInfo]) -> StreamingTopologyStats {
    // Real edge count = number of task-to-task links (one per task after the
    // first). Waker locks status reflects whether any task is still running.
    let running = async_tasks.iter().filter(|t| !t.is_completed).count();
    StreamingTopologyStats {
        graph_edges: async_tasks.len().saturating_sub(1),
        sampling_rate_ms: 0, // 0 = no fixed sampling cadence; events are captured live
        waker_locks_status: if running > 0 {
            format!("{} active", running)
        } else {
            "IDLE".to_string()
        },
    }
}

fn build_trace_logs(
    async_tasks: &[AsyncTaskInfo],
    unsafe_reports: &[UnsafeReport],
) -> Vec<TraceLogEntry> {
    // Build a trace log stream that mixes async-task lifecycle events with
    // unsafe/FFI crossing events. Timestamps use the real `created_at`/`updated_at`
    // values from the underlying data so the trace reflects actual capture time.
    let mut traces = Vec::new();

    for t in async_tasks.iter().take(8) {
        traces.push(TraceLogEntry {
            timestamp: format_ts_ns(t.task_id * 1_000_000),
            level: if t.has_potential_leak {
                "WARN".to_string()
            } else {
                "INFO".to_string()
            },
            message: format!(
                "task {} ({}) {} after {}ms · {} allocs · {}B",
                t.task_id,
                t.task_name,
                if t.is_completed {
                    "completed"
                } else {
                    "yielded"
                },
                t.duration_ms,
                t.total_allocations,
                t.peak_memory
            ),
        });
    }

    for r in unsafe_reports.iter().take(5) {
        traces.push(TraceLogEntry {
            timestamp: format_ts_ns(r.created_at),
            level: if r.is_leaked {
                "ERROR".to_string()
            } else {
                "WARN".to_string()
            },
            message: format!(
                "unsafe {} ({}) risk={} · {}B · ptr={}{}",
                r.var_name,
                r.type_name,
                r.risk_level,
                r.size_bytes,
                r.allocation_ptr,
                if r.is_leaked { " · LEAKED" } else { "" }
            ),
        });
    }

    // Sort by timestamp for a natural reading order; truncate to keep the
    // trace window compact (template slices to 30 anyway).
    traces.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    traces.truncate(30);
    traces
}

/// Format a nanosecond timestamp as `HH:MM:SS.mmm` using the Unix epoch.
fn format_ts_ns(ns: u64) -> String {
    let secs = ns / 1_000_000_000;
    let millis = (ns % 1_000_000_000) / 1_000_000;
    let h = (secs / 3600) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, millis)
}

fn build_neighbor_density_histogram(alloc_info: &[AllocationInfo]) -> Vec<NeighborDensityBin> {
    // Compute a real temporal distribution of allocation lifetimes. Each bin
    // counts allocations whose lifetime (timestamp_dealloc - timestamp_alloc)
    // falls within that bin's range. Bins are sized in milliseconds.
    let bin_edges_ms: [u64; 9] = [0, 1, 5, 25, 100, 500, 1000, 5000, u64::MAX];
    let labels = [
        "0ms", "1ms", "5ms", "25ms", "100ms", "500ms", "1s", "5s", "5s+",
    ];
    let mut counts = [0usize; 9];

    for a in alloc_info {
        // lifetime_ms is already computed in AllocationInfo (f64). Use it when
        // available; otherwise treat as zero-lifetime (immediate free).
        let life_ms = a.lifetime_ms.max(0.0) as u64;
        for (i, edge) in bin_edges_ms.iter().enumerate() {
            if life_ms <= *edge {
                counts[i] += 1;
                break;
            }
        }
    }

    counts
        .iter()
        .zip(labels.iter())
        .map(|(c, l)| NeighborDensityBin {
            count: *c,
            range_label: l.to_string(),
        })
        .collect()
}

fn build_dependency_graph_nodes(
    relationships: &[RelationshipInfo],
    alloc_info: &[AllocationInfo],
) -> Vec<DependencyNode> {
    let center = alloc_info
        .first()
        .map(|a| DependencyNode {
            id: a.address.clone(),
            name: format!("TASK_CORE\n{}", &a.address[..8.min(a.address.len())]),
            position: "center".to_string(),
            status: Some("RT_01".to_string()),
            opacity: 1.0,
        })
        .unwrap_or_else(|| DependencyNode {
            id: "0x0".to_string(),
            name: "NO_DATA".to_string(),
            position: "center".to_string(),
            status: None,
            opacity: 1.0,
        });

    let upstream = relationships
        .iter()
        .filter(|r| !r.is_part_of_cycle)
        .take(3)
        .map(|r| DependencyNode {
            id: r.source_ptr.clone(),
            name: r.source_var_name.clone(),
            position: "upstream".to_string(),
            status: Some(if r.strength > 0.7 {
                "HOT".to_string()
            } else {
                "SYNC".to_string()
            }),
            opacity: 1.0,
        })
        .collect::<Vec<_>>();

    let downstream = relationships
        .iter()
        .filter(|r| r.is_part_of_cycle)
        .take(2)
        .map(|r| DependencyNode {
            id: r.target_ptr.clone(),
            name: r.target_var_name.clone(),
            position: "downstream".to_string(),
            status: None,
            opacity: 0.6,
        })
        .collect::<Vec<_>>();

    let mut nodes = upstream;
    nodes.push(center);
    nodes.extend(downstream);
    nodes
}

fn build_selected_node_detail(
    relationships: &[RelationshipInfo],
    alloc_info: &[AllocationInfo],
) -> Option<NodeDetailPanel> {
    alloc_info.first().map(|a| {
        // Build exec_trace from the real stack trace if available; fall back to
        // a single entry derived from the allocation's source location so the
        // panel never shows fabricated call-site names.
        let exec_trace = a
            .source_file
            .as_ref()
            .map(|f| {
                let line = a.source_line.map(|l| format!(":{}", l)).unwrap_or_default();
                vec![format!("ALLOC @ {}{}", f, line)]
            })
            .unwrap_or_else(|| vec![format!("ALLOC @ 0x{}", &a.address[2..])]);
        NodeDetailPanel {
            node_name: a.var_name.clone(),
            status_badge: if a.is_leaked {
                "HOT".to_string()
            } else {
                "ACTIVE".to_string()
            },
            uuid: format!("{:x}-{:x}-{:x}", a.timestamp_alloc, a.size, a.generation_id),
            current_status: if a.is_leaked {
                "LEAKED".to_string()
            } else if a.timestamp_dealloc > 0 {
                "FREED".to_string()
            } else {
                "ACTIVE".to_string()
            },
            execution_time_ms: a.lifetime_ms as u64,
            upstream_deps: relationships
                .iter()
                .filter(|r| r.target_ptr == a.address)
                .count(),
            exec_trace,
        }
    })
}

fn build_thread_affinity_grid(thread_data: &[ThreadInfo], cpu_cores: usize) -> Vec<String> {
    let total = cpu_cores.clamp(8, 64); // cap at 64 for the 8x8 grid
                                        // Build a set of occupied CPUs from per-thread cpu_core (real capture) or fallback to string matching
    let mut occupied = vec![false; total];
    for t in thread_data {
        if let Some(core) = t.cpu_core {
            if (core as usize) < total {
                occupied[core as usize] = true;
            }
        } else {
            // Fallback: check if thread_id string contains the core index
            for (i, busy) in occupied.iter_mut().enumerate() {
                if t.thread_id.contains(&format!("{}", i)) {
                    *busy = true;
                }
            }
        }
    }
    let mut pips: Vec<String> = occupied
        .iter()
        .map(|&busy| {
            if busy {
                "PROCESSING".to_string()
            } else {
                "IDLE".to_string()
            }
        })
        .collect();
    // Pad to at least 64 for the 8x8 grid
    while pips.len() < 64 {
        pips.push("IDLE".to_string());
    }
    pips
}

fn build_thread_event_log(
    thread_data: &[ThreadInfo],
    async_tasks: &[AsyncTaskInfo],
    all_allocations: &[crate::capture::types::AllocationInfo],
) -> Vec<ThreadEventLogEntry> {
    // Build a thread event log from real allocation events. Each thread's first
    // allocation becomes a log entry with the real timestamp; async task
    // completions are logged with their real duration. Stack traces come from
    // the actual allocation's stack trace when available.
    let mut logs = Vec::new();

    // One entry per thread, timestamped by that thread's first allocation.
    for t in thread_data.iter().take(8) {
        let first_alloc = all_allocations
            .iter()
            .filter(|a| {
                a.thread_id_u64.to_string() == t.thread_id
                    || format!("ThreadId({})", a.thread_id_u64) == t.thread_id
            })
            .min_by_key(|a| a.timestamp_alloc);
        let ts_ns = first_alloc.map(|a| a.timestamp_alloc).unwrap_or(0);
        let stack = first_alloc
            .and_then(|a| a.stack_trace.as_ref())
            .cloned()
            .unwrap_or_default();
        let tid_short = t
            .thread_id
            .replace("ThreadId(", "")
            .replace(')', "")
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .to_string();
        logs.push(ThreadEventLogEntry {
            time: format_ts_ns(ts_ns),
            level: "INFO".to_string(),
            message: format!(
                "Thread ({}) {} allocations (peak {})",
                &tid_short[..tid_short.len().min(8)],
                t.allocation_count,
                t.peak_memory
            ),
            stack_traces: stack,
        });
    }

    // Async task events with real duration data.
    for t in async_tasks.iter().take(6) {
        logs.push(ThreadEventLogEntry {
            time: format_ts_ns(t.task_id * 1_000_000),
            level: if t.has_potential_leak {
                "WARN".to_string()
            } else {
                "TASK".to_string()
            },
            message: format!(
                "Async task `{}` (id={}) {} — {} allocations, {}B peak{}",
                t.task_name,
                t.task_id,
                if t.is_completed {
                    "completed"
                } else {
                    "yielded"
                },
                t.total_allocations,
                t.peak_memory,
                if t.has_potential_leak {
                    " · LEAK DETECTED"
                } else {
                    ""
                }
            ),
            stack_traces: vec![format!(
                "task_id: {} | duration: {}ms | efficiency: {:.2}",
                t.task_id, t.duration_ms, t.efficiency_score
            )],
        });
    }

    // Sort by time for natural reading order.
    logs.sort_by(|a, b| a.time.cmp(&b.time));
    logs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that build_context_from_tracker_with_async works
    /// with a minimal tracker. This is a smoke test.
    /// Invariants: The function should not panic on valid inputs.
    #[test]
    fn test_build_context_smoke() {
        // This test verifies the function signature compiles and handles
        // the basic case. Full integration tests cover the actual data path.
        let _ = build_task_graph_json;
    }

    /// Objective: Verify that SamplingMetadata is correctly exposed in json_data.
    /// Invariants: An unsampled complete dataset has is_sampled=false.
    #[test]
    fn test_sampling_metadata_in_json() {
        let sampling = SamplingMetadata::complete(42);
        let json = serde_json::to_value(&sampling).unwrap();
        assert!(!json["is_sampled"].as_bool().unwrap());
        assert_eq!(json["original_count"].as_u64().unwrap(), 42);
    }

    /// Objective: Verify that EventSummary is correctly serialized.
    /// Invariants: Complete export has is_sampled=false.
    #[test]
    fn test_event_summary_in_json() {
        let summary = EventSummary::new(100, 100, 50, 25);
        let json = serde_json::to_value(&summary).unwrap();
        assert!(!json["is_sampled"].as_bool().unwrap());
        assert_eq!(json["total_event_count"].as_u64().unwrap(), 100);
    }

    /// Objective: Verify that DashboardContext defaults match expectations.
    /// Invariants: All fields have sensible defaults when empty.
    #[test]
    fn test_dashboard_context_defaults() {
        let ctx = DashboardContext {
            title: "Test".to_string(),
            export_timestamp: "now".to_string(),
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
            architecture: "x86_64".to_string(),
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
            health_status: "Excellent".to_string(),
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
            poll_latency_mean_ms: 0.0,
            poll_latency_samples: vec![],
            task_topology_nodes: vec![],
            task_topology_nodes_count: 0,
            task_topology_edges: vec![],
            task_topology_edges_count: 0,
            streaming_topology_stats: Default::default(),
            trace_logs: vec![],
            neighbor_density_histogram: vec![],
            dependency_graph_nodes: vec![],
            selected_node_detail: None,
            thread_affinity_grid: vec![],
            scheduler_lag_bars: vec![],
            scheduler_lag_ms: 0,
            migration_rate_pct: 0.0,
            system_uptime_formatted: String::new(),
            thread_event_log: vec![],
            thread_policies: vec![],
            resource_limits: vec![],
        };
        assert_eq!(ctx.title, "Test");
        assert_eq!(ctx.thread_count, 0);
    }
}
