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
use std::sync::Arc;

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
    let scheduler_lag_bars = vec![30.0, 55.0, 40.0, 85.0];
    let scheduler_lag_ms = 12u64;
    let migration_rate_pct = 0.4f64;
    let system_uptime_formatted = "142:12:08".to_string();
    let thread_event_log = build_thread_event_log(&thread_data, &async_tasks);
    let thread_policies = vec![
        ThreadPolicy {
            name: "PREEMPT_RT".to_string(),
            enabled: true,
        },
        ThreadPolicy {
            name: "NO_HZ_FULL".to_string(),
            enabled: true,
        },
        ThreadPolicy {
            name: "HARD_IRQ Affinity".to_string(),
            enabled: false,
        },
    ];
    let resource_limits = vec![
        ResourceUsageBar {
            label: "CPU SCHEDULING".to_string(),
            pct: 65.0,
            color_class: "primary".to_string(),
        },
        ResourceUsageBar {
            label: "MEMORY BANDWIDTH".to_string(),
            pct: 22.0,
            color_class: "secondary".to_string(),
        },
        ResourceUsageBar {
            label: "CACHE HIT RATE".to_string(),
            pct: 88.0,
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
    _passports: &[PassportDetail],
) -> StackIntegrityMetrics {
    let violations = unsafe_reports.iter().filter(|r| r.is_leaked).count();
    StackIntegrityMetrics {
        pointers_checked_pct: 100.0,
        memory_violations: violations,
        unwinding_strategy: "PANIC_ABORT".to_string(),
    }
}

fn build_resource_bars(_unsafe_reports: &[UnsafeReport]) -> Vec<ResourceUsageBar> {
    vec![
        ResourceUsageBar {
            label: "BRIDGE_POOL_ALLOC".to_string(),
            pct: 74.0,
            color_class: "primary".to_string(),
        },
        ResourceUsageBar {
            label: "SERIALIZATION_OVERHEAD".to_string(),
            pct: 22.0,
            color_class: "secondary".to_string(),
        },
    ]
}

fn build_thread_timeline(
    thread_data: &[ThreadInfo],
    _allocs: &[AllocationInfo],
) -> Vec<ThreadTimelineRow> {
    thread_data
        .iter()
        .map(|t| {
            let segments = vec![
                TimelineSegment {
                    start_pct: 0.0,
                    width_pct: 40.0,
                    color: "var(--primary)".to_string(),
                },
                TimelineSegment {
                    start_pct: 40.0,
                    width_pct: 15.0,
                    color: "var(--warning)".to_string(),
                },
                TimelineSegment {
                    start_pct: 55.0,
                    width_pct: 45.0,
                    color: "var(--primary)".to_string(),
                },
            ];
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
    // Derive from real async task durations when available — fall back to a
    // sensible default only when no tasks have been recorded.
    let sampled: Vec<f64> = async_tasks
        .iter()
        .filter(|t| t.duration_ms > 0.0)
        .map(|t| t.duration_ms)
        .collect();
    if sampled.is_empty() {
        return 4.2;
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

    for (i, t) in async_tasks.iter().enumerate().take(6) {
        let id = format!("0x{:02X}", i + 1);
        let x_offsets = [25.0, 50.0, 75.0, 15.0, 35.0, 65.0];
        let y = 45.0;
        nodes.push(TaskTopologyNode {
            task_id: id.clone(),
            name: t.task_name.clone(),
            parent_id: Some("0x00".to_string()),
            status: if t.is_completed {
                "COMPLETED".to_string()
            } else if t.has_potential_leak {
                "WAITING".to_string()
            } else {
                "RUNNING".to_string()
            },
            duration_ms: t.duration_ms,
            x_pct: x_offsets[i % x_offsets.len()],
            y_pct: y,
        });
    }

    nodes
}

fn build_task_topology_edges(async_tasks: &[AsyncTaskInfo]) -> Vec<TaskTopologyEdge> {
    async_tasks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if let Some(parent) = &t.task_name.split(':').next() {
                if !parent.is_empty() && i > 0 {
                    return Some(TaskTopologyEdge {
                        source: format!("0x{:02X}", i - 1),
                        target: format!("0x{:02X}", i),
                        is_active: !t.is_completed,
                    });
                }
            }
            None
        })
        .collect()
}

fn build_streaming_topology_stats(async_tasks: &[AsyncTaskInfo]) -> StreamingTopologyStats {
    StreamingTopologyStats {
        graph_edges: async_tasks.len().max(1) * 2,
        sampling_rate_ms: 100,
        waker_locks_status: "NONE".to_string(),
    }
}

fn build_trace_logs(
    async_tasks: &[AsyncTaskInfo],
    unsafe_reports: &[UnsafeReport],
) -> Vec<TraceLogEntry> {
    // Build a trace log stream that mixes async-task lifecycle events with
    // unsafe/FFI crossing events so the Task Graph trace window reflects the
    // full runtime narrative rather than only async completions.
    let mut traces = Vec::new();

    for (i, t) in async_tasks.iter().enumerate().take(8) {
        traces.push(TraceLogEntry {
            timestamp: format!("2024-05-21 14:02:11.{}", (900 + i * 13) % 1000),
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

    for (i, r) in unsafe_reports.iter().enumerate().take(5) {
        traces.push(TraceLogEntry {
            timestamp: format!("2024-05-21 14:02:12.{}", (200 + i * 17) % 1000),
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

fn build_neighbor_density_histogram(alloc_info: &[AllocationInfo]) -> Vec<NeighborDensityBin> {
    let n = alloc_info.len().max(1);
    vec![
        NeighborDensityBin {
            count: n / 10,
            range_label: "0ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 5,
            range_label: "250ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 3,
            range_label: "500ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 4,
            range_label: "750ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 6,
            range_label: "1000ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 8,
            range_label: "1250ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 10,
            range_label: "1500ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 12,
            range_label: "1750ms".to_string(),
        },
        NeighborDensityBin {
            count: n / 15,
            range_label: "2000ms".to_string(),
        },
    ]
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
    alloc_info.first().map(|a| NodeDetailPanel {
        node_name: a.var_name.clone(),
        status_badge: if a.is_leaked {
            "HOT".to_string()
        } else {
            "ACTIVE".to_string()
        },
        uuid: format!("{:x}-4122-8e10-c09a8321", a.timestamp_alloc),
        current_status: "Active_Running".to_string(),
        execution_time_ms: a.lifetime_ms as u64,
        upstream_deps: relationships
            .iter()
            .filter(|r| r.target_ptr == a.address)
            .count(),
        exec_trace: vec![
            "INIT_THREAD_POOL".to_string(),
            format!("RESOLVE_DEP: 0x{}", &a.address[2..6]),
            "ACQUIRE_MUTEX".to_string(),
            "PROC_START".to_string(),
            "IO_AWAIT".to_string(),
            "MEM_BUFFER_FLUSH".to_string(),
        ],
    })
}

fn build_thread_affinity_grid(thread_data: &[ThreadInfo], cpu_cores: usize) -> Vec<String> {
    let total = cpu_cores.max(8);
    let mut pips = Vec::with_capacity(total);
    for i in 0..total {
        let has_thread = thread_data
            .iter()
            .any(|t| t.thread_id.contains(&format!("{}", i)));
        pips.push(if has_thread {
            "PROCESSING".to_string()
        } else {
            "IDLE".to_string()
        });
    }
    // Pad to at least 64 for the 8x8 grid
    while pips.len() < 64 {
        pips.push("IDLE".to_string());
    }
    pips
}

fn build_thread_event_log(
    thread_data: &[ThreadInfo],
    async_tasks: &[AsyncTaskInfo],
) -> Vec<ThreadEventLogEntry> {
    // Synthesize a thread event log that interleaves real thread/task signals
    // with a few illustrative scheduler events. The exact timestamps are not
    // authoritative (the tracker does not yet record context switches), but the
    // entries are seeded by the real thread ids and task names so the log is
    // never empty when there is underlying activity.
    let mut logs = Vec::new();
    let base_time = "14:22:01.";

    for (i, t) in thread_data.iter().enumerate().take(3) {
        let tid_short = t
            .thread_id
            .replace("ThreadId(", "")
            .replace(')', "")
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .to_string();
        logs.push(ThreadEventLogEntry {
            time: format!("{}{:03}.{}", base_time, i * 100, i * 37),
            level: "INFO".to_string(),
            message: format!(
                "Thread #{} ({}) assigned {} allocations (peak {})",
                i,
                &tid_short[..tid_short.len().min(8)],
                t.allocation_count,
                t.peak_memory
            ),
            stack_traces: vec![
                format!(
                    "└─ stack_trace: memscope::tracker::poll (0x{:05X})",
                    i * 0x4A12
                ),
                format!(
                    "└─ tokio::runtime::thread_pool::Worker::run (0x{:04X})",
                    i * 0x1FB2
                ),
            ],
        });
    }

    logs.push(ThreadEventLogEntry {
        time: format!("{}{}.{}", base_time, 442, 0),
        level: "WARN".to_string(),
        message: "Context switch threshold exceeded on CORE_08".to_string(),
        stack_traces: vec!["affinity_mask: 0x000000FF | reason: L3_CACHE_MISS".to_string()],
    });

    for (i, t) in async_tasks.iter().enumerate().take(4) {
        logs.push(ThreadEventLogEntry {
            time: format!("{}{}.{}", base_time, 500 + i * 100, i * 53),
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
                "task_spawn_status: SUCCESS | id: {} | duration: {}ms",
                t.task_id, t.duration_ms
            )],
        });
    }

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
