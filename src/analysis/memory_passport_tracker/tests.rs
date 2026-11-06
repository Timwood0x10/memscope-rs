//! Unit tests for the memory passport tracker.
//!
//! Split out of `memory_passport_tracker.rs` to keep that file within the project
//! line budget.

use super::*;

#[test]
fn test_passport_creation() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x1000;
    let size = 1024;

    let passport_id = tracker
        .create_passport(ptr, size, "test_context".to_string(), None, None)
        .expect("Failed to create passport");
    assert!(!passport_id.is_empty());

    let passport = tracker.get_passport(ptr).expect("Failed to get passport");
    assert_eq!(passport.allocation_ptr, ptr);
    assert_eq!(passport.size_bytes, size);
    assert_eq!(passport.lifecycle_events.len(), 1);
    assert_eq!(
        passport.lifecycle_events[0].event_type,
        PassportEventType::AllocatedInRust
    );
}

#[test]
fn test_handover_to_ffi() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x2000;

    tracker
        .create_passport(ptr, 512, "rust_context".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string())
        .expect("Failed to record handover");

    let passport = tracker.get_passport(ptr).expect("Failed to get passport");
    assert_eq!(passport.lifecycle_events.len(), 2);
    assert_eq!(
        passport.lifecycle_events[1].event_type,
        PassportEventType::HandoverToFfi
    );
}

#[test]
fn test_leak_detection() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x3000;

    // Create passport and hand over to FFI without reclaim
    tracker
        .create_passport(ptr, 256, "rust_context".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string())
        .expect("Failed to record handover");

    // Detect leaks at shutdown
    let leak_result = tracker.detect_leaks_at_shutdown();

    assert_eq!(leak_result.total_leaks, 1);
    assert_eq!(leak_result.leaked_passports.len(), 1);
    assert_eq!(leak_result.leak_details.len(), 1);
    assert_eq!(leak_result.leak_details[0].memory_address, ptr);

    // The status is a projection: detection must not rewrite the stored passport.
    let passport = tracker.get_passport(ptr).expect("Test operation failed");
    assert_eq!(
        passport.status_at_shutdown,
        PassportStatus::Unknown,
        "detection must not mutate the stored passport"
    );
    assert_eq!(
        tracker.assess_status(&passport),
        PassportStatus::InForeignCustody,
        "the projection must report foreign custody"
    );
}

#[test]
fn test_reclaim_prevents_leak() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x4000;

    // Create passport, hand over to FFI, then reclaim
    tracker
        .create_passport(ptr, 128, "rust_context".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string())
        .expect("Failed to record handover");
    tracker
        .record_reclaimed_by_rust(ptr, "rust_context".to_string(), "cleanup".to_string())
        .expect("Failed to record reclaim");

    // Detect leaks at shutdown
    let leak_result = tracker.detect_leaks_at_shutdown();

    assert_eq!(leak_result.total_leaks, 0);

    // The status is a projection: detection must not rewrite the stored passport.
    let passport = tracker.get_passport(ptr).expect("Test operation failed");
    assert_eq!(
        passport.status_at_shutdown,
        PassportStatus::Unknown,
        "detection must not mutate the stored passport"
    );
    assert_eq!(
        tracker.assess_status(&passport),
        PassportStatus::ReclaimedByRust
    );
}

#[test]
fn test_passport_validation() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x5000;

    tracker
        .create_passport(ptr, 64, "test_context".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi_context".to_string(), "test_func".to_string())
        .expect("Failed to record handover");

    let is_valid = tracker
        .validate_passport(ptr)
        .expect("Failed to validate passport");
    assert!(is_valid);
}

/// Objective: Verify a passport that never crossed the FFI boundary and was never
/// released is reported as still live, not as "freed by Rust" (plan A15).
/// Invariants: the projection is `StillLive`, detection reports no leak, and the
/// stored passport is not rewritten.
#[test]
fn live_passport_is_not_reported_as_freed_by_rust() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x5100;
    tracker
        .create_passport(ptr, 64, "rust_context".to_string(), None, None)
        .expect("Failed to create passport");

    let passport = tracker.get_passport(ptr).expect("passport must exist");
    assert_eq!(
        tracker.assess_status(&passport),
        PassportStatus::StillLive,
        "an untouched live block must not be labelled as released"
    );
    assert_eq!(
        tracker.detect_leaks_at_shutdown().total_leaks,
        0,
        "a block that never crossed the FFI boundary is not a leak"
    );
    assert_eq!(
        tracker
            .get_passport(ptr)
            .expect("passport must exist")
            .status_at_shutdown,
        PassportStatus::Unknown,
        "detection must not mutate the stored passport"
    );
}

