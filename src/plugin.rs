//! Plugin system for extending simulation functionality.
//!
//! This module provides a trait-based plugin system that allows extending
//! the simulation without modifying core code. Plugins can hook into various
//! points in the simulation lifecycle.
//!
//! # Example
//!
//! ```rust
//! use community_simulation::plugin::{Plugin, PluginContext};
//! use std::any::Any;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str {
//!         "MyPlugin"
//!     }
//!
//!     fn on_simulation_start(&mut self, context: &PluginContext) {
//!         println!("Simulation starting with {} persons", context.config.entity_count);
//!     }
//!
//!     fn as_any(&self) -> &dyn Any {
//!         self
//!     }
//!
//!     fn as_any_mut(&mut self) -> &mut dyn Any {
//!         self
//!     }
//! }
//! ```

use crate::config::SimulationConfig;
use crate::market::Market;
use crate::person::Person;
use crate::result::SimulationResult;
use crate::skill::SkillId;
use rand::Rng;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// Context provided to plugins containing simulation state.
///
/// # Performance Note
///
/// The `persons` field contains references to avoid unnecessary cloning of Person data
/// during plugin callbacks. Each Person struct may contain HashMaps, Vecs, and transaction
/// history, making cloning expensive in large simulations.
#[derive(Debug)]
pub struct PluginContext<'a> {
    /// The simulation configuration
    pub config: &'a SimulationConfig,
    /// Current simulation step
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Reference to all persons in the simulation (as references to avoid cloning)
    pub persons: &'a [&'a Person],
}

/// Trait that all plugins must implement.
///
/// Plugins can hook into various points in the simulation lifecycle
/// to extend functionality without modifying core code.
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Called once when the simulation is initialized.
    ///
    /// This is called before the first simulation step.
    fn on_simulation_start(&mut self, _context: &PluginContext) {}

    /// Called before each simulation step.
    ///
    /// This allows plugins to observe or modify state before trading occurs.
    fn on_step_start(&mut self, _context: &PluginContext) {}

    /// Called after each simulation step.
    ///
    /// This allows plugins to observe state changes after trading.
    fn on_step_end(&mut self, _context: &PluginContext) {}

    /// Called once when the simulation completes.
    ///
    /// This is called after all simulation steps have completed.
    fn on_simulation_end(&mut self, _context: &PluginContext, _result: &mut SimulationResult) {}

    /// Returns the plugin as Any for downcasting.
    ///
    /// This allows accessing plugin-specific methods after retrieval from registry.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the plugin as Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Registry for managing plugins.
///
/// The registry holds all registered plugins and provides methods
/// to invoke plugin hooks at appropriate points in the simulation.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Registers a plugin with the registry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use community_simulation::plugin::{Plugin, PluginRegistry, PluginContext};
    ///
    /// struct MyPlugin;
    /// impl Plugin for MyPlugin {
    ///     fn name(&self) -> &str { "MyPlugin" }
    ///     fn as_any(&self) -> &dyn std::any::Any { self }
    ///     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    /// }
    ///
    /// let mut registry = PluginRegistry::new();
    /// registry.register(Box::new(MyPlugin));
    /// ```
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        log::info!("Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    /// Returns the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns true if no plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Gets a reference to a plugin by name.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Gets a mutable reference to a plugin by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        for plugin in &mut self.plugins {
            if plugin.name() == name {
                return Some(plugin.as_mut());
            }
        }
        None
    }

    /// Invokes on_simulation_start for all plugins.
    pub fn on_simulation_start(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_simulation_start(context);
        }
    }

    /// Invokes on_step_start for all plugins.
    pub fn on_step_start(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_step_start(context);
        }
    }

    /// Invokes on_step_end for all plugins.
    pub fn on_step_end(&mut self, context: &PluginContext) {
        for plugin in &mut self.plugins {
            plugin.on_step_end(context);
        }
    }

    /// Invokes on_simulation_end for all plugins.
    pub fn on_simulation_end(&mut self, context: &PluginContext, result: &mut SimulationResult) {
        for plugin in &mut self.plugins {
            plugin.on_simulation_end(context, result);
        }
    }
}

