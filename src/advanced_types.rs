//! Advanced type analysis framework for complex Rust types
//!
//! This module provides a unified framework for analyzing complex Rust types
//! like Cell, RefCell, Mutex, RwLock, channels, etc. Instead of implementing
//! each type individually, we identify common patterns and provide a generic
//! analysis framework.

use crate::types::AllocationInfo;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Categories of advanced Rust types based on their memory and concurrency characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdvancedTypeCategory {
    /// Interior mutability types (Cell, RefCell, UnsafeCell)
    InteriorMutability,
    /// Synchronization primitives (Mutex, RwLock, Condvar)
    Synchronization,
    /// Channel types (Sender, Receiver, mpsc, etc.)
    Channel,
    /// Atomic types (AtomicBool, AtomicUsize, etc.)
    Atomic,
    /// Thread-local storage (ThreadLocal, LocalKey)
    ThreadLocal,
    /// Memory management (ManuallyDrop, MaybeUninit, Pin)
    MemoryManagement,
    /// Async primitives (Future, Waker, Context)
    Async,
}

/// Behavioral patterns that advanced types exhibit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeBehaviorPattern {
    /// Does this type provide interior mutability?
    pub has_interior_mutability: bool,
    /// Can this type be shared across threads?
    pub is_thread_safe: bool,
    /// Does this type involve blocking operations?
    pub can_block: bool,
    /// Does this type manage its own memory layout?
    pub manages_memory_layout: bool,
    /// Can this type cause deadlocks?
    pub deadlock_potential: bool,
    /// Does this type have runtime borrow checking?
    pub has_runtime_borrow_check: bool,
    /// Is this type zero-cost or has runtime overhead?
    pub has_runtime_overhead: bool,
}

/// Advanced type analysis information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedTypeInfo {
    /// Category of this advanced type
    pub category: AdvancedTypeCategory,
    /// Behavioral pattern
    pub behavior: TypeBehaviorPattern,
    /// Current state information
    pub state_info: TypeStateInfo,
    /// Potential issues or warnings
    pub potential_issues: Vec<TypeIssue>,
    /// Performance characteristics
    pub performance_info: PerformanceInfo,
}

/// Current state of the advanced type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeStateInfo {
    /// Is currently borrowed (for RefCell-like types)?
    pub is_borrowed: Option<bool>,
    /// Current borrow count (if applicable)
    pub borrow_count: Option<usize>,
    /// Is currently locked (for Mutex-like types)?
    pub is_locked: Option<bool>,
    /// Thread that currently owns the lock (if applicable)
    pub lock_owner_thread: Option<String>,
    /// Queue length for waiting threads/operations
    pub wait_queue_length: Option<usize>,
    /// Channel capacity and current usage
    pub channel_info: Option<ChannelStateInfo>,
}

/// Channel-specific state information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelStateInfo {
    /// Channel capacity (None for unbounded)
    pub capacity: Option<usize>,
    /// Current number of items in channel
    pub current_size: usize,
    /// Number of active senders
    pub sender_count: usize,
    /// Number of active receivers
    pub receiver_count: usize,
    /// Is the channel closed?
    pub is_closed: bool,
}

/// Potential issues with advanced types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Description of the issue
    pub description: String,
    /// Suggested fix or mitigation
    pub suggestion: Option<String>,
    /// Related code location if available
    pub location: Option<String>,
}

/// Severity levels for type issues
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Performance characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceInfo {
    /// Estimated overhead compared to direct access
    pub overhead_factor: f64,
    /// Memory overhead in bytes
    pub memory_overhead: usize,
    /// Whether operations are lock-free
    pub is_lock_free: bool,
    /// Typical operation latency category
    pub latency_category: LatencyCategory,
}

/// Latency categories for operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LatencyCategory {
    Immediate,    // < 1ns (atomic operations)
    Fast,         // 1-10ns (simple operations)
    Moderate,     // 10-100ns (syscalls, locks)
    Slow,         // 100ns-1μs (complex operations)
    VerySlow,     // > 1μs (blocking operations)
}

/// Trait for advanced type analysis
pub trait AdvancedTypeAnalyzer {
    /// Analyze the advanced type and return analysis information
    fn analyze_advanced_type(&self) -> AdvancedTypeInfo;
    
    /// Get current state snapshot
    fn get_current_state(&self) -> TypeStateInfo;
    
    /// Check for potential issues
    fn check_issues(&self) -> Vec<TypeIssue>;
    
    /// Get performance characteristics
    fn get_performance_info(&self) -> PerformanceInfo;
}

/// Generic analyzer that can handle most advanced types through pattern matching
pub struct GenericAdvancedTypeAnalyzer;

