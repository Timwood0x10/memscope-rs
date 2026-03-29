//! Capture Engine - Event capture backend
//!
//! This module provides the CaptureEngine which is responsible for
//! capturing memory events from the application and forwarding them
//! to the EventStore. The CaptureEngine does not store events itself;
//! it only captures and forwards them.

use crate::capture::backends::{CaptureBackend, CaptureBackendType};
use crate::event_store::SharedEventStore;

/// Capture Engine - Event capture backend
///
/// The CaptureEngine is responsible for capturing memory events from
/// the application and forwarding them to the EventStore. It does not
/// store events itself; it only captures and forwards them.
///
/// Key properties:
/// - Non-blocking: All capture operations are non-blocking
/// - Backend abstraction: Supports multiple capture backends
/// - Zero-storage: Events are forwarded to EventStore, not stored locally
pub struct CaptureEngine {
    /// The capture backend being used
    backend: Box<dyn CaptureBackend>,
    /// Reference to the event store for recording events
    event_store: SharedEventStore,
}

impl CaptureEngine {
    /// Create a new CaptureEngine with the specified backend
    ///
    /// # Arguments
    /// * `backend_type` - The type of capture backend to use
    /// * `event_store` - Reference to the event store
    pub fn new(backend_type: CaptureBackendType, event_store: SharedEventStore) -> Self {
        let backend = backend_type.create_backend();
        Self {
            backend,
            event_store,
        }
    }

    /// Capture an allocation event
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Allocation size in bytes
    /// * `thread_id` - Thread identifier
    pub fn capture_alloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_alloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a deallocation event
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `size` - Deallocation size in bytes
    /// * `thread_id` - Thread identifier
    pub fn capture_dealloc(&self, ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_dealloc(ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a reallocation event
    ///
    /// # Arguments
    /// * `ptr` - Memory pointer address
    /// * `old_size` - Old allocation size in bytes
    /// * `new_size` - New allocation size in bytes
    /// * `thread_id` - Thread identifier
    pub fn capture_realloc(&self, ptr: usize, old_size: usize, new_size: usize, thread_id: u64) {
        let event = self
            .backend
            .capture_realloc(ptr, old_size, new_size, thread_id);
        self.event_store.record(event);
    }

    /// Capture a move event
    ///
    /// # Arguments
    /// * `from_ptr` - Source pointer address
    /// * `to_ptr` - Destination pointer address
    /// * `size` - Size in bytes
    /// * `thread_id` - Thread identifier
    pub fn capture_move(&self, from_ptr: usize, to_ptr: usize, size: usize, thread_id: u64) {
        let event = self.backend.capture_move(from_ptr, to_ptr, size, thread_id);
        self.event_store.record(event);
    }

    /// Get the event store reference
    pub fn event_store(&self) -> &SharedEventStore {
        &self.event_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::EventStore;
    use std::sync::Arc;

    #[test]
    fn test_capture_engine_creation() {
        let event_store = Arc::new(EventStore::new());
        let engine = CaptureEngine::new(CaptureBackendType::Core, event_store);
        assert!(engine.event_store().is_empty());
    }

    #[test]
    fn test_capture_alloc() {
        let event_store = Arc::new(EventStore::new());
        let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
        engine.capture_alloc(0x1000, 1024, 1);
        assert_eq!(event_store.len(), 1);
    }

    #[test]
    fn test_capture_dealloc() {
        let event_store = Arc::new(EventStore::new());
        let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
        engine.capture_dealloc(0x1000, 1024, 1);
        assert_eq!(event_store.len(), 1);
    }

    #[test]
    fn test_capture_multiple_events() {
        let event_store = Arc::new(EventStore::new());
        let engine = CaptureEngine::new(CaptureBackendType::Core, event_store.clone());
        engine.capture_alloc(0x1000, 1024, 1);
        engine.capture_dealloc(0x1000, 1024, 1);
        engine.capture_alloc(0x2000, 2048, 1);
        assert_eq!(event_store.len(), 3);
    }
}
