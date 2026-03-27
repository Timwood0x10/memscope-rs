//! Unified tracking snapshot
//!
//! This is the core innovation that unifies all tracking strategies
//! into a single data model that can be used by all renderers.

use serde::{Deserialize, Serialize};
use super::common::{TrackingStrategy, current_timestamp};
use super::allocation::AllocationRecord;
use super::event::MemoryEvent;
use super::task::TaskRecord;
use super::stats::TrackingStats;

/// Unified tracking snapshot
///
// **This is the core innovation** - Unifying data from all strategies into one structure
///
/// Key ideas:
/// - Core strategy: Populates the `allocations` field
/// - Lockfree strategy: Populates the `events` field
/// - Async strategy: Populates the `tasks` field
/// - Unified strategy: Populates all fields
///
// This way:
/// 1. All strategies can be mapped to this structure
/// 2. The exporter does not need to know the specific strategy
/// 3. HTML can automatically generate different views based on the fields

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSnapshot {
    /// Tracking strategy used
    pub strategy: TrackingStrategy,
    
    /// Allocation records (Core strategy primarily)
    pub allocations: Vec<AllocationRecord>,
    
    /// Memory events (Lockfree strategy primarily)
    pub events: Vec<MemoryEvent>,
    
    /// Task records (Async strategy primarily)
    pub tasks: Vec<TaskRecord>,
    
    /// Overall statistics (all strategies)
    pub stats: TrackingStats,
    
    /// Snapshot timestamp
    pub timestamp: u64,
}

impl TrackingSnapshot {
    /// Create new empty snapshot
    pub fn new(strategy: TrackingStrategy) -> Self {
        Self {
            strategy,
            allocations: Vec::new(),
            events: Vec::new(),
            tasks: Vec::new(),
            stats: TrackingStats::new(),
            timestamp: current_timestamp(),
        }
    }

    /// Add allocation record
    pub fn add_allocation(&mut self, allocation: AllocationRecord) {
        self.allocations.push(allocation);
        self.stats.total_allocations += 1;
        self.stats.total_allocated += allocation.size as u64;
    }

    /// Add memory event
    pub fn add_event(&mut self, event: MemoryEvent) {
        self.events.push(event);
        match event.event_type {
            crate::data::event::EventType::Alloc
            | crate::data::event::EventType::Realloc
            | crate::data::event::EventType::TaskSpawn
            | crate::data::event::EventType::FfiAlloc => {
                self.stats.total_allocations += 1;
            }
            crate::data::event::EventType::Dealloc
            | crate::data::event::EventType::TaskEnd
            | crate::data::event::EventType::FfiFree => {
                self.stats.total_deallocations += 1;
            }
        }
    }

    /// Add task record
    pub fn add_task(&mut self, task: TaskRecord) {
        self.tasks.push(task);
    }

    /// Get active allocations
    pub fn active_allocations(&self) -> Vec<&AllocationRecord> {
        self.allocations.iter().filter(|a| a.is_active).collect()
    }

    /// Get leaked allocations
    pub fn leaked_allocations(&self) -> Vec<&AllocationRecord> {
        self.allocations.iter()
            .filter(|a| !a.is_active && a.dealloc_timestamp.is_some())
            .collect()
    }

    /// Get top N allocations by size
    pub fn top_allocations(&self, n: usize) -> Vec<&AllocationRecord> {
        let mut allocs = self.allocations.clone();
        allocs.sort_by(|a, b| b.size.cmp(&a.size));
        allocs.iter().take(n).collect()
    }

    /// Get tasks with potential leaks
    pub fn leaked_tasks(&self) -> Vec<&TaskRecord> {
        self.tasks.iter().filter(|t| t.has_leak()).collect()
    }

    /// Calculate statistics from snapshot
    pub fn calculate_stats(&mut self) {
        // Update active allocations
        self.stats.active_allocations = self.active_allocations().len() as u64;
        self.stats.active_memory = self.active_allocations()
            .iter()
            .map(|a| a.size)
            .sum::<usize>() as u64;

        // Update leaked allocations
        self.stats.leaked_allocations = self.leaked_allocations().len() as u64;
        self.stats.leaked_memory = self.leaked_allocations()
            .iter()
            .map(|a| a.size)
            .sum::<usize>() as u64;

        // Calculate peak memory from tasks
        for task in &self.tasks {
            if task.peak_memory > self.stats.peak_memory {
                self.stats.peak_memory = task.peak_memory as u64;
            }
        }

        // Calculate fragmentation
        self.stats.calculate_fragmentation();
    }

    /// Get memory usage summary
    pub fn memory_summary(&self) -> String {
        format!(
            "Total: {} bytes, Active: {} bytes, Peak: {} bytes, Leaked: {} bytes",
            self.stats.total_allocated,
            self.stats.active_memory,
            self.stats.peak_memory,
            self.stats.leaked_memory
        )
    }