/// Extension point for custom pricing mechanisms.
///
/// Implement this trait to define a pricing mechanism that is not covered by the
/// built-in [`Scenario`](crate::scenario::Scenario) variants. A custom strategy can be
/// installed with [`SimulationEngine::set_pricing_strategy`](crate::engine::SimulationEngine::set_pricing_strategy)
/// or by constructing a [`PriceUpdater::Custom`](crate::scenario::PriceUpdater) directly.
///
/// # Example
///
/// ```rust
/// use community_simulation::plugin::PricingStrategy;
/// use community_simulation::Market;
/// use rand::Rng;
///
/// #[derive(Debug)]
/// struct FixedPricing;
///
/// impl PricingStrategy for FixedPricing {
///     fn name(&self) -> &str {
///         "FixedPricing"
///     }
///
///     fn update_prices(&self, market: &mut Market, _rng: &mut dyn Rng) {
///         for (skill_id, skill) in market.skills.iter_mut() {
///             if let Some(history) = market.skill_price_history.get_mut(skill_id) {
///                 history.push(skill.current_price);
///             }
///         }
///     }
/// }
/// ```
pub trait PricingStrategy: Send + Sync + Debug {
    /// Returns the name of the pricing strategy.
    fn name(&self) -> &str;

    /// Updates all skill prices in the market.
    ///
    /// Implementations are responsible for respecting the market's price limits
    /// (`min_skill_price`, `max_skill_price` and `per_skill_price_limits`) and for
    /// appending the new price to `market.skill_price_history`.
    ///
    /// # Arguments
    ///
    /// * `market` - The market containing the skills whose prices should be updated
    /// * `rng` - Random number generator for stochastic price components
    fn update_prices(&self, market: &mut Market, rng: &mut dyn Rng);
}

/// Wrapper making a user provided [`PricingStrategy`] usable inside the simulation.
///
/// The strategy is stored behind an [`Arc`] so it can be cheaply cloned together with
/// the market it is attached to.
#[derive(Clone, Debug)]
pub struct CustomPricingStrategy {
    strategy: Arc<dyn PricingStrategy>,
}

impl CustomPricingStrategy {
    /// Wraps a pricing strategy so it can be used as a price updater.
    pub fn new(strategy: Arc<dyn PricingStrategy>) -> Self {
        Self { strategy }
    }

    /// Returns the name of the wrapped strategy.
    pub fn name(&self) -> &str {
        self.strategy.name()
    }

    /// Returns a reference to the wrapped strategy.
    pub fn strategy(&self) -> &Arc<dyn PricingStrategy> {
        &self.strategy
    }

    /// Delegates the price update to the wrapped strategy.
    pub fn update_prices<R: Rng + ?Sized>(&self, market: &mut Market, rng: &mut R) {
        // `&mut R` itself implements `Rng`, which allows creating a trait object
        // even when `R` is unsized.
        let mut rng_ref = rng;
        self.strategy.update_prices(market, &mut rng_ref);
    }
}

/// Extension point for custom agent decision logic.
///
/// Implement this trait to override how agents decide whether to purchase a needed
/// skill. A custom strategy can be installed with
/// [`SimulationEngine::set_agent_strategy`](crate::engine::SimulationEngine::set_agent_strategy).
///
/// # Example
///
/// ```rust
/// use community_simulation::plugin::AgentStrategy;
/// use community_simulation::{Person, SkillId};
///
/// #[derive(Debug)]
/// struct NeverSpendMoreThanHalf;
///
/// impl AgentStrategy for NeverSpendMoreThanHalf {
///     fn name(&self) -> &str {
///         "NeverSpendMoreThanHalf"
///     }
///
///     fn should_purchase(&self, person: &Person, _skill_id: &SkillId, price: f64) -> bool {
///         price <= person.money * 0.5
///     }
/// }
/// ```
pub trait AgentStrategy: Send + Sync + Debug {
    /// Returns the name of the agent strategy.
    fn name(&self) -> &str;

    /// Decides whether the given person should buy `skill_id` at `price`.
    ///
    /// The default implementation reproduces the built-in behaviour, which takes the
    /// person's [`Strategy`](crate::person::Strategy) and adaptive spending parameters
    /// into account.
    fn should_purchase(&self, person: &Person, _skill_id: &SkillId, price: f64) -> bool {
        person.can_afford_with_strategy(price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        start_called: bool,
        step_start_called: usize,
        step_end_called: usize,
        end_called: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                start_called: false,
                step_start_called: 0,
                step_end_called: 0,
                end_called: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn on_simulation_start(&mut self, _context: &PluginContext) {
            self.start_called = true;
        }

        fn on_step_start(&mut self, _context: &PluginContext) {
            self.step_start_called += 1;
        }

        fn on_step_end(&mut self, _context: &PluginContext) {
            self.step_end_called += 1;
        }

        fn on_simulation_end(&mut self, _context: &PluginContext, _result: &mut SimulationResult) {
            self.end_called = true;
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_plugin_registry_register() {
        let mut registry = PluginRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(TestPlugin::new("test1")));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        registry.register(Box::new(TestPlugin::new("test2")));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_plugin_registry_get() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin::new("test1")));
        registry.register(Box::new(TestPlugin::new("test2")));

        assert!(registry.get("test1").is_some());
        assert!(registry.get("test2").is_some());
        assert!(registry.get("test3").is_none());

        let plugin = registry.get("test1").unwrap();
        assert_eq!(plugin.name(), "test1");
    }

