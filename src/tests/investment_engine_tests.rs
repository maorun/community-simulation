//! Regression tests: the engine must create, mature, pay and complete investments.

use crate::config::SimulationConfig;
use crate::engine::SimulationEngine;

fn investment_config() -> SimulationConfig {
    SimulationConfig {
        entity_count: 10,
        max_steps: 40,
        seed: 11,
        initial_money_per_person: 500.0,
        enable_investments: true,
        investment_probability: 0.5,
        investment_duration: 10,
        investment_return_rate: 0.02,
        min_money_to_invest: 50.0,
        ..Default::default()
    }
}

#[test]
fn test_no_return_before_maturity_period() {
    let config = investment_config();
    let max_steps = config.max_steps;
    let mut engine = SimulationEngine::new(config);
    let mut paid_after_maturity = false;
    for _ in 0..max_steps {
        engine.step();
        let now = engine.get_current_step();
        for inv in engine.get_investments().values() {
            let maturity = inv.maturity_period().expect("engine sets a maturity period");
            if now <= inv.created_at_step + maturity {
                assert_eq!(inv.returns_paid, 0, "return paid before maturity at step {now}");
            }
            if inv.returns_paid > 0 {
                paid_after_maturity = true;
            }
        }
    }
    assert!(!engine.get_investments().is_empty(), "investments must be created");
    assert!(paid_after_maturity, "returns must be paid after maturity");
}

#[test]
fn test_investment_statistics_reflect_run() {
    let mut engine = SimulationEngine::new(investment_config());
    let result = engine.run();
    let stats = result.investment_statistics.expect("statistics must exist");
    assert!(stats.total_investments_created > 0);
    assert!(stats.total_returns_paid > 0.0);
    assert!(stats.total_investments_completed > 0);
    assert_eq!(
        stats.active_investments + stats.total_investments_completed,
        stats.total_investments_created
    );
}

#[test]
fn test_investments_keep_money_conserved() {
    use crate::invariant::{Invariant, MoneyConservationInvariant};
    let mut engine = SimulationEngine::new(investment_config());
    let initial: f64 = engine.get_entities().iter().map(|e| e.person_data.money).sum();
    let invariant = MoneyConservationInvariant::new(initial);
    for _ in 0..30 {
        engine.step();
        assert!(invariant.check(&engine).is_ok(), "money not conserved");
    }
    assert!(!engine.get_investments().is_empty());
}

#[test]
fn test_disabled_investments_are_deterministic_and_none() {
    let mut config = investment_config();
    config.enable_investments = false;
    let mut a = SimulationEngine::new(config.clone());
    let mut b = SimulationEngine::new(config);
    let ra = a.run();
    let rb = b.run();
    assert!(ra.investment_statistics.is_none());
    assert!(a.get_investments().is_empty());
    assert_eq!(ra.final_money_distribution, rb.final_money_distribution);
    assert_eq!(ra.total_steps, rb.total_steps);
}

#[test]
fn test_investment_config_validation() {
    let mut c = investment_config();
    c.investment_return_rate = 1.5;
    assert!(c.validate().is_err());
    let mut c = investment_config();
    c.investment_probability = -0.1;
    assert!(c.validate().is_err());
    let mut c = investment_config();
    c.investment_duration = 0;
    assert!(c.validate().is_err());
    let mut c = investment_config();
    c.min_money_to_invest = -1.0;
    assert!(c.validate().is_err());
    assert!(investment_config().validate().is_ok());
}

#[test]
fn test_first_return_arrives_exactly_at_maturity() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    let mut seen: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut checked = false;
    for _ in 0..30 {
        engine.step();
        let now = engine.get_current_step();
        for inv in engine.get_investments().values() {
            let first = inv.created_at_step + inv.maturity_period().expect("maturity set");
            if inv.returns_paid >= 1 {
                let at = *seen.entry(inv.id).or_insert(now);
                if at == now && inv.returns_paid == 1 {
                    // `current_step` advances at the end of `step()`, so the paying step is `now - 1`.
                    assert_eq!(now - 1, first, "first return must come at maturity");
                    checked = true;
                }
            }
        }
    }
    assert!(checked, "at least one first return must be observed");
}

#[test]
fn test_checkpoint_restores_investment_state() {
    let path = std::env::temp_dir().join("investment_engine_checkpoint_test.json");
    let mut a = SimulationEngine::new(investment_config());
    for _ in 0..15 {
        a.step();
    }
    a.save_checkpoint(&path).expect("save checkpoint");
    let mut b = SimulationEngine::load_checkpoint(&path).expect("load checkpoint");
    let _ = std::fs::remove_file(&path);
    assert!(!b.get_investments().is_empty());
    assert_eq!(a.get_investments().len(), b.get_investments().len());
    for (id, inv) in a.get_investments() {
        let other = b.get_investments().get(id).expect("investment survives load");
        assert_eq!(inv.returns_paid, other.returns_paid);
        assert_eq!(inv.is_completed, other.is_completed);
        assert!((inv.total_returns_paid - other.total_returns_paid).abs() < 1e-9);
    }
    // The restored id counter must not collide with existing ids.
    for _ in 0..10 {
        b.step();
    }
    let max_id = b.get_investments().keys().max().copied().expect("ids");
    assert_eq!(max_id + 1, b.get_investments().len(), "ids must stay unique and dense");
}

#[test]
fn test_inactive_target_pauses_payments() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    for _ in 0..4 {
        engine.step();
    }
    let (id, target) = {
        let inv = engine.get_investments().values().next().expect("investment exists");
        (inv.id, inv.target_id.expect("target"))
    };
    engine.get_entities_mut()[target].active = false;
    let paid_before = engine.get_investments()[&id].returns_paid;
    for _ in 0..12 {
        engine.step();
    }
    assert_eq!(engine.get_investments()[&id].returns_paid, paid_before, "paused while inactive");
    engine.get_entities_mut()[target].active = true;
    for _ in 0..3 {
        engine.step();
    }
    assert!(engine.get_investments()[&id].returns_paid > paid_before, "payments resume");
}
