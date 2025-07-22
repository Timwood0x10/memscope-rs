//! Borrow checker integration and analysis
//!
//! This module implements borrow tracking features from ComplexTypeForRust.md:
//! - Track borrow and mutable borrow lifetimes
//! - Runtime borrow checking integration
//! - Borrow conflict detection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global borrow analyzer instance
static GLOBAL_BORROW_ANALYZER: OnceLock<Arc<BorrowAnalyzer>> = OnceLock::new();

/// Get the global borrow analyzer instance
pub fn get_global_borrow_analyzer() -> Arc<BorrowAnalyzer> {
    GLOBAL_BORROW_ANALYZER
        .get_or_init(|| Arc::new(BorrowAnalyzer::new()))
        .clone()
}

/// Borrow analysis system
pub struct BorrowAnalyzer {
    /// Active borrows tracking
    active_borrows: Mutex<HashMap<usize, Vec<BorrowInfo>>>,
    /// Borrow history for analysis
    borrow_history: Mutex<Vec<BorrowEvent>>,
    /// Detected borrow conflicts
    conflicts: Mutex<Vec<BorrowConflict>>,
}

impl BorrowAnalyzer {
    /// Create a new borrow analyzer
    pub fn new() -> Self {
        Self {
            active_borrows: Mutex::new(HashMap::new()),
            borrow_history: Mutex::new(Vec::new()),
            conflicts: Mutex::new(Vec::new()),
        }
    }

    /// Track a new borrow
    pub fn track_borrow(&self, ptr: usize, borrow_type: BorrowType, var_name: &str) -> BorrowId {
        let borrow_id = BorrowId::new();
        let borrow_info = BorrowInfo {
            id: borrow_id,
            ptr,
            borrow_type: borrow_type.clone(),
            var_name: var_name.to_string(),
            start_time: current_timestamp(),
            end_time: None,
            thread_id: format!("{:?}", std::thread::current().id()),
            call_stack: capture_call_stack(),
        };

        // Check for conflicts before adding
        self.check_borrow_conflicts(ptr, &borrow_type, &borrow_info);

        // Add to active borrows
        if let Ok(mut active) = self.active_borrows.lock() {
            active.entry(ptr).or_insert_with(Vec::new).push(borrow_info.clone());
        }

        // Record the borrow event
        let event = BorrowEvent {
            borrow_info: borrow_info.clone(),
            event_type: BorrowEventType::BorrowStart,
            timestamp: current_timestamp(),
        };

        if let Ok(mut history) = self.borrow_history.lock() {
            history.push(event);
        }

        borrow_id
    }

    /// End a borrow
    pub fn end_borrow(&self, borrow_id: BorrowId) {
        let end_time = current_timestamp();

        // Find and remove from active borrows
        if let Ok(mut active) = self.active_borrows.lock() {
            for (_, borrows) in active.iter_mut() {
                if let Some(pos) = borrows.iter().position(|b| b.id == borrow_id) {
                    let mut borrow_info = borrows.remove(pos);
                    borrow_info.end_time = Some(end_time);

                    // Record the end event
                    let event = BorrowEvent {
                        borrow_info: borrow_info.clone(),
                        event_type: BorrowEventType::BorrowEnd,
                        timestamp: end_time,
                    };

                    if let Ok(mut history) = self.borrow_history.lock() {
                        history.push(event);
                    }
                    break;
                }
            }
        }
    }

    /// Check for borrow conflicts
    fn check_borrow_conflicts(&self, ptr: usize, new_borrow_type: &BorrowType, new_borrow: &BorrowInfo) {
        if let Ok(active) = self.active_borrows.lock() {
            if let Some(existing_borrows) = active.get(&ptr) {
                for existing in existing_borrows {
                    if self.is_conflicting_borrow(&existing.borrow_type, new_borrow_type) {
                        let conflict = BorrowConflict {
                            ptr,
                            existing_borrow: existing.clone(),
                            conflicting_borrow: new_borrow.clone(),
                            conflict_type: self.determine_conflict_type(&existing.borrow_type, new_borrow_type),
                            timestamp: current_timestamp(),
                        };

                        if let Ok(mut conflicts) = self.conflicts.lock() {
                            conflicts.push(conflict);
                        }
                    }
                }
            }
        }
    }

    /// Check if two borrow types conflict
    fn is_conflicting_borrow(&self, existing: &BorrowType, new: &BorrowType) -> bool {
        match (existing, new) {
            // Mutable borrow conflicts with any other borrow
            (BorrowType::Mutable, _) | (_, BorrowType::Mutable) => true,
            // Multiple immutable borrows are allowed
            (BorrowType::Immutable, BorrowType::Immutable) => false,
            // Shared borrows don't conflict with immutable
            (BorrowType::Shared, BorrowType::Immutable) | (BorrowType::Immutable, BorrowType::Shared) => false,
            // Other combinations are safe
            _ => false,
        }
    }

