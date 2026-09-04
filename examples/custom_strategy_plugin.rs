// Example demonstrating the plugin architecture extension points.
//
// The simulation exposes three trait-based extension points that allow adding new
// behaviour without modifying the core code:
//
//   * `Plugin`          - observe the simulation lifecycle
//   * `PricingStrategy` - define a custom price update mechanism
//   * `AgentStrategy`   - define custom agent purchase decisions
//
// This example implements all three and runs a simulation with them installed.
// The same pattern works from an external crate that depends on
// `community-simulation`.

use community_simulation::plugin::{AgentStrategy, Plugin, PluginContext, PricingStrategy};
use community_simulation::{Market, Person, SimulationConfig, SimulationEngine, SkillId};
use rand::Rng;
use std::any::Any;
use std::sync::Arc;

/// A custom pricing mechanism: prices drift towards the market's base price.
///
/// This is a mean-reverting mechanism that is not part of the built-in scenarios.
#[derive(Debug)]
struct MeanRevertingPricing {
    /// Fraction of the gap to the base price closed in each step (0.0 - 1.0).
    reversion_rate: f64,
}

impl PricingStrategy for MeanRevertingPricing {
    fn name(&self) -> &str {
        "MeanRevertingPricing"
    }

    fn update_prices(&self, market: &mut Market, _rng: &mut dyn Rng) {
        let base_price = market.base_skill_price;
        let min_price = market.min_skill_price;
        let max_price = market.max_skill_price;

        for (skill_id, skill) in market.skills.iter_mut() {
            let gap = base_price - skill.current_price;
            let new_price =
                (skill.current_price + gap * self.reversion_rate).clamp(min_price, max_price);
            skill.current_price = new_price;

            // Price history must be maintained by the strategy itself.
            if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                history.push(new_price);
            }
        }
    }
}

/// A custom agent decision rule: never spend more than a fixed share of the wealth.
#[derive(Debug)]
struct BudgetCappedAgent {
    /// Maximum share of a person's money that may be spent on a single purchase.
    max_share_of_wealth: f64,
}

impl AgentStrategy for BudgetCappedAgent {
    fn name(&self) -> &str {
        "BudgetCappedAgent"
    }

    fn should_purchase(&self, person: &Person, _skill_id: &SkillId, price: f64) -> bool {
        price <= person.money * self.max_share_of_wealth
    }
}

/// A lifecycle plugin that reports progress while the simulation runs.
struct ProgressPlugin {
    steps_observed: usize,
}

impl Plugin for ProgressPlugin {
    fn name(&self) -> &str {
        "ProgressPlugin"
    }

    fn on_simulation_start(&mut self, context: &PluginContext) {
        println!("Simulation started with {} persons", context.config.entity_count);
    }

    fn on_step_end(&mut self, _context: &PluginContext) {
        self.steps_observed += 1;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn main() {
    println!("===========================================");
    println!("Custom Strategy Plugin Demonstration");
    println!("===========================================\n");

    let config = SimulationConfig {
        max_steps: 100,
        entity_count: 30,
        seed: 42,
        initial_money_per_person: 100.0,
        base_skill_price: 10.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Install the extension points.
    engine.set_pricing_strategy(Arc::new(MeanRevertingPricing { reversion_rate: 0.2 }));
    engine.set_agent_strategy(Arc::new(BudgetCappedAgent { max_share_of_wealth: 0.25 }));
    engine.register_plugin(Box::new(ProgressPlugin { steps_observed: 0 }));

    let result = engine.run();

    println!("\nResults with custom strategies:");
    println!("  Steps:  {}", result.total_steps);
    println!("  Trades: {}", result.trade_volume_statistics.total_trades);
    println!("  Gini:   {:.3}", result.money_statistics.gini_coefficient);

    if let Some(plugin) = engine.plugin_registry().get("ProgressPlugin") {
        if let Some(progress) = plugin.as_any().downcast_ref::<ProgressPlugin>() {
            println!("  Steps observed by plugin: {}", progress.steps_observed);
        }
    }

    println!("\nFinal prices (mean-reverting towards the base price of 10.0):");
    for (skill_id, skill) in engine.get_market().skills.iter().take(5) {
        println!("  {}: ${:.2}", skill_id, skill.current_price);
    }
}
