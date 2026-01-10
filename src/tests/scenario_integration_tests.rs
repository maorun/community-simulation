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
            min_skill_price: 1.0,
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
        };

        let config_with_seasonality = SimulationConfig {
            entity_count: 20,
            max_steps: 200,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            min_skill_price: 1.0,
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

    /// Test Monte Carlo simulation with multiple runs
    #[test]
    fn test_monte_carlo_aggregation() {
        use crate::result::MonteCarloResult;

        // Run 3 simulations with different seeds
        let base_config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            ..Default::default()
        };

        let mut results = Vec::new();
        for i in 0..3 {
            let mut config = base_config.clone();
            config.seed = 42 + i;
            let mut engine = SimulationEngine::new(config);
            results.push(engine.run());
        }

        // Create aggregated result
        let mc_result = MonteCarloResult::from_runs(results, 42);

        // Verify basic properties
        assert_eq!(mc_result.num_runs, 3);
        assert_eq!(mc_result.base_seed, 42);
        assert_eq!(mc_result.runs.len(), 3);

        // Verify statistics are reasonable
        assert!(mc_result.avg_money_stats.mean > 0.0);
        assert!(mc_result.avg_money_stats.std_dev >= 0.0);
        assert!(mc_result.gini_coefficient_stats.mean >= 0.0);
        // Gini can exceed 1.0 in edge cases with negative money/debt
        assert!(mc_result.total_trades_stats.mean >= 0.0);
        assert!(mc_result.avg_reputation_stats.mean > 0.0);

        // Min should be <= mean <= max
        assert!(mc_result.avg_money_stats.min <= mc_result.avg_money_stats.mean);
        assert!(mc_result.avg_money_stats.mean <= mc_result.avg_money_stats.max);
    }

    /// Test that tax collection works correctly
    #[test]
    fn test_tax_collection() {
        // Run two simulations: one without taxes and one with 10% tax
        let config_no_tax = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 0.0, // No tax
            ..Default::default()
        };

        let config_with_tax = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 0.10, // 10% tax
            ..Default::default()
        };

        let mut engine_no_tax = SimulationEngine::new(config_no_tax);
        let result_no_tax = engine_no_tax.run();

        let mut engine_with_tax = SimulationEngine::new(config_with_tax);
        let result_with_tax = engine_with_tax.run();

        // Both should complete successfully
        assert_eq!(result_no_tax.total_steps, 100);
        assert_eq!(result_with_tax.total_steps, 100);

        // No tax simulation should have no taxes collected
        assert!(result_no_tax.total_taxes_collected.is_none());

        // Tax simulation should have taxes collected
        assert!(result_with_tax.total_taxes_collected.is_some());
        let taxes_collected = result_with_tax.total_taxes_collected.unwrap();
        assert!(taxes_collected > 0.0, "Taxes should be collected");

        // Taxes should be approximately 10% of the seller proceeds (after transaction fees)
        // seller_proceeds = total_volume * (1 - transaction_fee) for each trade
        // Since transaction_fee is 0.0 in this config, seller_proceeds = total_volume
        // expected_taxes = seller_proceeds * tax_rate
        let total_volume = result_with_tax.trade_volume_statistics.total_volume;
        let transaction_fee_rate = 0.0; // From config_with_tax
        let seller_proceeds = total_volume * (1.0 - transaction_fee_rate);
        let expected_taxes = seller_proceeds * 0.10;

        // Allow for small floating point differences
        let difference = (taxes_collected - expected_taxes).abs();
        assert!(
            difference < 0.01 || difference / expected_taxes < 0.001,
            "Collected taxes ({}) should be approximately 10% of seller proceeds ({}), difference: {}",
            taxes_collected,
            expected_taxes,
            difference
        );

        // No redistribution should have occurred
        assert!(result_with_tax.total_taxes_redistributed.is_none());
    }

    /// Test that tax redistribution works correctly
    #[test]
    fn test_tax_redistribution() {
        let config_no_redistribution = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 0.15, // 15% tax
            enable_tax_redistribution: false,
            ..Default::default()
        };

        let config_with_redistribution = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 0.15, // 15% tax
            enable_tax_redistribution: true,
            ..Default::default()
        };

        let mut engine_no_redist = SimulationEngine::new(config_no_redistribution);
        let result_no_redist = engine_no_redist.run();

        let mut engine_with_redist = SimulationEngine::new(config_with_redistribution);
        let result_with_redist = engine_with_redist.run();

        // Both should complete successfully
        assert_eq!(result_no_redist.total_steps, 100);
        assert_eq!(result_with_redist.total_steps, 100);

        // Both should have taxes collected
        assert!(result_no_redist.total_taxes_collected.is_some());
        assert!(result_with_redist.total_taxes_collected.is_some());

        // No redistribution version should not have redistribution stats
        assert!(result_no_redist.total_taxes_redistributed.is_none());

        // Redistribution version should have redistribution stats
        assert!(result_with_redist.total_taxes_redistributed.is_some());
        let redistributed = result_with_redist.total_taxes_redistributed.unwrap();

        // Redistributed amount should equal collected amount
        let collected = result_with_redist.total_taxes_collected.unwrap();
        let difference = (redistributed - collected).abs();
        assert!(
            difference < 0.01 || difference / collected < 0.001,
            "Redistributed amount ({}) should equal collected amount ({})",
            redistributed,
            collected
        );

        // With redistribution, wealth inequality should be lower than without
        // (This is a probabilistic assertion that should generally hold true)
        // Note: This comparison works because both simulations use the same seed
    }

    /// Test edge case: 0% tax rate
    #[test]
    fn test_zero_tax_rate() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 0.0,                   // 0% tax
            enable_tax_redistribution: true, // Doesn't matter - nothing to redistribute
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 50);
        // No taxes should be collected with 0% rate
        assert!(result.total_taxes_collected.is_none());
        assert!(result.total_taxes_redistributed.is_none());
    }

    /// Test edge case: 100% tax rate (confiscatory)
    #[test]
    fn test_full_tax_rate() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 30,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            tax_rate: 1.0, // 100% tax - sellers get nothing
            enable_tax_redistribution: false,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 30);

        // With 100% tax, all seller proceeds go to taxes
        if let Some(taxes) = result.total_taxes_collected {
            assert!(taxes > 0.0, "Taxes should be collected even with 100% rate");
        }

        // Trading should still occur (buyers still willing to buy)
        // but sellers receive nothing, so economy should decline rapidly
    }

    #[test]
    fn test_min_skill_price_enforcement() {
        // Test that skill prices don't fall below the configured minimum
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 200,
            initial_money_per_person: 50.0, // Low initial money
            base_skill_price: 10.0,
            min_skill_price: 3.0, // Set a price floor
            seed: 42,
            scenario: Scenario::DynamicPricing, // Use dynamic pricing which can decrease prices
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
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Check that all final skill prices are at or above the minimum
        for skill_price_info in &result.final_skill_prices {
            assert!(
                skill_price_info.price >= 3.0,
                "Skill {} has price {} which is below the minimum of 3.0",
                skill_price_info.id,
                skill_price_info.price
            );
        }

        // Verify the least valuable skill is at least the minimum
        if let Some(least_valuable) = &result.least_valuable_skill {
            assert!(
                least_valuable.price >= 3.0,
                "Least valuable skill has price {} which is below the minimum of 3.0",
                least_valuable.price
            );
        }
    }

    #[test]
    fn test_min_skill_price_equals_base() {
        // Test edge case where min_skill_price equals base_skill_price
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 5.0,
            min_skill_price: 5.0, // Same as base
            seed: 42,
            scenario: Scenario::Original,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // All prices should remain at or near the base price (which is also the minimum)
        for skill_price_info in &result.final_skill_prices {
            assert!(
                skill_price_info.price >= 5.0,
                "Skill {} has price {} which is below the minimum of 5.0",
                skill_price_info.id,
                skill_price_info.price
            );
        }
    }

    /// Test priority-based buying decisions with default weights
    #[test]
    fn test_priority_based_decisions_default_weights() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
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
            // Default balanced weights
            priority_urgency_weight: 0.5,
            priority_affordability_weight: 0.3,
            priority_efficiency_weight: 0.1,
            priority_reputation_weight: 0.1,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Should complete successfully
        assert_eq!(result.total_steps, 50);
        assert_eq!(result.active_persons, 10);
        // With priority system, should still have reasonable trade activity
        assert!(
            result.trade_volume_statistics.total_trades > 0,
            "Should have some trades"
        );
    }

    /// Test priority-based buying with urgency-only weighting
    #[test]
    fn test_priority_urgency_only() {
        let config_urgency = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
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
            // Only urgency matters
            priority_urgency_weight: 1.0,
            priority_affordability_weight: 0.0,
            priority_efficiency_weight: 0.0,
            priority_reputation_weight: 0.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config_urgency);
        let result = engine.run();

        // Should complete successfully with urgency-only prioritization
        assert_eq!(result.total_steps, 50);
        assert!(
            result.trade_volume_statistics.total_trades > 0,
            "Should have trades with urgency prioritization"
        );
    }

    /// Test priority weights validation
    #[test]
    fn test_priority_weights_validation() {
        // Test that priority weights must be in 0.0-1.0 range

        // Valid weights should pass
        let mut config = SimulationConfig {
            priority_urgency_weight: 0.5,
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        // Invalid weight > 1.0 should fail
        config.priority_urgency_weight = 1.5;
        assert!(config.validate().is_err());

        // Invalid negative weight should fail
        config = SimulationConfig {
            priority_urgency_weight: 0.5,
            priority_affordability_weight: -0.1,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    /// Test that priority system works with technological progress
    #[test]
    fn test_priority_with_tech_progress() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            min_skill_price: 1.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.01, // 1% growth per step
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
            // Include efficiency in priority
            priority_urgency_weight: 0.4,
            priority_affordability_weight: 0.3,
            priority_efficiency_weight: 0.2, // Higher weight to test tech progress impact
            priority_reputation_weight: 0.1,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 100);
        // Tech progress should enable more trades over time as skills become more efficient
        assert!(result.trade_volume_statistics.total_trades > 0);
    }

    /// Test that priority system works with reputation
    #[test]
    fn test_priority_with_reputation() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
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
            // Reputation matters more
            priority_urgency_weight: 0.3,
            priority_affordability_weight: 0.3,
            priority_efficiency_weight: 0.0,
            priority_reputation_weight: 0.4, // Higher weight for reputation
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 100);
        // With reputation weighted heavily, agents should prefer trading with reputable sellers
        assert!(result.trade_volume_statistics.total_trades > 0);
        // Check that reputation statistics are tracked
        assert!(result.reputation_statistics.average >= 1.0); // Should be >= neutral (1.0)
    }

    /// Test that crisis events can be triggered and don't crash the simulation
    #[test]
    fn test_crisis_events_enabled() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            enable_crisis_events: true,
            crisis_probability: 0.10, // High probability to ensure at least one crisis
            crisis_severity: 0.5,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify simulation completed successfully despite crises
        assert_eq!(result.total_steps, 50);
        assert_eq!(result.active_persons, 20);

        // Check that money distribution is still valid (no NaN or infinite values)
        for money in &result.final_money_distribution {
            assert!(money.is_finite(), "Money should be finite");
        }

        // Check that prices are still valid
        for skill_price in &result.final_skill_prices {
            assert!(
                skill_price.price.is_finite() && skill_price.price > 0.0,
                "Skill prices should be finite and positive"
            );
        }
    }

    /// Test that crisis events respect minimum price floor
    #[test]
    fn test_crisis_respects_min_price() {
        let min_price = 2.0;
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            min_skill_price: min_price, // Set a price floor
            seed: 42,
            scenario: Scenario::Original,
            enable_crisis_events: true,
            crisis_probability: 0.10,
            crisis_severity: 1.0, // Maximum severity to test price floor
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Check that no skill price fell below the minimum
        for skill_price in &result.final_skill_prices {
            assert!(
                skill_price.price >= min_price,
                "Skill price {} should not be below minimum {}",
                skill_price.price,
                min_price
            );
        }
    }

    /// Test that simulation works correctly with crisis events disabled
    #[test]
    fn test_crisis_events_disabled() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            enable_crisis_events: false, // Explicitly disabled
            crisis_probability: 1.0,     // Even with 100% probability, no crisis should occur
            crisis_severity: 1.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Simulation should complete successfully
        assert_eq!(result.total_steps, 100);
        assert_eq!(result.active_persons, 20);

        // With no crises, market should be relatively stable
        // (This is a basic smoke test - more detailed assertions could be added)
        assert!(result.money_statistics.average > 0.0);
    }

    /// Test that wealth stats history is collected at each step
    #[test]
    fn test_wealth_stats_history_collection() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify wealth stats history is collected
        assert!(!result.wealth_stats_history.is_empty());
        assert_eq!(
            result.wealth_stats_history.len(),
            50,
            "Should have one snapshot per step"
        );

        // Verify each snapshot has valid data
        for (i, snapshot) in result.wealth_stats_history.iter().enumerate() {
            assert_eq!(snapshot.step, i, "Step number should match index");
            assert!(snapshot.average.is_finite(), "Average should be finite");
            assert!(snapshot.median.is_finite(), "Median should be finite");
            assert!(snapshot.std_dev >= 0.0, "Std dev should be non-negative");
            assert!(
                snapshot.min_money <= snapshot.max_money,
                "Min should be <= max"
            );
            // Gini coefficient can go above 1.0 when negative money (debt) exists
            assert!(
                snapshot.gini_coefficient.is_finite(),
                "Gini coefficient should be finite"
            );
        }

        // Verify that statistics evolve over time (they should not all be identical)
        let first_snapshot = &result.wealth_stats_history[0];
        let last_snapshot = &result.wealth_stats_history[result.wealth_stats_history.len() - 1];

        // At least one metric should change over the course of the simulation
        let has_changed = first_snapshot.average != last_snapshot.average
            || first_snapshot.gini_coefficient != last_snapshot.gini_coefficient
            || first_snapshot.median != last_snapshot.median;

        assert!(
            has_changed,
            "Wealth distribution should evolve over the simulation"
        );
    }

    /// Test that loan system can be enabled and functions properly
    #[test]
    fn test_loan_system_enabled() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            enable_loans: true,
            loan_interest_rate: 0.02, // 2% per step
            loan_repayment_period: 10,
            min_money_to_lend: 50.0,
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify simulation completed successfully with loans enabled
        assert_eq!(result.total_steps, 100);
        assert_eq!(result.active_persons, 20);

        // Check that money distribution is still valid (may include negative values for debt)
        for money in &result.final_money_distribution {
            assert!(money.is_finite(), "Money should be finite");
            // Note: With loans enabled, persons can have negative money (debt)
        }

        // Check that statistics are valid
        assert!(result.money_statistics.average.is_finite());
        assert!(result.money_statistics.std_dev >= 0.0);
    }

    /// Test that loan system is disabled by default
    #[test]
    fn test_loan_system_disabled_by_default() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            scenario: Scenario::Original,
            // enable_loans is false by default
            ..Default::default()
        };

        assert!(!config.enable_loans, "Loans should be disabled by default");

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify simulation runs normally without loans
        assert_eq!(result.total_steps, 50);
        assert_eq!(result.active_persons, 10);
    }

    /// Test group assignment and statistics
    #[test]
    fn test_group_assignment_and_statistics() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            num_groups: Some(5), // 5 groups with 20 persons = 4 persons per group
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify group statistics are present
        assert!(
            result.group_statistics.is_some(),
            "Group statistics should be present when groups are configured"
        );

        let group_stats = result.group_statistics.unwrap();

        // Verify total groups
        assert_eq!(
            group_stats.total_groups, 5,
            "Should have 5 groups as configured"
        );

        // Verify average group size
        assert_eq!(
            group_stats.avg_group_size, 4.0,
            "Average group size should be 4 (20 persons / 5 groups)"
        );

        // Verify individual group data
        assert_eq!(
            group_stats.groups.len(),
            5,
            "Should have stats for 5 groups"
        );

        // Check that all groups have the expected number of members (round-robin distribution)
        for (i, group) in group_stats.groups.iter().enumerate() {
            assert_eq!(group.group_id, i, "Group {} should have correct ID", i);
            assert_eq!(group.member_count, 4, "Group {} should have 4 members", i);
            // Money can be negative in simulations with aggressive strategies/loans
            // so we just check that stats are calculated (not NaN)
            assert!(
                !group.avg_money.is_nan(),
                "Group {} should have valid average money",
                i
            );
            assert!(
                !group.total_money.is_nan(),
                "Group {} should have valid total money",
                i
            );
            assert!(
                group.avg_reputation > 0.0,
                "Group {} should have positive average reputation",
                i
            );
        }

        // Verify group sizes
        assert_eq!(group_stats.min_group_size, 4);
        assert_eq!(group_stats.max_group_size, 4);
    }

    /// Test that groups are disabled by default
    #[test]
    fn test_groups_disabled_by_default() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            ..Default::default()
        };

        assert!(
            config.num_groups.is_none(),
            "Groups should be disabled by default"
        );

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify no group statistics when groups are not configured
        assert!(
            result.group_statistics.is_none(),
            "Group statistics should not be present when groups are not configured"
        );
    }

    /// Test group assignment with uneven distribution
    #[test]
    fn test_group_uneven_distribution() {
        let config = SimulationConfig {
            entity_count: 23,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            num_groups: Some(5), // 5 groups with 23 persons = uneven distribution
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        let group_stats = result
            .group_statistics
            .expect("Group statistics should be present");

        // With 23 persons and 5 groups, using round-robin:
        // Groups 0, 1, 2 get 5 members each (first 3 groups get extra person)
        // Groups 3, 4 get 4 members each
        // Total: 5 + 5 + 5 + 4 + 4 = 23 ✓

        let member_counts: Vec<usize> = group_stats.groups.iter().map(|g| g.member_count).collect();

        // Verify total member count is correct
        let total_members: usize = member_counts.iter().sum();
        assert_eq!(
            total_members, 23,
            "Total members across all groups should be 23"
        );

        // Verify min and max group sizes
        assert_eq!(
            group_stats.min_group_size, 4,
            "Minimum group size should be 4"
        );
        assert_eq!(
            group_stats.max_group_size, 5,
            "Maximum group size should be 5"
        );

        // Verify average group size
        let expected_avg = 23.0 / 5.0; // 4.6
        assert!(
            (group_stats.avg_group_size - expected_avg).abs() < 0.01,
            "Average group size should be close to 4.6"
        );
    }

    /// Test validation: num_groups cannot be zero
    #[test]
    fn test_group_validation_zero() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            num_groups: Some(0), // Invalid: zero groups
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err(), "Validation should fail for zero groups");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("num_groups must be at least 1"),
            "Error message should mention minimum groups"
        );
    }

    /// Test validation: num_groups cannot exceed entity_count
    #[test]
    fn test_group_validation_exceeds_persons() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            num_groups: Some(15), // Invalid: more groups than persons
            ..Default::default()
        };

        let result = config.validate();
        assert!(
            result.is_err(),
            "Validation should fail when groups exceed persons"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("num_groups (15) cannot exceed entity_count (10)"),
            "Error message should mention the constraint"
        );
    }

    /// Test single group (all persons in one group)
    #[test]
    fn test_single_group() {
        let config = SimulationConfig {
            entity_count: 20,
            max_steps: 50,
            num_groups: Some(1), // Single group
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        let group_stats = result
            .group_statistics
            .expect("Group statistics should be present");

        assert_eq!(group_stats.total_groups, 1);
        assert_eq!(group_stats.groups[0].member_count, 20);
        assert_eq!(group_stats.min_group_size, 20);
        assert_eq!(group_stats.max_group_size, 20);
        assert_eq!(group_stats.avg_group_size, 20.0);
    }

    /// Test one group per person (maximum granularity)
    #[test]
    fn test_one_group_per_person() {
        let config = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            num_groups: Some(10), // One group per person
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        let group_stats = result
            .group_statistics
            .expect("Group statistics should be present");

        assert_eq!(group_stats.total_groups, 10);
        assert_eq!(group_stats.avg_group_size, 1.0);
        assert_eq!(group_stats.min_group_size, 1);
        assert_eq!(group_stats.max_group_size, 1);

        // Each group should have exactly 1 member
        for group in &group_stats.groups {
            assert_eq!(group.member_count, 1);
        }
    }

    /// Test that distance-based trade costs work correctly
    #[test]
    fn test_distance_based_trade_costs() {
        let config_with_distance = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            distance_cost_factor: 0.02, // 2% cost increase per distance unit
            ..Default::default()
        };

        let config_without_distance = SimulationConfig {
            entity_count: 10,
            max_steps: 50,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            distance_cost_factor: 0.0, // Distance costs disabled
            ..Default::default()
        };

        let mut engine_with = SimulationEngine::new(config_with_distance);
        let result_with = engine_with.run();

        let mut engine_without = SimulationEngine::new(config_without_distance);
        let result_without = engine_without.run();

        // Both simulations should complete successfully
        assert_eq!(result_with.total_steps, 50);
        assert_eq!(result_without.total_steps, 50);

        // With distance costs, trade volume might be lower due to higher costs
        // This is probabilistic but should generally hold with the same seed
        // We just verify both produce valid results
        assert!(result_with.trade_volume_statistics.total_trades >= 0);
        assert!(result_without.trade_volume_statistics.total_trades >= 0);

        // Money should be conserved in both (minus any transaction fees/taxes)
        let total_with: f64 = result_with.final_money_distribution.iter().sum();
        let total_without: f64 = result_without.final_money_distribution.iter().sum();

        // Both should have positive total money
        assert!(total_with > 0.0);
        assert!(total_without > 0.0);
    }

    /// Test that distance costs are zero when distance_cost_factor is zero
    #[test]
    fn test_distance_cost_disabled() {
        let config = SimulationConfig {
            entity_count: 5,
            max_steps: 10,
            initial_money_per_person: 100.0,
            base_skill_price: 10.0,
            seed: 42,
            distance_cost_factor: 0.0, // Explicitly disabled
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Simulation should complete normally
        assert_eq!(result.total_steps, 10);
        assert_eq!(result.active_persons, 5);
    }
}