    /// Determine the type of conflict
    fn determine_conflict_type(&self, existing: &BorrowType, new: &BorrowType) -> ConflictType {
        match (existing, new) {
            (BorrowType::Mutable, BorrowType::Mutable) => ConflictType::MultipleMutableBorrows,
            (BorrowType::Mutable, BorrowType::Immutable) | (BorrowType::Immutable, BorrowType::Mutable) => {
                ConflictType::MutableImmutableConflict
            }
            _ => ConflictType::Other,
        }
    }

    /// Get current active borrows for a pointer
    pub fn get_active_borrows(&self, ptr: usize) -> Vec<BorrowInfo> {
        if let Ok(active) = self.active_borrows.lock() {
            active.get(&ptr).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Get borrow statistics
    pub fn get_borrow_statistics(&self) -> BorrowStatistics {
        let history = self.borrow_history.lock().unwrap();
        let conflicts = self.conflicts.lock().unwrap();
        let active = self.active_borrows.lock().unwrap();

        let total_borrows = history.len();
        let active_borrows: usize = active.values().map(|v| v.len()).sum();
        let total_conflicts = conflicts.len();

        // Calculate borrow duration statistics
        let mut durations = Vec::new();
        for event in history.iter() {
            if let Some(end_time) = event.borrow_info.end_time {
                durations.push(end_time - event.borrow_info.start_time);
            }
        }

        let avg_borrow_duration = if !durations.is_empty() {
            durations.iter().sum::<u64>() / durations.len() as u64
        } else {
            0
        };

        let max_borrow_duration = durations.iter().max().copied().unwrap_or(0);

        // Count by borrow type
        let mut by_type = HashMap::new();
        for event in history.iter() {
            let type_name = format!("{:?}", event.borrow_info.borrow_type);
            *by_type.entry(type_name).or_insert(0) += 1;
        }

        BorrowStatistics {
            total_borrows,
            active_borrows,
            total_conflicts,
            avg_borrow_duration,
            max_borrow_duration,
            by_type,
        }
    }

    /// Get all detected conflicts
    pub fn get_conflicts(&self) -> Vec<BorrowConflict> {
        self.conflicts.lock().unwrap().clone()
    }

    /// Analyze borrow patterns
    pub fn analyze_borrow_patterns(&self) -> BorrowPatternAnalysis {
        let history = self.borrow_history.lock().unwrap();
        let conflicts = self.conflicts.lock().unwrap();

        // Analyze common patterns
        let mut patterns = Vec::new();

        // Pattern: Long-lived borrows
        let long_lived_threshold = 1_000_000; // 1ms in nanoseconds
        let long_lived_count = history.iter()
            .filter(|event| {
                if let Some(end_time) = event.borrow_info.end_time {
                    end_time - event.borrow_info.start_time > long_lived_threshold
                } else {
                    false
                }
            })
            .count();

        if long_lived_count > 0 {
            patterns.push(BorrowPattern {
                pattern_type: BorrowPatternType::LongLivedBorrows,
                description: format!("{} borrows lasted longer than 1ms", long_lived_count),
                severity: if long_lived_count > 10 { PatternSeverity::Warning } else { PatternSeverity::Info },
                suggestion: "Consider reducing borrow scope or using RAII patterns".to_string(),
            });
        }

        // Pattern: Frequent conflicts
        if conflicts.len() > 5 {
            patterns.push(BorrowPattern {
                pattern_type: BorrowPatternType::FrequentConflicts,
                description: format!("{} borrow conflicts detected", conflicts.len()),
                severity: PatternSeverity::Warning,
                suggestion: "Review borrow patterns and consider refactoring to reduce conflicts".to_string(),
            });
        }

        // Pattern: Many concurrent borrows
        let max_concurrent = self.calculate_max_concurrent_borrows();
        if max_concurrent > 10 {
            patterns.push(BorrowPattern {
                pattern_type: BorrowPatternType::HighConcurrency,
                description: format!("Up to {} concurrent borrows detected", max_concurrent),
                severity: PatternSeverity::Info,
                suggestion: "High concurrency detected - ensure this is intentional".to_string(),
            });
        }

        BorrowPatternAnalysis {
            patterns,
            total_events: history.len(),
            analysis_timestamp: current_timestamp(),
        }
    }

    /// Calculate maximum concurrent borrows
    fn calculate_max_concurrent_borrows(&self) -> usize {
        let active = self.active_borrows.lock().unwrap();
        active.values().map(|v| v.len()).max().unwrap_or(0)
    }
}

/// Unique identifier for a borrow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BorrowId(u64);

impl BorrowId {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Types of borrows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowType {
    /// Immutable borrow (&T)
    Immutable,
    /// Mutable borrow (&mut T)
    Mutable,
    /// Shared reference (Arc, Rc)
    Shared,
    /// Weak reference
    Weak,
}

/// Information about a borrow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowInfo {
    /// Unique borrow identifier
    pub id: BorrowId,
    /// Pointer being borrowed
    pub ptr: usize,
    /// Type of borrow
    pub borrow_type: BorrowType,
    /// Variable name
    pub var_name: String,
    /// When the borrow started
    pub start_time: u64,
    /// When the borrow ended (if it has ended)
    pub end_time: Option<u64>,
    /// Thread where the borrow occurred
    pub thread_id: String,
    /// Call stack at borrow time
    pub call_stack: Vec<String>,
}

/// Borrow event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowEvent {
    /// Borrow information
    pub borrow_info: BorrowInfo,
    /// Type of event
    pub event_type: BorrowEventType,
    /// Event timestamp
    pub timestamp: u64,
}

/// Types of borrow events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowEventType {
    /// Borrow started
    BorrowStart,
    /// Borrow ended
    BorrowEnd,
}

