use rand::Rng;
use serde::{Deserialize, Serialize};

/// Types of economic crises that can occur during the simulation.
///
/// Crisis events simulate unexpected economic shocks that test the resilience
/// of the simulated economy. Each crisis type has different effects on market
/// prices, demand, supply, or individual wealth.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CrisisEvent {
    /// Market crash - sudden price drop across all skills
    ///
    /// Reduces all skill prices by a significant percentage (typically 20-40%).
    /// Simulates sudden loss of market confidence or deflationary shock.
    MarketCrash,

    /// Demand shock - sudden drop in overall demand
    ///
    /// Reduces demand for all skills by a significant percentage (typically 30-50%).
    /// Simulates economic recession or loss of consumer spending.
    DemandShock,

    /// Supply shock - sudden reduction in available supply
    ///
    /// Reduces supply of all skills by a percentage (typically 20-40%).
    /// Simulates supply chain disruptions or labor shortages.
    SupplyShock,

    /// Currency devaluation - sudden reduction in everyone's purchasing power
    ///
    /// Reduces money holdings of all persons by a percentage (typically 10-30%).
    /// Simulates inflation, currency crisis, or wealth destruction.
    CurrencyDevaluation,

    /// Technology shock - sudden technological breakthrough making skills obsolete
    ///
    /// Randomly affects certain skills with massive value loss (typically 50-80%).
    /// Simulates technological disruption, automation, or paradigm shifts.
    TechnologyShock,
}

impl CrisisEvent {
    /// Get all possible crisis event types
    pub fn all_types() -> Vec<CrisisEvent> {
        vec![
            CrisisEvent::MarketCrash,
            CrisisEvent::DemandShock,
            CrisisEvent::SupplyShock,
            CrisisEvent::CurrencyDevaluation,
            CrisisEvent::TechnologyShock,
        ]
    }

    /// Get a human-readable name for this crisis type
    pub fn name(&self) -> &str {
        match self {
            CrisisEvent::MarketCrash => "Market Crash",
            CrisisEvent::DemandShock => "Demand Shock",
            CrisisEvent::SupplyShock => "Supply Shock",
            CrisisEvent::CurrencyDevaluation => "Currency Devaluation",
            CrisisEvent::TechnologyShock => "Technology Shock",
        }
    }

    /// Get a description of this crisis type's effects
    pub fn description(&self) -> &str {
        match self {
            CrisisEvent::MarketCrash => "Sudden price drop across all skills (20-40%)",
            CrisisEvent::DemandShock => "Sudden drop in overall demand (30-50%)",
            CrisisEvent::SupplyShock => "Reduction in available supply (20-40%)",
            CrisisEvent::CurrencyDevaluation => "Reduction in purchasing power (10-30%)",
            CrisisEvent::TechnologyShock => {
                "Technological disruption making certain skills obsolete (50-80%)"
            },
        }
    }

