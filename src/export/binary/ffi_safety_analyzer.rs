//! FFI safety analysis for binary to HTML conversion
//!
//! This module provides comprehensive analysis of Foreign Function Interface (FFI) safety
//! based on allocation data, identifying unsafe operations, risk assessment, and hotspot detection.

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FFI safety analysis results for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiSafetyAnalysis {
    /// Summary statistics for FFI safety
    pub summary: FfiSafetySummary,
    /// Unsafe operations detected
    pub unsafe_operations: Vec<UnsafeOperation>,
    /// FFI call relationships and hotspots
    pub ffi_hotspots: Vec<FfiHotspot>,
    /// Risk assessment by category
    pub risk_assessment: RiskAssessment,
    /// FFI call graph data for visualization
    pub call_graph: FfiCallGraph,
}

/// Summary statistics for FFI safety analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiSafetySummary {
    /// Total number of allocations analyzed
    pub total_allocations: usize,
    /// Number of potentially unsafe operations
    pub unsafe_operations_count: usize,
    /// Number of FFI-related allocations
    pub ffi_allocations_count: usize,
    /// Overall safety score (0-100, higher is safer)
    pub safety_score: u32,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// Memory used by FFI operations
    pub ffi_memory_usage: usize,
    /// Percentage of total memory used by FFI
    pub ffi_memory_percentage: f64,
}

/// Information about an unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeOperation {
    /// Unique identifier for the operation
    pub id: String,
    /// Type of unsafe operation
    pub operation_type: UnsafeOperationType,
    /// Memory address involved
    pub memory_address: usize,
    /// Size of memory involved
    pub memory_size: usize,
    /// Function or scope where operation occurred
    pub location: String,
    /// Stack trace if available
    pub stack_trace: Vec<String>,
    /// Risk level of this operation
    pub risk_level: RiskLevel,
    /// Description of the potential issue
    pub description: String,
    /// Timestamp when detected
    pub timestamp: u64,
}

/// FFI hotspot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiHotspot {
    /// Function or module name
    pub name: String,
    /// Number of FFI calls from this location
    pub call_count: usize,
    /// Total memory allocated through FFI calls
    pub total_memory: usize,
    /// Average allocation size
    pub average_size: f64,
    /// Risk score for this hotspot (0-100)
    pub risk_score: u32,
    /// Types of operations performed
    pub operation_types: Vec<String>,
    /// Related unsafe operations
    pub unsafe_operations: Vec<String>,
}

/// Risk assessment by different categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Memory safety risks
    pub memory_safety: CategoryRisk,
    /// Type safety risks
    pub type_safety: CategoryRisk,
    /// Concurrency safety risks
    pub concurrency_safety: CategoryRisk,
    /// Data integrity risks
    pub data_integrity: CategoryRisk,
    /// Overall risk distribution
    pub risk_distribution: HashMap<RiskLevel, usize>,
}

/// Risk information for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRisk {
    /// Risk level for this category
    pub level: RiskLevel,
    /// Number of issues in this category
    pub issue_count: usize,
    /// Description of main risks
    pub description: String,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// FFI call graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiCallGraph {
    /// Nodes in the call graph
    pub nodes: Vec<CallGraphNode>,
    /// Edges representing call relationships
    pub edges: Vec<CallGraphEdge>,
    /// Graph statistics
    pub statistics: GraphStatistics,
}

/// Node in the FFI call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNode {
    /// Unique node identifier
    pub id: String,
    /// Function or module name
    pub name: String,
    /// Node type (rust_function, ffi_function, external_library)
    pub node_type: NodeType,
    /// Number of allocations from this node
    pub allocation_count: usize,
    /// Total memory allocated
    pub memory_usage: usize,
    /// Risk level of this node
    pub risk_level: RiskLevel,
    /// Position for graph layout (x, y coordinates)
    pub position: (f64, f64),
}

/// Edge in the FFI call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Number of calls between nodes
    pub call_count: usize,
    /// Total memory transferred
    pub memory_transferred: usize,
    /// Edge type (safe_call, unsafe_call, ffi_boundary)
    pub edge_type: EdgeType,
}

/// Statistics for the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Number of FFI boundary crossings
    pub ffi_boundaries: usize,
    /// Maximum call depth
    pub max_depth: usize,
    /// Number of strongly connected components
    pub connected_components: usize,
}

