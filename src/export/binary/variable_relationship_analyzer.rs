//! Variable relationship analysis for binary to HTML conversion
//!
//! This module provides comprehensive analysis of variable relationships based on allocation data,
//! generating D3.js compatible graph data structures for visualization.

use crate::core::types::AllocationInfo;
use crate::export::binary::error::BinaryExportError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Variable relationship analysis results for dashboard display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRelationshipAnalysis {
    /// Graph data for D3.js visualization
    pub graph: RelationshipGraph,
    /// Summary statistics
    pub summary: RelationshipSummary,
    /// Relationship patterns detected
    pub patterns: Vec<RelationshipPattern>,
    /// Performance optimization data
    pub optimization: GraphOptimization,
}

/// D3.js compatible graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipGraph {
    /// Nodes representing variables/allocations
    pub nodes: Vec<GraphNode>,
    /// Edges representing relationships
    pub links: Vec<GraphEdge>,
    /// Graph metadata
    pub metadata: GraphMetadata,
}

/// Node in the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,
    /// Variable name (if available)
    pub name: String,
    /// Memory address
    pub address: usize,
    /// Memory size in bytes
    pub size: usize,
    /// Type name
    pub type_name: String,
    /// Scope where variable exists
    pub scope: String,
    /// Node category for visualization
    pub category: NodeCategory,
    /// Ownership status
    pub ownership: OwnershipStatus,
    /// Lifetime information
    pub lifetime: LifetimeInfo,
    /// Visual properties for D3.js
    pub visual: NodeVisual,
    /// Relationship statistics
    pub stats: NodeStats,
}

/// Edge in the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Relationship type
    pub relationship: RelationshipType,
    /// Strength of relationship (0.0 - 1.0)
    pub strength: f64,
    /// Direction of relationship
    pub direction: EdgeDirection,
    /// Visual properties for D3.js
    pub visual: EdgeVisual,
    /// Metadata about the relationship
    pub metadata: EdgeMetadata,
}

/// Graph metadata for D3.js layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Graph density (edges / max_possible_edges)
    pub density: f64,
    /// Clustering coefficient
    pub clustering_coefficient: f64,
    /// Average path length
    pub average_path_length: f64,
    /// Layout configuration
    pub layout: LayoutConfig,
    /// Performance hints
    pub performance: PerformanceHints,
}

/// Node category for visualization grouping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeCategory {
    /// Stack allocated variable
    Stack,
    /// Heap allocated variable
    Heap,
    /// Smart pointer (Box, Rc, Arc)
    SmartPointer,
    /// Reference (&T, &mut T)
    Reference,
    /// Raw pointer (*const T, *mut T)
    RawPointer,
    /// Collection (Vec, HashMap, etc.)
    Collection,
    /// Primitive type
    Primitive,
    /// Custom struct/enum
    Custom,
}

/// Ownership status of a variable
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OwnershipStatus {
    /// Owns the data
    Owner,
    /// Borrowed immutably
    BorrowedImmutable,
    /// Borrowed mutably
    BorrowedMutable,
    /// Shared ownership (Rc/Arc)
    Shared,
    /// Weak reference
    Weak,
    /// Unknown ownership
    Unknown,
}

/// Lifetime information for a variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeInfo {
    /// Start timestamp
    pub start: u64,
    /// End timestamp (if deallocated)
    pub end: Option<u64>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Lifetime category
    pub category: LifetimeCategory,
    /// Is still active
    pub is_active: bool,
}

/// Lifetime category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LifetimeCategory {
    /// Very short lived (<1ms)
    Instant,
    /// Short lived (1ms - 100ms)
    Short,
    /// Medium lived (100ms - 1s)
    Medium,
    /// Long lived (1s - 10s)
    Long,
    /// Very long lived (>10s)
    Persistent,
    /// Still active
    Active,
}

/// Visual properties for D3.js nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVisual {
    /// X coordinate (for fixed positioning)
    pub x: Option<f64>,
    /// Y coordinate (for fixed positioning)
    pub y: Option<f64>,
    /// Node radius
    pub radius: f64,
    /// Color (hex string)
    pub color: String,
    /// Opacity (0.0 - 1.0)
    pub opacity: f64,
    /// CSS class for styling
    pub css_class: String,
    /// Whether node is fixed in position
    pub fixed: bool,
}

/// Node statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    /// Number of incoming relationships
    pub in_degree: usize,
    /// Number of outgoing relationships
    pub out_degree: usize,
    /// Centrality score
    pub centrality: f64,
    /// Clustering coefficient
    pub clustering: f64,
    /// Page rank score
    pub page_rank: f64,
}

/// Type of relationship between variables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Ownership relationship (A owns B)
    Ownership,
    /// Borrowing relationship (A borrows B)
    Borrowing,
    /// Reference relationship (A references B)
    Reference,
    /// Containment relationship (A contains B)
    Containment,
    /// Dependency relationship (A depends on B)
    Dependency,
    /// Shared ownership (A and B share data)
    SharedOwnership,
    /// Temporal relationship (A created before B)
    Temporal,
    /// Memory adjacency (A and B are adjacent in memory)
    MemoryAdjacency,
    /// Type relationship (A and B have same type)
    TypeSimilarity,
}

