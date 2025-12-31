use crate::market::Market;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use strum_macros::EnumString;

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
    use crate::market::Market;
    use crate::skill::Skill;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_original_price_updater_price_increase() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::Original));
        market.price_elasticity_factor = 0.1;
        market.volatility_percentage = 0.0; // Disable volatility for predictable test

        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        market.demand_counts.insert(skill_id.clone(), 10);
        market.supply_counts.insert(skill_id.clone(), 5);

        let mut rng = StepRng::new(2, 1);
        let updater = OriginalPriceUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert!(new_price > 50.0);
    }

    #[test]
    fn test_original_price_updater_price_decrease() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::Original));
        market.price_elasticity_factor = 0.1;
        market.volatility_percentage = 0.0; // Disable volatility for predictable test

        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        market.demand_counts.insert(skill_id.clone(), 5);
        market.supply_counts.insert(skill_id.clone(), 10);

        let mut rng = StepRng::new(2, 1);
        let updater = OriginalPriceUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert!(new_price < 50.0);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_increase() {
        let mut market = Market::new(10.0, PriceUpdater::from(Scenario::DynamicPricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);
        market.sales_this_step.insert(skill_id.clone(), 1);

        let mut rng = StepRng::new(2, 1);
        let updater = DynamicPricingUpdater;
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
        let updater = DynamicPricingUpdater;
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
        let updater = DynamicPricingUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert_eq!(new_price, 1.0);
    }
}

/// Enum representing different price update strategies.
///
/// Each variant wraps a specific updater implementation that defines how
/// skill prices change in response to market conditions.
///
/// # Variants
///
/// * `Original` - Supply/demand-based pricing with random volatility
/// * `DynamicPricing` - Sales-based pricing that increases/decreases based on purchases
///
/// # Examples
///
/// ```
/// use simulation_framework::scenario::{PriceUpdater, Scenario};
///
/// let updater = PriceUpdater::from(Scenario::Original);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceUpdater {
    Original(OriginalPriceUpdater),
    DynamicPricing(DynamicPricingUpdater),
}

impl Default for PriceUpdater {
    fn default() -> Self {
        PriceUpdater::Original(OriginalPriceUpdater)
    }
}

impl PriceUpdater {
    /// Updates skill prices in the market using the configured strategy.
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing skills whose prices should be updated
    /// * `rng` - Random number generator for adding volatility (if applicable)
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
            Scenario::Original => PriceUpdater::Original(OriginalPriceUpdater),
            Scenario::DynamicPricing => PriceUpdater::DynamicPricing(DynamicPricingUpdater),
        }
    }
}

/// Price updater for the Original scenario.
///
/// This updater adjusts prices based on the ratio of demand to supply:
/// - When demand > supply: prices increase
/// - When demand < supply: prices decrease
/// - Adds random volatility for market realism
/// - Enforces min/max price boundaries
///
/// The adjustment magnitude is controlled by the market's `price_elasticity_factor`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OriginalPriceUpdater;

impl OriginalPriceUpdater {
    /// Updates skill prices based on supply and demand dynamics.
    ///
    /// # Algorithm
    ///
    /// 1. Calculate demand/supply ratio for each skill
    /// 2. Adjust price by `(ratio - 1.0) * elasticity_factor`
    /// 3. Add random fluctuation based on `volatility_percentage`
    /// 4. Clamp to min/max price boundaries
    /// 5. Record price in history
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing skills to update
    /// * `rng` - Random number generator for volatility
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, rng: &mut R) {
        for (skill_id, skill) in market.skills.iter_mut() {
            let demand = *market.demand_counts.get(skill_id).unwrap_or(&0) as f64;
            let supply = (*market.supply_counts.get(skill_id).unwrap_or(&1)).max(1) as f64;

            let mut new_price = skill.current_price;

            let demand_supply_ratio = demand / supply;

            let price_adjustment_factor =
                1.0 + (demand_supply_ratio - 1.0) * market.price_elasticity_factor;
            let demand_driven_price = new_price * price_adjustment_factor;

            new_price = demand_driven_price;

            let price_range_for_volatility = new_price * market.volatility_percentage;
            let random_fluctuation =
                rng.gen_range(-price_range_for_volatility..=price_range_for_volatility);
            new_price += random_fluctuation;

            new_price = new_price
                .max(market.min_skill_price)
                .min(market.max_skill_price);

            skill.current_price = new_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}

/// Price updater for the DynamicPricing scenario.
///
/// This updater adjusts prices based on sales activity:
/// - If a skill was sold in the current step: price increases by 5%
/// - If a skill was not sold: price decreases by 5%
/// - Enforces min/max price boundaries
///
/// This creates a simpler, more direct feedback mechanism compared to
/// the supply/demand approach.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicPricingUpdater;

impl DynamicPricingUpdater {
    /// Updates skill prices based on sales activity.
    ///
    /// # Algorithm
    ///
    /// 1. Check if skill was sold in current step
    /// 2. If sold: increase price by 5%
    /// 3. If not sold: decrease price by 5%
    /// 4. Clamp to min/max price boundaries
    /// 5. Record price in history
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing skills to update
    /// * `_rng` - Random number generator (unused in this strategy)
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
            new_price = new_price
                .max(market.min_skill_price)
                .min(market.max_skill_price);

            skill.current_price = new_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}
