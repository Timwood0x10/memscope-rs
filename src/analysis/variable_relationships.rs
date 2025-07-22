//! Variable Relationship Analysis
//!
//! This module provides functionality to detect and analyze relationships between variables,
//! building a comprehensive graph for visualization and analysis.

use crate::core::types::{AllocationInfo, TrackingResult};
use crate::{variable_registry::VariableInfo, CircularReferenceNode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of relationships between variables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// One variable references another (through pointers)
    References,
    /// One variable owns another's memory (Box, unique ownership)
    Owns,
    /// Variables share the same data (Rc/Arc clones)
    Clones,
    /// Variables are in the same scope (containment relationship)
    Contains,
    /// One variable depends on another for its existence
    DependsOn,
}

/// A node in the variable relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNode {
    /// Unique identifier (usually memory address as string)
    pub id: String,
    /// Variable name (user-defined or inferred)
    pub name: String,
    /// Type name
    pub type_name: String,
    /// Memory size in bytes
    pub size: usize,
    /// Scope name
    pub scope: String,
    /// Whether the variable is still active
    pub is_active: bool,
    /// Category of the variable
    pub category: VariableCategory,
    /// Smart pointer information if applicable
    pub smart_pointer_info: Option<SmartPointerInfo>,
    /// Timestamp when variable was created
    pub created_at: u64,
    /// Timestamp when variable was destroyed (if applicable)
    pub destroyed_at: Option<u64>,
}

/// Category of variable for visualization purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariableCategory {
    /// User-defined variable tracked explicitly
    UserVariable,
    /// System allocation inferred automatically
    SystemAllocation,
    /// Smart pointer (Rc, Arc, Box)
    SmartPointer,
    /// Collection (Vec, HashMap, etc.)
    Collection,
}

/// Smart pointer specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartPointerInfo {
    /// Type of smart pointer (Rc, Arc, Box)
    pub pointer_type: String,
    /// Reference count (for Rc/Arc)
    pub ref_count: Option<usize>,
    /// Address of the data being pointed to
    pub data_ptr: Option<usize>,
    /// List of clone addresses (for Rc/Arc)
    pub clones: Vec<usize>,
    /// Address this was cloned from
    pub cloned_from: Option<usize>,
}

/// A relationship edge between two variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRelationship {
    /// Source variable ID
    pub source: String,
    /// Target variable ID
    pub target: String,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Strength/weight of the relationship (0.0 to 1.0)
    pub weight: f64,
    /// Additional metadata about the relationship
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A cluster of related variables (e.g., same scope, same type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableCluster {
    /// Unique cluster ID
    pub id: String,
    /// Type of cluster (scope, type, etc.)
    pub cluster_type: ClusterType,
    /// Variables in this cluster
    pub variables: Vec<String>,
    /// Suggested layout position for visualization
    pub layout_hint: Option<LayoutHint>,
    /// Cluster metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of variable clusters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    /// Variables in the same scope
    Scope,
    /// Variables of the same type
    Type,
    /// Variables with similar lifetimes
    Lifetime,
    /// Variables in a smart pointer relationship
    SmartPointerGroup,
}

/// Layout hint for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutHint {
    /// Suggested X coordinate
    pub x: f64,
    /// Suggested Y coordinate
    pub y: f64,
    /// Suggested width
    pub width: Option<f64>,
    /// Suggested height
    pub height: Option<f64>,
}

/// Complete variable relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRelationshipGraph {
    /// All nodes in the graph
    pub nodes: Vec<VariableNode>,
    /// All relationships between nodes
    pub relationships: Vec<VariableRelationship>,
    /// Clusters of related variables
    pub clusters: Vec<VariableCluster>,
    /// Graph statistics
    pub statistics: GraphStatistics,
    /// Metadata about the graph
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Statistics about the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Total number of relationships
    pub total_relationships: usize,
    /// Number of circular references detected
    pub circular_references: usize,
    /// Size of the largest cluster
    pub largest_cluster_size: usize,
    /// Number of isolated nodes (no relationships)
    pub isolated_nodes: usize,
    /// Average relationships per node
    pub avg_relationships_per_node: f64,
}

/// Builder for constructing variable relationship graphs
pub struct VariableRelationshipBuilder {
    nodes: HashMap<String, VariableNode>,
    relationships: Vec<VariableRelationship>,
    clusters: Vec<VariableCluster>,
}

