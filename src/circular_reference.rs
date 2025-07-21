//! Circular reference detection for smart pointers
//!
//! This module provides functionality to detect circular references in Rc/Arc
//! smart pointers that can lead to memory leaks.

use crate::types::{AllocationInfo, SmartPointerInfo, SmartPointerType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a detected circular reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircularReference {
    /// The cycle path showing the circular reference chain
    pub cycle_path: Vec<CircularReferenceNode>,

    /// Suggested positions where Weak references should be used to break the cycle
    pub suggested_weak_positions: Vec<usize>,

    /// Estimated memory that would be leaked due to this cycle
    pub estimated_leaked_memory: usize,

    /// Severity level of this circular reference
    pub severity: CircularReferenceSeverity,

    /// Type of circular reference detected
    pub cycle_type: CircularReferenceType,
}

/// Node in a circular reference path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircularReferenceNode {
    /// Pointer address of this node
    pub ptr: usize,

    /// Data pointer this node points to
    pub data_ptr: usize,

    /// Variable name if available
    pub var_name: Option<String>,

    /// Type name of the smart pointer
    pub type_name: Option<String>,

    /// Smart pointer type
    pub pointer_type: SmartPointerType,

    /// Current reference count
    pub ref_count: usize,
}

/// Severity levels for circular references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircularReferenceSeverity {
    /// Low severity - small memory impact
    Low,
    /// Medium severity - moderate memory impact
    Medium,
    /// High severity - significant memory impact
    High,
    /// Critical severity - large memory impact or complex cycle
    Critical,
}

/// Types of circular references
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircularReferenceType {
    /// Simple two-node cycle (A -> B -> A)
    Simple,
    /// Complex multi-node cycle
    Complex,
    /// Self-referencing cycle (A -> A)
    SelfReference,
    /// Nested cycles (cycles within cycles)
    Nested,
}

/// Analysis result for circular reference detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularReferenceAnalysis {
    /// All detected circular references
    pub circular_references: Vec<CircularReference>,

    /// Total number of smart pointers analyzed
    pub total_smart_pointers: usize,

    /// Number of smart pointers involved in cycles
    pub pointers_in_cycles: usize,

    /// Total estimated leaked memory
    pub total_leaked_memory: usize,

    /// Analysis statistics
    pub statistics: CircularReferenceStatistics,
}

/// Statistics for circular reference analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularReferenceStatistics {
    /// Count by severity
    pub by_severity: HashMap<String, usize>,

    /// Count by type
    pub by_type: HashMap<String, usize>,

    /// Count by smart pointer type
    pub by_pointer_type: HashMap<String, usize>,

    /// Average cycle length
    pub average_cycle_length: f64,

    /// Largest cycle size
    pub largest_cycle_size: usize,
}

/// Graph representation for circular reference detection
#[derive(Debug)]
struct ReferenceGraph {
    /// Adjacency list: ptr -> list of data_ptrs it references
    adjacency: HashMap<usize, Vec<usize>>,

    /// Reverse mapping: data_ptr -> list of ptrs that reference it
    reverse_refs: HashMap<usize, Vec<usize>>,

    /// Smart pointer information by ptr
    smart_pointers: HashMap<usize, SmartPointerInfo>,

    /// Allocation information by ptr
    allocations: HashMap<usize, AllocationInfo>,
}

impl ReferenceGraph {
    /// Create a new reference graph from allocations
    fn new(allocations: &[AllocationInfo]) -> Self {
        let mut graph = ReferenceGraph {
            adjacency: HashMap::new(),
            reverse_refs: HashMap::new(),
            smart_pointers: HashMap::new(),
            allocations: HashMap::new(),
        };

        // Build the graph from smart pointer allocations
        for allocation in allocations {
            if let Some(ref smart_info) = allocation.smart_pointer_info {
                // Skip weak references for cycle detection (they don't create strong cycles)
                if smart_info.is_weak_reference {
                    continue;
                }

                graph
                    .smart_pointers
                    .insert(allocation.ptr, smart_info.clone());
                graph.allocations.insert(allocation.ptr, allocation.clone());

                // Add edge from this pointer to its data
                graph
                    .adjacency
                    .entry(allocation.ptr)
                    .or_insert_with(Vec::new);

                // Add reverse reference from data to this pointer
                graph
                    .reverse_refs
                    .entry(smart_info.data_ptr)
                    .or_insert_with(Vec::new)
                    .push(allocation.ptr);

                // Add edges to cloned pointers (they share the same data)
                for &clone_ptr in &smart_info.clones {
                    graph
                        .adjacency
                        .entry(allocation.ptr)
                        .or_insert_with(Vec::new)
                        .push(clone_ptr);
                }

                // Add edge from cloned_from if it exists
                if let Some(source_ptr) = smart_info.cloned_from {
                    graph
                        .adjacency
                        .entry(source_ptr)
                        .or_insert_with(Vec::new)
                        .push(allocation.ptr);
                }
            }
        }

        graph
    }

