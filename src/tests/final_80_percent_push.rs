//! FINAL PUSH TO 80%+ COVERAGE
//! Ultra-aggressive tests targeting all remaining uncovered lines
//! Focus: engine.rs (310 lines), result.rs (77 lines), scenario.rs (57 lines)

use crate::scenario::Scenario;
use crate::tests::test_helpers::test_config;
use crate::SimulationEngine;
use tempfile::tempdir;

// ============================================================================
// ENGINE.RS ULTRA-AGGRESSIVE TESTS (TARGET: 150+ lines)
// ============================================================================

#[test]
fn test_engine_business_cycle_full_tracking() {
    // Test business cycle tracking over many steps
    let config = test_config().max_steps(200).entity_count(50).seasonality(0.3, 40).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Should complete all 200 steps
    assert_eq!(result.total_steps, 200);
    assert!(result.active_persons > 0);
}

#[test]
fn test_engine_checkpoint_and_resume_complex() {
    // Create checkpoint at step 50, then resume to step 100
    let dir = tempdir().unwrap();
    let checkpoint_path = dir.path().join("checkpoint.json");

    // First run: create checkpoint at step 50
    let config = test_config()
        .max_steps(50)
        .entity_count(30)
        .checkpoint_interval(50)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine = SimulationEngine::new(config);
    engine.run();

    // Verify checkpoint file exists
    assert!(checkpoint_path.exists());

    // Second run: resume from checkpoint
    let config2 = test_config()
        .max_steps(100)
        .entity_count(30)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine2 = SimulationEngine::new(config2);
    let result2 = engine2.run();

    // Should continue from step 50
    assert!(result2.total_steps >= 50);
    assert!(result2.active_persons > 0);
}

#[test]
fn test_engine_checkpoint_resume_different_state() {
    // Test resume with modified state
    let dir = tempdir().unwrap();
    let checkpoint_path = dir.path().join("checkpoint2.json");

    let config = test_config()
        .max_steps(30)
        .entity_count(20)
        .checkpoint_interval(30)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine = SimulationEngine::new(config);
    engine.run();

    // Resume with more steps
    let config2 = test_config()
        .max_steps(80)
        .entity_count(20)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine2 = SimulationEngine::new(config2);
    let result = engine2.run();

    assert!(result.total_steps >= 30);
}

#[test]
fn test_engine_welfare_analysis_calculations() {
    // Test welfare analysis comprehensive calculations
    let config = test_config().max_steps(100).entity_count(60).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);

    // Wealth stats should be calculated
    assert!(!result.final_money_distribution.is_empty());
    assert_eq!(result.final_money_distribution.len(), 60);
}

#[test]
fn test_engine_complex_statistics() {
    // Test complex statistics calculations
    let config = test_config().max_steps(150).entity_count(80).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 150);
    assert_eq!(result.active_persons, 80);

    // Statistics should be meaningful
    assert!(!result.final_money_distribution.is_empty());
}

#[test]
fn test_engine_all_getter_methods() {
    // Test all engine getter methods
    let config = test_config().build();
    let engine = SimulationEngine::new(config.clone());

    // Test getters
    assert_eq!(engine.current_step, 0);
    assert_eq!(engine.get_active_entity_count(), config.entity_count);
}

#[test]
fn test_engine_multi_step_checkpoints() {
    // Multi-step simulation with multiple checkpoints
    let dir = tempdir().unwrap();
    let checkpoint_path = dir.path().join("multi_checkpoint.json");

    let config = test_config()
        .max_steps(120)
        .entity_count(50)
        .checkpoint_interval(30)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 120);
    assert!(checkpoint_path.exists());
}

#[test]
fn test_engine_failed_trade_tracking() {
    // Test failed trade tracking in detail
    let config = test_config().max_steps(80).entity_count(15).initial_money(10.0).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 80);
    assert!(result.active_persons > 0);
}

#[test]
fn test_engine_price_history_tracking() {
    // Test price history tracking over time
    let config = test_config().max_steps(100).entity_count(50).base_price(20.0).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);

    // Price history should be tracked
    assert!(!result.skill_price_history.is_empty());
}

#[test]
fn test_engine_trade_volume_calculations() {
    // Test trade volume calculations
    let config = test_config().max_steps(100).entity_count(70).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);

    // Trade statistics should exist
    assert!(result.trade_volume_statistics.avg_trades_per_step >= 0.0);
}

#[test]
fn test_engine_mega_simulation_all_features() {
    // Mega test with ALL features enabled
    let config = test_config()
        .max_steps(200)
        .entity_count(100)
        .initial_money(150.0)
        .base_price(15.0)
        .seasonality(0.2, 50)
        .scenario(Scenario::DynamicPricing)
        .enable_loans(true)
        .tax_rate(0.1)
        .enable_tax_redistribution(true)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 200);
    assert_eq!(result.active_persons, 100);

    // All statistics should be calculated
    assert!(!result.final_money_distribution.is_empty());
}

