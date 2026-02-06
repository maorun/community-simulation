//! Ultra-simplified tests to cover specific small gaps in crisis.rs, error.rs, voting.rs, 
//! parameter_sweep.rs, and market.rs Display/Debug implementations

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::result::IncrementalStats;
use crate::scenario::Scenario;

// ==================================================
// RESULT TESTS - IncrementalStats comprehensive coverage
// ==================================================

#[test]
fn test_incremental_stats_new_state() {
    let stats = IncrementalStats::new();
    assert_eq!(stats.count(), 0);
    assert_eq!(stats.mean(), 0.0);
    assert_eq!(stats.variance(), 0.0);
    assert_eq!(stats.std_dev(), 0.0);
}

#[test]
fn test_incremental_stats_single_value() {
    let mut stats = IncrementalStats::new();
    stats.update(42.0);
    
    assert_eq!(stats.count(), 1);
    assert_eq!(stats.mean(), 42.0);
    assert_eq!(stats.variance(), 0.0);
    assert_eq!(stats.std_dev(), 0.0);
}

#[test]
fn test_incremental_stats_multiple_values() {
    let mut stats = IncrementalStats::new();
    stats.update(10.0);
    stats.update(20.0);
    stats.update(30.0);
    
    assert_eq!(stats.count(), 3);
    assert_eq!(stats.mean(), 20.0);
    assert!((stats.variance() - 100.0).abs() < 1e-10);
    assert!((stats.std_dev() - 10.0).abs() < 1e-10);
}

#[test]
fn test_incremental_stats_reset() {
    let mut stats = IncrementalStats::new();
    stats.update(10.0);
    stats.update(20.0);
    stats.update(30.0);
    
    stats.reset();
    
    assert_eq!(stats.count(), 0);
    assert_eq!(stats.mean(), 0.0);
    assert_eq!(stats.variance(), 0.0);
}

// ==================================================
// SCENARIO TESTS - Covering scenario.rs methods
// ==================================================

#[test]
fn test_scenario_all_includes_all_variants() {
    let all = Scenario::all();
    assert_eq!(all.len(), 5);
    assert!(all.contains(&Scenario::Original));
    assert!(all.contains(&Scenario::DynamicPricing));
    assert!(all.contains(&Scenario::AdaptivePricing));
    assert!(all.contains(&Scenario::AuctionPricing));
    assert!(all.contains(&Scenario::ClimateChange));
}

#[test]
fn test_scenario_description_not_empty() {
    for scenario in Scenario::all() {
        assert!(!scenario.description().is_empty());
    }
}

#[test]
fn test_scenario_mechanism_not_empty() {
    for scenario in Scenario::all() {
        assert!(!scenario.mechanism().is_empty());
    }
}

#[test]
fn test_scenario_use_case_not_empty() {
    for scenario in Scenario::all() {
        assert!(!scenario.use_case().is_empty());
    }
}

#[test]
fn test_scenario_is_default_only_original() {
    assert!(Scenario::Original.is_default());
    assert!(!Scenario::DynamicPricing.is_default());
    assert!(!Scenario::AdaptivePricing.is_default());
    assert!(!Scenario::AuctionPricing.is_default());
    assert!(!Scenario::ClimateChange.is_default());
}

// ==================================================
// ENGINE TESTS - Running simulations with all scenarios
// ==================================================

#[test]
fn test_engine_run_all_scenarios() {
    for scenario in Scenario::all() {
        let mut config = SimulationConfig::default();
        config.scenario = scenario;
        config.max_steps = 15;
        
        let mut engine = SimulationEngine::new(config);
        
        for _ in 0..15 {
            engine.step();
        }
        
        assert_eq!(engine.get_current_step(), 15);
    }
}

#[test]
fn test_engine_with_many_features_enabled() {
    let mut config = SimulationConfig {
        max_steps: 25,
        entity_count: 20,
        ..Default::default()
    };
    
    // Enable many features
    config.enable_loans = true;
    config.enable_contracts = true;
    config.enable_insurance = true;
    config.enable_trade_agreements = true;
    config.enable_certification = true;
    config.enable_black_market = true;
    config.enable_environment = true;
    config.enable_voting = true;
    config.enable_trust_networks = true;
    config.enable_externalities = true;
    config.enable_technology_breakthroughs = true;
    config.enable_resource_pools = true;
    
    let mut engine = SimulationEngine::new(config);
    
    // Run simulation
    for _ in 0..25 {
        engine.step();
    }
    
    assert_eq!(engine.get_current_step(), 25);
    assert!(engine.get_entities().len() > 0);
}

#[test]
fn test_engine_checkpoint_save_and_load() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let checkpoint_path = temp_dir.path().join("checkpoint.json");
    
    let config = SimulationConfig {
        max_steps: 40,
        entity_count: 15,
        ..Default::default()
    };
    
    let mut engine = SimulationEngine::new(config);
    
    // Run halfway
    for _ in 0..20 {
        engine.step();
    }
    
    // Save checkpoint
    engine.save_checkpoint(&checkpoint_path).unwrap();
    assert!(checkpoint_path.exists());
    
    // Load checkpoint
    let loaded = SimulationEngine::load_checkpoint(&checkpoint_path).unwrap();
    assert_eq!(loaded.get_current_step(), 20);
    assert_eq!(loaded.get_entities().len(), 15);
    
    // Continue running
    let mut continued = loaded;
    for _ in 0..20 {
        continued.step();
    }
    
    assert_eq!(continued.get_current_step(), 40);
}
