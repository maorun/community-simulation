//! Network centrality analysis for trading networks.
//!
//! This module provides comprehensive network analysis capabilities including:
//! - Degree centrality (number of connections)
//! - Betweenness centrality (brokerage/bridge positions)
//! - Eigenvector centrality (influence based on connections)
//! - PageRank (importance based on weighted connections)
//!
//! These metrics help identify key traders, market structure, and trading patterns.

use crate::result::{NetworkEdge, NetworkNode};
use petgraph::algo::connected_components;
use petgraph::graph::NodeIndex;
use petgraph::{Graph, Undirected};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Centrality metrics for a single node (person) in the trading network
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeCentrality {
    /// Node identifier (person ID)
    pub node_id: String,
    /// Degree centrality: number of unique trading partners (normalized 0.0-1.0)
    pub degree_centrality: f64,
    /// Betweenness centrality: how often this node lies on shortest paths between other nodes (normalized 0.0-1.0)
    /// High values indicate "broker" or "bridge" positions
    pub betweenness_centrality: f64,
    /// Eigenvector centrality: influence based on connections to other well-connected nodes (normalized 0.0-1.0)
    /// High values indicate connections to important traders
    pub eigenvector_centrality: f64,
    /// PageRank score: importance based on weighted connections (normalized 0.0-1.0)
    /// High values indicate influential market participants
    pub pagerank: f64,
}

/// Complete network centrality analysis results
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CentralityAnalysis {
    /// Centrality metrics for each node
    pub node_centralities: Vec<NodeCentrality>,
    /// Network-level metrics
    pub network_metrics: NetworkCentralityMetrics,
    /// Top 5 nodes by degree centrality (most connections)
    pub top_degree: Vec<String>,
    /// Top 5 nodes by betweenness centrality (best brokers)
    pub top_betweenness: Vec<String>,
    /// Top 5 nodes by eigenvector centrality (most influential)
    pub top_eigenvector: Vec<String>,
    /// Top 5 nodes by PageRank (highest importance)
    pub top_pagerank: Vec<String>,
}

/// Network-level centrality metrics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkCentralityMetrics {
    /// Number of connected components (separate trading groups)
    pub connected_components: usize,
    /// Average degree centrality across all nodes
    pub avg_degree_centrality: f64,
    /// Average betweenness centrality across all nodes
    pub avg_betweenness_centrality: f64,
    /// Average eigenvector centrality across all nodes
    pub avg_eigenvector_centrality: f64,
    /// Average PageRank across all nodes
    pub avg_pagerank: f64,
    /// Network density (ratio of actual edges to possible edges, 0.0-1.0)
    pub network_density: f64,
}

