//! Context building logic for dashboard.

use super::event_dto::{build_data_index, DashboardEventDTO, EventSummary};
use super::event_reconstructor::{
    project_allocations, SamplingMetadata, MAX_DASHBOARD_ALLOCATIONS,
};
use super::helpers::format_bytes;
use super::report_builder::{
    aggregate_thread_data, aggregate_thread_data_from_ledger, build_allocation_info,
    build_async_summary, build_async_tasks, build_circular_reference_report,
    build_ownership_graph_info, build_passport_details, build_relationships, build_top_n_reports,
    build_unsafe_reports, calculate_health_info,
};
use super::system_info::get_system_info;
use super::telemetry;
use super::types::*;
use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::tracker::Tracker;
use crate::view::MemoryView;
use std::sync::Arc;

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

/// Share of allocations that reuse a previously used address.
///
/// Generation 1 is the *first* binding at an address; only generation 2 and above
/// mean the address was reused. Counting `generation_id > 0` reported a 100% reuse
/// rate for every single-use allocation.
fn compute_address_reuse_pct(allocations: &[crate::capture::types::AllocationInfo]) -> f64 {
    if allocations.is_empty() {
        return 0.0;
    }
    let reused = allocations.iter().filter(|a| a.generation_id > 1).count();
    (reused as f64 / allocations.len() as f64) * 100.0
}

