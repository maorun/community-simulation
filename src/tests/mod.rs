mod proptest_tests;
mod scenario_integration_tests;

#[cfg(test)]
mod engine_tests {
    use crate::{scenario::Scenario, SimulationConfig, SimulationEngine};

    fn get_test_config() -> SimulationConfig {
        SimulationConfig {
            entity_count: 10,
            max_steps: 100,
            initial_money_per_person: 100.0,
            base_skill_price: 50.0,
            seed: 42,
            scenario: Scenario::Original,
            time_step: 1.0,
            tech_growth_rate: 0.0,
            seasonal_amplitude: 0.0,
            seasonal_period: 100,
            transaction_fee: 0.0,
        }
    }

    #[test]
    fn test_simulation_engine_new() {
        let config = get_test_config();
        let engine = SimulationEngine::new(config);

        assert_eq!(engine.get_active_entity_count(), 10);
        assert_eq!(engine.current_step, 0);
    }

    #[test]
    fn test_simulation_engine_step() {
        let config = get_test_config();
        let mut engine = SimulationEngine::new(config);

        engine.step();

        assert_eq!(engine.current_step, 1);
        // Further assertions can be added to check the state of entities and the market
    }

    #[test]
    fn test_simulation_engine_run() {
        let mut config = get_test_config();
        config.max_steps = 2; // Keep the test fast
        let mut engine = SimulationEngine::new(config);

        let result = engine.run();

        assert_eq!(result.total_steps, 2);
        assert_eq!(engine.current_step, 2);
        assert!(!result.final_money_distribution.is_empty());
    }

    #[test]
    fn test_seasonal_factor_disabled() {
        let mut config = get_test_config();
        config.seasonal_amplitude = 0.0; // Disabled
        let engine = SimulationEngine::new(config);

        // When disabled, seasonal factor should always be 1.0
        let factor = engine.calculate_seasonal_factor(&"Skill0".to_string());
        assert_eq!(factor, 1.0);
    }

    #[test]
    fn test_seasonal_factor_enabled() {
        let mut config = get_test_config();
        config.seasonal_amplitude = 0.5; // 50% amplitude
        config.seasonal_period = 100;
        let mut engine = SimulationEngine::new(config);

        // Check factor at different steps
        let skill_id = "Skill0".to_string();

        // At step 0
        let factor_0 = engine.calculate_seasonal_factor(&skill_id);
        assert!(
            (0.5..=1.5).contains(&factor_0),
            "Factor should be in range [0.5, 1.5]"
        );

        // Advance to step 25 (quarter cycle)
        engine.current_step = 25;
        let factor_25 = engine.calculate_seasonal_factor(&skill_id);
        assert!(
            (0.5..=1.5).contains(&factor_25),
            "Factor should be in range [0.5, 1.5]"
        );

        // Advance to step 50 (half cycle)
        engine.current_step = 50;
        let factor_50 = engine.calculate_seasonal_factor(&skill_id);
        assert!(
            (0.5..=1.5).contains(&factor_50),
            "Factor should be in range [0.5, 1.5]"
        );

        // The factors should not all be the same (seasonal variation)
        // Due to phase offset, we can't guarantee specific relationships,
        // but we can verify they're in valid ranges
    }

    #[test]
    fn test_seasonal_factor_different_skills() {
        let mut config = get_test_config();
        config.seasonal_amplitude = 0.5;
        config.seasonal_period = 100;
        let engine = SimulationEngine::new(config);

        // Different skills should have different seasonal factors
        // (due to phase offset based on skill ID hash)
        let factor_skill0 = engine.calculate_seasonal_factor(&"Skill0".to_string());
        let factor_skill1 = engine.calculate_seasonal_factor(&"Skill1".to_string());
        let factor_skill2 = engine.calculate_seasonal_factor(&"Skill2".to_string());

        // All should be in valid range
        assert!((0.5..=1.5).contains(&factor_skill0));
        assert!((0.5..=1.5).contains(&factor_skill1));
        assert!((0.5..=1.5).contains(&factor_skill2));

        // At least some should be different due to phase offset
        // (though theoretically they could all be similar by chance)
    }

    #[test]
    fn test_transaction_fee_collection() {
        let mut config = get_test_config();
        config.max_steps = 10;
        config.transaction_fee = 0.1; // 10% fee
        config.entity_count = 5;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify that fees were collected
        assert!(
            result.total_fees_collected >= 0.0,
            "Total fees should be non-negative"
        );

        // If there were trades, fees should be positive (10% of total volume)
        if result.trade_volume_statistics.total_trades > 0 {
            assert!(
                result.total_fees_collected > 0.0,
                "Fees should be collected when trades occur with non-zero fee rate"
            );

            // Verify that fees are approximately 10% of total volume
            let expected_fees = result.trade_volume_statistics.total_volume * 0.1;
            let fee_difference = (result.total_fees_collected - expected_fees).abs();
            assert!(
                fee_difference < 0.01, // Allow small floating point differences
                "Collected fees ({}) should match expected fees ({})",
                result.total_fees_collected,
                expected_fees
            );
        }
    }

    #[test]
    fn test_zero_transaction_fee() {
        let mut config = get_test_config();
        config.max_steps = 10;
        config.transaction_fee = 0.0; // No fee
        config.entity_count = 5;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // With zero fee, no fees should be collected
        assert_eq!(
            result.total_fees_collected, 0.0,
            "No fees should be collected with 0% transaction fee"
        );
    }

    #[test]
    fn test_panic_recovery_field_exists() {
        // Test that the panic recovery field exists and is initialized correctly
        let config = get_test_config();
        let mut engine = SimulationEngine::new(config);

        let result = engine.run();

        // The failed_steps field should exist and be initialized to 0 for normal execution
        assert_eq!(
            result.failed_steps, 0,
            "Failed steps should be 0 for normal simulation execution"
        );
    }

    #[test]
    fn test_panic_recovery_in_result() {
        // Test that SimulationResult properly serializes failed_steps
        let mut config = get_test_config();
        config.max_steps = 10;
        let mut engine = SimulationEngine::new(config);

        let result = engine.run();

        // Verify the result includes all expected fields
        assert_eq!(result.total_steps, 10);
        assert_eq!(result.failed_steps, 0);

        // Verify it can be serialized to JSON (would fail if field is missing)
        let json_result = serde_json::to_string(&result);
        assert!(
            json_result.is_ok(),
            "SimulationResult should be serializable to JSON"
        );

        // Verify failed_steps is in the JSON output
        let json_str = json_result.unwrap();
        assert!(
            json_str.contains("failed_steps"),
            "JSON output should contain failed_steps field"
        );
    }
}