    /// Apply the crisis effect with a given severity.
    ///
    /// # Arguments
    /// * `base_value` - The base value to apply the crisis effect to
    /// * `severity` - Crisis severity from 0.0 to 1.0 (higher = more severe)
    /// * `rng` - Random number generator for adding randomness to effects
    ///
    /// # Returns
    /// The modified value after applying the crisis effect
    pub fn apply_effect<R: Rng>(&self, base_value: f64, severity: f64, rng: &mut R) -> f64 {
        // Randomness factors for crisis effects
        const STANDARD_RANDOMNESS_MIN: f64 = 0.9; // -10% randomness
        const STANDARD_RANDOMNESS_MAX: f64 = 1.1; // +10% randomness
        const MONEY_RANDOMNESS_MIN: f64 = 0.95; // -5% randomness for money (less volatile)
        const MONEY_RANDOMNESS_MAX: f64 = 1.05; // +5% randomness for money (less volatile)

        let severity = severity.clamp(0.0, 1.0);

        match self {
            CrisisEvent::MarketCrash => {
                // Price drop: 20% to 40% reduction, scaled by severity
                let drop_percentage = 0.20 + (severity * 0.20);
                let randomness =
                    rng.random_range(STANDARD_RANDOMNESS_MIN..=STANDARD_RANDOMNESS_MAX);
                base_value * (1.0 - drop_percentage) * randomness
            },
            CrisisEvent::DemandShock => {
                // Demand drop: 30% to 50% reduction, scaled by severity
                let drop_percentage = 0.30 + (severity * 0.20);
                let randomness =
                    rng.random_range(STANDARD_RANDOMNESS_MIN..=STANDARD_RANDOMNESS_MAX);
                base_value * (1.0 - drop_percentage) * randomness
            },
            CrisisEvent::SupplyShock => {
                // Supply drop: 20% to 40% reduction, scaled by severity
                let drop_percentage = 0.20 + (severity * 0.20);
                let randomness =
                    rng.random_range(STANDARD_RANDOMNESS_MIN..=STANDARD_RANDOMNESS_MAX);
                base_value * (1.0 - drop_percentage) * randomness
            },
            CrisisEvent::CurrencyDevaluation => {
                // Money reduction: 10% to 30% reduction, scaled by severity
                let drop_percentage = 0.10 + (severity * 0.20);
                let randomness = rng.random_range(MONEY_RANDOMNESS_MIN..=MONEY_RANDOMNESS_MAX);
                base_value * (1.0 - drop_percentage) * randomness
            },
            CrisisEvent::TechnologyShock => {
                // Technology shock: 50% to 80% price drop for obsolete skills
                let drop_percentage = 0.50 + (severity * 0.30);
                let randomness =
                    rng.random_range(STANDARD_RANDOMNESS_MIN..=STANDARD_RANDOMNESS_MAX);
                base_value * (1.0 - drop_percentage) * randomness
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_crisis_event_names() {
        assert_eq!(CrisisEvent::MarketCrash.name(), "Market Crash");
        assert_eq!(CrisisEvent::DemandShock.name(), "Demand Shock");
        assert_eq!(CrisisEvent::SupplyShock.name(), "Supply Shock");
        assert_eq!(CrisisEvent::CurrencyDevaluation.name(), "Currency Devaluation");
        assert_eq!(CrisisEvent::TechnologyShock.name(), "Technology Shock");
    }

    #[test]
    fn test_all_crisis_types() {
        let types = CrisisEvent::all_types();
        assert_eq!(types.len(), 5);
        assert!(types.contains(&CrisisEvent::MarketCrash));
        assert!(types.contains(&CrisisEvent::DemandShock));
        assert!(types.contains(&CrisisEvent::SupplyShock));
        assert!(types.contains(&CrisisEvent::CurrencyDevaluation));
        assert!(types.contains(&CrisisEvent::TechnologyShock));
    }

    #[test]
    fn test_market_crash_effect() {
        let mut rng = StdRng::seed_from_u64(5); // Deterministic RNG
        let base_price = 100.0;
        let severity = 0.5;

        let new_price = CrisisEvent::MarketCrash.apply_effect(base_price, severity, &mut rng);

        // Should reduce price by 20-40%, with severity=0.5 -> ~30% reduction
        // With randomness ±10%, expect roughly 60-80 range
        assert!(new_price < base_price);
        assert!(new_price > base_price * 0.5); // Not more than 50% drop
        assert!(new_price < base_price * 0.9); // Some reduction happened
    }

    #[test]
    fn test_crisis_severity_clamping() {
        let mut rng = StdRng::seed_from_u64(5);
        let base_value = 100.0;

        // Test severity > 1.0 is clamped
        let result1 = CrisisEvent::MarketCrash.apply_effect(base_value, 1.5, &mut rng);
        let result2 = CrisisEvent::MarketCrash.apply_effect(base_value, 1.0, &mut rng);
        // Both should have similar maximum effect
        assert!((result1 - result2).abs() < 5.0);

        // Test severity < 0.0 is clamped
        let result3 = CrisisEvent::MarketCrash.apply_effect(base_value, -0.5, &mut rng);
        let result4 = CrisisEvent::MarketCrash.apply_effect(base_value, 0.0, &mut rng);
        // Both should have similar minimum effect
        assert!((result3 - result4).abs() < 5.0);
    }

    #[test]
    fn test_currency_devaluation_effect() {
        let mut rng = StdRng::seed_from_u64(5);
        let base_money = 100.0;
        let severity = 0.5;

        let new_money =
            CrisisEvent::CurrencyDevaluation.apply_effect(base_money, severity, &mut rng);

        // Should reduce money by 10-30%, with severity=0.5 -> ~20% reduction
        assert!(new_money < base_money);
        assert!(new_money > base_money * 0.65); // Not more than 35% drop
        assert!(new_money < base_money * 0.95); // Some reduction happened
    }

    #[test]
    fn test_technology_shock_effect() {
        let mut rng = StdRng::seed_from_u64(5);
        let base_price = 100.0;
        let severity = 0.5;

        let new_price = CrisisEvent::TechnologyShock.apply_effect(base_price, severity, &mut rng);

        // Should reduce price by 50-80%, with severity=0.5 -> ~65% reduction
        // With randomness ±10%, expect roughly 25-40 range
        assert!(new_price < base_price);
        assert!(new_price < base_price * 0.5); // At least 50% drop
        assert!(new_price > base_price * 0.1); // Not complete elimination
    }

    #[test]
    fn test_crisis_descriptions() {
        assert!(CrisisEvent::MarketCrash.description().contains("20-40%"));
        assert!(CrisisEvent::DemandShock.description().contains("30-50%"));
        assert!(CrisisEvent::SupplyShock.description().contains("20-40%"));
        assert!(CrisisEvent::CurrencyDevaluation.description().contains("10-30%"));
        assert!(CrisisEvent::TechnologyShock.description().contains("50-80%"));
    }

    #[test]
    fn test_demand_shock_effect() {
        let mut rng = StdRng::seed_from_u64(7);
        let base_demand = 100.0;
        let severity = 0.5;

        let new_demand = CrisisEvent::DemandShock.apply_effect(base_demand, severity, &mut rng);

        // Should reduce demand by 30-50%, with severity=0.5 -> ~40% reduction
        assert!(new_demand < base_demand);
        assert!(new_demand > base_demand * 0.4); // Not more than 60% drop
        assert!(new_demand < base_demand * 0.8); // Some reduction happened
    }

    #[test]
    fn test_supply_shock_effect() {
        let mut rng = StdRng::seed_from_u64(9);
        let base_supply = 100.0;
        let severity = 0.5;

        let new_supply = CrisisEvent::SupplyShock.apply_effect(base_supply, severity, &mut rng);

        // Should reduce supply by 20-40%, with severity=0.5 -> ~30% reduction
        assert!(new_supply < base_supply);
        assert!(new_supply > base_supply * 0.5); // Not more than 50% drop
        assert!(new_supply < base_supply * 0.9); // Some reduction happened
    }

    #[test]
    fn test_crisis_with_zero_severity() {
        let mut rng = StdRng::seed_from_u64(11);
        let base_value = 100.0;

        // With zero severity, should have minimum effect
        let result = CrisisEvent::MarketCrash.apply_effect(base_value, 0.0, &mut rng);
        // Minimum drop is 20% plus randomness
        assert!(result < base_value);
        assert!(result > base_value * 0.6); // At least 60% remains
    }

    #[test]
    fn test_crisis_with_max_severity() {
        let mut rng = StdRng::seed_from_u64(13);
        let base_value = 100.0;

        // With max severity, should have maximum effect
        let result = CrisisEvent::MarketCrash.apply_effect(base_value, 1.0, &mut rng);
        // Maximum drop is 40% plus randomness
        assert!(result < base_value);
        assert!(result < base_value * 0.7); // At most 70% remains (40% drop + randomness)
    }

    #[test]
    fn test_crisis_event_equality() {
        assert_eq!(CrisisEvent::MarketCrash, CrisisEvent::MarketCrash);
        assert_ne!(CrisisEvent::MarketCrash, CrisisEvent::DemandShock);
    }

    #[test]
    fn test_crisis_event_clone() {
        let crisis = CrisisEvent::TechnologyShock;
        let cloned = crisis;
        assert_eq!(crisis, cloned);
    }

    #[test]
    fn test_all_crisis_effects_reduce_value() {
        let mut rng = StdRng::seed_from_u64(15);
        let base_value = 100.0;
        let severity = 0.5;

        for crisis in CrisisEvent::all_types() {
            let result = crisis.apply_effect(base_value, severity, &mut rng);
            assert!(result < base_value, "Crisis {:?} should reduce value", crisis);
            assert!(result > 0.0, "Crisis {:?} should not eliminate value completely", crisis);
        }
    }
}