/// Direction of relationship edge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeDirection {
    /// Directed edge (A -> B)
    Directed,
    /// Undirected edge (A <-> B)
    Undirected,
    /// Bidirectional edge (A <=> B)
    Bidirectional,
}

/// Visual properties for D3.js edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeVisual {
    /// Line width
    pub width: f64,
    /// Color (hex string)
    pub color: String,
    /// Opacity (0.0 - 1.0)
    pub opacity: f64,
    /// Line style (solid, dashed, dotted)
    pub style: String,
    /// CSS class for styling
    pub css_class: String,
    /// Arrow marker (for directed edges)
    pub marker: Option<String>,
}

/// Edge metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// When relationship was established
    pub created_at: u64,
    /// When relationship ended (if applicable)
    pub ended_at: Option<u64>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Source of relationship detection
    pub source: String,
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Layout configuration for D3.js
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Layout algorithm (force, hierarchical, circular)
    pub algorithm: String,
    /// Force simulation parameters
    pub force: ForceConfig,
    /// Hierarchical layout parameters
    pub hierarchical: Option<HierarchicalConfig>,
    /// Viewport dimensions
    pub viewport: ViewportConfig,
}

/// Force simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceConfig {
    /// Link force strength
    pub link_strength: f64,
    /// Charge force strength (repulsion)
    pub charge_strength: f64,
    /// Center force strength
    pub center_strength: f64,
    /// Collision force radius
    pub collision_radius: f64,
    /// Alpha decay rate
    pub alpha_decay: f64,
    /// Velocity decay
    pub velocity_decay: f64,
}

/// Hierarchical layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalConfig {
    /// Direction (top-down, bottom-up, left-right, right-left)
    pub direction: String,
    /// Level separation
    pub level_separation: f64,
    /// Node separation
    pub node_separation: f64,
    /// Tree separation
    pub tree_separation: f64,
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    /// Width in pixels
    pub width: f64,
    /// Height in pixels
    pub height: f64,
    /// Zoom level
    pub zoom: f64,
    /// Pan X offset
    pub pan_x: f64,
    /// Pan Y offset
    pub pan_y: f64,
}

/// Performance hints for large graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHints {
    /// Use canvas rendering for large graphs
    pub use_canvas: bool,
    /// Enable level-of-detail rendering
    pub use_lod: bool,
    /// Maximum nodes to render at once
    pub max_visible_nodes: usize,
    /// Enable clustering for dense areas
    pub enable_clustering: bool,
    /// Clustering threshold
    pub clustering_threshold: f64,
}

/// Summary statistics for relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSummary {
    /// Total variables analyzed
    pub total_variables: usize,
    /// Total relationships found
    pub total_relationships: usize,
    /// Relationship type distribution
    pub relationship_distribution: HashMap<RelationshipType, usize>,
    /// Ownership pattern distribution
    pub ownership_distribution: HashMap<OwnershipStatus, usize>,
    /// Average relationships per variable
    pub average_relationships: f64,
    /// Graph complexity score (0-100)
    pub complexity_score: u32,
    /// Memory efficiency insights
    pub memory_insights: MemoryInsights,
}

/// Detected relationship patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Nodes involved in pattern
    pub nodes: Vec<String>,
    /// Pattern confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Pattern category
    pub category: PatternCategory,
    /// Recommendations based on pattern
    pub recommendations: Vec<String>,
}

/// Category of relationship pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternCategory {
    /// Ownership chain (A owns B owns C)
    OwnershipChain,
    /// Circular reference
    CircularReference,
    /// Hub pattern (one node connected to many)
    Hub,
    /// Cluster pattern (tightly connected group)
    Cluster,
    /// Tree pattern (hierarchical structure)
    Tree,
    /// Memory leak pattern
    MemoryLeak,
    /// Optimization opportunity
    OptimizationOpportunity,
}

/// Memory insights from relationship analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInsights {
    /// Total memory represented in graph
    pub total_memory: usize,
    /// Memory fragmentation score
    pub fragmentation_score: f64,
    /// Sharing efficiency (shared memory / total memory)
    pub sharing_efficiency: f64,
    /// Lifetime distribution
    pub lifetime_distribution: HashMap<LifetimeCategory, usize>,
    /// Optimization suggestions
    pub optimizations: Vec<String>,
}

/// Graph optimization data for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOptimization {
    /// Simplified graph for overview
    pub simplified: Option<RelationshipGraph>,
    /// Clustering information
    pub clusters: Vec<NodeCluster>,
    /// Level-of-detail configurations
    pub lod_levels: Vec<LodLevel>,
    /// Rendering hints
    pub rendering: RenderingHints,
}

/// Node cluster for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCluster {
    /// Cluster ID
    pub id: String,
    /// Nodes in cluster
    pub nodes: Vec<String>,
    /// Cluster center position
    pub center: (f64, f64),
    /// Cluster radius
    pub radius: f64,
    /// Representative node
    pub representative: String,
    /// Cluster statistics
    pub stats: ClusterStats,
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    /// Number of nodes in cluster
    pub node_count: usize,
    /// Internal edge count
    pub internal_edges: usize,
    /// External edge count
    pub external_edges: usize,
    /// Cluster density
    pub density: f64,
    /// Total memory in cluster
    pub total_memory: usize,
}