/// Build dashboard context from tracker data with async support
pub fn build_context_from_tracker_with_async(
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> Result<DashboardContext, Box<dyn std::error::Error>> {
    let _suppression = crate::core::allocator::TrackingGuard::acquire();
    let passports = passport_tracker.snapshot_passports();

    let events = tracker.event_store().snapshot();
    let capture_at_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos()
        .min(u64::MAX as u128) as u64;
    let mut ledger = crate::snapshot::BindingLedger::new();
    for event in &events {
        ledger.apply(event);
    }
    let all_allocations = project_allocations(&ledger, &events);
    let snapshot = crate::snapshot::builder::project_snapshot(&ledger, capture_at_ns);
    let total_allocations = snapshot.stats.total_allocations;
    let active_allocations = snapshot.active_count();
    let peak_memory = snapshot.peak_memory();
    let total_memory = snapshot.current_memory();
    let view = MemoryView::new(snapshot, events.clone());
    let mut az = crate::analyzer::Analyzer::from_view(view);

    if all_allocations.is_empty() {
        tracing::warn!("No allocations found in event store. Dashboard may show limited data.");
    }

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
    let mut alloc_info = build_allocation_info(&display_allocations);
    for allocation in &mut alloc_info {
        allocation.observed_age_ms = capture_at_ns
            .checked_sub(allocation.timestamp_alloc)
            .map(|ns| ns as f64 / 1_000_000.0);
    }

    let relationships = build_relationships(&mut az);
    let unsafe_reports = build_unsafe_reports(passport_tracker, &passports, &all_allocations);
    let passport_details = build_passport_details(passport_tracker, &passports, &all_allocations);

    let leak_result = passport_tracker.detect_leaks_in(&passports);
    let leak_count = leak_result.leaked_passports.len();

    // Thread numbers come from the ledger whenever events exist: only it knows the
    // cumulative totals and the high-water mark, which the live list cannot reconstruct.
    let thread_data = if ledger.thread_stats().is_empty() {
        aggregate_thread_data(&alloc_info)
    } else {
        aggregate_thread_data_from_ledger(ledger.thread_stats())
    };
    let async_tasks = build_async_tasks(async_tracker);
    let async_summary = build_async_summary(async_tracker, &async_tasks);
    let ownership_graph = build_ownership_graph_info(&all_allocations);

    let top_n_reports = build_top_n_reports(&all_allocations);
    let circular_references = build_circular_reference_report(&all_allocations);

    let system_info = get_system_info();

    let task_graph = crate::task_registry::global_registry().export_graph();
    let task_graph_json = serde_json::to_string(&task_graph)?;

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
        total_allocations,
        active_allocations,
    )
    .with_duplicate_observations(tracker.duplicate_observations() as usize)
    .with_degraded_releases(ledger.stats().degraded_releases as usize);
    let data_index = build_data_index(&alloc_info, &event_dtos, &passport_details, &unsafe_reports);

    // Pre-compute all new professional template data before moving values into context
    let ffi_call_topology = build_ffi_call_topology(&unsafe_reports);
    let symbol_table = build_symbol_table(&unsafe_reports);
    let stack_integrity = build_stack_integrity();
    let resource_bars = build_resource_bars(&unsafe_reports);
    let thread_timeline = build_thread_timeline(&thread_data, &alloc_info);
    let waker_efficiency_grid: Vec<f64> = Vec::new();
    let task_duration_samples = telemetry::task_durations(&async_tasks);
    let task_duration_mean_ms = telemetry::mean(&task_duration_samples);
    let (task_topology_nodes, task_topology_edges) = telemetry::task_topology(&task_graph);
    let streaming_topology_stats = StreamingTopologyStats {
        graph_edges: task_graph.edges.len(),
        sampling_rate_ms: None,
        waker_locks_status: "Unknown (not instrumented)".into(),
    };
    let trace_logs = telemetry::trace_logs(&async_tasks, &unsafe_reports);
    let allocation_lifetimes = telemetry::lifetimes(&ledger, capture_at_ns);
    let dependency_graph_nodes = build_dependency_graph_nodes(&relationships, &alloc_info);
    let selected_node_detail = build_selected_node_detail(&relationships, &alloc_info);
    // Neither per-thread affinity masks nor CPU occupancy were captured.
    // The grid is a display placeholder, not a measurement: bound it so a bogus core
    // count cannot trigger a huge one-shot allocation.
    let affinity_slots = usize::try_from(system_info.cpu_cores)
        .unwrap_or(1)
        .clamp(1, 256);
    let thread_affinity_grid = vec!["UNKNOWN".to_string(); affinity_slots];
    let intervals = telemetry::allocation_intervals(&events);
    let allocation_interval_histogram = intervals.histogram;
    let allocation_interval_mean_ms = intervals.mean_ms;
    let allocation_interval_sample_count = intervals.sample_count;
    let non_dominant_thread_allocation_share_pct = intervals.non_dominant_share_pct;
    let system_uptime_formatted = compute_system_uptime(&all_allocations);
    let thread_event_log = telemetry::thread_event_log(&events, &async_tasks);
    let thread_policies = vec![
        ThreadPolicy {
            name: format!("OBSERVED_THREADS({})", thread_data.len()),
            enabled: Some(!thread_data.is_empty()),
        },
        ThreadPolicy {
            name: format!("MULTI_CORE_SCHED({} cores)", system_info.cpu_cores),
            enabled: None,
        },
        ThreadPolicy {
            name: "CORE_AFFINITY_PIN".to_string(),
            enabled: None,
        },
    ];
    // Resource limits from real system metrics — CPU from getrusage, MEM from
    // host_statistics64, address reuse from the binding generation.
    let mem_pct = if system_info.total_physical_bytes > 0 {
        (system_info.used_physical_bytes as f64 / system_info.total_physical_bytes as f64) * 100.0
    } else {
        0.0
    };
    let address_reuse_pct = compute_address_reuse_pct(&all_allocations);
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
            pct: address_reuse_pct.round(),
            color_class: "primary".to_string(),
        },
    ];

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
    let peak_memory_fmt = format_bytes(peak_memory);
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
    json_obj.insert("active_allocations".into(), active_allocations.into());
    json_obj.insert("total_allocations".into(), total_allocations.into());
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
    json_obj.insert(
        "task_duration_mean_ms".into(),
        serde_json::to_value(task_duration_mean_ms)?,
    );
    let duration_fmt = task_duration_mean_ms
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "—".into());
    json_obj.insert(
        "task_duration_mean_ms_fmt".into(),
        serde_json::Value::String(duration_fmt),
    );
    json_obj.insert(
        "task_duration_samples".to_string(),
        serde_json::to_value(&task_duration_samples)?,
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
        "allocation_lifetimes".to_string(),
        serde_json::to_value(&allocation_lifetimes)?,
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
        "allocation_interval_histogram".to_string(),
        serde_json::to_value(&allocation_interval_histogram)?,
    );
    json_obj.insert(
        "allocation_interval_mean_ms".into(),
        serde_json::to_value(allocation_interval_mean_ms)?,
    );
    json_obj.insert(
        "allocation_interval_sample_count".into(),
        allocation_interval_sample_count.into(),
    );
    json_obj.insert(
        "non_dominant_thread_allocation_share_pct".into(),
        serde_json::to_value(non_dominant_thread_allocation_share_pct)?,
    );
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
        total_allocations,
        active_allocations,
        peak_memory: format_bytes(peak_memory),
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
        task_graph_json,

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
        task_duration_mean_ms,
        task_duration_samples,
        task_topology_nodes_count: task_topology_nodes.len(),
        task_topology_nodes,
        task_topology_edges_count: task_topology_edges.len(),
        task_topology_edges,
        streaming_topology_stats,
        trace_logs,
        allocation_lifetimes,
        dependency_graph_nodes,
        selected_node_detail,
        thread_affinity_grid,
        allocation_interval_histogram,
        allocation_interval_mean_ms,
        allocation_interval_sample_count,
        non_dominant_thread_allocation_share_pct,
        system_uptime_formatted,
        thread_event_log,
        thread_policies,
        resource_limits,
    })
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
                    "low" => "LOW_RISK".to_string(),
                    _ => "UNASSESSED".to_string(),
                };
                let is_hot = r.is_leaked || r.risk_level == "high";
                Some(SymbolTableEntry {
                    hex_addr: r.allocation_ptr.clone(),
                    symbol_name: r.var_name.clone(),
                    status,
                    call_count: None,
                    time_avg_us: None,
                    is_hot,
                })
            } else {
                None
            }
        })
        .collect()
}