    /// Get allocation summary
    pub fn allocation_summary(&self) -> String {
        format!(
            "Total: {}, Active: {}, Leaked: {}",
            self.stats.total_allocations,
            self.stats.active_allocations,
            self.stats.leaked_allocations
        )
    }
}

impl Default for TrackingSnapshot {
    fn default() -> Self {
        Self::new(TrackingStrategy::Unified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_snapshot_new() {
        let snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        assert_eq!(snapshot.strategy, TrackingStrategy::Core);
        assert_eq!(snapshot.allocations.len(), 0);
        assert_eq!(snapshot.events.len(), 0);
        assert_eq!(snapshot.tasks.len(), 0);
    }

    #[test]
    fn test_tracking_snapshot_default() {
        let snapshot = TrackingSnapshot::default();
        assert_eq!(snapshot.strategy, TrackingStrategy::Unified);
    }

    #[test]
    fn test_add_allocation() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        let alloc = AllocationRecord::new(0x1000, 1024);
        
        snapshot.add_allocation(alloc);
        
        assert_eq!(snapshot.allocations.len(), 1);
        assert_eq!(snapshot.stats.total_allocations, 1);
    }

    #[test]
    fn test_add_event() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Lockfree);
        let event = MemoryEvent::alloc(0x1000, 1024);
        
        snapshot.add_event(event);
        
        assert_eq!(snapshot.events.len(), 1);
        assert_eq!(snapshot.stats.total_allocations, 1);
    }

    #[test]
    fn test_add_task() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Async);
        let task = TaskRecord::new(1,"task_one".to_string());
        
        snapshot.add_task(task);
        
        assert_eq!(snapshot.tasks.len(), 1);
    }

    #[test]
    fn test_active_allocations() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        let mut alloc1 = AllocationRecord::new(0x1000, 1024);
        let mut alloc2 = AllocationRecord::new(0x2000, 2048);
        
        alloc2.deallocate();
        
        snapshot.add_allocation(alloc1);
        snapshot.add_allocation(alloc2);
        
        let active = snapshot.active_allocations();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].ptr, 0x1000);
    }

    #[test]
    fn test_leaked_allocations() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        let mut alloc = AllocationRecord::new(0x1000, 1024);
        alloc.deallocate();
        
        snapshot.add_allocation(alloc);
        
        let leaked = snapshot.leaked_allocations();
        assert_eq!(leaked.len(), 1);
    }

    #[test]
    fn test_top_allocations() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        snapshot.add_allocation(AllocationRecord::new(0x1000, 512));
        snapshot.add_allocation(AllocationRecord::new(0x2000, 2048));
        snapshot.add_allocation(AllocationRecord::new(0x3000, 1024));
        
        let top = snapshot.top_allocations(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].size, 2048);  // Largest
        assert_eq!(top[1].size, 1024);  // Second largest
    }

    #[test]
    fn test_leaked_tasks() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Async);
        
        let mut task1 = TaskRecord::new(1,"task_one".to_string());
        let mut task2 = TaskRecord::new(2,"task_two".to_string());
        
        task1.record_allocation(1024);
        task1.complete();
        
        task2.record_allocation(2048);
        
        snapshot.add_task(task1);
        snapshot.add_task(task2);
        
        let leaked = snapshot.leaked_tasks();
        assert_eq!(leaked.len(), 1);
        assert_eq!(leaked[0].task_id, 1);
    }

    #[test]
    fn test_calculate_stats() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        let mut alloc1 = AllocationRecord::new(0x1000, 1024);
        let mut alloc2 = AllocationRecord::new(0x2000, 2048);
        alloc2.deallocate();
        
        snapshot.add_allocation(alloc1);
        snapshot.add_allocation(alloc2);
        
        snapshot.calculate_stats();
        
        assert_eq!(snapshot.stats.active_allocations, 1);
        assert_eq!(snapshot.stats.active_memory, 1024);
        assert_eq!(snapshot.stats.leaked_allocations, 1);
        assert_eq!(snapshot.stats.leaked_memory, 2048);
        assert_eq!(snapshot.stats.total_allocated, 3072);
    }

    #[test]
    fn test_memory_summary() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        let mut alloc = AllocationRecord::new(0x1000, 1024);
        snapshot.add_allocation(alloc);
        snapshot.calculate_stats();
        
        let summary = snapshot.memory_summary();
        assert!(summary.contains("Total: 1024 bytes"));
        assert!(summary.contains("Active: 1024 bytes"));
    }

    #[test]
    fn test_allocation_summary() {
        let mut snapshot = TrackingSnapshot::new(TrackingStrategy::Core);
        
        let mut alloc = AllocationRecord::new(0x1000, 1024);
        snapshot.add_allocation(alloc);
        snapshot.calculate_stats();
        
        let summary = snapshot.allocation_summary();
        assert!(summary.contains("Total: 1"));
        assert!(summary.contains("Active: 1"));
    }
}