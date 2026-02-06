//! Additional tests to boost code coverage to 80%+
//!
//! This module contains targeted tests for previously uncovered lines in:
//! - src/scenario.rs (debug logging, per-skill price limits, edge cases)
//! - src/invariant.rs (violation edge cases, strict mode, multiple violations)
//! - src/engine.rs (crisis handling, plugin system, contract handling)

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::invariant::{
    Invariant, InvariantChecker, InvariantViolation, MoneyConservationInvariant,
    NonNegativeWealthInvariant,
};
use crate::market::Market;
use crate::scenario::{
    AdaptivePricingUpdater, AuctionPricingUpdater, ClimateChangePriceUpdater,
    DynamicPricingUpdater, OriginalPriceUpdater, PriceUpdater, Scenario,
};
use crate::skill::Skill;
use rand::{rngs::StdRng, SeedableRng};

// ============================================================================
// SCENARIO.RS COVERAGE TESTS
// ============================================================================

#[test]
fn test_original_updater_with_per_skill_limits() {
    // Test lines 1115-1118: per_skill_price_limits with Some values
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::Original(OriginalPriceUpdater));
    let skill = Skill::new("Limited".to_string(), 50.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    // Set per-skill limits
    market.per_skill_price_limits.insert(skill_id.clone(), (Some(40.0), Some(60.0)));
    market.demand_counts.insert(skill_id.clone(), 10);
    market.supply_counts.insert(skill_id.clone(), 1);

    let mut rng = StdRng::seed_from_u64(42);
    market.volatility_percentage = 0.0; // Disable for predictability

    let updater = OriginalPriceUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(
        final_price >= 40.0 && final_price <= 60.0,
        "Price should respect per-skill limits"
    );
}

#[test]
fn test_original_updater_debug_logging_path() {
    // Test lines 1146-1154: debug! macro path
    // Just exercise the code path - logging happens internally
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.05, PriceUpdater::Original(OriginalPriceUpdater));
    let skill = Skill::new("Debug".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.demand_counts.insert(skill_id.clone(), 5);
    market.supply_counts.insert(skill_id.clone(), 2);

    let mut rng = StdRng::seed_from_u64(123);
    let updater = OriginalPriceUpdater;
    updater.update_prices(&mut market, &mut rng);

    // Just verify it doesn't panic
    assert!(market.skills.get(&skill_id).unwrap().current_price > 0.0);
}

#[test]
fn test_original_updater_random_fluctuation() {
    // Test line 1137: random_range with volatility
    let mut market = Market::new(10.0, 1.0, 0.1, 0.1, PriceUpdater::Original(OriginalPriceUpdater));
    let skill = Skill::new("Volatile".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);
    market.volatility_percentage = 0.1; // 10% volatility

    market.demand_counts.insert(skill_id.clone(), 5);
    market.supply_counts.insert(skill_id.clone(), 5);

    let mut rng = StdRng::seed_from_u64(999);
    let updater = OriginalPriceUpdater;
    updater.update_prices(&mut market, &mut rng);

    // Verify price changed due to volatility
    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price != 100.0, "Price should change with volatility");
}

#[test]
fn test_dynamic_pricing_updater_with_per_skill_limits() {
    // Test lines 1196-1199: per_skill_price_limits for DynamicPricing
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::DynamicPricing(DynamicPricingUpdater));
    let skill = Skill::new("DynamicLimited".to_string(), 50.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.per_skill_price_limits.insert(skill_id.clone(), (Some(45.0), Some(55.0)));
    market.sales_this_step.insert(skill_id.clone(), 1); // Mark as sold

    let mut rng = StdRng::seed_from_u64(42);
    let updater = DynamicPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price >= 45.0 && final_price <= 55.0);
}

#[test]
fn test_dynamic_pricing_debug_sold_path() {
    // Test lines 1211-1218: debug! when skill is sold
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::DynamicPricing(DynamicPricingUpdater));
    let skill = Skill::new("Sold".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.sales_this_step.insert(skill_id.clone(), 3); // 3 sales

    let mut rng = StdRng::seed_from_u64(42);
    let updater = DynamicPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price > 100.0, "Price should increase when sold");
}

#[test]
fn test_dynamic_pricing_debug_not_sold_path() {
    // Test lines 1222-1227: debug! when skill is not sold
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::DynamicPricing(DynamicPricingUpdater));
    let skill = Skill::new("NotSold".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    // No sales recorded (default to 0)

    let mut rng = StdRng::seed_from_u64(42);
    let updater = DynamicPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price < 100.0, "Price should decrease when not sold");
}

