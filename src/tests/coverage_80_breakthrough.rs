/// Tests specifically targeting uncovered code paths to reach 80% coverage
use crate::{
    crisis::CrisisEvent, engine::SimulationEngine, error::SimulationError, scenario::Scenario,
    SimulationConfig,
};

/// Test all crisis event types
#[test]
fn test_crisis_event_market_crash() {
    let mut rng = rand::rng();
    let base_value = 100.0;
    let severity = 0.5;

    let event = CrisisEvent::MarketCrash;
    let adjusted = event.apply_effect(base_value, severity, &mut rng);

    // Market crash should reduce value by 20-40% (so 60-80% of original)
    assert!(adjusted > 0.0);
    assert!(adjusted < base_value);
}

#[test]
fn test_crisis_event_demand_shock() {
    let mut rng = rand::rng();
    let base_value = 100.0;
    let severity = 0.5;

    let event = CrisisEvent::DemandShock;
    let adjusted = event.apply_effect(base_value, severity, &mut rng);

    // Demand shock should reduce value
    assert!(adjusted > 0.0);
    assert!(adjusted < base_value);
}

#[test]
fn test_crisis_event_supply_shock() {
    let mut rng = rand::rng();
    let base_value = 100.0;
    let severity = 0.5;

    let event = CrisisEvent::SupplyShock;
    let adjusted = event.apply_effect(base_value, severity, &mut rng);

    // Supply shock should reduce value
    assert!(adjusted > 0.0);
    assert!(adjusted < base_value);
}

#[test]
fn test_crisis_event_currency_devaluation() {
    let mut rng = rand::rng();
    let base_value = 100.0;
    let severity = 0.5;

    let event = CrisisEvent::CurrencyDevaluation;
    let adjusted = event.apply_effect(base_value, severity, &mut rng);

    // Currency devaluation should reduce value by 10-30%
    assert!(adjusted > 0.0);
    assert!(adjusted < base_value);
}

#[test]
fn test_crisis_event_technology_shock() {
    let mut rng = rand::rng();
    let base_value = 100.0;
    let severity = 0.5;

    let event = CrisisEvent::TechnologyShock;
    let adjusted = event.apply_effect(base_value, severity, &mut rng);

    // Technology shock should reduce value by 50-80%
    assert!(adjusted > 0.0);
    assert!(adjusted < base_value * 0.5);
}

#[test]
fn test_crisis_events_with_varying_severity() {
    let mut rng = rand::rng();
    let base_value = 100.0;

    for event in [
        CrisisEvent::MarketCrash,
        CrisisEvent::DemandShock,
        CrisisEvent::SupplyShock,
        CrisisEvent::CurrencyDevaluation,
        CrisisEvent::TechnologyShock,
    ] {
        // Test low severity
        let low = event.apply_effect(base_value, 0.0, &mut rng);
        assert!(low > 0.0);

        // Test high severity
        let high = event.apply_effect(base_value, 1.0, &mut rng);
        assert!(high > 0.0);
        assert!(high < low); // Higher severity should cause more reduction
    }
}

/// Test scenario with per-skill price limits
#[test]
fn test_scenario_per_skill_price_limits() {
    let mut config = SimulationConfig {
        entity_count: 10,
        max_steps: 5,
        scenario: Scenario::Original,
        ..Default::default()
    };

    // Set some per-skill price limits using String keys
    config
        .per_skill_price_limits
        .insert("skill_0".to_string(), (Some(5.0), Some(50.0)));
    config
        .per_skill_price_limits
        .insert("skill_1".to_string(), (Some(10.0), Some(100.0)));

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
}

/// Test original scenario price adjustment mechanism
#[test]
fn test_original_scenario_price_adjustment() {
    let config = SimulationConfig {
        entity_count: 20,
        max_steps: 10,
        scenario: Scenario::Original,
        price_elasticity_factor: 0.5,
        volatility_percentage: 0.1,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 10);
    assert!(result.trade_volume_statistics.total_trades > 0);
}

