//! Integration tests for Capture Backends

use memscope_rs::capture::backends::{
    AsyncBackend, CaptureBackend, CaptureBackendType, CoreBackend, LockfreeBackend,
    UnifiedCaptureBackend,
};
use memscope_rs::event_store::MemoryEventType;

#[test]
fn test_core_backend_alloc() {
    let backend = CoreBackend;
    let event = backend.capture_alloc(0x1000, 1024, 1);

    assert_eq!(event.ptr, 0x1000);
    assert_eq!(event.size, 1024);
    assert_eq!(event.thread_id, 1);
    assert!(matches!(event.event_type, MemoryEventType::Allocate));
}

#[test]
fn test_core_backend_dealloc() {
    let backend = CoreBackend;
    let event = backend.capture_dealloc(0x1000, 1024, 1);

    assert_eq!(event.ptr, 0x1000);
    assert!(matches!(event.event_type, MemoryEventType::Deallocate));
}

#[test]
fn test_core_backend_realloc() {
    let backend = CoreBackend;
    let event = backend.capture_realloc(0x1000, 1024, 2048, 1);

    assert_eq!(event.ptr, 0x1000);
    assert!(matches!(event.event_type, MemoryEventType::Reallocate));
}

#[test]
fn test_lockfree_backend_alloc() {
    let backend = LockfreeBackend;
    let event = backend.capture_alloc(0x2000, 2048, 2);

    assert_eq!(event.ptr, 0x2000);
    assert_eq!(event.size, 2048);
    assert!(event.call_stack_hash.is_some());
}

#[test]
fn test_lockfree_backend_dealloc() {
    let backend = LockfreeBackend;
    let event = backend.capture_dealloc(0x2000, 2048, 2);

    assert_eq!(event.ptr, 0x2000);
    assert!(event.call_stack_hash.is_some());
}

#[test]
fn test_async_backend_alloc() {
    let backend = AsyncBackend;
    let event = backend.capture_alloc(0x3000, 4096, 3);

    assert_eq!(event.ptr, 0x3000);
    assert_eq!(event.size, 4096);
}

#[test]
fn test_unified_backend_creation() {
    let backend = UnifiedCaptureBackend::new();
    let backend_type = backend.backend_type();

    assert!(matches!(
        backend_type,
        CaptureBackendType::Core | CaptureBackendType::Lockfree
    ));
}

#[test]
fn test_unified_backend_alloc() {
    let backend = UnifiedCaptureBackend::new();
    let event = backend.capture_alloc(0x4000, 8192, 4);

    assert_eq!(event.ptr, 0x4000);
    assert_eq!(event.size, 8192);
}

#[test]
fn test_backend_type_create_core() {
    let backend = CaptureBackendType::Core.create_backend();
    let event = backend.capture_alloc(0x1000, 100, 1);
    assert_eq!(event.ptr, 0x1000);
}

#[test]
fn test_backend_type_create_lockfree() {
    let backend = CaptureBackendType::Lockfree.create_backend();
    let event = backend.capture_alloc(0x2000, 200, 2);
    assert_eq!(event.ptr, 0x2000);
}

#[test]
fn test_backend_type_create_async() {
    let backend = CaptureBackendType::Async.create_backend();
    let event = backend.capture_alloc(0x3000, 300, 3);
    assert_eq!(event.ptr, 0x3000);
}

#[test]
fn test_backend_type_create_unified() {
    let backend = CaptureBackendType::Unified.create_backend();
    let event = backend.capture_alloc(0x4000, 400, 4);
    assert_eq!(event.ptr, 0x4000);
}

#[test]
fn test_backend_move_event() {
    let backend = CoreBackend;
    let event = backend.capture_move(0x1000, 0x2000, 1024, 1);

    assert_eq!(event.ptr, 0x2000);
    assert!(matches!(event.event_type, MemoryEventType::Move));
}

#[test]
fn test_multiple_backends_parallel() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                let backend = match i % 4 {
                    0 => CaptureBackendType::Core.create_backend(),
                    1 => CaptureBackendType::Lockfree.create_backend(),
                    2 => CaptureBackendType::Async.create_backend(),
                    _ => CaptureBackendType::Unified.create_backend(),
                };

                for j in 0..100 {
                    let event = backend.capture_alloc(0x1000 + j, 1024, i as u64);
                    assert_eq!(event.ptr, 0x1000 + j);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