/// Level-of-detail configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodLevel {
    /// Zoom level threshold
    pub zoom_threshold: f64,
    /// Maximum nodes to show
    pub max_nodes: usize,
    /// Edge simplification factor
    pub edge_simplification: f64,
    /// Label visibility threshold
    pub label_threshold: f64,
}

/// Rendering hints for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingHints {
    /// Use WebGL for rendering
    pub use_webgl: bool,
    /// Enable instanced rendering
    pub use_instancing: bool,
    /// Batch size for rendering
    pub batch_size: usize,
    /// Frame rate target
    pub target_fps: u32,
    /// Memory budget in MB
    pub memory_budget: usize,
}

/// Variable relationship analyzer
pub struct VariableRelationshipAnalyzer {
    /// Nodes being built
    nodes: HashMap<String, GraphNode>,
    /// Edges being built
    edges: Vec<GraphEdge>,
    /// Node ID counter
    node_counter: usize,
    /// Relationship detector
    relationship_detector: RelationshipDetector,
    /// Pattern detector
    pattern_detector: PatternDetector,
    /// Performance optimizer
    optimizer: GraphOptimizer,
}

/// Internal relationship detector
#[derive(Debug, Clone)]
struct RelationshipDetector {
    /// Address to node mapping
    address_map: HashMap<usize, String>,
    /// Type to nodes mapping
    type_map: HashMap<String, Vec<String>>,
    /// Scope to nodes mapping
    scope_map: HashMap<String, Vec<String>>,
    /// Temporal ordering
    temporal_order: Vec<(String, u64)>,
}

/// Internal pattern detector
#[derive(Debug, Clone)]
struct PatternDetector {
    /// Detected patterns
    patterns: Vec<RelationshipPattern>,
}

/// Internal graph optimizer
#[derive(Debug, Clone)]
struct GraphOptimizer {}

