//! Ownership Graph Engine
//!
//! This module provides a post-analysis engine for revealing Rust ownership propagation.
//! It consumes Memory Passport events to build ownership graphs and detect issues.
//!
//! Design principles:
//! - Zero runtime cost (post-analysis only)
//! - Minimal data model (only track critical operations)
//! - Focus on practical problems (Rc cycles, Arc clone storms)
//!
//! # Architecture
//!
//! ```text
//! Runtime Tracking
//!         │
//!         │ Allocation Tracker
//!         │
//!         │ Memory Passport Tracker
//!         │
//!         │ Passport.events
//!         ▼
//! Ownership Graph Engine
//!         │
//!         ▼
//! Dashboard Visualization
//! ```
//!
//! # Key Features
//!
//! - Rc/Arc Cycle Detection
//! - Arc Clone Storm Detection
//! - Ownership Chain Compression
//! - Root Cause Diagnostics

use serde::{Deserialize, Serialize};

use super::node_id::NodeId;

/// Object identifier - unified with NodeId.
///
/// This type alias provides backward compatibility.
/// Previously, ObjectId was a separate struct that wrapped u64.
/// Now it is unified with NodeId to avoid duplication.
/// Use NodeId directly for new code.
pub type ObjectId = NodeId;

/// Ownership operation types - only track critical operations
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipOp {
    /// Object creation
    Create,
    /// Object deallocation
    Drop,
    /// Rc clone operation
    RcClone,
    /// Arc clone operation
    ArcClone,
}

/// Ownership event recorded in passport
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnershipEvent {
    /// Timestamp
    pub ts: u64,
    /// Operation type
    pub op: OwnershipOp,
    /// Source object
    pub src: ObjectId,
    /// Destination object (optional)
    pub dst: Option<ObjectId>,
}

impl OwnershipEvent {
    /// Create a new ownership event
    pub fn new(ts: u64, op: OwnershipOp, src: ObjectId, dst: Option<ObjectId>) -> Self {
        Self { ts, op, src, dst }
    }
}

/// Node in ownership graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node identifier
    pub id: ObjectId,
    /// Type name
    pub type_name: String,
    /// Size in bytes
    pub size: usize,
}

/// Edge kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Owner relationship (A contains pointer to B)
    Owns,
    /// Contains relationship (Container → HeapOwner, e.g., HashMap → Vec)
    Contains,
    /// Borrow relationship (A borrows from B)
    Borrows,
    /// Rc clone edge
    RcClone,
    /// Arc clone edge
    ArcClone,
}

/// Edge in ownership graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// From object
    pub from: ObjectId,
    /// To object
    pub to: ObjectId,
    /// Edge kind
    pub op: EdgeKind,
}

/// Ownership graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipGraph {
    /// All nodes in the graph
    pub nodes: Vec<Node>,
    /// All edges in the graph
    pub edges: Vec<Edge>,
    /// Detected cycles
    pub cycles: Vec<Vec<ObjectId>>,
    /// Arc clone count for storm detection
    pub arc_clone_count: usize,
}

impl OwnershipGraph {
    /// Build ownership graph from MemoryView.
    ///
    /// Creates an ownership graph from the allocations in a MemoryView.
    /// This is a convenience method for the unified analyzer API.
    pub fn from_view(view: &crate::view::MemoryView) -> Self {
        let allocations = view.allocations();
        let passports: Vec<(ObjectId, String, usize, Vec<OwnershipEvent>)> = allocations
            .iter()
            .filter_map(|a| {
                a.ptr.map(|ptr| {
                    let id = ObjectId::from_ptr(ptr);
                    let type_name = a.type_name.clone().unwrap_or_else(|| "unknown".to_string());
                    // Create a basic create event for each allocation
                    let event = OwnershipEvent::new(a.allocated_at, OwnershipOp::Create, id, None);
                    (id, type_name, a.size, vec![event])
                })
            })
            .collect();

        Self::build(&passports)
    }

    /// Build ownership graph from passports with ownership events
    pub fn build<T: AsRef<[OwnershipEvent]>>(passports: &[(ObjectId, String, usize, T)]) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut arc_clone_count = 0;