/// Type of unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnsafeOperationType {
    /// Raw pointer dereference
    RawPointerDeref,
    /// Uninitialized memory access
    UninitializedMemory,
    /// Buffer overflow potential
    BufferOverflow,
    /// Use after free
    UseAfterFree,
    /// Double free
    DoubleFree,
    /// FFI boundary crossing
    FfiBoundary,
    /// Unsafe type casting
    UnsafeCast,
    /// Concurrent access without synchronization
    UnsafeConcurrency,
    /// Memory layout assumptions
    MemoryLayoutAssumption,
    /// Other unsafe operation
    Other(String),
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Low risk - generally safe operations
    Low,
    /// Medium risk - requires attention
    Medium,
    /// High risk - potentially dangerous
    High,
    /// Critical risk - immediate attention required
    Critical,
}

/// Type of node in the call graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Rust function
    RustFunction,
    /// FFI function
    FfiFunction,
    /// External library
    ExternalLibrary,
    /// System call
    SystemCall,
    /// Unknown type
    Unknown,
}

/// Type of edge in the call graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeType {
    /// Safe function call
    SafeCall,
    /// Unsafe function call
    UnsafeCall,
    /// FFI boundary crossing
    FfiBoundary,
    /// Memory transfer
    MemoryTransfer,
}

/// FFI safety analyzer for processing allocation data
pub struct FfiSafetyAnalyzer {
    /// Detected unsafe operations
    unsafe_operations: Vec<UnsafeOperation>,
    /// FFI hotspots tracker
    hotspots: HashMap<String, FfiHotspotTracker>,
    /// Call graph builder
    call_graph_builder: CallGraphBuilder,
    /// Risk assessment tracker
    risk_tracker: RiskTracker,
}

/// Internal tracker for FFI hotspots
#[derive(Debug, Clone)]
struct FfiHotspotTracker {
    call_count: usize,
    total_memory: usize,
    sizes: Vec<usize>,
    operation_types: Vec<String>,
    unsafe_operations: Vec<String>,
    risk_factors: Vec<String>,
}

/// Internal call graph builder
#[derive(Debug, Clone)]
struct CallGraphBuilder {
    nodes: HashMap<String, CallGraphNode>,
    edges: HashMap<(String, String), CallGraphEdge>,
    node_counter: usize,
}

/// Internal risk tracker
#[derive(Debug, Clone)]
struct RiskTracker {
    memory_safety_issues: usize,
    type_safety_issues: usize,
    concurrency_issues: usize,
    data_integrity_issues: usize,
}

impl FfiSafetyAnalyzer {
    /// Create a new FFI safety analyzer
    pub fn new() -> Self {
        Self {
            unsafe_operations: Vec::new(),
            hotspots: HashMap::new(),
            call_graph_builder: CallGraphBuilder::new(),
            risk_tracker: RiskTracker::new(),
        }
    }

    /// Analyze FFI safety from allocation data
    pub fn analyze_allocations(
        allocations: &[AllocationInfo],
    ) -> Result<FfiSafetyAnalysis, BinaryExportError> {
        let mut analyzer = Self::new();

        // Process each allocation for FFI safety analysis
        for allocation in allocations {
            analyzer.process_allocation(allocation)?;
        }

        // Generate analysis results
        analyzer.generate_analysis(allocations.len())
    }

    /// Process a single allocation for FFI safety analysis
    fn process_allocation(&mut self, allocation: &AllocationInfo) -> Result<(), BinaryExportError> {
        // Analyze stack trace for FFI patterns
        if let Some(ref stack_trace) = allocation.stack_trace {
            self.analyze_stack_trace(allocation, stack_trace)?;
        }

        // Analyze type name for FFI indicators
        if let Some(ref type_name) = allocation.type_name {
            self.analyze_type_safety(allocation, type_name)?;
        }

        // Analyze scope for unsafe operations
        if let Some(ref scope_name) = allocation.scope_name {
            self.analyze_scope_safety(allocation, scope_name)?;
        }

        // Check for memory safety issues
        self.analyze_memory_safety(allocation)?;

        Ok(())
    }