impl GenericAdvancedTypeAnalyzer {
    /// Analyze a type by its name and characteristics
    pub fn analyze_by_type_name(type_name: &str, allocation: &AllocationInfo) -> AdvancedTypeInfo {
        let category = Self::categorize_type(type_name);
        let behavior = Self::analyze_behavior_pattern(type_name, &category);
        let state_info = Self::extract_state_info(type_name, allocation);
        let potential_issues = Self::check_potential_issues(type_name, &category, &behavior);
        let performance_info = Self::analyze_performance(type_name, &category);
        
        AdvancedTypeInfo {
            category,
            behavior,
            state_info,
            potential_issues,
            performance_info,
        }
    }
    
    /// Categorize type based on its name
    fn categorize_type(type_name: &str) -> AdvancedTypeCategory {
        if type_name.contains("Cell") || type_name.contains("UnsafeCell") {
            AdvancedTypeCategory::InteriorMutability
        } else if type_name.contains("Mutex") || type_name.contains("RwLock") || 
                  type_name.contains("Condvar") || type_name.contains("Barrier") {
            AdvancedTypeCategory::Synchronization
        } else if type_name.contains("Sender") || type_name.contains("Receiver") || 
                  type_name.contains("mpsc") || type_name.contains("channel") {
            AdvancedTypeCategory::Channel
        } else if type_name.contains("Atomic") {
            AdvancedTypeCategory::Atomic
        } else if type_name.contains("ThreadLocal") || type_name.contains("LocalKey") {
            AdvancedTypeCategory::ThreadLocal
        } else if type_name.contains("ManuallyDrop") || type_name.contains("MaybeUninit") || 
                  type_name.contains("Pin") {
            AdvancedTypeCategory::MemoryManagement
        } else if type_name.contains("Future") || type_name.contains("Waker") || 
                  type_name.contains("Context") || type_name.contains("async") {
            AdvancedTypeCategory::Async
        } else {
            // Default fallback - try to infer from other characteristics
            AdvancedTypeCategory::MemoryManagement
        }
    }
    