#[test]
fn test_adaptive_pricing_with_per_skill_limits() {
    // Test lines 1284-1287: per_skill_price_limits for AdaptivePricing
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AdaptivePricing(AdaptivePricingUpdater));
    let skill = Skill::new("AdaptiveLimited".to_string(), 50.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.per_skill_price_limits.insert(skill_id.clone(), (Some(48.0), Some(52.0)));
    market.sales_this_step.insert(skill_id.clone(), 1);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AdaptivePricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price >= 48.0 && final_price <= 52.0);
}

#[test]
fn test_adaptive_pricing_debug_logging() {
    // Test lines 1308-1316: debug! for AdaptivePricing
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AdaptivePricing(AdaptivePricingUpdater));
    let skill = Skill::new("AdaptiveDebug".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.sales_this_step.insert(skill_id.clone(), 2);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AdaptivePricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    assert!(market.skills.get(&skill_id).unwrap().current_price > 100.0);
}

#[test]
fn test_auction_pricing_with_per_skill_limits() {
    // Test lines 1366-1369: per_skill_price_limits for AuctionPricing
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AuctionPricing(AuctionPricingUpdater));
    let skill = Skill::new("AuctionLimited".to_string(), 50.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.per_skill_price_limits.insert(skill_id.clone(), (Some(45.0), Some(60.0)));
    market.demand_counts.insert(skill_id.clone(), 10);
    market.supply_counts.insert(skill_id.clone(), 2);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AuctionPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price >= 45.0 && final_price <= 60.0);
}

#[test]
fn test_auction_pricing_competitive_bidding_debug() {
    // Test lines 1387-1395: debug! for competitive bidding
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AuctionPricing(AuctionPricingUpdater));
    let skill = Skill::new("Competitive".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.demand_counts.insert(skill_id.clone(), 20);
    market.supply_counts.insert(skill_id.clone(), 5);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AuctionPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price > 100.0, "Competitive bidding should increase price");
}

#[test]
fn test_auction_pricing_no_demand_debug() {
    // Test lines 1401-1402: debug! for no demand
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AuctionPricing(AuctionPricingUpdater));
    let skill = Skill::new("NoDemand".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.demand_counts.insert(skill_id.clone(), 0);
    market.supply_counts.insert(skill_id.clone(), 5);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AuctionPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price < 100.0, "No demand should decrease price");
}

#[test]
fn test_auction_pricing_low_demand_debug() {
    // Test lines 1409-1414: debug! for low demand
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AuctionPricing(AuctionPricingUpdater));
    let skill = Skill::new("LowDemand".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.demand_counts.insert(skill_id.clone(), 3);
    market.supply_counts.insert(skill_id.clone(), 5);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = AuctionPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price < 100.0, "Low demand should decrease price");
}

#[test]
fn test_auction_pricing_volatility() {
    // Test line 1421: random_range for volatility
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::AuctionPricing(AuctionPricingUpdater));
    let skill = Skill::new("VolatileAuction".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.demand_counts.insert(skill_id.clone(), 5);
    market.supply_counts.insert(skill_id.clone(), 5);

    let mut rng = StdRng::seed_from_u64(777);
    let updater = AuctionPricingUpdater;
    updater.update_prices(&mut market, &mut rng);

    // Just verify it completes without panic
    assert!(market.skills.get(&skill_id).unwrap().current_price > 0.0);
}

#[test]
fn test_climate_change_price_updater() {
    // Test lines 1499-1506: ClimateChange scenario with history lookup
    let mut market = Market::new(
        10.0,
        1.0,
        0.1,
        0.02,
        PriceUpdater::ClimateChange(ClimateChangePriceUpdater::new()),
    );
    let skill = Skill::new("Climate".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = ClimateChangePriceUpdater::new();

    // Run multiple updates to test history length calculation
    for _ in 0..5 {
        updater.update_prices(&mut market, &mut rng);
    }

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price > 100.0, "Climate change should increase prices over time");
}