    /// Analyze stack trace for FFI patterns and unsafe operations
    fn analyze_stack_trace(
        &mut self,
        allocation: &AllocationInfo,
        stack_trace: &[String],
    ) -> Result<(), BinaryExportError> {
        for (index, frame) in stack_trace.iter().enumerate() {
            // Detect FFI boundary crossings
            if self.is_ffi_boundary(frame) {
                self.record_ffi_operation(allocation, frame, index)?;
            }

            // Detect unsafe operations
            if self.is_unsafe_operation(frame) {
                self.record_unsafe_operation(allocation, frame, stack_trace)?;
            }

            // Build call graph
            self.call_graph_builder.add_call_frame(frame, allocation)?;
        }

        Ok(())
    }

    /// Analyze type name for safety implications
    fn analyze_type_safety(
        &mut self,
        allocation: &AllocationInfo,
        type_name: &str,
    ) -> Result<(), BinaryExportError> {
        // Check for raw pointer types
        if self.is_raw_pointer_type(type_name) {
            let operation = UnsafeOperation {
                id: format!("raw_ptr_{}", allocation.ptr),
                operation_type: UnsafeOperationType::RawPointerDeref,
                memory_address: allocation.ptr,
                memory_size: allocation.size,
                location: type_name.to_string(),
                stack_trace: allocation.stack_trace.clone().unwrap_or_default(),
                risk_level: RiskLevel::High,
                description: format!("Raw pointer type detected: {type_name}"),
                timestamp: allocation.timestamp_alloc,
            };
            self.unsafe_operations.push(operation);
            self.risk_tracker.type_safety_issues += 1;
        }

        // Check for FFI-related types
        if self.is_ffi_type(type_name) {
            self.record_ffi_type_usage(allocation, type_name)?;
        }

        Ok(())
    }

    /// Analyze scope for unsafe operations
    fn analyze_scope_safety(
        &mut self,
        allocation: &AllocationInfo,
        scope_name: &str,
    ) -> Result<(), BinaryExportError> {
        if scope_name.contains("unsafe") {
            let operation = UnsafeOperation {
                id: format!("unsafe_scope_{}", allocation.ptr),
                operation_type: UnsafeOperationType::FfiBoundary,
                memory_address: allocation.ptr,
                memory_size: allocation.size,
                location: scope_name.to_string(),
                stack_trace: allocation.stack_trace.clone().unwrap_or_default(),
                risk_level: RiskLevel::Medium,
                description: format!("Allocation in unsafe scope: {scope_name}"),
                timestamp: allocation.timestamp_alloc,
            };
            self.unsafe_operations.push(operation);
            self.risk_tracker.memory_safety_issues += 1;
        }

        Ok(())
    }

    /// Analyze memory safety patterns
    fn analyze_memory_safety(
        &mut self,
        allocation: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        // Check for potential use-after-free
        if allocation.is_leaked {
            let operation = UnsafeOperation {
                id: format!("leak_{}", allocation.ptr),
                operation_type: UnsafeOperationType::UseAfterFree,
                memory_address: allocation.ptr,
                memory_size: allocation.size,
                location: allocation
                    .scope_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                stack_trace: allocation.stack_trace.clone().unwrap_or_default(),
                risk_level: RiskLevel::High,
                description: "Potential memory leak detected".to_string(),
                timestamp: allocation.timestamp_alloc,
            };
            self.unsafe_operations.push(operation);
            self.risk_tracker.memory_safety_issues += 1;
        }

        // Check for large allocations (potential buffer overflow risk)
        if allocation.size > 1024 * 1024 {
            // Allocations larger than 1MB
            let operation = UnsafeOperation {
                id: format!("large_alloc_{}", allocation.ptr),
                operation_type: UnsafeOperationType::BufferOverflow,
                memory_address: allocation.ptr,
                memory_size: allocation.size,
                location: allocation
                    .scope_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                stack_trace: allocation.stack_trace.clone().unwrap_or_default(),
                risk_level: RiskLevel::Medium,
                description: format!(
                    "Large allocation detected: {size} bytes",
                    size = allocation.size
                ),
                timestamp: allocation.timestamp_alloc,
            };
            self.unsafe_operations.push(operation);
            self.risk_tracker.data_integrity_issues += 1;
        }

        Ok(())
    }

