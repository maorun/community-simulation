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
/// let mut market = Market::new(10.0, price_updater);
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
}

impl Market {
    /// Creates a new market with the specified base price and pricing strategy.
    ///
    /// # Arguments
    ///
    /// * `base_skill_price` - Initial/reference price for skills
    /// * `price_updater` - Strategy to use for updating prices
    ///
    /// # Examples
    ///
    /// ```
    /// use simulation_framework::Market;
    /// use simulation_framework::scenario::{PriceUpdater, Scenario};
    ///
    /// let updater = PriceUpdater::from(Scenario::Original);
    /// let market = Market::new(10.0, updater);
    /// ```
    pub fn new(base_skill_price: f64, price_updater: PriceUpdater) -> Self {
        Market {
            skills: HashMap::new(),
            demand_counts: HashMap::new(),
            supply_counts: HashMap::new(),
            base_skill_price,
            price_elasticity_factor: 0.1,
            volatility_percentage: 0.02,
            min_skill_price: 1.0,
            max_skill_price: 1000.0,
            skill_price_history: HashMap::new(),
            price_updater,
            sales_this_step: HashMap::new(),
        }
    }

    /// Adds a skill to the market.
    ///
    /// Initializes supply/demand counters and price history tracking for the skill.
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

    /// Updates all skill prices based on current supply, demand, and the configured pricing strategy.
    ///
    /// This method delegates to the configured [`PriceUpdater`] to perform the actual
    /// price adjustments. Price history is automatically recorded.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator for adding volatility (if applicable)
    pub fn update_prices<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let updater = self.price_updater.clone();
        updater.update_prices(self, rng);
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
}
