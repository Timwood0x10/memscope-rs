//! Advanced type analysis framework for complex Rust types
//!
//! This module provides a unified framework for analyzing complex Rust types
//! like Cell, RefCell, Mutex, RwLock, channels, etc. Instead of implementing
//! each type individually, we identify common patterns and provide a generic
//! analysis framework.

use crate::core::types::AllocationInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Informational level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
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
    /// < 1ns (atomic operations)
    Immediate,
    /// 1-10ns (simple operations)
    Fast,
    /// 10-100ns (syscalls, locks)
    Moderate,
    /// 100ns-1μs (complex operations)
    Slow,
    /// > 1μs (blocking operations)
    VerySlow,
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
        } else if type_name.contains("Mutex")
            || type_name.contains("RwLock")
            || type_name.contains("Condvar")
            || type_name.contains("Barrier")
        {
            AdvancedTypeCategory::Synchronization
        } else if type_name.contains("Sender")
            || type_name.contains("Receiver")
            || type_name.contains("mpsc")
            || type_name.contains("channel")
        {
            AdvancedTypeCategory::Channel
        } else if type_name.contains("Atomic") {
            AdvancedTypeCategory::Atomic
        } else if type_name.contains("ThreadLocal") || type_name.contains("LocalKey") {
            AdvancedTypeCategory::ThreadLocal
        } else if type_name.contains("ManuallyDrop")
            || type_name.contains("MaybeUninit")
            || type_name.contains("Pin")
        {
            AdvancedTypeCategory::MemoryManagement
        } else if type_name.contains("Future")
            || type_name.contains("Waker")
            || type_name.contains("Context")
            || type_name.contains("async")
        {
            AdvancedTypeCategory::Async
        } else {
            // Default fallback - try to infer from other characteristics
            AdvancedTypeCategory::MemoryManagement
        }
    }

    /// Analyze behavioral patterns
    fn analyze_behavior_pattern(
        type_name: &str,
        category: &AdvancedTypeCategory,
    ) -> TypeBehaviorPattern {
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
    fn check_potential_issues(
        type_name: &str,
        category: &AdvancedTypeCategory,
        behavior: &TypeBehaviorPattern,
    ) -> Vec<TypeIssue> {
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
            }
            AdvancedTypeCategory::Synchronization => {
                if behavior.deadlock_potential {
                    issues.push(TypeIssue {
                        severity: IssueSeverity::Warning,
                        description: "Synchronization primitive has deadlock potential".to_string(),
                        suggestion: Some(
                            "Ensure consistent lock ordering and consider using try_lock()"
                                .to_string(),
                        ),
                        location: None,
                    });
                }
            }
            AdvancedTypeCategory::Channel => {
                issues.push(TypeIssue {
                    severity: IssueSeverity::Info,
                    description: "Channel operations can block indefinitely".to_string(),
                    suggestion: Some("Consider using try_send/try_recv or timeouts".to_string()),
                    location: None,
                });
            }
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
                        overhead_factor: 2.0,                          // RefCell has runtime checks
                        memory_overhead: std::mem::size_of::<usize>(), // Borrow counter
                        is_lock_free: true,
                        latency_category: LatencyCategory::Fast,
                    }
                }
            }
            AdvancedTypeCategory::Synchronization => PerformanceInfo {
                overhead_factor: 10.0, // Significant overhead for locking
                memory_overhead: std::mem::size_of::<usize>() * 2, // Lock state + wait queue
                is_lock_free: false,
                latency_category: LatencyCategory::Moderate,
            },
            AdvancedTypeCategory::Channel => PerformanceInfo {
                overhead_factor: 5.0, // Moderate overhead
                memory_overhead: 64,  // Estimated buffer overhead
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
                overhead_factor: 3.0,                          // TLS lookup overhead
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
                memory_overhead: 32,  // Estimated state machine size
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
                let analysis =
                    GenericAdvancedTypeAnalyzer::analyze_by_type_name(type_name, allocation);

                // Clone data before moving analysis
                let category_key = format!("{:?}", analysis.category);
                let issues = analysis.potential_issues.clone();
                let overhead_factor = analysis.performance_info.overhead_factor;
                let memory_overhead = analysis.performance_info.memory_overhead;
                let is_lock_free = analysis.performance_info.is_lock_free;
                let latency_category = analysis.performance_info.latency_category.clone();

                // Move analysis to category
                by_category
                    .entry(category_key)
                    .or_insert_with(Vec::new)
                    .push(analysis);

                // Collect issues
                all_issues.extend(issues);

                // Aggregate performance data
                total_overhead_factor += overhead_factor;
                total_memory_overhead += memory_overhead;
                if is_lock_free {
                    lock_free_count += 1;
                }
                *latency_counts.entry(latency_category).or_insert(0) += 1;
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
            dominant_latency_category: latency_counts
                .iter()
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
    let statistics =
        generate_advanced_type_statistics(&by_category, &all_issues, &latency_counts, total_count);

    AdvancedTypeAnalysisReport {
        by_category,
        all_issues,
        performance_summary,
        statistics,
    }
}

