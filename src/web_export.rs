//! Web dashboard data export functionality
//! 
//! This module exports memory analysis data in JSON format
//! for consumption by the web dashboard

use crate::unsafe_ffi_tracker::{
    AllocationSource, BoundaryEventType, EnhancedAllocationInfo, 
    SafetyViolation, UnsafeFFITracker
};
use crate::types::{TrackingError, TrackingResult, MemoryStats};
use crate::tracker::MemoryTracker;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::File;
use std::io::Write;

/// Web dashboard data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct WebDashboardData {
    /// Timestamp when data was exported
    pub timestamp: u128,
    /// Basic memory statistics
    pub memory_stats: MemoryStats,
    /// Real-time metrics for dashboard
    pub metrics: DashboardMetrics,
    /// Enhanced allocations with source tracking
    pub allocations: Vec<WebAllocationInfo>,
    /// Safety violations
    pub violations: Vec<WebSafetyViolation>,
    /// Cross-boundary events timeline
    pub boundary_events: Vec<WebBoundaryEvent>,
    /// Call stack DNA data
    pub call_stack_dna: Vec<DnaSegment>,
    /// Memory radar data
    pub radar_data: RadarData,
    /// Memory passport data
    pub passport_data: PassportData,
}

/// Dashboard metrics for real-time display
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub unsafe_allocations: usize,
    pub ffi_allocations: usize,
    pub boundary_crossings: usize,
    pub safety_violations: usize,
    pub total_unsafe_memory: usize,
    pub active_allocations: usize,
    pub peak_memory: usize,
    pub memory_efficiency: f32,
}

/// Web-friendly allocation info
#[derive(Debug, Serialize, Deserialize)]
pub struct WebAllocationInfo {
    pub id: String,
    pub ptr: usize,
    pub size: usize,
    pub timestamp_alloc: u128,
    pub timestamp_dealloc: Option<u128>,
    pub is_active: bool,
    pub source_type: String,
    pub source_details: WebSourceDetails,
    pub boundary_events: Vec<WebBoundaryEvent>,
    pub safety_issues: Vec<String>,
    pub call_stack: Vec<WebStackFrame>,
}

/// Web-friendly source details
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSourceDetails {
    pub library_name: Option<String>,
    pub function_name: Option<String>,
    pub unsafe_location: Option<String>,
    pub risk_level: String,
}

/// Web-friendly stack frame
#[derive(Debug, Serialize, Deserialize)]
pub struct WebStackFrame {
    pub function_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
    pub is_unsafe: bool,
}

/// Web-friendly boundary event
#[derive(Debug, Serialize, Deserialize)]
pub struct WebBoundaryEvent {
    pub id: String,
    pub event_type: String,
    pub timestamp: u128,
    pub from_context: String,
    pub to_context: String,
    pub memory_size: usize,
    pub risk_assessment: String,
}

/// Web-friendly safety violation
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSafetyViolation {
    pub id: String,
    pub violation_type: String,
    pub severity: String,
    pub timestamp: u128,
    pub description: String,
    pub affected_memory: Vec<usize>,
    pub call_stack: Vec<WebStackFrame>,
    pub mitigation_suggestions: Vec<String>,
}

/// DNA segment for call stack visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct DnaSegment {
    pub id: String,
    pub segment_type: String, // "safe", "unsafe", "ffi"
    pub function_name: String,
    pub file_location: Option<String>,
    pub memory_operations: Vec<String>,
    pub risk_score: f32,
    pub position: f32, // 0.0 to 1.0 along the helix
}

/// Radar data for memory visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct RadarData {
    pub scan_radius: f32,
    pub memory_objects: Vec<RadarObject>,
    pub threat_level: String,
    pub scan_timestamp: u128,
}

/// Radar object representation
#[derive(Debug, Serialize, Deserialize)]
pub struct RadarObject {
    pub id: String,
    pub object_type: String, // "unsafe", "ffi", "safe"
    pub size: usize,
    pub position: RadarPosition,
    pub threat_level: f32, // 0.0 to 1.0
    pub is_active: bool,
    pub last_activity: u128,
}

