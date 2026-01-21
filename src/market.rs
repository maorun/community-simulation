//! Market mechanisms and price dynamics for the economic simulation.
//!
//! This module defines the [`Market`] struct, which coordinates all trading activity
//! and manages skill prices based on supply, demand, and configured pricing strategies.

use crate::{
    scenario::PriceUpdater,
    skill::{Skill, SkillId},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cached market statistics to reduce redundant calculations.
///
/// This cache stores frequently accessed aggregate statistics about the market.
/// It is invalidated whenever prices are updated to ensure data consistency.
#[derive(Clone, Debug, Default)]
struct MarketStatsCache {
    /// Average price across all skills in the market
    average_price: Option<f64>,
    /// Total value of all skills (sum of all prices)
    total_market_value: Option<f64>,
    /// Minimum skill price in the market
    min_price: Option<f64>,
    /// Maximum skill price in the market
    max_price: Option<f64>,
}

/// Represents the market where skills are traded and prices are determined.
///
/// The market tracks supply and demand for each skill, adjusts prices based on
/// configured strategies, and maintains historical price data for analysis.
///
/// # Price Update Mechanisms
///
/// Prices can be updated using different strategies:
/// - **Original**: Supply/demand-based with random volatility
/// - **DynamicPricing**: Sales-based (increase if sold, decrease if not)
///
/// # Examples
///
/// ```
/// use simulation_framework::{Market, Skill};
/// use simulation_framework::scenario::{PriceUpdater, Scenario};
///
/// let price_updater = PriceUpdater::from(Scenario::Original);
/// let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);
///
/// let skill = Skill::new("Programming".to_string(), 50.0);
/// market.add_skill(skill);
///
/// // Track supply and demand
/// market.increment_skill_supply(&"Programming".to_string());
/// market.increment_demand(&"Programming".to_string());
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Market {
    /// All skills available in the market, indexed by skill ID
    pub skills: HashMap<SkillId, Skill>,

    /// Count of how many persons need each skill (demand side)
    pub demand_counts: HashMap<SkillId, usize>,

    /// Count of how many persons provide each skill (supply side)
    pub supply_counts: HashMap<SkillId, usize>,

    /// Base price used as a reference for skill pricing
    pub base_skill_price: f64,

    /// Factor controlling how sensitive prices are to supply/demand imbalances
    ///
    /// Higher values mean prices change more dramatically when supply != demand.
    /// Typical values: 0.05 - 0.2
    pub price_elasticity_factor: f64,

    /// Percentage of random price fluctuation added each step
    ///
    /// Adds market volatility. For example, 0.02 means ±2% random variation.
    pub volatility_percentage: f64,

    /// Minimum allowed price for any skill
    pub min_skill_price: f64,

    /// Maximum allowed price for any skill
    pub max_skill_price: f64,

    /// Per-skill price limits (minimum, maximum) for regulatory controls.
    ///
    /// Enables skill-specific price regulations, overriding global min/max limits.
    /// When a skill has a specific limit set, that limit takes precedence over
    /// the global `min_skill_price` and `max_skill_price` values.
    ///
    /// Format: HashMap<SkillId, (Option<min>, Option<max>)>
    /// - None means no specific limit (fall back to global limit)
    /// - Some(value) means enforce this specific limit
    ///
    /// This enables studying regulatory interventions like:
    /// - Skill-specific minimum wages (price floors)
    /// - Skill-specific price caps (price ceilings)
    /// - Mixed regulatory regimes (some skills regulated, others free-market)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub per_skill_price_limits: HashMap<SkillId, (Option<f64>, Option<f64>)>,

    /// Historical price data for each skill across all simulation steps
    ///
    /// Maps skill IDs to a vector of prices, with one entry per simulation step.
    /// This data is useful for analyzing price trends and market dynamics.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub skill_price_history: HashMap<SkillId, Vec<f64>>,

    /// Strategy used to update prices each step
    #[serde(skip)]
    price_updater: PriceUpdater,

    /// Count of sales for each skill in the current step
    ///
    /// Used by the DynamicPricing scenario to track which skills were purchased.
    /// Reset at the beginning of each simulation step.
    #[serde(skip)]
    pub sales_this_step: HashMap<SkillId, usize>,

    /// Cached market statistics to avoid redundant calculations
    ///
    /// This cache stores aggregate statistics and is invalidated when prices change.
    /// Not serialized as it can be recomputed from skill data.
    #[serde(skip)]
    cache: MarketStatsCache,
}

impl Market {
    /// Creates a new market with the specified base price and pricing strategy.
    ///
    /// # Arguments
    ///
    /// * `base_skill_price` - Initial/reference price for skills
    /// * `min_skill_price` - Minimum allowed price floor for skills
    /// * `price_elasticity_factor` - Sensitivity to supply/demand imbalances (0.0-1.0)
    /// * `volatility_percentage` - Random price fluctuation percentage (0.0-0.5)
    /// * `price_updater` - Strategy to use for updating prices
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::Market;
    /// use simulation_framework::scenario::{PriceUpdater, Scenario};
    ///
    /// let updater = PriceUpdater::from(Scenario::Original);
    /// let market = Market::new(10.0, 1.0, 0.1, 0.02, updater);
    /// ```
    pub fn new(
        base_skill_price: f64,
        min_skill_price: f64,
        price_elasticity_factor: f64,
        volatility_percentage: f64,
        price_updater: PriceUpdater,
    ) -> Self {
        Market {
            skills: HashMap::new(),
            demand_counts: HashMap::new(),
            supply_counts: HashMap::new(),
            base_skill_price,
            price_elasticity_factor,
            volatility_percentage,
            min_skill_price,
            max_skill_price: 1000.0,
            per_skill_price_limits: HashMap::new(),
            skill_price_history: HashMap::new(),
            price_updater,
            sales_this_step: HashMap::new(),
            cache: MarketStatsCache::default(),
        }
    }

    /// Adds a skill to the market.
    ///
    /// Initializes supply/demand counters and price history tracking for the skill.
    /// The cache is invalidated as adding a skill changes aggregate statistics.
    ///
    /// # Arguments
    ///
    /// * `skill` - The skill to add to the market
    pub fn add_skill(&mut self, skill: Skill) {
        self.supply_counts.insert(skill.id.clone(), 0);
        self.demand_counts.insert(skill.id.clone(), 0);
        self.skill_price_history
            .insert(skill.id.clone(), Vec::new());
        self.skills.insert(skill.id.clone(), skill);
        // Invalidate cache since adding a skill changes aggregate statistics
        self.invalidate_cache();
    }

    /// Increments the supply counter for a skill.
    ///
    /// Should be called once for each person who provides this skill.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - Identifier of the skill
    pub fn increment_skill_supply(&mut self, skill_id: &SkillId) {
        *self.supply_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    /// Resets demand counters and sales tracking for a new simulation step.
    ///
    /// This should be called at the beginning of each simulation step before
    /// collecting new demand information.
    pub fn reset_demand_counts(&mut self) {
        for demand_count in self.demand_counts.values_mut() {
            *demand_count = 0;
        }
        self.sales_this_step.clear();
    }

    /// Increments the demand counter for a skill.
    ///
    /// Should be called once for each person who needs this skill.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - Identifier of the skill
    pub fn increment_demand(&mut self, skill_id: &SkillId) {
        *self.demand_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    /// Gets the current price of a skill.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - Identifier of the skill
    ///
    /// # Returns
    ///
    /// * `Some(price)` if the skill exists, `None` otherwise
    pub fn get_price(&self, skill_id: &SkillId) -> Option<f64> {
        self.skills.get(skill_id).map(|s| s.current_price)
    }

    /// Gets the efficiency multiplier of a skill.
    ///
    /// The efficiency multiplier represents technological progress and productivity improvements.
    /// Higher efficiency means the skill provides more value per unit cost.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - Identifier of the skill
    ///
    /// # Returns
    ///
    /// * `f64` - The efficiency multiplier (defaults to 1.0 if skill not found)
    pub fn get_skill_efficiency(&self, skill_id: &SkillId) -> f64 {
        self.skills
            .get(skill_id)
            .map(|s| s.efficiency_multiplier)
            .unwrap_or(1.0)
    }

    /// Updates all skill prices based on current supply, demand, and the configured pricing strategy.
    ///
    /// This method delegates to the configured [`PriceUpdater`] to perform the actual
    /// price adjustments. Price history is automatically recorded.
    /// The statistics cache is invalidated after price updates.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator for adding volatility (if applicable)
    pub fn update_prices<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let updater = self.price_updater.clone();
        updater.update_prices(self, rng);
        // Invalidate cache after price update
        self.invalidate_cache();
    }

    /// Invalidates the cached market statistics.
    ///
    /// This should be called whenever skill prices change to ensure cache consistency.
    fn invalidate_cache(&mut self) {
        self.cache.average_price = None;
        self.cache.total_market_value = None;
        self.cache.min_price = None;
        self.cache.max_price = None;
    }

    /// Gets the average price across all skills in the market.
    ///
    /// This method uses caching to avoid redundant calculations.
    /// The cache is automatically invalidated when prices change.
    ///
    /// # Returns
    ///
    /// The average price, or 0.0 if there are no skills in the market
    pub fn get_average_price(&mut self) -> f64 {
        if let Some(avg) = self.cache.average_price {
            return avg;
        }

        let avg = if self.skills.is_empty() {
            0.0
        } else {
            let sum: f64 = self.skills.values().map(|s| s.current_price).sum();
            sum / self.skills.len() as f64
        };

        self.cache.average_price = Some(avg);
        avg
    }

    /// Gets the total market value (sum of all skill prices).
    ///
    /// This method uses caching to avoid redundant calculations.
    /// The cache is automatically invalidated when prices change.
    ///
    /// # Returns
    ///
    /// The total market value
    pub fn get_total_market_value(&mut self) -> f64 {
        if let Some(total) = self.cache.total_market_value {
            return total;
        }

        let total: f64 = self.skills.values().map(|s| s.current_price).sum();
        self.cache.total_market_value = Some(total);
        total
    }

    /// Gets the minimum and maximum skill prices in the market.
    ///
    /// This method uses caching to avoid redundant calculations.
    /// The cache is automatically invalidated when prices change.
    ///
    /// # Returns
    ///
    /// A tuple of (min_price, max_price), or (0.0, 0.0) if there are no skills
    pub fn get_price_range(&mut self) -> (f64, f64) {
        // Check if both cached values exist
        if let (Some(min), Some(max)) = (self.cache.min_price, self.cache.max_price) {
            return (min, max);
        }

        // Compute min and max in a single pass
        let (min, max) = if self.skills.is_empty() {
            (0.0, 0.0)
        } else {
            self.skills
                .values()
                .map(|s| s.current_price)
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), price| {
                    (min.min(price), max.max(price))
                })
        };

        // Cache the values
        self.cache.min_price = Some(min);
        self.cache.max_price = Some(max);

        (min, max)
    }

    /// Gets all current skill prices as a map.
    ///
    /// # Returns
    ///
    /// A HashMap mapping skill IDs to their current prices
    pub fn get_all_skill_prices(&self) -> HashMap<SkillId, f64> {
        self.skills
            .iter()
            .map(|(id, skill)| (id.clone(), skill.current_price))
            .collect()
    }

    /// Gets the effective minimum price for a specific skill.
    ///
    /// Returns the per-skill minimum if set, otherwise returns the global minimum.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The ID of the skill to check
    ///
    /// # Returns
    ///
    /// The effective minimum price for this skill
    pub fn get_effective_min_price(&self, skill_id: &SkillId) -> f64 {
        self.per_skill_price_limits
            .get(skill_id)
            .and_then(|(min, _max)| *min)
            .unwrap_or(self.min_skill_price)
    }

    /// Gets the effective maximum price for a specific skill.
    ///
    /// Returns the per-skill maximum if set, otherwise returns the global maximum.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The ID of the skill to check
    ///
    /// # Returns
    ///
    /// The effective maximum price for this skill
    pub fn get_effective_max_price(&self, skill_id: &SkillId) -> f64 {
        self.per_skill_price_limits
            .get(skill_id)
            .and_then(|(_min, max)| *max)
            .unwrap_or(self.max_skill_price)
    }

    /// Sets per-skill price limits for a specific skill.
    ///
    /// This allows for skill-specific regulatory interventions,
    /// enabling study of targeted price controls and market regulations.
    ///
    /// # Arguments
    ///
    /// * `skill_id` - The ID of the skill to set limits for
    /// * `min_price` - Optional minimum price (None = use global limit)
    /// * `max_price` - Optional maximum price (None = use global limit)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::{Market, Skill};
    /// use simulation_framework::scenario::{PriceUpdater, Scenario};
    ///
    /// let updater = PriceUpdater::from(Scenario::Original);
    /// let mut market = Market::new(10.0, 1.0, 0.1, 0.02, updater);
    ///
    /// let skill = Skill::new("Programming".to_string(), 50.0);
    /// let skill_id = skill.id.clone();
    /// market.add_skill(skill);
    ///
    /// // Set minimum price of 25.0 for this skill
    /// market.set_per_skill_price_limits(&skill_id, Some(25.0), None);
    /// ```
    pub fn set_per_skill_price_limits(
        &mut self,
        skill_id: &SkillId,
        min_price: Option<f64>,
        max_price: Option<f64>,
    ) {
        if min_price.is_some() || max_price.is_some() {
            self.per_skill_price_limits
                .insert(skill_id.clone(), (min_price, max_price));
        } else {
            // If both are None, remove the entry
            self.per_skill_price_limits.remove(skill_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;

    #[test]
    fn test_cache_average_price() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        // Add some skills with different prices
        let skill1 = Skill::new("Programming".to_string(), 50.0);
        let skill2 = Skill::new("Design".to_string(), 30.0);
        let skill3 = Skill::new("Writing".to_string(), 20.0);

        market.add_skill(skill1);
        market.add_skill(skill2);
        market.add_skill(skill3);

        // First call should compute and cache
        let avg1 = market.get_average_price();
        assert!((avg1 - 33.333333).abs() < 0.001); // (50 + 30 + 20) / 3

        // Second call should return cached value (same result)
        let avg2 = market.get_average_price();
        assert_eq!(avg1, avg2);

        // Verify cache was hit by checking it's still Some
        assert!(market.cache.average_price.is_some());
    }

    #[test]
    fn test_cache_invalidation_on_price_update() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        // Add skills
        let skill = Skill::new("Programming".to_string(), 50.0);
        market.add_skill(skill);

        // Compute average to populate cache
        let _avg = market.get_average_price();
        assert!(market.cache.average_price.is_some());

        // Update prices (which should invalidate cache)
        let mut rng = rand::thread_rng();
        market.update_prices(&mut rng);

        // Cache should be invalidated
        assert!(market.cache.average_price.is_none());
    }

    #[test]
    fn test_cache_total_market_value() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        // Add skills
        let skill1 = Skill::new("Skill1".to_string(), 15.0);
        let skill2 = Skill::new("Skill2".to_string(), 25.0);
        let skill3 = Skill::new("Skill3".to_string(), 10.0);

        market.add_skill(skill1);
        market.add_skill(skill2);
        market.add_skill(skill3);

        // First call computes and caches
        let total1 = market.get_total_market_value();
        assert_eq!(total1, 50.0); // 15 + 25 + 10

        // Second call returns cached value
        let total2 = market.get_total_market_value();
        assert_eq!(total1, total2);
    }

    #[test]
    fn test_cache_price_range() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        // Add skills with different prices
        let skill1 = Skill::new("Expensive".to_string(), 100.0);
        let skill2 = Skill::new("Medium".to_string(), 50.0);
        let skill3 = Skill::new("Cheap".to_string(), 10.0);

        market.add_skill(skill1);
        market.add_skill(skill2);
        market.add_skill(skill3);

        // First call computes and caches
        let (min1, max1) = market.get_price_range();
        assert_eq!(min1, 10.0);
        assert_eq!(max1, 100.0);

        // Second call returns cached values
        let (min2, max2) = market.get_price_range();
        assert_eq!(min1, min2);
        assert_eq!(max1, max2);

        // Verify cache was populated
        assert!(market.cache.min_price.is_some());
        assert!(market.cache.max_price.is_some());
    }

    #[test]
    fn test_empty_market_statistics() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        // Test empty market
        let avg = market.get_average_price();
        assert_eq!(avg, 0.0);

        let total = market.get_total_market_value();
        assert_eq!(total, 0.0);

        let (min, max) = market.get_price_range();
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
    }

    #[test]
    fn test_cache_consistency_after_multiple_invalidations() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        let skill = Skill::new("TestSkill".to_string(), 30.0);
        market.add_skill(skill);

        // Get initial cached value
        let _avg1 = market.get_average_price();

        // Invalidate and recompute multiple times
        let mut rng = rand::thread_rng();
        for _ in 0..5 {
            market.update_prices(&mut rng);
            let _avg = market.get_average_price(); // Recompute after each invalidation
        }

        // Cache should still be functioning correctly
        let avg_final = market.get_average_price();
        assert!(avg_final > 0.0); // Price should still be positive
        assert!(market.cache.average_price.is_some());
    }

    #[test]
    fn test_per_skill_price_limits_basic() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        let skill = Skill::new("Programming".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        // Initially should use global limits
        assert_eq!(market.get_effective_min_price(&skill_id), 1.0);
        assert_eq!(market.get_effective_max_price(&skill_id), 1000.0);

        // Set per-skill limits
        market.set_per_skill_price_limits(&skill_id, Some(25.0), Some(100.0));

        // Should now use per-skill limits
        assert_eq!(market.get_effective_min_price(&skill_id), 25.0);
        assert_eq!(market.get_effective_max_price(&skill_id), 100.0);
    }

    #[test]
    fn test_per_skill_price_limits_partial() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        let skill = Skill::new("Design".to_string(), 30.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        // Set only minimum
        market.set_per_skill_price_limits(&skill_id, Some(15.0), None);
        assert_eq!(market.get_effective_min_price(&skill_id), 15.0);
        assert_eq!(market.get_effective_max_price(&skill_id), 1000.0); // Global max

        // Set only maximum
        market.set_per_skill_price_limits(&skill_id, None, Some(75.0));
        assert_eq!(market.get_effective_min_price(&skill_id), 1.0); // Global min
        assert_eq!(market.get_effective_max_price(&skill_id), 75.0);
    }

    #[test]
    fn test_per_skill_price_limits_removal() {
        let price_updater = PriceUpdater::from(Scenario::Original);
        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, price_updater);

        let skill = Skill::new("Writing".to_string(), 20.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        // Set per-skill limits
        market.set_per_skill_price_limits(&skill_id, Some(10.0), Some(50.0));
        assert_eq!(market.get_effective_min_price(&skill_id), 10.0);
        assert_eq!(market.get_effective_max_price(&skill_id), 50.0);

        // Remove limits by setting both to None
        market.set_per_skill_price_limits(&skill_id, None, None);
        assert_eq!(market.get_effective_min_price(&skill_id), 1.0); // Back to global
        assert_eq!(market.get_effective_max_price(&skill_id), 1000.0); // Back to global
    }
}
