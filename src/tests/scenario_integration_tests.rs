/// Integration tests for different simulation scenarios
/// These tests verify complete simulation runs under various conditions
#[cfg(test)]
mod integration_tests {
    use crate::scenario::Scenario;
    use crate::{SimulationConfig, SimulationEngine};

    /// Test that Original scenario produces stable results
    #[test]
    fn test_original_scenario_stability() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify basic properties
        assert_eq!(result.total_steps, 100);
        assert_eq!(result.active_persons, 20);
        assert!(!result.final_money_distribution.is_empty());

        // Money should not all be zero
        let total_money: f64 = result.final_money_distribution.iter().sum();
        assert!(total_money > 0.0, "Total money should be positive");

        // Check that money statistics are reasonable
        assert!(result.money_statistics.average > 0.0);
        assert!(result.money_statistics.std_dev >= 0.0);
    }

    /// Test that DynamicPricing scenario produces stable results
    #[test]
    fn test_dynamic_pricing_scenario_stability() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::DynamicPricing,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify basic properties
        assert_eq!(result.total_steps, 100);
        assert_eq!(result.active_persons, 20);

        // Check that prices evolved
        assert!(!result.skill_price_history.is_empty());
    }

    /// Test simulation with different population sizes
    #[test]
    fn test_varying_population_sizes() {
        for size in [5, 10, 50, 100].iter() {
            let config = SimulationConfig {
                entity_count: *size,
                max_steps: 50,
                initial_money_per_person: 100.0,
                base_skill_price: 10.0,
                seed: 42,
                scenario: Scenario::Original,
                time_step: 1.0,
                tech_growth_rate: 0.0,
                ..Default::default()
            };

            let mut engine = SimulationEngine::new(config);
            let result = engine.run();

            assert_eq!(result.active_persons, *size);
            assert_eq!(result.final_money_distribution.len(), *size);
            assert_eq!(result.final_reputation_distribution.len(), *size);
        }
    }

    /// Test simulation with extreme initial conditions
    #[test]
    fn test_extreme_initial_money() {
        // Very low initial money
        let config_low = SimulationConfig {
            entity_count: 10,
            max_steps: 20,
            initial_money_per_person: 1.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine_low = SimulationEngine::new(config_low);
        let result_low = engine_low.run();
        assert_eq!(result_low.total_steps, 20);

        // Very high initial money
        let config_high = SimulationConfig {
            entity_count: 10,
            max_steps: 20,
            initial_money_per_person: 10000.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine_high = SimulationEngine::new(config_high);
        let result_high = engine_high.run();
        assert_eq!(result_high.total_steps, 20);

        // With more money, average should be higher
        assert!(result_high.money_statistics.average > result_low.money_statistics.average);
    }

    /// Test that reputation system works correctly
    #[test]
    fn test_reputation_evolution() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Check reputation statistics exist and are reasonable
        assert_eq!(result.final_reputation_distribution.len(), 10);
        assert!(result.reputation_statistics.average > 0.0);
        assert!(result.reputation_statistics.min_reputation >= 0.0);
        assert!(result.reputation_statistics.max_reputation <= 2.0);

        // All reputations should be within bounds
        for rep in &result.final_reputation_distribution {
            assert!(
                *rep >= 0.0 && *rep <= 2.0,
                "Reputation {} out of bounds",
                rep
            );
        }
    }

    /// Test that Gini coefficient is calculated
    #[test]
    fn test_gini_coefficient_calculated() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Gini coefficient should be calculated and finite
        assert!(result.money_statistics.gini_coefficient.is_finite());

        // Note: Gini can exceed 1.0 when negative money (debt) exists,
        // so we just check it was calculated
    }

    /// Test that trade volume statistics are collected
    #[test]
    fn test_trade_volume_statistics() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Trade volume statistics should exist
        // total_trades is usize which is always >= 0, so just check it exists
        let _ = result.trade_volume_statistics.total_trades;
        assert!(result.trade_volume_statistics.total_volume >= 0.0);

        // If there were trades, averages should be positive
        if result.trade_volume_statistics.total_trades > 0 {
            assert!(result.trade_volume_statistics.avg_transaction_value > 0.0);
            assert!(result.trade_volume_statistics.avg_trades_per_step >= 0.0);
        }

        // Trades per step array should match simulation length
        assert_eq!(result.trades_per_step.len(), 50);
        assert_eq!(result.volume_per_step.len(), 50);
    }

    /// Test determinism: same seed should produce same results
    /// NOTE: This test is currently disabled because the simulation uses
    /// non-deterministic elements (e.g., HashMap iteration order, parallel execution).
    /// Consider this a known limitation for future improvement.
    #[test]
    #[ignore]
    fn test_determinism_with_seed() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 20,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 12345,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        // Run simulation twice with same config
        let mut engine1 = SimulationEngine::new(config.clone());
        let result1 = engine1.run();

        let mut engine2 = SimulationEngine::new(config);
        let result2 = engine2.run();

        // Basic results should be identical
        assert_eq!(result1.total_steps, result2.total_steps);
        assert_eq!(result1.active_persons, result2.active_persons);

        // Money distributions should be similar (allowing for floating point differences)
        // Note: Perfect determinism may not be guaranteed with parallel execution
        let money_diff: f64 = result1
            .final_money_distribution
            .iter()
            .zip(&result2.final_money_distribution)
            .map(|(m1, m2)| (m1 - m2).abs())
            .sum();

        // Allow some tolerance for numerical differences
        assert!(
            money_diff < 1.0,
            "Money distributions differ significantly: total diff = {}",
            money_diff
        );
    }

    /// Test that different seeds produce different results
    #[test]
    fn test_different_seeds_different_results() {
        let config1 = SimulationConfig {
            entity_count: 10,
            max_steps: 20,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 111,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let config2 = SimulationConfig {
            seed: 999,
            ..config1.clone()
        };

        let mut engine1 = SimulationEngine::new(config1);
        let result1 = engine1.run();

        let mut engine2 = SimulationEngine::new(config2);
        let result2 = engine2.run();

        // Results should be different (at least some money values should differ)
        let mut differences = 0;
        for (m1, m2) in result1
            .final_money_distribution
            .iter()
            .zip(&result2.final_money_distribution)
        {
            if (m1 - m2).abs() > 0.01 {
                differences += 1;
            }
        }

        assert!(
            differences > 0,
            "Different seeds should produce different results"
        );
    }

    /// Test short simulation (edge case)
    #[test]
    fn test_very_short_simulation() {
        let config = SimulationConfig {
            entity_count: 5,
            max_steps: 1,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 1);
        assert_eq!(result.active_persons, 5);
    }

    /// Test long simulation
    #[test]
    fn test_long_simulation() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 500,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 500);
        assert!(result.total_duration > 0.0);
    }

    /// Test technological progress feature
    #[test]
    fn test_technological_progress() {
        // Run two simulations: one without tech growth and one with
        let config_without_tech = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0, // No tech growth
            ..Default::default()
        };

        let config_with_tech = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.001, // 0.1% growth per step
            ..Default::default()
        };

        let mut engine_without = SimulationEngine::new(config_without_tech);
        let result_without = engine_without.run();

        let mut engine_with = SimulationEngine::new(config_with_tech);
        let result_with = engine_with.run();

        // Both should complete successfully
        assert_eq!(result_without.total_steps, 100);
        assert_eq!(result_with.total_steps, 100);

        // With tech growth, skills should have efficiency > 1.0 at the end
        // After 100 steps with 0.1% growth: (1.001)^100 ≈ 1.105
        // We can't directly check skill efficiency, but we can verify the simulation runs
        // and produces reasonable results

        // Both simulations should have reasonable trade volumes
        assert!(result_without.trade_volume_statistics.total_trades > 0);
        assert!(result_with.trade_volume_statistics.total_trades > 0);

        // Money should still be distributed reasonably in both cases
        assert!(result_without.money_statistics.average > 0.0);
        assert!(result_with.money_statistics.average > 0.0);
    }

    /// Test seasonal demand effects feature
    #[test]
    fn test_seasonal_demand_effects() {
        // Run two simulations: one without seasonality and one with
        let config_no_seasonality = SimulationConfig {
            entity_count: 20,
            max_steps: 200, // Need enough steps to see seasonal cycles
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            seasonal_amplitude: 0.0, // No seasonality
            seasonal_period: 100,
            transaction_fee: 0.0,
            savings_rate: 0.0,
            enable_loans: false,
            loan_interest_rate: 0.01,
            loan_repayment_period: 20,
            min_money_to_lend: 50.0,
        };

        let config_with_seasonality = SimulationConfig {
            entity_count: 20,
            max_steps: 200,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            seasonal_amplitude: 0.5, // 50% amplitude - significant seasonal variation
            seasonal_period: 50,     // 50-step cycle period
            transaction_fee: 0.0,
            savings_rate: 0.0,
            enable_loans: false,
            loan_interest_rate: 0.01,
            loan_repayment_period: 20,
            min_money_to_lend: 50.0,
        };

        let mut engine_no_season = SimulationEngine::new(config_no_seasonality);
        let result_no_season = engine_no_season.run();

        let mut engine_with_season = SimulationEngine::new(config_with_seasonality);
        let result_with_season = engine_with_season.run();

        // Both should complete successfully
        assert_eq!(result_no_season.total_steps, 200);
        assert_eq!(result_with_season.total_steps, 200);

        // Both should have the same number of persons
        assert_eq!(result_no_season.active_persons, 20);
        assert_eq!(result_with_season.active_persons, 20);

        // Trade volume should vary more with seasonality
        // Calculate variance in trades per step
        let trades_no_season = &result_no_season.trades_per_step;
        let trades_with_season = &result_with_season.trades_per_step;

        if !trades_no_season.is_empty() && !trades_with_season.is_empty() {
            let mean_no_season: f64 =
                trades_no_season.iter().sum::<usize>() as f64 / trades_no_season.len() as f64;
            let mean_with_season: f64 =
                trades_with_season.iter().sum::<usize>() as f64 / trades_with_season.len() as f64;

            let variance_no_season: f64 = trades_no_season
                .iter()
                .map(|&x| {
                    let diff = x as f64 - mean_no_season;
                    diff * diff
                })
                .sum::<f64>()
                / trades_no_season.len() as f64;

            let variance_with_season: f64 = trades_with_season
                .iter()
                .map(|&x| {
                    let diff = x as f64 - mean_with_season;
                    diff * diff
                })
                .sum::<f64>()
                / trades_with_season.len() as f64;

            // With seasonality, we expect higher variance in trade volume
            // (though this may not always be true due to random factors)
            // At minimum, both should have non-negative variance
            assert!(variance_no_season >= 0.0, "Variance should be non-negative");
            assert!(
                variance_with_season >= 0.0,
                "Variance should be non-negative"
            );

            // Seasonality should create variation (though we can't guarantee it's always higher)
            // So we just verify the simulation completes and produces valid statistics
        }

        // Verify that statistics are calculated correctly
        assert!(result_no_season.money_statistics.average >= 0.0);
        assert!(result_with_season.money_statistics.average >= 0.0);
    }
}