        for (id, type_name, size, events) in passports {
            // Add node
            nodes.push(Node {
                id: *id,
                type_name: type_name.clone(),
                size: *size,
            });

            // Process events
            for event in events.as_ref() {
                match event.op {
                    OwnershipOp::RcClone => {
                        if let Some(dst) = event.dst {
                            edges.push(Edge {
                                from: event.src,
                                to: dst,
                                op: EdgeKind::RcClone,
                            });
                        }
                    }
                    OwnershipOp::ArcClone => {
                        arc_clone_count += 1;
                        if let Some(dst) = event.dst {
                            edges.push(Edge {
                                from: event.src,
                                to: dst,
                                op: EdgeKind::ArcClone,
                            });
                        }
                    }
                    OwnershipOp::Create | OwnershipOp::Drop => {
                        // These don't create edges
                    }
                }
            }
        }

        // Compress clone chains
        Self::compress_clone_chains(&mut edges);

        // Detect cycles
        let cycles = Self::detect_cycles(&edges);

        OwnershipGraph {
            nodes,
            edges,
            cycles,
            arc_clone_count,
        }
    }

    /// Compress consecutive clone chains to reduce UI complexity.
    ///
    /// Uses a new vector to collect merged edges, avoiding O(n^2) complexity from repeated remove() calls.
    fn compress_clone_chains(edges: &mut Vec<Edge>) {
        if edges.len() < 2 {
            return;
        }

        let mut result: Vec<Edge> = Vec::with_capacity(edges.len());
        let mut i = 0;

        while i < edges.len() {
            let mut current = edges[i].clone();

            // Try to merge with subsequent edges
            while i + 1 < edges.len()
                && current.op == edges[i + 1].op
                && current.to == edges[i + 1].from
            {
                // Merge: extend current edge to skip the next one
                current.to = edges[i + 1].to;
                i += 1;
            }

            result.push(current);
            i += 1;
        }

        *edges = result;
    }

    /// Detect cycles using existing DFS implementation
    fn detect_cycles(edges: &[Edge]) -> Vec<Vec<ObjectId>> {
        use crate::analysis::relationship_cycle_detector;

        if edges.is_empty() {
            return Vec::new();
        }

        // Convert edges to the format expected by the cycle detector
        let relationships: Vec<(String, String, String)> = edges
            .iter()
            .map(|e| {
                (
                    format!("0x{:x}", e.from.0),
                    format!("0x{:x}", e.to.0),
                    format!("{:?}", e.op).to_lowercase(),
                )
            })
            .collect();

        let result = relationship_cycle_detector::detect_cycles_with_indices(&relationships);

        // Convert cycle indices back to ObjectIds
        let mut cycles = Vec::new();
        let mut obj_id_map: std::collections::HashMap<String, ObjectId> =
            std::collections::HashMap::new();

        for edge in edges {
            obj_id_map.insert(format!("0x{:x}", edge.from.0), edge.from);
            obj_id_map.insert(format!("0x{:x}", edge.to.0), edge.to);
        }

        for (from_idx, to_idx) in result.cycle_edges {
            if let (Some(from_label), Some(to_label)) = (
                result.node_labels.get(from_idx),
                result.node_labels.get(to_idx),
            ) {
                if let (Some(from_id), Some(to_id)) =
                    (obj_id_map.get(from_label), obj_id_map.get(to_label))
                {
                    // Simple cycle representation
                    cycles.push(vec![*from_id, *to_id]);
                }
            }
        }

        cycles
    }

    /// Check if Arc clone storm is detected
    pub fn has_arc_clone_storm(&self, threshold: usize) -> bool {
        self.arc_clone_count > threshold
    }

    /// Get all Rc clone edges
    pub fn rc_clones(&self) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.op == EdgeKind::RcClone)
            .collect()
    }

    /// Get all Arc clone edges
    pub fn arc_clones(&self) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.op == EdgeKind::ArcClone)
            .collect()
    }

    /// Generate diagnostics report for detected issues
    pub fn diagnostics(&self, arc_storm_threshold: usize) -> OwnershipDiagnostics {
        let mut issues = Vec::new();

        // Check for Rc cycles
        for cycle in &self.cycles {
            let cycle_type = self.detect_cycle_type(cycle);
            issues.push(DiagnosticIssue::RcCycle {
                nodes: cycle.clone(),
                cycle_type,
            });
        }

        // Check for Arc clone storm
        if self.has_arc_clone_storm(arc_storm_threshold) {
            issues.push(DiagnosticIssue::ArcCloneStorm {
                clone_count: self.arc_clone_count,
                threshold: arc_storm_threshold,
            });
        }

        OwnershipDiagnostics {
            issues,
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            rc_clone_count: self.rc_clones().len(),
            arc_clone_count: self.arc_clone_count,
        }
    }

    /// Detect the type of cycle (Rc or Arc)
    fn detect_cycle_type(&self, cycle: &[ObjectId]) -> CycleType {
        // Check if any edge in the cycle is an RcClone
        for edge in &self.edges {
            if cycle.contains(&edge.from)
                && cycle.contains(&edge.to)
                && edge.op == EdgeKind::RcClone
            {
                return CycleType::Rc;
            }
        }
        CycleType::Arc
    }

    /// Find root cause chain for memory growth
    pub fn find_root_cause(&self) -> Option<RootCauseChain> {
        // Check for Arc clone storm as root cause
        if self.arc_clone_count > 50 {
            return Some(RootCauseChain {
                root_cause: RootCause::ArcCloneStorm,
                description: format!(
                    "Arc clone storm detected: {} clones causing memory proliferation",
                    self.arc_clone_count
                ),
                impact: format!(
                    "Potential memory spike from {} Arc clone operations",
                    self.arc_clone_count
                ),
            });
        }

        // Check for Rc cycles as root cause
        if !self.cycles.is_empty() {
            return Some(RootCauseChain {
                root_cause: RootCause::RcCycle,
                description: format!(
                    "Rc retain cycle detected: {} cycles found",
                    self.cycles.len()
                ),
                impact: "Memory leak due to reference count cycles".to_string(),
            });
        }

        None
    }
}