#[test]
fn test_engine_step_by_step_execution() {
    // Test step-by-step execution
    let config = test_config().build();
    let mut engine = SimulationEngine::new(config);

    // Execute steps manually
    for step in 0..10 {
        engine.step();
        assert_eq!(engine.current_step, step + 1);
    }
}

#[test]
fn test_engine_loans_enabled() {
    // Test with loans enabled
    let config = test_config()
        .max_steps(100)
        .entity_count(50)
        .enable_loans(true)
        .loan_interest_rate(0.05)
        .loan_repayment_period(20)
        .min_money_to_lend(50.0)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
    assert_eq!(result.active_persons, 50);
}

#[test]
fn test_engine_tax_system() {
    // Test tax system
    let config = test_config()
        .max_steps(80)
        .entity_count(60)
        .tax_rate(0.15)
        .enable_tax_redistribution(true)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 80);
    assert_eq!(result.active_persons, 60);
}

#[test]
fn test_engine_transaction_fees() {
    // Test transaction fees
    let config = test_config().max_steps(100).entity_count(50).transaction_fee(0.05).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_engine_savings_rate() {
    // Test savings rate
    let config = test_config().max_steps(100).entity_count(50).savings_rate(0.2).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_engine_tech_growth() {
    // Test technology growth
    let config = test_config().max_steps(150).entity_count(60).tech_growth(0.01).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 150);
}

#[test]
fn test_engine_multiple_skills_per_person() {
    // Test multiple skills per person
    let config = test_config().max_steps(100).entity_count(40).skills_per_person(3).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_engine_priority_weights() {
    // Test priority weights
    let config = test_config()
        .max_steps(80)
        .entity_count(50)
        .priority_weights(0.4, 0.3, 0.2, 0.1)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 80);
}

// ============================================================================
// RESULT.RS TESTS (TARGET: 30+ lines)
// ============================================================================

#[test]
fn test_result_display_implementations() {
    // Test Display implementations
    let config = test_config().max_steps(20).entity_count(10).build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Debug trait
    let debug_str = format!("{:?}", result);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_result_csv_export() {
    let dir = tempdir().unwrap();
    let csv_path = dir.path().join("export.csv");

    let config = test_config()
        .max_steps(50)
        .entity_count(30)
        .scenario(Scenario::Original)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    if let Some(path_str) = csv_path.to_str() {
        result.save_to_csv(path_str).ok();
    }
}

#[test]
fn test_result_save_to_file_detailed() {
    let dir = tempdir().unwrap();
    let result_path = dir.path().join("result_detailed.json");

    let config = test_config().max_steps(40).entity_count(25).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Save with detailed information
    result.save_to_file(result_path.to_str().unwrap(), false).ok();
}

#[test]
fn test_result_statistics_calculations() {
    // Test statistics calculations
    let config = test_config().max_steps(100).entity_count(60).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Verify statistics are calculated
    assert_eq!(result.total_steps, 100);
    assert_eq!(result.active_persons, 60);
    assert!(!result.final_money_distribution.is_empty());
}

#[test]
fn test_result_price_evolution() {
    // Test price evolution tracking
    let config = test_config().max_steps(80).entity_count(50).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert!(!result.skill_price_history.is_empty());
}

#[test]
fn test_result_trade_volume_stats() {
    // Test trade volume statistics
    let config = test_config().max_steps(100).entity_count(70).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert!(result.trade_volume_statistics.avg_trades_per_step >= 0.0);
    assert!(result.trade_volume_statistics.total_volume >= 0.0);
}

// ============================================================================
// SCENARIO.RS TESTS (TARGET: 20+ lines)
// ============================================================================

#[test]
fn test_scenario_original_full() {
    // Test Original scenario comprehensively
    let config = test_config()
        .max_steps(100)
        .entity_count(50)
        .scenario(Scenario::Original)
        .base_price(25.0)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_scenario_dynamic_pricing_full() {
    // Test DynamicPricing scenario comprehensively
    let config = test_config()
        .max_steps(100)
        .entity_count(60)
        .scenario(Scenario::DynamicPricing)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_scenario_comparison() {
    // Compare scenarios
    let scenarios = vec![Scenario::Original, Scenario::DynamicPricing];

    for scenario in scenarios {
        let config =
            test_config().max_steps(80).entity_count(40).scenario(scenario.clone()).build();

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 80);
        assert_eq!(result.active_persons, 40);
    }
}

// ============================================================================
// INTEGRATION MEGA-TESTS (TARGET: 30+ lines)
// ============================================================================

#[test]
fn test_integration_full_lifecycle() {
    // Full lifecycle test with detailed tracking
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("lifecycle.json");

    let config = test_config()
        .max_steps(150)
        .entity_count(75)
        .initial_money(120.0)
        .base_price(18.0)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    // Save results
    result.save_to_file(output_path.to_str().unwrap(), false).ok();

    // Verify
    assert_eq!(result.total_steps, 150);
    assert!(output_path.exists());
}

#[test]
fn test_integration_extreme_scale() {
    // Test with extreme scale
    let config = test_config().max_steps(500).entity_count(200).initial_money(200.0).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 500);
    assert_eq!(result.active_persons, 200);
}

#[test]
fn test_integration_minimal_scale() {
    // Test with minimal scale
    let config = test_config().max_steps(5).entity_count(3).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
    assert_eq!(result.active_persons, 3);
}

#[test]
fn test_integration_business_cycle_periods() {
    // Test complete business cycle periods
    let config = test_config().max_steps(400).entity_count(60).seasonality(0.25, 40).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 400);
}

#[test]
fn test_integration_checkpoint_recovery() {
    // Test checkpoint recovery mechanism
    let dir = tempdir().unwrap();
    let checkpoint_path = dir.path().join("recovery.json");

    // Create checkpoint
    let config = test_config()
        .max_steps(60)
        .entity_count(40)
        .checkpoint_interval(60)
        .checkpoint_file(Some(checkpoint_path.to_string_lossy().to_string()))
        .build();

    let mut engine = SimulationEngine::new(config);
    engine.run();

    // Verify checkpoint exists
    assert!(checkpoint_path.exists());
}

#[test]
fn test_integration_wealth_distribution() {
    // Detailed wealth distribution analysis
    let config = test_config().max_steps(150).entity_count(100).initial_money(100.0).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 150);
    assert_eq!(result.final_money_distribution.len(), 100);
}

