//! Tests that the engine validates the configuration when it is created.

use crate::error::SimulationError;
use crate::tests::test_helpers::test_config;
use crate::{SimulationConfig, SimulationEngine};

fn invalid_config() -> SimulationConfig {
    let mut config = test_config().entity_count(10).max_steps(5).seed(1).build();
    config.enable_heuristic_agents = true;
    config.enable_nepotism = true;
    config
}

fn valid_config() -> SimulationConfig {
    test_config().entity_count(10).max_steps(5).seed(1).build()
}

#[test]
fn try_new_rejects_an_invalid_config() {
    let result = SimulationEngine::try_new(invalid_config());
    assert!(
        matches!(result, Err(SimulationError::ValidationError(_))),
        "an invalid config must give a ValidationError"
    );
}

#[test]
fn try_new_accepts_a_valid_config() {
    let engine = SimulationEngine::try_new(valid_config()).expect("valid config");
    assert_eq!(engine.get_entities().len(), 10);
}

#[test]
#[should_panic(expected = "invalid SimulationConfig")]
fn new_panics_on_an_invalid_config() {
    let _ = SimulationEngine::new(invalid_config());
}

#[test]
fn run_sweep_returns_an_error_for_an_invalid_base_config() {
    use crate::parameter_sweep::{ParameterRange, ParameterSweepResult};

    let range = ParameterRange::BasePrice { min: 5.0, max: 10.0, steps: 2 };
    let result = ParameterSweepResult::run_sweep(invalid_config(), range, 1, false);
    assert!(matches!(result, Err(SimulationError::ValidationError(_))));
}

#[test]
fn load_checkpoint_rejects_an_invalid_config() {
    let mut engine = SimulationEngine::new(valid_config());
    // One step makes the min and max money fields finite, so the file loads.
    engine.step();
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("checkpoint.json");
    engine.save_checkpoint(&path).expect("save");

    // Edit the text. A `serde_json::Value` round trip turns non-finite numbers into null.
    let text = std::fs::read_to_string(&path).expect("read");
    let edited = text
        .replacen("\"enable_heuristic_agents\": false", "\"enable_heuristic_agents\": true", 1)
        .replacen("\"enable_nepotism\": false", "\"enable_nepotism\": true", 1);
    assert_ne!(edited, text, "the checkpoint must contain both flags");
    std::fs::write(&path, edited).expect("write");

    let result = SimulationEngine::load_checkpoint(&path);
    let error = result.err().expect("an invalid config must fail");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("invalid config in checkpoint"), "message: {error}");
}
