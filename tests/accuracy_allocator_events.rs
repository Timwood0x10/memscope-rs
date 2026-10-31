//! Exercise the real GlobalAlloc path, including allocations made by the test harness.

#[cfg(not(feature = "tracking-allocator"))]
use memscope_rs::core::allocator::TrackingAllocator;
use memscope_rs::event_store::MemoryEventType;
use memscope_rs::tracker::Tracker;

#[global_allocator]
#[cfg(not(feature = "tracking-allocator"))]
static ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

/// Objective: Hook events and typed observations must describe one allocation.
/// Invariants: growth preserves serial identity; a real drop closes exactly that serial.
#[test]
fn allocator_lifecycle_and_fifty_thread_publication() {
    let tracker = Tracker::global();
    let mut buffer = Vec::<u8>::with_capacity(64);
    buffer.push(23);
    memscope_rs::track!(tracker, buffer);
    let first = tracker.events();
    let initial = first
        .iter()
        .rev()
        .find(|event| {
            event.ptr == buffer.as_ptr() as usize && event.event_type == MemoryEventType::Allocate
        })
        .expect("allocator allocation must precede metadata");
    let serial = initial
        .binding_serial
        .expect("hook allocation must carry serial");
    buffer.reserve_exact(4096);
    memscope_rs::track!(tracker, buffer);
    assert_eq!(
        buffer[0], 23,
        "realloc must preserve the old initialized prefix"
    );
    drop(buffer);
    let events: Vec<_> = tracker
        .events()
        .into_iter()
        .filter(|event| event.binding_serial == Some(serial))
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|e| e.event_type == MemoryEventType::Allocate)
            .count(),
        1,
        "typed observations cannot double-count a hook allocation"
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| e.event_type == MemoryEventType::Reallocate)
            .count(),
        1,
        "one growth must publish one realloc"
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| e.event_type == MemoryEventType::Deallocate)
            .count(),
        1,
        "actual drop must publish a serial-matched release"
    );
    assert_eq!(
        memscope_rs::snapshot::build_snapshot_from_events(&events).current_memory(),
        0,
        "hook replay must conserve bytes"
    );

    // A barrier forces fifty producers to overlap. Each buffer is named so we can
    // identify its serial without relying on addresses that the allocator may reuse.
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(50));
    let workers: Vec<_> = (0..50)
        .map(|index| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut data = Vec::<u8>::with_capacity(80 + index);
                barrier.wait();
                let tracker = Tracker::global();
                tracker.track_as(
                    &data,
                    "concurrent_hook_buffer",
                    file!(),
                    line!(),
                    module_path!(),
                );
                data.resize(2048 + index, 42);
                assert_eq!(
                    data[index], 42,
                    "concurrent resize must retain initialized data"
                );
            })
        })
        .collect();
    for worker in workers {
        worker
            .join()
            .expect("allocator producer must complete without deadlock");
    }
    let events = tracker.events();
    let serials: std::collections::HashSet<_> = events
        .iter()
        .filter(|event| event.var_name.as_deref() == Some("concurrent_hook_buffer"))
        .filter_map(|event| event.binding_serial)
        .collect();
    assert_eq!(
        serials.len(),
        50,
        "each producer must retain a distinct binding identity"
    );
    let workload: Vec<_> = events
        .into_iter()
        .filter(|event| {
            event
                .binding_serial
                .is_some_and(|serial| serials.contains(&serial))
        })
        .collect();
    let snapshot = memscope_rs::snapshot::build_snapshot_from_events(&workload);
    assert_eq!(
        snapshot.stats.total_allocations, 50,
        "fifty producers must not produce duplicate allocations"
    );
    assert_eq!(
        snapshot.stats.total_reallocations, 50,
        "each growth is recorded exactly once"
    );
    assert_eq!(
        snapshot.current_memory(),
        0,
        "all real drops must reach the event stream"
    );
}