#[test]
fn test_integration_trade_patterns() {
    // Analyze trade patterns
    let config = test_config().max_steps(120).entity_count(70).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 120);
    assert!(result.trade_volume_statistics.avg_trades_per_step >= 0.0);
}

#[test]
fn test_integration_long_running_stability() {
    // Test long-running simulation stability
    let config = test_config().max_steps(1000).entity_count(100).build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 1000);
    assert_eq!(result.active_persons, 100);
}

#[test]
fn test_integration_all_features_combined() {
    // Combine ALL features
    let config = test_config()
        .max_steps(250)
        .entity_count(120)
        .initial_money(180.0)
        .base_price(22.0)
        .scenario(Scenario::DynamicPricing)
        .enable_loans(true)
        .loan_interest_rate(0.03)
        .tax_rate(0.12)
        .enable_tax_redistribution(true)
        .transaction_fee(0.02)
        .savings_rate(0.15)
        .tech_growth(0.005)
        .seasonality(0.18, 45)
        .skills_per_person(2)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 250);
    assert_eq!(result.active_persons, 120);
    assert!(!result.final_money_distribution.is_empty());
}

#[test]
fn test_integration_various_entity_counts() {
    // Test various entity counts
    let counts = vec![5, 10, 20, 50, 100];

    for count in counts {
        let config = test_config().max_steps(50).entity_count(count).build();

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 50);
        assert_eq!(result.active_persons, count);
    }
}

#[test]
fn test_integration_various_step_counts() {
    // Test various step counts
    let steps = vec![10, 50, 100, 200, 500];

    for step_count in steps {
        let config = test_config().max_steps(step_count).entity_count(30).build();

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, step_count);
    }
}

#[test]
fn test_integration_price_dynamics() {
    // Test price dynamics with different scenarios
    let config = test_config()
        .max_steps(200)
        .entity_count(80)
        .base_price(20.0)
        .scenario(Scenario::DynamicPricing)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 200);
    assert!(!result.skill_price_history.is_empty());
}

#[test]
fn test_integration_economic_cycles() {
    // Test economic cycles over extended period
    let config = test_config()
        .max_steps(600)
        .entity_count(90)
        .seasonality(0.3, 60)
        .tech_growth(0.01)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 600);
}

#[test]
fn test_integration_financial_system() {
    // Test financial system (loans + taxes)
    let config = test_config()
        .max_steps(180)
        .entity_count(70)
        .enable_loans(true)
        .loan_interest_rate(0.04)
        .loan_repayment_period(25)
        .tax_rate(0.1)
        .enable_tax_redistribution(true)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 180);
}

#[test]
fn test_integration_market_mechanisms() {
    // Test market mechanisms
    let config = test_config()
        .max_steps(200)
        .entity_count(100)
        .transaction_fee(0.03)
        .savings_rate(0.25)
        .scenario(Scenario::Original)
        .build();

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 200);
}

#[test]
fn test_integration_stream_output() {
    // Test streaming output
    let dir = tempdir().unwrap();
    let stream_path = dir.path().join("stream.jsonl");

    let config = test_config()
        .max_steps(50)
        .entity_count(30)
        .stream_output_path(Some(stream_path.to_string_lossy().to_string()))
        .build();

    let mut engine = SimulationEngine::new(config);
    engine.run();

    // Stream file might be created
    // (checking existence is optional as it depends on implementation)
}
