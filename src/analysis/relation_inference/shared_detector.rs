//! Shared relation detection — Arc/Rc shared ownership.
//!
//! Detects when multiple allocations share ownership of the same underlying data
//! through Arc or Rc reference counting.
//!
//! # Detection Strategy
//!
//! Instead of hardcoding ArcInner offsets (which may change across Rust versions),
//! we use a graph-based approach:
//!
//! 1. After Owner detection, find all nodes that have >= 2 inbound Owner edges.
//! 2. If a node looks like an Arc/Rc control block (size ≈ 16 + inline data,
//!    with small integer patterns in the first 16 bytes), its owners share it.
//! 3. Add Shared edges between all pairs of owners.
//!
//! This avoids fragile offset assumptions and works with any Rust version.

use crate::analysis::relation_inference::{InferenceRecord, Relation, RelationEdge};

const MIN_SHARED_OWNERS: usize = 2;

pub fn detect_shared(
    records: &[InferenceRecord],
    existing_edges: &[RelationEdge],
) -> Vec<RelationEdge> {
    let mut owners_of: Vec<Vec<usize>> = vec![Vec::new(); records.len()];
    for edge in existing_edges {
        if edge.relation == Relation::Owns {
            owners_of[edge.to].push(edge.from);
        }
    }

    let mut relations = Vec::new();

    for (target_id, owners) in owners_of.iter().enumerate() {
        if owners.len() < MIN_SHARED_OWNERS {
            continue;
        }

        let target = &records[target_id];

        if !looks_like_arc_rc(target) {
            continue;
        }

        for i in 0..owners.len() {
            for j in (i + 1)..owners.len() {
                relations.push(RelationEdge {
                    from: owners[i],
                    to: owners[j],
                    relation: Relation::Shares,
                });
            }
        }
    }

    relations
}

