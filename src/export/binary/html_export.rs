//! Binary to HTML export functionality
//!
//! This module provides direct conversion from binary files to HTML dashboards
//! using the templates in ./templates/

use crate::export::binary::error::BinaryExportError;
use crate::export::binary::reader::BinaryReader;
use std::collections::HashMap;
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

    // Read the binary-specific HTML template
    let template_path = "templates/binary_dashboard.html";
    let template_content = fs::read_to_string(template_path)
        .map_err(|e| BinaryExportError::Io(e))?;

    // Read binary data using BinaryReader
    let mut reader = BinaryReader::new(&binary_path)?;
    let header = reader.read_header()?;
    
    // Generate JSON data for the dashboard
    let json_data = generate_dashboard_data(&mut reader, &header, project_name)?;
    
    // Replace placeholders in template (matching binary template format)
    let html_content = template_content
        .replace("{{PROJECT_NAME}}", project_name)
        .replace("{{BINARY_DATA}}", &json_data)  // Binary template expects this
        .replace("{{GENERATION_TIME}}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .replace("{{PROCESSING_TIME}}", "0")  // Will be updated with actual processing time
        .replace("{{CSS_CONTENT}}", "")  // Placeholder for CSS content
        .replace("{{JS_CONTENT}}", "");  // Placeholder for JS content

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

    // Generate complex types analysis
    let complex_types_analysis = generate_complex_types_analysis(&allocations);
    
    // Generate FFI safety analysis
    let ffi_analysis = generate_ffi_analysis(&allocations);
    
    // Generate variable relationships graph
    let variable_relationships = generate_variable_relationships(&allocations);

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
        // New fields to match template expectations
        complex_types: complex_types_analysis,
        unsafe_ffi: ffi_analysis,
        variable_relationships,
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
    // New fields to match template expectations
    complex_types: ComplexTypesAnalysis,
    unsafe_ffi: FfiAnalysis,
    variable_relationships: VariableRelationships,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::AllocationInfo;

    #[test]
    fn test_generate_complex_types_analysis() {
        let allocations = vec![
            AllocationInfo {
                ptr: 0x1000,
                size: 64,
                var_name: Some("vec_data".to_string()),
                type_name: Some("Vec<i32>".to_string()),
                scope_name: Some("main".to_string()),
                timestamp_alloc: 1000,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
            AllocationInfo {
                ptr: 0x2000,
                size: 32,
                var_name: Some("box_data".to_string()),
                type_name: Some("Box<String>".to_string()),
                scope_name: Some("main".to_string()),
                timestamp_alloc: 1100,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
        ];

        let analysis = generate_complex_types_analysis(&allocations);
        
        assert_eq!(analysis.summary.total_complex_types, 2);
        assert_eq!(analysis.summary.smart_pointers_count, 1);
        assert_eq!(analysis.summary.collections_count, 1);
        assert!(analysis.summary.average_complexity_score > 0.0);
        
        // Check that we have the expected categories
        assert!(!analysis.categorized_types.smart_pointers.is_empty());
        assert!(!analysis.categorized_types.collections.is_empty());
    }

    #[test]
    fn test_generate_ffi_analysis() {
        let allocations = vec![
            AllocationInfo {
                ptr: 0x3000,
                size: 8,
                var_name: Some("raw_ptr".to_string()),
                type_name: Some("*mut u8".to_string()),
                scope_name: Some("unsafe_block".to_string()),
                timestamp_alloc: 2000,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
        ];

        let analysis = generate_ffi_analysis(&allocations);
        
        assert_eq!(analysis.total_violations, 1);
        assert_eq!(analysis.risk_level, "Medium");
        assert_eq!(analysis.unsafe_operations.len(), 1);
        assert_eq!(analysis.unsafe_operations[0].operation_type, "MutableRawPointer");
        assert_eq!(analysis.unsafe_operations[0].risk_level, "High");
    }

    #[test]
    fn test_generate_variable_relationships() {
        let allocations = vec![
            AllocationInfo {
                ptr: 0x4000,
                size: 16,
                var_name: Some("var1".to_string()),
                type_name: Some("i32".to_string()),
                scope_name: Some("function1".to_string()),
                timestamp_alloc: 3000,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
            AllocationInfo {
                ptr: 0x5000,
                size: 16,
                var_name: Some("var2".to_string()),
                type_name: Some("i32".to_string()),
                scope_name: Some("function1".to_string()),
                timestamp_alloc: 3100,
                timestamp_dealloc: None,
                thread_id: "main".to_string(),
                borrow_count: 0,
                stack_trace: None,
                is_leaked: false,
                lifetime_ms: None,
                smart_pointer_info: None,
                memory_layout: None,
                generic_info: None,
                dynamic_type_info: None,
                runtime_state: None,
                stack_allocation: None,
                temporary_object: None,
                fragmentation_analysis: None,
                generic_instantiation: None,
                type_relationships: None,
                type_usage: None,
                function_call_tracking: None,
                lifecycle_tracking: None,
                access_tracking: None,
                drop_chain_analysis: None,
            },
        ];

        let relationships = generate_variable_relationships(&allocations);
        
        assert_eq!(relationships.nodes.len(), 2);
        assert_eq!(relationships.edges.len(), 1); // Should have one relationship edge
        assert!(relationships.categories.contains_key("primitive"));
        
        // Check that nodes have correct information
        let node1 = &relationships.nodes[0];
        assert_eq!(node1.name, "var1");
        assert_eq!(node1.type_name, "i32");
        assert_eq!(node1.category, "primitive");
    }
}

/// Generate complex types analysis from allocations
fn generate_complex_types_analysis(allocations: &[crate::core::types::AllocationInfo]) -> ComplexTypesAnalysis {
    let mut type_stats: HashMap<String, (usize, u64, u32)> = HashMap::new(); // (count, total_size, max_complexity)
    let mut smart_pointers = Vec::new();
    let mut collections = Vec::new();
    let mut generic_types = Vec::new();
    let mut primitive_types = Vec::new();
    let mut complexity_scores = HashMap::new();
    let mut generic_instances = HashMap::new();

    for alloc in allocations {
        if let Some(type_name) = &alloc.type_name {
            let normalized_type = normalize_type_name(type_name);
            let category = categorize_type(type_name);
            let complexity = calculate_type_complexity(type_name);
            
            // Update type statistics
            let entry = type_stats.entry(normalized_type.clone()).or_insert((0, 0, 0));
            entry.0 += 1; // count
            entry.1 += alloc.size as u64; // total_size
            entry.2 = entry.2.max(complexity); // max_complexity
            
            complexity_scores.insert(normalized_type.clone(), complexity);
            
            // Create type info
            let type_info = TypeInfo {
                type_name: normalized_type.clone(),
                count: entry.0,
                total_size: entry.1,
                complexity_score: complexity,
                category: category.clone(),
            };
            
            // Categorize types
            match category.as_str() {
                "smart_pointer" => {
                    if !smart_pointers.iter().any(|t: &TypeInfo| t.type_name == normalized_type) {
                        smart_pointers.push(type_info);
                    }
                }
                "collection" => {
                    if !collections.iter().any(|t: &TypeInfo| t.type_name == normalized_type) {
                        collections.push(type_info);
                    }
                }
                "generic" => {
                    if !generic_types.iter().any(|t: &TypeInfo| t.type_name == normalized_type) {
                        generic_types.push(type_info);
                    }
                    // Track generic instances
                    let base_type = extract_generic_base_type(type_name);
                    let entry = generic_instances.entry(base_type).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += alloc.size as u64;
                }
                "primitive" => {
                    if !primitive_types.iter().any(|t: &TypeInfo| t.type_name == normalized_type) {
                        primitive_types.push(type_info);
                    }
                }
                _ => {}
            }
        }
    }

    // Calculate summary statistics
    let total_complex_types = type_stats.len();
    let smart_pointers_count = smart_pointers.len();
    let collections_count = collections.len();
    let generic_types_count = generic_types.len();
    let average_complexity_score = if !complexity_scores.is_empty() {
        complexity_scores.values().sum::<u32>() as f64 / complexity_scores.len() as f64
    } else {
        0.0
    };

    // Create generic analysis
    let mut most_used_generics: Vec<GenericTypeUsage> = generic_instances
        .into_iter()
        .map(|(type_name, (count, size))| GenericTypeUsage {
            type_name,
            instance_count: count,
            total_size: size,
        })
        .collect();
    most_used_generics.sort_by(|a, b| b.instance_count.cmp(&a.instance_count));
    most_used_generics.truncate(10); // Top 10

    ComplexTypesAnalysis {
        summary: ComplexTypesSummary {
            total_complex_types,
            smart_pointers_count,
            collections_count,
            generic_types_count,
            average_complexity_score,
        },
        categorized_types: CategorizedTypes {
            smart_pointers,
            collections,
            generic_types,
            primitive_types,
        },
        complexity_scores,
        generic_analysis: GenericTypeAnalysis {
            total_generic_instances: most_used_generics.iter().map(|g| g.instance_count).sum(),
            unique_generic_types: most_used_generics.len(),
            most_used_generics,
        },
    }
}

/// Generate FFI safety analysis from allocations
fn generate_ffi_analysis(allocations: &[crate::core::types::AllocationInfo]) -> FfiAnalysis {
    let mut unsafe_operations = Vec::new();
    let mut security_hotspots = HashMap::new();
    let mut ffi_nodes = Vec::new();
    let ffi_edges = Vec::new();
    let mut violation_count = 0;

    for alloc in allocations {
        // Check for unsafe patterns
        if let Some(type_name) = &alloc.type_name {
            if is_unsafe_type(type_name) {
                violation_count += 1;
                let risk_level = assess_risk_level(type_name);
                
                unsafe_operations.push(UnsafeOperation {
                    ptr: format!("0x{:x}", alloc.ptr),
                    operation_type: classify_unsafe_operation(type_name),
                    risk_level: risk_level.clone(),
                    timestamp: alloc.timestamp_alloc,
                    location: alloc.scope_name.clone().unwrap_or_else(|| "Unknown".to_string()),
                });

                // Track security hotspots by location
                let location = alloc.scope_name.clone().unwrap_or_else(|| "Unknown".to_string());
                let entry = security_hotspots.entry(location.clone()).or_insert((0, 0));
                entry.0 += 1; // violation count
                entry.1 = entry.1.max(calculate_risk_score(&risk_level)); // max risk score
                
                // Add FFI node
                ffi_nodes.push(FfiNode {
                    id: format!("0x{:x}", alloc.ptr),
                    name: type_name.clone(),
                    node_type: "unsafe_allocation".to_string(),
                    risk_level,
                });
            }
        }
    }

    // Convert security hotspots
    let security_hotspots_vec: Vec<SecurityHotspot> = security_hotspots
        .into_iter()
        .map(|(location, (count, risk_score))| SecurityHotspot {
            location: location.clone(),
            violation_count: count,
            risk_score,
            description: format!("Location with {} unsafe operations", count),
        })
        .collect();

    // Determine overall risk level
    let risk_level = if violation_count == 0 {
        "Low".to_string()
    } else if violation_count < 10 {
        "Medium".to_string()
    } else {
        "High".to_string()
    };

    FfiAnalysis {
        total_violations: violation_count,
        risk_level,
        unsafe_operations,
        security_hotspots: security_hotspots_vec,
        ffi_call_graph: FfiCallGraph {
            nodes: ffi_nodes,
            edges: ffi_edges,
        },
    }
}

/// Generate variable relationships graph from allocations
fn generate_variable_relationships(allocations: &[crate::core::types::AllocationInfo]) -> VariableRelationships {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut categories = HashMap::new();
    let mut variable_map = HashMap::new();

    // Create nodes for each allocation
    for alloc in allocations {
        let var_name = alloc.var_name.clone().unwrap_or_else(|| format!("var_{:x}", alloc.ptr));
        let type_name = alloc.type_name.clone().unwrap_or_else(|| "Unknown".to_string());
        let category = categorize_type(&type_name);
        let complexity = calculate_type_complexity(&type_name);
        let color = get_category_color(&category);

        let node = GraphNode {
            id: format!("0x{:x}", alloc.ptr),
            name: var_name.clone(),
            type_name: type_name.clone(),
            size: alloc.size as u64,
            complexity_score: complexity,
            category: category.clone(),
            color: color.clone(),
        };

        nodes.push(node);
        variable_map.insert(alloc.ptr, (var_name, type_name, category.clone()));

        // Update category info
        let entry = categories.entry(category.clone()).or_insert(CategoryInfo {
            name: category.clone(),
            color,
            count: 0,
        });
        entry.count += 1;
    }

    // Create edges based on relationships (simplified - could be enhanced with actual relationship analysis)
    for (i, alloc1) in allocations.iter().enumerate() {
        for alloc2 in allocations.iter().skip(i + 1) {
            if let (Some(scope1), Some(scope2)) = (&alloc1.scope_name, &alloc2.scope_name) {
                if scope1 == scope2 && alloc1.timestamp_alloc.abs_diff(alloc2.timestamp_alloc) < 1000 {
                    // Same scope and close in time - likely related
                    edges.push(GraphEdge {
                        source: format!("0x{:x}", alloc1.ptr),
                        target: format!("0x{:x}", alloc2.ptr),
                        relationship_type: "scope_related".to_string(),
                        strength: 0.5,
                    });
                }
            }
        }
    }

    VariableRelationships {
        nodes,
        edges,
        categories,
    }
}

// Helper functions for type analysis
fn normalize_type_name(type_name: &str) -> String {
    // Remove generic parameters for grouping
    if let Some(pos) = type_name.find('<') {
        type_name[..pos].to_string()
    } else {
        type_name.to_string()
    }
}

fn categorize_type(type_name: &str) -> String {
    if type_name.contains("Box<") || type_name.contains("Rc<") || type_name.contains("Arc<") {
        "smart_pointer".to_string()
    } else if type_name.contains("Vec<") || type_name.contains("HashMap<") || type_name.contains("HashSet<") {
        "collection".to_string()
    } else if type_name.contains('<') && type_name.contains('>') {
        "generic".to_string()
    } else if matches!(type_name, "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "bool" | "char") {
        "primitive".to_string()
    } else {
        "custom".to_string()
    }
}

fn calculate_type_complexity(type_name: &str) -> u32 {
    let mut complexity = 1;
    
    // Count generic parameters
    complexity += type_name.matches('<').count() as u32;
    
    // Add complexity for smart pointers
    if type_name.contains("Box<") || type_name.contains("Rc<") || type_name.contains("Arc<") {
        complexity += 2;
    }
    
    // Add complexity for collections
    if type_name.contains("Vec<") || type_name.contains("HashMap<") {
        complexity += 3;
    }
    
    // Add complexity for nested types
    complexity += type_name.matches("::").count() as u32;
    
    complexity.min(10) // Cap at 10
}

fn extract_generic_base_type(type_name: &str) -> String {
    if let Some(pos) = type_name.find('<') {
        type_name[..pos].to_string()
    } else {
        type_name.to_string()
    }
}

fn is_unsafe_type(type_name: &str) -> bool {
    type_name.contains("*mut") || type_name.contains("*const") || 
    type_name.contains("unsafe") || type_name.contains("raw")
}

fn assess_risk_level(type_name: &str) -> String {
    if type_name.contains("*mut") {
        "High".to_string()
    } else if type_name.contains("*const") {
        "Medium".to_string()
    } else if type_name.contains("unsafe") {
        "High".to_string()
    } else {
        "Low".to_string()
    }
}

fn classify_unsafe_operation(type_name: &str) -> String {
    if type_name.contains("*mut") {
        "MutableRawPointer".to_string()
    } else if type_name.contains("*const") {
        "ConstRawPointer".to_string()
    } else if type_name.contains("unsafe") {
        "UnsafeBlock".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn calculate_risk_score(risk_level: &str) -> u32 {
    match risk_level {
        "High" => 8,
        "Medium" => 5,
        "Low" => 2,
        _ => 1,
    }
}

fn get_category_color(category: &str) -> String {
    match category {
        "smart_pointer" => "#FF6B6B".to_string(),
        "collection" => "#4ECDC4".to_string(),
        "generic" => "#45B7D1".to_string(),
        "primitive" => "#96CEB4".to_string(),
        "custom" => "#FFEAA7".to_string(),
        _ => "#DDA0DD".to_string(),
    }
}

// New data structures for enhanced analysis
#[derive(serde::Serialize)]
struct ComplexTypesAnalysis {
    summary: ComplexTypesSummary,
    categorized_types: CategorizedTypes,
    complexity_scores: HashMap<String, u32>,
    generic_analysis: GenericTypeAnalysis,
}

#[derive(serde::Serialize)]
struct ComplexTypesSummary {
    total_complex_types: usize,
    smart_pointers_count: usize,
    collections_count: usize,
    generic_types_count: usize,
    average_complexity_score: f64,
}

#[derive(serde::Serialize)]
struct CategorizedTypes {
    smart_pointers: Vec<TypeInfo>,
    collections: Vec<TypeInfo>,
    generic_types: Vec<TypeInfo>,
    primitive_types: Vec<TypeInfo>,
}

#[derive(serde::Serialize)]
struct TypeInfo {
    type_name: String,
    count: usize,
    total_size: u64,
    complexity_score: u32,
    category: String,
}

#[derive(serde::Serialize)]
struct GenericTypeAnalysis {
    total_generic_instances: usize,
    unique_generic_types: usize,
    most_used_generics: Vec<GenericTypeUsage>,
}

#[derive(serde::Serialize)]
struct GenericTypeUsage {
    type_name: String,
    instance_count: usize,
    total_size: u64,
}

#[derive(serde::Serialize)]
struct FfiAnalysis {
    total_violations: usize,
    risk_level: String,
    unsafe_operations: Vec<UnsafeOperation>,
    security_hotspots: Vec<SecurityHotspot>,
    ffi_call_graph: FfiCallGraph,
}

#[derive(serde::Serialize)]
struct UnsafeOperation {
    ptr: String,
    operation_type: String,
    risk_level: String,
    timestamp: u64,
    location: String,
}

#[derive(serde::Serialize)]
struct SecurityHotspot {
    location: String,
    violation_count: usize,
    risk_score: u32,
    description: String,
}

#[derive(serde::Serialize)]
struct FfiCallGraph {
    nodes: Vec<FfiNode>,
    edges: Vec<FfiEdge>,
}

#[derive(serde::Serialize)]
struct FfiNode {
    id: String,
    name: String,
    node_type: String,
    risk_level: String,
}

#[derive(serde::Serialize)]
struct FfiEdge {
    source: String,
    target: String,
    relationship_type: String,
}

#[derive(serde::Serialize)]
struct VariableRelationships {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    categories: HashMap<String, CategoryInfo>,
}

#[derive(serde::Serialize)]
struct GraphNode {
    id: String,
    name: String,
    type_name: String,
    size: u64,
    complexity_score: u32,
    category: String,
    color: String,
}

#[derive(serde::Serialize)]
struct GraphEdge {
    source: String,
    target: String,
    relationship_type: String,
    strength: f32,
}

#[derive(serde::Serialize)]
struct CategoryInfo {
    name: String,
    color: String,
    count: usize,
}