// Variable relationship mapper for building dependency graphs
// Creates visual maps of how variables interact across threads and processes

use crate::core::types::{TrackingResult, TrackingError};
use crate::export::fixed_hybrid_template::HybridAnalysisData;
use std::collections::{HashMap, HashSet};
use tracing::{info, debug, trace};

/// Variable relationship mapper for building dependency and interaction graphs
pub struct VariableRelationshipMapper {
    relationship_analyzers: Vec<Box<dyn RelationshipAnalyzer>>,
    graph_builder: GraphBuilder,
    dependency_detector: DependencyDetector,
}

impl VariableRelationshipMapper {
    /// Create new relationship mapper
    pub fn new() -> Self {
        info!("Initializing variable relationship mapper");
        
        let analyzers: Vec<Box<dyn RelationshipAnalyzer>> = vec![
            Box::new(TypeBasedAnalyzer::new()),
            Box::new(NameBasedAnalyzer::new()),
            Box::new(TemporalAnalyzer::new()),
            Box::new(SizeBasedAnalyzer::new()),
        ];
        
        Self {
            relationship_analyzers: analyzers,
            graph_builder: GraphBuilder::new(),
            dependency_detector: DependencyDetector::new(),
        }
    }
    
    /// Build comprehensive relationship graph
    pub fn build_relationship_graph(
        &mut self,
        data: &HashMap<u32, HybridAnalysisData>,
        max_depth: usize,
    ) -> TrackingResult<RelationshipGraph> {
        info!("Building relationship graph with max depth {}", max_depth);
        
        // Extract all variables from all processes
        let all_variables = self.extract_all_variables(data)?;
        debug!("Extracted {} variables for analysis", all_variables.len());
        
        // Build basic graph structure
        let mut graph = self.graph_builder.create_initial_graph(&all_variables)?;
        
        // Add relationships using each analyzer
        for analyzer in &mut self.relationship_analyzers {
            let relationships = analyzer.analyze_relationships(&all_variables)?;
            debug!("Analyzer '{}' found {} relationships", 
                   analyzer.name(), relationships.len());
            
            for relationship in relationships {
                graph.add_relationship(relationship)?;
            }
        }
        
        // Detect dependencies up to max depth
        let dependencies = self.dependency_detector.detect_dependencies(&graph, max_depth)?;
        graph.set_dependencies(dependencies);
        
        // Calculate relationship strengths
        graph.calculate_relationship_strengths()?;
        
        // Generate clusters for visualization
        graph.generate_clusters()?;
        
        info!("Relationship graph complete: {} nodes, {} edges, {} clusters",
              graph.node_count(), graph.edge_count(), graph.cluster_count());
        
        Ok(graph)
    }
    
    /// Extract all variables from process data
    fn extract_all_variables(
        &self,
        data: &HashMap<u32, HybridAnalysisData>,
    ) -> TrackingResult<Vec<VariableNode>> {
        let mut variables = Vec::new();
        
        for (process_id, process_data) in data {
            for (var_id, variable) in &process_data.variable_registry {
                let node = VariableNode {
                    id: format!("{}_{}", process_id, var_id),
                    process_id: *process_id,
                    thread_id: variable.thread_id,
                    name: variable.name.clone(),
                    type_info: variable.type_info.clone(),
                    memory_usage: variable.memory_usage,
                    allocation_count: variable.allocation_count,
                    lifecycle_stage: variable.lifecycle_stage.clone(),
                    creation_timestamp: 0, // Could be enhanced with actual timestamps
                    relationships: Vec::new(),
                };
                variables.push(node);
            }
        }
        
        Ok(variables)
    }
}

/// Graph builder for creating relationship structures
struct GraphBuilder {
    node_id_generator: NodeIdGenerator,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            node_id_generator: NodeIdGenerator::new(),
        }
    }
    
    fn create_initial_graph(&mut self, variables: &[VariableNode]) -> TrackingResult<RelationshipGraph> {
        let mut graph = RelationshipGraph::new();
        
        // Add all variables as nodes
        for variable in variables {
            graph.add_node(variable.clone());
        }
        
        trace!("Initial graph created with {} nodes", variables.len());
        Ok(graph)
    }
}