    /// Analyze behavioral patterns
    fn analyze_behavior_pattern(type_name: &str, category: &AdvancedTypeCategory) -> TypeBehaviorPattern {
        match category {
            AdvancedTypeCategory::InteriorMutability => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: !type_name.contains("Cell"), // RefCell is not thread-safe, Cell might be
                can_block: false,
                manages_memory_layout: false,
                deadlock_potential: false,
                has_runtime_borrow_check: type_name.contains("RefCell"),
                has_runtime_overhead: type_name.contains("RefCell"),
            },
            AdvancedTypeCategory::Synchronization => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: true,
                can_block: true,
                manages_memory_layout: false,
                deadlock_potential: true,
                has_runtime_borrow_check: false,
                has_runtime_overhead: true,
            },
            AdvancedTypeCategory::Channel => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: true,
                can_block: true,
                manages_memory_layout: true,
                deadlock_potential: false,
                has_runtime_borrow_check: false,
                has_runtime_overhead: true,
            },
            AdvancedTypeCategory::Atomic => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: true,
                can_block: false,
                manages_memory_layout: false,
                deadlock_potential: false,
                has_runtime_borrow_check: false,
                has_runtime_overhead: false,
            },
            AdvancedTypeCategory::ThreadLocal => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: false, // Thread-local by definition
                can_block: false,
                manages_memory_layout: true,
                deadlock_potential: false,
                has_runtime_borrow_check: false,
                has_runtime_overhead: true,
            },
            AdvancedTypeCategory::MemoryManagement => TypeBehaviorPattern {
                has_interior_mutability: false,
                is_thread_safe: true, // Usually just wrappers
                can_block: false,
                manages_memory_layout: true,
                deadlock_potential: false,
                has_runtime_borrow_check: false,
                has_runtime_overhead: false,
            },
            AdvancedTypeCategory::Async => TypeBehaviorPattern {
                has_interior_mutability: true,
                is_thread_safe: true,
                can_block: true, // Can suspend execution
                manages_memory_layout: true,
                deadlock_potential: false,
                has_runtime_borrow_check: false,
                has_runtime_overhead: true,
            },
        }
    }
    
    /// Extract current state information (limited without runtime introspection)
    fn extract_state_info(_type_name: &str, _allocation: &AllocationInfo) -> TypeStateInfo {
        // Note: Without runtime introspection, we can only provide limited state info
        // In a real implementation, this would require unsafe code or cooperation from the types
        TypeStateInfo {
            is_borrowed: None,
            borrow_count: None,
            is_locked: None,
            lock_owner_thread: None,
            wait_queue_length: None,
            channel_info: None,
        }
    }
    
    /// Check for potential issues based on type characteristics
    fn check_potential_issues(type_name: &str, category: &AdvancedTypeCategory, behavior: &TypeBehaviorPattern) -> Vec<TypeIssue> {
        let mut issues = Vec::new();
        
        // Check for common issues based on category
        match category {
            AdvancedTypeCategory::InteriorMutability => {
                if type_name.contains("RefCell") {
                    issues.push(TypeIssue {
                        severity: IssueSeverity::Warning,
                        description: "RefCell has runtime borrow checking overhead".to_string(),
                        suggestion: Some("Consider using Cell for Copy types or redesign to avoid interior mutability".to_string()),
                        location: None,
                    });
                }
            },
            AdvancedTypeCategory::Synchronization => {
                if behavior.deadlock_potential {
                    issues.push(TypeIssue {
                        severity: IssueSeverity::Warning,
                        description: "Synchronization primitive has deadlock potential".to_string(),
                        suggestion: Some("Ensure consistent lock ordering and consider using try_lock()".to_string()),
                        location: None,
                    });
                }
            },
            AdvancedTypeCategory::Channel => {
                issues.push(TypeIssue {
                    severity: IssueSeverity::Info,
                    description: "Channel operations can block indefinitely".to_string(),
                    suggestion: Some("Consider using try_send/try_recv or timeouts".to_string()),
                    location: None,
                });
            },
            _ => {}
        }
        
        issues
    }
    
    /// Analyze performance characteristics
    fn analyze_performance(type_name: &str, category: &AdvancedTypeCategory) -> PerformanceInfo {
        match category {
            AdvancedTypeCategory::InteriorMutability => {
                if type_name.contains("Cell") {
                    PerformanceInfo {
                        overhead_factor: 1.0, // Zero-cost abstraction
                        memory_overhead: 0,
                        is_lock_free: true,
                        latency_category: LatencyCategory::Immediate,
                    }
                } else {
                    PerformanceInfo {
                        overhead_factor: 2.0, // RefCell has runtime checks
                        memory_overhead: std::mem::size_of::<usize>(), // Borrow counter
                        is_lock_free: true,
                        latency_category: LatencyCategory::Fast,
                    }
                }
            },
            AdvancedTypeCategory::Synchronization => PerformanceInfo {
                overhead_factor: 10.0, // Significant overhead for locking
                memory_overhead: std::mem::size_of::<usize>() * 2, // Lock state + wait queue
                is_lock_free: false,
                latency_category: LatencyCategory::Moderate,
            },
            AdvancedTypeCategory::Channel => PerformanceInfo {
                overhead_factor: 5.0, // Moderate overhead
                memory_overhead: 64, // Estimated buffer overhead
                is_lock_free: false,
                latency_category: LatencyCategory::Moderate,
            },
            AdvancedTypeCategory::Atomic => PerformanceInfo {
                overhead_factor: 1.5, // Slight overhead for atomic operations
                memory_overhead: 0,
                is_lock_free: true,
                latency_category: LatencyCategory::Immediate,
            },
            AdvancedTypeCategory::ThreadLocal => PerformanceInfo {
                overhead_factor: 3.0, // TLS lookup overhead
                memory_overhead: std::mem::size_of::<usize>(), // TLS key
                is_lock_free: true,
                latency_category: LatencyCategory::Fast,
            },
            AdvancedTypeCategory::MemoryManagement => PerformanceInfo {
                overhead_factor: 1.0, // Usually zero-cost
                memory_overhead: 0,
                is_lock_free: true,
                latency_category: LatencyCategory::Immediate,
            },
            AdvancedTypeCategory::Async => PerformanceInfo {
                overhead_factor: 4.0, // State machine overhead
                memory_overhead: 32, // Estimated state machine size
                is_lock_free: true,
                latency_category: LatencyCategory::Fast,
            },
        }
    }
}

/// Analysis results for all advanced types in an allocation set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTypeAnalysisReport {
    /// Analysis by type category
    pub by_category: HashMap<String, Vec<AdvancedTypeInfo>>,
    /// All detected issues
    pub all_issues: Vec<TypeIssue>,
    /// Performance summary
    pub performance_summary: PerformanceSummary,
    /// Statistics
    pub statistics: AdvancedTypeStatistics,
}

/// Performance summary across all advanced types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total estimated overhead factor
    pub total_overhead_factor: f64,
    /// Total memory overhead in bytes
    pub total_memory_overhead: usize,
    /// Percentage of types that are lock-free
    pub lock_free_percentage: f64,
    /// Most common latency category
    pub dominant_latency_category: LatencyCategory,
}

