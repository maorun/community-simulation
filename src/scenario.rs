use crate::market::Market;
use log::debug;
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
    /// A scenario where prices adapt gradually based on sales with a learning rate.
    /// Uses exponential moving average for smoother price adjustments.
    AdaptivePricing,
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scenario::Original => write!(f, "Original"),
            Scenario::DynamicPricing => write!(f, "DynamicPricing"),
            Scenario::AdaptivePricing => write!(f, "AdaptivePricing"),
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
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::Original));
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
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::Original));
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
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::DynamicPricing));
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
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::DynamicPricing));
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
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::DynamicPricing));
        let skill = Skill::new("Test Skill".to_string(), 1.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = DynamicPricingUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        assert_eq!(new_price, 1.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_price_increase() {
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::AdaptivePricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);
        market.sales_this_step.insert(skill_id.clone(), 1);

        let mut rng = StepRng::new(2, 1);
        let updater = AdaptivePricingUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        // With learning rate 0.2, target is 55.0 (50 * 1.1)
        // new_price = 50 + 0.2 * (55 - 50) = 50 + 1 = 51.0
        assert_eq!(new_price, 51.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_price_decrease() {
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::AdaptivePricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = AdaptivePricingUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        // With learning rate 0.2, target is 45.0 (50 * 0.9)
        // new_price = 50 + 0.2 * (45 - 50) = 50 - 1 = 49.0
        assert_eq!(new_price, 49.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_smooth_adjustment() {
        let mut market = Market::new(10.0, 1.0, PriceUpdater::from(Scenario::AdaptivePricing));
        let skill = Skill::new("Test Skill".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = AdaptivePricingUpdater;

        // First step without sale
        updater.update_prices(&mut market, &mut rng);
        let price_after_1 = market.get_price(&skill_id).unwrap();
        assert_eq!(price_after_1, 49.0);

        // Second step without sale - should continue decreasing smoothly
        updater.update_prices(&mut market, &mut rng);
        let price_after_2 = market.get_price(&skill_id).unwrap();
        // target = 49 * 0.9 = 44.1, new = 49 + 0.2 * (44.1 - 49) = 49 - 0.98 = 48.02
        assert!((price_after_2 - 48.02).abs() < 0.01);
    }

    #[test]
    fn test_adaptive_pricing_updater_price_clamp() {
        let mut market = Market::new(1.0, 1.0, PriceUpdater::from(Scenario::AdaptivePricing));
        let skill = Skill::new("Test Skill".to_string(), 1.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        let mut rng = StepRng::new(2, 1);
        let updater = AdaptivePricingUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        // Should be clamped to min_skill_price (1.0)
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
/// * `AdaptivePricing` - Gradual price adaptation using exponential moving average
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
    AdaptivePricing(AdaptivePricingUpdater),
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
            PriceUpdater::AdaptivePricing(updater) => updater.update_prices(market, rng),
        }
    }
}

impl From<Scenario> for PriceUpdater {
    fn from(scenario: Scenario) -> Self {
        match scenario {
            Scenario::Original => PriceUpdater::Original(OriginalPriceUpdater),
            Scenario::DynamicPricing => PriceUpdater::DynamicPricing(DynamicPricingUpdater),
            Scenario::AdaptivePricing => PriceUpdater::AdaptivePricing(AdaptivePricingUpdater),
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

            let old_price = skill.current_price;
            new_price = new_price
                .max(market.min_skill_price)
                .min(market.max_skill_price);

            skill.current_price = new_price;

            debug!(
                "Original scenario price update: Skill {:?} ${:.2} -> ${:.2} (demand/supply: {:.2}/{:.2}, ratio: {:.2})",
                skill_id,
                old_price,
                new_price,
                demand,
                supply,
                demand_supply_ratio
            );

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

            let old_price = skill.current_price;
            let mut new_price = skill.current_price;

            if sales_count > 0 {
                // Increase price if the skill was sold
                new_price *= 1.0 + price_change_rate;
                debug!(
                    "DynamicPricing: Skill {:?} sold {} times, price ${:.2} -> ${:.2} (+{:.1}%)",
                    skill_id,
                    sales_count,
                    old_price,
                    new_price,
                    price_change_rate * 100.0
                );
            } else {
                // Decrease price if the skill was not sold
                new_price *= 1.0 - price_change_rate;
                debug!(
                    "DynamicPricing: Skill {:?} not sold, price ${:.2} -> ${:.2} (-{:.1}%)",
                    skill_id,
                    old_price,
                    new_price,
                    price_change_rate * 100.0
                );
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

/// Price updater for the AdaptivePricing scenario.
///
/// This updater gradually adjusts prices based on sales activity using a learning rate:
/// - Uses exponential moving average for smoother adjustments
/// - If a skill was sold: gradually increases price toward target (current * 1.1)
/// - If a skill was not sold: gradually decreases price toward target (current * 0.9)
/// - Learning rate of 0.2 means 20% of the difference is applied each step
/// - Enforces min/max price boundaries
///
/// This creates more stable price movements compared to DynamicPricing,
/// allowing the market to find equilibrium more naturally.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdaptivePricingUpdater;

impl AdaptivePricingUpdater {
    /// Updates skill prices using adaptive learning rate.
    ///
    /// # Algorithm
    ///
    /// 1. Check if skill was sold in current step
    /// 2. If sold: target = current_price * 1.1 (increase target)
    /// 3. If not sold: target = current_price * 0.9 (decrease target)
    /// 4. new_price = current_price + learning_rate * (target - current_price)
    /// 5. Clamp to min/max price boundaries
    /// 6. Record price in history
    ///
    /// The learning rate of 0.2 smooths out price changes, preventing
    /// rapid oscillations and allowing gradual convergence to market equilibrium.
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing skills to update
    /// * `_rng` - Random number generator (unused in this strategy)
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, _rng: &mut R) {
        let learning_rate = 0.2; // 20% adjustment per step
        let target_increase = 1.1; // Target 10% increase if sold
        let target_decrease = 0.9; // Target 10% decrease if not sold

        for (skill_id, skill) in market.skills.iter_mut() {
            let sales_count = *market.sales_this_step.get(skill_id).unwrap_or(&0);

            let current_price = skill.current_price;

            // Calculate target price based on sales
            let target_price = if sales_count > 0 {
                current_price * target_increase
            } else {
                current_price * target_decrease
            };

            // Apply exponential moving average for smooth adaptation
            let new_price = current_price + learning_rate * (target_price - current_price);

            // Clamp price to min/max boundaries
            let clamped_price = new_price
                .max(market.min_skill_price)
                .min(market.max_skill_price);

            debug!(
                "AdaptivePricing: Skill {:?} {} (sales: {}), price ${:.2} -> ${:.2} (target: ${:.2})",
                skill_id,
                if sales_count > 0 { "sold" } else { "not sold" },
                sales_count,
                current_price,
                clamped_price,
                target_price
            );

            skill.current_price = clamped_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(clamped_price);
            }
        }
    }
}
