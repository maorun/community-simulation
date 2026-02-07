//! Test helper utilities for creating simulation configurations
//!
//! This module provides convenient builder-style helpers for creating test configurations,
//! reducing boilerplate and making test intent clearer.

use crate::scenario::Scenario;
use crate::SimulationConfig;

/// Creates a minimal test configuration with sensible defaults for testing.
///
/// This provides a baseline configuration suitable for most tests:
/// - Small scale: 10 entities, 100 steps (fast execution)
/// - Deterministic: Fixed seed (42) for reproducibility
/// - Simple economy: Original scenario, no special features
/// - Disabled features: No loans, taxes, checkpoints, etc.
///
/// Use `with_*` methods to customize specific aspects while keeping other defaults.
///
/// # Examples
///
/// ```
/// use simulation_framework::tests::test_helpers::test_config;
///
/// // Minimal config for simple test
/// let config = test_config().build();
///
/// // Customize specific aspects
/// let config = test_config()
///     .entity_count(20)
///     .max_steps(200)
///     .scenario(simulation_framework::scenario::Scenario::DynamicPricing)
///     .build();
/// ```
pub fn test_config() -> TestConfigBuilder {
    TestConfigBuilder::default()
}

/// Builder for creating test configurations with a fluent API.
///
/// This builder starts with sensible test defaults and allows customizing
/// specific fields while keeping others at their default values.
///
/// # Examples
///
/// ```
/// use simulation_framework::tests::test_helpers::test_config;
/// use simulation_framework::scenario::Scenario;
///
/// let config = test_config()
///     .entity_count(50)
///     .max_steps(100)
///     .initial_money(200.0)
///     .base_price(25.0)
///     .scenario(Scenario::DynamicPricing)
///     .enable_loans(true)
///     .build();
/// ```
#[derive(Clone)]
pub struct TestConfigBuilder {
    config: SimulationConfig,
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self {
            config: SimulationConfig {
                entity_count: 10,
                max_steps: 100,
                initial_money_per_person: 100.0,
                base_skill_price: 50.0,
                min_skill_price: 1.0,
                seed: 42,
                scenario: Scenario::Original,
                time_step: 1.0,
                tech_growth_rate: 0.0,
                seasonal_amplitude: 0.0,
                seasonal_period: 100,
                transaction_fee: 0.0,
                savings_rate: 0.0,
                enable_loans: false,
                loan_interest_rate: 0.01,
                loan_repayment_period: 20,
                min_money_to_lend: 50.0,
                checkpoint_interval: 0,
                checkpoint_file: None,
                resume_from_checkpoint: false,
                tax_rate: 0.0,
                enable_tax_redistribution: false,
                skills_per_person: 1,
                stream_output_path: None,
                priority_urgency_weight: 0.5,
                priority_affordability_weight: 0.3,
                priority_efficiency_weight: 0.1,
                priority_reputation_weight: 0.1,
                ..Default::default()
            },
        }
    }
}

#[allow(dead_code)] // Helper methods may not all be used immediately in tests
impl TestConfigBuilder {
    /// Set the number of entities (persons) in the simulation
    pub fn entity_count(mut self, count: usize) -> Self {
        self.config.entity_count = count;
        self
    }

    /// Set the maximum number of simulation steps
    pub fn max_steps(mut self, steps: usize) -> Self {
        self.config.max_steps = steps;
        self
    }

    /// Set the initial money per person
    pub fn initial_money(mut self, money: f64) -> Self {
        self.config.initial_money_per_person = money;
        self
    }

    /// Set the base skill price
    pub fn base_price(mut self, price: f64) -> Self {
        self.config.base_skill_price = price;
        self
    }

    /// Set the minimum skill price floor
    pub fn min_price(mut self, price: f64) -> Self {
        self.config.min_skill_price = price;
        self
    }

    /// Set the random seed for deterministic testing
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    /// Set the simulation scenario
    pub fn scenario(mut self, scenario: Scenario) -> Self {
        self.config.scenario = scenario;
        self
    }

