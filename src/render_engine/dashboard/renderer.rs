//! Dashboard renderer using Handlebars templates

use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::tracker::Tracker;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Dashboard context for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardContext {
    /// Page title
    pub title: String,
    /// Export timestamp
    pub export_timestamp: String,
    /// Total memory allocated (formatted)
    pub total_memory: String,
    /// Total number of allocations
    pub total_allocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Peak memory usage (formatted)
    pub peak_memory: String,
    /// Number of threads
    pub thread_count: usize,
    /// Number of memory passports
    pub passport_count: usize,
    /// Number of memory leaks detected
    pub leak_count: usize,
    /// Number of unsafe operations
    pub unsafe_count: usize,
    /// Number of FFI operations
    pub ffi_count: usize,
    /// Allocation information
    pub allocations: Vec<AllocationInfo>,
    /// Variable relationships
    pub relationships: Vec<RelationshipInfo>,
    /// Unsafe/FFI reports
    pub unsafe_reports: Vec<UnsafeReport>,
    /// Detailed passport information
    pub passport_details: Vec<PassportDetail>,
    /// Count helper for template
    pub allocations_count: usize,
    /// Count helper for template
    pub relationships_count: usize,
    /// Count helper for template
    pub unsafe_reports_count: usize,
    /// JSON data string for injection (performance optimization)
    pub json_data: String,
    /// OS name
    pub os_name: String,
    /// Architecture
    pub architecture: String,
    /// CPU cores
    pub cpu_cores: usize,
    /// System resources
    pub system_resources: SystemResources,
    /// Thread analysis data
    pub threads: Vec<ThreadInfo>,
}

/// Allocation information for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Memory address
    pub address: String,
    /// Type name
    pub type_name: String,
    /// Allocation size in bytes
    pub size: usize,
    /// Variable name
    pub var_name: String,
    /// Timestamp
    pub timestamp: String,
    /// Thread ID
    pub thread_id: String,
    /// Borrow information
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    /// Clone information
    pub is_clone: bool,
    pub clone_count: usize,
    /// Allocation timestamp (nanoseconds)
    pub timestamp_alloc: u64,
    /// Deallocation timestamp (nanoseconds, 0 if not freed)
    pub timestamp_dealloc: u64,
    /// Lifetime in milliseconds
    pub lifetime_ms: f64,
    /// Whether memory is leaked
    pub is_leaked: bool,
    /// Allocation type (stack, heap, etc.)
    pub allocation_type: String,
    /// Whether this is a smart pointer
    pub is_smart_pointer: bool,
    /// Smart pointer type (Arc, Rc, Box, etc.)
    pub smart_pointer_type: String,
    /// Source file where allocation occurred
    pub source_file: Option<String>,
    /// Source line where allocation occurred
    pub source_line: Option<u32>,
}

/// Thread statistics for multithread dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThreadStats {
    /// Thread ID
    id: u64,
    /// Number of allocations
    allocations: usize,
    /// Total memory used
    memory: usize,
    /// Peak memory usage
    peak: usize,
    /// Thread status
    status: String,
}

/// Timeline allocation for multithread dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimelineAllocation {
    /// Timestamp
    timestamp: u64,
    /// Thread ID
    thread_id: u64,
    /// Allocation size
    size: usize,
    /// Variable name
    var_name: Option<String>,
}

/// Thread conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThreadConflict {
    /// Description of the conflict
    description: String,
    /// Threads involved
    threads: String,
    /// Conflict type
    #[serde(rename = "type")]
    conflict_type: String,
}

/// Variable relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInfo {
    /// Source pointer
    pub source_ptr: String,
    /// Source variable name
    pub source_var_name: String,
    /// Target pointer
    pub target_ptr: String,
    /// Target variable name
    pub target_var_name: String,
    /// Relationship type (reference, borrow, clone, copy, move, ownership_transfer)
    pub relationship_type: String,
    /// Relationship strength (0.0 to 1.0)
    pub strength: f64,
    /// Type name
    pub type_name: String,
    /// Color for visualization
    pub color: String,
    /// Whether this relationship is part of a detected cycle (true) or not (false)
    pub is_part_of_cycle: bool,
}

/// Unsafe/FFI report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeReport {
    /// Passport ID
    pub passport_id: String,
    /// Allocation pointer
    pub allocation_ptr: String,
    /// Variable name
    pub var_name: String,
    /// Type name
    pub type_name: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Created at timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Status at shutdown
    pub status: String,
    /// Lifecycle events
    pub lifecycle_events: Vec<LifecycleEventInfo>,
    /// Cross-boundary events
    pub cross_boundary_events: Vec<BoundaryEventInfo>,
    /// Whether this is a memory leak
    pub is_leaked: bool,
    /// Risk level (low, medium, high)
    pub risk_level: String,
    /// Risk factors
    pub risk_factors: Vec<String>,
}

/// Lifecycle event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEventInfo {
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: u64,
    /// Context
    pub context: String,
    /// Event icon
    pub icon: String,
    /// Event color
    pub color: String,
}

/// Boundary event information (FFI/Rust crossings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryEventInfo {
    /// Event type (RustToFfi, FfiToRust, etc.)
    pub event_type: String,
    /// Source context
    pub from_context: String,
    /// Target context
    pub to_context: String,
    /// Timestamp
    pub timestamp: u64,
    /// Direction icon
    pub icon: String,
    /// Direction color
    pub color: String,
}

/// Detailed passport information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportDetail {
    /// Passport ID
    pub passport_id: String,
    /// Allocation pointer
    pub allocation_ptr: String,
    /// Variable name
    pub var_name: String,
    /// Type name
    pub type_name: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Status at shutdown
    pub status: String,
    /// Created at
    pub created_at: u64,
    /// Updated at
    pub updated_at: u64,
    /// Whether leaked
    pub is_leaked: bool,
    /// Whether FFI tracked
    pub ffi_tracked: bool,
    /// Lifecycle events
    pub lifecycle_events: Vec<LifecycleEventInfo>,
    /// Cross-boundary events
    pub cross_boundary_events: Vec<BoundaryEventInfo>,
    /// Risk level
    pub risk_level: String,
    /// Risk confidence
    pub risk_confidence: f64,
}

/// System resources information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// OS name
    pub os_name: String,
    /// OS version
    pub os_version: String,
    /// CPU architecture
    pub architecture: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// Total physical memory (formatted)
    pub total_physical: String,
    /// Available physical memory (formatted)
    pub available_physical: String,
    /// Used physical memory (formatted)
    pub used_physical: String,
    /// Page size
    pub page_size: u64,
}

/// Thread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID (formatted as "Thread-N" instead of "ThreadId(N)")
    pub thread_id: String,
    /// Thread summary (e.g., "5 allocs, 1.2KB")
    pub thread_summary: String,
    /// Number of allocations
    pub allocation_count: usize,
    /// Current memory usage
    pub current_memory: String,
    /// Peak memory usage
    pub peak_memory: String,
    /// Total allocated
    pub total_allocated: String,
    /// Raw current memory in bytes for sorting
    pub current_memory_bytes: usize,
    /// Raw peak memory in bytes for sorting
    pub peak_memory_bytes: usize,
    /// Raw total allocated in bytes for sorting
    pub total_allocated_bytes: usize,
}

struct ThreadAggregator {
    allocation_count: usize,
    current_memory: usize,
    peak_memory: usize,
    total_allocated: usize,
}

/// Dashboard renderer
pub struct DashboardRenderer {
    handlebars: Handlebars<'static>,
}

impl DashboardRenderer {
    /// Create a new dashboard renderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();

        let template_path = format!(
            "{}/src/render_engine/dashboard/templates/dashboard_unified.html",
            env!("CARGO_MANIFEST_DIR")
        );
        handlebars.register_template_file("dashboard_unified", &template_path)?;

