//! Clone detection via grouped comparison with sliding time window.
//!
//! Detects when one allocation is a clone of another by grouping on
//! (type, size, stack_hash) and comparing content similarity within
//! a time-bounded sliding window.
//!
//! # False Positive Mitigation
//!
//! When `call_stack_hash` is None (common in synthetic tests), the system
//! applies stricter similarity thresholds to reduce false positives.

use std::collections::HashMap;

use crate::analysis::relation_inference::{InferenceRecord, Relation, RelationEdge};
use crate::analysis::unsafe_inference::TypeKind;

/// Configuration for clone detection.
#[derive(Debug, Clone)]
pub struct CloneConfig {
    /// Maximum time difference (nanoseconds) between two allocations
    /// to be considered potential clones.
    ///
    /// Default: 1ms (1_000_000 ns) - reduced from 10ms to minimize
    /// false positives in hot paths. Real clones typically happen
    /// within microseconds, not milliseconds.
    pub max_time_diff_ns: u64,
    /// Number of leading bytes to compare for content similarity.
    pub compare_bytes: usize,
    /// Minimum similarity ratio (0.0 - 1.0) to consider a clone.
    /// Default: 0.8 (80%).
    pub min_similarity: f64,
    /// Minimum similarity when call_stack_hash is None.
    /// Higher threshold reduces false positives in synthetic data.
    /// Default: 0.95 (95%).
    pub min_similarity_no_stack_hash: f64,
    /// Maximum clone edges per node to prevent explosion.
    /// Default: 10.
    pub max_clone_edges_per_node: usize,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            max_time_diff_ns: 1_000_000, // 1ms - reduced for hot path accuracy
            compare_bytes: 64,
            min_similarity: 0.8,
            min_similarity_no_stack_hash: 0.95,
            max_clone_edges_per_node: 10,
        }
    }
}

/// Detect Clone relationships among all inference records.
///
/// Groups allocations by `(TypeKind, size, call_stack_hash)` and then
/// uses a sliding time window to compare content similarity within each group.
/// This avoids O(n^2) worst-case by only comparing temporally close allocations.
///
/// # False Positive Mitigation
///
/// When `call_stack_hash` is None, applies stricter similarity threshold
/// (`min_similarity_no_stack_hash`) to reduce false positives.
///
/// # Arguments
///
/// * `records` - All inference records.
/// * `config` - Detection configuration.
pub fn detect_clones(records: &[InferenceRecord], config: &CloneConfig) -> Vec<RelationEdge> {
    let mut groups: HashMap<(TypeKind, usize, u64), Vec<usize>> = HashMap::new();
    for (i, record) in records.iter().enumerate() {
        let stack_hash = record.call_stack_hash.unwrap_or(0);
        let key = (record.type_kind, record.size, stack_hash);
        groups.entry(key).or_default().push(i);
    }

    let mut relations = Vec::new();
    let mut edge_count_per_node: HashMap<usize, usize> = HashMap::new();

    for group_indices in groups.values() {
        if group_indices.len() < 2 {
            continue;
        }

        let mut group: Vec<&InferenceRecord> = group_indices.iter().map(|&i| &records[i]).collect();
        group.sort_by_key(|r| r.alloc_time);

        let has_stack_hash = group.iter().any(|r| r.call_stack_hash.is_some());
        let min_similarity = if has_stack_hash {
            config.min_similarity
        } else {
            config.min_similarity_no_stack_hash
        };

        let mut left = 0;
        for right in 1..group.len() {
            while left < right
                && group[right]
                    .alloc_time
                    .saturating_sub(group[left].alloc_time)
                    > config.max_time_diff_ns
            {
                left += 1;
            }

            for i in left..right {
                let a = group[i];
                let b = group[right];

                let a_count = edge_count_per_node.get(&a.id).copied().unwrap_or(0);
                let b_count = edge_count_per_node.get(&b.id).copied().unwrap_or(0);

                if a_count >= config.max_clone_edges_per_node
                    || b_count >= config.max_clone_edges_per_node
                {
                    continue;
                }

                if content_similarity(a, b, config.compare_bytes) >= min_similarity {
                    relations.push(RelationEdge {
                        from: a.id,
                        to: b.id,
                        relation: Relation::Clone,
                    });
                    *edge_count_per_node.entry(a.id).or_insert(0) += 1;
                    *edge_count_per_node.entry(b.id).or_insert(0) += 1;
                }
            }
        }
    }

    relations
}

