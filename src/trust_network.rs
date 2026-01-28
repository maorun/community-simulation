use crate::person::PersonId;
use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trust level between two persons in the network.
/// Trust is transitive and decreases with social distance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Direct friends (1st degree connection): 100% trust
    Direct,
    /// Friends of friends (2nd degree connection): 50% trust
    SecondDegree,
    /// Friends of friends of friends (3rd degree connection): 25% trust
    ThirdDegree,
    /// No trust relationship exists beyond 3rd degree
    None,
}

impl TrustLevel {
    /// Returns the trust multiplier for price discounts.
    /// Higher trust = larger discount on trades.
    ///
    /// # Returns
    /// * `Direct`: 1.0 (full friendship discount applies)
    /// * `SecondDegree`: 0.5 (half the friendship discount)
    /// * `ThirdDegree`: 0.25 (quarter of the friendship discount)
    /// * `None`: 0.0 (no discount)
    pub fn discount_multiplier(&self) -> f64 {
        match self {
            TrustLevel::Direct => 1.0,
            TrustLevel::SecondDegree => 0.5,
            TrustLevel::ThirdDegree => 0.25,
            TrustLevel::None => 0.0,
        }
    }

    /// Converts social distance to trust level.
    ///
    /// # Arguments
    /// * `distance` - Number of friendship edges between two persons
    ///
    /// # Returns
    /// The corresponding trust level based on distance
    pub fn from_distance(distance: usize) -> Self {
        match distance {
            1 => TrustLevel::Direct,
            2 => TrustLevel::SecondDegree,
            3 => TrustLevel::ThirdDegree,
            _ => TrustLevel::None,
        }
    }
}

/// Manages trust relationships in the simulation based on friendship networks.
///
/// The trust network uses a graph-based model where:
/// - Nodes represent persons
/// - Edges represent friendships (undirected)
/// - Trust propagates transitively up to 3 degrees of separation
///
/// Trust levels decrease with social distance:
/// - Direct friends: Full trust (1.0x discount multiplier)
/// - 2nd degree: Partial trust (0.5x discount multiplier)
/// - 3rd degree: Low trust (0.25x discount multiplier)
/// - Beyond 3rd degree: No trust (0.0x)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustNetwork {
    /// Graph representing the friendship network
    /// Each node is a person, edges are friendships
    graph: UnGraph<PersonId, ()>,

    /// Maps person IDs to their graph node indices
    person_to_node: HashMap<PersonId, NodeIndex>,

    /// Cache of trust levels between pairs of persons
    /// Key: (person_a, person_b) where person_a < person_b
    /// Value: TrustLevel
    trust_cache: HashMap<(PersonId, PersonId), TrustLevel>,
}

impl TrustNetwork {
    /// Creates a new empty trust network.
    pub fn new() -> Self {
        TrustNetwork {
            graph: UnGraph::new_undirected(),
            person_to_node: HashMap::new(),
            trust_cache: HashMap::new(),
        }
    }

    /// Adds a person to the trust network.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person to add
    pub fn add_person(&mut self, person_id: PersonId) {
        if !self.person_to_node.contains_key(&person_id) {
            let node = self.graph.add_node(person_id);
            self.person_to_node.insert(person_id, node);
        }
    }

    /// Adds a friendship edge between two persons.
    /// If either person doesn't exist in the network, they are added first.
    /// Invalidates the trust cache since the network topology changed.
    ///
    /// # Arguments
    /// * `person_a` - ID of the first person
    /// * `person_b` - ID of the second person
    pub fn add_friendship(&mut self, person_a: PersonId, person_b: PersonId) {
        // Ensure both persons exist in the graph
        self.add_person(person_a);
        self.add_person(person_b);

        let node_a = self.person_to_node[&person_a];
        let node_b = self.person_to_node[&person_b];

        // Add edge if it doesn't exist
        if !self.graph.contains_edge(node_a, node_b) {
            self.graph.add_edge(node_a, node_b, ());
            // Invalidate cache since network changed
            self.trust_cache.clear();
        }
    }

