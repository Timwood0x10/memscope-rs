//! Dashboard renderer using Handlebars templates

use crate::tracker::Tracker;
use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::Path;
use handlebars::Handlebars;
use tracing::{info, warn};

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
    /// Thread ID
    pub thread_id: String,
    /// Number of allocations
    pub allocation_count: usize,
    /// Current memory usage
    pub current_memory: String,
    /// Peak memory usage
    pub peak_memory: String,
    /// Total allocated
    pub total_allocated: String,
}

/// Dashboard renderer
pub struct DashboardRenderer {
    handlebars: Handlebars<'static>,
}

impl DashboardRenderer {
    /// Create a new dashboard renderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();

        // Register base template (optional, for future use)
        if let Err(_) = handlebars.register_template_file("base", "src/render_engine/dashboard/templates/base.html") {
            // Base template is optional, ignore error
        }

        // Register standalone dashboard template (no external dependencies, works with file:// protocol)
        handlebars.register_template_file("standalone_dashboard", "src/render_engine/dashboard/templates/standalone_dashboard.html")?;

        // Register custom helpers
        handlebars.register_helper("format_bytes", Box::new(format_bytes_helper));
        handlebars.register_helper("gt", Box::new(greater_than_helper));
        handlebars.register_helper("contains", Box::new(contains_helper));
        handlebars.register_helper("json", Box::new(json_helper));