impl VariableRelationshipAnalyzer {
    /// Create a new variable relationship analyzer
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            node_counter: 0,
            relationship_detector: RelationshipDetector::new(),
            pattern_detector: PatternDetector::new(),
            optimizer: GraphOptimizer::new(),
        }
    }

    /// Analyze variable relationships from allocation data
    pub fn analyze_allocations(
        allocations: &[AllocationInfo],
    ) -> Result<VariableRelationshipAnalysis, BinaryExportError> {
        let mut analyzer = Self::new();

        // Build nodes from allocations
        for allocation in allocations {
            analyzer.create_node_from_allocation(allocation)?;
        }

        // Detect relationships between nodes
        analyzer.detect_relationships(allocations)?;

        // Detect patterns
        analyzer.detect_patterns()?;

        // Optimize for performance
        analyzer.optimize_graph()?;

        // Generate final analysis
        analyzer.generate_analysis()
    }

    /// Create a graph node from allocation info
    fn create_node_from_allocation(
        &mut self,
        allocation: &AllocationInfo,
    ) -> Result<(), BinaryExportError> {
        let node_id = format!("node_{}", self.node_counter);
        self.node_counter += 1;

        let name = allocation
            .var_name
            .clone()
            .unwrap_or_else(|| format!("alloc_{:x}", allocation.ptr));

        let type_name = allocation
            .type_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let scope = allocation
            .scope_name
            .clone()
            .unwrap_or_else(|| "global".to_string());

        let category = self.determine_node_category(&type_name, allocation);
        let ownership = self.determine_ownership_status(allocation);
        let lifetime = self.create_lifetime_info(allocation);
        let visual = self.create_node_visual(&category, allocation.size);
        let stats = NodeStats {
            in_degree: 0,
            out_degree: 0,
            centrality: 0.0,
            clustering: 0.0,
            page_rank: 0.0,
        };

        let node = GraphNode {
            id: node_id.clone(),
            name,
            address: allocation.ptr,
            size: allocation.size,
            type_name: type_name.clone(),
            scope: scope.clone(),
            category,
            ownership,
            lifetime,
            visual,
            stats,
        };

        // Update relationship detector mappings
        self.relationship_detector
            .address_map
            .insert(allocation.ptr, node_id.clone());
        self.relationship_detector
            .type_map
            .entry(type_name)
            .or_insert_with(Vec::new)
            .push(node_id.clone());
        self.relationship_detector
            .scope_map
            .entry(scope)
            .or_insert_with(Vec::new)
            .push(node_id.clone());
        self.relationship_detector
            .temporal_order
            .push((node_id.clone(), allocation.timestamp_alloc));

        self.nodes.insert(node_id, node);
        Ok(())
    }

    /// Determine node category from type and allocation info
    fn determine_node_category(
        &self,
        type_name: &str,
        allocation: &AllocationInfo,
    ) -> NodeCategory {
        if type_name.starts_with("Box<")
            || type_name.starts_with("Rc<")
            || type_name.starts_with("Arc<")
        {
            NodeCategory::SmartPointer
        } else if type_name.starts_with("&") {
            NodeCategory::Reference
        } else if type_name.starts_with("*const") || type_name.starts_with("*mut") {
            NodeCategory::RawPointer
        } else if type_name.starts_with("Vec<")
            || type_name.starts_with("HashMap<")
            || type_name.starts_with("HashSet<")
        {
            NodeCategory::Collection
        } else if self.is_primitive_type(type_name) {
            NodeCategory::Primitive
        } else if allocation.size < 1024 {
            // Small allocations are likely stack-allocated
            NodeCategory::Stack
        } else {
            // Larger allocations are likely heap-allocated
            NodeCategory::Heap
        }
    }

    /// Check if type is primitive
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "()"
        )
    }

    /// Determine ownership status
    fn determine_ownership_status(&self, allocation: &AllocationInfo) -> OwnershipStatus {
        if let Some(ref type_name) = allocation.type_name {
            if type_name.starts_with("&mut") {
                OwnershipStatus::BorrowedMutable
            } else if type_name.starts_with("&") {
                OwnershipStatus::BorrowedImmutable
            } else if type_name.starts_with("Rc<") || type_name.starts_with("Arc<") {
                OwnershipStatus::Shared
            } else if type_name.contains("Weak<") {
                OwnershipStatus::Weak
            } else {
                OwnershipStatus::Owner
            }
        } else {
            OwnershipStatus::Unknown
        }
    }

    /// Create lifetime information
    fn create_lifetime_info(&self, allocation: &AllocationInfo) -> LifetimeInfo {
        let start = allocation.timestamp_alloc;
        let end = allocation.timestamp_dealloc;
        let duration_ms = allocation.lifetime_ms;

        let category = if let Some(duration) = duration_ms {
            if duration < 1 {
                LifetimeCategory::Instant
            } else if duration < 100 {
                LifetimeCategory::Short
            } else if duration < 1000 {
                LifetimeCategory::Medium
            } else if duration < 10000 {
                LifetimeCategory::Long
            } else {
                LifetimeCategory::Persistent
            }
        } else {
            LifetimeCategory::Active
        };

        LifetimeInfo {
            start,
            end,
            duration_ms,
            category,
            is_active: end.is_none(),
        }
    }

    /// Create visual properties for node
    fn create_node_visual(&self, category: &NodeCategory, size: usize) -> NodeVisual {
        let radius = (size as f64).log10().max(3.0).min(20.0);

        let color = match category {
            NodeCategory::Stack => "#4CAF50".to_string(), // Green
            NodeCategory::Heap => "#2196F3".to_string(),  // Blue
            NodeCategory::SmartPointer => "#FF9800".to_string(), // Orange
            NodeCategory::Reference => "#9C27B0".to_string(), // Purple
            NodeCategory::RawPointer => "#F44336".to_string(), // Red
            NodeCategory::Collection => "#00BCD4".to_string(), // Cyan
            NodeCategory::Primitive => "#8BC34A".to_string(), // Light Green
            NodeCategory::Custom => "#607D8B".to_string(), // Blue Grey
        };

        let css_class = format!("node-{:?}", category).to_lowercase();

        NodeVisual {
            x: None,
            y: None,
            radius,
            color,
            opacity: 0.8,
            css_class,
            fixed: false,
        }
    }

    /// Detect relationships between nodes
    fn detect_relationships(
        &mut self,
        _allocations: &[AllocationInfo],
    ) -> Result<(), BinaryExportError> {
        // Sort by temporal order for temporal relationships
        self.relationship_detector
            .temporal_order
            .sort_by_key(|(_, timestamp)| *timestamp);

        // Detect various types of relationships
        self.detect_ownership_relationships()?;
        self.detect_type_relationships()?;
        self.detect_scope_relationships()?;
        self.detect_memory_adjacency_relationships()?;
        self.detect_temporal_relationships()?;

        Ok(())
    }

    /// Detect ownership relationships
    fn detect_ownership_relationships(&mut self) -> Result<(), BinaryExportError> {
        // Look for smart pointer relationships
        for (node_id, node) in &self.nodes {
            if matches!(node.category, NodeCategory::SmartPointer) {
                // Find potential owned data
                for (other_id, other_node) in &self.nodes {
                    if node_id != other_id && self.could_be_owned_by(node, other_node) {
                        let edge = self.create_edge(
                            node_id.clone(),
                            other_id.clone(),
                            RelationshipType::Ownership,
                            0.8,
                            EdgeDirection::Directed,
                        );
                        self.edges.push(edge);
                    }
                }
            }
        }
        Ok(())
    }

    /// Detect type relationships
    fn detect_type_relationships(&mut self) -> Result<(), BinaryExportError> {
        for (_type_name, node_ids) in &self.relationship_detector.type_map {
            if node_ids.len() > 1 {
                // Create type similarity relationships
                for i in 0..node_ids.len() {
                    for j in i + 1..node_ids.len() {
                        let edge = self.create_edge(
                            node_ids[i].clone(),
                            node_ids[j].clone(),
                            RelationshipType::TypeSimilarity,
                            0.5,
                            EdgeDirection::Undirected,
                        );
                        self.edges.push(edge);
                    }
                }
            }
        }
        Ok(())
    }

    /// Detect scope relationships
    fn detect_scope_relationships(&mut self) -> Result<(), BinaryExportError> {
        for (_scope_name, node_ids) in &self.relationship_detector.scope_map {
            if node_ids.len() > 1 {
                // Create containment relationships within same scope
                for i in 0..node_ids.len() {
                    for j in i + 1..node_ids.len() {
                        let edge = self.create_edge(
                            node_ids[i].clone(),
                            node_ids[j].clone(),
                            RelationshipType::Containment,
                            0.3,
                            EdgeDirection::Undirected,
                        );
                        self.edges.push(edge);
                    }
                }
            }
        }
        Ok(())
    }

    /// Detect memory adjacency relationships
    fn detect_memory_adjacency_relationships(&mut self) -> Result<(), BinaryExportError> {
        let mut addresses: Vec<(usize, String)> = self
            .nodes
            .iter()
            .map(|(id, node)| (node.address, id.clone()))
            .collect();
        addresses.sort_by_key(|(addr, _)| *addr);

        for i in 0..addresses.len().saturating_sub(1) {
            let (addr1, id1) = &addresses[i];
            let (addr2, id2) = &addresses[i + 1];

            let node1 = &self.nodes[id1];
            let _node2 = &self.nodes[id2];

            // Check if allocations are adjacent in memory
            if addr1 + node1.size == *addr2 {
                let edge = self.create_edge(
                    id1.clone(),
                    id2.clone(),
                    RelationshipType::MemoryAdjacency,
                    0.6,
                    EdgeDirection::Undirected,
                );
                self.edges.push(edge);
            }
        }
        Ok(())
    }

    /// Detect temporal relationships
    fn detect_temporal_relationships(&mut self) -> Result<(), BinaryExportError> {
        for i in 0..self
            .relationship_detector
            .temporal_order
            .len()
            .saturating_sub(1)
        {
            let (id1, _) = &self.relationship_detector.temporal_order[i];
            let (id2, _) = &self.relationship_detector.temporal_order[i + 1];

            // Create temporal relationship for consecutive allocations
            let edge = self.create_edge(
                id1.clone(),
                id2.clone(),
                RelationshipType::Temporal,
                0.2,
                EdgeDirection::Directed,
            );
            self.edges.push(edge);
        }
        Ok(())
    }

    /// Check if one node could be owned by another
    fn could_be_owned_by(&self, owner: &GraphNode, owned: &GraphNode) -> bool {
        // Simple heuristic: smart pointers could own heap allocations
        matches!(owner.category, NodeCategory::SmartPointer)
            && matches!(owned.category, NodeCategory::Heap | NodeCategory::Custom)
            && owner.scope == owned.scope
    }

    /// Create an edge between two nodes
    fn create_edge(
        &self,
        source: String,
        target: String,
        relationship: RelationshipType,
        strength: f64,
        direction: EdgeDirection,
    ) -> GraphEdge {
        let visual = self.create_edge_visual(&relationship, strength);
        let metadata = EdgeMetadata {
            created_at: 0, // Would be set based on analysis context
            ended_at: None,
            confidence: strength,
            source: "relationship_analyzer".to_string(),
            properties: HashMap::new(),
        };

        GraphEdge {
            source,
            target,
            relationship,
            strength,
            direction,
            visual,
            metadata,
        }
    }

    /// Create visual properties for edge
    fn create_edge_visual(&self, relationship: &RelationshipType, strength: f64) -> EdgeVisual {
        let width = (strength * 3.0).max(1.0);

        let color = match relationship {
            RelationshipType::Ownership => "#FF5722".to_string(), // Deep Orange
            RelationshipType::Borrowing => "#3F51B5".to_string(), // Indigo
            RelationshipType::Reference => "#9C27B0".to_string(), // Purple
            RelationshipType::Containment => "#4CAF50".to_string(), // Green
            RelationshipType::Dependency => "#FF9800".to_string(), // Orange
            RelationshipType::SharedOwnership => "#E91E63".to_string(), // Pink
            RelationshipType::Temporal => "#607D8B".to_string(),  // Blue Grey
            RelationshipType::MemoryAdjacency => "#00BCD4".to_string(), // Cyan
            RelationshipType::TypeSimilarity => "#795548".to_string(), // Brown
        };

        let style = match relationship {
            RelationshipType::Temporal => "dashed".to_string(),
            RelationshipType::TypeSimilarity => "dotted".to_string(),
            _ => "solid".to_string(),
        };

        EdgeVisual {
            width,
            color,
            opacity: (strength * 0.8).max(0.3),
            style,
            css_class: format!("edge-{:?}", relationship).to_lowercase(),
            marker: Some("arrow".to_string()),
        }
    }

    /// Detect patterns in the graph
    fn detect_patterns(&mut self) -> Result<(), BinaryExportError> {
        self.pattern_detector
            .detect_ownership_chains(&self.nodes, &self.edges)?;
        self.pattern_detector
            .detect_circular_references(&self.nodes, &self.edges)?;
        self.pattern_detector
            .detect_hub_patterns(&self.nodes, &self.edges)?;
        self.pattern_detector
            .detect_memory_leaks(&self.nodes, &self.edges)?;
        Ok(())
    }

    /// Optimize graph for performance
    fn optimize_graph(&mut self) -> Result<(), BinaryExportError> {
        self.optimizer
            .create_clusters(&mut self.nodes, &self.edges)?;
        self.optimizer
            .create_simplified_graph(&self.nodes, &self.edges)?;
        self.optimizer.create_lod_levels(&self.nodes, &self.edges)?;
        Ok(())
    }

    /// Generate the final analysis
    fn generate_analysis(&self) -> Result<VariableRelationshipAnalysis, BinaryExportError> {
        let nodes: Vec<GraphNode> = self.nodes.values().cloned().collect();
        let links = self.edges.clone();

        let metadata = self.create_graph_metadata(&nodes, &links);

        let graph = RelationshipGraph {
            nodes,
            links,
            metadata,
        };

        let summary = self.create_relationship_summary(&graph)?;
        let patterns = self.pattern_detector.patterns.clone();
        let optimization = self.optimizer.create_optimization_data(&graph)?;

        Ok(VariableRelationshipAnalysis {
            graph,
            summary,
            patterns,
            optimization,
        })
    }

    /// Create graph metadata
    fn create_graph_metadata(&self, nodes: &[GraphNode], edges: &[GraphEdge]) -> GraphMetadata {
        let node_count = nodes.len();
        let edge_count = edges.len();
        let max_edges = if node_count > 1 {
            node_count * (node_count - 1) / 2
        } else {
            1 // Avoid division by zero
        };
        let density = if max_edges > 0 {
            (edge_count as f64 / max_edges as f64).min(1.0) // Cap at 1.0
        } else {
            0.0
        };

        let layout = LayoutConfig {
            algorithm: "force".to_string(),
            force: ForceConfig {
                link_strength: 0.1,
                charge_strength: -300.0,
                center_strength: 0.1,
                collision_radius: 5.0,
                alpha_decay: 0.0228,
                velocity_decay: 0.4,
            },
            hierarchical: None,
            viewport: ViewportConfig {
                width: 800.0,
                height: 600.0,
                zoom: 1.0,
                pan_x: 0.0,
                pan_y: 0.0,
            },
        };

        let performance = PerformanceHints {
            use_canvas: node_count > 1000,
            use_lod: node_count > 500,
            max_visible_nodes: 1000,
            enable_clustering: node_count > 200,
            clustering_threshold: 0.7,
        };

        GraphMetadata {
            node_count,
            edge_count,
            density,
            clustering_coefficient: 0.0, // Would be calculated in real implementation
            average_path_length: 0.0,    // Would be calculated in real implementation
            layout,
            performance,
        }
    }

    /// Create relationship summary
    fn create_relationship_summary(
        &self,
        graph: &RelationshipGraph,
    ) -> Result<RelationshipSummary, BinaryExportError> {
        let total_variables = graph.nodes.len();
        let total_relationships = graph.links.len();

        let mut relationship_distribution = HashMap::new();
        let mut ownership_distribution = HashMap::new();

        for edge in &graph.links {
            *relationship_distribution
                .entry(edge.relationship.clone())
                .or_insert(0) += 1;
        }

        for node in &graph.nodes {
            *ownership_distribution
                .entry(node.ownership.clone())
                .or_insert(0) += 1;
        }

        let average_relationships = if total_variables > 0 {
            total_relationships as f64 / total_variables as f64
        } else {
            0.0
        };

        let complexity_score = self.calculate_complexity_score(graph);
        let memory_insights = self.create_memory_insights(graph);

        Ok(RelationshipSummary {
            total_variables,
            total_relationships,
            relationship_distribution,
            ownership_distribution,
            average_relationships,
            complexity_score,
            memory_insights,
        })
    }

    /// Calculate graph complexity score
    fn calculate_complexity_score(&self, graph: &RelationshipGraph) -> u32 {
        let base_score = (graph.metadata.density * 50.0) as u32;
        let edge_penalty = (graph.links.len() / 10).min(30) as u32;
        let node_penalty = (graph.nodes.len() / 50).min(20) as u32;

        (base_score + edge_penalty + node_penalty).min(100)
    }

    /// Create memory insights
    fn create_memory_insights(&self, graph: &RelationshipGraph) -> MemoryInsights {
        let total_memory: usize = graph.nodes.iter().map(|n| n.size).sum();

        let mut lifetime_distribution = HashMap::new();
        for node in &graph.nodes {
            *lifetime_distribution
                .entry(node.lifetime.category.clone())
                .or_insert(0) += 1;
        }

        MemoryInsights {
            total_memory,
            fragmentation_score: 0.5, // Simplified
            sharing_efficiency: 0.7,  // Simplified
            lifetime_distribution,
            optimizations: vec![
                "Consider using Rc/Arc for shared data".to_string(),
                "Review long-lived allocations".to_string(),
            ],
        }
    }
}

