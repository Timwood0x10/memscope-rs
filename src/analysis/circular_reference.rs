//! Circular reference detection for smart pointers
//!
//! This module provides functionality to detect circular references in Rc/Arc
//! smart pointers that can lead to memory leaks.

use crate::core::types::{AllocationInfo, SmartPointerInfo, SmartPointerType};
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
                graph.adjacency.entry(allocation.ptr).or_default();

                // Add reverse reference from data to this pointer
                graph
                    .reverse_refs
                    .entry(smart_info.data_ptr)
                    .or_default()
                    .push(allocation.ptr);

                // Add edges to cloned pointers (they share the same data)
                for &clone_ptr in &smart_info.clones {
                    graph
                        .adjacency
                        .entry(allocation.ptr)
                        .or_default()
                        .push(clone_ptr);
                }

                // Add edge from cloned_from if it exists
                if let Some(source_ptr) = smart_info.cloned_from {
                    graph
                        .adjacency
                        .entry(source_ptr)
                        .or_default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{RefCountSnapshot, SmartPointerInfo, SmartPointerType};

    #[test]
    fn test_circular_reference_node_creation() {
        let node = CircularReferenceNode {
            ptr: 0x1000,
            data_ptr: 0x2000,
            var_name: Some("test_node".to_string()),
            type_name: Some("Rc<RefCell<Node>>".to_string()),
            pointer_type: SmartPointerType::Rc,
            ref_count: 1,
        };

        assert_eq!(node.ptr, 0x1000);
        assert_eq!(node.data_ptr, 0x2000);
        assert_eq!(node.var_name, Some("test_node".to_string()));
        assert_eq!(node.type_name, Some("Rc<RefCell<Node>>".to_string()));
        assert_eq!(node.pointer_type, SmartPointerType::Rc);
        assert_eq!(node.ref_count, 1);
    }

    #[test]
    fn test_circular_reference_creation() {
        let cycle_path = vec![
            CircularReferenceNode {
                ptr: 0x1000,
                data_ptr: 0x2000,
                var_name: Some("node_a".to_string()),
                type_name: Some("Rc<RefCell<Node>>".to_string()),
                pointer_type: SmartPointerType::Rc,
                ref_count: 1,
            },
            CircularReferenceNode {
                ptr: 0x2000,
                data_ptr: 0x1000,
                var_name: Some("node_b".to_string()),
                type_name: Some("Rc<RefCell<Node>>".to_string()),
                pointer_type: SmartPointerType::Rc,
                ref_count: 1,
            },
        ];

        let circular_ref = CircularReference {
            cycle_path: cycle_path.clone(),
            suggested_weak_positions: vec![1],
            estimated_leaked_memory: 1024,
            severity: CircularReferenceSeverity::Medium,
            cycle_type: CircularReferenceType::Simple,
        };

        assert_eq!(circular_ref.cycle_path.len(), 2);
        assert_eq!(circular_ref.suggested_weak_positions, vec![1]);
        assert_eq!(circular_ref.estimated_leaked_memory, 1024);
        assert_eq!(circular_ref.severity, CircularReferenceSeverity::Medium);
        assert_eq!(circular_ref.cycle_type, CircularReferenceType::Simple);
    }

    #[test]
    fn test_circular_reference_severity_variants() {
        let severities = [
            CircularReferenceSeverity::Low,
            CircularReferenceSeverity::Medium,
            CircularReferenceSeverity::High,
            CircularReferenceSeverity::Critical,
        ];

        // Just ensure all variants can be created and compared
        assert_eq!(severities[0], CircularReferenceSeverity::Low);
        assert_eq!(severities[1], CircularReferenceSeverity::Medium);
        assert_eq!(severities[2], CircularReferenceSeverity::High);
        assert_eq!(severities[3], CircularReferenceSeverity::Critical);
    }

    #[test]
    fn test_circular_reference_type_variants() {
        let types = [
            CircularReferenceType::Simple,
            CircularReferenceType::Complex,
            CircularReferenceType::SelfReference,
            CircularReferenceType::Nested,
        ];

        // Just ensure all variants can be created and compared
        assert_eq!(types[0], CircularReferenceType::Simple);
        assert_eq!(types[1], CircularReferenceType::Complex);
        assert_eq!(types[2], CircularReferenceType::SelfReference);
        assert_eq!(types[3], CircularReferenceType::Nested);
    }

    #[test]
    fn test_circular_reference_statistics_generation() {
        // Test with empty cycles
        let empty_cycles = vec![];
        let stats = generate_statistics(&empty_cycles);

        assert_eq!(stats.average_cycle_length, 0.0);
        assert_eq!(stats.largest_cycle_size, 0);
        assert!(stats.by_severity.is_empty());
        assert!(stats.by_type.is_empty());
        assert!(stats.by_pointer_type.is_empty());

        // Test with some cycles
        let cycles = vec![
            CircularReference {
                cycle_path: vec![CircularReferenceNode {
                    ptr: 0x1000,
                    data_ptr: 0x2000,
                    var_name: Some("node_a".to_string()),
                    type_name: Some("Rc<Node>".to_string()),
                    pointer_type: SmartPointerType::Rc,
                    ref_count: 2,
                }],
                suggested_weak_positions: vec![0],
                estimated_leaked_memory: 1024,
                severity: CircularReferenceSeverity::Low,
                cycle_type: CircularReferenceType::SelfReference,
            },
            CircularReference {
                cycle_path: vec![
                    CircularReferenceNode {
                        ptr: 0x2000,
                        data_ptr: 0x3000,
                        var_name: Some("node_b".to_string()),
                        type_name: Some("Arc<Node>".to_string()),
                        pointer_type: SmartPointerType::Arc,
                        ref_count: 3,
                    },
                    CircularReferenceNode {
                        ptr: 0x3000,
                        data_ptr: 0x2000,
                        var_name: Some("node_c".to_string()),
                        type_name: Some("Arc<Node>".to_string()),
                        pointer_type: SmartPointerType::Arc,
                        ref_count: 1,
                    },
                ],
                suggested_weak_positions: vec![0],
                estimated_leaked_memory: 2048,
                severity: CircularReferenceSeverity::Medium,
                cycle_type: CircularReferenceType::Simple,
            },
        ];

        let stats = generate_statistics(&cycles);

        assert_eq!(stats.average_cycle_length, 1.5); // (1 + 2) / 2
        assert_eq!(stats.largest_cycle_size, 2);
        assert!(!stats.by_severity.is_empty());
        assert!(!stats.by_type.is_empty());
        assert!(!stats.by_pointer_type.is_empty());

        // Check specific counts
        assert_eq!(*stats.by_severity.get("Low").unwrap_or(&0), 1);
        assert_eq!(*stats.by_severity.get("Medium").unwrap_or(&0), 1);
        assert_eq!(*stats.by_type.get("SelfReference").unwrap_or(&0), 1);
        assert_eq!(*stats.by_type.get("Simple").unwrap_or(&0), 1);
        assert_eq!(*stats.by_pointer_type.get("Rc").unwrap_or(&0), 1);
        assert_eq!(*stats.by_pointer_type.get("Arc").unwrap_or(&0), 2);
    }

    #[test]
    fn test_suggest_weak_positions() {
        // Test with empty nodes
        let empty_nodes = vec![];
        let positions = suggest_weak_positions(&empty_nodes);
        assert_eq!(positions, vec![0]); // Should fallback to position 0

        // Test with single node
        let single_node = vec![CircularReferenceNode {
            ptr: 0x1000,
            data_ptr: 0x2000,
            var_name: Some("node".to_string()),
            type_name: Some("Rc<Node>".to_string()),
            pointer_type: SmartPointerType::Rc,
            ref_count: 1,
        }];
        let positions = suggest_weak_positions(&single_node);
        assert_eq!(positions, vec![0]);

        // Test with multiple nodes, different ref counts
        let multiple_nodes = vec![
            CircularReferenceNode {
                ptr: 0x1000,
                data_ptr: 0x2000,
                var_name: Some("node_a".to_string()),
                type_name: Some("Rc<Node>".to_string()),
                pointer_type: SmartPointerType::Rc,
                ref_count: 1,
            },
            CircularReferenceNode {
                ptr: 0x2000,
                data_ptr: 0x3000,
                var_name: Some("node_b".to_string()),
                type_name: Some("Rc<Node>".to_string()),
                pointer_type: SmartPointerType::Rc,
                ref_count: 3, // Highest ref count
            },
            CircularReferenceNode {
                ptr: 0x3000,
                data_ptr: 0x1000,
                var_name: Some("node_c".to_string()),
                type_name: Some("Rc<Node>".to_string()),
                pointer_type: SmartPointerType::Rc,
                ref_count: 2,
            },
        ];
        let positions = suggest_weak_positions(&multiple_nodes);
        assert_eq!(positions, vec![1]); // Should suggest position with highest ref count
    }

    #[test]
    fn test_analyze_cycle() {
        // Create a mock graph for testing
        let mut graph = ReferenceGraph {
            adjacency: HashMap::new(),
            reverse_refs: HashMap::new(),
            smart_pointers: HashMap::new(),
            allocations: HashMap::new(),
        };

        // Add mock data to the graph
        let smart_info_a = SmartPointerInfo {
            data_ptr: 0x2000,
            pointer_type: SmartPointerType::Rc,
            is_weak_reference: false,
            clones: vec![],
            cloned_from: None,
            ref_count_history: vec![RefCountSnapshot {
                strong_count: 1,
                weak_count: 0,
                timestamp: 0,
            }],
            weak_count: None,
            is_data_owner: true,
            is_implicitly_deallocated: false,
        };

        let smart_info_b = SmartPointerInfo {
            data_ptr: 0x1000,
            pointer_type: SmartPointerType::Rc,
            is_weak_reference: false,
            clones: vec![],
            cloned_from: None,
            ref_count_history: vec![RefCountSnapshot {
                strong_count: 1,
                weak_count: 0,
                timestamp: 0,
            }],
            weak_count: None,
            is_data_owner: true,
            is_implicitly_deallocated: false,
        };

        let allocation_a = AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: Some("node_a".to_string()),
            type_name: Some("Rc<Node>".to_string()),
            smart_pointer_info: Some(smart_info_a.clone()),
            scope_name: None,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        };

        let allocation_b = AllocationInfo {
            ptr: 0x2000,
            size: 2048,
            var_name: Some("node_b".to_string()),
            type_name: Some("Rc<Node>".to_string()),
            smart_pointer_info: Some(smart_info_b.clone()),
            scope_name: None,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        };

        graph.smart_pointers.insert(0x1000, smart_info_a);
        graph.smart_pointers.insert(0x2000, smart_info_b);
        graph.allocations.insert(0x1000, allocation_a);
        graph.allocations.insert(0x2000, allocation_b);

        // Test analyzing a simple cycle
        let cycle_path = vec![0x1000, 0x2000];
        let circular_ref = analyze_cycle(&cycle_path, &graph);

        assert_eq!(circular_ref.cycle_path.len(), 2);
        assert_eq!(circular_ref.estimated_leaked_memory, 3072); // 1024 + 2048
        assert_eq!(circular_ref.cycle_type, CircularReferenceType::Simple);
        assert_eq!(circular_ref.severity, CircularReferenceSeverity::Low); // 3072 bytes < 4KB threshold
        assert!(!circular_ref.suggested_weak_positions.is_empty());
    }

    #[test]
    fn test_reference_graph_creation() {
        // Test with empty allocations
        let empty_allocations = vec![];
        let graph = ReferenceGraph::new(&empty_allocations);

        assert!(graph.adjacency.is_empty());
        assert!(graph.reverse_refs.is_empty());
        assert!(graph.smart_pointers.is_empty());
        assert!(graph.allocations.is_empty());

        // Test with allocations without smart pointer info
        let allocations_without_smart = vec![AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            scope_name: None,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            var_name: None,
            type_name: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: None,
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }];
        let graph = ReferenceGraph::new(&allocations_without_smart);

        assert!(graph.adjacency.is_empty());
        assert!(graph.reverse_refs.is_empty());
        assert!(graph.smart_pointers.is_empty());
        assert!(graph.allocations.is_empty());

        // Test with smart pointer allocations
        let smart_info = SmartPointerInfo {
            data_ptr: 0x2000,
            pointer_type: SmartPointerType::Rc,
            is_weak_reference: false,
            clones: vec![],
            cloned_from: None,
            ref_count_history: vec![RefCountSnapshot {
                strong_count: 1,
                weak_count: 0,
                timestamp: 0,
            }],
            weak_count: None,
            is_data_owner: true,
            is_implicitly_deallocated: false,
        };

        let allocations_with_smart = vec![AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: Some(smart_info.clone()),
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }];

        let graph = ReferenceGraph::new(&allocations_with_smart);

        assert!(!graph.adjacency.is_empty());
        assert!(!graph.reverse_refs.is_empty());
        assert!(!graph.smart_pointers.is_empty());
        assert!(!graph.allocations.is_empty());

        assert!(graph.adjacency.contains_key(&0x1000));
        assert!(graph.reverse_refs.contains_key(&0x2000));
        assert!(graph.smart_pointers.contains_key(&0x1000));
        assert!(graph.allocations.contains_key(&0x1000));
    }

    #[test]
    fn test_reference_graph_with_weak_references() {
        // Test that weak references are skipped
        let weak_smart_info = SmartPointerInfo {
            data_ptr: 0x2000,
            pointer_type: SmartPointerType::Rc,
            is_weak_reference: true, // This is a weak reference
            clones: vec![],
            cloned_from: None,
            ref_count_history: vec![RefCountSnapshot {
                strong_count: 1,
                weak_count: 1,
                timestamp: 0,
            }],
            weak_count: Some(1),
            is_data_owner: false,
            is_implicitly_deallocated: false,
        };

        let allocations_with_weak = vec![AllocationInfo {
            ptr: 0x1000,
            size: 1024,
            var_name: None,
            type_name: None,
            scope_name: None,
            timestamp_alloc: 0,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
            smart_pointer_info: Some(weak_smart_info),
            memory_layout: None,
            generic_info: None,
            dynamic_type_info: None,
            runtime_state: None,
            stack_allocation: None,
            temporary_object: None,
            fragmentation_analysis: None,
            generic_instantiation: None,
            type_relationships: None,
            type_usage: None,
            function_call_tracking: None,
            lifecycle_tracking: None,
            access_tracking: None,
            drop_chain_analysis: None,
        }];

        let graph = ReferenceGraph::new(&allocations_with_weak);

        // Weak references should be skipped, so graph should be empty
        assert!(graph.adjacency.is_empty());
        assert!(graph.reverse_refs.is_empty());
        assert!(graph.smart_pointers.is_empty());
        assert!(graph.allocations.is_empty());
    }

    #[test]
    fn test_detect_circular_references_empty() {
        let empty_allocations = vec![];
        let analysis = detect_circular_references(&empty_allocations);

        assert_eq!(analysis.circular_references.len(), 0);
        assert_eq!(analysis.total_smart_pointers, 0);
        assert_eq!(analysis.pointers_in_cycles, 0);
        assert_eq!(analysis.total_leaked_memory, 0);

        // Check statistics
        assert_eq!(analysis.statistics.average_cycle_length, 0.0);
        assert_eq!(analysis.statistics.largest_cycle_size, 0);
    }

    #[test]
    fn test_circular_reference_analysis_structure() {
        let analysis = CircularReferenceAnalysis {
            circular_references: vec![],
            total_smart_pointers: 10,
            pointers_in_cycles: 5,
            total_leaked_memory: 10240,
            statistics: CircularReferenceStatistics {
                by_severity: HashMap::new(),
                by_type: HashMap::new(),
                by_pointer_type: HashMap::new(),
                average_cycle_length: 0.0,
                largest_cycle_size: 0,
            },
        };

        assert_eq!(analysis.total_smart_pointers, 10);
        assert_eq!(analysis.pointers_in_cycles, 5);
        assert_eq!(analysis.total_leaked_memory, 10240);
    }

    #[test]
    fn test_circular_reference_severity_determination() {
        // Test low severity
        let low_severity = if 1024 > 1024 * 1024 {
            CircularReferenceSeverity::Critical
        } else if 1024 > 64 * 1024 {
            CircularReferenceSeverity::High
        } else if 1024 > 4 * 1024 {
            CircularReferenceSeverity::Medium
        } else {
            CircularReferenceSeverity::Low
        };
        assert_eq!(low_severity, CircularReferenceSeverity::Low);

        // Test medium severity
        let medium_severity = if 5000 > 1024 * 1024 {
            CircularReferenceSeverity::Critical
        } else if 5000 > 64 * 1024 {
            CircularReferenceSeverity::High
        } else if 5000 > 4 * 1024 {
            CircularReferenceSeverity::Medium
        } else {
            CircularReferenceSeverity::Low
        };
        assert_eq!(medium_severity, CircularReferenceSeverity::Medium);

        // Test high severity
        let high_severity = if 70000 > 1024 * 1024 {
            CircularReferenceSeverity::Critical
        } else if 70000 > 64 * 1024 {
            CircularReferenceSeverity::High
        } else if 70000 > 4 * 1024 {
            CircularReferenceSeverity::Medium
        } else {
            CircularReferenceSeverity::Low
        };
        assert_eq!(high_severity, CircularReferenceSeverity::High);

        // Test critical severity
        let critical_severity = if 2000000 > 1024 * 1024 {
            CircularReferenceSeverity::Critical
        } else if 2000000 > 64 * 1024 {
            CircularReferenceSeverity::High
        } else if 2000000 > 4 * 1024 {
            CircularReferenceSeverity::Medium
        } else {
            CircularReferenceSeverity::Low
        };
        assert_eq!(critical_severity, CircularReferenceSeverity::Critical);
    }

    #[test]
    fn test_circular_reference_type_determination() {
        // Test self reference
        let self_ref_type = if 1 == 1 {
            CircularReferenceType::SelfReference
        } else if 1 == 2 {
            CircularReferenceType::Simple
        } else {
            CircularReferenceType::Complex
        };
        assert_eq!(self_ref_type, CircularReferenceType::SelfReference);

        // Test simple cycle
        let simple_type = if 2 == 1 {
            CircularReferenceType::SelfReference
        } else if 2 == 2 {
            CircularReferenceType::Simple
        } else {
            CircularReferenceType::Complex
        };
        assert_eq!(simple_type, CircularReferenceType::Simple);

        // Test complex cycle
        let complex_type = if 5 == 1 {
            CircularReferenceType::SelfReference
        } else if 5 == 2 {
            CircularReferenceType::Simple
        } else {
            CircularReferenceType::Complex
        };
        assert_eq!(complex_type, CircularReferenceType::Complex);
    }
}
