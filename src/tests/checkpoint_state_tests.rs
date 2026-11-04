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

    // The step right after the resume reads the saved sales. The previous-step counters
    // must match. The current counters differ, because load reseeds the RNG.
    straight.step();
    restored.step();
    assert_eq!(restored.heuristic_sales_sorted().0, straight.heuristic_sales_sorted().0);
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
