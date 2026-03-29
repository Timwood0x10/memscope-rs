//! Lockfree tracker types and data structures.
//!
//! This module contains type definitions for the lockfree memory tracking system.

use std::thread::ThreadId;

/// Memory event for lockfree tracking
#[derive(Debug, Clone)]
pub struct Event {
    /// Event timestamp
    pub timestamp: u64,
    /// Event type (allocation/deallocation)
    pub event_type: EventType,
    /// Memory pointer
    pub ptr: usize,
    /// Memory size
    pub size: usize,
    /// Call stack hash
    pub call_stack_hash: u64,
    /// Thread ID
    pub thread_id: ThreadId,
    /// Optional metadata
    pub metadata: Option<EventMetadata>,
}

impl Event {
    /// Create a new allocation event
    pub fn allocation(ptr: usize, size: usize, call_stack_hash: u64, thread_id: ThreadId) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: EventType::Allocation,
            ptr,
            size,
            call_stack_hash,
            thread_id,
            metadata: None,
        }
    }

    /// Create a new deallocation event
    pub fn deallocation(
        ptr: usize,
        size: usize,
        call_stack_hash: u64,
        thread_id: ThreadId,
    ) -> Self {
        Self {
            timestamp: Self::now(),
            event_type: EventType::Deallocation,
            ptr,
            size,
            call_stack_hash,
            thread_id,
            metadata: None,
        }
    }

    /// Get current timestamp
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

/// Event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Memory allocation event
    Allocation,
    /// Memory deallocation event
    Deallocation,
}

/// Event metadata
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// Optional variable name
    pub var_name: Option<String>,
    /// Optional type name
    pub type_name: Option<String>,
}

/// Frequency data for allocation patterns
#[derive(Debug, Clone)]
pub struct FrequencyData {
    /// Call stack hash
    pub call_stack_hash: u64,
    /// Number of allocations
    pub count: usize,
    /// Total size allocated
    pub total_size: usize,
    /// First allocation timestamp
    pub first_timestamp: u64,
    /// Last allocation timestamp
    pub last_timestamp: u64,
}

/// Allocation category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationCategory {
    /// Small allocations (< 1KB)
    Small,
    /// Medium allocations (1KB - 1MB)
    Medium,
    /// Large allocations (> 1MB)
    Large,
}

/// Memory statistics for lockfree tracking
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total number of allocations
    pub total_allocations: usize,
    /// Total allocated memory
    pub total_allocated: usize,
    /// Total number of deallocations
    pub total_deallocations: usize,
    /// Total deallocated memory
    pub total_deallocated: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Current active memory
    pub active_memory: usize,
}

/// Real call stack information
#[derive(Debug, Clone)]
pub struct RealCallStack {
    /// Stack frames
    pub frames: Vec<StackFrame>,
    /// Stack hash
    pub hash: u64,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Instruction pointer
    pub ip: usize,
    /// Function name
    pub function_name: Option<String>,
    /// File name
    pub file_name: Option<String>,
    /// Line number
    pub line_number: Option<u32>,
}

/// System metrics collection
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Number of active threads
    pub active_threads: usize,
}

/// Analysis data for lockfree tracking
#[derive(Debug, Clone, Default)]
pub struct AnalysisData {
    /// Memory statistics
    pub stats: MemoryStats,
    /// System metrics
    pub system_metrics: SystemMetrics,
    /// Frequency data
    pub frequency_data: Vec<FrequencyData>,
    /// Timestamp of analysis
    pub timestamp: u64,
}

/// Frequency pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyPattern {
    /// No clear pattern
    None,
    /// Frequent small allocations
    FrequentSmall,
    /// Infrequent large allocations
    InfrequentLarge,
    /// Mixed pattern
    Mixed,
}

/// Access pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPattern {
    /// Sequential access pattern
    Sequential,
    /// Random access pattern
    Random,
    /// Unknown pattern
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let thread_id = std::thread::current().id();
        let alloc_event = Event::allocation(0x1000, 1024, 12345, thread_id);
        assert_eq!(alloc_event.event_type, EventType::Allocation);
        assert_eq!(alloc_event.ptr, 0x1000);
        assert_eq!(alloc_event.size, 1024);

        let dealloc_event = Event::deallocation(0x1000, 1024, 12345, thread_id);
        assert_eq!(dealloc_event.event_type, EventType::Deallocation);
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.peak_memory, 0);
    }

    #[test]
    fn test_allocation_category() {
        let small = AllocationCategory::Small;
        let medium = AllocationCategory::Medium;
        let large = AllocationCategory::Large;

        assert_eq!(small, AllocationCategory::Small);
        assert_eq!(medium, AllocationCategory::Medium);
        assert_eq!(large, AllocationCategory::Large);
        assert_ne!(small, medium);
    }
}
