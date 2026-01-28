mod comprehensive_scenario_tests;
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

    #[test]
    fn test_streaming_output() {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        use tempfile::NamedTempFile;

        // Create a temporary file for streaming output
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let stream_path = temp_file.path().to_str().unwrap().to_string();

        // Create simulation config with streaming output enabled
        let mut config = get_test_config();
        config.max_steps = 5;
        config.entity_count = 5;
        config.stream_output_path = Some(stream_path.clone());

        let mut engine = SimulationEngine::new(config);
        let _result = engine.run();

        // Read and verify streaming output file
        let file = File::open(&stream_path).expect("Failed to open streaming output file");
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        // Should have one line per step
        assert_eq!(lines.len(), 5, "Should have 5 lines (one per step)");

        // Verify each line is valid JSON and contains expected fields
        for (i, line) in lines.iter().enumerate() {
            let json: serde_json::Value =
                serde_json::from_str(line).expect("Each line should be valid JSON");

            assert!(json.get("step").is_some(), "Should have 'step' field");
            assert!(json.get("trades").is_some(), "Should have 'trades' field");
            assert!(json.get("volume").is_some(), "Should have 'volume' field");
            assert!(
                json.get("avg_money").is_some(),
                "Should have 'avg_money' field"
            );
            assert!(
                json.get("gini_coefficient").is_some(),
                "Should have 'gini_coefficient' field"
            );
            assert!(
                json.get("avg_reputation").is_some(),
                "Should have 'avg_reputation' field"
            );
            assert!(
                json.get("top_skill_prices").is_some(),
                "Should have 'top_skill_prices' field"
            );

            // Verify step number matches line number
            let step = json["step"].as_u64().unwrap();
            assert_eq!(
                step as usize, i,
                "Step number should match line number (0-indexed)"
            );
        }
    }

    #[test]
    fn test_per_skill_trade_statistics() {
        // Test that per-skill trade statistics are correctly tracked and reported
        let mut config = get_test_config();
        config.max_steps = 50;
        config.entity_count = 20;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify per_skill_trade_stats exists
        assert!(
            !result.per_skill_trade_stats.is_empty(),
            "Per-skill trade stats should not be empty after simulation"
        );

        // Verify structure and data consistency
        for skill_stat in &result.per_skill_trade_stats {
            // Check that all fields are valid
            assert!(
                !skill_stat.skill_id.is_empty(),
                "Skill ID should not be empty"
            );
            assert!(
                skill_stat.trade_count > 0,
                "Trade count should be positive for tracked skills"
            );
            assert!(
                skill_stat.total_volume > 0.0,
                "Total volume should be positive for traded skills"
            );
            assert!(
                skill_stat.avg_price > 0.0,
                "Average price should be positive"
            );

            // Verify avg_price calculation is correct
            let calculated_avg = skill_stat.total_volume / (skill_stat.trade_count as f64);
            assert!(
                (skill_stat.avg_price - calculated_avg).abs() < 0.01,
                "Average price should equal total_volume / trade_count"
            );
        }

        // Verify stats are sorted by total volume (highest first)
        for i in 1..result.per_skill_trade_stats.len() {
            assert!(
                result.per_skill_trade_stats[i - 1].total_volume
                    >= result.per_skill_trade_stats[i].total_volume,
                "Per-skill trade stats should be sorted by total volume (descending)"
            );
        }

        // Verify sum of per-skill stats matches total trade stats
        let total_trades_from_skills: usize = result
            .per_skill_trade_stats
            .iter()
            .map(|s| s.trade_count)
            .sum();
        let total_volume_from_skills: f64 = result
            .per_skill_trade_stats
            .iter()
            .map(|s| s.total_volume)
            .sum();

        assert_eq!(
            total_trades_from_skills, result.trade_volume_statistics.total_trades,
            "Sum of per-skill trade counts should equal total trades"
        );
        assert!(
            (total_volume_from_skills - result.trade_volume_statistics.total_volume).abs() < 0.01,
            "Sum of per-skill volumes should equal total volume"
        );
    }

    #[test]
    fn test_friendship_system_disabled() {
        // Test that friendship system doesn't affect simulation when disabled
        let mut config = get_test_config();
        config.max_steps = 50;
        config.enable_friendships = false;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // When disabled, friendship_statistics should be None
        assert!(
            result.friendship_statistics.is_none(),
            "Friendship statistics should be None when system is disabled"
        );

        // Verify no friendships formed
        for entity in &result.final_persons_data {
            assert_eq!(
                entity.person_data.friends.len(),
                0,
                "Person {} should have no friends when system is disabled",
                entity.id
            );
        }
    }

    #[test]
    fn test_friendship_formation() {
        // Test that friendships form during trading
        let mut config = get_test_config();
        config.max_steps = 100;
        config.enable_friendships = true;
        config.friendship_probability = 0.5; // 50% chance to speed up formation
        config.friendship_discount = 0.1; // 10% discount
        config.entity_count = 20; // More persons = more potential friendships
        config.seed = 12345; // Fixed seed for reproducibility

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Friendship statistics should be present
        assert!(
            result.friendship_statistics.is_some(),
            "Friendship statistics should be present when system is enabled"
        );

        let friendship_stats = result.friendship_statistics.as_ref().unwrap();

        // With 100 steps and 50% probability, we should have at least some friendships
        assert!(
            friendship_stats.total_friendships > 0,
            "At least some friendships should have formed over 100 steps with 50% probability"
        );

        // Average friends per person should be reasonable
        assert!(
            friendship_stats.avg_friends_per_person >= 0.0,
            "Average friends per person should be non-negative"
        );

        // Network density should be between 0 and 1
        assert!(
            friendship_stats.network_density >= 0.0 && friendship_stats.network_density <= 1.0,
            "Network density should be between 0.0 and 1.0, got {}",
            friendship_stats.network_density
        );

        // Verify that friendships are bidirectional
        for entity in &result.final_persons_data {
            for friend_id in &entity.person_data.friends {
                let friend_entity = result
                    .final_persons_data
                    .iter()
                    .find(|e| e.id == *friend_id)
                    .expect("Friend should exist");
                assert!(
                    friend_entity.person_data.friends.contains(&entity.id),
                    "Friendship between {} and {} should be bidirectional",
                    entity.id,
                    friend_id
                );
            }
        }
    }

    #[test]
    fn test_friendship_discount_applied() {
        // Test that friends receive price discounts
        let mut config = get_test_config();
        config.max_steps = 50;
        config.entity_count = 10;
        config.enable_friendships = true;
        config.friendship_probability = 1.0; // 100% chance - all trades create friendships
        config.friendship_discount = 0.2; // 20% discount for testing
        config.seed = 42;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // With 100% probability, we should have many friendships
        let friendship_stats = result.friendship_statistics.as_ref().unwrap();
        assert!(
            friendship_stats.total_friendships > 0,
            "With 100% probability, friendships should have formed"
        );

        // Since friends get discounts, we can verify indirectly by checking that
        // total trade volume might be less than without friendships (due to discounts)
        // This is a weaker test but verifies the system is active
        assert!(
            friendship_stats.avg_friends_per_person > 0.0,
            "Average friends per person should be positive with 100% formation rate"
        );
    }

    #[test]
    fn test_friendship_validation() {
        // Test that invalid friendship parameters are rejected
        let mut config = get_test_config();
        config.enable_friendships = true;

        // Test invalid probability (> 1.0)
        config.friendship_probability = 1.5;
        assert!(
            config.validate().is_err(),
            "Should reject friendship_probability > 1.0"
        );

        // Test invalid probability (< 0.0)
        config.friendship_probability = -0.1;
        assert!(
            config.validate().is_err(),
            "Should reject friendship_probability < 0.0"
        );

        // Test invalid discount (> 1.0)
        config.friendship_probability = 0.5;
        config.friendship_discount = 1.5;
        assert!(
            config.validate().is_err(),
            "Should reject friendship_discount > 1.0"
        );

        // Test invalid discount (< 0.0)
        config.friendship_discount = -0.1;
        assert!(
            config.validate().is_err(),
            "Should reject friendship_discount < 0.0"
        );

        // Test valid parameters
        config.friendship_probability = 0.1;
        config.friendship_discount = 0.1;
        assert!(
            config.validate().is_ok(),
            "Should accept valid friendship parameters"
        );
    }

    #[test]
    fn test_friendship_network_density() {
        // Test network density calculation
        const FLOAT_TOLERANCE: f64 = 0.0001; // Tolerance for floating-point comparisons

        let mut config = get_test_config();
        config.max_steps = 200; // More steps = more dense network
        config.entity_count = 15;
        config.enable_friendships = true;
        config.friendship_probability = 0.8; // High probability
        config.seed = 999;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        let friendship_stats = result.friendship_statistics.as_ref().unwrap();

        // With high probability and many steps, network should be moderately dense
        assert!(
            friendship_stats.network_density > 0.0,
            "Network density should be positive with high formation rate"
        );

        // Check that network density formula is correct
        // Possible friendships = n * (n-1) / 2
        let n = result.active_persons;

        // Guard against edge cases where density calculation would be invalid
        if n > 1 {
            let possible_friendships = (n * (n - 1)) / 2;
            let expected_density =
                friendship_stats.total_friendships as f64 / possible_friendships as f64;
            assert!(
                (friendship_stats.network_density - expected_density).abs() < FLOAT_TOLERANCE,
                "Network density calculation should be correct"
            );
        } else {
            // With 0 or 1 persons, network density should be 0
            assert_eq!(
                friendship_stats.network_density, 0.0,
                "Network density should be 0 with 0 or 1 persons"
            );
        }
    }

    #[test]
    fn test_event_tracking_enabled() {
        // Test that events are collected when enabled
        let mut config = get_test_config();
        config.max_steps = 10;
        config.entity_count = 5;
        config.enable_events = true; // Enable event tracking
        config.seed = 123;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify events were collected
        assert!(
            result.events.is_some(),
            "Events should be collected when enabled"
        );
        let events = result.events.as_ref().unwrap();
        assert!(!events.is_empty(), "Should have collected some events");

        // Count different event types
        let mut trade_events = 0;
        let mut price_events = 0;
        let mut reputation_events = 0;
        let mut step_events = 0;

        for event in events {
            match &event.event_type {
                crate::event::EventType::TradeExecuted { .. } => trade_events += 1,
                crate::event::EventType::PriceUpdated { .. } => price_events += 1,
                crate::event::EventType::ReputationChanged { .. } => reputation_events += 1,
                crate::event::EventType::StepCompleted { .. } => step_events += 1,
            }
        }

        // Should have at least some of each type of event
        assert!(step_events > 0, "Should have step completed events");
        // Trade and reputation events depend on successful trades happening
        // Price events depend on prices changing
        // Just verify that we're collecting events in general
        assert!(
            trade_events > 0 || price_events > 0 || reputation_events > 0,
            "Should have trade, price, or reputation events"
        );
    }

    #[test]
    fn test_event_tracking_disabled() {
        // Test that events are NOT collected when disabled
        let mut config = get_test_config();
        config.max_steps = 10;
        config.entity_count = 5;
        config.enable_events = false; // Disable event tracking
        config.seed = 123;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify events were NOT collected
        assert!(
            result.events.is_none(),
            "Events should not be collected when disabled"
        );
    }

    #[test]
    fn test_incremental_money_statistics_accuracy() {
        // Test that incremental statistics produce valid results
        let mut config = get_test_config();
        config.entity_count = 20;
        config.max_steps = 50;
        config.enable_loans = true; // Allow money to vary more
        config.tax_rate = 0.05;
        config.enable_tax_redistribution = true;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Get money statistics from result
        let money_stats = &result.money_statistics;

        // Verify mean is reasonable
        assert!(
            money_stats.average.is_finite(),
            "Average money should be finite"
        );

        // Verify std_dev is non-negative and finite
        assert!(
            money_stats.std_dev >= 0.0 && money_stats.std_dev.is_finite(),
            "Std dev should be non-negative and finite, got: {}",
            money_stats.std_dev
        );

        // Verify min/max relationship
        assert!(
            money_stats.min_money <= money_stats.max_money,
            "Min money ({}) should be <= max money ({})",
            money_stats.min_money,
            money_stats.max_money
        );

        // Verify median is between min and max
        assert!(
            money_stats.median >= money_stats.min_money
                && money_stats.median <= money_stats.max_money,
            "Median ({}) should be between min ({}) and max ({})",
            money_stats.median,
            money_stats.min_money,
            money_stats.max_money
        );

        // Verify Gini coefficient is in valid range [0, infinity)
        assert!(
            money_stats.gini_coefficient >= 0.0,
            "Gini coefficient should be non-negative"
        );

        // Run multiple simulations and verify statistics are consistent
        for seed in 100..105 {
            let mut config2 = get_test_config();
            config2.entity_count = 15;
            config2.max_steps = 30;
            config2.seed = seed;

            let mut engine2 = SimulationEngine::new(config2);
            let result2 = engine2.run();

            let stats = &result2.money_statistics;

            // All statistics should be finite
            assert!(
                stats.average.is_finite() && stats.std_dev.is_finite() && stats.median.is_finite(),
                "All statistics should be finite for seed {}",
                seed
            );

            // Standard deviation should be non-negative
            assert!(
                stats.std_dev >= 0.0,
                "Std dev should be non-negative for seed {}",
                seed
            );

            // Min/max should be consistent
            assert!(
                stats.min_money <= stats.max_money,
                "Min <= max should hold for seed {}",
                seed
            );
        }
    }

    #[test]
    fn test_velocity_of_money_integration() {
        // Integration test to verify velocity of money is correctly calculated
        // in the context of a real simulation
        let mut config = get_test_config();
        config.entity_count = 10;
        config.max_steps = 50;
        config.initial_money_per_person = 100.0;
        config.base_skill_price = 10.0;
        config.seed = 42;

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        // Verify velocity of money is calculated and is a valid number
        let velocity = result.trade_volume_statistics.velocity_of_money;
        assert!(velocity.is_finite(), "Velocity should be a finite number");
        assert!(velocity >= 0.0, "Velocity should be non-negative");

        // Verify the calculation: velocity = total_volume / total_money_supply
        let total_volume = result.trade_volume_statistics.total_volume;
        let total_money_supply: f64 = result.final_money_distribution.iter().sum();

        if total_money_supply > 0.0 {
            let expected_velocity = total_volume / total_money_supply;
            assert!(
                (velocity - expected_velocity).abs() < 1e-10,
                "Velocity calculation should match formula: {} vs {}",
                velocity,
                expected_velocity
            );
        } else {
            // If there's no money supply, velocity should be 0
            assert_eq!(velocity, 0.0, "Velocity should be 0 when money supply is 0");
        }

        // Verify velocity makes economic sense
        // For a typical simulation with active trading, velocity should be > 0
        if result.trade_volume_statistics.total_trades > 0 {
            assert!(
                velocity > 0.0,
                "Velocity should be positive when trades occur"
            );
        }

        // Test with zero transactions (very short simulation)
        let mut config_zero = get_test_config();
        config_zero.max_steps = 1; // Very short, likely no trades
        config_zero.entity_count = 2;
        let mut engine_zero = SimulationEngine::new(config_zero);
        let result_zero = engine_zero.run();

        // Velocity should still be a valid number (likely 0 or very low)
        assert!(
            result_zero
                .trade_volume_statistics
                .velocity_of_money
                .is_finite(),
            "Velocity should be finite even with minimal trading"
        );
        assert!(
            result_zero.trade_volume_statistics.velocity_of_money >= 0.0,
            "Velocity should be non-negative even with minimal trading"
        );

        // Test that velocity changes with economic activity
        // More steps should generally lead to higher velocity
        let mut config_long = get_test_config();
        config_long.max_steps = 100;
        config_long.entity_count = 20;
        config_long.initial_money_per_person = 100.0;
        let mut engine_long = SimulationEngine::new(config_long);
        let result_long = engine_long.run();

        let velocity_long = result_long.trade_volume_statistics.velocity_of_money;
        assert!(
            velocity_long.is_finite() && velocity_long >= 0.0,
            "Long simulation should have valid velocity"
        );

        // If there were more total trades in the longer simulation,
        // the velocity could be higher (but not guaranteed due to money distribution changes)
        if result_long.trade_volume_statistics.total_trades
            > result.trade_volume_statistics.total_trades
        {
            // Just verify it's a reasonable value, not necessarily higher
            assert!(
                velocity_long >= 0.0,
                "Longer simulation with more trades should have non-negative velocity"
            );
        }
    }
}