/// Borrow conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowConflict {
    /// Pointer where conflict occurred
    pub ptr: usize,
    /// Existing borrow
    pub existing_borrow: BorrowInfo,
    /// Conflicting borrow attempt
    pub conflicting_borrow: BorrowInfo,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// When the conflict was detected
    pub timestamp: u64,
}

/// Types of borrow conflicts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Multiple mutable borrows
    MultipleMutableBorrows,
    /// Mutable and immutable borrow conflict
    MutableImmutableConflict,
    /// Other conflict type
    Other,
}

/// Borrow statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowStatistics {
    /// Total number of borrows tracked
    pub total_borrows: usize,
    /// Currently active borrows
    pub active_borrows: usize,
    /// Total conflicts detected
    pub total_conflicts: usize,
    /// Average borrow duration in nanoseconds
    pub avg_borrow_duration: u64,
    /// Maximum borrow duration in nanoseconds
    pub max_borrow_duration: u64,
    /// Count by borrow type
    pub by_type: HashMap<String, usize>,
}

/// Borrow pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowPatternAnalysis {
    /// Detected patterns
    pub patterns: Vec<BorrowPattern>,
    /// Total events analyzed
    pub total_events: usize,
    /// Analysis timestamp
    pub analysis_timestamp: u64,
}

/// Detected borrow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowPattern {
    /// Type of pattern
    pub pattern_type: BorrowPatternType,
    /// Description of the pattern
    pub description: String,
    /// Severity level
    pub severity: PatternSeverity,
    /// Suggested action
    pub suggestion: String,
}

/// Types of borrow patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BorrowPatternType {
    /// Long-lived borrows
    LongLivedBorrows,
    /// Frequent conflicts
    FrequentConflicts,
    /// High concurrency
    HighConcurrency,
    /// Nested borrows
    NestedBorrows,
}

/// Pattern severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Capture call stack (simplified)
fn capture_call_stack() -> Vec<String> {
    // In a real implementation, this would capture the actual call stack
    // For now, return a placeholder
    vec!["<call_stack_placeholder>".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_tracking() {
        let analyzer = BorrowAnalyzer::new();
        
        // Track an immutable borrow
        let borrow_id = analyzer.track_borrow(0x1000, BorrowType::Immutable, "test_var");
        
        // Check active borrows
        let active = analyzer.get_active_borrows(0x1000);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].borrow_type, BorrowType::Immutable);
        
        // End the borrow
        analyzer.end_borrow(borrow_id);
        
        // Check that it's no longer active
        let active = analyzer.get_active_borrows(0x1000);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_borrow_conflicts() {
        let analyzer = BorrowAnalyzer::new();
        
        // Track a mutable borrow
        analyzer.track_borrow(0x1000, BorrowType::Mutable, "test_var1");
        
        // Try to track another mutable borrow (should create conflict)
        analyzer.track_borrow(0x1000, BorrowType::Mutable, "test_var2");
        
        // Check conflicts
        let conflicts = analyzer.get_conflicts();
        assert!(!conflicts.is_empty());
        assert_eq!(conflicts[0].conflict_type, ConflictType::MultipleMutableBorrows);
    }
}