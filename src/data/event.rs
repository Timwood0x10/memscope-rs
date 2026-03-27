//! Memory event structures
//!
//! Used by Lockfree strategy to track high-performance events

use serde::{Deserialize, Serialize};
use super::common::{current_timestamp, current_thread_id};

/// Memory event record
///
/// Used by Lockfree strategy to track high-performance events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Memory address
    pub ptr: usize,
    /// Size in bytes
    pub size: usize,
    /// Event type
    pub event_type: EventType,
    /// Event timestamp (nanoseconds since Unix epoch)
    pub timestamp: u64,
    /// Thread ID
    pub thread_id: u32,
    /// Optional call stack hash
    pub stack_hash: Option<u64>,
    /// Optional CPU time (nanoseconds)
    pub cpu_time_ns: Option<u64>,
}

impl MemoryEvent {
    /// Create new memory event
    pub fn new(ptr: usize, size: usize, event_type: EventType) -> Self {
        Self {
            ptr,
            size,
            event_type,
            timestamp: current_timestamp(),
            thread_id: current_thread_id(),
            stack_hash: None,
            cpu_time_ns: None,
        }
    }

    /// Create allocation event
    pub fn alloc(ptr: usize, size: usize) -> Self {
        Self::new(ptr, size, EventType::Alloc)
    }

    /// Create deallocation event
    pub fn dealloc(ptr: usize) -> Self {
        Self::new(ptr, 0, EventType::Dealloc)
    }

    /// Create reallocation event
    pub fn realloc(old_ptr: usize, new_ptr: usize, size: usize) -> Self {
        Self::new(new_ptr, size, EventType::Realloc)
    }

    /// Set call stack hash
    pub fn with_stack_hash(mut self, hash: u64) -> Self {
        self.stack_hash = Some(hash);
        self
    }

    /// Set CPU time
    pub fn with_cpu_time(mut self, time_ns: u64) -> Self {
        self.cpu_time_ns = Some(time_ns);
        self
    }
}

/// Memory event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Memory allocation
    Alloc,
    /// Memory deallocation
    Dealloc,
    /// Memory reallocation
    Realloc,
    /// Task spawn (async)
    TaskSpawn,
    /// Task end (async)
    TaskEnd,
    /// FFI allocation
    FfiAlloc,
    /// FFI free
    FfiFree,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Alloc => write!(f, "Alloc"),
            EventType::Dealloc => write!(f, "Dealloc"),
            EventType::Realloc => write!(f, "Realloc"),
            EventType::TaskSpawn => write!(f, "TaskSpawn"),
            EventType::TaskEnd => write!(f, "TaskEnd"),
            EventType::FfiAlloc => write!(f, "FfiAlloc"),
            EventType::FfiFree => write!(f, "FfiFree"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_event_alloc() {
        let event = MemoryEvent::alloc(0x1000, 1024);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.size, 1024);
        assert_eq!(event.event_type, EventType::Alloc);
    }

    #[test]
    fn test_memory_event_dealloc() {
        let event = MemoryEvent::dealloc(0x1000);
        assert_eq!(event.ptr, 0x1000);
        assert_eq!(event.event_type, EventType::Dealloc);
    }

    #[test]
    fn test_memory_event_builder() {
        let event = MemoryEvent::alloc(0x1000, 1024)
            .with_stack_hash(123456)
            .with_cpu_time(1000);
        
        assert_eq!(event.stack_hash, Some(123456));
        assert_eq!(event.cpu_time_ns, Some(1000));
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::Alloc.to_string(), "Alloc");
        assert_eq!(EventType::Dealloc.to_string(), "Dealloc");
        assert_eq!(EventType::Realloc.to_string(), "Realloc");
    }
}