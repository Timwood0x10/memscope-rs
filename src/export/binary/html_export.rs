//! Binary to HTML export functionality
//!
//! This module provides direct conversion from binary files to HTML dashboards
//! using the templates in ./templates/

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::reader::BinaryReader;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Export binary data directly to HTML dashboard
pub fn export_binary_to_html<P: AsRef<Path>>(
    binary_path: P,
    output_path: P,
    project_name: &str,
) -> Result<(), BinaryExportError> {
    let start = std::time::Instant::now();
    tracing::info!("🎨 Starting binary to HTML export for {}", project_name);

    // Read the HTML template
    let template_path = "templates/dashboard.html";
    let template_content = fs::read_to_string(template_path)
        .map_err(|e| BinaryExportError::Io(e))?;

    // Read binary data using BinaryReader
    let mut reader = BinaryReader::new(&binary_path)?;
    let header = reader.read_header()?;
    
    // Generate JSON data for the dashboard
    let json_data = generate_dashboard_data(&mut reader, &header, project_name)?;
    
    // Replace placeholders in template (matching existing template format)
    let html_content = template_content
        .replace("{{PROJECT_NAME}}", project_name)
        .replace("{{MEMORY_DATA}}", &json_data)
        .replace("{{GENERATION_TIME}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());

    // Write HTML file
    let output_file = fs::File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);
    writer.write_all(html_content.as_bytes())?;
    writer.flush()?;

    let elapsed = start.elapsed();
    tracing::info!("✅ HTML dashboard generated in {}ms: {:?}", elapsed.as_millis(), output_path.as_ref());

    Ok(())
}

/// Generate comprehensive dashboard data from binary
fn generate_dashboard_data(
    reader: &mut BinaryReader,
    header: &crate::export::binary::format::FileHeader,
    project_name: &str,
) -> Result<String, BinaryExportError> {
    let mut allocations = Vec::new();
    let mut memory_stats = MemoryStats::new();
    let mut lifecycle_events = Vec::new();
    let mut performance_data = PerformanceData::new();

    // Read all allocations and collect statistics
    for i in 0..header.get_allocation_counts().0 {
        let allocation = reader.read_allocation()?;
        
        // Update statistics
        memory_stats.update(&allocation);
        
        // Create lifecycle event
        lifecycle_events.push(LifecycleEvent::from_allocation(&allocation, i));
        
        // Update performance data
        performance_data.update(&allocation);
        
        // Store allocation for detailed view
        allocations.push(allocation);
    }

    // Generate comprehensive JSON structure
    let dashboard_data = DashboardData {
        project_name: project_name.to_string(),
        summary: SummaryData {
            total_allocations: header.get_allocation_counts().0 as usize,
            total_memory: memory_stats.total_size,
            peak_memory: memory_stats.peak_memory,
            active_allocations: memory_stats.active_count,
            average_allocation_size: if header.get_allocation_counts().0 > 0 {
                memory_stats.total_size / header.get_allocation_counts().0 as u64
            } else { 0 },
        },
        memory_analysis: MemoryAnalysisData {
            allocations: allocations.into_iter().take(1000).map(AllocationData::from).collect(), // Limit for performance
            memory_timeline: memory_stats.timeline,
            size_distribution: memory_stats.size_distribution,
        },
        lifecycle_analysis: LifecycleAnalysisData {
            events: lifecycle_events.into_iter().take(1000).collect(), // Limit for performance
            scope_analysis: memory_stats.scope_analysis,
        },
        performance_analysis: performance_data,
        metadata: MetadataInfo {
            export_time: chrono::Utc::now().timestamp(),
            export_version: "2.0".to_string(),
            optimization_level: "High".to_string(),
            binary_file_size: 0, // Will be filled by caller
        },
    };

    // Serialize to JSON
    serde_json::to_string_pretty(&dashboard_data)
        .map_err(|e| BinaryExportError::SerializationError(format!("JSON serialization failed: {}", e)))
}

// Data structures for dashboard
#[derive(serde::Serialize)]
struct DashboardData {
    project_name: String,
    summary: SummaryData,
    memory_analysis: MemoryAnalysisData,
    lifecycle_analysis: LifecycleAnalysisData,
    performance_analysis: PerformanceData,
    metadata: MetadataInfo,
}

#[derive(serde::Serialize)]
struct SummaryData {
    total_allocations: usize,
    total_memory: u64,
    peak_memory: u64,
    active_allocations: usize,
    average_allocation_size: u64,
}