/// Type of ownership cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleType {
    /// Rc reference cycle (memory leak)
    Rc,
    /// Arc reference cycle (potential leak)
    Arc,
}

/// Diagnostic issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticIssue {
    /// Rc retain cycle detected
    RcCycle {
        nodes: Vec<ObjectId>,
        cycle_type: CycleType,
    },
    /// Arc clone storm detected
    ArcCloneStorm {
        clone_count: usize,
        threshold: usize,
    },
}

/// Diagnostics output for ownership graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipDiagnostics {
    /// Detected issues
    pub issues: Vec<DiagnosticIssue>,
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Number of Rc clone operations
    pub rc_clone_count: usize,
    /// Number of Arc clone operations
    pub arc_clone_count: usize,
}

impl OwnershipDiagnostics {
    /// Check if any issues were detected
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Get summary string for display
    pub fn summary(&self) -> String {
        let mut summary = format!(
            "Ownership Graph: {} nodes, {} edges\n",
            self.total_nodes, self.total_edges
        );
        for issue in &self.issues {
            match issue {
                DiagnosticIssue::RcCycle { nodes, cycle_type } => {
                    summary.push_str(&format!(
                        "🔴 {:?} Cycle detected: {} nodes\n",
                        cycle_type,
                        nodes.len()
                    ));
                }
                DiagnosticIssue::ArcCloneStorm {
                    clone_count,
                    threshold,
                } => {
                    summary.push_str(&format!(
                        "⚠ Arc Clone Storm: {} clones (threshold: {})\n",
                        clone_count, threshold
                    ));
                }
            }
        }
        summary
    }
}

/// Root cause of memory issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootCause {
    /// Arc clone storm causing memory proliferation
    ArcCloneStorm,
    /// Rc retain cycle causing memory leak
    RcCycle,
}

