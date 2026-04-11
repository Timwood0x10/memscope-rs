//! Context building logic for dashboard.

use super::helpers::{format_bytes, format_thread_id};
use super::system_info::get_system_info;
use super::types::*;
use crate::analysis::memory_passport_tracker::{
    MemoryPassportTracker, PassportEventType, PassportStatus,
};
use crate::analyzer::Analyzer;
use crate::tracker::Tracker;
use crate::view::MemoryView;
use std::collections::HashMap;
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
    let mut az = Analyzer::from_view(view);

    if all_allocations.is_empty() {
        tracing::warn!("No allocations found in event store. Dashboard may show limited data.");
    }

    let total_memory: usize = all_allocations.iter().map(|a| a.size).sum();

    let max_allocations = std::cmp::max(1, all_allocations.len() / 2);
    let limited_allocations: Vec<_> = all_allocations
        .iter()
        .take(max_allocations)
        .cloned()
        .collect();

    let alloc_info = build_allocation_info(&limited_allocations);
    let relationships = build_relationships(&mut az);
    let unsafe_reports = build_unsafe_reports(&passports, &all_allocations);
    let passport_details = build_passport_details(&passports, &all_allocations);

    let leak_result = passport_tracker.detect_leaks_at_shutdown();
    let leak_count = leak_result.leaked_passports.len();

    let thread_data = aggregate_thread_data(&alloc_info);
    let async_tasks = build_async_tasks(async_tracker);
    let async_summary = build_async_summary(async_tracker);
    let ownership_graph = build_ownership_graph_info(&all_allocations);

    let system_info = get_system_info();

    let health_info = calculate_health_info(
        &unsafe_reports,
        &passport_details,
        leak_count,
        alloc_info.len(),
    );

    let json_data = build_json_data(
        &alloc_info,
        &relationships,
        &unsafe_reports,
        &thread_data,
        &passport_details,
        tracker_analysis.active_allocations,
        tracker_analysis.total_allocations,
        leak_count,
        &async_tasks,
        &async_summary,
        &ownership_graph,
        health_info.health_score,
    )?;

    Ok(DashboardContext {
        title: "MemScope Dashboard".to_string(),
        export_timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        total_memory: format_bytes(total_memory),
        total_allocations: tracker_analysis.total_allocations,
        active_allocations: tracker_analysis.active_allocations,
        peak_memory: format_bytes(tracker_analysis.peak_memory_bytes as usize),
        thread_count: 1,
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
    })
}

/// Rebuild all allocations from event_store (unified data source).
pub fn rebuild_allocations_from_events(
    events: &[crate::event_store::event::MemoryEvent],
) -> Vec<crate::capture::backends::core_types::AllocationInfo> {
    use crate::event_store::event::MemoryEventType;

    let mut active_allocations: HashMap<
        usize,
        crate::capture::backends::core_types::AllocationInfo,
    > = HashMap::new();
    let mut container_allocations: Vec<crate::capture::backends::core_types::AllocationInfo> =
        Vec::new();

    for event in events {
        match event.event_type {
            MemoryEventType::Allocate => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let alloc = crate::capture::backends::core_types::AllocationInfo {
                    ptr: event.ptr,
                    size: event.size,
                    allocated_at_ns: event.timestamp,
                    var_name: event.var_name.clone(),
                    type_name: event.type_name.clone(),
                    thread_id: event.thread_id,
                    stack_trace: stack_trace.map(|s| vec![s]),
                };
                active_allocations.insert(event.ptr, alloc);
            }
            MemoryEventType::Metadata => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let alloc = crate::capture::backends::core_types::AllocationInfo {
                    ptr: 0,
                    size: event.size,
                    allocated_at_ns: event.timestamp,
                    var_name: event.var_name.clone(),
                    type_name: event.type_name.clone(),
                    thread_id: event.thread_id,
                    stack_trace: stack_trace.map(|s| vec![s]),
                };
                container_allocations.push(alloc);
            }
            MemoryEventType::Deallocate => {
                active_allocations.remove(&event.ptr);
            }
            _ => {}
        }
    }

    let mut all_allocations: Vec<_> = active_allocations.into_values().collect();

    let container_allocations_with_virtual_ptrs: Vec<_> = container_allocations
        .into_iter()
        .enumerate()
        .map(|(index, mut alloc)| {
            alloc.ptr = 0x10000000000 + index;
            alloc
        })
        .collect();

    all_allocations.extend(container_allocations_with_virtual_ptrs);
    all_allocations
}

