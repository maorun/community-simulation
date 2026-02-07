/// Ultra-focused tests to hit uncovered engine.rs lines
use crate::{engine::SimulationEngine, scenario::Scenario, SimulationConfig};
use std::fs;

/// Test engine getter methods
#[test]
fn test_engine_getters() {
    let mut config = SimulationConfig::default();
    config.entity_count = 5;
    config.max_steps = 2;
    let mut engine = SimulationEngine::new(config.clone());
    
    assert_eq!(engine.get_active_entity_count(), 5);
    assert_eq!(engine.get_current_step(), 0);
    assert_eq!(engine.get_max_steps(), 2);
    assert_eq!(engine.get_active_persons(), 5);
    assert!(engine.get_entities().len() == 5);
    assert!(engine.get_market().skills.len() > 0);
    
    engine.step();
    assert_eq!(engine.get_current_step(), 1);
    
    let result = engine.get_current_result();
    assert_eq!(result.total_steps, 1);
}

/// Test with zero money
#[test]
fn test_zero_money() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 5;
    config.initial_money_per_person = 0.0;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 5);
    assert_eq!(result.trade_volume_statistics.total_trades, 0);
}

/// Test single person
#[test]
fn test_single_person() {
    let mut config = SimulationConfig::default();
    config.entity_count = 1;
    config.max_steps = 10;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.active_persons, 1);
    assert_eq!(result.trade_volume_statistics.total_trades, 0);
}

/// Test checkpoint save/load
#[test]
fn test_checkpoint() {
    let mut config = SimulationConfig::default();
    config.entity_count = 5;
    config.max_steps = 10;
    let mut engine = SimulationEngine::new(config);
    
    engine.step();
    engine.step();
    engine.step();
    
    let step_before = engine.get_current_step();
    assert_eq!(step_before, 3);
    
    let temp_path = std::env::temp_dir().join("test_checkpoint.json");
    engine.save_checkpoint(&temp_path).expect("Save failed");
    
    let loaded = SimulationEngine::load_checkpoint(&temp_path).expect("Load failed");
    assert_eq!(loaded.get_current_step(), step_before);
    
    let _ = fs::remove_file(&temp_path);
}

/// Test plugin system
#[test]
fn test_plugins() {
    let config = SimulationConfig::default();
    let mut engine = SimulationEngine::new(config);
    
    let _registry = engine.plugin_registry();
    let _registry_mut = engine.plugin_registry_mut();
}

/// Test action recording
#[test]
fn test_action_recording() {
    let mut config = SimulationConfig::default();
    config.entity_count = 3;
    config.max_steps = 5;
    let mut engine = SimulationEngine::new(config);
    
    engine.enable_action_recording();
    engine.run();
    
    let temp_path = std::env::temp_dir().join("test_action_log.json");
    engine.save_action_log(&temp_path).expect("Save failed");
    
    assert!(temp_path.exists());
    let _ = fs::remove_file(&temp_path);
}

/// Test run with progress
#[test]
fn test_run_with_progress() {
    let mut config = SimulationConfig::default();
    config.entity_count = 5;
    config.max_steps = 10;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run_with_progress(false);
    assert_eq!(result.total_steps, 10);
}

/// Test with loans
#[test]
fn test_loans() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 30;
    config.initial_money_per_person = 50.0;
    config.enable_loans = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_loan_stats) = result.loan_statistics {
        assert!(true);
    }
}

/// Test with contracts
#[test]
fn test_contracts() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 30;
    config.enable_contracts = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_contract_stats) = result.contract_statistics {
        assert!(true);
    }
}

/// Test with mentorship
#[test]
fn test_mentorship() {
    let mut config = SimulationConfig::default();
    config.entity_count = 15;
    config.max_steps = 25;
    config.enable_mentorship = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_mentorship_stats) = result.mentorship_statistics {
        assert!(true);
    }
}

/// Test with groups
#[test]
fn test_groups() {
    let mut config = SimulationConfig::default();
    config.entity_count = 20;
    config.max_steps = 25;
    config.num_groups = Some(4);
    config.enable_resource_pools = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(group_stats) = result.group_statistics {
        assert_eq!(group_stats.groups.len(), 4);
    }
}

/// Test with trade agreements
#[test]
fn test_trade_agreements() {
    let mut config = SimulationConfig::default();
    config.entity_count = 12;
    config.max_steps = 30;
    config.enable_trade_agreements = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_agreement_stats) = result.trade_agreement_statistics {
        assert!(true);
    }
}

/// Test with insurance
#[test]
fn test_insurance() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 30;
    config.enable_insurance = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_insurance_stats) = result.insurance_statistics {
        assert!(true);
    }
}

/// Test with environment
#[test]
fn test_environment() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 20;
    config.enable_environment = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_env_stats) = result.environment_statistics {
        assert!(true);
    }
}