impl RelationshipDetector {
    fn new() -> Self {
        Self {
            address_map: HashMap::new(),
            type_map: HashMap::new(),
            scope_map: HashMap::new(),
            temporal_order: Vec::new(),
        }
    }
}

impl PatternDetector {
    fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    fn detect_ownership_chains(
        &mut self,
        _nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would detect chains of ownership relationships
        Ok(())
    }

    fn detect_circular_references(
        &mut self,
        _nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would detect circular reference patterns
        Ok(())
    }

    fn detect_hub_patterns(
        &mut self,
        _nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would detect hub patterns (nodes with many connections)
        Ok(())
    }

    fn detect_memory_leaks(
        &mut self,
        nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Detect potential memory leaks
        let leaked_nodes: Vec<String> = nodes
            .iter()
            .filter(|(_, node)| {
                !node.lifetime.is_active && node.lifetime.category == LifetimeCategory::Persistent
            })
            .map(|(id, _)| id.clone())
            .collect();

        if !leaked_nodes.is_empty() {
            let pattern = RelationshipPattern {
                name: "Potential Memory Leak".to_string(),
                description: "Long-lived allocations that may indicate memory leaks".to_string(),
                nodes: leaked_nodes,
                confidence: 0.6,
                category: PatternCategory::MemoryLeak,
                recommendations: vec![
                    "Review allocation lifetimes".to_string(),
                    "Consider using RAII patterns".to_string(),
                ],
            };
            self.patterns.push(pattern);
        }

        Ok(())
    }
}

