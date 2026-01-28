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
    /// A scenario where prices increase when multiple buyers compete for the same skill,
    /// simulating an auction mechanism where competitive demand drives prices up.
    AuctionPricing,
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scenario::Original => write!(f, "Original"),
            Scenario::DynamicPricing => write!(f, "DynamicPricing"),
            Scenario::AdaptivePricing => write!(f, "AdaptivePricing"),
            Scenario::AuctionPricing => write!(f, "AuctionPricing"),
        }
    }
}

impl Scenario {
    /// Get all available scenarios
    pub fn all() -> Vec<Scenario> {
        vec![
            Scenario::Original,
            Scenario::DynamicPricing,
            Scenario::AdaptivePricing,
            Scenario::AuctionPricing,
        ]
    }

    /// Get a brief description of this scenario
    pub fn description(&self) -> &str {
        match self {
            Scenario::Original => "Supply/demand-based pricing with volatility",
            Scenario::DynamicPricing => "Sales-based pricing mechanism",
            Scenario::AdaptivePricing => {
                "Gradual price adaptation using exponential moving average"
            },
            Scenario::AuctionPricing => "Competitive bidding mechanism",
        }
    }

    /// Get the mechanism details of this scenario
    pub fn mechanism(&self) -> &str {
        match self {
            Scenario::Original => "Prices adjust based on the ratio of buyers to sellers",
            Scenario::DynamicPricing => "Prices increase 5% when sold, decrease 5% when not sold",
            Scenario::AdaptivePricing => "Smooth price adjustments with 20% learning rate",
            Scenario::AuctionPricing => "Prices increase aggressively when multiple buyers compete",
        }
    }

    /// Get the best use case for this scenario
    pub fn use_case(&self) -> &str {
        match self {
            Scenario::Original => "Studying natural market dynamics and equilibrium",
            Scenario::DynamicPricing => "Studying price discovery and market feedback",
            Scenario::AdaptivePricing => "Modeling gradual market learning and stability",
            Scenario::AuctionPricing => "Studying auction dynamics and competitive markets",
        }
    }

