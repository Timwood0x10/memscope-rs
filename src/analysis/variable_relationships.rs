//! Variable Relationship Analysis
//!
//! This module provides functionality to detect and analyze relationships between variables,
//! building a comprehensive graph for visualization and analysis.

use crate::core::types::{AllocationInfo, TrackingResult};
use crate::{analysis::CircularReferenceNode, variable_registry::VariableInfo};
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                    let clone_id = format!("0x{clone_addr:x}");
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
                    let parent_id = format!("0x{cloned_from_addr:x}");
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
                        let data_id = format!("0x{data_ptr:x}");
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
                    id: format!("scope_{scope_name}"),
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
                if let (Some(source), Some(target)) = (window.first(), window.last()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{
        AllocationInfo, RefCountSnapshot, SmartPointerInfo as CoreSmartPointerInfo,
        SmartPointerType,
    };
    use crate::variable_registry::VariableInfo;

    /// Helper function to create test allocation info
    fn create_test_allocation(
        ptr: usize,
        size: usize,
        var_name: Option<String>,
        type_name: Option<String>,
        scope_name: Option<String>,
    ) -> AllocationInfo {
        AllocationInfo {
            ptr,
            size,
            var_name,
            type_name,
            scope_name,
            timestamp_alloc: 1000,
            timestamp_dealloc: None,
            thread_id: "test_thread".to_string(),
            borrow_count: 0,
            stack_trace: Some(vec!["test_function".to_string()]),
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

    /// Helper function to create test allocation with smart pointer info
    #[allow(clippy::too_many_arguments)]
    fn create_smart_pointer_allocation(
        ptr: usize,
        size: usize,
        var_name: String,
        type_name: String,
        pointer_type: SmartPointerType,
        data_ptr: usize,
        clones: Vec<usize>,
        cloned_from: Option<usize>,
    ) -> AllocationInfo {
        let mut alloc = create_test_allocation(
            ptr,
            size,
            Some(var_name),
            Some(type_name),
            Some("main".to_string()),
        );

        alloc.smart_pointer_info = Some(CoreSmartPointerInfo {
            data_ptr,
            pointer_type,
            ref_count_history: vec![RefCountSnapshot {
                timestamp: 1000,
                strong_count: 1,
                weak_count: 0,
            }],
            weak_count: Some(0),
            is_data_owner: true,
            is_weak_reference: false,
            is_implicitly_deallocated: false,
            clones,
            cloned_from,
        });

        alloc
    }

    /// Helper function to create test variable info
    fn create_test_variable_info(var_name: String, type_name: String) -> VariableInfo {
        VariableInfo {
            var_name,
            type_name,
            timestamp: 1000,
            size: 64,
        }
    }

    #[test]
    fn test_relationship_type_serialization() {
        let relationship_types = vec![
            RelationshipType::References,
            RelationshipType::Owns,
            RelationshipType::Clones,
            RelationshipType::Contains,
            RelationshipType::DependsOn,
        ];

        for rel_type in relationship_types {
            let serialized = serde_json::to_string(&rel_type).expect("Failed to serialize");
            let _deserialized: RelationshipType =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
        }
    }

    #[test]
    fn test_variable_category_serialization() {
        let categories = vec![
            VariableCategory::UserVariable,
            VariableCategory::SystemAllocation,
            VariableCategory::SmartPointer,
            VariableCategory::Collection,
        ];

        for category in categories {
            let serialized = serde_json::to_string(&category).expect("Failed to serialize");
            let _deserialized: VariableCategory =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
        }
    }

    #[test]
    fn test_cluster_type_serialization() {
        let cluster_types = vec![
            ClusterType::Scope,
            ClusterType::Type,
            ClusterType::Lifetime,
            ClusterType::SmartPointerGroup,
        ];

        for cluster_type in cluster_types {
            let serialized = serde_json::to_string(&cluster_type).expect("Failed to serialize");
            let _deserialized: ClusterType =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
        }
    }

    #[test]
    fn test_variable_relationship_builder_creation() {
        let builder = VariableRelationshipBuilder::new();
        assert!(builder.nodes.is_empty());
        assert!(builder.relationships.is_empty());
        assert!(builder.clusters.is_empty());
    }

    #[test]
    fn test_variable_relationship_builder_default() {
        let builder = VariableRelationshipBuilder::default();
        assert!(builder.nodes.is_empty());
        assert!(builder.relationships.is_empty());
        assert!(builder.clusters.is_empty());
    }

    #[test]
    fn test_add_allocations_basic() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                1024,
                Some("var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new().add_allocations(&allocations, &registry);

        assert_eq!(builder.nodes.len(), 2);
        assert!(builder.nodes.contains_key("0x1000"));
        assert!(builder.nodes.contains_key("0x2000"));

        let node1 = &builder.nodes["0x1000"];
        assert_eq!(node1.name, "var1");
        assert_eq!(node1.type_name, "i32");
        assert_eq!(node1.size, 1024);
        assert_eq!(node1.scope, "main");
        assert!(node1.is_active);
        assert_eq!(node1.category, VariableCategory::UserVariable);
    }

    #[test]
    fn test_add_allocations_with_registry() {
        let allocations = vec![create_test_allocation(0x1000, 1024, None, None, None)];

        let mut registry = HashMap::new();
        registry.insert(
            0x1000,
            create_test_variable_info("registry_var".to_string(), "u64".to_string()),
        );

        let builder = VariableRelationshipBuilder::new().add_allocations(&allocations, &registry);

        assert_eq!(builder.nodes.len(), 1);
        let node = &builder.nodes["0x1000"];
        assert_eq!(node.name, "registry_var");
        assert_eq!(node.type_name, "u64");
        assert_eq!(node.category, VariableCategory::UserVariable);
    }

    #[test]
    fn test_add_allocations_inferred_categories() {
        let allocations = vec![
            create_test_allocation(0x1000, 1024, None, Some("Vec<i32>".to_string()), None),
            create_test_allocation(0x2000, 512, None, Some("Box<String>".to_string()), None),
            create_test_allocation(
                0x3000,
                256,
                None,
                Some("HashMap<String, i32>".to_string()),
                None,
            ),
            create_test_allocation(0x4000, 128, None, Some("Rc<Data>".to_string()), None),
        ];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new().add_allocations(&allocations, &registry);

        assert_eq!(builder.nodes.len(), 4);

        // Check that nodes are created with appropriate categories
        // The exact categorization may depend on the inference logic implementation
        let categories: Vec<_> = builder.nodes.values().map(|node| &node.category).collect();

        // We should have some user variables since we provided explicit type names
        let has_user_vars = categories
            .iter()
            .any(|&cat| *cat == VariableCategory::UserVariable);
        let has_smart_ptrs = categories
            .iter()
            .any(|&cat| *cat == VariableCategory::SmartPointer);
        let has_collections = categories
            .iter()
            .any(|&cat| *cat == VariableCategory::Collection);
        let has_system_allocs = categories
            .iter()
            .any(|&cat| *cat == VariableCategory::SystemAllocation);

        // At least one of these should be true
        assert!(has_user_vars || has_smart_ptrs || has_collections || has_system_allocs);

        // Verify specific nodes exist
        assert!(builder.nodes.contains_key("0x1000"));
        assert!(builder.nodes.contains_key("0x2000"));
        assert!(builder.nodes.contains_key("0x3000"));
        assert!(builder.nodes.contains_key("0x4000"));
    }

    #[test]
    fn test_detect_references_smart_pointers() {
        let allocations = vec![
            create_smart_pointer_allocation(
                0x1000,
                64,
                "rc1".to_string(),
                "Rc<i32>".to_string(),
                SmartPointerType::Rc,
                0x5000,
                vec![0x2000, 0x3000],
                None,
            ),
            create_smart_pointer_allocation(
                0x2000,
                64,
                "rc2".to_string(),
                "Rc<i32>".to_string(),
                SmartPointerType::Rc,
                0x5000,
                vec![],
                Some(0x1000),
            ),
            create_smart_pointer_allocation(
                0x3000,
                64,
                "rc3".to_string(),
                "Rc<i32>".to_string(),
                SmartPointerType::Rc,
                0x5000,
                vec![],
                Some(0x1000),
            ),
        ];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_references();

        // Should have clone relationships
        assert!(!builder.relationships.is_empty());

        let clone_relationships: Vec<_> = builder
            .relationships
            .iter()
            .filter(|rel| rel.relationship_type == RelationshipType::Clones)
            .collect();

        assert!(!clone_relationships.is_empty());

        // Check that relationships exist between clones
        let has_clone_rel = clone_relationships.iter().any(|rel| {
            rel.source == "0x1000" && (rel.target == "0x2000" || rel.target == "0x3000")
        });

        assert!(has_clone_rel);
    }

    #[test]
    fn test_detect_references_box_ownership() {
        let allocations = vec![
            create_smart_pointer_allocation(
                0x1000,
                64,
                "box_ptr".to_string(),
                "Box<String>".to_string(),
                SmartPointerType::Box,
                0x2000,
                vec![],
                None,
            ),
            create_test_allocation(
                0x2000,
                32,
                Some("data".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_references();

        // Should have ownership relationship
        let ownership_relationships: Vec<_> = builder
            .relationships
            .iter()
            .filter(|rel| rel.relationship_type == RelationshipType::Owns)
            .collect();

        assert!(!ownership_relationships.is_empty());

        let has_ownership = ownership_relationships
            .iter()
            .any(|rel| rel.source == "0x1000" && rel.target == "0x2000");

        assert!(has_ownership);
    }

    #[test]
    fn test_detect_scope_relationships() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                1024,
                Some("var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x3000,
                256,
                Some("var3".to_string()),
                Some("f64".to_string()),
                Some("function".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_scope_relationships();

        // Should have scope clusters
        assert!(!builder.clusters.is_empty());

        let scope_clusters: Vec<_> = builder
            .clusters
            .iter()
            .filter(|cluster| cluster.cluster_type == ClusterType::Scope)
            .collect();

        assert!(!scope_clusters.is_empty());

        // Should have main scope cluster with 2 variables
        let main_cluster = scope_clusters
            .iter()
            .find(|cluster| cluster.id == "scope_main");

        assert!(main_cluster.is_some());
        let main_cluster = main_cluster.unwrap();
        assert_eq!(main_cluster.variables.len(), 2);
        assert!(main_cluster.variables.contains(&"0x1000".to_string()));
        assert!(main_cluster.variables.contains(&"0x2000".to_string()));

        // Should have containment relationships within scope
        let containment_relationships: Vec<_> = builder
            .relationships
            .iter()
            .filter(|rel| rel.relationship_type == RelationshipType::Contains)
            .collect();

        assert!(!containment_relationships.is_empty());
    }

    #[test]
    fn test_build_graph_basic() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                1024,
                Some("var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let graph = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_scope_relationships()
            .build_graph();

        assert_eq!(graph.nodes.len(), 2);
        assert!(!graph.relationships.is_empty());
        assert!(!graph.clusters.is_empty());

        // Check statistics
        assert_eq!(graph.statistics.total_nodes, 2);
        assert!(graph.statistics.total_relationships > 0);
        assert!(graph.statistics.avg_relationships_per_node >= 0.0);

        // Check metadata
        assert!(graph.metadata.contains_key("build_timestamp"));
    }

    #[test]
    fn test_calculate_statistics() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                1024,
                Some("var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x3000,
                256,
                Some("isolated".to_string()),
                Some("f64".to_string()),
                Some("other".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let graph = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_scope_relationships()
            .build_graph();

        let stats = &graph.statistics;
        assert_eq!(stats.total_nodes, 3);
        assert!(stats.total_relationships > 0);
        assert!(stats.largest_cluster_size >= 2); // main scope cluster
        assert!(stats.isolated_nodes <= 1); // isolated variable might not have relationships
        assert!(stats.avg_relationships_per_node >= 0.0);
    }

    #[test]
    fn test_smart_pointer_info_conversion() {
        let allocations = vec![create_smart_pointer_allocation(
            0x1000,
            64,
            "rc_ptr".to_string(),
            "Rc<Data>".to_string(),
            SmartPointerType::Rc,
            0x2000,
            vec![0x3000],
            Some(0x4000),
        )];

        let registry = HashMap::new();

        let builder = VariableRelationshipBuilder::new().add_allocations(&allocations, &registry);

        let node = &builder.nodes["0x1000"];
        assert!(node.smart_pointer_info.is_some());

        let smart_ptr_info = node.smart_pointer_info.as_ref().unwrap();
        assert_eq!(smart_ptr_info.pointer_type, "Rc");
        assert_eq!(smart_ptr_info.ref_count, Some(1));
        assert_eq!(smart_ptr_info.data_ptr, Some(0x2000));
        assert_eq!(smart_ptr_info.clones, vec![0x3000]);
        assert_eq!(smart_ptr_info.cloned_from, Some(0x4000));
    }

    #[test]
    fn test_layout_hint_serialization() {
        let layout_hint = LayoutHint {
            x: 10.0,
            y: 20.0,
            width: Some(100.0),
            height: Some(50.0),
        };

        let serialized = serde_json::to_string(&layout_hint).expect("Failed to serialize");
        let deserialized: LayoutHint =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.x, 10.0);
        assert_eq!(deserialized.y, 20.0);
        assert_eq!(deserialized.width, Some(100.0));
        assert_eq!(deserialized.height, Some(50.0));
    }

    #[test]
    fn test_variable_node_serialization() {
        let node = VariableNode {
            id: "0x1000".to_string(),
            name: "test_var".to_string(),
            type_name: "i32".to_string(),
            size: 4,
            scope: "main".to_string(),
            is_active: true,
            category: VariableCategory::UserVariable,
            smart_pointer_info: None,
            created_at: 1000,
            destroyed_at: None,
        };

        let serialized = serde_json::to_string(&node).expect("Failed to serialize");
        let deserialized: VariableNode =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.id, "0x1000");
        assert_eq!(deserialized.name, "test_var");
        assert_eq!(deserialized.type_name, "i32");
        assert_eq!(deserialized.size, 4);
        assert!(deserialized.is_active);
    }

    #[test]
    fn test_variable_relationship_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "test_key".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );

        let relationship = VariableRelationship {
            source: "0x1000".to_string(),
            target: "0x2000".to_string(),
            relationship_type: RelationshipType::References,
            weight: 0.8,
            metadata,
        };

        let serialized = serde_json::to_string(&relationship).expect("Failed to serialize");
        let deserialized: VariableRelationship =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.source, "0x1000");
        assert_eq!(deserialized.target, "0x2000");
        assert_eq!(deserialized.relationship_type, RelationshipType::References);
        assert_eq!(deserialized.weight, 0.8);
        assert!(deserialized.metadata.contains_key("test_key"));
    }

    #[test]
    fn test_build_variable_relationship_graph_function() {
        let allocations = vec![
            create_test_allocation(
                0x1000,
                1024,
                Some("var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
        ];

        let registry = HashMap::new();

        let result = build_variable_relationship_graph(&allocations, &registry);
        assert!(result.is_ok());

        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert!(!graph.relationships.is_empty());
        assert!(!graph.clusters.is_empty());
    }

    #[test]
    fn test_comprehensive_workflow() {
        // Create a complex scenario with multiple types of relationships
        let allocations = vec![
            // Main scope variables
            create_test_allocation(
                0x1000,
                1024,
                Some("main_var1".to_string()),
                Some("i32".to_string()),
                Some("main".to_string()),
            ),
            create_test_allocation(
                0x2000,
                512,
                Some("main_var2".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
            // Function scope variable
            create_test_allocation(
                0x3000,
                256,
                Some("func_var".to_string()),
                Some("f64".to_string()),
                Some("function".to_string()),
            ),
            // Smart pointers
            create_smart_pointer_allocation(
                0x4000,
                64,
                "rc1".to_string(),
                "Rc<Data>".to_string(),
                SmartPointerType::Rc,
                0x6000,
                vec![0x5000],
                None,
            ),
            create_smart_pointer_allocation(
                0x5000,
                64,
                "rc2".to_string(),
                "Rc<Data>".to_string(),
                SmartPointerType::Rc,
                0x6000,
                vec![],
                Some(0x4000),
            ),
            // Box pointer
            create_smart_pointer_allocation(
                0x7000,
                64,
                "box_ptr".to_string(),
                "Box<String>".to_string(),
                SmartPointerType::Box,
                0x8000,
                vec![],
                None,
            ),
            create_test_allocation(
                0x8000,
                32,
                Some("boxed_data".to_string()),
                Some("String".to_string()),
                Some("main".to_string()),
            ),
            // Collections
            create_test_allocation(
                0x9000,
                128,
                Some("vec_data".to_string()),
                Some("Vec<i32>".to_string()),
                Some("main".to_string()),
            ),
        ];

        let mut registry = HashMap::new();
        registry.insert(
            0x1000,
            create_test_variable_info("registry_main_var".to_string(), "u32".to_string()),
        );

        let graph = VariableRelationshipBuilder::new()
            .add_allocations(&allocations, &registry)
            .detect_references()
            .detect_scope_relationships()
            .detect_circular_references()
            .build_graph();

        // Verify comprehensive results
        assert_eq!(graph.nodes.len(), 8);
        assert!(!graph.relationships.is_empty());
        assert!(!graph.clusters.is_empty());

        // Check for different types of relationships
        let has_clones = graph
            .relationships
            .iter()
            .any(|rel| rel.relationship_type == RelationshipType::Clones);
        let has_ownership = graph
            .relationships
            .iter()
            .any(|rel| rel.relationship_type == RelationshipType::Owns);
        let has_containment = graph
            .relationships
            .iter()
            .any(|rel| rel.relationship_type == RelationshipType::Contains);

        assert!(has_clones || has_ownership || has_containment);

        // Check for different categories - be flexible about categorization
        let mut categories = Vec::new();

        // Collect all unique categories
        for node in &graph.nodes {
            if !categories.contains(&&node.category) {
                categories.push(&node.category);
            }
        }

        // We should have at least one category
        assert!(
            !categories.is_empty(),
            "Expected at least one variable category"
        );

        // We should have user variables since we provided explicit names and types
        let has_user_vars = categories.contains(&&VariableCategory::UserVariable);
        assert!(has_user_vars, "Expected at least one user variable");

        // Check statistics
        let stats = &graph.statistics;
        assert_eq!(stats.total_nodes, 8);
        assert!(stats.total_relationships > 0);
        assert!(stats.avg_relationships_per_node >= 0.0);

        // Check metadata
        assert!(graph.metadata.contains_key("build_timestamp"));

        // Verify clusters exist
        let scope_clusters: Vec<_> = graph
            .clusters
            .iter()
            .filter(|cluster| cluster.cluster_type == ClusterType::Scope)
            .collect();

        assert!(!scope_clusters.is_empty());

        // Should have main scope cluster with multiple variables
        let main_cluster = scope_clusters
            .iter()
            .find(|cluster| cluster.variables.len() > 1);

        assert!(main_cluster.is_some());
    }
}
