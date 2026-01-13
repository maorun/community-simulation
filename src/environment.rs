use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents different types of environmental resources consumed in economic activities.
///
/// Each resource type represents a category of natural resources or environmental
/// services that are consumed during production and trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    /// Energy resources (electricity, fuel, etc.)
    Energy,
    /// Water resources
    Water,
    /// Raw materials (metals, minerals, etc.)
    Materials,
    /// Land use and space
    Land,
}

impl Resource {
    /// Returns all resource types for iteration.
    pub fn all() -> [Resource; 4] {
        [
            Resource::Energy,
            Resource::Water,
            Resource::Materials,
            Resource::Land,
        ]
    }

    /// Returns a human-readable name for this resource type.
    pub fn name(&self) -> &'static str {
        match self {
            Resource::Energy => "Energy",
            Resource::Water => "Water",
            Resource::Materials => "Materials",
            Resource::Land => "Land",
        }
    }
}

/// Tracks environmental resource consumption and sustainability metrics.
///
/// The Environment tracks total resource consumption across all economic activities
/// and calculates sustainability metrics based on resource availability and usage patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Total consumption of each resource type across all transactions.
    pub total_consumption: HashMap<Resource, f64>,
    /// Available resource reserves (starting amounts).
    /// When consumption exceeds reserves, the economy is unsustainable.
    pub resource_reserves: HashMap<Resource, f64>,
    /// Current step number for tracking resource usage over time.
    pub current_step: usize,
}

impl Environment {
    /// Creates a new Environment with specified resource reserves.
    ///
    /// # Arguments
    /// * `resource_reserves` - Initial reserves for each resource type
    ///
    /// # Returns
    /// A new Environment instance with zero consumption
    pub fn new(resource_reserves: HashMap<Resource, f64>) -> Self {
        let mut total_consumption = HashMap::new();
        for resource in Resource::all() {
            total_consumption.insert(resource, 0.0);
        }

        Environment {
            total_consumption,
            resource_reserves,
            current_step: 0,
        }
    }

    /// Creates a new Environment with default resource reserves.
    ///
    /// Default reserves are set to provide a sustainable environment
    /// for typical simulation scenarios.
    ///
    /// # Returns
    /// A new Environment with default reserves:
    /// - Energy: 100,000 units
    /// - Water: 100,000 units
    /// - Materials: 100,000 units
    /// - Land: 10,000 units
    pub fn with_default_reserves() -> Self {
        let mut reserves = HashMap::new();
        reserves.insert(Resource::Energy, 100_000.0);
        reserves.insert(Resource::Water, 100_000.0);
        reserves.insert(Resource::Materials, 100_000.0);
        reserves.insert(Resource::Land, 10_000.0);

        Self::new(reserves)
    }

    /// Records resource consumption from an economic transaction.
    ///
    /// # Arguments
    /// * `resource_costs` - Map of resource types to amounts consumed
    pub fn consume_resources(&mut self, resource_costs: &HashMap<Resource, f64>) {
        for (resource, &amount) in resource_costs {
            *self.total_consumption.entry(*resource).or_insert(0.0) += amount;
        }
    }

    /// Advances the environment to the next simulation step.
    pub fn step(&mut self) {
        self.current_step += 1;
    }

    /// Calculates the sustainability score for each resource type.
    ///
    /// The sustainability score ranges from 0.0 to 1.0, where:
    /// - 1.0 = No consumption, fully sustainable
    /// - 0.5 = Consumed half of reserves
    /// - 0.0 = Completely depleted reserves
    /// - < 0.0 = Overconsumption (exceeded reserves)
    ///
    /// # Returns
    /// A map of resource types to sustainability scores
    pub fn sustainability_scores(&self) -> HashMap<Resource, f64> {
        let mut scores = HashMap::new();

        for resource in Resource::all() {
            let consumed = self.total_consumption.get(&resource).copied().unwrap_or(0.0);
            let reserves = self.resource_reserves.get(&resource).copied().unwrap_or(1.0);

            let score = if reserves > 0.0 {
                1.0 - (consumed / reserves)
            } else {
                0.0
            };

            scores.insert(resource, score);
        }

        scores
    }

    /// Calculates the overall sustainability score as the average of all resource scores.
    ///
    /// # Returns
    /// A value between -infinity and 1.0, where 1.0 is fully sustainable
    /// and negative values indicate overconsumption
    pub fn overall_sustainability_score(&self) -> f64 {
        let scores = self.sustainability_scores();
        let sum: f64 = scores.values().sum();
        sum / scores.len() as f64
    }

