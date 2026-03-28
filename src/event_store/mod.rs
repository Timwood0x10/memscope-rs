//! Event Store Engine
//!
//! This module provides the EventStore which is responsible for storing
//! all memory events across all tracking backends. It uses a lock-free
//! queue for high-performance concurrent access.

pub mod event;
pub mod store;

pub use event::{MemoryEvent, MemoryEventType};
pub use store::{EventStore, SharedEventStore};