//! Report building helpers for dashboard context.
//!
//! These functions transform raw allocation/event/passport data into
//! the dashboard's structured report types (`AllocationInfo`,
//! `RelationshipInfo`, `UnsafeReport`, `PassportDetail`, etc.).

use super::event_dto::{DashboardEventDTO, DataIndex, EventSummary};
use super::event_reconstructor::SamplingMetadata;
use super::inference::get_inferred_type_name;
use super::types::*;
use crate::analysis::memory_passport_tracker::{PassportEventType, PassportStatus};
use crate::analysis::top_n::TopNAnalyzer;
use crate::analyzer::Analyzer;
use std::collections::HashMap;

/// Build allocation info from allocations
pub fn build_allocation_info(
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
                generation_id: a.generation_id,
                provenance: infer_provenance(a),
                evidence: if a.generation_id > 0 {
                    EvidenceLevel::Observed
                } else {
                    EvidenceLevel::Inferred
                },
                confidence: if a.is_leaked {
                    RiskConfidence::Confirmed
                } else {
                    RiskConfidence::Likely
                },
                layout_snapshot: infer_layout_snapshot(&type_name, a.size, a.generation_id),
            }
        })
        .collect()
}

/// Calculate lifetime in milliseconds from timestamps
fn calculate_lifetime_ms(timestamp_alloc: u64, timestamp_dealloc: Option<u64>) -> f64 {
    match timestamp_dealloc {
        Some(dealloc) => (dealloc - timestamp_alloc) as f64 / 1_000_000.0,
        None => 0.0,
    }
}