    /// Set the technology growth rate
    pub fn tech_growth(mut self, rate: f64) -> Self {
        self.config.tech_growth_rate = rate;
        self
    }

    /// Set seasonal effects (amplitude and period)
    pub fn seasonality(mut self, amplitude: f64, period: usize) -> Self {
        self.config.seasonal_amplitude = amplitude;
        self.config.seasonal_period = period;
        self
    }

    /// Set the transaction fee percentage
    pub fn transaction_fee(mut self, fee: f64) -> Self {
        self.config.transaction_fee = fee;
        self
    }

    /// Set the savings rate
    pub fn savings_rate(mut self, rate: f64) -> Self {
        self.config.savings_rate = rate;
        self
    }

    /// Enable or disable loans
    pub fn enable_loans(mut self, enable: bool) -> Self {
        self.config.enable_loans = enable;
        self
    }

    /// Set loan interest rate
    pub fn loan_interest_rate(mut self, rate: f64) -> Self {
        self.config.loan_interest_rate = rate;
        self
    }

    /// Set loan repayment period
    pub fn loan_repayment_period(mut self, period: usize) -> Self {
        self.config.loan_repayment_period = period;
        self
    }

    /// Set minimum money required to lend
    pub fn min_money_to_lend(mut self, amount: f64) -> Self {
        self.config.min_money_to_lend = amount;
        self
    }

    /// Set the tax rate
    pub fn tax_rate(mut self, rate: f64) -> Self {
        self.config.tax_rate = rate;
        self
    }

    /// Enable or disable tax redistribution
    pub fn enable_tax_redistribution(mut self, enable: bool) -> Self {
        self.config.enable_tax_redistribution = enable;
        self
    }

    /// Set the number of skills per person
    pub fn skills_per_person(mut self, count: usize) -> Self {
        self.config.skills_per_person = count;
        self
    }

    /// Set checkpoint interval
    pub fn checkpoint_interval(mut self, interval: usize) -> Self {
        self.config.checkpoint_interval = interval;
        self
    }

    /// Set checkpoint file path
    pub fn checkpoint_file(mut self, path: Option<String>) -> Self {
        self.config.checkpoint_file = path;
        self
    }

    /// Enable or disable friendships
    pub fn enable_friendships(mut self, enable: bool) -> Self {
        self.config.enable_friendships = enable;
        self
    }

    /// Set friendship probability
    pub fn friendship_probability(mut self, prob: f64) -> Self {
        self.config.friendship_probability = prob;
        self
    }

    /// Set friendship discount
    pub fn friendship_discount(mut self, discount: f64) -> Self {
        self.config.friendship_discount = discount;
        self
    }

    /// Enable or disable influence tracking
    pub fn enable_influence(mut self, enable: bool) -> Self {
        self.config.enable_influence = enable;
        self
    }

    /// Enable or disable event tracking
    pub fn enable_events(mut self, enable: bool) -> Self {
        self.config.enable_events = enable;
        self
    }

    /// Set stream output path
    pub fn stream_output_path(mut self, path: Option<String>) -> Self {
        self.config.stream_output_path = path;
        self
    }

    /// Set priority weights for trading decisions
    pub fn priority_weights(
        mut self,
        urgency: f64,
        affordability: f64,
        efficiency: f64,
        reputation: f64,
    ) -> Self {
        self.config.priority_urgency_weight = urgency;
        self.config.priority_affordability_weight = affordability;
        self.config.priority_efficiency_weight = efficiency;
        self.config.priority_reputation_weight = reputation;
        self
    }

    /// Build and return the final SimulationConfig
    pub fn build(self) -> SimulationConfig {
        self.config
    }

    /// Build with a custom configuration function for advanced scenarios
    pub fn build_with<F>(mut self, f: F) -> SimulationConfig
    where
        F: FnOnce(&mut SimulationConfig),
    {
        f(&mut self.config);
        self.config
    }
}