/// Root cause chain for memory growth analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseChain {
    /// Root cause type
    pub root_cause: RootCause,
    /// Description of the root cause
    pub description: String,
    /// Impact on memory
    pub impact: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_from_ptr() {
        let id = ObjectId::from_ptr(0x1000);
        assert_eq!(id.0, 0x1000);
    }

    #[test]
    fn test_ownership_event_creation() {
        let id = NodeId(0x1000);
        let event = OwnershipEvent::new(1000, OwnershipOp::Create, id, None);
        assert_eq!(event.ts, 1000);
        assert_eq!(event.op, OwnershipOp::Create);
    }

    #[test]
    fn test_graph_build_empty() {
        let passports: Vec<(NodeId, String, usize, Vec<OwnershipEvent>)> = vec![];
        let graph = OwnershipGraph::build(&passports);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.cycles.is_empty());
    }

    #[test]
    fn test_graph_build_rc_clone() {
        let id1 = NodeId(0x1000);
        let id2 = NodeId(0x2000);
        let events = vec![OwnershipEvent::new(
            1000,
            OwnershipOp::RcClone,
            id1,
            Some(id2),
        )];
        let passports = vec![(id1, "Rc<i32>".to_string(), 8, events)];

        let graph = OwnershipGraph::build(&passports);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].op, EdgeKind::RcClone);
    }

    #[test]
    fn test_graph_build_arc_clone_storm() {
        let id1 = NodeId(0x1000);
        let mut events = Vec::new();
        for i in 0..100 {
            let dst = NodeId(0x2000 + i);
            events.push(OwnershipEvent::new(
                i,
                OwnershipOp::ArcClone,
                id1,
                Some(dst),
            ));
        }
        let passports = vec![(id1, "Arc<i32>".to_string(), 8, events)];

        let graph = OwnershipGraph::build(&passports);
        assert_eq!(graph.arc_clone_count, 100);
        assert!(graph.has_arc_clone_storm(50));
    }

    #[test]
    fn test_compress_clone_chains() {
        let id1 = NodeId(0x1000);
        let id2 = NodeId(0x2000);
        let id3 = NodeId(0x3000);
        let id4 = NodeId(0x4000);

        let events = vec![
            OwnershipEvent::new(1000, OwnershipOp::ArcClone, id1, Some(id2)),
            OwnershipEvent::new(2000, OwnershipOp::ArcClone, id2, Some(id3)),
            OwnershipEvent::new(3000, OwnershipOp::ArcClone, id3, Some(id4)),
        ];
        let passports = vec![(id1, "Arc<i32>".to_string(), 8, events)];

        let graph = OwnershipGraph::build(&passports);

        // After compression, we should have only 1 edge: id1 -> id4
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, id1);
        assert_eq!(graph.edges[0].to, id4);
    }

    #[test]
    fn test_diagnostics() {
        let id1 = NodeId(0x1000);
        let mut events = Vec::new();
        for i in 0..100 {
            let dst = NodeId(0x2000 + i);
            events.push(OwnershipEvent::new(
                i,
                OwnershipOp::ArcClone,
                id1,
                Some(dst),
            ));
        }
        let passports = vec![(id1, "Arc<i32>".to_string(), 8, events)];

        let graph = OwnershipGraph::build(&passports);
        let diagnostics = graph.diagnostics(50);

        assert!(diagnostics.has_issues());
        assert!(diagnostics.summary().contains("Arc Clone Storm"));
    }

    #[test]
    fn test_root_cause_detection() {
        let id1 = NodeId(0x1000);
        let mut events = Vec::new();
        for i in 0..100 {
            let dst = NodeId(0x2000 + i);
            events.push(OwnershipEvent::new(
                i,
                OwnershipOp::ArcClone,
                id1,
                Some(dst),
            ));
        }
        let passports = vec![(id1, "Arc<i32>".to_string(), 8, events)];

        let graph = OwnershipGraph::build(&passports);
        let root_cause = graph.find_root_cause();

        assert!(root_cause.is_some());
        let chain = root_cause.unwrap();
        assert_eq!(chain.root_cause, RootCause::ArcCloneStorm);
    }
}