/// Check if a type name represents an advanced type we should analyze
pub fn is_advanced_type(type_name: &str) -> bool {
    type_name.contains("Cell")
        || type_name.contains("Mutex")
        || type_name.contains("RwLock")
        || type_name.contains("Atomic")
        || type_name.contains("Sender")
        || type_name.contains("Receiver")
        || type_name.contains("ThreadLocal")
        || type_name.contains("ManuallyDrop")
        || type_name.contains("MaybeUninit")
        || type_name.contains("Pin")
        || type_name.contains("Future")
        || type_name.contains("Waker")
        || type_name.contains("Condvar")
        || type_name.contains("Barrier")
        || type_name.contains("Arc<")
        || type_name.contains("Rc<")
        || type_name.contains("Weak<")
        || type_name.contains("Box<dyn")
        || type_name.contains("Cow<")
        || type_name.contains("HashMap<")
        || type_name.contains("BTreeMap<")
        || type_name.contains("PhantomData<")
}

/// Analyze a type and return its category
pub fn get_type_category(type_name: &str) -> Option<AdvancedTypeCategory> {
    if type_name.contains("Cell<")
        || type_name.contains("RefCell<")
        || type_name.contains("UnsafeCell<")
    {
        Some(AdvancedTypeCategory::InteriorMutability)
    } else if type_name.contains("Mutex<")
        || type_name.contains("RwLock<")
        || type_name.contains("Condvar")
    {
        Some(AdvancedTypeCategory::Synchronization)
    } else if type_name.contains("mpsc::")
        || type_name.contains("Sender<")
        || type_name.contains("Receiver<")
    {
        Some(AdvancedTypeCategory::Channel)
    } else if type_name.contains("Atomic") {
        Some(AdvancedTypeCategory::Atomic)
    } else if type_name.contains("ThreadLocal<") || type_name.contains("LocalKey<") {
        Some(AdvancedTypeCategory::ThreadLocal)
    } else if type_name.contains("ManuallyDrop<")
        || type_name.contains("MaybeUninit<")
        || type_name.contains("Pin<")
    {
        Some(AdvancedTypeCategory::MemoryManagement)
    } else if type_name.contains("Future")
        || type_name.contains("Stream")
        || type_name.contains("Waker")
        || type_name.contains("Context")
    {
        Some(AdvancedTypeCategory::Async)
    } else {
        None
    }
}