/// Build allocation info from allocations
fn build_allocation_info(
    allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> Vec<AllocationInfo> {
    allocations
        .iter()
        .map(|a| {
            let original_type_name = a.type_name.clone().unwrap_or_else(|| "unknown".to_string());
            let type_name = get_inferred_type_name(&original_type_name, a.size);

            let is_smart_pointer =
                type_name.contains("Arc") || type_name.contains("Rc") || type_name.contains("Box");
            let smart_pointer_type = if type_name.contains("Arc") {
                "Arc".to_string()
            } else if type_name.contains("Rc") {
                "Rc".to_string()
            } else if type_name.contains("Box") {
                "Box".to_string()
            } else {
                String::new()
            };

            AllocationInfo {
                address: format!("0x{:x}", a.ptr),
                type_name: type_name.clone(),
                size: a.size,
                var_name: a.var_name.clone().unwrap_or_else(|| "unknown".to_string()),
                timestamp: format!("{:?}", a.allocated_at_ns),
                thread_id: format!("{}", a.thread_id),
                immutable_borrows: 0,
                mutable_borrows: 0,
                is_clone: false,
                clone_count: 0,
                timestamp_alloc: a.allocated_at_ns,
                timestamp_dealloc: 0,
                lifetime_ms: 0.0,
                is_leaked: true,
                allocation_type: "heap".to_string(),
                is_smart_pointer,
                smart_pointer_type,
                source_file: extract_user_source_file(&a.stack_trace),
                source_line: extract_user_source_line(&a.stack_trace),
            }
        })
        .collect()
}

/// Build relationships from analyzer
fn build_relationships(az: &mut Analyzer) -> Vec<RelationshipInfo> {
    let graph_edges = az.graph().relationships();

    let mut relationships: Vec<RelationshipInfo> = graph_edges
        .iter()
        .map(|edge| {
            let (rel_type, color, strength) = match edge.relation {
                crate::analyzer::Relation::Owns => ("ownership_transfer", "#dc2626", 1.0),
                crate::analyzer::Relation::Contains => ("contains", "#f59e0b", 0.6),
                crate::analyzer::Relation::Slice => ("immutable_borrow", "#3b82f6", 0.8),
                crate::analyzer::Relation::Clone => ("clone", "#10b981", 0.9),
                crate::analyzer::Relation::Shares => ("Arc", "#8b5cf6", 0.7),
                crate::analyzer::Relation::Evolution => ("evolution", "#06b6d4", 0.5),
            };

            let type_name = edge
                .from_type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            RelationshipInfo {
                source_ptr: if edge.is_container_source {
                    format!("container_{}", edge.from_ptr)
                } else {
                    format!("0x{:x}", edge.from_ptr)
                },
                source_var_name: edge.from_var_name.clone().unwrap_or_else(|| {
                    if edge.is_container_source {
                        format!("container_{}", edge.from_ptr)
                    } else {
                        format!("alloc_{}", edge.from_ptr)
                    }
                }),
                target_ptr: if edge.is_container_target {
                    format!("container_{}", edge.to_ptr)
                } else {
                    format!("0x{:x}", edge.to_ptr)
                },
                target_var_name: edge.to_var_name.clone().unwrap_or_else(|| {
                    if edge.is_container_target {
                        format!("container_{}", edge.to_ptr)
                    } else {
                        format!("alloc_{}", edge.to_ptr)
                    }
                }),
                relationship_type: rel_type.to_string(),
                strength,
                type_name,
                color: color.to_string(),
                is_part_of_cycle: false,
                is_container_source: edge.is_container_source,
                is_container_target: edge.is_container_target,
            }
        })
        .collect();

    let cycle_edges: std::collections::HashSet<(String, String)> = {
        let rel_tuples: Vec<(String, String, String)> = relationships
            .iter()
            .map(|r| {
                (
                    r.source_ptr.clone(),
                    r.target_ptr.clone(),
                    r.type_name.clone(),
                )
            })
            .collect();
        let result = crate::analysis::detect_cycles_in_relationships(&rel_tuples);
        result.cycle_edges
    };

    for rel in &mut relationships {
        if cycle_edges.contains(&(rel.source_ptr.clone(), rel.target_ptr.clone())) {
            rel.is_part_of_cycle = true;
            rel.color = "#ef4444".to_string();
        }
    }

    relationships
}

/// Build unsafe reports from passports
fn build_unsafe_reports(
    passports: &HashMap<usize, crate::analysis::memory_passport_tracker::MemoryPassport>,
    all_allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> Vec<UnsafeReport> {
    passports
        .values()
        .filter(|p| !p.lifecycle_events.is_empty())
        .map(|p| build_unsafe_report(p, all_allocations))
        .collect()
}

/// Build single unsafe report
fn build_unsafe_report(
    p: &crate::analysis::memory_passport_tracker::MemoryPassport,
    all_allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> UnsafeReport {
    let lifecycle_events = build_lifecycle_events(&p.lifecycle_events);
    let cross_boundary_events = build_boundary_events(&lifecycle_events);

    let is_leaked = p.status_at_shutdown == PassportStatus::InForeignCustody
        || p.status_at_shutdown == PassportStatus::HandoverToFfi;
    let risk_level = if is_leaked {
        "high".to_string()
    } else if !cross_boundary_events.is_empty() {
        "medium".to_string()
    } else {
        "low".to_string()
    };

    let var_name = if p.var_name != "-" {
        p.var_name.clone()
    } else {
        all_allocations
            .iter()
            .find(|a| a.ptr == p.allocation_ptr)
            .and_then(|a| a.var_name.clone())
            .unwrap_or_else(|| "-".to_string())
    };

    let type_name = if p.type_name != "-" {
        p.type_name.clone()
    } else {
        let from_alloc = all_allocations
            .iter()
            .find(|a| a.ptr == p.allocation_ptr)
            .and_then(|a| a.type_name.clone())
            .unwrap_or_else(|| "-".to_string());

        if from_alloc != "-" {
            from_alloc
        } else {
            infer_type_from_size(p.size_bytes)
        }
    };

    let mut risk_factors = Vec::new();
    if is_leaked {
        risk_factors.push("Memory leaked at shutdown".to_string());
    }
    if !cross_boundary_events.is_empty() {
        risk_factors.push(format!(
            "Crosses FFI boundary {} times",
            cross_boundary_events.len()
        ));
    }
    if cross_boundary_events.len() > 3 {
        risk_factors.push("Frequent boundary crossings".to_string());
    }

    UnsafeReport {
        passport_id: p.passport_id.clone(),
        allocation_ptr: format!("0x{:x}", p.allocation_ptr),
        var_name,
        type_name,
        size_bytes: p.size_bytes,
        created_at: p.created_at,
        updated_at: p.updated_at,
        status: format!("{:?}", p.status_at_shutdown),
        lifecycle_events,
        cross_boundary_events,
        is_leaked,
        risk_level,
        risk_factors,
    }
}

/// Build passport details
fn build_passport_details(
    passports: &HashMap<usize, crate::analysis::memory_passport_tracker::MemoryPassport>,
    all_allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> Vec<PassportDetail> {
    passports
        .values()
        .map(|p| build_passport_detail(p, all_allocations))
        .collect()
}

/// Build single passport detail
fn build_passport_detail(
    p: &crate::analysis::memory_passport_tracker::MemoryPassport,
    all_allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> PassportDetail {
    let lifecycle_events = build_lifecycle_events(&p.lifecycle_events);
    let cross_boundary_events = build_boundary_events(&lifecycle_events);

    let var_name = if p.var_name != "-" {
        p.var_name.clone()
    } else {
        all_allocations
            .iter()
            .find(|a| a.ptr == p.allocation_ptr)
            .and_then(|a| a.var_name.clone())
            .unwrap_or_else(|| "-".to_string())
    };

    let type_name = if p.type_name != "-" {
        p.type_name.clone()
    } else {
        let from_alloc = all_allocations
            .iter()
            .find(|a| a.ptr == p.allocation_ptr)
            .and_then(|a| a.type_name.clone())
            .unwrap_or_else(|| "-".to_string());

        if from_alloc != "-" {
            from_alloc
        } else {
            infer_type_from_size(p.size_bytes)
        }
    };

    let is_leaked = p.status_at_shutdown == PassportStatus::InForeignCustody
        || p.status_at_shutdown == PassportStatus::HandoverToFfi;
    let risk_level = if is_leaked {
        "high".to_string()
    } else if !cross_boundary_events.is_empty() {
        "medium".to_string()
    } else {
        "low".to_string()
    };

    PassportDetail {
        passport_id: p.passport_id.clone(),
        allocation_ptr: format!("0x{:x}", p.allocation_ptr),
        var_name,
        type_name,
        size_bytes: p.size_bytes,
        status: format!("{:?}", p.status_at_shutdown),
        created_at: p.created_at,
        updated_at: p.updated_at,
        is_leaked,
        ffi_tracked: !cross_boundary_events.is_empty(),
        lifecycle_events,
        cross_boundary_events,
        risk_level,
        risk_confidence: 0.85,
    }
}

/// Build lifecycle events
fn build_lifecycle_events(
    events: &[crate::analysis::memory_passport_tracker::PassportEvent],
) -> Vec<LifecycleEventInfo> {
    events
        .iter()
        .map(|event| {
            let (icon, color, context) = match &event.event_type {
                PassportEventType::AllocatedInRust => (
                    "🟢".to_string(),
                    "#10b981".to_string(),
                    "Rust Allocation".to_string(),
                ),
                PassportEventType::HandoverToFfi => (
                    "⬇️".to_string(),
                    "#f59e0b".to_string(),
                    "Handover to FFI".to_string(),
                ),
                PassportEventType::FreedByForeign => (
                    "🔵".to_string(),
                    "#3b82f6".to_string(),
                    "Freed by Foreign".to_string(),
                ),
                PassportEventType::ReclaimedByRust => (
                    "⬆️".to_string(),
                    "#10b981".to_string(),
                    "Reclaimed by Rust".to_string(),
                ),
                PassportEventType::BoundaryAccess => (
                    "🔄".to_string(),
                    "#8b5cf6".to_string(),
                    "Boundary Access".to_string(),
                ),
                PassportEventType::OwnershipTransfer => (
                    "↔️".to_string(),
                    "#dc2626".to_string(),
                    "Ownership Transfer".to_string(),
                ),
                PassportEventType::ValidationCheck => (
                    "✅".to_string(),
                    "#10b981".to_string(),
                    "Validation Check".to_string(),
                ),
                PassportEventType::CorruptionDetected => (
                    "🚨".to_string(),
                    "#dc2626".to_string(),
                    "Corruption Detected".to_string(),
                ),
            };

            LifecycleEventInfo {
                event_type: format!("{:?}", event.event_type),
                timestamp: event.timestamp,
                context,
                icon,
                color,
            }
        })
        .collect()
}

/// Build boundary events
fn build_boundary_events(lifecycle_events: &[LifecycleEventInfo]) -> Vec<BoundaryEventInfo> {
    lifecycle_events
        .iter()
        .filter(|e| e.event_type.contains("Handover") || e.event_type.contains("Reclaimed"))
        .map(|e| {
            let (event_type, from, to, icon, color) = if e.event_type.contains("HandoverToFfi") {
                (
                    "RustToFfi".to_string(),
                    "Rust".to_string(),
                    "FFI".to_string(),
                    "⬇️".to_string(),
                    "#f59e0b".to_string(),
                )
            } else if e.event_type.contains("ReclaimedByRust") {
                (
                    "FfiToRust".to_string(),
                    "FFI".to_string(),
                    "Rust".to_string(),
                    "⬆️".to_string(),
                    "#10b981".to_string(),
                )
            } else {
                (
                    e.event_type.clone(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "❓".to_string(),
                    "#6b7280".to_string(),
                )
            };

            BoundaryEventInfo {
                event_type,
                from_context: from,
                to_context: to,
                timestamp: e.timestamp,
                icon,
                color,
            }
        })
        .collect()
}

/// Aggregate thread data
fn aggregate_thread_data(allocations: &[AllocationInfo]) -> Vec<ThreadInfo> {
    let mut thread_map: HashMap<String, ThreadAggregator> = HashMap::new();

    for alloc in allocations {
        let entry = thread_map.entry(alloc.thread_id.clone()).or_default();
        entry.allocation_count += 1;
        entry.current_memory += alloc.size;
        entry.total_allocated += alloc.size;
        if alloc.size > entry.peak_memory {
            entry.peak_memory = alloc.size;
        }
    }

    thread_map
        .into_iter()
        .map(|(raw_tid, agg)| {
            let summary = format!(
                "{} allocs, {}",
                agg.allocation_count,
                format_bytes(agg.current_memory)
            );
            let thread_id = format_thread_id(&raw_tid);
            ThreadInfo {
                thread_id,
                thread_summary: summary,
                allocation_count: agg.allocation_count,
                current_memory: format_bytes(agg.current_memory),
                peak_memory: format_bytes(agg.peak_memory),
                total_allocated: format_bytes(agg.total_allocated),
                current_memory_bytes: agg.current_memory,
                peak_memory_bytes: agg.peak_memory,
                total_allocated_bytes: agg.total_allocated,
            }
        })
        .collect()
}

/// Build async tasks
fn build_async_tasks(
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> Vec<AsyncTaskInfo> {
    if let Some(tracker) = async_tracker {
        let profiles = tracker.get_all_profiles();
        profiles
            .into_iter()
            .map(|p| {
                let is_completed = p.is_completed();
                let has_potential_leak = p.has_potential_leak();
                let task_type_str = format!("{:?}", p.task_type);
                AsyncTaskInfo {
                    task_id: p.task_id,
                    task_name: p.task_name,
                    task_type: task_type_str,
                    total_bytes: p.total_bytes,
                    current_memory: p.current_memory,
                    peak_memory: p.peak_memory,
                    total_allocations: p.total_allocations,
                    duration_ms: p.duration_ns as f64 / 1_000_000.0,
                    efficiency_score: p.efficiency_score,
                    is_completed,
                    has_potential_leak,
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// Build async summary
fn build_async_summary(
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> AsyncSummary {
    if let Some(tracker) = async_tracker {
        let stats = tracker.get_stats();
        AsyncSummary {
            total_tasks: stats.total_tasks,
            active_tasks: stats.active_tasks,
            total_allocations: stats.total_allocations,
            total_memory_bytes: stats.total_memory,
            peak_memory_bytes: stats.peak_memory,
        }
    } else {
        AsyncSummary {
            total_tasks: 0,
            active_tasks: 0,
            total_allocations: 0,
            total_memory_bytes: 0,
            peak_memory_bytes: 0,
        }
    }
}

/// Build ownership graph info
fn build_ownership_graph_info(
    allocations: &[crate::capture::backends::core_types::AllocationInfo],
) -> OwnershipGraphInfo {
    use crate::analysis::node_id::NodeId;
    use crate::analysis::ownership_graph::{DiagnosticIssue, OwnershipGraph, OwnershipOp};

    let passports: Vec<(
        NodeId,
        String,
        usize,
        Vec<crate::analysis::ownership_graph::OwnershipEvent>,
    )> = allocations
        .iter()
        .map(|alloc| {
            let id = NodeId::from_ptr(alloc.ptr);
            let type_name = alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let size = alloc.size;

            let events = vec![crate::analysis::ownership_graph::OwnershipEvent::new(
                alloc.allocated_at_ns,
                OwnershipOp::Create,
                id,
                None,
            )];

            (id, type_name, size, events)
        })
        .collect();

    let graph = OwnershipGraph::build(&passports);
    let diagnostics = graph.diagnostics(50);

    let issues = diagnostics
        .issues
        .iter()
        .map(|issue| match issue {
            DiagnosticIssue::RcCycle { cycle_type, .. } => OwnershipIssue {
                issue_type: "RcCycle".to_string(),
                severity: "error".to_string(),
                description: format!("{:?} retain cycle detected", cycle_type),
            },
            DiagnosticIssue::ArcCloneStorm {
                clone_count,
                threshold,
            } => OwnershipIssue {
                issue_type: "ArcCloneStorm".to_string(),
                severity: "warning".to_string(),
                description: format!(
                    "Arc clone storm: {} clones (threshold: {})",
                    clone_count, threshold
                ),
            },
        })
        .collect();

    let root_cause = graph.find_root_cause().map(|rc| RootCauseInfo {
        cause: format!("{:?}", rc.root_cause),
        description: rc.description,
        impact: rc.impact,
    });

    OwnershipGraphInfo {
        total_nodes: graph.nodes.len(),
        total_edges: graph.edges.len(),
        total_cycles: graph.cycles.len(),
        rc_clone_count: diagnostics.rc_clone_count,
        arc_clone_count: diagnostics.arc_clone_count,
        has_issues: diagnostics.has_issues(),
        issues,
        root_cause,
    }
}

/// Build JSON data for template injection
#[allow(clippy::too_many_arguments)]
fn build_json_data(
    alloc_info: &[AllocationInfo],
    relationships: &[RelationshipInfo],
    unsafe_reports: &[UnsafeReport],
    thread_data: &[ThreadInfo],
    passport_details: &[PassportDetail],
    active_allocations: usize,
    total_allocations: usize,
    leak_count: usize,
    async_tasks: &[AsyncTaskInfo],
    async_summary: &AsyncSummary,
    ownership_graph: &OwnershipGraphInfo,
    health_score: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    #[derive(serde::Serialize)]
    struct DashboardData<'a> {
        allocations: &'a [AllocationInfo],
        relationships: &'a [RelationshipInfo],
        unsafe_reports: &'a [UnsafeReport],
        threads: &'a [ThreadInfo],
        passport_details: &'a [PassportDetail],
        active_allocations: usize,
        total_allocations: usize,
        leak_count: usize,
        async_tasks: &'a [AsyncTaskInfo],
        async_summary: &'a AsyncSummary,
        ownership_graph: &'a OwnershipGraphInfo,
        health_score: u32,
    }

    let data = DashboardData {
        allocations: alloc_info,
        relationships,
        unsafe_reports,
        threads: thread_data,
        passport_details,
        active_allocations,
        total_allocations,
        leak_count,
        async_tasks,
        async_summary,
        ownership_graph,
        health_score,
    };

    Ok(serde_json::to_string(&data)?)
}

/// Calculate health info
fn calculate_health_info(
    unsafe_reports: &[UnsafeReport],
    passport_details: &[PassportDetail],
    leak_count: usize,
    total_allocs: usize,
) -> HealthInfo {
    let high_risk_count = unsafe_reports
        .iter()
        .filter(|r| r.risk_level == "high")
        .count();
    let clean_passport_count = passport_details.iter().filter(|p| !p.is_leaked).count();
    let active_passport_count = passport_details
        .iter()
        .filter(|p| p.status == "active")
        .count();
    let leaked_passport_count = passport_details.iter().filter(|p| p.is_leaked).count();
    let ffi_tracked_count = passport_details.iter().filter(|p| p.ffi_tracked).count();

    let total_allocs = total_allocs.max(1);
    let unsafe_count = unsafe_reports.len();
    let leak_score = (100.0 - (leak_count as f64 / total_allocs as f64) * 100.0).max(0.0);
    let unsafe_score = (100.0 - (unsafe_count as f64 / total_allocs as f64) * 50.0).max(0.0);
    let risk_score = (100.0 - high_risk_count as f64 * HIGH_RISK_PENALTY).max(0.0);
    let health_score = ((leak_score + unsafe_score + risk_score) / 3.0).round() as u32;

    let health_status = if health_score >= 80 {
        "✅ Excellent"
    } else if health_score >= 60 {
        "⚠️ Good"
    } else {
        "🚨 Needs Attention"
    };

    let safe_ops_count = total_allocs.saturating_sub(unsafe_count);
    let safe_code_percent = ((safe_ops_count as f64 / total_allocs as f64) * 100.0).round() as u32;

    HealthInfo {
        health_score,
        health_status: health_status.to_string(),
        safe_ops_count,
        high_risk_count,
        clean_passport_count,
        active_passport_count,
        leaked_passport_count,
        ffi_tracked_count,
        safe_code_percent,
    }
}

/// Extract user source file from stack trace
fn extract_user_source_file(stack_trace: &Option<Vec<String>>) -> Option<String> {
    if let Some(ref frames) = stack_trace {
        for frame in frames {
            let frame_lower = frame.to_lowercase();
            if !frame_lower.contains("/rustc/")
                && !frame_lower.contains("/library/")
                && !frame_lower.contains("memscope")
                && !frame_lower.contains(".cargo/registry")
                && !frame_lower.contains("/src/core/")
                && !frame_lower.contains("/src/capture/")
                && !frame_lower.contains("/src/unified/")
                && !frame_lower.contains("/src/tracker")
            {
                if let Some(file_part) = frame.split(':').next() {
                    let file_name = file_part.split('/').next_back().unwrap_or(file_part);
                    if !file_name.starts_with('<') && file_name.contains(".rs") {
                        return Some(file_part.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract user source line from stack trace
fn extract_user_source_line(stack_trace: &Option<Vec<String>>) -> Option<u32> {
    if let Some(ref frames) = stack_trace {
        for frame in frames {
            let frame_lower = frame.to_lowercase();
            if !frame_lower.contains("/rustc/")
                && !frame_lower.contains("/library/")
                && !frame_lower.contains("memscope")
                && !frame_lower.contains(".cargo/registry")
                && !frame_lower.contains("/src/core/")
                && !frame_lower.contains("/src/capture/")
                && !frame_lower.contains("/src/unified/")
                && !frame_lower.contains("/src/tracker")
            {
                if let Some(line_part) = frame.rsplit(':').next() {
                    if let Ok(line) = line_part.parse::<u32>() {
                        return Some(line);
                    }
                }
            }
        }
    }
    None
}

/// Infer type from size
fn infer_type_from_size(size: usize) -> String {
    match size {
        8 => "*mut c_void (30%)".to_string(),
        16 => "&[T] (25%)".to_string(),
        24 => "Vec<_>/String (15%)".to_string(),
        32 | 48 | 64 => "CStruct (10%)".to_string(),
        n if n.is_power_of_two() && n >= 64 => {
            format!("Vec<_>/[u8] ({}%)", 10 + n.trailing_zeros() as u8)
        }
        n if (32..=256).contains(&n) => "[u8] (10%)".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Get inferred type name
fn get_inferred_type_name(type_name: &str, size: usize) -> String {
    if type_name != "unknown" && type_name != "-" && !type_name.is_empty() {
        return type_name.to_string();
    }
    infer_type_from_size(size)
}

/// Health info structure
struct HealthInfo {
    health_score: u32,
    health_status: String,
    safe_ops_count: usize,
    high_risk_count: usize,
    clean_passport_count: usize,
    active_passport_count: usize,
    leaked_passport_count: usize,
    ffi_tracked_count: usize,
    safe_code_percent: u32,
}

/// Helper to convert string to C string (cached for sysctl calls)
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn c(s: &str) -> *const libc::c_char {
    use std::ffi::CString;
    thread_local! {
        static KERN_OSRELEASE: CString = CString::new("kern.osrelease").unwrap();
        static HW_MACHINE: CString = CString::new("hw.machine").unwrap();
        static HW_PAGESIZE: CString = CString::new("hw.pagesize").unwrap();
    }
    match s {
        "kern.osrelease" => KERN_OSRELEASE.with(|c| c.as_ptr()),
        "hw.machine" => HW_MACHINE.with(|c| c.as_ptr()),
        "hw.pagesize" => HW_PAGESIZE.with(|c| c.as_ptr()),
        _ => std::ptr::null(),
    }
}