/// Calculates comprehensive centrality analysis for a trading network.
///
/// # Arguments
///
/// * `nodes` - List of network nodes (persons)
/// * `edges` - List of network edges (trading relationships)
///
/// # Returns
///
/// Complete centrality analysis with node-level and network-level metrics.
///
/// # Example
///
/// ```
/// use simulation_framework::centrality::calculate_centrality;
/// use simulation_framework::result::{NetworkNode, NetworkEdge};
///
/// let nodes = vec![
///     NetworkNode { id: "Person1".to_string(), money: 100.0, reputation: 1.0, trade_count: 5, unique_partners: 2 },
///     NetworkNode { id: "Person2".to_string(), money: 150.0, reputation: 1.2, trade_count: 3, unique_partners: 1 },
/// ];
/// let edges = vec![
///     NetworkEdge { source: "Person1".to_string(), target: "Person2".to_string(), weight: 3, total_value: 150.0 },
/// ];
///
/// let analysis = calculate_centrality(&nodes, &edges);
/// assert!(!analysis.node_centralities.is_empty());
/// ```
pub fn calculate_centrality(nodes: &[NetworkNode], edges: &[NetworkEdge]) -> CentralityAnalysis {
    // Handle empty network
    if nodes.is_empty() {
        return CentralityAnalysis {
            node_centralities: vec![],
            network_metrics: NetworkCentralityMetrics {
                connected_components: 0,
                avg_degree_centrality: 0.0,
                avg_betweenness_centrality: 0.0,
                avg_eigenvector_centrality: 0.0,
                avg_pagerank: 0.0,
                network_density: 0.0,
            },
            top_degree: vec![],
            top_betweenness: vec![],
            top_eigenvector: vec![],
            top_pagerank: vec![],
        };
    }

    // Build graph
    let mut graph: Graph<String, f64, Undirected> = Graph::new_undirected();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Add nodes
    for node in nodes {
        let idx = graph.add_node(node.id.clone());
        node_indices.insert(node.id.clone(), idx);
    }

    // Add edges with weights (total_value as weight)
    for edge in edges {
        if let (Some(&source_idx), Some(&target_idx)) =
            (node_indices.get(&edge.source), node_indices.get(&edge.target))
        {
            graph.add_edge(source_idx, target_idx, edge.total_value);
        }
    }

    // Calculate centrality metrics
    let degree_centrality = calculate_degree_centrality(&graph, &node_indices);
    let betweenness_centrality = calculate_betweenness_centrality(&graph, &node_indices);
    let eigenvector_centrality = calculate_eigenvector_centrality(&graph, &node_indices);
    let pagerank = calculate_pagerank(&graph, &node_indices);

    // Build node centrality results
    let mut node_centralities: Vec<NodeCentrality> = nodes
        .iter()
        .map(|node| NodeCentrality {
            node_id: node.id.clone(),
            degree_centrality: degree_centrality.get(&node.id).copied().unwrap_or(0.0),
            betweenness_centrality: betweenness_centrality.get(&node.id).copied().unwrap_or(0.0),
            eigenvector_centrality: eigenvector_centrality.get(&node.id).copied().unwrap_or(0.0),
            pagerank: pagerank.get(&node.id).copied().unwrap_or(0.0),
        })
        .collect();

    // Calculate network-level metrics
    let connected_components = connected_components(&graph);
    let n = nodes.len() as f64;
    let avg_degree = node_centralities.iter().map(|nc| nc.degree_centrality).sum::<f64>() / n;
    let avg_betweenness =
        node_centralities.iter().map(|nc| nc.betweenness_centrality).sum::<f64>() / n;
    let avg_eigenvector =
        node_centralities.iter().map(|nc| nc.eigenvector_centrality).sum::<f64>() / n;
    let avg_pagerank = node_centralities.iter().map(|nc| nc.pagerank).sum::<f64>() / n;

    let network_density = if nodes.len() > 1 {
        let actual_edges = edges.len() as f64;
        let possible_edges = (nodes.len() * (nodes.len() - 1)) as f64 / 2.0;
        actual_edges / possible_edges
    } else {
        0.0
    };

    // Find top nodes by each metric
    node_centralities.sort_by(|a, b| {
        b.degree_centrality
            .partial_cmp(&a.degree_centrality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_degree: Vec<String> =
        node_centralities.iter().take(5).map(|nc| nc.node_id.clone()).collect();

    node_centralities.sort_by(|a, b| {
        b.betweenness_centrality
            .partial_cmp(&a.betweenness_centrality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_betweenness: Vec<String> =
        node_centralities.iter().take(5).map(|nc| nc.node_id.clone()).collect();

    node_centralities.sort_by(|a, b| {
        b.eigenvector_centrality
            .partial_cmp(&a.eigenvector_centrality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_eigenvector: Vec<String> =
        node_centralities.iter().take(5).map(|nc| nc.node_id.clone()).collect();

    node_centralities
        .sort_by(|a, b| b.pagerank.partial_cmp(&a.pagerank).unwrap_or(std::cmp::Ordering::Equal));
    let top_pagerank: Vec<String> =
        node_centralities.iter().take(5).map(|nc| nc.node_id.clone()).collect();

    // Sort by node ID for consistency in output
    node_centralities.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    CentralityAnalysis {
        node_centralities,
        network_metrics: NetworkCentralityMetrics {
            connected_components,
            avg_degree_centrality: avg_degree,
            avg_betweenness_centrality: avg_betweenness,
            avg_eigenvector_centrality: avg_eigenvector,
            avg_pagerank,
            network_density,
        },
        top_degree,
        top_betweenness,
        top_eigenvector,
        top_pagerank,
    }
}

/// Calculate degree centrality for all nodes (normalized 0.0-1.0)
fn calculate_degree_centrality(
    graph: &Graph<String, f64, Undirected>,
    node_indices: &HashMap<String, NodeIndex>,
) -> HashMap<String, f64> {
    let n = graph.node_count();
    if n <= 1 {
        return node_indices.keys().map(|id| (id.clone(), 0.0)).collect();
    }

    let max_degree = (n - 1) as f64;
    node_indices
        .iter()
        .map(|(id, &idx)| {
            let degree = graph.neighbors(idx).count() as f64;
            let normalized = degree / max_degree;
            (id.clone(), normalized)
        })
        .collect()
}

/// Calculate betweenness centrality using Brandes' algorithm (normalized 0.0-1.0)
/// This is the standard efficient algorithm for computing betweenness centrality
fn calculate_betweenness_centrality(
    graph: &Graph<String, f64, Undirected>,
    node_indices: &HashMap<String, NodeIndex>,
) -> HashMap<String, f64> {
    let n = graph.node_count();
    if n <= 2 {
        return node_indices.keys().map(|id| (id.clone(), 0.0)).collect();
    }

    // Initialize betweenness scores
    let mut betweenness: HashMap<NodeIndex, f64> =
        node_indices.values().map(|&idx| (idx, 0.0)).collect();

    // For each node as source
    for &source in node_indices.values() {
        // Compute shortest paths from source using BFS
        let mut stack: Vec<NodeIndex> = Vec::new();
        let mut predecessors: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
        let mut distance: HashMap<NodeIndex, usize> = HashMap::new();
        let mut paths: HashMap<NodeIndex, usize> = HashMap::new();

        distance.insert(source, 0);
        paths.insert(source, 1);

        let mut queue: std::collections::VecDeque<NodeIndex> = std::collections::VecDeque::new();
        queue.push_back(source);

        // BFS
        while let Some(v) = queue.pop_front() {
            stack.push(v);
            // INVARIANT: v came from queue, which only contains nodes already in distance map
            let dist_v =
                *distance.get(&v).expect("BFS invariant: node in queue must be in distance map");

            for neighbor in graph.neighbors(v) {
                // First time visiting neighbor?
                if let std::collections::hash_map::Entry::Vacant(e) = distance.entry(neighbor) {
                    e.insert(dist_v + 1);
                    queue.push_back(neighbor);
                }

                // Is this shortest path to neighbor via v?
                // INVARIANT: neighbor was just inserted or already existed in distance map above
                let neighbor_dist = *distance
                    .get(&neighbor)
                    .expect("BFS invariant: neighbor must be in distance map");
                if neighbor_dist == dist_v + 1 {
                    // INVARIANT: v was already verified to be in paths via the queue invariant
                    let paths_v =
                        *paths.get(&v).expect("BFS invariant: node in queue must be in paths map");
                    *paths.entry(neighbor).or_insert(0) += paths_v;
                    predecessors.entry(neighbor).or_insert_with(Vec::new).push(v);
                }
            }
        }

        // Accumulate betweenness from leaves to root
        let mut dependency: HashMap<NodeIndex, f64> =
            node_indices.values().map(|&idx| (idx, 0.0)).collect();

        while let Some(w) = stack.pop() {
            if let Some(preds) = predecessors.get(&w) {
                // INVARIANT: w came from stack, which only contains nodes visited in BFS and added to paths
                let paths_w =
                    *paths.get(&w).expect("Brandes invariant: node in stack must be in paths map")
                        as f64;
                let dep_w = dependency.get(&w).copied().unwrap_or(0.0);

                for &v in preds {
                    // INVARIANT: predecessors only contains nodes visited in BFS and added to paths
                    let paths_v = *paths
                        .get(&v)
                        .expect("Brandes invariant: predecessor must be in paths map")
                        as f64;
                    let contribution = (paths_v / paths_w) * (1.0 + dep_w);
                    *dependency.entry(v).or_insert(0.0) += contribution;
                }
            }

            if w != source {
                *betweenness.entry(w).or_insert(0.0) += dependency.get(&w).copied().unwrap_or(0.0);
            }
        }
    }

    // Normalize: for undirected graphs, divide by 2 and normalize by (n-1)(n-2)/2
    let max_betweenness = ((n - 1) * (n - 2)) as f64 / 2.0;

    node_indices
        .iter()
        .map(|(id, &idx)| {
            let bc = betweenness.get(&idx).copied().unwrap_or(0.0) / 2.0; // Divide by 2 for undirected
            let normalized = if max_betweenness > 0.0 {
                bc / max_betweenness
            } else {
                0.0
            };
            (id.clone(), normalized)
        })
        .collect()
}

/// Calculate eigenvector centrality using power iteration (normalized 0.0-1.0)
fn calculate_eigenvector_centrality(
    graph: &Graph<String, f64, Undirected>,
    node_indices: &HashMap<String, NodeIndex>,
) -> HashMap<String, f64> {
    let n = graph.node_count();
    if n == 0 {
        return HashMap::new();
    }

    // Initialize all nodes with equal centrality
    let mut centrality: HashMap<NodeIndex, f64> =
        node_indices.values().map(|&idx| (idx, 1.0 / n as f64)).collect();

    // Power iteration (100 iterations should be sufficient)
    for _ in 0..100 {
        let mut new_centrality: HashMap<NodeIndex, f64> = HashMap::new();

        for &idx in node_indices.values() {
            let sum: f64 = graph
                .neighbors(idx)
                .map(|neighbor| centrality.get(&neighbor).copied().unwrap_or(0.0))
                .sum();
            new_centrality.insert(idx, sum);
        }

        // Normalize
        let norm: f64 = new_centrality.values().map(|&v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 {
            for value in new_centrality.values_mut() {
                *value /= norm;
            }
        }

        centrality = new_centrality;
    }

    // Convert to string keys
    node_indices
        .iter()
        .map(|(id, &idx)| {
            let ec = centrality.get(&idx).copied().unwrap_or(0.0);
            (id.clone(), ec)
        })
        .collect()
}

/// Calculate PageRank using power iteration (normalized 0.0-1.0)
fn calculate_pagerank(
    graph: &Graph<String, f64, Undirected>,
    node_indices: &HashMap<String, NodeIndex>,
) -> HashMap<String, f64> {
    let n = graph.node_count();
    if n == 0 {
        return HashMap::new();
    }

    let damping = 0.85;
    let epsilon = 1e-6;

    // Initialize all nodes with equal PageRank
    let mut pagerank: HashMap<NodeIndex, f64> =
        node_indices.values().map(|&idx| (idx, 1.0 / n as f64)).collect();

    // Power iteration
    for _ in 0..100 {
        let mut new_pagerank: HashMap<NodeIndex, f64> = HashMap::new();

        for &idx in node_indices.values() {
            let mut sum = 0.0;
            for neighbor in graph.neighbors(idx) {
                let neighbor_degree = graph.neighbors(neighbor).count() as f64;
                if neighbor_degree > 0.0 {
                    sum += pagerank.get(&neighbor).copied().unwrap_or(0.0) / neighbor_degree;
                }
            }
            let pr = (1.0 - damping) / n as f64 + damping * sum;
            new_pagerank.insert(idx, pr);
        }

        // Check convergence
        let diff: f64 = node_indices
            .values()
            .map(|&idx| {
                let old = pagerank.get(&idx).copied().unwrap_or(0.0);
                let new = new_pagerank.get(&idx).copied().unwrap_or(0.0);
                (new - old).abs()
            })
            .sum();

        pagerank = new_pagerank;

        if diff < epsilon {
            break;
        }
    }

    // Normalize to 0.0-1.0 range
    let max_pr = pagerank.values().copied().fold(0.0, f64::max);
    if max_pr > 0.0 {
        for value in pagerank.values_mut() {
            *value /= max_pr;
        }
    }

    // Convert to string keys
    node_indices
        .iter()
        .map(|(id, &idx)| {
            let pr = pagerank.get(&idx).copied().unwrap_or(0.0);
            (id.clone(), pr)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_network() {
        let nodes = vec![];
        let edges = vec![];
        let analysis = calculate_centrality(&nodes, &edges);
        assert_eq!(analysis.node_centralities.len(), 0);
        assert_eq!(analysis.network_metrics.connected_components, 0);
    }

    #[test]
    fn test_single_node() {
        let nodes = vec![NetworkNode {
            id: "Person1".to_string(),
            money: 100.0,
            reputation: 1.0,
            trade_count: 0,
            unique_partners: 0,
        }];
        let edges = vec![];
        let analysis = calculate_centrality(&nodes, &edges);
        assert_eq!(analysis.node_centralities.len(), 1);
        assert_eq!(analysis.node_centralities[0].degree_centrality, 0.0);
    }

    #[test]
    fn test_two_nodes() {
        let nodes = vec![
            NetworkNode {
                id: "Person1".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Person2".to_string(),
                money: 150.0,
                reputation: 1.2,
                trade_count: 1,
                unique_partners: 1,
            },
        ];
        let edges = vec![NetworkEdge {
            source: "Person1".to_string(),
            target: "Person2".to_string(),
            weight: 1,
            total_value: 50.0,
        }];
        let analysis = calculate_centrality(&nodes, &edges);
        assert_eq!(analysis.node_centralities.len(), 2);
        // Both nodes should have degree centrality of 1.0 (connected to all other nodes)
        assert_eq!(analysis.node_centralities[0].degree_centrality, 1.0);
        assert_eq!(analysis.node_centralities[1].degree_centrality, 1.0);
        assert_eq!(analysis.network_metrics.connected_components, 1);
    }

    #[test]
    fn test_star_network() {
        // One central node connected to 4 others
        let nodes = vec![
            NetworkNode {
                id: "Center".to_string(),
                money: 500.0,
                reputation: 2.0,
                trade_count: 4,
                unique_partners: 4,
            },
            NetworkNode {
                id: "Spoke1".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Spoke2".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Spoke3".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Spoke4".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
        ];
        let edges = vec![
            NetworkEdge {
                source: "Center".to_string(),
                target: "Spoke1".to_string(),
                weight: 1,
                total_value: 100.0,
            },
            NetworkEdge {
                source: "Center".to_string(),
                target: "Spoke2".to_string(),
                weight: 1,
                total_value: 100.0,
            },
            NetworkEdge {
                source: "Center".to_string(),
                target: "Spoke3".to_string(),
                weight: 1,
                total_value: 100.0,
            },
            NetworkEdge {
                source: "Center".to_string(),
                target: "Spoke4".to_string(),
                weight: 1,
                total_value: 100.0,
            },
        ];
        let analysis = calculate_centrality(&nodes, &edges);
        assert_eq!(analysis.node_centralities.len(), 5);

        // Center node should have highest degree centrality (4/4 = 1.0)
        let center_centrality =
            analysis.node_centralities.iter().find(|nc| nc.node_id == "Center").unwrap();
        assert_eq!(center_centrality.degree_centrality, 1.0);

        // Spoke nodes should have lower degree centrality (1/4 = 0.25)
        let spoke_centrality =
            analysis.node_centralities.iter().find(|nc| nc.node_id == "Spoke1").unwrap();
        assert_eq!(spoke_centrality.degree_centrality, 0.25);

        // Center should be in top_degree
        assert_eq!(analysis.top_degree[0], "Center");
    }

    #[test]
    fn test_disconnected_network() {
        let nodes = vec![
            NetworkNode {
                id: "Person1".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Person2".to_string(),
                money: 150.0,
                reputation: 1.2,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Person3".to_string(),
                money: 200.0,
                reputation: 1.5,
                trade_count: 1,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Person4".to_string(),
                money: 250.0,
                reputation: 1.8,
                trade_count: 1,
                unique_partners: 1,
            },
        ];
        let edges = vec![
            NetworkEdge {
                source: "Person1".to_string(),
                target: "Person2".to_string(),
                weight: 1,
                total_value: 50.0,
            },
            NetworkEdge {
                source: "Person3".to_string(),
                target: "Person4".to_string(),
                weight: 1,
                total_value: 75.0,
            },
        ];
        let analysis = calculate_centrality(&nodes, &edges);
        // Should have 2 connected components
        assert_eq!(analysis.network_metrics.connected_components, 2);
    }

    #[test]
    fn test_nan_handling_in_sorting() {
        // Test that NaN values in centrality scores don't cause panics
        let nodes = vec![
            NetworkNode {
                id: "Person1".to_string(),
                money: 100.0,
                reputation: 1.0,
                trade_count: 5,
                unique_partners: 2,
            },
            NetworkNode {
                id: "Person2".to_string(),
                money: f64::NAN, // NaN value
                reputation: f64::NAN,
                trade_count: 3,
                unique_partners: 1,
            },
            NetworkNode {
                id: "Person3".to_string(),
                money: 200.0,
                reputation: 2.0,
                trade_count: 4,
                unique_partners: 2,
            },
        ];

        let edges = vec![NetworkEdge {
            source: "Person1".to_string(),
            target: "Person3".to_string(),
            weight: 1,
            total_value: 100.0,
        }];

        // This should not panic even with NaN values
        let analysis = calculate_centrality(&nodes, &edges);

        // Verify that analysis completes without panic
        assert_eq!(analysis.node_centralities.len(), 3);
    }
}
