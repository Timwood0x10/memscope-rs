//! Context building logic for dashboard.

use super::helpers::{format_bytes, format_thread_id};
use super::system_info::get_system_info;
use super::types::*;
use crate::analysis::memory_passport_tracker::{
    MemoryPassportTracker, PassportEventType, PassportStatus,
};
use crate::analysis::top_n::TopNAnalyzer;
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

    // Build Top N reports
    let top_n_reports = build_top_n_reports(&all_allocations);

    // Build circular reference report
    let circular_references = build_circular_reference_report(&all_allocations);

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
        top_allocation_sites: top_n_reports.top_allocation_sites,
        top_leaked_allocations: top_n_reports.top_leaked_allocations,
        top_temporary_churn: top_n_reports.top_temporary_churn,
        circular_references,
    })
}

/// Build Top N reports from allocations
fn build_top_n_reports(allocations: &[crate::capture::types::AllocationInfo]) -> TopNReports {
    let converted_allocations = allocations.to_vec();

    let analyzer = TopNAnalyzer::new(converted_allocations);

    let top_allocation_sites = analyzer
        .top_allocation_sites(10)
        .iter()
        .map(|site| TopAllocationSite {
            name: site.name.clone(),
            total_bytes: site.total_bytes,
            allocation_count: site.allocation_count,
        })
        .collect();

    let top_leaked_allocations = analyzer
        .top_leaked_bytes(10)
        .iter()
        .map(|leaked| TopLeakedAllocation {
            address: format!("0x{:x}", leaked.ptr),
            size: leaked.size,
            type_name: leaked
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            timestamp_alloc: leaked.timestamp_alloc,
            stack_trace: leaked.stack_trace.clone(),
        })
        .collect();

    let top_temporary_churn = analyzer
        .top_temporary_churn(10, 100) // 100ms threshold
        .iter()
        .map(|churn| TopTemporaryChurn {
            name: churn.name.clone(),
            allocation_count: churn.allocation_count,
            total_bytes: churn.total_bytes,
            average_lifetime_ms: churn.average_lifetime_ms,
        })
        .collect();

    TopNReports {
        top_allocation_sites,
        top_leaked_allocations,
        top_temporary_churn,
    }
}

/// Build circular reference report from allocations
fn build_circular_reference_report(
    allocations: &[crate::capture::types::AllocationInfo],
) -> CircularReferenceReport {
    // Use existing circular reference detection
    let analysis = crate::analysis::circular_reference::detect_circular_references(allocations);

    CircularReferenceReport {
        count: analysis.circular_references.len(),
        total_leaked_memory: analysis.total_leaked_memory,
        pointers_in_cycles: analysis.pointers_in_cycles,
        total_smart_pointers: analysis.total_smart_pointers,
        has_cycles: !analysis.circular_references.is_empty(),
    }
}

/// Top N reports container
struct TopNReports {
    top_allocation_sites: Vec<TopAllocationSite>,
    top_leaked_allocations: Vec<TopLeakedAllocation>,
    top_temporary_churn: Vec<TopTemporaryChurn>,
}