/// Objective: Verify detection is a pure projection, repeatable and side-effect free.
/// Invariants: two consecutive detections agree and the stored statuses stay unchanged.
#[test]
fn detect_is_read_only_and_repeatable() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x5200;
    tracker
        .create_passport(ptr, 128, "rust_context".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi_context".to_string(), "malloc".to_string())
        .expect("Failed to record handover");

    let first = tracker.detect_leaks_at_shutdown();
    let second = tracker.detect_leaks_at_shutdown();

    assert_eq!(first.total_leaks, 1, "the handed-over block is a leak");
    assert_eq!(
        first.total_leaks, second.total_leaks,
        "repeated detection must not drift"
    );
    assert_eq!(
        tracker
            .get_passport(ptr)
            .expect("passport must exist")
            .status_at_shutdown,
        PassportStatus::Unknown,
        "detection must not write a shutdown verdict into the passport"
    );
}

/// Objective: Verify address reuse archives the previous passport instead of silently
/// overwriting it, and assigns a new generation (plan A15).
/// Invariants: current passport is generation 2, the archived one keeps generation 1
/// and its handover evidence.
#[test]
fn address_reuse_archives_previous_passport() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x5300;
    tracker
        .create_passport(ptr, 64, "first".to_string(), None, None)
        .expect("Failed to create first passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi".to_string(), "malloc".to_string())
        .expect("Failed to record handover");

    tracker
        .create_passport(ptr, 128, "second".to_string(), None, None)
        .expect("Failed to create second passport");

    let current = tracker.get_passport(ptr).expect("current passport");
    assert_eq!(current.generation, 2, "the new block must be generation 2");
    assert_eq!(current.size_bytes, 128, "the new block keeps its own size");

    let archived = tracker.get_archived_passports();
    assert_eq!(archived.len(), 1, "the replaced passport must be archived");
    assert!(archived[0].superseded, "the archived record must be marked");
    assert_eq!(archived[0].generation, 1, "generation 1 keeps its identity");
    assert!(
        archived[0]
            .lifecycle_events
            .iter()
            .any(|event| event.event_type == PassportEventType::HandoverToFfi),
        "the archived record must keep the handover evidence"
    );
}

/// Objective: Verify a truncated lifecycle chain is flagged (plan §7.3 item 6).
/// Invariants: the passport reports `history_truncated` and carries the note in its
/// metadata once events exceed the configured cap.
#[test]
fn truncated_history_is_flagged() {
    let config = PassportTrackerConfig {
        max_events_per_passport: 3,
        ..PassportTrackerConfig::default()
    };
    let tracker = MemoryPassportTracker::new(config);
    let ptr = 0x5400;
    tracker
        .create_passport(ptr, 32, "ctx".to_string(), None, None)
        .expect("Failed to create passport");

    for _ in 0..5 {
        tracker
            .record_passport_event(
                ptr,
                PassportEventType::ValidationCheck,
                "check".to_string(),
                HashMap::new(),
            )
            .expect("Failed to record event");
    }

    let passport = tracker.get_passport(ptr).expect("passport must exist");
    assert!(
        passport.history_truncated,
        "a truncated chain must be flagged instead of looking complete"
    );
    assert!(
        passport.metadata.contains_key("history_truncated"),
        "the truncation must be visible to report consumers"
    );
    assert_eq!(
        passport.lifecycle_events.len(),
        3,
        "the cap must still be respected"
    );
}

/// Objective: Verify statistics reflect the read-only projection, so a leak counted in the
/// JSON is also counted by `get_stats()` (plan A15-2 follow-up).
/// Invariants: `leaks_detected` and `passports_by_status` match the assessed status.
#[test]
fn stats_reflect_projected_leaks() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x6100;
    tracker
        .create_passport(ptr, 96, "ctx".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi".to_string(), "malloc".to_string())
        .expect("Failed to record handover");

    let stats = tracker.get_stats();
    assert_eq!(
        stats.leaks_detected, 1,
        "the stats must report the projected leak without running detection"
    );
    assert_eq!(
        stats.passports_by_status.get("InForeignCustody"),
        Some(&1),
        "the per-status counter must come from the projection"
    );
}