/// Infer a TypeLayoutSnapshot from allocation attributes at report-build time.
fn infer_layout_snapshot(
    type_name: &str,
    size: usize,
    generation_id: usize,
) -> Option<crate::capture::types::TypeLayoutSnapshot> {
    let gen = generation_id;

    // Try Vec<T> — pattern like "Vec<i32>" or "std::vec::Vec<u8>"
    if let Some(inner) = type_name
        .rsplit("Vec<")
        .nth(1)
        .and_then(|_| type_name.split("Vec<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let elem_size = estimate_type_size(inner);
        let cap = if elem_size > 0 { size / elem_size } else { 0 };
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Vec<{}>", inner),
            size_of_t: std::mem::size_of::<*const usize>() * 3,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::Container,
            pointer_width: crate::capture::types::PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<*const usize>() * 3,
            allocated_size_bytes: size,
            used_size_bytes: 0,
            reserved_size_bytes: size,
            element_size: Some(elem_size),
            element_align: Some(elem_size),
            container_len: Some(0),
            container_capacity: Some(cap),
            strong_count: None,
            weak_count: None,
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // String / std::string::String
    if type_name == "String" || type_name.contains("std::string::String") {
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: "String".to_string(),
            size_of_t: std::mem::size_of::<*const usize>() * 3,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::Container,
            pointer_width: crate::capture::types::PointerWidth::Unknown,
            logical_size_bytes: std::mem::size_of::<*const usize>() * 3,
            allocated_size_bytes: size,
            used_size_bytes: 0,
            reserved_size_bytes: size,
            element_size: Some(1),
            element_align: Some(1),
            container_len: Some(0),
            container_capacity: Some(size),
            strong_count: None,
            weak_count: None,
            pointee_type: Some("u8".to_string()),
            generation_id: gen,
        });
    }

    // Box<T>
    if let Some(inner) = type_name
        .rsplit("Box<")
        .nth(1)
        .and_then(|_| type_name.split("Box<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Box<{}>", inner),
            size_of_t: std::mem::size_of::<usize>(),
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>(),
            allocated_size_bytes: pointee_size,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: None,
            weak_count: None,
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Arc<T>
    if let Some(inner) = type_name
        .rsplit("Arc<")
        .nth(1)
        .and_then(|_| type_name.split("Arc<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Arc<{}>", inner),
            size_of_t: std::mem::size_of::<usize>() * 2,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>() * 2,
            allocated_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: Some(1),
            weak_count: Some(0),
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Rc<T>
    if let Some(inner) = type_name
        .rsplit("Rc<")
        .nth(1)
        .and_then(|_| type_name.split("Rc<").nth(1))
        .and_then(|s| s.strip_suffix('>'))
        .map(|s| s.trim())
    {
        let pointee_size = estimate_type_size(inner);
        return Some(crate::capture::types::TypeLayoutSnapshot {
            type_name: format!("Rc<{}>", inner),
            size_of_t: std::mem::size_of::<usize>() * 2,
            align_of_t: std::mem::align_of::<usize>(),
            is_sized: true,
            repr_hint: crate::capture::types::ReprHint::Rust,
            layout_kind: crate::capture::types::LayoutKind::SmartPointer,
            pointer_width: crate::capture::types::PointerWidth::Thin,
            logical_size_bytes: std::mem::size_of::<usize>() * 2,
            allocated_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            used_size_bytes: pointee_size,
            reserved_size_bytes: pointee_size + std::mem::size_of::<usize>() * 2,
            element_size: Some(pointee_size),
            element_align: Some(pointee_size),
            container_len: None,
            container_capacity: None,
            strong_count: Some(1),
            weak_count: Some(0),
            pointee_type: Some(inner.to_string()),
            generation_id: gen,
        });
    }

    // Fallback: generic snapshot from type name and size
    let layout_kind = crate::capture::types::infer_layout_kind_from_name(type_name);
    Some(crate::capture::types::TypeLayoutSnapshot {
        type_name: type_name.to_string(),
        size_of_t: size,
        align_of_t: 0,
        is_sized: true,
        repr_hint: crate::capture::types::ReprHint::Unknown,
        layout_kind,
        pointer_width: crate::capture::types::PointerWidth::Unknown,
        logical_size_bytes: size,
        allocated_size_bytes: size,
        used_size_bytes: size,
        reserved_size_bytes: size,
        element_size: None,
        element_align: None,
        container_len: None,
        container_capacity: None,
        strong_count: None,
        weak_count: None,
        pointee_type: None,
        generation_id: gen,
    })
}

/// Estimate the size of a Rust type from its name string.
fn estimate_type_size(type_name: &str) -> usize {
    match type_name.trim() {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" | "char" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" | "usize" | "isize" => 8,
        "u128" | "i128" => 16,
        "()" => 0,
        "*mut u8" | "*const u8" | "*mut c_void" | "*const c_void" | "String" => 8,
        s if s.contains("String") => 24, // String struct on stack
        s if s.starts_with('&') => 16,   // fat reference (ptr + len)
        s if s.contains('*') => 8,       // pointer
        _ => 8,                          // default fallback
    }
}

/// Infer pointer provenance from allocation attributes
fn infer_provenance(a: &crate::capture::types::AllocationInfo) -> String {
    if a.smart_pointer_info.is_some() {
        return "smart_pointer".to_string();
    }
    if a.clone_info.as_ref().map(|c| c.is_clone).unwrap_or(false) {
        return "clone".to_string();
    }
    if a.is_leaked && a.generation_id > 0 {
        return "reallocated_then_leaked".to_string();
    }
    if a.generation_id > 0 {
        return "reallocation".to_string();
    }
    "allocator".to_string()
}

/// Build relationships from analyzer
pub fn build_relationships(az: &mut Analyzer) -> Vec<RelationshipInfo> {
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
                crate::analyzer::Relation::ArcClone => ("Arc_clone", "#8b5cf6", 0.7),
                crate::analyzer::Relation::RcClone => ("Rc_clone", "#10b981", 0.9),
                crate::analyzer::Relation::ImmutableBorrow => ("immutable_borrow", "#3b82f6", 0.8),
                crate::analyzer::Relation::MutableBorrow => ("mutable_borrow", "#f59e0b", 0.9),
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
pub fn build_unsafe_reports(
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
pub fn build_passport_details(
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
pub fn build_lifecycle_events(
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

/// Aggregate thread data from allocation info
pub fn aggregate_thread_data(allocations: &[AllocationInfo]) -> Vec<ThreadInfo> {
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
                super::helpers::format_bytes(agg.current_memory)
            );
            let thread_id = super::helpers::format_thread_id(&raw_tid);
            ThreadInfo {
                thread_id,
                thread_summary: summary,
                allocation_count: agg.allocation_count,
                current_memory: super::helpers::format_bytes(agg.current_memory),
                peak_memory: super::helpers::format_bytes(agg.peak_memory),
                total_allocated: super::helpers::format_bytes(agg.total_allocated),
                current_memory_bytes: agg.current_memory,
                peak_memory_bytes: agg.peak_memory,
                total_allocated_bytes: agg.total_allocated,
            }
        })
        .collect()
}

/// Build async tasks
pub fn build_async_tasks(
    async_tracker: Option<&std::sync::Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
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
pub fn build_async_summary(
    async_tracker: Option<&std::sync::Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
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
pub fn build_ownership_graph_info(
    allocations: &[crate::capture::types::AllocationInfo],
) -> OwnershipGraphInfo {
    let total_nodes = allocations.len();
    OwnershipGraphInfo {
        total_nodes,
        total_edges: 0,
        total_cycles: 0,
        rc_clone_count: 0,
        arc_clone_count: 0,
        has_issues: false,
        issues: vec![],
        root_cause: None,
    }
}

/// Build Top N reports from allocations
pub fn build_top_n_reports(allocations: &[crate::capture::types::AllocationInfo]) -> TopNReports {
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
        .top_temporary_churn(10, 100)
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

/// Top N reports container
pub struct TopNReports {
    pub top_allocation_sites: Vec<TopAllocationSite>,
    pub top_leaked_allocations: Vec<TopLeakedAllocation>,
    pub top_temporary_churn: Vec<TopTemporaryChurn>,
}

/// Build circular reference report from allocations
pub fn build_circular_reference_report(
    allocations: &[crate::capture::types::AllocationInfo],
) -> CircularReferenceReport {
    let analysis = crate::analysis::circular_reference::detect_circular_references(allocations);
    CircularReferenceReport {
        count: analysis.circular_references.len(),
        total_leaked_memory: analysis.total_leaked_memory,
        pointers_in_cycles: analysis.pointers_in_cycles,
        total_smart_pointers: analysis.total_smart_pointers,
        has_cycles: !analysis.circular_references.is_empty(),
    }
}

/// Calculate health info
pub fn calculate_health_info(
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
    let high_risk_penalty = 10.0;
    let leak_score = (100.0 - (leak_count as f64 / total_allocs as f64) * 100.0).max(0.0);
    let unsafe_score = (100.0 - (unsafe_count as f64 / total_allocs as f64) * 50.0).max(0.0);
    let risk_score = (100.0 - high_risk_count as f64 * high_risk_penalty).max(0.0);
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

/// Health info structure
pub struct HealthInfo {
    pub health_score: u32,
    pub health_status: String,
    pub safe_ops_count: usize,
    pub high_risk_count: usize,
    pub clean_passport_count: usize,
    pub active_passport_count: usize,
    pub leaked_passport_count: usize,
    pub ffi_tracked_count: usize,
    pub safe_code_percent: u32,
}

/// Build JSON data for template injection, including event DTOs, DataIndex, and sampling metadata.
#[allow(clippy::too_many_arguments, dead_code)]
pub fn build_json_data(
    alloc_info: &[super::types::AllocationInfo],
    relationships: &[super::types::RelationshipInfo],
    unsafe_reports: &[super::types::UnsafeReport],
    thread_data: &[super::types::ThreadInfo],
    passport_details: &[super::types::PassportDetail],
    active_allocations: usize,
    total_allocations: usize,
    leak_count: usize,
    async_tasks: &[super::types::AsyncTaskInfo],
    async_summary: &super::types::AsyncSummary,
    ownership_graph: &super::types::OwnershipGraphInfo,
    health_score: u32,
    task_graph_json: &str,
    events: &[DashboardEventDTO],
    event_summary: &EventSummary,
    data_index: &DataIndex,
    sampling: &SamplingMetadata,
) -> Result<String, Box<dyn std::error::Error>> {
    #[derive(serde::Serialize)]
    struct DashboardData<'a> {
        allocations: &'a [super::types::AllocationInfo],
        relationships: &'a [super::types::RelationshipInfo],
        unsafe_reports: &'a [super::types::UnsafeReport],
        threads: &'a [super::types::ThreadInfo],
        passport_details: &'a [super::types::PassportDetail],
        active_allocations: usize,
        total_allocations: usize,
        leak_count: usize,
        async_tasks: &'a [super::types::AsyncTaskInfo],
        async_summary: &'a super::types::AsyncSummary,
        ownership_graph: &'a super::types::OwnershipGraphInfo,
        health_score: u32,
        task_graph_json: &'a str,
        events: &'a [DashboardEventDTO],
        event_summary: &'a EventSummary,
        data_index: &'a DataIndex,
        sampling: &'a SamplingMetadata,
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
        task_graph_json,
        events,
        event_summary,
        data_index,
        sampling,
    };

    Ok(serde_json::to_string(&data)?)
}

/// Infer type from size (fallback when no type name is recorded)
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

#[cfg(test)]
mod tests {
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
        assert!(
            health.health_status.contains("Excellent") || health.health_status.contains("Good")
        );
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
        let summary = build_async_summary(None);
        assert_eq!(summary.total_tasks, 0);
        assert_eq!(summary.active_tasks, 0);
    }
}
