//! Export functionality for the render engine
//!
//! This module provides export functionality for memory tracking data,
//! including JSON export, lifetime analysis, and variable relationships.

use tracing::{debug, warn};

use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::capture::platform::memory_info::PlatformMemoryInfo;
use crate::core::{MemScopeError, MemScopeResult};
use crate::render_engine::dashboard::{rebuild_allocations_from_events, DashboardRenderer};
use crate::tracker::Tracker;
use serde_json::json;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::Arc,
};

mod ownership;
mod snapshot_json;
mod task_graph;
pub use ownership::{export_ownership_graph_json, export_ownership_graph_snapshot};
pub use snapshot_json::{
    export_snapshot_to_json, export_snapshot_with_containers, ExportError, ExportJsonOptions,
    OptimizationLevel, SchemaValidator,
};
pub use task_graph::{
    export_task_graph_json, export_task_graph_json_with_accounting, export_task_graph_snapshot,
};

/// Nanosecond wall-clock stamp for the immutable event projections.
fn capture_cut_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

pub fn export_all_json<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    // Suppress allocator tracking during the entire export. The export path
    // performs large amounts of allocation (event replay, JSON serialisation,
    // HTML rendering) that are not user allocations. Without suppression,
    // these self-allocations enter the event stream, creating a feedback loop
    // that can inflate memory usage to many GB and run for minutes.
    //
    // `TrackingGuard::acquire` sets a thread-local flag that the custom
    // allocator checks before recording. The guard restores the flag on drop,
    // so tracking resumes immediately after the export completes.
    let _suppression = crate::core::allocator::TrackingGuard::acquire();

    let path_ref = path.as_ref();

    // Use event_store as unified data source (includes both HeapOwner and Container allocations)
    let events = tracker.event_store().snapshot();
    // Auxiliary metadata sources have independent locks. Freeze each once before I/O;
    // event-derived counters always use `events`, not these independently sampled caches.
    let passports = passport_tracker.snapshot_passports();
    let mut task_graph = crate::task_registry::global_registry().export_graph();
    let async_metadata = crate::capture::backends::profile_replay::ProfileMetadata::capture(
        &task_graph,
        Some(async_tracker),
    );
    async_metadata.reconcile_graph(&mut task_graph);
    let captured_at = capture_cut_ns();
    let async_rebuilt = crate::capture::backends::profile_replay::rebuild_at(
        &events,
        &task_graph,
        &async_metadata,
        captured_at,
    );
    let borrow_history =
        crate::analysis::borrow_analysis::get_global_borrow_analyzer().get_borrow_history();

    // Replay once and project that single cut: the ledger keeps the transition-based
    // totals and peaks that the live list can no longer reconstruct, and the projection
    // keeps every identity (including overlapping generations) so the rows and the
    // counters describe the same thing.
    let mut ledger = crate::snapshot::BindingLedger::new();
    for event in &events {
        ledger.apply(event);
    }
    let snapshot = crate::snapshot::builder::project_snapshot(&ledger, captured_at);
    let options = ExportJsonOptions::default();

    std::fs::create_dir_all(path_ref)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Starting export_snapshot_to_json");

    // Container (stack-side) metadata from the same event slice; it is exported next to
    // the heap rows but never counted in the statistics.
    let containers: Vec<crate::capture::types::AllocationInfo> =
        rebuild_allocations_from_events(&events)
            .into_iter()
            .filter(|allocation| allocation.ptr >= crate::analysis::VIRTUAL_PTR_BASE)
            .collect();

    export_snapshot_with_containers(&snapshot, &containers, path_ref, &options)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_snapshot_to_json");

    debug!("Starting export_memory_passports_json");

    write_memory_passports_json(path_ref, passport_tracker, &passports)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_memory_passports_json");

    debug!("Starting export_leak_detection_json");

    write_leak_detection_json(path_ref, &passport_tracker.detect_leaks_in(&passports))
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_leak_detection_json");

    debug!("Starting export_unsafe_ffi_json");

    write_unsafe_ffi_json(path_ref, passport_tracker, &passports)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_unsafe_ffi_json");

    debug!("Starting export_system_resources_json");

    export_system_resources_json(path_ref)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_system_resources_json");

    debug!("Starting export_async_analysis_json");

    write_async_analysis_json(path_ref, async_rebuilt)
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_async_analysis_json");

    debug!("Starting export_ownership_graph_json");

    // Same event slice as the snapshot above, so still one cut. Container rows keep the
    // virtual ids that `rebuild_allocations_from_events` assigns them; the graph builder
    // recognises those ids and never dereferences them.
    let typed_allocations: Vec<crate::capture::types::AllocationInfo> =
        rebuild_allocations_from_events(&events);

    debug!(
        allocations = typed_allocations.len(),
        "Converted allocations to typed format"
    );

    export_ownership_graph_snapshot(
        path_ref,
        &typed_allocations,
        &events,
        &borrow_history,
        captured_at,
    )
    .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_ownership_graph_json");

    // Export task graph JSON
    export_task_graph_snapshot(path_ref, &events, Some(&task_graph))
        .map_err(|e| MemScopeError::error("export", "export_all_json", e.to_string()))?;

    debug!("Completed export_task_graph_json");

    debug!("All exports completed successfully");

    Ok(())
}