/// Compute content similarity between two records.
///
/// Compares the first `max_bytes` of each record's memory content
/// and returns the ratio of matching bytes.
fn content_similarity(a: &InferenceRecord, b: &InferenceRecord, max_bytes: usize) -> f64 {
    let memory_a = match &a.memory {
        Some(m) => m,
        None => return 0.0,
    };
    let memory_b = match &b.memory {
        Some(m) => m,
        None => return 0.0,
    };

    let len = max_bytes.min(memory_a.len()).min(memory_b.len());
    if len == 0 {
        return 0.0;
    }

    let mut matching = 0usize;
    for i in 0..len {
        let byte_a: u8 = memory_a.read_u8(i).unwrap_or(0);
        let byte_b: u8 = memory_b.read_u8(i).unwrap_or(0);
        if byte_a == byte_b {
            matching += 1;
        }
    }

    matching as f64 / len as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_inference::{OwnedMemoryView, TypeKind};

    fn make_record(
        id: usize,
        ptr: usize,
        size: usize,
        memory: Vec<u8>,
        stack_hash: Option<u64>,
        alloc_time: u64,
        type_kind: TypeKind,
    ) -> InferenceRecord {
        InferenceRecord {
            id,
            ptr,
            size,
            memory: Some(OwnedMemoryView::new(memory)),
            type_kind,
            confidence: 80,
            call_stack_hash: stack_hash,
            alloc_time,
        }
    }

    #[test]
    fn test_clone_detection_identical_content() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(
                0,
                0x1000,
                24,
                content.clone(),
                Some(123),
                1000,
                TypeKind::Vec,
            ),
            make_record(1, 0x2000, 24, content, Some(123), 5000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, 0);
        assert_eq!(edges[0].to, 1);
        assert_eq!(edges[0].relation, Relation::Clone);
    }

    #[test]
    fn test_clone_detection_different_content() {
        let content_a = vec![0xAAu8; 64];
        let content_b = vec![0xBBu8; 64];
        let records = vec![
            make_record(0, 0x1000, 24, content_a, Some(123), 1000, TypeKind::Vec),
            make_record(1, 0x2000, 24, content_b, Some(123), 5000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_time_window_exceeded() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(
                0,
                0x1000,
                24,
                content.clone(),
                Some(123),
                1000,
                TypeKind::Vec,
            ),
            make_record(1, 0x2000, 24, content, Some(123), 20_000_000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_different_types() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(
                0,
                0x1000,
                24,
                content.clone(),
                Some(123),
                1000,
                TypeKind::Vec,
            ),
            make_record(1, 0x2000, 24, content, Some(123), 5000, TypeKind::String),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_different_stack_hashes() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(
                0,
                0x1000,
                24,
                content.clone(),
                Some(111),
                1000,
                TypeKind::Vec,
            ),
            make_record(1, 0x2000, 24, content, Some(222), 5000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_no_memory() {
        let records = vec![
            InferenceRecord {
                id: 0,
                ptr: 0x1000,
                size: 24,
                memory: None,
                type_kind: TypeKind::Vec,
                confidence: 80,
                call_stack_hash: Some(123),
                alloc_time: 1000,
            },
            InferenceRecord {
                id: 1,
                ptr: 0x2000,
                size: 24,
                memory: None,
                type_kind: TypeKind::Vec,
                confidence: 80,
                call_stack_hash: Some(123),
                alloc_time: 5000,
            },
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_partial_similarity() {
        let content_a = vec![0xAAu8; 64];
        let mut content_b = vec![0xAAu8; 64];
        for i in 50..64 {
            content_b[i] = 0xBB;
        }

        let records = vec![
            make_record(0, 0x1000, 24, content_a, Some(123), 1000, TypeKind::Vec),
            make_record(1, 0x2000, 24, content_b, Some(123), 5000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_sliding_window_efficiency() {
        // Create 5 records with identical content and timestamps within 1ms window.
        // All records have time diff < 1ms from each other, so the sliding window
        // should compare all pairs and detect C(5,2) = 10 clone edges.
        let content = vec![0xCCu8; 64];
        let records: Vec<InferenceRecord> = (0..5)
            .map(|i| {
                make_record(
                    i,
                    0x1000 + i * 0x1000,
                    24,
                    content.clone(),
                    Some(999),
                    1000_u64 + (i as u64) * 100_000, // 0.1ms apart, total span 0.4ms < 1ms
                    TypeKind::Vec,
                )
            })
            .collect();

        let edges = detect_clones(&records, &CloneConfig::default());
        assert_eq!(edges.len(), 10); // C(5,2) = 10
    }

    #[test]
    fn test_clone_config_defaults() {
        let config = CloneConfig::default();
        assert_eq!(config.max_time_diff_ns, 1_000_000); // 1ms
        assert_eq!(config.compare_bytes, 64);
        assert!((config.min_similarity - 0.8).abs() < f64::EPSILON);
        assert!((config.min_similarity_no_stack_hash - 0.95).abs() < f64::EPSILON);
        assert_eq!(config.max_clone_edges_per_node, 10);
    }

    #[test]
    fn test_clone_detection_different_sizes() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(
                0,
                0x1000,
                24,
                content.clone(),
                Some(123),
                1000,
                TypeKind::Vec,
            ),
            make_record(1, 0x2000, 48, content, Some(123), 5000, TypeKind::Vec),
        ];

        let edges = detect_clones(&records, &CloneConfig::default());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_clone_detection_no_stack_hash_stricter_threshold() {
        let content = vec![0xAAu8; 64];
        let records = vec![
            make_record(0, 0x1000, 24, content.clone(), None, 1000, TypeKind::Vec),
            make_record(1, 0x2000, 24, content, None, 5000, TypeKind::Vec),
        ];

        let config = CloneConfig::default();
        let edges = detect_clones(&records, &config);

        assert_eq!(edges.len(), 1, "Identical content should still be detected");
    }

    #[test]
    fn test_clone_detection_no_stack_hash_partial_similarity_rejected() {
        let content_a = vec![0xAAu8; 64];
        let mut content_b = vec![0xAAu8; 64];
        for i in 60..64 {
            content_b[i] = 0xBB;
        }

        let records = vec![
            make_record(0, 0x1000, 24, content_a, None, 1000, TypeKind::Vec),
            make_record(1, 0x2000, 24, content_b, None, 5000, TypeKind::Vec),
        ];

        let config = CloneConfig::default();
        let edges = detect_clones(&records, &config);

        assert!(
            edges.is_empty(),
            "Partial similarity should be rejected with no stack hash"
        );
    }

    #[test]
    fn test_clone_detection_max_edges_per_node() {
        let content = vec![0xCCu8; 64];
        let records: Vec<InferenceRecord> = (0..20)
            .map(|i| {
                make_record(
                    i,
                    0x1000 + i * 0x1000,
                    24,
                    content.clone(),
                    Some(999),
                    1000,
                    TypeKind::Vec,
                )
            })
            .collect();

        let config = CloneConfig {
            max_clone_edges_per_node: 3,
            ..Default::default()
        };
        let edges = detect_clones(&records, &config);

        let mut edge_count: HashMap<usize, usize> = HashMap::new();
        for edge in &edges {
            *edge_count.entry(edge.from).or_insert(0) += 1;
            *edge_count.entry(edge.to).or_insert(0) += 1;
        }

        for (&_node, &count) in &edge_count {
            assert!(
                count <= config.max_clone_edges_per_node,
                "Node has {} edges, max is {}",
                count,
                config.max_clone_edges_per_node
            );
        }
    }
}