    /// Calculates the trust level between two persons based on their social distance.
    /// Uses cached results when available to improve performance.
    ///
    /// # Arguments
    /// * `person_a` - ID of the first person
    /// * `person_b` - ID of the second person
    ///
    /// # Returns
    /// The trust level between the two persons
    pub fn get_trust_level(&mut self, person_a: PersonId, person_b: PersonId) -> TrustLevel {
        // Same person has no trust relationship (they don't trade with themselves)
        if person_a == person_b {
            return TrustLevel::None;
        }

        // Create canonical key (smaller ID first) for cache lookup
        let cache_key = if person_a < person_b {
            (person_a, person_b)
        } else {
            (person_b, person_a)
        };

        // Check cache first
        if let Some(&trust_level) = self.trust_cache.get(&cache_key) {
            return trust_level;
        }

        // Calculate trust level if not cached
        let trust_level = self.calculate_trust_level(person_a, person_b);

        // Store in cache
        self.trust_cache.insert(cache_key, trust_level);

        trust_level
    }

    /// Calculates the trust level by finding the shortest path between two persons.
    /// Uses Dijkstra's algorithm for efficient pathfinding.
    fn calculate_trust_level(&self, person_a: PersonId, person_b: PersonId) -> TrustLevel {
        // Get graph nodes for both persons
        let node_a = match self.person_to_node.get(&person_a) {
            Some(&node) => node,
            None => return TrustLevel::None, // Person not in network
        };

        let node_b = match self.person_to_node.get(&person_b) {
            Some(&node) => node,
            None => return TrustLevel::None, // Person not in network
        };

        // Use Dijkstra to find shortest path (all edges have weight 1)
        let distances = dijkstra(&self.graph, node_a, Some(node_b), |_| 1);

        // Get distance to target node
        match distances.get(&node_b) {
            Some(&distance) => TrustLevel::from_distance(distance as usize),
            None => TrustLevel::None, // No path exists
        }
    }

    /// Returns statistics about the trust network.
    pub fn get_statistics(&self) -> TrustNetworkStats {
        let total_persons = self.person_to_node.len();
        let total_friendships = self.graph.edge_count();

        // Count trust relationships by degree
        let mut direct_trusts = 0;
        let mut second_degree_trusts = 0;
        let mut third_degree_trusts = 0;

        // This is expensive - only calculate when needed for statistics
        // We don't cache all pairs, only those that are queried during simulation
        for (person_a, node_a) in &self.person_to_node {
            let distances = dijkstra(&self.graph, *node_a, None, |_| 1);

            for (node_b, distance) in distances {
                let person_b = self.graph[node_b];
                if person_a < &person_b {
                    // Only count each pair once
                    match distance {
                        1 => direct_trusts += 1,
                        2 => second_degree_trusts += 1,
                        3 => third_degree_trusts += 1,
                        _ => {},
                    }
                }
            }
        }

        TrustNetworkStats {
            total_persons,
            total_friendships,
            direct_trust_relationships: direct_trusts,
            second_degree_trust_relationships: second_degree_trusts,
            third_degree_trust_relationships: third_degree_trusts,
        }
    }

    /// Clears the trust cache.
    /// Should be called when the friendship network changes significantly.
    pub fn clear_cache(&mut self) {
        self.trust_cache.clear();
    }
}

impl Default for TrustNetwork {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about trust relationships in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustNetworkStats {
    /// Total number of persons in the trust network
    pub total_persons: usize,

    /// Total number of direct friendships (edges in the graph)
    pub total_friendships: usize,

    /// Number of direct trust relationships (1st degree connections)
    /// This equals total_friendships
    pub direct_trust_relationships: usize,

    /// Number of second-degree trust relationships
    /// (persons connected through one mutual friend)
    pub second_degree_trust_relationships: usize,

    /// Number of third-degree trust relationships
    /// (persons connected through two intermediate friends)
    pub third_degree_trust_relationships: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_discount_multipliers() {
        assert_eq!(TrustLevel::Direct.discount_multiplier(), 1.0);
        assert_eq!(TrustLevel::SecondDegree.discount_multiplier(), 0.5);
        assert_eq!(TrustLevel::ThirdDegree.discount_multiplier(), 0.25);
        assert_eq!(TrustLevel::None.discount_multiplier(), 0.0);
    }