/// Radar position (polar coordinates)
#[derive(Debug, Serialize, Deserialize)]
pub struct RadarPosition {
    pub angle: f32, // 0.0 to 360.0 degrees
    pub distance: f32, // 0.0 to 1.0 (normalized)
}

/// Memory passport data
#[derive(Debug, Serialize, Deserialize)]
pub struct PassportData {
    pub rust_territory: TerritoryData,
    pub ffi_territory: TerritoryData,
    pub border_crossings: Vec<BorderCrossing>,
    pub security_alerts: Vec<SecurityAlert>,
}

/// Territory data for passport system
#[derive(Debug, Serialize, Deserialize)]
pub struct TerritoryData {
    pub total_memory: usize,
    pub active_objects: usize,
    pub security_level: String,
    pub recent_activity: Vec<TerritoryActivity>,
}

/// Territory activity
#[derive(Debug, Serialize, Deserialize)]
pub struct TerritoryActivity {
    pub timestamp: u128,
    pub activity_type: String,
    pub memory_size: usize,
    pub description: String,
}

/// Border crossing event
#[derive(Debug, Serialize, Deserialize)]
pub struct BorderCrossing {
    pub id: String,
    pub timestamp: u128,
    pub direction: String, // "rust_to_ffi", "ffi_to_rust"
    pub memory_size: usize,
    pub passport_status: String, // "valid", "suspicious", "denied"
    pub inspection_notes: Vec<String>,
}

/// Security alert
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub timestamp: u128,
    pub description: String,
    pub affected_memory: Vec<usize>,
    pub recommended_actions: Vec<String>,
}

/// Export comprehensive web dashboard data
pub fn export_web_dashboard_data<P: AsRef<Path>>(
    tracker: &MemoryTracker,
    unsafe_ffi_tracker: &UnsafeFFITracker,
    path: P,
) -> TrackingResult<()> {
    let path = path.as_ref();
    tracing::info!("Exporting web dashboard data to: {}", path.display());

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Gather all data
    let memory_stats = tracker.get_stats()?;
    let enhanced_allocations = unsafe_ffi_tracker.get_enhanced_allocations()?;
    let safety_violations = unsafe_ffi_tracker.get_safety_violations()?;

    // Build dashboard data
    let dashboard_data = WebDashboardData {
        timestamp,
        memory_stats: memory_stats.clone(),
        metrics: build_dashboard_metrics(&memory_stats, &enhanced_allocations, &safety_violations),
        allocations: convert_allocations(&enhanced_allocations),
        violations: convert_violations(&safety_violations),
        boundary_events: extract_boundary_events(&enhanced_allocations),
        call_stack_dna: build_dna_segments(&enhanced_allocations),
        radar_data: build_radar_data(&enhanced_allocations),
        passport_data: build_passport_data(&enhanced_allocations, &safety_violations),
    };

    // Write to file
    let json_data = serde_json::to_string_pretty(&dashboard_data)
        .map_err(|e| TrackingError::SerializationError(format!("JSON serialization failed: {e}")))?;

    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())?;

    tracing::info!("Successfully exported web dashboard data");
    Ok(())
}

/// Build dashboard metrics
fn build_dashboard_metrics(
    memory_stats: &MemoryStats,
    enhanced_allocations: &[EnhancedAllocationInfo],
    safety_violations: &[SafetyViolation],
) -> DashboardMetrics {
    let unsafe_allocations = enhanced_allocations.iter()
        .filter(|a| matches!(a.source, AllocationSource::UnsafeRust { .. }))
        .count();
    
    let ffi_allocations = enhanced_allocations.iter()
        .filter(|a| matches!(a.source, AllocationSource::FfiC { .. }))
        .count();
    
    let boundary_crossings: usize = enhanced_allocations.iter()
        .map(|a| a.cross_boundary_events.len())
        .sum();
    
    let total_unsafe_memory: usize = enhanced_allocations.iter()
        .filter(|a| !matches!(a.source, AllocationSource::RustSafe))
        .map(|a| a.base.size)
        .sum();
    
    let memory_efficiency = if memory_stats.peak_memory > 0 {
        let current_memory = memory_stats.total_allocated - memory_stats.total_deallocated;
        (current_memory as f32 / memory_stats.peak_memory as f32) * 100.0
    } else {
        100.0
    };

    DashboardMetrics {
        unsafe_allocations,
        ffi_allocations,
        boundary_crossings,
        safety_violations: safety_violations.len(),
        total_unsafe_memory,
        active_allocations: memory_stats.active_allocations,
        peak_memory: memory_stats.peak_memory,
        memory_efficiency,
    }
}

