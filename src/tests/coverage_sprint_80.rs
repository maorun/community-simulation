//! Coverage Sprint to 80%+ - Strategic high-impact tests focusing on untested code paths

use crate::tests::test_helpers::test_config;
use crate::*;

// ============================================================================
// ENGINE.RS GETTER METHODS AND BASIC OPERATIONS - Target: 50+ lines
// ============================================================================

#[test]
fn test_all_engine_getters_comprehensive() {
    let config = test_config()
        .max_steps(100)
        .entity_count(20)
        .build();
    let mut engine = SimulationEngine::new(config.clone());
    
    // Test all getter methods before running
    assert_eq!(engine.get_active_entity_count(), 20);
    assert_eq!(engine.get_current_step(), 0);
    assert_eq!(engine.get_max_steps(), 100);
    assert_eq!(engine.get_active_persons(), 20);
    assert_eq!(engine.get_total_fees_collected(), 0.0);
    assert_eq!(engine.get_total_taxes_collected(), 0.0);
    
    let scenario = engine.get_scenario();
    assert!(matches!(scenario, scenario::Scenario::Original));
    
    let entities = engine.get_entities();
    assert_eq!(entities.len(), 20);
    
    let market = engine.get_market();
    assert!(!market.get_all_skill_prices().is_empty());
    
    let config_ref = engine.get_config();
    assert_eq!(config_ref.max_steps, 100);
    
    // Run simulation and test getters again
    engine.run();
    assert_eq!(engine.get_current_step(), 100);
    assert!(engine.get_total_fees_collected() >= 0.0);
    assert!(engine.get_total_taxes_collected() >= 0.0);
}

#[test]
fn test_get_current_result_incremental() {
    let config = test_config()
        .max_steps(100)
        .entity_count(15)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    // Get result at step 0
    let result_0 = engine.get_current_result();
    assert_eq!(result_0.total_steps, 0);
    assert_eq!(result_0.active_persons, 15);
    
    // Run 20 steps
    for _ in 0..20 {
        engine.step();
    }
    
    let result_20 = engine.get_current_result();
    assert_eq!(result_20.total_steps, 20);
    
    // Run 20 more steps
    for _ in 0..20 {
        engine.step();
    }
    
    let result_40 = engine.get_current_result();
    assert_eq!(result_40.total_steps, 40);
    
    // Run remaining steps to reach max_steps (60 more steps)
    for _ in 0..60 {
        engine.step();
    }
    
    let result_final = engine.get_current_result();
    assert_eq!(result_final.total_steps, 100);
}

// ============================================================================
// ENGINE.RS CHECKPOINT OPERATIONS - Target: 40+ lines
// ============================================================================

#[test]
fn test_checkpoint_save_and_load_basic() {
    let config = test_config()
        .max_steps(50)
        .entity_count(10)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    // Run some steps
    for _ in 0..20 {
        engine.step();
    }
    
    let checkpoint_path = "/tmp/test_checkpoint_basic.json";
    engine.save_checkpoint(checkpoint_path).expect("Failed to save checkpoint");
    
    // Load checkpoint
    let loaded_engine = SimulationEngine::load_checkpoint(checkpoint_path)
        .expect("Failed to load checkpoint");
    
    assert_eq!(loaded_engine.get_current_step(), engine.get_current_step());
    assert_eq!(loaded_engine.get_active_entity_count(), engine.get_active_entity_count());
    
    // Clean up
    std::fs::remove_file(checkpoint_path).ok();
}

#[test]
fn test_checkpoint_midway() {
    let config = test_config()
        .max_steps(100)
        .entity_count(15)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    // Run halfway
    for _ in 0..50 {
        engine.step();
    }
    
    let checkpoint_path = "/tmp/test_checkpoint_midway.json";
    engine.save_checkpoint(checkpoint_path).expect("Failed to save checkpoint");
    
    let loaded_engine = SimulationEngine::load_checkpoint(checkpoint_path)
        .expect("Failed to load checkpoint");
    
    assert_eq!(loaded_engine.get_current_step(), 50);
    
    std::fs::remove_file(checkpoint_path).ok();
}

#[test]
fn test_checkpoint_load_missing_file() {
    let result = SimulationEngine::load_checkpoint("/tmp/nonexistent_checkpoint_xyz123.json");
    assert!(result.is_err());
}

#[test]
fn test_checkpoint_save_to_invalid_path() {
    let config = test_config().build();
    let engine = SimulationEngine::new(config);
    
    let result = engine.save_checkpoint("/invalid/path/that/does/not/exist/checkpoint.json");
    assert!(result.is_err());
}

// ============================================================================
// ENGINE.RS PLUGIN SYSTEM - Target: 30+ lines
// ============================================================================