    /// Check if a stack frame indicates FFI boundary
    fn is_ffi_boundary(&self, frame: &str) -> bool {
        frame.contains("::ffi::")
            || frame.contains("extern \"C\"")
            || frame.contains("libc::")
            || frame.contains("_c_")
            || frame.contains("sys::")
            || frame.ends_with("_sys")
    }

    /// Check if a stack frame indicates unsafe operation
    fn is_unsafe_operation(&self, frame: &str) -> bool {
        frame.contains("unsafe")
            || frame.contains("transmute")
            || frame.contains("from_raw")
            || frame.contains("as_ptr")
            || frame.contains("as_mut_ptr")
    }

    /// Check if a type is a raw pointer type
    fn is_raw_pointer_type(&self, type_name: &str) -> bool {
        type_name.starts_with("*const")
            || type_name.starts_with("*mut")
            || type_name.contains("NonNull")
            || type_name.contains("RawPtr")
    }

    /// Check if a type is FFI-related
    fn is_ffi_type(&self, type_name: &str) -> bool {
        type_name.starts_with("c_")
            || type_name.contains("CString")
            || type_name.contains("CStr")
            || type_name.contains("OsString")
            || type_name.contains("PathBuf")
            || type_name.contains("libc::")
    }

    /// Record FFI operation
    fn record_ffi_operation(
        &mut self,
        allocation: &AllocationInfo,
        frame: &str,
        _index: usize,
    ) -> Result<(), BinaryExportError> {
        let location = self.extract_function_name(frame);

        let tracker = self
            .hotspots
            .entry(location.clone())
            .or_insert(FfiHotspotTracker {
                call_count: 0,
                total_memory: 0,
                sizes: Vec::new(),
                operation_types: Vec::new(),
                unsafe_operations: Vec::new(),
                risk_factors: Vec::new(),
            });

        tracker.call_count += 1;
        tracker.total_memory += allocation.size;
        tracker.sizes.push(allocation.size);
        tracker.operation_types.push("ffi_call".to_string());

        Ok(())
    }

    /// Record unsafe operation
    fn record_unsafe_operation(
        &mut self,
        allocation: &AllocationInfo,
        frame: &str,
        stack_trace: &[String],
    ) -> Result<(), BinaryExportError> {
        let operation = UnsafeOperation {
            id: format!("unsafe_{}", allocation.ptr),
            operation_type: self.classify_unsafe_operation(frame),
            memory_address: allocation.ptr,
            memory_size: allocation.size,
            location: self.extract_function_name(frame),
            stack_trace: stack_trace.to_vec(),
            risk_level: self.assess_risk_level(frame),
            description: format!("Unsafe operation detected in: {frame}"),
            timestamp: allocation.timestamp_alloc,
        };

        self.unsafe_operations.push(operation);
        self.risk_tracker.memory_safety_issues += 1;

        Ok(())
    }

    /// Record FFI type usage
    fn record_ffi_type_usage(
        &mut self,
        allocation: &AllocationInfo,
        type_name: &str,
    ) -> Result<(), BinaryExportError> {
        let location = format!("ffi_type_{type_name}");

        let tracker = self.hotspots.entry(location).or_insert(FfiHotspotTracker {
            call_count: 0,
            total_memory: 0,
            sizes: Vec::new(),
            operation_types: Vec::new(),
            unsafe_operations: Vec::new(),
            risk_factors: Vec::new(),
        });

        tracker.call_count += 1;
        tracker.total_memory += allocation.size;
        tracker.sizes.push(allocation.size);
        tracker.operation_types.push("ffi_type".to_string());

        Ok(())
    }