/// Convert enhanced allocations to web format
fn convert_allocations(enhanced_allocations: &[EnhancedAllocationInfo]) -> Vec<WebAllocationInfo> {
    enhanced_allocations.iter().enumerate().map(|(i, alloc)| {
        let (source_type, source_details) = match &alloc.source {
            AllocationSource::RustSafe => (
                "safe".to_string(),
                WebSourceDetails {
                    library_name: None,
                    function_name: None,
                    unsafe_location: None,
                    risk_level: "low".to_string(),
                }
            ),
            AllocationSource::UnsafeRust { unsafe_block_location, .. } => (
                "unsafe".to_string(),
                WebSourceDetails {
                    library_name: None,
                    function_name: None,
                    unsafe_location: Some(unsafe_block_location.clone()),
                    risk_level: "high".to_string(),
                }
            ),
            AllocationSource::FfiC { library_name, function_name, .. } => (
                "ffi".to_string(),
                WebSourceDetails {
                    library_name: Some(library_name.clone()),
                    function_name: Some(function_name.clone()),
                    unsafe_location: None,
                    risk_level: "medium".to_string(),
                }
            ),
            AllocationSource::CrossBoundary { .. } => (
                "cross_boundary".to_string(),
                WebSourceDetails {
                    library_name: None,
                    function_name: None,
                    unsafe_location: None,
                    risk_level: "critical".to_string(),
                }
            ),
        };

        WebAllocationInfo {
            id: format!("alloc_{}", i),
            ptr: alloc.base.ptr,
            size: alloc.base.size,
            timestamp_alloc: alloc.base.timestamp_alloc,
            timestamp_dealloc: alloc.base.timestamp_dealloc,
            is_active: alloc.base.is_active(),
            source_type,
            source_details,
            boundary_events: alloc.cross_boundary_events.iter().enumerate().map(|(j, event)| {
                WebBoundaryEvent {
                    id: format!("boundary_{}_{}", i, j),
                    event_type: match event.event_type {
                        BoundaryEventType::RustToFfi => "rust_to_ffi".to_string(),
                        BoundaryEventType::FfiToRust => "ffi_to_rust".to_string(),
                        BoundaryEventType::OwnershipTransfer => "ownership_transfer".to_string(),
                        BoundaryEventType::SharedAccess => "shared_access".to_string(),
                    },
                    timestamp: event.timestamp,
                    from_context: event.from_context.clone(),
                    to_context: event.to_context.clone(),
                    memory_size: alloc.base.size,
                    risk_assessment: assess_boundary_risk(&event.event_type),
                }
            }).collect(),
            safety_issues: alloc.safety_violations.iter().map(|v| format!("{:?}", v)).collect(),
            call_stack: alloc.call_stack.iter().map(|frame| WebStackFrame {
                function_name: frame.function_name.clone(),
                file_name: frame.file_name.clone(),
                line_number: frame.line_number,
                is_unsafe: frame.is_unsafe,
            }).collect(),
        }
    }).collect()
}