    /// Detect all circular references in the graph
    fn detect_cycles(&self) -> Vec<Vec<usize>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &ptr in self.smart_pointers.keys() {
            if !visited.contains(&ptr) {
                self.dfs_detect_cycles(ptr, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    /// Depth-first search to detect cycles
    fn dfs_detect_cycles(
        &self,
        ptr: usize,
        visited: &mut HashSet<usize>,
        rec_stack: &mut HashSet<usize>,
        path: &mut Vec<usize>,
        cycles: &mut Vec<Vec<usize>>,
    ) {
        visited.insert(ptr);
        rec_stack.insert(ptr);
        path.push(ptr);

        if let Some(neighbors) = self.adjacency.get(&ptr) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_detect_cycles(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&neighbor) {
                    // Found a cycle - extract the cycle path
                    if let Some(cycle_start) = path.iter().position(|&x| x == neighbor) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&ptr);
    }
}

/// Detect circular references in smart pointer allocations
pub fn detect_circular_references(allocations: &[AllocationInfo]) -> CircularReferenceAnalysis {
    let graph = ReferenceGraph::new(allocations);
    let raw_cycles = graph.detect_cycles();

    let mut circular_references = Vec::new();
    let mut total_leaked_memory = 0;
    let mut pointers_in_cycles = HashSet::new();

    // Process each detected cycle
    for cycle_path in raw_cycles {
        if cycle_path.len() < 2 {
            continue; // Skip trivial cycles
        }

        let circular_ref = analyze_cycle(&cycle_path, &graph);
        total_leaked_memory += circular_ref.estimated_leaked_memory;

        for node in &circular_ref.cycle_path {
            pointers_in_cycles.insert(node.ptr);
        }

        circular_references.push(circular_ref);
    }

    // Generate statistics
    let statistics = generate_statistics(&circular_references);

    CircularReferenceAnalysis {
        circular_references,
        total_smart_pointers: graph.smart_pointers.len(),
        pointers_in_cycles: pointers_in_cycles.len(),
        total_leaked_memory,
        statistics,
    }
}

/// Analyze a single cycle to create a CircularReference
fn analyze_cycle(cycle_path: &[usize], graph: &ReferenceGraph) -> CircularReference {
    let mut nodes = Vec::new();
    let mut total_memory = 0;

    // Build cycle nodes
    for &ptr in cycle_path {
        if let (Some(smart_info), Some(allocation)) =
            (graph.smart_pointers.get(&ptr), graph.allocations.get(&ptr))
        {
            let node = CircularReferenceNode {
                ptr,
                data_ptr: smart_info.data_ptr,
                var_name: allocation.var_name.clone(),
                type_name: allocation.type_name.clone(),
                pointer_type: smart_info.pointer_type.clone(),
                ref_count: smart_info
                    .latest_ref_counts()
                    .map(|snapshot| snapshot.strong_count)
                    .unwrap_or(1),
            };

            total_memory += allocation.size;
            nodes.push(node);
        }
    }

    // Determine cycle type
    let cycle_type = if cycle_path.len() == 1 {
        CircularReferenceType::SelfReference
    } else if cycle_path.len() == 2 {
        CircularReferenceType::Simple
    } else {
        CircularReferenceType::Complex
    };

    // Determine severity based on memory impact and cycle complexity
    let severity = if total_memory > 1024 * 1024 {
        // > 1MB
        CircularReferenceSeverity::Critical
    } else if total_memory > 64 * 1024 {
        // > 64KB
        CircularReferenceSeverity::High
    } else if total_memory > 4 * 1024 {
        // > 4KB
        CircularReferenceSeverity::Medium
    } else {
        CircularReferenceSeverity::Low
    };

    // Suggest weak reference positions (break the cycle at the longest-lived reference)
    let suggested_weak_positions = suggest_weak_positions(&nodes);

    CircularReference {
        cycle_path: nodes,
        suggested_weak_positions,
        estimated_leaked_memory: total_memory,
        severity,
        cycle_type,
    }
}

/// Suggest positions where weak references should be used to break cycles
fn suggest_weak_positions(nodes: &[CircularReferenceNode]) -> Vec<usize> {
    // Simple heuristic: suggest breaking at the node with the highest reference count
    // (likely to be the most shared and least critical to make weak)
    if let Some((index, _)) = nodes
        .iter()
        .enumerate()
        .max_by_key(|(_, node)| node.ref_count)
    {
        vec![index]
    } else {
        vec![0] // Fallback to first position
    }
}

/// Generate statistics for the analysis
fn generate_statistics(circular_references: &[CircularReference]) -> CircularReferenceStatistics {
    let mut by_severity = HashMap::new();
    let mut by_type = HashMap::new();
    let mut by_pointer_type = HashMap::new();
    let mut total_cycle_length = 0;
    let mut largest_cycle_size = 0;

    for circular_ref in circular_references {
        // Count by severity
        let severity_key = format!("{:?}", circular_ref.severity);
        *by_severity.entry(severity_key).or_insert(0) += 1;

        // Count by type
        let type_key = format!("{:?}", circular_ref.cycle_type);
        *by_type.entry(type_key).or_insert(0) += 1;

        // Count by pointer type
        for node in &circular_ref.cycle_path {
            let pointer_type_key = format!("{:?}", node.pointer_type);
            *by_pointer_type.entry(pointer_type_key).or_insert(0) += 1;
        }

        // Track cycle sizes
        let cycle_length = circular_ref.cycle_path.len();
        total_cycle_length += cycle_length;
        largest_cycle_size = largest_cycle_size.max(cycle_length);
    }

    let average_cycle_length = if circular_references.is_empty() {
        0.0
    } else {
        total_cycle_length as f64 / circular_references.len() as f64
    };

    CircularReferenceStatistics {
        by_severity,
        by_type,
        by_pointer_type,
        average_cycle_length,
        largest_cycle_size,
    }
}