/// Export async task analysis to JSON
///
/// This function rebuilds task profiles from the event stream rather than
/// reading live `AsyncTracker` state. This ensures the export always sees
/// a consistent, complete snapshot regardless of concurrent tracking
/// activity (no lock contention, no data loss from `try_lock` failures).
pub fn export_async_analysis_json<P: AsRef<Path>>(
    path: P,
    events: &[crate::event_store::MemoryEvent],
    task_graph: &crate::task_registry::TaskGraph,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    // Suppress allocator tracking during the entire export. The JSON
    // serialisation and event replay perform many internal allocations
    // that are not user allocations. Without suppression, these
    // self-allocations enter the event stream, creating a feedback loop
    // that inflates memory usage and corrupts the exported statistics.
    let _suppression = crate::core::allocator::TrackingGuard::acquire();

    let path_ref = path.as_ref();

    // Rebuild profiles from the event stream, merging live tracker metadata
    // (task names, completion times, creation timestamps) in one pass.
    let rebuilt =
        crate::capture::backends::rebuild_with_live_tracker(events, task_graph, async_tracker);
    write_async_analysis_json(path_ref, rebuilt)
}

fn write_async_analysis_json(
    path_ref: &Path,
    rebuilt: crate::capture::backends::profile_replay::RebuiltProfiles,
) -> MemScopeResult<()> {
    let profiles = rebuilt.profiles;
    let live_allocations = rebuilt.live_allocations;

    let total_tasks = profiles.len() as u64;
    let active_tasks = profiles.iter().filter(|p| !p.is_completed()).count() as u64;

    let async_data = json!({
        "summary": {
            "total_tasks": total_tasks,
            "active_tasks": active_tasks,
            "total_allocations": rebuilt.total_allocations,
            "total_memory_bytes": rebuilt.total_memory,
            "active_memory_bytes": rebuilt.current_memory,
            "peak_memory_bytes": rebuilt.global_peak_memory,
        },
        "task_profiles": profiles.iter().map(|p| json!({
            "task_id": p.task_id,
            "task_name": p.task_name,
            "task_type": format!("{:?}", p.task_type),
            "created_at_ms": p.created_at_ms,
            "completed_at_ms": p.completed_at_ms,
            "total_bytes": p.total_bytes,
            "current_memory": p.current_memory,
            "peak_memory": p.peak_memory,
            "total_allocations": p.total_allocations,
            "total_deallocations": p.total_deallocations,
            "duration_ns": p.duration_ns,
            "allocation_rate": p.allocation_rate,
            "efficiency_score": p.efficiency_score,
            "average_allocation_size": p.average_allocation_size,
            "is_completed": p.is_completed(),
            "has_potential_leak": p.has_potential_leak(),
        })).collect::<Vec<_>>(),
        "allocations": live_allocations.iter().map(|a| json!({
            "ptr": format!("0x{:x}", a.ptr),
            "size": a.size,
            "timestamp": a.timestamp,
            "task_id": a.task_id,
            "var_name": a.var_name,
            "type_name": a.type_name,
        })).collect::<Vec<_>>(),
    });

    let async_path = path_ref.join("async_analysis.json");
    let file = File::create(async_path)
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &async_data)
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;
    writer
        .flush()
        .map_err(|e| MemScopeError::error("export", "export_async_analysis_json", e.to_string()))?;

    Ok(())
}

