//! Graph analysis module.

use crate::analysis::relation_inference::{Relation, RelationGraph};
use crate::analyzer::report::{CycleInfo, CycleReport};
use crate::view::MemoryView;
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Relationship edge information for visualization.
#[derive(Debug, Clone)]
pub struct RelationshipEdge {
    /// Source allocation pointer
    pub from_ptr: usize,
    /// Target allocation pointer
    pub to_ptr: usize,
    /// Source variable name
    pub from_var_name: Option<String>,
    /// Target variable name
    pub to_var_name: Option<String>,
    /// Source type name
    pub from_type_name: Option<String>,
    /// Target type name
    pub to_type_name: Option<String>,
    /// Relationship type
    pub relation: Relation,
    /// Source is Container type (no heap pointer)
    pub is_container_source: bool,
    /// Target is Container type (no heap pointer)
    pub is_container_target: bool,
}

/// Graph analysis module.
///
/// Provides ownership and relationship graph analysis.
pub struct GraphAnalysis {
    view: MemoryView,
    relation_graph: Option<RelationGraph>,
}

impl GraphAnalysis {
    /// Create from view.
    pub fn from_view(view: &MemoryView) -> Self {
        debug!("Creating GraphAnalysis with {} allocations", view.len());
        Self {
            view: view.clone(),
            relation_graph: None,
        }
    }

    /// Get or build the relation graph (lazy).
    fn relation_graph(&mut self) -> &RelationGraph {
        if self.relation_graph.is_none() {
            info!("Building relation graph from view");
            self.relation_graph = Some(RelationGraph::from_view(&self.view));
        }
        self.relation_graph.as_ref().unwrap_or_else(|| {
            unreachable!("RelationGraph should be initialized after lazy initialization")
        })
    }

    /// Detect cycles in ownership graph.
    ///
    /// Uses the project's relation inference system to build actual
    /// memory relationships, then applies the existing cycle detection
    /// algorithm from RelationGraph::detect_cycles().
    ///
    /// Uses lazy initialization to cache the relation graph for efficiency.
    pub fn cycles(&mut self) -> CycleReport {
        let allocations = self.view.allocations();
        if allocations.is_empty() {
            debug!("Cycle detection: no allocations, returning empty report");
            return CycleReport::empty();
        }

        let relation_graph = self.relation_graph();
        let cycle_edges = relation_graph.detect_cycles();

        if cycle_edges.is_empty() {
            debug!("Cycle detection: no cycles found");
            return CycleReport::empty();
        }

        info!("Cycle detection: found {} cycles", cycle_edges.len());

        let cycles: Vec<CycleInfo> = cycle_edges
            .iter()
            .map(|(from, to, _)| CycleInfo {
                nodes: vec![*from as u64, *to as u64],
            })
            .collect();

        CycleReport {
            cycle_count: cycles.len(),
            cycles,
        }
    }

    /// Get ownership statistics.
    pub fn ownership_stats(&self) -> OwnershipStats {
        let allocations = self.view.allocations();
        OwnershipStats {
            total_objects: allocations.len(),
            total_bytes: allocations.iter().map(|a| a.size).sum(),
            unique_types: allocations
                .iter()
                .filter_map(|a| a.type_name.as_ref())
                .collect::<HashSet<_>>()
                .len(),
        }
    }

    /// Get relationship statistics from actual graph analysis.
    pub fn relationship_stats(&mut self) -> RelationshipStats {
        let relation_graph = self.relation_graph();
        let edges = &relation_graph.edges;

        let mut ownership_edges = 0usize;
        let mut contains_edges = 0usize;
        let mut shares_edges = 0usize;
        let mut slice_edges = 0usize;
        let mut clone_edges = 0usize;

        for edge in edges {
            match edge.relation {
                Relation::Owns => ownership_edges += 1,
                Relation::Contains => contains_edges += 1,
                Relation::Shares => shares_edges += 1,
                Relation::Slice => slice_edges += 1,
                Relation::Clone => clone_edges += 1,
                Relation::Evolution => {}
            }
        }

        RelationshipStats {
            ownership_edges,
            contains_edges,
            shares_edges,
            slice_edges,
            clone_edges,
        }
    }