/// Create behavior pattern for a type
pub fn create_behavior_pattern(type_name: &str) -> TypeBehaviorPattern {
    let category = get_type_category(type_name);

    match category {
        Some(AdvancedTypeCategory::InteriorMutability) => TypeBehaviorPattern {
            has_interior_mutability: true,
            is_thread_safe: !type_name.contains("Cell<"), // Cell is not thread-safe, RefCell is not either
            can_block: false,
            manages_memory_layout: false,
            deadlock_potential: false,
            has_runtime_borrow_check: type_name.contains("RefCell<"),
            has_runtime_overhead: type_name.contains("RefCell<"),
        },
        Some(AdvancedTypeCategory::Synchronization) => TypeBehaviorPattern {
            has_interior_mutability: true,
            is_thread_safe: true,
            can_block: true,
            manages_memory_layout: false,
            deadlock_potential: true,
            has_runtime_borrow_check: false,
            has_runtime_overhead: true,
        },
        Some(AdvancedTypeCategory::Channel) => TypeBehaviorPattern {
            has_interior_mutability: false,
            is_thread_safe: true,
            can_block: true,
            manages_memory_layout: true,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: true,
        },
        Some(AdvancedTypeCategory::Atomic) => TypeBehaviorPattern {
            has_interior_mutability: true,
            is_thread_safe: true,
            can_block: false,
            manages_memory_layout: false,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: false,
        },
        Some(AdvancedTypeCategory::ThreadLocal) => TypeBehaviorPattern {
            has_interior_mutability: true,
            is_thread_safe: false,
            can_block: false,
            manages_memory_layout: true,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: true,
        },
        Some(AdvancedTypeCategory::MemoryManagement) => TypeBehaviorPattern {
            has_interior_mutability: type_name.contains("Pin<"),
            is_thread_safe: false,
            can_block: false,
            manages_memory_layout: true,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: false,
        },
        Some(AdvancedTypeCategory::Async) => TypeBehaviorPattern {
            has_interior_mutability: false,
            is_thread_safe: false,
            can_block: false,
            manages_memory_layout: true,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: true,
        },
        None => TypeBehaviorPattern {
            has_interior_mutability: false,
            is_thread_safe: false,
            can_block: false,
            manages_memory_layout: false,
            deadlock_potential: false,
            has_runtime_borrow_check: false,
            has_runtime_overhead: false,
        },
    }
}

/// Analyze a type and create AdvancedTypeInfo
pub fn analyze_type(allocation: &AllocationInfo) -> Option<AdvancedTypeInfo> {
    let type_name = allocation.type_name.as_ref()?;

    if !is_advanced_type(type_name) {
        return None;
    }

    let category = get_type_category(type_name)?;
    let behavior = create_behavior_pattern(type_name);

    let mut issues = Vec::new();
    let _recommendations: Vec<String> = Vec::new();

    // Analyze potential issues based on type behavior
    if behavior.deadlock_potential {
        issues.push(TypeIssue {
            severity: IssueSeverity::Warning,
            description: "Type has deadlock potential - ensure proper lock ordering".to_string(),
            location: Some(format!("ptr: 0x{:x}", allocation.ptr)),
            suggestion: Some(
                "Consider using timeout-based locking or lock hierarchies".to_string(),
            ),
        });
    }

    if behavior.has_runtime_overhead && allocation.size > 1024 {
        issues.push(TypeIssue {
            severity: IssueSeverity::Warning,
            description: "Large allocation with runtime overhead".to_string(),
            location: Some(format!(
                "ptr: 0x{:x}, size: {}",
                allocation.ptr, allocation.size
            )),
            suggestion: Some(
                "Consider using more efficient alternatives for large data".to_string(),
            ),
        });
    }

    Some(AdvancedTypeInfo {
        category,
        behavior: behavior.clone(),
        state_info: TypeStateInfo {
            is_borrowed: None,
            borrow_count: Some(allocation.borrow_count),
            is_locked: None,
            lock_owner_thread: None,
            wait_queue_length: None,
            channel_info: None,
        },
        potential_issues: issues,
        performance_info: PerformanceInfo {
            overhead_factor: if behavior.has_runtime_overhead {
                2.0
            } else {
                1.0
            },
            memory_overhead: calculate_overhead(type_name),
            is_lock_free: !behavior.can_block,
            latency_category: if behavior.can_block {
                LatencyCategory::Moderate
            } else {
                LatencyCategory::Fast
            },
        },
    })
}

/// Calculate overhead for a type
fn calculate_overhead(type_name: &str) -> usize {
    if type_name.contains("RefCell<") {
        std::mem::size_of::<isize>() // BorrowFlag is private, use isize
    } else if type_name.contains("Mutex<") {
        64 // Approximate mutex overhead
    } else if type_name.contains("RwLock<") {
        96 // Approximate RwLock overhead
    } else if type_name.contains("Arc<") || type_name.contains("Rc<") {
        std::mem::size_of::<usize>() * 2 // Strong + weak counts
    } else {
        0
    }
}