/// Rebuild all allocations from event_store (unified data source).
pub fn rebuild_allocations_from_events(
    events: &[crate::event_store::event::MemoryEvent],
) -> Vec<crate::capture::types::AllocationInfo> {
    use crate::event_store::event::MemoryEventType;

    let mut active_allocations: HashMap<usize, crate::capture::types::AllocationInfo> =
        HashMap::new();
    let mut container_allocations: Vec<crate::capture::types::AllocationInfo> = Vec::new();
    let mut clone_info_map: HashMap<usize, crate::capture::types::CloneInfo> = HashMap::new();

    for event in events {
        match event.event_type {
            MemoryEventType::Allocate => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let mut alloc = crate::capture::types::AllocationInfo::new(event.ptr, event.size);
                alloc.timestamp_alloc = event.timestamp;
                alloc.var_name = event.var_name.clone();
                alloc.type_name = event.type_name.clone();
                alloc.thread_id = std::thread::current().id();
                alloc.thread_id_u64 = event.thread_id;
                alloc.stack_trace = stack_trace.map(|s| vec![s]);
                alloc.module_path = event.module_path.clone();
                active_allocations.insert(event.ptr, alloc);
            }
            MemoryEventType::Metadata => {
                let stack_trace = event
                    .source_file
                    .as_ref()
                    .map(|file| format!("{}:{}", file, event.source_line.unwrap_or(0)));
                let mut alloc = crate::capture::types::AllocationInfo::new(0, event.size);
                alloc.timestamp_alloc = event.timestamp;
                alloc.var_name = event.var_name.clone();
                alloc.type_name = event.type_name.clone();
                alloc.thread_id = std::thread::current().id();
                alloc.thread_id_u64 = event.thread_id;
                alloc.stack_trace = stack_trace.map(|s| vec![s]);
                alloc.module_path = event.module_path.clone();
                container_allocations.push(alloc);
            }
            MemoryEventType::Clone => {
                // Track clone information for CloneInfo
                if let (Some(source_ptr), Some(target_ptr)) =
                    (event.clone_source_ptr, event.clone_target_ptr)
                {
                    // Increment clone count for source
                    clone_info_map
                        .entry(source_ptr)
                        .and_modify(|info| info.clone_count += 1)
                        .or_insert(crate::capture::types::CloneInfo {
                            clone_count: 1,
                            is_clone: false,
                            original_ptr: None,
                            _source: None,
                            _confidence: None,
                        });

                    // Mark target as a clone
                    clone_info_map
                        .entry(target_ptr)
                        .and_modify(|info| {
                            info.is_clone = true;
                            info.original_ptr = Some(source_ptr);
                        })
                        .or_insert(crate::capture::types::CloneInfo {
                            clone_count: 0,
                            is_clone: true,
                            original_ptr: Some(source_ptr),
                            _source: None,
                            _confidence: None,
                        });
                }
            }
            MemoryEventType::Deallocate => {
                active_allocations.remove(&event.ptr);
            }
            _ => {}
        }
    }

    let mut all_allocations: Vec<_> = active_allocations.into_values().collect();

    // Auto-detect smart pointers and fill smart_pointer_info
    for alloc in &mut all_allocations {
        let type_name_lower = alloc
            .type_name
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        // Detect smart pointer types from type name
        let (is_smart_pointer, pointer_type) = if type_name_lower.contains("arc") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Arc,
            )
        } else if type_name_lower.contains("rc") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Rc,
            )
        } else if type_name_lower.contains("box") {
            (
                true,
                crate::capture::types::smart_pointer::SmartPointerType::Box,
            )
        } else {
            (
                false,
                crate::capture::types::smart_pointer::SmartPointerType::Rc,
            )
        };

        if is_smart_pointer {
            // Check if we have clone info for this pointer
            if let Some(clone_info) = clone_info_map.get(&alloc.ptr) {
                let smart_info = crate::capture::types::smart_pointer::SmartPointerInfo {
                    data_ptr: alloc.ptr,
                    pointer_type,
                    is_data_owner: !clone_info.is_clone,
                    ref_count_history: vec![],
                    weak_count: None,
                    cloned_from: clone_info.original_ptr,
                    clones: vec![],
                    is_implicitly_deallocated: false,
                    is_weak_reference: false,
                };
                alloc.smart_pointer_info = Some(smart_info);
            } else {
                // Auto-create smart_pointer_info for smart pointers without explicit clone tracking
                let smart_info = crate::capture::types::smart_pointer::SmartPointerInfo {
                    data_ptr: alloc.ptr,
                    pointer_type,
                    is_data_owner: true,
                    ref_count_history: vec![],
                    weak_count: None,
                    cloned_from: None,
                    clones: vec![],
                    is_implicitly_deallocated: false,
                    is_weak_reference: false,
                };
                alloc.smart_pointer_info = Some(smart_info);
            }
        }
    }

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

/// Calculate lifetime in milliseconds from timestamps
fn calculate_lifetime_ms(timestamp_alloc: u64, timestamp_dealloc: Option<u64>) -> f64 {
    match timestamp_dealloc {
        Some(dealloc) => (dealloc - timestamp_alloc) as f64 / 1_000_000.0,
        None => 0.0,
    }
}

