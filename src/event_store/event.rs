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
    /// Metadata event for Container/Value types (no heap allocation)
    Metadata,
    /// Clone event
    Clone,
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
            MemoryEventType::Metadata => write!(f, "Metadata"),
            MemoryEventType::Clone => write!(f, "Clone"),
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
    /// Optional source file path
    pub source_file: Option<String>,
    /// Optional source line number
    pub source_line: Option<u32>,
    /// Optional module path
    pub module_path: Option<String>,
    /// Clone source pointer (for Clone events)
    pub clone_source_ptr: Option<usize>,
    /// Clone target pointer (for Clone events)
    pub clone_target_ptr: Option<usize>,
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
            source_file: None,
            source_line: None,
            module_path: None,
            clone_source_ptr: None,
            clone_target_ptr: None,
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
            source_file: None,
            source_line: None,
            module_path: None,
            clone_source_ptr: None,
            clone_target_ptr: None,
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
            source_file: None,
            source_line: None,
            module_path: None,
            clone_source_ptr: None,
            clone_target_ptr: None,
        }
    }

    /// Create a new metadata event for Container/Value types (no heap allocation)
    pub fn metadata(var_name: String, type_name: String, thread_id: u64, size: usize) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Metadata,
            ptr: 0, // No heap pointer
            size,
            old_size: None,
            thread_id,
            var_name: Some(var_name),
            type_name: Some(type_name),
            call_stack_hash: None,
            thread_name: None,
            source_file: None,
            source_line: None,
            module_path: None,
            clone_source_ptr: None,
            clone_target_ptr: None,
        }
    }

    /// Create a new clone event
    pub fn clone_event(
        source_ptr: usize,
        target_ptr: usize,
        size: usize,
        thread_id: u64,
        var_name: Option<String>,
        type_name: Option<String>,
    ) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: MemoryEventType::Clone,
            ptr: target_ptr,
            size,
            old_size: None,
            thread_id,
            var_name,
            type_name,
            call_stack_hash: None,
            thread_name: None,
            source_file: None,
            source_line: None,
            module_path: None,
            clone_source_ptr: Some(source_ptr),
            clone_target_ptr: Some(target_ptr),
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

    /// Set source file path
    pub fn with_source_file(mut self, file: String) -> Self {
        self.source_file = Some(file);
        self
    }

    /// Set source line number
    pub fn with_source_line(mut self, line: u32) -> Self {
        self.source_line = Some(line);
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
            MemoryEventType::Allocate | MemoryEventType::Reallocate | MemoryEventType::Metadata
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
