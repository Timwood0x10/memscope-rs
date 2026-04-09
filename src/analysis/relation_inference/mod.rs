//! Relation Inference - Memory relationship graph construction.
//!
//! Infers relationships between heap allocations by analyzing pointer patterns,
//! content similarity, and allocation metadata.
//!
//! # Supported Relations
//!
//! | Relation | Meaning | Signal |
//! |----------|---------|--------|
//! | **Owner** | A holds a pointer into B | Pointer scan + RangeMap |
//! | **Slice** | A is a sub-region of B | Pointer falls inside B (not at start) |
//! | **Clone** | A is a copy of B | Same type/size/stack + content match |
//!
//! # Architecture
//!
//! ```text
//! ActiveAllocation + MemoryView
//!        │
//!        ▼
//!   UTI Engine (TypeGuess)
//!        │
//!        ▼
//!   RangeMap (address → alloc index)
//!        │
//!        ▼
//!   Relation Engine (Owner / Slice / Clone)
//!        │
//!        ▼
//!   RelationGraph
//! ```

mod clone_detector;
mod graph_builder;
mod pointer_scan;
mod range_map;
mod shared_detector;
mod slice_detector;

pub use clone_detector::{detect_clones, CloneConfig};
pub use graph_builder::{GraphBuilderConfig, RelationGraphBuilder};
pub use pointer_scan::{detect_owner, InferenceRecord};
pub use range_map::RangeMap;
pub use shared_detector::detect_shared;
pub use slice_detector::detect_slice;

/// A relationship between two allocations in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    /// A owns or points into B (e.g., Vec metadata → heap buffer).
    Owner,
    /// A is a view into a sub-region of B (e.g., &[T] into Vec).
    Slice,
    /// A is a copy of B (same type, size, stack, content).
    Clone,
    /// A and B share ownership of the same Arc/Rc inner data.
    Shared,
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relation::Owner => write!(f, "Owner"),
            Relation::Slice => write!(f, "Slice"),
            Relation::Clone => write!(f, "Clone"),
            Relation::Shared => write!(f, "Shared"),
        }
    }
}

/// An edge in the relation graph.
#[derive(Debug, Clone)]
pub struct RelationEdge {
    /// Source allocation ID (index into the allocations list).
    pub from: usize,
    /// Target allocation ID.
    pub to: usize,
    /// The type of relationship.
    pub relation: Relation,
}

/// A relation graph connecting allocations.
#[derive(Debug, Default, Clone)]
pub struct RelationGraph {
    /// All edges in the graph.
    pub edges: Vec<RelationEdge>,
}