    /// Check if this is the default scenario
    pub fn is_default(&self) -> bool {
        matches!(self, Scenario::Original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::Market;
    use crate::skill::Skill;
    use rand::rngs::mock::StepRng;

    /// Helper function to create a market with common test defaults
    ///
    /// # Parameters
    /// * `scenario` - The pricing scenario to use
    /// * `base_price` - Base price for skills (min_skill_price parameter)
    ///
    /// # Returns
    /// A Market configured for deterministic testing with:
    /// * `price_change_factor` = 1.0 (default multiplier for price changes)
    /// * `price_elasticity_factor` = 0.1 (sensitivity to supply/demand)
    /// * `volatility_percentage` = 0.0 (disabled for predictable tests)
    fn create_test_market(scenario: Scenario, base_price: f64) -> Market {
        let mut market = Market::new(base_price, 1.0, 0.1, 0.02, PriceUpdater::from(scenario));
        market.price_elasticity_factor = 0.1;
        market.volatility_percentage = 0.0; // Disable volatility for predictable tests
        market
    }

    /// Helper function to setup a skill in a market and return the skill_id
    fn setup_skill_in_market(market: &mut Market, initial_price: f64) -> String {
        let skill = Skill::new("Test Skill".to_string(), initial_price);
        let skill_id = skill.id.clone();
        market.add_skill(skill);
        skill_id
    }

    /// Base price used for all test markets to ensure consistency
    const TEST_BASE_PRICE: f64 = 10.0;

    /// Configuration for a price updater test
    struct PriceUpdateTestConfig {
        initial_price: f64,
        updater: PriceUpdater,
        demand: Option<usize>,
        supply: Option<usize>,
        sales: Option<usize>,
    }

    impl PriceUpdateTestConfig {
        fn original(initial_price: f64, demand: usize, supply: usize) -> Self {
            Self {
                initial_price,
                updater: PriceUpdater::Original(OriginalPriceUpdater),
                demand: Some(demand),
                supply: Some(supply),
                sales: None,
            }
        }

        fn dynamic_pricing(initial_price: f64, sales: Option<usize>) -> Self {
            Self {
                initial_price,
                updater: PriceUpdater::DynamicPricing(DynamicPricingUpdater),
                demand: None,
                supply: None,
                sales,
            }
        }

        fn adaptive_pricing(initial_price: f64, sales: Option<usize>) -> Self {
            Self {
                initial_price,
                updater: PriceUpdater::AdaptivePricing(AdaptivePricingUpdater),
                demand: None,
                supply: None,
                sales,
            }
        }

        fn auction_pricing(initial_price: f64, demand: usize, supply: usize) -> Self {
            Self {
                initial_price,
                updater: PriceUpdater::AuctionPricing(AuctionPricingUpdater),
                demand: Some(demand),
                supply: Some(supply),
                sales: None,
            }
        }

        /// Derive the scenario from the updater type
        fn scenario(&self) -> Scenario {
            match self.updater {
                PriceUpdater::Original(_) => Scenario::Original,
                PriceUpdater::DynamicPricing(_) => Scenario::DynamicPricing,
                PriceUpdater::AdaptivePricing(_) => Scenario::AdaptivePricing,
                PriceUpdater::AuctionPricing(_) => Scenario::AuctionPricing,
            }
        }
    }

    /// Execute a price update test with the given configuration and return the new price
    ///
    /// This helper reduces test code duplication by encapsulating the common pattern:
    /// 1. Create market with scenario
    /// 2. Setup skill with initial price
    /// 3. Configure market conditions (demand, supply, sales)
    /// 4. Create deterministic RNG
    /// 5. Execute price update
    /// 6. Return the new price
    fn execute_price_update_test(config: PriceUpdateTestConfig) -> f64 {
        let mut market = create_test_market(config.scenario(), TEST_BASE_PRICE);
        let skill_id = setup_skill_in_market(&mut market, config.initial_price);

        // Configure market conditions
        if let Some(demand) = config.demand {
            market.demand_counts.insert(skill_id.clone(), demand);
        }
        if let Some(supply) = config.supply {
            market.supply_counts.insert(skill_id.clone(), supply);
        }
        if let Some(sales) = config.sales {
            market.sales_this_step.insert(skill_id.clone(), sales);
        }

        // Execute price update with deterministic RNG
        let mut rng = StepRng::new(2, 1);
        config.updater.update_prices(&mut market, &mut rng);

        // Return new price
        market.get_price(&skill_id).unwrap()
    }

    #[test]
    fn test_original_price_updater_price_increase() {
        let config = PriceUpdateTestConfig::original(50.0, 10, 5);
        let new_price = execute_price_update_test(config);
        assert!(new_price > 50.0);
    }

    #[test]
    fn test_original_price_updater_price_decrease() {
        let config = PriceUpdateTestConfig::original(50.0, 5, 10);
        let new_price = execute_price_update_test(config);
        assert!(new_price < 50.0);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_increase() {
        let config = PriceUpdateTestConfig::dynamic_pricing(50.0, Some(1));
        let new_price = execute_price_update_test(config);
        assert_eq!(new_price, 52.5);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_decrease() {
        let config = PriceUpdateTestConfig::dynamic_pricing(50.0, None);
        let new_price = execute_price_update_test(config);
        assert_eq!(new_price, 47.5);
    }

    #[test]
    fn test_dynamic_pricing_updater_price_clamp() {
        let config = PriceUpdateTestConfig::dynamic_pricing(1.0, None);
        let new_price = execute_price_update_test(config);
        assert_eq!(new_price, 1.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_price_increase() {
        let config = PriceUpdateTestConfig::adaptive_pricing(50.0, Some(1));
        let new_price = execute_price_update_test(config);
        // With learning rate 0.2, target is 55.0 (50 * 1.1)
        // new_price = 50 + 0.2 * (55 - 50) = 50 + 1 = 51.0
        assert_eq!(new_price, 51.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_price_decrease() {
        let config = PriceUpdateTestConfig::adaptive_pricing(50.0, None);
        let new_price = execute_price_update_test(config);
        // With learning rate 0.2, target is 45.0 (50 * 0.9)
        // new_price = 50 + 0.2 * (45 - 50) = 50 - 1 = 49.0
        assert_eq!(new_price, 49.0);
    }

    #[test]
    fn test_adaptive_pricing_updater_smooth_adjustment() {
        let mut market = create_test_market(Scenario::AdaptivePricing, 10.0);
        let skill_id = setup_skill_in_market(&mut market, 50.0);

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
        let config = PriceUpdateTestConfig::adaptive_pricing(1.0, None);
        let new_price = execute_price_update_test(config);
        // Should be clamped to min_skill_price (1.0)
        assert_eq!(new_price, 1.0);
    }

    #[test]
    fn test_auction_pricing_competitive_demand() {
        // High demand (10) vs low supply (2) = competitive bidding
        let config = PriceUpdateTestConfig::auction_pricing(50.0, 10, 2);
        let new_price = execute_price_update_test(config);
        // Price should increase significantly due to competitive bidding
        assert!(new_price > 50.0);
        assert!(new_price > 55.0); // Should be at least 10% increase
    }

    #[test]
    fn test_auction_pricing_no_demand() {
        // No demand at all
        let config = PriceUpdateTestConfig::auction_pricing(50.0, 0, 5);
        let new_price = execute_price_update_test(config);
        // Price should decrease faster (8%) when no demand
        assert!(new_price < 50.0);
        assert!(new_price < 47.0); // Should be roughly 8% decrease
    }

    #[test]
    fn test_auction_pricing_low_demand() {
        // Low demand (2) vs supply (5) = no competition
        let config = PriceUpdateTestConfig::auction_pricing(50.0, 2, 5);
        let new_price = execute_price_update_test(config);
        // Price should decrease gently (3%) when demand < supply but > 0
        // Account for 2% random volatility (max ±1.0 from 48.5)
        assert!(new_price < 50.0);
        assert!(new_price > 47.0); // Should be roughly 3% decrease ±2% volatility
    }

    #[test]
    fn test_auction_pricing_price_clamp() {
        // No demand, should decrease but be clamped at min_skill_price
        let config = PriceUpdateTestConfig::auction_pricing(1.0, 0, 5);
        let new_price = execute_price_update_test(config);
        // Should be clamped to min_skill_price (1.0)
        assert_eq!(new_price, 1.0);
    }

    #[test]
    fn test_per_skill_price_limits_enforcement() {
        // Test that per-skill price limits are enforced during price updates
        let mut market = create_test_market(Scenario::Original, 10.0);
        let skill_id = setup_skill_in_market(&mut market, 50.0);

        // Set per-skill limits: min 30.0, max 70.0
        market.set_per_skill_price_limits(&skill_id, Some(30.0), Some(70.0));

        // Simulate high demand to push price above max
        market.demand_counts.insert(skill_id.clone(), 20);
        market.supply_counts.insert(skill_id.clone(), 1);

        let mut rng = StepRng::new(2, 1);
        let updater = OriginalPriceUpdater;
        updater.update_prices(&mut market, &mut rng);

        let new_price = market.get_price(&skill_id).unwrap();
        // Price should be clamped to per-skill max (70.0), not global max (1000.0)
        assert!(new_price <= 70.0, "Price {} should be clamped to 70.0", new_price);

        // Now simulate low demand to push price below min
        market.demand_counts.insert(skill_id.clone(), 0);
        market.supply_counts.insert(skill_id.clone(), 20);

        updater.update_prices(&mut market, &mut rng);
        let new_price = market.get_price(&skill_id).unwrap();
        // Price should be clamped to per-skill min (30.0), not global min (1.0)
        assert!(new_price >= 30.0, "Price {} should be clamped to 30.0", new_price);
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
/// * `AuctionPricing` - Competitive bidding mechanism where demand intensity drives prices
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
    AuctionPricing(AuctionPricingUpdater),
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
            PriceUpdater::AuctionPricing(updater) => updater.update_prices(market, rng),
        }
    }
}

impl From<Scenario> for PriceUpdater {
    fn from(scenario: Scenario) -> Self {
        match scenario {
            Scenario::Original => PriceUpdater::Original(OriginalPriceUpdater),
            Scenario::DynamicPricing => PriceUpdater::DynamicPricing(DynamicPricingUpdater),
            Scenario::AdaptivePricing => PriceUpdater::AdaptivePricing(AdaptivePricingUpdater),
            Scenario::AuctionPricing => PriceUpdater::AuctionPricing(AuctionPricingUpdater),
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
            // Cache price limits to avoid duplicate HashMap lookups for better performance
            let (min_opt, max_opt) = market
                .per_skill_price_limits
                .get(skill_id)
                .map(|(min, max)| (*min, *max))
                .unwrap_or((None, None));
            let min_price = min_opt.unwrap_or(market.min_skill_price);
            let max_price = max_opt.unwrap_or(market.max_skill_price);

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
                rng.random_range(-price_range_for_volatility..=price_range_for_volatility);
            new_price += random_fluctuation;

            let old_price = skill.current_price;
            // Apply per-skill price limits (if set) or global limits
            new_price = new_price.max(min_price).min(max_price);

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
            // Cache price limits to avoid duplicate HashMap lookups for better performance
            let (min_opt, max_opt) = market
                .per_skill_price_limits
                .get(skill_id)
                .map(|(min, max)| (*min, *max))
                .unwrap_or((None, None));
            let min_price = min_opt.unwrap_or(market.min_skill_price);
            let max_price = max_opt.unwrap_or(market.max_skill_price);

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

            // Clamp price to per-skill or global boundaries
            new_price = new_price.max(min_price).min(max_price);

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
            // Cache price limits to avoid duplicate HashMap lookups for better performance
            let (min_opt, max_opt) = market
                .per_skill_price_limits
                .get(skill_id)
                .map(|(min, max)| (*min, *max))
                .unwrap_or((None, None));
            let min_price = min_opt.unwrap_or(market.min_skill_price);
            let max_price = max_opt.unwrap_or(market.max_skill_price);

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

            // Clamp price to per-skill or global boundaries
            let clamped_price = new_price.max(min_price).min(max_price);

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

/// Price updater for the AuctionPricing scenario.
///
/// This updater simulates an auction mechanism where prices increase when multiple
/// buyers compete for the same skill, creating a bidding war effect:
/// - High demand relative to supply (many buyers): price increases significantly
/// - Low demand: price decreases gradually
/// - No demand: price decreases more rapidly (unsold inventory)
/// - Includes small random volatility for realism
/// - Enforces min/max price boundaries
///
/// The adjustment is more aggressive than Original scenario when demand is competitive,
/// simulating the psychological effect of auction bidding where prices can spike
/// when multiple parties want the same resource.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuctionPricingUpdater;

impl AuctionPricingUpdater {
    /// Updates skill prices based on auction-style competitive demand.
    ///
    /// # Algorithm
    ///
    /// 1. Calculate demand/supply ratio for each skill
    /// 2. If demand > supply (competitive): apply aggressive price increase
    ///    - Use quadratic factor to simulate bidding war intensity
    ///    - Higher competition leads to exponentially higher prices
    /// 3. If demand <= supply: apply moderate decrease
    /// 4. If demand == 0: apply faster decrease (no interest)
    /// 5. Add small random volatility
    /// 6. Clamp to min/max price boundaries
    /// 7. Record price in history
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing skills to update
    /// * `rng` - Random number generator for volatility
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, rng: &mut R) {
        for (skill_id, skill) in market.skills.iter_mut() {
            // Cache price limits to avoid duplicate HashMap lookups for better performance
            let (min_opt, max_opt) = market
                .per_skill_price_limits
                .get(skill_id)
                .map(|(min, max)| (*min, *max))
                .unwrap_or((None, None));
            let min_price = min_opt.unwrap_or(market.min_skill_price);
            let max_price = max_opt.unwrap_or(market.max_skill_price);

            let demand = *market.demand_counts.get(skill_id).unwrap_or(&0) as f64;
            let supply = (*market.supply_counts.get(skill_id).unwrap_or(&1)).max(1) as f64;

            let old_price = skill.current_price;
            let mut new_price = old_price;

            if demand > supply {
                // Competitive bidding: price increases based on competition intensity
                // Use quadratic factor to simulate bidding war psychology
                let competition_factor = demand / supply;
                // Aggressive increase: base 10% + additional based on competition squared
                let increase_rate = 0.10 + (competition_factor - 1.0).powi(2) * 0.05;
                new_price *= 1.0 + increase_rate.min(0.30); // Cap at 30% increase per step

                debug!(
                    "AuctionPricing: Skill {:?} competitive bidding (demand/supply: {:.2}/{:.2}), price ${:.2} -> ${:.2} (+{:.1}%)",
                    skill_id,
                    demand,
                    supply,
                    old_price,
                    new_price,
                    increase_rate * 100.0
                );
            } else if demand == 0.0 {
                // No demand: faster price decrease (unsold inventory)
                new_price *= 0.92; // 8% decrease

                debug!(
                    "AuctionPricing: Skill {:?} no demand, price ${:.2} -> ${:.2} (-8.0%)",
                    skill_id, old_price, new_price
                );
            } else {
                // Low/moderate demand: gentle price decrease
                new_price *= 0.97; // 3% decrease

                debug!(
                    "AuctionPricing: Skill {:?} low demand (demand/supply: {:.2}/{:.2}), price ${:.2} -> ${:.2} (-3.0%)",
                    skill_id,
                    demand,
                    supply,
                    old_price,
                    new_price
                );
            }

            // Add small random volatility (2% max fluctuation)
            let price_range_for_volatility = new_price * 0.02;
            let random_fluctuation =
                rng.random_range(-price_range_for_volatility..=price_range_for_volatility);
            new_price += random_fluctuation;

            // Clamp price to per-skill or global boundaries
            new_price = new_price.max(min_price).min(max_price);

            skill.current_price = new_price;

            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}

// ============================================================================
// Demand Generation Strategies
// ============================================================================

/// Strategy for generating demand (number of needed skills per person).
///
/// Different demand generation strategies create different market dynamics:
/// - Uniform: All persons have similar demand levels (balanced market)
/// - Concentrated: Some persons have high demand, others low (unequal market)
/// - Cyclical: Demand varies periodically over time (dynamic market)
///
/// This trait enables experimentation with different demand patterns to study
/// their effects on market behavior, wealth distribution, and economic activity.
pub trait DemandGeneratorTrait: Send + Sync + Debug {
    /// Generate the number of skills a person needs in the current step.
    ///
    /// # Arguments
    ///
    /// * `person_id` - Unique identifier of the person requesting skills
    /// * `step` - Current simulation step number
    /// * `rng` - Random number generator for stochastic behavior
    ///
    /// # Returns
    ///
    /// Number of skills this person should need (typically 1-5)
    fn generate_demand_count<R: Rng + ?Sized>(
        &self,
        person_id: usize,
        step: usize,
        rng: &mut R,
    ) -> usize;
}

/// Demand generation strategy types.
///
/// Each variant represents a different approach to generating demand:
/// - `Uniform`: Random uniform distribution (baseline)
/// - `Concentrated`: Pareto-like distribution (inequality)
/// - `Cyclical`: Time-varying cyclical demand (dynamics)
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Default)]
#[strum(serialize_all = "PascalCase")]
pub enum DemandStrategy {
    /// Uniform random distribution: each person has 2-5 needs with equal probability.
    /// This is the default strategy that maintains current behavior.
    #[default]
    Uniform,
    /// Concentrated distribution: Most persons have low demand (2-3), few have high (4-5).
    /// Uses Pareto principle to simulate markets with unequal demand patterns.
    Concentrated,
    /// Cyclical distribution: Demand varies over time in a sine wave pattern.
    /// Creates periodic market dynamics with expansion and contraction phases.
    Cyclical,
}

impl std::fmt::Display for DemandStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DemandStrategy::Uniform => write!(f, "Uniform"),
            DemandStrategy::Concentrated => write!(f, "Concentrated"),
            DemandStrategy::Cyclical => write!(f, "Cyclical"),
        }
    }
}

/// Enum wrapping different demand generator implementations.
///
/// Provides a unified interface for demand generation while allowing
/// different strategies to be selected at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemandGenerator {
    Uniform(UniformDemandGenerator),
    Concentrated(ConcentratedDemandGenerator),
    Cyclical(CyclicalDemandGenerator),
}

impl Default for DemandGenerator {
    fn default() -> Self {
        DemandGenerator::Uniform(UniformDemandGenerator)
    }
}

impl DemandGenerator {
    /// Generate demand count using the configured strategy.
    pub fn generate_demand_count<R: Rng + ?Sized>(
        &self,
        person_id: usize,
        step: usize,
        rng: &mut R,
    ) -> usize {
        match self {
            DemandGenerator::Uniform(gen) => gen.generate_demand_count(person_id, step, rng),
            DemandGenerator::Concentrated(gen) => gen.generate_demand_count(person_id, step, rng),
            DemandGenerator::Cyclical(gen) => gen.generate_demand_count(person_id, step, rng),
        }
    }
}

impl From<DemandStrategy> for DemandGenerator {
    fn from(strategy: DemandStrategy) -> Self {
        match strategy {
            DemandStrategy::Uniform => DemandGenerator::Uniform(UniformDemandGenerator),
            DemandStrategy::Concentrated => {
                DemandGenerator::Concentrated(ConcentratedDemandGenerator)
            },
            DemandStrategy::Cyclical => DemandGenerator::Cyclical(CyclicalDemandGenerator),
        }
    }
}

/// Uniform demand generator - baseline strategy.
///
/// Generates demand uniformly distributed between 2 and 5 skills per person.
/// This maintains the current simulation behavior and provides a balanced baseline.
///
/// # Characteristics
/// - Equal probability for 2, 3, 4, or 5 needs
/// - No variation over time or between persons
/// - Stable, predictable market dynamics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniformDemandGenerator;

impl DemandGeneratorTrait for UniformDemandGenerator {
    fn generate_demand_count<R: Rng + ?Sized>(
        &self,
        _person_id: usize,
        _step: usize,
        rng: &mut R,
    ) -> usize {
        rng.random_range(2..=5)
    }
}

/// Concentrated demand generator - inequality strategy.
///
/// Uses a Pareto-like distribution where most persons have low demand (2-3 needs)
/// and fewer persons have high demand (4-5 needs). This simulates markets with
/// unequal consumption patterns.
///
/// # Characteristics
/// - 70% of persons: 2-3 needs (low consumers)
/// - 30% of persons: 4-5 needs (high consumers)
/// - Creates demand inequality alongside wealth inequality
/// - Tests market resilience to concentrated demand
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcentratedDemandGenerator;

impl DemandGeneratorTrait for ConcentratedDemandGenerator {
    fn generate_demand_count<R: Rng + ?Sized>(
        &self,
        _person_id: usize,
        _step: usize,
        rng: &mut R,
    ) -> usize {
        // 70% chance of low demand (2-3), 30% chance of high demand (4-5)
        let roll: f64 = rng.random();
        if roll < 0.7 {
            rng.random_range(2..=3) // Low demand
        } else {
            rng.random_range(4..=5) // High demand
        }
    }
}

/// Cyclical demand generator - dynamic strategy.
///
/// Demand varies over time in a sine wave pattern, creating periodic expansion
/// and contraction phases. This simulates business cycles and seasonal variation
/// at the aggregate demand level.
///
/// # Characteristics
/// - Demand oscillates between 2 and 5 needs
/// - Period of 100 steps (configurable via constant)
/// - Phase offset per person creates variety
/// - Tests market adaptation to changing conditions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CyclicalDemandGenerator;

impl DemandGeneratorTrait for CyclicalDemandGenerator {
    fn generate_demand_count<R: Rng + ?Sized>(
        &self,
        person_id: usize,
        step: usize,
        _rng: &mut R,
    ) -> usize {
        const CYCLE_PERIOD: f64 = 100.0;
        const MIN_DEMAND: usize = 2;
        const MAX_DEMAND: usize = 5;
        const PHASE_OFFSET_MULTIPLIER: f64 = 0.1;
        const FULL_CYCLE_MULTIPLIER: f64 = 2.0; // Full sine wave cycle

        // Calculate phase offset based on person_id for variety
        let phase_offset = (person_id as f64) * PHASE_OFFSET_MULTIPLIER;

        // Calculate current position in cycle
        let cycle_position = (step as f64 + phase_offset) / CYCLE_PERIOD;
        let sine_value = (cycle_position * FULL_CYCLE_MULTIPLIER * std::f64::consts::PI).sin();

        // Map sine wave [-1, 1] to demand range [MIN_DEMAND, MAX_DEMAND]
        let normalized = (sine_value + 1.0) / 2.0; // Map to [0, 1]
        let demand_range = (MAX_DEMAND - MIN_DEMAND) as f64;
        let demand = MIN_DEMAND + (normalized * demand_range).round() as usize;

        demand.clamp(MIN_DEMAND, MAX_DEMAND)
    }
}

#[cfg(test)]
mod demand_tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_uniform_demand_generator_range() {
        let generator = UniformDemandGenerator;
        let mut rng = StepRng::new(2, 1);

        // Test that generated values are always in valid range
        for _ in 0..100 {
            let demand = generator.generate_demand_count(0, 0, &mut rng);
            assert!((2..=5).contains(&demand), "Demand {} out of range", demand);
        }
    }

    #[test]
    fn test_concentrated_demand_generator_range() {
        let generator = ConcentratedDemandGenerator;
        let mut rng = StepRng::new(2, 1);

        // Test that generated values are always in valid range
        for _ in 0..100 {
            let demand = generator.generate_demand_count(0, 0, &mut rng);
            assert!((2..=5).contains(&demand), "Demand {} out of range", demand);
        }
    }

    #[test]
    fn test_cyclical_demand_generator_range() {
        let generator = CyclicalDemandGenerator;
        let mut rng = StepRng::new(2, 1);

        // Test across multiple steps to cover full cycle
        for step in 0..200 {
            let demand = generator.generate_demand_count(0, step, &mut rng);
            assert!((2..=5).contains(&demand), "Demand {} out of range", demand);
        }
    }

    #[test]
    fn test_cyclical_demand_generator_varies_over_time() {
        let generator = CyclicalDemandGenerator;
        let mut rng = StepRng::new(2, 1);

        let mut demands = Vec::new();
        for step in 0..100 {
            demands.push(generator.generate_demand_count(0, step, &mut rng));
        }

        // Check that we see different demand values (not constant)
        let unique_values: std::collections::HashSet<_> = demands.iter().cloned().collect();
        assert!(unique_values.len() > 1, "Cyclical demand should vary over time");
    }

    #[test]
    fn test_demand_generator_enum_conversion() {
        let uniform_gen = DemandGenerator::from(DemandStrategy::Uniform);
        let concentrated_gen = DemandGenerator::from(DemandStrategy::Concentrated);
        let cyclical_gen = DemandGenerator::from(DemandStrategy::Cyclical);

        let mut rng = StepRng::new(2, 1);

        // Verify each generates valid values
        assert!(uniform_gen.generate_demand_count(0, 0, &mut rng) >= 2);
        assert!(concentrated_gen.generate_demand_count(0, 0, &mut rng) >= 2);
        assert!(cyclical_gen.generate_demand_count(0, 0, &mut rng) >= 2);
    }
}
