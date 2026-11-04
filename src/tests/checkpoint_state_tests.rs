//! Tests for the full checkpoint state: lemons market exits, heuristic sales counters
//! and the effective environment consumption.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::environment::Resource;
use crate::tests::test_helpers::test_config;
use crate::{SimulationConfig, SimulationEngine};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Checkpoint file in the temp directory with a unique name. It is deleted on drop.
struct TempCheckpoint(PathBuf);

impl TempCheckpoint {
    fn new(label: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!("community_sim_ckpt_{}_{}_{}.json", std::process::id(), label, id);
        Self(std::env::temp_dir().join(name))
    }
}

impl Drop for TempCheckpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn lemons_config() -> SimulationConfig {
    let mut config = test_config().entity_count(40).max_steps(60).seed(7).build();
    config.enable_information_asymmetry = true;
    config.inspection_cost = 0.0;
    config.lemons_exit_threshold_steps = 3;
    config.lemons_exit_quality_margin = 0.2;
    config
}

fn heuristic_config() -> SimulationConfig {
    let mut config = test_config().entity_count(60).max_steps(80).seed(4242).build();
    config.enable_friendships = true;
    config.enable_education = true;
    config.learning_probability = 0.3;
    config.learning_cost_multiplier = 0.2;
    config.enable_heuristic_agents = true;
    config.heuristic_agent_share = 1.0;
    config
}

fn environment_config() -> SimulationConfig {
    let mut config = test_config().entity_count(30).max_steps(20).seed(11).build();
    config.enable_environment = true;
    config
}

fn consumption(engine: &SimulationEngine, resource: Resource) -> f64 {
    engine
        .get_environment()
        .expect("environment must exist")
        .total_consumption
        .get(&resource)
        .copied()
        .unwrap_or(0.0)
}

#[test]
fn lemons_exits_survive_a_checkpoint_round_trip() {
    let mut engine = SimulationEngine::new(lemons_config());
    for _ in 0..40 {
        engine.step();
        if engine.lemons_exited_sellers_count() > 0 {
            break;
        }
    }
    let exited = engine.lemons_exited_sellers_count();
    assert!(exited > 0, "the scenario must produce exits");

    let file = TempCheckpoint::new("lemons");
    engine.save_checkpoint(&file.0).expect("save");
    let mut restored = SimulationEngine::load_checkpoint(&file.0).expect("load");

    assert_eq!(restored.lemons_exited_sellers_count(), exited);
    assert_eq!(restored.lemons_streaks_sorted(), engine.lemons_streaks_sorted());
    assert_eq!(restored.lemons_avg_quality_value(), engine.lemons_avg_quality_value());
    let pairs = engine.lemons_exited_pairs();
    assert_eq!(restored.lemons_exited_pairs(), pairs);
    assert_eq!(pairs.len(), exited);
    for (entity_id, skill_id) in &pairs {
        assert!(
            !restored.providers_of(skill_id).contains(entity_id),
            "seller {entity_id} must stay out of the supply of {skill_id}"
        );
    }

    // A resumed run must not count the same exits again.
    for _ in 0..10 {
        restored.step();
    }
    let after = restored.lemons_exited_pairs();
    for pair in &pairs {
        assert!(after.contains(pair));
    }
    assert_eq!(restored.lemons_exited_sellers_count(), after.len());
}

#[test]
fn lemons_state_stays_empty_when_the_mechanism_is_off() {
    let mut config = lemons_config();
    config.lemons_exit_threshold_steps = 0;
    let mut engine = SimulationEngine::new(config);
    for _ in 0..15 {
        engine.step();
    }
    assert!(engine.lemons_exited_pairs().is_empty());
    assert!(engine.lemons_streaks_sorted().is_empty());
    assert_eq!(engine.lemons_exited_sellers_count(), 0);
}

#[test]
fn heuristic_sales_counters_survive_a_checkpoint_round_trip() {
    let mut engine = SimulationEngine::new(heuristic_config());
    for _ in 0..10 {
        engine.step();
    }
    let (previous, current) = engine.heuristic_sales_sorted();
    assert!(!previous.is_empty(), "previous sales must exist");

    let file = TempCheckpoint::new("heuristic");
    engine.save_checkpoint(&file.0).expect("save");
    let restored = SimulationEngine::load_checkpoint(&file.0).expect("load");

    assert_eq!(restored.heuristic_sales_sorted(), (previous, current));
}