/// Dashboard template type — the single merged template id, plus a Custom variant
/// for registry-based rendering of external templates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DashboardTemplate {
    /// Unified merged dashboard (default) — bundles all 8 modes in one HTML file
    #[default]
    Unified,
    /// Custom template by ID (for registry-based rendering of external templates)
    Custom(String),
}

impl std::fmt::Display for DashboardTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardTemplate::Unified => write!(f, "dashboard_unified"),
            DashboardTemplate::Custom(id) => write!(f, "{}", id),
        }
    }
}

impl DashboardTemplate {
    /// Get the template ID used by the Handlebars registry
    pub fn template_id(&self) -> &str {
        match self {
            DashboardTemplate::Unified => "dashboard_unified",
            DashboardTemplate::Custom(id) => id.as_str(),
        }
    }

    /// Get a human-readable name for this template
    pub fn name(&self) -> &str {
        match self {
            DashboardTemplate::Unified => "Unified Dashboard",
            DashboardTemplate::Custom(_) => "Custom Template",
        }
    }
}

/// Export HTML dashboard from tracker data
///
/// This function generates a complete HTML dashboard from the tracker data,
/// including memory analysis, variable relationships, unsafe/FFI tracking,
/// and system resources. The dashboard is rendered using Handlebars templates.
pub fn export_dashboard_html<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    export_dashboard_html_with_template(
        path,
        tracker,
        passport_tracker,
        DashboardTemplate::default(),
        None,
    )
}

/// Export HTML dashboard with async tracker support
pub fn export_dashboard_html_with_async<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: &Arc<crate::capture::backends::async_tracker::AsyncTracker>,
) -> MemScopeResult<()> {
    export_dashboard_html_with_template(
        path,
        tracker,
        passport_tracker,
        DashboardTemplate::default(),
        Some(async_tracker),
    )
}

/// Export HTML dashboard with specific template
///
/// This function generates a complete HTML dashboard from the tracker data
/// using the specified template type.
pub fn export_dashboard_html_with_template<P: AsRef<Path>>(
    path: P,
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    template: DashboardTemplate,
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> MemScopeResult<()> {
    // Suppress allocator tracking during HTML rendering for the same reason
    // as export_all_json: the renderer allocates heavily (template parsing,
    // context building, HTML generation) and those allocations must not
    // pollute the event stream.
    let _suppression = crate::core::allocator::TrackingGuard::acquire();

    let path_ref = path.as_ref();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(path_ref).map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to create output directory: {}", e),
        )
    })?;

    // Create dashboard renderer
    let renderer = DashboardRenderer::new().map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to create dashboard renderer: {}", e),
        )
    })?;

    // Render HTML from tracker data using selected template
    let context = renderer
        .build_context_from_tracker_with_async(tracker, passport_tracker, async_tracker)
        .map_err(|e| {
            MemScopeError::error(
                "export",
                "export_dashboard_html_with_template",
                format!("Failed to build context: {}", e),
            )
        })?;

    let html_content = match template {
        // The unified template and any custom external template are rendered via the registry
        DashboardTemplate::Unified | DashboardTemplate::Custom(_) => {
            let template_id = template.template_id();
            renderer
                .render_with_template(template_id, &context)
                .map_err(|e| {
                    MemScopeError::error(
                        "export",
                        "export_dashboard_html_with_template",
                        format!("Failed to render {} template: {}", template.name(), e),
                    )
                })?
        }
    };

    // Write HTML to file
    let output_file = path_ref.join(format!("{}_dashboard.html", template));
    std::fs::write(&output_file, html_content).map_err(|e| {
        MemScopeError::error(
            "export",
            "export_dashboard_html_with_template",
            format!("Failed to write HTML file: {}", e),
        )
    })?;

    tracing::info!("✅ Dashboard HTML exported to: {:?}", output_file);

    Ok(())
}

pub fn export_memory_passports_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    write_memory_passports_json(
        base_path.as_ref(),
        passport_tracker,
        &passport_tracker.snapshot_passports(),
    )
}