    /// Extract function name from stack frame
    fn extract_function_name(&self, frame: &str) -> String {
        // Simple extraction - in real implementation, this would be more sophisticated
        if let Some(start) = frame.find("::") {
            if let Some(end) = frame[start + 2..].find("::") {
                frame[start + 2..start + 2 + end].to_string()
            } else {
                frame[start + 2..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            }
        } else {
            frame
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        }
    }

    /// Classify the type of unsafe operation
    fn classify_unsafe_operation(&self, frame: &str) -> UnsafeOperationType {
        if frame.contains("transmute") {
            UnsafeOperationType::UnsafeCast
        } else if frame.contains("from_raw") {
            UnsafeOperationType::RawPointerDeref
        } else if frame.contains("ffi") {
            UnsafeOperationType::FfiBoundary
        } else {
            UnsafeOperationType::Other("unknown_unsafe".to_string())
        }
    }

    /// Assess risk level based on operation
    fn assess_risk_level(&self, frame: &str) -> RiskLevel {
        if frame.contains("transmute") || frame.contains("from_raw") {
            RiskLevel::High
        } else if frame.contains("unsafe") {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Generate the final analysis results
    fn generate_analysis(
        &mut self,
        total_allocations: usize,
    ) -> Result<FfiSafetyAnalysis, BinaryExportError> {
        // Calculate summary statistics
        let ffi_allocations_count = self.hotspots.values().map(|h| h.call_count).sum();
        let ffi_memory_usage = self.hotspots.values().map(|h| h.total_memory).sum();
        let ffi_memory_percentage = if total_allocations > 0 {
            (ffi_allocations_count as f64 / total_allocations as f64) * 100.0
        } else {
            0.0
        };

        let safety_score = self.calculate_safety_score();
        let risk_level = self.determine_overall_risk_level();

        let summary = FfiSafetySummary {
            total_allocations,
            unsafe_operations_count: self.unsafe_operations.len(),
            ffi_allocations_count,
            safety_score,
            risk_level,
            ffi_memory_usage,
            ffi_memory_percentage,
        };

        // Generate hotspots
        let ffi_hotspots = self.generate_hotspots()?;

        // Generate risk assessment
        let risk_assessment = self.generate_risk_assessment()?;

        // Generate call graph
        let call_graph = self.call_graph_builder.build_graph()?;

        Ok(FfiSafetyAnalysis {
            summary,
            unsafe_operations: self.unsafe_operations.clone(),
            ffi_hotspots,
            risk_assessment,
            call_graph,
        })
    }

    /// Calculate overall safety score (0-100)
    fn calculate_safety_score(&self) -> u32 {
        let base_score = 100u32;
        let unsafe_penalty = (self.unsafe_operations.len() as u32).min(50);
        let risk_penalty = match self.determine_overall_risk_level() {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 10,
            RiskLevel::High => 25,
            RiskLevel::Critical => 50,
        };

        base_score.saturating_sub(unsafe_penalty + risk_penalty)
    }

    /// Determine overall risk level
    fn determine_overall_risk_level(&self) -> RiskLevel {
        let critical_count = self
            .unsafe_operations
            .iter()
            .filter(|op| op.risk_level == RiskLevel::Critical)
            .count();
        let high_count = self
            .unsafe_operations
            .iter()
            .filter(|op| op.risk_level == RiskLevel::High)
            .count();

        if critical_count > 0 {
            RiskLevel::Critical
        } else if high_count > 3 {
            RiskLevel::High
        } else if high_count > 0 || self.unsafe_operations.len() > 5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Generate hotspots from tracked data
    fn generate_hotspots(&self) -> Result<Vec<FfiHotspot>, BinaryExportError> {
        let mut hotspots = Vec::new();

        for (name, tracker) in &self.hotspots {
            let average_size = if tracker.call_count > 0 {
                tracker.total_memory as f64 / tracker.call_count as f64
            } else {
                0.0
            };

            let risk_score = self.calculate_hotspot_risk_score(tracker);

            let hotspot = FfiHotspot {
                name: name.clone(),
                call_count: tracker.call_count,
                total_memory: tracker.total_memory,
                average_size,
                risk_score,
                operation_types: tracker.operation_types.clone(),
                unsafe_operations: tracker.unsafe_operations.clone(),
            };

            hotspots.push(hotspot);
        }

        // Sort by risk score (descending)
        hotspots.sort_by(|a, b| b.risk_score.cmp(&a.risk_score));

        Ok(hotspots)
    }

    /// Calculate risk score for a hotspot
    fn calculate_hotspot_risk_score(&self, tracker: &FfiHotspotTracker) -> u32 {
        let mut score = 0u32;

        // Base score from call count
        score += (tracker.call_count as u32).min(50);

        // Memory usage factor
        if tracker.total_memory > 1024 * 1024 {
            score += 20;
        }

        // Risk factors
        score += (tracker.risk_factors.len() as u32 * 10).min(30);

        score.min(100)
    }

    /// Generate risk assessment
    fn generate_risk_assessment(&self) -> Result<RiskAssessment, BinaryExportError> {
        let memory_safety = CategoryRisk {
            level: if self.risk_tracker.memory_safety_issues > 5 {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            },
            issue_count: self.risk_tracker.memory_safety_issues,
            description: "Memory safety issues detected in FFI operations".to_string(),
            recommendations: vec![
                "Review unsafe memory operations".to_string(),
                "Add bounds checking".to_string(),
                "Use safe wrapper types".to_string(),
            ],
        };

        let type_safety = CategoryRisk {
            level: if self.risk_tracker.type_safety_issues > 3 {
                RiskLevel::High
            } else {
                RiskLevel::Low
            },
            issue_count: self.risk_tracker.type_safety_issues,
            description: "Type safety concerns in FFI boundaries".to_string(),
            recommendations: vec![
                "Validate type conversions".to_string(),
                "Use newtype wrappers".to_string(),
            ],
        };

        let concurrency_safety = CategoryRisk {
            level: RiskLevel::Low,
            issue_count: self.risk_tracker.concurrency_issues,
            description: "Concurrency safety analysis".to_string(),
            recommendations: vec!["Review thread safety".to_string()],
        };

        let data_integrity = CategoryRisk {
            level: if self.risk_tracker.data_integrity_issues > 2 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            issue_count: self.risk_tracker.data_integrity_issues,
            description: "Data integrity concerns".to_string(),
            recommendations: vec!["Validate data consistency".to_string()],
        };

        // Count risk distribution
        let mut risk_distribution = HashMap::new();
        for operation in &self.unsafe_operations {
            *risk_distribution
                .entry(operation.risk_level.clone())
                .or_insert(0) += 1;
        }

        Ok(RiskAssessment {
            memory_safety,
            type_safety,
            concurrency_safety,
            data_integrity,
            risk_distribution,
        })
    }
}

impl CallGraphBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_counter: 0,
        }
    }

    fn add_call_frame(
        &mut self,
        frame: &str,
        allocation: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let node_id = format!("node_{}", self.node_counter);
        self.node_counter += 1;

        let node = CallGraphNode {
            id: node_id.clone(),
            name: frame.to_string(),
            node_type: self.classify_node_type(frame),
            allocation_count: 1,
            memory_usage: allocation.size,
            risk_level: RiskLevel::Low,
            position: (0.0, 0.0), // Will be calculated during layout
        };

        self.nodes.insert(node_id, node);
        Ok(())
    }

    fn classify_node_type(&self, frame: &str) -> NodeType {
        if frame.contains("::ffi::") || frame.contains("extern \"C\"") {
            NodeType::FfiFunction
        } else if frame.contains("libc::") || frame.contains("sys::") {
            NodeType::ExternalLibrary
        } else {
            NodeType::RustFunction
        }
    }

    fn build_graph(&self) -> Result<FfiCallGraph, BinaryExportError> {
        let nodes: Vec<CallGraphNode> = self.nodes.values().cloned().collect();
        let edges: Vec<CallGraphEdge> = self.edges.values().cloned().collect();

        let statistics = GraphStatistics {
            node_count: nodes.len(),
            edge_count: edges.len(),
            ffi_boundaries: edges
                .iter()
                .filter(|e| e.edge_type == EdgeType::FfiBoundary)
                .count(),
            max_depth: 0,            // Would be calculated in real implementation
            connected_components: 1, // Simplified
        };

        Ok(FfiCallGraph {
            nodes,
            edges,
            statistics,
        })
    }
}

impl RiskTracker {
    fn new() -> Self {
        Self {
            memory_safety_issues: 0,
            type_safety_issues: 0,
            concurrency_issues: 0,
            data_integrity_issues: 0,
        }
    }
}

impl Default for FfiSafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_allocation_with_stack_trace(
        ptr: usize,
        size: usize,
        stack_trace: Vec<String>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: Some("test_var".to_string()),
            type_name: Some("TestType".to_string()),
            scope_name: Some("test_scope".to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: Some(stack_trace),
            is_leaked: false,
            lifetime_ms: None,
            borrow_info: None,
            clone_info: None,
            ownership_history_available: false,
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
        }
    }