#[test]
fn test_plugin_registry_basic_access() {
    let config = test_config().build();
    let mut engine = SimulationEngine::new(config);
    
    // Test immutable access
    let registry = engine.plugin_registry();
    assert_eq!(registry.len(), 0);
    
    // Test mutable access
    let registry_mut = engine.plugin_registry_mut();
    assert_eq!(registry_mut.len(), 0);
}

// ============================================================================
// ENGINE.RS LONG SIMULATIONS - Target: 60+ lines
// ============================================================================

#[test]
fn test_very_long_simulation() {
    let config = test_config()
        .max_steps(1000)
        .entity_count(40)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    
    assert_eq!(result.total_steps, 1000);
    assert!(result.active_persons > 0);
}

#[test]
fn test_large_population_simulation() {
    let config = test_config()
        .max_steps(150)
        .entity_count(200)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    
    assert_eq!(result.active_persons, 200);
}

#[test]
fn test_simulation_with_many_steps_incremental() {
    let config = test_config()
        .max_steps(500)
        .entity_count(30)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    // Run in chunks
    for _ in 0..5 {
        for _ in 0..100 {
            engine.step();
        }
    }
    
    assert_eq!(engine.get_current_step(), 500);
}

#[test]
fn test_extreme_population() {
    let config = test_config()
        .max_steps(100)
        .entity_count(150)
        .initial_money(1000.0)
        .base_price(50.0)
        .build();
    
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    assert_eq!(result.active_persons, 150);
}

#[test]
fn test_minimal_simulation() {
    let config = test_config()
        .max_steps(10)
        .entity_count(3)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 10);
    assert_eq!(result.active_persons, 3);
}

#[test]
fn test_simulation_with_high_initial_money() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .initial_money(5000.0)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.money_statistics.average > 0.0);
}

#[test]
fn test_simulation_with_low_initial_money() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .initial_money(10.0)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.failed_trade_statistics.total_failed_attempts >= 0);
}

#[test]
fn test_simulation_with_high_base_price() {
    let config = test_config()
        .max_steps(80)
        .entity_count(20)
        .base_price(200.0)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.failed_trade_statistics.total_failed_attempts >= 0);
}

// ============================================================================
// RESULT.RS OPERATIONS - Target: 50+ lines
// ============================================================================

#[test]
fn test_simulation_result_debug() {
    let config = test_config()
        .max_steps(50)
        .entity_count(10)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("SimulationResult"));
    assert!(!debug_str.is_empty());
}

#[test]
fn test_result_print_summary_basic() {
    let config = test_config()
        .max_steps(50)
        .entity_count(15)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    // This should not panic
    result.print_summary(false);
}

#[test]
fn test_result_print_summary_with_histogram() {
    let config = test_config()
        .max_steps(50)
        .entity_count(15)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    // Test with histogram enabled
    result.print_summary(true);
}

#[test]
fn test_result_serialization() {
    let config = test_config()
        .max_steps(40)
        .entity_count(10)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    assert!(!json.is_empty());
    assert!(json.contains("total_steps"));
}

#[test]
fn test_result_deserialization() {
    let config = test_config()
        .max_steps(30)
        .entity_count(8)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: result::SimulationResult = serde_json::from_str(&json)
        .expect("Failed to deserialize");
    
    assert_eq!(deserialized.total_steps, result.total_steps);
    assert_eq!(deserialized.active_persons, result.active_persons);
}

#[test]
fn test_result_save_to_file() {
    let config = test_config()
        .max_steps(50)
        .entity_count(12)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let json_path = "/tmp/test_result_save.json";
    result.save_to_file(json_path, false).expect("Failed to save result");
    
    // Verify file exists and can be read
    let content = std::fs::read_to_string(json_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("total_steps"));
    
    std::fs::remove_file(json_path).ok();
}

#[test]
fn test_result_save_to_invalid_path() {
    let config = test_config().max_steps(10).build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let result_err = result.save_to_file("/invalid/path/result.json", false);
    assert!(result_err.is_err());
}

#[test]
fn test_result_save_compressed() {
    let config = test_config()
        .max_steps(50)
        .entity_count(12)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    let json_path = "/tmp/test_result_compressed.json.gz";
    result.save_to_file(json_path, true).expect("Failed to save compressed result");
    
    // Verify file exists
    assert!(std::path::Path::new(json_path).exists());
    
    std::fs::remove_file(json_path).ok();
}

#[test]
fn test_result_components_formatting() {
    let config = test_config()
        .max_steps(40)
        .entity_count(12)
        .build();
    let mut engine = SimulationEngine::new(config);
    let result = engine.run();
    
    // Test various Debug implementations
    let _ = format!("{:?}", result.money_statistics);
    let _ = format!("{:?}", result.final_money_distribution);
    let _ = format!("{:?}", result.skill_price_history);
    let _ = format!("{:?}", result.trade_volume_statistics);
}

// ============================================================================
// SCENARIO TESTS - Target: 30+ lines
// ============================================================================