    #[test]
    fn test_plugin_lifecycle() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin::new("test")));

        let config = SimulationConfig {
            max_steps: 5,
            entity_count: 10,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            ..Default::default()
        };

        let context =
            PluginContext { config: &config, current_step: 0, total_steps: 5, persons: &[] };

        // Test lifecycle hooks
        registry.on_simulation_start(&context);

        for _ in 0..5 {
            registry.on_step_start(&context);
            registry.on_step_end(&context);
        }

        // Verify calls without needing SimulationResult
        let plugin = registry.get_mut("test").unwrap();
        let test_plugin = plugin.as_any_mut().downcast_mut::<TestPlugin>().unwrap();

        assert!(test_plugin.start_called);
        assert_eq!(test_plugin.step_start_called, 5);
        assert_eq!(test_plugin.step_end_called, 5);
    }

    /// A minimal custom pricing strategy used to verify the extension point.
    #[derive(Debug)]
    struct DoublingPricingStrategy;

    impl PricingStrategy for DoublingPricingStrategy {
        fn name(&self) -> &str {
            "DoublingPricing"
        }

        fn update_prices(&self, market: &mut Market, _rng: &mut dyn Rng) {
            let max_price = market.max_skill_price;
            for (skill_id, skill) in market.skills.iter_mut() {
                skill.current_price = (skill.current_price * 2.0).min(max_price);
                if let Some(history) = market.skill_price_history.get_mut(skill_id) {
                    history.push(skill.current_price);
                }
            }
        }
    }

    /// A custom agent strategy that refuses every purchase.
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

    /// A custom agent strategy relying on the default decision logic.
    #[derive(Debug)]
    struct DefaultAgentStrategy;

    impl AgentStrategy for DefaultAgentStrategy {
        fn name(&self) -> &str {
            "DefaultAgent"
        }
    }

    #[test]
    fn test_custom_pricing_strategy_updates_market_prices() {
        use crate::scenario::PriceUpdater;
        use crate::skill::Skill;
        use rand::{rngs::StdRng, SeedableRng};

        let mut market = Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::default());
        market.add_skill(Skill::new("Test Skill".to_string(), 10.0));

        let strategy = CustomPricingStrategy::new(Arc::new(DoublingPricingStrategy));
        assert_eq!(strategy.name(), "DoublingPricing");
        market.set_price_updater(PriceUpdater::Custom(strategy));

        let mut rng = StdRng::seed_from_u64(42);
        market.update_prices(&mut rng);

        let price = market.skills.get("Test Skill").unwrap().current_price;
        assert!((price - 20.0).abs() < f64::EPSILON, "Expected doubled price, got {}", price);
        assert_eq!(market.skill_price_history.get("Test Skill").unwrap().len(), 1);
    }

    #[test]
    fn test_custom_pricing_strategy_accessors() {
        let strategy = CustomPricingStrategy::new(Arc::new(DoublingPricingStrategy));
        assert_eq!(strategy.strategy().name(), "DoublingPricing");
        assert_eq!(strategy.clone().name(), "DoublingPricing");
    }

    #[test]
    fn test_agent_strategy_custom_decision() {
        use crate::person::{Location, Strategy};

        let person =
            Person::new(0, 100.0, vec![], Strategy::Balanced, Location::new(0.0, 0.0), 0.9);
        let strategy = NeverBuyStrategy;

        assert_eq!(strategy.name(), "NeverBuy");
        assert!(!strategy.should_purchase(&person, &"Test Skill".to_string(), 1.0));
    }

    #[test]
    fn test_agent_strategy_default_decision_matches_builtin() {
        use crate::person::{Location, Strategy};

        let person =
            Person::new(0, 100.0, vec![], Strategy::Balanced, Location::new(0.0, 0.0), 0.9);
        let strategy = DefaultAgentStrategy;
        let skill_id = "Test Skill".to_string();

        assert_eq!(strategy.name(), "DefaultAgent");
        assert_eq!(
            strategy.should_purchase(&person, &skill_id, 50.0),
            person.can_afford_with_strategy(50.0)
        );
        assert_eq!(
            strategy.should_purchase(&person, &skill_id, 5_000.0),
            person.can_afford_with_strategy(5_000.0)
        );
    }
}