/// Objective: Verify an archived generation takes part in leak detection.
/// Invariants: the older passport's handover evidence is still reported after the address
/// was reused.
#[test]
fn archived_passport_takes_part_in_detection() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    let ptr = 0x6200;
    tracker
        .create_passport(ptr, 64, "first".to_string(), None, None)
        .expect("Failed to create first passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi".to_string(), "malloc".to_string())
        .expect("Failed to record handover");
    let archived_id = tracker
        .get_passport(ptr)
        .expect("first passport")
        .passport_id;

    tracker
        .create_passport(ptr, 128, "second".to_string(), None, None)
        .expect("Failed to create second passport");

    let result = tracker.detect_leaks_at_shutdown();
    assert!(
        result.leaked_passports.contains(&archived_id),
        "the archived generation must still be reported as leaked"
    );
}

/// Objective: Verify the archive is part of the passport budget, so address reuse cannot
/// grow the analyzer without bound (plan A15 follow-up).
/// Invariants: current + archived passports never exceed `max_passports`.
#[test]
fn archive_respects_passport_budget() {
    let config = PassportTrackerConfig {
        max_passports: 2,
        ..PassportTrackerConfig::default()
    };
    let tracker = MemoryPassportTracker::new(config);
    let ptr = 0x6300;

    for generation in 0..12u64 {
        tracker
            .create_passport(
                ptr,
                (generation as usize + 1) * 8,
                format!("ctx-{generation}"),
                None,
                None,
            )
            .expect("creating a passport at a reused address must succeed");
    }

    let total = tracker.get_all_passports().len() + tracker.get_archived_passports().len();
    assert!(
        total <= 2,
        "current plus archived passports must respect max_passports, got {total}"
    );
}

/// Objective: Verify a truncated history must not produce a confident verdict
/// (plan §7.3 item 6).
/// Invariants: the passport is flagged and the assessed status is `Unknown`.
#[test]
fn truncated_history_yields_unknown_status() {
    let config = PassportTrackerConfig {
        max_events_per_passport: 2,
        ..PassportTrackerConfig::default()
    };
    let tracker = MemoryPassportTracker::new(config);
    let ptr = 0x6400;
    tracker
        .create_passport(ptr, 48, "ctx".to_string(), None, None)
        .expect("Failed to create passport");
    tracker
        .record_handover_to_ffi(ptr, "ffi".to_string(), "malloc".to_string())
        .expect("Failed to record handover");
    for _ in 0..2 {
        tracker
            .record_passport_event(
                ptr,
                PassportEventType::ValidationCheck,
                "check".to_string(),
                HashMap::new(),
            )
            .expect("Failed to record event");
    }

    let passport = tracker.get_passport(ptr).expect("passport must exist");
    assert!(
        passport.history_truncated,
        "the chain must be flagged as truncated"
    );
    assert_eq!(
        tracker.assess_status(&passport),
        PassportStatus::Unknown,
        "a truncated chain cannot support a confident status"
    );
    assert_eq!(
        tracker.detect_leaks_at_shutdown().total_leaks,
        0,
        "an incomplete chain must not be reported as a confirmed leak"
    );
}

#[test]
fn concurrent_reuse_snapshot_contains_each_retained_generation_once() {
    let tracker = MemoryPassportTracker::new(PassportTrackerConfig::default());
    tracker
        .create_passport_simple(0x6500, 64, "initial".into())
        .unwrap();
    std::thread::scope(|scope| {
        for _ in 0..2 {
            let tracker = &tracker;
            scope.spawn(move || {
                for _ in 0..32 {
                    tracker
                        .create_passport_simple(0x6500, 64, "reuse".into())
                        .unwrap();
                }
            });
        }
        for _ in 0..64 {
            let records = tracker.snapshot_passports();
            assert_eq!(records.iter().filter(|p| !p.superseded).count(), 1);
            for (index, passport) in records.iter().enumerate() {
                assert_eq!(passport.generation, index as u64 + 1);
            }
            assert!(!records.last().unwrap().superseded);
        }
    });
    assert_eq!(tracker.snapshot_passports().len(), 65);
}

#[test]
fn frozen_passport_assessment_is_independent_of_later_reuse() {
    let tracker = MemoryPassportTracker::default();
    let id = tracker
        .create_passport_simple(0x6600, 64, "rust".into())
        .unwrap();
    tracker
        .record_handover_to_ffi(0x6600, "ffi".into(), "consume".into())
        .unwrap();
    let frozen = tracker.snapshot_passports();
    tracker
        .record_reclaimed_by_rust(0x6600, "rust".into(), "return".into())
        .unwrap();
    tracker
        .create_passport_simple(0x6600, 128, "reuse".into())
        .unwrap();
    assert_eq!(tracker.detect_leaks_in(&frozen).leaked_passports, vec![id]);
    assert_eq!(tracker.detect_leaks_at_shutdown().total_leaks, 0);
    assert!(!frozen[0].superseded);
}