        Ok(Self { handlebars })
    }
    
    /// Render dashboard from tracker data
    pub fn render_from_tracker(
        &self,
        tracker: &Tracker,
        passport_tracker: &Arc<MemoryPassportTracker>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let allocations = tracker.inner().get_active_allocations().unwrap_or_default();
        let passports = passport_tracker.get_all_passports();
        let analysis = tracker.analyze();
        
        let total_memory: usize = allocations.iter().map(|a| a.size).sum();
        
        // Build allocation info
        let alloc_info: Vec<AllocationInfo> = allocations.iter().map(|a| {
            let type_name = a.type_name.clone().unwrap_or_else(|| "unknown".to_string());
            let timestamp_alloc = a.timestamp_alloc;
            let timestamp_dealloc = a.timestamp_dealloc.unwrap_or(0);
            let lifetime_ms = if timestamp_dealloc > 0 {
                (timestamp_dealloc - timestamp_alloc) as f64 / 1_000_000.0
            } else {
                0.0
            };

            // Determine if smart pointer
            let is_smart_pointer = type_name.contains("Arc") || type_name.contains("Rc") || type_name.contains("Box");
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
                timestamp: format!("{:?}", a.timestamp_alloc),
                thread_id: format!("{}", a.thread_id),
                immutable_borrows: a.borrow_info.as_ref()
                    .map(|b| b.immutable_borrows)
                    .unwrap_or(0),
                mutable_borrows: a.borrow_info.as_ref()
                    .map(|b| b.mutable_borrows)
                    .unwrap_or(0),
                is_clone: a.clone_info.as_ref()
                    .map(|c| c.is_clone)
                    .unwrap_or(false),
                clone_count: a.clone_info.as_ref()
                    .map(|c| c.clone_count)
                    .unwrap_or(0),
                timestamp_alloc,
                timestamp_dealloc,
                lifetime_ms,
                is_leaked: timestamp_dealloc == 0,
                allocation_type: "heap".to_string(), // TODO: Detect stack vs heap
                is_smart_pointer,
                smart_pointer_type,
            }
        }).collect();
        
        // Build variable relationships with real relationship types
        let mut relationships: Vec<RelationshipInfo> = Vec::new();

        for (i, a1) in alloc_info.iter().enumerate() {
            for a2 in alloc_info.iter().skip(i + 1) {
                // Clone relationships (same type and size, both clones)
                if a1.is_clone && a2.is_clone && a1.type_name == a2.type_name && a1.size == a2.size {
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: "clone".to_string(),
                        strength: 0.9,
                        type_name: a1.type_name.clone(),
                        color: "#10b981".to_string(), // Green for clone
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
                        color: "#dc2626".to_string(), // Red for ownership transfer
                    });
                }

                // Borrow relationships (same address, different borrow counts)
                if a1.address == a2.address &&
                   (a1.immutable_borrows != a2.immutable_borrows ||
                    a1.mutable_borrows != a2.mutable_borrows) {
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
                        color: "#3b82f6".to_string(), // Blue for borrow
                    });
                }

                // Smart pointer relationships (Arc/Rc with same original data)
                if a1.is_smart_pointer && a2.is_smart_pointer &&
                   a1.smart_pointer_type == a2.smart_pointer_type {
                    relationships.push(RelationshipInfo {
                        source_ptr: a1.address.clone(),
                        source_var_name: a1.var_name.clone(),
                        target_ptr: a2.address.clone(),
                        target_var_name: a2.var_name.clone(),
                        relationship_type: format!("smart_pointer_{}", a1.smart_pointer_type.to_lowercase()),
                        strength: 0.7,
                        type_name: a1.type_name.clone(),
                        color: "#8b5cf6".to_string(), // Purple for smart pointer
                    });
                }
            }
        }

        // Remove duplicates
        relationships.sort_by(|a, b| {
            (&a.source_ptr, &a.target_ptr).cmp(&(&b.source_ptr, &b.target_ptr))
        });
        relationships.dedup_by(|a, b| {
            a.source_ptr == b.source_ptr && a.target_ptr == b.target_ptr
        });
        
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
                            context: context,
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

        // Prepare JSON data for direct injection (performance optimization)
        #[allow(dead_code)]
        #[derive(serde::Serialize)]
        struct DashboardData<'a> {
            allocations: &'a [AllocationInfo],
            relationships: &'a [RelationshipInfo],
            unsafe_reports: &'a [UnsafeReport],
        }

        let data = DashboardData {
            allocations: &alloc_info,
            relationships: &relationships,
            unsafe_reports: &unsafe_reports,
        };

        let json_data: String = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize dashboard data: {}", e))?;

        // Get system information directly using platform-specific functions
        let (os_name, os_version, architecture, cpu_cores, page_size, total_physical, available_physical, used_physical) = {
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
                    ) == 0 {
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
                    ) == 0 {
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
                    ) == 0 {
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
                    ) != 0 {
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
                    ) != 0 {
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
                    ) != 0 {
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

                ("macOS".to_string(), os_version, architecture, cpu_cores, page_size, total, available_physical, used_physical)
            }

            #[cfg(not(target_os = "macos"))]
            {
                ("Unknown".to_string(), "Unknown".to_string(), "unknown".to_string(), 1, 4096, 16 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024)
            }
        };

        let context = DashboardContext {
            title: "MemScope Dashboard".to_string(),
            export_timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            total_memory: format_bytes(total_memory),
            total_allocations: allocations.len(),
            active_allocations: allocations.len(),
            peak_memory: format_bytes(analysis.peak_memory_bytes as usize),
            thread_count: 1, // TODO: Read from actual thread data
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
            threads: vec![ThreadInfo {
                thread_id: "0".to_string(),
                allocation_count: allocations.len(),
                current_memory: format_bytes(total_memory),
                peak_memory: format_bytes(analysis.peak_memory_bytes as usize),
                total_allocated: format_bytes(total_memory),
            }],
        };
        
        self.render_dashboard(&context)
    }
    
    /// Render dashboard from context
    pub fn render_dashboard(&self, context: &DashboardContext) -> Result<String, Box<dyn std::error::Error>> {
        // Use standalone dashboard to avoid CORS issues with file:// protocol
        self.handlebars.render("standalone_dashboard", context)
            .map_err(|e| format!("Template rendering error: {}", e).into())
    }

    /// Render standalone dashboard (no external dependencies, works with file:// protocol)
    pub fn render_standalone_dashboard(&self, context: &DashboardContext) -> Result<String, Box<dyn std::error::Error>> {
        self.handlebars.render("standalone_dashboard", context)
            .map_err(|e| format!("Template rendering error: {}", e).into())
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
    let json_string = serde_json::to_string(param).map_err(|e| {
        handlebars::RenderError::new(format!("Failed to serialize to JSON: {}", e))
    })?;
    out.write(&json_string)?;
    Ok(())
}

impl Default for DashboardRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create dashboard renderer")
    }
}