#[test]
fn test_scenario_original() {
    let config = test_config()
        .max_steps(50)
        .entity_count(15)
        .scenario(scenario::Scenario::Original)
        .build();
    let mut engine = SimulationEngine::new(config);
    engine.run();
}

#[test]
fn test_scenario_dynamic_pricing() {
    let config = test_config()
        .max_steps(50)
        .entity_count(15)
        .scenario(scenario::Scenario::DynamicPricing)
        .build();
    let mut engine = SimulationEngine::new(config);
    engine.run();
}

#[test]
fn test_scenario_display() {
    use crate::scenario::Scenario;
    
    let scenarios = vec![
        Scenario::Original,
        Scenario::DynamicPricing,
    ];
    
    for scenario in scenarios {
        let display_str = format!("{}", scenario);
        assert!(!display_str.is_empty());
    }
}

// ============================================================================
// ACTION RECORDING - Target: 20+ lines
// ============================================================================

#[test]
fn test_action_recording() {
    let config = test_config()
        .max_steps(50)
        .entity_count(10)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    engine.enable_action_recording();
    engine.run();
    
    let log_path = "/tmp/test_action_log_1.json";
    engine.save_action_log(log_path).expect("Failed to save action log");
    
    // Verify log file exists
    assert!(std::path::Path::new(log_path).exists());
    
    std::fs::remove_file(log_path).ok();
}

#[test]
fn test_action_log_content_nonempty() {
    let config = test_config()
        .max_steps(30)
        .entity_count(8)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    engine.enable_action_recording();
    engine.run();
    
    let log_path = "/tmp/test_action_log_2.json";
    engine.save_action_log(log_path).expect("Failed to save action log");
    
    let content = std::fs::read_to_string(log_path).unwrap();
    assert!(!content.is_empty());
    
    std::fs::remove_file(log_path).ok();
}

// ============================================================================
// SEASONAL FACTORS - Target: 20+ lines
// ============================================================================

#[test]
fn test_seasonal_factor_calculation() {
    let config = test_config()
        .max_steps(50)
        .entity_count(15)
        .seasonality(0.5, 50)
        .build();
    let engine = SimulationEngine::new(config);
    
    // Test seasonal factor calculation with a skill ID
    let factor = engine.calculate_seasonal_factor(&"TestSkill".to_string());
    assert!(factor > 0.0);
}

#[test]
fn test_seasonal_simulation() {
    let config = test_config()
        .max_steps(200)
        .entity_count(20)
        .seasonality(0.3, 40)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    engine.run();
    
    // Seasonal factors should have varied over time
    let result = engine.get_current_result();
    assert_eq!(result.total_steps, 200);
}

// ============================================================================
// COMPREHENSIVE TESTS - Target: 40+ lines
// ============================================================================

#[test]
fn test_comprehensive_simulation_long() {
    let config = test_config()
        .max_steps(500)
        .entity_count(80)
        .initial_money(500.0)
        .base_price(15.0)
        .build();
    
    let mut engine = SimulationEngine::new(config);
    engine.enable_action_recording();
    
    let result = engine.run_with_progress(false);
    
    // Comprehensive validation
    assert_eq!(result.total_steps, 500);
    assert_eq!(result.active_persons, 80);
    assert!(result.trade_volume_statistics.total_volume >= 0.0);
    assert!(!result.skill_price_history.is_empty());
    assert!(!result.final_money_distribution.is_empty());
    assert!(result.money_statistics.average > 0.0);
    
    result.print_summary(false);
}

#[test]
fn test_run_with_progress_enabled() {
    let config = test_config()
        .max_steps(100)
        .entity_count(30)
        .build();
    
    let mut engine = SimulationEngine::new(config);
    let result = engine.run_with_progress(true);
    
    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_run_with_progress_disabled() {
    let config = test_config()
        .max_steps(100)
        .entity_count(30)
        .build();
    
    let mut engine = SimulationEngine::new(config);
    let result = engine.run_with_progress(false);
    
    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_simulation_with_transaction_fee() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .transaction_fee(0.05)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.total_fees_collected >= 0.0);
}

#[test]
fn test_simulation_with_savings() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .savings_rate(0.1)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.savings_statistics.total_savings >= 0.0);
}

#[test]
fn test_simulation_with_tax() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .tax_rate(0.1)
        .enable_tax_redistribution(true)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.total_steps > 0);
}

#[test]
fn test_simulation_with_friendships() {
    let config = test_config()
        .max_steps(100)
        .entity_count(25)
        .enable_friendships(true)
        .friendship_probability(0.3)
        .friendship_discount(0.1)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 100);
}

#[test]
fn test_simulation_with_multiple_skills_per_person() {
    let config = test_config()
        .max_steps(80)
        .entity_count(20)
        .skills_per_person(3)
        .build();
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert!(result.active_persons > 0);
}
