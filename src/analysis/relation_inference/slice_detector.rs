//! Slice detection — allocation pointing into the middle of another.
//!
//! A Slice relationship exists when allocation A's pointer falls strictly
//! inside allocation B's address range (not at the start), and A is small
//! enough to be a slice metadata (fat pointer).
//!
//! # False Positive Mitigation
//!
//! Slice detection requires `type_kind == FatPtr` to reduce false positives.
//! This ensures only fat pointer types (&[T], &str, etc.) are considered
//! as potential slice metadata, filtering out struct fields that happen
//! to point into other allocations.

use crate::analysis::relation_inference::{InferenceRecord, RangeMap, Relation, RelationEdge};
use crate::analysis::unsafe_inference::TypeKind;

const MIN_VALID_POINTER: usize = 0x1000;

/// Maximum size for a slice allocation to be considered a Slice relationship.
/// Fat pointers (&[T], &str) are 16 bytes; we allow up to 256 to cover
/// small wrapper structs while filtering out large struct false positives.
const MAX_SLICE_SIZE: usize = 256;

/// Detect Slice relationships for all allocations.
///
/// A Slice edge A → B is created when:
/// 1. A's pointer falls strictly inside B's range: `B.start < A.ptr < B.end`
/// 2. A's memory range fits within B: `A.ptr + A.size <= B.end`
/// 3. A is small enough: `A.size <= MAX_SLICE_SIZE`
/// 4. A is inferred as FatPtr type (reduces false positives)
///
/// # Arguments
///
/// * `records` - All inference records with allocation metadata.
/// * `allocations` - Original allocations list for address range lookup.
/// * `range_map` - Pre-built RangeMap for O(log n) address lookups.
pub fn detect_slice(
    records: &[InferenceRecord],
    allocations: &[crate::snapshot::types::ActiveAllocation],
    range_map: &RangeMap,
) -> Vec<RelationEdge> {
    let mut relations = Vec::new();

    for record in records {
        // Type constraint: only FatPtr types can be slice metadata
        // This filters out struct fields that happen to point into other allocations
        if record.type_kind != TypeKind::FatPtr {
            continue;
        }

        if record.size > MAX_SLICE_SIZE {
            continue;
        }

        let ptr = record.ptr;
        if ptr == 0 || ptr < MIN_VALID_POINTER {
            continue;
        }

        let Some(target_id) = range_map.find_containing(ptr) else {
            continue;
        };

        if target_id == record.id {
            continue;
        }

        let target = &allocations[target_id];
        let target_start = match target.ptr {
            Some(p) => p,
            None => continue, // Skip allocations without heap pointer
        };
        let target_end = target_start.saturating_add(target.size);

        if ptr == target_start {
            continue;
        }

        let slice_end = ptr.saturating_add(record.size);
        if slice_end > target_end {
            continue;
        }

        relations.push(RelationEdge {
            from: record.id,
            to: target_id,
            relation: Relation::Slice,
        });
    }

    relations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::unsafe_inference::TypeKind;

    fn make_alloc(ptr: usize, size: usize) -> crate::snapshot::types::ActiveAllocation {
        crate::snapshot::types::ActiveAllocation {
            ptr: Some(ptr),
            kind: crate::core::types::TrackKind::HeapOwner { ptr, size },
            size,
            allocated_at: 0,
            var_name: None,
            type_name: None,
            thread_id: 0,
            call_stack_hash: None,
        }
    }

    fn make_record(id: usize, ptr: usize, size: usize) -> InferenceRecord {
        InferenceRecord {
            id,
            ptr,
            size,
            memory: None,
            type_kind: TypeKind::FatPtr,
            confidence: 80,
            call_stack_hash: None,
            alloc_time: 0,
        }
    }

    #[test]
    fn test_slice_size_boundary_256() {
        // A 256-byte allocation pointing into a buffer should be detected.
        let slice_ptr: usize = 0x10050;
        let allocs = vec![
            make_alloc(0x5000, 256), // exactly at MAX_SLICE_SIZE boundary
            make_alloc(0x10000, 0x2000),
        ];
        let range_map = RangeMap::new(&allocs);
        let records = vec![make_record(0, slice_ptr, 256)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert_eq!(edges.len(), 1);

        // A 257-byte allocation should NOT be detected (exceeds threshold).
        let allocs2 = vec![make_alloc(0x5000, 257), make_alloc(0x10000, 0x2000)];
        let records2 = vec![make_record(0, slice_ptr, 257)];
        let edges2 = detect_slice(&records2, &allocs2, &range_map);
        assert!(edges2.is_empty());
    }

    #[test]
    fn test_slice_detection_basic() {
        // Scenario: a small allocation (fat pointer metadata) at 0x5000
        // points into the middle of a large buffer at 0x10000.
        // All allocations are non-overlapping — realistic heap layout.
        let slice_ptr: usize = 0x10050; // Inside the buffer, not at start
        let allocs = vec![
            make_alloc(0x5000, 16),      // slice metadata (alloc[0])
            make_alloc(0x10000, 0x2000), // large buffer (alloc[1])
        ];
        let range_map = RangeMap::new(&allocs);

        // Record[0] represents the slice: ptr=0x10050 (inside alloc[1])
        let records = vec![make_record(0, slice_ptr, 16)];
        let edges = detect_slice(&records, &allocs, &range_map);

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from, 0); // slice →
        assert_eq!(edges[0].to, 1); // ... buffer
        assert_eq!(edges[0].relation, Relation::Slice);
    }

    #[test]
    fn test_slice_too_large() {
        let allocs = vec![make_alloc(0x1000, 1024), make_alloc(0x5000, 4096)];
        let range_map = RangeMap::new(&allocs);

        let records = vec![make_record(0, 0x1000, 1024)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_slice_at_target_start_is_not_slice() {
        // Slice pointer at the exact start of the target buffer → should be Owner, not Slice.
        // Non-overlapping: alloc[0] at 0x5000 (metadata), alloc[1] at 0x10000 (buffer).
        let allocs = vec![make_alloc(0x5000, 16), make_alloc(0x10000, 0x2000)];
        let range_map = RangeMap::new(&allocs);

        // Slice pointer equals the target's start address.
        let records = vec![make_record(0, 0x10000, 16)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert!(edges.is_empty()); // Points to start → Owner, not Slice
    }

    #[test]
    fn test_slice_overflowing_target() {
        // Slice extends beyond the target's end → should not match.
        // slice_ptr=0x100F0, size=256 → end=0x101F0
        // target alloc at 0x10000, size=0x100 → end=0x10100
        // 0x101F0 > 0x10100 → overflows
        let slice_ptr: usize = 0x100F0;
        let allocs = vec![
            make_alloc(0x5000, 256),    // slice metadata (alloc[0])
            make_alloc(0x10000, 0x100), // target buffer (alloc[1])
        ];
        let range_map = RangeMap::new(&allocs);

        let records = vec![make_record(0, slice_ptr, 256)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert!(edges.is_empty()); // Slice extends past target end
    }

    #[test]
    fn test_slice_no_containing_allocation() {
        let allocs = vec![make_alloc(0x9000, 100)];
        let range_map = RangeMap::new(&allocs);

        let records = vec![make_record(0, 0x1000, 16)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_slice_self_reference_skipped() {
        // Record[0]'s ptr (0x5010) falls inside alloc[0] itself (0x5000..0x5064).
        // Self-reference should be skipped.
        let allocs = vec![make_alloc(0x5000, 100)];
        let range_map = RangeMap::new(&allocs);

        // ptr=0x5010 is inside alloc[0] (0x5000..0x5064)
        let records = vec![make_record(0, 0x5010, 16)];
        let edges = detect_slice(&records, &allocs, &range_map);
        assert!(edges.is_empty()); // target_id == record.id → skipped
    }

    #[test]
    fn test_slice_multiple_candidates() {
        // Realistic heap layout: two large buffers and two small slice metadatas.
        // alloc[0]: 0x10000..0x12000 (buffer A, 8KB)
        // alloc[1]: 0x20000..0x22000 (buffer B, 8KB)
        // alloc[2]: 0x5000 (slice A metadata, 16 bytes)
        // alloc[3]: 0x6000 (slice B metadata, 16 bytes)
        // alloc[4]: 0x7000 (unrelated small alloc)
        //
        // record[2] at 0x5000 has ptr=0x10050 (inside buffer A) → Slice → alloc[0]
        // record[3] at 0x6000 has ptr=0x20080 (inside buffer B) → Slice → alloc[1]
        // record[4] at 0x7000 has ptr=0x9000 (no containing alloc) → no edge
        let allocs = vec![
            make_alloc(0x10000, 0x2000), // buffer A (id=0)
            make_alloc(0x20000, 0x2000), // buffer B (id=1)
            make_alloc(0x5000, 16),      // slice A metadata (id=2)
            make_alloc(0x6000, 16),      // slice B metadata (id=3)
            make_alloc(0x7000, 16),      // unrelated (id=4)
        ];
        let range_map = RangeMap::new(&allocs);

        // We need records whose ptr fields point INTO the buffers.
        // record[2]: metadata at 0x5000, but its ptr field is 0x10050 (inside buffer A)
        // record[3]: metadata at 0x6000, but its ptr field is 0x20080 (inside buffer B)
        let records = vec![
            make_record(2, 0x10050, 16), // slice into buffer A
            make_record(3, 0x20080, 16), // slice into buffer B
            make_record(4, 0x9000, 16),  // no containing allocation
        ];

        let edges = detect_slice(&records, &allocs, &range_map);
        assert_eq!(edges.len(), 2);

        // Verify the edges point to the right targets.
        let mut targets: Vec<(usize, usize)> = edges.iter().map(|e| (e.from, e.to)).collect();
        targets.sort();
        assert_eq!(targets, vec![(2, 0), (3, 1)]);
    }
}
