//! Core memory event types for the event store
//!
//! This module defines the unified memory event type used across
//! all engines in the memscope architecture.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of memory event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryEventType {
    /// Memory allocation event
    Allocate,
    /// Memory deallocation event
    Deallocate,
    /// Memory reallocation event
    Reallocate,
    /// Memory move event
    Move,
    /// Memory borrow event
    Borrow,
    /// Memory return event
    Return,
}

impl fmt::Display for MemoryEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryEventType::Allocate => write!(f, "Allocate"),
            MemoryEventType::Deallocate => write!(f, "Deallocate"),
            MemoryEventType::Reallocate => write!(f, "Reallocate"),
            MemoryEventType::Move => write!(f, "Move"),
            MemoryEventType::Borrow => write!(f, "Borrow"),
            MemoryEventType::Return => write!(f, "Return"),
        }
    }
}

/// Unified memory event type
///
/// This structure captures all essential information about memory operations
/// across all tracking backends (core, lockfree, async, unified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Event timestamp (nanoseconds since epoch)
    pub timestamp: u64,
    /// Event type
    pub event_type: MemoryEventType,
    /// Memory pointer address
    pub ptr: usize,
    /// Allocation size in bytes
    pub size: usize,
    /// Previous allocation size (for Reallocate events)
    pub old_size: Option<usize>,
    /// Thread identifier
    pub thread_id: u64,
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
    /// Optional call stack hash
    pub call_stack_hash: Option<u64>,
    /// Optional thread name
    pub thread_name: Option<String>,
}

impl MemoryEvent {
    /// Create a new allocation event
    pub fn allocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Allocate,
            ptr,
            size,
            old_size: None,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Create a new deallocation event
    pub fn deallocate(ptr: usize, size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Deallocate,
            ptr,
            size,
            old_size: None,
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Create a new reallocation event
    pub fn reallocate(ptr: usize, old_size: usize, new_size: usize, thread_id: u64) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Reallocate,
            ptr,
            size: new_size,
            old_size: Some(old_size),
            thread_id,
            var_name: None,
            type_name: None,
            call_stack_hash: None,
            thread_name: None,
        }
    }

    /// Get current timestamp in nanoseconds
    /// Returns 0 if system time is before Unix epoch (should not happen in practice)
    pub fn now() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or_default()
    }

    /// Set variable name
    pub fn with_var_name(mut self, name: String) -> Self {
        self.var_name = Some(name);
        self
    }

    /// Set type name
    pub fn with_type_name(mut self, name: String) -> Self {
        self.type_name = Some(name);
        self
    }

    /// Set call stack hash
    pub fn with_call_stack_hash(mut self, hash: u64) -> Self {
        self.call_stack_hash = Some(hash);
        self
    }

    /// Set thread name
    pub fn with_thread_name(mut self, name: String) -> Self {
        self.thread_name = Some(name);
        self
    }

    /// Check if this is an allocation event
    pub fn is_allocation(&self) -> bool {
        matches!(
            self.event_type,
            MemoryEventType::Allocate | MemoryEventType::Reallocate
        )
    }

    /// Check if this is a deallocation event
    pub fn is_deallocation(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Deallocate)
    }

    /// Check if this is a move event
    pub fn is_move(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Move)
    }

    /// Check if this is a borrow event
    pub fn is_borrow(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Borrow)
    }

    /// Check if this is a return event
    pub fn is_return(&self) -> bool {
        matches!(self.event_type, MemoryEventType::Return)
    }
}
