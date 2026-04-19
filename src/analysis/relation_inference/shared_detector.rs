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
//! 4. Arc-specific detection: Find all allocations that look like Arc pointers
//!    (size = 8, pointing to a valid address with ArcInner pattern).
//!    Group them by the target address and add Shared edges.
//!
//! This avoids fragile offset assumptions and works with any Rust version.

use crate::analysis::relation_inference::{InferenceRecord, Relation, RelationEdge};

const MIN_SHARED_OWNERS: usize = 2;

pub fn detect_shared(
    records: &[InferenceRecord],
    existing_edges: &[RelationEdge],
) -> Vec<RelationEdge> {
    let mut relations = Vec::new();

    // Strategy 1: Owner-based detection (for Rc)
    let mut owners_of: Vec<Vec<usize>> = vec![Vec::new(); records.len()];
    for edge in existing_edges {
        if edge.relation == Relation::Owns {
            owners_of[edge.to].push(edge.from);
        }
    }

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

    // Strategy 2: StackOwner-based detection (for Arc/Rc)
    // Find all StackOwner allocations (Arc/Rc) and group them by their heap_ptr
    // Since Arc/Rc objects are on stack (8 bytes) but point to heap, we can detect
    // clones by finding multiple StackOwner objects pointing to the same heap allocation
    // Note: We don't rely on type_kind here because UTI Engine may misclassify Arc as Vec
    let mut stack_owners: Vec<(usize, usize)> = Vec::new(); // (record_id, heap_ptr)
    for (i, record) in records.iter().enumerate() {
        // Check if this allocation has stack_ptr metadata
        // This indicates it's a StackOwner (Arc/Rc) tracked via the new StackOwner track_kind
        if let Some(stack_ptr) = record.stack_ptr {
            if stack_ptr > 0x1000 {
                // Group by heap_ptr (record.ptr) which is the heap allocation
                stack_owners.push((i, record.ptr));
            }
        }
    }

    // Group StackOwner allocations by heap pointer
    let mut heap_to_records: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (record_id, heap_ptr) in stack_owners {
        heap_to_records.entry(heap_ptr).or_default().push(record_id);
    }

    // Add ArcClone edges for StackOwner objects pointing to the same heap allocation
    for (_heap_ptr, record_ids) in heap_to_records {
        if record_ids.len() >= 2 {
            for i in 0..record_ids.len() {
                for j in (i + 1)..record_ids.len() {
                    // Use ArcClone for StackOwner-based clone detection
                    relations.push(RelationEdge {
                        from: record_ids[i],
                        to: record_ids[j],
                        relation: Relation::ArcClone,
                    });
                }
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

    // 放宽阈值，增加检测范围
    let strong_valid = (1..=10000).contains(&strong); // 从 1000 提升到 10000
    let weak_valid = weak <= 1000; // 从 100 提升到 1000

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
            confidence: 100,
            call_stack_hash: None,
            alloc_time: 0,
            stack_ptr: None,
        }
    }

    fn make_record_with_stack_ptr(
        id: usize,
        ptr: usize,
        stack_ptr: usize,
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
            confidence: 100,
            call_stack_hash: None,
            alloc_time: 0,
            stack_ptr: Some(stack_ptr),
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
    fn test_looks_like_arc_rc_strong_count_boundary_10000() {
        // strong_count = 10000 should be valid (upper boundary).
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&10000usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_strong_count_exceeds_10000() {
        // strong_count = 10001 should be invalid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&10001usize.to_le_bytes());
        data[8..16].copy_from_slice(&0usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(!looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_weak_count_boundary_1000() {
        // weak_count = 1000 should be valid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1usize.to_le_bytes());
        data[8..16].copy_from_slice(&1000usize.to_le_bytes());

        let record = make_record(0, 0x1000, 48, Some(data), TypeKind::Buffer);
        assert!(looks_like_arc_rc(&record));
    }

    #[test]
    fn test_looks_like_arc_rc_weak_count_exceeds_1000() {
        // weak_count = 1001 should be invalid.
        let mut data = vec![0u8; 48];
        data[0..8].copy_from_slice(&1usize.to_le_bytes());
        data[8..16].copy_from_slice(&1001usize.to_le_bytes());

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
    fn test_stackowner_arc_clone_detection() {
        // Test StackOwner-based Arc clone detection
        // Multiple Arc objects on stack pointing to the same heap allocation
        let heap_ptr = 0x3000; // ArcInner address
        let stack_ptr1 = 0x7ff0000; // First Arc object on stack
        let stack_ptr2 = 0x7ff0008; // Second Arc object on stack
        let stack_ptr3 = 0x7ff0010; // Third Arc object on stack

        let records = vec![
            // Arc clone 1: stack_ptr1 -> heap_ptr
            make_record_with_stack_ptr(0, heap_ptr, stack_ptr1, 48, None, TypeKind::Buffer),
            // Arc clone 2: stack_ptr2 -> heap_ptr
            make_record_with_stack_ptr(1, heap_ptr, stack_ptr2, 48, None, TypeKind::Buffer),
            // Arc clone 3: stack_ptr3 -> heap_ptr
            make_record_with_stack_ptr(2, heap_ptr, stack_ptr3, 48, None, TypeKind::Buffer),
            // Unrelated allocation
            make_record(3, 0x4000, 24, None, TypeKind::Vec),
        ];

        let existing_edges = vec![];

        let shared = detect_shared(&records, &existing_edges);

        // Should detect Arc clones and return ArcClone edges
        // With 3 Arc clones pointing to the same heap, we should get edges between them
        assert!(
            !shared.is_empty(),
            "StackOwner Arc clones should be detected"
        );

        // Verify that the edges are ArcClone type
        let arc_clone_edges: Vec<_> = shared
            .iter()
            .filter(|e| matches!(e.relation, Relation::ArcClone))
            .collect();
        assert!(!arc_clone_edges.is_empty(), "Should have ArcClone edges");

        println!(
            "Detected {} ArcClone edges from 3 Arc clones",
            arc_clone_edges.len()
        );
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