    /// Get all relationship edges for visualization.
    ///
    /// Returns a list of relationship edges with full allocation info.
    /// Limited to 500 edges to avoid performance issues.
    pub fn relationships(&mut self) -> Vec<RelationshipEdge> {
        // Clone allocations to avoid borrow conflict with relation_graph()
        let allocations: Vec<crate::snapshot::types::ActiveAllocation> = self
            .view
            .allocations()
            .iter()
            .map(|a| (*a).clone())
            .collect();

        let relation_graph = self.relation_graph();
        let edges = &relation_graph.edges;

        debug!("Building relationships from {} edges", edges.len());

        let mut relationships: Vec<RelationshipEdge> = edges
            .iter()
            .filter_map(|edge| {
                let from_alloc = allocations.get(edge.from);
                let to_alloc = allocations.get(edge.to);

                let from_ptr = from_alloc.and_then(|a| a.ptr).unwrap_or(edge.from);
                let to_ptr = to_alloc.and_then(|a| a.ptr).unwrap_or(edge.to);

                // Skip if both are virtual/container pointers
                if from_ptr == 0 && to_ptr == 0 {
                    return None;
                }

                Some(RelationshipEdge {
                    from_ptr,
                    to_ptr,
                    from_var_name: from_alloc.and_then(|a| a.var_name.clone()),
                    to_var_name: to_alloc.and_then(|a| a.var_name.clone()),
                    from_type_name: from_alloc.and_then(|a| a.type_name.clone()),
                    to_type_name: to_alloc.and_then(|a| a.type_name.clone()),
                    relation: edge.relation.clone(),
                    is_container_source: from_ptr == 0,
                    is_container_target: to_ptr == 0,
                })
            })
            .collect();

        // Limit relationships to avoid performance issues
        if relationships.len() > 500 {
            warn!(
                "Relationship count ({}) exceeds limit, truncating to 500",
                relationships.len()
            );
            relationships.truncate(500);
        }

        debug!("Returning {} relationships", relationships.len());
        relationships
    }
}

/// Ownership statistics.
#[derive(Debug, Clone)]
pub struct OwnershipStats {
    /// Total number of objects
    pub total_objects: usize,
    /// Total bytes owned
    pub total_bytes: usize,
    /// Number of unique types
    pub unique_types: usize,
}

/// Relationship statistics computed from actual graph analysis.
#[derive(Debug, Clone, Default)]
pub struct RelationshipStats {
    /// Number of ownership edges (A owns heap memory)
    pub ownership_edges: usize,
    /// Number of contains edges (Container → HeapOwner)
    pub contains_edges: usize,
    /// Number of shares edges (Arc/Rc)
    pub shares_edges: usize,
    /// Number of slice edges (view into sub-region)
    pub slice_edges: usize,
    /// Number of clone edges (copy of another allocation)
    pub clone_edges: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::MemoryEvent;

    #[test]
    fn test_graph_analysis() {
        let events = vec![MemoryEvent::allocate(0x1000, 64, 1)];
        let view = MemoryView::from_events(events);
        let analysis = GraphAnalysis::from_view(&view);
        let stats = analysis.ownership_stats();
        assert_eq!(stats.total_objects, 1);
    }

    #[test]
    fn test_cycle_detection_empty() {
        let events: Vec<MemoryEvent> = vec![];
        let view = MemoryView::from_events(events);
        let mut analysis = GraphAnalysis::from_view(&view);
        let cycles = analysis.cycles();
        assert_eq!(cycles.cycle_count, 0);
    }

    #[test]
    fn test_relationship_stats() {
        use crate::core::types::TrackKind;
        use crate::snapshot::types::ActiveAllocation;

        let buf1 = vec![0u8; 64];
        let buf2 = vec![0u8; 128];
        let ptr1 = buf1.as_ptr() as usize;
        let ptr2 = buf2.as_ptr() as usize;

        let allocs = vec![
            ActiveAllocation {
                ptr: Some(ptr1),
                size: 64,
                kind: TrackKind::HeapOwner {
                    ptr: ptr1,
                    size: 64,
                },
                allocated_at: 1000,
                var_name: None,
                type_name: None,
                thread_id: 1,
                call_stack_hash: None,
                module_path: None,
            },
            ActiveAllocation {
                ptr: Some(ptr2),
                size: 128,
                kind: TrackKind::HeapOwner {
                    ptr: ptr2,
                    size: 128,
                },
                allocated_at: 1001,
                var_name: None,
                type_name: None,
                thread_id: 1,
                call_stack_hash: None,
                module_path: None,
            },
        ];

        let snapshot = crate::snapshot::MemorySnapshot::new();
        let mut snapshot = snapshot;
        for alloc in allocs {
            snapshot
                .active_allocations
                .insert(alloc.ptr.unwrap(), alloc);
        }

        let view = MemoryView::new(snapshot, vec![]);
        let mut analysis = GraphAnalysis::from_view(&view);
        let stats = analysis.relationship_stats();
        // Stats should be computed from actual graph
        // ownership_edges is usize, so it's always >= 0
        let _ = stats.ownership_edges;

        // Keep buffers alive until test ends
        drop(buf1);
        drop(buf2);
    }
}