    /// Checks if the environment is sustainable (no resource is depleted).
    ///
    /// # Returns
    /// `true` if all resources have positive sustainability scores, `false` otherwise
    pub fn is_sustainable(&self) -> bool {
        self.sustainability_scores().values().all(|&score| score >= 0.0)
    }

    /// Gets the remaining reserves for a specific resource.
    ///
    /// # Arguments
    /// * `resource` - The resource type to check
    ///
    /// # Returns
    /// The remaining amount (reserves - consumption), can be negative if overconsumed
    pub fn remaining_reserves(&self, resource: Resource) -> f64 {
        let reserves = self.resource_reserves.get(&resource).copied().unwrap_or(0.0);
        let consumed = self.total_consumption.get(&resource).copied().unwrap_or(0.0);
        reserves - consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_all() {
        let all = Resource::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&Resource::Energy));
        assert!(all.contains(&Resource::Water));
        assert!(all.contains(&Resource::Materials));
        assert!(all.contains(&Resource::Land));
    }

    #[test]
    fn test_environment_new() {
        let mut reserves = HashMap::new();
        reserves.insert(Resource::Energy, 1000.0);
        reserves.insert(Resource::Water, 500.0);

        let env = Environment::new(reserves);
        assert_eq!(env.current_step, 0);
        assert_eq!(env.total_consumption.len(), 4);
        assert_eq!(*env.resource_reserves.get(&Resource::Energy).unwrap(), 1000.0);
    }

    #[test]
    fn test_environment_default_reserves() {
        let env = Environment::with_default_reserves();
        assert_eq!(*env.resource_reserves.get(&Resource::Energy).unwrap(), 100_000.0);
        assert_eq!(*env.resource_reserves.get(&Resource::Water).unwrap(), 100_000.0);
        assert_eq!(*env.resource_reserves.get(&Resource::Materials).unwrap(), 100_000.0);
        assert_eq!(*env.resource_reserves.get(&Resource::Land).unwrap(), 10_000.0);
    }

    #[test]
    fn test_consume_resources() {
        let mut env = Environment::with_default_reserves();
        let mut costs = HashMap::new();
        costs.insert(Resource::Energy, 100.0);
        costs.insert(Resource::Water, 50.0);

        env.consume_resources(&costs);

        assert_eq!(*env.total_consumption.get(&Resource::Energy).unwrap(), 100.0);
        assert_eq!(*env.total_consumption.get(&Resource::Water).unwrap(), 50.0);
    }

    #[test]
    fn test_sustainability_score() {
        let mut env = Environment::with_default_reserves();
        let mut costs = HashMap::new();
        costs.insert(Resource::Energy, 50_000.0); // Consume half of energy

        env.consume_resources(&costs);

        let scores = env.sustainability_scores();
        let energy_score = scores.get(&Resource::Energy).unwrap();
        assert!((energy_score - 0.5).abs() < 0.001); // Should be ~0.5
    }

    #[test]
    fn test_overall_sustainability_score() {
        let mut env = Environment::with_default_reserves();
        assert!((env.overall_sustainability_score() - 1.0).abs() < 0.001); // No consumption = 1.0

        let mut costs = HashMap::new();
        for resource in Resource::all() {
            let reserves = env.resource_reserves.get(&resource).copied().unwrap_or(0.0);
            costs.insert(resource, reserves / 2.0); // Consume half of each
        }

        env.consume_resources(&costs);
        assert!((env.overall_sustainability_score() - 0.5).abs() < 0.001); // Half consumed = 0.5
    }

    #[test]
    fn test_is_sustainable() {
        let mut env = Environment::with_default_reserves();
        assert!(env.is_sustainable());

        let mut costs = HashMap::new();
        costs.insert(Resource::Energy, 150_000.0); // Exceed reserves

        env.consume_resources(&costs);
        assert!(!env.is_sustainable()); // Should be unsustainable now
    }

    #[test]
    fn test_remaining_reserves() {
        let mut env = Environment::with_default_reserves();
        let mut costs = HashMap::new();
        costs.insert(Resource::Energy, 10_000.0);

        env.consume_resources(&costs);

        let remaining = env.remaining_reserves(Resource::Energy);
        assert_eq!(remaining, 90_000.0);
    }

    #[test]
    fn test_step() {
        let mut env = Environment::with_default_reserves();
        assert_eq!(env.current_step, 0);
        env.step();
        assert_eq!(env.current_step, 1);
    }
}