    #[test]
    fn test_ffi_boundary_detection() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert!(analyzer.is_ffi_boundary("module::ffi::function"));
        assert!(analyzer.is_ffi_boundary("extern \"C\" fn test"));
        assert!(analyzer.is_ffi_boundary("libc::malloc"));
        assert!(!analyzer.is_ffi_boundary("std::vec::Vec::new"));
    }

    #[test]
    fn test_unsafe_operation_detection() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert!(analyzer.is_unsafe_operation("unsafe { transmute(ptr) }"));
        assert!(analyzer.is_unsafe_operation("Vec::from_raw_parts"));
        assert!(analyzer.is_unsafe_operation("slice.as_mut_ptr()"));
        assert!(!analyzer.is_unsafe_operation("Vec::new()"));
    }

    #[test]
    fn test_raw_pointer_type_detection() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert!(analyzer.is_raw_pointer_type("*const u8"));
        assert!(analyzer.is_raw_pointer_type("*mut i32"));
        assert!(analyzer.is_raw_pointer_type("NonNull<T>"));
        assert!(!analyzer.is_raw_pointer_type("Box<T>"));
        assert!(!analyzer.is_raw_pointer_type("&mut T"));
    }

    #[test]
    fn test_ffi_type_detection() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert!(analyzer.is_ffi_type("c_int"));
        assert!(analyzer.is_ffi_type("CString"));
        assert!(analyzer.is_ffi_type("CStr"));
        assert!(analyzer.is_ffi_type("libc::size_t"));
        assert!(!analyzer.is_ffi_type("String"));
        assert!(!analyzer.is_ffi_type("Vec<u8>"));
    }

    #[test]
    fn test_ffi_safety_analysis() {
        let allocations = vec![
            create_test_allocation_with_stack_trace(
                0x1000,
                1024,
                vec![
                    "main::test".to_string(),
                    "module::ffi::unsafe_function".to_string(),
                    "libc::malloc".to_string(),
                ],
            ),
            create_test_allocation_with_stack_trace(
                0x2000,
                2048,
                vec![
                    "safe_function".to_string(),
                    "std::vec::Vec::new".to_string(),
                ],
            ),
        ];

        let analysis =
            FfiSafetyAnalyzer::analyze_allocations(&allocations).expect("Failed to get test value");

        assert_eq!(analysis.summary.total_allocations, 2);
        assert!(analysis.summary.unsafe_operations_count > 0);
        assert!(analysis.summary.ffi_allocations_count > 0);
        assert!(analysis.summary.safety_score <= 100);
        assert!(!analysis.ffi_hotspots.is_empty());
    }

    #[test]
    fn test_risk_level_assessment() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert_eq!(analyzer.assess_risk_level("transmute"), RiskLevel::High);
        assert_eq!(analyzer.assess_risk_level("from_raw"), RiskLevel::High);
        assert_eq!(
            analyzer.assess_risk_level("unsafe block"),
            RiskLevel::Medium
        );
        assert_eq!(analyzer.assess_risk_level("safe_function"), RiskLevel::Low);
    }

    #[test]
    fn test_function_name_extraction() {
        let analyzer = FfiSafetyAnalyzer::new();

        assert_eq!(
            analyzer.extract_function_name("module::submodule::function"),
            "submodule"
        );
        assert_eq!(analyzer.extract_function_name("std::vec::Vec::new"), "vec");
        assert_eq!(
            analyzer.extract_function_name("simple_function"),
            "simple_function"
        );
    }

    #[test]
    fn test_safety_score_calculation() {
        let mut analyzer = FfiSafetyAnalyzer::new();

        // Add some unsafe operations
        analyzer.unsafe_operations.push(UnsafeOperation {
            id: "test1".to_string(),
            operation_type: UnsafeOperationType::RawPointerDeref,
            memory_address: 0x1000,
            memory_size: 1024,
            location: "test".to_string(),
            stack_trace: vec![],
            risk_level: RiskLevel::High,
            description: "Test".to_string(),
            timestamp: 1000,
        });

        let score = analyzer.calculate_safety_score();
        assert!(score < 100);
    }
}