#[test]
fn heuristic_resume_matches_an_uninterrupted_run() {
    let mut straight = SimulationEngine::new(heuristic_config());
    for _ in 0..10 {
        straight.step();
    }
    let file = TempCheckpoint::new("heuristic_resume");
    straight.save_checkpoint(&file.0).expect("save");
    let mut restored = SimulationEngine::load_checkpoint(&file.0).expect("load");

    // The checkpoint keeps the RNG state, so both runs stay identical.
    straight.step();
    restored.step();
    assert_eq!(restored.heuristic_sales_sorted(), straight.heuristic_sales_sorted());
}

#[test]
fn old_checkpoint_without_new_fields_loads_and_converts_consumption() {
    let mut engine = SimulationEngine::new(environment_config());
    for _ in 0..10 {
        engine.step();
    }
    let energy = consumption(&engine, Resource::Energy);
    let water = consumption(&engine, Resource::Water);
    assert!(energy > 0.0 && water > 0.0, "the scenario must consume resources");

    let file = TempCheckpoint::new("old");
    engine.save_checkpoint(&file.0).expect("save");
    let text = std::fs::read_to_string(&file.0).expect("read");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
    let object = value.as_object_mut().expect("object");
    for key in [
        "lemons_under_value_streaks",
        "lemons_exited_pairs",
        "lemons_exited_sellers",
        "lemons_avg_quality",
        "heuristic_previous_sales",
        "heuristic_current_sales",
        "environment_consumption_effective",
    ] {
        assert!(object.remove(key).is_some(), "checkpoint must contain {key}");
    }
    std::fs::write(&file.0, serde_json::to_string(&value).expect("json")).expect("write");

    let restored = SimulationEngine::load_checkpoint(&file.0).expect("old checkpoint must load");
    let expected_energy = energy * Resource::Energy.degradation_factor();
    let expected_water = water * Resource::Water.degradation_factor();
    assert!((consumption(&restored, Resource::Energy) - expected_energy).abs() < 1e-9);
    assert!((consumption(&restored, Resource::Water) - expected_water).abs() < 1e-9);
    assert_eq!(restored.lemons_exited_sellers_count(), 0);
    assert_eq!(restored.lemons_avg_quality_value(), 3.0, "old files use the start value");
}

#[test]
fn new_checkpoint_does_not_multiply_consumption_again() {
    let mut engine = SimulationEngine::new(environment_config());
    for _ in 0..10 {
        engine.step();
    }
    let energy = consumption(&engine, Resource::Energy);

    let file = TempCheckpoint::new("new");
    engine.save_checkpoint(&file.0).expect("save");
    let first = SimulationEngine::load_checkpoint(&file.0).expect("load");
    assert!((consumption(&first, Resource::Energy) - energy).abs() < 1e-9);

    // A second save and load must keep the value.
    first.save_checkpoint(&file.0).expect("save again");
    let second = SimulationEngine::load_checkpoint(&file.0).expect("load again");
    assert!((consumption(&second, Resource::Energy) - energy).abs() < 1e-9);
}

/// Config with several systems that draw random numbers.
fn rng_resume_config() -> SimulationConfig {
    let mut config = test_config().entity_count(60).max_steps(60).seed(2024).build();
    config.enable_friendships = true;
    config.enable_education = true;
    config.learning_probability = 0.3;
    config.learning_cost_multiplier = 0.2;
    config.enable_investments = true;
    config.investment_probability = 0.5;
    config.investment_duration = 10;
    config.min_money_to_invest = 50.0;
    config.enable_heuristic_agents = true;
    config.heuristic_agent_share = 0.5;
    config
}

/// Money per entity id, sorted by id.
fn money_by_id(engine: &SimulationEngine) -> Vec<(usize, f64)> {
    let mut money: Vec<(usize, f64)> =
        engine.get_entities().iter().map(|e| (e.id, e.person_data.money)).collect();
    money.sort_by_key(|(id, _)| *id);
    money
}

/// Price per skill id, sorted by id.
fn prices_sorted(engine: &SimulationEngine) -> Vec<(String, f64)> {
    let market = engine.get_market();
    let mut prices: Vec<(String, f64)> = market
        .skills
        .keys()
        .map(|id| (id.to_string(), market.get_price(id).expect("price")))
        .collect();
    prices.sort_by(|a, b| a.0.cmp(&b.0));
    prices
}