        handlebars.register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars.register_helper("gt", Box::new(greater_than_helper));
        handlebars.register_helper("contains", Box::new(contains_helper));
        handlebars.register_helper("json", Box::new(json_helper));

        Ok(Self { handlebars })
    }

    /// Extract user source file from stack trace (filter out Rust internals)
    fn extract_user_source_file(stack_trace: &Option<Vec<String>>) -> Option<String> {
        if let Some(ref frames) = stack_trace {
            for frame in frames {
                let frame_lower = frame.to_lowercase();
                if !frame_lower.contains("/rustc/")
                    && !frame_lower.contains("/library/")
                    && !frame_lower.contains("memscope")
                    && !frame_lower.contains(".cargo/registry")
                    && !frame_lower.contains("/src/core/")
                    && !frame_lower.contains("/src/capture/")
                    && !frame_lower.contains("/src/unified/")
                    && !frame_lower.contains("/src/tracker")
                {
                    if let Some(file_part) = frame.split(':').next() {
                        let file_name = file_part.split('/').last().unwrap_or(file_part);
                        if !file_name.starts_with('<') && file_name.contains(".rs") {
                            return Some(file_part.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract user source line from stack trace (filter out Rust internals)
    fn extract_user_source_line(stack_trace: &Option<Vec<String>>) -> Option<u32> {
        if let Some(ref frames) = stack_trace {
            for frame in frames {
                let frame_lower = frame.to_lowercase();
                if !frame_lower.contains("/rustc/")
                    && !frame_lower.contains("/library/")
                    && !frame_lower.contains("memscope")
                    && !frame_lower.contains(".cargo/registry")
                    && !frame_lower.contains("/src/core/")
                    && !frame_lower.contains("/src/capture/")
                    && !frame_lower.contains("/src/unified/")
                    && !frame_lower.contains("/src/tracker")
                {
                    if let Some(line_part) = frame.rsplit(':').next() {
                        if let Ok(line) = line_part.parse::<u32>() {
                            return Some(line);
                        }
                    }
                }
            }
        }
        None
    }

    /// Build dashboard context from tracker data
    pub fn build_context_from_tracker(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> Result<DashboardContext, Box<dyn std::error::Error>> {
        let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
        let passports = passport_tracker.get_all_passports();
        let analysis = tracker.analyze();

        let total_memory: usize = allocations.iter().map(|a| a.size).sum();

        // Build allocation info
        let alloc_info: Vec<AllocationInfo> = allocations
            .iter()
            .map(|a| {
                let type_name = a.type_name.clone().unwrap_or_else(|| "unknown".to_string());
                let timestamp_alloc = a.allocated_at_ns;
                let timestamp_dealloc = 0u64;
                let lifetime_ms = 0.0;

                // Determine if smart pointer
                let is_smart_pointer = type_name.contains("Arc")
                    || type_name.contains("Rc")
                    || type_name.contains("Box");
                let smart_pointer_type = if type_name.contains("Arc") {
                    "Arc".to_string()
                } else if type_name.contains("Rc") {
                    "Rc".to_string()
                } else if type_name.contains("Box") {
                    "Box".to_string()
                } else {
                    String::new()
                };

                AllocationInfo {
                    address: format!("0x{:x}", a.ptr),
                    type_name: type_name.clone(),
                    size: a.size,
                    var_name: a.var_name.clone().unwrap_or_else(|| "unknown".to_string()),
                    timestamp: format!("{:?}", a.allocated_at_ns),
                    thread_id: format!("{}", a.thread_id),
                    immutable_borrows: 0,
                    mutable_borrows: 0,
                    is_clone: false,
                    clone_count: 0,
                    timestamp_alloc,
                    timestamp_dealloc,
                    lifetime_ms,
                    is_leaked: true,
                    allocation_type: "heap".to_string(),
                    is_smart_pointer,
                    smart_pointer_type,
                    source_file: Self::extract_user_source_file(&a.stack_trace),
                    source_line: Self::extract_user_source_line(&a.stack_trace),
                }
            })
            .collect();

        // Build variable relationships with real relationship types
        let mut relationships: Vec<RelationshipInfo> = Vec::new();

        for (i, a1) in alloc_info.iter().enumerate() {
            for a2 in alloc_info.iter().skip(i + 1) {
                // Clone relationships (same type and size, both clones)
                if a1.is_clone && a2.is_clone && a1.type_name == a2.type_name && a1.size == a2.size
                {
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: "clone".to_string(),
                        strength: 0.9,
                        type_name: a1.type_name.clone(),
                        color: "#10b981".to_string(),
                        is_part_of_cycle: false,
                    });
                }

                // Ownership transfer (same pointer, different clone count)
                if a1.address == a2.address && a1.clone_count != a2.clone_count {
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: "ownership_transfer".to_string(),
                        strength: 1.0,
                        type_name: a1.type_name.clone(),
                        color: "#dc2626".to_string(),
                        is_part_of_cycle: false,
                    });
                }

                // Borrow relationships (same address, different borrow counts)
                if a1.address == a2.address
                    && (a1.immutable_borrows != a2.immutable_borrows
                        || a1.mutable_borrows != a2.mutable_borrows)
                {
                    let borrow_type = if a1.mutable_borrows > 0 || a2.mutable_borrows > 0 {
                        "mutable_borrow"
                    } else {
                        "immutable_borrow"
                    };
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: borrow_type.to_string(),
                        strength: 0.8,
                        type_name: a1.type_name.clone(),
                        color: "#3b82f6".to_string(),
                        is_part_of_cycle: false,
                    });
                }

                // Smart pointer relationships (Arc/Rc with same original data)
                if a1.is_smart_pointer
                    && a2.is_smart_pointer
                    && a1.smart_pointer_type == a2.smart_pointer_type
                {
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: format!(
                            "smart_pointer_{}",
                            a1.smart_pointer_type.to_lowercase()
                        ),
                        strength: 0.7,
                        type_name: a1.type_name.clone(),
                        color: "#8b5cf6".to_string(),
                        is_part_of_cycle: false,
                    });
                }
            }
        }

        // Remove duplicates
        relationships
            .sort_by(|a, b| (&a.source_ptr, &a.target_ptr).cmp(&(&b.source_ptr, &b.target_ptr)));
        relationships.dedup_by(|a, b| a.source_ptr == b.source_ptr && a.target_ptr == b.target_ptr);

        // Detect cycles in relationships and mark cycle edges
        let cycle_edges: std::collections::HashSet<(String, String)> = {
            let rel_tuples: Vec<(String, String, String)> = relationships
                .iter()
                .map(|r| {
                    (
                        r.source_ptr.clone(),
                        r.target_ptr.clone(),
                        r.type_name.clone(),
                    )
                })
                .collect();
            let result = crate::analysis::detect_cycles_in_relationships(&rel_tuples);
            result.cycle_edges
        };

        for rel in &mut relationships {
            if cycle_edges.contains(&(rel.source_ptr.clone(), rel.target_ptr.clone())) {
                rel.is_part_of_cycle = true;
                rel.color = "#ef4444".to_string();
            }
        }

        // Build unsafe reports from passports
        let unsafe_reports: Vec<UnsafeReport> = passports.values()
            .filter(|p| !p.lifecycle_events.is_empty())
            .map(|p| {
                // Build lifecycle events
                let lifecycle_events: Vec<LifecycleEventInfo> = p.lifecycle_events.iter()
                    .map(|event| {
                        let (icon, color, context) = match &event.event_type {
                            crate::analysis::memory_passport_tracker::PassportEventType::AllocatedInRust => {
                                ("🟢".to_string(), "#10b981".to_string(), "Rust Allocation".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::HandoverToFfi => {
                                ("⬇️".to_string(), "#f59e0b".to_string(), "Handover to FFI".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::FreedByForeign => {
                                ("🔵".to_string(), "#3b82f6".to_string(), "Freed by Foreign".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::ReclaimedByRust => {
                                ("⬆️".to_string(), "#10b981".to_string(), "Reclaimed by Rust".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::BoundaryAccess => {
                                ("🔄".to_string(), "#8b5cf6".to_string(), "Boundary Access".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::OwnershipTransfer => {
                                ("↔️".to_string(), "#dc2626".to_string(), "Ownership Transfer".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::ValidationCheck => {
                                ("✅".to_string(), "#10b981".to_string(), "Validation Check".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::CorruptionDetected => {
                                ("🚨".to_string(), "#dc2626".to_string(), "Corruption Detected".to_string())
                            }
                        };

                        LifecycleEventInfo {
                            event_type: format!("{:?}", event.event_type),
                            timestamp: event.timestamp,
                            context,
                            icon,
                            color,
                        }
                    })
                    .collect();

                // Build cross-boundary events
                let cross_boundary_events: Vec<BoundaryEventInfo> = lifecycle_events.iter()
                    .filter(|e| e.event_type.contains("Handover") || e.event_type.contains("Reclaimed"))
                    .map(|e| {
                        let (event_type, from, to, icon, color) = if e.event_type.contains("HandoverToFfi") {
                            ("RustToFfi".to_string(), "Rust".to_string(), "FFI".to_string(), "⬇️".to_string(), "#f59e0b".to_string())
                        } else if e.event_type.contains("ReclaimedByRust") {
                            ("FfiToRust".to_string(), "FFI".to_string(), "Rust".to_string(), "⬆️".to_string(), "#10b981".to_string())
                        } else {
                            (e.event_type.clone(), "Unknown".to_string(), "Unknown".to_string(), "❓".to_string(), "#6b7280".to_string())
                        };

                        BoundaryEventInfo {
                            event_type,
                            from_context: from,
                            to_context: to,
                            timestamp: e.timestamp,
                            icon,
                            color,
                        }
                    })
                    .collect();

                // Determine risk level
                let is_leaked = p.status_at_shutdown == crate::analysis::memory_passport_tracker::PassportStatus::InForeignCustody ||
                               p.status_at_shutdown == crate::analysis::memory_passport_tracker::PassportStatus::HandoverToFfi;
                let risk_level = if is_leaked {
                    "high".to_string()
                } else if !cross_boundary_events.is_empty() {
                    "medium".to_string()
                } else {
                    "low".to_string()
                };

                // Find variable name from allocations
                let var_name = allocations.iter()
                    .find(|a| a.ptr == p.allocation_ptr)
                    .and_then(|a| a.var_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let type_name = allocations.iter()
                    .find(|a| a.ptr == p.allocation_ptr)
                    .and_then(|a| a.type_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                // Risk factors
                let mut risk_factors = Vec::new();
                if is_leaked {
                    risk_factors.push("Memory leaked at shutdown".to_string());
                }
                if !cross_boundary_events.is_empty() {
                    risk_factors.push(format!("Crosses FFI boundary {} times", cross_boundary_events.len()));
                }
                if cross_boundary_events.len() > 3 {
                    risk_factors.push("Frequent boundary crossings".to_string());
                }

                UnsafeReport {
                    passport_id: p.passport_id.clone(),
                    allocation_ptr: format!("0x{:x}", p.allocation_ptr),
                    var_name,
                    type_name,
                    size_bytes: p.size_bytes,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                    status: format!("{:?}", p.status_at_shutdown),
                    lifecycle_events,
                    cross_boundary_events,
                    is_leaked,
                    risk_level,
                    risk_factors,
                }
            })
            .collect();

        // Build passport details
        let passport_details: Vec<PassportDetail> = passports.values()
            .map(|p| {
                // Build lifecycle events
                let lifecycle_events: Vec<LifecycleEventInfo> = p.lifecycle_events.iter()
                    .map(|event| {
                        let (icon, color, context) = match &event.event_type {
                            crate::analysis::memory_passport_tracker::PassportEventType::AllocatedInRust => {
                                ("🟢".to_string(), "#10b981".to_string(), "Rust Allocation".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::HandoverToFfi => {
                                ("⬇️".to_string(), "#f59e0b".to_string(), "Handover to FFI".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::FreedByForeign => {
                                ("🔵".to_string(), "#3b82f6".to_string(), "Freed by Foreign".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::ReclaimedByRust => {
                                ("⬆️".to_string(), "#10b981".to_string(), "Reclaimed by Rust".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::BoundaryAccess => {
                                ("🔄".to_string(), "#8b5cf6".to_string(), "Boundary Access".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::OwnershipTransfer => {
                                ("↔️".to_string(), "#dc2626".to_string(), "Ownership Transfer".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::ValidationCheck => {
                                ("✅".to_string(), "#10b981".to_string(), "Validation Check".to_string())
                            }
                            crate::analysis::memory_passport_tracker::PassportEventType::CorruptionDetected => {
                                ("🚨".to_string(), "#dc2626".to_string(), "Corruption Detected".to_string())
                            }
                        };

                        LifecycleEventInfo {
                            event_type: format!("{:?}", event.event_type),
                            timestamp: event.timestamp,
                            context,
                            icon: icon.to_string(),
                            color,
                        }
                    })
                    .collect();

                // Build cross-boundary events
                let cross_boundary_events: Vec<BoundaryEventInfo> = lifecycle_events.iter()
                    .filter(|e| e.event_type.contains("Handover") || e.event_type.contains("Reclaimed"))
                    .map(|e| {
                        let (event_type, from, to, icon, color) = if e.event_type.contains("HandoverToFfi") {
                            ("RustToFfi".to_string(), "Rust".to_string(), "FFI".to_string(), "⬇️".to_string(), "#f59e0b".to_string())
                        } else if e.event_type.contains("ReclaimedByRust") {
                            ("FfiToRust".to_string(), "FFI".to_string(), "Rust".to_string(), "⬆️".to_string(), "#10b981".to_string())
                        } else {
                            (e.event_type.clone(), "Unknown".to_string(), "Unknown".to_string(), "❓".to_string(), "#6b7280".to_string())
                        };

                        BoundaryEventInfo {
                            event_type,
                            from_context: from,
                            to_context: to,
                            timestamp: e.timestamp,
                            icon,
                            color,
                        }
                    })
                    .collect();

                // Find variable name from allocations
                let var_name = allocations.iter()
                    .find(|a| a.ptr == p.allocation_ptr)
                    .and_then(|a| a.var_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                let type_name = allocations.iter()
                    .find(|a| a.ptr == p.allocation_ptr)
                    .and_then(|a| a.type_name.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                // Determine risk level
                let is_leaked = p.status_at_shutdown == crate::analysis::memory_passport_tracker::PassportStatus::InForeignCustody ||
                               p.status_at_shutdown == crate::analysis::memory_passport_tracker::PassportStatus::HandoverToFfi;
                let risk_level = if is_leaked {
                    "high".to_string()
                } else if !cross_boundary_events.is_empty() {
                    "medium".to_string()
                } else {
                    "low".to_string()
                };

                PassportDetail {
                    passport_id: p.passport_id.clone(),
                    allocation_ptr: format!("0x{:x}", p.allocation_ptr),
                    var_name,
                    type_name,
                    size_bytes: p.size_bytes,
                    status: format!("{:?}", p.status_at_shutdown),
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                    is_leaked,
                    ffi_tracked: !cross_boundary_events.is_empty(),
                    lifecycle_events,
                    cross_boundary_events,
                    risk_level,
                    risk_confidence: 0.85, // Default confidence
                }
            })
            .collect();

        // Perform leak detection
        let leak_result = passport_tracker.detect_leaks_at_shutdown();
        let leak_count = leak_result.leaked_passports.len();

        // Build thread data from allocations
        let thread_data = Self::aggregate_thread_data(&alloc_info);

        // Prepare JSON data for direct injection (performance optimization)
        #[allow(dead_code)]
        #[derive(serde::Serialize)]
        struct DashboardData<'a> {
            allocations: &'a [AllocationInfo],
            relationships: &'a [RelationshipInfo],
            unsafe_reports: &'a [UnsafeReport],
            threads: &'a [ThreadInfo],
            passport_details: &'a [PassportDetail],
            active_allocations: usize,
            total_allocations: usize,
            leak_count: usize,
        }

        let data = DashboardData {
            allocations: &alloc_info,
            relationships: &relationships,
            unsafe_reports: &unsafe_reports,
            threads: &thread_data,
            passport_details: &passport_details,
            active_allocations: analysis.active_allocations,
            total_allocations: analysis.total_allocations,
            leak_count,
        };

        let json_data: String = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize dashboard data: {}", e))?;

        // Get system information directly using platform-specific functions
        let (
            os_name,
            os_version,
            architecture,
            cpu_cores,
            page_size,
            total_physical,
            available_physical,
            used_physical,
        ) = {
            #[cfg(target_os = "macos")]
            {
                // Get OS version
                let os_version = unsafe {
                    let mut size: libc::size_t = 256;
                    let mut buf = [0u8; 256];
                    if libc::sysctlbyname(
                        b"kern.osrelease\0".as_ptr() as *const libc::c_char,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) == 0
                    {
                        String::from_utf8_lossy(&buf[..size - 1]).to_string()
                    } else {
                        "Unknown".to_string()
                    }
                };

                // Get architecture
                let architecture = unsafe {
                    let mut size: libc::size_t = 256;
                    let mut buf = [0u8; 256];
                    if libc::sysctlbyname(
                        b"hw.machine\0".as_ptr() as *const libc::c_char,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) == 0
                    {
                        let arch_str = String::from_utf8_lossy(&buf[..size - 1]).to_string();
                        if arch_str.contains("arm64") || arch_str.contains("arm") {
                            "arm64".to_string()
                        } else {
                            arch_str
                        }
                    } else {
                        "unknown".to_string()
                    }
                };

                // Get CPU cores
                let mut size = std::mem::size_of::<u32>();
                let mut cpu_cores: u32 = 1;
                unsafe {
                    let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_NCPU];
                    if libc::sysctl(
                        mib.as_mut_ptr(),
                        mib.len() as libc::c_uint,
                        &mut cpu_cores as *mut u32 as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) == 0
                    {
                        // Successfully got CPU cores
                    }
                }

                // Get page size
                let mut page_size: u64 = 4096;
                unsafe {
                    size = std::mem::size_of::<u64>();
                    if libc::sysctlbyname(
                        b"hw.pagesize\0".as_ptr() as *const libc::c_char,
                        &mut page_size as *mut u64 as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) != 0
                    {
                        page_size = 4096;
                    }
                }

                // Get total physical memory
                let mut total: u64 = 0;
                let mut size = std::mem::size_of::<u64>();
                unsafe {
                    let mut mib: [libc::c_int; 2] = [libc::CTL_HW, libc::HW_MEMSIZE];
                    if libc::sysctl(
                        mib.as_mut_ptr(),
                        mib.len() as libc::c_uint,
                        &mut total as *mut u64 as *mut libc::c_void,
                        &mut size,
                        std::ptr::null_mut(),
                        0,
                    ) != 0
                    {
                        total = 16 * 1024 * 1024 * 1024; // 16GB default
                    }
                }

                // Get available memory
                let mut vm_stats: libc::vm_statistics64 = unsafe { std::mem::zeroed() };
                let mut count = libc::HOST_VM_INFO64_COUNT;
                let (available_physical, used_physical) = unsafe {
                    if libc::host_statistics64(
                        libc::mach_host_self(),
                        libc::HOST_VM_INFO64,
                        &mut vm_stats as *mut _ as libc::host_info64_t,
                        &mut count,
                    ) != 0
                    {
                        // Fall back to simple calculation
                        (total / 2, total / 2)
                    } else {
                        let free = vm_stats.free_count as u64 * page_size;
                        let active = vm_stats.active_count as u64 * page_size;
                        let inactive = vm_stats.inactive_count as u64 * page_size;
                        let wired = vm_stats.wire_count as u64 * page_size;
                        let used = active + wired;
                        let available = free + inactive;

                        (available, used)
                    }
                };

                (
                    "macOS".to_string(),
                    os_version,
                    architecture,
                    cpu_cores,
                    page_size,
                    total,
                    available_physical,
                    used_physical,
                )
            }

            #[cfg(not(target_os = "macos"))]
            {
                (
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "unknown".to_string(),
                    1,
                    4096,
                    16 * 1024 * 1024 * 1024,
                    8 * 1024 * 1024 * 1024,
                    8 * 1024 * 1024 * 1024,
                )
            }
        };

        let context = DashboardContext {
            title: "MemScope Dashboard".to_string(),
            export_timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            total_memory: format_bytes(total_memory),
            total_allocations: analysis.total_allocations,
            active_allocations: analysis.active_allocations,
            peak_memory: format_bytes(analysis.peak_memory_bytes as usize),
            thread_count: 1,
            passport_count: passports.len(),
            leak_count,
            unsafe_count: unsafe_reports.len(),
            ffi_count: unsafe_reports.len(),
            allocations: alloc_info.clone(),
            relationships: relationships.clone(),
            unsafe_reports: unsafe_reports.clone(),
            passport_details: passport_details.clone(),
            allocations_count: alloc_info.len(),
            relationships_count: relationships.len(),
            unsafe_reports_count: unsafe_reports.len(),
            json_data,
            os_name: os_name.clone(),
            architecture: architecture.clone(),
            cpu_cores: cpu_cores as usize,
            system_resources: SystemResources {
                os_name: os_name.clone(),
                os_version: os_version.clone(),
                architecture: architecture.clone(),
                cpu_cores,
                total_physical: format_bytes(total_physical as usize),
                available_physical: format_bytes(available_physical as usize),
                used_physical: format_bytes(used_physical as usize),
                page_size,
            },
            threads: Self::aggregate_thread_data(&alloc_info),
        };

        Ok(context)
    }

    fn aggregate_thread_data(allocations: &[AllocationInfo]) -> Vec<ThreadInfo> {
        use std::collections::HashMap;
        let mut thread_map: HashMap<String, ThreadAggregator> = HashMap::new();

        for alloc in allocations {
            let entry = thread_map
                .entry(alloc.thread_id.clone())
                .or_insert_with(|| ThreadAggregator {
                    allocation_count: 0,
                    current_memory: 0,
                    peak_memory: 0,
                    total_allocated: 0,
                });
            entry.allocation_count += 1;
            entry.current_memory += alloc.size;
            entry.total_allocated += alloc.size;
            if alloc.size > entry.peak_memory {
                entry.peak_memory = alloc.size;
            }
        }

        thread_map
            .into_iter()
            .map(|(raw_tid, agg)| {
                let summary = format!(
                    "{} allocs, {}",
                    agg.allocation_count,
                    format_bytes(agg.current_memory)
                );
                let thread_id = format_thread_id(&raw_tid);
                ThreadInfo {
                    thread_id,
                    thread_summary: summary,
                    allocation_count: agg.allocation_count,
                    current_memory: format_bytes(agg.current_memory),
                    peak_memory: format_bytes(agg.peak_memory),
                    total_allocated: format_bytes(agg.total_allocated),
                    current_memory_bytes: agg.current_memory,
                    peak_memory_bytes: agg.peak_memory,
                    total_allocated_bytes: agg.total_allocated,
                }
            })
            .collect()
    }

    /// Render dashboard from tracker data (for standalone template)
    pub fn render_from_tracker(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let context = self.build_context_from_tracker(tracker, passport_tracker)?;
        self.render_dashboard(&context)
    }

    /// Render dashboard from context
    pub fn render_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.render_unified_dashboard(context)
    }

    /// Render standalone dashboard (no external dependencies, works with file:// protocol)
    pub fn render_standalone_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.render_unified_dashboard(context)
    }

    /// Render unified dashboard (multi-mode in single HTML)
    pub fn render_unified_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert(
            "title".to_string(),
            serde_json::Value::String(context.title.clone()),
        );
        template_data.insert(
            "export_timestamp".to_string(),
            serde_json::Value::String(context.export_timestamp.clone()),
        );
        template_data.insert(
            "total_memory".to_string(),
            serde_json::Value::String(context.total_memory.clone()),
        );
        template_data.insert(
            "total_allocations".to_string(),
            serde_json::Value::Number(context.total_allocations.into()),
        );
        template_data.insert(
            "active_allocations".to_string(),
            serde_json::Value::Number(context.active_allocations.into()),
        );
        template_data.insert(
            "peak_memory".to_string(),
            serde_json::Value::String(context.peak_memory.clone()),
        );
        template_data.insert(
            "thread_count".to_string(),
            serde_json::Value::Number(context.thread_count.into()),
        );
        template_data.insert(
            "passport_count".to_string(),
            serde_json::Value::Number(context.passport_count.into()),
        );
        template_data.insert(
            "leak_count".to_string(),
            serde_json::Value::Number(context.leak_count.into()),
        );
        template_data.insert(
            "unsafe_count".to_string(),
            serde_json::Value::Number(context.unsafe_count.into()),
        );
        template_data.insert(
            "ffi_count".to_string(),
            serde_json::Value::Number(context.ffi_count.into()),
        );
        template_data.insert(
            "allocations_count".to_string(),
            serde_json::Value::Number(context.allocations_count.into()),
        );
        template_data.insert(
            "relationships_count".to_string(),
            serde_json::Value::Number(context.relationships_count.into()),
        );
        template_data.insert(
            "unsafe_reports_count".to_string(),
            serde_json::Value::Number(context.unsafe_reports_count.into()),
        );
        template_data.insert(
            "os_name".to_string(),
            serde_json::Value::String(context.os_name.clone()),
        );
        template_data.insert(
            "architecture".to_string(),
            serde_json::Value::String(context.architecture.clone()),
        );
        template_data.insert(
            "cpu_cores".to_string(),
            serde_json::Value::Number(context.cpu_cores.into()),
        );
        template_data.insert(
            "json_data".to_string(),
            serde_json::Value::String(context.json_data.clone()),
        );

        template_data.insert(
            "allocations".to_string(),
            serde_json::to_value(&context.allocations)?,
        );
        template_data.insert(
            "passport_details".to_string(),
            serde_json::to_value(&context.passport_details)?,
        );
        template_data.insert(
            "relationships".to_string(),
            serde_json::to_value(&context.relationships)?,
        );
        template_data.insert(
            "unsafe_reports".to_string(),
            serde_json::to_value(&context.unsafe_reports)?,
        );
        template_data.insert(
            "threads".to_string(),
            serde_json::to_value(&context.threads)?,
        );

        self.handlebars
            .render("dashboard_unified", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render binary dashboard (legacy template)
    pub fn render_binary_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let legacy_data = self.to_legacy_binary_data(context);
        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert("BINARY_DATA".to_string(), legacy_data);
        template_data.insert(
            "PROJECT_NAME".to_string(),
            serde_json::Value::String("MemScope Memory Analysis".to_string()),
        );

        self.handlebars
            .render("binary_dashboard", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render clean dashboard (legacy template)
    pub fn render_clean_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let legacy_data = self.to_legacy_binary_data(context);
        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert("BINARY_DATA".to_string(), legacy_data.clone());
        template_data.insert("json_data".to_string(), legacy_data);
        template_data.insert(
            "PROJECT_NAME".to_string(),
            serde_json::Value::String("MemScope Memory Analysis".to_string()),
        );

        self.handlebars
            .render("clean_dashboard", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render hybrid dashboard (legacy template)
    pub fn render_hybrid_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let variables_data = serde_json::Value::Array(
            context
                .allocations
                .iter()
                .map(|a| {
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "var_name".to_string(),
                        serde_json::Value::String(a.var_name.clone()),
                    );
                    map.insert(
                        "type_name".to_string(),
                        serde_json::Value::String(a.type_name.clone()),
                    );
                    map.insert("size".to_string(), serde_json::Value::Number(a.size.into()));
                    map.insert(
                        "address".to_string(),
                        serde_json::Value::String(a.address.clone()),
                    );
                    map.insert(
                        "is_leaked".to_string(),
                        serde_json::Value::Bool(a.is_leaked),
                    );
                    map.insert(
                        "timestamp_alloc".to_string(),
                        serde_json::Value::Number(a.timestamp_alloc.into()),
                    );
                    map.insert(
                        "timestamp_dealloc".to_string(),
                        serde_json::Value::Number(a.timestamp_dealloc.into()),
                    );
                    map.insert(
                        "thread_id".to_string(),
                        serde_json::Value::String(a.thread_id.clone()),
                    );
                    serde_json::Value::Object(map)
                })
                .collect(),
        );

        let threads_data = serde_json::Value::Array(
            context
                .threads
                .iter()
                .map(|t| {
                    let mut map = serde_json::Map::new();
                    map.insert(
                        "thread_id".to_string(),
                        serde_json::Value::String(t.thread_id.clone()),
                    );
                    map.insert(
                        "allocation_count".to_string(),
                        serde_json::Value::String(t.allocation_count.to_string()),
                    );
                    map.insert(
                        "current_memory".to_string(),
                        serde_json::Value::String(t.current_memory.clone()),
                    );
                    map.insert(
                        "peak_memory".to_string(),
                        serde_json::Value::String(t.peak_memory.clone()),
                    );
                    map.insert(
                        "total_allocated".to_string(),
                        serde_json::Value::String(t.total_allocated.clone()),
                    );
                    serde_json::Value::Object(map)
                })
                .collect(),
        );

        let tasks_data = serde_json::Value::Array(Vec::new());

        let total_memory: usize = context.allocations.iter().map(|a| a.size).sum();
        let efficiency = if context.total_allocations > 0 {
            (context.active_allocations as f64 / context.total_allocations as f64 * 100.0) as usize
        } else {
            100
        };

        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert("VARIABLES_DATA".to_string(), variables_data);
        template_data.insert("THREADS_DATA".to_string(), threads_data);
        template_data.insert("TASKS_DATA".to_string(), tasks_data);
        template_data.insert(
            "PROJECT_NAME".to_string(),
            serde_json::Value::String("MemScope Memory Analysis".to_string()),
        );
        template_data.insert(
            "TOTAL_MEMORY".to_string(),
            serde_json::Value::String(format_bytes(total_memory)),
        );
        template_data.insert(
            "TOTAL_VARIABLES".to_string(),
            serde_json::Value::Number(context.allocations.len().into()),
        );
        template_data.insert(
            "THREAD_COUNT".to_string(),
            serde_json::Value::Number(context.thread_count.into()),
        );
        template_data.insert(
            "EFFICIENCY".to_string(),
            serde_json::Value::String(format!("{}%", efficiency)),
        );

        self.handlebars
            .render("hybrid_dashboard", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render performance dashboard (legacy template)
    pub fn render_performance_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Prepare performance data in expected format
        let performance_data = serde_json::json!({
            "allocations": context.allocations.iter().map(|a| {
                serde_json::json!({
                    "timestamp": a.timestamp_alloc,
                    "memory": a.size,
                    "var_name": a.var_name,
                    "type_name": a.type_name
                })
            }).collect::<Vec<_>>(),
            "total_memory": context.total_memory,
            "peak_memory": context.peak_memory,
            "allocations_count": context.total_allocations,
            "thread_count": context.thread_count
        });

        // Prepare efficiency data
        let efficiency_data = serde_json::json!({
            "memory_efficiency": if context.total_allocations > 0 {
                (context.active_allocations as f64 / context.total_allocations as f64 * 100.0)
            } else { 100.0 },
            "fragmentation": "0.0", // TODO: Calculate actual fragmentation
            "reclamation_rate": "0.0", // TODO: Calculate actual reclamation rate
            "average_size": if context.allocations.is_empty() {
                0
            } else {
                context.allocations.iter().map(|a| a.size).sum::<usize>() / context.allocations.len()
            }
        });

        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert(
            "PERFORMANCE_DATA",
            serde_json::to_string(&performance_data)?,
        );
        template_data.insert("EFFICIENCY_DATA", serde_json::to_string(&efficiency_data)?);
        template_data.insert("PROJECT_NAME", "MemScope Memory Analysis".to_string());

        self.handlebars
            .render("performance_dashboard", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render multithread dashboard (new template for thread analysis)
    pub fn render_multithread_dashboard(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let threads_data = self.prepare_thread_data(context)?;
        let allocation_data = self.prepare_allocation_timeline_data(context)?;
        let conflict_data = self.prepare_conflict_data(context)?;

        let conflict_count = conflict_data.as_array().map(|a| a.len()).unwrap_or(0);
        let mut template_data = std::collections::BTreeMap::new();
        template_data.insert("THREADS_DATA".to_string(), threads_data);
        template_data.insert("ALLOCATION_DATA".to_string(), allocation_data);
        template_data.insert("CONFLICT_DATA".to_string(), conflict_data);
        template_data.insert(
            "PROJECT_NAME".to_string(),
            serde_json::Value::String("MemScope Memory Analysis".to_string()),
        );
        template_data.insert(
            "THREAD_COUNT".to_string(),
            serde_json::Value::Number(context.thread_count.into()),
        );
        template_data.insert(
            "TOTAL_MEMORY".to_string(),
            serde_json::Value::String(context.total_memory.clone()),
        );
        template_data.insert(
            "TOTAL_ALLOCATIONS".to_string(),
            serde_json::Value::Number(context.total_allocations.into()),
        );
        template_data.insert(
            "CONFLICT_COUNT".to_string(),
            serde_json::Value::Number(conflict_count.into()),
        );

        self.handlebars
            .render("multithread_template", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Prepare thread data for multithread dashboard
    fn prepare_thread_data(
        &self,
        context: &DashboardContext,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut thread_map: std::collections::HashMap<String, ThreadStats> =
            std::collections::HashMap::new();

        for allocation in &context.allocations {
            let thread_id = allocation.thread_id.clone();
            let stats = thread_map
                .entry(thread_id.clone())
                .or_insert_with(|| ThreadStats {
                    id: thread_id.parse::<u64>().unwrap_or(0),
                    allocations: 0,
                    memory: 0,
                    peak: 0,
                    status: "active".to_string(),
                });

            stats.allocations += 1;
            stats.memory += allocation.size;
            if allocation.size > stats.peak {
                stats.peak = allocation.size;
            }
        }

        let threads: Vec<ThreadStats> = thread_map.into_values().collect();
        serde_json::to_value(&threads).map_err(|e| e.into())
    }

    /// Prepare allocation timeline data for multithread dashboard
    fn prepare_allocation_timeline_data(
        &self,
        context: &DashboardContext,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let timeline: Vec<TimelineAllocation> = context
            .allocations
            .iter()
            .map(|a| TimelineAllocation {
                timestamp: a.timestamp_alloc,
                thread_id: a.thread_id.parse::<u64>().unwrap_or(0),
                size: a.size,
                var_name: Some(a.var_name.clone()),
            })
            .collect();

        serde_json::to_value(&timeline).map_err(|e| e.into())
    }

    /// Prepare conflict data for multithread dashboard
    fn prepare_conflict_data(
        &self,
        context: &DashboardContext,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut conflicts: Vec<ThreadConflict> = Vec::new();

        let mut address_map: std::collections::HashMap<String, Vec<&AllocationInfo>> =
            std::collections::HashMap::new();

        for allocation in &context.allocations {
            address_map
                .entry(allocation.address.clone())
                .or_default()
                .push(allocation);
        }

        for (address, allocations) in &address_map {
            if allocations.len() > 1 {
                let thread_ids: Vec<u64> = allocations
                    .iter()
                    .map(|a| a.thread_id.parse::<u64>().unwrap_or(0))
                    .collect();
                let unique_threads: std::collections::HashSet<u64> =
                    thread_ids.iter().cloned().collect();

                if unique_threads.len() > 1 {
                    conflicts.push(ThreadConflict {
                        description: format!(
                            "Address {} accessed by {} threads",
                            address,
                            unique_threads.len()
                        ),
                        threads: thread_ids
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        conflict_type: "Data Race".to_string(),
                    });
                }
            }
        }

        serde_json::to_value(&conflicts).map_err(|e| e.into())
    }

    /// Build base async data map with common fields
    fn build_async_base_map(
        context: &DashboardContext,
        subtitle: &str,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "title".to_string(),
            serde_json::Value::String("Async Performance Dashboard".to_string()),
        );
        map.insert(
            "subtitle".to_string(),
            serde_json::Value::String(subtitle.to_string()),
        );
        map.insert(
            "total_tasks".to_string(),
            serde_json::Value::Number(context.allocations.len().into()),
        );
        map.insert(
            "active_tasks".to_string(),
            serde_json::Value::Number(context.active_allocations.into()),
        );
        map.insert(
            "completed_tasks".to_string(),
            serde_json::Value::Number(
                context
                    .allocations
                    .iter()
                    .filter(|a| a.timestamp_dealloc > 0)
                    .count()
                    .into(),
            ),
        );
        map.insert(
            "failed_tasks".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "cpu_usage_avg".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "cpu_usage_peak".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "cpu_cores".to_string(),
            serde_json::Value::Number(context.cpu_cores.into()),
        );
        map.insert(
            "context_switches".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "total_memory_mb".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(Self::parse_bytes_to_mb(&context.total_memory))
                    .unwrap(),
            ),
        );
        map.insert(
            "peak_memory_mb".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(Self::parse_bytes_to_mb(&context.peak_memory))
                    .unwrap(),
            ),
        );
        map.insert(
            "total_allocations".to_string(),
            serde_json::Value::Number(context.total_allocations.into()),
        );
        map.insert(
            "memory_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "io_throughput".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_read_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_write_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_io_ops".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "network_throughput".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_sent_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_received_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_latency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "efficiency_score".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "resource_balance".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "bottleneck_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "optimization_potential".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "futures_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "total_polls".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "avg_poll_time".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "ready_rate".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "cpu_intensive_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "cpu_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "cpu_intensive_tasks".to_string(),
            serde_json::Value::Array(vec![]),
        );
        map.insert(
            "memory_intensive_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "memory_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "memory_intensive_tasks".to_string(),
            serde_json::Value::Array(vec![]),
        );
        map.insert(
            "io_intensive_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "io_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "io_intensive_tasks".to_string(),
            serde_json::Value::Array(vec![]),
        );
        map.insert(
            "network_intensive_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "network_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "network_intensive_tasks".to_string(),
            serde_json::Value::Array(vec![]),
        );
        map.insert(
            "executor_utilization".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_queue_length".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "blocking_tasks_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "deadlock_risk".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "gc_pressure".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_fragmentation".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "peak_alloc_rate".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "waker_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "immediate_ready_percent".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map
    }

    /// Parse bytes string to MB (helper function)
    fn parse_bytes_to_mb(bytes_str: &str) -> f64 {
        let num_str: String = bytes_str
            .chars()
            .filter(|c| c.is_digit(10) || *c == '.')
            .collect();
        let num: f64 = num_str.parse().unwrap_or(0.0);
        if bytes_str.contains("GB") {
            num * 1024.0
        } else if bytes_str.contains("MB") {
            num
        } else if bytes_str.contains("KB") {
            num / 1024.0
        } else {
            num / 1024.0 / 1024.0
        }
    }

    /// Render async template (legacy template)
    pub fn render_async_template(
        &self,
        context: &DashboardContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let has_async_tasks = context.allocations.iter().any(|a| {
            a.type_name.contains("Future")
                || a.type_name.contains("Task")
                || a.type_name.contains("async")
                || a.type_name.contains("Waker")
        });

        let async_data = if has_async_tasks {
            self.prepare_async_data(context)?
        } else {
            serde_json::Value::Object(Self::build_async_base_map(
                context,
                "No async tasks detected",
            ))
        };

        let mut template_data = std::collections::BTreeMap::new();
        if let serde_json::Value::Object(map) = &async_data {
            for (key, value) in map {
                template_data.insert(key.clone(), value.clone());
            }
        }
        template_data.insert(
            "PROJECT_NAME".to_string(),
            serde_json::Value::String("MemScope Async Performance Analysis".to_string()),
        );

        self.handlebars
            .render("async_template", &template_data)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Prepare async-specific data from context
    fn prepare_async_data(
        &self,
        context: &DashboardContext,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut cpu_intensive_tasks: Vec<serde_json::Value> = Vec::new();
        let mut memory_intensive_tasks: Vec<serde_json::Value> = Vec::new();
        let mut io_intensive_tasks: Vec<serde_json::Value> = Vec::new();
        let mut network_intensive_tasks: Vec<serde_json::Value> = Vec::new();

        for (idx, alloc) in context.allocations.iter().enumerate() {
            let task_type = if alloc.type_name.contains("Future") {
                "future"
            } else if alloc.type_name.contains("Task") {
                "task"
            } else if alloc.type_name.contains("Channel") {
                "channel"
            } else {
                "async_op"
            };

            let status = if alloc.is_leaked {
                "leaked"
            } else if alloc.timestamp_dealloc > 0 {
                "completed"
            } else {
                "active"
            };
            let mut task_map = serde_json::Map::new();
            task_map.insert("task_id".to_string(), serde_json::Value::Number(idx.into()));
            task_map.insert(
                "task_name".to_string(),
                serde_json::Value::String(if alloc.var_name.is_empty() {
                    format!("async_{}", idx)
                } else {
                    alloc.var_name.clone()
                }),
            );
            task_map.insert(
                "source_file".to_string(),
                serde_json::Value::String("unknown".to_string()),
            );
            task_map.insert(
                "source_line".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "task_type".to_string(),
                serde_json::Value::String(task_type.to_string()),
            );
            task_map.insert(
                "cpu_usage".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "cpu_cycles".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "instructions".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "cache_misses".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "allocated_mb".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(alloc.size as f64 / 1024.0 / 1024.0).unwrap(),
                ),
            );
            task_map.insert(
                "memory_usage_percent".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "peak_memory_mb".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(alloc.size as f64 / 1024.0 / 1024.0).unwrap(),
                ),
            );
            task_map.insert(
                "allocation_count".to_string(),
                serde_json::Value::Number(1.into()),
            );
            task_map.insert(
                "heap_fragmentation".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "bytes_read_mb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "bytes_written_mb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "avg_latency_us".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "queue_depth".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "bytes_sent_mb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "bytes_received_mb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "active_connections".to_string(),
                serde_json::Value::Number(0.into()),
            );
            task_map.insert(
                "avg_latency_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
            );
            task_map.insert(
                "status".to_string(),
                serde_json::Value::String(status.to_string()),
            );
            task_map.insert(
                "duration_ms".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(alloc.lifetime_ms as f64).unwrap(),
                ),
            );
            let task_data = serde_json::Value::Object(task_map);

            if alloc.type_name.contains("Future") || alloc.type_name.contains("Stream") {
                cpu_intensive_tasks.push(task_data);
            } else if alloc.type_name.contains("Channel") || alloc.type_name.contains("Mutex") {
                memory_intensive_tasks.push(task_data);
            } else if alloc.type_name.contains("Tcp") || alloc.type_name.contains("Udp") {
                network_intensive_tasks.push(task_data);
            } else {
                io_intensive_tasks.push(task_data);
            }
        }

        let memory_efficiency = if context.total_allocations > 0 {
            context.active_allocations as f64 / context.total_allocations as f64 * 100.0
        } else {
            100.0
        };

        let mut map = serde_json::Map::new();
        map.insert(
            "title".to_string(),
            serde_json::Value::String("Async Performance Dashboard".to_string()),
        );
        map.insert(
            "subtitle".to_string(),
            serde_json::Value::String("Rust Async Runtime Analysis".to_string()),
        );
        map.insert(
            "total_tasks".to_string(),
            serde_json::Value::Number(context.allocations.len().into()),
        );
        map.insert(
            "active_tasks".to_string(),
            serde_json::Value::Number(context.active_allocations.into()),
        );
        map.insert(
            "completed_tasks".to_string(),
            serde_json::Value::Number(
                context
                    .allocations
                    .iter()
                    .filter(|a| a.timestamp_dealloc > 0)
                    .count()
                    .into(),
            ),
        );
        map.insert(
            "failed_tasks".to_string(),
            serde_json::Value::Number(context.leak_count.into()),
        );
        map.insert(
            "cpu_usage_avg".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "cpu_usage_peak".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "cpu_cores".to_string(),
            serde_json::Value::Number(context.cpu_cores.into()),
        );
        map.insert(
            "context_switches".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "total_memory_mb".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(Self::parse_bytes_to_mb(&context.total_memory))
                    .unwrap(),
            ),
        );
        map.insert(
            "peak_memory_mb".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(Self::parse_bytes_to_mb(&context.peak_memory))
                    .unwrap(),
            ),
        );
        map.insert(
            "total_allocations".to_string(),
            serde_json::Value::Number(context.total_allocations.into()),
        );
        map.insert(
            "memory_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(memory_efficiency).unwrap()),
        );
        map.insert(
            "io_throughput".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_read_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_write_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_io_ops".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "network_throughput".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_sent_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "total_received_mb".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_latency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "efficiency_score".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "resource_balance".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "bottleneck_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "optimization_potential".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "futures_count".to_string(),
            serde_json::Value::Number(cpu_intensive_tasks.len().into()),
        );
        map.insert(
            "total_polls".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "avg_poll_time".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "ready_rate".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "cpu_intensive_count".to_string(),
            serde_json::Value::Number(cpu_intensive_tasks.len().into()),
        );
        map.insert(
            "cpu_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "cpu_intensive_tasks".to_string(),
            serde_json::Value::Array(cpu_intensive_tasks),
        );
        map.insert(
            "memory_intensive_count".to_string(),
            serde_json::Value::Number(memory_intensive_tasks.len().into()),
        );
        map.insert(
            "memory_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "memory_intensive_tasks".to_string(),
            serde_json::Value::Array(memory_intensive_tasks),
        );
        map.insert(
            "io_intensive_count".to_string(),
            serde_json::Value::Number(io_intensive_tasks.len().into()),
        );
        map.insert(
            "io_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "io_intensive_tasks".to_string(),
            serde_json::Value::Array(io_intensive_tasks),
        );
        map.insert(
            "network_intensive_count".to_string(),
            serde_json::Value::Number(network_intensive_tasks.len().into()),
        );
        map.insert(
            "network_avg_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "network_intensive_tasks".to_string(),
            serde_json::Value::Array(network_intensive_tasks),
        );
        map.insert(
            "executor_utilization".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_queue_length".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "blocking_tasks_count".to_string(),
            serde_json::Value::Number(0.into()),
        );
        map.insert(
            "deadlock_risk".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "gc_pressure".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "avg_fragmentation".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "peak_alloc_rate".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        map.insert(
            "waker_efficiency".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        map.insert(
            "immediate_ready_percent".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(100.0).unwrap()),
        );
        Ok(serde_json::Value::Object(map))
    }

    /// Convert new DashboardContext to legacy binary data format
    fn to_legacy_binary_data(&self, context: &DashboardContext) -> serde_json::Value {
        // Calculate type distribution
        let mut type_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut total_size: usize = 0;

        for alloc in &context.allocations {
            let type_name =
                if alloc.type_name.contains("Vec") || alloc.type_name.contains("vec::Vec") {
                    "dynamic_array"
                } else if alloc.type_name.contains("String") || alloc.type_name.contains("str") {
                    "string"
                } else if alloc.type_name.contains("Box")
                    || alloc.type_name.contains("Rc")
                    || alloc.type_name.contains("Arc")
                {
                    "smart_pointer"
                } else if alloc.type_name.contains("[") && alloc.type_name.contains("u8") {
                    "byte_array"
                } else if alloc.size > 1024 * 1024 {
                    "large_buffer"
                } else {
                    "custom"
                }
                .to_string();

            *type_counts.entry(type_name).or_insert(0) += 1;
            total_size += alloc.size;
        }

        // Calculate statistics
        let average_size = if context.allocations.is_empty() {
            0
        } else {
            total_size / context.allocations.len()
        };

        // Build lifetime events from allocations
        let lifetime_events: Vec<serde_json::Value> = context.allocations.iter().map(|a| {
            serde_json::json!({
                "address": a.address,
                "events": [{
                    "context": "initial_allocation",
                    "event_type": "Created",
                    "timestamp": a.timestamp_alloc
                }],
                "lifetime_ms": a.lifetime_ms,
                "size": a.size,
                "timestamp_alloc": a.timestamp_alloc,
                "timestamp_dealloc": if a.timestamp_dealloc > 0 { Some(a.timestamp_dealloc) } else { None },
                "type_name": a.type_name,
                "var_name": a.var_name
            })
        }).collect();

        serde_json::json!({
            "memory_analysis": {
                "allocations": context.allocations.iter().map(|a| {
                    serde_json::json!({
                        "var_name": a.var_name,
                        "type_name": a.type_name,
                        "size": a.size,
                        "address": a.address,
                        "timestamp": a.timestamp,
                        "timestamp_alloc": a.timestamp_alloc,
                        "timestamp_dealloc": if a.timestamp_dealloc > 0 { Some(a.timestamp_dealloc) } else { None },
                        "lifetime_ms": a.lifetime_ms,
                        "is_leaked": a.is_leaked,
                        "thread_id": a.thread_id,
                        "immutable_borrows": a.immutable_borrows,
                        "mutable_borrows": a.mutable_borrows,
                        "is_clone": a.is_clone,
                        "clone_count": a.clone_count,
                        "allocation_type": a.allocation_type,
                        "is_smart_pointer": a.is_smart_pointer,
                        "smart_pointer_type": a.smart_pointer_type,
                        "borrow_info": {
                            "immutable_borrows": a.immutable_borrows,
                            "max_concurrent_borrows": a.immutable_borrows + a.mutable_borrows,
                            "mutable_borrows": a.mutable_borrows
                        },
                        "clone_info": {
                            "clone_count": a.clone_count,
                            "is_clone": a.is_clone,
                            "original_ptr": null
                        },
                        "ownership_history_available": false,
                        "type": if a.type_name.contains("Vec") || a.type_name.contains("vec::Vec") {
                            "dynamic_array"
                        } else if a.type_name.contains("String") || a.type_name.contains("str") {
                            "string"
                        } else if a.type_name.contains("Box") || a.type_name.contains("Rc") || a.type_name.contains("Arc") {
                            "smart_pointer"
                        } else {
                            "custom"
                        }
                    })
                }).collect::<Vec<_>>(),
                "metadata": {
                    "export_timestamp": context.export_timestamp,
                    "export_version": "2.0",
                    "specification": "memscope-rs memory analysis",
                    "total_allocations": context.allocations.len(),
                    "total_size_bytes": total_size
                },
                "statistics": {
                    "average_size_bytes": average_size,
                    "total_allocations": context.allocations.len(),
                    "total_size_bytes": total_size
                },
                "type_distribution": type_counts
            },
            "lifetime": {
                "metadata": {
                    "export_timestamp": context.export_timestamp,
                    "export_version": "2.0",
                    "specification": "memscope-rs lifetime tracking",
                    "total_tracked_allocations": context.allocations.len()
                },
                "ownership_histories": lifetime_events
            },
            "complex_types": {
                "smart_pointers": context.allocations.iter().filter(|a| a.is_smart_pointer).count(),
                "collections": context.allocations.iter().filter(|a| {
                    a.type_name.contains("Vec") || a.type_name.contains("HashMap") || a.type_name.contains("BTreeMap")
                }).count()
            },
            "unsafe_ffi": {
                "passports": context.passport_details,
                "reports": context.unsafe_reports,
                "cross_boundary_events": context.unsafe_reports.iter()
                    .flat_map(|r| r.cross_boundary_events.iter())
                    .count()
            },
            "performance": {
                "total_memory": context.total_memory,
                "peak_memory": context.peak_memory,
                "total_allocations": context.total_allocations,
                "active_allocations": context.active_allocations,
                "thread_count": context.thread_count,
                "passport_count": context.passport_count,
                "leak_count": context.leak_count,
                "unsafe_count": context.unsafe_count,
                "ffi_count": context.ffi_count
            },
            "system_resources": {
                "os_name": context.os_name,
                "architecture": context.architecture,
                "cpu_cores": context.cpu_cores,
                "system_info": context.system_resources
            },
            "threads": context.threads
        })
    }
}

/// Format thread_id from "ThreadId(5)" to "Thread-5"
fn format_thread_id(raw: &str) -> String {
    if raw.starts_with("ThreadId(") && raw.ends_with(')') {
        let num = &raw[9..raw.len() - 1];
        format!("Thread-{}", num)
    } else {
        raw.to_string()
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Custom Handlebars helpers
fn format_bytes_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).unwrap().value();
    if let Some(bytes) = param.as_u64() {
        let formatted = format_bytes(bytes as usize);
        out.write(&formatted)?;
    }
    Ok(())
}

fn greater_than_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param1 = h.param(0).unwrap().value();
    let param2 = h.param(1).unwrap().value();

    if let (Some(v1), Some(v2)) = (param1.as_u64(), param2.as_u64()) {
        if v1 > v2 {
            out.write("true")?;
        }
    }
    Ok(())
}

fn contains_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let haystack = h.param(0).unwrap().value();
    let needle = h.param(1).unwrap().value();

    if let (Some(h_str), Some(n_str)) = (haystack.as_str(), needle.as_str()) {
        if h_str.contains(n_str) {
            out.write("true")?;
        }
    }
    Ok(())
}

fn json_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).unwrap().value();
    let json_string = serde_json::to_string(param)
        .map_err(|e| handlebars::RenderError::new(format!("Failed to serialize to JSON: {}", e)))?;
    out.write(&json_string)?;
    Ok(())
}

impl Default for DashboardRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create dashboard renderer")
    }
}