impl GraphOptimizer {
    fn new() -> Self {
        Self {}
    }

    fn create_clusters(
        &mut self,
        _nodes: &mut HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would create node clusters for performance
        Ok(())
    }

    fn create_simplified_graph(
        &mut self,
        _nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would create simplified version of graph
        Ok(())
    }

    fn create_lod_levels(
        &mut self,
        _nodes: &HashMap<String, GraphNode>,
        _edges: &[GraphEdge],
    ) -> Result<(), BinaryExportError> {
        // Implementation would create level-of-detail configurations
        Ok(())
    }

    fn create_optimization_data(
        &self,
        _graph: &RelationshipGraph,
    ) -> Result<GraphOptimization, BinaryExportError> {
        Ok(GraphOptimization {
            simplified: None,
            clusters: Vec::new(),
            lod_levels: vec![
                LodLevel {
                    zoom_threshold: 0.5,
                    max_nodes: 100,
                    edge_simplification: 0.8,
                    label_threshold: 0.3,
                },
                LodLevel {
                    zoom_threshold: 1.0,
                    max_nodes: 500,
                    edge_simplification: 0.5,
                    label_threshold: 0.6,
                },
            ],
            rendering: RenderingHints {
                use_webgl: true,
                use_instancing: true,
                batch_size: 1000,
                target_fps: 60,
                memory_budget: 256,
            },
        })
    }
}

