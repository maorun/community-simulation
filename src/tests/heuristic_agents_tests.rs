// Tests for heuristic (rule-of-thumb) agents

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;
use crate::person::Heuristic;
use crate::tests::test_helpers::test_config;

fn heuristic_config(enabled: bool, share: f64) -> SimulationConfig {
    let mut config = test_config().entity_count(60).max_steps(80).seed(4242).build();
    config.enable_friendships = true;
    // Education adds extra sellers per skill, so seller choice matters.
    config.enable_education = true;
    config.learning_probability = 0.3;
    config.learning_cost_multiplier = 0.2;
    config.enable_heuristic_agents = enabled;
    config.heuristic_agent_share = share;
    config
}

#[test]
fn test_heuristic_agents_change_average_wealth_vs_optimizing_agents() {
    let mut engine = SimulationEngine::new(heuristic_config(true, 0.5));
    let result = engine.run();

    let stats = result.heuristic_statistics.expect("heuristic statistics must exist");
    let heuristic_count = stats.cheapest_seller.agent_count
        + stats.trust_friends.agent_count
        + stats.follow_majority.agent_count;
    assert_eq!(heuristic_count, 30);
    assert_eq!(stats.optimizing.agent_count, 30);
    assert!(stats.cheapest_seller.agent_count > 0);
    assert!(stats.trust_friends.agent_count > 0);
    assert!(stats.follow_majority.agent_count > 0);

    let heuristic_wealth = (stats.cheapest_seller.average_wealth
        * stats.cheapest_seller.agent_count as f64
        + stats.trust_friends.average_wealth * stats.trust_friends.agent_count as f64
        + stats.follow_majority.average_wealth * stats.follow_majority.agent_count as f64)
        / heuristic_count as f64;
    let gap = (heuristic_wealth - stats.optimizing.average_wealth).abs();
    assert!(
        gap > 0.5,
        "heuristic and optimizing agents must differ in wealth, gap was {gap} \
         (heuristic {heuristic_wealth}, optimizing {})",
        stats.optimizing.average_wealth
    );
}

#[test]
fn test_seller_rules_change_simulation_outcome() {
    // Both runs use the same flag, so only the seller rules differ (share 0.0 = no rule).
    let without_rules = SimulationEngine::new(heuristic_config(true, 0.0)).run();
    let with_rules = SimulationEngine::new(heuristic_config(true, 1.0)).run();

    assert!(without_rules
        .final_persons_data
        .iter()
        .all(|e| e.person_data.heuristic.is_none()));
    assert!(with_rules.final_persons_data.iter().all(|e| e.person_data.heuristic.is_some()));
    let money = |r: &crate::result::SimulationResult| -> Vec<f64> {
        r.final_persons_data.iter().map(|e| e.person_data.money).collect()
    };
    assert_ne!(money(&without_rules), money(&with_rules));
}

#[test]
fn test_heuristic_agents_are_deterministic() {
    let first = SimulationEngine::new(heuristic_config(true, 0.5)).run();
    let second = SimulationEngine::new(heuristic_config(true, 0.5)).run();

    assert_eq!(first.heuristic_statistics, second.heuristic_statistics);
    assert_eq!(
        first.trade_volume_statistics.total_trades,
        second.trade_volume_statistics.total_trades
    );
    let heuristics = |r: &crate::result::SimulationResult| -> Vec<Option<Heuristic>> {
        r.final_persons_data.iter().map(|e| e.person_data.heuristic).collect()
    };
    assert_eq!(heuristics(&first), heuristics(&second));
}

#[test]
fn test_share_is_ignored_when_flag_is_off() {
    // The share value must not matter while the flag is off.
    let a = SimulationEngine::new(heuristic_config(false, 0.5)).run();
    let b = SimulationEngine::new(heuristic_config(false, 1.0)).run();

    assert!(a.heuristic_statistics.is_none());
    assert!(b.heuristic_statistics.is_none());
    assert_eq!(a.trade_volume_statistics.total_trades, b.trade_volume_statistics.total_trades);
    assert!(a.final_persons_data.iter().all(|e| e.person_data.heuristic.is_none()));
    let money = |r: &crate::result::SimulationResult| -> Vec<f64> {
        r.final_persons_data.iter().map(|e| e.person_data.money).collect()
    };
    assert_eq!(money(&a), money(&b));
}

#[test]
fn test_validation_rejects_invalid_heuristic_share() {
    for share in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        let config = heuristic_config(true, share);
        assert!(config.validate().is_err(), "share {share} must be rejected");
    }
    for share in [0.0, 0.5, 1.0] {
        assert!(heuristic_config(true, share).validate().is_ok());
    }
}

#[test]
fn test_config_without_heuristic_fields_deserializes_with_defaults() {
    let json = serde_json::to_value(SimulationConfig::default()).expect("serialize");
    let mut map = json.as_object().expect("object").clone();
    map.remove("enable_heuristic_agents");
    map.remove("heuristic_agent_share");
    let config: SimulationConfig =
        serde_json::from_value(serde_json::Value::Object(map)).expect("deserialize");
    assert!(!config.enable_heuristic_agents);
    assert_eq!(config.heuristic_agent_share, 0.5);
}

#[test]
fn test_validate_rejects_heuristic_agents_with_nepotism() {
    let mut both = heuristic_config(true, 0.5);
    both.enable_nepotism = true;
    match both.validate() {
        Err(crate::error::SimulationError::ValidationError(msg)) => {
            assert!(msg.contains("enable_heuristic_agents"), "unexpected message: {msg}");
            assert!(msg.contains("enable_nepotism"), "unexpected message: {msg}");
        },
        other => panic!("expected ValidationError, got {other:?}"),
    }

    let mut only_nepotism = heuristic_config(false, 0.5);
    only_nepotism.enable_nepotism = true;
    assert!(only_nepotism.validate().is_ok());
    assert!(heuristic_config(true, 0.5).validate().is_ok());
}