    #[test]
    fn test_trust_level_from_distance() {
        assert_eq!(TrustLevel::from_distance(1), TrustLevel::Direct);
        assert_eq!(TrustLevel::from_distance(2), TrustLevel::SecondDegree);
        assert_eq!(TrustLevel::from_distance(3), TrustLevel::ThirdDegree);
        assert_eq!(TrustLevel::from_distance(4), TrustLevel::None);
        assert_eq!(TrustLevel::from_distance(0), TrustLevel::None);
    }

    #[test]
    fn test_empty_network() {
        let mut network = TrustNetwork::new();
        assert_eq!(network.get_trust_level(1, 2), TrustLevel::None);
    }

    #[test]
    fn test_direct_friendship() {
        let mut network = TrustNetwork::new();
        network.add_friendship(1, 2);

        assert_eq!(network.get_trust_level(1, 2), TrustLevel::Direct);
        assert_eq!(network.get_trust_level(2, 1), TrustLevel::Direct);
    }

    #[test]
    fn test_second_degree_trust() {
        let mut network = TrustNetwork::new();
        // 1 -- 2 -- 3
        network.add_friendship(1, 2);
        network.add_friendship(2, 3);

        assert_eq!(network.get_trust_level(1, 2), TrustLevel::Direct);
        assert_eq!(network.get_trust_level(2, 3), TrustLevel::Direct);
        assert_eq!(network.get_trust_level(1, 3), TrustLevel::SecondDegree);
    }

    #[test]
    fn test_third_degree_trust() {
        let mut network = TrustNetwork::new();
        // 1 -- 2 -- 3 -- 4
        network.add_friendship(1, 2);
        network.add_friendship(2, 3);
        network.add_friendship(3, 4);

        assert_eq!(network.get_trust_level(1, 4), TrustLevel::ThirdDegree);
        assert_eq!(network.get_trust_level(4, 1), TrustLevel::ThirdDegree);
    }

    #[test]
    fn test_no_trust_beyond_third_degree() {
        let mut network = TrustNetwork::new();
        // 1 -- 2 -- 3 -- 4 -- 5
        network.add_friendship(1, 2);
        network.add_friendship(2, 3);
        network.add_friendship(3, 4);
        network.add_friendship(4, 5);

        assert_eq!(network.get_trust_level(1, 5), TrustLevel::None);
    }

    #[test]
    fn test_shortest_path_preference() {
        let mut network = TrustNetwork::new();
        // Triangle: 1 -- 2 -- 3
        //            \       /
        //             -------
        // 1 and 3 are both direct friends and 2nd degree
        // Should return Direct (shorter path)
        network.add_friendship(1, 2);
        network.add_friendship(2, 3);
        network.add_friendship(1, 3);

        assert_eq!(network.get_trust_level(1, 3), TrustLevel::Direct);
    }

    #[test]
    fn test_same_person_has_no_trust() {
        let mut network = TrustNetwork::new();
        network.add_person(1);

        assert_eq!(network.get_trust_level(1, 1), TrustLevel::None);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut network = TrustNetwork::new();
        network.add_person(1);
        network.add_person(2);

        // Initially no trust
        assert_eq!(network.get_trust_level(1, 2), TrustLevel::None);

        // Add friendship
        network.add_friendship(1, 2);

        // Should now have direct trust (cache was invalidated)
        assert_eq!(network.get_trust_level(1, 2), TrustLevel::Direct);
    }

    #[test]
    fn test_statistics() {
        let mut network = TrustNetwork::new();
        // Create a simple network: 1 -- 2 -- 3
        network.add_friendship(1, 2);
        network.add_friendship(2, 3);

        let stats = network.get_statistics();
        assert_eq!(stats.total_persons, 3);
        assert_eq!(stats.total_friendships, 2);
        assert_eq!(stats.direct_trust_relationships, 2);
        assert_eq!(stats.second_degree_trust_relationships, 1); // 1-3
    }
}