#[test]
fn resumed_run_matches_an_uninterrupted_run() {
    let total_steps = 40;
    let saved_at = 15;

    let mut straight = SimulationEngine::new(rng_resume_config());
    for _ in 0..total_steps {
        straight.step();
    }

    let mut first_part = SimulationEngine::new(rng_resume_config());
    for _ in 0..saved_at {
        first_part.step();
    }
    let file = TempCheckpoint::new("rng_resume");
    first_part.save_checkpoint(&file.0).expect("save");
    let mut resumed = SimulationEngine::load_checkpoint(&file.0).expect("load");
    for _ in saved_at..total_steps {
        resumed.step();
    }

    let straight_result = straight.get_current_result();
    let resumed_result = resumed.get_current_result();
    assert!(straight_result.trades_per_step.iter().sum::<usize>() > 0, "trades must happen");
    assert_eq!(resumed_result.trades_per_step, straight_result.trades_per_step);
    assert_eq!(
        resumed_result.final_money_distribution,
        straight_result.final_money_distribution
    );
    assert_eq!(money_by_id(&resumed), money_by_id(&straight));
    assert_eq!(prices_sorted(&resumed), prices_sorted(&straight));
    assert_eq!(resumed.get_investments().len(), straight.get_investments().len());
}

#[test]
fn checkpoint_without_rng_state_still_loads() {
    let mut engine = SimulationEngine::new(rng_resume_config());
    for _ in 0..10 {
        engine.step();
    }
    let file = TempCheckpoint::new("no_rng_state");
    engine.save_checkpoint(&file.0).expect("save");
    let text = std::fs::read_to_string(&file.0).expect("read");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
    let object = value.as_object_mut().expect("object");
    assert!(object.remove("rng_state").is_some(), "checkpoint must contain rng_state");
    assert!(object.remove("skill_providers").is_some(), "checkpoint must contain providers");
    std::fs::write(&file.0, serde_json::to_string(&value).expect("json")).expect("write");

    let mut restored = SimulationEngine::load_checkpoint(&file.0).expect("old checkpoint loads");
    assert_eq!(restored.get_current_step(), 10);
    restored.step();
    assert_eq!(restored.get_current_step(), 11);
}

/// Saves a checkpoint, edits its JSON and returns the load result.
fn load_edited(
    label: &str,
    edit: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
) -> std::io::Result<SimulationEngine> {
    let mut engine = SimulationEngine::new(rng_resume_config());
    for _ in 0..5 {
        engine.step();
    }
    let file = TempCheckpoint::new(label);
    engine.save_checkpoint(&file.0).expect("save");
    let text = std::fs::read_to_string(&file.0).expect("read");
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
    edit(value.as_object_mut().expect("object"));
    std::fs::write(&file.0, serde_json::to_string(&value).expect("json")).expect("write");
    SimulationEngine::load_checkpoint(&file.0)
}

#[test]
fn checkpoint_with_wrong_rng_state_length_is_rejected() {
    let result = load_edited("rng_len", |o| {
        o.insert("rng_state".into(), serde_json::json!([1, 2, 3]));
    });
    assert_eq!(result.err().map(|e| e.kind()), Some(std::io::ErrorKind::InvalidData));
}

#[test]
fn checkpoint_with_out_of_range_provider_is_rejected() {
    let result = load_edited("providers_range", |o| {
        o.insert("skill_providers".into(), serde_json::json!([["Skill0", [100000]]]));
    });
    assert_eq!(result.err().map(|e| e.kind()), Some(std::io::ErrorKind::InvalidData));
}

#[test]
fn checkpoint_with_duplicate_provider_skill_is_rejected() {
    let result = load_edited("providers_dup", |o| {
        o.insert("skill_providers".into(), serde_json::json!([["Skill0", [0]], ["Skill0", [1]]]));
    });
    assert_eq!(result.err().map(|e| e.kind()), Some(std::io::ErrorKind::InvalidData));
}

#[test]
fn provider_order_survives_a_checkpoint_round_trip() {
    let mut engine = SimulationEngine::new(rng_resume_config());
    for _ in 0..25 {
        engine.step();
    }
    let file = TempCheckpoint::new("provider_order");
    engine.save_checkpoint(&file.0).expect("save");
    let restored = SimulationEngine::load_checkpoint(&file.0).expect("load");
    for id in engine.get_market().skills.keys() {
        assert_eq!(restored.providers_of(id), engine.providers_of(id));
    }
}

#[test]
fn checkpoint_with_invalid_rng_word_position_is_rejected() {
    let result = load_edited("rng_nibble", |o| {
        let mut bytes = vec![0u8; 49];
        bytes[48] = 0xF0;
        o.insert("rng_state".into(), serde_json::json!(bytes));
    });
    assert_eq!(result.err().map(|e| e.kind()), Some(std::io::ErrorKind::InvalidData));
}