/// Node ID generator for unique identification
struct NodeIdGenerator {
    next_id: usize,
}

impl NodeIdGenerator {
    fn new() -> Self {
        Self { next_id: 0 }
    }
    
    fn next(&mut self) -> String {
        let id = format!("node_{}", self.next_id);
        self.next_id += 1;
        id
    }
}

/// Dependency detector for finding variable dependencies
struct DependencyDetector {
    max_analysis_depth: usize,
}

impl DependencyDetector {
    fn new() -> Self {
        Self {
            max_analysis_depth: 5,
        }
    }
    
    fn detect_dependencies(
        &mut self,
        graph: &RelationshipGraph,
        max_depth: usize,
    ) -> TrackingResult<Vec<VariableDependency>> {
        debug!("Detecting dependencies with max depth {}", max_depth);
        
        let mut dependencies = Vec::new();
        let depth_limit = max_depth.min(self.max_analysis_depth);
        
        // For each node, find its dependencies
        for node in graph.nodes() {
            let node_dependencies = self.find_node_dependencies(graph, node, depth_limit)?;
            dependencies.extend(node_dependencies);
        }
        
        trace!("Found {} dependencies", dependencies.len());
        Ok(dependencies)
    }
    
    fn find_node_dependencies(
        &self,
        graph: &RelationshipGraph,
        start_node: &VariableNode,
        max_depth: usize,
    ) -> TrackingResult<Vec<VariableDependency>> {
        let mut dependencies = Vec::new();
        let mut visited = HashSet::new();
        
        self.dfs_dependencies(graph, start_node, &mut dependencies, &mut visited, 0, max_depth)?;
        
        Ok(dependencies)
    }
    
    fn dfs_dependencies(
        &self,
        graph: &RelationshipGraph,
        current_node: &VariableNode,
        dependencies: &mut Vec<VariableDependency>,
        visited: &mut HashSet<String>,
        current_depth: usize,
        max_depth: usize,
    ) -> TrackingResult<()> {
        if current_depth >= max_depth || visited.contains(&current_node.id) {
            return Ok(());
        }
        
        visited.insert(current_node.id.clone());
        
        // Find connected nodes
        for relationship in graph.get_relationships_for_node(&current_node.id) {
            let connected_node_id = if relationship.source_id == current_node.id {
                &relationship.target_id
            } else {
                &relationship.source_id
            };
            
            if let Some(connected_node) = graph.get_node(connected_node_id) {
                // Create dependency if it's a strong relationship
                if relationship.strength > 0.6 {
                    dependencies.push(VariableDependency {
                        dependent_variable: current_node.id.clone(),
                        dependency_variable: connected_node.id.clone(),
                        dependency_type: self.classify_dependency_type(&relationship),
                        strength: relationship.strength,
                        description: format!(
                            "{} depends on {} via {}",
                            current_node.name,
                            connected_node.name,
                            relationship.relationship_type
                        ),
                    });
                }
                
                // Continue DFS
                self.dfs_dependencies(
                    graph,
                    connected_node,
                    dependencies,
                    visited,
                    current_depth + 1,
                    max_depth,
                )?;
            }
        }
        
        Ok(())
    }
    
    fn classify_dependency_type(&self, relationship: &VariableRelationship) -> DependencyType {
        match relationship.relationship_type.as_str() {
            "type_similarity" => DependencyType::TypeBased,
            "name_similarity" => DependencyType::Logical,
            "temporal_correlation" => DependencyType::Temporal,
            "size_correlation" => DependencyType::Resource,
            _ => DependencyType::Other,
        }
    }
}

// Relationship analyzers

/// Trait for relationship analysis algorithms
trait RelationshipAnalyzer {
    fn name(&self) -> &str;
    fn analyze_relationships(&mut self, variables: &[VariableNode]) -> TrackingResult<Vec<VariableRelationship>>;
}