impl Default for VariableRelationshipAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_allocation(
        ptr: usize,
        size: usize,
        var_name: Option<&str>,
        type_name: Option<&str>,
        scope_name: Option<&str>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name: var_name.map(|s| s.to_string()),
            type_name: type_name.map(|s| s.to_string()),
            scope_name: scope_name.map(|s| s.to_string()),
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "main".to_string(),
            borrow_count: 0,
            stack_trace: None,
            is_leaked: false,
            lifetime_ms: Some(100),
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
    fn test_node_category_determination() {
        let analyzer = VariableRelationshipAnalyzer::new();
        let allocation =
            create_test_allocation(0x1000, 1024, Some("test"), Some("Vec<u8>"), Some("main"));

        let category = analyzer.determine_node_category("Vec<u8>", &allocation);
        assert_eq!(category, NodeCategory::Collection);

        let category = analyzer.determine_node_category("Box<i32>", &allocation);
        assert_eq!(category, NodeCategory::SmartPointer);

        let category = analyzer.determine_node_category("&str", &allocation);
        assert_eq!(category, NodeCategory::Reference);

        let category = analyzer.determine_node_category("*mut u8", &allocation);
        assert_eq!(category, NodeCategory::RawPointer);
    }

    #[test]
    fn test_ownership_status_determination() {
        let analyzer = VariableRelationshipAnalyzer::new();

        let allocation =
            create_test_allocation(0x1000, 1024, Some("test"), Some("&mut i32"), Some("main"));
        let ownership = analyzer.determine_ownership_status(&allocation);
        assert_eq!(ownership, OwnershipStatus::BorrowedMutable);

        let allocation =
            create_test_allocation(0x1000, 1024, Some("test"), Some("&i32"), Some("main"));
        let ownership = analyzer.determine_ownership_status(&allocation);
        assert_eq!(ownership, OwnershipStatus::BorrowedImmutable);

        let allocation =
            create_test_allocation(0x1000, 1024, Some("test"), Some("Rc<i32>"), Some("main"));
        let ownership = analyzer.determine_ownership_status(&allocation);
        assert_eq!(ownership, OwnershipStatus::Shared);
    }

    #[test]
    fn test_lifetime_category_determination() {
        let analyzer = VariableRelationshipAnalyzer::new();

        let mut allocation =
            create_test_allocation(0x1000, 1024, Some("test"), Some("i32"), Some("main"));
        allocation.lifetime_ms = Some(0);
        let lifetime = analyzer.create_lifetime_info(&allocation);
        assert_eq!(lifetime.category, LifetimeCategory::Instant);

        allocation.lifetime_ms = Some(50);
        let lifetime = analyzer.create_lifetime_info(&allocation);
        assert_eq!(lifetime.category, LifetimeCategory::Short);

        allocation.lifetime_ms = Some(500);
        let lifetime = analyzer.create_lifetime_info(&allocation);
        assert_eq!(lifetime.category, LifetimeCategory::Medium);

        allocation.lifetime_ms = None;
        let lifetime = analyzer.create_lifetime_info(&allocation);
        assert_eq!(lifetime.category, LifetimeCategory::Active);
    }

    #[test]
    fn test_variable_relationship_analysis() {
        let allocations = vec![
            create_test_allocation(0x1000, 1024, Some("vec1"), Some("Vec<u8>"), Some("main")),
            create_test_allocation(0x2000, 2048, Some("box1"), Some("Box<i32>"), Some("main")),
            create_test_allocation(0x3000, 512, Some("ref1"), Some("&str"), Some("main")),
            create_test_allocation(0x4000, 256, Some("int1"), Some("i32"), Some("helper")),
        ];

        let analysis = VariableRelationshipAnalyzer::analyze_allocations(&allocations)
            .expect("Failed to get test value");

        assert_eq!(analysis.graph.nodes.len(), 4);
        assert_eq!(analysis.summary.total_variables, 4);
        assert!(analysis.summary.total_relationships > 0);
        assert!(!analysis.summary.relationship_distribution.is_empty());
        assert!(!analysis.summary.ownership_distribution.is_empty());
    }

    #[test]
    fn test_primitive_type_detection() {
        let analyzer = VariableRelationshipAnalyzer::new();

        assert!(analyzer.is_primitive_type("i32"));
        assert!(analyzer.is_primitive_type("f64"));
        assert!(analyzer.is_primitive_type("bool"));
        assert!(analyzer.is_primitive_type("char"));
        assert!(!analyzer.is_primitive_type("String"));
        assert!(!analyzer.is_primitive_type("Vec<u8>"));
    }

    #[test]
    fn test_graph_metadata_creation() {
        let nodes = vec![GraphNode {
            id: "node1".to_string(),
            name: "test1".to_string(),
            address: 0x1000,
            size: 1024,
            type_name: "i32".to_string(),
            scope: "main".to_string(),
            category: NodeCategory::Primitive,
            ownership: OwnershipStatus::Owner,
            lifetime: LifetimeInfo {
                start: 1000,
                end: None,
                duration_ms: None,
                category: LifetimeCategory::Active,
                is_active: true,
            },
            visual: NodeVisual {
                x: None,
                y: None,
                radius: 5.0,
                color: "#4CAF50".to_string(),
                opacity: 0.8,
                css_class: "node-primitive".to_string(),
                fixed: false,
            },
            stats: NodeStats {
                in_degree: 0,
                out_degree: 1,
                centrality: 0.5,
                clustering: 0.0,
                page_rank: 0.25,
            },
        }];

        let edges = vec![];
        let analyzer = VariableRelationshipAnalyzer::new();
        let metadata = analyzer.create_graph_metadata(&nodes, &edges);

        assert_eq!(metadata.node_count, 1);
        assert_eq!(metadata.edge_count, 0);
        assert_eq!(metadata.density, 0.0);
        assert_eq!(metadata.layout.algorithm, "force");
    }

    #[test]
    fn test_edge_creation() {
        let analyzer = VariableRelationshipAnalyzer::new();

        let edge = analyzer.create_edge(
            "node1".to_string(),
            "node2".to_string(),
            RelationshipType::Ownership,
            0.8,
            EdgeDirection::Directed,
        );

        assert_eq!(edge.source, "node1");
        assert_eq!(edge.target, "node2");
        assert_eq!(edge.relationship, RelationshipType::Ownership);
        assert_eq!(edge.strength, 0.8);
        assert_eq!(edge.direction, EdgeDirection::Directed);
    }

    #[test]
    fn test_memory_insights_creation() {
        let graph = RelationshipGraph {
            nodes: vec![GraphNode {
                id: "node1".to_string(),
                name: "test1".to_string(),
                address: 0x1000,
                size: 1024,
                type_name: "i32".to_string(),
                scope: "main".to_string(),
                category: NodeCategory::Primitive,
                ownership: OwnershipStatus::Owner,
                lifetime: LifetimeInfo {
                    start: 1000,
                    end: None,
                    duration_ms: Some(100),
                    category: LifetimeCategory::Short,
                    is_active: false,
                },
                visual: NodeVisual {
                    x: None,
                    y: None,
                    radius: 5.0,
                    color: "#4CAF50".to_string(),
                    opacity: 0.8,
                    css_class: "node-primitive".to_string(),
                    fixed: false,
                },
                stats: NodeStats {
                    in_degree: 0,
                    out_degree: 0,
                    centrality: 0.0,
                    clustering: 0.0,
                    page_rank: 0.0,
                },
            }],
            links: vec![],
            metadata: GraphMetadata {
                node_count: 1,
                edge_count: 0,
                density: 0.0,
                clustering_coefficient: 0.0,
                average_path_length: 0.0,
                layout: LayoutConfig {
                    algorithm: "force".to_string(),
                    force: ForceConfig {
                        link_strength: 0.1,
                        charge_strength: -300.0,
                        center_strength: 0.1,
                        collision_radius: 5.0,
                        alpha_decay: 0.0228,
                        velocity_decay: 0.4,
                    },
                    hierarchical: None,
                    viewport: ViewportConfig {
                        width: 800.0,
                        height: 600.0,
                        zoom: 1.0,
                        pan_x: 0.0,
                        pan_y: 0.0,
                    },
                },
                performance: PerformanceHints {
                    use_canvas: false,
                    use_lod: false,
                    max_visible_nodes: 1000,
                    enable_clustering: false,
                    clustering_threshold: 0.7,
                },
            },
        };

        let analyzer = VariableRelationshipAnalyzer::new();
        let insights = analyzer.create_memory_insights(&graph);

        assert_eq!(insights.total_memory, 1024);
        assert!(!insights.lifetime_distribution.is_empty());
        assert!(!insights.optimizations.is_empty());
    }
}
