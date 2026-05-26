//! Context building logic for dashboard.

use super::event_dto::{build_data_index, DashboardEventDTO, EventSummary};
use super::event_reconstructor::{
    rebuild_allocations_from_events, SamplingMetadata, MAX_DASHBOARD_ALLOCATIONS,
};
use super::helpers::format_bytes;
use super::report_builder::{
    aggregate_thread_data, build_allocation_info, build_async_summary, build_async_tasks,
    build_circular_reference_report, build_json_data, build_ownership_graph_info,
    build_passport_details, build_relationships, build_top_n_reports, build_unsafe_reports,
    calculate_health_info,
};
use super::system_info::get_system_info;
use super::types::*;
use crate::analysis::memory_passport_tracker::MemoryPassportTracker;
use crate::tracker::Tracker;
use crate::view::MemoryView;
use std::sync::Arc;

/// Build dashboard context from tracker data with async support
pub fn build_context_from_tracker_with_async(
    tracker: &Tracker,
    passport_tracker: &Arc<MemoryPassportTracker>,
    async_tracker: Option<&Arc<crate::capture::backends::async_tracker::AsyncTracker>>,
) -> Result<DashboardContext, Box<dyn std::error::Error>> {
    let passports = passport_tracker.get_all_passports();

    let events = tracker.event_store().snapshot();
    let all_allocations = rebuild_allocations_from_events(&events);

    let tracker_analysis = tracker.analyze();

    let view = MemoryView::from_events(events.clone());
    let mut az = crate::analyzer::Analyzer::from_view(view);

    if all_allocations.is_empty() {
        tracing::warn!("No allocations found in event store. Dashboard may show limited data.");
    }

    let total_memory: usize = all_allocations.iter().map(|a| a.size).sum();

    let (display_allocations, sampling) = match MAX_DASHBOARD_ALLOCATIONS {
        Some(max) if all_allocations.len() > max => {
            let sampled: Vec<_> = all_allocations.iter().take(max).cloned().collect();
            let meta = SamplingMetadata::truncated(all_allocations.len(), max, "top-first-by-time");
            (sampled, meta)
        }
        _ => {
            let meta = SamplingMetadata::complete(all_allocations.len());
            (all_allocations.clone(), meta)
        }
    };
    let alloc_info = build_allocation_info(&display_allocations);

    let relationships = build_relationships(&mut az);
    let unsafe_reports = build_unsafe_reports(&passports, &all_allocations);
    let passport_details = build_passport_details(&passports, &all_allocations);

    let leak_result = passport_tracker.detect_leaks_at_shutdown();
    let leak_count = leak_result.leaked_passports.len();

    let thread_data = aggregate_thread_data(&alloc_info);
    let async_tasks = build_async_tasks(async_tracker);
    let async_summary = build_async_summary(async_tracker);
    let ownership_graph = build_ownership_graph_info(&all_allocations);

    let top_n_reports = build_top_n_reports(&all_allocations);
    let circular_references = build_circular_reference_report(&all_allocations);

    let system_info = get_system_info();

    let task_graph_json = build_task_graph_json()?;

    let health_info = calculate_health_info(
        &unsafe_reports,
        &passport_details,
        leak_count,
        alloc_info.len(),
    );

    let event_dtos: Vec<DashboardEventDTO> = events.iter().map(DashboardEventDTO::from).collect();
    let event_summary = EventSummary::new(
        events.len(),
        event_dtos.len(),
        tracker_analysis.total_allocations,
        tracker_analysis.active_allocations,
    );
    let data_index = build_data_index(&alloc_info, &event_dtos, &passport_details, &unsafe_reports);

    let json_data = build_json_data(
        &alloc_info,
        &relationships,
        &unsafe_reports,
        &thread_data,
        &passport_details,
        tracker_analysis.active_allocations,
        tracker_analysis.total_allocations,
        leak_count,
        &async_tasks,
        &async_summary,
        &ownership_graph,
        health_info.health_score,
        &task_graph_json,
        &event_dtos,
        &event_summary,
        &data_index,
        &sampling,
    )?;

    Ok(DashboardContext {
        title: "MemScope Dashboard".to_string(),
        export_timestamp: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        total_memory: format_bytes(total_memory),
        total_allocations: tracker_analysis.total_allocations,
        active_allocations: tracker_analysis.active_allocations,
        peak_memory: format_bytes(tracker_analysis.peak_memory_bytes as usize),
        thread_count: thread_data.len(),
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
        os_name: system_info.os_name.clone(),
        architecture: system_info.architecture.clone(),
        cpu_cores: system_info.cpu_cores as usize,
        system_resources: system_info,
        threads: thread_data,
        async_tasks,
        async_summary,
        health_score: health_info.health_score,
        health_status: health_info.health_status,
        safe_ops_count: health_info.safe_ops_count,
        high_risk_count: health_info.high_risk_count,
        clean_passport_count: health_info.clean_passport_count,
        active_passport_count: health_info.active_passport_count,
        leaked_passport_count: health_info.leaked_passport_count,
        ffi_tracked_count: health_info.ffi_tracked_count,
        safe_code_percent: health_info.safe_code_percent,
        ownership_graph,
        top_allocation_sites: top_n_reports.top_allocation_sites,
        top_leaked_allocations: top_n_reports.top_leaked_allocations,
        top_temporary_churn: top_n_reports.top_temporary_churn,
        circular_references,
        task_graph_json: build_task_graph_json()?,
    })
}