/// Test dynamic pricing scenario
#[test]
fn test_dynamic_pricing_scenario() {
    let config = SimulationConfig {
        entity_count: 15,
        max_steps: 10,
        scenario: Scenario::DynamicPricing,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 10);
    // Dynamic pricing should still allow some trades
    assert!(result.active_persons == 15);
}

/// Test scenario with high volatility
#[test]
fn test_high_volatility_scenario() {
    let config = SimulationConfig {
        entity_count: 10,
        max_steps: 5,
        volatility_percentage: 0.5, // 50% volatility
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
}

/// Test scenario with very high price elasticity
#[test]
fn test_high_price_elasticity() {
    let config = SimulationConfig {
        entity_count: 10,
        max_steps: 5,
        price_elasticity_factor: 2.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
}

/// Test error Display implementations
#[test]
fn test_error_display_coverage() {
    // Test UnsupportedConfigFormat
    let err = SimulationError::UnsupportedConfigFormat(".txt".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Unsupported configuration file format"));
    assert!(display.contains(".txt"));
}

/// Test config loading errors
#[test]
fn test_config_load_nonexistent_file() {
    let result = SimulationConfig::from_file("/nonexistent/path/config.yaml");
    assert!(result.is_err());

    match result {
        Err(SimulationError::ConfigFileRead(_)) => {},
        _ => panic!("Expected ConfigFileRead error"),
    }
}

/// Test config with unsupported format
#[test]
fn test_config_unsupported_format() {
    use std::fs;
    use std::io::Write;

    let temp_path = std::env::temp_dir().join("test_config.txt");
    let mut file = fs::File::create(&temp_path).unwrap();
    write!(file, "some content").unwrap();

    let result = SimulationConfig::from_file(&temp_path);
    let _ = fs::remove_file(&temp_path);

    assert!(result.is_err());
    match result {
        Err(SimulationError::UnsupportedConfigFormat(ext)) => {
            assert_eq!(ext, "txt"); // Without the dot
        },
        _ => panic!("Expected UnsupportedConfigFormat error"),
    }
}

/// Test config with minimum values
#[test]
fn test_config_minimum_values() {
    let config = SimulationConfig {
        entity_count: 1,
        max_steps: 1,
        initial_money_per_person: 1.0,
        base_skill_price: 1.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 1);
    assert_eq!(result.active_persons, 1);
}

/// Test config with maximum practical values
#[test]
fn test_config_large_values() {
    let config = SimulationConfig {
        entity_count: 100,
        max_steps: 50,
        initial_money_per_person: 10000.0,
        base_skill_price: 1000.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    engine.step();
    engine.step();

    let result = engine.get_current_result();
    assert!(result.active_persons == 100);
}

/// Test various scenario configurations
#[test]
fn test_all_scenario_types() {
    for scenario in [Scenario::Original, Scenario::DynamicPricing] {
        let config = SimulationConfig {
            entity_count: 5,
            max_steps: 2,
            scenario: scenario.clone(),
            ..Default::default()
        };

        let mut engine = SimulationEngine::new(config);
        let result = engine.run();

        assert_eq!(result.total_steps, 2);
        assert_eq!(result.active_persons, 5);
    }
}

/// Test engine with mixed feature flags
#[test]
fn test_engine_with_features() {
    let mut config = SimulationConfig { entity_count: 10, max_steps: 5, ..Default::default() };

    // Enable various features
    config.enable_loans = true;
    config.enable_contracts = true;
    config.enable_mentorship = true;
    config.enable_resource_pools = true;
    config.enable_trade_agreements = true;
    config.enable_insurance = true;
    config.enable_environment = true;
    config.enable_assets = true;
    config.enable_black_market = true;
    config.enable_automation = true;
    config.enable_friendships = true;
    config.enable_production = true;
    config.enable_externalities = true;
    config.enable_investments = true;
    config.enable_crisis_events = true;
    config.enable_technology_breakthroughs = true;

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);
}

/// Test engine with partial feature flags
#[test]
fn test_engine_partial_features() {
    let mut config = SimulationConfig { entity_count: 8, max_steps: 3, ..Default::default() };

    // Enable only some features
    config.enable_loans = true;
    config.enable_friendships = true;
    config.enable_production = false;
    config.enable_crisis_events = false;

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 3);
}

/// Test config with extreme volatility
#[test]
fn test_extreme_volatility() {
    let config = SimulationConfig {
        entity_count: 5,
        max_steps: 3,
        volatility_percentage: 1.0, // 100% volatility
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 3);
}

/// Test config with zero volatility
#[test]
fn test_zero_volatility() {
    let config = SimulationConfig {
        entity_count: 5,
        max_steps: 3,
        volatility_percentage: 0.0, // No volatility
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 3);
}

/// Test config with various price ranges
#[test]
fn test_price_range_variations() {
    let config = SimulationConfig {
        entity_count: 10,
        max_steps: 5,
        min_skill_price: 1.0,
        base_skill_price: 50.0,
        ..Default::default()
    };

    let mut engine = SimulationEngine::new(config);
    let result = engine.run();

    assert_eq!(result.total_steps, 5);

    // Verify prices stay within bounds
    let market = engine.get_market();
    for skill in market.skills.values() {
        assert!(skill.current_price >= 1.0);
    }
}

/// Test config TOML parsing path
#[test]
fn test_config_toml_parsing() {
    use std::fs;
    use std::io::Write;

    let temp_path = std::env::temp_dir().join("test_config.toml");
    let mut file = fs::File::create(&temp_path).unwrap();
    write!(
        file,
        r#"
entity_count = 5
max_steps = 10
initial_money_per_person = 100.0
"#
    )
    .unwrap();

    let result = SimulationConfig::from_file(&temp_path);
    let _ = fs::remove_file(&temp_path);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.entity_count, 5);
    assert_eq!(config.max_steps, 10);
}

/// Test config YAML parsing with various keys
#[test]
fn test_config_yaml_comprehensive() {
    use std::fs;
    use std::io::Write;

    let temp_path = std::env::temp_dir().join("test_config.yml");
    let mut file = fs::File::create(&temp_path).unwrap();
    write!(
        file,
        r#"
entity_count: 15
max_steps: 20
initial_money_per_person: 200.0
base_skill_price: 25.0
volatility_percentage: 0.15
"#
    )
    .unwrap();

    let result = SimulationConfig::from_file(&temp_path);
    let _ = fs::remove_file(&temp_path);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.entity_count, 15);
    assert_eq!(config.max_steps, 20);
}

/// Test invalid TOML parsing
#[test]
fn test_config_invalid_toml() {
    use std::fs;
    use std::io::Write;

    let temp_path = std::env::temp_dir().join("test_invalid.toml");
    let mut file = fs::File::create(&temp_path).unwrap();
    write!(file, "invalid toml [[[ content").unwrap();

    let result = SimulationConfig::from_file(&temp_path);
    let _ = fs::remove_file(&temp_path);

    assert!(result.is_err());
    match result {
        Err(SimulationError::TomlParse(_)) => {},
        _ => panic!("Expected TomlParse error"),
    }
}

/// Test invalid YAML parsing
#[test]
fn test_config_invalid_yaml() {
    use std::fs;
    use std::io::Write;

    let temp_path = std::env::temp_dir().join("test_invalid.yaml");
    let mut file = fs::File::create(&temp_path).unwrap();
    write!(file, "invalid: yaml: content: [[[").unwrap();

    let result = SimulationConfig::from_file(&temp_path);
    let _ = fs::remove_file(&temp_path);

    assert!(result.is_err());
    match result {
        Err(SimulationError::YamlParse(_)) => {},
        _ => panic!("Expected YamlParse error"),
    }
}
