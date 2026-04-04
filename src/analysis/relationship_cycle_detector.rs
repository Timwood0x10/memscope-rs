use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CycleDetectionResult {
    pub cycle_edges: HashSet<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct CycleDetectionResultIndices {
    pub cycle_edges: Vec<(usize, usize)>,
    pub node_labels: Vec<String>,
}

/// Detect cycles in a relationship graph using DFS.
///
/// The function accepts owned Strings because:
/// 1. The return type requires owned cycle edge data
/// 2. The DFS algorithm stores nodes in a path Vec
/// 3. Relationship data is typically small (tens to hundreds), so cloning overhead is negligible
///
/// For very large graphs (10k+ relationships), consider using `detect_cycles_with_indices`.
pub fn detect_cycles_in_relationships(
    relationships: &[(String, String, String)],
) -> CycleDetectionResult {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut edge_set: HashSet<(String, String)> = HashSet::new();

    for (source, target, _) in relationships {
        graph
            .entry(source.clone())
            .or_default()
            .push(target.clone());
        edge_set.insert((source.clone(), target.clone()));
    }

    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: HashSet<String> = HashSet::new();
    let mut path: Vec<String> = Vec::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            dfs_detect_cycles(
                node,
                &graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    let mut cycle_edges: HashSet<(String, String)> = HashSet::new();
    for cycle in &cycles {
        for i in 0..cycle.len() {
            let from = cycle[i].clone();
            let to = cycle[(i + 1) % cycle.len()].clone();
            if edge_set.contains(&(from.clone(), to.clone())) {
                cycle_edges.insert((from, to));
            }
        }
    }

    CycleDetectionResult { cycle_edges }
}

fn dfs_detect_cycles(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_detect_cycles(neighbor, graph, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(neighbor) {
                if let Some(cycle_start) = path.iter().position(|p| p == neighbor) {
                    let cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

/// High-performance cycle detection using integer indices.
///
/// This version is optimized for large graphs (10k+ relationships) by:
/// 1. Using integer indices instead of strings for graph traversal
/// 2. Avoiding string cloning during DFS
/// 3. Using a single visited state array instead of HashSet
///
/// Uses direct string-to-index mapping to avoid hash collisions.
pub fn detect_cycles_with_indices(
    relationships: &[(String, String, String)],
) -> CycleDetectionResultIndices {
    if relationships.is_empty() {
        return CycleDetectionResultIndices {
            cycle_edges: Vec::new(),
            node_labels: Vec::new(),
        };
    }

    let mut unique_nodes: Vec<String> = Vec::new();
    let mut node_to_idx: HashMap<String, usize> = HashMap::new();

    for (source, target, _) in relationships {
        if !node_to_idx.contains_key(source) {
            let idx = unique_nodes.len();
            unique_nodes.push(source.clone());
            node_to_idx.insert(source.clone(), idx);
        }
        if !node_to_idx.contains_key(target) {
            let idx = unique_nodes.len();
            unique_nodes.push(target.clone());
            node_to_idx.insert(target.clone(), idx);
        }
    }

    let n = unique_nodes.len();
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_set: HashSet<(usize, usize)> = HashSet::new();

    for (source, target, _) in relationships {
        let source_idx = node_to_idx[source];
        let target_idx = node_to_idx[target];
        graph[source_idx].push(target_idx);
        edge_set.insert((source_idx, target_idx));
    }

    let mut visited: Vec<bool> = vec![false; n];
    let mut rec_stack: Vec<bool> = vec![false; n];
    let mut path: Vec<usize> = Vec::new();
    let mut cycle_edge_indices: Vec<(usize, usize)> = Vec::new();

    for start in 0..n {
        if !visited[start] {
            dfs_detect_cycles_indices(
                start,
                &graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycle_edge_indices,
            );
        }
    }

    CycleDetectionResultIndices {
        cycle_edges: cycle_edge_indices,
        node_labels: unique_nodes,
    }
}

fn dfs_detect_cycles_indices(
    node: usize,
    graph: &[Vec<usize>],
    visited: &mut Vec<bool>,
    rec_stack: &mut Vec<bool>,
    path: &mut Vec<usize>,
    cycle_edges: &mut Vec<(usize, usize)>,
) {
    visited[node] = true;
    rec_stack[node] = true;
    path.push(node);

    for &neighbor in &graph[node] {
        if !visited[neighbor] {
            dfs_detect_cycles_indices(neighbor, graph, visited, rec_stack, path, cycle_edges);
        } else if rec_stack[neighbor] {
            cycle_edges.push((node, neighbor));
        }
    }

    path.pop();
    rec_stack[node] = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycles() {
        let relationships = vec![
            ("0x1".to_string(), "0x2".to_string(), "clone".to_string()),
            ("0x2".to_string(), "0x3".to_string(), "clone".to_string()),
        ];
        let result = detect_cycles_in_relationships(&relationships);
        assert!(result.cycle_edges.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let relationships = vec![
            ("0x1".to_string(), "0x2".to_string(), "clone".to_string()),
            ("0x2".to_string(), "0x1".to_string(), "clone".to_string()),
        ];
        let result = detect_cycles_in_relationships(&relationships);
        assert!(!result.cycle_edges.is_empty());
    }

    #[test]
    fn test_self_loop() {
        let relationships = vec![("0x1".to_string(), "0x1".to_string(), "clone".to_string())];
        let result = detect_cycles_in_relationships(&relationships);
        assert!(!result.cycle_edges.is_empty());
    }

    #[test]
    fn test_indices_no_cycles() {
        let relationships = vec![
            ("0x1".to_string(), "0x2".to_string(), "clone".to_string()),
            ("0x2".to_string(), "0x3".to_string(), "clone".to_string()),
        ];
        let result = detect_cycles_with_indices(&relationships);
        assert!(result.cycle_edges.is_empty());
    }

    #[test]
    fn test_indices_simple_cycle() {
        let relationships = vec![
            ("0x1".to_string(), "0x2".to_string(), "clone".to_string()),
            ("0x2".to_string(), "0x1".to_string(), "clone".to_string()),
        ];
        let result = detect_cycles_with_indices(&relationships);
        assert!(!result.cycle_edges.is_empty());
    }
}