/// Statistics for advanced type usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTypeStatistics {
    /// Count by category
    pub by_category: HashMap<String, usize>,
    /// Count by issue severity
    pub by_issue_severity: HashMap<String, usize>,
    /// Count by latency category
    pub by_latency_category: HashMap<String, usize>,
    /// Total advanced types analyzed
    pub total_advanced_types: usize,
}

/// Analyze all advanced types in a set of allocations
pub fn analyze_advanced_types(allocations: &[AllocationInfo]) -> AdvancedTypeAnalysisReport {
    let mut by_category: HashMap<String, Vec<AdvancedTypeInfo>> = HashMap::new();
    let mut all_issues = Vec::new();
    let mut total_overhead_factor = 0.0;
    let mut total_memory_overhead = 0;
    let mut lock_free_count = 0;
    let mut total_count = 0;
    let mut latency_counts: HashMap<LatencyCategory, usize> = HashMap::new();
    
    for allocation in allocations {
        if let Some(type_name) = &allocation.type_name {
            // Check if this is an advanced type we should analyze
            if is_advanced_type(type_name) {
                let analysis = GenericAdvancedTypeAnalyzer::analyze_by_type_name(type_name, allocation);
                
                // Categorize
                let category_key = format!("{:?}", analysis.category);
                by_category.entry(category_key).or_insert_with(Vec::new).push(analysis.clone());
                
                // Collect issues
                all_issues.extend(analysis.potential_issues.clone());
                
                // Aggregate performance data
                total_overhead_factor += analysis.performance_info.overhead_factor;
                total_memory_overhead += analysis.performance_info.memory_overhead;
                if analysis.performance_info.is_lock_free {
                    lock_free_count += 1;
                }
                *latency_counts.entry(analysis.performance_info.latency_category.clone()).or_insert(0) += 1;
                total_count += 1;
            }
        }
    }
    
    // Calculate performance summary
    let performance_summary = if total_count > 0 {
        PerformanceSummary {
            total_overhead_factor: total_overhead_factor / total_count as f64,
            total_memory_overhead,
            lock_free_percentage: (lock_free_count as f64 / total_count as f64) * 100.0,
            dominant_latency_category: latency_counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(category, _)| category.clone())
                .unwrap_or(LatencyCategory::Fast),
        }
    } else {
        PerformanceSummary {
            total_overhead_factor: 1.0,
            total_memory_overhead: 0,
            lock_free_percentage: 100.0,
            dominant_latency_category: LatencyCategory::Immediate,
        }
    };
    
    // Generate statistics
    let statistics = generate_advanced_type_statistics(&by_category, &all_issues, &latency_counts, total_count);
    
    AdvancedTypeAnalysisReport {
        by_category,
        all_issues,
        performance_summary,
        statistics,
    }
}

/// Check if a type name represents an advanced type we should analyze
pub fn is_advanced_type(type_name: &str) -> bool {
    type_name.contains("Cell") || 
    type_name.contains("Mutex") || 
    type_name.contains("RwLock") ||
    type_name.contains("Atomic") ||
    type_name.contains("Sender") ||
    type_name.contains("Receiver") ||
    type_name.contains("ThreadLocal") ||
    type_name.contains("ManuallyDrop") ||
    type_name.contains("MaybeUninit") ||
    type_name.contains("Pin") ||
    type_name.contains("Future") ||
    type_name.contains("Waker") ||
    type_name.contains("Condvar") ||
    type_name.contains("Barrier")
}

/// Generate statistics for the analysis
fn generate_advanced_type_statistics(
    by_category: &HashMap<String, Vec<AdvancedTypeInfo>>,
    all_issues: &[TypeIssue],
    latency_counts: &HashMap<LatencyCategory, usize>,
    total_count: usize,
) -> AdvancedTypeStatistics {
    let mut by_category_stats = HashMap::new();
    for (category, types) in by_category {
        by_category_stats.insert(category.clone(), types.len());
    }
    
    let mut by_issue_severity = HashMap::new();
    for issue in all_issues {
        let severity_key = format!("{:?}", issue.severity);
        *by_issue_severity.entry(severity_key).or_insert(0) += 1;
    }
    
    let mut by_latency_category = HashMap::new();
    for (category, count) in latency_counts {
        by_latency_category.insert(format!("{:?}", category), *count);
    }
    
    AdvancedTypeStatistics {
        by_category: by_category_stats,
        by_issue_severity,
        by_latency_category,
        total_advanced_types: total_count,
    }
}