impl RelationGraph {
    /// Create a new empty relation graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single edge to the graph.
    ///
    /// Does not add duplicate edges (same from/to/relation).
    pub fn add_edge(&mut self, from: usize, to: usize, relation: Relation) {
        // Skip self-edges.
        if from == to {
            return;
        }
        // Skip duplicates.
        let exists = self
            .edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.relation == relation);
        if !exists {
            self.edges.push(RelationEdge { from, to, relation });
        }
    }

    /// Add multiple edges at once.
    pub fn add_edges(&mut self, edges: Vec<RelationEdge>) {
        for edge in edges {
            self.add_edge(edge.from, edge.to, edge.relation);
        }
    }

    /// Get all inbound edges to a given node.
    pub fn get_inbound_edges(&self, node: usize) -> Vec<&RelationEdge> {
        self.edges.iter().filter(|e| e.to == node).collect()
    }

    /// Get all outbound edges from a given node.
    pub fn get_outbound_edges(&self, node: usize) -> Vec<&RelationEdge> {
        self.edges.iter().filter(|e| e.from == node).collect()
    }

    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get all unique node IDs referenced in the graph.
    pub fn all_nodes(&self) -> Vec<usize> {
        let mut nodes: Vec<usize> = self.edges.iter().flat_map(|e| [e.from, e.to]).collect();
        nodes.sort();
        nodes.dedup();
        nodes
    }

    /// Detect cycles in the relation graph using DFS.
    ///
    /// Returns a list of cycle edges as `(from, to, relation)` tuples.
    /// Empty result means no cycles detected.
    pub fn detect_cycles(&self) -> Vec<(usize, usize, Relation)> {
        if self.edges.is_empty() {
            return Vec::new();
        }

        use crate::analysis::relationship_cycle_detector::detect_cycles_direct;

        let relationships: Vec<(usize, usize)> =
            self.edges.iter().map(|e| (e.from, e.to)).collect();

        let cycle_indices = detect_cycles_direct(&relationships);

        cycle_indices
            .into_iter()
            .filter_map(|(from_idx, to_idx)| {
                self.edges.iter().find_map(|e| {
                    if e.from == from_idx && e.to == to_idx {
                        Some((e.from, e.to, e.relation.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_add_edge() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);

        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.edges[0].from, 0);
        assert_eq!(graph.edges[0].to, 1);
        assert_eq!(graph.edges[0].relation, Relation::Owner);
    }

    #[test]
    fn test_graph_no_self_edges() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 0, Relation::Owner);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_no_duplicate_edges() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(0, 1, Relation::Owner);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_graph_different_relations_allowed() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(0, 1, Relation::Clone);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_graph_inbound_outbound_edges() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 2, Relation::Owner);
        graph.add_edge(1, 2, Relation::Owner);
        graph.add_edge(2, 3, Relation::Slice);

        let inbound_to_2 = graph.get_inbound_edges(2);
        assert_eq!(inbound_to_2.len(), 2);

        let outbound_from_2 = graph.get_outbound_edges(2);
        assert_eq!(outbound_from_2.len(), 1);
    }

    #[test]
    fn test_graph_all_nodes() {
        let mut graph = RelationGraph::new();
        graph.add_edge(3, 1, Relation::Owner);
        graph.add_edge(1, 2, Relation::Slice);

        let nodes = graph.all_nodes();
        assert_eq!(nodes, vec![1, 2, 3]);
    }

    #[test]
    fn test_graph_add_edges_batch() {
        let mut graph = RelationGraph::new();
        graph.add_edges(vec![
            RelationEdge {
                from: 0,
                to: 1,
                relation: Relation::Owner,
            },
            RelationEdge {
                from: 1,
                to: 2,
                relation: Relation::Slice,
            },
        ]);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_graph_detect_cycles_none() {
        // Linear chain: 0 → 1 → 2 → no cycles.
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(1, 2, Relation::Slice);

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_graph_detect_cycles_simple() {
        // Cycle: 0 → 1 → 0
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(1, 0, Relation::Owner);

        let cycles = graph.detect_cycles();
        // Should detect at least one cycle edge.
        assert!(!cycles.is_empty());
        // Both edges should be part of the cycle.
        let edge_pairs: Vec<_> = cycles.iter().map(|(f, t, _)| (*f, *t)).collect();
        assert!(edge_pairs.contains(&(0, 1)) || edge_pairs.contains(&(1, 0)));
    }

    #[test]
    fn test_graph_detect_cycles_longer() {
        // Cycle: 0 → 1 → 2 → 0
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(1, 2, Relation::Slice);
        graph.add_edge(2, 0, Relation::Clone);

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
        // Verify the cycle includes the back-edge (2 → 0).
        let has_back_edge = cycles.iter().any(|(f, t, _)| *f == 2 && *t == 0);
        assert!(has_back_edge, "cycle should contain back-edge (2, 0)");
    }

    #[test]
    fn test_graph_detect_cycles_empty() {
        let graph = RelationGraph::new();
        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_graph_detect_cycles_self_loop_blocked() {
        // Self-edges are blocked by add_edge, so no cycle possible.
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 0, Relation::Owner);
        assert_eq!(graph.edge_count(), 0);

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_graph_detect_cycles_diamond() {
        // Diamond: 0 → 1, 0 → 2, 1 → 3, 2 → 3, 3 → 0
        // Cycles: 0→1→3→0 and 0→2→3→0
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owner);
        graph.add_edge(0, 2, Relation::Owner);
        graph.add_edge(1, 3, Relation::Slice);
        graph.add_edge(2, 3, Relation::Slice);
        graph.add_edge(3, 0, Relation::Clone);

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }
}