/// Convert safety violations to web format
fn convert_violations(safety_violations: &[SafetyViolation]) -> Vec<WebSafetyViolation> {
    safety_violations.iter().enumerate().map(|(i, violation)| {
        let (violation_type, severity, description, mitigation) = match violation {
            SafetyViolation::DoubleFree { .. } => (
                "double_free".to_string(),
                "critical".to_string(),
                "Attempt to free memory that has already been freed".to_string(),
                vec![
                    "Review memory management logic".to_string(),
                    "Use RAII patterns".to_string(),
                    "Consider smart pointers".to_string(),
                ]
            ),
            SafetyViolation::InvalidFree { attempted_pointer, .. } => (
                "invalid_free".to_string(),
                "high".to_string(),
                format!("Attempt to free invalid pointer: 0x{:x}", attempted_pointer),
                vec![
                    "Validate pointers before freeing".to_string(),
                    "Track allocation sources".to_string(),
                    "Use memory debugging tools".to_string(),
                ]
            ),
            SafetyViolation::PotentialLeak { .. } => (
                "memory_leak".to_string(),
                "medium".to_string(),
                "Memory allocated but not freed within expected timeframe".to_string(),
                vec![
                    "Review cleanup logic".to_string(),
                    "Use automatic memory management".to_string(),
                    "Implement proper destructors".to_string(),
                ]
            ),
            SafetyViolation::CrossBoundaryRisk { description, .. } => (
                "cross_boundary_risk".to_string(),
                "high".to_string(),
                description.clone(),
                vec![
                    "Review FFI interface design".to_string(),
                    "Validate cross-boundary transfers".to_string(),
                    "Use safe FFI patterns".to_string(),
                ]
            ),
        };

        WebSafetyViolation {
            id: format!("violation_{}", i),
            violation_type,
            severity,
            timestamp: get_violation_timestamp(violation),
            description,
            affected_memory: get_affected_memory(violation),
            call_stack: get_violation_call_stack(violation),
            mitigation_suggestions: mitigation,
        }
    }).collect()
}

/// Extract boundary events from allocations
fn extract_boundary_events(enhanced_allocations: &[EnhancedAllocationInfo]) -> Vec<WebBoundaryEvent> {
    let mut events = Vec::new();
    let mut event_id = 0;

    for alloc in enhanced_allocations {
        for event in &alloc.cross_boundary_events {
            events.push(WebBoundaryEvent {
                id: format!("global_boundary_{}", event_id),
                event_type: match event.event_type {
                    BoundaryEventType::RustToFfi => "rust_to_ffi".to_string(),
                    BoundaryEventType::FfiToRust => "ffi_to_rust".to_string(),
                    BoundaryEventType::OwnershipTransfer => "ownership_transfer".to_string(),
                    BoundaryEventType::SharedAccess => "shared_access".to_string(),
                },
                timestamp: event.timestamp,
                from_context: event.from_context.clone(),
                to_context: event.to_context.clone(),
                memory_size: alloc.base.size,
                risk_assessment: assess_boundary_risk(&event.event_type),
            });
            event_id += 1;
        }
    }

    // Sort by timestamp
    events.sort_by_key(|e| e.timestamp);
    events
}

/// Build DNA segments for call stack visualization
fn build_dna_segments(enhanced_allocations: &[EnhancedAllocationInfo]) -> Vec<DnaSegment> {
    let mut segments = Vec::new();
    let mut segment_id = 0;

    for alloc in enhanced_allocations {
        for (i, frame) in alloc.call_stack.iter().enumerate() {
            let segment_type = if frame.is_unsafe {
                "unsafe"
            } else {
                match &alloc.source {
                    AllocationSource::FfiC { .. } => "ffi",
                    _ => "safe",
                }
            };

            let risk_score = calculate_risk_score(&alloc.source, frame.is_unsafe);
            let position = i as f32 / alloc.call_stack.len().max(1) as f32;

            segments.push(DnaSegment {
                id: format!("dna_segment_{}", segment_id),
                segment_type: segment_type.to_string(),
                function_name: frame.function_name.clone(),
                file_location: frame.file_name.clone(),
                memory_operations: vec![format!("Allocation: {} bytes", alloc.base.size)],
                risk_score,
                position,
            });
            segment_id += 1;
        }
    }

    segments
}