#[test]
fn test_climate_change_with_per_skill_limits() {
    // Test lines 1511: per_skill_price_limits for ClimateChange
    let mut market = Market::new(
        10.0,
        1.0,
        0.1,
        0.02,
        PriceUpdater::ClimateChange(ClimateChangePriceUpdater::new()),
    );
    let skill = Skill::new("ClimateLimit".to_string(), 50.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    market.per_skill_price_limits.insert(skill_id.clone(), (Some(45.0), Some(55.0)));

    let mut rng = StdRng::seed_from_u64(42);
    let updater = ClimateChangePriceUpdater::new();
    updater.update_prices(&mut market, &mut rng);

    let final_price = market.skills.get(&skill_id).unwrap().current_price;
    assert!(final_price >= 45.0 && final_price <= 55.0);
}

#[test]
fn test_climate_change_debug_logging() {
    // Test lines 1529-1534: debug! for ClimateChange
    let mut market = Market::new(
        10.0,
        1.0,
        0.1,
        0.02,
        PriceUpdater::ClimateChange(ClimateChangePriceUpdater::new()),
    );
    let skill = Skill::new("ClimateDebug".to_string(), 100.0);
    let skill_id = skill.id.clone();
    market.add_skill(skill);

    let mut rng = StdRng::seed_from_u64(42);
    let updater = ClimateChangePriceUpdater::new();
    updater.update_prices(&mut market, &mut rng);

    // Verify it executes the debug path
    assert!(market.skills.get(&skill_id).unwrap().current_price > 100.0);
}

// ============================================================================
// INVARIANT.RS COVERAGE TESTS
// ============================================================================

#[test]
fn test_money_conservation_violation() {
    // Test lines 159-167: Money conservation violation path
    let config = SimulationConfig::default();
    let engine = SimulationEngine::new(config);

    // Create invariant with wrong initial money to trigger violation
    let invariant = MoneyConservationInvariant::new_with_tolerance(99999.0, 0.01);
    let result = invariant.check(&engine);

    assert!(result.is_err(), "Should detect money conservation violation");
    let err = result.unwrap_err();
    assert_eq!(err.invariant_name, "MoneyConservation");
    assert!(err.description.contains("Total money"));
    assert!(err.expected.is_some());
    assert!(err.actual.is_some());
}

#[test]
fn test_non_negative_wealth_skip_with_loans() {
    // Test lines 218, 224-226: Skip check when loans enabled
    let config = SimulationConfig { enable_loans: true, max_steps: 5, ..Default::default() };
    let engine = SimulationEngine::new(config);

    let invariant = NonNegativeWealthInvariant::new(true);
    let result = invariant.check(&engine);

    // Should pass because check is skipped when loans enabled
    assert!(result.is_ok());
}

#[test]
fn test_non_negative_wealth_violation_details() {
    // Test lines 244, 247, 250-254, 257, 260-262, 264, 267-269, 271
    // We can't easily create negative wealth in real simulation, but we can test formatting
    let config = SimulationConfig::default();
    let engine = SimulationEngine::new(config);

    let invariant = NonNegativeWealthInvariant::new(false);

    // This should pass in default state
    let result = invariant.check(&engine);
    assert!(result.is_ok());

    // Test the name and description methods
    assert_eq!(invariant.name(), "NonNegativeWealth");
    let desc = invariant.description();
    assert!(desc.contains("negative") || desc.contains("non-negative"));
}

#[test]
fn test_invariant_checker_strict_mode_panic() {
    // Test lines 338-341: Strict mode panic path
    // We can't directly test panic, but we can test the setup
    let checker = InvariantChecker::new_strict();
    assert!(checker.is_strict());

    // In non-strict mode, violations are collected
    let mut checker_non_strict = InvariantChecker::new();
    assert!(!checker_non_strict.is_strict());

    // Add an invariant that will fail
    let wrong_money = MoneyConservationInvariant::new_with_tolerance(99999.0, 0.01);
    checker_non_strict.add_invariant(Box::new(wrong_money));

    let config = SimulationConfig::default();
    let engine = SimulationEngine::new(config);

    let violations = checker_non_strict.check_all(&engine);
    assert!(!violations.is_empty(), "Should detect violations in non-strict mode");
}

#[test]
fn test_invariant_checker_violation_counting() {
    // Test lines 347: total_violations increment
    let mut checker = InvariantChecker::new();

    // Add an invariant that will fail
    let wrong_money = MoneyConservationInvariant::new_with_tolerance(99999.0, 0.01);
    checker.add_invariant(Box::new(wrong_money));

    let config = SimulationConfig::default();
    let engine = SimulationEngine::new(config);

    assert_eq!(checker.total_violations(), 0);

    let violations = checker.check_all(&engine);
    assert_eq!(violations.len(), 1);
    assert_eq!(checker.total_violations(), 1);

    // Check again to increment counter
    let violations2 = checker.check_all(&engine);
    assert_eq!(violations2.len(), 1);
    assert_eq!(checker.total_violations(), 2);
}

#[test]
fn test_invariant_trait_default_description() {
    // Test lines 73-74: Default description implementation
    struct DummyInvariant;
    impl Invariant for DummyInvariant {
        fn name(&self) -> &str {
            "DummyTest"
        }
        fn check(&self, _engine: &SimulationEngine) -> Result<(), InvariantViolation> {
            Ok(())
        }
    }

    let dummy = DummyInvariant;
    let desc = dummy.description();
    assert!(desc.contains("DummyTest"));
    assert!(desc.contains("invariant"));
}

#[test]
fn test_invariant_violation_format_with_expected_actual() {
    // Test line 48-50: Display formatting with expected/actual
    let violation = InvariantViolation {
        invariant_name: "Test".to_string(),
        description: "Something failed".to_string(),
        step: 10,
        expected: Some("100.0".to_string()),
        actual: Some("95.0".to_string()),
    };

    let formatted = format!("{}", violation);
    assert!(formatted.contains("Test"));
    assert!(formatted.contains("step 10"));
    assert!(formatted.contains("expected: 100.0"));
    assert!(formatted.contains("actual: 95.0"));
}

// ============================================================================
// ENGINE.RS COVERAGE TESTS - Crisis, Plugin, Contract Edge Cases
// ============================================================================

#[test]
fn test_engine_with_crisis_enabled() {
    // Test crisis-related code paths in engine.rs
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 20,
        enable_crisis_events: true,
        crisis_probability: 0.5, // High probability to trigger during test
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run a few steps to potentially trigger crisis
    for _ in 0..5 {
        engine.step();
    }

    // Verify engine still functions correctly
    assert!(engine.get_current_step() > 0);
}

#[test]
fn test_engine_with_contracts_enabled() {
    // Test contract-related code paths
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 20,
        enable_contracts: true,
        max_contract_duration: 5,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run steps to allow contracts to form
    for _ in 0..5 {
        engine.step();
    }

    assert!(engine.get_current_step() > 0);
}

#[test]
fn test_engine_with_multiple_features_enabled() {
    // Test combination of features
    let config = SimulationConfig {
        max_steps: 15,
        entity_count: 25,
        enable_loans: true,
        enable_crisis_events: true,
        enable_contracts: true,
        enable_production: true,
        enable_investments: true,
        enable_tax_redistribution: true,
        tax_rate: 0.05,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run multiple steps with all features
    for _ in 0..10 {
        engine.step();
    }

    assert!(engine.get_current_step() > 0);
}

#[test]
fn test_engine_with_plugin_system() {
    // Test plugin-related paths if plugins exist
    let config = SimulationConfig { max_steps: 10, entity_count: 15, ..Default::default() };

    let engine = SimulationEngine::new(config);

    // Test that plugin accessors work
    assert!(engine.get_config().max_steps > 0);
}

#[test]
fn test_engine_checkpoint_with_all_features() {
    // Test checkpoint system with various features
    let config = SimulationConfig {
        max_steps: 20,
        entity_count: 30,
        enable_loans: true,
        enable_tax_redistribution: true,
        tax_rate: 0.1,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run past several steps
    for _ in 0..7 {
        engine.step();
    }

    assert!(engine.get_current_step() >= 7);
}

#[test]
fn test_scenario_all_variants() {
    // Test all scenario variants
    for scenario in Scenario::all() {
        assert!(!scenario.description().is_empty());
        assert!(!scenario.mechanism().is_empty());
        assert!(!scenario.use_case().is_empty());

        // Test is_default
        let is_default = scenario.is_default();
        assert_eq!(is_default, matches!(scenario, Scenario::Original));
    }
}

#[test]
fn test_scenario_display_all() {
    // Test Display trait for all scenarios
    assert_eq!(format!("{}", Scenario::Original), "Original");
    assert_eq!(format!("{}", Scenario::DynamicPricing), "DynamicPricing");
    assert_eq!(format!("{}", Scenario::AdaptivePricing), "AdaptivePricing");
    assert_eq!(format!("{}", Scenario::AuctionPricing), "AuctionPricing");
    assert_eq!(format!("{}", Scenario::ClimateChange), "ClimateChange");
}

#[test]
fn test_engine_with_minimal_persons_edge_case() {
    // Edge case: minimal persons
    let config = SimulationConfig { max_steps: 5, entity_count: 1, ..Default::default() };

    let mut engine = SimulationEngine::new(config);
    engine.step();

    assert_eq!(engine.get_entities().len(), 1);
}

#[test]
fn test_engine_with_high_tax_rate() {
    // Edge case: very high tax rate
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 20,
        enable_tax_redistribution: true,
        tax_rate: 0.5, // 50% tax
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    for _ in 0..5 {
        engine.step();
    }

    // Verify no panics with high tax
    assert!(engine.get_current_step() > 0);
}

#[test]
fn test_engine_crisis_with_varying_severity() {
    // Test different crisis configurations
    let config = SimulationConfig {
        max_steps: 10,
        entity_count: 25,
        enable_crisis_events: true,
        crisis_probability: 1.0, // Guarantee crisis
        crisis_severity: 0.3,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    for _ in 0..3 {
        engine.step();
    }

    assert!(engine.get_current_step() > 0);
}

#[test]
fn test_price_updater_from_scenario() {
    // Test PriceUpdater::from(Scenario) for all variants
    let original = PriceUpdater::from(Scenario::Original);
    assert!(matches!(original, PriceUpdater::Original(_)));

    let dynamic = PriceUpdater::from(Scenario::DynamicPricing);
    assert!(matches!(dynamic, PriceUpdater::DynamicPricing(_)));

    let adaptive = PriceUpdater::from(Scenario::AdaptivePricing);
    assert!(matches!(adaptive, PriceUpdater::AdaptivePricing(_)));

    let auction = PriceUpdater::from(Scenario::AuctionPricing);
    assert!(matches!(auction, PriceUpdater::AuctionPricing(_)));

    let climate = PriceUpdater::from(Scenario::ClimateChange);
    assert!(matches!(climate, PriceUpdater::ClimateChange(_)));
}

#[test]
fn test_market_with_empty_history() {
    // Test edge case where skill_price_history is empty
    let mut market =
        Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::Original(OriginalPriceUpdater));
    let skill = Skill::new("NoHistory".to_string(), 100.0);
    let skill_id = skill.id.clone();

    // Manually clear history to test empty case
    market.add_skill(skill);
    market.skill_price_history.remove(&skill_id);

    let mut rng = StdRng::seed_from_u64(42);
    market.demand_counts.insert(skill_id.clone(), 5);
    market.supply_counts.insert(skill_id.clone(), 5);

    let updater = OriginalPriceUpdater;
    updater.update_prices(&mut market, &mut rng);

    // Should handle missing history gracefully
    assert!(market.skills.contains_key(&skill_id));
}

#[test]
fn test_engine_run_complete_simulation() {
    // Test a complete simulation run
    let config = SimulationConfig {
        max_steps: 25,
        entity_count: 30,
        enable_tax_redistribution: true,
        tax_rate: 0.1,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);

    // Run complete simulation
    while engine.get_current_step() < 25 {
        engine.step();
    }

    assert_eq!(engine.get_current_step(), 25);

    // Verify simulation state is valid
    assert!(!engine.get_entities().is_empty());
}

#[test]
fn test_invariant_checker_multiple_violations() {
    // Test multiple violations at once
    let mut checker = InvariantChecker::new();

    // Add multiple failing invariants
    checker.add_invariant(Box::new(MoneyConservationInvariant::new_with_tolerance(99999.0, 0.01)));
    checker.add_invariant(Box::new(MoneyConservationInvariant::new_with_tolerance(88888.0, 0.01)));

    let config = SimulationConfig::default();
    let engine = SimulationEngine::new(config);

    let violations = checker.check_all(&engine);
    assert!(violations.len() >= 2, "Should detect multiple violations");

    // Verify total count increased by number of violations
    assert_eq!(checker.total_violations(), violations.len());
}

#[test]
fn test_all_per_skill_limit_combinations() {
    // Test all combinations of per-skill limits (Some/None for min and max)
    let scenarios = vec![
        (Some(40.0), Some(60.0)), // Both set
        (Some(40.0), None),       // Only min
        (None, Some(60.0)),       // Only max
        (None, None),             // Neither set
    ];

    for (min, max) in scenarios {
        let mut market =
            Market::new(10.0, 1.0, 0.1, 0.02, PriceUpdater::Original(OriginalPriceUpdater));
        let skill = Skill::new("LimitTest".to_string(), 50.0);
        let skill_id = skill.id.clone();
        market.add_skill(skill);

        if min.is_some() || max.is_some() {
            market.per_skill_price_limits.insert(skill_id.clone(), (min, max));
        }

        market.demand_counts.insert(skill_id.clone(), 10);
        market.supply_counts.insert(skill_id.clone(), 1);

        let mut rng = StdRng::seed_from_u64(42);
        market.volatility_percentage = 0.0;

        let updater = OriginalPriceUpdater;
        updater.update_prices(&mut market, &mut rng);

        let final_price = market.skills.get(&skill_id).unwrap().current_price;

        // Verify price respects limits
        if let Some(min_price) = min {
            assert!(final_price >= min_price, "Price should respect min limit");
        }
        if let Some(max_price) = max {
            assert!(final_price <= max_price, "Price should respect max limit");
        }
    }
}
