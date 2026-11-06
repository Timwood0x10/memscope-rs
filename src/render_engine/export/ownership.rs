//! Ownership graph projection and JSON export.

use crate::analysis::is_virtual_pointer;
use crate::analysis::node_id::NodeId;
use crate::analysis::ownership_graph::{EdgeKind, OwnershipGraph, OwnershipOp};
use crate::core::{MemScopeError, MemScopeResult};
use crate::snapshot::ActiveAllocation;
use serde_json::json;
use std::path::Path;
use tracing::debug;

/// Export ownership graph analysis to JSON
///
/// This function exports the ownership graph including:
/// - Node information (objects with their types and sizes)
/// - Edge information (clone relationships)
/// - Detected cycles (Rc/Arc retain cycles)
/// - Diagnostics (clone storms, cycle warnings)
pub fn export_ownership_graph_json<P: AsRef<Path>>(
    base_path: P,
    allocations: &[crate::capture::types::AllocationInfo],
    event_store: &crate::event_store::EventStore,
) -> MemScopeResult<()> {
    let events = event_store.snapshot();
    let borrow_history =
        crate::analysis::borrow_analysis::get_global_borrow_analyzer().get_borrow_history();
    export_ownership_graph_snapshot(
        base_path,
        allocations,
        &events,
        &borrow_history,
        super::capture_cut_ns(),
    )
}

/// Project ownership using captured events and borrow history, without consulting live state.
pub fn export_ownership_graph_snapshot<P: AsRef<Path>>(
    base_path: P,
    allocations: &[crate::capture::types::AllocationInfo],
    events: &[crate::event_store::MemoryEvent],
    borrow_history: &[crate::analysis::borrow_analysis::BorrowEvent],
    captured_at: u64,
) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();
    // Build ownership graph from allocations
    let graph = build_ownership_graph_from_allocations(allocations, events);
    // Get diagnostics
    let diagnostics = graph.diagnostics(50);
    // Convert nodes to JSON
    let nodes_json: Vec<_> = graph
        .nodes
        .iter()
        .map(|node| {
            json!({
                "id": format!("0x{:x}", node.id.0), "type_name": node.type_name,
                "size": node.size, "stack_ptr": node.stack_ptr.map(|p| format!("0x{:x}", p)),
            })
        })
        .collect();
    // Convert edges to JSON (including borrow edges from BorrowAnalyzer)
    let mut edges_json: Vec<_> = graph.edges.iter().map(|edge| {
        json!({
            "from": format!("0x{:x}", edge.from.0), "to": format!("0x{:x}", edge.to.0),
            "kind": match edge.op {
                EdgeKind::Owns => "Owns", EdgeKind::Contains => "Contains", EdgeKind::Borrows => "Borrows",
                EdgeKind::RcClone => "RcClone", EdgeKind::ArcClone => "ArcClone", EdgeKind::Move => "Move",
                EdgeKind::SharedBorrow => "SharedBorrow", EdgeKind::MutBorrow => "MutBorrow",
            },
        })
    }).collect();
    // Add borrow edges from BorrowAnalyzer
    for event in borrow_history {
        let edge_kind = match event.borrow_info.borrow_type {
            crate::analysis::borrow_analysis::BorrowType::Immutable => "SharedBorrow",
            crate::analysis::borrow_analysis::BorrowType::Mutable => "MutBorrow",
            crate::analysis::borrow_analysis::BorrowType::Shared => "Borrows",
            crate::analysis::borrow_analysis::BorrowType::Weak => "Borrows",
        };
        edges_json.push(json!({
            "from": format!("0x{:x}", event.borrow_info.ptr), "to": format!("0x{:x}", event.borrow_info.ptr),
            "kind": edge_kind, "var_name": event.borrow_info.var_name,
            "thread_id": event.borrow_info.thread_id, "borrow_id": format!("{:?}", event.borrow_info.id),
        }));
    }
    // Convert cycles to JSON
    let cycles_json: Vec<_> = graph
        .cycles
        .iter()
        .map(|cycle| {
            let nodes: Vec<_> = cycle.iter().map(|id| format!("0x{:x}", id.0)).collect();
            json!({ "nodes": nodes, })
        })
        .collect();
    // Convert issues to JSON
    let issues_json: Vec<_> = diagnostics.issues.iter().map(|issue| match issue {
        crate::analysis::ownership_graph::DiagnosticIssue::RcCycle { nodes, cycle_type } => {
            json!({
                "type": "RcCycle", "cycle_type": format!("{:?}", cycle_type),
                "nodes": nodes.iter().map(|id| format!("0x{:x}", id.0)).collect::<Vec<_>>(), "severity": "error",
            })
        }
        crate::analysis::ownership_graph::DiagnosticIssue::ArcCloneStorm { clone_count, threshold } => {
            json!({ "type": "ArcCloneStorm", "clone_count": clone_count, "threshold": threshold, "severity": "warning", })
        }
    }).collect();
    // Build root cause info
    let root_cause_json = graph.find_root_cause().map(|rc| {
        json!({
            "cause": match rc.root_cause {
                crate::analysis::ownership_graph::RootCause::ArcCloneStorm => "ArcCloneStorm",
                crate::analysis::ownership_graph::RootCause::RcCycle => "RcCycle",
            },
            "description": rc.description, "impact": rc.impact,
        })
    });
    let json_data = json!({
        "metadata": {
            "export_version": "2.0", "specification": "ownership graph analysis",
            "timestamp": captured_at / 1_000_000_000,
            "snapshot_timestamp_ns": captured_at,
            "events_source": "event_snapshot", "borrow_source": "borrow_history_snapshot"
        },
        "summary": {
            "total_nodes": graph.nodes.len(), "total_edges": edges_json.len(), "total_cycles": graph.cycles.len(),
            "rc_clone_count": diagnostics.rc_clone_count, "arc_clone_count": diagnostics.arc_clone_count,
            "has_issues": diagnostics.has_issues(),
        },
        "nodes": nodes_json, "edges": edges_json, "cycles": cycles_json,
        "diagnostics": { "issues": issues_json, "root_cause": root_cause_json, },
    });
    let file_path = base_path.join("ownership_graph.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_ownership_graph_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_ownership_graph_json", e.to_string())
    })?;
    Ok(())
}

