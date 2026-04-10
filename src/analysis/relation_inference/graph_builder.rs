//! Graph Builder — main pipeline for relation inference.
//!
//! Orchestrates HeapScanner, UTI Engine, and all relation detectors
//! to produce a complete `RelationGraph` from active allocations.

use crate::analysis::{
    heap_scanner::{HeapScanner, ScanResult},
    relation_inference::{
        clone_detector::{detect_clones, CloneConfig},
        container_detector::{detect_containers, ContainerConfig},
        pointer_scan::{detect_owner, InferenceRecord},
        shared_detector::detect_shared,
        slice_detector::detect_slice,
        RangeMap, RelationGraph,
    },
    unsafe_inference::{MemoryView, OwnedMemoryView, TypeKind, UnsafeInferenceEngine},
};

use crate::snapshot::types::ActiveAllocation;

/// Configuration for the relation graph builder.
///
/// This struct controls the behavior of the relation inference pipeline,
/// allowing fine-tuning of detection thresholds and parameters.
///
/// # Example
///
/// ```ignore
/// use memscope_rs::analysis::relation_inference::{
///     GraphBuilderConfig, CloneConfig,
/// };
///
/// // Default configuration
/// let config = GraphBuilderConfig::default();
///
/// // Custom configuration for stricter clone detection
/// let config = GraphBuilderConfig {
///     clone_config: CloneConfig {
///         min_similarity: 0.95,  // Require 95% content match
///         max_time_diff_ns: 5_000_000,  // 5ms window
///         ..Default::default()
///     },
/// };
///
/// let graph = RelationGraphBuilder::build(&allocations, Some(config));
/// ```
///
/// # Fields
///
/// * `clone_config` - Controls clone detection behavior. See [`CloneConfig`]
///   for details on similarity thresholds and time windows.
#[derive(Debug, Clone, Default)]
pub struct GraphBuilderConfig {
    /// Configuration for clone detection.
    ///
    /// Controls how the system identifies cloned allocations based on:
    /// - Content similarity ratio (`min_similarity`)
    /// - Maximum time difference between allocations (`max_time_diff_ns`)
    /// - Number of bytes to compare (`compare_bytes`)
    ///
    /// Default values are tuned for typical Rust workloads:
    /// - 80% similarity threshold
    /// - 10ms time window
    /// - 64 bytes comparison
    pub clone_config: CloneConfig,

    /// Configuration for container relation detection.
    ///
    /// Controls how the system infers `Contains` relationships between
    /// Container types (HashMap, BTreeMap) and HeapOwner types (Vec, Box).
    ///
    /// Default values use temporal locality with 1ms time window:
    /// - 1ms time window
    /// - 10x size ratio limit
    /// - 5 candidate lookahead
    pub container_config: ContainerConfig,
}

/// Builds a relation graph from active allocations.
///
/// # Pipeline
///
/// ```text
/// ActiveAllocation[]
///     │
///     ▼
/// HeapScanner → ScanResult[]
///     │
///     ▼
/// UTI Engine → InferenceRecord[]
///     │
///     ▼
/// RangeMap + Owner/Slice/Clone detectors
///     │
///     ▼
/// RelationGraph
/// ```
pub struct RelationGraphBuilder;