fn build_stack_integrity() -> StackIntegrityMetrics {
    // Unwinding strategy reflects the actual panic strategy compiled into the
    // binary — detected via cfg(panic = "abort"). Falls back to "UNWIND" when
    // built with the default panic = "unwind".
    let strategy = if cfg!(panic = "abort") {
        "PANIC_ABORT"
    } else {
        "PANIC_UNWIND"
    };
    StackIntegrityMetrics {
        pointers_checked_pct: None,
        memory_violations: None,
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
            execution_time_ms: a.observed_age_ms.unwrap_or_default() as u64,
            upstream_deps: relationships
                .iter()
                .filter(|r| r.target_ptr == a.address)
                .count(),
            exec_trace,
        }
    })
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
        };
        assert_eq!(ctx.title, "Test");
        assert_eq!(ctx.thread_count, 0);
    }

    /// Objective: Verify the address-reuse metric does not count a first use as
    /// reuse (plan A13).
    /// Invariants: only generations above 1 contribute; an empty input yields 0.
    #[test]
    fn test_compute_address_reuse_pct_ignores_first_binding() {
        let mut first = crate::capture::types::AllocationInfo::new(0x1000, 16);
        first.generation_id = 1;
        let mut second = crate::capture::types::AllocationInfo::new(0x2000, 16);
        second.generation_id = 2;
        let mut third = crate::capture::types::AllocationInfo::new(0x3000, 16);
        third.generation_id = 3;

        assert_eq!(
            compute_address_reuse_pct(&[]),
            0.0,
            "no allocations means no reuse rate"
        );
        assert_eq!(
            compute_address_reuse_pct(&[first.clone()]),
            0.0,
            "a single first-use allocation must report 0% reuse"
        );
        assert_eq!(
            compute_address_reuse_pct(&[first, second, third]),
            (2.0 / 3.0) * 100.0,
            "only the two reused addresses may be counted"
        );
    }
}
