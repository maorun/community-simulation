//! Integration tests for the custom pricing and agent strategy extension points.

use crate::plugin::{AgentStrategy, PricingStrategy};
use crate::tests::test_helpers::test_config;
use crate::{Market, Person, SimulationEngine, SkillId};
use rand::Rng;
use std::sync::Arc;

/// Pricing strategy that keeps every price constant.
#[derive(Debug)]
struct FrozenPricingStrategy;

impl PricingStrategy for FrozenPricingStrategy {
    fn name(&self) -> &str {
        "FrozenPricing"
    }

    fn update_prices(&self, market: &mut Market, _rng: &mut dyn Rng) {
        for (skill_id, skill) in market.skills.iter_mut() {
            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(skill.current_price);
            }
        }
    }
}

/// Agent strategy that never buys anything.
#[derive(Debug)]
struct NeverBuyStrategy;

impl AgentStrategy for NeverBuyStrategy {
    fn name(&self) -> &str {
        "NeverBuy"
    }

    fn should_purchase(&self, _person: &Person, _skill_id: &SkillId, _price: f64) -> bool {
        false
    }
}

#[test]
fn test_engine_uses_custom_pricing_strategy() {
    let config = test_config().max_steps(20).build();
    let mut engine = SimulationEngine::new(config);
    engine.set_pricing_strategy(Arc::new(FrozenPricingStrategy));

    let initial_prices: Vec<(SkillId, f64)> = engine
        .get_market()
        .skills
        .iter()
        .map(|(id, skill)| (id.clone(), skill.current_price))
        .collect();

    engine.run();

    for (skill_id, initial_price) in initial_prices {
        let final_price = engine.get_market().skills.get(&skill_id).unwrap().current_price;
        assert!(
            (final_price - initial_price).abs() < f64::EPSILON,
            "Custom pricing strategy should keep prices constant for {}",
            skill_id
        );
    }
}

#[test]
fn test_engine_uses_custom_agent_strategy() {
    let config = test_config().max_steps(20).build();

    let mut baseline = SimulationEngine::new(config.clone());
    let baseline_result = baseline.run();
    assert!(
        baseline_result.trade_volume_statistics.total_trades > 0,
        "Baseline simulation should produce transactions"
    );

    let mut engine = SimulationEngine::new(config);
    engine.set_agent_strategy(Arc::new(NeverBuyStrategy));
    let result = engine.run();

    assert_eq!(
        result.trade_volume_statistics.total_trades, 0,
        "Agent strategy refusing all purchases should prevent trades"
    );
}