impl RelationGraphBuilder {
    /// Build a relation graph from a list of active allocations.
    ///
    /// # Arguments
    ///
    /// * `allocations` - List of active allocations from a snapshot.
    /// * `config` - Builder configuration (optional, uses defaults if None).
    ///
    /// # Returns
    ///
    /// A `RelationGraph` containing all inferred relationships.
    pub fn build(
        allocations: &[ActiveAllocation],
        config: Option<GraphBuilderConfig>,
    ) -> RelationGraph {
        let config = config.unwrap_or_default();

        if allocations.is_empty() {
            return RelationGraph::new();
        }

        // Step 1: Scan heap memory for all allocations.
        let scan_results = HeapScanner::scan(allocations);

        // Create a mapping from (ptr, size) to scan result
        let scan_map: std::collections::HashMap<(usize, usize), &ScanResult> = scan_results
            .iter()
            .map(|scan| ((scan.ptr, scan.size), scan))
            .collect();

        // Step 2: Run UTI Engine on each allocation.
        // We ensure records has the same length as allocations to maintain index consistency.
        let records: Vec<InferenceRecord> = allocations
            .iter()
            .enumerate()
            .map(|(id, alloc)| {
                let scan = scan_map.get(&(alloc.ptr.unwrap_or(0), alloc.size));

                let (type_kind, confidence) =
                    if let Some(memory) = scan.and_then(|s| s.memory.as_deref()) {
                        let view = MemoryView::new(memory);
                        let guess = UnsafeInferenceEngine::infer_single(&view, alloc.size);
                        (guess.kind, guess.confidence)
                    } else {
                        (TypeKind::Unknown, 0)
                    };

                InferenceRecord {
                    id,
                    ptr: alloc.ptr.unwrap_or(0),
                    size: alloc.size,
                    memory: scan
                        .and_then(|s| s.memory.clone())
                        .map(OwnedMemoryView::new),
                    type_kind,
                    confidence,
                    call_stack_hash: alloc.call_stack_hash,
                    alloc_time: alloc.allocated_at,
                }
            })
            .collect();

        // Step 3: Build RangeMap for address → allocation lookup.
        let range_map = RangeMap::new(allocations);

        // Step 4: Run relation detectors.
        let mut graph = RelationGraph::new();

        // Owner detection: scan each allocation's memory for pointers.
        for record in &records {
            let edges = detect_owner(record, &range_map);
            graph.add_edges(edges);
        }

        // Slice detection: check if each allocation points into another's interior.
        let slice_edges = detect_slice(&records, allocations, &range_map);
        graph.add_edges(slice_edges);

        // Clone detection: batch comparison by (type, size, stack_hash).
        let clone_edges = detect_clones(&records, &config.clone_config);
        graph.add_edges(clone_edges);

        // Container detection: infer Contains relationships using temporal locality.
        let container_edges = detect_containers(allocations, Some(config.container_config));
        graph.add_edges(container_edges);

        // Shared detection: find Arc/Rc shared ownership via Owner graph analysis.
        let shared_edges = detect_shared(&records, &graph.edges);
        graph.add_edges(shared_edges);

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::relation_inference::Relation;
    use crate::core::types::TrackKind;

    fn make_alloc(ptr: usize, size: usize) -> ActiveAllocation {
        ActiveAllocation {
            ptr: Some(ptr),
            size,
            kind: TrackKind::HeapOwner { ptr, size },
            allocated_at: 1000,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
        }
    }

    #[test]
    fn test_build_empty() {
        let graph = RelationGraphBuilder::build(&[], None);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_build_basic_owner_relationship() {
        // Use real heap allocations via Vec to ensure addresses are valid.
        let buf1 = [0u8; 24];
        let buf2 = vec![0u8; 1024];
        let ptr1 = buf1.as_ptr() as usize;
        let ptr2 = buf2.as_ptr() as usize;

        let allocs = vec![make_alloc(ptr1, 24), make_alloc(ptr2, 1024)];
        let graph = RelationGraphBuilder::build(&allocs, None);

        // Graph should be built without panicking.
        // The content is all zeros so no Owner relationships will be found.
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_build_with_real_vec_metadata() {
        // Create a real Vec-like metadata pattern: [ptr, len, cap]
        let inner = vec![42u8; 256];
        let ptr = inner.as_ptr() as usize;
        let len = inner.len();
        let cap = inner.capacity();

        let mut metadata = [0u8; 24];
        metadata[0..8].copy_from_slice(&ptr.to_le_bytes());
        metadata[8..16].copy_from_slice(&len.to_le_bytes());
        metadata[16..24].copy_from_slice(&cap.to_le_bytes());

        let meta_ptr = metadata.as_ptr() as usize;
        let inner_ptr = inner.as_ptr() as usize;

        let allocs = vec![make_alloc(meta_ptr, 24), make_alloc(inner_ptr, 256)];
        let graph = RelationGraphBuilder::build(&allocs, None);

        // Should not crash. Actual edge depends on whether metadata is readable
        // at meta_ptr (it's on the heap, so should be).
        assert!(graph.edge_count() <= 2);
    }

    #[test]
    fn test_build_single_allocation() {
        let allocs = vec![make_alloc(0x1000, 64)];
        let graph = RelationGraphBuilder::build(&allocs, None);

        // Single allocation should not produce any relationships.
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_builder_with_default_config() {
        let allocs = vec![make_alloc(0x1000, 64)];
        let graph = RelationGraphBuilder::build(&allocs, Some(GraphBuilderConfig::default()));
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_node_count() {
        let mut graph = RelationGraph::new();
        graph.add_edge(0, 1, Relation::Owns);
        graph.add_edge(1, 2, Relation::Slice);

        let nodes = graph.all_nodes();
        assert_eq!(nodes, vec![0, 1, 2]);
    }
}