#[derive(serde::Serialize)]
struct MemoryAnalysisData {
    allocations: Vec<AllocationData>,
    memory_timeline: Vec<TimelinePoint>,
    size_distribution: Vec<SizeDistribution>,
}

#[derive(serde::Serialize)]
struct LifecycleAnalysisData {
    events: Vec<LifecycleEvent>,
    scope_analysis: ScopeAnalysis,
}

#[derive(serde::Serialize)]
struct AllocationData {
    id: u64,
    size: u64,
    type_name: String,
    location: String,
    timestamp: u64,
    status: String,
}

impl From<crate::core::types::AllocationInfo> for AllocationData {
    fn from(alloc: crate::core::types::AllocationInfo) -> Self {
        Self {
            id: alloc.ptr as u64, // Use ptr as unique ID
            size: alloc.size as u64,
            type_name: alloc.type_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            location: alloc.scope_name.clone().unwrap_or_else(|| "Unknown".to_string()),
            timestamp: alloc.timestamp_alloc,
            status: if alloc.is_active() { "Active".to_string() } else { "Freed".to_string() },
        }
    }
}

#[derive(serde::Serialize)]
struct LifecycleEvent {
    id: u64,
    event_type: String,
    timestamp: u64,
    size: u64,
    location: String,
}

impl LifecycleEvent {
    fn from_allocation(alloc: &crate::core::types::AllocationInfo, index: u32) -> Self {
        Self {
            id: alloc.ptr as u64,
            event_type: if alloc.is_active() { "Allocation".to_string() } else { "Deallocation".to_string() },
            timestamp: alloc.timestamp_alloc,
            size: alloc.size as u64,
            location: alloc.scope_name.clone().unwrap_or_else(|| format!("Location_{}", index)),
        }
    }
}

#[derive(serde::Serialize)]
struct PerformanceData {
    allocation_distribution: Vec<AllocationDistribution>,
    memory_performance: MemoryPerformance,
    export_performance: ExportPerformance,
}

impl PerformanceData {
    fn new() -> Self {
        Self {
            allocation_distribution: Vec::new(),
            memory_performance: MemoryPerformance::default(),
            export_performance: ExportPerformance::default(),
        }
    }

    fn update(&mut self, _alloc: &crate::core::types::AllocationInfo) {
        // Update performance metrics
        self.memory_performance.total_allocations += 1;
    }
}

#[derive(serde::Serialize, Default)]
struct MemoryPerformance {
    total_allocations: u64,
    peak_memory_usage: u64,
    average_allocation_time: f64,
}

#[derive(serde::Serialize, Default)]
struct ExportPerformance {
    export_time_ms: u64,
    compression_ratio: f64,
    throughput_mb_per_sec: f64,
}

#[derive(serde::Serialize)]
struct AllocationDistribution {
    size_range: String,
    count: u64,
    percentage: f64,
}

#[derive(serde::Serialize)]
struct TimelinePoint {
    timestamp: u64,
    memory_usage: u64,
    allocation_count: u64,
}

#[derive(serde::Serialize)]
struct SizeDistribution {
    size_range: String,
    count: u64,
    total_size: u64,
}

#[derive(serde::Serialize, Default)]
struct ScopeAnalysis {
    total_scopes: u64,
    average_scope_lifetime: f64,
    max_nested_depth: u32,
}

#[derive(serde::Serialize)]
struct MetadataInfo {
    export_time: i64,
    export_version: String,
    optimization_level: String,
    binary_file_size: u64,
}

// Helper struct for collecting memory statistics
struct MemoryStats {
    total_size: u64,
    peak_memory: u64,
    active_count: usize,
    timeline: Vec<TimelinePoint>,
    size_distribution: Vec<SizeDistribution>,
    scope_analysis: ScopeAnalysis,
}

impl MemoryStats {
    fn new() -> Self {
        Self {
            total_size: 0,
            peak_memory: 0,
            active_count: 0,
            timeline: Vec::new(),
            size_distribution: Vec::new(),
            scope_analysis: ScopeAnalysis::default(),
        }
    }

    fn update(&mut self, alloc: &crate::core::types::AllocationInfo) {
        self.total_size += alloc.size as u64;
        if alloc.is_active() {
            self.active_count += 1;
        }
        self.peak_memory = self.peak_memory.max(self.total_size);
        
        // Add timeline point every 100 allocations
        if self.timeline.len() < 1000 && self.timeline.len() % 10 == 0 {
            self.timeline.push(TimelinePoint {
                timestamp: alloc.timestamp_alloc,
                memory_usage: self.total_size,
                allocation_count: self.active_count as u64,
            });
        }
    }
}