impl VariableRelationshipBuilder {
    /// Create a new relationship builder
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relationships: Vec::new(),
            clusters: Vec::new(),
        }
    }

    /// Add allocations to the graph
    pub fn add_allocations(
        mut self,
        allocations: &[AllocationInfo],
        registry: &HashMap<usize, VariableInfo>,
    ) -> Self {
        for alloc in allocations {
            let node = self.create_node_from_allocation(alloc, registry);
            self.nodes.insert(node.id.clone(), node);
        }
        self
    }

    /// Detect reference relationships between variables
    pub fn detect_references(mut self) -> Self {
        // Look for smart pointer relationships
        let nodes: Vec<_> = self.nodes.values().cloned().collect();

        for node in &nodes {
            if let Some(smart_ptr_info) = &node.smart_pointer_info {
                // Create relationships for smart pointer clones
                for &clone_addr in &smart_ptr_info.clones {
                    let clone_id = format!("0x{:x}", clone_addr);
                    if self.nodes.contains_key(&clone_id) {
                        self.relationships.push(VariableRelationship {
                            source: node.id.clone(),
                            target: clone_id,
                            relationship_type: RelationshipType::Clones,
                            weight: 1.0,
                            metadata: HashMap::new(),
                        });
                    }
                }

                // Create relationship to cloned_from
                if let Some(cloned_from_addr) = smart_ptr_info.cloned_from {
                    let parent_id = format!("0x{:x}", cloned_from_addr);
                    if self.nodes.contains_key(&parent_id) {
                        self.relationships.push(VariableRelationship {
                            source: parent_id,
                            target: node.id.clone(),
                            relationship_type: RelationshipType::Clones,
                            weight: 1.0,
                            metadata: HashMap::new(),
                        });
                    }
                }

                // Create ownership relationship for Box pointers
                if smart_ptr_info.pointer_type == "Box" {
                    if let Some(data_ptr) = smart_ptr_info.data_ptr {
                        let data_id = format!("0x{:x}", data_ptr);
                        if self.nodes.contains_key(&data_id) {
                            self.relationships.push(VariableRelationship {
                                source: node.id.clone(),
                                target: data_id,
                                relationship_type: RelationshipType::Owns,
                                weight: 1.0,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        self
    }

    /// Detect scope-based relationships
    pub fn detect_scope_relationships(mut self) -> Self {
        // Group variables by scope
        let mut scope_groups: HashMap<String, Vec<String>> = HashMap::new();

        for node in self.nodes.values() {
            scope_groups
                .entry(node.scope.clone())
                .or_default()
                .push(node.id.clone());
        }

        // Create scope clusters and containment relationships
        for (scope_name, variable_ids) in scope_groups {
            if variable_ids.len() > 1 {
                // Create cluster
                self.clusters.push(VariableCluster {
                    id: format!("scope_{}", scope_name),
                    cluster_type: ClusterType::Scope,
                    variables: variable_ids.clone(),
                    layout_hint: None,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert(
                            "scope_name".to_string(),
                            serde_json::Value::String(scope_name.clone()),
                        );
                        meta
                    },
                });

                // Create containment relationships within scope
                for i in 0..variable_ids.len() {
                    for j in (i + 1)..variable_ids.len() {
                        self.relationships.push(VariableRelationship {
                            source: variable_ids[i].clone(),
                            target: variable_ids[j].clone(),
                            relationship_type: RelationshipType::Contains,
                            weight: 0.3, // Lower weight for scope relationships
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert(
                                    "scope".to_string(),
                                    serde_json::Value::String(scope_name.clone()),
                                );
                                meta
                            },
                        });
                    }
                }
            }
        }

        self
    }

    /// Detect circular references
    pub fn detect_circular_references(mut self) -> Self {
        // Use existing circular reference detection logic
        let allocations: Vec<AllocationInfo> = self
            .nodes
            .values()
            .filter_map(|node| self.node_to_allocation_info(node))
            .collect();

        let circular_analysis =
            crate::analysis::circular_reference::detect_circular_references(&allocations);

        // Add circular reference relationships
        for cycle in &circular_analysis.circular_references {
            for window in cycle.cycle_path.windows(2) {
                if let (Some(source), Some(target)) = (window.get(0), window.get(1)) {
                    let source_id = format!("0x{:p}", source as *const CircularReferenceNode);
                    let target_id = format!("0x{:p}", target as *const CircularReferenceNode);

                    if self.nodes.contains_key(&source_id) && self.nodes.contains_key(&target_id) {
                        self.relationships.push(VariableRelationship {
                            source: source_id,
                            target: target_id,
                            relationship_type: RelationshipType::DependsOn,
                            weight: 0.8, // High weight for circular dependencies
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert(
                                    "circular_reference".to_string(),
                                    serde_json::Value::Bool(true),
                                );
                                meta.insert(
                                    "cycle_id".to_string(),
                                    serde_json::Value::String(format!(
                                        "cycle_{}",
                                        cycle.cycle_path.len()
                                    )),
                                );
                                meta
                            },
                        });
                    }
                }
            }
        }

        self
    }

    /// Build the final relationship graph
    pub fn build_graph(self) -> VariableRelationshipGraph {
        let statistics = self.calculate_statistics();

        VariableRelationshipGraph {
            nodes: self.nodes.into_values().collect(),
            relationships: self.relationships,
            clusters: self.clusters,
            statistics,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "build_timestamp".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    )),
                );
                meta
            },
        }
    }

    /// Create a node from allocation info
    fn create_node_from_allocation(
        &self,
        alloc: &AllocationInfo,
        registry: &HashMap<usize, VariableInfo>,
    ) -> VariableNode {
        let id = format!("0x{:x}", alloc.ptr);

        // Get variable info from registry or infer
        let (name, type_name, category) = if let Some(var_info) = registry.get(&alloc.ptr) {
            (
                var_info.var_name.clone(),
                var_info.type_name.clone(),
                VariableCategory::UserVariable,
            )
        } else if let (Some(var_name), Some(type_name)) = (&alloc.var_name, &alloc.type_name) {
            (
                var_name.clone(),
                type_name.clone(),
                VariableCategory::UserVariable,
            )
        } else {
            let (inferred_name, inferred_type) =
                crate::variable_registry::VariableRegistry::infer_allocation_info_cached(alloc);
            let category = if inferred_type.contains("Vec") || inferred_type.contains("HashMap") {
                VariableCategory::Collection
            } else if inferred_type.contains("Box")
                || inferred_type.contains("Rc")
                || inferred_type.contains("Arc")
            {
                VariableCategory::SmartPointer
            } else {
                VariableCategory::SystemAllocation
            };
            (inferred_name, inferred_type, category)
        };

        let scope = alloc
            .scope_name
            .as_deref()
            .unwrap_or_else(|| {
                if category == VariableCategory::UserVariable {
                    "main"
                } else {
                    "system"
                }
            })
            .to_string();

        VariableNode {
            id,
            name,
            type_name,
            size: alloc.size,
            scope,
            is_active: alloc.timestamp_dealloc.is_none(),
            category,
            smart_pointer_info: alloc
                .smart_pointer_info
                .as_ref()
                .map(|info| SmartPointerInfo {
                    pointer_type: format!("{:?}", info.pointer_type),
                    ref_count: Some(
                        info.ref_count_history
                            .last()
                            .map(|s| s.strong_count)
                            .unwrap_or(0),
                    ),
                    data_ptr: Some(info.data_ptr),
                    clones: info.clones.clone(),
                    cloned_from: info.cloned_from,
                }),
            created_at: alloc.timestamp_alloc,
            destroyed_at: alloc.timestamp_dealloc,
        }
    }

    /// Convert node back to allocation info for analysis
    fn node_to_allocation_info(&self, node: &VariableNode) -> Option<AllocationInfo> {
        let ptr = usize::from_str_radix(&node.id[2..], 16).ok()?;

        // Create a simplified AllocationInfo for compatibility
        Some(AllocationInfo::new(ptr, node.size))
    }

    /// Calculate graph statistics
    fn calculate_statistics(&self) -> GraphStatistics {
        let total_nodes = self.nodes.len();
        let total_relationships = self.relationships.len();

        let circular_references = self
            .relationships
            .iter()
            .filter(|rel| {
                rel.metadata
                    .get("circular_reference")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .count();

        let largest_cluster_size = self
            .clusters
            .iter()
            .map(|cluster| cluster.variables.len())
            .max()
            .unwrap_or(0);

        // Find isolated nodes (nodes with no relationships)
        let mut connected_nodes = HashSet::new();
        for rel in &self.relationships {
            connected_nodes.insert(&rel.source);
            connected_nodes.insert(&rel.target);
        }
        let isolated_nodes = total_nodes - connected_nodes.len();

        let avg_relationships_per_node = if total_nodes > 0 {
            total_relationships as f64 / total_nodes as f64
        } else {
            0.0
        };

        GraphStatistics {
            total_nodes,
            total_relationships,
            circular_references,
            largest_cluster_size,
            isolated_nodes,
            avg_relationships_per_node,
        }
    }
}

impl Default for VariableRelationshipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a complete variable relationship graph from allocations and registry
pub fn build_variable_relationship_graph(
    allocations: &[AllocationInfo],
    registry: &HashMap<usize, VariableInfo>,
) -> TrackingResult<VariableRelationshipGraph> {
    let graph = VariableRelationshipBuilder::new()
        .add_allocations(allocations, registry)
        .detect_references()
        .detect_scope_relationships()
        .detect_circular_references()
        .build_graph();

    Ok(graph)
}