/// Build allocation info from allocations
fn build_allocation_info(
    allocations: &[crate::capture::types::AllocationInfo],
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
                timestamp: format!("{:?}", a.timestamp_alloc),
                thread_id: format!("{:?}", a.thread_id),
                immutable_borrows: 0,
                mutable_borrows: 0,
                is_clone: a.clone_info.as_ref().map(|i| i.is_clone).unwrap_or(false),
                clone_count: a.clone_info.as_ref().map(|i| i.clone_count).unwrap_or(0),
                timestamp_alloc: a.timestamp_alloc,
                timestamp_dealloc: a.timestamp_dealloc.unwrap_or(0),
                lifetime_ms: calculate_lifetime_ms(a.timestamp_alloc, a.timestamp_dealloc),
                is_leaked: a.timestamp_dealloc.is_none(),
                allocation_type: if is_smart_pointer {
                    smart_pointer_type.clone()
                } else {
                    "heap".to_string()
                },
                is_smart_pointer,
                smart_pointer_type,
                source_file: a
                    .stack_trace
                    .as_ref()
                    .and_then(|s| s.first())
                    .map(|s| s.split(':').next().unwrap_or("").to_string()),
                source_line: a
                    .stack_trace
                    .as_ref()
                    .and_then(|s| s.first())
                    .and_then(|s| s.split(':').nth(1).and_then(|l| l.parse().ok())),
                module_path: a.module_path.clone(),
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
    all_allocations: &[crate::capture::types::AllocationInfo],
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
    all_allocations: &[crate::capture::types::AllocationInfo],
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
    all_allocations: &[crate::capture::types::AllocationInfo],
) -> Vec<PassportDetail> {
    passports
        .values()
        .map(|p| build_passport_detail(p, all_allocations))
        .collect()
}