/// Build ownership graph from allocation data
fn build_ownership_graph_from_allocations(
    allocations: &[crate::capture::types::AllocationInfo],
    events: &[crate::event_store::MemoryEvent],
) -> OwnershipGraph {
    debug!(
        allocations = allocations.len(),
        "Starting build_ownership_graph"
    );
    use crate::analysis::relation_inference::{detect_containers, Relation, RelationGraphBuilder};
    use crate::event_store::MemoryEventType;
    // Convert allocations to passport format for graph building
    // IMPORTANT: For allocations with ptr=0, assign virtual unique IDs to avoid collisions
    debug!("Converting allocations to passports");
    let passports: Vec<(
        NodeId,
        String,
        usize,
        Vec<crate::analysis::ownership_graph::OwnershipEvent>,
    )> = allocations
        .iter()
        .enumerate()
        .map(|(idx, alloc)| {
            let unique_ptr = if alloc.ptr == 0 {
                crate::analysis::VIRTUAL_PTR_BASE + idx
            } else {
                alloc.ptr
            };
            let id = NodeId::from_ptr(unique_ptr);
            let type_name = alloc
                .type_name
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let size = alloc.size;
            // Generate ownership events from allocation info
            let events = vec![crate::analysis::ownership_graph::OwnershipEvent::new(
                alloc.timestamp_alloc,
                OwnershipOp::Create,
                id,
                None,
            )];
            // Detect smart pointer clones from type name
            if type_name.contains("Arc<") || type_name.contains("Rc<") {
                // Check for clone patterns
                // Note: In real tracking, clone events would be recorded at runtime
                // Here we infer from type and allocation patterns
            }
            (id, type_name, size, events)
        })
        .collect();
    debug!(passports = passports.len(), "Created passports");
    debug!("Building initial ownership graph");
    let mut graph = OwnershipGraph::build(&passports);
    debug!(
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        "Initial graph built"
    );
    // Build separate lists for HeapOwner and Container allocations
    // with their original indices preserved for correct edge mapping
    // Step 1: Collect HeapOwner allocations
    let _heap_owner_original_indices: Vec<usize> = allocations
        .iter()
        .enumerate()
        .filter(|(_, a)| a.timestamp_dealloc.is_none())
        .map(|(i, _)| i)
        .collect();
    let heap_owner_allocations: Vec<ActiveAllocation> = allocations
        .iter()
        .enumerate()
        .filter(|(_, a)| a.timestamp_dealloc.is_none())
        .filter_map(|(_idx, a)| {
            // Skip Container types (ptr is 0 or virtual pointer)
            if a.ptr == 0 || is_virtual_pointer(a.ptr) {
                return None;
            }
            Some(ActiveAllocation {
                ptr: Some(a.ptr),
                kind: crate::core::types::TrackKind::HeapOwner {
                    ptr: a.ptr,
                    size: a.size,
                },
                size: a.size,
                allocated_at: a.timestamp_alloc,
                var_name: a.var_name.clone(),
                type_name: a.type_name.clone(),
                thread_id: a.thread_id_u64,
                call_stack_hash: None,
                module_path: a.module_path.clone(),
                stack_ptr: a.stack_ptr,
            })
        })
        .collect();
    // Update stack_ptr for HeapOwner nodes from AllocationInfo
    for (i, alloc) in heap_owner_allocations.iter().enumerate() {
        if i < graph.nodes.len() {
            graph.nodes[i].stack_ptr = alloc.stack_ptr;
        }
    }
    // Step 2: Collect Container events from event_store
    // IMPORTANT: Only include containers that match HeapOwner thread_id
    let valid_thread_ids: std::collections::HashSet<u64> =
        heap_owner_allocations.iter().map(|a| a.thread_id).collect();
    let container_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == MemoryEventType::Metadata)
        .filter(|e| valid_thread_ids.contains(&e.thread_id))
        .filter_map(|e| {
            let type_name = e.type_name.clone().unwrap_or_default();
            let var_name = e.var_name.clone().unwrap_or_default();
            let is_container = type_name.contains("HashMap")
                || type_name.contains("BTreeMap")
                || type_name.contains("VecDeque")
                || type_name.contains("RefCell")
                || type_name.contains("RwLock");
            if is_container {
                return Some((e, type_name, var_name));
            }
            None
        })
        .collect();
    // Assign virtual pointers to Container allocations to avoid ptr=0 collisions
    let container_allocations: Vec<ActiveAllocation> = container_events
        .iter()
        .enumerate()
        .map(|(idx, (e, type_name, var_name))| {
            let virtual_ptr = 0x300000000u64 as usize + idx;
            ActiveAllocation {
                ptr: Some(virtual_ptr),
                kind: crate::core::types::TrackKind::Container,
                size: e.size.max(1),
                allocated_at: e.timestamp,
                var_name: Some(var_name.clone()),
                type_name: Some(type_name.clone()),
                thread_id: e.thread_id,
                call_stack_hash: e.call_stack_hash,
                module_path: e.module_path.clone(),
                stack_ptr: None,
            }
        })
        .collect();
    let mut all_for_relation: Vec<ActiveAllocation> = Vec::new();
    all_for_relation.extend(heap_owner_allocations.clone());
    all_for_relation.extend(container_allocations.clone());
    // Step 4: Run container detection with appropriate config
    debug!("Running container detection");
    let container_config = crate::analysis::relation_inference::ContainerConfig {
        time_window_ns: 10_000_000, // 10ms (more permissive for test scenarios)
        size_ratio: 10000,          // Allow very large ratio for containers with small size
        lookahead: 10,              // Look at more candidates
    };
    let container_edges = detect_containers(&all_for_relation, Some(container_config));
    debug!(
        container_edges = container_edges.len(),
        "Container detection completed"
    );
    // Step 5: Run RelationGraphBuilder for HeapOwner edges (Owns, Shares, Clone)
    debug!("Running RelationGraphBuilder");
    let relation_graph = RelationGraphBuilder::build(&heap_owner_allocations, None);
    debug!(
        edges = relation_graph.edges.len(),
        "RelationGraphBuilder completed"
    );
    // Step 6: Add Container nodes to graph
    // Use virtual pointer range for Container IDs to avoid collisions with HeapOwner IDs
    const CONTAINER_PTR_BASE: usize = 0x300000000;
    let heap_owner_count = graph.nodes.len();
    for (idx, (e, type_name, _var_name)) in container_events.iter().enumerate() {
        let node_id = NodeId::from_ptr(CONTAINER_PTR_BASE + idx);
        graph.nodes.push(crate::analysis::ownership_graph::Node {
            id: node_id,
            type_name: type_name.clone(),
            size: e.size,
            stack_ptr: None,
        });
    }
    // Step 7: Add HeapOwner edges (Owns, Shares, Clone)
    for edge in &relation_graph.edges {
        let from_id = graph.nodes[edge.from].id;
        let to_id = graph.nodes[edge.to].id;
        let edge_kind = match edge.relation {
            Relation::Owns => EdgeKind::Owns,
            Relation::Contains => EdgeKind::Contains,
            Relation::Slice => EdgeKind::Borrows,
            Relation::Clone => EdgeKind::RcClone,
            Relation::Shares => EdgeKind::ArcClone,
            Relation::Evolution => EdgeKind::Contains,
            Relation::ArcClone => EdgeKind::ArcClone,
            Relation::RcClone => EdgeKind::RcClone,
            Relation::ImmutableBorrow => EdgeKind::SharedBorrow,
            Relation::MutableBorrow => EdgeKind::MutBorrow,
        };
        graph.edges.push(crate::analysis::ownership_graph::Edge {
            from: from_id,
            to: to_id,
            op: edge_kind,
        });
    }
    // Step 8: Add Container → HeapOwner edges (Contains)
    for edge in &container_edges {
        // edge.from is Container index in all_for_relation
        // edge.to is HeapOwner index in all_for_relation
        let from_all_idx = edge.from;
        let to_all_idx = edge.to;
        // Validate indices
        if from_all_idx < heap_owner_count || to_all_idx >= heap_owner_count {
            continue; // Invalid edge indices
        }
        // Convert from container index to graph node index
        let container_graph_idx = from_all_idx - heap_owner_count;
        if container_graph_idx >= container_events.len() {
            continue; // Invalid container index
        }
        let from_id = graph.nodes[heap_owner_count + container_graph_idx].id;
        let to_id = graph.nodes[to_all_idx].id;
        graph.edges.push(crate::analysis::ownership_graph::Edge {
            from: from_id,
            to: to_id,
            op: EdgeKind::Contains,
        });
    }
    debug!(
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        "Final ownership graph built"
    );
    graph
}