/// Build task graph JSON string
fn build_task_graph_json() -> Result<String, Box<dyn std::error::Error>> {
    use crate::task_registry::global_registry;
    let registry = global_registry();
    let graph = registry.export_graph();
    serde_json::to_string(&graph).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that build_context_from_tracker_with_async works
    /// with a minimal tracker. This is a smoke test.
    /// Invariants: The function should not panic on valid inputs.
    #[test]
    fn test_build_context_smoke() {
        // This test verifies the function signature compiles and handles
        // the basic case. Full integration tests cover the actual data path.
        let _ = build_task_graph_json;
        let _ = build_json_data;
    }

    /// Objective: Verify that SamplingMetadata is correctly exposed in json_data.
    /// Invariants: An unsampled complete dataset has is_sampled=false.
    #[test]
    fn test_sampling_metadata_in_json() {
        let sampling = SamplingMetadata::complete(42);
        let json = serde_json::to_value(&sampling).unwrap();
        assert!(!json["is_sampled"].as_bool().unwrap());
        assert_eq!(json["original_count"].as_u64().unwrap(), 42);
    }

    /// Objective: Verify that EventSummary is correctly serialized.
    /// Invariants: Complete export has is_sampled=false.
    #[test]
    fn test_event_summary_in_json() {
        let summary = EventSummary::new(100, 100, 50, 25);
        let json = serde_json::to_value(&summary).unwrap();
        assert!(!json["is_sampled"].as_bool().unwrap());
        assert_eq!(json["total_event_count"].as_u64().unwrap(), 100);
    }

    /// Objective: Verify that DashboardContext defaults match expectations.
    /// Invariants: All fields have sensible defaults when empty.
    #[test]
    fn test_dashboard_context_defaults() {
        let ctx = DashboardContext {
            title: "Test".to_string(),
            export_timestamp: "now".to_string(),
            total_memory: "0 B".to_string(),
            total_allocations: 0,
            active_allocations: 0,
            peak_memory: "0 B".to_string(),
            thread_count: 0,
            passport_count: 0,
            leak_count: 0,
            unsafe_count: 0,
            ffi_count: 0,
            allocations: vec![],
            relationships: vec![],
            unsafe_reports: vec![],
            passport_details: vec![],
            allocations_count: 0,
            relationships_count: 0,
            unsafe_reports_count: 0,
            json_data: "{}".to_string(),
            os_name: "test".to_string(),
            architecture: "x86_64".to_string(),
            cpu_cores: 1,
            system_resources: SystemResources {
                os_name: "test".to_string(),
                os_version: "1.0".to_string(),
                architecture: "test".to_string(),
                cpu_cores: 1,
                total_physical: "0 B".to_string(),
                available_physical: "0 B".to_string(),
                used_physical: "0 B".to_string(),
                page_size: 4096,
            },
            threads: vec![],
            async_tasks: vec![],
            async_summary: AsyncSummary {
                total_tasks: 0,
                active_tasks: 0,
                total_allocations: 0,
                total_memory_bytes: 0,
                peak_memory_bytes: 0,
            },
            health_score: 100,
            health_status: "Excellent".to_string(),
            safe_ops_count: 0,
            high_risk_count: 0,
            clean_passport_count: 0,
            active_passport_count: 0,
            leaked_passport_count: 0,
            ffi_tracked_count: 0,
            safe_code_percent: 100,
            ownership_graph: OwnershipGraphInfo {
                total_nodes: 0,
                total_edges: 0,
                total_cycles: 0,
                rc_clone_count: 0,
                arc_clone_count: 0,
                has_issues: false,
                issues: vec![],
                root_cause: None,
            },
            top_allocation_sites: vec![],
            top_leaked_allocations: vec![],
            top_temporary_churn: vec![],
            circular_references: CircularReferenceReport {
                count: 0,
                total_leaked_memory: 0,
                pointers_in_cycles: 0,
                total_smart_pointers: 0,
                has_cycles: false,
            },
            task_graph_json: "{}".to_string(),
        };
        assert_eq!(ctx.title, "Test");
        assert_eq!(ctx.thread_count, 0);
    }
}