/// Build single passport detail
fn build_passport_detail(
    p: &crate::analysis::memory_passport_tracker::MemoryPassport,
    all_allocations: &[crate::capture::types::AllocationInfo],
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

/// Build ownership graph information
fn build_ownership_graph_info(
    allocations: &[crate::capture::types::AllocationInfo],
) -> OwnershipGraphInfo {
    // Simplified ownership graph info for now
    let total_nodes = allocations.len();
    let total_edges = 0;
    let total_cycles = 0;

    OwnershipGraphInfo {
        total_nodes,
        total_edges,
        total_cycles,
        rc_clone_count: 0,
        arc_clone_count: 0,
        has_issues: false,
        issues: vec![],
        root_cause: None,
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that infer_type_from_size returns correct types.
    /// Invariants: Size 8 should return pointer type, 24 should return Vec/String.
    #[test]
    fn test_infer_type_from_size() {
        assert!(
            infer_type_from_size(8).contains("c_void"),
            "Size 8 should be pointer type"
        );
        assert!(
            infer_type_from_size(16).contains("[T]"),
            "Size 16 should be slice type"
        );
        assert!(
            infer_type_from_size(24).contains("Vec"),
            "Size 24 should be Vec/String type"
        );
        assert!(
            infer_type_from_size(32).contains("CStruct"),
            "Size 32 should be CStruct"
        );
        assert!(
            infer_type_from_size(48).contains("CStruct"),
            "Size 48 should be CStruct"
        );
        assert!(
            infer_type_from_size(64).contains("CStruct"),
            "Size 64 should be CStruct"
        );
        assert!(
            infer_type_from_size(128).contains("Vec"),
            "Size 128 should be Vec type"
        );
        assert!(
            infer_type_from_size(100).contains("[u8]"),
            "Size 100 should be byte array"
        );
        assert_eq!(
            infer_type_from_size(17),
            "unknown",
            "Size 17 should be unknown"
        );
    }

    /// Objective: Verify that get_inferred_type_name prefers explicit type name.
    /// Invariants: Non-empty type name should be returned as-is.
    #[test]
    fn test_get_inferred_type_name_explicit() {
        let result = get_inferred_type_name("MyType", 100);
        assert_eq!(result, "MyType", "Should return explicit type name");
    }

    /// Objective: Verify that get_inferred_type_name infers for unknown types.
    /// Invariants: Unknown type should trigger size-based inference.
    #[test]
    fn test_get_inferred_type_name_unknown() {
        let result = get_inferred_type_name("unknown", 24);
        assert!(result.contains("Vec"), "Should infer Vec type for size 24");

        let result = get_inferred_type_name("-", 8);
        assert!(
            result.contains("c_void"),
            "Should infer pointer type for size 8"
        );

        let result = get_inferred_type_name("", 16);
        assert!(
            result.contains("[T]"),
            "Should infer slice type for size 16"
        );
    }

    /// Objective: Verify that HealthInfo struct can be created.
    /// Invariants: All fields should be accessible.
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

        assert_eq!(info.health_score, 85, "health_score should be 85");
        assert_eq!(info.health_status, "Good", "health_status should be Good");
        assert_eq!(info.safe_ops_count, 100, "safe_ops_count should be 100");
        assert_eq!(info.high_risk_count, 5, "high_risk_count should be 5");
    }

    /// Objective: Verify that AsyncSummary can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_async_summary_creation() {
        let summary = AsyncSummary {
            total_tasks: 10,
            active_tasks: 5,
            total_allocations: 100,
            total_memory_bytes: 10240,
            peak_memory_bytes: 5120,
        };

        assert_eq!(summary.total_tasks, 10, "total_tasks should match");
        assert_eq!(summary.active_tasks, 5, "active_tasks should match");
        assert_eq!(
            summary.total_allocations, 100,
            "total_allocations should match"
        );
    }

    /// Objective: Verify that OwnershipGraphInfo can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_ownership_graph_info_creation() {
        let info = OwnershipGraphInfo {
            total_nodes: 10,
            total_edges: 15,
            total_cycles: 2,
            rc_clone_count: 5,
            arc_clone_count: 3,
            has_issues: true,
            issues: vec![OwnershipIssue {
                issue_type: "cycle".to_string(),
                severity: "warning".to_string(),
                description: "Cycle detected".to_string(),
            }],
            root_cause: Some(RootCauseInfo {
                cause: "circular_reference".to_string(),
                description: "Circular reference detected".to_string(),
                impact: "Memory leak potential".to_string(),
            }),
        };

        assert_eq!(info.total_nodes, 10, "total_nodes should match");
        assert_eq!(info.total_edges, 15, "total_edges should match");
        assert!(info.has_issues, "has_issues should be true");
    }

    /// Objective: Verify that OwnershipIssue can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_ownership_issue_creation() {
        let issue = OwnershipIssue {
            issue_type: "cycle".to_string(),
            severity: "error".to_string(),
            description: "Reference cycle detected".to_string(),
        };

        assert_eq!(issue.issue_type, "cycle", "issue_type should match");
        assert_eq!(issue.severity, "error", "severity should match");
    }

    /// Objective: Verify that RootCauseInfo can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_root_cause_info_creation() {
        let info = RootCauseInfo {
            cause: "circular_reference".to_string(),
            description: "A references B, B references A".to_string(),
            impact: "Memory will not be freed".to_string(),
        };

        assert_eq!(info.cause, "circular_reference", "cause should match");
        assert_eq!(
            info.description, "A references B, B references A",
            "description should match"
        );
    }

    /// Objective: Verify that SystemResources can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_system_resources_creation() {
        let resources = SystemResources {
            os_name: "macOS".to_string(),
            os_version: "14.0".to_string(),
            architecture: "arm64".to_string(),
            cpu_cores: 8,
            total_physical: "16 GB".to_string(),
            available_physical: "8 GB".to_string(),
            used_physical: "8 GB".to_string(),
            page_size: 4096,
        };

        assert_eq!(resources.os_name, "macOS", "os_name should match");
        assert_eq!(resources.cpu_cores, 8, "cpu_cores should match");
        assert_eq!(resources.page_size, 4096, "page_size should match");
    }

    /// Objective: Verify that ThreadStats can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_thread_stats_creation() {
        let stats = ThreadStats {
            id: 1,
            allocations: 10,
            memory: 1024,
            peak: 512,
            status: "active".to_string(),
        };

        assert_eq!(stats.id, 1, "id should match");
        assert_eq!(stats.allocations, 10, "allocations should match");
        assert_eq!(stats.memory, 1024, "memory should match");
    }

    /// Objective: Verify that TimelineAllocation can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_timeline_allocation_creation() {
        let alloc = TimelineAllocation {
            timestamp: 1000,
            thread_id: 1,
            size: 1024,
            var_name: Some("buffer".to_string()),
        };

        assert_eq!(alloc.timestamp, 1000, "timestamp should match");
        assert_eq!(alloc.thread_id, 1, "thread_id should match");
        assert_eq!(alloc.size, 1024, "size should match");
    }

    /// Objective: Verify that ThreadConflict can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_thread_conflict_creation() {
        let conflict = ThreadConflict {
            description: "Data race detected".to_string(),
            threads: "Thread-1, Thread-2".to_string(),
            conflict_type: "data_race".to_string(),
        };

        assert_eq!(
            conflict.description, "Data race detected",
            "description should match"
        );
        assert_eq!(
            conflict.threads, "Thread-1, Thread-2",
            "threads should match"
        );
    }

    /// Objective: Verify that extract_user_source_file extracts user code correctly.
    /// Invariants: Should return None for std/rustc frames, Some for user frames.
    #[test]
    fn test_extract_user_source_file() {
        let std_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/library/std/src/thread.rs:50".to_string(),
        ]);
        assert!(
            extract_user_source_file(&std_frames).is_none(),
            "Should return None for std frames"
        );

        let user_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/Users/test/my_project/src/main.rs:42".to_string(),
        ]);
        let result = extract_user_source_file(&user_frames);
        assert!(result.is_some(), "Should return Some for user frames");
        assert!(
            result.unwrap().contains("main.rs"),
            "Should contain main.rs"
        );
    }

    /// Objective: Verify that extract_user_source_line extracts line number correctly.
    /// Invariants: Should return None for std/rustc frames, Some for user frames.
    #[test]
    fn test_extract_user_source_line() {
        let std_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/library/std/src/thread.rs:50".to_string(),
        ]);
        assert!(
            extract_user_source_line(&std_frames).is_none(),
            "Should return None for std frames"
        );

        let user_frames = Some(vec![
            "/rustc/library/alloc/src/vec.rs:100".to_string(),
            "/Users/test/my_project/src/main.rs:42".to_string(),
        ]);
        let result = extract_user_source_line(&user_frames);
        assert!(result.is_some(), "Should return Some for user frames");
        assert_eq!(result.unwrap(), 42, "Should extract line number 42");
    }

    /// Objective: Verify that calculate_health_info returns correct health score.
    /// Invariants: Health score should be calculated correctly based on inputs.
    #[test]
    fn test_calculate_health_info_empty() {
        let health = calculate_health_info(&[], &[], 0, 0);
        assert!(health.health_score > 0, "Health score should be positive");
        assert!(
            health.health_status.contains("Excellent") || health.health_status.contains("Good"),
            "Empty data should have good health status"
        );
    }

    /// Objective: Verify that calculate_health_info handles leaks correctly.
    /// Invariants: Leaks should reduce health score.
    #[test]
    fn test_calculate_health_info_with_leaks() {
        let health_no_leaks = calculate_health_info(&[], &[], 0, 100);
        let health_with_leaks = calculate_health_info(&[], &[], 50, 100);

        assert!(
            health_no_leaks.health_score > health_with_leaks.health_score,
            "Leaks should reduce health score"
        );
    }

    /// Objective: Verify that calculate_health_info handles unsafe reports correctly.
    /// Invariants: High risk reports should reduce health score.
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
        };

        let health_safe = calculate_health_info(&[safe_report], &[], 0, 100);
        let health_high_risk = calculate_health_info(&[high_risk_report], &[], 0, 100);

        assert!(
            health_safe.health_score > health_high_risk.health_score,
            "High risk should reduce health score"
        );
    }

    /// Objective: Verify that LifecycleEventInfo can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_lifecycle_event_info_creation() {
        let event = LifecycleEventInfo {
            timestamp: 1000,
            event_type: "AllocatedInRust".to_string(),
            icon: "🟢".to_string(),
            color: "#10b981".to_string(),
            context: "Rust Allocation".to_string(),
        };

        assert_eq!(event.timestamp, 1000, "timestamp should match");
        assert_eq!(
            event.event_type, "AllocatedInRust",
            "event_type should match"
        );
        assert_eq!(event.icon, "🟢", "icon should match");
    }

    /// Objective: Verify that PassportDetail can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_passport_detail_creation() {
        let detail = PassportDetail {
            passport_id: "passport-1".to_string(),
            allocation_ptr: "0x1000".to_string(),
            var_name: "buffer".to_string(),
            type_name: "Vec<u8>".to_string(),
            size_bytes: 1024,
            status: "active".to_string(),
            created_at: 1000,
            updated_at: 2000,
            is_leaked: false,
            ffi_tracked: false,
            lifecycle_events: vec![],
            cross_boundary_events: vec![],
            risk_level: "low".to_string(),
            risk_confidence: 0.85,
        };

        assert_eq!(detail.passport_id, "passport-1", "passport_id should match");
        assert_eq!(detail.size_bytes, 1024, "size_bytes should match");
        assert_eq!(detail.status, "active", "status should match");
    }

    /// Objective: Verify that UnsafeReport can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_unsafe_report_creation() {
        let report = UnsafeReport {
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
        };

        assert_eq!(report.passport_id, "passport-1", "passport_id should match");
        assert_eq!(report.risk_level, "medium", "risk_level should match");
    }

    /// Objective: Verify that BoundaryEventInfo can be created.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_boundary_event_info_creation() {
        let event = BoundaryEventInfo {
            event_type: "RustToFfi".to_string(),
            from_context: "Rust".to_string(),
            to_context: "FFI".to_string(),
            timestamp: 1000,
            icon: "⬇️".to_string(),
            color: "#f59e0b".to_string(),
        };

        assert_eq!(event.event_type, "RustToFfi", "event_type should match");
        assert_eq!(event.from_context, "Rust", "from_context should match");
        assert_eq!(event.to_context, "FFI", "to_context should match");
    }

    /// Objective: Verify that extract_user_source_file handles memscope frames correctly.
    /// Invariants: Should return None for memscope internal frames.
    #[test]
    fn test_extract_user_source_file_memscope() {
        let memscope_frames = Some(vec![
            "/Users/test/memscope/src/tracker.rs:100".to_string(),
            "/Users/test/.cargo/registry/src/memscope-rs/src/lib.rs:50".to_string(),
        ]);
        assert!(
            extract_user_source_file(&memscope_frames).is_none(),
            "Should return None for memscope frames"
        );
    }

    /// Objective: Verify that extract_user_source_line handles memscope frames correctly.
    /// Invariants: Should return None for memscope internal frames.
    #[test]
    fn test_extract_user_source_line_memscope() {
        let memscope_frames = Some(vec![
            "/Users/test/memscope/src/tracker.rs:100".to_string(),
            "/Users/test/.cargo/registry/src/memscope-rs/src/lib.rs:50".to_string(),
        ]);
        assert!(
            extract_user_source_line(&memscope_frames).is_none(),
            "Should return None for memscope frames"
        );
    }

    /// Objective: Verify that extract_user_source_file handles None stack trace.
    /// Invariants: Should return None for None stack trace.
    #[test]
    fn test_extract_user_source_file_none() {
        assert!(
            extract_user_source_file(&None).is_none(),
            "Should return None for None stack trace"
        );
    }

    /// Objective: Verify that extract_user_source_line handles None stack trace.
    /// Invariants: Should return None for None stack trace.
    #[test]
    fn test_extract_user_source_line_none() {
        assert!(
            extract_user_source_line(&None).is_none(),
            "Should return None for None stack trace"
        );
    }

    /// Objective: Verify that calculate_health_info handles passport details correctly.
    /// Invariants: Leaked passports should affect health score.
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
        };

        let health_clean = calculate_health_info(&[], &[clean_passport], 0, 100);
        let health_leaked = calculate_health_info(&[], &[leaked_passport], 1, 100);

        assert!(
            health_clean.clean_passport_count >= health_leaked.clean_passport_count,
            "Clean passport count should be higher for clean passports"
        );
    }

    /// Objective: Verify that ThreadInfo can be created with all fields.
    /// Invariants: All fields should be properly set.
    #[test]
    fn test_thread_info_creation() {
        let info = ThreadInfo {
            thread_id: "Thread-1".to_string(),
            thread_summary: "10 allocs, 1.5KB".to_string(),
            allocation_count: 10,
            current_memory: "1.5 KB".to_string(),
            peak_memory: "2.0 KB".to_string(),
            total_allocated: "5.0 KB".to_string(),
            current_memory_bytes: 1536,
            peak_memory_bytes: 2048,
            total_allocated_bytes: 5120,
        };

        assert_eq!(info.thread_id, "Thread-1", "thread_id should match");
        assert_eq!(info.allocation_count, 10, "allocation_count should match");
        assert_eq!(
            info.current_memory_bytes, 1536,
            "current_memory_bytes should match"
        );
    }
}