/// Build radar data
fn build_radar_data(enhanced_allocations: &[EnhancedAllocationInfo]) -> RadarData {
    let memory_objects: Vec<RadarObject> = enhanced_allocations.iter().enumerate().map(|(i, alloc)| {
        let object_type = match &alloc.source {
            AllocationSource::RustSafe => "safe",
            AllocationSource::UnsafeRust { .. } => "unsafe",
            AllocationSource::FfiC { .. } => "ffi",
            AllocationSource::CrossBoundary { .. } => "cross_boundary",
        };

        let threat_level = calculate_threat_level(&alloc.source, alloc.base.size);
        let angle = (i as f32 * 137.5) % 360.0; // Golden angle distribution
        let distance = (alloc.base.size as f32).log10() / 10.0; // Log scale for size

        RadarObject {
            id: format!("radar_obj_{}", i),
            object_type: object_type.to_string(),
            size: alloc.base.size,
            position: RadarPosition {
                angle,
                distance: distance.min(1.0).max(0.1),
            },
            threat_level,
            is_active: alloc.base.is_active(),
            last_activity: alloc.base.timestamp_alloc,
        }
    }).collect();

    let threat_level = if memory_objects.iter().any(|obj| obj.threat_level > 0.8) {
        "critical"
    } else if memory_objects.iter().any(|obj| obj.threat_level > 0.5) {
        "high"
    } else if memory_objects.iter().any(|obj| obj.threat_level > 0.2) {
        "medium"
    } else {
        "low"
    };

    RadarData {
        scan_radius: 1.0,
        memory_objects,
        threat_level: threat_level.to_string(),
        scan_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
}

/// Build passport data
fn build_passport_data(
    enhanced_allocations: &[EnhancedAllocationInfo],
    safety_violations: &[SafetyViolation],
) -> PassportData {
    let mut rust_memory = 0;
    let mut ffi_memory = 0;
    let mut rust_objects = 0;
    let mut ffi_objects = 0;
    let mut border_crossings = Vec::new();

    for (i, alloc) in enhanced_allocations.iter().enumerate() {
        match &alloc.source {
            AllocationSource::RustSafe | AllocationSource::UnsafeRust { .. } => {
                rust_memory += alloc.base.size;
                rust_objects += 1;
            },
            AllocationSource::FfiC { .. } | AllocationSource::CrossBoundary { .. } => {
                ffi_memory += alloc.base.size;
                ffi_objects += 1;
            },
        }

        // Add border crossings
        for (j, event) in alloc.cross_boundary_events.iter().enumerate() {
            let direction = match event.event_type {
                BoundaryEventType::RustToFfi | BoundaryEventType::OwnershipTransfer => "rust_to_ffi",
                BoundaryEventType::FfiToRust => "ffi_to_rust",
                BoundaryEventType::SharedAccess => "bidirectional",
            };

            border_crossings.push(BorderCrossing {
                id: format!("crossing_{}_{}", i, j),
                timestamp: event.timestamp,
                direction: direction.to_string(),
                memory_size: alloc.base.size,
                passport_status: "valid".to_string(), // Could be enhanced with actual validation
                inspection_notes: vec![
                    format!("From: {}", event.from_context),
                    format!("To: {}", event.to_context),
                ],
            });
        }
    }

    let security_alerts: Vec<SecurityAlert> = safety_violations.iter().enumerate().map(|(i, violation)| {
        SecurityAlert {
            id: format!("alert_{}", i),
            alert_type: format!("{:?}", violation).split('(').next().unwrap_or("Unknown").to_string(),
            severity: match violation {
                SafetyViolation::DoubleFree { .. } => "critical",
                SafetyViolation::InvalidFree { .. } => "high",
                SafetyViolation::CrossBoundaryRisk { .. } => "high",
                SafetyViolation::PotentialLeak { .. } => "medium",
            }.to_string(),
            timestamp: get_violation_timestamp(violation),
            description: format!("Safety violation detected: {:?}", violation),
            affected_memory: get_affected_memory(violation),
            recommended_actions: vec![
                "Review memory management practices".to_string(),
                "Run additional safety checks".to_string(),
            ],
        }
    }).collect();

    PassportData {
        rust_territory: TerritoryData {
            total_memory: rust_memory,
            active_objects: rust_objects,
            security_level: if safety_violations.is_empty() { "secure" } else { "alert" }.to_string(),
            recent_activity: vec![], // Could be populated with recent allocations
        },
        ffi_territory: TerritoryData {
            total_memory: ffi_memory,
            active_objects: ffi_objects,
            security_level: if safety_violations.is_empty() { "secure" } else { "alert" }.to_string(),
            recent_activity: vec![], // Could be populated with recent allocations
        },
        border_crossings,
        security_alerts,
    }
}

// Helper functions
fn assess_boundary_risk(event_type: &BoundaryEventType) -> String {
    match event_type {
        BoundaryEventType::RustToFfi => "medium".to_string(),
        BoundaryEventType::FfiToRust => "high".to_string(),
        BoundaryEventType::OwnershipTransfer => "high".to_string(),
        BoundaryEventType::SharedAccess => "critical".to_string(),
    }
}

fn calculate_risk_score(source: &AllocationSource, is_unsafe: bool) -> f32 {
    let base_score = match source {
        AllocationSource::RustSafe => 0.1,
        AllocationSource::UnsafeRust { .. } => 0.7,
        AllocationSource::FfiC { .. } => 0.5,
        AllocationSource::CrossBoundary { .. } => 0.9,
    };
    
    if is_unsafe { base_score + 0.2 } else { base_score }
}

fn calculate_threat_level(source: &AllocationSource, size: usize) -> f32 {
    let base_threat = match source {
        AllocationSource::RustSafe => 0.1,
        AllocationSource::UnsafeRust { .. } => 0.6,
        AllocationSource::FfiC { .. } => 0.4,
        AllocationSource::CrossBoundary { .. } => 0.8,
    };
    
    // Larger allocations are potentially more threatening
    let size_factor = (size as f32).log10() / 10.0;
    (base_threat + size_factor * 0.3).min(1.0)
}

fn get_violation_timestamp(violation: &SafetyViolation) -> u128 {
    match violation {
        SafetyViolation::DoubleFree { timestamp, .. } => *timestamp,
        SafetyViolation::InvalidFree { timestamp, .. } => *timestamp,
        SafetyViolation::PotentialLeak { leak_detection_timestamp, .. } => *leak_detection_timestamp,
        SafetyViolation::CrossBoundaryRisk { .. } => {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        },
    }
}

fn get_affected_memory(violation: &SafetyViolation) -> Vec<usize> {
    match violation {
        SafetyViolation::DoubleFree { .. } => vec![], // Could extract from stack frames
        SafetyViolation::InvalidFree { attempted_pointer, .. } => vec![*attempted_pointer],
        SafetyViolation::PotentialLeak { .. } => vec![], // Could extract from allocation info
        SafetyViolation::CrossBoundaryRisk { .. } => vec![],
    }
}

fn get_violation_call_stack(violation: &SafetyViolation) -> Vec<WebStackFrame> {
    match violation {
        SafetyViolation::DoubleFree { second_free_stack, .. } => {
            second_free_stack.iter().map(|frame| WebStackFrame {
                function_name: frame.function_name.clone(),
                file_name: frame.file_name.clone(),
                line_number: frame.line_number,
                is_unsafe: frame.is_unsafe,
            }).collect()
        },
        SafetyViolation::InvalidFree { stack, .. } => {
            stack.iter().map(|frame| WebStackFrame {
                function_name: frame.function_name.clone(),
                file_name: frame.file_name.clone(),
                line_number: frame.line_number,
                is_unsafe: frame.is_unsafe,
            }).collect()
        },
        SafetyViolation::PotentialLeak { allocation_stack, .. } => {
            allocation_stack.iter().map(|frame| WebStackFrame {
                function_name: frame.function_name.clone(),
                file_name: frame.file_name.clone(),
                line_number: frame.line_number,
                is_unsafe: frame.is_unsafe,
            }).collect()
        },
        SafetyViolation::CrossBoundaryRisk { stack, .. } => {
            stack.iter().map(|frame| WebStackFrame {
                function_name: frame.function_name.clone(),
                file_name: frame.file_name.clone(),
                line_number: frame.line_number,
                is_unsafe: frame.is_unsafe,
            }).collect()
        },
    }
}