fn write_memory_passports_json(
    base_path: &Path,
    passport_tracker: &MemoryPassportTracker,
    passports: &[crate::analysis::memory_passport_tracker::MemoryPassport],
) -> MemScopeResult<()> {
    let passport_data: Vec<_> = passports
        .iter()
        .map(|p| {
            serde_json::json!({
                "passport_id": p.passport_id,
                "allocation_ptr": format!("0x{:x}", p.allocation_ptr),
                "generation": p.generation,
                "superseded": p.superseded,
                "size_bytes": p.size_bytes,
                "created_at": p.created_at,
                "lifecycle_events": p.lifecycle_events.len(),
                "history_truncated": p.history_truncated,
                "status": format!("{:?}", passport_tracker.assess_status(p)),
                "status_source": "assessed_at_export",
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "memory passport tracking",
            "total_passports": passports.len()
        },
        "memory_passports": passport_data,
    });

    let file_path = base_path.join("memory_passports.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_memory_passports_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_memory_passports_json", e.to_string())
    })?;

    Ok(())
}

pub fn export_leak_detection_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    write_leak_detection_json(
        base_path.as_ref(),
        &passport_tracker.detect_leaks_at_shutdown(),
    )
}

fn write_leak_detection_json(
    base_path: &Path,
    leak_result: &crate::analysis::memory_passport_tracker::LeakDetectionResult,
) -> MemScopeResult<()> {
    let leak_details: Vec<_> = leak_result
        .leak_details
        .iter()
        .map(|detail| {
            serde_json::json!({
                "passport_id": detail.passport_id,
                "memory_address": format!("0x{:x}", detail.memory_address),
                "size_bytes": detail.size_bytes,
                "lifecycle_summary": detail.lifecycle_summary,
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "leak detection",
            "leaks_detected": leak_result.total_leaks
        },
        "leak_detection": {
            "total_leaks": leak_result.total_leaks,
            "leak_details": leak_details
        }
    });

    let file_path = base_path.join("leak_detection.json");
    let json_string = serde_json::to_string_pretty(&json_data)
        .map_err(|e| MemScopeError::error("export", "export_leak_detection_json", e.to_string()))?;
    std::fs::write(&file_path, json_string)
        .map_err(|e| MemScopeError::error("export", "export_leak_detection_json", e.to_string()))?;

    Ok(())
}

pub fn export_unsafe_ffi_json<P: AsRef<Path>>(
    base_path: P,
    passport_tracker: &Arc<MemoryPassportTracker>,
) -> MemScopeResult<()> {
    write_unsafe_ffi_json(
        base_path.as_ref(),
        passport_tracker,
        &passport_tracker.snapshot_passports(),
    )
}

fn write_unsafe_ffi_json(
    base_path: &Path,
    passport_tracker: &MemoryPassportTracker,
    passports: &[crate::analysis::memory_passport_tracker::MemoryPassport],
) -> MemScopeResult<()> {
    use crate::analysis::memory_passport_tracker::PassportStatus;

    let ffi_reports: Vec<_> = passports
        .iter()
        .filter(|p| {
            matches!(
                passport_tracker.assess_status(p),
                PassportStatus::HandoverToFfi
                    | PassportStatus::InForeignCustody
                    | PassportStatus::FreedByForeign
            )
        })
        .map(|p| {
            serde_json::json!({
                "passport_id": p.passport_id,
                "allocation_ptr": format!("0x{:x}", p.allocation_ptr),
                "generation": p.generation,
                "superseded": p.superseded,
                "size_bytes": p.size_bytes,
                "history_truncated": p.history_truncated,
                "status": format!("{:?}", passport_tracker.assess_status(p)),
                "created_at": p.created_at,
                "boundary_events": p.lifecycle_events.iter().map(|e| {
                    serde_json::json!({
                        "timestamp": e.timestamp,
                        "event_type": format!("{:?}", e.event_type),
                        "context": e.context,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "unsafe FFI tracking",
            "total_ffi_reports": ffi_reports.len(),
            "total_memory_passports": passports.len()
        },
        "unsafe_reports": ffi_reports,
        "memory_passports": passports.len()
    });

    let file_path = base_path.join("unsafe_ffi.json");
    let json_string = serde_json::to_string_pretty(&json_data)
        .map_err(|e| MemScopeError::error("export", "export_unsafe_ffi_json", e.to_string()))?;
    std::fs::write(&file_path, json_string)
        .map_err(|e| MemScopeError::error("export", "export_unsafe_ffi_json", e.to_string()))?;

    Ok(())
}

/// Export system resource monitoring data to JSON
///
/// This function exports comprehensive system resource statistics including:
/// - Memory statistics (virtual, physical, process, system)
/// - CPU information (cores, cache, system info)
/// - Memory pressure indicators
pub fn export_system_resources_json<P: AsRef<Path>>(base_path: P) -> MemScopeResult<()> {
    let base_path = base_path.as_ref();

    // Collect memory statistics
    let mut memory_info = PlatformMemoryInfo::new();
    let _ = memory_info.initialize();

    let memory_stats = match memory_info.collect_stats() {
        Ok(stats) => stats,
        Err(e) => {
            warn!(error = %e, "Failed to collect memory stats");
            return Err(MemScopeError::error(
                "export",
                "export_system_resources_json",
                e.to_string(),
            ));
        }
    };

    // Collect system information
    let system_info = match memory_info.get_system_info() {
        Ok(info) => info,
        Err(e) => {
            warn!(error = %e, "Failed to collect system info");
            return Err(MemScopeError::error(
                "export",
                "export_system_resources_json",
                e.to_string(),
            ));
        }
    };

    // Build JSON structure
    let json_data = serde_json::json!({
        "metadata": {
            "export_version": "2.0",
            "specification": "system resource monitoring",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        },
        "system_info": {
            "os_name": system_info.os_name,
            "os_version": system_info.os_version,
            "architecture": system_info.architecture,
            "cpu_cores": system_info.cpu_cores,
            "page_size": system_info.page_size,
            "large_page_size": system_info.large_page_size,
            "cpu_cache": {
                "l1_cache_size": system_info.cpu_cache.l1_cache_size,
                "l2_cache_size": system_info.cpu_cache.l2_cache_size,
                "l3_cache_size": system_info.cpu_cache.l3_cache_size,
                "cache_line_size": system_info.cpu_cache.cache_line_size
            },
            "mmu_info": {
                "virtual_address_bits": system_info.mmu_info.virtual_address_bits,
                "physical_address_bits": system_info.mmu_info.physical_address_bits,
                "aslr_enabled": system_info.mmu_info.aslr_enabled,
                "nx_bit_supported": system_info.mmu_info.nx_bit_supported
            }
        },
        "memory_stats": {
            "virtual_memory": {
                "total_virtual": memory_stats.virtual_memory.total_virtual,
                "available_virtual": memory_stats.virtual_memory.available_virtual,
                "used_virtual": memory_stats.virtual_memory.used_virtual,
                "reserved": memory_stats.virtual_memory.reserved,
                "committed": memory_stats.virtual_memory.committed
            },
            "physical_memory": {
                "total_physical": memory_stats.physical_memory.total_physical,
                "available_physical": memory_stats.physical_memory.available_physical,
                "used_physical": memory_stats.physical_memory.used_physical,
                "cached": memory_stats.physical_memory.cached,
                "buffers": memory_stats.physical_memory.buffers,
                "swap": {
                    "total_swap": memory_stats.physical_memory.swap.total_swap,
                    "used_swap": memory_stats.physical_memory.swap.used_swap,
                    "available_swap": memory_stats.physical_memory.swap.available_swap,
                    "swap_in_rate": memory_stats.physical_memory.swap.swap_in_rate,
                    "swap_out_rate": memory_stats.physical_memory.swap.swap_out_rate
                }
            },
            "process_memory": {
                "virtual_size": memory_stats.process_memory.virtual_size,
                "resident_size": memory_stats.process_memory.resident_size,
                "shared_size": memory_stats.process_memory.shared_size,
                "private_size": memory_stats.process_memory.private_size,
                "heap_size": memory_stats.process_memory.heap_size,
                "stack_size": memory_stats.process_memory.stack_size,
                "mapped_files": memory_stats.process_memory.mapped_files,
                "peak_usage": memory_stats.process_memory.peak_usage
            },
            "system_memory": {
                "allocation_count": memory_stats.system_memory.allocation_count,
                "deallocation_count": memory_stats.system_memory.deallocation_count,
                "active_allocations": memory_stats.system_memory.active_allocations,
                "total_allocated": memory_stats.system_memory.total_allocated,
                "total_deallocated": memory_stats.system_memory.total_deallocated,
                "fragmentation_level": memory_stats.system_memory.fragmentation_level,
                "large_pages": {
                    "supported": memory_stats.system_memory.large_pages.supported,
                    "total_large_pages": memory_stats.system_memory.large_pages.total_large_pages,
                    "used_large_pages": memory_stats.system_memory.large_pages.used_large_pages,
                    "page_size": memory_stats.system_memory.large_pages.page_size
                }
            },
            "pressure_indicators": {
                "pressure_level": format!("{:?}", memory_stats.pressure_indicators.pressure_level),
                "low_memory": memory_stats.pressure_indicators.low_memory,
                "swapping_active": memory_stats.pressure_indicators.swapping_active,
                "allocation_failure_rate": memory_stats.pressure_indicators.allocation_failure_rate,
                "gc_pressure": memory_stats.pressure_indicators.gc_pressure
            }
        }
    });

    let file_path = base_path.join("system_resources.json");
    let json_string = serde_json::to_string_pretty(&json_data).map_err(|e| {
        MemScopeError::error("export", "export_system_resources_json", e.to_string())
    })?;
    std::fs::write(&file_path, json_string).map_err(|e| {
        MemScopeError::error("export", "export_system_resources_json", e.to_string())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::snapshot_json::{estimate_json_size, get_or_compute_type_info};
    use super::*;

    /// Objective: Verify that OptimizationLevel enum variants exist.
    /// Invariants: All variants should be accessible.
    #[test]
    fn test_optimization_level_variants() {
        let _low = OptimizationLevel::Low;
        let _medium = OptimizationLevel::Medium;
        let _high = OptimizationLevel::High;
        let _maximum = OptimizationLevel::Maximum;
    }

    /// Objective: Verify that OptimizationLevel default is Medium.
    /// Invariants: Default should return Medium variant.
    #[test]
    fn test_optimization_level_default() {
        let level = OptimizationLevel::default();
        assert!(
            matches!(level, OptimizationLevel::Medium),
            "Default should be Medium"
        );
    }

    /// Objective: Verify that SchemaValidator creates with default settings.
    /// Invariants: New validator should have strict_mode disabled.
    #[test]
    fn test_schema_validator_new() {
        let validator = SchemaValidator::new();
        let data = serde_json::json!({"test": "value"});
        let result = validator.validate(&data);
        assert!(result.is_ok(), "Validation should pass for any object");
    }

    /// Objective: Verify that SchemaValidator with strict mode validates required fields.
    /// Invariants: Strict mode should require timestamp, allocations, stats fields.
    #[test]
    fn test_schema_validator_strict_mode() {
        let validator = SchemaValidator::new().with_strict_mode(true);

        let missing_fields = serde_json::json!({"other": "data"});
        let result = validator.validate(&missing_fields);
        assert!(result.is_err(), "Should fail with missing required fields");

        let valid_data = serde_json::json!({
            "timestamp": 123,
            "allocations": [],
            "stats": {}
        });
        let result = validator.validate(&valid_data);
        assert!(result.is_ok(), "Should pass with all required fields");
    }

    /// Objective: Verify that SchemaValidator rejects non-object data.
    /// Invariants: Non-object JSON should be rejected.
    #[test]
    fn test_schema_validator_non_object() {
        let validator = SchemaValidator::new();
        let data = serde_json::json!("not an object");
        let result = validator.validate(&data);
        assert!(result.is_err(), "Should reject non-object data");
    }

    /// Objective: Verify that ExportJsonOptions has correct defaults.
    /// Invariants: Default options should enable parallel processing and streaming.
    #[test]
    fn test_export_json_options_default() {
        let options = ExportJsonOptions::default();
        assert!(
            options.parallel_processing,
            "parallel_processing should be true by default"
        );
        assert!(
            options.streaming_writer,
            "streaming_writer should be true by default"
        );
        assert!(
            options.enable_type_cache,
            "enable_type_cache should be true by default"
        );
        assert!(
            options.adaptive_optimization,
            "adaptive_optimization should be true by default"
        );
        assert!(
            !options.schema_validation,
            "schema_validation should be false by default"
        );
        assert!(
            !options.security_analysis,
            "security_analysis should be false by default"
        );
    }

    /// Objective: Verify that ExportJsonOptions builder methods work.
    /// Invariants: Builder methods should set corresponding fields.
    #[test]
    fn test_export_json_options_builders() {
        let options = ExportJsonOptions::default()
            .fast_export_mode(true)
            .security_analysis(true)
            .streaming_writer(false)
            .schema_validation(true)
            .integrity_hashes(true)
            .batch_size(500)
            .adaptive_optimization(false)
            .max_cache_size(5000)
            .include_low_severity(true)
            .thread_count(Some(4));

        assert!(options.fast_export_mode, "fast_export_mode should be true");
        assert!(
            options.security_analysis,
            "security_analysis should be true"
        );
        assert!(
            !options.streaming_writer,
            "streaming_writer should be false"
        );
        assert!(
            options.schema_validation,
            "schema_validation should be true"
        );
        assert!(options.integrity_hashes, "integrity_hashes should be true");
        assert_eq!(options.batch_size, 500, "batch_size should be 500");
        assert!(
            !options.adaptive_optimization,
            "adaptive_optimization should be false"
        );
        assert_eq!(
            options.max_cache_size, 5000,
            "max_cache_size should be 5000"
        );
        assert!(
            options.include_low_severity,
            "include_low_severity should be true"
        );
        assert_eq!(
            options.thread_count,
            Some(4),
            "thread_count should be Some(4)"
        );
    }

    /// Objective: Verify that ExportError variants exist and display correctly.
    /// Invariants: Error variants should be displayable.
    #[test]
    fn test_export_error_variants() {
        let io_err = ExportError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let json_err = ExportError::Json(serde_json::from_str::<i32>("invalid").unwrap_err());
        let export_err = ExportError::ExportFailed("test error".to_string());

        assert!(
            format!("{}", io_err).contains("IO error"),
            "Should contain IO error"
        );
        assert!(
            format!("{}", json_err).contains("JSON error"),
            "Should contain JSON error"
        );
        assert!(
            format!("{}", export_err).contains("Export failed"),
            "Should contain Export failed"
        );
    }

    /// Objective: Verify that estimate_json_size works for different JSON types.
    /// Invariants: Size estimation should be reasonable.
    #[test]
    fn test_estimate_json_size() {
        let string_val = serde_json::json!("hello world");
        let num_val = serde_json::json!(42);
        let array_val = serde_json::json!([1, 2, 3]);
        let object_val = serde_json::json!({"key": "value"});

        let string_size = estimate_json_size(&string_val);
        let num_size = estimate_json_size(&num_val);
        let array_size = estimate_json_size(&array_val);
        let object_size = estimate_json_size(&object_val);

        assert!(string_size > 0, "String size should be positive");
        assert!(num_size > 0, "Number size should be positive");
        assert!(array_size > 0, "Array size should be positive");
        assert!(object_size > 0, "Object size should be positive");
    }

    /// Objective: Verify that get_or_compute_type_info categorizes types correctly.
    /// Invariants: Type categorization should match expected patterns.
    #[test]
    fn test_get_or_compute_type_info() {
        assert_eq!(
            get_or_compute_type_info("Vec<i32>", 100),
            "dynamic_array",
            "Vec should be dynamic_array"
        );
        assert_eq!(
            get_or_compute_type_info("String", 24),
            "string",
            "String should be string"
        );
        assert_eq!(
            get_or_compute_type_info("Box<i32>", 8),
            "smart_pointer",
            "Box should be smart_pointer"
        );
        assert_eq!(
            get_or_compute_type_info("Rc<String>", 8),
            "smart_pointer",
            "Rc should be smart_pointer"
        );
        assert_eq!(
            get_or_compute_type_info("Arc<i32>", 16),
            "smart_pointer",
            "Arc should be smart_pointer"
        );
        assert_eq!(
            get_or_compute_type_info("[u8; 100]", 100),
            "byte_array",
            "u8 array should be byte_array"
        );
        assert_eq!(
            get_or_compute_type_info("CustomType", 1024 * 1024 * 2),
            "large_buffer",
            "Large allocation should be large_buffer"
        );
        assert_eq!(
            get_or_compute_type_info("MyType", 100),
            "custom",
            "Unknown type should be custom"
        );
    }
}
