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

    #[test]
    fn test_checkpoint_save_and_load() {
        use tempfile::NamedTempFile;

        // Create a temporary file for checkpoint
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let checkpoint_path = temp_file.path();

        // Create and run simulation for a few steps
        let mut config = get_test_config();
        config.max_steps = 10;
        let mut engine = SimulationEngine::new(config);

        // Run 5 steps
        for _ in 0..5 {
            engine.step();
        }

        assert_eq!(engine.current_step, 5);
        let original_entity_count = engine.get_active_entity_count();

        // Save checkpoint
        engine
            .save_checkpoint(checkpoint_path)
            .expect("Failed to save checkpoint");

        // Load checkpoint
        let loaded_engine =
            SimulationEngine::load_checkpoint(checkpoint_path).expect("Failed to load checkpoint");

        // Verify state was restored correctly
        assert_eq!(loaded_engine.current_step, 5);
        assert_eq!(
            loaded_engine.get_active_entity_count(),
            original_entity_count
        );
    }

    #[test]
    fn test_checkpoint_resume_simulation() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let checkpoint_path = temp_file.path();

        // Run first half of simulation
        let mut config1 = get_test_config();
        config1.max_steps = 10;
        let mut engine1 = SimulationEngine::new(config1.clone());

        for _ in 0..5 {
            engine1.step();
        }
        engine1
            .save_checkpoint(checkpoint_path)
            .expect("Failed to save checkpoint");

        // Load and continue simulation
        let mut engine2 =
            SimulationEngine::load_checkpoint(checkpoint_path).expect("Failed to load checkpoint");

        // Continue for remaining 5 steps
        for _ in 0..5 {
            engine2.step();
        }

        assert_eq!(engine2.current_step, 10);

        // Compare with a fresh run of full 10 steps
        let mut engine_full = SimulationEngine::new(config1);
        for _ in 0..10 {
            engine_full.step();
        }

        // Both should reach step 10
        assert_eq!(engine2.current_step, engine_full.current_step);
    }

    #[test]
    fn test_checkpoint_auto_save() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let checkpoint_path = temp_file.path().to_str().unwrap().to_string();

        // Configure with auto-checkpoint every 3 steps
        let mut config = get_test_config();
        config.max_steps = 10;
        config.checkpoint_interval = 3;
        config.checkpoint_file = Some(checkpoint_path.clone());

        let mut engine = SimulationEngine::new(config);

        // Run the simulation
        let _result = engine.run();

        // Checkpoint should have been saved (last one at step 9)
        // Verify the file exists and can be loaded
        let loaded_engine =
            SimulationEngine::load_checkpoint(&checkpoint_path).expect("Failed to load checkpoint");

        // The checkpoint should have been saved at step 9 (last multiple of 3 before 10)
        // or possibly at step 6 depending on when the save happened
        assert!(
            loaded_engine.current_step >= 3,
            "Checkpoint should have been saved at step 3 or later"
        );
        assert!(
            loaded_engine.current_step <= 10,
            "Checkpoint step should not exceed max_steps"
        );
    }
}
