use crate::skill::{Skill, SkillId};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::Rng; // For volatility

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub skills: HashMap<SkillId, Skill>,
    pub demand_counts: HashMap<SkillId, usize>,
    pub supply_counts: HashMap<SkillId, usize>,
    pub base_skill_price: f64,

    pub price_elasticity_factor: f64,
    pub volatility_percentage: f64,
    pub min_skill_price: f64,
    pub max_skill_price: f64,

    // For tracking price development over time
    // Key: SkillId, Value: Vec of prices, one entry per step
    #[serde(skip_serializing_if = "HashMap::is_empty")] // Don't serialize if we decide not to fill it
    pub skill_price_history: HashMap<SkillId, Vec<f64>>,
}

impl Market {
    pub fn new(base_skill_price: f64) -> Self {
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
        }
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.supply_counts.insert(skill.id.clone(), 0);
        self.demand_counts.insert(skill.id.clone(), 0);
        // Initialize price history vector for this skill
        self.skill_price_history.insert(skill.id.clone(), Vec::new());
        self.skills.insert(skill.id.clone(), skill);
    }

    pub fn increment_skill_supply(&mut self, skill_id: &SkillId) {
        *self.supply_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    pub fn reset_demand_counts(&mut self) {
        for demand_count in self.demand_counts.values_mut() {
            *demand_count = 0;
        }
    }

    pub fn increment_demand(&mut self, skill_id: &SkillId) {
        *self.demand_counts.entry(skill_id.clone()).or_insert(0) += 1;
    }

    pub fn get_price(&self, skill_id: &SkillId) -> Option<f64> {
        self.skills.get(skill_id).map(|s| s.current_price)
    }

    // Price adjustment logic refined
    pub fn update_prices<R: Rng>(&mut self, rng: &mut R) {
        for (skill_id, skill) in self.skills.iter_mut() {
            let demand = *self.demand_counts.get(skill_id).unwrap_or(&0) as f64;
            // Supply is fixed for now (1 per person offering the skill).
            // If we allowed variable supply (e.g. person chooses to work more/less), this would change.
            let supply = (*self.supply_counts.get(skill_id).unwrap_or(&1)).max(1) as f64; // Ensure supply is at least 1 to avoid division by zero.

            let mut new_price = skill.current_price;

            // Demand/Supply Ratio Effect
            // If demand > supply, price tends to increase.
            // If supply > demand, price tends to decrease.
            // A simple model: price changes proportionally to the ratio imbalance.
            // Example: if D/S ratio is 2 (demand twice supply), price increases.
            // If D/S ratio is 0.5 (supply twice demand), price decreases.

            let demand_supply_ratio = if supply > 0.0 { demand / supply } else { demand }; // If supply is 0, treat ratio as high if demand > 0

            // Calculate a target price based on demand/supply ratio relative to a neutral ratio of 1.0
            // If demand_supply_ratio > 1, price increases. If < 1, price decreases.
            // The change is smoothed by the elasticity factor.
            let price_adjustment_factor = 1.0 + (demand_supply_ratio - 1.0) * self.price_elasticity_factor;
            let demand_driven_price = new_price * price_adjustment_factor;

            new_price = demand_driven_price;

            // Add random volatility
            // Volatility is a percentage of the current demand-driven price.
            let price_range_for_volatility = new_price * self.volatility_percentage;
            let random_fluctuation = rng.gen_range(-price_range_for_volatility..=price_range_for_volatility);
            new_price += random_fluctuation;

            // Clamp price to min/max boundaries
            new_price = new_price.max(self.min_skill_price).min(self.max_skill_price);

            skill.current_price = new_price;

            // Record price history
            if let Some(history) = self.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }

    pub fn get_all_skill_prices(&self) -> HashMap<SkillId, f64> {
        self.skills.iter().map(|(id, skill)| (id.clone(), skill.current_price)).collect()
    }
}