fn looks_like_arc_rc(record: &InferenceRecord) -> bool {
    if record.size < 16 || record.size > 1024 {
        return false;
    }

    let memory = match &record.memory {
        Some(m) => m,
        None => return false,
    };

    if memory.len() < 16 {
        return false;
    }

    let strong = memory.read_usize(0).unwrap_or(usize::MAX);
    let weak = memory.read_usize(8).unwrap_or(usize::MAX);

    let strong_valid = (1..=1000).contains(&strong);
    let weak_valid = weak <= 100;

    strong_valid && weak_valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_inference::{OwnedMemoryView, TypeKind};

    fn make_record(
        id: usize,
        ptr: usize,
        size: usize,
        memory: Option<Vec<u8>>,
        type_kind: TypeKind,
    ) -> InferenceRecord {
        InferenceRecord {
            id,
            ptr,
            size,
            memory: memory.map(OwnedMemoryView::new),
            type_kind,
            confidence: 80,
            call_stack_hash: None,
            alloc_time: 0,
        }
    }

    #[test]
    fn test_shared_detection_basic() {
        let mut arc_inner_data = vec![0u8; 48];
        arc_inner_data[0..8].copy_from_slice(&2usize.to_le_bytes());
        arc_inner_data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x2000, 24, None, TypeKind::Vec),
            make_record(2, 0x3000, 48, Some(arc_inner_data), TypeKind::Buffer),
        ];

        let existing_edges = vec![
            RelationEdge {
                from: 0,
                to: 2,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 1,
                to: 2,
                relation: Relation::Owns,
            },
        ];

        let shared = detect_shared(&records, &existing_edges);
        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].from, 0);
        assert_eq!(shared[0].to, 1);
        assert_eq!(shared[0].relation, Relation::Shares);
    }

    #[test]
    fn test_no_shared_with_single_owner() {
        let mut arc_inner_data = vec![0u8; 48];
        arc_inner_data[0..8].copy_from_slice(&1usize.to_le_bytes());
        arc_inner_data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x3000, 48, Some(arc_inner_data), TypeKind::Buffer),
        ];

        let existing_edges = vec![RelationEdge {
            from: 0,
            to: 1,
            relation: Relation::Owns,
        }];

        let shared = detect_shared(&records, &existing_edges);
        assert!(shared.is_empty());
    }

    #[test]
    fn test_no_shared_when_not_arc_like() {
        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x2000, 24, None, TypeKind::Vec),
            make_record(2, 0x3000, 4096, None, TypeKind::Buffer),
        ];

        let existing_edges = vec![
            RelationEdge {
                from: 0,
                to: 2,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 1,
                to: 2,
                relation: Relation::Owns,
            },
        ];

        let shared = detect_shared(&records, &existing_edges);
        assert!(shared.is_empty());
    }

    #[test]
    fn test_shared_three_owners() {
        let mut arc_inner_data = vec![0u8; 48];
        arc_inner_data[0..8].copy_from_slice(&3usize.to_le_bytes());
        arc_inner_data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x2000, 24, None, TypeKind::Vec),
            make_record(2, 0x2500, 24, None, TypeKind::Vec),
            make_record(3, 0x3000, 48, Some(arc_inner_data), TypeKind::Buffer),
        ];

        let existing_edges = vec![
            RelationEdge {
                from: 0,
                to: 3,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 1,
                to: 3,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 2,
                to: 3,
                relation: Relation::Owns,
            },
        ];

        let shared = detect_shared(&records, &existing_edges);
        assert_eq!(shared.len(), 3);
    }

    #[test]
    fn test_looks_like_arc_rc_valid() {
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&2usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_too_small() {
        let record = make_record(0, 0x1000, 8, None, TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_too_large() {
        let record = make_record(0, 0x1000, 2048, None, TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_no_memory() {
        let record = make_record(0, 0x1000, 48, None, TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_invalid_strong() {
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&0usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_invalid_weak() {
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1usize.to_le_bytes());
        data[8..16].copy_from_slice(&9999usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_strong_count_zero() {
        // strong_count = 0 is invalid (no live references).
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&0usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_strong_count_boundary_1000() {
        // strong_count = 1000 should be valid (upper boundary).
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1000usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_strong_count_exceeds_1000() {
        // strong_count = 1001 should be invalid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1001usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_weak_count_boundary_100() {
        // weak_count = 100 should be valid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1usize.to_le_bytes());
        data[8..16].copy_from_slice(&100usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_weak_count_exceeds_100() {
        // weak_count = 101 should be invalid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1usize.to_le_bytes());
        data[8..16].copy_from_slice(&101usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_no_shared_for_regular_vec_type() {
        // Regular Vec types should NOT produce Shared edges even with multiple owners.
        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x2000, 24, None, TypeKind::Vec),
            // A buffer that doesn't look like Arc/Rc (random data, not strong/weak pattern).
            make_record(2, 0x3000, 64, Some(vec![0xAAu8; 64]), TypeKind::Buffer),
        ];

        let existing_edges = vec![
            RelationEdge {
                from: 0,
                to: 2,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 1,
                to: 2,
                relation: Relation::Owns,
            },
        ];

        let shared = detect_shared(&records, &existing_edges);
        assert!(
            shared.is_empty(),
            "Regular Vec buffer should not produce Shared edges"
        );
    }

    #[test]
    fn test_shared_detection_with_rc_like_data() {
        // Rc has the same memory layout as Arc (strong, weak, data).
        let mut rc_inner_data = vec![0u8; 32];
        rc_inner_data[0..8].copy_from_slice(&2usize.to_le_bytes()); // strong = 2
        rc_inner_data[8..16].copy_from_slice(&1usize.to_le_bytes()); // weak = 1

        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x2000, 24, None, TypeKind::Vec),
            make_record(2, 0x3000, 32, Some(rc_inner_data), TypeKind::Buffer),
        ];

        let existing_edges = vec![
            RelationEdge {
                from: 0,
                to: 2,
                relation: Relation::Owns,
            },
            RelationEdge {
                from: 1,
                to: 2,
                relation: Relation::Owns,
            },
        ];

        let shared = detect_shared(&records, &existing_edges);
        assert_eq!(shared.len(), 1, "Rc-like data should produce Shared edge");
        assert_eq!(shared[0].from, 0);
        assert_eq!(shared[0].to, 1);
    }

    #[test]
    fn test_no_shared_with_only_one_owner() {
        // Even with Arc-like data, a single owner should not produce Shared.
        let mut arc_inner_data = vec![0u8; 48];
        arc_inner_data[0..8].copy_from_slice(&1usize.to_le_bytes());
        arc_inner_data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let records = vec![
            make_record(0, 0x1000, 24, None, TypeKind::Vec),
            make_record(1, 0x3000, 48, Some(arc_inner_data), TypeKind::Buffer),
        ];

        let existing_edges = vec![RelationEdge {
            from: 0,
            to: 1,
            relation: Relation::Owns,
        }];

        let shared = detect_shared(&records, &existing_edges);
        assert!(
            shared.is_empty(),
            "Single owner should not produce Shared edges"
        );
    }
}