/// Calculate alignment requirements
fn calculate_alignment(type_name: &str) -> usize {
    if type_name.contains("Atomic") {
        8 // Most atomics require 8-byte alignment
    } else if type_name.contains("Mutex<") || type_name.contains("RwLock<") {
        std::mem::align_of::<std::sync::Mutex<()>>()
    } else {
        std::mem::align_of::<usize>()
    }
}

/// Analyze cache behavior
fn analyze_cache_behavior(type_name: &str) -> String {
    if type_name.contains("HashMap<") || type_name.contains("BTreeMap<") {
        "Poor - scattered allocations".to_string()
    } else if type_name.contains("Vec<") || type_name.contains("String") {
        "Good - contiguous memory".to_string()
    } else if type_name.contains("Atomic") {
        "Excellent - cache-line optimized".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Analyze cache impact
fn analyze_cache_impact(type_name: &str) -> String {
    if type_name.contains("Mutex<") || type_name.contains("RwLock<") {
        "High - false sharing potential".to_string()
    } else if type_name.contains("Atomic") {
        "Medium - cache line contention".to_string()
    } else {
        "Low".to_string()
    }
}

/// Calculate memory overhead percentage
fn calculate_memory_overhead_percentage(type_name: &str) -> u32 {
    if type_name.contains("RefCell<") {
        10 // ~10% overhead for borrow checking
    } else if type_name.contains("Mutex<") {
        25 // ~25% overhead for synchronization
    } else if type_name.contains("Arc<") || type_name.contains("Rc<") {
        15 // ~15% overhead for reference counting
    } else {
        5 // Default small overhead
    }
}

/// Analyze scalability concerns
fn analyze_scalability(type_name: &str) -> Vec<String> {
    let mut concerns = Vec::new();

    if type_name.contains("Mutex<") {
        concerns.push("Lock contention under high concurrency".to_string());
    }

    if type_name.contains("HashMap<") {
        concerns.push("Hash collision performance degradation".to_string());
    }

    if type_name.contains("RefCell<") {
        concerns.push("Runtime borrow check overhead".to_string());
    }

    if type_name.contains("mpsc::") {
        concerns.push("Channel capacity and blocking behavior".to_string());
    }

    concerns
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

// ===== Enhanced Interior Mutability Detection =====

/// Enhanced interior mutability detection for Cell/RefCell types
pub fn detect_interior_mutability_patterns(
    allocations: &[AllocationInfo],
) -> InteriorMutabilityReport {
    let mut cell_instances = Vec::new();
    let mut refcell_instances = Vec::new();
    let mut unsafe_cell_instances = Vec::new();
    let mut runtime_borrow_violations = Vec::new();

    for allocation in allocations {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("Cell<") {
                cell_instances.push(CellInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    thread_safe: true, // Cell is always thread-safe
                    zero_cost: true,   // Cell has no runtime overhead
                });
            } else if type_name.contains("RefCell<") {
                let instance = RefCellInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    current_borrow_count: allocation.borrow_count,
                    has_active_mut_borrow: allocation.borrow_count > 0,
                    runtime_check_overhead: true,
                };

                // Check for potential runtime borrow violations
                if allocation.borrow_count > 1 {
                    runtime_borrow_violations.push(BorrowViolation {
                        ptr: allocation.ptr,
                        violation_type: BorrowViolationType::MultipleBorrows,
                        borrow_count: allocation.borrow_count,
                        timestamp: allocation.timestamp_alloc,
                    });
                }

                refcell_instances.push(instance);
            } else if type_name.contains("UnsafeCell<") {
                unsafe_cell_instances.push(UnsafeCellInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    requires_unsafe_access: true,
                });
            }
        }
    }

    let total_types = cell_instances.len() + refcell_instances.len() + unsafe_cell_instances.len();

    InteriorMutabilityReport {
        cell_instances,
        refcell_instances,
        unsafe_cell_instances,
        runtime_borrow_violations,
        total_interior_mutability_types: total_types,
        analysis_timestamp: current_timestamp(),
    }
}