/// Type-based relationship analyzer
struct TypeBasedAnalyzer {
    name: String,
}

impl TypeBasedAnalyzer {
    fn new() -> Self {
        Self {
            name: "TypeBasedAnalyzer".to_string(),
        }
    }
}

impl RelationshipAnalyzer for TypeBasedAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn analyze_relationships(&mut self, variables: &[VariableNode]) -> TrackingResult<Vec<VariableRelationship>> {
        let mut relationships = Vec::new();
        
        // Group variables by type
        let mut type_groups: HashMap<String, Vec<&VariableNode>> = HashMap::new();
        for variable in variables {
            type_groups.entry(variable.type_info.clone()).or_default().push(variable);
        }
        
        // Create relationships within type groups
        for (type_name, group_variables) in type_groups {
            if group_variables.len() > 1 {
                for i in 0..group_variables.len() {
                    for j in (i + 1)..group_variables.len() {
                        let var1 = group_variables[i];
                        let var2 = group_variables[j];
                        
                        // Calculate type-based relationship strength
                        let strength = self.calculate_type_similarity_strength(var1, var2);
                        
                        if strength > 0.3 { // Minimum threshold
                            relationships.push(VariableRelationship {
                                id: format!("type_{}_{}", var1.id, var2.id),
                                source_id: var1.id.clone(),
                                target_id: var2.id.clone(),
                                relationship_type: "type_similarity".to_string(),
                                strength,
                                description: format!("Shared type: {}", type_name),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
        
        trace!("Type-based analyzer found {} relationships", relationships.len());
        Ok(relationships)
    }
}

impl TypeBasedAnalyzer {
    fn calculate_type_similarity_strength(&self, var1: &VariableNode, var2: &VariableNode) -> f64 {
        let mut strength = 0.5; // Base strength for same type
        
        // Boost strength for same thread
        if var1.thread_id == var2.thread_id {
            strength += 0.2;
        }
        
        // Boost strength for same process
        if var1.process_id == var2.process_id {
            strength += 0.1;
        }
        
        // Boost strength for similar memory usage
        let size_ratio = (var1.memory_usage.min(var2.memory_usage) as f64) 
                        / (var1.memory_usage.max(var2.memory_usage) as f64);
        if size_ratio > 0.8 {
            strength += 0.2;
        }
        
        strength.min(1.0)
    }
}

/// Name-based relationship analyzer
struct NameBasedAnalyzer {
    name: String,
}

impl NameBasedAnalyzer {
    fn new() -> Self {
        Self {
            name: "NameBasedAnalyzer".to_string(),
        }
    }
}

impl RelationshipAnalyzer for NameBasedAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn analyze_relationships(&mut self, variables: &[VariableNode]) -> TrackingResult<Vec<VariableRelationship>> {
        let mut relationships = Vec::new();
        
        for i in 0..variables.len() {
            for j in (i + 1)..variables.len() {
                let var1 = &variables[i];
                let var2 = &variables[j];
                
                let similarity = self.calculate_name_similarity(&var1.name, &var2.name);
                
                if similarity > 0.6 { // Name similarity threshold
                    relationships.push(VariableRelationship {
                        id: format!("name_{}_{}", var1.id, var2.id),
                        source_id: var1.id.clone(),
                        target_id: var2.id.clone(),
                        relationship_type: "name_similarity".to_string(),
                        strength: similarity,
                        description: format!("Similar names: '{}' and '{}'", var1.name, var2.name),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        trace!("Name-based analyzer found {} relationships", relationships.len());
        Ok(relationships)
    }
}

impl NameBasedAnalyzer {
    fn calculate_name_similarity(&self, name1: &str, name2: &str) -> f64 {
        if name1 == name2 {
            return 1.0;
        }
        
        // Simple prefix/suffix similarity
        let common_prefix = name1.chars()
            .zip(name2.chars())
            .take_while(|(c1, c2)| c1 == c2)
            .count();
        
        let common_suffix = name1.chars().rev()
            .zip(name2.chars().rev())
            .take_while(|(c1, c2)| c1 == c2)
            .count();
        
        let max_len = name1.len().max(name2.len());
        if max_len == 0 {
            0.0
        } else {
            (common_prefix + common_suffix) as f64 / max_len as f64
        }
    }
}

/// Temporal relationship analyzer
struct TemporalAnalyzer {
    name: String,
}

impl TemporalAnalyzer {
    fn new() -> Self {
        Self {
            name: "TemporalAnalyzer".to_string(),
        }
    }
}

impl RelationshipAnalyzer for TemporalAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn analyze_relationships(&mut self, variables: &[VariableNode]) -> TrackingResult<Vec<VariableRelationship>> {
        let mut relationships = Vec::new();
        
        // Group by creation time windows (simplified)
        let mut time_windows: HashMap<u64, Vec<&VariableNode>> = HashMap::new();
        for variable in variables {
            let time_window = variable.creation_timestamp / 1000; // 1-second windows
            time_windows.entry(time_window).or_default().push(variable);
        }
        
        // Create temporal relationships within windows
        for (time_window, window_variables) in time_windows {
            if window_variables.len() > 1 {
                for i in 0..window_variables.len() {
                    for j in (i + 1)..window_variables.len() {
                        let var1 = window_variables[i];
                        let var2 = window_variables[j];
                        
                        relationships.push(VariableRelationship {
                            id: format!("temporal_{}_{}", var1.id, var2.id),
                            source_id: var1.id.clone(),
                            target_id: var2.id.clone(),
                            relationship_type: "temporal_correlation".to_string(),
                            strength: 0.7, // Strong temporal correlation
                            description: format!("Created in same time window: {}", time_window),
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
        
        trace!("Temporal analyzer found {} relationships", relationships.len());
        Ok(relationships)
    }
}

/// Size-based relationship analyzer
struct SizeBasedAnalyzer {
    name: String,
}

impl SizeBasedAnalyzer {
    fn new() -> Self {
        Self {
            name: "SizeBasedAnalyzer".to_string(),
        }
    }
}

impl RelationshipAnalyzer for SizeBasedAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn analyze_relationships(&mut self, variables: &[VariableNode]) -> TrackingResult<Vec<VariableRelationship>> {
        let mut relationships = Vec::new();
        
        // Group by size ranges
        let size_ranges = [(0, 1024), (1024, 1024*1024), (1024*1024, u64::MAX)];
        
        for &(min_size, max_size) in &size_ranges {
            let range_variables: Vec<_> = variables.iter()
                .filter(|v| v.memory_usage >= min_size && v.memory_usage < max_size)
                .collect();
            
            if range_variables.len() > 1 {
                for i in 0..range_variables.len() {
                    for j in (i + 1)..range_variables.len() {
                        let var1 = range_variables[i];
                        let var2 = range_variables[j];
                        
                        let size_correlation = self.calculate_size_correlation(var1.memory_usage, var2.memory_usage);
                        
                        if size_correlation > 0.8 {
                            relationships.push(VariableRelationship {
                                id: format!("size_{}_{}", var1.id, var2.id),
                                source_id: var1.id.clone(),
                                target_id: var2.id.clone(),
                                relationship_type: "size_correlation".to_string(),
                                strength: size_correlation,
                                description: format!("Similar memory usage: {} bytes", var1.memory_usage),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
        
        trace!("Size-based analyzer found {} relationships", relationships.len());
        Ok(relationships)
    }
}

impl SizeBasedAnalyzer {
    fn calculate_size_correlation(&self, size1: u64, size2: u64) -> f64 {
        if size1 == 0 && size2 == 0 {
            return 1.0;
        }
        
        let ratio = (size1.min(size2) as f64) / (size1.max(size2) as f64);
        ratio
    }
}

// Data structures

/// Variable node in the relationship graph
#[derive(Debug, Clone)]
pub struct VariableNode {
    pub id: String,
    pub process_id: u32,
    pub thread_id: usize,
    pub name: String,
    pub type_info: String,
    pub memory_usage: u64,
    pub allocation_count: u64,
    pub lifecycle_stage: crate::export::fixed_hybrid_template::LifecycleStage,
    pub creation_timestamp: u64,
    pub relationships: Vec<String>, // IDs of related variables
}

/// Relationship between variables
#[derive(Debug, Clone)]
pub struct VariableRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub strength: f64,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

/// Variable dependency information
#[derive(Debug, Clone)]
pub struct VariableDependency {
    pub dependent_variable: String,
    pub dependency_variable: String,
    pub dependency_type: DependencyType,
    pub strength: f64,
    pub description: String,
}

/// Types of dependencies
#[derive(Debug, Clone)]
pub enum DependencyType {
    TypeBased,
    Logical,
    Temporal,
    Resource,
    Other,
}

/// Complete relationship graph
#[derive(Debug)]
pub struct RelationshipGraph {
    nodes: HashMap<String, VariableNode>,
    relationships: HashMap<String, VariableRelationship>,
    dependencies: Vec<VariableDependency>,
    clusters: Vec<VariableCluster>,
}

impl RelationshipGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relationships: HashMap::new(),
            dependencies: Vec::new(),
            clusters: Vec::new(),
        }
    }
    
    fn add_node(&mut self, node: VariableNode) {
        self.nodes.insert(node.id.clone(), node);
    }
    
    fn add_relationship(&mut self, relationship: VariableRelationship) -> TrackingResult<()> {
        self.relationships.insert(relationship.id.clone(), relationship);
        Ok(())
    }
    
    fn set_dependencies(&mut self, dependencies: Vec<VariableDependency>) {
        self.dependencies = dependencies;
    }
    
    fn calculate_relationship_strengths(&mut self) -> TrackingResult<()> {
        // Relationship strengths are already calculated by analyzers
        // This could be enhanced with graph-wide normalization
        Ok(())
    }
    
    fn generate_clusters(&mut self) -> TrackingResult<()> {
        debug!("Generating variable clusters");
        
        // Simple clustering based on strong relationships
        let mut clusters = Vec::new();
        let mut clustered_nodes = HashSet::new();
        
        for relationship in self.relationships.values() {
            if relationship.strength > 0.7 && 
               !clustered_nodes.contains(&relationship.source_id) &&
               !clustered_nodes.contains(&relationship.target_id) {
                
                let cluster = VariableCluster {
                    id: format!("cluster_{}", clusters.len()),
                    member_nodes: vec![relationship.source_id.clone(), relationship.target_id.clone()],
                    cluster_type: ClusterType::StrongRelationship,
                    cohesion_score: relationship.strength,
                };
                
                clustered_nodes.insert(relationship.source_id.clone());
                clustered_nodes.insert(relationship.target_id.clone());
                clusters.push(cluster);
            }
        }
        
        self.clusters = clusters;
        trace!("Generated {} clusters", self.clusters.len());
        Ok(())
    }
    
    pub fn nodes(&self) -> impl Iterator<Item = &VariableNode> {
        self.nodes.values()
    }
    
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn edge_count(&self) -> usize {
        self.relationships.len()
    }
    
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }
    
    pub fn get_node(&self, id: &str) -> Option<&VariableNode> {
        self.nodes.get(id)
    }
    
    pub fn get_relationships_for_node(&self, node_id: &str) -> Vec<&VariableRelationship> {
        self.relationships.values()
            .filter(|r| r.source_id == node_id || r.target_id == node_id)
            .collect()
    }
    
    pub fn get_dependencies(&self) -> &[VariableDependency] {
        &self.dependencies
    }
    
    pub fn get_clusters(&self) -> &[VariableCluster] {
        &self.clusters
    }
}

/// Variable cluster for grouping related variables
#[derive(Debug, Clone)]
pub struct VariableCluster {
    pub id: String,
    pub member_nodes: Vec<String>,
    pub cluster_type: ClusterType,
    pub cohesion_score: f64,
}

/// Types of variable clusters
#[derive(Debug, Clone)]
pub enum ClusterType {
    StrongRelationship,
    TypeBased,
    ProcessLocal,
    ThreadLocal,
}