/// Creates a configuration for a quick test (very small, fast execution).
///
/// Suitable for smoke tests and basic functionality checks:
/// - 5 entities
/// - 20 steps
/// - Other defaults from `test_config()`
///
/// # Example
///
/// ```
/// use simulation_framework::tests::test_helpers::quick_test_config;
///
/// let config = quick_test_config();
/// ```
pub fn quick_test_config() -> SimulationConfig {
    test_config().entity_count(5).max_steps(20).build()
}

/// Creates a configuration for loan testing.
///
/// Pre-configured with loans enabled and reasonable loan parameters:
/// - Loans enabled
/// - 1% interest rate
/// - 20 step repayment period
/// - 50.0 minimum money to lend
/// - Other defaults from `test_config()`
///
/// # Example
///
/// ```
/// use simulation_framework::tests::test_helpers::loan_test_config;
///
/// let config = loan_test_config();
/// ```
pub fn loan_test_config() -> SimulationConfig {
    test_config()
        .enable_loans(true)
        .loan_interest_rate(0.01)
        .loan_repayment_period(20)
        .min_money_to_lend(50.0)
        .build()
}

/// Creates a configuration for tax testing.
///
/// Pre-configured with taxation enabled:
/// - 10% tax rate
/// - Tax redistribution enabled
/// - Other defaults from `test_config()`
///
/// # Example
///
/// ```
/// use simulation_framework::tests::test_helpers::tax_test_config;
///
/// let config = tax_test_config();
/// ```
pub fn tax_test_config() -> SimulationConfig {
    test_config().tax_rate(0.1).enable_tax_redistribution(true).build()
}

/// Creates a configuration for seasonality testing.
///
/// Pre-configured with seasonal effects:
/// - 50% seasonal amplitude
/// - 100 step seasonal period
/// - Other defaults from `test_config()`
///
/// # Example
///
/// ```
/// use simulation_framework::tests::test_helpers::seasonal_test_config;
///
/// let config = seasonal_test_config();
/// ```
pub fn seasonal_test_config() -> SimulationConfig {
    test_config().seasonality(0.5, 100).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = test_config().build();
        assert_eq!(config.entity_count, 10);
        assert_eq!(config.max_steps, 100);
        assert_eq!(config.initial_money_per_person, 100.0);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn test_builder_customization() {
        let config = test_config()
            .entity_count(50)
            .max_steps(200)
            .initial_money(500.0)
            .seed(999)
            .build();

        assert_eq!(config.entity_count, 50);
        assert_eq!(config.max_steps, 200);
        assert_eq!(config.initial_money_per_person, 500.0);
        assert_eq!(config.seed, 999);
    }

    #[test]
    fn test_quick_config() {
        let config = quick_test_config();
        assert_eq!(config.entity_count, 5);
        assert_eq!(config.max_steps, 20);
    }

    #[test]
    fn test_loan_config() {
        let config = loan_test_config();
        assert!(config.enable_loans);
        assert_eq!(config.loan_interest_rate, 0.01);
        assert_eq!(config.loan_repayment_period, 20);
    }

    #[test]
    fn test_tax_config() {
        let config = tax_test_config();
        assert_eq!(config.tax_rate, 0.1);
        assert!(config.enable_tax_redistribution);
    }

    #[test]
    fn test_seasonal_config() {
        let config = seasonal_test_config();
        assert_eq!(config.seasonal_amplitude, 0.5);
        assert_eq!(config.seasonal_period, 100);
    }

    #[test]
    fn test_chained_methods() {
        let config = test_config()
            .entity_count(25)
            .enable_loans(true)
            .tax_rate(0.15)
            .seasonality(0.3, 50)
            .build();

        assert_eq!(config.entity_count, 25);
        assert!(config.enable_loans);
        assert_eq!(config.tax_rate, 0.15);
        assert_eq!(config.seasonal_amplitude, 0.3);
        assert_eq!(config.seasonal_period, 50);
    }
}