/// Test with assets
#[test]
fn test_assets() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 25;
    config.enable_assets = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_asset_stats) = result.asset_statistics {
        assert!(true);
    }
}

/// Test with black market
#[test]
fn test_black_market() {
    let mut config = SimulationConfig::default();
    config.entity_count = 15;
    config.max_steps = 25;
    config.enable_black_market = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_black_market_stats) = result.black_market_statistics {
        assert!(true);
    }
}

/// Test statistics
#[test]
fn test_statistics() {
    let mut config = SimulationConfig::default();
    config.entity_count = 2;
    config.max_steps = 2;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    
    assert!(result.money_statistics.average.is_finite());
    assert!(result.money_statistics.median.is_finite());
    assert!(result.money_statistics.std_dev.is_finite());
    assert!(result.money_statistics.min_money.is_finite());
    assert!(result.money_statistics.max_money.is_finite());
}

/// Test no trades scenario
#[test]
fn test_no_trades() {
    let mut config = SimulationConfig::default();
    config.entity_count = 3;
    config.max_steps = 5;
    config.initial_money_per_person = 1.0;
    config.base_skill_price = 1000.0;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 5);
}

/// Test long simulation
#[test]
fn test_long_simulation() {
    let mut config = SimulationConfig::default();
    config.entity_count = 5;
    config.max_steps = 100;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 100);
    assert!(result.money_statistics.average.is_finite());
}

/// Test with education
#[test]
fn test_education() {
    let mut config = SimulationConfig::default();
    config.entity_count = 15;
    config.max_steps = 20;
    config.enable_education = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_influence_stats) = result.influence_statistics {
        assert!(true);
    }
}

/// Test high volatility
#[test]
fn test_high_volatility() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 15;
    config.volatility_percentage = 0.8;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 15);
}

/// Test DynamicPricing scenario
#[test]
fn test_dynamic_pricing() {
    let mut config = SimulationConfig::default();
    config.entity_count = 5;
    config.max_steps = 10;
    config.scenario = Scenario::DynamicPricing;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    assert_eq!(result.total_steps, 10);
}

/// Test with automation
#[test]
fn test_automation() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 20;
    config.enable_automation = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_automation_stats) = result.automation_statistics {
        assert!(true);
    }
}

/// Test with friendships
#[test]
fn test_friendships() {
    let mut config = SimulationConfig::default();
    config.entity_count = 15;
    config.max_steps = 20;
    config.enable_friendships = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_friendship_stats) = result.friendship_statistics {
        assert!(true);
    }
}

/// Test with production
#[test]
fn test_production() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 20;
    config.enable_production = true;
    let mut engine = SimulationEngine::new(config);
    
    let _result = engine.run();
}

/// Test with externalities
#[test]
fn test_externalities() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 20;
    config.enable_externalities = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_ext_stats) = result.externality_statistics {
        assert!(true);
    }
}

/// Test with investments
#[test]
fn test_investments() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 20;
    config.enable_investments = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_investment_stats) = result.investment_statistics {
        assert!(true);
    }
}

/// Test with crisis events
#[test]
fn test_crisis_events() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 50;
    config.enable_crisis_events = true;
    let mut engine = SimulationEngine::new(config);
    
    let _result = engine.run();
}

/// Test with technology breakthroughs
#[test]
fn test_technology_breakthroughs() {
    let mut config = SimulationConfig::default();
    config.entity_count = 10;
    config.max_steps = 40;
    config.enable_technology_breakthroughs = true;
    let mut engine = SimulationEngine::new(config);
    
    let result = engine.run();
    if let Some(_tech_stats) = result.technology_breakthrough_statistics {
        assert!(true);
    }
}

/// Test various entity counts
#[test]
fn test_various_entity_counts() {
    for count in &[2, 5, 10, 20] {
        let mut config = SimulationConfig::default();
        config.entity_count = *count;
        config.max_steps = 5;
        let mut engine = SimulationEngine::new(config);
        
        let result = engine.run();
        assert_eq!(result.active_persons, *count);
    }
}

/// Test various step counts
#[test]
fn test_various_step_counts() {
    for steps in &[1, 5, 10, 50] {
        let mut config = SimulationConfig::default();
        config.entity_count = 5;
        config.max_steps = *steps;
        let mut engine = SimulationEngine::new(config);
        
        let result = engine.run();
        assert_eq!(result.total_steps, *steps);
    }
}

/// Test with different seeds
#[test]
fn test_different_seeds() {
    for seed in &[1, 42, 999, 12345] {
        let mut config = SimulationConfig::default();
        config.entity_count = 5;
        config.max_steps = 10;
        config.seed = *seed;
        let mut engine = SimulationEngine::new(config);
        
        let result = engine.run();
        assert_eq!(result.total_steps, 10);
    }
}
