use crate::market::Market;
use rand::Rng;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use std::fmt::{Debug, Display};

/// Defines the different simulation scenarios that can be run.
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Default)]
pub enum Scenario {
    /// The original simulation scenario, where prices are determined by supply and demand.
    #[default]
    Original,
    /// A scenario where prices change based on sales history.
    DynamicPricing,
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scenario::Original => write!(f, "Original"),
            Scenario::DynamicPricing => write!(f, "DynamicPricing"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Skill, SkillId};
    use crate::market::Market;
    use std::collections::HashMap;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_dynamic_pricing_updater_price_increase() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::DynamicPricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);
        market.sales_this_step.insert(skill_id.clone(), 1);

        let mut rng = StepRng::new(2, 1);
        let updater = DynamicPricingUpdater::default();
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert_eq!(new_price, 52.5);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_decrease() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::DynamicPricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = DynamicPricingUpdater::default();
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert_eq!(new_price, 47.5);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_clamp() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::DynamicPricing));
        let skill = Skill::new("Test Skill".to_string(), 1.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = DynamicPricingUpdater::default();
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert_eq!(new_price, 1.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceUpdater {
    Original(OriginalPriceUpdater),
    DynamicPricing(DynamicPricingUpdater),
}

impl Default for PriceUpdater {
    fn default() -> Self {
        PriceUpdater::Original(OriginalPriceUpdater::default())
    }
}

impl PriceUpdater {
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, rng: &mut R) {
        match self {
            PriceUpdater::Original(updater) => updater.update_prices(market, rng),
            PriceUpdater::DynamicPricing(updater) => updater.update_prices(market, rng),
        }
    }
}

impl From<Scenario> for PriceUpdater {
    fn from(scenario: Scenario) -> Self {
        match scenario {
            Scenario::Original => PriceUpdater::Original(OriginalPriceUpdater::default()),
            Scenario::DynamicPricing => PriceUpdater::DynamicPricing(DynamicPricingUpdater::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OriginalPriceUpdater;

impl OriginalPriceUpdater {
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, rng: &mut R) {
        for (skill_id, skill) in market.skills.iter_mut() {
            let demand = *market.demand_counts.get(skill_id).unwrap_or(&0) as f64;
            let supply = (*market.supply_counts.get(skill_id).unwrap_or(&1)).max(1) as f64;

            let mut new_price = skill.current_price;

            let demand_supply_ratio = demand / supply;

            let price_adjustment_factor = 1.0 + (demand_supply_ratio - 1.0) * market.price_elasticity_factor;
            let demand_driven_price = new_price * price_adjustment_factor;

            new_price = demand_driven_price;

            let price_range_for_volatility = new_price * market.volatility_percentage;
            let random_fluctuation = rng.gen_range(-price_range_for_volatility..=price_range_for_volatility);
            new_price += random_fluctuation;

            new_price = new_price.max(market.min_skill_price).min(market.max_skill_price);

            skill.current_price = new_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicPricingUpdater;

impl DynamicPricingUpdater {
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, _rng: &mut R) {
        let price_change_rate = 0.05; // 5% price change per step

        for (skill_id, skill) in market.skills.iter_mut() {
            let sales_count = *market.sales_this_step.get(skill_id).unwrap_or(&0);

            let mut new_price = skill.current_price;

            if sales_count > 0 {
                // Increase price if the skill was sold
                new_price *= 1.0 + price_change_rate;
            } else {
                // Decrease price if the skill was not sold
                new_price *= 1.0 - price_change_rate;
            }

            // Clamp price to min/max boundaries
            new_price = new_price.max(market.min_skill_price).min(market.max_skill_price);

            skill.current_price = new_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}
