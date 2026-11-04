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

fn education_skill(inv: &crate::investment::Investment) -> crate::skill::SkillId {
    match &inv.investment_type {
        crate::investment::InvestmentType::Education { skill_id, .. } => skill_id.clone(),
        other => panic!("expected an education investment, got {other:?}"),
    }
}

#[test]
fn test_investment_skill_is_new_and_learned_at_maturity() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    let mut start_skills: std::collections::HashMap<usize, bool> = std::collections::HashMap::new();
    let mut granted_seen = false;
    for _ in 0..40 {
        engine.step();
        let now = engine.get_current_step();
        for inv in engine.get_investments().values() {
            let target = inv.target_id.expect("target");
            let skill = education_skill(inv);
            let maturity = inv.maturity_period().expect("maturity");
            let has = engine.get_entities()[target].person_data.has_skill(&skill);
            let own = engine.get_entities()[target]
                .person_data
                .own_skills
                .iter()
                .any(|s| s.id == skill);
            assert!(!own, "the investment skill must be new to the target");
            start_skills.entry(inv.id).or_insert(has);
            // Another matured investment may have taught the same skill to this target.
            let taught_by_other = engine.get_investments().values().any(|o| {
                o.id != inv.id
                    && o.skill_granted
                    && o.target_id == inv.target_id
                    && education_skill(o) == skill
            });
            if now <= inv.created_at_step + maturity && !taught_by_other {
                assert!(!has, "target has the skill before maturity");
                assert!(!inv.skill_granted);
            }
            if inv.skill_granted {
                granted_seen = true;
                assert!(has, "target must hold a granted skill");
            }
        }
    }
    assert!(granted_seen, "at least one skill must be granted");
    assert!(start_skills.values().all(|had| !had), "no target had the skill at creation");
    let result = engine.run();
    let stats = result.investment_statistics.expect("statistics");
    assert!(stats.skills_granted > 0);
}

#[test]
fn test_investment_skill_is_granted_once() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    for _ in 0..40 {
        engine.step();
        for e in engine.get_entities() {
            let mut ids: Vec<_> = e.person_data.learned_skills.iter().map(|s| &s.id).collect();
            let len = ids.len();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), len, "duplicate learned skill");
        }
    }
    let result = engine.run();
    let stats = result.investment_statistics.expect("statistics");
    let mut pairs: Vec<(usize, String)> = engine
        .get_investments()
        .values()
        .filter(|i| i.skill_granted)
        .map(|i| (i.target_id.expect("target"), education_skill(i)))
        .collect();
    let granted = pairs.len();
    pairs.sort();
    pairs.dedup();
    assert_eq!(pairs.len(), granted, "one skill is taught to one target only once");
    assert_eq!(stats.skills_granted, granted);
    assert!(stats.skills_granted > 0);
}

#[test]
fn test_granted_skill_makes_target_a_provider() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    for _ in 0..40 {
        engine.step();
    }
    let granted: Vec<_> = engine.get_investments().values().filter(|i| i.skill_granted).collect();
    assert!(!granted.is_empty());
    for inv in granted {
        let target = inv.target_id.expect("target");
        let skill = education_skill(inv);
        let entity = &engine.get_entities()[target];
        assert!(entity.person_data.learned_skills.iter().any(|s| s.id == skill));
        assert!(engine.providers_of(&skill).contains(&entity.id), "target must be a provider");
    }
}

#[test]
fn test_investment_without_skill_flags_deserializes() {
    let mut engine = SimulationEngine::new(investment_config());
    for _ in 0..3 {
        engine.step();
    }
    let inv = engine.get_investments().values().next().expect("investment");
    let mut value = serde_json::to_value(inv).expect("serialize");
    let object = value.as_object_mut().expect("object");
    object.remove("skill_granted");
    object.remove("skill_resolved");
    let restored: crate::investment::Investment = serde_json::from_value(value).expect("load");
    assert!(!restored.skill_granted && !restored.skill_resolved);
}

#[test]
fn test_investment_skills_are_deterministic() {
    let run = || {
        let mut engine = SimulationEngine::new(investment_config());
        for _ in 0..40 {
            engine.step();
        }
        let mut learned: Vec<(usize, Vec<String>)> = engine
            .get_entities()
            .iter()
            .map(|e| {
                (e.id, e.person_data.learned_skills.iter().map(|s| s.id.to_string()).collect())
            })
            .collect();
        learned.sort();
        learned
    };
    let a = run();
    assert_eq!(a, run());
    assert!(a.iter().any(|(_, l)| !l.is_empty()), "some skill must be learned");
}

#[test]
fn test_disabled_investments_grant_no_skill() {
    let mut config = investment_config();
    config.enable_investments = false;
    let mut engine = SimulationEngine::new(config);
    for _ in 0..40 {
        engine.step();
    }
    assert!(engine.get_entities().iter().all(|e| e.person_data.learned_skills.is_empty()));
}

#[test]
fn test_checkpoint_keeps_skill_state_and_does_not_relearn() {
    let path = std::env::temp_dir().join("investment_skill_checkpoint_test.json");
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut a = SimulationEngine::new(config);
    for _ in 0..20 {
        a.step();
    }
    assert!(a.get_investments().values().any(|i| i.skill_granted));
    a.save_checkpoint(&path).expect("save checkpoint");
    let mut b = SimulationEngine::load_checkpoint(&path).expect("load checkpoint");
    let _ = std::fs::remove_file(&path);
    for (id, inv) in a.get_investments() {
        let other = &b.get_investments()[id];
        assert_eq!(inv.skill_resolved, other.skill_resolved);
        assert_eq!(inv.skill_granted, other.skill_granted);
    }
    let learned = |e: &SimulationEngine| -> usize {
        e.get_entities().iter().map(|x| x.person_data.learned_skills.len()).sum()
    };
    let before = learned(&b);
    let resolved_ids: Vec<usize> = b
        .get_investments()
        .values()
        .filter(|i| i.skill_resolved)
        .map(|i| i.id)
        .collect();
    for _ in 0..10 {
        b.step();
    }
    for id in resolved_ids {
        assert!(b.get_investments()[&id].skill_resolved, "resolved flag must stay set");
    }
    // Only investments that mature after the load may add skills; none repeats.
    let new_resolved = b
        .get_investments()
        .values()
        .filter(|i| i.skill_resolved && i.skill_granted)
        .count();
    assert!(learned(&b) >= before);
    assert!(
        learned(&b) - before <= new_resolved,
        "learned skills must come from granted investments only"
    );
}

#[test]
fn test_inactive_target_learns_skill_after_reactivation() {
    let mut config = investment_config();
    config.initial_money_per_person = 100000.0;
    let mut engine = SimulationEngine::new(config);
    engine.step();
    let (id, target, skill, created, maturity) = {
        let inv = engine.get_investments().values().next().expect("investment exists");
        (
            inv.id,
            inv.target_id.expect("target"),
            education_skill(inv),
            inv.created_at_step,
            inv.maturity_period().expect("maturity"),
        )
    };
    engine.get_entities_mut()[target].active = false;
    while engine.get_current_step() <= created + maturity + 2 {
        engine.step();
    }
    assert!(!engine.get_investments()[&id].skill_resolved, "inactive target defers learning");
    assert!(!engine.get_entities()[target]
        .person_data
        .learned_skills
        .iter()
        .any(|s| s.id == skill));
    engine.get_entities_mut()[target].active = true;
    engine.step();
    assert!(engine.get_investments()[&id].skill_resolved);
    assert!(engine.get_entities()[target].person_data.has_skill(&skill));
}