/// Enhanced concurrency primitive monitoring for Mutex/RwLock types
pub fn monitor_concurrency_primitives(
    allocations: &[AllocationInfo],
) -> ConcurrencyPrimitiveReport {
    let mut mutex_instances = Vec::new();
    let mut rwlock_instances = Vec::new();
    let mut condvar_instances = Vec::new();
    let lock_contentions = Vec::new();

    for allocation in allocations {
        if let Some(type_name) = &allocation.type_name {
            if type_name.contains("Mutex<") {
                mutex_instances.push(MutexInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    is_locked: false, // Would need runtime tracking
                    lock_owner_thread: None,
                    waiting_threads: 0,
                    total_lock_acquisitions: 0,
                    total_wait_time_ns: 0,
                });
            } else if type_name.contains("RwLock<") {
                rwlock_instances.push(RwLockInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    read_count: 0,
                    has_write_lock: false,
                    waiting_readers: 0,
                    waiting_writers: 0,
                    total_read_acquisitions: 0,
                    total_write_acquisitions: 0,
                });
            } else if type_name.contains("Condvar") {
                condvar_instances.push(CondvarInstance {
                    ptr: allocation.ptr,
                    type_name: type_name.clone(),
                    size: allocation.size,
                    waiting_threads: 0,
                    total_notifications: 0,
                });
            }
        }
    }

    let deadlock_score =
        calculate_deadlock_potential_by_count(mutex_instances.len(), rwlock_instances.len());

    ConcurrencyPrimitiveReport {
        mutex_instances,
        rwlock_instances,
        condvar_instances,
        lock_contentions,
        deadlock_potential_score: deadlock_score,
        analysis_timestamp: current_timestamp(),
    }
}

// ===== Supporting Types and Functions =====

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Calculate deadlock potential based on lock count
fn calculate_deadlock_potential_by_count(mutex_count: usize, rwlock_count: usize) -> f64 {
    let total_locks = mutex_count + rwlock_count;
    if total_locks <= 1 {
        return 0.0;
    }

    // Simple heuristic: more locks = higher deadlock potential
    // In reality, this would analyze lock ordering and dependency graphs
    (total_locks as f64).log2() / 10.0
}

// ===== Type Definitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteriorMutabilityReport {
    pub cell_instances: Vec<CellInstance>,
    pub refcell_instances: Vec<RefCellInstance>,
    pub unsafe_cell_instances: Vec<UnsafeCellInstance>,
    pub runtime_borrow_violations: Vec<BorrowViolation>,
    pub total_interior_mutability_types: usize,
    pub analysis_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub thread_safe: bool,
    pub zero_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefCellInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub current_borrow_count: usize,
    pub has_active_mut_borrow: bool,
    pub runtime_check_overhead: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeCellInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub requires_unsafe_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorrowViolation {
    pub ptr: usize,
    pub violation_type: BorrowViolationType,
    pub borrow_count: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BorrowViolationType {
    MultipleBorrows,
    MutableBorrowConflict,
    BorrowAfterMove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyPrimitiveReport {
    pub mutex_instances: Vec<MutexInstance>,
    pub rwlock_instances: Vec<RwLockInstance>,
    pub condvar_instances: Vec<CondvarInstance>,
    pub lock_contentions: Vec<LockContention>,
    pub deadlock_potential_score: f64,
    pub analysis_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutexInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub is_locked: bool,
    pub lock_owner_thread: Option<String>,
    pub waiting_threads: usize,
    pub total_lock_acquisitions: u64,
    pub total_wait_time_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RwLockInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub read_count: usize,
    pub has_write_lock: bool,
    pub waiting_readers: usize,
    pub waiting_writers: usize,
    pub total_read_acquisitions: u64,
    pub total_write_acquisitions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CondvarInstance {
    pub ptr: usize,
    pub type_name: String,
    pub size: usize,
    pub waiting_threads: usize,
    pub total_notifications: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockContention {
    pub lock_ptr: usize,
    pub contention_type: ContentionType,
    pub waiting_time_ns: u64,
    pub thread_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentionType {
    MutexContention,
    RwLockReadContention,
    RwLockWriteContention,
}
