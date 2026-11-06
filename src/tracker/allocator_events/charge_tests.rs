//! Charge marks belong to allocator bindings, independently of local observations.

use super::*;

/// Objective: Exercise migration and reclamation while fifty producers share the mark map.
/// Invariants: successful moves clear only their old mark, and releases reclaim every new mark.
#[test]
fn fifty_concurrent_charge_migrations_keep_their_identity() {
    let tracker = Tracker::new();
    std::thread::scope(|scope| {
        for index in 0..50usize {
            let tracker = &tracker;
            scope.spawn(move || {
                let _suppression = TrackingGuard::acquire();
                let old = 0x7400_0000 + index * 4096;
                let new = old + 2048;
                tracker.record_allocator_allocation(old, 64);
                let outcome = allocator_charge(old);
                assert!(
                    outcome.is_some(),
                    "producer {index} must publish a charge mark"
                );
                tracker.track_reallocation(old, new, 64, 128);
                assert_eq!(
                    allocator_charge(old),
                    None,
                    "producer {index} must retire its old address"
                );
                assert_eq!(
                    allocator_charge(new),
                    outcome,
                    "producer {index} must retain its own outcome"
                );
                tracker
                    .track_deallocation(new)
                    .expect("release migrated identity");
                assert_eq!(
                    allocator_charge(new),
                    None,
                    "producer {index} must reclaim its final mark"
                );
            });
        }
    });
}

/// Objective: Keep one charge attempt across in-place, failed and moving reallocations.
/// Invariants: an old address loses its mark only after success, and final release removes it.
#[test]
fn charge_mark_follows_successful_reallocation() {
    let tracker = Tracker::new();
    let old = 0x7110_0000;
    let new = 0x7120_0000;
    tracker.record_allocator_allocation(old, 64);
    let outcome = allocator_charge(old);
    assert!(
        outcome.is_some(),
        "the allocator must record a charge attempt"
    );

    let failed = tracker.allocator_reallocation(old, 64, usize::MAX, std::ptr::null_mut);
    assert!(
        failed.is_null(),
        "the injected failed realloc must return null"
    );
    assert_eq!(
        allocator_charge(old),
        outcome,
        "failure must preserve the original mark"
    );

    tracker.track_reallocation(old, old, 64, 128);
    assert_eq!(
        allocator_charge(old),
        outcome,
        "an in-place resize must preserve the mark"
    );
    tracker.track_reallocation(old, new, 128, 256);
    assert_eq!(
        allocator_charge(old),
        None,
        "migration must not leave an old-address mark"
    );
    assert_eq!(
        allocator_charge(new),
        outcome,
        "the new address must retain its charge outcome"
    );
    tracker
        .track_deallocation(new)
        .expect("release migrated binding");
    assert_eq!(
        allocator_charge(new),
        None,
        "release must reclaim the migrated mark"
    );
}

/// Objective: Isolate markers when local trackers or old generations release an address.
/// Invariants: only the producing tracker and matching live serial can clear the current mark.
#[test]
fn foreign_and_stale_releases_preserve_charge_mark() {
    let producer = Tracker::new();
    let observer = Tracker::new();
    let ptr = 0x7130_0000;
    producer.record_allocator_allocation(ptr, 64);
    let serial = producer
        .inner
        .binding_serial(ptr)
        .expect("original allocator serial");
    producer.record_allocator_allocation(ptr, 128);
    let outcome = allocator_charge(ptr);

    observer
        .inner
        .track_allocation(ptr, 128)
        .expect("register local observation");
    observer
        .track_deallocation(ptr)
        .expect("release local observation");
    assert_eq!(
        allocator_charge(ptr),
        outcome,
        "a local tracker cannot remove the allocator mark"
    );
    producer
        .track_deallocation_of(ptr, serial)
        .expect("close old allocator generation");
    assert_eq!(
        allocator_charge(ptr),
        outcome,
        "closing an older serial must preserve the new mark"
    );
    producer
        .track_deallocation(ptr)
        .expect("release current allocator binding");
    assert_eq!(
        allocator_charge(ptr),
        None,
        "the matching final release must reclaim the mark"
    );
}
